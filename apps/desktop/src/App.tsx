/**
 * KeePassEx Desktop App
 */
import { useEffect, useState } from 'react';
import { HashRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { I18nextProvider } from 'react-i18next';
import i18next from 'i18next';
import { useVaultStore } from './store/vault';
import { useSettingsStore } from './store/settings';
import { useTabsStore } from './store/tabs';
import { ErrorBoundary } from './components/ErrorBoundary';
import { WelcomePage } from './pages/WelcomePage';
import { UnlockPage } from './pages/UnlockPage';
import { MainLayout } from './layouts/MainLayout';
import { VaultPage } from './pages/VaultPage';
import { EntryDetailPage } from './pages/EntryDetailPage';
import { SettingsPage } from './pages/SettingsPage';
import { SecurityPage } from './pages/SecurityPage';
import { HealthPage } from './pages/HealthPage';
import { GeneratorPage } from './pages/GeneratorPage';
import { ImportExportPage } from './pages/ImportExportPage';
import { SyncPage } from './pages/SyncPage';
import { BreachPage } from './pages/BreachPage';
import { EmergencyAccessPage } from './pages/EmergencyAccessPage';
import { PluginsPage } from './pages/PluginsPage';
import { HardwareKeyPage } from './pages/HardwareKeyPage';
import { StatisticsPage } from './pages/StatisticsPage';
import { AuditLogPage } from './pages/AuditLogPage';
import { VaultComparePage } from './pages/VaultComparePage';
import { PasswordPolicyPage } from './pages/PasswordPolicyPage';
import { BackupPage } from './pages/BackupPage';
import { SteganographyPage } from './pages/SteganographyPage';
import { AnalyticsPage } from './pages/AnalyticsPage';
import { TeamPage } from './pages/TeamPage';
import { CommandPalette } from './components/CommandPalette';
import { IdleLockManager } from './components/IdleLockManager';
import { VaultTabBar } from './components/VaultTabBar';
import { listen } from '@tauri-apps/api/event';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: 30_000, retry: 1 },
  },
});

export function App() {
  const { isOpen, isLocked } = useVaultStore();
  const { init: initSettings } = useSettingsStore();
  const { tabs } = useTabsStore();
  const [paletteOpen, setPaletteOpen] = useState(false);

  useEffect(() => {
    initSettings();

    // Listen for vault-locked event from tray
    const unlisten = listen('vault-locked', () => {
      useVaultStore.getState().setLocked(true);
      // Also lock all tabs
      useTabsStore.getState().lockAllTabs();
    });

    // Global keyboard shortcut: Cmd/Ctrl+K → command palette
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen && !isLocked) setPaletteOpen(p => !p);
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      unlisten.then(fn => fn());
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen, isLocked]);

  // Show tab bar when multiple vaults are open or tabs feature is active
  const showTabBar = tabs.length > 1;

  return (
    <I18nextProvider i18n={i18next}>
      <ErrorBoundary>
        <QueryClientProvider client={queryClient}>
          <HashRouter>
            {/* Global managers (no UI) */}
            <IdleLockManager />

            {/* Multi-vault tab bar — shown when more than one vault is open */}
            {showTabBar && isOpen && !isLocked && (
              <VaultTabBar
                onOpenNewVault={() => {
                  /* navigate to welcome to open another vault */
                }}
              />
            )}

            {/* Command palette */}
            <CommandPalette
              open={paletteOpen && isOpen && !isLocked}
              onClose={() => setPaletteOpen(false)}
            />

            <Routes>
              {/* Welcome / Open vault */}
              {!isOpen && (
                <>
                  <Route path="/" element={<WelcomePage />} />
                  <Route path="/unlock" element={<UnlockPage />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </>
              )}

              {/* Locked vault */}
              {isOpen && isLocked && (
                <>
                  <Route path="/unlock" element={<UnlockPage />} />
                  <Route path="*" element={<Navigate to="/unlock" replace />} />
                </>
              )}

              {/* Open vault */}
              {isOpen && !isLocked && (
                <Route
                  element={
                    <ErrorBoundary>
                      <MainLayout onOpenPalette={() => setPaletteOpen(true)} />
                    </ErrorBoundary>
                  }
                >
                  <Route path="/" element={<Navigate to="/vault" replace />} />
                  <Route path="/vault" element={<VaultPage />} />
                  <Route path="/vault/entry/:uuid" element={<EntryDetailPage />} />
                  <Route path="/health" element={<HealthPage />} />
                  <Route path="/breach" element={<BreachPage />} />
                  <Route path="/generator" element={<GeneratorPage />} />
                  <Route path="/import-export" element={<ImportExportPage />} />
                  <Route path="/sync" element={<SyncPage />} />
                  <Route path="/emergency-access" element={<EmergencyAccessPage />} />
                  <Route path="/plugins" element={<PluginsPage />} />
                  <Route path="/settings" element={<SettingsPage />} />
                  <Route path="/settings/security" element={<SecurityPage />} />
                  <Route path="/settings/hardware-key" element={<HardwareKeyPage />} />
                  <Route path="/settings/statistics" element={<StatisticsPage />} />
                  <Route path="/settings/audit-log" element={<AuditLogPage />} />
                  <Route path="/settings/password-policy" element={<PasswordPolicyPage />} />
                  <Route path="/settings/backup" element={<BackupPage />} />
                  <Route path="/vault/compare" element={<VaultComparePage />} />
                  <Route path="/settings/steganography" element={<SteganographyPage />} />
                  <Route path="/settings/analytics" element={<AnalyticsPage />} />
                  <Route path="/team" element={<TeamPage />} />
                  <Route path="*" element={<Navigate to="/vault" replace />} />
                </Route>
              )}
            </Routes>
          </HashRouter>
        </QueryClientProvider>
      </ErrorBoundary>
    </I18nextProvider>
  );
}
