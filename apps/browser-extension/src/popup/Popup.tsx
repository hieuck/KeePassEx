/**
 * KeePassEx Browser Extension — Popup UI
 * Enhanced with: OTP inline display, password save detection,
 * smart URL matching, passkey indicator, keyboard navigation,
 * recently used entries, dark mode, i18n (EN/VI)
 */
import React, { useState, useEffect, useRef, useCallback } from 'react';
import browser from 'webextension-polyfill';

interface EntryOption {
  uuid: string;
  title: string;
  username: string;
  url: string;
  hasOtp: boolean;
  hasPasskey?: boolean;
  matchScore?: 'exact' | 'domain' | 'subdomain';
  lastUsed?: string;
}

interface OtpState {
  code: string;
  remaining: number;
  period: number;
}

interface VaultStatus {
  connected: boolean;
  locked: boolean;
  vaultName?: string;
}

type View = 'loading' | 'disconnected' | 'locked' | 'entries' | 'save-prompt';

// ─── i18n (EN / VI) ───────────────────────────────────────────────────────────

const STRINGS: Record<string, Record<string, string>> = {
  en: {
    appName: 'KeePassEx',
    connecting: 'Connecting to KeePassEx...',
    notRunning: 'KeePassEx not running',
    notRunningDesc: 'Open the KeePassEx desktop app to use browser integration.',
    vaultLocked: 'Vault is locked',
    vaultLockedDesc: 'Unlock your vault in the KeePassEx app.',
    openApp: 'Open KeePassEx',
    unlockVault: 'Unlock Vault',
    searchPlaceholder: 'Search entries...',
    noMatches: 'No matching entries',
    noMatchesFor: 'No entries for',
    matchesFor: 'matches for',
    match: 'match',
    fill: 'Fill',
    copyPassword: 'Copy password',
    copyOtp: 'Copy OTP',
    copied: 'Copied!',
    generate: 'Generate password',
    recentlyUsed: 'Recently used',
    savePassword: 'Save password?',
    savePasswordDesc: 'KeePassEx detected new credentials on this page.',
    save: 'Save',
    dismiss: 'Dismiss',
    matchExact: 'Exact match',
    matchDomain: 'Domain match',
    matchSubdomain: 'Subdomain match',
    passkey: 'Passkey',
    otpExpires: 'expires in',
    settings: 'Settings',
  },
  vi: {
    appName: 'KeePassEx',
    connecting: 'Đang kết nối KeePassEx...',
    notRunning: 'KeePassEx chưa chạy',
    notRunningDesc: 'Mở ứng dụng KeePassEx để dùng tích hợp trình duyệt.',
    vaultLocked: 'Kho đang bị khóa',
    vaultLockedDesc: 'Mở khóa kho trong ứng dụng KeePassEx.',
    openApp: 'Mở KeePassEx',
    unlockVault: 'Mở khóa kho',
    searchPlaceholder: 'Tìm kiếm mục...',
    noMatches: 'Không tìm thấy mục phù hợp',
    noMatchesFor: 'Không có mục cho',
    matchesFor: 'kết quả cho',
    match: 'kết quả',
    fill: 'Điền',
    copyPassword: 'Sao chép mật khẩu',
    copyOtp: 'Sao chép OTP',
    copied: 'Đã sao chép!',
    generate: 'Tạo mật khẩu',
    recentlyUsed: 'Dùng gần đây',
    savePassword: 'Lưu mật khẩu?',
    savePasswordDesc: 'KeePassEx phát hiện thông tin đăng nhập mới trên trang này.',
    save: 'Lưu',
    dismiss: 'Bỏ qua',
    matchExact: 'Khớp chính xác',
    matchDomain: 'Khớp tên miền',
    matchSubdomain: 'Khớp tên miền con',
    passkey: 'Passkey',
    otpExpires: 'hết hạn sau',
    settings: 'Cài đặt',
  },
};

