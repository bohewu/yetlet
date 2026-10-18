// App-owned YT-100 development bridge, never imported by production Rust/build.
import { randomBytes } from 'node:crypto';
import { get } from 'node:http';
import { readFileSync, realpathSync } from 'node:fs';
import { isAbsolute, join, relative, sep } from 'node:path';
import { createSecureContext } from 'node:tls';

export const productionCsp = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const maxHtmlBytes = 256 * 1024;
const head = '<!doctype html><html lang="zh-Hant"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Yetlet</title>';
const assets = /^(?:<link rel="stylesheet" href="\/static\/assets\/[A-Za-z0-9_-]+\.css">){1,128}<script type="module" src="\/static\/assets\/[A-Za-z0-9_-]+\.js"><\/script><\/head><body>/;
const mark = /<img id="local-mark" src="\/static\/assets\/[A-Za-z0-9_-]+\.svg" alt="Local check mark" width="80" height="48">/g;
const publicPaths = new Set([
  '/', '/@vite/client', '/@vite/env', '/favicon.ico',
  // Vite 8.3.4 normalizes its internal @vite/env import to this exact app-local module.
  // The sole node_modules exception does not grant access to other packages/files.
  '/node_modules/vite/dist/client/env.mjs',
]);

export function devPort(value = '5173') {
  if (typeof value !== 'string' || !/^[1-9][0-9]{3,4}$/.test(value)) {
    throw new Error('YETLET_VITE_PORT must be ASCII decimal 1024-65535, except 3000');
  }
  const port = Number(value);
  if (port < 1024 || port > 65535 || port === 3000) throw new Error('Invalid Vite dev port');
  return port;
}

export function devTls(root, env = process.env) {
  const certPath = env.YETLET_DEV_TLS_CERT;
  const keyPath = env.YETLET_DEV_TLS_KEY;
  if (certPath === undefined && keyPath === undefined) return undefined;
  try {
    const frontend = realpathSync(root);
    for (const path of [certPath, keyPath]) {
      if (typeof path !== 'string' || !isAbsolute(path)) throw new Error();
      for (const candidate of [path, realpathSync(path)]) {
        const location = relative(frontend, candidate);
        if (!isAbsolute(location) && location !== '..' && !location.startsWith(`..${sep}`)) throw new Error();
      }
    }
    const https = { cert: readFileSync(certPath), key: readFileSync(keyPath) };
    createSecureContext(https);
    return https;
  } catch {
    // Never surface credential contents, paths or OpenSSL diagnostics in Vite logs.
    throw new Error('Invalid development TLS credentials');
  }
}

export function allowedOrigin(headers, port, websocket = false, https = false) {
  return headers.host === `127.0.0.1:${port}`
    && (headers.origin === `${https ? 'https' : 'http'}://127.0.0.1:${port}` || (!websocket && headers.origin === undefined));
}

