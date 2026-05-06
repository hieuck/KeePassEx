/**
 * SecurityPage — Quantum-Resistant Encryption settings
 *
 * Allows users to enable/disable PQC (Post-Quantum Cryptography) for their vault.
 * Uses the X25519 + Kyber-768 hybrid mode implemented in packages/core/src/crypto/pqc.rs
 * and integrated via packages/core/src/kdbx/pqc_header.rs.
 *
 * No competitor (KeePass, KeePassXC, Keepassium, KeePass2Android) has this feature.
 */
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';

type PqcStatus = 'classical' | 'hybrid' | 'unknown';

export function SecurityPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { meta } = useVaultStore();

  const [pqcStatus, setPqcStatus] = useState<PqcStatus>('unknown');
  const [migrating, setMigrating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleEnablePqc = async () => {
    setMigrating(true);
    setError(null);
    setSuccess(null);
    try {
      await invoke('migrate_to_pqc');
      setPqcStatus('hybrid');
      setSuccess(t('quantumResistant.migrateSuccess'));
    } catch (e) {
      setError(t('quantumResistant.migrateFailed', { error: String(e) }));
    } finally {
      setMigrating(false);
    }
  };

  const handleDisablePqc = async () => {
    setMigrating(true);
    setError(null);
    setSuccess(null);
    try {
      await invoke('downgrade_from_pqc');
      setPqcStatus('classical');
      setSuccess(t('quantumResistant.statusDisabled'));
    } catch (e) {
      setError(t('quantumResistant.migrateFailed', { error: String(e) }));
    } finally {
      setMigrating(false);
    }
  };

  const isHybrid = pqcStatus === 'hybrid';

  return (
    <div className="security-page">
      <div className="security-header">
        <button
          className="back-btn"
          onClick={() => navigate('/settings')}
          aria-label={t('common.back')}
        >
          ‹ {t('common.back')}
        </button>
        <h2>🔐 {t('quantumResistant.title')}</h2>
        <p className="security-subtitle">{t('quantumResistant.subtitle')}</p>
      </div>

      <div className="security-content">
        {/* Status card */}
        <div className={`pqc-status-card ${isHybrid ? 'pqc-enabled' : 'pqc-disabled'}`}>
          <div className="pqc-status-icon">{isHybrid ? '🛡️' : '🔓'}</div>
          <div className="pqc-status-info">
            <p className="pqc-status-label">
              {isHybrid
                ? t('quantumResistant.statusEnabled')
                : t('quantumResistant.statusDisabled')}
            </p>
            <p className="pqc-status-algo">
              {t('quantumResistant.algorithm')}:{' '}
              {isHybrid
                ? t('quantumResistant.algorithmHybrid')
                : t('quantumResistant.algorithmClassical')}
            </p>
          </div>
        </div>

        {/* Description */}
        <div className="pqc-description">
          <p>{t('quantumResistant.description')}</p>
        </div>

        {/* Why Kyber */}
        <div className="pqc-info-card">
          <h3>❓ {t('quantumResistant.whyKyber')}</h3>
          <p>{t('quantumResistant.whyKyberDesc')}</p>
        </div>

        {/* Performance note */}
        <div className="pqc-info-card pqc-warning">
          <h3>⚡ {t('quantumResistant.performanceNote')}</h3>
          <p>{t('quantumResistant.performanceNoteDesc')}</p>
        </div>

        {/* Compatibility note */}
        <div className="pqc-info-card pqc-warning">
          <h3>⚠️ {t('quantumResistant.compatibilityNote')}</h3>
          <p>{t('quantumResistant.compatibilityNoteDesc')}</p>
        </div>

        {/* Error / success feedback */}
        {error && (
          <div className="pqc-feedback pqc-error" role="alert">
            {error}
          </div>
        )}
        {success && (
          <div className="pqc-feedback pqc-success" role="status">
            {success}
          </div>
        )}

        {/* Action button */}
        {meta && (
          <div className="pqc-actions">
            {!isHybrid ? (
              <button
                className="btn btn-primary pqc-action-btn"
                onClick={handleEnablePqc}
                disabled={migrating}
                aria-busy={migrating}
              >
                {migrating ? t('quantumResistant.migrating') : t('quantumResistant.enable')}
              </button>
            ) : (
              <button
                className="btn btn-secondary pqc-action-btn"
                onClick={handleDisablePqc}
                disabled={migrating}
                aria-busy={migrating}
              >
                {migrating ? t('quantumResistant.migrating') : t('quantumResistant.disable')}
              </button>
            )}
          </div>
        )}

        {!meta && <p className="pqc-no-vault">{t('errors.vaultLocked')}</p>}
      </div>

      <style>{`
        .security-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .security-header {
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border);
          flex-shrink: 0;
        }
        .back-btn {
          background: none; border: none; cursor: pointer;
          color: var(--color-primary); font-size: 14px; padding: 0;
          margin-bottom: var(--space-sm);
        }
        .security-header h2 { font-size: 16px; font-weight: 600; margin: 0; }
        .security-subtitle { font-size: 13px; color: var(--color-text-secondary); margin: 4px 0 0; }
        .security-content {
          flex: 1; overflow-y: auto;
          padding: var(--space-xl);
          display: flex; flex-direction: column; gap: var(--space-lg);
          max-width: 560px;
        }
        .pqc-status-card {
          display: flex; align-items: center; gap: var(--space-md);
          padding: var(--space-lg);
          border-radius: var(--radius-md);
          border: 1px solid;
        }
        .pqc-enabled { background: rgba(16,185,129,0.08); border-color: rgba(16,185,129,0.3); }
        .pqc-disabled { background: var(--color-bg-secondary); border-color: var(--color-border); }
        .pqc-status-icon { font-size: 32px; }
        .pqc-status-label { font-size: 15px; font-weight: 600; color: var(--color-text); }
        .pqc-status-algo { font-size: 12px; color: var(--color-text-secondary); margin-top: 2px; }
        .pqc-description { font-size: 14px; color: var(--color-text-secondary); line-height: 1.5; }
        .pqc-info-card {
          padding: var(--space-md);
          background: var(--color-bg-secondary);
          border-radius: var(--radius-sm);
          border-left: 3px solid var(--color-primary);
        }
        .pqc-warning { border-left-color: var(--color-warning, #f59e0b); }
        .pqc-info-card h3 { font-size: 13px; font-weight: 600; margin: 0 0 4px; }
        .pqc-info-card p { font-size: 13px; color: var(--color-text-secondary); margin: 0; line-height: 1.5; }
        .pqc-feedback {
          padding: var(--space-md);
          border-radius: var(--radius-sm);
          font-size: 13px;
        }
        .pqc-error { background: rgba(239,68,68,0.1); color: var(--color-danger, #ef4444); }
        .pqc-success { background: rgba(16,185,129,0.1); color: var(--color-success, #10b981); }
        .pqc-actions { display: flex; gap: var(--space-md); }
        .pqc-action-btn { min-width: 200px; }
        .pqc-no-vault { font-size: 13px; color: var(--color-text-secondary); }
      `}</style>
    </div>
  );
}
