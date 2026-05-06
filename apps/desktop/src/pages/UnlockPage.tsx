/**
 * Vault unlock page — shown when vault is locked
 */
import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';

export function UnlockPage() {
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { unlockVault, meta, closeVault } = useVaultStore();
  const { settings } = useSettingsStore();
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);
  const isVi = settings.language === 'vi';

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!password.trim()) return;

    setLoading(true);
    setError(null);
    try {
      await unlockVault(password);
      navigate('/vault');
    } catch {
      setError(isVi ? 'Sai mật khẩu chính. Vui lòng thử lại.' : 'Wrong master password. Please try again.');
      setPassword('');
      inputRef.current?.focus();
    } finally {
      setLoading(false);
    }
  };

  const handleClose = async () => {
    await closeVault();
    navigate('/');
  };

  return (
    <div className="unlock-page">
      <div className="unlock-card">
        <div className="unlock-hero">
          <span className="unlock-icon">🔒</span>
          <h1 className="unlock-title">KeePassEx</h1>
          {meta && (
            <p className="unlock-vault-name">{meta.name}</p>
          )}
        </div>

        <form onSubmit={handleUnlock} className="unlock-form">
          <div className="form-group">
            <label htmlFor="master-password" className="form-label">
              {isVi ? 'Mật khẩu chính' : 'Master Password'}
            </label>
            <input
              id="master-password"
              ref={inputRef}
              type="password"
              className={`form-input ${error ? 'form-input-error' : ''}`}
              value={password}
              onChange={e => setPassword(e.target.value)}
              placeholder={isVi ? 'Nhập mật khẩu chính...' : 'Enter master password...'}
              autoComplete="current-password"
              disabled={loading}
              aria-describedby={error ? 'unlock-error' : undefined}
            />
            {error && (
              <p id="unlock-error" className="form-error" role="alert">
                {error}
              </p>
            )}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-lg"
            disabled={loading || !password.trim()}
          >
            {loading ? (isVi ? 'Đang mở khóa...' : 'Unlocking...') : (isVi ? '🔓 Mở khóa' : '🔓 Unlock')}
          </button>
        </form>

        <button
          className="btn btn-ghost"
          onClick={handleClose}
          disabled={loading}
        >
          {isVi ? 'Đóng kho' : 'Close Vault'}
        </button>
      </div>

      <style>{`
        .unlock-page {
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100vh;
          background: var(--color-bg);
        }
        .unlock-card {
          width: 100%;
          max-width: 360px;
          padding: var(--space-2xl);
          display: flex;
          flex-direction: column;
          gap: var(--space-xl);
          background: var(--color-surface);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-lg);
        }
        .unlock-hero {
          text-align: center;
        }
        .unlock-icon { font-size: 48px; display: block; margin-bottom: var(--space-md); }
        .unlock-title { font-size: 24px; font-weight: 700; }
        .unlock-vault-name { color: var(--color-text-secondary); font-size: 14px; margin-top: 4px; }
        .unlock-form { display: flex; flex-direction: column; gap: var(--space-md); }
        .form-group { display: flex; flex-direction: column; gap: var(--space-xs); }
        .form-label { font-size: 13px; font-weight: 500; color: var(--color-text-secondary); }
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
        .form-input:focus { border-color: var(--color-primary); }
        .form-input-error { border-color: var(--color-danger) !important; }
        .form-error { font-size: 12px; color: var(--color-danger); }
        .btn-ghost {
          background: none;
          border: none;
          color: var(--color-text-secondary);
          font-size: 13px;
          cursor: pointer;
          padding: var(--space-sm);
          text-align: center;
          text-decoration: underline;
        }
        .btn-ghost:hover { color: var(--color-text); }
      `}</style>
    </div>
  );
}
