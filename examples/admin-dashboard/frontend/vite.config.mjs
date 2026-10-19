import { lstatSync } from 'node:fs';
import { join as dotenvPath } from 'node:path';

// Runs on every config resolution, before Vite reads frontend dotenv files.
const frontendDotenv = {
  name: 'yetlet-frontend-dotenv',
  enforce: 'pre',
  config(_config, { mode }) {
    for (const name of ['.env', '.env.local', `.env.${mode}`, `.env.${mode}.local`]) {
      if (name.includes('/') || name.includes('\\')) {
        throw new Error('Yetlet frontend dotenv must stay in the frontend root');
      }
      const stat = lstatSync(dotenvPath(root, name), { throwIfNoEntry: false });
      if (stat && (stat.isSymbolicLink() || !stat.isFile())) {
        throw new Error(`Yetlet frontend dotenv must be a regular, unlinked file: ${name}`);
      }
    }
  },
};
import { resolve } from 'node:path';

// CLI execution supplies its selected app root, independent of frontend nesting.
// Direct npm at the default frontend/ location uses the literal ../src fallback.
const rustSource = {
  name: 'yetlet-rust-source',
  enforce: 'pre',
  transform(source, id) {
    const normalize = value => process.platform === 'win32' ? value.replaceAll('\\', '/') : value;
    const css = normalize(fileURLToPath(new URL('app.css', import.meta.url)));
    if (id.split('?')[0] !== css || !process.env.YETLET_PROJECT_ROOT) return null;
    const path = normalize(resolve(process.env.YETLET_PROJECT_ROOT, 'src/main.rs'));
    const escaped = path.replace(/["\\\n\r\f]/g, char => `\\${char.codePointAt(0).toString(16)} `);
    return source.replace('@source "../src/main.rs";', `@source "${escaped}";`);
  },
};
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
import tailwindcss from '@tailwindcss/vite';

const root = fileURLToPath(new URL('.', import.meta.url));

// Production build only. Frontend-root dotenv only, public copying, inlining or HMR.
export default defineConfig({
  root,
  envDir: root,
  publicDir: false,
  base: '/static/',
  plugins: [frontendDotenv, rustSource, tailwindcss()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    manifest: true,
    assetsInlineLimit: 0,
    modulePreload: { polyfill: false },
    sourcemap: false,
    target: 'es2022',
    rolldownOptions: {
      input: {
        app: fileURLToPath(new URL('app.ts', import.meta.url)),
      },
    },
  },
});
