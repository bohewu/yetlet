# M6 Completion — Handler APIs, Opt-in Modules and Demo Migration

**Date:** 2026-10-09
**Scope:** YT-030 through YT-034, locally verified prototype delivery. No commit/push or production-release approval is implied.

## Acceptance map

| Task | Implemented behavior | Evidence |
|---|---|---|
| YT-030 / M6a | Thin App over native Axum Router; ordinary handlers, groups/state/middleware and native Router escape hatch | Five existing `app_builder` tests and compiling rustdoc usage |
| YT-031 / M6b | `Page` response, `App::page(path, title, handler)`, native typed bindings/results/rejections | Five `handler_results` tests; SSR title/content escaping, strict CSP on success and generic 500, HEAD, typed State/Path/Query/Json, redirect and Result errors |
| YT-032 / M6c | `configure` and `try_configure` take ordinary setup functions; no runtime registry/DI | Three `configuration` tests: setup once, scoped typed extension, absence and typed configuration failure |
| YT-033 / M6d | Independent `yetlet-request-id` opt-in crate; no reverse core dependency | Seven `opt_in` tests: absent/scoped/global, untrusted ID replacement, 404/405, short circuit, invalid setup, exhaustion and 12 concurrent requests |
| YT-034 / M6e | Real Todo routes use App, Page, native State/Json and explicit RequestIds; no Controller or custom binder | Full existing API/pipeline tests, Chrome/Firefox E2E, dependency graphs and release before/after measurement |

**Observed:** 66 workspace unit/integration tests, 4 rustdoc tests, strict Clippy, format, native/minimal-example build and wasm32 compilation passed. The original M6a work is preserved; its earlier checkpoint is recorded in [M6a](M6A-VERIFICATION.md).

## Small response API

A no-argument async function returning shared `View` can use:

```rust
use yetlet_core::View;
use yetlet_server_axum::App;
async fn home() -> View { View::text("Hello") }
let app: axum::Router = App::new().page("/", "Home", home).into_router();
```

For parameters or application errors, return `Page` or `Result<Page, E>` through `.get`, retaining native Axum extraction and `E: IntoResponse`. `Page::new(title, view)` replaces repetitive manual rendering. `Page` does not change the shared/browser View type or add HTTP dependencies to it. Json, Redirect, status/header tuples and errors remain native Axum types, re-exported where convenient.

`App::page` deliberately supports only no-argument async View functions. Parameterized handlers use `.get` and `Page`; there is no duplicate generic model binder, blanket direct-View IntoResponse implementation, or response class hierarchy. Render errors retain a generic 500 plus strict CSP; explicit application error responses keep their chosen status/body.

## Explicit extension contract

See [Extensions](EXTENSIONS.md). Configuration is an ordinary `FnOnce(App<S>) -> App<S>` (or typed Result), invoked once at setup. The optional provider is selected by an application dependency and registration, never by a core feature default. Native Tower layers and State/Extensions remain the actual mechanisms.

The request-ID counter starts at 1 for a fresh provider, is shared across clones and is not globally unique. It replaces untrusted inbound IDs and fails with 503 on exhaustion. The core/minimal app has no correlation behavior unless installed. Session/Auth/Cache providers remain unimplemented and were not part of the bounded M6 module proof.

## Todo migration and preserved contracts

- Routing/group setup now uses `App`, with native `MethodRouter` for combined GET/POST and per-route limits/guards.
- The SSR root returns `Page`, reads only an optional typed `RequestId`, and preserves the original title, escaped markup, CSP and data attribute.
- API handlers use `State<TodoApiState>` and native `Json<TodoCommand>` extraction. The route explicitly installs the same 4 KiB body cap. Native rejections are mapped back to the existing typed error envelope (400/413), with header checks preserving 403/415; domain failures retain 404/422.
- The old request-ID middleware moved from demo-local code to the optional crate; API-scope middleware remains an ordinary configured setup function.
- The existing Search Island and browser source are untouched. Their lazy activation, local input/click behavior, typed HTTP calls and negative-state/422 scenarios passed in Chrome 155 and Firefox 157.

