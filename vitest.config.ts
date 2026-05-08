import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
    // Focused test scope — only files that work in jsdom without React Native
    include: [
      'packages/i18n/src/__tests__/**/*.{test,spec}.{ts,tsx}',
      'shared/**/__tests__/**/*.{test,spec}.{ts,tsx}',
      'apps/desktop/src/__tests__/store.test.ts',
      'apps/browser-extension/src/__tests__/**/*.{test,spec}.{ts,tsx}',
    ],
    exclude: [
      '**/node_modules/**',
      '**/dist/**',
      '**/src-tauri/**',
      'apps/mobile/**',
      'apps/watch/**',
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'dist/', '**/*.d.ts', '**/*.config.*', '**/index.ts'],
      thresholds: { lines: 60, functions: 60, branches: 50, statements: 60 },
    },
  },
  resolve: {
    alias: {
      '@keepassex/ui': resolve(__dirname, 'packages/ui/src'),
      '@keepassex/i18n': resolve(__dirname, 'packages/i18n/src'),
      '@keepassex/types': resolve(__dirname, 'shared/types/src'),
      '@keepassex/constants': resolve(__dirname, 'shared/constants/src'),
      '@keepassex/utils': resolve(__dirname, 'shared/utils/src'),
      react: resolve(__dirname, 'apps/desktop/node_modules/react'),
      'react-dom': resolve(__dirname, 'apps/desktop/node_modules/react-dom'),
      'react/jsx-dev-runtime': resolve(
        __dirname,
        'apps/desktop/node_modules/react/jsx-dev-runtime'
      ),
      'react/jsx-runtime': resolve(__dirname, 'apps/desktop/node_modules/react/jsx-runtime'),
      'react-i18next': resolve(__dirname, 'apps/desktop/node_modules/react-i18next'),
    },
  },
});
