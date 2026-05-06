/**
 * SteganographyPage — Hide vault inside image/video files
 *
 * Exclusive KeePassEx feature: No competitor has this.
 * Embeds encrypted vault into PNG/JPEG/MP4/AVI files using LSB steganography.
 */
import React, { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

type StegMode = 'embed' | 'extract';
type StegFormat = 'png' | 'jpeg' | 'mp4' | 'avi';

interface StegStatus {
  type: 'idle' | 'loading' | 'success' | 'error';
  message?: string;
}

export function SteganographyPage() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<StegMode>('embed');
  const [carrierPath, setCarrierPath] = useState('');
  const [vaultPath, setVaultPath] = useState('');
  const [outputPath, setOutputPath] = useState('');
  const [stegPassword, setStegPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [status, setStatus] = useState<StegStatus>({ type: 'idle' });
  const [detectedFormat, setDetectedFormat] = useState<StegFormat | null>(null);
  const [hasVault, setHasVault] = useState<boolean | null>(null);

  const selectCarrier = useCallback(async () => {
    const selected = await open({
      filters: [
        {
          name: t('steganography.selectCarrier'),
          extensions: ['png', 'jpg', 'jpeg', 'mp4', 'avi'],
        },
      ],
    });
    if (selected && typeof selected === 'string') {
      setCarrierPath(selected);
      // Detect format and check for embedded vault
      try {
        const result = await invoke<{ format: StegFormat; hasVault: boolean }>(
          'detect_steg_carrier',
          { path: selected }
        );
        setDetectedFormat(result.format);
        setHasVault(result.hasVault);
      } catch {
        setDetectedFormat(null);
        setHasVault(null);
      }
    }
  }, [t]);

  const selectVault = useCallback(async () => {
    const selected = await open({
      filters: [{ name: 'KDBX Vault', extensions: ['kdbx'] }],
    });
    if (selected && typeof selected === 'string') {
      setVaultPath(selected);
    }
  }, []);

  const selectOutput = useCallback(async () => {
    const ext = detectedFormat ?? 'png';
    const selected = await save({
      filters: [{ name: 'Image/Video', extensions: [ext] }],
    });
    if (selected) {
      setOutputPath(selected);
    }
  }, [detectedFormat]);

  const handleEmbed = useCallback(async () => {
    if (!carrierPath || !vaultPath || !outputPath || !stegPassword) return;
    if (stegPassword !== confirmPassword) {
      setStatus({ type: 'error', message: t('vault.passwordsDoNotMatch') });
      return;
    }

    setStatus({ type: 'loading' });
    try {
      await invoke('steg_embed_vault', {
        carrierPath,
        vaultPath,
        outputPath,
        stegPassword,
      });
      setStatus({
        type: 'success',
        message: t('steganography.embedSuccess', {
          file: outputPath.split(/[\\/]/).pop(),
        }),
      });
    } catch (err) {
      setStatus({
        type: 'error',
        message: t('steganography.embedFailed', { error: String(err) }),
      });
    }
  }, [carrierPath, vaultPath, outputPath, stegPassword, confirmPassword, t]);

  const handleExtract = useCallback(async () => {
    if (!carrierPath || !stegPassword) return;

    const outputVaultPath = await save({
      filters: [{ name: 'KDBX Vault', extensions: ['kdbx'] }],
      defaultPath: 'extracted_vault.kdbx',
    });
    if (!outputVaultPath) return;

    setStatus({ type: 'loading' });
    try {
      await invoke('steg_extract_vault', {
        carrierPath,
        outputPath: outputVaultPath,
        stegPassword,
      });
      setStatus({ type: 'success', message: t('steganography.extractSuccess') });
    } catch (err) {
      setStatus({
        type: 'error',
        message: t('steganography.extractFailed', { error: String(err) }),
      });
    }
  }, [carrierPath, stegPassword, t]);

  const formatIcon: Record<StegFormat, string> = {
    png: '🖼️',
    jpeg: '📷',
    mp4: '🎬',
    avi: '🎥',
  };

  return (
    <div className="page-container" role="main" aria-label={t('steganography.title')}>
      {/* Header */}
      <div className="page-header">
        <h1 className="page-title">🕵️ {t('steganography.title')}</h1>
        <p className="page-subtitle">{t('steganography.subtitle')}</p>
        <span className="badge badge-exclusive" aria-label={t('steganography.uniqueFeature')}>
          ✨ {t('steganography.uniqueFeature')}
        </span>
      </div>

      {/* Mode Tabs */}
      <div className="tab-bar" role="tablist" aria-label="Steganography mode">
        <button
          role="tab"
          aria-selected={mode === 'embed'}
          className={`tab ${mode === 'embed' ? 'tab-active' : ''}`}
          onClick={() => setMode('embed')}
        >
          📥 {t('steganography.embed')}
        </button>
        <button
          role="tab"
          aria-selected={mode === 'extract'}
          className={`tab ${mode === 'extract' ? 'tab-active' : ''}`}
          onClick={() => setMode('extract')}
        >
          📤 {t('steganography.extract')}
        </button>
      </div>

      <div className="page-content">
        {/* Carrier File Selection */}
        <section className="form-section" aria-labelledby="carrier-label">
          <label id="carrier-label" className="form-label">
            {t('steganography.selectCarrier')}
          </label>
          <div className="file-input-row">
            <input
              type="text"
              className="form-input"
              value={carrierPath}
              readOnly
              placeholder={t('steganography.selectCarrier')}
              aria-label={t('steganography.selectCarrier')}
            />
            <button className="btn btn-secondary" onClick={selectCarrier}>
              {t('common.browse', 'Browse...')}
            </button>
          </div>

          {/* Format detection badge */}
          {detectedFormat && (
            <div className="format-badge" role="status">
              {formatIcon[detectedFormat]} {detectedFormat.toUpperCase()}
              {hasVault === true && (
                <span className="vault-detected-badge">🔐 {t('steganography.vaultDetected')}</span>
              )}
            </div>
          )}
        </section>

        {/* Embed-specific fields */}
        {mode === 'embed' && (
          <>
            <section className="form-section" aria-labelledby="vault-label">
              <label id="vault-label" className="form-label">
                {t('steganography.selectVault')}
              </label>
              <div className="file-input-row">
                <input
                  type="text"
                  className="form-input"
                  value={vaultPath}
                  readOnly
                  placeholder="vault.kdbx"
                  aria-label={t('steganography.selectVault')}
                />
                <button className="btn btn-secondary" onClick={selectVault}>
                  {t('common.browse', 'Browse...')}
                </button>
              </div>
            </section>

            <section className="form-section" aria-labelledby="output-label">
              <label id="output-label" className="form-label">
                {t('steganography.outputFile')}
              </label>
              <div className="file-input-row">
                <input
                  type="text"
                  className="form-input"
                  value={outputPath}
                  readOnly
                  placeholder={t('steganography.outputFile')}
                  aria-label={t('steganography.outputFile')}
                />
                <button className="btn btn-secondary" onClick={selectOutput}>
                  {t('common.browse', 'Browse...')}
                </button>
              </div>
            </section>
          </>
        )}

        {/* Steganography Password */}
        <section className="form-section" aria-labelledby="steg-pwd-label">
          <label id="steg-pwd-label" className="form-label">
            {t('steganography.stegPassword')}
          </label>
          <p className="form-hint">{t('steganography.stegPasswordDesc')}</p>
          <div className="password-input-row">
            <input
              type={showPassword ? 'text' : 'password'}
              className="form-input"
              value={stegPassword}
              onChange={e => setStegPassword(e.target.value)}
              placeholder={t('steganography.stegPasswordPlaceholder')}
              aria-label={t('steganography.stegPassword')}
              autoComplete="new-password"
            />
            <button
              className="btn btn-icon"
              onClick={() => setShowPassword(p => !p)}
              aria-label={showPassword ? t('entry.hidePassword') : t('entry.showPassword')}
            >
              {showPassword ? '🙈' : '👁️'}
            </button>
          </div>

          {mode === 'embed' && (
            <input
              type={showPassword ? 'text' : 'password'}
              className="form-input mt-2"
              value={confirmPassword}
              onChange={e => setConfirmPassword(e.target.value)}
              placeholder={t('steganography.confirmStegPassword')}
              aria-label={t('steganography.confirmStegPassword')}
              autoComplete="new-password"
            />
          )}
        </section>

        {/* Status */}
        {status.type !== 'idle' && (
          <div className={`status-banner status-${status.type}`} role="status" aria-live="polite">
            {status.type === 'loading' && (
              <span>
                ⏳ {mode === 'embed' ? t('steganography.embedding') : t('steganography.extracting')}
              </span>
            )}
            {status.type === 'success' && <span>✅ {status.message}</span>}
            {status.type === 'error' && <span>❌ {status.message}</span>}
          </div>
        )}

        {/* Action Button */}
        <button
          className="btn btn-primary btn-large"
          onClick={mode === 'embed' ? handleEmbed : handleExtract}
          disabled={
            status.type === 'loading' ||
            !carrierPath ||
            !stegPassword ||
            (mode === 'embed' && (!vaultPath || !outputPath))
          }
          aria-busy={status.type === 'loading'}
        >
          {mode === 'embed' ? `📥 ${t('steganography.embed')}` : `📤 ${t('steganography.extract')}`}
        </button>

        {/* Supported Formats Info */}
        <section className="info-section" aria-labelledby="formats-label">
          <h3 id="formats-label">{t('steganography.supportedFormats')}</h3>
          <ul className="format-list">
            <li>🖼️ {t('steganography.formatPng')}</li>
            <li>📷 {t('steganography.formatJpeg')}</li>
            <li>🎬 {t('steganography.formatMp4')}</li>
            <li>🎥 {t('steganography.formatAvi')}</li>
          </ul>
        </section>

        {/* Security Note */}
        <section className="security-note" aria-labelledby="security-note-label">
          <h3 id="security-note-label">🔒 {t('steganography.securityNote')}</h3>
          <p>{t('steganography.securityNoteDesc')}</p>
        </section>
      </div>
    </div>
  );
}
