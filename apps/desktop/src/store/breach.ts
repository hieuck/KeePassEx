/**
 * Breach check state store — Zustand
 */
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { VaultBreachReport } from '@keepassex/types';

interface BreachState {
  report: VaultBreachReport | null;
  loading: boolean;
  error: string | null;
  lastChecked: Date | null;
  checkVault: (online: boolean) => Promise<void>;
  checkPassword: (password: string, online: boolean) => Promise<{ isBreached: boolean; count: number }>;
  clearReport: () => void;
}

export const useBreachStore = create<BreachState>((set) => ({
  report: null,
  loading: false,
  error: null,
  lastChecked: null,

  checkVault: async (online) => {
    set({ loading: true, error: null });
    try {
      const report = await invoke<VaultBreachReport>('check_vault_breaches', { online });
      set({ report, loading: false, lastChecked: new Date() });
    } catch (e: unknown) {
      set({
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      });
    }
  },

  checkPassword: async (password, online) => {
    const result = await invoke<{ isBreached: boolean; breachCount: number }>(
      'check_password_breach',
      { password, online }
    );
    return { isBreached: result.isBreached, count: result.breachCount };
  },

  clearReport: () => set({ report: null, error: null }),
}));
