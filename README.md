# Yetlet

**Yetlet** is an experimental, HTML-first Rust universal UI runtime. A shared Rust `View`/component source supports native SSR and browser WASM/DOM without a default remote event circuit.

> **Status:** M0–M7 are implemented and locally verified at **experimental prototype** scope. M7 closes with renderer-neutral binding contracts, optional local graph/keyed-DOM ownership and one production Todo local-list adoption. A first final review found a queued/in-flight action disposal race; the lifetime repair, replacement full validation and repeat read-only reviewer now pass. No general VDOM, template macro, dependency-tracked signals/forms framework, resumability or WebGPU renderer is selected.

## Design principles

- **HTML first:** Pure SSR requires no client JS/WASM. Server and browser targets produce separate build outputs.
- **Local events first:** DOM events remain in the browser. Remote work uses explicit typed HTTP commands instead of transmitting raw events.
- **Independent rendering and transport:** No default SignalR/LiveView-like circuit, custom HTTP/TLS stack, or CSS engine.
- **Strict CSP:** Escaped SSR output and an allowlisted browser DOM subset. The `/app` CSP allows `'wasm-unsafe-eval'` where necessary, but not `'unsafe-eval'` or `'unsafe-inline'`.
- **Modular, opt-in capabilities:** `configure`/`try_configure` use ordinary setup functions, not a plugin manager. `yetlet-request-id` proves independent scoped/global installation; Session, Cookie Auth and Cache remain future providers. See [Extensions](docs/EXTENSIONS.md) and [ADR 0005](docs/adr/0005-opt-in-capability-modules.md).
- **Handler-first, Component-first, MVC optional (approved direction):** Ordinary Rust handlers and shared Views, typed HTTP results with minimal scaffolding. Full Controllers, model binders, DI and a mandatory MVC file layout are not required. See [ADR 0006](docs/adr/0006-handler-first-mvc-optional.md).
- **Bounded keyed updates, not a global renderer mandate:** Applications opt into explicit local graph and keyed ownership for proven surfaces; simple/independent subtrees may retain direct mount replacement. See [ADR 0007](docs/adr/0007-bounded-keyed-dom-and-local-graph.md).

## Implemented and planned functionality

| Mode / route | Status | Behavior |
|---|---|---|
| `GET /` | Implemented (M1) | Native SSR TodoList; zero browser runtime |
| `GET /app` | Implemented (M2/M7i) | Rust/WASM mount; keyed local Todo graph plus separate typed HTTP action |
| `GET /healthz` | Implemented | Health check |
| `GET /api/ping` | Implemented | Standalone JSON API endpoint |
| `GET /search` | Implemented (M3) | Complete SSR search page; Search WASM loads only after interaction and filters locally |
| `GET/POST /api/todos` | Implemented (M4) | Typed snapshot API and validated semantic Create/Toggle commands, with structured errors |
| Canvas/WebGPU | Deferred research | Independent scene renderer, not a DOM replacement |

Server and browser targets compile the same `TodoList` source from `examples/todo-ui`. On `/app`, the local toggle dispatches a typed command through an explicitly owned local graph and keyed DOM binding without a request; the separate server-create button uses `HttpExecutor` to POST a typed Todo command, then renders the server snapshot in its own host. `/search` continues to render HTML immediately and activate Search WASM lazily from escaped versioned state. **DOM events are never forwarded as remote events.**

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

`configure`/`try_configure` execute ordinary module setup functions once at application construction. Providers compose native scoped middleware and typed request/application state. The Todo server explicitly selects `yetlet-request-id`; the M8c cart explicitly selects `yetlet-session` only for `/cart`. A minimal app depends on neither. See [Extensions](docs/EXTENSIONS.md), [complete M6 evidence](docs/M6-VERIFICATION.md) and [M8c evidence](docs/M8C-VERIFICATION.md).

A runnable provider-free example is available with `cargo run -p yetlet-server-axum --example minimal`.

## Run the examples

Requirements: Rust stable with a declared MSRV of 1.88, Python 3.11+, and the WASM target. The current CI/security hardening round verifies the native workspace against Rust 1.88 after upgrading the transitive `time` dependency to its patched release; browser/WASM validation remains on current stable. Locally validated with Rust 1.99, wasm-bindgen 0.2.129, Chrome 155 and Firefox 157 (headless Firefox requires geckodriver).

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"

python3 scripts/build_browser.py
cargo run -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/` (SSR), `http://127.0.0.1:3000/app` (browser/WASM), and `http://127.0.0.1:3000/search` (SSR plus lazy Search Island). The Git-ignored `dist/` directory contains build-time generated assets and the content-hash manifest. The server never compiles WASM on request; `/app` explicitly returns HTTP 503 when browser assets have not been built.

