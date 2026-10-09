# Sample Applications

**Status:** M8a–M8e are locally complete; the sample contract, applications, DX audit and final evidence freeze are retained.

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
| `todo` | `yetlet-todo-server` | Full stack | Yes | SSR, Browser/WASM, Search Island, typed HTTP actions and keyed local updates |
| `rest-api` | `yetlet-rest-api` | API | No | Typed Book CRUD, RequestContext policy, structured errors and request correlation |
| `session-cart` | `yetlet-cart-server` | Server rendered | No | Opt-in server-side Session, native forms, CSRF boundary and refresh persistence |

## M8 delivery sequence

- **M8a / YT-044:** establish this catalog, documentation and validation gate;
- **M8b / YT-045:** add an API-only REST sample with no browser dependency;
- **M8c / YT-046:** add an SSR shopping cart using an explicit opt-in Session provider;
- **M8d / YT-047:** audit application boilerplate and promote only repeated, proven improvements;
- **M8e / YT-048:** run the complete regression, dependency/bundle review and evidence freeze.

Run:

```bash
python3 scripts/check_samples.py
python3 scripts/check_docs.py
```
