/**
 * Multi-vault tabs store — Zustand
 *
 * Allows opening multiple vaults simultaneously in separate tabs.
 * Each tab has its own vault state, selected group, and search query.
 * This feature surpasses KeePass/KeePassXC which only support one vault at a time.
 */
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { VaultMeta } from './vault';

export interface VaultTab {
  /** Unique tab ID */
  id: string;
  /** Vault file path */
  path: string;
  /** Vault metadata (name, entry count, etc.) */
  meta: VaultMeta;
  /** Whether this tab's vault is locked */
  isLocked: boolean;
  /** Currently selected group UUID in this tab */
  selectedGroupUuid: string | null;
  /** Search query for this tab */
  searchQuery: string;
  /** Whether this tab has unsaved changes */
  isDirty: boolean;
  /** When this tab was last accessed */
  lastAccessed: string;
}

interface TabsState {
  /** All open vault tabs */
  tabs: VaultTab[];
  /** ID of the currently active tab */
  activeTabId: string | null;

  // ─── Actions ───────────────────────────────────────────────────────────────

  /** Open a vault in a new tab (or switch to existing tab if already open) */
  openTab: (path: string, password: string, keyFileData?: Uint8Array) => Promise<VaultTab>;

  /** Close a tab and its vault */
  closeTab: (tabId: string) => Promise<void>;

  /** Switch to a different tab */
  switchTab: (tabId: string) => void;

  /** Lock a specific tab's vault */
  lockTab: (tabId: string) => Promise<void>;

  /** Unlock a specific tab's vault */
  unlockTab: (tabId: string, password: string, keyFileData?: Uint8Array) => Promise<void>;

  /** Update selected group for the active tab */
  setTabSelectedGroup: (tabId: string, groupUuid: string | null) => void;

  /** Update search query for a tab */
  setTabSearchQuery: (tabId: string, query: string) => void;

  /** Mark a tab as dirty (unsaved changes) */
  markTabDirty: (tabId: string, dirty: boolean) => void;

  /** Get the currently active tab */
  getActiveTab: () => VaultTab | null;

  /** Lock all open tabs */
  lockAllTabs: () => Promise<void>;

  /** Move a tab to a different position */
  reorderTab: (tabId: string, newIndex: number) => void;
}

function generateTabId(): string {
  return `tab-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

export const useTabsStore = create<TabsState>()(
  persist(
    (set, get) => ({
      tabs: [],
      activeTabId: null,

      openTab: async (path, password, keyFileData) => {
        // Check if this vault is already open in a tab
        const existing = get().tabs.find(t => t.path === path);
        if (existing) {
          // Switch to existing tab and unlock if needed
          set({ activeTabId: existing.id });
          if (existing.isLocked) {
            await get().unlockTab(existing.id, password, keyFileData);
          }
          return existing;
        }

        // Open vault via Tauri — multi-vault support uses indexed vault slots
        const meta = await invoke<VaultMeta>('open_vault_tab', {
          args: {
            path,
            password,
            key_file_data: keyFileData ? Array.from(keyFileData) : null,
          },
        });

        const newTab: VaultTab = {
          id: generateTabId(),
          path,
          meta,
          isLocked: false,
          selectedGroupUuid: null,
          searchQuery: '',
          isDirty: false,
          lastAccessed: new Date().toISOString(),
        };

        set(state => ({
          tabs: [...state.tabs, newTab],
          activeTabId: newTab.id,
        }));

        return newTab;
      },

      closeTab: async tabId => {
        const tab = get().tabs.find(t => t.id === tabId);
        if (!tab) return;

        // Close vault in backend
        await invoke('close_vault_tab', { path: tab.path }).catch(() => {
          // Ignore errors on close — vault may already be closed
        });

        set(state => {
          const remaining = state.tabs.filter(t => t.id !== tabId);
          let newActiveId = state.activeTabId;

          if (state.activeTabId === tabId) {
            // Switch to adjacent tab
            const idx = state.tabs.findIndex(t => t.id === tabId);
            const next = remaining[idx] ?? remaining[idx - 1] ?? null;
            newActiveId = next?.id ?? null;
          }

          return { tabs: remaining, activeTabId: newActiveId };
        });
      },

      switchTab: tabId => {
        set(state => ({
          activeTabId: tabId,
          tabs: state.tabs.map(t =>
            t.id === tabId ? { ...t, lastAccessed: new Date().toISOString() } : t
          ),
        }));
      },

      lockTab: async tabId => {
        await invoke('lock_vault_tab', {
          path: get().tabs.find(t => t.id === tabId)?.path,
        }).catch(() => {});

        set(state => ({
          tabs: state.tabs.map(t => (t.id === tabId ? { ...t, isLocked: true } : t)),
        }));
      },

      unlockTab: async (tabId, password, keyFileData) => {
        const tab = get().tabs.find(t => t.id === tabId);
        if (!tab) throw new Error('Tab not found');

        const meta = await invoke<VaultMeta>('open_vault_tab', {
          args: {
            path: tab.path,
            password,
            key_file_data: keyFileData ? Array.from(keyFileData) : null,
          },
        });

        set(state => ({
          tabs: state.tabs.map(t => (t.id === tabId ? { ...t, isLocked: false, meta } : t)),
        }));
      },

      setTabSelectedGroup: (tabId, groupUuid) => {
        set(state => ({
          tabs: state.tabs.map(t => (t.id === tabId ? { ...t, selectedGroupUuid: groupUuid } : t)),
        }));
      },

      setTabSearchQuery: (tabId, query) => {
        set(state => ({
          tabs: state.tabs.map(t => (t.id === tabId ? { ...t, searchQuery: query } : t)),
        }));
      },

      markTabDirty: (tabId, dirty) => {
        set(state => ({
          tabs: state.tabs.map(t => (t.id === tabId ? { ...t, isDirty: dirty } : t)),
        }));
      },

      getActiveTab: () => {
        const { tabs, activeTabId } = get();
        return tabs.find(t => t.id === activeTabId) ?? null;
      },

      lockAllTabs: async () => {
        const { tabs } = get();
        await Promise.allSettled(tabs.filter(t => !t.isLocked).map(t => get().lockTab(t.id)));
      },

      reorderTab: (tabId, newIndex) => {
        set(state => {
          const tabs = [...state.tabs];
          const currentIndex = tabs.findIndex(t => t.id === tabId);
          if (currentIndex === -1) return state;

          const [tab] = tabs.splice(currentIndex, 1);
          tabs.splice(Math.max(0, Math.min(newIndex, tabs.length)), 0, tab);
          return { tabs };
        });
      },
    }),
    {
      name: 'keepassex-tabs',
      // Only persist tab metadata (paths, not vault data)
      partialize: state => ({
        tabs: state.tabs.map(t => ({
          id: t.id,
          path: t.path,
          meta: t.meta,
          isLocked: true, // Always start locked after restart
          selectedGroupUuid: null,
          searchQuery: '',
          isDirty: false,
          lastAccessed: t.lastAccessed,
        })),
        activeTabId: state.activeTabId,
      }),
    }
  )
);
