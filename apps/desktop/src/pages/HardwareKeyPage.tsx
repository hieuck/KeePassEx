/**
 * Hardware Key settings page
 */
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';

interface HardwareKeyInfo {
  key_type: string;
  device_id: string;
  label: string;
  firmware_version?: string;
  serial_number?: string;
  is_connected: boolean;
}

type SetupStep = 'select-type' | 'configure' | 'test' | 'done';

export function HardwareKeyPage() {
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [detectedKeys, setDetectedKeys] = useState<HardwareKeyInfo[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [step, setStep] = useState<SetupStep>('select-type');
  const [selectedType, setSelectedType] = useState('yubikey_hmac');
  const [selectedSlot, setSelectedSlot] = useState(2);
  const [label, setLabel] = useState('');
  const [requireTouch, setRequireTouch] = useState(true);
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [currentConfig, setCurrentConfig] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadCurrentConfig();
    scanForKeys();
  }, []);

  async function loadCurrentConfig() {
    try {
      setCurrentConfig(await invoke<string | null>('get_hardware_key_config'));
    } catch {}
  }

  async function scanForKeys() {
    setIsScanning(true);
    setError(null);
    try {
      setDetectedKeys(await invoke<HardwareKeyInfo[]>('list_hardware_keys_cmd'));
    } catch (e) {
      setError(String(e));
    } finally {
      setIsScanning(false);
    }
  }

  async function testKey() {
    setIsTesting(true);
    setTestResult(null);
    setError(null);
    try {
      const result = await invoke<boolean>('test_hardware_key_cmd', {
        args: { key_type: selectedType, slot: selectedSlot, label, require_touch: requireTouch },
      });
      setTestResult(result);
      if (result) setStep('done');
    } catch (e) {
      setError(String(e));
      setTestResult(false);
    } finally {
      setIsTesting(false);
    }
  }

  async function saveConfig() {
    setIsSaving(true);
    setError(null);
    try {
      await invoke('configure_hardware_key', {
        args: { key_type: selectedType, slot: selectedSlot, label, require_touch: requireTouch },
      });
      await invoke('save_vault');
      setCurrentConfig(JSON.stringify({ key_type: selectedType, label }));
      setStep('done');
    } catch (e) {
      setError(String(e));
    } finally {
      setIsSaving(false);
    }
  }

  async function removeConfig() {
    if (!confirm(t('hardwareKey.confirmRemove'))) return;
    try {
      await invoke('remove_hardware_key');
      await invoke('save_vault');
      setCurrentConfig(null);
      setStep('select-type');
    } catch (e) {
      setError(String(e));
    }
  }

  const keyTypes = [
    {
      id: 'yubikey_hmac',
      name: t('hardwareKey.typeYubikeyHmac'),
      desc: 'Most compatible, works offline',
      icon: '🔑',
      recommended: true,
    },
    {
      id: 'fido2',
      name: t('hardwareKey.typeFido2'),
      desc: 'Any FIDO2/WebAuthn security key',
      icon: '🛡️',
    },
    {
      id: 'smart_card',
      name: t('hardwareKey.typeSmartCard'),
      desc: 'CAC, PIV enterprise cards',
      icon: '💳',
    },
    {
      id: 'onlykey',
      name: t('hardwareKey.typeOnlyKey'),
      desc: 'OnlyKey hardware token',
      icon: '🔐',
    },
  ];

  return (
    <div className="hw-page">
      <div className="hw-header">
        <button className="btn-back" onClick={() => navigate('/settings')} aria-label="Back">
          ← {t('settings.title')}
        </button>
        <h2>🔑 {t('hardwareKey.title')}</h2>
      </div>

      <div className="hw-content">
        {currentConfig && (
          <div className="hw-current">
            <span className="hw-current-icon">✅</span>
            <div className="hw-current-info">
              <p className="hw-current-title">{t('hardwareKey.connected')}</p>
              <p className="hw-current-desc">{t('hardwareKey.requireTouchDesc')}</p>
            </div>
            <button className="btn btn-danger-outline" onClick={removeConfig}>
              {t('hardwareKey.remove')}
            </button>
          </div>
        )}

        <div className="hw-section">
          <div className="hw-section-header">
            <h3>{t('hardwareKey.connected')}</h3>
            <button className="btn btn-sm" onClick={scanForKeys} disabled={isScanning}>
              {isScanning ? '⏳' : '🔄'} Scan
            </button>
          </div>
          {detectedKeys.length === 0 ? (
            <div className="hw-empty">
              <span>🔌</span>
              <p>{t('hardwareKey.notDetected')}</p>
              <p className="hw-empty-hint">{t('hardwareKey.notDetectedDesc')}</p>
            </div>
          ) : (
            <div className="hw-key-list">
              {detectedKeys.map((key, i) => (
                <div key={i} className="hw-key-item">
                  <span
                    className="hw-key-status"
                    style={{ color: key.is_connected ? '#22c55e' : '#ef4444' }}
                  >
                    {key.is_connected ? '●' : '○'}
                  </span>
                  <div className="hw-key-info">
                    <p className="hw-key-label">{key.label}</p>
                    <p className="hw-key-type">{key.key_type}</p>
                    {key.serial_number && <p className="hw-key-serial">S/N: {key.serial_number}</p>}
                  </div>
                  {key.firmware_version && (
                    <span className="hw-key-fw">FW {key.firmware_version}</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="hw-section">
          <h3>{t('hardwareKey.setupTitle')}</h3>
          <p className="hw-section-desc">{t('hardwareKey.setupDesc')}</p>

          {(step === 'select-type' || step === 'configure') && (
            <div className="hw-step">
              <p className="hw-step-label">1. {t('hardwareKey.type')}</p>
              <div className="hw-type-grid">
                {keyTypes.map(kt => (
                  <button
                    key={kt.id}
                    className={`hw-type-card ${selectedType === kt.id ? 'selected' : ''}`}
                    onClick={() => {
                      setSelectedType(kt.id);
                      setStep('configure');
                    }}
                    aria-pressed={selectedType === kt.id}
                  >
                    <span className="hw-type-icon">{kt.icon}</span>
                    <span className="hw-type-name">
                      {kt.name}
                      {kt.recommended && <span className="hw-badge">Recommended</span>}
                    </span>
                    <span className="hw-type-desc">{kt.desc}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {step === 'configure' && (
            <div className="hw-step">
              <p className="hw-step-label">2. {t('hardwareKey.configure')}</p>
              {selectedType === 'yubikey_hmac' && (
                <div className="hw-field">
                  <label className="hw-label">{t('hardwareKey.slot')}</label>
                  <div className="hw-radio-group">
                    {[1, 2].map(slot => (
                      <label key={slot} className="hw-radio">
                        <input
                          type="radio"
                          name="slot"
                          value={slot}
                          checked={selectedSlot === slot}
                          onChange={() => setSelectedSlot(slot)}
                        />
                        {slot === 1 ? t('hardwareKey.slot1') : t('hardwareKey.slot2')}
                        {slot === 2 && <span className="hw-badge">Default</span>}
                      </label>
                    ))}
                  </div>
                </div>
              )}
              <div className="hw-field">
                <label className="hw-label" htmlFor="hw-label-input">
                  {t('hardwareKey.label')}
                </label>
                <input
                  id="hw-label-input"
                  className="hw-input"
                  type="text"
                  value={label}
                  onChange={e => setLabel(e.target.value)}
                  placeholder={t('hardwareKey.labelPlaceholder')}
                />
              </div>
              <div className="hw-field hw-field-row">
                <div>
                  <p className="hw-label">{t('hardwareKey.requireTouch')}</p>
                  <p className="hw-field-desc">{t('hardwareKey.requireTouchDesc')}</p>
                </div>
                <button
                  role="switch"
                  aria-checked={requireTouch}
                  className="toggle"
                  style={{
                    background: requireTouch ? 'var(--color-primary)' : 'var(--color-border)',
                  }}
                  onClick={() => setRequireTouch(v => !v)}
                >
                  <div className="toggle-thumb" style={{ left: requireTouch ? 22 : 2 }} />
                </button>
              </div>
              <div className="hw-actions">
                <button className="btn btn-primary" onClick={() => setStep('test')}>
                  Next: Test →
                </button>
              </div>
            </div>
          )}

          {step === 'test' && (
            <div className="hw-step">
              <p className="hw-step-label">3. {t('hardwareKey.testKey')}</p>
              <div className="hw-test-box">
                <span className="hw-test-icon">
                  {isTesting
                    ? '⏳'
                    : testResult === true
                      ? '✅'
                      : testResult === false
                        ? '❌'
                        : '🔑'}
                </span>
                <p className="hw-test-desc">
                  {isTesting
                    ? t('hardwareKey.touchKey')
                    : testResult === true
                      ? t('hardwareKey.testSuccess')
                      : testResult === false
                        ? t('hardwareKey.testFailed')
                        : t('hardwareKey.insertKey')}
                </p>
              </div>
              {error && <div className="hw-error">⚠️ {error}</div>}
              <div className="hw-actions">
                <button className="btn" onClick={() => setStep('configure')}>
                  ← {t('common.back')}
                </button>
                <button className="btn btn-secondary" onClick={testKey} disabled={isTesting}>
                  🔑 {t('hardwareKey.testKey')}
                </button>
                {testResult === true && (
                  <button className="btn btn-primary" onClick={saveConfig} disabled={isSaving}>
                    {isSaving ? '⏳' : '✅'} {t('common.save')}
                  </button>
                )}
              </div>
            </div>
          )}

          {step === 'done' && (
            <div className="hw-step hw-done">
              <span className="hw-done-icon">🎉</span>
              <h3>{t('hardwareKey.testSuccess')}</h3>
              <p>{t('hardwareKey.setupDesc')}</p>
              <button className="btn btn-primary" onClick={() => navigate('/settings')}>
                {t('common.back')}
              </button>
            </div>
          )}
        </div>
      </div>

      <style>{`
        .hw-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .hw-header { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; }
        .hw-header h2 { font-size:16px; font-weight:600; }
        .btn-back { background:none; border:none; cursor:pointer; font-size:13px; color:var(--color-primary); padding:var(--space-xs) var(--space-sm); border-radius:var(--radius-sm); }
        .btn-back:hover { background:var(--color-bg-secondary); }
        .hw-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:600px; }
        .hw-current { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-md); background:rgba(34,197,94,0.08); border:1px solid rgba(34,197,94,0.3); border-radius:var(--radius-md); }
        .hw-current-icon { font-size:24px; }
        .hw-current-info { flex:1; }
        .hw-current-title { font-size:14px; font-weight:600; }
        .hw-current-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .hw-section { background:var(--color-bg-secondary); border-radius:var(--radius-md); padding:var(--space-lg); display:flex; flex-direction:column; gap:var(--space-md); }
        .hw-section h3 { font-size:15px; font-weight:600; }
        .hw-section-header { display:flex; align-items:center; justify-content:space-between; }
        .hw-section-desc { font-size:13px; color:var(--color-text-secondary); }
        .hw-empty { display:flex; flex-direction:column; align-items:center; gap:var(--space-sm); padding:var(--space-xl); color:var(--color-text-secondary); font-size:13px; }
        .hw-empty span { font-size:32px; }
        .hw-empty-hint { font-size:12px; opacity:0.7; }
        .hw-key-list { display:flex; flex-direction:column; gap:var(--space-sm); }
        .hw-key-item { display:flex; align-items:center; gap:var(--space-md); padding:var(--space-sm) var(--space-md); background:var(--color-bg); border-radius:var(--radius-sm); border:1px solid var(--color-border); }
        .hw-key-status { font-size:10px; }
        .hw-key-info { flex:1; }
        .hw-key-label { font-size:13px; font-weight:500; }
        .hw-key-type { font-size:11px; color:var(--color-text-secondary); }
        .hw-key-serial { font-size:11px; color:var(--color-text-tertiary); }
        .hw-key-fw { font-size:11px; color:var(--color-text-tertiary); }
        .hw-step { display:flex; flex-direction:column; gap:var(--space-md); }
        .hw-step-label { font-size:13px; font-weight:600; color:var(--color-text-secondary); }
        .hw-type-grid { display:grid; grid-template-columns:1fr 1fr; gap:var(--space-sm); }
        .hw-type-card { display:flex; flex-direction:column; gap:4px; padding:var(--space-md); background:var(--color-bg); border:2px solid var(--color-border); border-radius:var(--radius-md); cursor:pointer; text-align:left; transition:border-color 0.15s; }
        .hw-type-card:hover { border-color:var(--color-primary); }
        .hw-type-card.selected { border-color:var(--color-primary); background:rgba(37,99,235,0.05); }
        .hw-type-icon { font-size:20px; }
        .hw-type-name { font-size:13px; font-weight:600; display:flex; align-items:center; gap:6px; flex-wrap:wrap; }
        .hw-type-desc { font-size:11px; color:var(--color-text-secondary); }
        .hw-badge { font-size:10px; padding:1px 6px; border-radius:10px; background:rgba(37,99,235,0.12); color:var(--color-primary); font-weight:500; }
        .hw-field { display:flex; flex-direction:column; gap:var(--space-xs); }
        .hw-field-row { flex-direction:row; align-items:center; justify-content:space-between; }
        .hw-label { font-size:13px; font-weight:500; }
        .hw-field-desc { font-size:12px; color:var(--color-text-secondary); }
        .hw-input { padding:var(--space-sm) var(--space-md); border:1px solid var(--color-border); border-radius:var(--radius-sm); background:var(--color-bg); font-size:13px; color:var(--color-text); }
        .hw-radio-group { display:flex; gap:var(--space-md); }
        .hw-radio { display:flex; align-items:center; gap:var(--space-xs); font-size:13px; cursor:pointer; }
        .hw-actions { display:flex; gap:var(--space-sm); flex-wrap:wrap; }
        .hw-test-box { display:flex; flex-direction:column; align-items:center; gap:var(--space-md); padding:var(--space-xl); background:var(--color-bg); border-radius:var(--radius-md); border:1px solid var(--color-border); }
        .hw-test-icon { font-size:40px; }
        .hw-test-desc { font-size:13px; color:var(--color-text-secondary); text-align:center; }
        .hw-error { padding:var(--space-sm) var(--space-md); background:rgba(239,68,68,0.1); border:1px solid rgba(239,68,68,0.3); border-radius:var(--radius-sm); font-size:12px; color:#ef4444; }
        .hw-done { align-items:center; text-align:center; padding:var(--space-xl); }
        .hw-done-icon { font-size:48px; }
        .hw-done h3 { font-size:18px; font-weight:700; }
        .hw-done p { font-size:13px; color:var(--color-text-secondary); max-width:320px; }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background 0.2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left 0.2s; box-shadow:0 1px 3px rgba(0,0,0,0.2); }
        .btn-sm { font-size:12px; padding:var(--space-xs) var(--space-sm); }
        .btn-danger-outline { background:none; border:1px solid rgba(239,68,68,0.4); color:#ef4444; border-radius:var(--radius-sm); padding:var(--space-xs) var(--space-md); cursor:pointer; font-size:13px; }
        .btn-danger-outline:hover { background:rgba(239,68,68,0.08); }
      `}</style>
    </div>
  );
}
