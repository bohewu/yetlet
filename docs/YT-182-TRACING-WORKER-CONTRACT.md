# YT-182 — Opt-in request tracing and safe worker propagation

Date: 2026-10-25 (Asia/Taipei). Status: **Independent contract Reviewer PASS,
zero issues/required followups. Implementation/new proof separately accepted;
YT-182 Done at bounded application-reference scope.**

## Authority and smallest consumer

The maintainer authorizes contract-first minimal opt-in integration, native
middleware, safe worker propagation and independent acceptance before Done.
No generic AOP, old passing-proof replay or new-change Commit/Push. The 19
YT-181 paths are committed as `a10db2b`, strict/builtin Guard/hook PASS with
zero findings, tree `2c2e6ea89a02dfcc5ba3d1bf6e345521312537fb`; no Push.

Select Board/Admin as **one actual underlying consumer**. Starter's existing
request spans/subscriber and [YT-123](YT-123-TRACING-CORRELATION-STUDY.md) /
[YT-128](YT-128-OBSERVABILITY.md) provide comparison/evidence; do not modify or
rerun Starter to manufacture a second consumer. No `yetlet-tracing` crate,
common tracing trait, Core field or extraction claim is selected. Keep native
Axum/Tower middleware, typed request extensions, `tracing` futures/synchronous
scopes and explicit application startup. Existing Auth/Session/CSRF, SQL grants,
limits, outcomes and drain ownership stay unchanged.

## Activation and dependency boundary

Add the application feature **`request-tracing`**, default off. Board's tracing
and existing request-ID dependencies are optional normal dependencies. Test
capture may use the existing mature tracing subscriber library as a development
dependency; that does not install a normal subscriber. Admin forwards the
feature and opts into its own native JSON subscriber dependency.

Admin's existing Settings loader selects **`request_tracing.enabled`**, default
false, environment **`APP_REQUEST_TRACING__ENABLED`**. True without the compiled
feature fails startup validation; false does not initialize a subscriber or
construct request-ID/span/worker-trace state. Read once; no hot reload, RUST_LOG/
EnvFilter, generic provider registry or automatic exporter. Existing constructors
retain their behavior; explicit new application startup constructors select this
mode and compose with either Session and optional commit-write authorization.
Use a small private startup selection value rather than replacing native handlers.
No browser/WASM feature receives these server dependencies.

Board never sets a global subscriber. Admin initializes one only for explicitly
enabled tracing, before constructing/serving the app. A conflicting subscriber
initialization is a redacted startup error, never silent replacement. The
subscriber admits only the fixed Board reference target at INFO and higher,
with other targets OFF; do not accidentally enable dependency/raw-URI diagnostics.
This is the reference host's choice, not a framework logging default. Reuse
existing resolved versions where compatible; document exact lock/dependency
changes rather than implying zero transitive tracing in ordinary Axum.

## HTTP scope and trusted local correlation

Install the optional layers **inside Board's existing outer transport/security
middleware**: security → generated RequestId → request tracing → Session/Auth/
handler → normal response/security finalization. This preserves CSP/private
headers even for early request-ID exhaustion. Outer Host/Origin refusals and
Admin's separate Vite static router remain outside tracing; no complete-edge
coverage claim. Normal handler/Session/Auth failures and fallback routes inside
Board remain observable without changing routing or status.

Use exactly one existing `RequestIds` provider per selected Board. Read its
server-assigned typed extension, never an incoming request/trace header. Preserve
its response-header semantics and finite exhaustion behavior; do not use sentinel
0 as identity. These numeric IDs are provider/process-local, predictable and
reset on provider recreation; they are not durable, globally unique, tenant,
authentication, idempotency or distributed trace identities.

Create an explicit root span with no inherited ambient parent. Only these
application fields are selected before formatting:

| Record | Allowed fields/meaning |
| --- | --- |
| HTTP root span | Fixed span/target, generated numeric `request_id`, method from GET/HEAD/POST/PUT/PATCH/DELETE/OPTIONS/OTHER |
| Response constructed | Fixed event, numeric status, integer elapsed milliseconds; Response return only |
| HTTP future dropped | Fixed event and integer elapsed milliseconds; future destruction, not proof of client disconnect, timeout, worker abort or rollback |
| SQL worker span/events | Fixed operation/category, started and completed events, completion result only `ok`/`error`; linked selected local HTTP root |

No URI/path/query, route parameter, header/cookie, Principal/State/Request Debug,
user/tenant label, SQL/parameter/business body, Session/CSRF/OIDC/secret value,
uncontrolled error chain or policy/generation/ticket content enters these fields.
No body reads, buffering, serialization/hydration projection or business response
change is needed. Request IDs are not metric labels; no metric or durable audit
channel is introduced. The selected span must not inherit arbitrary caller span
fields; attributes that automatically record function arguments are not used.

## Safe async and blocking lifetimes

Instrument the request future with native `Instrument`, without holding an
entered span guard across `.await`. A small completion guard emits exactly the
selected Response-return or future-drop observation, using its own safe span
and scoped Dispatch rather than relying on ambient cleanup context. A 200 does not mean complete
body delivery. SSE/stream/body chunks, remote propagation, detached jobs and
background retry identities are outside this first slice.

