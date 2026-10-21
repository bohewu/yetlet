import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
import { lstatSync } from 'node:fs';
import { join } from 'node:path';
import tailwindcss from '@tailwindcss/vite';

const root = fileURLToPath(new URL('.', import.meta.url));
const frontendDotenv = {
  name: 'yetlet-frontend-dotenv',
  enforce: 'pre',
  config(_config, { mode }) {
    if (mode.includes('/') || mode.includes('\\')) throw new Error('Invalid frontend mode');
    for (const name of ['.env', '.env.local', `.env.${mode}`, `.env.${mode}.local`]) {
      const stat = lstatSync(join(root, name), { throwIfNoEntry: false });
      if (stat && (stat.isSymbolicLink() || !stat.isFile())) throw new Error('Frontend dotenv must be a regular, unlinked file');
    }
  },
};

export default defineConfig({
  root,
  envDir: root,
  publicDir: false,
  base: './',
  appType: 'mpa',
  plugins: [frontendDotenv, tailwindcss(),],
  build: {
    outDir: 'dist', emptyOutDir: true, manifest: true, assetsInlineLimit: 0,
    sourcemap: false, target: 'es2022', modulePreload: { polyfill: false },
  },
  // A local preview of built output, not a production hosting service.
  preview: { host: '127.0.0.1', port: 5173, strictPort: true },
});
