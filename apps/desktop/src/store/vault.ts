/**
 * Vault state store — Zustand
 */
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from './settings';

export interface VaultMeta {
  name: string;
  description: string;
  entryCount: number;
  groupCount: number;
  path: string;
}

export interface EntryDto {
  uuid: string;
  groupUuid: string;
  title: string;
  username: string;
  url: string;
  notes: string;
  iconId: number;
  tags: string[];
  hasPassword: boolean;
  hasOtp: boolean;
  hasPasskey: boolean;
  hasSshKey: boolean;
  hasAttachments: boolean;
  isExpired: boolean;
  expiry?: string;
  createdAt: string;
  modifiedAt: string;
  customFields: CustomFieldDto[];
}

export interface CustomFieldDto {
  key: string;
  value: string;
  protected: boolean;
}

export interface GroupDto {
  uuid: string;
  parentUuid?: string;
  name: string;
  notes: string;
  iconId: number;
  isExpanded: boolean;
  entryCount: number;
  childGroupCount: number;
}

interface VaultState {
  isOpen: boolean;
  isLocked: boolean;
  meta: VaultMeta | null;
  selectedGroupUuid: string | null;
  searchQuery: string;

  // Actions
  openVault: (path: string, password: string, keyFileData?: Uint8Array) => Promise<void>;
  createVault: (path: string, name: string, password: string) => Promise<void>;
  closeVault: () => Promise<void>;
  lockVault: () => Promise<void>;
  unlockVault: (password: string) => Promise<void>;
  setLocked: (locked: boolean) => void;
  setSelectedGroup: (uuid: string | null) => void;
  setSearchQuery: (query: string) => void;
}

export const useVaultStore = create<VaultState>((set, get) => ({
  isOpen: false,
  isLocked: false,
  meta: null,
  selectedGroupUuid: null,
  searchQuery: '',

  openVault: async (path, password, keyFileData) => {
    const meta = await invoke<VaultMeta>('open_vault', {
      args: {
        path,
        password,
        key_file_data: keyFileData ? Array.from(keyFileData) : null,
      },
    });
    set({ isOpen: true, isLocked: false, meta });

    // Track in recent vaults
    const { addRecentVault } = useSettingsStore.getState();
    await addRecentVault({
      path,
      name: meta.name,
      lastOpened: new Date().toISOString(),
    });
  },

  createVault: async (path, name, password) => {
    const meta = await invoke<VaultMeta>('create_vault', {
      args: { path, name, password },
    });
    set({ isOpen: true, isLocked: false, meta });
  },

  closeVault: async () => {
    await invoke('close_vault');
    set({ isOpen: false, isLocked: false, meta: null, selectedGroupUuid: null });
  },

  lockVault: async () => {
    await invoke('lock_vault');
    set({ isLocked: true });
  },

  unlockVault: async (password) => {
    // Re-open with stored path
    const { meta } = get();
    if (!meta) throw new Error('No vault to unlock');
    await invoke('open_vault', {
      args: { path: meta.path, password },
    });
    set({ isLocked: false });
  },

  setLocked: (locked) => set({ isLocked: locked }),
  setSelectedGroup: (uuid) => set({ selectedGroupUuid: uuid }),
  setSearchQuery: (query) => set({ searchQuery: query }),
}));
