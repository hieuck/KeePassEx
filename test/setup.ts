/**
 * Vitest global test setup
 */
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri API for desktop tests
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Mock React Native modules for shared component tests
vi.mock('react-native', () => ({
  StyleSheet: { create: (s: unknown) => s, hairlineWidth: 1 },
  Platform: { OS: 'web', select: (obj: Record<string, unknown>) => obj.web ?? obj.default },
  Dimensions: { get: () => ({ width: 375, height: 812 }) },
  Appearance: { getColorScheme: () => 'light', addChangeListener: vi.fn() },
}));

// Mock clipboard
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn(() => Promise.resolve()),
    readText: vi.fn(() => Promise.resolve('')),
  },
});

// Suppress console.log in tests (keep warn/error)
vi.spyOn(console, 'log').mockImplementation(() => {});
