# @NAME@ — opt-in Vite production assets (YT-099, locally verified experimental scope)

This unpublished application owns its Rust adapter in `src/vite.rs`; it is not
a stable Yetlet asset API. Rust renders meaningful, escaped HTML with JavaScript
disabled. Node is build tooling only, never started by Yetlet or HTTP requests.
The following explicit steps supplement the native/config/cloud instructions
below. Do not run a standalone CSS compiler for this profile.

Use Rust 1.99+, Node compatible with Vite (tested 24.21.0), and npm. The two
template lockfiles pin Vite 8.3.4 and TypeScript 7.0.2. Only `--with-tailwind`
selects Tailwind 4.3.3 and its official `@tailwindcss/vite` 4.3.3 plugin; plain
Vite has neither dependency. Type checking is separate from transpilation.

From this generated app root, explicitly provision and build public frontend
dependencies (or add `--offline` to `npm ci` when the cache is ready):

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run typecheck
npm run build
cargo generate-lockfile --offline
cargo fmt --all -- --check
cargo check --offline --locked --all-targets
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo build --offline --locked
```

Scaffolding writes pinned frontend lock bytes; it never generates a Cargo lock,
installs, fetches or builds. If Rust dependencies are absent, provision them as
a separate operator choice. Repeating `npm ci`/typecheck/build uses the same
locked dependency tree. `companion.ts` is a second build entry demonstrating a
shared static JS/CSS/SVG dependency; only `app.ts` is selected for initial SSR.
Lazy JS/CSS loads on the button click, never in initial CSS links.

Before starting, select the **absolute path** to this app's built `dist`:

```sh
export YETLET_ASSET_ROOT="$PWD/dist"
# Set the config profile/token described below if --with-config was selected.
cargo run --offline --locked
```

PowerShell: `$env:YETLET_ASSET_ROOT = (Resolve-Path ./dist).Path`, followed by
the same Cargo run command. Missing/unbuilt/invalid roots fail before listening.
No `.env` discovery: Vite uses `envDir: false`, `publicDir: false`, no asset
inlining or source maps. Never put secrets in `VITE_*` or frontend source; those
values are public when intentionally referenced. Config SecretString data stays
server-side and is never passed to Vite or serialized to HTML.

The trusted, operator-owned dist tree must remain quiescent during startup.
The adapter bounds the manifest (128 KiB), nodes (128), edges (512), files (256),
individual assets (2 MiB) and total bytes (16 MiB). It rejects unsafe paths,
symlinks beneath the selected root, unknown schemas/types, missing imports and
cycles; it snapshots whitelisted CSS/JS/SVG once. This is not atomic isolation
against a hostile filesystem or an SVG sanitizer. Static dependency-first DFS,
manifest array order and first occurrence determine CSS cascade/dedup. Dynamic
CSS is excluded from initial HTML. Requests only read snapshot bytes under
`/static/`, with fixed MIME, nosniff, no-store and unchanged strict DEFAULT_CSP.
There is no inline/data/CDN allowance, runtime compilation, dev server or HMR.

No-store is conservative, not a rolling-deploy asset-retention guarantee. An old
page can request a removed hash after a restart; retain old builds and route
versions at deployment time under a separately reviewed policy. Existing Docker,
Compose and Kubernetes generators reject this app until YT-101. Cloud here means
only local bind/health/graceful shutdown composition. HMR belongs to YT-100.
