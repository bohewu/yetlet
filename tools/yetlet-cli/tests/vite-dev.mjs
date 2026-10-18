// Direct Node product tests; no listener, browser, dependency install or test framework.
import assert from 'node:assert/strict';
import { after, test } from 'node:test';
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { EventEmitter } from 'node:events';
import { allowedOrigin, allowedPath, devBridge, devDocument, devPort, productionCsp } from '../templates/vite/vite.dev.bridge.mjs';

const fixture = mkdtempSync(join(tmpdir(), 'yetlet-vite-source-'));
const root = join(fixture, 'frontend');
const sourcePaths = [
  '/app.js', '/shared.js', '/lazy.js', '/companion.js', '/custom.ts', '/other.js',
  '/src/app.js', '/nested/user-module.ts', '/modules/helpers.mjs', '/modules/types.mts',
  '/settings/theme.ts', '/components/settings/panel.ts', '/config/client.ts', '/user.settings.ts',
  '/server/status.ts', '/backend/client.ts', '/env.js', '/config.ts',
  '/styles/custom.css', '/images/custom.svg', '/space%20name.js', '/%E6%A8%A1%E7%B5%84/%E6%8C%89%E9%88%95.ts',
];
for (const path of [...sourcePaths, '/dist/private.ts', '/.hidden/private.js', '/vite.dev.config.mjs']) {
  const file = join(root, decodeURIComponent(path).slice(1));
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, 'export const value = 1;\n');
}
after(() => rmSync(fixture, { recursive: true, force: true }));

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

test('frontend source paths admit custom modules and assets without a filename inventory', () => {
  for (const path of ['/', '/@vite/client', '/@vite/env', '/app.ts', '/shared.css?t=12345678', '/mark.svg?import', '/app.css?direct', ...sourcePaths,
    '/%61pp.js', '/app%2ejs', '/nested/模組.ts', '/nested/icon.svg?import&t=123', '/styles/custom.css?t=123&direct']) assert(allowedPath(path), path);
  for (const path of ['/.env', '/.env.local', '/server.key', '/server.pem', '/server.crt', '/base.toml', '/src/main.rs', '/package.json', '/package-lock.json',
    '/vite.dev.config.mjs', '/vite.dev.bridge.mjs', '/vite.config.mjs', '/tailwind.config.js', '/postcss.config.mjs', '/config/base.toml', '/settings/secrets.toml',
    '/target/private.ts', '/.hidden/source.ts', '/nested/.env.ts', '/dist/app.js',
    '/@fs/etc/passwd', '/@fs/etc/passwd.js', '/@id/module.js', '/%40fs/etc/passwd.js', '/%2eenv', '/./app.ts', '/app.ts/../.env', '/%2e%2e/outside.js',
    '/nested/%2e%2e/app.js', '/nested%2fapp.js', '/nested%5capp.js', '/%252e%252e/outside.js', '/app.js%3fimport', '/app.js%23fragment',
    '//app.ts', '/app.ts?raw', '/app.ts?url', '/app.ts?x=1', '/app.ts??import', '/app.ts#x', '/hello/name', 'http://evil.example/', '/C:/outside.js',
    '/nested\\app.js', '/control%00.js', '/control%0a.js', '/bad%ff.js', '/bad%.js', null]) assert(!allowedPath(path), String(path));
});

