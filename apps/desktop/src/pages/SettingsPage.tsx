/**
 * Settings page — General, Security, Integration, Advanced, About + Check for Updates
 */
import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import { useSettingsStore } from '../store/settings';

interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version?: string;
  release_notes?: string;
  download_url?: string;
  release_date?: string;
}

export function SettingsPage() {
  const { settings, update } = useSettingsStore();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [appVersion, setAppVersion] = useState('0.1.0');
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  // Load app version on mount
  useEffect(() => {
    invoke<string>('get_app_version')
      .then(v => setAppVersion(v))
      .catch(() => {});
  }, []);

  const handleCheckForUpdates = async () => {
    setCheckingUpdate(true);
    setUpdateError(null);
    setUpdateInfo(null);
    try {
      const info = await invoke<UpdateInfo>('check_for_updates');
      setUpdateInfo(info);
    } catch (e: unknown) {
      setUpdateError(e instanceof Error ? e.message : String(e));
    } finally {
      setCheckingUpdate(false);
    }
  };

  const handleOpenUrl = (url: string) => {
    open(url).catch(() => window.open(url, '_blank'));
  };

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h2>⚙️ {t('settings.title')}</h2>
      </div>

      <div className="settings-content">
        {/* ── General ── */}
        <SettingsSection title={t('settings.general')}>
          <SettingRow label={t('settings.language')}>
            <select
              className="settings-select"
              value={settings.language}
              onChange={e => update({ language: e.target.value })}
              aria-label={t('settings.language')}
            >
              <option value="en">🇺🇸 English</option>
              <option value="vi">🇻🇳 Tiếng Việt</option>
              <option value="zh">🇨🇳 简体中文</option>
              <option value="ja">🇯🇵 日本語</option>
              <option value="ko">🇰🇷 한국어</option>
              <option value="es">🇪🇸 Español</option>
              <option value="fr">🇫🇷 Français</option>
              <option value="de">🇩🇪 Deutsch</option>
              <option value="pt">🇧🇷 Português</option>
              <option value="ru">🇷🇺 Русский</option>
            </select>
          </SettingRow>

          <SettingRow label={t('settings.theme')}>
            <select
              className="settings-select"
              value={settings.theme}
              onChange={e => update({ theme: e.target.value })}
              aria-label={t('settings.theme')}
            >
              <option value="system">{t('settings.themeSystem')}</option>
              <option value="light">{t('settings.themeLight')}</option>
              <option value="dark">{t('settings.themeDark')}</option>
              <option value="oled">{t('settings.themeOled')}</option>
            </select>
          </SettingRow>

          <SettingRow label={t('settings.minimizeToTray')}>
            <Toggle
              checked={settings.minimizeToTray}
              onChange={v => update({ minimizeToTray: v })}
              aria-label={t('settings.minimizeToTray')}
            />
          </SettingRow>

          <SettingRow label={t('settings.startMinimized')}>
            <Toggle
              checked={settings.startMinimized}
              onChange={v => update({ startMinimized: v })}
              aria-label={t('settings.startMinimized')}
            />
          </SettingRow>
        </SettingsSection>

        {/* ── Security ── */}
        <SettingsSection title={t('settings.security')}>
          <SettingRow label={t('settings.lockOnMinimize')}>
            <Toggle
              checked={settings.lockOnMinimize}
              onChange={v => update({ lockOnMinimize: v })}
              aria-label={t('settings.lockOnMinimize')}
            />
          </SettingRow>

          <SettingRow label={t('settings.lockOnScreenLock')}>
            <Toggle
              checked={settings.lockOnScreenLock}
              onChange={v => update({ lockOnScreenLock: v })}
              aria-label={t('settings.lockOnScreenLock')}
            />
          </SettingRow>

          <SettingRow label={t('settings.lockAfterIdle')}>
            <select
              className="settings-select"
              value={settings.lockAfterIdleMinutes ?? 0}
              onChange={e => update({ lockAfterIdleMinutes: Number(e.target.value) || undefined })}
              aria-label={t('settings.lockAfterIdle')}
            >
              <option value={0}>{t('settings.clipboardClearNever')}</option>
              <option value={1}>{t('settings.lockAfterIdleMinutes', { minutes: 1 })}</option>
              <option value={5}>{t('settings.lockAfterIdleMinutes', { minutes: 5 })}</option>
              <option value={10}>{t('settings.lockAfterIdleMinutes', { minutes: 10 })}</option>
              <option value={30}>{t('settings.lockAfterIdleMinutes', { minutes: 30 })}</option>
            </select>
          </SettingRow>

          <SettingRow label={t('settings.clipboardClearDelay')}>
            <select
              className="settings-select"
              value={settings.clipboardClearSeconds ?? 10}
              onChange={e => update({ clipboardClearSeconds: Number(e.target.value) || undefined })}
              aria-label={t('settings.clipboardClearDelay')}
            >
              <option value={0}>{t('settings.clipboardClearNever')}</option>
              <option value={10}>
                {t('settings.clipboardClearDelaySeconds', { seconds: 10 })}
              </option>
              <option value={30}>
                {t('settings.clipboardClearDelaySeconds', { seconds: 30 })}
              </option>
              <option value={60}>
                {t('settings.clipboardClearDelaySeconds', { seconds: 60 })}
              </option>
            </select>
          </SettingRow>

          <SettingRow label={t('settings.showPasswordInList')}>
            <Toggle
              checked={settings.showPasswordInList}
              onChange={v => update({ showPasswordInList: v })}
              aria-label={t('settings.showPasswordInList')}
            />
          </SettingRow>
        </SettingsSection>

        {/* ── Integration ── */}
        <SettingsSection title={t('settings.advanced')}>
          <SettingRow
            label={t('settings.browserIntegration')}
            description={t('settings.browserIntegrationDesc')}
          >
            <Toggle
              checked={settings.browserIntegration}
              onChange={v => update({ browserIntegration: v })}
              aria-label={t('settings.browserIntegration')}
            />
          </SettingRow>

          <SettingRow label={t('settings.sshAgent')} description={t('settings.sshAgentDesc')}>
            <Toggle
              checked={settings.sshAgentEnabled}
              onChange={v => update({ sshAgentEnabled: v })}
              aria-label={t('settings.sshAgent')}
            />
          </SettingRow>
        </SettingsSection>

        {/* ── Advanced ── */}
        <SettingsSection title={t('advanced.title')}>
          <NavSettingRow
            label={`🔐 ${t('quantumResistant.title')}`}
            description={t('quantumResistant.subtitle')}
            href="/settings/security"
          />
          <NavSettingRow
            label={`💾 ${t('scheduledBackup.title')}`}
            description={t('scheduledBackup.subtitle')}
            href="/settings/backup"
          />
          <NavSettingRow
            label={`🔑 ${t('hardwareKey.title')}`}
            description={t('hardwareKey.subtitle')}
            href="/settings/hardware-key"
          />
          <NavSettingRow
            label={`🆘 ${t('emergencyAccess.title')}`}
            description={t('emergencyAccess.subtitle')}
            href="/emergency-access"
          />
          <NavSettingRow
            label={`🔧 ${t('plugins.title')}`}
            description={t('plugins.subtitle')}
            href="/plugins"
          />
          <NavSettingRow
            label={`📊 ${t('statistics.title')}`}
            description={t('statistics.title')}
            href="/settings/statistics"
          />
          <NavSettingRow
            label={`🛡️ ${t('passwordPolicy.title')}`}
            description={t('passwordPolicy.subtitle')}
            href="/settings/password-policy"
          />
          <NavSettingRow
            label={`📋 ${t('auditLog.title')}`}
            description={t('auditLog.subtitle')}
            href="/settings/audit-log"
          />
          <NavSettingRow
            label={`🔀 ${t('vaultCompare.title')}`}
            description={t('vaultCompare.subtitle')}
            href="/vault/compare"
          />
          <NavSettingRow
            label={`📈 ${t('analytics.title')}`}
            description={t('analytics.subtitle')}
            href="/settings/analytics"
          />
          <NavSettingRow
            label={`🕵️ ${t('steganography.title')}`}
            description={t('steganography.subtitle')}
            href="/settings/steganography"
          />
        </SettingsSection>

        {/* ── Updates ── */}
        <SettingsSection title={t('settings.checkForUpdates')}>
          <SettingRow
            label={t('settings.checkForUpdates')}
            description={t('settings.checkForUpdates')}
          >
            <Toggle
              checked={settings.checkForUpdates}
              onChange={v => update({ checkForUpdates: v })}
              aria-label={t('settings.checkForUpdates')}
            />
          </SettingRow>

          {/* Check now button */}
          <div className="update-section">
            <button
              className="btn btn-secondary"
              onClick={handleCheckForUpdates}
              disabled={checkingUpdate}
              aria-busy={checkingUpdate}
            >
              {checkingUpdate ? `⏳ ${t('common.loading')}` : `🔄 ${t('settings.checkForUpdates')}`}
            </button>

            {/* Update result */}
            {updateError && (
              <div className="update-error" role="alert">
                ⚠️ {updateError}
              </div>
            )}

            {updateInfo && !updateInfo.available && (
              <div className="update-ok" role="status">
                ✅ {t('app.version', { version: updateInfo.current_version })} — {t('common.ok')}
              </div>
            )}

            {updateInfo?.available && (
              <div className="update-available" role="status">
                <div className="update-available-header">
                  🎉{' '}
                  <strong>
                    {t('settings.version')} {updateInfo.latest_version}
                  </strong>
                  {updateInfo.release_date && (
                    <span className="update-date"> · {updateInfo.release_date}</span>
                  )}
                </div>
                {updateInfo.release_notes && (
                  <p className="update-notes">{updateInfo.release_notes}</p>
                )}
                {updateInfo.download_url && (
                  <button
                    className="btn btn-primary"
                    onClick={() => handleOpenUrl(updateInfo.download_url!)}
                  >
                    ⬇️ Download
                  </button>
                )}
                <button
                  className="btn btn-secondary"
                  onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex/releases')}
                >
                  📋 View Release Notes
                </button>
              </div>
            )}
          </div>
        </SettingsSection>

        {/* ── Help ── */}
        <SettingsSection title={t('common.help')}>
          <div className="help-grid">
            <HelpLink
              icon="📖"
              label={t('settings.about')}
              description={t('settings.openSourceLicenses')}
              onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex/tree/main/docs')}
            />
            <HelpLink
              icon="🐛"
              label={t('settings.reportBug')}
              description="GitHub Issues"
              onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex/issues/new')}
            />
            <HelpLink
              icon="💬"
              label="Discussions"
              description="GitHub Discussions"
              onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex/discussions')}
            />
            <HelpLink
              icon="🔒"
              label={t('settings.security')}
              description="security@keepassex.app"
              onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex/security')}
            />
            <HelpLink
              icon="📝"
              label={t('settings.changelog')}
              description="CHANGELOG.md"
              onClick={() =>
                handleOpenUrl('https://github.com/keepassex/keepassex/blob/main/CHANGELOG.md')
              }
            />
            <HelpLink
              icon="⌨️"
              label={t('settings.defaultAutoTypeSequence')}
              description="Ctrl+K"
              onClick={() => navigate('/settings/statistics')}
            />
          </div>
        </SettingsSection>

        {/* ── About ── */}
        <SettingsSection title={t('settings.about')}>
          <div className="about-card">
            <div className="about-logo">🔐</div>
            <div className="about-info">
              <h3 className="about-name">{t('app.name')}</h3>
              <p className="about-tagline">{t('app.tagline')}</p>
            </div>

            <div className="about-rows">
              <AboutRow label={t('settings.version')} value={`v${appVersion}`} />
              <AboutRow label="License" value="GPL-3.0" />
              <AboutRow label="Format" value="KDBX 4.x" />
              <AboutRow label="Languages" value="10 (EN/VI/ZH/JA/KO/ES/FR/DE/PT/RU)" />
              <AboutRow label="Platforms" value="Desktop · Mobile · Watch · Browser · CLI" />
              <AboutRow
                label={t('settings.openSourceLicenses')}
                value="GitHub"
                link="https://github.com/keepassex/keepassex"
                onLinkClick={handleOpenUrl}
              />
            </div>

            <div className="about-actions">
              <button
                className="btn btn-secondary"
                onClick={() => handleOpenUrl('https://github.com/keepassex/keepassex')}
              >
                ⭐ Star on GitHub
              </button>
              <button
                className="btn btn-secondary"
                onClick={() =>
                  handleOpenUrl('https://github.com/keepassex/keepassex/blob/main/CONTRIBUTING.md')
                }
              >
                🤝 Contribute
              </button>
            </div>
          </div>
        </SettingsSection>
      </div>

      <style>{`
        .settings-page { display:flex; flex-direction:column; height:100%; overflow:hidden; }
        .settings-header { padding:var(--space-md) var(--space-xl); border-bottom:1px solid var(--color-border); flex-shrink:0; }
        .settings-header h2 { font-size:16px; font-weight:600; }
        .settings-content { flex:1; overflow-y:auto; padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-xl); max-width:560px; }
        .settings-section { display:flex; flex-direction:column; gap:2px; }
        .settings-section-title { font-size:11px; font-weight:600; text-transform:uppercase; letter-spacing:.08em; color:var(--color-text-secondary); padding:0 var(--space-md); margin-bottom:var(--space-xs); }
        .settings-row { display:flex; align-items:center; justify-content:space-between; padding:var(--space-md); background:var(--color-bg-secondary); border-radius:var(--radius-sm); gap:var(--space-lg); }
        .settings-row-info { flex:1; }
        .settings-row-label { font-size:14px; color:var(--color-text); }
        .settings-row-desc { font-size:12px; color:var(--color-text-secondary); margin-top:2px; }
        .settings-select { background:var(--color-bg); border:1px solid var(--color-border); border-radius:var(--radius-sm); padding:var(--space-xs) var(--space-sm); font-size:13px; color:var(--color-text); cursor:pointer; }
        .toggle { width:44px; height:24px; border-radius:12px; border:none; cursor:pointer; position:relative; transition:background .2s; flex-shrink:0; }
        .toggle-thumb { position:absolute; top:2px; width:20px; height:20px; border-radius:50%; background:white; transition:left .2s; box-shadow:0 1px 3px rgba(0,0,0,.2); }
        /* Updates */
        .update-section { padding:var(--space-md); background:var(--color-bg-secondary); border-radius:var(--radius-sm); display:flex; flex-direction:column; gap:var(--space-md); }
        .update-error { font-size:13px; color:var(--color-danger); padding:var(--space-sm); background:rgba(220,38,38,.08); border-radius:var(--radius-sm); }
        .update-ok { font-size:13px; color:var(--color-success); }
        .update-available { display:flex; flex-direction:column; gap:var(--space-sm); padding:var(--space-md); background:rgba(37,99,235,.06); border:1px solid rgba(37,99,235,.2); border-radius:var(--radius-md); }
        .update-available-header { font-size:14px; }
        .update-date { font-size:12px; color:var(--color-text-secondary); }
        .update-notes { font-size:12px; color:var(--color-text-secondary); line-height:1.5; max-height:80px; overflow-y:auto; }
        /* Help */
        .help-grid { display:grid; grid-template-columns:1fr 1fr; gap:var(--space-sm); }
        .help-link { display:flex; flex-direction:column; gap:4px; padding:var(--space-md); background:var(--color-bg-secondary); border-radius:var(--radius-md); border:1px solid var(--color-border); cursor:pointer; text-align:left; transition:background .1s, border-color .1s; }
        .help-link:hover { background:var(--color-bg-tertiary); border-color:var(--color-primary); }
        .help-link-icon { font-size:20px; }
        .help-link-label { font-size:13px; font-weight:600; color:var(--color-text); }
        .help-link-desc { font-size:11px; color:var(--color-text-secondary); }
        /* About */
        .about-card { background:var(--color-bg-secondary); border-radius:var(--radius-lg); border:1px solid var(--color-border); padding:var(--space-xl); display:flex; flex-direction:column; gap:var(--space-lg); }
        .about-logo { font-size:48px; text-align:center; }
        .about-info { text-align:center; }
        .about-name { font-size:20px; font-weight:700; color:var(--color-text); }
        .about-tagline { font-size:13px; color:var(--color-text-secondary); margin-top:4px; }
        .about-rows { display:flex; flex-direction:column; gap:var(--space-xs); }
        .about-row { display:flex; justify-content:space-between; align-items:center; font-size:13px; padding:var(--space-xs) 0; border-bottom:1px solid var(--color-border); }
        .about-row:last-child { border-bottom:none; }
        .about-row-label { color:var(--color-text-secondary); }
        .about-row-value { color:var(--color-text); font-weight:500; }
        .about-row-link { color:var(--color-primary); cursor:pointer; background:none; border:none; font-size:13px; font-weight:500; padding:0; }
        .about-row-link:hover { text-decoration:underline; }
        .about-actions { display:flex; gap:var(--space-sm); justify-content:center; flex-wrap:wrap; }
        /* Nav rows */
        .nav-setting-row { cursor:pointer; width:100%; text-align:left; border:none; }
      `}</style>
    </div>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SettingsSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="settings-section">
      <p className="settings-section-title">{title}</p>
      {children}
    </div>
  );
}

