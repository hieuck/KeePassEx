/**
 * KeePassEx Themes — Light, Dark, OLED
 */
import { tokens } from './tokens';

export type ThemeMode = 'light' | 'dark' | 'oled';

export interface Theme {
  mode: ThemeMode;
  background: string;
  backgroundSecondary: string;
  backgroundTertiary: string;
  surface: string;
  surfaceSecondary: string;
  border: string;
  borderFocus: string;
  text: string;
  textSecondary: string;
  textTertiary: string;
  textInverse: string;
  primary: string;
  primaryText: string;
  success: string;
  warning: string;
  danger: string;
  info: string;
  tint: string;
  tabBar: string;
  tabBarBorder: string;
  statusBar: 'light-content' | 'dark-content';
}

export const lightTheme: Theme = {
  mode: 'light',
  background: tokens.color.white,
  backgroundSecondary: tokens.color.gray50,
  backgroundTertiary: tokens.color.gray100,
  surface: tokens.color.white,
  surfaceSecondary: tokens.color.gray50,
  border: tokens.color.gray200,
  borderFocus: tokens.color.primary,
  text: tokens.color.gray900,
  textSecondary: tokens.color.gray600,
  textTertiary: tokens.color.gray400,
  textInverse: tokens.color.white,
  primary: tokens.color.primary,
  primaryText: tokens.color.white,
  success: tokens.color.success,
  warning: tokens.color.warning,
  danger: tokens.color.danger,
  info: tokens.color.info,
  tint: tokens.color.primary,
  tabBar: tokens.color.white,
  tabBarBorder: tokens.color.gray200,
  statusBar: 'dark-content',
};

export const darkTheme: Theme = {
  mode: 'dark',
  background: tokens.color.gray900,
  backgroundSecondary: tokens.color.gray800,
  backgroundTertiary: tokens.color.gray700,
  surface: tokens.color.gray800,
  surfaceSecondary: tokens.color.gray700,
  border: tokens.color.gray700,
  borderFocus: tokens.color.primaryLight,
  text: tokens.color.gray50,
  textSecondary: tokens.color.gray400,
  textTertiary: tokens.color.gray600,
  textInverse: tokens.color.gray900,
  primary: tokens.color.primaryLight,
  primaryText: tokens.color.white,
  success: '#22C55E',
  warning: '#F59E0B',
  danger: '#EF4444',
  info: '#22D3EE',
  tint: tokens.color.primaryLight,
  tabBar: tokens.color.gray900,
  tabBarBorder: tokens.color.gray800,
  statusBar: 'light-content',
};

export const oledTheme: Theme = {
  ...darkTheme,
  mode: 'oled',
  background: tokens.color.black,
  backgroundSecondary: '#0A0A0A',
  backgroundTertiary: '#111111',
  surface: '#0D0D0D',
  surfaceSecondary: '#111111',
  border: '#1A1A1A',
  tabBar: tokens.color.black,
  tabBarBorder: '#1A1A1A',
};

export const themes: Record<ThemeMode, Theme> = {
  light: lightTheme,
  dark: darkTheme,
  oled: oledTheme,
};
