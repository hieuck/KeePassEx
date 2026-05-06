/**
 * Scheduled Backup page — configure automatic vault backups
 * Unique feature: no competitor has built-in scheduled backup
 */
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '../store/settings';

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
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';

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
  });

  const { data: savedConfig } = useQuery({
    queryKey: ['backup-config'],
    queryFn: () => invoke<BackupConfig>('get_backup_config'),
    onSuccess: (data: BackupConfig) => setConfig(data),
  });

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
    if (typeof selected === 'string') {
      setConfig(c => ({ ...c, destination: selected }));
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const FREQUENCIES = [
    { id: 'on_save', label: isVi ? 'Mỗi lần lưu' : 'On every save' },
    { id: 'daily', label: isVi ? 'Hàng ngày' : 'Daily' },
    { id: 'weekly', label: isVi ? 'Hàng tuần' : 'Weekly' },
    { id: 'monthly', label: isVi ? 'Hàng tháng' : 'Monthly' },
  ];

  return (
    <div className="backup-page">
      {/* Header */}
      <div className="backup-header">
        <button className="btn-back" onClick={() => navigate('/settings')}>
          ← {isVi ? 'Cài đặt' : 'Settings'}
        </button>
        <h2>{isVi ? '💾 Sao lưu tự động' : '💾 Scheduled Backup'}</h2>
        <p className="backup-subtitle">
          {isVi
            ? 'Tự động sao lưu kho theo lịch — tính năng độc quyền của KeePassEx'
            : 'Automatically backup your vault on schedule — exclusive KeePassEx feature'}
        </p>
      </div>

      <div className="backup-content">
        {/* Configuration */}
        <div className="backup-section">
          <div className="backup-section-header">
            <h3>{isVi ? 'Cấu hình' : 'Configuration'}</h3>
            <div className="backup-enable-row">
              <span>{isVi ? 'Bật sao lưu tự động' : 'Enable automatic backup'}</span>
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
              {/* Destination */}
              <div className="backup-field">
                <label className="backup-label">
                  {isVi ? 'Thư mục đích' : 'Backup destination'}
                </label>
                <div className="backup-path-row">
                  <span className="backup-path">
                    {config.destination || (isVi ? 'Chưa chọn...' : 'Not selected...')}
                  </span>
                  <button className="btn btn-secondary" onClick={browseDestination}>
                    {isVi ? 'Duyệt...' : 'Browse...'}
                  </button>
                </div>
              </div>

              {/* Frequency */}
              <div className="backup-field">
                <label className="backup-label">{isVi ? 'Tần suất' : 'Frequency'}</label>
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

              {/* Max backups */}
              <div className="backup-field">
                <label className="backup-label">{isVi ? 'Giữ tối đa' : 'Keep at most'}</label>
                <div className="backup-max-row">
                  <input
                    type="number"
                    className="form-input backup-max-input"
                    value={config.max_backups}
                    min={1}
                    max={100}
                    onChange={e => setConfig(c => ({ ...c, max_backups: Number(e.target.value) }))}
                    aria-label={isVi ? 'Số bản sao lưu tối đa' : 'Maximum backups to keep'}
                  />
                  <span className="backup-max-label">{isVi ? 'bản sao lưu' : 'backup files'}</span>
                </div>
              </div>

              {/* Timestamp in filename */}
              <div className="backup-field backup-field-row">
                <div>
                  <p className="backup-label">
                    {isVi ? 'Thêm thời gian vào tên tệp' : 'Include timestamp in filename'}
                  </p>
                  <p className="backup-field-desc">
                    {isVi
                      ? 'Ví dụ: vault_backup_20250506_143022.kdbx'
                      : 'e.g. vault_backup_20250506_143022.kdbx'}
                  </p>
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

              {/* Last backup */}
              {config.last_backup_at && (
                <div className="backup-last">
                  <span>✅ {isVi ? 'Sao lưu lần cuối' : 'Last backup'}:</span>
                  <span>{new Date(config.last_backup_at).toLocaleString()}</span>
                </div>
              )}
            </div>
          )}

          {/* Save + Backup Now */}
          <div className="backup-actions">
            <button
              className="btn btn-primary"
              onClick={() => saveMutation.mutate(config)}
              disabled={saveMutation.isPending}
            >
              {saveMutation.isPending ? '⏳' : '💾'} {isVi ? 'Lưu cấu hình' : 'Save Configuration'}
            </button>
            {config.enabled && config.destination && (
              <button
                className="btn btn-secondary"
                onClick={() => backupNowMutation.mutate()}
                disabled={backupNowMutation.isPending}
              >
                {backupNowMutation.isPending ? '⏳' : '📦'} {isVi ? 'Sao lưu ngay' : 'Backup Now'}
              </button>
            )}
          </div>
        </div>

        {/* Backup history */}
        <div className="backup-section">
          <h3>{isVi ? 'Lịch sử sao lưu' : 'Backup History'}</h3>
          {backupsLoading ? (
            <p className="backup-loading">⏳ {isVi ? 'Đang tải...' : 'Loading...'}</p>
          ) : backups.length === 0 ? (
            <div className="backup-empty">
              <span>📦</span>
              <p>{isVi ? 'Chưa có bản sao lưu nào' : 'No backups yet'}</p>
            </div>
          ) : (
            <div className="backup-list">
              {backups.map((backup, i) => (
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
                        if (
                          confirm(
                            isVi
                              ? 'Khôi phục từ bản sao lưu này? Kho hiện tại sẽ bị thay thế.'
                              : 'Restore from this backup? Current vault will be replaced.'
                          )
                        ) {
                          restoreMutation.mutate(backup.path);
                        }
                      }}
                    >
                      ↩ {isVi ? 'Khôi phục' : 'Restore'}
                    </button>
                    <button
                      className="btn-icon"
                      onClick={() => {
                        if (confirm(isVi ? 'Xóa bản sao lưu này?' : 'Delete this backup?')) {
                          deleteMutation.mutate(backup.path);
                        }
                      }}
                      aria-label={isVi ? 'Xóa bản sao lưu' : 'Delete backup'}
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
        .backup-header {
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .backup-header h2 { font-size:16px; font-weight:700; margin:4px 0; }
        .backup-subtitle { font-size:13px; color:var(--color-text-secondary); }
        .btn-back {
          background:none; border:none; cursor:pointer; color:var(--color-primary);
          font-size:13px; padding:0; margin-bottom:4px;
        }
        .backup-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl); max-width:600px;
        }
        .backup-section {
          background:var(--color-bg-secondary); border-radius:var(--radius-md);
          padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md);
          border:1px solid var(--color-border);
        }
        .backup-section-header { display:flex; flex-direction:column; gap:var(--space-sm); }
        .backup-section h3 { font-size:14px; font-weight:600; }
        .backup-enable-row { display:flex; align-items:center; justify-content:space-between; }
        .backup-fields { display:flex; flex-direction:column; gap:var(--space-md); }
        .backup-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .backup-field-row { flex-direction:row; align-items:center; justify-content:space-between; }
        .backup-label { font-size:13px; font-weight:500; }
        .backup-field-desc { font-size:11px; color:var(--color-text-secondary); }
        .backup-path-row { display:flex; gap:var(--space-sm); align-items:center; }
        .backup-path {
          flex:1; padding:var(--space-sm) var(--space-md);
          background:var(--color-bg); border:1px solid var(--color-border);
          border-radius:var(--radius-sm); font-size:13px; color:var(--color-text);
          overflow:hidden; text-overflow:ellipsis; white-space:nowrap;
        }
        .backup-freq-grid { display:grid; grid-template-columns:repeat(4,1fr); gap:var(--space-sm); }
        .backup-freq-btn {
          padding:var(--space-sm); border:1px solid var(--color-border);
          border-radius:var(--radius-sm); background:var(--color-bg);
          cursor:pointer; font-size:12px; color:var(--color-text-secondary);
          transition:all .1s;
        }
        .backup-freq-btn:hover { border-color:var(--color-primary); color:var(--color-primary); }
        .backup-freq-btn.active { background:var(--color-primary); border-color:var(--color-primary); color:white; }
        .backup-max-row { display:flex; align-items:center; gap:var(--space-sm); }
        .backup-max-input { width:80px; }
        .backup-max-label { font-size:13px; color:var(--color-text-secondary); }
        .backup-last {
          display:flex; gap:var(--space-sm); font-size:12px; color:var(--color-text-secondary);
          padding:var(--space-sm) var(--space-md); background:rgba(34,197,94,.08);
          border-radius:var(--radius-sm); border:1px solid rgba(34,197,94,.2);
        }
        .backup-actions { display:flex; gap:var(--space-sm); flex-wrap:wrap; }
        .backup-loading { font-size:13px; color:var(--color-text-secondary); }
        .backup-empty {
          display:flex; flex-direction:column; align-items:center; gap:var(--space-md);
          padding:var(--space-xl); color:var(--color-text-secondary); font-size:13px;
        }
        .backup-empty span { font-size:32px; }
        .backup-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .backup-item {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-sm) var(--space-md);
          background:var(--color-bg); border-radius:var(--radius-sm);
          border:1px solid var(--color-border);
        }
        .backup-item-info { flex:1; }
        .backup-item-name { font-size:13px; font-weight:500; color:var(--color-text); }
        .backup-item-meta { font-size:11px; color:var(--color-text-secondary); }
        .backup-item-actions { display:flex; gap:var(--space-xs); align-items:center; }
        .btn-sm { font-size:12px; padding:3px 10px; }
        .toggle {
          width:44px; height:24px; border-radius:12px; border:none;
          cursor:pointer; position:relative; transition:background .2s; flex-shrink:0;
        }
        .toggle-thumb {
          position:absolute; top:2px; width:20px; height:20px;
          border-radius:50%; background:white; transition:left .2s;
          box-shadow:0 1px 3px rgba(0,0,0,.2);
        }
        .icon-btn {
          background:none; border:none; cursor:pointer; font-size:16px;
          color:var(--color-text-secondary); padding:4px 6px; border-radius:var(--radius-sm);
        }
        .icon-btn:hover { background:var(--color-bg-tertiary); }
      `}</style>
    </div>
  );
}
