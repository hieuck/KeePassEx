import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

const root = path.resolve(__dirname, '../..');

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },

  resolve: {
    alias: {
      // Local src alias
      '@': path.resolve(__dirname, './src'),
      // Workspace packages — point Vite directly to source files
      '@keepassex/ui': path.resolve(root, 'packages/ui/src/web.ts'),
      '@keepassex/i18n': path.resolve(root, 'packages/i18n/src/index.ts'),
      '@keepassex/types': path.resolve(root, 'shared/types/src/index.ts'),
      '@keepassex/constants': path.resolve(root, 'shared/constants/src/index.ts'),
      '@keepassex/utils': path.resolve(root, 'shared/utils/src/index.ts'),
    },
  },

  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));
