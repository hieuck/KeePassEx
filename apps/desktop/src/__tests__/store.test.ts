/**
 * Desktop store tests
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useVaultStore } from '../store/vault';
import { useSettingsStore } from '../store/settings';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// ─── Vault Store ──────────────────────────────────────────────────────────────

describe('useVaultStore', () => {
  beforeEach(() => {
    // Reset store state
    useVaultStore.setState({
      isOpen: false,
      isLocked: false,
      meta: null,
      selectedGroupUuid: null,
      searchQuery: '',
    });
  });

  it('starts with vault closed', () => {
    const { isOpen, isLocked, meta } = useVaultStore.getState();
    expect(isOpen).toBe(false);
    expect(isLocked).toBe(false);
    expect(meta).toBeNull();
  });

  it('setLocked updates locked state', () => {
    useVaultStore.getState().setLocked(true);
    expect(useVaultStore.getState().isLocked).toBe(true);

    useVaultStore.getState().setLocked(false);
    expect(useVaultStore.getState().isLocked).toBe(false);
  });

  it('setSelectedGroup updates selected group', () => {
    const uuid = '550e8400-e29b-41d4-a716-446655440000';
    useVaultStore.getState().setSelectedGroup(uuid);
    expect(useVaultStore.getState().selectedGroupUuid).toBe(uuid);

    useVaultStore.getState().setSelectedGroup(null);
    expect(useVaultStore.getState().selectedGroupUuid).toBeNull();
  });

  it('setSearchQuery updates search query', () => {
    useVaultStore.getState().setSearchQuery('github');
    expect(useVaultStore.getState().searchQuery).toBe('github');
  });

  it('openVault calls invoke and updates state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockMeta = {
      name: 'Test Vault',
      description: '',
      entryCount: 5,
      groupCount: 2,
      path: '/test/vault.kdbx',
    };
    vi.mocked(invoke).mockResolvedValueOnce(mockMeta);

    await useVaultStore.getState().openVault('/test/vault.kdbx', 'password');

    const state = useVaultStore.getState();
    expect(state.isOpen).toBe(true);
    expect(state.isLocked).toBe(false);
    expect(state.meta?.name).toBe('Test Vault');
    expect(state.meta?.entryCount).toBe(5);
  });

  it('closeVault resets state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    // Set some state first
    useVaultStore.setState({
      isOpen: true,
      meta: { name: 'Test', description: '', entryCount: 1, groupCount: 1, path: '/test.kdbx' },
    });

    await useVaultStore.getState().closeVault();

    const state = useVaultStore.getState();
    expect(state.isOpen).toBe(false);
    expect(state.meta).toBeNull();
    expect(state.selectedGroupUuid).toBeNull();
  });
});

// ─── Settings Store ───────────────────────────────────────────────────────────

describe('useSettingsStore', () => {
  beforeEach(() => {
    useSettingsStore.setState({
      settings: {
        language: 'en',
        theme: 'system',
        lockOnMinimize: false,
        lockOnScreenLock: true,
        lockAfterIdleMinutes: 5,
        clipboardClearSeconds: 10,
        showPasswordInList: false,
        minimizeToTray: true,
        startMinimized: false,
        checkForUpdates: true,
        browserIntegration: true,
        sshAgentEnabled: false,
        defaultAutoTypeSequence: '{USERNAME}{TAB}{PASSWORD}{ENTER}',
        recentVaults: [],
      },
    });
  });

  it('has correct default settings', () => {
    const { settings } = useSettingsStore.getState();
    expect(settings.language).toBe('en');
    expect(settings.clipboardClearSeconds).toBe(10);
    expect(settings.lockOnScreenLock).toBe(true);
  });

  it('update changes settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await useSettingsStore.getState().update({ language: 'vi' });
    expect(useSettingsStore.getState().settings.language).toBe('vi');
  });

  it('update persists partial changes', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await useSettingsStore.getState().update({ clipboardClearSeconds: 30 });
    const { settings } = useSettingsStore.getState();
    expect(settings.clipboardClearSeconds).toBe(30);
    expect(settings.language).toBe('en'); // unchanged
  });
});
