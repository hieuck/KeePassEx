/**
 * KeePassEx Design Tokens
 * Consistent across all platforms
 */

export const tokens = {
  color: {
    // Brand
    primary: '#2563EB',       // Blue 600
    primaryDark: '#1D4ED8',   // Blue 700
    primaryLight: '#3B82F6',  // Blue 500

    // Semantic
    success: '#16A34A',       // Green 600
    warning: '#D97706',       // Amber 600
    danger: '#DC2626',        // Red 600
    info: '#0891B2',          // Cyan 600

    // Password strength
    strengthVeryWeak: '#DC2626',
    strengthWeak: '#EA580C',
    strengthFair: '#D97706',
    strengthStrong: '#16A34A',
    strengthVeryStrong: '#059669',

    // Neutral
    gray50: '#F9FAFB',
    gray100: '#F3F4F6',
    gray200: '#E5E7EB',
    gray300: '#D1D5DB',
    gray400: '#9CA3AF',
    gray500: '#6B7280',
    gray600: '#4B5563',
    gray700: '#374151',
    gray800: '#1F2937',
    gray900: '#111827',
    gray950: '#030712',

    // OLED
    black: '#000000',
    white: '#FFFFFF',
  },

  space: {
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    '2xl': 32,
    '3xl': 48,
    '4xl': 64,
  },

  size: {
    xs: 24,
    sm: 32,
    md: 40,
    lg: 48,
    xl: 56,
    '2xl': 64,
  },

  radius: {
    xs: 4,
    sm: 6,
    md: 8,
    lg: 12,
    xl: 16,
    full: 9999,
  },

  fontSize: {
    xs: 11,
    sm: 13,
    md: 15,
    lg: 17,
    xl: 20,
    '2xl': 24,
    '3xl': 30,
    '4xl': 36,
  },

  fontWeight: {
    regular: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },

  lineHeight: {
    tight: 1.2,
    normal: 1.5,
    relaxed: 1.75,
  },

  shadow: {
    sm: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 1 },
      shadowOpacity: 0.05,
      shadowRadius: 2,
      elevation: 1,
    },
    md: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 2 },
      shadowOpacity: 0.1,
      shadowRadius: 4,
      elevation: 3,
    },
    lg: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 4 },
      shadowOpacity: 0.15,
      shadowRadius: 8,
      elevation: 6,
    },
  },
} as const;

export type Tokens = typeof tokens;