User-facing runnable applications are listed in `examples/samples.toml` and [the sample guide](docs/SAMPLES.md). Validate the catalog with `python3 scripts/check_samples.py`; internal browser entry crates and research fixtures are deliberately not presented as standalone application samples.

A minimal native-only Hello World application has no optional providers or browser toolchain:

```bash
cargo run -p yetlet-hello-world
```

It listens on `http://127.0.0.1:3003` by default, with a script-free SSR `/` and a safely escaped `/hello/{name}` route. See [M9a evidence](docs/M9A-VERIFICATION.md); independent reviewer `wc_job_Htk-Quo_aFExMJs3` PASS.

The native-only REST sample requires no browser toolchain:

```bash
cargo run -p yetlet-rest-api
```

It listens on `http://127.0.0.1:3001` by default and exposes `/healthz`, `/api/books` and `/api/books/{id}`.

The native SSR Session cart also requires no browser toolchain:

```bash
cargo run -p yetlet-cart-server
```

It listens on `http://127.0.0.1:3002` by default and rejects non-loopback listeners while using its explicit insecure-localhost cookie override. `/healthz` stays outside Session scope; `/cart` uses strict-CSP forms, a server-side cart and a session-bound CSRF token.

The native Middleware Lab demonstrates request/context scope and streaming without new Yetlet infrastructure:

```bash
cargo run -p yetlet-middleware-lab
```

It listens on `http://127.0.0.1:3004` by default; inspect `/outside`, `/lab/plain`, `/lab/guarded`, `/lab/stream` and `/lab/broken-stream` for global/group/method-specific middleware, reverse unwind, 404/405 and streaming failure semantics. See [M9b evidence](docs/M9B-VERIFICATION.md); independent reviewer `wc_job_UnKflDaA8ZQHXCce` PASS.

The optional, **stateless** SSR Forms Validation sample demonstrates accessible field errors and no-side-effect PRG without JavaScript or Session:

```bash
cargo run -p yetlet-forms-validation
```

It listens on `http://127.0.0.1:3005` by default. Routes are `/form`, POST `/form/submit`, and `/form/success`. A valid POST redirects with HTTP 303, but **stores nothing**; this is not a claim of credentialed CSRF defense. See [M9c evidence](docs/M9C-VERIFICATION.md); final independent reviewer `wc_job_lGtk_ZIoAvK4F6PW` PASS.

The native File Server passed M9d independent acceptance review:

```bash
cargo run -p yetlet-file-server
```

It requires loopback (default port 3006), exposes a fixed static fixture and token-gated uploads/downloads, and stores only bounded temporary text files. See the [File Server README](examples/file-server/README.md) for the private local development token and curl workflow, and [M9d evidence](docs/M9D-VERIFICATION.md) for verification status. It is not a public file-hosting or identity service.

The SQL-first SQLite CRUD sample passed M9e independent repeat review:

```bash
cargo run -p yetlet-sqlite-crud
```

It defaults to loopback port 3007 and `target/yetlet-sqlite-crud.sqlite3`. It uses application-local direct `rusqlite`, explicit SQL migrations/transactions and bounded `spawn_blocking`; no Yetlet ORM or mandatory data layer is introduced. Reviewer `wc_job_Q3pBfUqPYGYNSIrD` found and prompted a fail-closed schema-column repair; repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. See the [comparison](docs/M9E-DATA-ACCESS-COMPARISON.md) and [M9e evidence](docs/M9E-VERIFICATION.md).

The native SSE Notifications sample passed M9f independent repeat review:

```bash
cargo run -p yetlet-sse-notifications
```

It listens on loopback port 3008 by default. `/events` uses bounded replay, heartbeat comments and `Last-Event-ID`; `/notifications` is a loopback command-oriented JSON publisher. No browser runtime, WebSocket or framework-global event circuit is introduced. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control-character validation occurred after trim; raw-input validation fixed it and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f evidence](docs/M9F-VERIFICATION.md).

The isolated Multi-Island Dashboard passed M9g final independent review:

```bash
python3 scripts/build_multi_island_dashboard.py && cargo run -p yetlet-multi-island-dashboard
```

It listens on loopback port 3009 and keeps generated Counter/Filter bundles under `target/multi-island-dashboard-dist/`, outside production `dist/`. Complete SSR remains when either bundle or the whole manifest is missing/invalid. Final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS after retained Chrome/Firefox fallback evidence was supplied. See [M9g evidence](docs/M9G-VERIFICATION.md).

The loopback-only Auth Notes fixture passed M9h security-focused review:

```bash
cargo run -p yetlet-auth-notes
```

