# M28-F2 / YT-282-C1 — Opt-in Web/Axum owned adapter

- **Date:** 2026-11-19.
- **Status:** Bounded public Web adapter completed at source-only C1 scope: 9/9 focused real HTTP tests, 1/1 compile-fail doctest, Clippy/Fmt PASS; independent R2 actual PASS. Full YT-282/F2 remain open.
- **Authority:** ADR0013 Accepted scoped F2 amendment; YT-282-B public std-only
  core is independently reviewed R2 actual PASS. Full YT-282 and F2 open.
- **Package:** `yetlet-hosts` (source-only, `publish = false`). Default features
  are empty. The selected `web-axum` feature explicitly brings in
  `yetlet/server-axum+composition`, Axum and Tokio, while direct Pure Web
  and unrelated Yetlet/default feature graphs remain independent.

## Concrete public boundary

The original `web_axum` adapter exposed the boundary below. This dated record
retains its original API names and execution evidence. Its current
[`HttpWorker`](../crates/yetlet/src/web/worker.rs) and
[command bridge](../crates/yetlet/src/command_bridge.rs) live in separately
enabled `web` and `command-bridge` modules after YT-290/292/293:

- `HttpWorker::start(router)` binds loopback ephemeral TCP and owns one Tokio
  runtime thread plus its original `JoinHandle` and independent graceful
  shutdown one-shot. `start_on` selects an address explicitly; no TLS or
  wildcard/LAN exposure is inferred. The source accepts native Axum `Router`,
  including one from `yetlet::server_axum::App::into_router()`. The caller must
  observe an **actual HTTP 200** on its chosen bounded readiness endpoint
  before marking the worker ready in the Composition Core. Bind/spawn alone
  does not satisfy HTTP readiness.
- `CommandBridge<C,S>` supplies a bounded `CommandIngress<C>`, a single
  nonblocking domain `CommandDrain<C>` and a cloneable read-only `SnapshotReader<S>`.
  The queue close fence serializes concurrent `try_submit` and closure under
  one mutex. Closed, full, disconnected and poisoned are distinct rejections.
  Only the **application domain owner** receives and applies commands;
  accepted HTTP 202 is not evidence of application.
- Only the domain-owned `CommandBridge::publish_snapshot` publishes
  revisions; `SnapshotReader` exposed to HTTP handlers has no write method.
  Copied projections carry a caller-selected generation/revision, support
  latest and exact pinned readback, reject stale or cross-generation
  publications, and evict oldest entries at a bounded capacity. A new generation uses a new
  bridge. No second mutable C1 Simulation or game-specific schema is created.
- `CommandDrain::discard_after_close` verifies **its own** admission gate
  rather than trusting an unrelated bridge's ingress. A pre-closure discard
  is rejected; a closed gate can discard accepted-but-unapplied commands.
- Cross-thread `C: Send + 'static` and `S: Clone + Send + Sync + 'static`
  apply at the bridge edge only. There is no blanket Send/Sync requirement
  on caller-local `Rc` or a universal Host Trait.
- `HttpWorker::request_stop` signals graceful HTTP shutdown through a
  one-shot independent of the bounded command queue. `observe_within`
  returns `WorkerCompletion::Pending` while the **same** original JoinHandle
  exists and `Joined(Result)` only after actual completion. Worker success,
  failure and failure-to-signal are distinct. A previously exited worker may
  reject the stop signal but is still joined by the already-reviewed
  Composition Core. A permanently held handler can prevent hard exit;
  deadlines are cooperative, not preemption.
- **No implicit Drop join.** Dropping the worker's one-shot sender initiates
  graceful shutdown, but dropping the original JoinHandle detaches its thread
  without proving drain or completion. The caller must retain the worker and
  its owner across Pending. Already-owned Web worker startup failures must
  be handled by the caller's root; a bind error acquired no worker. This module does
  not manage borrowed Tokio runtimes.

## Verified real behavior and unverified gates

Focused integration tests exercise real TCP loopback HTTP with original
owned Tokio worker and a distinct application domain owner:

- HTTP readiness probe; typed POST queue admission (202) vs full (429) vs
  closed (503), and no implicit application before a domain step;
- copied latest/exact revision readback, stale/cross-generation rejection,
  bounded eviction, safe discard semantics;
- graceful HTTP drain of a real in-flight held handler, Pending then
  original Join, and socket rebind after worker exit;
- composed Core admission-first stop, original worker completion and
  explicit HTTP bind failure rolling back a previously acquired local owner;
- existing native Pure-Web `App::into_router()` compatibility, no Web wrapper
  requirement for ordinary callers; invalid readiness paths and wrong routes
  must not report Ready.

- rejected or discarded command destructors can reenter their own command
  gate, without holding the mutex during Drop; the snapshot HTTP reader has
  no publication method (compile-fail doctest);
- after a prior worker is joined, a different server may successfully reuse
  its bound port, but the old worker can never claim its 200 as readiness.

