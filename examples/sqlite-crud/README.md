# SQLite CRUD — SQL-first, Application-owned Persistence

This native-only sample uses **direct parameterized SQL** through `rusqlite`. It deliberately does not provide or depend on a Yetlet ORM, active record, `DbContext`, repository/unit-of-work framework, migration product, DI container or browser runtime.

## Run

From the repository root:

```bash
cargo run -p yetlet-sqlite-crud
```

The anonymous sample listens on `127.0.0.1:3007` by default and rejects non-loopback listeners. Its default database is `target/yetlet-sqlite-crud.sqlite3`; override local test paths with `YETLET_SQLITE_PATH` and the listener with `YETLET_SQLITE_LISTEN`.

Routes:

- `GET` `/healthz` — verifies the database and reports schema version 1.
- `GET` `/api/notes` with optional `?limit=50` — parameterized, bounded list query.
- `POST` `/api/notes` — creates a note and an audit record atomically.
- `GET` `/api/notes/{id}` — reads one typed row.
- `PUT` `/api/notes/{id}` — updates with `expected_version` optimistic concurrency.
- `DELETE` `/api/notes/{id}` with `?expected_version=2` — versioned atomic deletion.
Example:

```bash
curl -i -X POST http://127.0.0.1:3007/api/notes \
  -H 'content-type: application/json' \
  -d '{"title":"Direct SQL","body":"No ORM involved."}'
```

Mutations require exactly one `application/json` content type where a JSON body is used and reject any `Origin` header; this command-oriented local API intentionally does not enable browser-origin mutations or CORS. JSON is limited to 8 KiB. Titles are unique under SQLite `NOCASE` comparison (ASCII case-insensitive), and updates/deletes require the current positive version.

## Data-access boundary

`Database` owns a path plus a four-operation semaphore. Each request acquires a permit, opens/configures one SQLite connection, and performs synchronous work inside `tokio::task::spawn_blocking`; no SQLite operation blocks an async executor thread. Startup owns migration `migrations/0001_init.sql`, `PRAGMA user_version`, WAL and busy-timeout configuration. Every mutation uses `BEGIN IMMEDIATE`, explicit parameter binding and an audit insert in the same transaction. Failed audit writes, stale versions and constraint failures roll back the entire transaction.

New database files are created with owner-only mode `0600` on Unix before SQLite opens them. The bundled SQLite library improves single-binary portability for this prototype but increases native build size. Opening a connection per bounded operation is intentionally simpler than introducing a pool; applications with measured pool/concurrency requirements may select a mature pool explicitly. The database path is operator configuration, not user input.

See [M9e data-access comparison](../../docs/M9E-DATA-ACCESS-COMPARISON.md) for the bounded `rusqlite` versus `sqlx` spike and [M9e verification](../../docs/M9E-VERIFICATION.md) for retained test/reviewer evidence.

## Verify

```bash
cargo test --locked -p yetlet-sqlite-crud --all-targets
cargo build --locked -p yetlet-sqlite-crud
cargo xtask smoke sqlite-crud
cargo tree --locked -p yetlet-sqlite-crud --edges normal
cargo xtask check samples
```

This is a local persistence and transaction-semantics sample, not production authentication, encryption-at-rest, backup, multi-process clustering, online migration, load-test or durability certification.
