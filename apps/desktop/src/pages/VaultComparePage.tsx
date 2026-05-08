/**
 * Vault Compare page — diff two KDBX vaults and merge changes
 * Unique feature: no competitor has this
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';

interface VaultDiff {
  only_in_first: EntryRef[];
  only_in_second: EntryRef[];
  modified: EntryModification[];
  identical_count: number;
  first_total: number;
  second_total: number;
}

interface EntryRef {
  uuid: string;
  title: string;
  username: string;
  url: string;
  modified_at: string;
}

interface EntryModification {
  uuid: string;
  title: string;
  newer_in: 'First' | 'Second' | 'Same';
  first_modified: string;
  second_modified: string;
  changed_fields: string[];
}

type MergeStrategy = 'KeepFirst' | 'KeepSecond' | 'KeepNewer' | 'KeepBoth';

export function VaultComparePage() {
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [vault1Path, setVault1Path] = useState('');
  const [vault2Path, setVault2Path] = useState('');
  const [password1, setPassword1] = useState('');
  const [password2, setPassword2] = useState('');
  const [diff, setDiff] = useState<VaultDiff | null>(null);
  const [mergeStrategy, setMergeStrategy] = useState<MergeStrategy>('KeepNewer');
  const [activeFilter, setActiveFilter] = useState<'all' | 'added' | 'removed' | 'modified'>('all');

  const compareMutation = useMutation({
    mutationFn: async () => {
      return invoke<VaultDiff>('compare_vaults_cmd', {
        vault1Path,
        vault2Path,
        password1,
        password2: password2 || password1,
      });
    },
    onSuccess: data => setDiff(data),
  });

  const mergeMutation = useMutation({
    mutationFn: async () => {
      return invoke('merge_vaults_cmd', {
        targetVaultPath: vault1Path,
        sourceVaultPath: vault2Path,
        password: password1,
        strategy: mergeStrategy,
      });
    },
    onSuccess: () => {
      navigate('/vault');
    },
  });

  const browseVault = async (which: 1 | 2) => {
    const selected = await open({
      filters: [{ name: 'KeePass Vault', extensions: ['kdbx'] }],
      multiple: false,
    });
    if (typeof selected === 'string') {
      if (which === 1) setVault1Path(selected);
      else setVault2Path(selected);
    }
  };

  const filteredOnlyFirst = diff?.only_in_first ?? [];
  const filteredOnlySecond = diff?.only_in_second ?? [];
  const filteredModified = diff?.modified ?? [];

  const totalDiffs = filteredOnlyFirst.length + filteredOnlySecond.length + filteredModified.length;

  return (
    <div className="compare-page">
      {/* Header */}
      <div className="compare-header">
        <button className="btn-back" onClick={() => navigate(-1)}>
          ← {t('common.back')}
        </button>
        <h2>🔀 {t('vaultCompare.title')}</h2>
        <p className="compare-subtitle">{t('vaultCompare.subtitle')}</p>
      </div>

      <div className="compare-content">
        {/* Vault selection */}
        <div className="compare-vaults">
          <VaultSelector
            label={t('vaultCompare.selectFirst')}
            path={vault1Path}
            password={password1}
            onBrowse={() => browseVault(1)}
            onPasswordChange={setPassword1}
            passwordPlaceholder={t('vault.masterPassword')}
            browsePlaceholder={t('common.none')}
            browseLabel={t('syncExt.browsePath')}
          />
          <div className="compare-vs">⇄</div>
          <VaultSelector
            label={t('vaultCompare.selectSecond')}
            path={vault2Path}
            password={password2}
            onBrowse={() => browseVault(2)}
            onPasswordChange={setPassword2}
            passwordPlaceholder={t('vault.masterPassword')}
            browsePlaceholder={t('common.none')}
            browseLabel={t('syncExt.browsePath')}
          />
        </div>

        {/* Compare button */}
        <div className="compare-actions">
          <button
            className="btn btn-primary"
            onClick={() => compareMutation.mutate()}
            disabled={!vault1Path || !vault2Path || !password1 || compareMutation.isPending}
          >
            {compareMutation.isPending
              ? `⏳ ${t('vaultCompare.comparing')}`
              : `🔍 ${t('vaultCompare.compare')}`}
          </button>
        </div>

        {/* Error */}
        {compareMutation.isError && (
          <div className="compare-error">⚠️ {String(compareMutation.error)}</div>
        )}

        {/* Results */}
        {diff && (
          <div className="compare-results">
            {/* Summary */}
            <div className="compare-summary">
              <div className="summary-stat">
                <span className="summary-num">{diff.first_total}</span>
                <span className="summary-label">{t('vaultCompare.onlyInFirst')}</span>
              </div>
              <div className="summary-stat">
                <span className="summary-num">{diff.second_total}</span>
                <span className="summary-label">{t('vaultCompare.onlyInSecond')}</span>
              </div>
              <div className="summary-stat summary-stat--identical">
                <span className="summary-num">{diff.identical_count}</span>
                <span className="summary-label">{t('vaultCompare.identical')}</span>
              </div>
              <div className="summary-stat summary-stat--diff">
                <span className="summary-num">{totalDiffs}</span>
                <span className="summary-label">{t('vaultCompare.modified')}</span>
              </div>
            </div>

            {totalDiffs === 0 ? (
              <div className="compare-identical">
                <span>✅</span>
                <p>{t('vaultCompare.noChanges')}</p>
              </div>
            ) : (
              <>
                {/* Filter tabs */}
                <div className="compare-filters" role="tablist">
                  {[
                    { id: 'all', label: t('vaultFilter.all'), count: totalDiffs },
                    {
                      id: 'added',
                      label: t('vaultCompare.onlyInSecond'),
                      count: filteredOnlySecond.length,
                    },
                    {
                      id: 'removed',
                      label: t('vaultCompare.onlyInFirst'),
                      count: filteredOnlyFirst.length,
                    },
                    {
                      id: 'modified',
                      label: t('vaultCompare.modified'),
                      count: filteredModified.length,
                    },
                  ].map(f => (
                    <button
                      key={f.id}
                      role="tab"
                      aria-selected={activeFilter === f.id}
                      className={`compare-filter-btn${activeFilter === f.id ? ' active' : ''}`}
                      onClick={() => setActiveFilter(f.id as typeof activeFilter)}
                    >
                      {f.label}
                      {f.count > 0 && <span className="filter-count">{f.count}</span>}
                    </button>
                  ))}
                </div>

                {/* Diff list */}
                <div className="diff-list">
                  {(activeFilter === 'all' || activeFilter === 'added') &&
                    filteredOnlySecond.map(e => (
                      <DiffRow key={e.uuid} type="added" entry={e} addedLabel={t('common.add')} />
                    ))}
                  {(activeFilter === 'all' || activeFilter === 'removed') &&
                    filteredOnlyFirst.map(e => (
                      <DiffRow
                        key={e.uuid}
                        type="removed"
                        entry={e}
                        addedLabel={t('common.delete')}
                      />
                    ))}
                  {(activeFilter === 'all' || activeFilter === 'modified') &&
                    filteredModified.map(m => (
                      <ModifiedRow
                        key={m.uuid}
                        mod={m}
                        changedLabel={t('vaultCompare.modified')}
                        vault1NewerLabel={t('vaultCompare.onlyInFirst')}
                        vault2NewerLabel={t('vaultCompare.onlyInSecond')}
                        sameTimeLabel={t('vaultCompare.identical')}
                      />
                    ))}
                </div>

                {/* Merge section */}
                <div className="merge-section">
                  <h3>{t('vaultCompare.mergeAll')}</h3>
                  <div className="merge-strategy">
                    <label className="merge-label">{t('importExport.conflictStrategy')}</label>
                    <select
                      className="merge-select"
                      value={mergeStrategy}
                      onChange={e => setMergeStrategy(e.target.value as MergeStrategy)}
                    >
                      <option value="KeepNewer">{t('importExport.conflictSkip')}</option>
                      <option value="KeepFirst">{t('vaultCompare.onlyInFirst')}</option>
                      <option value="KeepSecond">{t('vaultCompare.onlyInSecond')}</option>
                      <option value="KeepBoth">{t('importExport.conflictKeepBoth')}</option>
                    </select>
                  </div>
                  <button
                    className="btn btn-primary"
                    onClick={() => {
                      if (confirm(t('common.confirm'))) {
                        mergeMutation.mutate();
                      }
                    }}
                    disabled={mergeMutation.isPending}
                  >
                    {mergeMutation.isPending
                      ? `⏳ ${t('common.loading')}`
                      : `🔀 ${t('vaultCompare.mergeSelected')}`}
                  </button>
                </div>
              </>
            )}
          </div>
        )}
      </div>

      <style>{`
        .compare-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .compare-header {
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .compare-header h2 { font-size:18px; font-weight:700; margin:4px 0; }
        .compare-subtitle { font-size:13px; color:var(--color-text-secondary); }
        .btn-back {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:13px; padding:0; margin-bottom:4px;
        }
        .compare-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl);
        }
        .compare-vaults { display:grid; grid-template-columns:1fr auto 1fr; gap:var(--space-lg); align-items:start; }
        .compare-vs { font-size:24px; color:var(--color-text-tertiary); padding-top:32px; text-align:center; }
        .vault-selector { display:flex; flex-direction:column; gap:var(--space-sm); }
        .vault-selector-label { font-size:12px; font-weight:600; color:var(--color-text-secondary); text-transform:uppercase; letter-spacing:.05em; }
        .vault-path-row { display:flex; gap:var(--space-sm); }
        .vault-path {
          flex:1; padding:var(--space-sm) var(--space-md);
          background:var(--color-bg-secondary); border:1px solid var(--color-border);
          border-radius:var(--radius-sm); font-size:13px; color:var(--color-text);
          overflow:hidden; text-overflow:ellipsis; white-space:nowrap;
        }
        .compare-actions { display:flex; justify-content:center; }
        .compare-error {
          padding:var(--space-md); background:rgba(239,68,68,.1);
          border:1px solid rgba(239,68,68,.3); border-radius:var(--radius-md);
          font-size:13px; color:#ef4444;
        }
        .compare-results { display:flex; flex-direction:column; gap:var(--space-lg); }
        .compare-summary { display:flex; gap:var(--space-lg); }
        .summary-stat {
          flex:1; display:flex; flex-direction:column; align-items:center; gap:4px;
          padding:var(--space-md); background:var(--color-bg-secondary);
          border-radius:var(--radius-md); border:1px solid var(--color-border);
        }
        .summary-num { font-size:28px; font-weight:700; color:var(--color-text); }
        .summary-label { font-size:11px; color:var(--color-text-secondary); }
        .summary-stat--identical .summary-num { color:#16a34a; }
        .summary-stat--diff .summary-num { color:#f59e0b; }
        .compare-identical {
          display:flex; flex-direction:column; align-items:center; gap:var(--space-md);
          padding:var(--space-2xl); color:var(--color-text-secondary); font-size:15px;
        }
        .compare-identical span { font-size:40px; }
        .compare-filters { display:flex; gap:2px; border-bottom:1px solid var(--color-border); }
        .compare-filter-btn {
          background:none; border:none; cursor:pointer; padding:var(--space-sm) var(--space-md);
          font-size:13px; color:var(--color-text-secondary);
          border-bottom:2px solid transparent; margin-bottom:-1px;
          display:flex; align-items:center; gap:6px;
        }
        .compare-filter-btn:hover { color:var(--color-text); }
        .compare-filter-btn.active { color:var(--color-primary); border-bottom-color:var(--color-primary); font-weight:500; }
        .filter-count {
          background:var(--color-bg-tertiary); color:var(--color-text-secondary);
          font-size:10px; padding:1px 5px; border-radius:10px;
        }
        .diff-list { display:flex; flex-direction:column; gap:2px; }
        .diff-row {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-sm) var(--space-md);
          border-radius:var(--radius-sm); border:1px solid transparent;
        }
        .diff-row--added { background:rgba(34,197,94,.06); border-color:rgba(34,197,94,.2); }
        .diff-row--removed { background:rgba(239,68,68,.06); border-color:rgba(239,68,68,.2); }
        .diff-row--modified { background:rgba(234,179,8,.06); border-color:rgba(234,179,8,.2); }
        .diff-badge {
          font-size:11px; font-weight:700; padding:2px 6px; border-radius:4px;
          flex-shrink:0; min-width:24px; text-align:center;
        }
        .diff-badge--added { background:#dcfce7; color:#16a34a; }
        .diff-badge--removed { background:#fee2e2; color:#ef4444; }
        .diff-badge--modified { background:#fef9c3; color:#ca8a04; }
        .diff-info { flex:1; }
        .diff-title { font-size:13px; font-weight:500; color:var(--color-text); }
        .diff-username { font-size:11px; color:var(--color-text-secondary); }
        .diff-changed { font-size:11px; color:var(--color-text-tertiary); }
        .diff-newer { font-size:11px; color:var(--color-primary); }
        .merge-section {
          padding:var(--space-lg); background:var(--color-bg-secondary);
          border-radius:var(--radius-md); border:1px solid var(--color-border);
          display:flex; flex-direction:column; gap:var(--space-md);
        }
        .merge-section h3 { font-size:14px; font-weight:600; }
        .merge-strategy { display:flex; align-items:center; gap:var(--space-md); }
        .merge-label { font-size:13px; color:var(--color-text-secondary); }
        .merge-select {
          background:var(--color-bg); border:1px solid var(--color-border);
          border-radius:var(--radius-sm); padding:var(--space-xs) var(--space-sm);
          font-size:13px; color:var(--color-text); cursor:pointer;
        }
      `}</style>
    </div>
  );
}

