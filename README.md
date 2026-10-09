# Yetlet

**Yetlet** is an experimental, HTML-first Rust universal UI runtime. A shared Rust `View`/component source supports native SSR and browser WASM/DOM without a default remote event circuit.

> **Status:** M0–M6 are implemented and locally verified at **experimental prototype** scope. M6 includes handler/SSR results, explicit module composition, an independent optional Request ID provider and the migrated Todo example. Public release still needs licensing, support policy and production/security qualification; generic resumability and WebGPU are not implemented.

## Design principles

- **HTML first:** Pure SSR requires no client JS/WASM. Server and browser targets produce separate build outputs.
- **Local events first:** DOM events remain in the browser. Remote work uses explicit typed HTTP commands instead of transmitting raw events.
- **Independent rendering and transport:** No default SignalR/LiveView-like circuit, custom HTTP/TLS stack, or CSS engine.
- **Strict CSP:** Escaped SSR output and an allowlisted browser DOM subset. The `/app` CSP allows `'wasm-unsafe-eval'` where necessary, but not `'unsafe-eval'` or `'unsafe-inline'`.
- **Modular, opt-in capabilities:** `configure`/`try_configure` use ordinary setup functions, not a plugin manager. `yetlet-request-id` proves independent scoped/global installation; Session, Cookie Auth and Cache remain future providers. See [Extensions](docs/EXTENSIONS.md) and [ADR 0005](docs/adr/0005-opt-in-capability-modules.md).
- **Handler-first, Component-first, MVC optional (approved direction):** Ordinary Rust handlers and shared Views, typed HTTP results with minimal scaffolding. Full Controllers, model binders, DI and a mandatory MVC file layout are not required. See [ADR 0006](docs/adr/0006-handler-first-mvc-optional.md).

## Implemented and planned functionality

| Mode / route | Status | Behavior |
|---|---|---|
| `GET /` | Implemented (M1) | Native SSR TodoList; zero browser runtime |
| `GET /app` | Implemented (M2) | External JS loader and Rust/WASM DOM mount with a local toggle |
| `GET /healthz` | Implemented | Health check |
| `GET /api/ping` | Implemented | Standalone JSON API endpoint |
| `GET /search` | Implemented (M3) | Complete SSR search page; Search WASM loads only after interaction and filters locally |
| `GET/POST /api/todos` | Implemented (M4) | Typed snapshot API and validated semantic Create/Toggle commands, with structured errors |
| Canvas/WebGPU | Deferred research | Independent scene renderer, not a DOM replacement |

Server and browser targets compile the same `TodoList` source from `examples/todo-ui`. On `/app`, the local toggle uses an injected `LocalExecutor` without a request; the separate server-create button uses an injected `HttpExecutor` to POST a typed Todo command, then renders the server's snapshot. `/search` continues to render HTML immediately and activate Search WASM lazily from escaped versioned state. **DOM events are never forwarded as remote events.**

## HTTP middleware and request context

`yetlet-server-axum` now provides an owning `RequestContext<S>` with typed request-local extensions and separate application state. Middleware uses native Axum `Next`/`from_fn_with_state`, nested Router scopes and Tower layers. `with_global` adds an explicit pre-routing boundary around a completed Router. API and SSR routes share this pipeline; browser/WASM code does not depend on it.

See [HTTP pipeline usage and limits](docs/HTTP-PIPELINE.md) and [ADR 0004](docs/adr/0004-request-context-and-middleware.md). The existing anonymous API remains a prototype; request context is not authentication, a DI container or a session subsystem.

## M6 — Minimal handlers, results and opt-in composition

`App<S>` remains a thin native Axum Router facade. A complete minimal SSR/JSON application needs no Controller, mandatory RequestContext, DI container or optional module:

```rust
use yetlet_core::View;
use yetlet_server_axum::{App, response::Json};

async fn home() -> View { View::text("Hello") }
async fn status() -> Json<&'static str> { Json("ready") }

let app: axum::Router = App::new()
    .page("/", "Home", home)
    .group("/api", |api| api.get("/status", status))
    .into_router();
```

For typed parameters or application errors, use `.get` with native `Path`, `Query`, `State`, `Json` extractors and return `Page::new(title, view)` or `Result<Page, E>`. `App::page` is specifically a no-argument async View-handler convenience; no duplicate model binder is introduced. `Page` retains escaped SSR, strict CSP and generic 500 rendering errors. Json, Redirect and status/header tuples retain Axum semantics.

`configure`/`try_configure` execute ordinary module setup functions once at application construction. Providers compose native scoped middleware and typed request/application state. The Todo server explicitly selects `RequestIds::new().global(app)` from the separate `yetlet-request-id` crate; a minimal app does not depend on or initialize it. See [Extensions](docs/EXTENSIONS.md) and [complete M6 evidence](docs/M6-VERIFICATION.md).

A runnable provider-free example is available with `cargo run -p yetlet-server-axum --example minimal`.

## Run the examples