It generates synthetic Alice/Bob login codes in the local terminal, uses opt-in Session rotation/CSRF, and rechecks note ownership on every request. Reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS. It is not a password, OIDC, MFA or production identity provider. See [M9h evidence](docs/M9H-VERIFICATION.md).

The larger Modular Shop reference application passed M9i repeat independent review:

```bash
cargo run -p yetlet-modular-shop
```

It listens on loopback port 3008, composes synthetic identity, SQL catalog, Session cart and transactional orders through explicit Rust modules, and can disable Orders with `YETLET_SHOP_DISABLE_ORDERS=1`. First reviewer `wc_job_a_WnhFSftzNoYYgf` found and prompted smoke readiness cleanup; repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. It is not a payment or production commerce system. See [M9i evidence](docs/M9I-VERIFICATION.md).

`python3 scripts/build_browser.py --release` produces an optimized browser bundle. Do not confuse debug WASM bytes with production transfer sizes or performance.

## Verify

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked \
  -p yetlet-todo-browser \
  -p yetlet-search-island \
  -p yetlet-reactive \
  -p yetlet-dom-keyed \
  -p yetlet-render-bench \
  -p yetlet-keyed-bindings \
  --target wasm32-unknown-unknown

python3 scripts/check_docs.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
cargo test --locked -p yetlet-session -p yetlet-cart-server --all-targets
cargo build --locked -p yetlet-cart-server
python3 scripts/smoke_cart.py
```

For the **release-mode** informational baseline and a separate Firefox E2E check:

```bash
cargo build --release --locked -p yetlet-todo-server
python3 scripts/build_browser.py --release
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m5-baseline-release.json
```

The isolated M7a render benchmark is built and measured separately; it does not modify production `dist/`:

```bash
python3 scripts/build_browser.py --release
python3 scripts/measure_render_strategies.py \
  --capture-production-before target/m7a-production-before.json
python3 scripts/build_render_bench.py --release
python3 scripts/measure_render_strategies.py --profile release \
  --rows 200 --updates 500 --rounds 10 --seed 7 \
  --production-before target/m7a-production-before.json \
  --out target/m7a-render-strategies.json
