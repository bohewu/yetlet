# browser-tasks — browser-only Rust/WASM tasks

Created by the existing CLI with the explicit settings override:

```sh
yetlet new browser-tasks --template browser --tailwind
```

This persists `cli.target = "browser"` in `base.toml`; plain `yetlet new` keeps
the native SSR default. Browser creation implies Vite/public CLI settings.
`--vite` and `--settings` are accepted but unnecessary here. `--service` is a
native-only profile. No dependency install, lock generation or process starts
during creation. Yetlet crates are unpublished previews: before local use,
replace their exact registry declarations with your trusted checkout's paths.

Prepare Rust/Cargo 1.99+, `wasm32-unknown-unknown`, `wasm-bindgen-cli` 0.2.129,
Node ^20.19 or >=22.12 and the selected dependencies separately. Put
`wasm-bindgen` on PATH, or explicitly set `WASM_BINDGEN` to its executable.

```sh
cd browser-tasks
cargo generate-lockfile --offline
npm ci --prefix frontend --ignore-scripts --no-audit --no-fund
yetlet build
npm run typecheck --prefix frontend
yetlet run
```

Open `http://127.0.0.1:5173/`. Ctrl-C stops the owned preview process tree.
`run` previews an existing build with Vite; it does not build, watch Rust or
start a native Yetlet server. Browser builds use optimized WASM in either
invocation of the existing `--release` option.

For the explicitly selected Rust development loop, use `yetlet watch` instead.
It requires both `auto_start=true` and `auto_build=true`, initially builds frozen
Rust/WASM with matching wasm-bindgen, and serves Vite development on the same
strict loopback port. App `src/`, `Cargo.toml`, `Cargo.lock` and `build.rs` edits
serialize/coalesce rebuilds. Successful complete generations cause full reload,
resetting tab-local Rust state; no graph/keyed identity restoration is promised.
Vite owns CSS HMR and TS/JS updates (the task entry reloads rather than remounting
an existing Rust runtime). Compilation failure keeps the last good JS/WASM pair;
before any success the server returns 503. Correct the source and retry after
successful publication. Ctrl-C closes the owned compiler/Vite tree and removes
private generations. External path dependencies are not automatically watched.
Development CSS uses a fresh document nonce; production CSP stays unchanged.
`watch` does not write production `generated/` or `dist/`; run `build` to package.

The existing Loader reads base/profile settings and declared `APP_CLI__*`
overrides. `cli.vite.root` can move `frontend/` within the app. `auto_build=false`
or `auto_start=false` explicitly skips that stage with exit 0 and no child.
Browser watch skips the entire loop when either switch is false.
Browser targets require Vite enabled and native Rust watch/TLS disabled. Server
settings and SecretProvider are not browser dependencies; the CLI removes
backend `APP_*` and secret/config selectors before starting frontend tooling.
Only public `VITE_*` values belong in frontend dotenv files. Never place a
credential in any browser configuration, build input or shipped asset.

The task list adds, renames, completes and filters items using Yetlet DOM,
keyed ownership and a local Rust graph. No action makes an API call. It keeps
up to 200 tasks only in this tab's memory; reload starts with the sample list.
JavaScript and WebAssembly are required. The initial error/loading/disabled
controls are static HTML, with no SSR/hydration or server-generated page shell.

## Customizable startup shell

Initial HTML links `wasm-startup.css` and `app.css` directly, so the brand,
loading text, indicator and recovery link appear before the JS entry or Wasm.
Customize the initial HTML panel/text and external CSS, including
`--wasm-startup-accent` / `--wasm-startup-track`; no helper runs to draw that shell.
The default panel covers this application's startup region. A full-page overlay
or different branding is an explicit CSS/markup choice; the helper imposes no
page layout, WebGPU, native server or Core dependency.

The optional `createStartupShell` asset receives explicit DOM elements and
text messages. Its ticket methods guard loading/ready/error/cancel/dispose UI;
the bootstrap still owns actual imports, mount and resource disposal. The entry
uses dynamic imports inside its error boundary, checks page lifetime after async
boundaries and calls ready only after Rust mount succeeds. Reload is the starter's
manual recovery policy and discards this tab's task state. Cached browser
Back/Forward navigation preserves the live page/runtime; a real page disposal
retires it. There is no automatic
retry or progress percentage. Static recovery/noJS text remains if the entry
itself cannot load. The decorative indicator respects reduced motion.

In repository consumers, `assets/wasm-startup.mjs` / `.css` are canonical and
these frontend files are identical distribution copies. CLI browser creation
copies the same canonical bytes; it does not fetch a package at application
startup. Browser watch rewrites only its matching dynamic generated imports;
existing generation/source/CSP/host/origin protections remain.

Deploy **only `frontend/dist/`** to an HTTP(S) static host. No Node, Rust binary,
settings files, Cargo sources, frontend dotenv or API server is needed at runtime.
The relative asset base supports root or a directory prefix ending in `/`.
Use a host that serves JS/CSS/WASM with their correct MIME types and preserves
the HTML CSP; strengthen headers for your deployment separately. There is one
page, no history router or deep-route fallback. `file://` loading is unsupported.
Vite preview is a local check, not production hosting. CSP permits only
same-origin scripts/styles/fetches and WASM compilation, with no inline script
or general JavaScript eval.

```sh
cargo test --offline --locked
cargo clippy --offline --locked --target wasm32-unknown-unknown -- -D warnings
```

## Repository reference qualification

This reference was created by `yetlet new` in a protected temporary parent,
then moved here and given explicit relative Cargo dependencies and a separately
prepared lock. Its [contract and evidence](../../docs/YT-104-BROWSER-STARTER.md)
record Linux Chrome desktop/mobile, a separate dist-only static host, native
Windows GNU CLI build/run/console-stop and independent Reviewer PASS. Done at
that selected scope; production hosting, Windows browser/MSVC, HMR and durable
task storage remain outside it. If moving `frontend/` to a different depth,
keep the CSS `@source` path aligned with the Rust sources as well as updating
`cli.vite.root`.

The later [YT-112 development-loop amendment](../../docs/YT-112-IMPLEMENTATION.md)
is In progress pending independent review and actual browser/lifecycle validation.
The original YT-104 qualification above does not attest to browser watch.
