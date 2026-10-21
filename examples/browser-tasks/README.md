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
start a native Yetlet server. Use `build` again after changing sources. Browser
builds use optimized WASM in either invocation of the existing `--release`
option. `watch` is not selected for this first browser contract.

The existing Loader reads base/profile settings and declared `APP_CLI__*`
overrides. `cli.vite.root` can move `frontend/` within the app. `auto_build=false`
or `auto_start=false` explicitly skips that stage with exit 0 and no child.
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