```

The combined M7b/M7c/M7d authoring, local-reactivity and lifecycle prototype is also isolated from production routes/assets. M7e reruns both this fixture and the production regressions before freezing the evidence:

```bash
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
```

Do not mix debug and release asset builds. Results are recorded in [M5 verification](docs/M5-VERIFICATION.md); size estimates refer to local gzip artifacts, not actual transfer bytes.

Browser smoke tests use isolated headless profiles and temporary localhost servers, stopping only their own processes. M2 checks shared DOM/local clicks; M3 covers lazy islands and fail-closed state; M4 covers the explicit typed HTTP boundary. M7i additionally checks stable keyed row/list identity, explicit graph/listener disposal, readable detached DOM and duplicate-free remount in Chrome and Firefox. See [M7i verification](docs/M7I-VERIFICATION.md).

## Workspace structure

- `yetlet-core` — Renderer-neutral `View` model plus validated component/instance/key/slot and inert-attribute binding contracts.
- `yetlet-reactive` — Optional strong-owner/weak-handle synchronous local message, batching and commit graph.
- `yetlet-html` — HTML serializer with escaping and a safe element/attribute policy.
- `yetlet-dom` — Direct browser DOM renderer with a restricted HTML subset.
- `yetlet-dom-keyed` — Optional keyed child reconciliation, entry-state ownership and disposable browser listeners.
- `yetlet-actions` — Typed commands and injectable LocalExecutor/HttpExecutor contracts.
- `yetlet-server-axum` — Thin App Builder, Page response, explicit setup composition, RequestContext and native Tower middleware.
- `yetlet-request-id` — Optional process-local numeric request correlation; the core does not depend on it.
- `yetlet-session` — Optional process-local server-side Session, expiry, CSRF and cookie lifecycle provider.
- `examples/todo-ui` — Shared TodoList, local transitions, and versioned search state.
- `examples/todo-browser` — Rust/WASM entrypoint with a keyed local graph and separate typed HTTP action path.
- `examples/search-island` — Lazy Search WASM, SSR DOM preservation, and local input handling.
- `examples/render-bench` — Isolated M7a strategy fixture; not a production route or public renderer API.
- `examples/keyed-bindings` — Isolated fixed-shape authoring and cross-browser lifecycle/form-state prototype; its reactive module re-exports optional `yetlet-reactive`.
- `examples/todo-server` — SSR/browser routes and in-memory typed GET/POST Todo API.
- `examples/rest-api` — Native-only typed Book CRUD with RequestContext policy and no browser/WASM dependency.
- `examples/cart-server` — Native-only SSR cart with explicitly scoped `yetlet-session` and no browser/WASM dependency.
- `scripts/build_browser.py` — Content-addressed browser asset pipeline with isolated Todo/Search Cargo builds.

**Known limitations:** Only the production Todo local list uses the promoted keyed graph; remote Todo results and Search result updates retain bounded subtree replacement. The fixed keyed/static-slot authoring shape, text-input/focus policy and conditional slot remain experimental. Nested/general component trees, async effects, IME/mobile/WebKit coverage and generic hydration/resumability remain absent. The Todo API and Session cart are process-local/in-memory; the Session proof is anonymous, single-node and not production auth/CSRF qualification. Durable persistence, stable API, full IME/a11y matrix and cross-framework benchmarks remain absent. See [M7i verification](docs/M7I-VERIFICATION.md), [M8c verification](docs/M8C-VERIFICATION.md) and [release readiness](docs/RELEASE-READINESS.md).

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
- [M7a render-strategy benchmark](docs/M7A-VERIFICATION.md)
- [M7b keyed/static-slot authoring prototype](docs/M7B-VERIFICATION.md)
- [M7c explicitly owned local-reactivity prototype](docs/M7C-VERIFICATION.md)
- [M7d keyed lifecycle and form-state verification](docs/M7D-VERIFICATION.md)
- [M7e renderer regression and no-decision evidence freeze](docs/M7E-VERIFICATION.md)
- [M7f renderer-neutral binding contracts](docs/M7F-VERIFICATION.md)
- [M7g optional synchronous local graph](docs/M7G-VERIFICATION.md)
- [M7h optional keyed DOM ownership](docs/M7H-VERIFICATION.md)
- [M7i production Todo keyed-graph adoption](docs/M7I-VERIFICATION.md)
- [M7j final architecture and evidence freeze](docs/M7J-VERIFICATION.md)
- [Sample application contract and catalog](docs/SAMPLES.md)
- [M8a sample contract verification](docs/M8A-VERIFICATION.md)
- [M8b API-only REST verification](docs/M8B-VERIFICATION.md)
- [M8c Session and SSR cart verification](docs/M8C-VERIFICATION.md)
- [M8d sample developer-experience audit](docs/M8D-VERIFICATION.md)
- [M8e final sample architecture and evidence freeze](docs/M8E-VERIFICATION.md)
- [M9 proposed sample expansion and API contract qualification plan](docs/M9-PLAN.md)
- [M9 planning reviewer gate](docs/M9-PLANNING-VERIFICATION.md)
- [M9e SQLite data-access comparison](docs/M9E-DATA-ACCESS-COMPARISON.md)
- [M9e SQL-first SQLite CRUD verification](docs/M9E-VERIFICATION.md)
- [M9f bounded SSE Notifications verification](docs/M9F-VERIFICATION.md)
- [M9g independent multi-island dashboard verification](docs/M9G-VERIFICATION.md)
- [M9h synthetic authentication and private-notes verification](docs/M9H-VERIFICATION.md)
- [M9i modular shop integration verification](docs/M9I-VERIFICATION.md)
- [Public API contract classification](docs/API-CONTRACTS.md)
- [M9j final compatibility and evidence freeze](docs/M9J-VERIFICATION.md)
- [Post-M9 CI and security review](docs/CI-SECURITY-REVIEW.md)
- [Module composition and optional providers](docs/EXTENSIONS.md)
- [Prototype release readiness and blockers](docs/RELEASE-READINESS.md)
- [ADR 0001](docs/adr/0001-separation-of-concerns.md)
- [ADR 0002](docs/adr/0002-events-and-actions.md)
- [ADR 0003](docs/adr/0003-renderers-and-loading.md)
- [ADR 0004 — RequestContext and middleware](docs/adr/0004-request-context-and-middleware.md)
- [ADR 0005 — Optional capability modules](docs/adr/0005-opt-in-capability-modules.md)
- [ADR 0006 — Handler-first and optional MVC](docs/adr/0006-handler-first-mvc-optional.md)
- [ADR 0007 — Bounded keyed DOM and local graph](docs/adr/0007-bounded-keyed-dom-and-local-graph.md)

M0–M9 are locally complete at prototype scope. M8 freezes the runnable sample contract; M9 adds twelve independently reviewed applications, a 170-export compatibility classification and complete native/browser/provider evidence freeze. The post-M9 CI/security round patches `RUSTSEC-2026-0009`, raises and locally verifies the Rust 1.88 MSRV, and hardens hosted CI; its new remote run must still be observed to terminal success after push. Repeat reviewers do not make Yetlet SemVer-stable or production-ready. SQL access remains application opt-in and Yetlet has no built-in ORM or mandatory DB layer. Public-release readiness (`YT-020`) is **Blocked** pending licensing, compatibility/support, document metadata, repository governance and production security policy. Current code is not a production release.
