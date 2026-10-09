# M6a Verification — Minimal Handler-First Application Facade

**Milestone:** M6a / YT-030
**Scope:** Native Axum/Tower application composition with ordinary Rust handlers.
**Status:** Historical M6a checkpoint. M6b–M6e were pending at this checkpoint and are now completed in [M6 verification](M6-VERIFICATION.md). The scope, counts and limitations below describe M6a, not the current full M6 state; public-release qualification remains separate.

## Implemented surface

`yetlet-server-axum::App<S>` contains **only** an underlying `axum::Router<S>`. It does not own a second route table, a runtime controller registry, dependency-injection container or renderer.

| API | Delegation | Why it exists |
|---|---|---|
| `App::new()` | `Router::new()` | Start a minimal handler-first application |
| `get(path, handler)` / `post(path, handler)` | `routing::get/post` | Register ordinary Rust handler functions |
| `route(path, MethodRouter)` | `Router::route` | Keep native method combinations and route middleware |
| `group(prefix, \|group\| ...)` | `Router::nest` | Express scoped route organization; supports nested groups |
| `route_layer(layer)` | `Router::route_layer` | Attach Tower middleware to routes already registered |
| `with_state(state)` | `Router::with_state` | Keep Axum typed state and extraction semantics |
| `from_router` / `into_router` | Existing `Router<S>` | No migration requirement or custom server runner |
| `map_router` | Transformation of `Router<S>` | Use native fallbacks, services, nesting and other routing APIs |

Direct Axum `Path/Query/Json/State` extractors, `IntoResponse`, CSP/SSR helpers, `RequestContext<S>` and the previously implemented `middleware::with_global` remain compatible. The facade is **native-server-only** and does not add any runtime imports to browser WASM.

## Compiled API example

```rust
use axum::{Json, Router, response::Response};
use yetlet_core::View;
use yetlet_server_axum::{App, HtmlPageError, render_page};

async fn home() -> Result<Response, HtmlPageError> {
    render_page("Home", &View::text("Hello"))
}
async fn ping() -> Json<&'static str> {
    Json("ready")
}

let router: Router = App::new()
    .get("/", home)
    .group("/api", |api| api.get("/ping", ping))
    .into_router();
```

This exact pattern is exercised by the Rust documentation test in `src/app.rs`. No Controller class, request context parameter, result hierarchy or special macros are required. The example deliberately uses the existing `render_page` to preserve the HTML/CSP response contract.

## Native integration coverage

`crates/yetlet-server-axum/tests/app_builder.rs` exercises five real-router scenarios:

1. Native SSR and JSON responses from ordinary async handlers, with strict CSP and correct default 404.
2. Nested route groups, typed path/query extraction, POST response and native 404/405/Allow semantics.
3. Scoped middleware, early short-circuit, outer global middleware and 404 response wrapping.
4. Native `State<S>` extraction and direct Router customization for fallback/redirect.
5. Reuse of an existing `axum::Router` and a native combined `MethodRouter`.

Other middleware/RequestContext integration suites remain in place. Run:

```bash
cargo test --locked -p yetlet-server-axum --all-targets
cargo test --locked -p yetlet-server-axum --doc
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/check_docs.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
```

**Observed local result (2026-10-09):** Rust format check, strict Clippy, **51/51 workspace Rust tests** (including 5 new App Builder tests), the compiled rustdoc usage example, native/wasm32 builds, Chrome headless E2E and Firefox 157 headless E2E all passed. Documentation checks passed across 19 English Markdown documents and 34 backlog items. The browser asset manifest remained at digest `dbd3b1631850d93a`, unchanged from M5's debug baseline. These are local development checks, not remote CI or independent security assurance.

## Limitations and future work

- This is **M6a only**. `App::page` or a handler that returns `View` directly is not implemented. Rendering still uses `render_page` explicitly. M6b (YT-031) evaluates minimal SSR/typed result conveniences without implementing an MVC result hierarchy.
- `get` and `post` are convenience methods; `route` accepts native `MethodRouter` for PUT, DELETE, PATCH, etc., and `map_router` exposes other native Axum operations. No duplicate HTTP method-routing layer.
- `group` uses Axum nesting; root-prefix restrictions and route matching are Axum's. Applying `route_layer` only affects routes registered before it; unmatched requests require explicit outer `with_global` middleware.
- Session, Cookie Authentication, Cache, Output Cache, tracing and a plugin registry are **not** implemented. They remain opt-in future capabilities under [ADR 0005](adr/0005-opt-in-capability-modules.md).
- The existing Todo server intentionally remains on raw Axum Router until the planned YT-034 demo migration. No native latency/size improvement is claimed, and no production/security release assurance is inferred.

See [ADR 0006](adr/0006-handler-first-mvc-optional.md) and [Roadmap](ROADMAP.md).
