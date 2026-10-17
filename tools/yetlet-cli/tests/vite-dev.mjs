// Direct Node product tests; no listener, browser, dependency install or test framework.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { EventEmitter } from 'node:events';
import { allowedOrigin, allowedPath, devBridge, devDocument, devPort, productionCsp } from '../templates/vite/vite.dev.bridge.mjs';

const prefix = '<!doctype html><html lang="zh-Hant"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Yetlet</title>';
const tags = '<link rel="stylesheet" href="/static/assets/shared-abc.css"><link rel="stylesheet" href="/static/assets/app-123.css"><script type="module" src="/static/assets/app-abc.js"></script>';
const image = '<img id="local-mark" src="/static/assets/mark-abc.svg" alt="Local check mark" width="80" height="48">';
const html = prefix + tags + '</head><body><p>Rust &lt;safe &amp; useful&gt;</p>' + image + '</body></html>';

test('port input is exact and bounded without implicit listener fallbacks', () => {
  assert.equal(devPort(), 5173);
  for (const value of ['1024', '5173', '65535']) assert.equal(devPort(value), Number(value));
  for (const value of ['', '0', '80', '1023', '3000', '65536', '05173', '+5173', '5173.0', ' 5173', '５１７３', '5173\n', 5173, null]) {
    assert.throws(() => devPort(value), undefined, String(value));
  }
});

test('HTTP and WebSocket require exact host and explicit same dev origin', () => {
  for (const port of [5173, 6173]) {
    const host = `127.0.0.1:${port}`;
    assert(allowedOrigin({ host }, port));
    assert(!allowedOrigin({ host }, port, true));
    assert(allowedOrigin({ host, origin: `http://${host}` }, port, true));
    for (const origin of ['null', 'https://' + host, 'http://localhost:' + port, 'http://127.0.0.1:3000', 'https://evil.example']) {
      assert(!allowedOrigin({ host, origin }, port));
      assert(!allowedOrigin({ host, origin }, port, true));
    }
    for (const bad of ['evil.example', 'localhost:' + port, '127.0.0.1', host + '.evil', [host]]) {
      assert(!allowedOrigin({ host: bad }, port));
    }
  }
});

test('finite source request allowlist rejects secret/config/raw/traversal paths', () => {
  for (const path of ['/', '/@vite/client', '/@vite/env', '/app.ts', '/shared.css?t=12345678', '/mark.svg?import', '/app.css?direct']) assert(allowedPath(path), path);
  for (const path of ['/.env', '/.env.local', '/server.key', '/server.pem', '/server.crt', '/base.toml', '/src/main.rs', '/package.json', '/vite.dev.config.mjs', '/vite.dev.bridge.mjs', '/@fs/etc/passwd', '/%2eenv', '/./app.ts', '/app.ts/../.env', '//app.ts', '/app.ts?raw', '/app.ts?url', '/app.ts?x=1', '/app.ts??import', '/app.ts#x', '/hello/name', 'http://evil.example/', null]) assert(!allowedPath(path), String(path));
});

test('the one pinned Vite client env import reaches Vite middleware; other node_modules stay refused', () => {
  const internal = '/node_modules/vite/dist/client/env.mjs';
  assert(allowedPath(internal), 'Vite 8 rewrites @vite/env to this exact app-local URL');
  for (const denied of [
    '/node_modules/', '/node_modules/vite/package.json',
    '/node_modules/vite/dist/client/client.mjs',
    '/node_modules/vite/dist/client/env.mjs/extra',
    '/node_modules/vite/dist/client/env.mjs?raw',
    '/@fs//node_modules/vite/dist/client/env.mjs',
  ]) assert(!allowedPath(denied), denied);

  let middleware;
  const httpServer = new EventEmitter();
  devBridge(5173).configureServer({
    httpServer,
    middlewares: { use(callback) { middleware = callback; } },
  });
  assert.equal(typeof middleware, 'function');
  const headers = { host: '127.0.0.1:5173', origin: 'http://127.0.0.1:5173' };
  const acceptedResponse = { setHeader() {}, writeHead() { throw new Error('forwarded to Vite instead'); }, end() { throw new Error('forwarded to Vite instead'); } };
  let forwarded = 0;
  middleware({ headers, url: internal, method: 'GET' }, acceptedResponse, () => { forwarded += 1; });
  assert.equal(forwarded, 1, 'must call next() instead of returning 403');

  const forbiddenResponse = {
    status: null, writeHead(status) { this.status = status; }, end() {},
  };
  middleware({ headers, url: '/node_modules/vite/package.json', method: 'GET' }, forbiddenResponse, () => { forwarded += 1; });
  assert.equal(forbiddenResponse.status, 403);
  assert.equal(forwarded, 1, 'the exception must not expose package metadata');
  httpServer.emit('close');
});

test('only exact generated markup transforms, preserving escaped body and local SVG identity', () => {
  const result = devDocument(html, 5173);
  assert(result.html.includes('<p>Rust &lt;safe &amp; useful&gt;</p>'));
  assert(result.html.includes('src="http://127.0.0.1:5173/mark.svg"'));
  assert(result.html.includes('<script type="module" src="/@vite/client"></script><script type="module" src="/app.ts"></script>'));
  assert(!result.html.includes('/static/'));
  for (const bad of [html.replace(tags, ''), html.replace('app-abc.js', '../evil.js'), html.replace(image, ''), html.replace(image, image + image), html.replace('charset="utf-8"', 'charset="other"'), html.replace('</body>', 'x'.repeat(256 * 1024) + '</body>')]) {
    assert.throws(() => devDocument(bad, 5173));
  }
});

test('fresh cryptographic nonce matches HTML and only the two intended dev CSP additions', () => {
  const seen = new Set();
  for (let i = 0; i < 64; i++) {
    const result = devDocument(html, 6173);
    assert.match(result.nonce, /^[A-Za-z0-9+/]{24}$/);
    assert(!seen.has(result.nonce));
    seen.add(result.nonce);
    assert(result.html.includes(`<meta property="csp-nonce" nonce="${result.nonce}">`));
    assert.equal(result.csp.replace(` 'nonce-${result.nonce}'`, '').replace(' ws://127.0.0.1:6173', ''), productionCsp);
    for (const forbidden of ['unsafe-inline', 'unsafe-eval', 'data:', '*', 'https://']) assert(!result.csp.includes(forbidden));
  }
});

test('CLI listener overrides fail closed before starting a server', () => {
  const plugin = devBridge(5173);
  const config = { base: '/', server: { host: '127.0.0.1', port: 5173, strictPort: true, ws: { host: '127.0.0.1', port: 5173 } } };
  plugin.configResolved(config);
  for (const change of [{ host: '0.0.0.0' }, { port: 5174 }, { strictPort: false }, { https: {} }, { middlewareMode: true }, { ws: { host: 'evil.example', port: 5173 } }]) {
    assert.throws(() => plugin.configResolved({ ...config, server: { ...config.server, ...change } }));
  }
});
