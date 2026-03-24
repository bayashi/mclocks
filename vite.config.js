import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig } from 'vite';

//console.log("process.env", process.env)

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ mode }) => ({
  clearScreen: false,
  resolve: {
    alias:
      mode === 'e2e'
        ? {
            '@tauri-apps/plugin-clipboard-manager': path.resolve(
              __dirname,
              'test/mocks/tauri-plugin-clipboard-manager.js'
            ),
          }
        : {},
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: [
        '**/src-tauri/**',
      ],
    },
  },
  envPrefix: [
    'VITE_',
    'TAURI_ENV_*',
  ],
  build: {
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));