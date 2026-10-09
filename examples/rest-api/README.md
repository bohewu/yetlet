# API-only REST Sample

This native-only sample demonstrates typed JSON CRUD with ordinary Rust handlers, Axum extractors, Yetlet `App`, `RequestContext` middleware and opt-in request correlation. It contains no browser crate, DOM renderer, WASM target or generated asset.

## Run

From the repository root:

```bash
cargo run -p yetlet-rest-api
```

The default listener is `http://127.0.0.1:3001`; override it with `YETLET_REST_LISTEN`.

Inspect these routes:

- `http://127.0.0.1:3001/healthz` — typed health JSON;
- `http://127.0.0.1:3001/api/books` — list and create books;
- `/api/books/{id}` — read, versioned update and delete one book.

Example:

```bash
curl -i http://127.0.0.1:3001/api/books
curl -i -X POST http://127.0.0.1:3001/api/books \
  -H 'content-type: application/json' \
  -d '{"title":"The Rust Book","author":"Rust Project","year":2019}'
```

Mutations accept same-origin browser requests or non-browser clients without an `Origin` header. JSON bodies are limited to 8 KiB. The store is process-local and in-memory; this is not an authentication, persistence, distributed-concurrency or production proxy reference.
 Duplicate or malformed `Origin`/`Host` values fail closed. The comparison intentionally supports direct HTTP only; it is not a forwarded-header or trusted-proxy policy.

Validation, resource and middleware-policy failures on registered handlers use the documented JSON error envelope. Native Axum routing remains visible: an unsupported method on a registered `/api` route returns an empty `405` with API-version, no-store, nosniff and request-id headers; an unregistered path such as `/api/missing` returns an empty `404` with only the globally installed request-id header. Clients must not assume the JSON envelope or API headers for routes that were never registered.

## Verify

```bash
cargo test --locked -p yetlet-rest-api --all-targets
cargo build --locked -p yetlet-rest-api
python3 scripts/smoke_rest_api.py
```

Retained scope and dependency evidence is recorded in [M8b verification](../../docs/M8B-VERIFICATION.md).
