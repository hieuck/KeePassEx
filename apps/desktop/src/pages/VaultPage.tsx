/**
 * Main vault page — entry list with group tree, bulk operations, keyboard shortcuts,
 * and entry preview pane (split view)
 */
import { useState, useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useVaultStore, type EntryDto } from '../store/vault';
import { useSettingsStore } from '../store/settings';
import { EntryRow } from '../components/EntryRow';
import { BulkActionBar } from '../components/BulkActionBar';
import { OtpDisplay } from '@keepassex/ui';

type SortField = 'title' | 'username' | 'modified' | 'created';
type SortDir = 'asc' | 'desc';
type FilterType = 'all' | 'favorites' | 'otp' | 'expiring' | 'noPassword';

export function VaultPage() {
  const { selectedGroupUuid, searchQuery } = useVaultStore();
  const { settings } = useSettingsStore();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { t } = useTranslation();

  const [selectedUuids, setSelectedUuids] = useState<Set<string>>(new Set());
  const [sortField, setSortField] = useState<SortField>('title');
  const [sortDir, setSortDir] = useState<SortDir>('asc');
  const [lastClickedUuid, setLastClickedUuid] = useState<string | null>(null);
  const [previewUuid, setPreviewUuid] = useState<string | null>(null);
  const [activeFilter, setActiveFilter] = useState<FilterType>('all');
  const [showPreview, setShowPreview] = useState(true);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [previewPassword, setPreviewPassword] = useState('');
  const [showPreviewPassword, setShowPreviewPassword] = useState(false);
  const [previewOtp, setPreviewOtp] = useState<{
    code: string;
    remainingSeconds: number;
    period: number;
  } | null>(null);

  // Fetch entries
  const { data: entries = [], isLoading } = useQuery({
    queryKey: ['entries', selectedGroupUuid, searchQuery],
    queryFn: async () => {
      if (searchQuery.trim()) {
        return invoke<EntryDto[]>('search_entries', { query: searchQuery });
      }
      return invoke<EntryDto[]>('get_entries', { groupUuid: selectedGroupUuid ?? null });
    },
  });

  // Fetch preview entry detail
  const { data: previewEntry } = useQuery({
    queryKey: ['entry', previewUuid],
    queryFn: () =>
      invoke<
        EntryDto & {
          notes: string;
          customFields: Array<{ key: string; value: string; protected: boolean }>;
        }
      >('get_entry', { uuid: previewUuid, includePassword: false }),
    enabled: !!previewUuid,
  });

  // OTP for preview
  useEffect(() => {
    if (!previewEntry?.hasOtp || !previewUuid) {
      setPreviewOtp(null);
      return;
    }
    const refresh = () => {
      invoke<{ code: string; remainingSeconds: number; period: number }>('generate_totp', {
        entryUuid: previewUuid,
      })
        .then(setPreviewOtp)
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [previewEntry?.hasOtp, previewUuid]);

  // Copy password mutation
  const copyPassword = useMutation({
    mutationFn: async (uuid: string) => {
      const password = await invoke<string>('get_entry_password', { uuid });
      await invoke('copy_to_clipboard', {
        text: password,
        clearAfterSeconds: settings.clipboardClearSeconds ?? 10,
      });
    },
  });

  const copyToClipboard = async (text: string, field: string) => {
    await invoke('copy_to_clipboard', {
      text,
      clearAfterSeconds: settings.clipboardClearSeconds ?? 10,
    });
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  const revealPreviewPassword = async () => {
    if (!previewUuid) return;
    const pw = await invoke<string>('get_entry_password', { uuid: previewUuid });
    setPreviewPassword(pw);
    setShowPreviewPassword(true);
  };

  // Apply filter
  const filteredEntries = entries.filter(e => {
    switch (activeFilter) {
      case 'favorites':
        return (e as EntryDto & { isFavorite?: boolean }).isFavorite;
      case 'otp':
        return e.hasOtp;
      case 'expiring':
        return e.isExpired || (e as EntryDto & { isExpiringSoon?: boolean }).isExpiringSoon;
      case 'noPassword':
        return !e.hasPassword;
      default:
        return true;
    }
  });

  // Sort entries
  const sortedEntries = [...filteredEntries].sort((a, b) => {
    let cmp = 0;
    switch (sortField) {
      case 'title':
        cmp = a.title.localeCompare(b.title);
        break;
      case 'username':
        cmp = a.username.localeCompare(b.username);
        break;
      case 'modified':
        cmp = a.modifiedAt.localeCompare(b.modifiedAt);
        break;
      case 'created':
        cmp = a.createdAt.localeCompare(b.createdAt);
        break;
    }
    return sortDir === 'asc' ? cmp : -cmp;
  });

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+A — select all
      if ((e.ctrlKey || e.metaKey) && e.key === 'a' && !e.shiftKey) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        e.preventDefault();
        setSelectedUuids(new Set(sortedEntries.map(e => e.uuid)));
      }
      // Escape — clear selection
      if (e.key === 'Escape') {
        setSelectedUuids(new Set());
      }
      // Delete — delete selected
      if (e.key === 'Delete' && selectedUuids.size > 0) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        // Handled by BulkActionBar
      }
      // N — new entry
      if (e.key === 'n' && !e.ctrlKey && !e.metaKey) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
        navigate('/vault/entry/new');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [sortedEntries, selectedUuids, navigate]);

  const handleEntryClick = useCallback(
    (uuid: string, e?: React.MouseEvent) => {
      if (e?.shiftKey && lastClickedUuid) {
        // Range selection
        const uuids = sortedEntries.map(e => e.uuid);
        const startIdx = uuids.indexOf(lastClickedUuid);
        const endIdx = uuids.indexOf(uuid);
        const [from, to] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];
        const rangeUuids = uuids.slice(from, to + 1);
        setSelectedUuids(prev => new Set([...prev, ...rangeUuids]));
      } else if (e?.ctrlKey || e?.metaKey) {
        // Toggle selection
        setSelectedUuids(prev => {
          const next = new Set(prev);
          if (next.has(uuid)) next.delete(uuid);
          else next.add(uuid);
          return next;
        });
      } else {
        // Navigate to entry
        navigate(`/vault/entry/${uuid}`);
      }
      setLastClickedUuid(uuid);
    },
    [sortedEntries, lastClickedUuid, navigate]
  );

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir(d => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortField(field);
      setSortDir('asc');
    }
  };

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return <span style={{ opacity: 0.3 }}>↕</span>;
    return <span>{sortDir === 'asc' ? '↑' : '↓'}</span>;
  };

  const FILTERS = [
    { id: 'all' as FilterType, label: t('vaultFilter.all') },
    { id: 'favorites' as FilterType, label: t('vaultFilter.favorites') },
    { id: 'otp' as FilterType, label: t('vaultFilter.withOtp') },
    { id: 'expiring' as FilterType, label: t('vaultFilter.expiring') },
    { id: 'noPassword' as FilterType, label: t('vaultFilter.noPassword') },
  ];

  return (
    <div className="vault-page">
      {/* Bulk action bar */}
      <BulkActionBar
        selectedUuids={Array.from(selectedUuids)}
        onClearSelection={() => setSelectedUuids(new Set())}
      />

      {/* Toolbar */}
      <div className="vault-toolbar">
        <h2 className="vault-toolbar-title">
          {searchQuery ? `${t('common.search')}: "${searchQuery}"` : t('group.allEntries')}
        </h2>
        <div className="vault-toolbar-actions">
          {/* Filter chips */}
          <div className="filter-chips" role="group" aria-label={t('entry.sortBy')}>
            {FILTERS.map(f => (
              <button
                key={f.id}
                className={`filter-chip${activeFilter === f.id ? ' active' : ''}`}
                onClick={() => setActiveFilter(f.id)}
                aria-pressed={activeFilter === f.id}
              >
                {f.label}
              </button>
            ))}
          </div>

          {/* Sort controls */}
          <div className="sort-controls" role="group" aria-label={t('entry.sortBy')}>
            <button
              className={`sort-btn ${sortField === 'title' ? 'active' : ''}`}
              onClick={() => handleSort('title')}
            >
              {t('entry.sortByTitle')} <SortIcon field="title" />
            </button>
            <button
              className={`sort-btn ${sortField === 'modified' ? 'active' : ''}`}
              onClick={() => handleSort('modified')}
            >
              {t('entry.sortByModified')} <SortIcon field="modified" />
            </button>
          </div>

          <span className="entry-count" aria-live="polite">
            {filteredEntries.length}
            {filteredEntries.length !== entries.length && `/${entries.length}`}{' '}
            {t('vault.statistics_entries', { count: 0 }).replace('0', '').trim()}
            {selectedUuids.size > 0 && ` · ${t('bulk.selected', { count: selectedUuids.size })}`}
          </span>

          {/* Preview toggle */}
          <button
            className={`icon-btn${showPreview ? ' active' : ''}`}
            onClick={() => setShowPreview(v => !v)}
            title={t('common.info')}
            aria-label={t('common.info')}
            aria-pressed={showPreview}
          >
            ⊞
          </button>

          <button
            className="btn btn-primary"
            onClick={() => navigate('/vault/entry/new')}
            aria-label={`${t('entry.new')} (N)`}
            title="N (new entry)"
          >
            + {t('entry.new')}
          </button>
        </div>
      </div>

      {/* Split view: list + preview */}
      <div
        className={`vault-split${showPreview && previewUuid ? ' vault-split--with-preview' : ''}`}
      >
        {/* Entry list */}
        <div
          className="entry-list"
          role="list"
          aria-label={t('entry.searchPlaceholder')}
          aria-multiselectable="true"
        >
          {isLoading ? (
            <div className="entry-list-empty">
              <span style={{ fontSize: 32 }}>⏳</span>
              <p>{t('common.loading')}</p>
            </div>
          ) : sortedEntries.length === 0 ? (
            <div className="entry-list-empty">
              <span style={{ fontSize: 48 }}>🔑</span>
              <p className="empty-state-title">
                {searchQuery ? t('entry.noEntries') : t('entry.noEntries')}
              </p>
              {!searchQuery && (
                <button className="btn btn-primary" onClick={() => navigate('/vault/entry/new')}>
                  {t('entry.new')}
                </button>
              )}
            </div>
          ) : (
            sortedEntries.map(entry => (
              <div
                key={entry.uuid}
                role="listitem"
                aria-selected={selectedUuids.has(entry.uuid) || previewUuid === entry.uuid}
                className={[
                  selectedUuids.has(entry.uuid) ? 'entry-selected' : '',
                  previewUuid === entry.uuid ? 'entry-previewing' : '',
                ]
                  .filter(Boolean)
                  .join(' ')}
              >
                <EntryRow
                  entry={entry}
                  onPress={(uuid, e) => {
                    const me = e as React.MouseEvent;
                    if (me?.shiftKey && lastClickedUuid) {
                      const uuids = sortedEntries.map(e => e.uuid);
                      const startIdx = uuids.indexOf(lastClickedUuid);
                      const endIdx = uuids.indexOf(uuid);
                      const [from, to] =
                        startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];
                      setSelectedUuids(prev => new Set([...prev, ...uuids.slice(from, to + 1)]));
                    } else if (me?.ctrlKey || me?.metaKey) {
                      setSelectedUuids(prev => {
                        const next = new Set(prev);
                        if (next.has(uuid)) next.delete(uuid);
                        else next.add(uuid);
                        return next;
                      });
                    } else if (me?.detail === 2) {
                      // Double click → open full page
                      navigate(`/vault/entry/${uuid}`);
                    } else {
                      // Single click → preview
                      setPreviewUuid(prev => (prev === uuid ? null : uuid));
                      setPreviewPassword('');
                      setShowPreviewPassword(false);
                    }
                    setLastClickedUuid(uuid);
                  }}
                  onCopyPassword={uuid => copyPassword.mutate(uuid)}
                />
              </div>
            ))
          )}
        </div>

        {/* Preview pane */}
        {showPreview && previewUuid && previewEntry && (
          <div className="preview-pane" role="complementary" aria-label={t('common.info')}>
            {/* Preview header */}
            <div className="preview-header">
              <h3 className="preview-title">{previewEntry.title}</h3>
              <div className="preview-header-actions">
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={() => navigate(`/vault/entry/${previewUuid}`)}
                  aria-label={t('common.ok')}
                >
                  {t('common.ok')}
                </button>
                <button
                  className="icon-btn"
                  onClick={() => setPreviewUuid(null)}
                  aria-label={t('common.close')}
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Preview fields */}
            <div className="preview-fields">
              {/* Username */}
              {previewEntry.username && (
                <PreviewField
                  label={t('entry.username')}
                  value={previewEntry.username}
                  onCopy={() => copyToClipboard(previewEntry.username, 'username')}
                  copied={copiedField === 'username'}
                />
              )}

              {/* Password */}
              <div className="preview-field">
                <span className="preview-field-label">{t('entry.password')}</span>
                <div className="preview-field-row">
                  <span className="preview-field-value" style={{ fontFamily: 'monospace' }}>
                    {showPreviewPassword ? previewPassword : '••••••••••••'}
                  </span>
                  <div className="preview-field-actions">
                    <button
                      className="icon-btn"
                      onClick={
                        showPreviewPassword
                          ? () => setShowPreviewPassword(false)
                          : revealPreviewPassword
                      }
                      aria-label={
                        showPreviewPassword ? t('entry.hidePassword') : t('entry.showPassword')
                      }
                    >
                      {showPreviewPassword ? '🙈' : '👁'}
                    </button>
                    <button
                      className={`icon-btn${copiedField === 'password' ? ' copied' : ''}`}
                      onClick={() =>
                        copyToClipboard(showPreviewPassword ? previewPassword : '', 'password')
                      }
                      aria-label={t('entry.copyPassword')}
                    >
                      {copiedField === 'password' ? '✓' : '⎘'}
                    </button>
                  </div>
                </div>
              </div>

              {/* URL */}
              {previewEntry.url && (
                <div className="preview-field">
                  <span className="preview-field-label">{t('entry.url')}</span>
                  <div className="preview-field-row">
                    <a
                      href={previewEntry.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="preview-url"
                    >
                      {previewEntry.url}
                    </a>
                    <button
                      className={`icon-btn${copiedField === 'url' ? ' copied' : ''}`}
                      onClick={() => copyToClipboard(previewEntry.url, 'url')}
                      aria-label={t('entry.copyUrl')}
                    >
                      {copiedField === 'url' ? '✓' : '⎘'}
                    </button>
                  </div>
                </div>
              )}

              {/* OTP */}
              {previewEntry.hasOtp && previewOtp && (
                <div className="preview-field">
                  <span className="preview-field-label">{t('otp.title')}</span>
                  <OtpDisplay
                    otp={previewOtp}
                    onCopy={code => copyToClipboard(code, 'otp')}
                    copied={copiedField === 'otp'}
                    compact
                    label={t('otp.title')}
                    expiresInLabel={t('otp.refreshIn', { seconds: '' }).split('{{')[0].trim()}
                    copyLabel={t('entry.copyOtp')}
                  />
                </div>
              )}

              {/* Notes */}
              {(previewEntry as typeof previewEntry & { notes?: string }).notes && (
                <div className="preview-field">
                  <span className="preview-field-label">{t('entry.notes')}</span>
                  <p className="preview-notes">
                    {(previewEntry as typeof previewEntry & { notes?: string }).notes}
                  </p>
                </div>
              )}

              {/* Tags */}
              {previewEntry.tags?.length > 0 && (
                <div className="preview-field">
                  <span className="preview-field-label">{t('entry.tags')}</span>
                  <div className="preview-tags">
                    {previewEntry.tags.map(tag => (
                      <span key={tag} className="preview-tag">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {/* Badges */}
              <div className="preview-badges">
                {previewEntry.hasOtp && (
                  <span className="preview-badge preview-badge--otp">{t('otp.title')}</span>
                )}
                {previewEntry.hasPasskey && (
                  <span className="preview-badge preview-badge--passkey">{t('passkey.title')}</span>
                )}
                {previewEntry.hasSshKey && (
                  <span className="preview-badge preview-badge--ssh">{t('ssh.title')}</span>
                )}
                {previewEntry.isExpired && (
                  <span className="preview-badge preview-badge--expired">{t('entry.expired')}</span>
                )}
              </div>

              {/* Metadata */}
              <div className="preview-meta">
                <span>
                  {t('entry.sortByModified')}:{' '}
                  {new Date(previewEntry.modifiedAt).toLocaleDateString()}
                </span>
                <span>
                  {t('entry.sortByCreated')}:{' '}
                  {new Date(previewEntry.createdAt).toLocaleDateString()}
                </span>
              </div>
            </div>
          </div>
        )}
      </div>

      <style>{`
        .vault-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .vault-toolbar {
          display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap;
          padding: var(--space-sm) var(--space-xl); gap: var(--space-sm);
          border-bottom: 1px solid var(--color-border); flex-shrink: 0;
        }
        .vault-toolbar-title { font-size: 15px; font-weight: 600; }
        .vault-toolbar-actions { display: flex; align-items: center; gap: var(--space-sm); flex-wrap: wrap; }
        .filter-chips { display: flex; gap: 2px; }
        .filter-chip {
          background: none; border: 1px solid var(--color-border); cursor: pointer;
          font-size: 11px; color: var(--color-text-secondary);
          padding: 2px var(--space-sm); border-radius: var(--radius-full);
          transition: all .1s;
        }
        .filter-chip:hover { border-color: var(--color-primary); color: var(--color-primary); }
        .filter-chip.active { background: var(--color-primary); border-color: var(--color-primary); color: white; }
        .sort-controls { display: flex; gap: 2px; }
        .sort-btn {
          background: none; border: none; cursor: pointer; font-size: 12px;
          color: var(--color-text-secondary); padding: var(--space-xs) var(--space-sm);
          border-radius: var(--radius-sm); transition: background .1s;
        }
        .sort-btn:hover { background: var(--color-bg-tertiary); color: var(--color-text); }
        .sort-btn.active { color: var(--color-primary); font-weight: 500; }
        .entry-count { font-size: 12px; color: var(--color-text-secondary); white-space: nowrap; }
        .icon-btn {
          background: none; border: none; cursor: pointer; font-size: 16px;
          color: var(--color-text-secondary); padding: 4px 6px; border-radius: var(--radius-sm);
          transition: background .1s, color .1s;
        }
        .icon-btn:hover { background: var(--color-bg-tertiary); color: var(--color-text); }
        .icon-btn.active { color: var(--color-primary); }
        .icon-btn.copied { color: var(--color-success, #16a34a); }
        .btn-sm { font-size: 12px; padding: 3px 10px; }
        .vault-split { display: flex; flex: 1; overflow: hidden; }
        .vault-split--with-preview .entry-list { flex: 0 0 55%; border-right: 1px solid var(--color-border); }
        .entry-list { flex: 1; overflow-y: auto; }
        .entry-list-empty {
          display: flex; flex-direction: column; align-items: center;
          justify-content: center; gap: var(--space-lg); height: 100%;
          color: var(--color-text-secondary); padding: var(--space-2xl);
        }
        .entry-selected { background: rgba(37, 99, 235, 0.06); }
        .entry-previewing { background: rgba(37, 99, 235, 0.1); border-left: 3px solid var(--color-primary); }

        /* Preview pane */
        .preview-pane {
          flex: 1; overflow-y: auto; display: flex; flex-direction: column;
          background: var(--color-bg);
        }
        .preview-header {
          display: flex; align-items: center; justify-content: space-between;
          padding: var(--space-md) var(--space-lg);
          border-bottom: 1px solid var(--color-border); flex-shrink: 0;
          background: var(--color-bg-secondary);
        }
        .preview-title { font-size: 15px; font-weight: 600; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .preview-header-actions { display: flex; gap: var(--space-xs); flex-shrink: 0; }
        .preview-fields {
          padding: var(--space-lg); display: flex; flex-direction: column; gap: var(--space-md);
        }
        .preview-field { display: flex; flex-direction: column; gap: 4px; }
        .preview-field-label {
          font-size: 11px; font-weight: 600; color: var(--color-text-secondary);
          text-transform: uppercase; letter-spacing: .05em;
        }
        .preview-field-row { display: flex; align-items: center; gap: var(--space-sm); }
        .preview-field-value { flex: 1; font-size: 14px; color: var(--color-text); word-break: break-all; }
        .preview-field-actions { display: flex; gap: 2px; flex-shrink: 0; }
        .preview-url { flex: 1; font-size: 13px; color: var(--color-primary); text-decoration: none; word-break: break-all; }
        .preview-url:hover { text-decoration: underline; }
        .preview-notes { font-size: 13px; color: var(--color-text); white-space: pre-wrap; line-height: 1.5; }
        .preview-tags { display: flex; flex-wrap: wrap; gap: 4px; }
        .preview-tag {
          background: var(--color-bg-tertiary); color: var(--color-text-secondary);
          padding: 2px 8px; border-radius: var(--radius-full); font-size: 11px;
        }
        .preview-badges { display: flex; flex-wrap: wrap; gap: 4px; }
        .preview-badge {
          font-size: 10px; font-weight: 700; padding: 2px 7px; border-radius: 4px;
        }
        .preview-badge--otp { background: #eff6ff; color: #2563eb; }
        .preview-badge--passkey { background: #f0fdf4; color: #16a34a; }
        .preview-badge--ssh { background: #f5f3ff; color: #7c3aed; }
        .preview-badge--expired { background: #fef2f2; color: #ef4444; }
        .preview-meta {
          display: flex; gap: var(--space-lg); font-size: 11px; color: var(--color-text-tertiary);
          padding-top: var(--space-md); border-top: 1px solid var(--color-border);
        }
      `}</style>
    </div>
  );
}

// ─── Preview Field ────────────────────────────────────────────────────────────

function PreviewField({
  label,
  value,
  onCopy,
  copied,
}: {
  label: string;
  value: string;
  onCopy: () => void;
  copied: boolean;
}) {
  return (
    <div className="preview-field">
      <span className="preview-field-label">{label}</span>
      <div className="preview-field-row">
        <span className="preview-field-value">{value}</span>
        <button
          className={`icon-btn${copied ? ' copied' : ''}`}
          onClick={onCopy}
          aria-label={`Copy ${label}`}
        >
          {copied ? '✓' : '⎘'}
        </button>
      </div>
    </div>
  );
}
