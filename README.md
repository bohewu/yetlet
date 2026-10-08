# Yetlet

**Yetlet** is an experimental, HTML-first Rust universal UI runtime. A shared Rust `View`/component source supports native SSR and browser WASM/DOM without a default remote event circuit.

> **Status:** M0–M4 prototype milestones have passed local validation; the complete MVP is still pending M5. The Search Island uses versioned state v1, and the browser/server now support explicit typed LocalExecutor/HttpExecutor Todo actions. Generic hydration/resumability and WebGPU remain unimplemented. APIs are experimental.

## Design principles

- **HTML first:** Pure SSR requires no client JS/WASM. Server and browser targets produce separate build outputs.
- **Local events first:** DOM events remain in the browser. Remote work will use explicit typed HTTP commands instead of transmitting raw events.
- **Independent rendering and transport:** No default SignalR/LiveView-like circuit, custom HTTP/TLS stack, or CSS engine.
- **Strict CSP:** Escaped SSR output and an allowlisted browser DOM subset. The `/app` CSP allows `'wasm-unsafe-eval'` where necessary, but not `'unsafe-eval'` or `'unsafe-inline'`.

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

## Run the examples

Requirements: Rust stable (declared MSRV 1.85, **not separately verified**), Python 3.11+, and the WASM target. Locally validated with Rust 1.99, wasm-bindgen 0.2.129, and Chrome 155.

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

Browser smoke tests use an isolated headless Chrome profile and temporary localhost server, stopping only their own processes. M2 checks cover shared DOM and local clicks; M3 covers lazy islands, SSR preservation, local filtering, and version-incompatible fail-closed behavior. M4 checks cover an explicit typed HTTP POST, the returned server snapshot, typed 422 failure visibility, and no network traffic from local clicks. See [M2](docs/M2-VERIFICATION.md), [M3](docs/M3-VERIFICATION.md), and [M4 verification](docs/M4-VERIFICATION.md).

## Workspace structure

- `yetlet-core` — Renderer-neutral `View` model.
- `yetlet-html` — HTML serializer with escaping and a safe element/attribute policy.
- `yetlet-dom` — Direct browser DOM renderer with a restricted HTML subset.
- `yetlet-actions` — Typed commands and injectable LocalExecutor/HttpExecutor contracts.
- `yetlet-server-axum` — RequestContext, native Tower middleware boundary, HTTP response/CSP adapter.
- `examples/todo-ui` — Shared TodoList, local transitions, and versioned search state.
- `examples/todo-browser` — Rust/WASM entrypoint with local event binding.
- `examples/search-island` — Lazy Search WASM, SSR DOM preservation, and local input handling.
- `examples/todo-server` — SSR/browser routes and in-memory typed GET/POST Todo API.
- `scripts/build_browser.py` — Content-addressed browser asset pipeline.

**Known limitations:** Browser and island updates use owned-subtree replacement. M3's bounded Search Island v1 schema is **not** generic hydration/resumability. M4's anonymous in-memory Todo API is a demo, **not** an authenticated or production-ready service. HTTPS reverse proxies need trusted origin/scheme handling; complete CSRF/authentication, idempotency, fine-grained reactivity, browser compatibility, and comparative benchmarks are outside the current proof.

## Documentation

- [Architecture and module boundaries](docs/ARCHITECTURE.md)
- [Full MVP specification](docs/MVP.md)
- [Roadmap](docs/ROADMAP.md)
- [Backlog](docs/BACKLOG.md)
- [M2 verification record](docs/M2-VERIFICATION.md)
- [M3 verification record](docs/M3-VERIFICATION.md)
- [M4 typed actions/API verification](docs/M4-VERIFICATION.md)
- [ADR 0001](docs/adr/0001-separation-of-concerns.md)
- [ADR 0002](docs/adr/0002-events-and-actions.md)
- [ADR 0003](docs/adr/0003-renderers-and-loading.md)

The next milestone is M5 (`YT-019`–`YT-020`): a full acceptance/performance baseline and release-ready documentation. M3/M4 changes remain uncommitted and unpushed. Licensing remains a maintainer decision.