Before the existing `Database::run` blocking dispatch, capture context only if
the Store has tracing explicitly selected **and** the current span matches this
reference's fixed root name/target. Unknown, nested foreign or absent ambient
context is not forwarded; direct untraced Store/control calls are not falsely
attributed to a request. Clone only that selected safe span and its application
Dispatch, never Request/Extensions/Session/Principal solely for diagnostics.

Inside the blocking closure, install the captured Dispatch for the synchronous
scope and explicitly parent a fixed SQL-worker span to the selected root. Native
`in_scope` is appropriate only around synchronous work; restore prior thread
context after the closure. The worker retains its existing semaphore permit,
connection/transaction ownership and drain behavior even after caller/request
future drop. Its completion event may therefore follow HTTP future-drop; keep
the same safe originating correlation without pretending the request succeeded.

Worker `ok` means the selected closure returned success; an arbitrary read or
worker completion is not a durable business-commit/audit guarantee. Panic,
shutdown, process loss or filtered/output-failed telemetry can leave no terminal
event. Missing events/span drop never infer rollback, successful revocation or
permission for a retry. Preserve all YT-137/181 outcome semantics and current
reconciliation authorization. Do not add task queues, retries, worker changes,
sampling service, protocol parser, background agent or generic tracing registry.

## Subscriber/output ownership

Use explicitly selected native JSON formatting and a finite target allowlist.
No collector, OTLP transport, background buffer or new output queue. The host
owns the synchronous writer (stderr in this reference), access, retention and
drain. Slow output can delay enabled tracing; no zero overhead/bounded writer
latency claim. Ordinary writer errors must not become an authorization fallback,
rewrite a business result or imply audit delivery. Trusted custom subscribers
can have their own behavior; this reference does not sanitize arbitrary external
instrumentation or make a hostile subscriber safe.

Test capture uses bounded in-memory native writer output, not a new telemetry
framework. Preserve separate observations of request lifecycle and started SQL
work. Subscriber startup failure is separate from runtime writer failure; do
not silently install a fallback global logger or rerun an exposure favorably.

## Minimum new evidence and acceptance

After contract acceptance, run only new focused cases and necessary changed-
source compile/lint/format/doc checks:

- Feature/startup off: no request-ID/header/span/worker emitter installation;
  existing defaults compile and the new enabled setting refuses absent feature.
- Actual in-process authenticated SSR and JSON/triage paths: generated correlation
  reaches SQL workers and stays consistent with the selected header/root. Supply
  incoming ID/trace/header/query/body secret sentinels; full captured projection
  excludes them and preserves response/security/Session behavior.
- Two concurrently polled request scopes keep distinct IDs; unknown ambient
  spans/control calls do not get forwarded as trusted HTTP context.
- Drop the instrumented request future after a real blocking worker has started:
  record future-drop separately, retain the permit/work, then complete/drain the
  worker with the same selected safe root. Use a bounded focused test barrier,
  not a new runtime hook or task framework.
- A failing capture writer does not change an actual request/domain outcome.
  Finish/reap test work and remove owned temporary SQLite files.

Use the existing Cargo/test patterns and narrow new-name filters; do not replay
old Auth/Valkey/OIDC/browser/Windows/benchmark or YT-181 test matrices. No live
server, Docker, Azure or performance qualification is selected. Independent
contract and implementation review each retain their separate scope; YT-182
becomes Done only after new integration evidence and final Reviewer PASS.
YT-181 remains Done, YT-183/184 Todo and YT-096/YT-020 release gates unchanged.

Primary mechanisms are native [Instrument](https://docs.rs/tracing/latest/tracing/trait.Instrument.html),
[Dispatch](https://docs.rs/tracing/latest/tracing/dispatcher/index.html) and
[target filtering](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/targets/struct.Targets.html).
Versioned web retrieval for the first two was unavailable in this session;
installed tracing 0.1.44/tracing-core source was inspected for scoped Dispatch
and future subscriber propagation. No compile/runtime claim follows from docs
inspection; the selected field/trust/lifetime policy is application-owned.


## Contract acceptance and subsequent integration

Independent contract Reviewer `/root/yt182_safe_tracing_contract_review_20261025`
returned PASS, zero issues/required followups; five document, thirteen evidence
and eighteen baseline/source hashes matched before/after review. The contract-
stage docs/whitespace checks passed (75 required documents, 184 items). Those
historical bindings describe the source **before** integration; they do not
claim that subsequently modified Board/Admin files remain unchanged.

The [implementation/new evidence](YT-182-TRACING-WORKER-IMPLEMENTATION.md) records
the later minimal integration, original failures and focused corrections. The
future-drop case exposed that a span alone does not select the event's subscriber
during future destruction; the completion guard now retains scoped Dispatch as
well. This preserves the selected trust/lifecycle contract. No passing old test
or provider/benchmark qualification was replayed.


Independent implementation Reviewer
`/root/yt182_safe_tracing_implementation_review_20261025` subsequently returns
PASS, zero issues/required followups, with 21 implementation/81 evidence/15
retained bindings stable and matching native trace/resolver observation. YT-182
is now Done at that bounded reference scope; see the separate implementation
record for original failures, successful new evidence and remaining limits.
