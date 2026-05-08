/**
 * Main application layout — sidebar + content
 */

import { Outlet, NavLink, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useVaultStore } from '../store/vault';
import { GroupTree } from '../components/GroupTree';
import { SearchBar } from '../components/SearchBar';

interface MainLayoutProps {
  onOpenPalette: () => void;
}

export function MainLayout({ onOpenPalette }: MainLayoutProps) {
  const { meta, lockVault, closeVault, setSearchQuery } = useVaultStore();
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleLock = async () => {
    await lockVault();
    navigate('/unlock');
  };

  const navItems = [
    { to: '/vault', icon: '🔑', label: t('vault.open') },
    { to: '/health', icon: '🛡️', label: t('health.title') },
    { to: '/breach', icon: '🔍', label: t('breach.title') },
    { to: '/generator', icon: '⚡', label: t('generator.title') },
    { to: '/team', icon: '👥', label: t('team.title') },
    { to: '/import-export', icon: '📥', label: `${t('common.import')}/${t('common.export')}` },
    { to: '/sync', icon: '🔄', label: t('sync.title') },
    { to: '/emergency-access', icon: '🆘', label: t('emergencyAccess.title') },
    { to: '/plugins', icon: '🔧', label: t('plugins.title') },
    { to: '/settings', icon: '⚙️', label: t('settings.title') },
  ];

  const settingsItems = [
    { to: '/settings/analytics', icon: '📊', label: t('analytics.title') },
    { to: '/settings/steganography', icon: '🕵️', label: t('steganography.title') },
    { to: '/settings/backup', icon: '💾', label: t('scheduledBackup.title') },
    { to: '/settings/audit-log', icon: '📋', label: t('auditLog.title') },
    { to: '/settings/password-policy', icon: '🔒', label: t('passwordPolicy.title') },
    { to: '/settings/statistics', icon: '📈', label: t('statistics.title') },
    { to: '/vault/compare', icon: '⚖️', label: t('vaultCompare.title') },
  ];

  return (
    <div className="main-layout">
      {/* Sidebar */}
      <aside className="sidebar" role="navigation" aria-label={t('settings.general')}>
        {/* Vault name + palette trigger */}
        <div className="sidebar-header">
          <span className="vault-icon">🔐</span>
          <span className="vault-name">{meta?.name ?? 'KeePassEx'}</span>
          <button
            className="palette-trigger"
            onClick={onOpenPalette}
            title={`${t('common.search')} (Ctrl+K)`}
            aria-label="Open command palette"
          >
            ⌘K
          </button>
        </div>

        {/* Search */}
        <div className="sidebar-search">
          <SearchBar placeholder={t('entry.searchPlaceholder')} onChange={setSearchQuery} />
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
              <span>{item.label}</span>
            </NavLink>
          ))}
        </nav>

        {/* Advanced / Settings sub-nav */}
        <details className="sidebar-advanced">
          <summary className="sidebar-advanced-toggle">
            <span aria-hidden="true">🔧</span>
            <span>{t('advanced.title')}</span>
          </summary>
          <nav className="sidebar-subnav" aria-label="Advanced settings">
            {settingsItems.map(item => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) => `nav-item nav-item-sub ${isActive ? 'active' : ''}`}
              >
                <span aria-hidden="true">{item.icon}</span>
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>
        </details>

        {/* Bottom actions */}
        <div className="sidebar-footer">
          <button
            className="btn-icon"
            onClick={handleLock}
            title={t('vault.lock')}
            aria-label={t('vault.lock')}
          >
            🔒
          </button>
          <span className="sidebar-footer-sep" />
          <button
            className="btn-icon"
            onClick={closeVault}
            title={t('vault.close')}
            aria-label={t('vault.close')}
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
