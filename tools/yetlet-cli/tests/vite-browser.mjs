// YT-099: focused, opt-in actual browser acceptance for one generated Vite app.
// This QA script is not scaffolded, installed, or used by the Rust runtime.
import assert from 'node:assert/strict';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { resolve, join } from 'node:path';
import { chromium } from '../../../experiments/vite-integration/node_modules/playwright-core/index.mjs';

const argumentsMap = new Map();
let isCloud = false;
let withConfig = false;
for (let i = 2; i < process.argv.length; ++i) {
  const key = process.argv[i];
  if (key === '--cloud' || key === '--config') {
    assert(!(key === '--cloud' ? isCloud : withConfig), 'Duplicate flag');
    if (key === '--cloud') isCloud = true; else withConfig = true;
    continue;
  }
  assert(['--binary', '--dist', '--port', '--mode', '--out'].includes(key), 'Unknown flag');
  assert(!argumentsMap.has(key) && process.argv[i + 1], 'Missing or duplicate value');
  argumentsMap.set(key, process.argv[++i]);
}
for (const required of ['--binary', '--dist', '--port', '--mode']) {
  assert(argumentsMap.has(required), 'Missing required argument: ' + required);
}
const mode = argumentsMap.get('--mode');
assert(['plain', 'tailwind'].includes(mode), 'Expected plain or tailwind mode');
const port = Number(argumentsMap.get('--port'));
assert(Number.isInteger(port) && port > 0 && port <= 65535, 'Invalid port');
const origin = 'http://127.0.0.1:' + port;
const binary = resolve(argumentsMap.get('--binary'));
const dist = resolve(argumentsMap.get('--dist'));
const output = argumentsMap.has('--out') ? resolve(argumentsMap.get('--out')) : null;
const sentinel = '000123-yt099-synthetic-only';
const strictCsp = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const evidence = { task_id: 'YT-099', mode, withConfig, isCloud, status: 'running', cleanup: {} };
let browser;
let server;
let backendLog = '';
let cleanupPromise;

