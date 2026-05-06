/**
 * Theme store for mobile
 */
import { create } from 'zustand';
import { Appearance } from 'react-native';
import { lightTheme, darkTheme, oledTheme, type Theme, type ThemeMode } from '@keepassex/ui';

interface ThemeState {
  mode: ThemeMode | 'system';
  theme: Theme;
  setMode: (mode: ThemeMode | 'system') => void;
}

function resolveTheme(mode: ThemeMode | 'system'): Theme {
  if (mode === 'system') {
    const colorScheme = Appearance.getColorScheme();
    return colorScheme === 'dark' ? darkTheme : lightTheme;
  }
  switch (mode) {
    case 'dark': return darkTheme;
    case 'oled': return oledTheme;
    default: return lightTheme;
  }
}

export const useThemeStore = create<ThemeState>((set) => ({
  mode: 'system',
  theme: resolveTheme('system'),

  setMode: (mode) => {
    set({ mode, theme: resolveTheme(mode) });
  },
}));

// Listen for system theme changes
Appearance.addChangeListener(({ colorScheme }) => {
  const { mode, setMode } = useThemeStore.getState();
  if (mode === 'system') {
    setMode('system'); // Re-resolve
  }
});