function VaultSelector({
  label,
  path,
  password,
  onBrowse,
  onPasswordChange,
  passwordPlaceholder,
  browsePlaceholder,
  browseLabel,
}: {
  label: string;
  path: string;
  password: string;
  onBrowse: () => void;
  onPasswordChange: (v: string) => void;
  passwordPlaceholder?: string;
  browsePlaceholder?: string;
  browseLabel?: string;
}) {
  return (
    <div className="vault-selector">
      <span className="vault-selector-label">{label}</span>
      <div className="vault-path-row">
        <span className="vault-path">{path || browsePlaceholder}</span>
        <button className="btn btn-secondary" onClick={onBrowse}>
          {browseLabel ?? '...'}
        </button>
      </div>
      <input
        type="password"
        className="form-input"
        value={password}
        onChange={e => onPasswordChange(e.target.value)}
        placeholder={passwordPlaceholder}
        autoComplete="current-password"
      />
    </div>
  );
}

function DiffRow({
  type,
  entry,
  addedLabel,
}: {
  type: 'added' | 'removed';
  entry: EntryRef;
  addedLabel: string;
}) {
  const badge = type === 'added' ? '+' : '−';
  return (
    <div className={`diff-row diff-row--${type}`}>
      <span className={`diff-badge diff-badge--${type}`}>{badge}</span>
      <div className="diff-info">
        <p className="diff-title">{entry.title}</p>
        <p className="diff-username">{entry.username}</p>
      </div>
      <span className="diff-changed">{new Date(entry.modified_at).toLocaleDateString()}</span>
    </div>
  );
}

function ModifiedRow({
  mod,
  changedLabel,
  vault1NewerLabel,
  vault2NewerLabel,
  sameTimeLabel,
}: {
  mod: EntryModification;
  changedLabel: string;
  vault1NewerLabel: string;
  vault2NewerLabel: string;
  sameTimeLabel: string;
}) {
  const newerLabel =
    mod.newer_in === 'First'
      ? vault1NewerLabel
      : mod.newer_in === 'Second'
        ? vault2NewerLabel
        : sameTimeLabel;

  return (
    <div className="diff-row diff-row--modified">
      <span className="diff-badge diff-badge--modified">~</span>
      <div className="diff-info">
        <p className="diff-title">{mod.title}</p>
        <p className="diff-changed">
          {changedLabel}: {mod.changed_fields.join(', ')}
        </p>
      </div>
      <span className="diff-newer">{newerLabel}</span>
    </div>
  );
}
