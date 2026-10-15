# Multi-Island Dashboard — Independent SSR/WASM Lifecycles

This sample renders one complete dashboard on the server and optionally activates two independently built Rust/WASM islands. The Counter and Filter have separate packages, assets, state/version checks, listeners and disposal lifetimes. There is no general hydration engine, resumability protocol, framework-global event circuit or shared island state.

## Run

From the repository root:

```bash
cargo xtask build multi-island-dashboard && cargo run -p yetlet-multi-island-dashboard
```

The anonymous prototype listens on `127.0.0.1:3009` by default and rejects non-loopback listeners. Override local test paths with `YETLET_DASHBOARD_LISTEN` and `YETLET_DASHBOARD_ASSET_ROOT`.

Routes:

- `/healthz` — native health endpoint.
- `/dashboard` — complete SSR dashboard containing both island fallbacks.

Generated assets live only under ignored `target/multi-island-dashboard-dist/`, not production `dist/`. The server validates one content-addressed manifest, every exact same-origin URL and each referenced loader/module/WASM file. If the manifest is missing or malformed, the page stays script-free with strict CSP. If only one island bundle is missing or invalid, only that loader is omitted; both SSR panels remain readable.

## Island contracts

- **Counter:** local increment plus a delayed increment. Its runtime token makes callbacks queued before disposal unable to mutate retired DOM. Dispose removes only Counter listeners; remount keeps its owned DOM identity and current count.
- **Filter:** preserves the SSR search input node and focus while replacing only its own result rows and count. Dispose removes only the input listener; remount recomputes from the live input value.
- Both roots require their own `data-yetlet-island`, `data-island-version="1"` and versioned JSON state. A failed mount marks only that root and leaves its SSR markup and sibling untouched.
- Each small external loader lazy-imports only its own module after a local `focusin` or `yetlet-activate` event. Activating Counter does not fetch Filter WASM and vice versa.

The global `__yetletDashboard` object exists only as an explicit demonstration/test handle for `activate`, `mount` and `dispose`; it is not a framework API or cross-island state bus.

## Verify

```bash
cargo test --locked -p yetlet-multi-island-dashboard --all-targets
cargo check --locked --target wasm32-unknown-unknown \
  -p yetlet-dashboard-counter-island \
  -p yetlet-dashboard-filter-island
cargo xtask build multi-island-dashboard --release
cargo build --release --locked -p yetlet-multi-island-dashboard
cargo xtask smoke multi-island-dashboard-chrome --profile release
cargo xtask smoke multi-island-dashboard-firefox --profile release
cargo xtask check samples
```

See [M9g verification](../../docs/M9G-VERIFICATION.md) for retained Chrome/Firefox, manifest-fallback, lifecycle, bundle and dependency-isolation evidence. This remains a local prototype, not a generalized island/hydration contract or browser support guarantee.
