// Narrow YT-100 browser gate. This script is not scaffolded or used by production.
// Run only against disposable, already-built generated Vite apps.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { once } from 'node:events';
import { chromium } from '../../../experiments/vite-integration/node_modules/playwright-core/index.mjs';

const args = new Map();
const flags = new Set(['--cloud', '--config']);
for (let i = 2; i < process.argv.length; i++) {
  const key = process.argv[i];
  if (flags.has(key)) { assert(!args.has(key), 'Duplicate flag'); args.set(key, true); continue; }
  assert(['--app', '--backend', '--mode', '--out', '--port'].includes(key), 'Unsupported argument');
  assert(!args.has(key) && process.argv[i + 1], 'Missing/duplicate argument');
  args.set(key, process.argv[++i]);
}
for (const required of ['--app', '--backend', '--mode', '--out']) assert(args.has(required), required);
const mode = args.get('--mode');
assert(['plain', 'tailwind'].includes(mode));
const app = resolve(args.get('--app'));
const backendBin = resolve(args.get('--backend'));
const outputDir = resolve(args.get('--out'));
const vitePort = args.has('--port') ? Number(args.get('--port')) : 5173;
assert(Number.isInteger(vitePort) && vitePort >= 1024 && vitePort <= 65535 && vitePort !== 3000);
const devBase = 'http://127.0.0.1:' + vitePort;
const backendBase = 'http://127.0.0.1:3000';
const cloud = Boolean(args.get('--cloud')), config = Boolean(args.get('--config'));
const token = '000123-yt100-synthetic';
const result = { task: 'YT-100', mode, status: 'running', devBase, cloud, config, checks: {}, cleanup: {} };
const processes = [];
let browser;
let page;
let originalCss;
const cssPath = join(app, 'shared.css');
const logs = {};
const expectedCsp = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

async function terminate(name) {
  const record = processes.find((p) => p.name === name && !p.stopped);
  if (!record) return;
  const child = record.child;
  if (child.exitCode === null && child.signalCode === null) {
    const complete = once(child, 'exit');
    child.kill('SIGTERM');
    let timeout;
    try {
      await Promise.race([complete, new Promise((_, reject) => {
        timeout = setTimeout(() => reject(new Error('Owned ' + name + ' failed to exit')), 7000);
      })]);
    } finally {
      clearTimeout(timeout);
    }
  }
  record.stopped = true;
  result.cleanup[name] = { exitCode: child.exitCode, signalCode: child.signalCode };
}
function launch(name, executable, params, environment, cwd) {
  const child = spawn(executable, params, {
    cwd, env: environment, stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true,
  });
  const record = { name, child, stopped: false };
  child.on('error', (error) => { logs[name] = (logs[name] ?? '') + 'process error: ' + error.message; });
  processes.push(record);
  logs[name] = '';
  child.stdout.on('data', (d) => { logs[name] = (logs[name] + d.toString()).slice(-32768); });
  child.stderr.on('data', (d) => { logs[name] = (logs[name] + d.toString()).slice(-32768); });
  return child;
}
const envBase = { PATH: process.env.PATH ?? '', HOME: process.env.HOME ?? '', USERPROFILE: process.env.USERPROFILE ?? '', RUSTUP_TOOLCHAIN: process.env.RUSTUP_TOOLCHAIN ?? '' };
const backendEnv = {
  ...envBase,
  YETLET_ASSET_ROOT: join(app, 'dist'), YETLET_HOST: '127.0.0.1',
  YETLET_PORT: '3000', APP_ENVIRONMENT: 'Production',
  APP_SERVER__HOST: '127.0.0.1', APP_SERVER__PORT: '3000',
  ...(config ? { APP_SERVICE__TOKEN: token } : {}),
};
const viteEnv = { ...envBase, NODE_ENV: 'development', YETLET_VITE_PORT: String(vitePort) };

