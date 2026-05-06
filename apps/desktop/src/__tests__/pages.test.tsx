/**
 * Desktop page tests
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Mock stores
vi.mock('../store/settings', () => ({
  useSettingsStore: () => ({
    settings: {
      language: 'en',
      clipboardClearSeconds: 10,
      recentVaults: [],
      lockAfterIdleMinutes: 5,
      lockOnScreenLock: true,
      lockOnMinimize: false,
      showPasswordInList: false,
      minimizeToTray: true,
      startMinimized: false,
      checkForUpdates: true,
      browserIntegration: true,
      sshAgentEnabled: false,
      defaultAutoTypeSequence: '{USERNAME}{TAB}{PASSWORD}{ENTER}',
      theme: 'system',
    },
    update: vi.fn(),
    addRecentVault: vi.fn(),
  }),
}));

vi.mock('../store/vault', () => ({
  useVaultStore: () => ({
    isOpen: false,
    isLocked: false,
    meta: null,
    selectedGroupUuid: null,
    searchQuery: '',
    openVault: vi.fn(),
    createVault: vi.fn(),
    closeVault: vi.fn(),
    lockVault: vi.fn(),
    unlockVault: vi.fn(),
    setLocked: vi.fn(),
    setSelectedGroup: vi.fn(),
    setSearchQuery: vi.fn(),
  }),
}));

vi.mock('../store/breach', () => ({
  useBreachStore: () => ({
    report: null,
    loading: false,
    error: null,
    lastChecked: null,
    checkVault: vi.fn(),
    checkPassword: vi.fn(),
    clearReport: vi.fn(),
  }),
}));

vi.mock('../store/sync', () => ({
  useSyncStore: () => ({
    status: null,
    syncing: false,
    lastResult: null,
    error: null,
    fetchStatus: vi.fn(),
    configure: vi.fn(),
    syncNow: vi.fn(),
    testConnection: vi.fn(),
  }),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

// ─── WelcomePage ──────────────────────────────────────────────────────────────

import { WelcomePage } from '../pages/WelcomePage';

describe('WelcomePage', () => {
  it('renders app name', () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    expect(screen.getByText('KeePassEx')).toBeTruthy();
  });

  it('renders open vault button', () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Open Vault/i)).toBeTruthy();
  });

  it('renders create vault button', () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Create New Vault/i)).toBeTruthy();
  });

  it('renders feature highlights', () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    // Feature highlights use i18n keys — check for known translated values
    expect(screen.getByText(/Hardware Key/i)).toBeTruthy();
  });

  it('shows tagline', () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Your passwords, your control/i)).toBeTruthy();
  });

  it('shows password dialog when open vault is clicked', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    vi.mocked(open).mockResolvedValueOnce('/test/vault.kdbx');

    render(<WelcomePage />, { wrapper: createWrapper() });
    fireEvent.click(screen.getByText(/Open Vault/i));

    await waitFor(() => {
      expect(screen.getByPlaceholderText(/Master Password/i)).toBeTruthy();
    });
  });

  it('shows create vault dialog when create is clicked', async () => {
    render(<WelcomePage />, { wrapper: createWrapper() });
    fireEvent.click(screen.getByText(/Create New Vault/i));

    await waitFor(() => {
      expect(screen.getByLabelText(/Vault Name/i)).toBeTruthy();
    });
  });
});

// ─── UnlockPage ───────────────────────────────────────────────────────────────

import { UnlockPage } from '../pages/UnlockPage';

describe('UnlockPage', () => {
  it('renders password input', () => {
    render(<UnlockPage />, { wrapper: createWrapper() });
    expect(screen.getByLabelText(/Master Password/i)).toBeTruthy();
  });

  it('renders unlock button', () => {
    render(<UnlockPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Unlock/i)).toBeTruthy();
  });

  it('shows error on wrong password', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Wrong master password'));

    render(<UnlockPage />, { wrapper: createWrapper() });

    const input = screen.getByLabelText(/Master Password/i);
    fireEvent.change(input, { target: { value: 'wrongpassword' } });

    const form = input.closest('form');
    if (form) fireEvent.submit(form);

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeTruthy();
    });
  });
});

// ─── GeneratorPage ────────────────────────────────────────────────────────────

import { GeneratorPage } from '../pages/GeneratorPage';

describe('GeneratorPage', () => {
  it('renders generator title', () => {
    render(<GeneratorPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Generator/i)).toBeTruthy();
  });

  it('renders generate button', () => {
    render(<GeneratorPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Generate/i)).toBeTruthy();
  });

  it('renders mode selector', () => {
    render(<GeneratorPage />, { wrapper: createWrapper() });
    expect(screen.getByText('Random')).toBeTruthy();
    expect(screen.getByText('Passphrase')).toBeTruthy();
  });

  it('renders length slider', () => {
    render(<GeneratorPage />, { wrapper: createWrapper() });
    expect(screen.getByLabelText(/Length/i)).toBeTruthy();
  });
});

// ─── SettingsPage ─────────────────────────────────────────────────────────────

import { SettingsPage } from '../pages/SettingsPage';

describe('SettingsPage', () => {
  it('renders settings title', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Settings/i)).toBeTruthy();
  });

  it('renders language selector', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByLabelText('Language')).toBeTruthy();
  });

  it('renders theme selector', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByLabelText('Theme')).toBeTruthy();
  });

  it('renders security section', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Security/i)).toBeTruthy();
  });

  it('renders emergency access link', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Emergency Access/i)).toBeTruthy();
  });

  it('renders plugins link', () => {
    render(<SettingsPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Plugins/i)).toBeTruthy();
  });
});

// ─── BreachPage ───────────────────────────────────────────────────────────────

import { BreachPage } from '../pages/BreachPage';

describe('BreachPage', () => {
  it('renders breach monitor title', () => {
    render(<BreachPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Breach Monitor/i)).toBeTruthy();
  });

  it('renders run check button', () => {
    render(<BreachPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Run Check/i)).toBeTruthy();
  });

  it('renders mode selector', () => {
    render(<BreachPage />, { wrapper: createWrapper() });
    expect(screen.getByText('Offline')).toBeTruthy();
    expect(screen.getByText(/Online/i)).toBeTruthy();
  });

  it('shows no breaches when report is clean', async () => {
    const { useBreachStore } = vi.mocked(await import('../store/breach'));
    // Already mocked with null report — no results shown
    render(<BreachPage />, { wrapper: createWrapper() });
    expect(screen.queryByText(/breached/i)).toBeNull();
  });
});

// ─── AuditLogPage ─────────────────────────────────────────────────────────────

import { AuditLogPage } from '../pages/AuditLogPage';

describe('AuditLogPage', () => {
  it('renders audit log title', () => {
    render(<AuditLogPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Audit Log/i)).toBeTruthy();
  });

  it('renders filter tabs', () => {
    render(<AuditLogPage />, { wrapper: createWrapper() });
    expect(screen.getByText('All')).toBeTruthy();
    expect(screen.getByText('Security')).toBeTruthy();
    expect(screen.getByText('Access')).toBeTruthy();
  });

  it('renders export button', () => {
    render(<AuditLogPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Export/i)).toBeTruthy();
  });

  it('renders clear log button', () => {
    render(<AuditLogPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Clear Log/i)).toBeTruthy();
  });
});

// ─── BackupPage ───────────────────────────────────────────────────────────────

import { BackupPage } from '../pages/BackupPage';

describe('BackupPage', () => {
  it('renders backup title', () => {
    render(<BackupPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Scheduled Backup/i)).toBeTruthy();
  });

  it('renders enable toggle', () => {
    render(<BackupPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Enable automatic backup/i)).toBeTruthy();
  });

  it('renders save configuration button', () => {
    render(<BackupPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Save Configuration/i)).toBeTruthy();
  });

  it('renders backup history section', () => {
    render(<BackupPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Backup History/i)).toBeTruthy();
  });
});

// ─── VaultComparePage ─────────────────────────────────────────────────────────

import { VaultComparePage } from '../pages/VaultComparePage';

describe('VaultComparePage', () => {
  it('renders compare vaults title', () => {
    render(<VaultComparePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Compare Vaults/i)).toBeTruthy();
  });

  it('renders compare button', () => {
    render(<VaultComparePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Compare/i)).toBeTruthy();
  });

  it('renders first vault selector', () => {
    render(<VaultComparePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/First vault/i)).toBeTruthy();
  });

  it('renders second vault selector', () => {
    render(<VaultComparePage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Second vault/i)).toBeTruthy();
  });
});

// ─── PasswordPolicyPage ───────────────────────────────────────────────────────

import { PasswordPolicyPage } from '../pages/PasswordPolicyPage';

describe('PasswordPolicyPage', () => {
  it('renders password policies title', () => {
    render(<PasswordPolicyPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Password Policies/i)).toBeTruthy();
  });

  it('renders test password section', () => {
    render(<PasswordPolicyPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/Test a Password/i)).toBeTruthy();
  });

  it('renders test button', () => {
    render(<PasswordPolicyPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/^Test$/i)).toBeTruthy();
  });
});
