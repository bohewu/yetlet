# YT-127 — bounded Starter admission and overload

Date: 2026-10-23. Status: **Done** at bounded A1 scope after independent
Reviewer PASS, zero issues and required followups. Scope is the opt-in
Production Starter application, not a framework-wide provider or deployment
qualification. All five Phase 2 capabilities have subsequently been independently accepted at
their bounded scopes; the next remaining decision is YT-096 D1–D6.

## Selected policy and ownership

| Boundary | Selected behavior |
|---|---|
| SQL dispatch | Four permits per Store and its clones; immediate `try_acquire_owned`, zero permit waiters. Tokio blocking-pool queued work also holds a permit. Saturation/closure maps through existing `StoreError::Unavailable` to 503 |
| HTTP handlers | 32 active futures per constructed router, shared by router clones; immediate admission, zero queue. Admission precedes Session/body/handler work and returns 503 on exhaustion or closed Store |
| Request deadline | Five seconds from admission to Response creation, including body parsing and awaited SQL. 504 explicitly states that a write may still complete |
| Readiness | Existing health route bypasses HTTP admission, shares the four SQL permits, and bounds its caller wait to one second. Saturation, probe failure or closed/lifecycle-unready Store returns 503 |
| Liveness | Existing route bypasses admission and SQL, returning 200 while reachable |
| Shutdown | Signals mark unready and close Store admission before Axum draining. Wait for HTTP handlers and actual SQL closure completion. Ten-second grace; success emits `stopped`, expiry emits `shutdown_incomplete` and exits 1 |

These constants are deliberately app-owned. Existing `Store::open`, `web::app`
and all public `StoreError` variants remain source compatible. An additive
`web::app_with_admission` constructor accepts `Admission`: 1–1024 active requests
and a positive request timeout of at most 60 seconds. No existing TOML/environment
key changes or mandatory configuration fields are introduced. Core/HTML/DOM/browser
source and dependencies are unchanged. Tokio's existing native `time` feature
is made explicit; `http-body` is a dev dependency for a deterministic stalled-body
fixture, using the already locked package. No package version is upgraded.

Fail-fast admission intentionally avoids queue allocations and wait cancellation
races; it provides no fairness guarantee or burst smoothing. A declared normal
load here is at most four concurrent small requests on uncontended local SQLite.
Larger concurrency or storage contention may correctly produce 503. Limits are
per Store/router instance, not distributed, and do not control arrivals over time:
rate-limit quotas/429 remain a separate optional capability. No SSE or other
long-lived application route exists, so none is introduced or exempted. Only the
actual health routes have exceptions. Existing 404/405 fallbacks stay bounded.

## Cancellation, operations and client outcomes

Dropping an unpolled Store operation dispatches nothing. Once a permit is acquired,
dispatch has no intervening await; the blocking closure owns a completion guard
and the permit. Dropping the async waiter, including after a request deadline,
does not release that capacity. The connection closes before the guard releases
its permit and notifies drain waiters; errors and unwinding follow RAII cleanup.
Closing admission is permanent for that Store and its clones. Work admitted
before closure can finish, while future acquisition fails. Drain waits for every
permit, including work whose original HTTP waiter no longer exists.

