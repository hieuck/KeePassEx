/**
 * Entry detail / edit page — complete with OTP, custom fields, attachments,
 * history viewer, passkey section, SSH key section, password strength
 */
import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useSettingsStore } from '../store/settings';
import { OtpSetupDialog } from '../components/OtpSetupDialog';
import { CustomFieldEditor, type CustomField } from '../components/CustomFieldEditor';
import { AttachmentViewer, type Attachment } from '../components/AttachmentViewer';
import { TagInput } from '../components/TagInput';
import { OtpDisplay } from '@keepassex/ui';
import { PasswordField } from '@keepassex/ui';
import { EntryHistoryViewer } from '@keepassex/ui';
import type { EntryDto } from '../store/vault';

export function EntryDetailPage() {
  const { uuid } = useParams<{ uuid: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { settings } = useSettingsStore();
  const { t } = useTranslation();
  const isNew = uuid === 'new';

  const [editing, setEditing] = useState(isNew);
  const [showPassword, setShowPassword] = useState(false);
  const [password, setPassword] = useState('');
  const [showOtpSetup, setShowOtpSetup] = useState(false);
  const [form, setForm] = useState({
    title: '',
    username: '',
    password: '',
    url: '',
    notes: '',
    tags: [] as string[],
    iconId: 0,
    expiry: '',
    customFields: [] as CustomField[],
  });
  const [otpCode, setOtpCode] = useState<{
    code: string;
    remainingSeconds: number;
    period: number;
  } | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [tagInput, setTagInput] = useState('');
  const [activeTab, setActiveTab] = useState<'fields' | 'history' | 'passkeys' | 'ssh'>('fields');
  const [passwordStrength, setPasswordStrength] = useState<{
    score: 0 | 1 | 2 | 3 | 4;
    entropy: number;
    label: string;
  } | null>(null);

  // Fetch entry
  const { data: entry } = useQuery({
    queryKey: ['entry', uuid],
    queryFn: () => invoke<EntryDto>('get_entry', { uuid, includePassword: false }),
    enabled: !isNew && !!uuid,
  });

  // Fetch entry history
  const { data: history = [] } = useQuery({
    queryKey: ['entry-history', uuid],
    queryFn: () =>
      invoke<
        Array<{
          uuid: string;
          modifiedAt: string;
          title: string;
          username: string;
          url: string;
          notes: string;
          hasPassword: boolean;
        }>
      >('get_entry_history', { uuid }),
    enabled: !isNew && !!uuid,
  });

  // Populate form when entry loads
  useEffect(() => {
    if (entry) {
      setForm({
        title: entry.title,
        username: entry.username,
        password: '',
        url: entry.url,
        notes: entry.notes,
        tags: entry.tags,
        iconId: entry.iconId,
        expiry: entry.expiry ?? '',
        customFields: (entry.customFields ?? []).map(f => ({
          key: f.key,
          value: f.value,
          protected: f.protected,
        })),
      });
    }
  }, [entry]);

  // OTP refresh
  useEffect(() => {
    if (!entry?.hasOtp || !uuid) return;
    const refresh = () => {
      invoke<{ code: string; remainingSeconds: number; period: number }>('generate_totp', {
        entryUuid: uuid,
      })
        .then(setOtpCode)
        .catch(() => {});
    };
    refresh();
    const interval = setInterval(refresh, 1000);
    return () => clearInterval(interval);
  }, [entry?.hasOtp, uuid]);

  // Password strength check when editing
  useEffect(() => {
    if (!editing || !form.password) {
      setPasswordStrength(null);
      return;
    }
    invoke<{ score: 0 | 1 | 2 | 3 | 4; entropy: number; strengthLabel: string }>(
      'check_password_strength',
      {
        password: form.password,
      }
    )
      .then(r =>
        setPasswordStrength({ score: r.score, entropy: r.entropy, label: r.strengthLabel })
      )
      .catch(() => {});
  }, [form.password, editing]);

  // Save mutation
  const saveMutation = useMutation({
    mutationFn: async () => {
      if (isNew) {
        await invoke('create_entry', {
          args: {
            group_uuid: '00000000-0000-0000-0000-000000000000',
            title: form.title,
            username: form.username,
            password: form.password,
            url: form.url,
            notes: form.notes,
            tags: form.tags,
            icon_id: form.iconId,
          },
        });
      } else {
        await invoke('update_entry', {
          args: {
            uuid,
            title: form.title,
            username: form.username,
            password: form.password,
            url: form.url,
            notes: form.notes,
            tags: form.tags,
            icon_id: form.iconId,
            expiry: form.expiry || null,
            custom_fields: form.customFields,
          },
        });
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
      setEditing(false);
      if (isNew) navigate('/vault');
    },
  });

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: () => invoke('delete_entry', { uuid, permanent: false }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      navigate('/vault');
    },
  });

  const copyToClipboard = async (text: string, field: string) => {
    await invoke('copy_to_clipboard', {
      text,
      clearAfterSeconds: settings.clipboardClearSeconds ?? 10,
    });
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const revealPassword = async () => {
    if (!uuid) return;
    const pw = await invoke<string>('get_entry_password', { uuid });
    setPassword(pw);
    setShowPassword(true);
  };

  const handleOtpSave = async (uri: string) => {
    if (uuid && uuid !== 'new') {
      try {
        await invoke('set_entry_otp', { entryUuid: uuid, uri });
        queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
      } catch (e) {
        console.error('Failed to save OTP:', e);
      }
    }
    setShowOtpSetup(false);
  };

  const addTag = () => {
    const tag = tagInput.trim();
    if (tag && !form.tags.includes(tag)) {
      setForm(f => ({ ...f, tags: [...f.tags, tag] }));
    }
    setTagInput('');
  };

  const removeTag = (tag: string) => {
    setForm(f => ({ ...f, tags: f.tags.filter(t => t !== tag) }));
  };

  if (!isNew && !entry) {
    return (
      <div className="entry-detail-loading">
        <p>{t('common.loading')}</p>
      </div>
    );
  }

  return (
    <div className="entry-detail">
      {/* OTP Setup Dialog */}
      {showOtpSetup && (
        <OtpSetupDialog
          entryUuid={uuid ?? ''}
          onSave={handleOtpSave}
          onClose={() => setShowOtpSetup(false)}
        />
      )}

      {/* Header */}
      <div className="entry-detail-header">
        <button className="btn-icon" onClick={() => navigate(-1)} aria-label={t('common.back')}>
          ←
        </button>
        <h2 className="entry-detail-title">{isNew ? t('entry.new') : (entry?.title ?? '')}</h2>
        <div className="entry-detail-actions">
          {!isNew && !editing && (
            <>
              <button className="btn btn-secondary" onClick={() => setEditing(true)}>
                {t('common.edit')}
              </button>
              <button
                className="btn btn-secondary"
                title={t('entry.autoType')}
                onClick={() => invoke('auto_type_entry', { entryUuid: uuid })}
              >
                ⌨️ {t('entry.autoType')}
              </button>
              <button
                className="btn btn-danger"
                onClick={() => {
                  if (confirm(t('entry.confirmDelete', { title: entry?.title }))) {
                    deleteMutation.mutate();
                  }
                }}
              >
                {t('common.delete')}
              </button>
            </>
          )}
          {editing && (
            <>
              <button
                className="btn btn-primary"
                onClick={() => saveMutation.mutate()}
                disabled={saveMutation.isPending || !form.title.trim()}
              >
                {saveMutation.isPending ? '...' : t('common.save')}
              </button>
              {!isNew && (
                <button className="btn btn-secondary" onClick={() => setEditing(false)}>
                  {t('common.cancel')}
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {/* Tabs */}
      {!isNew && (
        <div className="entry-tabs" role="tablist">
          {[
            { id: 'fields', label: t('common.info') ?? 'Details' },
            { id: 'history', label: t('entry.history'), count: history.length },
            { id: 'passkeys', label: t('passkey.title'), show: entry?.hasPasskey },
            { id: 'ssh', label: t('ssh.title'), show: entry?.hasSshKey },
          ]
            .filter(t => t.show !== false)
            .map(tab => (
              <button
                key={tab.id}
                role="tab"
                aria-selected={activeTab === tab.id}
                className={`entry-tab${activeTab === tab.id ? ' entry-tab--active' : ''}`}
                onClick={() => setActiveTab(tab.id as typeof activeTab)}
              >
                {tab.label}
                {tab.count !== undefined && tab.count > 0 && (
                  <span className="entry-tab-badge">{tab.count}</span>
                )}
              </button>
            ))}
        </div>
      )}

      {/* Content */}
      <div className="entry-detail-content">
        {/* ── Details tab ── */}
        {(isNew || activeTab === 'fields') && (
          <>
            {/* Title */}
            <FieldRow
              label={t('entry.title')}
              editing={editing}
              value={entry?.title}
              inputValue={form.title}
              onInputChange={v => setForm(f => ({ ...f, title: v }))}
              placeholder={t('entry.title')}
            />

            {/* Username */}
            <FieldRow
              label={t('entry.username')}
              editing={editing}
              value={entry?.username}
              inputValue={form.username}
              onInputChange={v => setForm(f => ({ ...f, username: v }))}
              placeholder={t('entry.username')}
              onCopy={() => copyToClipboard(entry?.username ?? form.username, 'username')}
              copied={copied === 'username'}
            />

            {/* Password — use PasswordField component */}
            <div className="field-row">
              <span className="field-label">{t('entry.password')}</span>
              <PasswordField
                value={editing ? form.password : showPassword ? password : ''}
                onChange={v => setForm(f => ({ ...f, password: v }))}
                onCopy={() => copyToClipboard(showPassword ? password : '', 'password')}
                onGenerate={
                  editing
                    ? () =>
                        invoke<string>('generate_password', {
                          config: {
                            mode: 'random',
                            length: 20,
                            useUppercase: true,
                            useLowercase: true,
                            useDigits: true,
                            useSymbols: true,
                            excludeAmbiguous: false,
                            excludeChars: '',
                            minUppercase: 0,
                            minLowercase: 0,
                            minDigits: 0,
                            minSymbols: 0,
                            wordCount: 6,
                            wordSeparator: '-',
                            capitalizeWords: false,
                            includeNumber: false,
                          },
                        }).then(pw => setForm(f => ({ ...f, password: pw })))
                    : undefined
                }
                readOnly={!editing}
                copied={copied === 'password'}
                strengthScore={passwordStrength?.score}
                strengthLabel={passwordStrength?.label}
                entropy={passwordStrength?.entropy}
                placeholder={t('entry.password')}
                showLabel={t('entry.showPassword')}
                hideLabel={t('entry.hidePassword')}
                copyLabel={t('entry.copyPassword')}
                generateLabel={t('entry.generatePassword')}
              />
            </div>

            {/* URL */}
            <FieldRow
              label={t('entry.url')}
              editing={editing}
              value={entry?.url}
              inputValue={form.url}
              onInputChange={v => setForm(f => ({ ...f, url: v }))}
              placeholder="https://"
              onCopy={() => copyToClipboard(entry?.url ?? form.url, 'url')}
              copied={copied === 'url'}
              isUrl
            />

            {/* OTP — use OtpDisplay component */}
            <div className="field-row">
              <div className="field-label-row">
                <span className="field-label">{t('otp.title')}</span>
                {editing && (
                  <button className="btn-link" onClick={() => setShowOtpSetup(true)}>
                    {entry?.hasOtp ? t('common.edit') : `+ ${t('entry.addOtp')}`}
                  </button>
                )}
              </div>
              {entry?.hasOtp && otpCode ? (
                <OtpDisplay
                  otp={otpCode}
                  onCopy={code => copyToClipboard(code, 'otp')}
                  copied={copied === 'otp'}
                  label={t('otp.title')}
                  expiresInLabel={t('otp.refreshIn', { seconds: '' }).split('{{')[0].trim()}
                  copyLabel={t('entry.copyOtp')}
                />
              ) : !editing ? (
                <span className="field-value" style={{ color: 'var(--color-text-tertiary)' }}>
                  {t('common.none')}
                </span>
              ) : null}
            </div>

            {/* Notes */}
            <div className="field-row">
              <span className="field-label">{t('entry.notes')}</span>
              {editing ? (
                <textarea
                  className="form-input form-textarea"
                  value={form.notes}
                  onChange={e => setForm(f => ({ ...f, notes: e.target.value }))}
                  placeholder={t('entry.notes')}
                  rows={4}
                />
              ) : (
                <span className="field-value field-value-notes">{entry?.notes || '—'}</span>
              )}
            </div>

            {/* Tags */}
            <div className="field-row">
              <span className="field-label">{t('entry.tags')}</span>
              {editing ? (
                <TagInput
                  tags={form.tags}
                  onChange={tags => setForm(f => ({ ...f, tags }))}
                  placeholder={t('entry.tags')}
                />
              ) : (
                <div className="tags-list">
                  {(entry?.tags ?? []).map(tag => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                  {(entry?.tags ?? []).length === 0 && (
                    <span className="field-value" style={{ color: 'var(--color-text-tertiary)' }}>
                      —
                    </span>
                  )}
                </div>
              )}
            </div>

            {/* Expiry */}
            {(editing || entry?.expiry) && (
              <div className="field-row">
                <span className="field-label">{t('entry.expiry')}</span>
                {editing ? (
                  <input
                    type="date"
                    className="form-input"
                    value={form.expiry ? form.expiry.slice(0, 10) : ''}
                    onChange={e =>
                      setForm(f => ({
                        ...f,
                        expiry: e.target.value ? new Date(e.target.value).toISOString() : '',
                      }))
                    }
                  />
                ) : (
                  <span className={`field-value ${entry?.isExpired ? 'text-danger' : ''}`}>
                    {entry?.expiry ? new Date(entry.expiry).toLocaleDateString() : '—'}
                    {entry?.isExpired && ` (${t('entry.expired')})`}
                  </span>
                )}
              </div>
            )}

            {/* Custom Fields */}
            <div className="field-row">
              <span className="field-label">{t('entry.customFields')}</span>
              <CustomFieldEditor
                fields={
                  editing
                    ? form.customFields
                    : (entry?.customFields ?? []).map(f => ({
                        key: f.key,
                        value: f.value,
                        protected: f.protected,
                      }))
                }
                onChange={fields => setForm(f => ({ ...f, customFields: fields }))}
                readOnly={!editing}
              />
            </div>

            {/* Attachments */}
            <div className="field-row">
              <span className="field-label">{t('entry.attachments')}</span>
              <AttachmentViewer
                attachments={(entry?.customFields ?? [])
                  .filter(f => f.key.startsWith('_attachment_'))
                  .map(f => ({ name: f.key.replace('_attachment_', ''), dataRef: f.value }))}
                entryUuid={uuid ?? ''}
                readOnly={!editing}
              />
            </div>

            {/* Metadata */}
            {!editing && entry && (
              <div className="entry-meta">
                <span>
                  {t('entry.sortByCreated')}: {new Date(entry.createdAt).toLocaleDateString()}
                </span>
                <span>
                  {t('entry.sortByModified')}: {new Date(entry.modifiedAt).toLocaleDateString()}
                </span>
              </div>
            )}
          </>
        )}

        {/* ── History tab ── */}
        {activeTab === 'history' && !isNew && (
          <EntryHistoryViewer
            history={history}
            onRestore={async h => {
              if (confirm(t('entry.restoreFromHistory'))) {
                await invoke('restore_entry_from_history', {
                  entryUuid: uuid,
                  historyUuid: h.uuid,
                });
                queryClient.invalidateQueries({ queryKey: ['entry', uuid] });
                queryClient.invalidateQueries({ queryKey: ['entry-history', uuid] });
                setActiveTab('fields');
              }
            }}
            onClearHistory={async () => {
              await invoke('clear_entry_history', { uuid });
              queryClient.invalidateQueries({ queryKey: ['entry-history', uuid] });
            }}
            historyLabel={t('entry.history')}
            noHistoryLabel={t('common.none')}
            restoreLabel={t('entry.restoreFromHistory')}
            clearLabel={t('entry.clearHistory')}
            confirmClearLabel={t('entry.clearHistory')}
            modifiedLabel={t('entry.sortByModified')}
            changedFieldsLabel={t('common.info') ?? 'Changed'}
          />
        )}

        {/* ── Passkeys tab ── */}
        {activeTab === 'passkeys' && !isNew && <PasskeysTab entryUuid={uuid ?? ''} />}

        {/* ── SSH tab ── */}
        {activeTab === 'ssh' && !isNew && <SshKeyTab entryUuid={uuid ?? ''} />}
      </div>

      <style>{`
        .entry-detail { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .entry-detail-header {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .entry-detail-title { flex:1; font-size:16px; font-weight:600; }
        .entry-detail-actions { display:flex; gap:var(--space-sm); }
        .entry-tabs {
          display:flex; gap:2px; padding:0 var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .entry-tab {
          background:none; border:none; cursor:pointer;
          padding:var(--space-sm) var(--space-md);
          font-size:13px; color:var(--color-text-secondary);
          border-bottom:2px solid transparent; margin-bottom:-1px;
          display:flex; align-items:center; gap:6px;
          transition:color 0.1s, border-color 0.1s;
        }
        .entry-tab:hover { color:var(--color-text); }
        .entry-tab--active { color:var(--color-primary); border-bottom-color:var(--color-primary); font-weight:500; }
        .entry-tab-badge {
          background:var(--color-bg-tertiary); color:var(--color-text-secondary);
          font-size:10px; padding:1px 5px; border-radius:10px;
        }
        .entry-detail-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-md); max-width:640px;
        }
        .field-row { display:flex; flex-direction:column; gap:var(--space-xs); }
        .field-label-row { display:flex; align-items:center; justify-content:space-between; }
        .field-label {
          font-size:12px; font-weight:500; color:var(--color-text-secondary);
          text-transform:uppercase; letter-spacing:.05em;
        }
        .field-value-row { display:flex; align-items:center; gap:var(--space-sm); }
        .field-value { flex:1; font-size:15px; color:var(--color-text); word-break:break-all; }
        .field-value-notes { font-family:inherit; white-space:pre-wrap; line-height:1.6; }
        .field-actions { display:flex; gap:4px; flex-shrink:0; }
        .otp-code {
          font-size:22px; font-weight:700; font-family:'SF Mono','Consolas',monospace;
          letter-spacing:4px; flex:1;
        }
        .otp-timer { font-size:13px; color:var(--color-text-secondary); min-width:28px; }
        .form-textarea { resize:vertical; min-height:80px; font-family:inherit; line-height:1.5; }
        .tags-container { display:flex; flex-direction:column; gap:var(--space-sm); }
        .tags-list { display:flex; flex-wrap:wrap; gap:var(--space-xs); }
        .tag {
          display:flex; align-items:center; gap:4px;
          background:var(--color-bg-tertiary); color:var(--color-text-secondary);
          padding:2px 8px; border-radius:var(--radius-full); font-size:12px;
        }
        .tag-remove {
          background:none; border:none; cursor:pointer; color:var(--color-text-tertiary);
          font-size:10px; padding:0; line-height:1;
        }
        .tag-remove:hover { color:var(--color-danger); }
        .tag-input-row { display:flex; gap:var(--space-sm); }
        .tag-input { flex:1; }
        .entry-meta {
          display:flex; gap:var(--space-xl); font-size:12px; color:var(--color-text-tertiary);
          padding-top:var(--space-md); border-top:1px solid var(--color-border);
        }
        .btn-link {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:12px; padding:0; text-decoration:underline;
        }
        .btn-icon.copied { color:var(--color-success); }
        .entry-detail-loading {
          display:flex; align-items:center; justify-content:center;
          height:100%; color:var(--color-text-secondary);
        }
        .btn-danger {
          background:var(--color-danger); color:white; border:none;
          padding:var(--space-sm) var(--space-md); border-radius:var(--radius-md);
          cursor:pointer; font-size:13px;
        }
        .text-danger { color:var(--color-danger); }
        .field-link { color:var(--color-primary); text-decoration:none; font-size:15px; }
        .field-link:hover { text-decoration:underline; }
        .section-desc { font-size:13px; color:var(--color-text-secondary); margin-bottom:var(--space-md); }
        .passkey-placeholder, .ssh-placeholder {
          display:flex; flex-direction:column; align-items:center; gap:var(--space-md);
          padding:var(--space-2xl); color:var(--color-text-tertiary); font-size:13px;
          background:var(--color-bg-secondary); border-radius:var(--radius-md);
          border:1px dashed var(--color-border);
        }
        .passkey-placeholder span, .ssh-placeholder span { font-size:32px; }
      `}</style>
    </div>
  );
}

// ─── Field Row Component ──────────────────────────────────────────────────────

interface FieldRowProps {
  label: string;
  value?: string;
  editing: boolean;
  inputValue: string;
  onInputChange: (v: string) => void;
  placeholder?: string;
  onCopy?: () => void;
  copied?: boolean;
  isUrl?: boolean;
}

function FieldRow({
  label,
  value,
  editing,
  inputValue,
  onInputChange,
  placeholder,
  onCopy,
  copied,
  isUrl,
}: FieldRowProps) {
  return (
    <div className="field-row">
      <span className="field-label">{label}</span>
      <div className="field-value-row">
        {editing ? (
          <input
            type="text"
            className="form-input"
            value={inputValue}
            onChange={e => onInputChange(e.target.value)}
            placeholder={placeholder}
            style={{ flex: 1 }}
          />
        ) : (
          <>
            {isUrl && value ? (
              <a
                href={value}
                target="_blank"
                rel="noopener noreferrer"
                className="field-link"
                style={{ flex: 1 }}
              >
                {value || '—'}
              </a>
            ) : (
              <span className="field-value" style={{ flex: 1 }}>
                {value || '—'}
              </span>
            )}
            {onCopy && (
              <button
                className={`btn-icon ${copied ? 'copied' : ''}`}
                onClick={onCopy}
                aria-label={`Copy ${label}`}
              >
                {copied ? '✓' : '⎘'}
              </button>
            )}
          </>
        )}
      </div>
    </div>
  );
}

// ─── Passkeys Tab ─────────────────────────────────────────────────────────────

interface PasskeyDto {
  index: number;
  credentialId: string;
  rpId: string;
  rpName: string;
  userName: string;
  userDisplayName: string;
  signCount: number;
  createdAt: string;
  lastUsedAt?: string;
  backupEligible: boolean;
  backupState: boolean;
}

function PasskeysTab({ entryUuid }: { entryUuid: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [showAddForm, setShowAddForm] = useState(false);
  const [addForm, setAddForm] = useState({
    rpId: '',
    rpName: '',
    userName: '',
    userDisplayName: '',
    credentialId: '',
    userId: '',
    privateKeyPem: '',
    backupEligible: false,
  });

  const { data: passkeys = [], isLoading } = useQuery({
    queryKey: ['passkeys', entryUuid],
    queryFn: () => invoke<PasskeyDto[]>('get_entry_passkeys', { entryUuid }),
    enabled: !!entryUuid,
  });

  const addMutation = useMutation({
    mutationFn: () =>
      invoke('add_entry_passkey', {
        args: {
          entry_uuid: entryUuid,
          credential_id:
            addForm.credentialId ||
            Array.from(crypto.getRandomValues(new Uint8Array(16)))
              .map(b => b.toString(16).padStart(2, '0'))
              .join(''),
          rp_id: addForm.rpId,
          rp_name: addForm.rpName,
          user_id:
            addForm.userId ||
            Array.from(crypto.getRandomValues(new Uint8Array(16)))
              .map(b => b.toString(16).padStart(2, '0'))
              .join(''),
          user_name: addForm.userName,
          user_display_name: addForm.userDisplayName,
          private_key_pem: addForm.privateKeyPem,
          backup_eligible: addForm.backupEligible,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['passkeys', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
      setShowAddForm(false);
      setAddForm({
        rpId: '',
        rpName: '',
        userName: '',
        userDisplayName: '',
        credentialId: '',
        userId: '',
        privateKeyPem: '',
        backupEligible: false,
      });
    },
  });

  const removeMutation = useMutation({
    mutationFn: (index: number) =>
      invoke('remove_entry_passkey', { entryUuid, passkeyIndex: index }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['passkeys', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
    },
  });

  return (
    <div className="passkeys-section">
      <div className="section-header">
        <h3 className="section-title">🔑 {t('passkey.title')}</h3>
        <button className="btn btn-primary btn-sm" onClick={() => setShowAddForm(v => !v)}>
          {showAddForm ? t('common.cancel') : `+ ${t('passkey.add')}`}
        </button>
      </div>

      <p className="section-desc" style={{ marginBottom: 'var(--space-md)' }}>
        {t('passkey.description')}
      </p>

      {/* Add form */}
      {showAddForm && (
        <div className="passkey-add-form">
          <div className="form-row-2">
            <div className="field-row">
              <label className="field-label">{t('passkey.rpId')}</label>
              <input
                className="form-input"
                placeholder="example.com"
                value={addForm.rpId}
                onChange={e => setAddForm(f => ({ ...f, rpId: e.target.value }))}
              />
            </div>
            <div className="field-row">
              <label className="field-label">{t('passkey.rpName')}</label>
              <input
                className="form-input"
                placeholder="Example"
                value={addForm.rpName}
                onChange={e => setAddForm(f => ({ ...f, rpName: e.target.value }))}
              />
            </div>
          </div>
          <div className="form-row-2">
            <div className="field-row">
              <label className="field-label">{t('entry.username')}</label>
              <input
                className="form-input"
                placeholder="user@example.com"
                value={addForm.userName}
                onChange={e => setAddForm(f => ({ ...f, userName: e.target.value }))}
              />
            </div>
            <div className="field-row">
              <label className="field-label">{t('passkey.displayName')}</label>
              <input
                className="form-input"
                placeholder="User Name"
                value={addForm.userDisplayName}
                onChange={e => setAddForm(f => ({ ...f, userDisplayName: e.target.value }))}
              />
            </div>
          </div>
          <div className="field-row">
            <label className="field-label">{t('passkey.privateKey')}</label>
            <textarea
              className="form-input form-textarea"
              placeholder="-----BEGIN PRIVATE KEY-----&#10;...&#10;-----END PRIVATE KEY-----"
              value={addForm.privateKeyPem}
              onChange={e => setAddForm(f => ({ ...f, privateKeyPem: e.target.value }))}
              rows={4}
            />
          </div>
          <label className="check-option">
            <input
              type="checkbox"
              checked={addForm.backupEligible}
              onChange={e => setAddForm(f => ({ ...f, backupEligible: e.target.checked }))}
            />
            <span>{t('passkey.backupEligible')}</span>
          </label>
          <button
            className="btn btn-primary"
            onClick={() => addMutation.mutate()}
            disabled={addMutation.isPending || !addForm.rpId || !addForm.userName}
          >
            {addMutation.isPending ? '...' : t('passkey.save')}
          </button>
        </div>
      )}

      {/* Passkey list */}
      {isLoading ? (
        <p style={{ color: 'var(--color-text-tertiary)', fontSize: 13 }}>{t('common.loading')}</p>
      ) : passkeys.length === 0 ? (
        <div className="empty-section">
          <span style={{ fontSize: 32 }}>🔑</span>
          <p>{t('passkey.noPasskeys')}</p>
          <p style={{ fontSize: 12 }}>{t('passkey.noPasskeysHint')}</p>
        </div>
      ) : (
        <div className="passkey-list">
          {passkeys.map(pk => (
            <div key={pk.index} className="passkey-item">
              <div className="passkey-item-icon">🔑</div>
              <div className="passkey-item-info">
                <div className="passkey-item-rp">
                  <strong>{pk.rpName || pk.rpId}</strong>
                  <span className="passkey-item-domain">{pk.rpId}</span>
                </div>
                <div className="passkey-item-user">{pk.userDisplayName || pk.userName}</div>
                <div className="passkey-item-meta">
                  {t('entry.sortByCreated')}: {new Date(pk.createdAt).toLocaleDateString()}
                  {pk.lastUsedAt &&
                    ` · ${t('passkey.lastUsed')}: ${new Date(pk.lastUsedAt).toLocaleDateString()}`}
                  {` · ${t('passkey.signCount')}: ${pk.signCount}`}
                </div>
              </div>
              <div className="passkey-item-badges">
                {pk.backupEligible && (
                  <span className="badge badge-green">{t('passkey.backupEligible')}</span>
                )}
                {pk.backupState && (
                  <span className="badge badge-blue">{t('passkey.backedUp')}</span>
                )}
              </div>
              <button
                className="btn-icon"
                onClick={() => {
                  if (confirm(t('passkey.confirmRemove'))) removeMutation.mutate(pk.index);
                }}
                aria-label={t('common.delete')}
                title={t('common.delete')}
              >
                🗑
              </button>
            </div>
          ))}
        </div>
      )}

      <style>{`
        .section-header { display:flex; align-items:center; justify-content:space-between; margin-bottom:var(--space-sm); }
        .section-title { font-size:14px; font-weight:600; }
        .passkey-add-form { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-md); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); margin-bottom:var(--space-lg); }
        .form-row-2 { display:grid; grid-template-columns:1fr 1fr; gap:var(--space-md); }
        .passkey-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .passkey-item { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-md); background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-md); }
        .passkey-item-icon { font-size:24px; flex-shrink:0; }
        .passkey-item-info { flex:1; min-width:0; }
        .passkey-item-rp { display:flex; align-items:center; gap:var(--space-sm); }
        .passkey-item-domain { font-size:12px; color:var(--color-text-tertiary); }
        .passkey-item-user { font-size:13px; color:var(--color-text-secondary); }
        .passkey-item-meta { font-size:11px; color:var(--color-text-tertiary); margin-top:2px; }
        .passkey-item-badges { display:flex; gap:4px; flex-shrink:0; }
        .badge { font-size:10px; font-weight:600; padding:2px 6px; border-radius:var(--radius-full); }
        .badge-green { background:#f0fdf4; color:#16a34a; }
        .badge-blue { background:#eff6ff; color:#2563eb; }
        .empty-section { display:flex; flex-direction:column; align-items:center; gap:var(--space-md); padding:var(--space-2xl); color:var(--color-text-tertiary); font-size:13px; background:var(--color-bg-secondary); border-radius:var(--radius-md); border:1px dashed var(--color-border); text-align:center; }
        .check-option { display:flex; align-items:center; gap:var(--space-sm); font-size:13px; cursor:pointer; }
      `}</style>
    </div>
  );
}

// ─── SSH Key Tab ──────────────────────────────────────────────────────────────

interface SshKeyDto {
  keyType: string;
  publicKey: string;
  comment: string;
  fingerprint: string;
  addToAgent: boolean;
  agentDuration?: number;
  confirmBeforeUse: boolean;
}

function SshKeyTab({ entryUuid }: { entryUuid: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [privateKey, setPrivateKey] = useState('');
  const [copied, setCopied] = useState(false);
  const [form, setForm] = useState({
    privateKey: '',
    publicKey: '',
    comment: '',
    keyType: 'ed25519',
    addToAgent: true,
    agentDuration: undefined as number | undefined,
    confirmBeforeUse: false,
  });

  const { data: sshKey, isLoading } = useQuery({
    queryKey: ['ssh-key', entryUuid],
    queryFn: () => invoke<SshKeyDto | null>('get_entry_ssh_key', { entryUuid }),
    enabled: !!entryUuid,
  });

  const saveMutation = useMutation({
    mutationFn: () =>
      invoke('set_entry_ssh_key', {
        args: {
          entry_uuid: entryUuid,
          private_key: form.privateKey,
          public_key: form.publicKey,
          comment: form.comment,
          key_type: form.keyType,
          add_to_agent: form.addToAgent,
          agent_duration: form.agentDuration ?? null,
          confirm_before_use: form.confirmBeforeUse,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ssh-key', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
      setEditing(false);
    },
  });

  const removeMutation = useMutation({
    mutationFn: () => invoke('remove_entry_ssh_key', { entryUuid }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ssh-key', entryUuid] });
      queryClient.invalidateQueries({ queryKey: ['entry', entryUuid] });
    },
  });

  const loadToAgent = useMutation({
    mutationFn: () => invoke('load_ssh_key_to_agent', { entryUuid }),
  });

  const revealPrivateKey = async () => {
    const pk = await invoke<string>('get_entry_ssh_private_key', { entryUuid });
    setPrivateKey(pk);
    setShowPrivateKey(true);
  };

  const copyPublicKey = async () => {
    if (sshKey?.publicKey) {
      await invoke('copy_to_clipboard', { text: sshKey.publicKey, clearAfterSeconds: 30 });
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (isLoading)
    return (
      <p style={{ color: 'var(--color-text-tertiary)', fontSize: 13 }}>{t('common.loading')}</p>
    );

  return (
    <div className="ssh-section">
      <div className="section-header">
        <h3 className="section-title">🔐 {t('ssh.title')}</h3>
        <div style={{ display: 'flex', gap: 'var(--space-sm)' }}>
          {sshKey && !editing && (
            <>
              <button
                className="btn btn-secondary btn-sm"
                onClick={() => loadToAgent.mutate()}
                disabled={loadToAgent.isPending}
              >
                {loadToAgent.isPending ? '...' : `⚡ ${t('ssh.addToAgent')}`}
              </button>
              <button
                className="btn btn-secondary btn-sm"
                onClick={() => {
                  setForm({
                    privateKey: '',
                    publicKey: sshKey.publicKey,
                    comment: sshKey.comment,
                    keyType: sshKey.keyType,
                    addToAgent: sshKey.addToAgent,
                    agentDuration: sshKey.agentDuration,
                    confirmBeforeUse: sshKey.confirmBeforeUse,
                  });
                  setEditing(true);
                }}
              >
                {t('common.edit')}
              </button>
              <button
                className="btn btn-danger btn-sm"
                onClick={() => {
                  if (confirm(t('ssh.confirmRemove'))) removeMutation.mutate();
                }}
              >
                {t('common.delete')}
              </button>
            </>
          )}
          {!sshKey && !editing && (
            <button className="btn btn-primary btn-sm" onClick={() => setEditing(true)}>
              + {t('ssh.addKey')}
            </button>
          )}
        </div>
      </div>

      {/* Edit form */}
      {editing && (
        <div className="ssh-edit-form">
          <div className="field-row">
            <label className="field-label">{t('ssh.keyType')}</label>
            <select
              className="form-input"
              value={form.keyType}
              onChange={e => setForm(f => ({ ...f, keyType: e.target.value }))}
            >
              <option value="ed25519">Ed25519 (recommended)</option>
              <option value="rsa4096">RSA 4096-bit</option>
              <option value="ecdsa-p256">ECDSA P-256</option>
              <option value="ecdsa-p384">ECDSA P-384</option>
            </select>
          </div>
          <div className="field-row">
            <label className="field-label">{t('ssh.privateKey')}</label>
            <textarea
              className="form-input form-textarea"
              placeholder="-----BEGIN OPENSSH PRIVATE KEY-----&#10;...&#10;-----END OPENSSH PRIVATE KEY-----"
              value={form.privateKey}
              onChange={e => setForm(f => ({ ...f, privateKey: e.target.value }))}
              rows={6}
            />
          </div>
          <div className="field-row">
            <label className="field-label">{t('ssh.publicKey')}</label>
            <textarea
              className="form-input form-textarea"
              placeholder="ssh-ed25519 AAAA... comment"
              value={form.publicKey}
              onChange={e => setForm(f => ({ ...f, publicKey: e.target.value }))}
              rows={3}
            />
          </div>
          <div className="field-row">
            <label className="field-label">{t('ssh.comment')}</label>
            <input
              className="form-input"
              placeholder="user@hostname"
              value={form.comment}
              onChange={e => setForm(f => ({ ...f, comment: e.target.value }))}
            />
          </div>
          <label className="check-option">
            <input
              type="checkbox"
              checked={form.addToAgent}
              onChange={e => setForm(f => ({ ...f, addToAgent: e.target.checked }))}
            />
            <span>{t('ssh.addToAgent')}</span>
          </label>
          {form.addToAgent && (
            <div className="field-row">
              <label className="field-label">
                {t('ssh.agentDuration')} ({t('common.seconds')}, {t('ssh.emptyForever')})
              </label>
              <input
                className="form-input"
                type="number"
                min={0}
                placeholder="∞"
                value={form.agentDuration ?? ''}
                onChange={e =>
                  setForm(f => ({
                    ...f,
                    agentDuration: e.target.value ? Number(e.target.value) : undefined,
                  }))
                }
                style={{ width: 120 }}
              />
            </div>
          )}
          <label className="check-option">
            <input
              type="checkbox"
              checked={form.confirmBeforeUse}
              onChange={e => setForm(f => ({ ...f, confirmBeforeUse: e.target.checked }))}
            />
            <span>{t('ssh.confirmBeforeUse')}</span>
          </label>
          <div style={{ display: 'flex', gap: 'var(--space-sm)' }}>
            <button
              className="btn btn-primary"
              onClick={() => saveMutation.mutate()}
              disabled={saveMutation.isPending || !form.privateKey || !form.publicKey}
            >
              {saveMutation.isPending ? '...' : t('common.save')}
            </button>
            <button className="btn btn-secondary" onClick={() => setEditing(false)}>
              {t('common.cancel')}
            </button>
          </div>
        </div>
      )}

      {/* SSH key display */}
      {sshKey && !editing && (
        <div className="ssh-key-display">
          <div className="ssh-key-header">
            <span className="ssh-key-type-badge">{sshKey.keyType}</span>
            <span className="ssh-key-comment">{sshKey.comment}</span>
          </div>
          <div className="ssh-fingerprint">
            <span className="field-label">{t('ssh.fingerprint')}</span>
            <code className="ssh-fingerprint-value">{sshKey.fingerprint}</code>
          </div>
          <div className="ssh-public-key-row">
            <span className="field-label">{t('ssh.publicKey')}</span>
            <div className="ssh-public-key-actions">
              <code className="ssh-public-key-preview">{sshKey.publicKey.slice(0, 40)}...</code>
              <button
                className={`btn-icon ${copied ? 'copied' : ''}`}
                onClick={copyPublicKey}
                aria-label={t('entry.copyPassword')}
              >
                {copied ? '✓' : '⎘'}
              </button>
            </div>
          </div>
          <div className="ssh-private-key-row">
            <span className="field-label">{t('ssh.privateKey')}</span>
            <div style={{ display: 'flex', gap: 'var(--space-sm)', alignItems: 'center' }}>
              {showPrivateKey ? (
                <code className="ssh-private-key-value">{privateKey.slice(0, 30)}...</code>
              ) : (
                <span style={{ color: 'var(--color-text-tertiary)', fontSize: 13 }}>
                  ••••••••••••
                </span>
              )}
              <button
                className="btn-icon"
                onClick={showPrivateKey ? () => setShowPrivateKey(false) : revealPrivateKey}
              >
                {showPrivateKey ? '🙈' : '👁'}
              </button>
            </div>
          </div>
          <div className="ssh-settings">
            {sshKey.addToAgent && <span className="badge badge-green">{t('ssh.addToAgent')}</span>}
            {sshKey.confirmBeforeUse && (
              <span className="badge badge-blue">{t('ssh.confirmBeforeUse')}</span>
            )}
            {sshKey.agentDuration && (
              <span className="badge badge-blue">{sshKey.agentDuration}s</span>
            )}
          </div>
        </div>
      )}

      {!sshKey && !editing && (
        <div className="empty-section">
          <span style={{ fontSize: 32 }}>🔐</span>
          <p>{t('ssh.noKey')}</p>
          <p style={{ fontSize: 12 }}>{t('ssh.noKeyHint')}</p>
        </div>
      )}

      <style>{`
        .ssh-edit-form { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-md); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); margin-bottom:var(--space-lg); }
        .ssh-key-display { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-md); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); }
        .ssh-key-header { display:flex; align-items:center; gap:var(--space-sm); }
        .ssh-key-type-badge { background:var(--color-primary); color:white; font-size:11px; font-weight:700; padding:2px 8px; border-radius:var(--radius-full); }
        .ssh-key-comment { font-size:13px; color:var(--color-text-secondary); }
        .ssh-fingerprint { display:flex; flex-direction:column; gap:4px; }
        .ssh-fingerprint-value { font-family:'SF Mono','Consolas',monospace; font-size:12px; color:var(--color-text); background:var(--color-bg-tertiary); padding:4px 8px; border-radius:var(--radius-sm); word-break:break-all; }
        .ssh-public-key-row { display:flex; flex-direction:column; gap:4px; }
        .ssh-public-key-actions { display:flex; align-items:center; gap:var(--space-sm); }
        .ssh-public-key-preview { font-family:'SF Mono','Consolas',monospace; font-size:12px; color:var(--color-text-secondary); }
        .ssh-private-key-row { display:flex; flex-direction:column; gap:4px; }
        .ssh-private-key-value { font-family:'SF Mono','Consolas',monospace; font-size:12px; color:var(--color-text); }
        .ssh-settings { display:flex; gap:4px; flex-wrap:wrap; }
        .btn-danger { background:var(--color-danger); color:white; border:none; padding:var(--space-xs) var(--space-md); border-radius:var(--radius-md); cursor:pointer; font-size:12px; }
        .btn-danger:hover { opacity:0.9; }
      `}</style>
    </div>
  );
}
