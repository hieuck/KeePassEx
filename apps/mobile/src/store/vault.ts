/**
 * Mobile vault store — Zustand
 * Uses native Rust bridge via JSI/NativeModules
 */
import { create } from 'zustand';
import { NativeModules } from 'react-native';

const { KeePassExCore } = NativeModules;

export interface VaultMeta {
  name: string;
  description: string;
  entryCount: number;
  groupCount: number;
  path: string;
}

interface VaultState {
  isOpen: boolean;
  isLocked: boolean;
  meta: VaultMeta | null;
  selectedGroupUuid: string | null;
  searchQuery: string;

  openVault: (path: string, password: string, keyFileData?: Uint8Array) => Promise<void>;
  createVault: (path: string, name: string, password: string) => Promise<void>;
  closeVault: () => void;
  lockVault: () => void;
  unlockVault: (password: string) => Promise<void>;
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
    const meta = await KeePassExCore.openVault(path, password, keyFileData ?? null);
    set({ isOpen: true, isLocked: false, meta });
  },

  createVault: async (path, name, password) => {
    const meta = await KeePassExCore.createVault(path, name, password);
    set({ isOpen: true, isLocked: false, meta });
  },

  closeVault: () => {
    KeePassExCore.closeVault();
    set({ isOpen: false, isLocked: false, meta: null, selectedGroupUuid: null });
  },

  lockVault: () => {
    KeePassExCore.lockVault();
    set({ isLocked: true });
  },

  unlockVault: async (password) => {
    const { meta } = get();
    if (!meta) throw new Error('No vault to unlock');
    await KeePassExCore.openVault(meta.path, password, null);
    set({ isLocked: false });
  },

  setSelectedGroup: (uuid) => set({ selectedGroupUuid: uuid }),
  setSearchQuery: (query) => set({ searchQuery: query }),
}));
