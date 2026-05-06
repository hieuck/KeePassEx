/**
 * Settings page
 */
import React from 'react';
import { useNavigate } from 'react-router-dom';
import { useSettingsStore } from '../store/settings';

export function SettingsPage() {
  const { settings, update } = useSettingsStore();
  const navigate = useNavigate();
  const isVi = settings.language === 'vi';

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h2>{isVi ? '⚙️ Cài đặt' : '⚙️ Settings'}</h2>
      </div>

      <div className="settings-content">
        {/* General */}
        <SettingsSection title={isVi ? 'Chung' : 'General'}>
          <SettingRow label={isVi ? 'Ngôn ngữ' : 'Language'}>
            <select
              className="settings-select"
              value={settings.language}
              onChange={e => update({ language: e.target.value })}
              aria-label="Language"
            >
              <option value="en">English</option>
              <option value="vi">Tiếng Việt</option>
            </select>
          </SettingRow>

          <SettingRow label={isVi ? 'Giao diện' : 'Theme'}>
            <select
              className="settings-select"
              value={settings.theme}
              onChange={e => update({ theme: e.target.value })}
              aria-label="Theme"
            >
              <option value="system">{isVi ? 'Theo hệ thống' : 'System Default'}</option>
              <option value="light">{isVi ? 'Sáng' : 'Light'}</option>
              <option value="dark">{isVi ? 'Tối' : 'Dark'}</option>
            </select>
          </SettingRow>
        </SettingsSection>

        {/* Security */}
        <SettingsSection title={isVi ? 'Bảo mật' : 'Security'}>
          <SettingRow
            label={isVi ? 'Khóa khi thu nhỏ' : 'Lock on minimize'}
            description={
              isVi
                ? 'Tự động khóa kho khi thu nhỏ cửa sổ'
                : 'Automatically lock vault when window is minimized'
            }
          >
            <Toggle
              checked={settings.lockOnMinimize}
              onChange={v => update({ lockOnMinimize: v })}
              aria-label="Lock on minimize"
            />
          </SettingRow>

          <SettingRow
            label={isVi ? 'Khóa khi màn hình khóa' : 'Lock on screen lock'}
            description={
              isVi ? 'Khóa kho khi hệ thống khóa màn hình' : 'Lock vault when system screen locks'
            }
          >
            <Toggle
              checked={settings.lockOnScreenLock}
              onChange={v => update({ lockOnScreenLock: v })}
              aria-label="Lock on screen lock"
            />
          </SettingRow>

          <SettingRow label={isVi ? 'Khóa sau khi không hoạt động' : 'Lock after idle'}>
            <select
              className="settings-select"
              value={settings.lockAfterIdleMinutes ?? 0}
              onChange={e => update({ lockAfterIdleMinutes: Number(e.target.value) || undefined })}
              aria-label="Lock after idle"
            >
              <option value={0}>{isVi ? 'Không bao giờ' : 'Never'}</option>
              <option value={1}>1 {isVi ? 'phút' : 'minute'}</option>
              <option value={5}>5 {isVi ? 'phút' : 'minutes'}</option>
              <option value={10}>10 {isVi ? 'phút' : 'minutes'}</option>
              <option value={30}>30 {isVi ? 'phút' : 'minutes'}</option>
            </select>
          </SettingRow>

          <SettingRow label={isVi ? 'Xóa bộ nhớ tạm sau' : 'Clear clipboard after'}>
            <select
              className="settings-select"
              value={settings.clipboardClearSeconds ?? 0}
              onChange={e => update({ clipboardClearSeconds: Number(e.target.value) || undefined })}
              aria-label="Clear clipboard after"
            >
              <option value={0}>{isVi ? 'Không bao giờ' : 'Never'}</option>
              <option value={10}>10 {isVi ? 'giây' : 'seconds'}</option>
              <option value={30}>30 {isVi ? 'giây' : 'seconds'}</option>
              <option value={60}>60 {isVi ? 'giây' : 'seconds'}</option>
            </select>
          </SettingRow>

          <SettingRow
            label={isVi ? 'Hiện mật khẩu trong danh sách' : 'Show passwords in list'}
            description={
              isVi
                ? 'Hiển thị mật khẩu trong danh sách mục (không khuyến nghị)'
                : 'Show passwords in entry list (not recommended)'
            }
          >
            <Toggle
              checked={settings.showPasswordInList}
              onChange={v => update({ showPasswordInList: v })}
              aria-label="Show passwords in list"
            />
          </SettingRow>
        </SettingsSection>

        {/* App behavior */}
        <SettingsSection title={isVi ? 'Ứng dụng' : 'Application'}>
          <SettingRow label={isVi ? 'Thu nhỏ vào khay hệ thống' : 'Minimize to system tray'}>
            <Toggle
              checked={settings.minimizeToTray}
              onChange={v => update({ minimizeToTray: v })}
              aria-label="Minimize to tray"
            />
          </SettingRow>

          <SettingRow label={isVi ? 'Khởi động thu nhỏ' : 'Start minimized'}>
            <Toggle
              checked={settings.startMinimized}
              onChange={v => update({ startMinimized: v })}
              aria-label="Start minimized"
            />
          </SettingRow>

          <SettingRow label={isVi ? 'Kiểm tra cập nhật' : 'Check for updates'}>
            <Toggle
              checked={settings.checkForUpdates}
              onChange={v => update({ checkForUpdates: v })}
              aria-label="Check for updates"
            />
          </SettingRow>
        </SettingsSection>

        {/* Integration */}
        <SettingsSection title={isVi ? 'Tích hợp' : 'Integration'}>
          <SettingRow
            label={isVi ? 'Tích hợp trình duyệt' : 'Browser Integration'}
            description={
              isVi
                ? 'Bật hỗ trợ tiện ích mở rộng trình duyệt KeePassEx'
                : 'Enable KeePassEx browser extension support'
            }
          >
            <Toggle
              checked={settings.browserIntegration}
              onChange={v => update({ browserIntegration: v })}
              aria-label="Browser integration"
            />
          </SettingRow>

          <SettingRow
            label="SSH Agent"
            description={isVi ? 'Chạy SSH agent tích hợp' : 'Run built-in SSH agent'}
          >
            <Toggle
              checked={settings.sshAgentEnabled}
              onChange={v => update({ sshAgentEnabled: v })}
              aria-label="SSH agent"
            />
          </SettingRow>
        </SettingsSection>

        {/* Advanced */}
        <SettingsSection title={isVi ? 'Nâng cao' : 'Advanced'}>
          <NavSettingRow
            label={isVi ? '💾 Sao lưu tự động' : '💾 Scheduled Backup'}
            description={
              isVi ? 'Tự động sao lưu kho theo lịch' : 'Automatically backup vault on schedule'
            }
            href="/settings/backup"
          />
          <NavSettingRow
            label={isVi ? '🔑 Khóa phần cứng' : '🔑 Hardware Key'}
            description={isVi ? 'YubiKey, FIDO2, Smart Card' : 'YubiKey, FIDO2, Smart Card'}
            href="/settings/hardware-key"
          />
          <NavSettingRow
            label={isVi ? '🆘 Truy cập khẩn cấp' : '🆘 Emergency Access'}
            description={isVi ? 'Quản lý liên hệ tin cậy' : 'Manage trusted contacts'}
            href="/emergency-access"
          />
          <NavSettingRow
            label={isVi ? '🔧 Plugin' : '🔧 Plugins'}
            description={isVi ? 'Cài đặt và quản lý plugin' : 'Install and manage plugins'}
            href="/plugins"
          />
          <NavSettingRow
            label={isVi ? '📊 Thống kê kho' : '📊 Vault Statistics'}
            description={isVi ? 'Xem thống kê chi tiết về kho' : 'View detailed vault statistics'}
            href="/settings/statistics"
          />
          <NavSettingRow
            label={isVi ? '🛡️ Chính sách mật khẩu' : '🛡️ Password Policies'}
            description={isVi ? 'Định nghĩa yêu cầu mật khẩu' : 'Define password requirements'}
            href="/settings/password-policy"
          />
          <NavSettingRow
            label={isVi ? '📋 Nhật ký kiểm tra' : '📋 Audit Log'}
            description={
              isVi ? 'Theo dõi truy cập và thay đổi kho' : 'Track vault access and changes'
            }
            href="/settings/audit-log"
          />
          <NavSettingRow
            label={isVi ? '🔀 So sánh kho' : '🔀 Compare Vaults'}
            description={isVi ? 'So sánh và hợp nhất hai kho' : 'Diff and merge two vaults'}
            href="/vault/compare"
          />
          <NavSettingRow
            label={isVi ? '⚙️ Cài đặt nâng cao' : '⚙️ Advanced Settings'}
            description={isVi ? 'Lịch sử, bộ nhớ, auto-type' : 'History, memory, auto-type'}
            href="/settings/advanced"
          />
        </SettingsSection>

        {/* About */}
        <SettingsSection title={isVi ? 'Giới thiệu' : 'About'}>
          <div className="about-info">
            <div className="about-row">
              <span className="about-label">{isVi ? 'Phiên bản' : 'Version'}</span>
              <span className="about-value">1.0.0</span>
            </div>
            <div className="about-row">
              <span className="about-label">{isVi ? 'Giấy phép' : 'License'}</span>
              <span className="about-value">GPL-3.0</span>
            </div>
            <div className="about-row">
              <span className="about-label">{isVi ? 'Mã nguồn' : 'Source code'}</span>
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
