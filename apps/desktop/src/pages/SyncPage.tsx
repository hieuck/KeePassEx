/**
 * Sync configuration & status page — desktop
 * Supports all providers including KeePassEx self-hosted server
 */
import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useSyncStore } from '../store/sync';

type SyncProvider =
  | 'local'
  | 'webdav'
  | 'icloud'
  | 'gdrive'
  | 'onedrive'
  | 'dropbox'
  | 's3'
  | 'sftp'
  | 'keepassex_server';

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

interface ServerAuthResult {
  access_token: string;
  refresh_token: string;
  user_id: string;
  email: string;
}

const PROVIDERS: {
  value: SyncProvider;
  label: string;
  icon: string;
  needsAuth: boolean;
  isServer?: boolean;
}[] = [
  {
    value: 'keepassex_server',
    label: 'KeePassEx Server',
    icon: '🔐',
    needsAuth: true,
    isServer: true,
  },
  { value: 'local', label: 'Local Folder', icon: '📁', needsAuth: false },
  { value: 'webdav', label: 'WebDAV', icon: '🌐', needsAuth: true },
  { value: 'icloud', label: 'iCloud Drive', icon: '☁️', needsAuth: false },
  { value: 'gdrive', label: 'Google Drive', icon: '🔵', needsAuth: true },
  { value: 'onedrive', label: 'OneDrive', icon: '🔷', needsAuth: true },
  { value: 'dropbox', label: 'Dropbox', icon: '📦', needsAuth: true },
  { value: 's3', label: 'Amazon S3', icon: '🟠', needsAuth: true },
  { value: 'sftp', label: 'SFTP', icon: '🔒', needsAuth: true },
];