test('custom JS/TS, nested CSS/SVG and import/HMR queries reach Vite middleware', () => {
  let middleware;
  const httpServer = new EventEmitter();
  devBridge(5173).configureServer({
    config: { root }, httpServer,
    middlewares: { use(callback) { middleware = callback; } },
  });
  const headers = { host: '127.0.0.1:5173', origin: 'http://127.0.0.1:5173' };
  try {
    for (const path of sourcePaths) {
      for (const query of ['', '?import', '?direct', '?t=12345678', '?import&t=12345678', '?t=12345678&direct']) {
        for (const method of ['GET', 'HEAD']) {
          const response = {
            headers: {},
            setHeader(name, value) { this.headers[name] = value; },
            writeHead() { assert.fail(`${method} ${path}${query} must reach Vite`); },
            end() { assert.fail(`${method} ${path}${query} must reach Vite`); },
          };
          let forwarded = 0;
          middleware({ headers, url: path + query, method }, response, () => { forwarded += 1; });
          assert.equal(forwarded, 1, `${method} ${path}${query}`);
          assert.equal(response.headers['X-Content-Type-Options'], 'nosniff');
          assert.equal(response.headers['Cache-Control'], 'no-store');
        }
      }
    }
    for (const url of [
      '/app.jsx', '/app.tsx', '/data.json', '/app.js/extra', '/app.js.map', '/./app.js', '/lazy.js/../app.js', '//app.js',
      '/vite.dev.bridge.mjs', '/config/base.toml', '/.env.js', '/node_modules/private.js', '/@fs/etc/secret.js',
      '/app.js%3fimport', '/app.js#fragment', '/nested%2fapp.js', '/%252e%252e/app.js',
      '/app.js?raw', '/app.js?url', '/app.js?x=1', '/app.js?', '/app.js??import',
      '/app.js?import&import', '/app.js?import&direct', '/app.js?t=1&t=2', '/app.js?%69mport', '/app.js?%74=1', '/app.js?t=%31',
      '/app.js?t=1%0a', '/app.js?t=12345678901234567',
    ]) {
      const response = { status: null, writeHead(status) { this.status = status; }, end() {} };
      middleware({ headers, url, method: 'GET' }, response, () => { assert.fail(`${url} must be refused`); });
      assert.equal(response.status, 403, url);
    }
    for (const [requestHeaders, method, status] of [
      [{ ...headers, host: 'localhost:5173' }, 'GET', 403],
      [{ ...headers, origin: 'https://evil.example' }, 'GET', 403],
      [headers, 'POST', 405],
      [headers, 'PUT', 405],
      [headers, 'DELETE', 405],
    ]) {
      const response = { status: null, writeHead(code) { this.status = code; }, end() {} };
      middleware({ headers: requestHeaders, url: '/app.js', method }, response, () => { assert.fail('request must be refused'); });
      assert.equal(response.status, status);
    }
  } finally {
    httpServer.emit('close');
    assert.equal(httpServer.listenerCount('upgrade'), 0);
  }
});

test('physical source paths stay inside frontend root and cannot alias protected files', () => {
  const outside = join(fixture, 'backend');
  mkdirSync(outside);
  writeFileSync(join(outside, 'private.ts'), 'export const value = 2;');
  symlinkSync(outside, join(root, 'escape'), 'junction');
  symlinkSync(join(root, 'dist'), join(root, 'alias'), 'junction');
  symlinkSync(join(root, '.hidden'), join(root, 'hidden-alias'), 'junction');
  symlinkSync(join(root, 'src'), join(root, 'local-alias'), 'junction');
  let middleware;
  const httpServer = new EventEmitter();
  devBridge(5173).configureServer({
    config: { root }, httpServer,
    middlewares: { use(callback) { middleware = callback; } },
  });
  try {
    for (const url of ['/escape/private.ts', '/alias/private.ts', '/hidden-alias/private.js']) {
      assert(allowedPath(url), 'public name alone is insufficient');
      const response = { status: null, writeHead(status) { this.status = status; }, end() {} };
      middleware({ headers: { host: '127.0.0.1:5173' }, url, method: 'GET' }, response, () => assert.fail(url));
      assert.equal(response.status, 403, url);
    }
    let forwarded = 0;
    middleware({ headers: { host: '127.0.0.1:5173' }, url: '/local-alias/app.js', method: 'GET' },
      { setHeader() {}, writeHead() { assert.fail('in-root source alias should pass'); }, end() {} }, () => { forwarded += 1; });
    assert.equal(forwarded, 1);
  } finally {
    httpServer.emit('close');
    assert.equal(httpServer.listenerCount('upgrade'), 0);
  }
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
    config: { root }, httpServer,
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
