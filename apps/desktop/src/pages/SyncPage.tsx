/**
 * Sync configuration & status page — desktop
 */
import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';
import { useSyncStore } from '../store/sync';

type SyncProvider = 'local' | 'webdav' | 'icloud' | 'gdrive' | 'onedrive' | 'dropbox' | 's3' | 'sftp';

interface SyncStatus {
  configured: boolean;
  provider: string | null;
  remotePath: string | null;
  autoSync: boolean;
  lastSync: string | null;
  lastSyncStatus: string | null;
}

interface SyncResult {
  status: string;
  entriesUploaded: number;
  entriesDownloaded: number;
  conflicts: number;
  error: string | null;
}

const PROVIDERS: { value: SyncProvider; labelEn: string; labelVi: string; icon: string; needsAuth: boolean }[] = [
  { value: 'local',    labelEn: 'Local Folder',   labelVi: 'Thư mục cục bộ', icon: '📁', needsAuth: false },
  { value: 'webdav',   labelEn: 'WebDAV',          labelVi: 'WebDAV',          icon: '🌐', needsAuth: true  },
  { value: 'icloud',   labelEn: 'iCloud Drive',    labelVi: 'iCloud Drive',    icon: '☁️', needsAuth: false },
  { value: 'gdrive',   labelEn: 'Google Drive',    labelVi: 'Google Drive',    icon: '🔵', needsAuth: true  },
  { value: 'onedrive', labelEn: 'OneDrive',        labelVi: 'OneDrive',        icon: '🔷', needsAuth: true  },
  { value: 'dropbox',  labelEn: 'Dropbox',         labelVi: 'Dropbox',         icon: '📦', needsAuth: true  },
  { value: 's3',       labelEn: 'Amazon S3',       labelVi: 'Amazon S3',       icon: '🟠', needsAuth: true  },
  { value: 'sftp',     labelEn: 'SFTP',            labelVi: 'SFTP',            icon: '🔒', needsAuth: true  },
];

