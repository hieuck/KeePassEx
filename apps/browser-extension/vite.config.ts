import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import { copyFileSync, mkdirSync } from 'fs';

export default defineConfig(({ mode }) => {
  const isFirefox = mode === 'firefox';
  const outDir = isFirefox ? 'dist-firefox' : 'dist-chrome';

  return {
    plugins: [
      react(),
      {
        name: 'copy-manifest',
        closeBundle() {
          const manifest = isFirefox ? 'manifest.firefox.json' : 'manifest.chrome.json';
          mkdirSync(outDir, { recursive: true });
          copyFileSync(manifest, `${outDir}/manifest.json`);
        },
      },
    ],
    build: {
      outDir,
      emptyOutDir: true,
      rollupOptions: {
        input: {
          popup: resolve(__dirname, 'popup.html'),
          background: resolve(__dirname, 'src/background.ts'),
          content: resolve(__dirname, 'src/content.ts'),
        },
        output: {
          entryFileNames: '[name].js',
          chunkFileNames: 'chunks/[name]-[hash].js',
          assetFileNames: 'assets/[name]-[hash][extname]',
        },
      },
      target: 'es2020',
      minify: mode !== 'development',
      sourcemap: mode === 'development',
    },
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
  };
});
