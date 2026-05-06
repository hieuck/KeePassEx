/**
 * OTP Setup Dialog — desktop
 * Add or edit TOTP/HOTP for an entry
 */
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';

interface OtpSetupDialogProps {
  entryUuid: string;
  existingUri?: string;
  onSave: (uri: string) => void;
  onClose: () => void;
}

interface ParsedOtp {
  secret: string;
  algorithm: string;
  digits: number;
  period: number;
  otpType: string;
  issuer?: string;
  account?: string;
}

function isValidBase32(s: string): boolean {
  const cleaned = s.toUpperCase().replace(/\s|-/g, '');
  return /^[A-Z2-7]+=*$/.test(cleaned) && cleaned.length >= 16;
}

export function OtpSetupDialog({ entryUuid, existingUri, onSave, onClose }: OtpSetupDialogProps) {
  const { settings } = useSettingsStore();
  const isVi = settings.language === 'vi';

  const [mode, setMode] = useState<'manual' | 'uri'>('manual');
  const [secret, setSecret] = useState('');
  const [issuer, setIssuer] = useState('');
  const [account, setAccount] = useState('');
  const [algorithm, setAlgorithm] = useState<'SHA1' | 'SHA256' | 'SHA512'>('SHA1');
  const [digits, setDigits] = useState(6);
  const [period, setPeriod] = useState(30);
  const [uri, setUri] = useState(existingUri ?? '');
  const [error, setError] = useState('');
  const [preview, setPreview] = useState<string | null>(null);

  const handleParseUri = async () => {
    setError('');
    try {
      const parsed = await invoke<ParsedOtp>('parse_otp_uri', { uri });
      setSecret(parsed.secret);
      setIssuer(parsed.issuer ?? '');
      setAccount(parsed.account ?? '');
      setAlgorithm(parsed.algorithm as 'SHA1' | 'SHA256' | 'SHA512');
      setDigits(parsed.digits);
      setPeriod(parsed.period);
      setMode('manual');
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handlePreview = async () => {
    if (!isValidBase32(secret)) {
      setError(isVi ? 'Khóa bí mật không hợp lệ (phải là base32)' : 'Invalid secret key (must be base32)');
      return;
    }
    setError('');
    // Build URI and generate preview code
    const cleanSecret = secret.toUpperCase().replace(/\s|-/g, '');
    const testUri = `otpauth://totp/${encodeURIComponent(issuer || 'KeePassEx')}:${encodeURIComponent(account || 'user')}?secret=${cleanSecret}&algorithm=${algorithm}&digits=${digits}&period=${period}`;
    try {
      // We can't generate TOTP without saving first, so just show the URI
      setPreview(testUri);
    } catch {
      setPreview(null);
    }
  };

  const handleSave = () => {
    if (!isValidBase32(secret)) {
      setError(isVi ? 'Khóa bí mật không hợp lệ' : 'Invalid secret key');
      return;
    }
    const cleanSecret = secret.toUpperCase().replace(/\s|-/g, '');
    const finalUri = `otpauth://totp/${encodeURIComponent(issuer || 'KeePassEx')}:${encodeURIComponent(account || 'user')}?secret=${cleanSecret}&algorithm=${algorithm}&digits=${digits}&period=${period}${issuer ? `&issuer=${encodeURIComponent(issuer)}` : ''}`;
    onSave(finalUri);
  };

  return (
    <div className="otp-dialog-overlay" onClick={e => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="otp-dialog" role="dialog" aria-modal="true" aria-label={isVi ? 'Thiết lập OTP' : 'Set up OTP'}>
        <div className="otp-dialog-header">
          <h2>{isVi ? '⏱ Thiết lập OTP' : '⏱ Set Up OTP'}</h2>
          <button className="btn-icon" onClick={onClose} aria-label="Close">✕</button>
        </div>

        {/* Mode tabs */}
        <div className="otp-tabs">
          <button
            className={`otp-tab ${mode === 'manual' ? 'active' : ''}`}
            onClick={() => setMode('manual')}
            role="tab"
            aria-selected={mode === 'manual'}
          >
            {isVi ? '✏️ Nhập thủ công' : '✏️ Manual Entry'}
          </button>
          <button
            className={`otp-tab ${mode === 'uri' ? 'active' : ''}`}
            onClick={() => setMode('uri')}
            role="tab"
            aria-selected={mode === 'uri'}
          >
            {isVi ? '🔗 Từ URI' : '🔗 From URI'}
          </button>
        </div>

        <div className="otp-dialog-body">
          {mode === 'uri' ? (
            <div className="otp-uri-section">
              <label className="otp-label" htmlFor="otp-uri">
                {isVi ? 'URI (otpauth://...)' : 'URI (otpauth://...)'}
              </label>
              <textarea
                id="otp-uri"
                className="otp-textarea"
                value={uri}
                onChange={e => setUri(e.target.value)}
                placeholder="otpauth://totp/Example:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example"
                rows={3}
              />
              <button className="btn btn-secondary" onClick={handleParseUri} disabled={!uri.trim()}>
                {isVi ? '📋 Phân tích URI' : '📋 Parse URI'}
              </button>
            </div>
          ) : (
            <div className="otp-manual-section">
              {/* Secret */}
              <div className="otp-field">
                <label className="otp-label" htmlFor="otp-secret">
                  {isVi ? 'Khóa bí mật *' : 'Secret Key *'}
                </label>
                <input
                  id="otp-secret"
                  type="text"
                  className={`form-input otp-secret-input ${error ? 'form-input-error' : ''}`}
                  value={secret}
                  onChange={e => { setSecret(e.target.value); setError(''); }}
                  placeholder="JBSWY3DPEHPK3PXP"
                  style={{ fontFamily: 'monospace', letterSpacing: '2px' }}
                  autoComplete="off"
                  spellCheck={false}
                />
              </div>

              {/* Issuer + Account */}
              <div className="otp-row">
                <div className="otp-field">
                  <label className="otp-label" htmlFor="otp-issuer">
                    {isVi ? 'Nhà phát hành' : 'Issuer'}
                  </label>
                  <input
                    id="otp-issuer"
                    type="text"
                    className="form-input"
                    value={issuer}
                    onChange={e => setIssuer(e.target.value)}
                    placeholder="GitHub"
                  />
                </div>
                <div className="otp-field">
                  <label className="otp-label" htmlFor="otp-account">
                    {isVi ? 'Tài khoản' : 'Account'}
                  </label>
                  <input
                    id="otp-account"
                    type="text"
                    className="form-input"
                    value={account}
                    onChange={e => setAccount(e.target.value)}
                    placeholder="user@example.com"
                  />
                </div>
              </div>

              {/* Algorithm + Digits + Period */}
              <div className="otp-row">
                <div className="otp-field">
                  <label className="otp-label" htmlFor="otp-algo">
                    {isVi ? 'Thuật toán' : 'Algorithm'}
                  </label>
                  <select
                    id="otp-algo"
                    className="form-input"
                    value={algorithm}
                    onChange={e => setAlgorithm(e.target.value as 'SHA1' | 'SHA256' | 'SHA512')}
                  >
                    <option value="SHA1">SHA-1</option>
                    <option value="SHA256">SHA-256</option>
                    <option value="SHA512">SHA-512</option>
                  </select>
                </div>
                <div className="otp-field">
                  <label className="otp-label" htmlFor="otp-digits">
                    {isVi ? 'Số chữ số' : 'Digits'}
                  </label>
                  <select
                    id="otp-digits"
                    className="form-input"
                    value={digits}
                    onChange={e => setDigits(Number(e.target.value))}
                  >
                    <option value={6}>6</option>
                    <option value={8}>8</option>
                  </select>
                </div>
                <div className="otp-field">
                  <label className="otp-label" htmlFor="otp-period">
                    {isVi ? 'Chu kỳ (s)' : 'Period (s)'}
                  </label>
                  <select
                    id="otp-period"
                    className="form-input"
                    value={period}
                    onChange={e => setPeriod(Number(e.target.value))}
                  >
                    <option value={30}>30</option>
                    <option value={60}>60</option>
                  </select>
                </div>
              </div>
            </div>
          )}

          {error && (
            <p className="otp-error" role="alert">⚠️ {error}</p>
          )}

          {preview && (
            <div className="otp-preview">
              <p className="otp-preview-label">{isVi ? 'URI đã tạo:' : 'Generated URI:'}</p>
              <code className="otp-preview-uri">{preview}</code>
            </div>
          )}
        </div>

        <div className="otp-dialog-footer">
          <button className="btn btn-secondary" onClick={handlePreview} disabled={!secret.trim()}>
            {isVi ? '👁 Xem trước' : '👁 Preview'}
          </button>
          <div style={{ flex: 1 }} />
          <button className="btn btn-secondary" onClick={onClose}>
            {isVi ? 'Hủy' : 'Cancel'}
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={!secret.trim()}
          >
            {isVi ? '💾 Lưu OTP' : '💾 Save OTP'}
          </button>
        </div>
      </div>

      <style>{`
        .otp-dialog-overlay {
          position: fixed; inset: 0; background: rgba(0,0,0,.5);
          display: flex; align-items: center; justify-content: center;
          z-index: 1000;
        }
        .otp-dialog {
          background: var(--color-surface); border: 1px solid var(--color-border);
          border-radius: var(--radius-lg); width: 100%; max-width: 520px;
          box-shadow: 0 24px 64px rgba(0,0,0,.2); overflow: hidden;
          display: flex; flex-direction: column; max-height: 90vh;
        }
        .otp-dialog-header {
          display: flex; align-items: center; justify-content: space-between;
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border);
        }
        .otp-dialog-header h2 { font-size: 16px; font-weight: 600; }
        .otp-tabs {
          display: flex; border-bottom: 1px solid var(--color-border);
          padding: 0 var(--space-xl);
        }
        .otp-tab {
          padding: var(--space-sm) var(--space-lg); background: none; border: none;
          border-bottom: 2px solid transparent; cursor: pointer;
          font-size: 13px; color: var(--color-text-secondary);
          transition: color .15s, border-color .15s;
        }
        .otp-tab:hover { color: var(--color-text); }
        .otp-tab.active { color: var(--color-primary); border-bottom-color: var(--color-primary); font-weight: 500; }
        .otp-dialog-body { padding: var(--space-xl); overflow-y: auto; display: flex; flex-direction: column; gap: var(--space-md); }
        .otp-uri-section, .otp-manual-section { display: flex; flex-direction: column; gap: var(--space-md); }
        .otp-field { display: flex; flex-direction: column; gap: var(--space-xs); flex: 1; }
        .otp-row { display: flex; gap: var(--space-md); }
        .otp-label { font-size: 12px; font-weight: 500; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: .05em; }
        .otp-secret-input { font-family: 'SF Mono', 'Consolas', monospace !important; letter-spacing: 2px; }
        .otp-textarea {
          border: 1px solid var(--color-border); border-radius: var(--radius-md);
          padding: var(--space-sm) var(--space-md); font-size: 13px;
          background: var(--color-bg); color: var(--color-text);
          resize: vertical; font-family: monospace;
        }
        .otp-error { font-size: 13px; color: var(--color-danger); }
        .otp-preview {
          background: var(--color-bg-secondary); border: 1px solid var(--color-border);
          border-radius: var(--radius-md); padding: var(--space-md);
        }
        .otp-preview-label { font-size: 12px; color: var(--color-text-secondary); margin-bottom: 4px; }
        .otp-preview-uri {
          font-size: 11px; font-family: monospace; color: var(--color-text);
          word-break: break-all; display: block;
        }
        .otp-dialog-footer {
          display: flex; align-items: center; gap: var(--space-sm);
          padding: var(--space-md) var(--space-xl);
          border-top: 1px solid var(--color-border);
        }
      `}</style>
    </div>
  );
}