function useLocale() {
  const lang = navigator.language.startsWith('vi') ? 'vi' : 'en';
  return (key: string) => STRINGS[lang]?.[key] ?? STRINGS.en[key] ?? key;
}

export function Popup() {
  const t = useLocale();
  const [view, setView] = useState<View>('loading');
  const [entries, setEntries] = useState<EntryOption[]>([]);
  const [recentEntries, setRecentEntries] = useState<EntryOption[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<EntryOption[]>([]);
  const [vaultStatus, setVaultStatus] = useState<VaultStatus | null>(null);
  const [currentUrl, setCurrentUrl] = useState('');
  const [copiedUuid, setCopiedUuid] = useState<string | null>(null);
  const [otpStates, setOtpStates] = useState<Record<string, OtpState>>({});
  const [focusedIndex, setFocusedIndex] = useState(0);
  const [savePromptData, setSavePromptData] = useState<{ username: string; url: string } | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    init();
  }, []);

  useEffect(() => {
    if (searchQuery.trim()) {
      performSearch(searchQuery);
    } else {
      setSearchResults([]);
    }
  }, [searchQuery]);

  // OTP countdown timer
  useEffect(() => {
    const interval = setInterval(() => {
      setOtpStates(prev => {
        const next = { ...prev };
        for (const uuid of Object.keys(next)) {
          const state = next[uuid];
          if (state.remaining > 1) {
            next[uuid] = { ...state, remaining: state.remaining - 1 };
          } else {
            refreshOtp(uuid);
          }
        }
        return next;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const refreshOtp = useCallback(async (uuid: string) => {
    const response = await browser.runtime.sendMessage({
      action: 'GENERATE_OTP',
      payload: { entryUuid: uuid },
    });
    if (response?.success && response.data) {
      setOtpStates(prev => ({
        ...prev,
        [uuid]: {
          code: response.data.code,
          remaining: response.data.remainingSeconds ?? 30,
          period: response.data.period ?? 30,
        },
      }));
    }
  }, []);

  const init = async () => {
    const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
    const url = tab?.url ?? '';
    setCurrentUrl(url);

    const statusResponse = await browser.runtime.sendMessage({ action: 'GET_VAULT_STATUS' });
    if (!statusResponse?.success) { setView('disconnected'); return; }

    const status: VaultStatus = statusResponse.data;
    setVaultStatus(status);
    if (status.locked) { setView('locked'); return; }

    const [entriesResp, recentResp, saveResp] = await Promise.all([
      browser.runtime.sendMessage({ action: 'GET_CREDENTIALS_FOR_URL', payload: { url } }),
      browser.runtime.sendMessage({ action: 'GET_RECENT_ENTRIES', payload: { limit: 3 } }),
      browser.runtime.sendMessage({ action: 'GET_PENDING_SAVE' }),
    ]);

    if (entriesResp?.success && Array.isArray(entriesResp.data)) setEntries(entriesResp.data);
    if (recentResp?.success && Array.isArray(recentResp.data)) setRecentEntries(recentResp.data);

    if (saveResp?.success && saveResp.data) {
      setSavePromptData(saveResp.data);
      setView('save-prompt');
      return;
    }

    setView('entries');
    setTimeout(() => searchRef.current?.focus(), 50);
  };

  const performSearch = async (query: string) => {
    const response = await browser.runtime.sendMessage({
      action: 'SEARCH_ENTRIES',
      payload: { query },
    });
    if (response?.success) setSearchResults(response.data ?? []);
  };

  const handleFill = async (entry: EntryOption) => {
    const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id) return;
    await browser.runtime.sendMessage({ action: 'AUTOFILL', payload: { entryUuid: entry.uuid, tabId: tab.id } });
    await browser.runtime.sendMessage({ action: 'TRACK_USAGE', payload: { entryUuid: entry.uuid } });
    window.close();
  };

  const handleCopyPassword = async (entry: EntryOption) => {
    await browser.runtime.sendMessage({ action: 'COPY_PASSWORD', payload: { entryUuid: entry.uuid } });
    setCopiedUuid(`pw-${entry.uuid}`);
    setTimeout(() => setCopiedUuid(null), 2000);
  };

  const handleShowOtp = async (entry: EntryOption) => {
    if (otpStates[entry.uuid]) {
      await navigator.clipboard.writeText(otpStates[entry.uuid].code);
      setCopiedUuid(`otp-${entry.uuid}`);
      setTimeout(() => setCopiedUuid(null), 2000);
    } else {
      await refreshOtp(entry.uuid);
    }
  };

  const handleGeneratePassword = async () => {
    const response = await browser.runtime.sendMessage({ action: 'GENERATE_PASSWORD', payload: { mode: 'random', length: 20 } });
    if (response?.success && response.data) {
      const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
      if (tab?.id) await browser.tabs.sendMessage(tab.id, { action: 'FILL_GENERATED_PASSWORD', payload: { password: response.data } });
      window.close();
    }
  };

  const openApp = () => browser.runtime.sendMessage({ action: 'OPEN_APP' });

  const displayEntries = searchQuery.trim() ? searchResults : entries;
  const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  const getHostname = (url: string) => { try { return new URL(url).hostname; } catch { return url; } };

  return (
    <div className={`popup${isDark ? ' dark' : ''}`}>
      <div className="header">
        <span className="logo">🔐</span>
        <span className="title">{t('appName')}</span>
        {vaultStatus?.vaultName && <span className="vault-name">{vaultStatus.vaultName}</span>}
        <button className="icon-btn" onClick={openApp} title={t('openApp')} aria-label={t('openApp')}>↗</button>
      </div>

      {view === 'entries' && (
        <div className="search-bar">
          <span className="search-icon" aria-hidden="true">🔍</span>
          <input
            ref={searchRef}
            type="search"
            className="search-input"
            placeholder={t('searchPlaceholder')}
            value={searchQuery}
            onChange={e => { setSearchQuery(e.target.value); setFocusedIndex(0); }}
            aria-label={t('searchPlaceholder')}
          />
          <button className="icon-btn" onClick={handleGeneratePassword} title={t('generate')} aria-label={t('generate')}>⚡</button>
        </div>
      )}

      <div className="content">
        {view === 'loading' && (
          <div className="state-view"><span className="state-icon">⏳</span><p>{t('connecting')}</p></div>
        )}
        {view === 'disconnected' && (
          <div className="state-view">
            <span className="state-icon">🔌</span>
            <p className="state-title">{t('notRunning')}</p>
            <p className="state-desc">{t('notRunningDesc')}</p>
            <button className="btn-primary" onClick={openApp}>{t('openApp')}</button>
          </div>
        )}
        {view === 'locked' && (
          <div className="state-view">
            <span className="state-icon">🔒</span>
            <p className="state-title">{t('vaultLocked')}</p>
            <p className="state-desc">{t('vaultLockedDesc')}</p>
            <button className="btn-primary" onClick={openApp}>{t('unlockVault')}</button>
          </div>
        )}
        {view === 'save-prompt' && savePromptData && (
          <div className="save-prompt">
            <span className="save-icon">💾</span>
            <p className="save-title">{t('savePassword')}</p>
            <p className="save-desc">{t('savePasswordDesc')}</p>
            <p className="save-url">{savePromptData.url}</p>
            <p className="save-username">{savePromptData.username}</p>
            <div className="save-actions">
              <button className="btn-secondary" onClick={() => { setSavePromptData(null); setView('entries'); }}>{t('dismiss')}</button>
              <button className="btn-primary" onClick={async () => {
                await browser.runtime.sendMessage({ action: 'SAVE_CREDENTIALS', payload: savePromptData });
                setSavePromptData(null); setView('entries');
              }}>{t('save')}</button>
            </div>
          </div>
        )}
        {view === 'entries' && (
          <div>
            {!searchQuery && recentEntries.length > 0 && (
              <div className="section">
                <p className="section-label">{t('recentlyUsed')}</p>
                {recentEntries.map(entry => (
                  <EntryRow key={`r-${entry.uuid}`} entry={entry} isFocused={false}
                    copiedUuid={copiedUuid} otpState={otpStates[entry.uuid]}
                    onFill={() => handleFill(entry)} onCopyPassword={() => handleCopyPassword(entry)}
                    onShowOtp={() => handleShowOtp(entry)} t={t} />
                ))}
              </div>
            )}
            {displayEntries.length === 0 ? (
              <div className="state-view">
                <span className="state-icon">🔍</span>
                <p className="state-desc">{searchQuery ? t('noMatches') : `${t('noMatchesFor')} ${getHostname(currentUrl)}`}</p>
              </div>
            ) : (
              <div className="section">
                {!searchQuery && (
                  <p className="section-label">
                    {displayEntries.length} {t('matchesFor')} <strong>{getHostname(currentUrl)}</strong>
                  </p>
                )}
                <div role="list">
                  {displayEntries.map((entry, idx) => (
                    <EntryRow key={entry.uuid} entry={entry} isFocused={idx === focusedIndex}
                      copiedUuid={copiedUuid} otpState={otpStates[entry.uuid]}
                      onFill={() => handleFill(entry)} onCopyPassword={() => handleCopyPassword(entry)}
                      onShowOtp={() => handleShowOtp(entry)} t={t} />
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      <style>{`
        *{box-sizing:border-box;margin:0;padding:0}
        body{width:340px;min-height:200px}
        .popup{display:flex;flex-direction:column;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;font-size:14px;color:#111827;background:#fff}
        .popup.dark{color:#f9fafb;background:#1f2937}
        .header{display:flex;align-items:center;gap:8px;padding:10px 14px;background:#f9fafb;border-bottom:1px solid #e5e7eb}
        .popup.dark .header{background:#111827;border-color:#374151}
        .logo{font-size:18px}.title{font-weight:700;font-size:15px}
        .vault-name{flex:1;font-size:11px;color:#6b7280;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
        .icon-btn{background:none;border:none;cursor:pointer;color:#6b7280;font-size:15px;padding:3px 6px;border-radius:4px}
        .icon-btn:hover{background:#f3f4f6}
        .popup.dark .icon-btn:hover{background:#374151}
        .search-bar{display:flex;align-items:center;gap:8px;padding:8px 12px;border-bottom:1px solid #e5e7eb}
        .popup.dark .search-bar{border-color:#374151}
        .search-icon{font-size:12px;color:#9ca3af}
        .search-input{flex:1;border:none;outline:none;font-size:13px;color:inherit;background:none}
        .search-input::placeholder{color:#9ca3af}
        .content{flex:1;overflow-y:auto;max-height:380px}
        .state-view{display:flex;flex-direction:column;align-items:center;gap:8px;padding:24px 16px;text-align:center}
        .state-icon{font-size:32px}.state-title{font-weight:600;font-size:15px}
        .state-desc{font-size:13px;color:#6b7280;line-height:1.5}
        .btn-primary{background:#2563eb;color:white;border:none;border-radius:8px;padding:8px 16px;font-size:13px;font-weight:500;cursor:pointer;margin-top:4px}
        .btn-primary:hover{background:#1d4ed8}
        .btn-secondary{background:#f3f4f6;color:#374151;border:none;border-radius:8px;padding:8px 16px;font-size:13px;cursor:pointer}
        .section{display:flex;flex-direction:column}
        .section-label{font-size:11px;color:#9ca3af;padding:8px 14px 4px;font-weight:500;text-transform:uppercase;letter-spacing:.05em}
        .entry-row{display:flex;align-items:center;border-bottom:1px solid #f3f4f6}
        .popup.dark .entry-row{border-color:#374151}
        .entry-row.focused{background:#eff6ff}
        .popup.dark .entry-row.focused{background:#1e3a5f}
        .entry-main{flex:1;display:flex;flex-direction:column;align-items:flex-start;padding:9px 14px;background:none;border:none;cursor:pointer;text-align:left;gap:2px;min-width:0}
        .entry-main:hover{background:#f9fafb}
        .popup.dark .entry-main:hover{background:#374151}
        .entry-main:focus-visible{outline:2px solid #2563eb;outline-offset:-2px}
        .entry-title-row{display:flex;align-items:center;gap:5px;width:100%}
        .entry-title{font-weight:500;color:inherit;font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:160px}
        .entry-username{font-size:11px;color:#6b7280;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
        .badge{font-size:9px;padding:1px 5px;border-radius:4px;font-weight:600;white-space:nowrap;flex-shrink:0}
        .badge-passkey{background:#f0fdf4;color:#16a34a}
        .entry-actions{display:flex;gap:1px;padding-right:6px;flex-shrink:0}
        .action-btn{background:none;border:none;cursor:pointer;padding:5px 7px;border-radius:5px;font-size:13px;color:#9ca3af;display:flex;flex-direction:column;align-items:center;gap:1px}
        .action-btn:hover{background:#f3f4f6;color:#374151}
        .popup.dark .action-btn:hover{background:#374151;color:#f9fafb}
        .action-btn.copied{color:#16a34a}
        .otp-code{font-size:11px;font-family:monospace;font-weight:700;color:#2563eb}
        .otp-timer{font-size:9px;color:#9ca3af}
        .otp-urgent .otp-code{color:#dc2626}
        .otp-urgent .otp-timer{color:#dc2626}
        .save-prompt{display:flex;flex-direction:column;align-items:center;gap:8px;padding:20px 16px;text-align:center}
        .save-icon{font-size:28px}.save-title{font-weight:600;font-size:15px}
        .save-desc{font-size:12px;color:#6b7280}.save-url{font-size:12px;color:#2563eb;font-weight:500}
        .save-username{font-size:12px;color:#374151}.save-actions{display:flex;gap:8px;margin-top:4px}
      `}</style>
    </div>
  );
}

// ─── EntryRow sub-component ───────────────────────────────────────────────────

function EntryRow({
  entry, isFocused, copiedUuid, otpState, onFill, onCopyPassword, onShowOtp, t,
}: {
  entry: EntryOption;
  isFocused: boolean;
  copiedUuid: string | null;
  otpState?: OtpState;
  onFill: () => void;
  onCopyPassword: () => void;
  onShowOtp: () => void;
  t: (key: string) => string;
}) {
  const pwCopied = copiedUuid === `pw-${entry.uuid}`;
  const otpCopied = copiedUuid === `otp-${entry.uuid}`;
  const isUrgent = otpState && otpState.remaining <= 5;

  return (
    <div className={`entry-row${isFocused ? ' focused' : ''}`} role="listitem">
      <button className="entry-main" onClick={onFill} aria-label={`${t('fill')} ${entry.title}`}>
        <div className="entry-title-row">
          <span className="entry-title">{entry.title}</span>
          {entry.hasPasskey && <span className="badge badge-passkey">🔑 {t('passkey')}</span>}
        </div>
        <span className="entry-username">{entry.username}</span>
      </button>
      <div className="entry-actions">
        {/* Copy password */}
        <button className={`action-btn${pwCopied ? ' copied' : ''}`}
          onClick={onCopyPassword} title={t('copyPassword')} aria-label={`${t('copyPassword')} ${entry.title}`}>
          {pwCopied ? '✓' : '⎘'}
        </button>
        {/* OTP */}
        {entry.hasOtp && (
          <button className={`action-btn${otpCopied ? ' copied' : ''}${isUrgent ? ' otp-urgent' : ''}`}
            onClick={onShowOtp} title={t('copyOtp')} aria-label={`${t('copyOtp')} ${entry.title}`}>
            {otpState ? (
              <>
                <span className="otp-code">{otpState.code.slice(0, 3)} {otpState.code.slice(3)}</span>
                <span className="otp-timer">{otpState.remaining}s</span>
              </>
            ) : '⏱'}
          </button>
        )}
      </div>
    </div>
  );
}
