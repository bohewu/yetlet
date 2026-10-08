# M4 Typed Actions and HTTP API — Verification

**Date:** 2026-10-08
**Status:** Local native/browser prototype evidence, not a GitHub Actions run, authentication implementation, production security certification, or independent review.
**Backlog:** YT-014, YT-015, YT-016, YT-018

## Implemented boundaries

1. **Semantic contracts:** `yetlet-actions` defines `Command` with associated response, `ActionExecutor<C>`, `LocalExecutor<F>`, `HttpExecutor`, `ActionError`, and a JSON error envelope. Raw DOM events never cross this boundary. Local-only commands require no serde implementation; serialization is required only by the HTTP executor.
2. **Shared domain:** `examples/todo-ui` defines strictly deserializable `TodoCommand` variants (`toggle` and `create`), `TodoSnapshot`, and one validated `apply_todo_command` transition. The same command works locally or over HTTP.
3. **Local browser action:** `examples/todo-browser` injects a `LocalExecutor` using browser-owned state. Its toggle button updates the TodoList without network traffic.
4. **Explicit remote action:** A separate button creates a typed `TodoCommand::Create`, uses `HttpExecutor("/api/todos")` with `fetch` and JSON, and mounts the returned server snapshot. It never sends a DOM event.
5. **Server adapter:** `examples/todo-server/src/api.rs` serves `GET /api/todos` and `POST /api/todos` using a per-router in-memory store protected by `Mutex`. The shared domain transition executes through the local executor.
6. **Renderer separation:** Native SSR `/` stays script-free. Browser `/app` and Search Island `/search` retain their existing independent renderers/loaders.

## Wire contract

A typed JSON mutation uses `POST /api/todos` with `Content-Type: application/json`:

```json
{"type":"create","title":"Saved from browser"}
```

A response includes a revision and complete snapshot:

```json
{"revision":1,"items":[{"id":1,"title":"Example title","completed":false}]}
```

The response illustrates the **shape**, not a complete fixture: the actual initial server snapshot contains two Todos. The other command variant is `{"type":"toggle","id":1}`. The demo permits up to 100 items and titles of 1–100 non-control characters after trimming.

Typed failures have a stable JSON envelope:

```json
{"error":{"code":"invalid_title","message":"Invalid command fields"}}
```

## Verification commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/check_docs.py
```

| Assertion | Local evidence |
|---|---|
| Typed local execution | Unit tests verify typed input/output, error propagation, Todo transitions, and unchanged state after failed commands |
| Todo API GET/POST | Integration tests verify create/toggle/read consistency, revisions, and shared store within one Router |
| Structured HTTP errors | Invalid title → 422; unknown id → 404; unknown fields → 400; wrong content type → 415; oversized body → 413 |
| Browser origin checks | Mismatched Origin → 403; no CORS opt-in. Origin checking is not authentication. |
| Browser local executor | Chrome two-click test emits **zero HTTP requests**, updates local state, class and text |
| Browser HTTP executor | Chrome sends one explicit POST containing only `{"type":"create","title":"Remote E2E task"}`; response mounts a 3-item server snapshot |
| Browser error flow | Invalid title returns a visible typed 422 `invalid_title` response; server revision remains unchanged |
| CSP/DOM safety | External scripts; no `unsafe-inline`/`unsafe-eval`; only `wasm-unsafe-eval` where necessary; DOM uses text nodes and SSR attributes are escaped |
| Regression | M2 DOM and M3 lazy Search Island checks remain passing, including invalid state fail-closed fallback |
| Transport | No WebSocket, raw event forwarding, implicit retry, or request-time compilation |

## Security limitations

- **Anonymous in-memory demo only.** There are no users, permissions, persistence, credentials, audit logs or sessions. Origin and JSON content-type checks reduce accidental cross-site browser posting but do **not** replace real authentication and CSRF protection for credentialed apps.
- Origin checks assume direct **HTTP** to the requested Host. HTTPS reverse proxies require explicit trusted host/scheme handling and session/CSRF policy.
- Request bodies are capped at 4 KiB. Response limits, timeouts, rate limiting, transactional concurrency policies, and idempotency keys are not generalized framework features.
- The HTTP executor does not retry non-idempotent commands automatically.
- Validation covers a single headless Chrome environment. Independent security review, full browser/IME/accessibility compatibility, and performance benchmarking remain pending.

## Next gate

M5 (`YT-019`–`YT-020`) is the remaining complete-MVP acceptance matrix and performance/compatibility baseline. Custom transports and remote event circuits remain future, explicit opt-in research, not defaults.

See [Architecture](ARCHITECTURE.md), [MVP](MVP.md), [Roadmap](ROADMAP.md), and [ADR 0002](adr/0002-events-and-actions.md).
