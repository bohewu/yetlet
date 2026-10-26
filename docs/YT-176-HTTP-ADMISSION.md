# YT-176 — Opt-in fail-fast HTTP admission for the Auth reference

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded local application-
integration scope; final independent Reviewer PASS, zero issues/followups.**

## Authority and selected slice

The user selects the fast-rejection option from
[YT-175](YT-175-SESSION-BURST-OVERLOAD-POLICY.md): establish a contract, then
minimally integrate opt-in bounded outer HTTP admission, keep Session limits,
add zero queue and zero automatic retry, reuse passed evidence and obtain
independent Reviewer PASS. New changes remain uncommitted/unpushed.

The four YT-175 documents are committed as `b42f794`, strict/builtin Commit
Guard PASS with zero findings, tree `a71f29bbc4703c7eff3cffc5d7fae539797d7f74`;
no Push. Preserve [YT-171](YT-171-AUTH-CAPACITY-RECOVERY.md),
[YT-172](YT-172-SESSION-ADMISSION-BURSTS.md),
[YT-173](YT-173-HTTP-SESSION-TIMELINE.md) and
[YT-174](YT-174-HOST-PASSIVE-SAMPLING.md), including every failed request/review,
observer limitation, native receipt and cleanup result. No old proof is replayed.

The consumer is the existing opt-in `auth-postgres` binary in
`examples/axum-workload-comparison`. This application-owned reference proves
composition without adding a framework crate, Core API, generic timeout system,
rate limiter or dependency. Preserve ADRs
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md).

## Activation and scope

Add an explicit example feature `auth-workload-admission`, enabling only the
existing `auth-session-postgres` feature. Default features remain empty. The
feature-off binary has no admission module, environment lookup, semaphore or
middleware. Existing benchmark selections and build features do not activate it.

With the feature selected, read the application-owned
`YT176_HTTP_MAX_IN_FLIGHT` once at startup, before any backend connection.
An absent value disables installation and creates no gate. A present value
must be ASCII digits representing 1–1024; zero, malformed/non-Unicode or
out-of-range input fails startup with the existing redacted error. There is no
hot reload, new Settings-provider contract or additional enable flag. An
explicit value, such as four, is a chosen local reference cap, not a universal
or production recommendation. Session remains four active/eight total,
80ms active wait/400ms per-API deadline, with unchanged worker ownership,
TTL/payload/SQL limits, reconciliation and shutdown behavior.

Construct one gate and share it across all existing Session-scoped routes:
API documents, private SSR, identity and logout-form/logout, and login form/login.
Native Axum middleware goes outside the existing Session/Auth/handler chain;
private rejection and existing outer security middleware must cover all scoped
paths, including early rejection.
The existing `/health` and optional `/fixture/observations` routes remain
outside the gate; this creates no new health bypass or business route.

## Admission and lifetime invariants

- Use one shared semaphore and **immediate try-acquisition**, with zero extra
  permit waiters, HTTP buffer, delay, retry or downstream call on rejection.
  Do not use a waiting concurrency limiter or add a second queue.
- A full/closed gate returns bounded generic **503**, no-store, Vary: Cookie,
  nosniff and the existing strict CSP. It emits no Set-Cookie, redirect,
  authentication challenge or invented Retry-After. Rejection must precede
  Session parsing/resolution, Auth lookup, SQL or handler execution.
- An admitted request retains its HTTP permit through downstream response
  construction **and Session finalization**. Preserve downstream status/body/
  headers; normal auth/CSRF/error handling remains authoritative. The permit
  ends at ResponseReady/future drop, not wire/body delivery or connection close.
- Error responses and request cancellation release the HTTP permit. This never
  releases a provider permit held by real outstanding backend work. Cancellation,
  timeout, a finalizer 503 or a lost reply cannot prove rollback or make an
  uncertain write safe to replay. No automatic retry is added anywhere.
- Gate clones/route clones share accounting. Separately constructed gates and
  processes do not share capacity. The gate bounds HTTP request futures, not
  backend operations, network connections or distributed/per-tenant quota.
  Background work/fan-out/outstanding cancelled callers can still fill Session
  admission; a zero-queue gate does not promise all burst requests succeed.

