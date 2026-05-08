/**
 * Welcome page — shown when no vault is open
 * Shows recent vaults and open/create actions
 */
import { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';
import { formatRelativeTime } from '@keepassex/utils';

export function WelcomePage() {
  const { openVault, createVault } = useVaultStore();
  const { settings } = useSettingsStore();
  const { t, i18n } = useTranslation();
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const recentVaults = settings.recentVaults ?? [];

  const [showPasswordDialog, setShowPasswordDialog] = useState(false);
  const [selectedVaultPath, setSelectedVaultPath] = useState<string | null>(null);
  const [passwordInput, setPasswordInput] = useState('');

  const handleOpenVault = async (filePath?: string) => {
    setError(null);
    let path = filePath;

    if (!path) {
      const selected = await open({
        filters: [{ name: 'KeePass Database', extensions: ['kdbx', 'kdb'] }],
        multiple: false,
      });
      if (!selected || typeof selected !== 'string') return;
      path = selected;
    }

    // Show password dialog instead of window.prompt()
    setSelectedVaultPath(path);
    setShowPasswordDialog(true);
  };

  const handlePasswordSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedVaultPath || !passwordInput.trim()) return;

    setLoading(true);
    try {
      await openVault(selectedVaultPath, passwordInput);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
      setPasswordInput('');
    } finally {
      setLoading(false);
      setShowPasswordDialog(false);
      setSelectedVaultPath(null);
      setPasswordInput('');
    }
  };

  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [createForm, setCreateForm] = useState({ name: '', password: '', confirmPassword: '' });

  const handleCreateVault = async () => {
    setError(null);
    setCreateForm({ name: t('app.name'), password: '', confirmPassword: '' });
    setShowCreateDialog(true);
  };

  const handleCreateSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!createForm.name.trim() || !createForm.password) return;

    if (createForm.password !== createForm.confirmPassword) {
      setError(t('vault.passwordsDoNotMatch'));
      return;
    }

    const filePath = await save({
      filters: [{ name: 'KeePass Database', extensions: ['kdbx'] }],
      defaultPath: `${createForm.name.trim()}.kdbx`,
    });
    if (!filePath) return;

    setLoading(true);
    try {
      await createVault(filePath, createForm.name.trim(), createForm.password);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
      setShowCreateDialog(false);
      setCreateForm({ name: '', password: '', confirmPassword: '' });
    }
  };

  const formatVaultTime = (dateStr: string): string => formatRelativeTime(dateStr, i18n.language);

  return (
    <div className="welcome-page">
      {/* Password Dialog */}
      {showPasswordDialog && (
        <div className="dialog-overlay" onClick={() => setShowPasswordDialog(false)}>
          <div className="dialog" onClick={e => e.stopPropagation()}>
            <h3 className="dialog-title">{t('vault.masterPassword')}</h3>
            <form onSubmit={handlePasswordSubmit} className="dialog-form">
              <input
                type="password"
                className="form-input"
                value={passwordInput}
                onChange={e => setPasswordInput(e.target.value)}
                placeholder={t('vault.masterPassword')}
                autoFocus
                disabled={loading}
              />
              <div className="dialog-actions">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => {
                    setShowPasswordDialog(false);
                    setPasswordInput('');
                  }}
                  disabled={loading}
                >
                  {t('common.cancel')}
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={loading || !passwordInput.trim()}
                >
                  {loading ? t('vault.unlocking') : t('vault.unlock')}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Create Vault Dialog */}
      {showCreateDialog && (
        <div className="dialog-overlay" onClick={() => setShowCreateDialog(false)}>
          <div className="dialog" onClick={e => e.stopPropagation()}>
            <h3 className="dialog-title">{t('vault.create')}</h3>
            <form onSubmit={handleCreateSubmit} className="dialog-form">
              <div className="form-group">
                <label htmlFor="vault-name" className="form-label">
                  {t('vault.name')}
                </label>
                <input
                  id="vault-name"
                  type="text"
                  className="form-input"
                  value={createForm.name}
                  onChange={e => setCreateForm(f => ({ ...f, name: e.target.value }))}
                  placeholder={t('vault.name')}
                  autoFocus
                  disabled={loading}
                />
              </div>
              <div className="form-group">
                <label htmlFor="vault-password" className="form-label">
                  {t('vault.masterPassword')}
                </label>
                <input
                  id="vault-password"
                  type="password"
                  className="form-input"
                  value={createForm.password}
                  onChange={e => setCreateForm(f => ({ ...f, password: e.target.value }))}
                  placeholder={t('vault.masterPassword')}
                  disabled={loading}
                />
              </div>
              <div className="form-group">
                <label htmlFor="vault-confirm" className="form-label">
                  {t('vault.confirmPassword')}
                </label>
                <input
                  id="vault-confirm"
                  type="password"
                  className="form-input"
                  value={createForm.confirmPassword}
                  onChange={e => setCreateForm(f => ({ ...f, confirmPassword: e.target.value }))}
                  placeholder={t('vault.confirmPassword')}
                  disabled={loading}
                />
              </div>
              <div className="dialog-actions">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => {
                    setShowCreateDialog(false);
                    setCreateForm({ name: '', password: '', confirmPassword: '' });
                  }}
                  disabled={loading}
                >
                  {t('common.cancel')}
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={
                    loading ||
                    !createForm.name.trim() ||
                    !createForm.password ||
                    !createForm.confirmPassword
                  }
                >
                  {loading ? t('vault.saving') : t('common.create')}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Hero */}
      <div className="welcome-hero">
        <div className="app-logo" aria-hidden="true">
          🔐
        </div>
        <h1 className="app-name">{t('app.name')}</h1>
        <p className="app-tagline">{t('app.tagline')}</p>
      </div>

      {/* Main actions */}
      <div className="welcome-actions">
        <button
          className="btn btn-primary btn-lg"
          onClick={() => handleOpenVault()}
          disabled={loading}
          aria-label={t('vault.open')}
        >
          {loading ? '⏳' : '📂'} {t('vault.open')}
        </button>

        <button
          className="btn btn-secondary btn-lg"
          onClick={handleCreateVault}
          disabled={loading}
          aria-label={t('vault.create')}
        >
          ✨ {t('vault.create')}
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="error-banner" role="alert">
          ⚠️ {error}
        </div>
      )}

      {/* Recent vaults */}
      {recentVaults.length > 0 && (
        <div className="recent-vaults">
          <h2 className="recent-title">🕐 {t('vault.recentVaults')}</h2>
          <div className="recent-list" role="list">
            {recentVaults.slice(0, 5).map(vault => (
              <button
                key={vault.path}
                className="recent-item"
                onClick={() => handleOpenVault(vault.path)}
                disabled={loading}
                role="listitem"
                aria-label={`${t('vault.open')} ${vault.name}`}
              >
                <span className="recent-icon" aria-hidden="true">
                  🔐
                </span>
                <div className="recent-info">
                  <span className="recent-name">{vault.name}</span>
                  <span className="recent-path" title={vault.path}>
                    {vault.path.length > 50 ? '...' + vault.path.slice(-47) : vault.path}
                  </span>
                </div>
                <span className="recent-time">{formatVaultTime(vault.lastOpened)}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Feature highlights */}
      <div className="welcome-features" aria-label="Key features">
        {[
          { icon: '🔒', label: t('hardwareKey.title') },
          { icon: '📱', label: t('settings.general') },
          { icon: '🔑', label: t('passkey.title') },
          { icon: '🛡️', label: t('breach.title') },
          { icon: '🔄', label: t('sync.title') },
          { icon: '🌐', label: t('browserExtension.fill') },
        ].map((f, i) => (
          <div key={i} className="feature-item">
            <span aria-hidden="true">{f.icon}</span>
            <span>{f.label}</span>
          </div>
        ))}
      </div>

      <style>{`
        .welcome-page {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 100vh;
          gap: var(--space-xl);
          padding: var(--space-2xl);
          background: var(--color-bg);
          overflow-y: auto;
        }
        .welcome-hero { text-align: center; }
        .app-logo { font-size: 72px; line-height: 1; margin-bottom: var(--space-md); }
        .app-name { font-size: 36px; font-weight: 700; color: var(--color-text); }
        .app-tagline { font-size: 16px; color: var(--color-text-secondary); margin-top: var(--space-sm); }
        .welcome-actions {
          display: flex;
          flex-direction: column;
          gap: var(--space-md);
          width: 100%;
          max-width: 340px;
        }
        .recent-vaults {
          width: 100%;
          max-width: 480px;
        }
        .recent-title {
          font-size: 13px;
          font-weight: 600;
          color: var(--color-text-secondary);
          text-transform: uppercase;
          letter-spacing: 0.08em;
          margin-bottom: var(--space-sm);
          padding: 0 var(--space-xs);
        }
        .recent-list {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }
        .recent-item {
          display: flex;
          align-items: center;
          gap: var(--space-md);
          padding: var(--space-sm) var(--space-md);
          background: var(--color-bg-secondary);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-md);
          cursor: pointer;
          text-align: left;
          transition: background 0.1s, border-color 0.1s;
          width: 100%;
        }
        .recent-item:hover:not(:disabled) {
          background: var(--color-bg-tertiary);
          border-color: var(--color-primary);
        }
        .recent-item:disabled { opacity: 0.5; cursor: not-allowed; }
        .recent-icon { font-size: 20px; flex-shrink: 0; }
        .recent-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
        .recent-name { font-size: 14px; font-weight: 500; color: var(--color-text); }
        .recent-path {
          font-size: 11px;
          color: var(--color-text-tertiary);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .recent-time { font-size: 12px; color: var(--color-text-tertiary); flex-shrink: 0; }
        .welcome-features {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: var(--space-sm);
          max-width: 480px;
          width: 100%;
        }
        .feature-item {
          display: flex;
          align-items: center;
          gap: var(--space-sm);
          font-size: 12px;
          color: var(--color-text-secondary);
          padding: var(--space-sm);
          background: var(--color-bg-secondary);
          border-radius: var(--radius-sm);
        }
        @media (max-width: 600px) {
          .welcome-features { grid-template-columns: repeat(2, 1fr); }
        }
        .dialog-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .dialog {
          background: var(--color-surface);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-lg);
          padding: var(--space-xl);
          width: 100%;
          max-width: 400px;
          box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }
        .dialog-title {
          font-size: 18px;
          font-weight: 600;
          margin-bottom: var(--space-lg);
        }
        .dialog-form {
          display: flex;
          flex-direction: column;
          gap: var(--space-md);
        }
        .form-group {
          display: flex;
          flex-direction: column;
          gap: var(--space-xs);
        }
        .form-label {
          font-size: 13px;
          font-weight: 500;
          color: var(--color-text-secondary);
        }
        .form-input {
          border: 1px solid var(--color-border);
          border-radius: var(--radius-md);
          padding: var(--space-sm) var(--space-md);
          font-size: 15px;
          background: var(--color-bg);
          color: var(--color-text);
          outline: none;
          transition: border-color 0.15s;
        }
        .form-input:focus {
          border-color: var(--color-primary);
        }
        .dialog-actions {
          display: flex;
          gap: var(--space-sm);
          justify-content: flex-end;
          margin-top: var(--space-md);
        }
      `}</style>
    </div>
  );
}
