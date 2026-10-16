# SSE Notifications — Bounded Native Event Stream

This native-only application demonstrates explicit HTTP Server-Sent Events through Axum's mature `Sse` response. It has no browser/WASM requirement, WebSocket, hidden event circuit, Session, database, ORM or automatic DOM event forwarding.

## Run

From the repository root:

```bash
cargo run -p yetlet-sse-notifications
```

The anonymous prototype listens on `127.0.0.1:3008` by default and rejects non-loopback listeners. Override the local test listener with `YETLET_SSE_LISTEN`.

Routes:

- `/healthz` — JSON health response.
- `/metrics` — bounded Hub counters and capacities.
- `/events` — `text/event-stream` endpoint with one-second heartbeat comments and optional `Last-Event-ID` replay.
- `/notifications` — local command-oriented JSON POST for publishing one notification.

Example clients:

```bash
curl -N http://127.0.0.1:3008/events
curl -i -X POST http://127.0.0.1:3008/notifications \
  -H 'content-type: application/json' \
  -d '{"message":"build complete"}'
```

## Bounded delivery policy

The application-owned `Hub` retains the newest **16 events**, admits at most **8 subscribers**, and gives each subscriber a **4-event queue**. Event IDs are monotonic unsigned integers and never wrap. A publisher never waits on a slow subscriber: when a queue is full, that subscriber is removed and its stream closes after draining already queued events. The client reconnects with its last received `Last-Event-ID` and receives retained events newer than that cursor.

A cursor older than the replay window returns HTTP 409 `replay_gap`; a cursor newer than the stream returns 409 `future_cursor`; malformed or duplicate cursor headers return 400. This bounded prototype chooses disconnect-and-replay rather than unbounded memory or blocking every publisher. It is at-least-once across reconnect from the client's perspective; consumers should treat IDs as deduplication keys.

The one-second `: heartbeat` comment keeps idle local demonstrations observable. Dropping the HTTP body unregisters the subscriber. `/metrics` exposes active subscribers, retained events, latest ID and lagged disconnect count for the sample's verification; it is not a general monitoring API.

Publishing accepts exactly one `application/json` Content-Type, rejects all browser `Origin` headers, limits JSON to 2 KiB, and accepts 1–200 non-control characters. There is no CORS or authentication. Loopback-only admission is the safety boundary, so this is not a production notification service or multi-process broker.

## Verify

```bash
cargo test --locked -p yetlet-sse-notifications --all-targets
cargo build --locked -p yetlet-sse-notifications
cargo xtask smoke sse-notifications
cargo tree --locked -p yetlet-sse-notifications --edges normal
cargo xtask check samples
```

See [M9f verification](../../docs/M9F-VERIFICATION.md) for retained regression, dependency-isolation and reviewer evidence.
