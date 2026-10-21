# @NAME@ — explicit local Vite HMR profile (YT-100, experimental)

This section applies only because `new --assets vite --with-vite-dev` was
selected. The production instructions below still describe `npm run build`
and the unchanged Rust server. Node is additionally an explicitly started
development process in this profile; Yetlet never starts it automatically.

First run the frontend installation, typecheck, production build and native
lock/build steps below. **Build dist once before development**: Rust validates
and snapshots the production manifest before listening, even during HMR.
Rebuild dist and restart Rust when checking production changes. HMR alone does
not update that snapshot or reload Rust handlers.

Use two intentional foreground terminals, without a process supervisor:

1. Terminal A: set absolute `YETLET_ASSET_ROOT` to this app's `dist`, then run
   `yetlet dev --manifest-path /absolute/path/to/@NAME@/Cargo.toml` using your
   separately built local CLI. Rust must listen on **127.0.0.1:3000**. Plain apps
   already use that default; cloud apps use `YETLET_HOST=127.0.0.1` and
   `YETLET_PORT=3000`; config apps use `APP_SERVER__HOST=127.0.0.1` and
   `APP_SERVER__PORT=3000` plus their app-root settings/token instructions below.
   Keep all backend-only credentials in terminal A, never terminal B.
2. Terminal B: from the app root, run `npm run dev`, then visit
   **http://127.0.0.1:5173/**. Edit `app.css` or `shared.css` to see style HMR
   without a document reload. Set `YETLET_VITE_PORT` explicitly to another ASCII
   decimal port 1024–65535, except 3000, before starting if needed. The listener,
   WebSocket, origin checks and CSP use that exact port; an occupied port fails
   instead of selecting another. Do not override Vite's host/port via CLI flags.

PowerShell uses `$env:YETLET_ASSET_ROOT = (Resolve-Path ./dist).Path` and
`$env:APP_SERVER__PORT = '3000'` (or the applicable cloud variables). Use separate
terminal environments for the backend and frontend, also on Windows. Native
Windows HMR/lifecycle qualification remains a separate acceptance gate.

Ctrl-C in A ends its native foreground Cargo group/Windows Job; Ctrl-C in B
ends Vite and its watcher/WebSocket. Stop both explicitly. Stopping Rust leaves
Vite available with a bounded plain-text 503; restarting Rust restores the page.
Stopping/restarting Vite requires revisiting its origin when it is ready. Direct
http://127.0.0.1:3000/ continues to serve the validated production snapshot with
the exact production DEFAULT_CSP, meaningful SSR and styles with JavaScript off.
The Vite origin also preserves meaningful Rust SSR with JavaScript off, but dev
styles require Vite's JavaScript CSS injection. Only `/` is bridged, read-only;
other Rust routes, actions, cookies and authentication are deliberately excluded.

The dev bridge replaces only the generated root page's exact production asset
tags and local SVG URL. Unexpected, oversized, invalid or unavailable backend
HTML fails closed with 503 within three seconds. Each HTML response gets a fresh
cryptographic nonce in `meta[property=csp-nonce]`, used by Vite's injected CSS.
Dev CSP adds only that style nonce and the exact loopback WebSocket connection;
no unsafe-inline/eval, data, wildcard, external fonts or CDN. No backend headers,
cookies, token or credentials cross the bridge.

Vite binds only 127.0.0.1, uses strictPort, explicit same-origin CORS/HTTP and WS
checks, strict app-root filesystem access and sensitive-file denial, with no
dotenv discovery or public directory. Source requests are physically confined
to this frontend root; app settings, Rust sources, tooling configs, locks and
`/@fs/` access remain blocked except Vite's one internal env module. Keep this trusted application tree
free of secrets in frontend source, `VITE_*`, or imports; source imports are
public. New JS/TS, CSS and SVG files may use normal imports and subdirectories
without a filename registration step. Root HTML still uses the generated bridge
contract. `vite.dev.config.mjs` and `vite.dev.bridge.mjs` are dev-only; never
deploy them or expose this experimental dev server outside the local host.

The two-terminal arrangement is a temporary, explicit architecture choice.
There is no automatic startup, detached process, public-listener fallback, shared
asset crate or support for any other asset provider. Independent browser,
lifecycle, Windows and Reviewer evidence is tracked in YT-100 separately.
