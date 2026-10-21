import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
@PLUGIN_IMPORT@
const root = fileURLToPath(new URL('.', import.meta.url));

// Production build only. No dotenv discovery, public copying, inlining or HMR.
export default defineConfig({
  root,
  envDir: false,
  publicDir: false,
  base: '/static/',
  plugins: [@PLUGIN@],
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
        companion: fileURLToPath(new URL('companion.ts', import.meta.url)),
      },
    },
  },
});