Requirements: Rust stable (declared MSRV 1.85, **not separately verified**), Python 3.11+, and the WASM target. Locally validated with Rust 1.99, wasm-bindgen 0.2.129, Chrome 155 and Firefox 157 (headless Firefox requires geckodriver).

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"

python3 scripts/build_browser.py
cargo run -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/` (SSR), `http://127.0.0.1:3000/app` (browser/WASM), and `http://127.0.0.1:3000/search` (SSR plus lazy Search Island). The Git-ignored `dist/` directory contains build-time generated assets and the content-hash manifest. The server never compiles WASM on request; `/app` explicitly returns HTTP 503 when browser assets have not been built.

`python3 scripts/build_browser.py --release` produces an optimized browser bundle. Do not confuse debug WASM bytes with production transfer sizes or performance.

## Verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown

python3 scripts/check_docs.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
```

For the **release-mode** informational baseline and a separate Firefox E2E check:

```bash
cargo build --release --locked -p yetlet-todo-server
python3 scripts/build_browser.py --release
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m5-baseline-release.json
```

Do not mix debug and release asset builds. Results are recorded in [M5 verification](docs/M5-VERIFICATION.md); size estimates refer to local gzip artifacts, not actual transfer bytes.

Browser smoke tests use an isolated headless Chrome profile and temporary localhost server, stopping only their own processes. M2 checks cover shared DOM and local clicks; M3 covers lazy islands, SSR preservation, local filtering, and version-incompatible fail-closed behavior. M4 checks cover an explicit typed HTTP POST, the returned server snapshot, typed 422 failure visibility, and no network traffic from local clicks. See [M2](docs/M2-VERIFICATION.md), [M3](docs/M3-VERIFICATION.md), and [M4 verification](docs/M4-VERIFICATION.md).

## Workspace structure

- `yetlet-core` — Renderer-neutral `View` model.
- `yetlet-html` — HTML serializer with escaping and a safe element/attribute policy.
- `yetlet-dom` — Direct browser DOM renderer with a restricted HTML subset.
- `yetlet-actions` — Typed commands and injectable LocalExecutor/HttpExecutor contracts.
- `yetlet-server-axum` — Thin App Builder, Page response, explicit setup composition, RequestContext and native Tower middleware.
- `yetlet-request-id` — Optional process-local numeric request correlation; the core does not depend on it.
- `examples/todo-ui` — Shared TodoList, local transitions, and versioned search state.
- `examples/todo-browser` — Rust/WASM entrypoint with local event binding.
- `examples/search-island` — Lazy Search WASM, SSR DOM preservation, and local input handling.
- `examples/todo-server` — SSR/browser routes and in-memory typed GET/POST Todo API.
- `scripts/build_browser.py` — Content-addressed browser asset pipeline.

**Known limitations:** Browser/island updates use owned-subtree replacement; SearchStateV1 is **not** generic hydration/resumability. The Todo API is an anonymous in-memory demo, not a user-data service. HTTP origin checks need a trusted scheme/host behind TLS reverse proxies, and there is no production auth/CSRF, persistence, stable API, full IME/a11y matrix, or comparative benchmark. Only Chrome 155 and Firefox 157 headless browser scenarios were tested; see [release readiness](docs/RELEASE-READINESS.md).

## Documentation

- [Architecture and module boundaries](docs/ARCHITECTURE.md)
- [Full MVP specification](docs/MVP.md)
- [Roadmap](docs/ROADMAP.md)
- [Backlog](docs/BACKLOG.md)
- [M2 verification record](docs/M2-VERIFICATION.md)
- [M3 verification record](docs/M3-VERIFICATION.md)
- [M4 typed actions/API verification](docs/M4-VERIFICATION.md)
- [M5 acceptance and measured baseline](docs/M5-VERIFICATION.md)
- [M6a App Builder verification](docs/M6A-VERIFICATION.md)
- [Complete M6 verification and performance comparison](docs/M6-VERIFICATION.md)
- [Module composition and optional providers](docs/EXTENSIONS.md)
- [Prototype release readiness and blockers](docs/RELEASE-READINESS.md)
- [ADR 0001](docs/adr/0001-separation-of-concerns.md)
- [ADR 0002](docs/adr/0002-events-and-actions.md)
- [ADR 0003](docs/adr/0003-renderers-and-loading.md)
- [ADR 0004 — RequestContext and middleware](docs/adr/0004-request-context-and-middleware.md)
- [ADR 0005 — Optional capability modules](docs/adr/0005-opt-in-capability-modules.md)
- [ADR 0006 — Handler-first and optional MVC](docs/adr/0006-handler-first-mvc-optional.md)

The M5 acceptance/performance baseline (`YT-019`) is locally verified and committed on `master`. Public-release readiness (`YT-020`) is **Blocked** pending licensing and support/security policy. All M6 tasks (`YT-030`–`YT-034`) are locally complete, including Todo migration and before/after measurement. MVC/Controllers remain unnecessary; Session/Auth/Cache implementations were not part of the M6 capability proof and remain unimplemented. Current code is not a production release.
