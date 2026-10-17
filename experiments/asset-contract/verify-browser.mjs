import assert from 'node:assert/strict';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { resolve, join } from 'node:path';
import { chromium } from '../vite-integration/node_modules/playwright-core/index.mjs';

// One application test: the caller supplies a built backend and an unused loopback port.
const options = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i];
  assert(['--backend', '--dist', '--standalone', '--port', '--out'].includes(key), 'Unknown argument');
  assert(process.argv[i + 1] && !options.has(key), 'Missing or duplicate argument');
  options.set(key, process.argv[i + 1]);
}
assert(options.has('--backend') && options.has('--dist') && options.has('--port') && options.has('--standalone'),
  'Usage: node verify-browser.mjs --backend <binary> --dist <dist> --standalone <css-root> --port <port> [--out <directory>]');
const port = Number(options.get('--port'));
assert(Number.isInteger(port) && port > 0 && port <= 65535, 'Invalid port');
const backend = resolve(options.get('--backend'));
const dist = resolve(options.get('--dist'));
const standalone = resolve(options.get('--standalone'));
const output = options.has('--out') ? resolve(options.get('--out')) : undefined;
const origin = `http://127.0.0.1:${port}`;
const dummy = 'YT098_SYNTHETIC_SERVER_ONLY';
const csp = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
let browser;
let server;
let serverLog = '';
let cleanupPromise;
const evidence = { task_id: 'YT-098', status: 'running', profiles: [], cleanup: {} };

function cleanup() {
  return cleanupPromise ??= (async () => {
    if (browser) {
      await browser.close();
      evidence.cleanup.browser_closed = true;
    }
    if (server && server.exitCode === null && server.signalCode === null) {
      const exited = once(server, 'exit');
      server.kill('SIGTERM');
      let timeout;
      try {
        await Promise.race([
          exited,
          new Promise((done) => { timeout = setTimeout(() => { server.kill('SIGKILL'); done(); }, 5000); }),
        ]);
        if (server.exitCode === null && server.signalCode === null) await exited;
      } finally {
        clearTimeout(timeout);
      }
    }
    evidence.cleanup.server_closed = !server || server.exitCode !== null || server.signalCode !== null;
    evidence.cleanup.server_exit = server?.exitCode;
    evidence.cleanup.server_signal = server?.signalCode;
  })();
}

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => { void cleanup().finally(() => process.exit(130)); });
}

async function secureContext(javaScriptEnabled) {
  const context = await browser.newContext({ javaScriptEnabled, viewport: { width: 840, height: 650 } });
  await context.route('**/*', async (route) => {
    assert.equal(new URL(route.request().url()).origin, origin, 'External request attempted');
    await route.continue();
  });
  return context;
}

