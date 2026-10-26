// App-owned browser dev loop. Production build never imports this entry.
import { createServer, preview, normalizePath } from 'vite';
import { randomBytes } from 'node:crypto';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative, isAbsolute, sep } from 'node:path';
import production from './vite.config.mjs';
import { buildWasm, projectRoot } from './build-wasm.mjs';

const root = realpathSync(production.root);
const port = 5173, origin = `http://127.0.0.1:${port}`;
const placeholder = 'YETLET_REQUEST_NONCE';
const publicPaths = new Set(['/', '/@vite/client', '/@vite/env', '/node_modules/vite/dist/client/env.mjs']);

function sourcePath(raw) {
  if (typeof raw !== 'string' || raw.length > 2048) return;
  const [path, query, extra] = raw.split('?');
  if (extra !== undefined || (query !== undefined && !/^(?:import|direct|t=[0-9]{1,16})(?:&(?:import|direct|t=[0-9]{1,16}))*$/.test(query))) return;
  if (publicPaths.has(path)) return path;
  if (!path.startsWith('/') || /%2f|%5c/i.test(path)) return;
  let decoded;
  try { decoded = decodeURIComponent(path); } catch { return; }
  if (/[\\%?#:\u0000-\u001f\u007f]/.test(decoded)) return;
  const parts = decoded.slice(1).split('/');
  if (parts.some((part) => !part || part.startsWith('.') || /^(?:node_modules|dist|target|generated)$/i.test(part))) return;
  if (!/\.(?:m?js|m?ts|css|svg)$/.test(parts.at(-1))
    || /^(?:vite\..*|build-wasm|serve|postcss\.config|tailwind\.config)\./i.test(parts.at(-1))) return;
  try {
    const actual = relative(root, realpathSync(join(root, decoded.slice(1))));
    if (isAbsolute(actual) || actual === '..' || actual.startsWith(`..${sep}`)) return;
    // Refuse aliases to config, dotenv, node_modules or other denied files too.
    if (actual.split(sep).some((part) => part.startsWith('.') || /^(?:node_modules|dist|target|generated)$/i.test(part))
      || !/\.(?:m?js|m?ts|css|svg)$/.test(actual)
      || /^(?:vite\..*|build-wasm|serve|postcss\.config|tailwind\.config)\./i.test(actual.split(sep).at(-1))) return;
  } catch { return; }
  return decoded;
}

async function watch() {
  const temporary = await mkdtemp(join(tmpdir(), 'yetlet-browser-watch-'));
  const generations = new Map();
  const controller = new AbortController();
  let current = 0, sequence = 0, pending = false, active, timer, stopping = false, server;
  const watched = [join(projectRoot, 'src'), ...['Cargo.toml', 'Cargo.lock', 'build.rs'].map((name) => join(projectRoot, name))].map(normalizePath);
  const rustInput = (path) => watched.some((item) => normalizePath(path) === item || normalizePath(path).startsWith(item + '/'));
  async function build() {
    while (pending && !stopping) {
      pending = false;
      const number = ++sequence, directory = join(temporary, String(number));
      console.log(`YT_BROWSER_BUILD ${number} started`);
      try {
        await buildWasm(directory, controller.signal);
        // Read the complete pair before publishing; HTTP never sees bindgen writes.
        const js = await readFile(join(directory, 'tasks.js'));
        const wasm = await readFile(join(directory, 'tasks_bg.wasm'));
        if (!stopping) {
          generations.set(number, { js, wasm });
          current = number;
          server.environments.client.moduleGraph.invalidateAll();
          server.ws.send({ type: 'full-reload', path: '*' });
          console.log(`YT_BROWSER_BUILD ${number} published`);
        }
      } catch (error) {
        if (!stopping) console.error(`YT_BROWSER_BUILD ${number} failed; ${current ? 'keeping last good generation' : 'runtime unavailable'}; waiting for edits`, error.message);
      } finally {
        await rm(directory, { recursive: true, force: true });
      }
    }
  }
  function schedule() {
    if (stopping) return;
    pending = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (!active) active = build().finally(() => { active = undefined; });
    }, 80);
  }
  const plugin = {
    name: 'yetlet-browser-watch',
    // Rewrite the scaffold entry before Vite's TypeScript transform normalizes imports.
    enforce: 'pre',
    transform(code, id) {
      if (id.split('?')[0] !== normalizePath(join(root, 'app.ts')) || !current) return;
      return `const __yetletWasmModuleUrl = '/__yetlet_wasm/${current}/tasks.js';\n` + code.replace("await import('./generated/tasks.js')", 'await import(/* @vite-ignore */ __yetletWasmModuleUrl)')
        .replace("await import('./generated/tasks_bg.wasm?url')", `await Promise.resolve({ default: '/__yetlet_wasm/${current}/tasks_bg.wasm' })`);
    },
    // Rust events belong to our serialized build loop, never Vite's reload path.
    handleHotUpdate(context) { if (rustInput(context.file)) return []; },
    configureServer(vite) {
      vite.watcher.add(watched);
      vite.watcher.on('all', (_event, path) => { if (rustInput(path)) schedule(); });
      vite.httpServer.prependListener('upgrade', (req, socket) => {
        if (req.headers.host !== `127.0.0.1:${port}` || req.headers.origin !== origin
          || !/^\/\?token=[A-Za-z0-9_-]{12}$/.test(req.url ?? '')) socket.destroy();
      });
      vite.middlewares.use(async (req, res, next) => {
        const reply = (status, body, type = 'text/plain; charset=utf-8') => {
          res.statusCode = status;
          res.setHeader('Content-Type', type);
          res.end(req.method === 'HEAD' ? undefined : body);
        };
        res.setHeader('Cache-Control', 'no-store');
        res.setHeader('X-Content-Type-Options', 'nosniff');
        if (req.headers.host !== `127.0.0.1:${port}` || (req.headers.origin !== undefined && req.headers.origin !== origin)) return reply(403, 'Development request refused\n');
        if (!['GET', 'HEAD'].includes(req.method)) return reply(405, 'Read-only development profile\n');
        // Vite annotates dynamic JS imports; only that exact module query is admitted.
        const asset = /^\/__yetlet_wasm\/([1-9][0-9]*)\/(tasks\.js(?:\?import)?|tasks_bg\.wasm)$/.exec(req.url ?? '');
        if (asset) {
          const pair = generations.get(Number(asset[1]));
          if (!pair) return reply(404, 'Generation unavailable\n');
          const module = asset[2] !== 'tasks_bg.wasm';
          return reply(200, module ? pair.js : pair.wasm, module ? 'text/javascript' : 'application/wasm');
        }
        const path = sourcePath(req.url);
        if (!path) return reply(403, 'Development request refused\n');
        if (!current) return reply(503, 'Browser runtime unavailable. Correct the Rust build and retry after a successful generation.\n');
        if (path !== '/') return next();
        try {
          const nonce = randomBytes(18).toString('base64');
          const source = await readFile(join(root, 'index.html'), 'utf8');
          const html = (await vite.transformIndexHtml('/', source))
            .replaceAll(placeholder, nonce)
            .replace("style-src 'self'", `style-src 'self' 'nonce-${nonce}'`)
            .replace("connect-src 'self'", `connect-src 'self' ws://127.0.0.1:${port}`);
          const csp = /http-equiv="Content-Security-Policy" content="([^"]+)"/.exec(html)?.[1];
          if (!csp) throw new Error('Expected browser CSP');
          res.setHeader('Content-Security-Policy', csp);
          reply(200, html, 'text/html; charset=utf-8');
        } catch { reply(503, 'Browser document unavailable\n'); }
      });
    },
  };
  async function close() {
    if (stopping) return;
    stopping = true;
    clearTimeout(timer);
    controller.abort();
    await server?.close();
    await active;
    await rm(temporary, { recursive: true, force: true });
    console.log('YT_BROWSER_STOP closed Vite and removed owned generations');
  }
  const stop = () => { close().catch((error) => { console.error(error); process.exitCode = 1; }); };
  process.once('SIGINT', stop);
  process.once('SIGTERM', stop);
  if (process.platform === 'win32') process.once('SIGBREAK', stop);
  try {
    server = await createServer({
      ...production, configFile: false, base: '/', appType: 'custom',
      html: { cspNonce: placeholder },
      plugins: [...production.plugins, plugin],
      optimizeDeps: { noDiscovery: true, include: [] },
      server: {
        host: '127.0.0.1', port, strictPort: true, allowedHosts: [], origin,
        cors: { origin }, ws: { protocol: 'ws', host: '127.0.0.1', port, clientPort: port },
        fs: { strict: true, allow: [root], deny: ['.env', '.env.*', '*.{toml,lock,crt,pem,key}', '**/.git/**', '**/.aws/**', '**/.ssh/**'] },
      },
    });
    if (stopping) { await server.close(); return; }
    await server.listen();
    server.printUrls();
    schedule();
  } catch (error) {
    await close();
    throw error;
  }
}

if (process.env.YETLET_BROWSER_WATCH === '1') await watch();
else {
  const server = await preview({ ...production, configFile: false });
  server.printUrls();
}