Independent Reviewer R1 `wc_job_YXkMeXhzL6007VmQ` terminal exit0 returned
**actual FAIL4** (one P1 gate reentry/deadlock, three P2 read-only publication,
rebound readiness and inaccurate Drop semantics). Original JSON/logs retained.
The four concrete source/document repairs and 2 additional focused tests are
included; **9/9 real Web tests**, **1/1 compile-fail doctest**, scoped Clippy
-D warnings and Rustfmt PASS. Independent R2 `wc_job_JIc_QonqtBfQIfO0`
terminal exit0, **actual JSON PASS**, `issues=[]`, `required_followups=[]`.
The reviewed Web source and tests were not changed after R2; only status
metadata was added.

Actual source-focused test/Clippy/Fmt and dependency receipts are recorded
under ignored `.pipeline-output/m28-f2-yt282c1-20261119/`. Preserve all
negative and earlier F1 records. New Rust changes are not covered by old F1
runtime evidence; no Native/UDP/GPU, external custom-Host YT-283, package
qualification YT-284 or physical D4 verification is claimed. The source-only
crate is not a published release and does not change CLI or existing Web API.

## YT-282-G overall source-gate repair — 2026-11-20

The independent whole-gate Reviewer R1 `wc_job_YCMclXRrlLFOxokV`
terminated exit0 but returned **actual FAIL3/P2**, including two material
Web/Axum issues. The previous bounded C1 R2 PASS remains valid only for its
original source; the actual R1 failed findings and all prior receipts are kept.

- Bounded immutable Snapshot history formerly dropped the evicted application
  value under the write guard. Publishing now retains the evicted value until
  after guard release. A new same-history destructor reentrancy unit test
  checks the lock is available, reads the exact evicted revision and passes;
  no hidden thread or global lock was introduced.
- A stopping Web worker can still have an original JoinHandle Pending while
  draining an in-flight HTTP request after the listener has closed. Its
  readiness probe now rejects once the worker's shutdown sender is consumed,
  so a replacement's 200 cannot qualify the original worker. The existing
  real held-request test checks this denial while the original Join is Pending;
  the handler still drains and the original handle Joins after release.

Current changed-source focused checks: Web library unit **1/1** and real
loopback HTTP **9/9**, combined three-host **8/8**, plus affected external
consumer library **17/17**, binary **7/7** and compile-fail doctests **5/5**.
Scoped Clippy `-D warnings`, Fmt and build exit0. The first Fmt exit1 and
subsequent formatting correction are retained as original repair evidence.
No GUI/GPU, full F1 migration, unrelated native tests or historical reviewers
were rerun. At this R1 repair checkpoint, independent YT-282-G R2 review was pending;
its later material findings and correction are recorded below. Whole YT-282
is not yet accepted and YT-283/284/F2 remain open. The new raw receipts
are ignored under `.pipeline-output/m28-f2-yt282g-20261120/`.

## YT-282-G R2 read-lock findings and repair — 2026-11-20

Independent overall-gate Reviewer R2 `wc_job_xv_go3S7UYT4jyZM` reached terminal
exit0 but returned actual **FAIL2/P2**, including a further Web read-lock defect.
Original R1 FAIL3 and R2 FAIL2 results, logs and source fences remain unchanged.

Although evicted snapshots were already dropped outside the history write lock,
`latest()` and `at_revision()` still called caller-controlled `S::clone()`
under the history read guard. A blocked Clone could prevent foreground
publication and shutdown. `History<S>` now retains `Arc<Published<S>>`
internally; the reader selects and Arc-clones the pinned entry under the read
guard, releases that guard, then performs the application Clone. The two public
read methods still return owned snapshots with exact generation and revision.
Evicted payloads remain dropped outside the writer lock.

A bounded deterministic two-thread test checks both read methods: while a
reader's user Clone is held, the writer publishes revision 1, and the reader
still returns its selected immutable revision 0. The test always releases and
joins both threads. Current Web library unit tests **2/2**, real HTTP **9/9**,
combined three-Host **8/8**, and affected external Consumer library **17/17**,
binary **7/7**, compile-fail doctests **5/5** PASS. Focused Rustfmt, Clippy
`-D warnings` and Build exit0; the original formatting failure was retained.

No new GPU/GUI was launched. The older C1 Reviewer R2 PASS and original
YT-282-G R1/R2 FAILs retain their historical source identity.
At that source checkpoint, YT-282-G R3 review was pending. R3 then returned
actual FAIL1/P2 for the separate external consumer route. Its bounded repair
passed exact-source focused tests, and independent whole-gate R4
`wc_job_iebUf3KFJs9dqYlS` terminal exit0 returned actual PASS with no
issues/followups. **YT-282 source-only gate is now accepted**, not F2 overall;
full F1 YT-283, YT-284 package/compatibility and physical D4 remain open.
No new GPU/GUI run occurred. Raw receipts remain under the ignored
`.pipeline-output/m28-f2-yt282g-20261120/` directory.