try {
  await access('/usr/bin/google-chrome', constants.X_OK);
  await access(backend, constants.X_OK);
  const chrome = spawnSync('/usr/bin/google-chrome', ['--version'], { encoding: 'utf8', timeout: 5000 });
  assert.equal(chrome.status, 0, 'Installed Chrome preflight failed');
  evidence.chrome = chrome.stdout.trim();
  if (output) await mkdir(output, { recursive: true });
  server = spawn(backend, ['--dist', dist, '--standalone', standalone, '--port', String(port)], {
    env: { ...process.env, BACKEND_ONLY_SECRET: dummy },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  await new Promise((ready, reject) => {
    const timer = setTimeout(() => reject(new Error('Backend readiness timed out')), 10000);
    const onExit = () => { clearTimeout(timer); reject(new Error('Backend exited before readiness')); };
    server.once('error', (error) => { clearTimeout(timer); reject(error); });
    server.once('exit', onExit);
    server.stderr.on('data', (bytes) => { serverLog = (serverLog + bytes).slice(-32768); });
    server.stdout.on('data', (bytes) => {
      serverLog = (serverLog + bytes).slice(-32768);
      if (serverLog.includes(`ASSET_CONTRACT_READY ${origin}`)) {
        clearTimeout(timer);
        server.removeListener('exit', onExit);
        ready();
      }
    });
  });
  browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true });

  for (const profile of ['plain', 'tailwind']) {
    const manifest = JSON.parse(await readFile(join(dist, profile, '.vite/manifest.json'), 'utf8'));
    const entry = manifest[`front/${profile}.ts`];
    assert(entry.isEntry && entry.imports?.length && entry.css?.length, 'Missing emitted static dependency/CSS');
    assert(Object.values(manifest).some((chunk) => chunk.assets?.some((path) => path.endsWith('.svg'))));
    assert(Object.values(manifest).some((chunk) => chunk.dynamicImports?.length));
    const lazy = manifest['front/lazy.ts'];
    assert(lazy?.isDynamicEntry && lazy.css?.length, 'Missing emitted lazy JS/CSS');
    const errors = [];
    const requests = [];
    const responses = [];
    const context = await secureContext(true);
    await context.addInitScript(() => {
      window.viteSpikeViolations = [];
      document.addEventListener('securitypolicyviolation', (event) => {
        window.viteSpikeViolations.push(`${event.violatedDirective}: ${event.blockedURI}`);
      });
    });
    const page = await context.newPage();
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('request', (request) => requests.push(request.url()));
    page.on('response', (response) => responses.push({
      url: response.url(), status: response.status(), mime: response.headers()['content-type'],
      nosniff: response.headers()['x-content-type-options'], cache: response.headers()['cache-control'],
    }));
    const response = await page.goto(`${origin}/${profile}`, { waitUntil: 'networkidle', timeout: 15000 });
    assert.equal(response.status(), 200);
    assert.equal(response.headers()['content-security-policy'], csp);
    assert.equal(response.headers()['cache-control'], 'no-store');
    const html = await response.text();
    assert(!html.includes(dummy) && !html.includes('BACKEND_ONLY_SECRET'));
    assert(html.includes('&lt;safe &amp; useful&gt;'));
    assert(!html.includes('<style') && !html.includes('style=') && !html.includes('data:'));
    assert.equal(await page.title(), `Asset contract | ${profile}`);
    assert.equal(await page.locator('#ssr-proof').innerText(), 'Rust SSR survives without JavaScript: <safe & useful>');
    await page.waitForFunction((name) => document.querySelector('#load-lazy')?.dataset.ready === name, profile);
    assert.equal(await page.locator('#shared-proof').evaluate((node) => getComputedStyle(node).borderLeftWidth), '7px');
    if (profile === 'plain') {
      assert.equal(await page.locator('#plain-proof').evaluate((node) => getComputedStyle(node).backgroundColor), 'rgb(219, 234, 254)');
    } else {
      assert.equal(await page.locator('#tailwind-proof').evaluate((node) => getComputedStyle(node).paddingTop), '16px');
      assert.equal(await page.locator('#tailwind-proof').evaluate((node) => getComputedStyle(node).fontWeight), '700');
    }
    assert(await page.locator('#local-mark').evaluate((node) => node.complete && node.naturalWidth === 80));
    assert.equal(await page.locator('#lazy-result').evaluate((node) => getComputedStyle(node).color), 'rgb(71, 85, 105)');
    const lazyPaths = [lazy.file, ...lazy.css].map((path) => `${origin}/${profile}/${path}`);
    assert(lazyPaths.every((url) => !requests.includes(url)), 'Lazy assets fetched before interaction');
    const initialStyles = await page.locator('link[rel="stylesheet"]').evaluateAll((nodes) => nodes.map((node) => node.href));
    assert(initialStyles[0].includes('shared-') && initialStyles[1].includes('app-'));
    assert.equal(new Set(initialStyles).size, initialStyles.length);

    await page.locator('#load-lazy').click();
    await page.waitForFunction(() => document.querySelector('#lazy-result')?.textContent === 'Lazy module loaded: 1');
    assert.equal(await page.locator('#lazy-result').evaluate((node) => getComputedStyle(node).color), 'rgb(126, 34, 206)');
    assert.equal(await page.locator('#lazy-result').evaluate((node) => getComputedStyle(node).borderBottomWidth), '3px');
    await page.locator('#load-lazy').click();
    await page.waitForFunction(() => document.querySelector('#lazy-result')?.textContent === 'Lazy module loaded: 2');

    const afterStyles = await page.locator('link[rel="stylesheet"]').evaluateAll((nodes) => nodes.map((node) => node.href));
    assert.deepEqual(afterStyles.slice(0, initialStyles.length), initialStyles, 'Lazy load reordered initial CSS');
    assert.equal(new Set(afterStyles).size, afterStyles.length, 'Shared CSS duplicated');
    assert.equal(afterStyles.length, initialStyles.length + lazy.css.length);
    assert.equal(await page.locator('#shared-proof').evaluate((node) => getComputedStyle(node).borderLeftWidth), '7px');
    for (const url of lazyPaths) {
      const asset = responses.find((item) => item.url === url);
      assert.equal(asset?.status, 200, `Lazy response missing: ${url}`);
      assert.equal(asset.mime, url.endsWith('.css') ? 'text/css; charset=utf-8' : 'text/javascript; charset=utf-8');
    }
    for (const asset of responses) {
      assert.equal(asset.status, 200);
      assert.equal(new URL(asset.url).origin, origin);
      assert.equal(asset.nosniff, 'nosniff');
      assert.equal(asset.cache, 'no-store');
      if (asset.url.endsWith('.svg')) assert.equal(asset.mime, 'image/svg+xml');
    }
    assert.deepEqual(await page.evaluate(() => window.viteSpikeViolations), []);
    assert.deepEqual(errors, []);
    if (output) await page.screenshot({ path: join(output, `${profile}.png`) });
    await context.close();

    const noJs = await secureContext(false);
    const nativePage = await noJs.newPage();
    await nativePage.goto(`${origin}/${profile}`, { waitUntil: 'networkidle', timeout: 15000 });
    assert((await nativePage.locator('#ssr-proof').innerText()).includes('<safe & useful>'));
    assert.equal(await nativePage.locator('#lazy-result').innerText(), 'Lazy module has not loaded.');
    assert.equal(await nativePage.locator('#load-lazy').getAttribute('data-ready'), null);
    await noJs.close();
    const badUrls = [
      `/${profile}/.vite/manifest.json`, `/${profile}/assets/not-allowed.js`,
      `/${profile}/assets/%2e%2e%2fpackage.json`, `/${profile}/assets/%00.js`,
      `/${profile}/../Cargo.toml`, `/${profile}/${entry.file}?unexpected=1`,
    ];
    for (const path of badUrls) {
      const bad = await fetch(`${origin}${path}`, { redirect: 'manual', signal: AbortSignal.timeout(5000) });
      assert.equal(bad.status, 404, path);
      await bad.arrayBuffer();
    }
    evidence.profiles.push({ profile, status: 'pass', js_disabled_ssr: true, csp_violations: [],
      console_errors: errors, lazy_responses: lazyPaths, responses, bad_urls_404: badUrls });
  }

  // Independent standalone CLI CSS consumer: meaningful SSR with zero application JS.
  for (const javaScriptEnabled of [true, false]) {
    const context = await secureContext(javaScriptEnabled);
    const page = await context.newPage();
    const errors = [];
    const responses = [];
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
    page.on('response', (response) => responses.push({ url: response.url(), status: response.status(), headers: response.headers() }));
    if (javaScriptEnabled) await context.addInitScript(() => {
      window.assetViolations = [];
      document.addEventListener('securitypolicyviolation', (e) => window.assetViolations.push(e.violatedDirective));
    });
    const response = await page.goto(`${origin}/standalone`, { waitUntil: 'networkidle', timeout: 15000 });
    assert.equal(response.status(), 200);
    assert.equal(response.headers()['content-security-policy'], csp);
    const html = await response.text();
    assert(html.includes('&lt;safe &amp; useful&gt;') && !html.includes(dummy));
    assert(!html.includes('<script') && !html.includes('<style') && !html.includes('style='));
    assert((await page.locator('#ssr-proof').innerText()).includes('<safe & useful>'));
    assert.equal(await page.locator('#tailwind-proof').evaluate((node) => getComputedStyle(node).paddingTop), '16px');
    assert.equal(await page.locator('#tailwind-proof').evaluate((node) => getComputedStyle(node).fontWeight), '700');
    assert.equal(await page.locator('script').count(), 0);
    assert(responses.some((r) => r.url.endsWith('/assets/standalone.css') && r.headers['content-type'] === 'text/css; charset=utf-8'));
    assert(!responses.some((r) => r.url.endsWith('.js')));
    for (const r of responses) {
      assert.equal(r.status, 200); assert.equal(r.headers['cache-control'], 'no-store');
      assert.equal(r.headers['x-content-type-options'], 'nosniff');
    }
    if (javaScriptEnabled) assert.deepEqual(await page.evaluate(() => window.assetViolations), []);
    assert.deepEqual(errors, []);
    if (output && javaScriptEnabled) await page.screenshot({ path: join(output, 'standalone.png') });
    await context.close();
    evidence.profiles.push({ profile: 'standalone', javaScriptEnabled, status: 'pass', responses });
  }
  for (const path of ['/standalone/assets/missing.css', '/standalone/assets/standalone.css?x=1', '/standalone/.vite/manifest.json']) {
    const response = await fetch(`${origin}${path}`, { signal: AbortSignal.timeout(5000) });
    assert.equal(response.status, 404); await response.arrayBuffer();
  }
  evidence.status = 'pass';
} catch (error) {
  evidence.status = 'fail';
  evidence.error = error.stack ?? String(error);
  process.exitCode = 1;
} finally {
  try { await cleanup(); } catch (error) {
    evidence.status = 'fail';
    evidence.cleanup.error = String(error);
    process.exitCode = 1;
  }
  if (output) {
    await writeFile(join(output, 'browser.json'), `${JSON.stringify(evidence, null, 2)}\n`);
    await writeFile(join(output, 'backend.log'), serverLog);
  }
  console.log(JSON.stringify(evidence, null, 2));
}