No timeout, priority lane, fail-open/stale/Memory fallback, synthetic successful
Auth result, provider/SQL tuning, extra manager or hard-coded production cap
is selected. The separate rate limiter remains opt-in and unchanged.

## Minimal new verification

First obtain independent contract review, then implement the private module,
small startup/router feature wiring and example usage documentation. Reuse
existing native Axum/Tower test seams; add focused in-process router tests for
this new behavior only:

| New acceptance case | Required evidence |
| --- | --- |
| Activation | Pure input tests for unset/valid/invalid cap; feature-off compile and unchanged default feature/dependency selection; no process-global environment mutation |
| Early shared rejection | Hold an admitted request at a controlled inner boundary, then reject requests across API/SSR/auth route names before inner Session/Auth/handler counters advance; verify exact generic 503/body/private+CSP headers and absent Set-Cookie/Location/Retry-After |
| Finalization | Let a handler finish while a synthetic inner finalizer remains pending; capacity stays occupied until finalization completes; preserve the admitted response and show recovery on the same gate |
| Error/cancellation | Downstream error and aborted request release HTTP capacity without retry; keep backend-permit semantics unchanged in provider source and retained evidence |
| Disabled/scope | Disabled installation preserves downstream behavior; health/observation remain usable while the scoped gate is full; actual runtime route wiring is independently source-reviewed |

Use deterministic oneshot/Notify coordination, finite task counts, bounded
completion guards and explicit task reaping; do not use timing sleeps as proof
of queue capacity. The synthetic inner boundary tests composition and permit
lifetime; it is **not live Valkey/SQL or real Session-finalizer qualification**.
The actual router's outer layer position and unchanged real Session finalizer
are checked against source. Old accepted backend/Auth/CSRF/tenant/cancellation/
drain/oracle tests are reused, not rerun. No new live-backend or capacity claim
is made from in-process tests.

Run only new-module tests with the standalone example's locked/offline
`auth-postgres` target and the new feature; filter out all pre-existing tests.
Check the changed auth binary without the new feature, focused formatting,
new docs/links/DAG and diff whitespace. Build/check commands are bounded at
600s each; focused test execution must have a finite timeout. Capture native
exits and retain failures. A product failure permits only a material minimal
repair; a harness/operational seam gets one smallest canonical correction,
without a new run or replaying passed checks. No extra harness framework,
load generator, observation API, service/container/image/cloud allocation or
qualification matrix is needed. Test tasks/processes must finish or be reaped.

## Acceptance boundary

Expected changes: this report, Backlog/Roadmap/index, the example Cargo feature,
`auth_postgres.rs`, one private `auth_postgres/admission.rs` module with its new
tests, and the example README. No lockfile/package graph/provider/Core/browser
change is selected. Keep old source/evidence bindings except the explicitly
changed example manifest/router/README; retain their pre-change hashes.

Final independent Reviewer PASS precedes Done. Report new focused evidence,
unchanged Session parameters and middleware order, disabled behavior, remaining
capacity/body-delivery/cancellation limits, actual files and Git state.
YT-096 remains In progress with D1–D6 Pending; YT-020 stays Blocked. No further
benchmark, integration, new-change Commit or Push is automatically activated.

## Implementation and focused evidence

Independent contract Reviewer
`/root/yt176_http_admission_contract_review_20261025` returns PASS, zero issues/
required followups, before implementation. Native role/model/high effort are
verified; all four contract files, forty-one accepted and three pre-change
bindings remain stable through that review.

The new [private module](../examples/axum-workload-comparison/src/bin/auth_postgres/admission.rs)
contains the only new gate and six focused tests. The
[example manifest](../examples/axum-workload-comparison/Cargo.toml) adds only
`auth-workload-admission = ["auth-session-postgres"]`. The
[actual router](../examples/axum-workload-comparison/src/bin/auth_postgres.rs)
reads the optional cap first in `run()`, before backend setup, then installs
one gate **after constructing the Session-wrapped scoped router**. Axum's outer
request path is existing security/CSP → new admission → Session → Auth where
applicable → handler; response finalization unwinds inside the held HTTP permit.
Health is merged outside the scoped router and the optional observation route
remains outside; existing endpoint behavior is preserved. No handler/provider
implementation is replaced.