function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="settings-row">
      <div className="settings-row-info">
        <p className="settings-row-label">{label}</p>
        {description && <p className="settings-row-desc">{description}</p>}
      </div>
      {children}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
  'aria-label': ariaLabel,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  'aria-label': string;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      className="toggle"
      style={{ background: checked ? 'var(--color-primary)' : 'var(--color-border)' }}
      onClick={() => onChange(!checked)}
    >
      <div className="toggle-thumb" style={{ left: checked ? 22 : 2 }} />
    </button>
  );
}

function NavSettingRow({
  label,
  description,
  href,
}: {
  label: string;
  description?: string;
  href: string;
}) {
  const navigate = useNavigate();
  return (
    <button
      className="settings-row nav-setting-row"
      onClick={() => navigate(href)}
      aria-label={label}
    >
      <div className="settings-row-info">
        <p className="settings-row-label">{label}</p>
        {description && <p className="settings-row-desc">{description}</p>}
      </div>
      <span style={{ color: 'var(--color-text-tertiary)', fontSize: 18 }}>›</span>
    </button>
  );
}

function HelpLink({
  icon,
  label,
  description,
  onClick,
}: {
  icon: string;
  label: string;
  description: string;
  onClick: () => void;
}) {
  return (
    <button className="help-link" onClick={onClick} aria-label={label}>
      <span className="help-link-icon" aria-hidden="true">
        {icon}
      </span>
      <span className="help-link-label">{label}</span>
      <span className="help-link-desc">{description}</span>
    </button>
  );
}

function AboutRow({
  label,
  value,
  link,
  onLinkClick,
}: {
  label: string;
  value: string;
  link?: string;
  onLinkClick?: (url: string) => void;
}) {
  return (
    <div className="about-row">
      <span className="about-row-label">{label}</span>
      {link && onLinkClick ? (
        <button className="about-row-link" onClick={() => onLinkClick(link)}>
          {value}
        </button>
      ) : (
        <span className="about-row-value">{value}</span>
      )}
    </div>
  );
}
