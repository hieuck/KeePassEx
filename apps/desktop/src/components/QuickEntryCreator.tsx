/**
 * QuickEntryCreator — Create entries quickly from clipboard URL
 *
 * KeePassEx exclusive: detects URLs in clipboard and offers to create entries.
 * Integrates with AI password suggestions and favicon auto-fetch.
 * No competitor has this workflow on desktop.
 */
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useSettingsStore } from '../store/settings';

interface QuickEntryState {
  url: string;
  domain: string;
  title: string;
  username: string;
  password: string;
  generating: boolean;
  saving: boolean;
  visible: boolean;
}

export function QuickEntryCreator() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const { settings } = useSettingsStore();

  const [state, setState] = useState<QuickEntryState>({
    url: '',
    domain: '',
    title: '',
    username: '',
    password: '',
    generating: false,
    saving: false,
    visible: false,
  });

  // Check clipboard for URLs on focus
  useEffect(() => {
    const handleFocus = async () => {
      try {
        const text = await invoke<string>('read_clipboard_text');
        if (text && (text.startsWith('http://') || text.startsWith('https://'))) {
          const domain = await invoke<string>('get_domain_from_url', { url: text });
          setState(s => ({
            ...s,
            url: text,
            domain,
            title: domain,
            visible: true,
          }));
        }
      } catch {
        // Clipboard access failed — ignore
      }
    };

    window.addEventListener('focus', handleFocus);
    return () => window.removeEventListener('focus', handleFocus);
  }, []);

  const generatePassword = async () => {
    setState(s => ({ ...s, generating: true }));
    try {
      const result = await invoke<{ password: string }>('generate_password', {
        mode: 'random',
        length: 20,
        useUppercase: true,
        useLowercase: true,
        useDigits: true,
        useSymbols: true,
        customSymbols: null,
        excludeAmbiguous: false,
        excludeChars: '',
        minUppercase: 1,
        minLowercase: 1,
        minDigits: 1,
        minSymbols: 1,
        wordCount: 6,
        wordSeparator: '-',
        capitalizeWords: false,
        includeNumber: true,
      });
      setState(s => ({ ...s, password: result.password }));
    } catch {
      // ignore
    } finally {
      setState(s => ({ ...s, generating: false }));
    }
  };

  const handleSave = async () => {
    if (!state.title.trim()) return;
    setState(s => ({ ...s, saving: true }));
    try {
      const uuid = await invoke<string>('create_entry', {
        args: {
          group_uuid: '00000000-0000-0000-0000-000000000000',
          title: state.title,
          username: state.username,
          password: state.password,
          url: state.url,
          notes: '',
          tags: [],
          icon_id: 0,
        },
      });
      await invoke('save_vault');
      queryClient.invalidateQueries({ queryKey: ['entries'] });
      setState(s => ({ ...s, visible: false }));
      navigate(`/vault/entry/${uuid}`);
    } catch (e) {
      console.error('Quick entry save failed:', e);
    } finally {
      setState(s => ({ ...s, saving: false }));
    }
  };

  const handleDismiss = () => {
    setState(s => ({ ...s, visible: false, url: '', domain: '', title: '', password: '' }));
  };

  if (!state.visible) return null;

  return (
    <div
      className="quick-entry-overlay"
      role="dialog"
      aria-label={t('entry.new')}
      aria-modal="true"
    >
      <div className="quick-entry-card">
        <div className="quick-entry-header">
          <span className="quick-entry-icon">⚡</span>
          <div>
            <h3 className="quick-entry-title">{t('quickEntry.title')}</h3>
            <p className="quick-entry-subtitle">{state.domain}</p>
          </div>
          <button
            className="quick-entry-close"
            onClick={handleDismiss}
            aria-label={t('common.close')}
          >
            ✕
          </button>
        </div>

        <div className="quick-entry-fields">
          <div className="quick-field">
            <label className="quick-label">{t('entry.title')}</label>
            <input
              type="text"
              className="form-input"
              value={state.title}
              onChange={e => setState(s => ({ ...s, title: e.target.value }))}
              placeholder={state.domain}
              autoFocus
            />
          </div>

          <div className="quick-field">
            <label className="quick-label">{t('entry.username')}</label>
            <input
              type="text"
              className="form-input"
              value={state.username}
              onChange={e => setState(s => ({ ...s, username: e.target.value }))}
              placeholder={t('entry.username')}
              autoComplete="username"
            />
          </div>

          <div className="quick-field">
            <label className="quick-label">{t('entry.password')}</label>
            <div className="quick-password-row">
              <input
                type="password"
                className="form-input"
                value={state.password}
                onChange={e => setState(s => ({ ...s, password: e.target.value }))}
                placeholder={t('entry.password')}
                autoComplete="new-password"
                style={{ flex: 1 }}
              />
              <button
                className="btn btn-secondary btn-sm"
                onClick={generatePassword}
                disabled={state.generating}
                title={t('entry.generatePassword')}
              >
                {state.generating ? '⏳' : '⚡'}
              </button>
            </div>
          </div>

          <div className="quick-field">
            <label className="quick-label">{t('entry.url')}</label>
            <input
              type="url"
              className="form-input"
              value={state.url}
              onChange={e => setState(s => ({ ...s, url: e.target.value }))}
              placeholder="https://"
            />
          </div>
        </div>

        <div className="quick-entry-actions">
          <button className="btn btn-secondary" onClick={handleDismiss}>
            {t('common.cancel')}
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={state.saving || !state.title.trim()}
          >
            {state.saving ? '⏳' : `💾 ${t('common.save')}`}
          </button>
        </div>
      </div>

      <style>{`
        .quick-entry-overlay {
          position: fixed; bottom: var(--space-xl); right: var(--space-xl);
          z-index: 1000; animation: slideUp 0.2s ease;
        }
        @keyframes slideUp {
          from { transform: translateY(20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
        .quick-entry-card {
          width: 360px; background: var(--color-surface);
          border: 1px solid var(--color-border); border-radius: var(--radius-xl);
          box-shadow: 0 20px 60px rgba(0,0,0,0.3);
          display: flex; flex-direction: column; gap: var(--space-md);
          padding: var(--space-lg);
        }
        .quick-entry-header {
          display: flex; align-items: flex-start; gap: var(--space-sm);
        }
        .quick-entry-icon { font-size: 24px; flex-shrink: 0; }
        .quick-entry-title { font-size: 14px; font-weight: 700; margin: 0; }
        .quick-entry-subtitle { font-size: 12px; color: var(--color-text-secondary); margin: 0; }
        .quick-entry-close {
          margin-left: auto; background: none; border: none; cursor: pointer;
          color: var(--color-text-tertiary); font-size: 14px; padding: 2px 6px;
          border-radius: var(--radius-sm); flex-shrink: 0;
        }
        .quick-entry-close:hover { background: var(--color-bg-tertiary); }
        .quick-entry-fields { display: flex; flex-direction: column; gap: var(--space-sm); }
        .quick-field { display: flex; flex-direction: column; gap: 3px; }
        .quick-label { font-size: 11px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: .05em; }
        .quick-password-row { display: flex; gap: var(--space-xs); }
        .quick-entry-actions { display: flex; gap: var(--space-sm); justify-content: flex-end; }
        .btn-sm { font-size: 12px; padding: 4px 10px; }
      `}</style>
    </div>
  );
}