`from_environment` uses `var_os` and a pure parser: unset creates no gate;
non-Unicode, malformed and out-of-range values return only `Err(())`. Zero is
invalid rather than silently disabling protection. An enabled installation
shares an Arc-backed semaphore across all router/layer clones. A failed
try-acquisition never calls `next`; admitted responses return unchanged. The
new gate introduces no timer, wait queue, retry, metrics endpoint or shutdown
worker. Its owned HTTP permit is RAII-managed through `next.run().await`.
The example README documents feature/cap activation and the scope limits.

| New check | Actual result |
| --- | --- |
| Focused `auth-postgres` tests, feature `auth-workload-admission`, filter `admission::tests::` | **6 passed, 0 failed; 1 pre-existing oracle test filtered out**; native exit 0 |
| Auth binary check, only existing `auth-session-postgres` feature | **PASS**, native exit 0; admission module/configuration/wiring compiled out |
| New module focused rustfmt | **PASS**; existing main changes stay limited to cfg/startup/layer wiring |
| Literal manifest/lock/source binding check | Default features empty; dependency/dev-dependency entries and all other feature definitions unchanged; lockfiles and 41 accepted bindings unchanged |
| New documentation and diff whitespace | **PASS**: 75 documents/176 backlog items, links/Markdown/whitespace/DAG and `git diff --check` |

Both Cargo commands use `--offline --locked` with the standalone example
manifest and a task-local target directory, no research compiler cfg. Native
command/exit/elapsed output is retained in `focused-tests-native.json` and
`feature-off-native.json`; formatting and feature-boundary receipts are separate.
Only the six new tests execute; previous oracle/provider/Auth/security/load
checks and all accepted analysis remain unreplayed.

The shared-gate test holds one request after its handler has returned, inside a
synthetic finalizer. Seven Session-scoped API/SSR/Auth method/path combinations
all return the exact generic private/CSP 503 without advancing inner counters
or setting a cookie/redirect/retry hint/challenge. Health/observation remain
usable. Releasing that finalizer preserves the original status/body/headers,
returns the permit and permits a successful same-gate request. Separate cases
cover disabled installation, closed-gate rejection, downstream error, aborted/
reaped request and subsequent recovery without hidden retries. Pure parser
cases include bounds, malformed values and Unix non-Unicode input without
mutating the process environment. Test coordination uses Notify and bounded
future/task waits; no timing sleeps or real store are used.

This is **in-process composition evidence**, with synthetic Session/Auth/
finalizer counters. Real Session/Auth/SQL qualification is reused. Actual router
layering, Session parameters and the unchanged real finalizer await are source-
reviewed; no claim is made that these tests executed Valkey or proved a new
capacity/error-rate/SLO. ResponseReady lifetime also does not bound socket/body
delivery. Cancelling the HTTP future frees only its own permit; existing live
backend ownership remains governed by the unchanged provider.

No backend connection, HTTP listener, container, image, cloud allocation or
service starts during this delivery. All test requests are Tower in-process
calls; spawned tasks are joined or aborted/reaped, and native test/check
processes finish with exit 0. There are no active test resources to remove.
The forty-one accepted bindings preserve all previous raw outcomes/reviews,
provider implementation, exact historical binaries/caller/SQL and lockfiles.

Final independent Reviewer `/root/yt176_http_admission_final_review_20261025`
returns **PASS**, zero issues/required followups, with native reviewer/model/high
effort verified. All eight source, forty-three task-evidence and forty-one
accepted bindings remain stable through review; three pre-change hashes match
committed baseline `b42f794`. Mark YT-176 Done only at this bounded local
application-integration scope. Only four documentation status/review-reference
updates follow PASS; code/manifest/example usage remain unchanged. Preserve
all review/native receipts and old failures under
`target/yt176-http-admission-20261025/evidence/`; `closeout.json` records final
source hashes, unchanged inputs and exact Git state. All eight new paths remain
uncommitted/unpushed. The next boundary, if separately selected, is an isolated
new enabled-gate workload with an explicit application cap and failure/latency
expectations. Do not infer that zero-queue early rejection increases successful
burst acceptance or removes provider Busy under every producer/cancellation
pattern. No benchmark or further integration is automatically activated.
