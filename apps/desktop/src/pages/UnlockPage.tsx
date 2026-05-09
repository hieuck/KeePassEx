/**
 * Vault unlock page — shown when vault is locked
 * Enhanced: ZKPV hint display, vault stats, animated lock icon,
 * failed attempt counter, biometric hint
 */
import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';

export function UnlockPage() {
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [failedAttempts, setFailedAttempts] = useState(0);
  const [showHint, setShowHint] = useState(false);
  const { unlockVault, meta, closeVault } = useVaultStore();
  const { t } = useTranslation();
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);

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
      const attempts = failedAttempts + 1;
      setFailedAttempts(attempts);
      setError(t('vault.wrongPassword'));
      setPassword('');
      inputRef.current?.focus();
      // Show hint after 3 failed attempts
      if (attempts >= 3) setShowHint(true);
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
      {/* Background pattern */}
      <div className="unlock-bg" aria-hidden="true" />

      <div className="unlock-card">
        {/* Lock icon with animation */}
        <div className="unlock-hero">
          <div className={`unlock-icon-wrap ${loading ? 'unlocking' : ''}`}>
            <span className="unlock-icon" aria-hidden="true">
              {loading ? '🔓' : failedAttempts > 0 ? '🔒' : '🔐'}
            </span>
          </div>
          <h1 className="unlock-title">KeePassEx</h1>
          {meta && (
            <div className="unlock-vault-info">
              <p className="unlock-vault-name">{meta.name}</p>
              <p className="unlock-vault-stats">
                {t('vault.statistics_entries', { count: meta.entryCount })}
              </p>
            </div>
          )}
        </div>

        {/* Failed attempts warning */}
        {failedAttempts >= 3 && (
          <div className="unlock-warning" role="alert">
            ⚠️ {failedAttempts} {t('auditLog.events.failed_unlock_attempt')}
          </div>
        )}

        <form onSubmit={handleUnlock} className="unlock-form">
          <div className="form-group">
            <label htmlFor="master-password" className="form-label">
              {t('vault.masterPassword')}
            </label>
            <div className="password-input-wrap">
              <input
                id="master-password"
                ref={inputRef}
                type="password"
                className={`form-input ${error ? 'form-input-error' : ''}`}
                value={password}
                onChange={e => setPassword(e.target.value)}
                placeholder="••••••••••••"
                autoComplete="current-password"
                disabled={loading}
                aria-describedby={error ? 'unlock-error' : undefined}
              />
              {password.length > 0 && (
                <span
                  className="password-strength-dot"
                  style={{
                    background:
                      password.length < 8
                        ? '#ef4444'
                        : password.length < 12
                          ? '#f59e0b'
                          : '#16a34a',
                  }}
                  aria-hidden="true"
                />
              )}
            </div>
            {error && (
              <p id="unlock-error" className="form-error" role="alert">
                {error}
              </p>
            )}
          </div>

          <button
            type="submit"
            className="btn btn-primary btn-lg unlock-btn"
            disabled={loading || !password.trim()}
            aria-busy={loading}
          >
            {loading ? (
              <span className="unlock-loading">
                <span className="unlock-spinner" aria-hidden="true" />
                {t('vault.unlocking')}
              </span>
            ) : (
              `🔓 ${t('vault.unlock')}`
            )}
          </button>
        </form>

        {/* ZKPV hint (shown after 3 failed attempts) */}
        {showHint && (
          <div className="unlock-hint" role="note">
            <button
              className="hint-toggle"
              onClick={() => setShowHint(false)}
              aria-label="Dismiss hint"
            >
              💡 {t('zkpv.hint')}
            </button>
          </div>
        )}

        <div className="unlock-footer">
          <button className="btn btn-ghost" onClick={handleClose} disabled={loading}>
            {t('vault.close')}
          </button>
        </div>
      </div>

      <style>{`
        .unlock-page {
          display: flex; align-items: center; justify-content: center;
          height: 100vh; background: var(--color-bg); position: relative; overflow: hidden;
        }
        .unlock-bg {
          position: absolute; inset: 0;
          background: radial-gradient(ellipse at 50% 0%, rgba(37,99,235,0.08) 0%, transparent 70%);
          pointer-events: none;
        }
        .unlock-card {
          width: 100%; max-width: 380px; padding: var(--space-2xl);
          display: flex; flex-direction: column; gap: var(--space-xl);
          background: var(--color-surface); border: 1px solid var(--color-border);
          border-radius: var(--radius-xl);
          box-shadow: 0 20px 60px rgba(0,0,0,0.15);
          position: relative; z-index: 1;
          animation: slideUp 0.3s ease;
        }
        @keyframes slideUp {
          from { transform: translateY(20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
        .unlock-hero { text-align: center; display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); }
        .unlock-icon-wrap { position: relative; }
        .unlock-icon { font-size: 52px; display: block; transition: transform 0.3s; }
        .unlocking .unlock-icon { animation: pulse 0.6s ease infinite; }
        @keyframes pulse { 0%,100% { transform: scale(1); } 50% { transform: scale(1.1); } }
        .unlock-title { font-size: 26px; font-weight: 800; letter-spacing: -0.5px; }
        .unlock-vault-info { display: flex; flex-direction: column; gap: 2px; }
        .unlock-vault-name { color: var(--color-text-secondary); font-size: 14px; font-weight: 500; }
        .unlock-vault-stats { color: var(--color-text-tertiary); font-size: 12px; }
        .unlock-warning {
          background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.3);
          border-radius: var(--radius-md); padding: var(--space-sm) var(--space-md);
          font-size: 12px; color: #ef4444; text-align: center;
        }
        .unlock-form { display: flex; flex-direction: column; gap: var(--space-md); }
        .form-group { display: flex; flex-direction: column; gap: var(--space-xs); }
        .form-label { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: .05em; }
        .password-input-wrap { position: relative; }
        .form-input {
          width: 100%; border: 1px solid var(--color-border); border-radius: var(--radius-md);
          padding: var(--space-sm) var(--space-md); font-size: 16px;
          background: var(--color-bg); color: var(--color-text); outline: none;
          transition: border-color 0.15s, box-shadow 0.15s; box-sizing: border-box;
        }
        .form-input:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(37,99,235,0.1); }
        .form-input-error { border-color: var(--color-danger) !important; }
        .password-strength-dot {
          position: absolute; right: 12px; top: 50%; transform: translateY(-50%);
          width: 8px; height: 8px; border-radius: 50%; transition: background 0.3s;
        }
        .form-error { font-size: 12px; color: var(--color-danger); }
        .unlock-btn { width: 100%; font-size: 15px; font-weight: 600; padding: var(--space-md); }
        .unlock-loading { display: flex; align-items: center; gap: var(--space-sm); justify-content: center; }
        .unlock-spinner {
          width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3);
          border-top-color: white; border-radius: 50%; animation: spin 0.6s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }
        .unlock-hint {
          background: rgba(245,158,11,0.1); border: 1px solid rgba(245,158,11,0.3);
          border-radius: var(--radius-md); padding: var(--space-sm) var(--space-md);
        }
        .hint-toggle {
          background: none; border: none; cursor: pointer; font-size: 12px;
          color: #d97706; width: 100%; text-align: left;
        }
        .unlock-footer { display: flex; justify-content: center; }
        .btn-ghost {
          background: none; border: none; color: var(--color-text-secondary);
          font-size: 13px; cursor: pointer; padding: var(--space-sm);
          text-decoration: underline; border-radius: var(--radius-sm);
        }
        .btn-ghost:hover { color: var(--color-text); }
      `}</style>
    </div>
  );
}