## Reproducible checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --locked -p yetlet-server-axum -p yetlet-request-id --doc
cargo build --locked -p yetlet-server-axum --example minimal
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/check_docs.py

# Matched debug server and browser assets, isolated headless regression:
python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/smoke_firefox.py --profile debug

# Release measurement using the pre-existing M5 tool:
cargo build --release --locked -p yetlet-todo-server
python3 scripts/build_browser.py --release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m6-after.json
```

No new benchmark runner or smoke-test framework was introduced. Browser tests use isolated temporary profiles and stop their owned processes. The generated measurement files are Git-ignored; this document retains the results and method.

## Dependency and artifact isolation

Normal Cargo graphs verify that `yetlet-server-axum` (including its minimal example) does not depend on `yetlet-request-id`, while the Todo server explicitly does. The Browser and Search Island normal wasm32 graphs contain neither the provider nor Axum, Tower, Hyper or the server adapter. The added lockfile package is the local provider, not a new third-party runtime family.

All release JS/WASM/CSS asset SHA-256 values match the pre-migration build. The manifest digest remains `3f3c5f074478c525`, also the M5 release digest. HTML response sizes are unchanged: `/` 532 bytes, `/app` 554 bytes, `/search` 1,431 bytes.

| Release artifact set | M5 / before | M6 after |
|---|---:|---:|
| Browser WASM, uncompressed | 216,006 bytes | 216,006 bytes |
| Search WASM, uncompressed | 179,716 bytes | 179,716 bytes |
| Browser JS + WASM, gzip estimate | 81,307 bytes | 81,307 bytes |
| Initial Search JS loader, gzip estimate | 551 bytes | 551 bytes |
| Activated Search JS + WASM, gzip estimate | 74,481 bytes | 74,481 bytes |

Gzip estimates are local artifact compression, not actual network bytes, and exclude CSS/favicon.

## Measured comparison, including the observed native increase

Same local tool and sample counts: 30 SSR request-to-first-byte measurements and 5 Chrome navigation/activation measurements; Linux x86_64, rustc 1.99.0, Python 3.14.4, Chrome 155.0.8059.39. Both same-session measurements use release native/WASM builds; browser cache disabled.

| Median | Historical M5 | Before migration this session | M6 after |
|---|---:|---:|---:|
| Local SSR first HTTP byte (TCP already connected) | 0.034 ms | 0.019 ms | 0.033 ms |
| Browser navigation to WASM-ready | 48.5 ms | 49.5 ms | 49.0 ms |
| Search interaction to island-ready | 3.7 ms | 3.3 ms | 3.8 ms |

The same-session SSR sample **increased by 0.014 ms**; Search activation increased by 0.5 ms while full Browser activation decreased by 0.5 ms. This is not a controlled isolation of module overhead: process/CPU scheduling and local browser state are not held fixed. Do not claim zero latency overhead, statistical equivalence, or an improvement from these small sequential samples. Against the separately recorded M5 snapshot, native SSR is close (0.033 versus 0.034 ms). No performance budget was previously specified, so the acceptance is measured disclosure and functional regression evidence, not a numerical speed guarantee.

Reports: `target/m6-before.json`, `target/m6-after.json`; the historical comparison is [M5](M5-VERIFICATION.md). The same-session pre-migration report was captured before changing the Todo or result/provider implementations.

## Limits not closed by M6

The framework API remains experimental. This milestone does not implement sessions, cookie authentication, caches, dynamic plugin discovery, Controller conventions, full hydration/resumability, tracing backends or GPU rendering. Origin policy, authentication/CSRF, persistent storage, MSRV verification, broad IME/accessibility coverage and independent production security assurance remain separate [release gates](RELEASE-READINESS.md). Existing tests are local; GitHub Actions has not been run remotely for this work.
