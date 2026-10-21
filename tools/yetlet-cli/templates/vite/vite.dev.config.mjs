// Explicit development entry only. npm run build still uses vite.config.mjs.
import { defineConfig } from 'vite';
import production from './vite.config.mjs';
import { devBridge, devPort, devTls } from './vite.dev.bridge.mjs';

export default defineConfig(({ command }) => {
  if (command !== 'serve') throw new Error('Use npm run build for production');
  const port = devPort(process.env.YETLET_VITE_PORT);
  const https = devTls(production.root);
  const origin = `${https ? 'https' : 'http'}://127.0.0.1:${port}`;
  return {
    ...production,
    base: '/',
    appType: 'custom',
    plugins: [...production.plugins, devBridge(port, https)],
    optimizeDeps: { noDiscovery: true, include: [] },
    server: {
      host: '127.0.0.1',
      port,
      strictPort: true,
      https,
      allowedHosts: [],
      origin,
      cors: { origin },
      ws: { protocol: https ? 'wss' : 'ws', host: '127.0.0.1', port, clientPort: port },
      fs: {
        strict: true,
        allow: [production.root],
        deny: ['.env', '.env.*', '*.{crt,pem,key}', '**/.git/**', '**/.aws/**', '**/.ssh/**'],
      },
    },
  };
});
