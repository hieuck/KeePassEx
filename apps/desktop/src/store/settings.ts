/**
 * Settings store — Zustand
 */
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { changeLocale, type SupportedLocale } from '@keepassex/i18n';
import type { RecentVault } from '@keepassex/types';

export interface AppSettings {
  language: string;
  theme: string;
  lockOnMinimize: boolean;
  lockOnScreenLock: boolean;
  lockAfterIdleMinutes?: number;
  clipboardClearSeconds?: number;
  showPasswordInList: boolean;
  minimizeToTray: boolean;
  startMinimized: boolean;
  checkForUpdates: boolean;
  browserIntegration: boolean;
  sshAgentEnabled: boolean;
  defaultAutoTypeSequence: string;
  recentVaults: RecentVault[];
}

interface SettingsState {
  settings: AppSettings;
  init: () => Promise<void>;
  update: (partial: Partial<AppSettings>) => Promise<void>;
  addRecentVault: (vault: RecentVault) => Promise<void>;
}

const defaults: AppSettings = {
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
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaults,

  init: async () => {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      const merged = { ...defaults, ...settings };
      set({ settings: merged });
      // Apply saved language immediately on startup — no restart required
      if (merged.language && merged.language !== 'en') {
        await changeLocale(merged.language as SupportedLocale).catch(() => {});
      }
    } catch {
      // Use defaults
    }
  },

  update: async partial => {
    const newSettings = { ...get().settings, ...partial };
    set({ settings: newSettings });
    // Apply language change live — no restart required
    if (partial.language) {
      await changeLocale(partial.language as SupportedLocale).catch(() => {});
    }
    try {
      await invoke('save_settings', { settings: newSettings });
    } catch {
      // Silently fail — settings are still updated in memory
    }
  },

  addRecentVault: async vault => {
    const current = get().settings.recentVaults;
    const filtered = current.filter(v => v.path !== vault.path);
    const updated = [vault, ...filtered].slice(0, 10);
    await get().update({ recentVaults: updated });
  },
}));