function cleanup() {
  return cleanupPromise ??= (async () => {
    if (browser) {
      await browser.close();
      evidence.cleanup.browser_closed = true;
    }
    if (server && server.exitCode === null && server.signalCode === null) {
      const done = once(server, 'exit');
      server.kill('SIGTERM');
      let timer;
      try {
        await Promise.race([
          done,
          new Promise((resolveDone) => {
            timer = setTimeout(() => { server.kill('SIGKILL'); resolveDone(); }, 5000);
          }),
        ]);
        if (server.exitCode === null && server.signalCode === null) await done;
      } finally {
        clearTimeout(timer);
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

try {
  await access('/usr/bin/google-chrome', constants.X_OK);
  await access(binary, constants.X_OK);
  await access(join(dist, '.vite/manifest.json'), constants.R_OK);
  const version = spawnSync('/usr/bin/google-chrome', ['--version'], { encoding: 'utf8', timeout: 5000 });
  assert.equal(version.status, 0, 'Chrome preflight');
  evidence.chrome = version.stdout.trim();
  if (output) await mkdir(output);
  const permittedEnv = {
    PATH: process.env.PATH ?? '/usr/bin:/bin',
    YETLET_ASSET_ROOT: dist,
    YETLET_HOST: '127.0.0.1',
    YETLET_PORT: String(port),
    APP_ENVIRONMENT: 'Production',
    APP_SERVER__HOST: '127.0.0.1',
    APP_SERVER__PORT: String(port),
    ...(withConfig ? { APP_SERVICE__TOKEN: sentinel } : {}),
  };
  server = spawn(binary, [], {
    cwd: resolve(join(dist, '..')),
    env: permittedEnv,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  await new Promise((ready, reject) => {
    const timeout = setTimeout(() => reject(new Error('Backend readiness timeout')), 12000);
    const onExit = () => { clearTimeout(timeout); reject(new Error('Backend exited before readiness')); };
    server.once('error', (err) => { clearTimeout(timeout); reject(err); });
    server.once('exit', onExit);
    server.stderr.on('data', (data) => { backendLog = (backendLog + data).slice(-32768); });
    server.stdout.on('data', (data) => {
      backendLog = (backendLog + data).slice(-32768);
      if (backendLog.includes('Listening on http://127.0.0.1:' + port)) {
        clearTimeout(timeout);
        server.removeListener('exit', onExit);
        ready();
      }
    });
  });
  browser = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true });
  const manifest = JSON.parse(await readFile(join(dist, '.vite/manifest.json'), 'utf8'));
  const entry = manifest['app.ts'];
  assert(entry?.isEntry && entry?.imports?.length && entry?.css?.length, 'No static CSS graph');
  const lazy = manifest['lazy.ts'];
  assert(lazy?.isDynamicEntry && lazy?.css?.length, 'No lazy CSS');
  const seenAssets = [];
  for (const javaScriptEnabled of [true, false]) {
    const context = await browser.newContext({ javaScriptEnabled, viewport: { width: 840, height: 650 } });
    const errors = [];
    await context.route('**/*', (route) => {
      assert.equal(new URL(route.request().url()).origin, origin, 'External request refused');
      return route.continue();
    });
    if (javaScriptEnabled) await context.addInitScript(() => {
      window.viteAcceptanceViolations = [];
      document.addEventListener('securitypolicyviolation', (event) => {
        window.viteAcceptanceViolations.push(event.violatedDirective);
      });
    });
    try {
      const page = await context.newPage();
      page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
      page.on('pageerror', (error) => errors.push(error.message));
      page.on('response', (res) => {
        seenAssets.push({
          url: res.url(), status: res.status(), mime: res.headers()['content-type'],
          nosniff: res.headers()['x-content-type-options'], cache: res.headers()['cache-control'],
        });
      });
      const response = await page.goto(origin + '/', { waitUntil: 'networkidle', timeout: 15000 });
      assert.equal(response.status(), 200);
      assert.equal(response.headers()['content-security-policy'], strictCsp);
      assert.equal(response.headers()['cache-control'], 'no-store');
      const html = await response.text();
      assert(html.includes('&lt;safe &amp; useful&gt;'), 'Missing escaped Rust SSR');
      assert(!html.includes(sentinel), 'Secret sentinel leaked into HTML');
      assert(!html.includes('<style') && !html.includes('unsafe-inline') && !html.includes('data:'), 'CSP regression');
      assert((await page.locator('#ssr-proof').innerText()).includes('<safe & useful>'));
      assert.equal(await page.locator('#shared-proof').evaluate((el) => getComputedStyle(el).borderLeftWidth), '7px');
      assert(await page.locator('#local-mark').evaluate((el) => el.complete && el.naturalWidth === 80), 'SVG missing');
      if (mode === 'tailwind') {
        assert.equal(await page.locator('#tailwind-proof').evaluate((el) => getComputedStyle(el).paddingTop), '16px');
        assert.equal(await page.locator('#tailwind-proof').evaluate((el) => getComputedStyle(el).fontWeight), '700');
      } else {
        assert.equal(await page.locator('#plain-proof').evaluate((el) => getComputedStyle(el).backgroundColor), 'rgb(219, 234, 254)');
      }
      if (javaScriptEnabled) {
        await page.waitForFunction(() => document.querySelector('#load-lazy')?.dataset.ready === 'vite');
        const initial = await page.locator('link[rel="stylesheet"]').evaluateAll((nodes) => nodes.map((node) => node.href));
        assert(initial[0].includes('shared-') && initial[1].includes('app-'));
        assert.equal(new Set(initial).size, initial.length);
        const lazyPaths = [lazy.file, ...lazy.css].map((p) => origin + '/static/' + p);
        assert(lazyPaths.every((p) => !seenAssets.some((a) => a.url === p)), 'Lazy imported early');
        for (const count of [1, 2]) {
          await page.locator('#load-lazy').click();
          await page.waitForFunction((c) => document.querySelector('#lazy-result')?.textContent === 'Lazy module loaded: ' + c, count);
        }
        assert.equal(await page.locator('#lazy-result').evaluate((el) => getComputedStyle(el).borderBottomWidth), '3px');
        const after = await page.locator('link[rel="stylesheet"]').evaluateAll((nodes) => nodes.map((node) => node.href));
        assert.deepEqual(after.slice(0, initial.length), initial, 'CSS order changed');
        assert.equal(new Set(after).size, after.length, 'CSS duplicate');
        assert.equal(after.length, initial.length + lazy.css.length);
        assert(lazyPaths.every((p) => seenAssets.some((a) => a.url === p && a.status === 200)));
        assert.deepEqual(await page.evaluate(() => window.viteAcceptanceViolations), []);
        if (output) await page.screenshot({ path: join(output, mode + '.png') });
      } else {
        assert.equal(await page.locator('#lazy-result').innerText(), 'Lazy module has not loaded.');
      }
      assert.deepEqual(errors, []);
      evidence['js_' + (javaScriptEnabled ? 'on' : 'off')] = 'pass';
    } finally {
      await context.close();
    }
  }
  for (const asset of seenAssets) {
    assert.equal(asset.status, 200);
    assert.equal(new URL(asset.url).origin, origin);
    assert.equal(asset.cache, 'no-store');
    assert.equal(asset.nosniff, 'nosniff');
  }
  if (isCloud) {
    for (const path of ['/health/live', '/health/ready']) {
      const reply = await fetch(origin + path, { signal: AbortSignal.timeout(5000) });
      assert.equal(reply.status, 200);
      await reply.arrayBuffer();
    }
  }
  for (const path of [
    '/static/.vite/manifest.json', '/static/assets/nonexistent.js',
    '/static/assets/%2e%2e%2fCargo.toml', '/static/assets/%00.js',
    '/static/assets/' + entry.file + '?unexpected=1',
  ]) {
    const reply = await fetch(origin + path, { redirect: 'manual', signal: AbortSignal.timeout(5000) });
    assert.equal(reply.status, 404, path);
    await reply.arrayBuffer();
  }
  evidence.responses = seenAssets;
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
    await writeFile(join(output, 'browser.json'), JSON.stringify(evidence, null, 2) + '\n');
    await writeFile(join(output, 'backend.log'), backendLog);
  }
  console.log(JSON.stringify(evidence, null, 2));
}
