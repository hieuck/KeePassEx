import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    setupFiles: ['../../test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/**/*.test.{ts,tsx}', 'src/**/__tests__/**'],
    },
  },
  resolve: {
    alias: {
      '@keepassex/ui': resolve(__dirname, '../../packages/ui/src'),
      '@keepassex/i18n': resolve(__dirname, '../../packages/i18n/src'),
      '@keepassex/types': resolve(__dirname, '../../shared/types/src'),
      '@keepassex/constants': resolve(__dirname, '../../shared/constants/src'),
    },
  },
});
