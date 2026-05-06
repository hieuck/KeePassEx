/**
 * KeePassEx Mobile — Settings store
 * Persists settings to AsyncStorage
 */
import { create } from 'zustand';
import type { Language, ThemeMode } from '@keepassex/types';

// Use AsyncStorage for persistence on mobile
let AsyncStorage: { getItem: (key: string) => Promise<string | null>; setItem: (key: string, value: string) => Promise<void> };
try {
  AsyncStorage = require('@react-native-async-storage/async-storage').default;
} catch {
  // Fallback for environments without AsyncStorage
  const store: Record<string, string> = {};
  AsyncStorage = {
    getItem: async (key) => store[key] ?? null,
    setItem: async (key, value) => { store[key] = value; },
  };
}

const SETTINGS_KEY = 'keepassex_settings';

export interface MobileSettings {
  language: Language;
  lockOnBackground: boolean;
  biometricUnlock: boolean;
  clipboardClearSeconds: number;
  screenCaptureProtection: boolean;
  autoSync: boolean;
}

const defaults: MobileSettings = {
  language: 'en',
  lockOnBackground: true,
  biometricUnlock: true,
  clipboardClearSeconds: 10,
  screenCaptureProtection: true,
  autoSync: false,
};

interface SettingsState {
  settings: MobileSettings;
  loaded: boolean;
  load: () => Promise<void>;
  update: (partial: Partial<MobileSettings>) => Promise<void>;
}

export const useMobileSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaults,
  loaded: false,

  load: async () => {
    try {
      const json = await AsyncStorage.getItem(SETTINGS_KEY);
      if (json) {
        const saved = JSON.parse(json) as Partial<MobileSettings>;
        set({ settings: { ...defaults, ...saved }, loaded: true });
      } else {
        set({ loaded: true });
      }
    } catch {
      set({ loaded: true });
    }
  },

  update: async (partial) => {
    const newSettings = { ...get().settings, ...partial };
    set({ settings: newSettings });
    try {
      await AsyncStorage.setItem(SETTINGS_KEY, JSON.stringify(newSettings));
    } catch {
      // Silently fail
    }
  },
}));
