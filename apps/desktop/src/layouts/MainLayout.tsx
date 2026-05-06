/**
 * Main application layout — sidebar + content
 */
import React from 'react';
import { Outlet, NavLink, useNavigate } from 'react-router-dom';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';
import { GroupTree } from '../components/GroupTree';
import { SearchBar } from '../components/SearchBar';

interface MainLayoutProps {
  onOpenPalette: () => void;
}

export function MainLayout({ onOpenPalette }: MainLayoutProps) {
  const { meta, lockVault, closeVault, setSearchQuery } = useVaultStore();
  const { settings } = useSettingsStore();
  const navigate = useNavigate();
  const isVi = settings.language === 'vi';

  const handleLock = async () => {
    await lockVault();
    navigate('/unlock');
  };

  const navItems = [
    { to: '/vault', icon: '🔑', labelEn: 'Vault', labelVi: 'Kho mật khẩu' },
    { to: '/health', icon: '🛡️', labelEn: 'Health', labelVi: 'Sức khỏe' },
    { to: '/breach', icon: '🔍', labelEn: 'Breach', labelVi: 'Rò rỉ' },
    { to: '/generator', icon: '⚡', labelEn: 'Generator', labelVi: 'Tạo mật khẩu' },
    { to: '/team', icon: '👥', labelEn: 'Team', labelVi: 'Nhóm' },
    { to: '/import-export', icon: '📥', labelEn: 'Import/Export', labelVi: 'Nhập/Xuất' },
    { to: '/sync', icon: '🔄', labelEn: 'Sync', labelVi: 'Đồng bộ' },
    { to: '/emergency-access', icon: '🆘', labelEn: 'Emergency Access', labelVi: 'Khẩn cấp' },
    { to: '/plugins', icon: '🔧', labelEn: 'Plugins', labelVi: 'Plugin' },
    { to: '/settings', icon: '⚙️', labelEn: 'Settings', labelVi: 'Cài đặt' },
  ];

  const settingsItems = [
    { to: '/settings/analytics', icon: '📊', labelEn: 'Analytics', labelVi: 'Phân tích' },
    { to: '/settings/steganography', icon: '🕵️', labelEn: 'Steganography', labelVi: 'Ẩn dữ liệu' },
    { to: '/settings/backup', icon: '💾', labelEn: 'Backup', labelVi: 'Sao lưu' },
    { to: '/settings/audit-log', icon: '📋', labelEn: 'Audit Log', labelVi: 'Nhật ký' },
    { to: '/settings/password-policy', icon: '🔒', labelEn: 'Policies', labelVi: 'Chính sách' },
    { to: '/settings/statistics', icon: '📈', labelEn: 'Statistics', labelVi: 'Thống kê' },
    { to: '/vault/compare', icon: '⚖️', labelEn: 'Compare', labelVi: 'So sánh' },
  ];

  return (
    <div className="main-layout">
      {/* Sidebar */}
      <aside className="sidebar" role="navigation" aria-label="Main navigation">
        {/* Vault name + palette trigger */}
        <div className="sidebar-header">
          <span className="vault-icon">🔐</span>
          <span className="vault-name">{meta?.name ?? 'KeePassEx'}</span>
          <button
            className="palette-trigger"
            onClick={onOpenPalette}
            title={isVi ? 'Bảng lệnh (Ctrl+K)' : 'Command palette (Ctrl+K)'}
            aria-label="Open command palette"
          >
            ⌘K
          </button>
        </div>

        {/* Search */}
        <div className="sidebar-search">
          <SearchBar placeholder={isVi ? 'Tìm kiếm...' : 'Search...'} onChange={setSearchQuery} />
        </div>

        {/* Group tree */}
        <div className="sidebar-groups">
          <GroupTree />
        </div>

        {/* Navigation */}
        <nav className="sidebar-nav" aria-label="App sections">
          {navItems.map(item => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}
            >
              <span aria-hidden="true">{item.icon}</span>
              <span>{isVi ? item.labelVi : item.labelEn}</span>
            </NavLink>
          ))}
        </nav>

        {/* Advanced / Settings sub-nav */}
        <details className="sidebar-advanced">
          <summary className="sidebar-advanced-toggle">
            <span aria-hidden="true">🔧</span>
            <span>{isVi ? 'Nâng cao' : 'Advanced'}</span>
          </summary>
          <nav className="sidebar-subnav" aria-label="Advanced settings">
            {settingsItems.map(item => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) => `nav-item nav-item-sub ${isActive ? 'active' : ''}`}
              >
                <span aria-hidden="true">{item.icon}</span>
                <span>{isVi ? item.labelVi : item.labelEn}</span>
              </NavLink>
            ))}
          </nav>
        </details>

        {/* Bottom actions */}
        <div className="sidebar-footer">
          <button
            className="btn-icon"
            onClick={handleLock}
            title={isVi ? 'Khóa kho' : 'Lock vault'}
            aria-label={isVi ? 'Khóa kho' : 'Lock vault'}
          >
            🔒
          </button>
          <span className="sidebar-footer-sep" />
          <button
            className="btn-icon"
            onClick={closeVault}
            title={isVi ? 'Đóng kho' : 'Close vault'}
            aria-label={isVi ? 'Đóng kho' : 'Close vault'}
          >
            ✕
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="main-content" role="main">
        <Outlet />
      </main>

      <style>{`
        .palette-trigger {
          background: var(--color-bg-tertiary);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-sm);
          padding: 2px 6px;
          font-size: 11px;
          color: var(--color-text-secondary);
          cursor: pointer;
          flex-shrink: 0;
          font-family: inherit;
        }
        .palette-trigger:hover { background: var(--color-border); }
        .sidebar-footer-sep { flex: 1; }
      `}</style>
    </div>
  );
}