async function fetchStatus(url, expected, timeoutMs = 12000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(900) });
      if (response.status === expected) return response;
    } catch {}
    await new Promise((done) => setTimeout(done, 100));
  }
  throw new Error('Expected HTTP ' + expected + ': ' + url);
}
async function checkPortClosed(url) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    try {
      await fetch(url, { signal: AbortSignal.timeout(650) });
    } catch {
      return;
    }
    await new Promise((done) => setTimeout(done, 100));
  }
  throw new Error('Owned listener remained open: ' + url);
}

try {
  assert(existsSync(backendBin), 'Missing explicit prebuilt Rust app');
  assert(existsSync(join(app, 'node_modules/vite/bin/vite.js')), 'Run explicit npm ci first');
  assert(existsSync(join(app, 'dist/.vite/manifest.json')), 'Run explicit npm run build first');
  await mkdir(outputDir);
  originalCss = await readFile(cssPath, 'utf8');
  assert(originalCss.includes('7px'), 'Expected task-owned unmodified shared CSS fixture');

  launch('rust', backendBin, [], backendEnv, app);
  await fetchStatus(backendBase + '/', 200);
  const direct = await fetchStatus(backendBase + '/', 200);
  assert.equal(direct.headers.get('content-security-policy'), expectedCsp, 'Production CSP was modified');
  if (cloud) {
    for (const path of ['/health/live', '/health/ready']) {
      assert.equal((await fetchStatus(backendBase + path, 200)).status, 200);
    }
  }
  result.checks.backendProductionCspAndHealth = 'pass';

  launch('vite', process.execPath, [join(app, 'node_modules/vite/bin/vite.js'), '--config', 'vite.dev.config.mjs'], viteEnv, app);
  await fetchStatus(devBase + '/', 200);
  browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true });
  const context = await browser.newContext({ viewport: { width: 840, height: 650 } });
  const errors = [], cspViolations = [], websockets = [];
  await context.route('**/*', async (route) => {
    assert.equal(new URL(route.request().url()).origin, devBase, 'External request blocked');
    await route.continue();
  });
  await context.addInitScript(() => {
    window.__yt100_csp = [];
    document.addEventListener('securitypolicyviolation', (event) => window.__yt100_csp.push(event.violatedDirective));
  });
  page = await context.newPage();
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  page.on('pageerror', (e) => errors.push(e.message));
  page.on('websocket', (s) => websockets.push(s.url()));
  const initial = await page.goto(devBase + '/', { waitUntil: 'networkidle', timeout: 15000 });
  assert.equal(initial.status(), 200);
  const devCsp = initial.headers()['content-security-policy'];
  assert(devCsp.includes("style-src 'self' 'nonce-") && devCsp.includes('connect-src \'self\' ws://127.0.0.1:' + vitePort));
  assert(!devCsp.includes('unsafe-inline') && !devCsp.includes('unsafe-eval') && !devCsp.includes('data:'));
  assert((await initial.text()).includes('&lt;safe &amp; useful&gt;'));
  await page.waitForFunction(() => document.querySelector('#load-lazy')?.dataset.ready === 'vite');
  assert.equal(await page.locator('#shared-proof').evaluate((n) => getComputedStyle(n).borderLeftWidth), '7px');
  assert(await page.locator('#local-mark').evaluate((n) => n.complete && n.naturalWidth === 80));
  if (mode === 'tailwind') {
    assert.equal(await page.locator('#tailwind-proof').evaluate((n) => getComputedStyle(n).paddingTop), '16px');
  }
  await page.waitForFunction(() => [...document.querySelectorAll('style')].some((s) => s.nonce && s.nonce === document.querySelector('meta[property=csp-nonce]')?.nonce));
  assert(websockets.some((url) => url.startsWith('ws://127.0.0.1:' + vitePort)), 'HMR websocket missing');
  result.checks.devStyleNonceAndWebSocket = 'pass';
  await page.evaluate(() => { window.__yt100_marker = 'kept'; });
  const updated = originalCss.replace('7px', '11px');
  await writeFile(cssPath, updated);
  await page.waitForFunction(() => getComputedStyle(document.querySelector('#shared-proof')).borderLeftWidth === '11px', null, { timeout: 20000 });
  assert.equal(await page.evaluate(() => window.__yt100_marker), 'kept', 'Page was reloaded instead of CSS HMR');
  assert.deepEqual(await page.evaluate(() => window.__yt100_csp), []);
  assert.deepEqual(errors, []);
  result.checks.actualCssHmrNoDocumentReload = 'pass';
  // Give Vite's filesystem watcher a distinct update window for fixture restore.
  // Without spacing, the immediate second write can coalesce with the first event.
  await new Promise((done) => setTimeout(done, 1000));
  await writeFile(cssPath, originalCss); originalCss = undefined;
  await page.waitForFunction(() => getComputedStyle(document.querySelector('#shared-proof')).borderLeftWidth === '7px', null, { timeout: 20000 });
  result.checks.actualCssRestore = 'pass';
  const bad = ['/base.toml', '/.env.local', '/vite.dev.config.mjs', '/src/main.rs', '/@fs/etc/passwd', '/app.ts?raw'];
  for (const path of bad) {
    const response = await fetch(devBase + path, { signal: AbortSignal.timeout(4000) });
    assert(response.status === 403 || response.status === 404, 'Denied path exposed: ' + path + ' ' + response.status);
  }
  result.checks.devSourceDenied = 'pass';
  await page.screenshot({ path: join(outputDir, 'hmr-' + mode + '.png') });
  await page.close();
  await context.close();
  const noJs = await browser.newContext({ javaScriptEnabled: false });
  const noJsPage = await noJs.newPage();
  const noJsResponse = await noJsPage.goto(devBase + '/', { waitUntil: 'domcontentloaded', timeout: 15000 });
  assert.equal(noJsResponse.status(), 200);
  assert((await noJsPage.locator('#ssr-proof').innerText()).includes('<safe & useful>'));
  await noJs.close();
  result.checks.noJsRustSsr = 'pass';

  await terminate('rust');
  assert.equal((await fetchStatus(devBase + '/', 503)).status, 503);
  result.checks.backendUnavailable503 = 'pass';
  launch('rust', backendBin, [], backendEnv, app);
  await fetchStatus(devBase + '/', 200);
  result.checks.backendRestart = 'pass';
  await terminate('vite');
  await checkPortClosed(devBase + '/@vite/client');
  result.checks.devStop = 'pass';
  launch('vite', process.execPath, [join(app, 'node_modules/vite/bin/vite.js'), '--config', 'vite.dev.config.mjs'], viteEnv, app);
  const restart = await fetchStatus(devBase + '/', 200);
  assert(restart.headers.get('content-security-policy')?.includes("style-src 'self' 'nonce-"));
  result.checks.devRestart = 'pass';
  await terminate('vite');
  await checkPortClosed(devBase + '/@vite/client');
  await terminate('rust');
  await checkPortClosed(backendBase + '/');
  result.checks.devStopAndCleanup = 'pass';
  result.status = 'pass';
} catch (error) {
  result.status = 'fail';
  result.error = error.stack ?? String(error);
  process.exitCode = 1;
} finally {
  if (originalCss !== undefined) {
    try { await writeFile(cssPath, originalCss); result.cleanup.cssRestored = true; }
    catch (error) { result.status = 'fail'; result.cleanup.cssRestoreError = String(error); process.exitCode = 1; }
  } else result.cleanup.cssRestored = true;
  if (page && !page.isClosed()) await page.close().catch(() => {});
  if (browser) { await browser.close().catch(() => {}); result.cleanup.browserClosed = true; }
  for (const { name } of [...processes].reverse()) {
    try { await terminate(name); } catch (error) { result.status = 'fail'; result.cleanup[name + 'Error'] = String(error); process.exitCode = 1; }
  }
  if (existsSync(outputDir)) {
    await writeFile(join(outputDir, 'result.json'), JSON.stringify(result, null, 2) + '\n');
    await writeFile(join(outputDir, 'rust.log'), logs.rust ?? '');
    await writeFile(join(outputDir, 'vite.log'), logs.vite ?? '');
  }
  console.log(JSON.stringify(result, null, 2));
}
