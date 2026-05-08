/**
 * Scheduled Backup page — configure automatic vault backups
 */
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';

interface BackupConfig {
  enabled: boolean;
  frequency: 'on_save' | 'daily' | 'weekly' | 'monthly';
  destination: string;
  max_backups: number;
  timestamp_in_filename: boolean;
  last_backup_at?: string;
}

interface BackupRecord {
  path: string;
  created_at: string;
  size_bytes: number;
  vault_name: string;
}

export function BackupPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  const { isOpen, isLocked } = useVaultStore();

  const [config, setConfig] = useState<BackupConfig>({
    enabled: false,
    frequency: 'daily',
    destination: '',
    max_backups: 10,
    timestamp_in_filename: true,
  });

  const { data: backups = [], isLoading: backupsLoading } = useQuery({
    queryKey: ['backups'],
    queryFn: () => invoke<BackupRecord[]>('list_backups'),
    enabled: isOpen && !isLocked,
  });

  const { data: backupConfig } = useQuery({
    queryKey: ['backup-config'],
    queryFn: () => invoke<BackupConfig>('get_backup_config'),
    enabled: isOpen && !isLocked,
  });

  // Sync config from query result
  useEffect(() => {
    if (backupConfig) setConfig(backupConfig);
  }, [backupConfig]);

  if (!isOpen) {
    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          gap: 16,
          color: 'var(--color-text-secondary)',
        }}
      >
        <span style={{ fontSize: 48 }}>🔐</span>
        <p>Mở kho mật khẩu để xem trang này</p>
      </div>
    );
  }

  const saveMutation = useMutation({
    mutationFn: (cfg: BackupConfig) => invoke('save_backup_config', { config: cfg }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['backup-config'] }),
  });

  const backupNowMutation = useMutation({
    mutationFn: () => invoke('backup_now'),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['backups'] }),
  });

  const restoreMutation = useMutation({
    mutationFn: (path: string) => invoke('restore_from_backup', { backupPath: path }),
    onSuccess: () => navigate('/vault'),
  });

  const deleteMutation = useMutation({
    mutationFn: (path: string) => invoke('delete_backup', { backupPath: path }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['backups'] }),
  });

  const browseDestination = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setConfig(c => ({ ...c, destination: selected }));
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const FREQUENCIES = [
    { id: 'on_save', label: t('scheduledBackup.frequencyOnSave') },
    { id: 'daily', label: t('scheduledBackup.frequencyDaily') },
    { id: 'weekly', label: t('scheduledBackup.frequencyWeekly') },
    { id: 'monthly', label: t('scheduledBackup.frequencyMonthly') },
  ];

  return (
    <div className="backup-page">
      <div className="backup-header">
        <button className="btn-back" onClick={() => navigate('/settings')}>
          ← {t('settings.title')}
        </button>
        <h2>💾 {t('scheduledBackup.title')}</h2>
        <p className="backup-subtitle">{t('scheduledBackup.subtitle')}</p>
      </div>

      <div className="backup-content">
        <div className="backup-section">
          <div className="backup-section-header">
            <h3>{t('common.settings')}</h3>
            <div className="backup-enable-row">
              <span>{t('scheduledBackup.enable')}</span>
              <button
                role="switch"
                aria-checked={config.enabled}
                className="toggle"
                style={{
                  background: config.enabled ? 'var(--color-primary)' : 'var(--color-border)',
                }}
                onClick={() => setConfig(c => ({ ...c, enabled: !c.enabled }))}
              >
                <div className="toggle-thumb" style={{ left: config.enabled ? 22 : 2 }} />
              </button>
            </div>
          </div>

          {config.enabled && (
            <div className="backup-fields">
              <div className="backup-field">
                <label className="backup-label">{t('scheduledBackup.destination')}</label>
                <div className="backup-path-row">
                  <span className="backup-path">{config.destination || t('common.none')}</span>
                  <button className="btn btn-secondary" onClick={browseDestination}>
                    {t('scheduledBackup.browse')}
                  </button>
                </div>
              </div>

              <div className="backup-field">
                <label className="backup-label">{t('scheduledBackup.frequency')}</label>
                <div className="backup-freq-grid">
                  {FREQUENCIES.map(f => (
                    <button
                      key={f.id}
                      className={`backup-freq-btn${config.frequency === f.id ? ' active' : ''}`}
                      onClick={() =>
                        setConfig(c => ({ ...c, frequency: f.id as BackupConfig['frequency'] }))
                      }
                      aria-pressed={config.frequency === f.id}
                    >
                      {f.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="backup-field">
                <label className="backup-label">{t('scheduledBackup.maxBackups')}</label>
                <div className="backup-max-row">
                  <input
                    type="number"
                    className="form-input backup-max-input"
                    value={config.max_backups}
                    min={1}
                    max={100}
                    onChange={e => setConfig(c => ({ ...c, max_backups: Number(e.target.value) }))}
                  />
                  <span className="backup-max-label">{t('scheduledBackup.maxBackupsUnit')}</span>
                </div>
              </div>

              <div className="backup-field backup-field-row">
                <div>
                  <p className="backup-label">{t('scheduledBackup.timestampInFilename')}</p>
                  <p className="backup-field-desc">{t('scheduledBackup.timestampExample')}</p>
                </div>
                <button
                  role="switch"
                  aria-checked={config.timestamp_in_filename}
                  className="toggle"
                  style={{
                    background: config.timestamp_in_filename
                      ? 'var(--color-primary)'
                      : 'var(--color-border)',
                  }}
                  onClick={() =>
                    setConfig(c => ({ ...c, timestamp_in_filename: !c.timestamp_in_filename }))
                  }
                >
                  <div
                    className="toggle-thumb"
                    style={{ left: config.timestamp_in_filename ? 22 : 2 }}
                  />
                </button>
              </div>

              {config.last_backup_at && (
                <div className="backup-last">
                  <span>✅ {t('scheduledBackup.lastBackup')}:</span>
                  <span>{new Date(config.last_backup_at).toLocaleString()}</span>
                </div>
              )}
            </div>
          )}

          <div className="backup-actions">
            <button
              className="btn btn-primary"
              onClick={() => saveMutation.mutate(config)}
              disabled={saveMutation.isPending}
            >
              {saveMutation.isPending ? '⏳' : '💾'} {t('scheduledBackup.saveConfig')}
            </button>
            {config.enabled && config.destination && (
              <button
                className="btn btn-secondary"
                onClick={() => backupNowMutation.mutate()}
                disabled={backupNowMutation.isPending}
              >
                {backupNowMutation.isPending ? '⏳' : '📦'} {t('scheduledBackup.backupNow')}
              </button>
            )}
          </div>
        </div>

        <div className="backup-section">
          <h3>{t('scheduledBackup.history')}</h3>
          {backupsLoading ? (
            <p className="backup-loading">⏳ {t('common.loading')}</p>
          ) : backups.length === 0 ? (
            <div className="backup-empty">
              <span>📦</span>
              <p>{t('scheduledBackup.noBackups')}</p>
            </div>
          ) : (
            <div className="backup-list">
              {backups.map(backup => (
                <div key={backup.path} className="backup-item">
                  <div className="backup-item-info">
                    <p className="backup-item-name">{backup.path.split(/[/\\]/).pop()}</p>
                    <p className="backup-item-meta">
                      {new Date(backup.created_at).toLocaleString()} ·{' '}
                      {formatBytes(backup.size_bytes)}
                    </p>
                  </div>
                  <div className="backup-item-actions">
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => {
                        if (confirm(t('scheduledBackup.confirmRestore')))
                          restoreMutation.mutate(backup.path);
                      }}
                    >
                      ↩ {t('scheduledBackup.restore')}
                    </button>
                    <button
                      className="btn-icon"
                      onClick={() => {
                        if (confirm(t('scheduledBackup.confirmDelete')))
                          deleteMutation.mutate(backup.path);
                      }}
                      aria-label="Delete backup"
                    >
                      🗑
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <style>{`
        .backup-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .backup-header { padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; }
        .backup-header h2 { font-size:16px; font-weight:700; margin:4px 0; }
        .backup-subtitle { font-size:13px; color:var(--color-text-secondary); }
        .btn-back { background:none; border:none; cursor:pointer; color:var(--color-primary); font-size:13px; padding:0; margin-bottom:4px; }
        .backup-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:600px; }
        .backup-section { background:var(--color-bg-secondary); border-radius:var(--radius-md); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); border:1px solid var(--color-border); }
        .backup-section-header { display:flex; flex-direction:column; gap:var(--space-sm); }
        .backup-section h3 { font-size:14px; font-weight:600; }
        .backup-enable-row { display:flex; align-items:center; justify-content:space-between; }
        .backup-fields { display:flex; flex-direction:column; gap:var(--space-md); }
        .backup-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .backup-field-row { flex-direction:row; align-items:center; justify-content:space-between; }
        .backup-label { font-size:13px; font-weight:500; }
        .backup-field-desc { font-size:11px; color:var(--color-text-secondary); }
        .backup-path-row { display:flex; gap:var(--space-sm); align-items:center; }
        .backup-path { flex:1; padding:var(--space-sm) var(--space-md); background:var(--color-bg); border:1px solid var(--color-border); border-radius:var(--radius-sm); font-size:13px; color:var(--color-text); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
        .backup-freq-grid { display:grid; grid-template-columns:repeat(4,1fr); gap:var(--space-sm); }
        .backup-freq-btn { padding:var(--space-sm); border:1px solid var(--color-border); border-radius:var(--radius-sm); background:var(--color-bg); cursor:pointer; font-size:12px; color:var(--color-text-secondary); transition:all .1s; }
        .backup-freq-btn:hover { border-color:var(--color-primary); color:var(--color-primary); }
        .backup-freq-btn.active { background:var(--color-primary); border-color:var(--color-primary); color:white; }
        .backup-max-row { display:flex; align-items:center; gap:var(--space-sm); }
        .backup-max-input { width:80px; }
        .backup-max-label { font-size:13px; color:var(--color-text-secondary); }
        .backup-last { display:flex; gap:var(--space-sm); font-size:12px; color:var(--color-text-secondary); padding:var(--space-sm) var(--space-md); background:rgba(34,197,94,.08); border-radius:var(--radius-sm); border:1px solid rgba(34,197,94,.2); }
        .backup-actions { display:flex; gap:var(--space-sm); flex-wrap:wrap; }
        .backup-loading { font-size:13px; color:var(--color-text-secondary); }
        .backup-empty { display:flex; flex-direction:column; align-items:center; gap:var(--space-md); padding:var(--space-xl); color:var(--color-text-secondary); font-size:13px; }
        .backup-empty span { font-size:32px; }
        .backup-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .backup-item { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-sm) var(--space-md); background:var(--color-bg); border-radius:var(--radius-sm); border:1px solid var(--color-border); }
        .backup-item-info { flex:1; }
        .backup-item-name { font-size:13px; font-weight:500; color:var(--color-text); }
        .backup-item-meta { font-size:11px; color:var(--color-text-secondary); }
        .backup-item-actions { display:flex; gap:var(--space-xs); align-items:center; }
        .btn-sm { font-size:12px; padding:3px 10px; }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background .2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2); }
        .btn-icon { background:none; border:none; cursor:pointer; font-size:16px; color:var(--color-text-secondary); padding:4px 6px; border-radius:var(--radius-sm); }
        .btn-icon:hover { background:var(--color-bg-tertiary); }
      `}</style>
    </div>
  );
}
