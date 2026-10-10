# Sample Applications

**Status:** M8a–M8e, M9 planning and M9a–M9e have independent reviewers. Eight catalogued applications are locally verified; remaining M9 samples are planned.

Yetlet samples exist to validate application ergonomics, dependency isolation and security boundaries. They are not a parallel tutorial framework and do not authorize new core abstractions merely to make a demo look shorter.

## Catalog contract

`examples/samples.toml` is the machine-readable catalog for user-facing runnable applications. `scripts/check_samples.py` validates the catalog using only the Python standard library.

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
| `middleware-lab` | `yetlet-middleware-lab` | API | No | Native Tower middleware scopes, request extensions, short-circuit, streaming |
| `file-server` | `yetlet-file-server` | Server rendered | No | Fixed static fixture, bounded protected multipart upload, opaque-ID streaming download and cleanup |
| `forms-validation` | `yetlet-forms-validation` | Server rendered | No | Stateless SSR forms, escaped accessible errors and no-side-effect 303 PRG |
| `sqlite-crud` | `yetlet-sqlite-crud` | API | No | Direct parameterized SQLite, explicit transactions, restart persistence and optimistic versions |

## M8 delivery sequence

- **M8a / YT-044:** establish this catalog, documentation and validation gate;
- **M8b / YT-045:** add an API-only REST sample with no browser dependency;
- **M8c / YT-046:** add an SSR shopping cart using an explicit opt-in Session provider;
- **M8d / YT-047:** audit application boilerplate and promote only repeated, proven improvements;
- **M8e / YT-048:** run the complete regression, dependency/bundle review and evidence freeze.

## M9 expansion: Hello World, Middleware Lab, Forms, File Server and SQLite verified

The [review-gated M9 plan](M9-PLAN.md) preserves the agreed public-facing display sequence: `hello-world`, `todo`, `rest-api`, `session-cart`, `auth-notes`, `middleware-lab`, `file-server`, `sse-notifications`, `modular-shop`; supplements are `forms-validation`, `sqlite-crud` and `multi-island-dashboard`.

**Verified now:** Todo, REST, Session Cart and native Hello World (`YT-050` reviewer `wc_job_Htk-Quo_aFExMJs3` PASS). Middleware Lab is now also verified (reviewer `wc_job_UnKflDaA8ZQHXCce` PASS). M9 implementation is sequenced by dependency/security prerequisites rather than display order. Forms Validation has now passed its repeat reviewer `wc_job_lGtk_ZIoAvK4F6PW` (after the English-language accessibility repair); all remaining planned samples must be independently reviewed before being considered complete. M9 planning reviewer `wc_job_fr93PI1YLOtR3KjE` already passed.

The bounded [File Server](../examples/file-server/README.md) serves an exact public fixture and token-protected temporary uploads/downloads on loopback. Its [M9d evidence](M9D-VERIFICATION.md) and exact catalog entry passed reviewer `wc_job_g8JSu80-Jseg6lmG` before promotion. No arbitrary document root, browser upload form or default storage provider is implied.

SQLite/data access stays **SQL-first, application-local and opt-in**: no Yetlet ORM, mandatory DB adapter or automatic ORM dependency. The bounded M9e comparison selects direct `rusqlite` for this one-file sample and isolates synchronous calls through a bounded blocking-task gate. After repairing reviewer-discovered incomplete-schema handling, repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS and the eighth catalog entry was promoted. Any external ORM remains an explicit future application choice, not a framework default.

Run:

```bash
python3 scripts/check_samples.py
python3 scripts/check_docs.py
```