The two-second SQLite busy timeout concerns database lock contention, not an
operation-wide or filesystem deadline. No unsupported cancellation/rollback
promise is made for started SQL. The locked application uses Tokio **1.53.3**;
its local `src/task/blocking.rs` documentation confirms non-abortable started
work and potentially indefinite runtime destruction. The upstream
[spawn_blocking reference](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
describes the API; the versioned 1.53.3 web page was unavailable, so the installed
locked source was authoritative. The older [YT-029 study](YT-029-HTTP-DEADLINE-RESEARCH.md)
remains research evidence at its own versions, not new execution evidence.

A 504, lost response, or 503 is not evidence of rollback. A create can commit
without the client learning its new ID; no idempotency key exists. Clients must
inspect persistent state and reconcile uncertain writes rather than blindly
retrying. An early read can race a still-running operation and is not proof of
failure. Versioned status updates still use `expected_version`; reload and decide
against current state. No new retry or data policy is invented. API timeout errors
retain the existing JSON error envelope; browser timeout errors are bounded text.
Capacity/storage-unavailable errors retain the existing 503 category/message.
No Retry-After header implies that an uncertain mutation is safe to replay.

HTTP deadlines are cooperative and finish at Response creation. They do not
bound TCP connections, header arrival, body transmission after Response, or an
unresponsive runtime. Operators still own transport and edge limits. The SQL
capacity bound survives a response timeout, but operation duration can exceed it.
The shutdown grace explicitly exits the process on incomplete draining instead
of returning through Tokio runtime drop and waiting indefinitely. This may leave
write outcomes unknown and skips remaining Rust destructors; check/reopen the
database and reconcile before retrying. It is a bounded application shutdown
policy, not an OS watchdog or backup/recovery qualification.

## Verification and source identity

Executed in the current checkout on branch `docs/yt113-123-opt-in-capability-plan`,
HEAD `5db8b8c61b6d319a641814dff4af493ce3e5d965`, with T0 documentation plus this
uncommitted implementation. No commit, push, checkout, staging or external deployment
occurred. Exact relevant-file SHA-256 values, dirty paths, environment and command
results are retained under
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T1/`.

Changed files: Starter `src/store.rs`, `src/web.rs`, `src/main.rs`, `tests/http.rs`,
`Cargo.toml`, `Cargo.lock`, `README.md`; this record and the YT-127 entries in
`BACKLOG.md`, `ROADMAP.md`, `PRODUCTION-READINESS-PHASE2.md`. Settings/schema,
SQL CRUD, Session/CSRF/authentication implementation and migration remain unchanged.

| Evidence | Result and limit |
|---|---|
| Starter product tests | 10 library plus 10 HTTP tests pass. Existing migration/conflict/reopen, Session/CSRF, Bearer/origin, bounded validation, CSP/escaping, readiness and secret-safe logging tests pass |
| SQL saturation/caller cancellation | Four channel-blocked started closures; 1,000 further operations rejected immediately; readiness unavailable. Abort caller tasks; all permits remain held until explicit release, then recovery succeeds |
| Late writes and drain | Started closure held across timeout/caller abortion; close rejects further work; drain remains pending until release; reopened DB contains exactly the late write. Separate HTTP 504 under SQLite lock is followed by successful persisted creation after lock release |
| HTTP saturation/timeout | Signalled pending request-body fixture fills one configured slot. Excess API/browser requests return 503 with security/request-ID headers. Health remains reachable, body wait returns 504, no pre-dispatch write occurs, admission recovers, closed Store refuses readiness/work |
| Declared normal load | 20 batches of four concurrent writes: 80/80 return 201 and 80 rows persist, with no unexpected rejection |
| Actual localhost overload/recovery | Production profile, default limits, isolated synthetic SQLite. Four normal writes return 201. Hold an external SQLite write transaction, dispatch four writes, observe readiness 503, then 20/20 excess requests return 503 and liveness 200. Release lock: four writes return 201 and readiness returns 200 |
| Actual HTTP admission/deadline | Default 32 stalled upload handlers cause excess 503 while readiness/liveness remain 200. All 32 return 504 after 5.012 seconds, with security headers; zero rows, recovered API/readiness, clean exit 0 |
| Actual shutdown during overload | Again hold four writes; SIGTERM logs shutdown request while process remains alive. Release lock, receive four 201 responses, exit 0/`stopped`; reopen verifies all 12 rows from the three batches |
| Actual grace exhaustion | Local slow client with 1,024-byte receive buffer and 30 pipelined reads of 100 synthetic 4,000-character issues leaves response transmission blocked. SIGTERM exits 1 after 10.054 seconds, logs `shutdown_incomplete`, never `stopped` |
| Static/document checks | Targeted fmt, strict all-target Clippy, locked offline build, repository docs and whitespace checks; exact commands/exits retained in task result |

Commands from repository root:

```bash
cargo test --offline --manifest-path examples/production-starter/Cargo.toml
cargo build --locked --offline --manifest-path examples/production-starter/Cargo.toml
cargo clippy --locked --offline --manifest-path examples/production-starter/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path examples/production-starter/Cargo.toml --check
cargo xtask check docs
git diff --check
```

The initial offline test updated only the root application's dependency membership
in its lockfile for the existing `http-body` package. Build/Clippy then use
`--locked`. Actual HTTP observations use one-shot Python standard-library
`http.client`, `sqlite3`, socket and subprocess calls, with bounded waits and
finally-block child cleanup; no new validator framework or load harness is added.
Exact one-shot commands are retained in `commands.json`. Raw outputs are
`http-deadline-results.json`, `http-deadline-server.log`,
`localhost-results.json`, `localhost-server.log`,
`shutdown-limit-results.json`, `shutdown-limit-server.log`, and the command logs.

Cleanup: owned PID 1317393/listener 49581, PID 1317594/listener 43923 and
PID 1319005/listener 48033 were reaped; all listeners refused subsequent connections. Temporary SQLite/WAL/SHM
directories were removed. All product-test async/blocking work was joined or
drained. No browser, watcher, external service or background listener remains.

## Remaining boundaries and acceptance

Independent Reviewer `/root/prod2_admission_review_yt127_20261023` returned
**PASS**, zero issues and required followups; YT-127 is Done at this bounded
scope. The result is retained in the run observations at
`observations/T1/independent-review.json`; role/model/effective effort were
verified before controller acceptance. Only acceptance bookkeeping changed
after review, with no runtime or contract edits. This is
local Linux functional evidence, not B1 load/soak/capacity measurement. Windows,
browser/WASM, visual/accessibility, external deployment, stalled storage under
forced exit and whole-connection/network protection are not newly qualified.
Existing browser/CSP and YT-103 asset evidence retain their original scopes;
this task runs only source-relevant HTTP security assertions. A2 observability,
B1 local qualification, B2 application restore and C1 compatibility have
subsequently been accepted at their separately recorded scopes.
No major architecture or security/data-policy decision is open in this slice.
Candidate `ce56e08`, YT-096 In progress/D1–D6 Pending and YT-020 Blocked remain.