export function allowedPath(raw) {
  // Decode once before URL normalization; encoded separators and traversal stay refused.
  if (typeof raw !== 'string' || raw.length > 2048) return false;
  const [path, query, extra] = raw.split('?');
  if (extra !== undefined || (query !== undefined
    && !/^(?:(?:import|direct)(?:&t=[0-9]{1,16})?|t=[0-9]{1,16}(?:&(?:import|direct))?)$/.test(query))) return false;
  if (publicPaths.has(path)) return true;
  if (!path.startsWith('/') || /%2f|%5c/i.test(path)) return false;
  let decoded;
  try { decoded = decodeURIComponent(path); } catch { return false; }
  if (decoded.startsWith('/@') || /[\\%?#:\u0000-\u001f\u007f]/.test(decoded)) return false;
  const parts = decoded.slice(1).split('/');
  if (parts.some((part) => !part || part.startsWith('.')
    || /^(?:node_modules|dist|target)$/i.test(part))) return false;
  const filename = parts.at(-1);
  return /\.(?:m?js|m?ts|css|svg)$/.test(filename)
    && !/^(?:vite(?:\.dev)?\.(?:config|bridge)|(?:tailwind|postcss)\.config)\./i.test(filename);
}

export function devDocument(html, port, https = false) {
  devPort(String(port));
  if (typeof html !== 'string' || Buffer.byteLength(html) > maxHtmlBytes || !html.startsWith(head)) {
    throw new Error('Unexpected backend HTML');
  }
  const tail = html.slice(head.length);
  const block = assets.exec(tail)?.[0];
  if (!block || (html.match(mark) ?? []).length !== 1 || !html.endsWith('</body></html>')) {
    throw new Error('Expected generated production asset markup');
  }
  const nonce = randomBytes(18).toString('base64');
  const body = tail.slice(block.length).replace(mark,
    `<img id="local-mark" src="${https ? 'https' : 'http'}://127.0.0.1:${port}/mark.svg" alt="Local check mark" width="80" height="48">`);
  return {
    nonce,
    csp: productionCsp.replace("style-src 'self'", `style-src 'self' 'nonce-${nonce}'`)
      .replace("connect-src 'self'", `connect-src 'self' ${https ? 'wss' : 'ws'}://127.0.0.1:${port}`),
    html: head + `<meta property="csp-nonce" nonce="${nonce}"><script type="module" src="/@vite/client"></script><script type="module" src="/app.ts"></script></head><body>` + body,
  };
}

function reply(res, status, body, csp = productionCsp, html = false) {
  res.writeHead(status, {
    'Content-Type': html ? 'text/html; charset=utf-8' : 'text/plain; charset=utf-8',
    'Content-Security-Policy': csp,
    'X-Content-Type-Options': 'nosniff',
    'Cache-Control': 'no-store',
  });
  res.end(body);
}

export function devBridge(port, https) {
  devPort(String(port));
  return {
    name: 'yetlet-local-dev-bridge',
    apply: 'serve',
    configResolved(config) {
      const resolvedTls = config.server.https;
      const expectedTls = https
        ? resolvedTls && Object.keys(resolvedTls).length === 2
          && Buffer.isBuffer(resolvedTls.cert) && resolvedTls.cert.equals(https.cert)
          && Buffer.isBuffer(resolvedTls.key) && resolvedTls.key.equals(https.key)
        : !resolvedTls;
      // CLI overrides must never silently select a public listener or another port.
      if (config.server.host !== '127.0.0.1' || config.server.port !== port
        || !config.server.strictPort || config.base !== '/' || !expectedTls
        || config.server.middlewareMode || config.server.ws?.port !== port
        || config.server.ws?.host !== '127.0.0.1'
        || config.server.ws?.protocol !== (https ? 'wss' : 'ws')) {
        throw new Error('Use the configured loopback development profile');
      }
    },
    configureServer(server) {
      const root = realpathSync(server.config.root);
      const pending = new Set();
      const upgrade = (req, socket) => {
        if (!allowedOrigin(req.headers, port, true, Boolean(https)) || !/^\/\?token=[A-Za-z0-9_-]{12}$/.test(req.url ?? '')) {
          socket.destroy();
        }
      };
      server.httpServer.prependListener('upgrade', upgrade);
      server.httpServer.once('close', () => {
        server.httpServer.removeListener('upgrade', upgrade);
        for (const request of pending) request.destroy();
      });
      server.middlewares.use((req, res, next) => {
        if (!allowedOrigin(req.headers, port, false, Boolean(https)) || !allowedPath(req.url)) {
          reply(res, 403, 'Development request refused\n');
          return;
        }
        if (!['GET', 'HEAD'].includes(req.method)) {
          reply(res, 405, 'Read-only development profile\n');
          return;
        }
        const path = req.url.split('?')[0];
        if (!publicPaths.has(path)) {
          // Vite's imported safeModulePaths can bypass fs.allow. Keep physical sources
          // inside the frontend root, including symlinks and aliases to protected files.
          let source;
          try { source = relative(root, realpathSync(join(root, decodeURIComponent(path).slice(1)))); }
          catch { reply(res, 403, 'Development request refused\n'); return; }
          if (isAbsolute(source) || source === '..' || source.startsWith(`..${sep}`)
            || !allowedPath('/' + source.split(sep).join('/'))) {
            reply(res, 403, 'Development request refused\n');
            return;
          }
        }
        // All other explicitly allowed source requests stay with Vite, never Rust.
        res.setHeader('X-Content-Type-Options', 'nosniff');
        res.setHeader('Cache-Control', 'no-store');
        if (req.url !== '/') { next(); return; }
        let bytes = 0;
        const chunks = [];
        let finished = false;
        const unavailable = () => {
          if (finished) return;
          finished = true;
          reply(res, 503, 'Yetlet backend unavailable. Build dist, start Rust on 127.0.0.1:3000, then retry.\n');
        };
        // Fixed HTTP loopback root only. No client headers/cookies/credentials forwarded.
        const backend = get({ hostname: '127.0.0.1', port: 3000, path: '/', agent: false,
          headers: { Accept: 'text/html' } }, (incoming) => {
          if (incoming.statusCode !== 200 || incoming.headers['content-type'] !== 'text/html; charset=utf-8'
            || incoming.headers['content-security-policy'] !== productionCsp) {
            unavailable(); backend.destroy(); return;
          }
          incoming.on('data', (chunk) => {
            bytes += chunk.length;
            if (bytes > maxHtmlBytes) { unavailable(); backend.destroy(); }
            else chunks.push(chunk);
          });
          incoming.on('error', unavailable);
          incoming.on('end', () => {
            if (finished) return;
            try {
              const html = new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks));
              const transformed = devDocument(html, port, Boolean(https));
              finished = true;
              reply(res, 200, req.method === 'HEAD' ? undefined : transformed.html, transformed.csp, true);
            } catch { unavailable(); }
          });
        });
        pending.add(backend);
        const timeout = setTimeout(() => { unavailable(); backend.destroy(); }, 3000);
        backend.on('error', unavailable);
        backend.once('close', () => { clearTimeout(timeout); pending.delete(backend); });
        res.once('close', () => backend.destroy());
      });
    },
  };
}
