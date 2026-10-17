# Vite integration feasibility spike

**Experimental, unpublished and application-owned.** This is a bounded proof of
Rust SSR consuming Vite production assets, not a new Yetlet core API, public
asset crate or supported CLI profile. See the [experiment evidence and future
work](../../docs/VITE-SPIKE-VERIFICATION.md). Existing framework/CLI sources,
root Cargo workspace and YT-091 standalone Tailwind scaffolding are unchanged.

## What is demonstrated

One Rust manifest loader serves two separately built profiles: `/plain` uses
ordinary CSS and TypeScript; `/tailwind` uses the official `@tailwindcss/vite`
plugin. Both use shared static imports, CSS dependencies, an external SVG and a
dynamically imported TypeScript module with its own CSS. Yetlet renders the HTML;
Vite does not replace Rust SSR and no Node process is needed to serve the built
application. JavaScript-disabled pages retain meaningful escaped SSR content.

The single experiment package contains both profiles' devDependencies; the plain
build does not activate the Tailwind plugin, but this combined fixture is not a
proof that Tailwind can be removed from that package. A genuinely dependency-free
plain Vite CLI template belongs to future YT-099.

## Run the production-build experiment

Use Rust 1.99 and Node compatible with Vite's declared engine (tested Node
24.21.0). From this experiment directory, provision public dependencies explicitly:

```sh
npm ci --ignore-scripts --no-audit --no-fund
cargo fetch --locked
npm run typecheck
npm run build
cargo test --offline --locked
cargo build --offline --locked
./target/debug/yetlet-vite-spike --dist "$PWD/dist" --port 3039
```

Visit `http://127.0.0.1:3039/plain` and `http://127.0.0.1:3039/tailwind`.
Stop with Ctrl-C. The port must be unused; the server binds only to loopback.
The explicit dist argument identifies trusted build output. Missing or invalid
manifests/assets fail startup with bounded messages. Build both profiles before
running the Rust test that loads actual emitted manifests.

Pinned development tools: Vite 8.3.4, Tailwind and `@tailwindcss/vite` 4.3.3,
TypeScript 7.0.2, and Playwright Core 1.64.1. The package lock records transitive
versions and integrity metadata. TypeScript checking is a separate command, not
an assumption about Vite transpilation. The Rust package has its own lockfile.
The later repeatability check used offline `npm ci` and reproduced all 18 output
files byte-for-byte; this is one-host evidence, not universal reproducibility.

## Security and asset boundary

Vite has `envDir: false`, `publicDir: false`, `assetsInlineLimit: 0`, no source
maps, and a disabled module-preload polyfill for the tested modern browser.
The Tailwind profile explicitly scans Rust sources. The server retains Yetlet's
unchanged `DEFAULT_CSP`: no inline script/style, external origin or `data:`
allowance is added. Only validated manifest asset paths are served; no request
selects an arbitrary disk file. JavaScript, CSS and SVG have explicit MIME and
`nosniff` headers. All responses use conservative `no-store` caching.

The loader bounds manifest, record, per-file and total sizes; rejects traversal,
unsupported types, missing references/assets and cycles; checks canonical paths
stay inside the selected build directory; and snapshots bytes before listening.
Build directories must be operator-owned and quiescent during startup. This is
not race-resistant hostile filesystem isolation or an arbitrary SVG sanitizer.
The fixture expects exactly one statically imported SVG. Its dependency-first
CSS traversal and cycle restrictions are experimental choices, not a finalized
universal asset contract. Future YT-098 must explicitly settle CSS cascade order
and compatibility with each producer rather than copying this reader blindly.

Only an explicitly named synthetic backend-secret sentinel was used in checks.
It was absent from generated assets and HTML. Do not put secrets into `VITE_*`
variables: disabling dotenv discovery does not change the client exposure rules
for intentionally referenced public-prefixed environment values.

## Focused browser verification

This test uses installed Linux `/usr/bin/google-chrome` through Playwright Core;
it downloads no browser. A built backend, both dist profiles and an unused port
are required. Choose a new output directory outside the repository:

```sh
node verify-browser.mjs --backend target/debug/yetlet-vite-spike \
  --dist dist --port 3039 --out /tmp/yetlet-vite-browser-check
```

It owns and closes its server/browser, checks actual styles and SVG, clicks the
lazy-module button twice, verifies lazy JS/CSS network responses, checks console
and CSP errors, repeats SSR with JavaScript disabled, and rejects unsupported
asset URLs. It records JSON and two screenshots. This is a focused product test,
not a generic test orchestration framework. Do not run against user services or
reuse their port.

## Deliberately not implemented

Vite development serving/HMR, Windows and other browser qualification, WASM
bundling, public generic asset APIs, formal `--assets vite` CLI support, deployment
packaging and rolling asset retention are **not** accepted by this experiment.
HMR needs explicit development-only origin/WebSocket and stylesheet nonce
handling; a same-origin proxy alone does not solve inline style injection.
Production CSP must not be weakened to make development tooling work.
Existing Docker/Compose/Kubernetes generators still reject modified application
contracts. YT-020 release remains Blocked.
