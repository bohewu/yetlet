import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig(({ mode }) => {
  if (mode !== 'plain' && mode !== 'tailwind') throw new Error('Select plain or tailwind');
  return {
    root,
    envDir: false,
    publicDir: false,
    base: `/${mode}/`,
    plugins: mode === 'tailwind' ? [tailwindcss()] : [],
    build: {
      outDir: `dist/${mode}`,
      emptyOutDir: true,
      manifest: true,
      assetsInlineLimit: 0,
      modulePreload: { polyfill: false },
      sourcemap: false,
      target: 'es2022',
      rolldownOptions: {
        input: {
          app: fileURLToPath(new URL(`front/${mode}.ts`, import.meta.url)),
          companion: fileURLToPath(new URL('front/companion.ts', import.meta.url)),
        },
      },
    },
  };
});
