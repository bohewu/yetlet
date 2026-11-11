# Sample Applications

**Status:** Thirteen catalog applications are locally verified at bounded prototype scope. The original twelve-application M8/M9 compatibility/evidence freeze remains historical; YT-241 Style Showcase was independently reviewed and then promoted separately.

Yetlet samples exist to validate application ergonomics, dependency isolation and security boundaries. They are not a parallel tutorial framework and do not authorize new core abstractions merely to make a demo look shorter.

## Choose a starting point

Begin with [Hello World](../examples/hello-world/README.md) for native SSR or [REST API](../examples/rest-api/README.md) for HTTP-only development. Use [Todo](../examples/todo-server/README.md) for shared SSR/WASM Views and [Session cart](../examples/cart-server/README.md) for an explicit server capability. Use [Style Showcase](../examples/style-showcase/README.md) to compare native previews with real Rust View, Style and generated CSS sources. Each sample-local README provides exact commands and limitations. The [documentation index](README.md) separates current guidance from historical qualification.

## Startup paths in this checkout

The source API follows **Simple first, customizable when needed**. The
[API reference](API-REFERENCE.md#source-only-checkout-additions) shows a minimal
`App::new().page(...).serve()` and an explicit
`App::from_router(router).serve_on(address)` example. These methods and the
optional Prelude are source-only additions after SDK `0.9.0-alpha.7`.

The non-catalog SDK minimal example uses the fixed `127.0.0.1:7878` HTTP default.
Run it from the repository root with the following command.

```bash
cargo run --locked -p yetlet --features server-axum --example server-axum-minimal
```

The thirteen catalog applications preserve their own environment/port contracts.

| Sample | Environment variable | Default HTTP address | Startup |
|---|---|---|---|
| Hello World | `YETLET_HELLO_LISTEN` | `127.0.0.1:3003` | `from_router(...).serve_on(address)` |
| REST API | `YETLET_REST_LISTEN` | `127.0.0.1:3001` | `from_router(...).serve_on(address)` |
| Session Cart | `YETLET_CART_LISTEN` | `127.0.0.1:3002` | `from_router(...).serve_on(address)` |
| Middleware Lab | `YETLET_MW_LISTEN` | `127.0.0.1:3004` | `from_router(...).serve_on(address)` |
| Forms Validation | `YETLET_FORMS_LISTEN` | `127.0.0.1:3005` | `from_router(...).serve_on(address)` |
| File Server | `YETLET_FILES_LISTEN` | `127.0.0.1:3006` | Native listener and graceful shutdown |
| Auth Notes | `YETLET_AUTH_LISTEN` | `127.0.0.1:3010` | Native listener and graceful shutdown |
| Modular Shop | `YETLET_SHOP_LISTEN` | `127.0.0.1:3008` | Native listener and graceful shutdown |
| SSE Notifications | `YETLET_SSE_LISTEN` | `127.0.0.1:3008` | Native listener and graceful shutdown |
| SQLite CRUD | `YETLET_SQLITE_LISTEN` | `127.0.0.1:3007` | Native listener and graceful shutdown |
| Todo | `YETLET_LISTEN` | `127.0.0.1:3000` | `from_router(...).serve_on(address)` |
| Multi-Island Dashboard | `YETLET_DASHBOARD_LISTEN` | `127.0.0.1:3009` | Native listener and graceful shutdown |
| Style Showcase | `YETLET_SHOWCASE_LISTEN` | `127.0.0.1:3011` | `from_router(...).serve_on(address)` |

The five basic HTTP samples, Todo and Style Showcase retain a native listener
branch for port `0`, allowing the banner to print `local_addr()` after binding.
The five lifecycle-sensitive capability samples and Dashboard keep native
`TcpListener`, `axum::serve` and `with_graceful_shutdown` visible. `into_router()`
and direct Axum/Tower composition remain first-class. Prelude imports are optional.
Modular Shop and SSE share port `3008`, so concurrent runs require explicit
separate addresses. Each local README retains its exact root-level catalog command.

Todo and Dashboard use explicit browser build commands for their WASM journeys.
Todo also supports native-only SSR with `cargo xtask build todo-css` before
`cargo run --locked -p yetlet-todo-server`. Style Showcase's catalog command is
native-only, with `browser_runtime = false`; its optional companion is built
separately with `cargo xtask build showcase-browser`.

`serve()` and `serve_on()` use plain HTTP with caller-owned Tokio runtime and
explicit TCP addresses. HTTPS/TLS requires an explicit secure edge or native TLS
integration. The methods never infer TLS, create certificates or start a hidden
runtime. The minimal default uses a fixed port and returns bind errors.

## Catalog contract

`examples/samples.toml` is the machine-readable catalog for user-facing runnable applications. `cargo xtask check samples` validates the catalog with the isolated Rust tool. The browser build field names an exact supported Rust command; package identity, repository source, companions and README commands remain checked.

Each entry declares:

- a stable sample ID, title, workspace package and directory;
- one copyable root-level run command;
- routes plus sample-local source markers that establish each route;
- whether a browser runtime is required, its companion packages and explicit build script;
- optional Yetlet crates deliberately selected by the primary or declared companion packages;
- retained evidence supporting its current status.

Every catalogued directory also owns a local `README.md`. Internal packages such as browser entry crates, shared component crates and research fixtures remain ordinary workspace members but are not required to appear in the user-facing catalog.

Catalog paths, route sources, build scripts and evidence must resolve inside the repository. Commands must exactly match a README line and launch the declared primary package. Route declarations carry source markers, and optional dependencies must appear directly in the primary or declared companion manifests.

## Structure rules

1. Keep one runnable application package per catalog entry.
2. Use ordinary Rust handlers, Axum extractors and narrow Yetlet values; do not add mandatory Controllers, DI or source-layout conventions.
3. Keep browser builds explicit. An API-only sample must not require `wasm32`, wasm-bindgen, generated browser assets or a browser dependency.
4. Declare optional capabilities in the primary package or an explicit companion package. The Todo server owns request correlation, while its browser companion owns keyed/reactive dependencies. Applications that do not use Session, request correlation or browser updates must not select those crates.
5. Include task-specific native tests and a bounded localhost smoke test when network behavior is part of the sample.
6. Record limitations and distinguish local prototype verification from production assurance.

## Current catalog

| ID | Package | Kind | Browser runtime | Purpose |
|---|---|---|---:|---|
| `hello-world` | `yetlet-hello-world` | Server rendered | No | Minimal native SSR, HTML escaping, strict CSP and zero optional providers |
| `todo` | `yetlet-todo-server` | Full stack | Yes | SSR, Browser/WASM, Search Island, typed HTTP actions and keyed local updates |
| `rest-api` | `yetlet-rest-api` | API | No | Typed Book CRUD, RequestContext policy, structured errors and request correlation |
| `session-cart` | `yetlet-cart-server` | Server rendered | No | Opt-in server-side Session, native forms, CSRF boundary and refresh persistence |
| `auth-notes` | `yetlet-auth-notes` | Server rendered | No | Synthetic authentication, Session rotation, CSRF and per-user note authorization |
| `modular-shop` | `yetlet-modular-shop` | Server rendered | No | Explicit modules, Session cart and transactional/idempotent direct-SQL orders |
| `middleware-lab` | `yetlet-middleware-lab` | API | No | Native Tower middleware scopes, request extensions, short-circuit, streaming |
| `file-server` | `yetlet-file-server` | Server rendered | No | Fixed static fixture, bounded protected multipart upload, opaque-ID streaming download and cleanup |
| `sse-notifications` | `yetlet-sse-notifications` | API | No | Bounded SSE replay, heartbeat, disconnect cleanup and lagging-client policy |
| `forms-validation` | `yetlet-forms-validation` | Server rendered | No | Stateless SSR forms, escaped accessible errors and no-side-effect 303 PRG |
| `sqlite-crud` | `yetlet-sqlite-crud` | API | No | Direct parameterized SQLite, explicit transactions, restart persistence and optimistic versions |
| `multi-island-dashboard` | `yetlet-multi-island-dashboard` | Full stack | Yes | Complete SSR plus independently built/lazy Counter and Filter islands |
| `style-showcase` | `yetlet-style-showcase` | Server rendered | No; optional WASM | Four-category style gallery, actual Rust/CSS disclosures and optional local class switching |

YT-241 Style Showcase passed independent implementation Reviewer `wc_job_SjBc9esAHAREdWS7` before its separate parent-owned catalog promotion. Its catalog command and routes work with native Rust/Cargo only. The optional `yetlet-style-showcase-browser` package and exact build command are documented in its README; they are not prerequisites of the native catalog entry. The measured Chrome/Firefox scope and Firefox limitations remain explicit there.

## Research fixtures

The five [completed research studies](README.md#completed-research-prototypes) are bounded application-owned prototypes, separate from this thirteen-application catalog. Use their fixture READMEs and retained measurements to understand the decisions and limitations; they do not select additional supported core APIs or change application qualification.

Run the existing navigation checks:

```bash
cargo xtask check samples
cargo xtask check docs
```

## Retained sample delivery history

The expandable record below preserves intermediate M8/M9 planning and promotion claims, reviewer identities and evidence links. Its references to remaining planned work describe those earlier stages; the current catalog above has thirteen applications verified at bounded prototype scope, including the separately reviewed YT-241 addition.

<details>
<summary>M8/M9 delivery sequence and promotion evidence</summary>

## M8 delivery sequence

- **M8a / YT-044:** establish this catalog, documentation and validation gate;
- **M8b / YT-045:** add an API-only REST sample with no browser dependency;
- **M8c / YT-046:** add an SSR shopping cart using an explicit opt-in Session provider;
- **M8d / YT-047:** audit application boilerplate and promote only repeated, proven improvements;
- **M8e / YT-048:** run the complete regression, dependency/bundle review and evidence freeze.

## M9 expansion: all twelve samples and final freeze verified

The [review-gated M9 plan](M9-PLAN.md) preserves the agreed public-facing display sequence: `hello-world`, `todo`, `rest-api`, `session-cart`, `auth-notes`, `middleware-lab`, `file-server`, `sse-notifications`, `modular-shop`; supplements are `forms-validation`, `sqlite-crud` and `multi-island-dashboard`.

**Verified now:** Todo, REST, Session Cart and native Hello World (`YT-050` reviewer `wc_job_Htk-Quo_aFExMJs3` PASS). Middleware Lab is now also verified (reviewer `wc_job_UnKflDaA8ZQHXCce` PASS). M9 implementation is sequenced by dependency/security prerequisites rather than display order. Forms Validation has now passed its repeat reviewer `wc_job_lGtk_ZIoAvK4F6PW` (after the English-language accessibility repair); all remaining planned samples must be independently reviewed before being considered complete. M9 planning reviewer `wc_job_fr93PI1YLOtR3KjE` already passed.

The bounded [File Server](../examples/file-server/README.md) serves an exact public fixture and token-protected temporary uploads/downloads on loopback. Its [M9d evidence](M9D-VERIFICATION.md) and exact catalog entry passed reviewer `wc_job_g8JSu80-Jseg6lmG` before promotion. No arbitrary document root, browser upload form or default storage provider is implied.

SQLite/data access stays **SQL-first, application-local and opt-in**: no Yetlet ORM, mandatory DB adapter or automatic ORM dependency. The bounded M9e comparison selects direct `rusqlite` for this one-file sample and isolates synchronous calls through a bounded blocking-task gate. After repairing reviewer-discovered incomplete-schema handling, repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS and the eighth catalog entry was promoted. Any external ORM remains an explicit future application choice, not a framework default.

The M9f SSE source is native HTTP only: an explicit app-owned Hub bounds replay and per-subscriber queues, disconnects lagging consumers and supports `Last-Event-ID` replay. After repairing reviewer-discovered control-character normalization, repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS and the ninth catalog entry was promoted.

M9g uses a separate server sample and two separate WASM companion packages. Complete SSR survives missing, malformed or partially invalid manifests; each loader fetches only its own module/WASM and owns only its root/listeners/lifetime. After reviewers required whole-manifest browser evidence and retained exit-0 logs, final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS and the tenth catalog entry was promoted.

M9h keeps authentication, Session, authorization and note storage distinct. OS-random synthetic terminal codes authenticate two fixture users; successful login/logout rotate Session identity/CSRF, and every note route checks server-owned ownership. The complete isolation/security matrix and reviewer `wc_job_EgSLng_VafIvs3ju` passed, and the eleventh catalog entry was promoted. This remains a loopback fixture, not a production identity provider.

The [Modular Shop](../examples/modular-shop/README.md) composes Identity, Catalog, Session Cart and SQL Orders through ordinary typed setup functions. Its rollback/idempotency/authorization/disabled-module matrix and repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` passed before the exact twelfth catalog entry was promoted. No plugin registry, DI container or ORM is introduced.


</details>
