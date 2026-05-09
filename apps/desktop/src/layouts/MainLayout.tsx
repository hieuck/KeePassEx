/**
 * Main application layout — sidebar + content
 * With vault health score widget and rotation summary badge
 */

import { Outlet, NavLink, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';
import { GroupTree } from '../components/GroupTree';
import { SearchBar } from '../components/SearchBar';

interface MainLayoutProps {
  onOpenPalette: () => void;
}

export function MainLayout({ onOpenPalette }: MainLayoutProps) {
  const { meta, lockVault, closeVault, setSearchQuery, isOpen, isLocked } = useVaultStore();
  const { t } = useTranslation();
  const navigate = useNavigate();

  // Health score for sidebar widget
  const { data: healthScore } = useQuery({
    queryKey: ['health-score-sidebar'],
    queryFn: async () => {
      const report = await invoke<{ score: number }>('audit_vault');
      return report.score;
    },
    enabled: isOpen && !isLocked,
    staleTime: 120_000,
    retry: false,
  });

  // Rotation summary for badge
  const { data: rotationSummary } = useQuery({
    queryKey: ['rotation-summary-sidebar'],
    queryFn: () => invoke<{ overdue: number; soon: number }>('get_rotation_summary'),
    enabled: isOpen && !isLocked,
    staleTime: 120_000,
    retry: false,
  });

  const urgentRotations = (rotationSummary?.overdue ?? 0) + (rotationSummary?.soon ?? 0);

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

        {/* Vault Health Score Widget */}
        {healthScore !== undefined && (
          <button
            className="health-widget"
            onClick={() => navigate('/health')}
            title={`${t('health.title')}: ${healthScore}/100`}
            aria-label={`${t('health.title')}: ${healthScore}/100`}
          >
            <div
              className="health-score-ring"
              style={{
                background: `conic-gradient(${
                  healthScore >= 90
                    ? '#16a34a'
                    : healthScore >= 70
                      ? '#22c55e'
                      : healthScore >= 50
                        ? '#f59e0b'
                        : '#ef4444'
                } ${healthScore * 3.6}deg, var(--color-border) 0deg)`,
              }}
            >
              <span className="health-score-num">{healthScore}</span>
            </div>
            <div className="health-widget-info">
              <span className="health-widget-label">{t('health.score')}</span>
              {urgentRotations > 0 && (
                <span className="health-rotation-badge">{urgentRotations} 🔄</span>
              )}
            </div>
          </button>
        )}

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
        .health-widget {
          display: flex; align-items: center; gap: var(--space-sm);
          margin: var(--space-xs) var(--space-sm);
          padding: var(--space-xs) var(--space-sm);
          background: var(--color-bg-secondary); border: 1px solid var(--color-border);
          border-radius: var(--radius-md); cursor: pointer; width: calc(100% - var(--space-md));
          transition: background .1s;
        }
        .health-widget:hover { background: var(--color-bg-tertiary); }
        .health-score-ring {
          width: 32px; height: 32px; border-radius: 50%;
          display: flex; align-items: center; justify-content: center;
          flex-shrink: 0;
        }
        .health-score-num {
          font-size: 10px; font-weight: 700; color: var(--color-text);
          background: var(--color-surface); width: 24px; height: 24px;
          border-radius: 50%; display: flex; align-items: center; justify-content: center;
        }
        .health-widget-info { display: flex; flex-direction: column; gap: 1px; }
        .health-widget-label { font-size: 11px; color: var(--color-text-secondary); font-weight: 500; }
        .health-rotation-badge { font-size: 10px; color: #f59e0b; font-weight: 600; }
      `}</style>
    </div>
  );
}
