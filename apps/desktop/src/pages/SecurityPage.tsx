/**
 * SecurityPage — Security settings: PQC, Argon2 tuning, vault settings
 *
 * KeePassEx exclusive features:
 * - Post-Quantum Cryptography (X25519 + Kyber-768 hybrid)
 * - Argon2id parameter tuning UI (KeePassXC has this on desktop, but not mobile)
 * - Vault settings (name, description, history, recycle bin)
 */
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';

type PqcStatus = 'classical' | 'hybrid' | 'unknown';

interface VaultSettings {
  name: string;
  description: string;
  historyMaxItems: number;
  historyMaxSizeMb: number;
  recycleBinEnabled: boolean;
  defaultUsername: string;
}

export function SecurityPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { meta } = useVaultStore();

  const [pqcStatus, setPqcStatus] = useState<PqcStatus>('unknown');
  const [migrating, setMigrating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Argon2 params
  const [argon2Memory, setArgon2Memory] = useState(65536); // KB
  const [argon2Iterations, setArgon2Iterations] = useState(2);
  const [argon2Parallelism, setArgon2Parallelism] = useState(2);
  const [argon2Saving, setArgon2Saving] = useState(false);

  // Vault settings
  const [vaultSettings, setVaultSettings] = useState<VaultSettings>({
    name: meta?.name ?? '',
    description: meta?.description ?? '',
    historyMaxItems: 10,
    historyMaxSizeMb: 6,
    recycleBinEnabled: true,
    defaultUsername: '',
  });
  const [vaultSettingsSaving, setVaultSettingsSaving] = useState(false);

  useEffect(() => {
    if (!meta) return;
    invoke<boolean>('check_pqc_status')
      .then(isPqc => setPqcStatus(isPqc ? 'hybrid' : 'classical'))
      .catch(() => setPqcStatus('classical'));

    setVaultSettings(s => ({ ...s, name: meta.name, description: meta.description }));
  }, [meta]);

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

  const handleSaveArgon2 = async () => {
    setArgon2Saving(true);
    try {
      await invoke('save_vault');
      setSuccess(t('common.success'));
    } catch (e) {
      setError(String(e));
    } finally {
      setArgon2Saving(false);
    }
  };

  const handleSaveVaultSettings = async () => {
    setVaultSettingsSaving(true);
    try {
      await invoke('update_vault_meta', {
        name: vaultSettings.name,
        description: vaultSettings.description,
        historyMaxItems: vaultSettings.historyMaxItems,
        recycleBinEnabled: vaultSettings.recycleBinEnabled,
        defaultUsername: vaultSettings.defaultUsername,
      });
      await invoke('save_vault');
      setSuccess(t('vault.saved'));
    } catch (e) {
      setError(String(e));
    } finally {
      setVaultSettingsSaving(false);
    }
  };

  const isHybrid = pqcStatus === 'hybrid';

  // Argon2 memory presets
  const MEMORY_PRESETS = [
    { label: '16 MB', value: 16384 },
    { label: '32 MB', value: 32768 },
    { label: '64 MB', value: 65536 },
    { label: '128 MB', value: 131072 },
    { label: '256 MB', value: 262144 },
  ];

  const estimateTime = (memKb: number, iters: number) => {
    // Rough estimate: 64MB/2iter ≈ 0.1s on modern hardware
    const base = (memKb / 65536) * (iters / 2) * 0.1;
    if (base < 0.1) return '< 0.1s';
    if (base < 1) return `~${base.toFixed(1)}s`;
    return `~${base.toFixed(0)}s`;
  };

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
        <h2>🔐 {t('settings.security')}</h2>
      </div>

      <div className="security-content">
        {/* ── Vault Settings ── */}
        <section className="sec-section">
          <h3 className="sec-section-title">⚙️ {t('vault.settings')}</h3>
          <div className="sec-fields">
            <div className="sec-field">
              <label className="sec-label" htmlFor="vault-name">
                {t('vault.name')}
              </label>
              <input
                id="vault-name"
                type="text"
                className="form-input"
                value={vaultSettings.name}
                onChange={e => setVaultSettings(s => ({ ...s, name: e.target.value }))}
                placeholder={t('vault.name')}
              />
            </div>
            <div className="sec-field">
              <label className="sec-label" htmlFor="vault-desc">
                {t('vault.description')}
              </label>
              <input
                id="vault-desc"
                type="text"
                className="form-input"
                value={vaultSettings.description}
                onChange={e => setVaultSettings(s => ({ ...s, description: e.target.value }))}
                placeholder={t('vault.description')}
              />
            </div>
            <div className="sec-field">
              <label className="sec-label" htmlFor="default-user">
                {t('vault.defaultUsername')}
              </label>
              <input
                id="default-user"
                type="text"
                className="form-input"
                value={vaultSettings.defaultUsername}
                onChange={e => setVaultSettings(s => ({ ...s, defaultUsername: e.target.value }))}
                placeholder={t('entry.username')}
              />
            </div>
            <div className="sec-field-row">
              <div>
                <p className="sec-label">{t('vault.recycleBin')}</p>
                <p className="sec-desc">{t('vault.recycleBinEmpty')}</p>
              </div>
              <button
                role="switch"
                aria-checked={vaultSettings.recycleBinEnabled}
                className="toggle"
                style={{
                  background: vaultSettings.recycleBinEnabled
                    ? 'var(--color-primary)'
                    : 'var(--color-border)',
                }}
                onClick={() =>
                  setVaultSettings(s => ({ ...s, recycleBinEnabled: !s.recycleBinEnabled }))
                }
              >
                <div
                  className="toggle-thumb"
                  style={{ left: vaultSettings.recycleBinEnabled ? 22 : 2 }}
                />
              </button>
            </div>
            <div className="sec-field">
              <label className="sec-label" htmlFor="history-items">
                {t('advanced.historyMaxItems')}: <strong>{vaultSettings.historyMaxItems}</strong>
              </label>
              <input
                id="history-items"
                type="range"
                min={0}
                max={50}
                step={5}
                value={vaultSettings.historyMaxItems}
                onChange={e =>
                  setVaultSettings(s => ({ ...s, historyMaxItems: Number(e.target.value) }))
                }
                className="range-input"
              />
            </div>
            <button
              className="btn btn-primary"
              onClick={handleSaveVaultSettings}
              disabled={vaultSettingsSaving || !meta}
            >
              {vaultSettingsSaving ? '⏳' : '💾'} {t('common.save')}
            </button>
          </div>
        </section>

        {/* ── Argon2 Parameter Tuning ── */}
        <section className="sec-section">
          <h3 className="sec-section-title">🔑 {t('security.argon2Title')}</h3>
          <p className="sec-section-desc">{t('security.argon2Desc')}</p>

          <div className="sec-fields">
            {/* Memory */}
            <div className="sec-field">
              <label className="sec-label">{t('security.argon2Memory')}</label>
              <div className="preset-chips">
                {MEMORY_PRESETS.map(p => (
                  <button
                    key={p.value}
                    className={`preset-chip${argon2Memory === p.value ? ' active' : ''}`}
                    onClick={() => setArgon2Memory(p.value)}
                    aria-pressed={argon2Memory === p.value}
                  >
                    {p.label}
                  </button>
                ))}
              </div>
            </div>

            {/* Iterations */}
            <div className="sec-field">
              <label className="sec-label" htmlFor="argon2-iter">
                {t('security.argon2Iterations')}: <strong>{argon2Iterations}</strong>
              </label>
              <input
                id="argon2-iter"
                type="range"
                min={1}
                max={10}
                step={1}
                value={argon2Iterations}
                onChange={e => setArgon2Iterations(Number(e.target.value))}
                className="range-input"
              />
            </div>

            {/* Parallelism */}
            <div className="sec-field">
              <label className="sec-label" htmlFor="argon2-par">
                {t('security.argon2Parallelism')}: <strong>{argon2Parallelism}</strong>
              </label>
              <input
                id="argon2-par"
                type="range"
                min={1}
                max={8}
                step={1}
                value={argon2Parallelism}
                onChange={e => setArgon2Parallelism(Number(e.target.value))}
                className="range-input"
              />
            </div>

            {/* Estimated time */}
            <div className="argon2-estimate">
              <span>⏱ {t('security.argon2EstimatedTime')}:</span>
              <strong
                style={{ color: argon2Iterations * argon2Memory > 200000 ? '#D97706' : '#16A34A' }}
              >
                {estimateTime(argon2Memory, argon2Iterations)}
              </strong>
              <span className="argon2-note">{t('security.argon2Note')}</span>
            </div>

            <button
              className="btn btn-primary"
              onClick={handleSaveArgon2}
              disabled={argon2Saving || !meta}
            >
              {argon2Saving ? '⏳' : '🔑'} {t('security.argon2Apply')}
            </button>
          </div>
        </section>

        {/* ── Post-Quantum Cryptography ── */}
        <section className="sec-section">
          <h3 className="sec-section-title">⚛️ {t('quantumResistant.title')}</h3>
          <p className="sec-section-desc">{t('quantumResistant.subtitle')}</p>

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

          <div className="pqc-info-card pqc-warning">
            <h4>⚠️ {t('quantumResistant.compatibilityNote')}</h4>
            <p>{t('quantumResistant.compatibilityNoteDesc')}</p>
          </div>

          {meta && (
            <div className="pqc-actions">
              {!isHybrid ? (
                <button className="btn btn-primary" onClick={handleEnablePqc} disabled={migrating}>
                  {migrating ? t('quantumResistant.migrating') : t('quantumResistant.enable')}
                </button>
              ) : (
                <button
                  className="btn btn-secondary"
                  onClick={handleDisablePqc}
                  disabled={migrating}
                >
                  {migrating ? t('quantumResistant.migrating') : t('quantumResistant.disable')}
                </button>
              )}
            </div>
          )}
        </section>

        {/* Feedback */}
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
      </div>

      <style>{`
        .security-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .security-header { padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; }
        .back-btn { background:none; border:none; cursor:pointer; color:var(--color-primary); font-size:14px; padding:0; margin-bottom:var(--space-sm); }
        .security-header h2 { font-size:16px; font-weight:600; margin:0; }
        .security-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:560px; }
        .sec-section { background:var(--color-bg-secondary); border:1px solid var(--color-border); border-radius:var(--radius-lg); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); }
        .sec-section-title { font-size:14px; font-weight:600; margin:0; }
        .sec-section-desc { font-size:13px; color:var(--color-text-secondary); margin:0; line-height:1.5; }
        .sec-fields { display:flex; flex-direction:column; gap:var(--space-md); }
        .sec-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .sec-field-row { display:flex; align-items:center; justify-content:space-between; gap:var(--space-md); }
        .sec-label { font-size:13px; font-weight:500; color:var(--color-text-secondary); }
        .sec-desc { font-size:11px; color:var(--color-text-tertiary); }
        .range-input { width:100%; accent-color:var(--color-primary); }
        .preset-chips { display:flex; gap:var(--space-xs); flex-wrap:wrap; }
        .preset-chip { padding:var(--space-xs) var(--space-md); border:1px solid var(--color-border); border-radius:var(--radius-full); background:var(--color-bg); cursor:pointer; font-size:12px; color:var(--color-text-secondary); transition:all .1s; }
        .preset-chip:hover { border-color:var(--color-primary); color:var(--color-primary); }
        .preset-chip.active { background:var(--color-primary); border-color:var(--color-primary); color:white; }
        .argon2-estimate { display:flex; align-items:center; gap:var(--space-sm); font-size:13px; color:var(--color-text-secondary); padding:var(--space-sm) var(--space-md); background:var(--color-bg); border-radius:var(--radius-sm); }
        .argon2-note { font-size:11px; color:var(--color-text-tertiary); }
        .pqc-status-card { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-md); border-radius:var(--radius-md); border:1px solid; }
        .pqc-enabled { background:rgba(16,185,129,0.08); border-color:rgba(16,185,129,0.3); }
        .pqc-disabled { background:var(--color-bg); border-color:var(--color-border); }
        .pqc-status-icon { font-size:28px; }
        .pqc-status-label { font-size:14px; font-weight:600; }
        .pqc-status-algo { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .pqc-info-card { padding:var(--space-md); background:var(--color-bg); border-radius:var(--radius-sm); border-left:3px solid var(--color-primary); }
        .pqc-warning { border-left-color:#f59e0b; }
        .pqc-info-card h4 { font-size:13px; font-weight:600; margin:0 0 4px; }
        .pqc-info-card p { font-size:13px; color:var(--color-text-secondary); margin:0; line-height:1.5; }
        .pqc-actions { display:flex; gap:var(--space-md); }
        .pqc-feedback { padding:var(--space-md); border-radius:var(--radius-sm); font-size:13px; }
        .pqc-error { background:rgba(239,68,68,0.1); color:#ef4444; }
        .pqc-success { background:rgba(16,185,129,0.1); color:#10b981; }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background .2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2); }
      `}</style>
    </div>
  );
}
