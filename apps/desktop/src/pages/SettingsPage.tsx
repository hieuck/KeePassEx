/**
 * Settings page
 */
import React from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useSettingsStore } from '../store/settings';

export function SettingsPage() {
  const { settings, update } = useSettingsStore();
  const navigate = useNavigate();
  const { t } = useTranslation();

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h2>⚙️ {t('settings.title')}</h2>
      </div>

      <div className="settings-content">
        {/* General */}
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
            </select>
          </SettingRow>
        </SettingsSection>

        {/* Security */}
        <SettingsSection title={t('settings.security')}>
          <SettingRow
            label={t('settings.lockOnMinimize')}
            description={t('settings.lockOnMinimize')}
          >
            <Toggle
              checked={settings.lockOnMinimize}
              onChange={v => update({ lockOnMinimize: v })}
              aria-label={t('settings.lockOnMinimize')}
            />
          </SettingRow>

          <SettingRow
            label={t('settings.lockOnScreenLock')}
            description={t('settings.lockOnScreenLock')}
          >
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
              value={settings.clipboardClearSeconds ?? 0}
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

          <SettingRow
            label={t('settings.showPasswordInList')}
            description={t('settings.showPasswordInList')}
          >
            <Toggle
              checked={settings.showPasswordInList}
              onChange={v => update({ showPasswordInList: v })}
              aria-label={t('settings.showPasswordInList')}
            />
          </SettingRow>
        </SettingsSection>

        {/* App behavior */}
        <SettingsSection title={t('settings.general')}>
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

          <SettingRow label={t('settings.checkForUpdates')}>
            <Toggle
              checked={settings.checkForUpdates}
              onChange={v => update({ checkForUpdates: v })}
              aria-label={t('settings.checkForUpdates')}
            />
          </SettingRow>
        </SettingsSection>

        {/* Integration */}
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

        {/* Advanced */}
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
            label={`⚙️ ${t('advanced.title')}`}
            description={t('advanced.database')}
            href="/settings/advanced"
          />
        </SettingsSection>

        {/* About */}
        <SettingsSection title={t('settings.about')}>
          <div className="about-info">
            <div className="about-row">
              <span className="about-label">{t('settings.version')}</span>
              <span className="about-value">1.0.0</span>
            </div>
            <div className="about-row">
              <span className="about-label">{t('settings.openSourceLicenses')}</span>
              <span className="about-value">GPL-3.0</span>
            </div>
            <div className="about-row">
              <span className="about-label">{t('settings.reportBug')}</span>
              <a
                href="https://github.com/keepassex/keepassex"
                target="_blank"
                rel="noopener noreferrer"
                className="about-link"
              >
                GitHub
              </a>
            </div>
          </div>
        </SettingsSection>
      </div>

      <style>{`
        .settings-page { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
        .settings-header {
          padding: var(--space-md) var(--space-xl);
          border-bottom: 1px solid var(--color-border);
          flex-shrink: 0;
        }
        .settings-header h2 { font-size: 16px; font-weight: 600; }
        .settings-content {
          flex: 1;
          overflow-y: auto;
          padding: var(--space-xl);
          display: flex;
          flex-direction: column;
          gap: var(--space-xl);
          max-width: 560px;
        }
        .settings-section { display: flex; flex-direction: column; gap: 2px; }
        .settings-section-title {
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          color: var(--color-text-secondary);
          padding: 0 var(--space-md);
          margin-bottom: var(--space-xs);
        }
        .settings-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: var(--space-md);
          background: var(--color-bg-secondary);
          border-radius: var(--radius-sm);
          gap: var(--space-lg);
        }
        .settings-row-info { flex: 1; }
        .settings-row-label { font-size: 14px; color: var(--color-text); }
        .settings-row-desc { font-size: 12px; color: var(--color-text-secondary); margin-top: 2px; }
        .settings-select {
          background: var(--color-bg);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-sm);
          padding: var(--space-xs) var(--space-sm);
          font-size: 13px;
          color: var(--color-text);
          cursor: pointer;
        }
        .toggle {
          width: 44px;
          height: 24px;
          border-radius: 12px;
          border: none;
          cursor: pointer;
          position: relative;
          transition: background 0.2s;
          flex-shrink: 0;
        }
        .toggle-thumb {
          position: absolute;
          top: 2px;
          width: 20px;
          height: 20px;
          border-radius: 50%;
          background: white;
          transition: left 0.2s;
          box-shadow: 0 1px 3px rgba(0,0,0,0.2);
        }
        .about-info { display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-md); background: var(--color-bg-secondary); border-radius: var(--radius-sm); }
        .about-row { display: flex; justify-content: space-between; font-size: 13px; }
        .about-label { color: var(--color-text-secondary); }
        .about-value { color: var(--color-text); }
        .about-link { color: var(--color-primary); text-decoration: none; }
        .about-link:hover { text-decoration: underline; }
      `}</style>
    </div>
  );
}

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
      style={{ cursor: 'pointer', width: '100%', textAlign: 'left', border: 'none' }}
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