export function SyncPage() {
  const { t } = useTranslation();

  const [provider, setProvider] = useState<SyncProvider>('keepassex_server');
  const [remotePath, setRemotePath] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [serverUrl, setServerUrl] = useState('');
  const [autoSync, setAutoSync] = useState(false);
  const [syncInterval, setSyncInterval] = useState(300);
  const [conflict, setConflict] = useState('merge');
  const [testResult, setTestResult] = useState<string | null>(null);

  // KeePassEx Server auth state
  const [serverEmail, setServerEmail] = useState('');
  const [serverPassword, setServerPassword] = useState('');
  const [serverAuthMode, setServerAuthMode] = useState<'login' | 'register'>('login');
  const [serverToken, setServerToken] = useState<string | null>(null);
  const [serverAuthError, setServerAuthError] = useState<string | null>(null);
  const [serverAuthLoading, setServerAuthLoading] = useState(false);

  const { data: status } = useQuery({
    queryKey: ['sync-status'],
    queryFn: () => invoke<SyncStatus>('get_sync_status'),
    refetchInterval: 30_000,
  });

  const configureMutation = useMutation({
    mutationFn: () =>
      invoke('configure_sync', {
        args: {
          provider,
          remote_path: provider === 'keepassex_server' ? serverUrl : remotePath,
          auto_sync: autoSync,
          sync_interval_seconds: syncInterval,
          conflict_resolution: conflict,
          username: username || null,
          password: password || null,
          server_url: serverUrl || null,
          // For KeePassEx Server, pass the JWT token as the credential token
          token: serverToken || null,
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
        provider,
        remote_path: provider === 'keepassex_server' ? serverUrl : remotePath,
        username: username || null,
        password: password || null,
        token: serverToken || null,
      });
      setTestResult(
        ok ? `✅ ${t('syncExt.testSuccess')}` : `❌ ${t('syncExt.testFailed', { error: '' })}`
      );
    } catch (e: unknown) {
      setTestResult(`❌ ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleServerAuth = async () => {
    if (!serverUrl.trim() || !serverEmail.trim() || !serverPassword.trim()) return;
    setServerAuthLoading(true);
    setServerAuthError(null);
    try {
      const endpoint =
        serverAuthMode === 'login'
          ? `${serverUrl.trim().replace(/\/$/, '')}/api/v1/auth/login`
          : `${serverUrl.trim().replace(/\/$/, '')}/api/v1/auth/register`;

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: serverEmail, password: serverPassword }),
      });

      if (!response.ok) {
        const err = await response.json().catch(() => ({ error: 'Unknown error' }));
        throw new Error(err.error || `HTTP ${response.status}`);
      }

      const data: ServerAuthResult = await response.json();
      setServerToken(data.access_token);
      setServerAuthError(null);
    } catch (e: unknown) {
      setServerAuthError(e instanceof Error ? e.message : String(e));
    } finally {
      setServerAuthLoading(false);
    }
  };

  const selectedProvider = PROVIDERS.find(p => p.value === provider)!;
  const isServerProvider = provider === 'keepassex_server';

  return (
    <div className="sync-page">
      <div className="sync-header">
        <h2>🔄 {t('sync.title')}</h2>
        {status?.configured && (
          <button
            className="btn btn-primary"
            onClick={() => syncNowMutation.mutate()}
            disabled={syncNowMutation.isPending}
          >
            {syncNowMutation.isPending ? t('sync.syncing') : `🔄 ${t('sync.syncNow')}`}
          </button>
        )}
      </div>

      <div className="sync-content">
        {/* Status card */}
        {status && (
          <div className="sync-status-card">
            <div className="sync-status-row">
              <span
                className="sync-status-dot"
                style={{
                  background: status.configured
                    ? 'var(--color-success)'
                    : 'var(--color-text-tertiary)',
                }}
              />
              <span className="sync-status-label">
                {status.configured ? `${t('sync.provider')}: ${status.provider}` : t('common.none')}
              </span>
            </div>
            {status.lastSync && (
              <p className="sync-last">{t('sync.lastSync', { time: status.lastSync })}</p>
            )}
            {syncNowMutation.data && (
              <p
                className="sync-result"
                style={{
                  color:
                    syncNowMutation.data.status === 'success'
                      ? 'var(--color-success)'
                      : 'var(--color-danger)',
                }}
              >
                {syncNowMutation.data.status === 'success'
                  ? `✅ ${t('sync.syncSuccess')}`
                  : `❌ ${syncNowMutation.data.error}`}
              </p>
            )}
          </div>
        )}

        {/* Provider selection */}
        <section className="sync-section">
          <h3 className="sync-section-title">{t('sync.provider')}</h3>
          <div className="provider-grid" role="radiogroup" aria-label={t('sync.provider')}>
            {PROVIDERS.map(p => (
              <button
                key={p.value}
                role="radio"
                aria-checked={provider === p.value}
                className={`provider-btn ${provider === p.value ? 'selected' : ''} ${p.isServer ? 'provider-btn-featured' : ''}`}
                onClick={() => setProvider(p.value)}
              >
                <span className="provider-icon">{p.icon}</span>
                <span className="provider-label">{p.label}</span>
                {p.isServer && <span className="provider-badge">Self-hosted</span>}
              </button>
            ))}
          </div>
        </section>

        {/* KeePassEx Server — dedicated auth flow */}
        {isServerProvider && (
          <section className="sync-section">
            <h3 className="sync-section-title">🔐 {t('server.title')}</h3>
            <p className="sync-desc">{t('server.zeroKnowledgeDesc')}</p>

            <div className="sync-form-row">
              <label className="sync-label" htmlFor="kpx-server-url">
                {t('server.serverUrl')}
              </label>
              <input
                id="kpx-server-url"
                type="url"
                className="sync-input"
                value={serverUrl}
                onChange={e => setServerUrl(e.target.value)}
                placeholder={t('server.serverUrlPlaceholder')}
              />
            </div>

            {!serverToken ? (
              <>
                {/* Auth mode toggle */}
                <div className="server-auth-tabs">
                  <button
                    className={`server-auth-tab ${serverAuthMode === 'login' ? 'active' : ''}`}
                    onClick={() => setServerAuthMode('login')}
                  >
                    {t('server.login')}
                  </button>
                  <button
                    className={`server-auth-tab ${serverAuthMode === 'register' ? 'active' : ''}`}
                    onClick={() => setServerAuthMode('register')}
                  >
                    {t('server.register')}
                  </button>
                </div>

                <div className="sync-form-row">
                  <label className="sync-label" htmlFor="kpx-email">
                    {t('server.email')}
                  </label>
                  <input
                    id="kpx-email"
                    type="email"
                    className="sync-input"
                    value={serverEmail}
                    onChange={e => setServerEmail(e.target.value)}
                    autoComplete="email"
                  />
                </div>

                <div className="sync-form-row">
                  <label className="sync-label" htmlFor="kpx-server-pass">
                    {t('server.password')}
                  </label>
                  <input
                    id="kpx-server-pass"
                    type="password"
                    className="sync-input"
                    value={serverPassword}
                    onChange={e => setServerPassword(e.target.value)}
                    autoComplete="current-password"
                  />
                </div>

                {serverAuthError && <p className="sync-error">{serverAuthError}</p>}

                <button
                  className="btn btn-primary"
                  onClick={handleServerAuth}
                  disabled={serverAuthLoading || !serverUrl || !serverEmail || !serverPassword}
                  aria-busy={serverAuthLoading}
                >
                  {serverAuthLoading
                    ? t('common.loading')
                    : serverAuthMode === 'login'
                      ? t('server.login')
                      : t('server.register')}
                </button>
              </>
            ) : (
              <div className="server-connected-card">
                <span className="server-connected-icon">✅</span>
                <div>
                  <p className="server-connected-label">
                    {t('server.connected', { url: serverUrl })}
                  </p>
                  <p className="server-connected-sub">{t('server.zeroKnowledge')}</p>
                </div>
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={() => {
                    setServerToken(null);
                    setServerEmail('');
                    setServerPassword('');
                  }}
                >
                  {t('server.disconnect')}
                </button>
              </div>
            )}
          </section>
        )}

        {/* Standard provider config */}
        {!isServerProvider && (
          <section className="sync-section">
            <h3 className="sync-section-title">{t('sync.remotePath')}</h3>
            {selectedProvider.needsAuth && (
              <div className="sync-form-row">
                <label className="sync-label" htmlFor="server-url">
                  {t('syncExt.webdavUrl')}
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
                {provider === 'local' ? t('syncExt.localPath') : t('sync.remotePath')}
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
                    {t('syncExt.webdavUsername')}
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
                    {t('syncExt.webdavPassword')}
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
                🔌 {t('syncExt.testConnection')}
              </button>
            </div>
            {testResult && (
              <p
                className="sync-test-result"
                style={{
                  color: testResult.startsWith('✅')
                    ? 'var(--color-success)'
                    : 'var(--color-danger)',
                }}
              >
                {testResult}
              </p>
            )}
          </section>
        )}

        {/* Options */}
        <section className="sync-section">
          <h3 className="sync-section-title">{t('settings.advanced')}</h3>

          <div className="sync-toggle-row">
            <div>
              <p className="sync-toggle-label">{t('sync.autoSync')}</p>
              <p className="sync-toggle-desc">{t('sync.syncInterval')}</p>
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
                {t('sync.syncInterval')}
              </label>
              <select
                id="sync-interval"
                className="sync-select"
                value={syncInterval}
                onChange={e => setSyncInterval(Number(e.target.value))}
              >
                <option value={60}>1 min</option>
                <option value={300}>5 min</option>
                <option value={900}>15 min</option>
                <option value={3600}>1 hr</option>
              </select>
            </div>
          )}

          <div className="sync-form-row">
            <label className="sync-label" htmlFor="conflict-res">
              {t('importExport.conflictStrategy')}
            </label>
            <select
              id="conflict-res"
              className="sync-select"
              value={conflict}
              onChange={e => setConflict(e.target.value)}
            >
              <option value="merge">{t('sync.merge')}</option>
              <option value="keepLocal">{t('sync.keepLocal')}</option>
              <option value="keepRemote">{t('sync.keepRemote')}</option>
              <option value="askUser">{t('common.confirm')}</option>
            </select>
          </div>
        </section>

        {/* Save */}
        <button
          className="btn btn-primary"
          onClick={() => configureMutation.mutate()}
          disabled={
            configureMutation.isPending || (isServerProvider ? !serverToken : !remotePath.trim())
          }
        >
          {configureMutation.isPending ? t('common.loading') : `💾 ${t('common.save')}`}
        </button>
        {configureMutation.isSuccess && (
          <p style={{ color: 'var(--color-success)', fontSize: 13 }}>✅ {t('sync.syncSuccess')}</p>
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
        .sync-desc { font-size:13px; color:var(--color-text-secondary); }
        .sync-error { font-size:13px; color:var(--color-danger); }
        .provider-grid {
          display:grid; grid-template-columns:repeat(3,1fr); gap:var(--space-sm);
        }
        .provider-btn {
          display:flex; flex-direction:column; align-items:center; gap:4px;
          padding:var(--space-sm); border:1px solid var(--color-border);
          border-radius:var(--radius-md); background:none; cursor:pointer;
          transition:background .1s, border-color .1s; position:relative;
        }
        .provider-btn:hover { background:var(--color-bg-secondary); }
        .provider-btn.selected { border-color:var(--color-primary); background:#EFF6FF; }
        .provider-btn-featured { border-color:var(--color-primary); }
        .provider-icon { font-size:20px; }
        .provider-label { font-size:11px; color:var(--color-text-secondary); text-align:center; }
        .provider-badge {
          font-size:9px; background:var(--color-primary); color:white;
          padding:1px 5px; border-radius:4px; font-weight:600;
        }
        .server-auth-tabs { display:flex; gap:0; border:1px solid var(--color-border); border-radius:var(--radius-md); overflow:hidden; }
        .server-auth-tab {
          flex:1; padding:var(--space-sm); border:none; background:none;
          cursor:pointer; font-size:13px; color:var(--color-text-secondary);
        }
        .server-auth-tab.active { background:var(--color-primary); color:white; font-weight:600; }
        .server-connected-card {
          display:flex; align-items:center; gap:var(--space-md);
          padding:var(--space-md); background:rgba(16,185,129,.08);
          border:1px solid rgba(16,185,129,.3); border-radius:var(--radius-md);
        }
        .server-connected-icon { font-size:24px; }
        .server-connected-label { font-size:14px; font-weight:600; }
        .server-connected-sub { font-size:12px; color:var(--color-text-secondary); }
        .btn-sm { padding:4px 10px; font-size:12px; }
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
import { useQuery, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useSyncStore } from '../store/sync';

type SyncProvider =
  | 'local'
  | 'webdav'
  | 'icloud'
  | 'gdrive'
  | 'onedrive'
  | 'dropbox'
  | 's3'
  | 'sftp';

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

const PROVIDERS: { value: SyncProvider; label: string; icon: string; needsAuth: boolean }[] = [
  { value: 'local', label: 'Local Folder', icon: '📁', needsAuth: false },
  { value: 'webdav', label: 'WebDAV', icon: '🌐', needsAuth: true },
  { value: 'icloud', label: 'iCloud Drive', icon: '☁️', needsAuth: false },
  { value: 'gdrive', label: 'Google Drive', icon: '🔵', needsAuth: true },
  { value: 'onedrive', label: 'OneDrive', icon: '🔷', needsAuth: true },
  { value: 'dropbox', label: 'Dropbox', icon: '📦', needsAuth: true },
  { value: 's3', label: 'Amazon S3', icon: '🟠', needsAuth: true },
  { value: 'sftp', label: 'SFTP', icon: '🔒', needsAuth: true },
];

export function SyncPage() {
  const { t } = useTranslation();

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
    mutationFn: () =>
      invoke('configure_sync', {
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
        provider,
        remote_path: remotePath,
        username: username || null,
        password: password || null,
      });
      setTestResult(
        ok ? `✅ ${t('syncExt.testSuccess')}` : `❌ ${t('syncExt.testFailed', { error: '' })}`
      );
    } catch (e: unknown) {
      setTestResult(`❌ ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const selectedProvider = PROVIDERS.find(p => p.value === provider)!;

  return (
    <div className="sync-page">
      <div className="sync-header">
        <h2>🔄 {t('sync.title')}</h2>
        {status?.configured && (
          <button
            className="btn btn-primary"
            onClick={() => syncNowMutation.mutate()}
            disabled={syncNowMutation.isPending}
          >
            {syncNowMutation.isPending ? t('sync.syncing') : `🔄 ${t('sync.syncNow')}`}
          </button>
        )}
      </div>

      <div className="sync-content">
        {/* Status card */}
        {status && (
          <div className="sync-status-card">
            <div className="sync-status-row">
              <span
                className="sync-status-dot"
                style={{
                  background: status.configured
                    ? 'var(--color-success)'
                    : 'var(--color-text-tertiary)',
                }}
              />
              <span className="sync-status-label">
                {status.configured ? `${t('sync.provider')}: ${status.provider}` : t('common.none')}
              </span>
            </div>
            {status.lastSync && (
              <p className="sync-last">{t('sync.lastSync', { time: status.lastSync })}</p>
            )}
            {syncNowMutation.data && (
              <p
                className="sync-result"
                style={{
                  color:
                    syncNowMutation.data.status === 'success'
                      ? 'var(--color-success)'
                      : 'var(--color-danger)',
                }}
              >
                {syncNowMutation.data.status === 'success'
                  ? `✅ ${t('sync.syncSuccess')}`
                  : `❌ ${syncNowMutation.data.error}`}
              </p>
            )}
          </div>
        )}

        {/* Provider selection */}
        <section className="sync-section">
          <h3 className="sync-section-title">{t('sync.provider')}</h3>
          <div className="provider-grid" role="radiogroup" aria-label={t('sync.provider')}>
            {PROVIDERS.map(p => (
              <button
                key={p.value}
                role="radio"
                aria-checked={provider === p.value}
                className={`provider-btn ${provider === p.value ? 'selected' : ''}`}
                onClick={() => setProvider(p.value)}
              >
                <span className="provider-icon">{p.icon}</span>
                <span className="provider-label">{p.label}</span>
              </button>
            ))}
          </div>
        </section>

        {/* Remote path */}
        <section className="sync-section">
          <h3 className="sync-section-title">{t('sync.remotePath')}</h3>
          {selectedProvider.needsAuth && (
            <div className="sync-form-row">
              <label className="sync-label" htmlFor="server-url">
                {t('syncExt.webdavUrl')}
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
              {provider === 'local' ? t('syncExt.localPath') : t('sync.remotePath')}
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
                  {t('syncExt.webdavUsername')}
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
                  {t('syncExt.webdavPassword')}
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
              🔌 {t('syncExt.testConnection')}
            </button>
          </div>
          {testResult && (
            <p
              className="sync-test-result"
              style={{
                color: testResult.startsWith('✅') ? 'var(--color-success)' : 'var(--color-danger)',
              }}
            >
              {testResult}
            </p>
          )}
        </section>

        {/* Options */}
        <section className="sync-section">
          <h3 className="sync-section-title">{t('settings.advanced')}</h3>

          <div className="sync-toggle-row">
            <div>
              <p className="sync-toggle-label">{t('sync.autoSync')}</p>
              <p className="sync-toggle-desc">{t('sync.syncInterval')}</p>
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
                {t('sync.syncInterval')}
              </label>
              <select
                id="sync-interval"
                className="sync-select"
                value={interval}
                onChange={e => setInterval(Number(e.target.value))}
              >
                <option value={60}>1 min</option>
                <option value={300}>5 min</option>
                <option value={900}>15 min</option>
                <option value={3600}>1 hr</option>
              </select>
            </div>
          )}

          <div className="sync-form-row">
            <label className="sync-label" htmlFor="conflict-res">
              {t('importExport.conflictStrategy')}
            </label>
            <select
              id="conflict-res"
              className="sync-select"
              value={conflict}
              onChange={e => setConflict(e.target.value)}
            >
              <option value="merge">{t('sync.merge')}</option>
              <option value="keepLocal">{t('sync.keepLocal')}</option>
              <option value="keepRemote">{t('sync.keepRemote')}</option>
              <option value="askUser">{t('common.confirm')}</option>
            </select>
          </div>
        </section>

        {/* Save */}
        <button
          className="btn btn-primary"
          onClick={() => configureMutation.mutate()}
          disabled={configureMutation.isPending || !remotePath.trim()}
        >
          {configureMutation.isPending ? t('common.loading') : `💾 ${t('common.save')}`}
        </button>
        {configureMutation.isSuccess && (
          <p style={{ color: 'var(--color-success)', fontSize: 13 }}>✅ {t('sync.syncSuccess')}</p>
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