export function SyncPage() {
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';

  const [provider, setProvider] = useState<SyncProvider>('local');
  const [remotePath, setRemotePath] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [serverUrl, setServerUrl] = useState('');
  const [autoSync, setAutoSync] = useState(false);
  const [interval, setInterval] = useState(300);
  const [conflict, setConflict] = useState('merge');
  const [testResult, setTestResult] = useState<string | null>(null);

  const { data: status } = useQuery({
    queryKey: ['sync-status'],
    queryFn: () => invoke<SyncStatus>('get_sync_status'),
    refetchInterval: 30_000,
  });

  const configureMutation = useMutation({
    mutationFn: () => invoke('configure_sync', {
      args: {
        provider,
        remote_path: remotePath,
        auto_sync: autoSync,
        sync_interval_seconds: interval,
        conflict_resolution: conflict,
        username: username || null,
        password: password || null,
        server_url: serverUrl || null,
      },
    }),
  });

  const syncNowMutation = useMutation({
    mutationFn: () => invoke<SyncResult>('sync_now'),
  });

  const handleTestConnection = async () => {
    setTestResult(null);
    try {
      const ok = await invoke<boolean>('test_sync_connection', {
        provider, remote_path: remotePath,
        username: username || null,
        password: password || null,
      });
      setTestResult(ok
        ? (isVi ? '✅ Kết nối thành công!' : '✅ Connection successful!')
        : (isVi ? '❌ Kết nối thất bại' : '❌ Connection failed'));
    } catch (e: unknown) {
      setTestResult(`❌ ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const selectedProvider = PROVIDERS.find(p => p.value === provider)!;

  return (
    <div className="sync-page">
      <div className="sync-header">
        <h2>{isVi ? '🔄 Đồng bộ' : '🔄 Sync'}</h2>
        {status?.configured && (
          <button
            className="btn btn-primary"
            onClick={() => syncNowMutation.mutate()}
            disabled={syncNowMutation.isPending}
          >
            {syncNowMutation.isPending
              ? (isVi ? 'Đang đồng bộ...' : 'Syncing...')
              : (isVi ? '🔄 Đồng bộ ngay' : '🔄 Sync Now')}
          </button>
        )}
      </div>

      <div className="sync-content">
        {/* Status card */}
        {status && (
          <div className="sync-status-card">
            <div className="sync-status-row">
              <span className="sync-status-dot" style={{
                background: status.configured ? 'var(--color-success)' : 'var(--color-text-tertiary)'
              }} />
              <span className="sync-status-label">
                {status.configured
                  ? (isVi ? `Đã cấu hình: ${status.provider}` : `Configured: ${status.provider}`)
                  : (isVi ? 'Chưa cấu hình' : 'Not configured')}
              </span>
            </div>
            {status.lastSync && (
              <p className="sync-last">
                {isVi ? `Đồng bộ lần cuối: ${status.lastSync}` : `Last sync: ${status.lastSync}`}
              </p>
            )}
            {syncNowMutation.data && (
              <p className="sync-result" style={{
                color: syncNowMutation.data.status === 'success' ? 'var(--color-success)' : 'var(--color-danger)'
              }}>
                {syncNowMutation.data.status === 'success'
                  ? (isVi ? '✅ Đồng bộ thành công' : '✅ Sync successful')
                  : `❌ ${syncNowMutation.data.error}`}
              </p>
            )}
          </div>
        )}

        {/* Provider selection */}
        <section className="sync-section">
          <h3 className="sync-section-title">{isVi ? 'Nhà cung cấp' : 'Provider'}</h3>
          <div className="provider-grid" role="radiogroup" aria-label={isVi ? 'Chọn nhà cung cấp' : 'Select provider'}>
            {PROVIDERS.map(p => (
              <button
                key={p.value}
                role="radio"
                aria-checked={provider === p.value}
                className={`provider-btn ${provider === p.value ? 'selected' : ''}`}
                onClick={() => setProvider(p.value)}
              >
                <span className="provider-icon">{p.icon}</span>
                <span className="provider-label">{isVi ? p.labelVi : p.labelEn}</span>
              </button>
            ))}
          </div>
        </section>

        {/* Remote path */}
        <section className="sync-section">
          <h3 className="sync-section-title">
            {isVi ? 'Đường dẫn từ xa' : 'Remote Path'}
          </h3>
          {selectedProvider.needsAuth && (
            <div className="sync-form-row">
              <label className="sync-label" htmlFor="server-url">
                {isVi ? 'URL máy chủ' : 'Server URL'}
              </label>
              <input
                id="server-url"
                type="url"
                className="sync-input"
                value={serverUrl}
                onChange={e => setServerUrl(e.target.value)}
                placeholder="https://dav.example.com"
              />
            </div>
          )}
          <div className="sync-form-row">
            <label className="sync-label" htmlFor="remote-path">
              {provider === 'local'
                ? (isVi ? 'Đường dẫn thư mục' : 'Folder path')
                : (isVi ? 'Đường dẫn tệp từ xa' : 'Remote file path')}
            </label>
            <input
              id="remote-path"
              type="text"
              className="sync-input"
              value={remotePath}
              onChange={e => setRemotePath(e.target.value)}
              placeholder={provider === 'local' ? '/path/to/backup/' : '/keepassex/vault.kdbx'}
            />
          </div>

          {selectedProvider.needsAuth && (
            <>
              <div className="sync-form-row">
                <label className="sync-label" htmlFor="sync-user">
                  {isVi ? 'Tên đăng nhập' : 'Username'}
                </label>
                <input
                  id="sync-user"
                  type="text"
                  className="sync-input"
                  value={username}
                  onChange={e => setUsername(e.target.value)}
                  autoComplete="username"
                />
              </div>
              <div className="sync-form-row">
                <label className="sync-label" htmlFor="sync-pass">
                  {isVi ? 'Mật khẩu' : 'Password'}
                </label>
                <input
                  id="sync-pass"
                  type="password"
                  className="sync-input"
                  value={password}
                  onChange={e => setPassword(e.target.value)}
                  autoComplete="current-password"
                />
              </div>
            </>
          )}

          <div className="sync-actions">
            <button className="btn btn-secondary" onClick={handleTestConnection}>
              {isVi ? '🔌 Kiểm tra kết nối' : '🔌 Test Connection'}
            </button>
          </div>
          {testResult && (
            <p className="sync-test-result" style={{
              color: testResult.startsWith('✅') ? 'var(--color-success)' : 'var(--color-danger)'
            }}>
              {testResult}
            </p>
          )}
        </section>

        {/* Options */}
        <section className="sync-section">
          <h3 className="sync-section-title">{isVi ? 'Tùy chọn' : 'Options'}</h3>

          <div className="sync-toggle-row">
            <div>
              <p className="sync-toggle-label">{isVi ? 'Tự động đồng bộ' : 'Auto sync'}</p>
              <p className="sync-toggle-desc">{isVi ? 'Đồng bộ định kỳ khi kho đang mở' : 'Periodically sync while vault is open'}</p>
            </div>
            <button
              role="switch"
              aria-checked={autoSync}
              className="toggle"
              style={{ background: autoSync ? 'var(--color-primary)' : 'var(--color-border)' }}
              onClick={() => setAutoSync(v => !v)}
            >
              <div className="toggle-thumb" style={{ left: autoSync ? 22 : 2 }} />
            </button>
          </div>

          {autoSync && (
            <div className="sync-form-row">
              <label className="sync-label" htmlFor="sync-interval">
                {isVi ? 'Chu kỳ (giây)' : 'Interval (seconds)'}
              </label>
              <select
                id="sync-interval"
                className="sync-select"
                value={interval}
                onChange={e => setInterval(Number(e.target.value))}
              >
                <option value={60}>1 {isVi ? 'phút' : 'minute'}</option>
                <option value={300}>5 {isVi ? 'phút' : 'minutes'}</option>
                <option value={900}>15 {isVi ? 'phút' : 'minutes'}</option>
                <option value={3600}>1 {isVi ? 'giờ' : 'hour'}</option>
              </select>
            </div>
          )}

          <div className="sync-form-row">
            <label className="sync-label" htmlFor="conflict-res">
              {isVi ? 'Xử lý xung đột' : 'Conflict resolution'}
            </label>
            <select
              id="conflict-res"
              className="sync-select"
              value={conflict}
              onChange={e => setConflict(e.target.value)}
            >
              <option value="merge">{isVi ? 'Hợp nhất (khuyến nghị)' : 'Merge (recommended)'}</option>
              <option value="keepLocal">{isVi ? 'Giữ bản cục bộ' : 'Keep local'}</option>
              <option value="keepRemote">{isVi ? 'Giữ bản từ xa' : 'Keep remote'}</option>
              <option value="askUser">{isVi ? 'Hỏi tôi' : 'Ask me'}</option>
            </select>
          </div>
        </section>

        {/* Save */}
        <button
          className="btn btn-primary"
          onClick={() => configureMutation.mutate()}
          disabled={configureMutation.isPending || !remotePath.trim()}
        >
          {configureMutation.isPending
            ? (isVi ? 'Đang lưu...' : 'Saving...')
            : (isVi ? '💾 Lưu cấu hình' : '💾 Save Configuration')}
        </button>
        {configureMutation.isSuccess && (
          <p style={{ color: 'var(--color-success)', fontSize: 13 }}>
            ✅ {isVi ? 'Đã lưu cấu hình đồng bộ' : 'Sync configuration saved'}
          </p>
        )}
      </div>

      <style>{`
        .sync-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .sync-header {
          display:flex; align-items:center; justify-content:space-between;
          padding:var(--space-md) var(--space-xl);
          border-bottom:1px solid var(--color-border); flex-shrink:0;
        }
        .sync-header h2 { font-size:16px; font-weight:600; }
        .sync-content {
          flex:1; overflow-y:auto; padding:var(--space-xl);
          display:flex; flex-direction:column; gap:var(--space-xl); max-width:600px;
        }
        .sync-status-card {
          background:var(--color-bg-secondary); border:1px solid var(--color-border);
          border-radius:var(--radius-md); padding:var(--space-md);
          display:flex; flex-direction:column; gap:var(--space-xs);
        }
        .sync-status-row { display:flex; align-items:center; gap:var(--space-sm); }
        .sync-status-dot { width:8px; height:8px; border-radius:50%; flex-shrink:0; }
        .sync-status-label { font-size:14px; font-weight:500; }
        .sync-last { font-size:12px; color:var(--color-text-secondary); }
        .sync-result { font-size:13px; font-weight:500; }
        .sync-section { display:flex; flex-direction:column; gap:var(--space-md); }
        .sync-section-title { font-size:14px; font-weight:600; }
        .provider-grid {
          display:grid; grid-template-columns:repeat(4,1fr); gap:var(--space-sm);
        }
        .provider-btn {
          display:flex; flex-direction:column; align-items:center; gap:4px;
          padding:var(--space-sm); border:1px solid var(--color-border);
          border-radius:var(--radius-md); background:none; cursor:pointer;
          transition:background .1s, border-color .1s;
        }
        .provider-btn:hover { background:var(--color-bg-secondary); }
        .provider-btn.selected { border-color:var(--color-primary); background:#EFF6FF; }
        .provider-icon { font-size:20px; }
        .provider-label { font-size:11px; color:var(--color-text-secondary); text-align:center; }
        .sync-form-row { display:flex; flex-direction:column; gap:var(--space-xs); }
        .sync-label { font-size:12px; font-weight:500; color:var(--color-text-secondary); text-transform:uppercase; letter-spacing:.05em; }
        .sync-input {
          border:1px solid var(--color-border); border-radius:var(--radius-md);
          padding:var(--space-sm) var(--space-md); font-size:14px;
          background:var(--color-bg); color:var(--color-text);
        }
        .sync-input:focus { outline:none; border-color:var(--color-primary); }
        .sync-select {
          border:1px solid var(--color-border); border-radius:var(--radius-md);
          padding:var(--space-sm) var(--space-md); font-size:14px;
          background:var(--color-bg); color:var(--color-text); cursor:pointer;
        }
        .sync-actions { display:flex; gap:var(--space-sm); }
        .sync-test-result { font-size:13px; font-weight:500; }
        .sync-toggle-row {
          display:flex; align-items:center; justify-content:space-between;
          padding:var(--space-md); background:var(--color-bg-secondary);
          border-radius:var(--radius-sm); gap:var(--space-lg);
        }
        .sync-toggle-label { font-size:14px; }
        .sync-toggle-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .toggle {
          width:44px; height:24px; border-radius:12px; border:none; cursor:pointer;
          position:relative; transition:background .2s; flex-shrink:0;
        }
        .toggle-thumb {
          position:absolute; top:2px; width:20px; height:20px; border-radius:50%;
          background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2);
        }
      `}</style>
    </div>
  );
}
