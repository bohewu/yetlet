# YT-172 — Session admission bursts and queue attribution

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded provider-research scope; contract, corrected mapping and final
independent Reviewer PASS, zero issues/required followups; actual cleanup verified.
All startup/glue failures and 96 measured rejections remain retained.**

## Authority and preserved evidence

The user authorizes research of short Session admission bursts and queue
behavior, contract first and then minimal measurements, preserving current
limits/security semantics, accepted evidence and all failures. Remove every
new owned test resource and obtain independent Reviewer PASS before Done.
New changes must remain uncommitted; no Push is authorized.

The previously authorized twelve YT-170/171 paths are committed as `fc56727`
after strict/builtin Commit Guard PASS, zero findings, exact tree
`5504fd4b0957e94c8a00877b7e38a7cf9a20dc07`. The worktree is clean before this
study. [YT-171](YT-171-AUTH-CAPACITY-RECOVERY.md) and all its original reviews,
failed points and cleanup remain unchanged: one JSON 500/s `busy` and nineteen
SSR 2000/s `busy`, no SQL errors/drops, and two successful same-PID recoveries.
These finite observations are not a universal capacity ceiling.

Preserve ADRs [0001](adr/0001-separation-of-concerns.md),
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md). No Core, provider public API,
Auth/authorization, SQL, HTTP output, cookie/CSRF, browser/default dependency,
production configuration or deployment change is selected.

## Bounded question and measurement boundary

Existing application histograms combine Session admission, active-slot waiting,
Redis client/network/server work and response decoding. They cannot separate
queue time, and sparse VU/resource gauges cannot resolve short bursts.

First analyze only the retained YT-171 JSON 500/s and SSR 2000/s raw captures.
Join each start/completion/status by synthetic probe ID. Report the twenty
rejections' client start clusters and overlapping full-call intervals, with
explicit millisecond/tie uncertainty. Client HTTP spans are not server Session
occupancy or a synchronized provider clock; correlation alone cannot establish
why the admission gate filled. Do not execute old workloads again.

Then conduct one small complementary **provider-read** experiment, not a
replacement HTTP capacity qualification. Use the unchanged real Valkey-backed
Store, its authoritative read-only Lua operation and normal synthetic create
setup. Capture internal acquisition/reply boundaries with private `cfg(test)`
instrumentation and one explicitly ignored research test. Production builds
contain no probe fields/timers/capture, new Cargo feature, dependency or API.

Keep four active operations/eight total admission slots, 80ms queue wait,
400ms caller deadline, existing worker-owned permit retention/reconnect/drain,
256-byte payload limit and 300s absolute fixture TTL. Do not hold permits,
pause Valkey, inject faults, bypass the gate, retry calls, enlarge queues or tune
provider/driver settings. Eight server-generated fixture records are created
before capture and reused read-only; private IDs/state/CSRF never enter reports.

## Once-only cases and attribution

| Case | Fixed selection |
| --- | --- |
| Shapes | Barrier-released simultaneous wave; nominal 1ms spacing within a wave |
| Sizes | 4, 8, 12 and 16 concurrent tasks, for both shapes |
| Exposure | Eight predeclared rounds per case; rotate the eight-case order by round, no selected retries or extra warmup |
| Total | 640 measured reads, then eight sequential recovery reads on the original Store/process |
| Native scheduler | Tokio two-worker runtime on CPUs 12–13; record actual starts/spacing, do not assume timer precision or simultaneous dispatch |
| Observation | Monotonic per-call start, successful admission, active-acquisition start/end, reply-return time and closed outcome; bounded 1024 records, no identity or inputs |

The acquisition-await span includes scheduling and its fast path; a sampled
zero-active-permit count is a racy hint of contention, not an atomic joint
occupancy snapshot. Post-acquire reply elapsed includes worker scheduling,
redis-rs multiplexing, transport and Valkey work. It is not server execution
time or exact worker permit-release time. Retain capture/observer cost and
test-build scope; do not compare these timings directly with the old HTTP
binary's latency or claim the exact historical burst was reproduced.

Each case reports attempted/completed/validated/error categories, actual wave
start spread, acquisition/reply/total distributions and all failures. `Busy`
and `QueueTimeout` remain ordinary observed pressure; do not retry them or turn
them into successful reads. Validate every successful snapshot against its
synthetic expected state/revision/valid CSRF shape without logging the values.
Unexpected provider/record/deadline/native/capture/resource failure stops new
waves and proceeds to cleanup; preserve partial results and unstarted cases.
Recovery requires eight valid reads with zero errors; report failure honestly.
An absent timeout or rejection is unobserved, not a qualified timeout guarantee.

No fairness guarantee, long-running SLO, full-HTTP capacity, physical bottleneck
or permit-change recommendation follows automatically. If precise historical
causation remains unresolved, say so and propose a separate bounded question.

## Isolation, budgets and acceptance

Reuse the accepted Valkey 9.1.2 digest, with one new exact-owned loopback-only
container/namespace, CPU 10, one CPU, 128MiB memory with zero swap, 64 PIDs,
1024 FDs, 64MB noeviction, no persistence/restart, read-only root/private tmpfs.
No PostgreSQL, HTTP server, cloud resource or shared service is needed.
Keep IDs only in process memory and redact all raw errors/inputs.

The one native test process uses CPUs 12–13, two CPU quota, 512MiB/zero swap,
128 tasks/512 FDs, Nice 10, no restart and a 90s hard unit bound. Before/after
native resource/context snapshots describe this finite probe, not per-request
resource attribution. Keep new output below 64MiB, at least 1GiB free disk,
180s acquisition and 600s new build/check bounds. From first container startup,
use one 270s absolute budget with a 90s cleanup reserve; never reset it or
allocate a replacement run. A failed native invocation is not replayed.

Only new probe compilation/lint/format and this ignored test need execution.
Use locked/offline Cargo with the existing cache. Reuse all prior provider,
Auth/tenant/CSRF/outage/cancellation/drain/browser/CSP qualification and old
load points. No new validator, harness framework or operational qualification
workstream. One smallest canonical glue correction per distinct seam is bounded;
a repeated infrastructure signature stops without replacement evidence.

Stop/reap the test and redis-rs runtime/owned workers; drain the original Store,
remove the exact new container/unit/tmpfs and verify absent owned PIDs/listeners.
Remove only newly acquired unused image references; retain pre-existing caches.
On every outcome preserve raw failures and redacted records. Resource cleanup
is required even if compilation, the test, parsing or review fails.

Independent contract review precedes instrumentation/acquisition; concrete
mapping review precedes the sole service/test startup. Final results Reviewer
PASS precedes Done and reporting. Update Backlog/Roadmap/index/provider usage
with actual evidence. YT-096/D1–D6 and YT-020 remain unchanged; no new-change
Commit/Push or further implementation is activated.

## Concrete implementation before startup

Independent contract Reviewer
`/root/yt172_session_admission_contract_review_20261025` returns **PASS, zero
issues/required followups** before implementation. The only production-source
edit is private `cfg(test)` hooks in
[`store.rs`](../crates/yetlet-session-redis/src/store.rs); all observation state,
monotonic clocks and the ignored
[research module](../crates/yetlet-session-redis/src/store/admission_probe.rs)
are excluded from normal builds. Existing timeout/error propagation is preserved
without changing awaits, admission/worker ownership or Lua/security semantics.
Normal provider compilation and new test-binary compilation/strict Clippy pass
with locked/offline Cargo. No test case or service has run yet.

The concrete probe uses eight single-read recovery waves after its 64 measured
waves. It stores at most 1024 non-secret records and measures aggregate/max
append-lock/copy duration separately; this partial observation cost excludes
clock/atomic/branch cost and is not a zero-overhead claim. Actual provider
acquisition and reply spans, round rotation and validation are retained. Every
wave waits for the unchanged four/eight slots to become quiescent before the
next; no artificial permit hold, delay, fault or old test is used.

Retained-event analysis reads the two old raw files once, verifies their accepted
SHA-256/populations and preserves all twenty errors. The JSON rejection shares
an 8-start millisecond; eleven SSR rejections occur in the 37–38ms start cluster.
Other events have different overlap bounds, so historical causation is still
unresolved. Full records are in
`target/yt172-session-admission-20261025/evidence/retained-burst-analysis.json`.
A native cgroup wrapper admits CPU/memory/FD/nice placement before the one new
Rust test, retains before/after charged resource data and waits/reaps its child.
The task-local execution seam removes its exact container/unit on every outcome;
no PostgreSQL/HTTP service or runtime feature is added. Mapping review precedes
startup; native outcomes and final review remain pending.

## Startup and cleanup history retained

The initial mapping Reviewer finds two P2 glue defects: readiness/unit commands
could consume cleanup reserve, and a failed container stop could skip removal.
Both are corrected in the same canonical seam before any startup; the same
independent Reviewer then returns PASS, zero issues/required followups. The
original FAIL, source and manifest remain in the evidence directory.

The first container startup exits 1 before any Rust test or unit exists. Its
image-default root user differs from the accepted YT-171 UID/tmpfs setup;
container stderr is not retained after removal, so that exact entrypoint error
is not claimed as proven. Cleanup passes in 0.879s. One smallest startup payload
correction restores accepted UID 10001:10001 and owned tmpfs permissions. The
independent mapping followup returns PASS before the first actual measurement.
This is the same task/plan/port/container name and original 270s clock, with
43.496s work allowance still remaining immediately before resume. No old load,
probe or passing check is replayed, and no clock/window/run is replaced.
The failed startup's inspect/native/cleanup files remain under
`evidence/initial-startup-failure/`; two container-start attempts and exactly one
Rust probe invocation are recorded, rather than described as a first-try success.

The Rust test exits 0 (one passed, one old test filtered), completes in 0.37s
(native child 0.415s) and drains normally with zero forced worker reaps. The
outer execution returns 1 because `systemctl stop` returns 5 after the transient
unit has already unloaded. Its original `cleanup.json` remains **FAIL**, with
all error/native receipts preserved. A direct read-only cleanup confirmation
finds `LoadState=not-found`, `ActiveState=inactive`, `MainPID=0`; the wrapper,
Rust child and container main PIDs are absent, the container is absent and the
listener can be rebound. This closes the resource-removal requirement without
rerunning the successful probe or changing its source. The 138.212s cleanup
receipt remains within the original deadline. No raw failure is rewritten as
native success; the harmless already-unloaded-unit classification remains a
known limitation of the task-local cleanup wrapper.

## Measured burst and acquisition behavior

All 648 attempts have one complete record and matching closed outcome, with
zero capture overflow: 552 successful validated reads and 96 `busy`, no queue
or caller timeout/unavailable/record error. The measured 640 reads contain 544
successes and 96 rejections; eight sequential recovery reads all succeed on the
original Store/process. All 72 waves finish quiescent at four active/eight total
free slots. Running worker peak is four and running workers after drain is zero.
Rejected calls are not reclassified as successful reads.

Every row covers eight predeclared rotated waves. Acquisition/reply timings
include admitted calls only (all successful here). Total columns retain both
all calls and successful calls so fast rejections cannot bias the valid result.
Timings are monotonic raw nearest-rank p95 in microseconds; the acquisition span
includes scheduling and the fast path, not only time blocked on the semaphore.

| Shape / size | Attempts | Valid | Busy | Acquire p95 | Reply p95 | Total all p95 | Total valid p95 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Barrier / 4 | 32 | 32 | 0 | 1.698 | 686.378 | 689.031 | 689.031 |
| Barrier / 8 | 64 | 64 | 0 | 627.839 | 722.585 | 1148.521 | 1148.521 |
| Barrier / 12 | 96 | 64 | 32 | 665.292 | 797.183 | 1121.241 | 1138.838 |
| Barrier / 16 | 128 | 64 | 64 | 628.068 | 742.301 | 1099.231 | 1147.529 |
| Nominal 1ms / 4 | 32 | 32 | 0 | 1.536 | 473.876 | 476.063 | 476.063 |
| Nominal 1ms / 8 | 64 | 64 | 0 | 1.359 | 394.280 | 395.799 | 395.799 |
| Nominal 1ms / 12 | 96 | 96 | 0 | 1.518 | 454.726 | 456.581 | 456.581 |
| Nominal 1ms / 16 | 128 | 128 | 0 | 1.323 | 433.401 | 435.479 | 435.479 |

Barrier waves of four have no observed zero-active-slot hint. At eight, twelve
and sixteen, each case records 32 zero-active-slot hints (four per wave); all
such requests eventually acquire a slot, with maximum recorded acquisition
await below 803us. Twelve/sixteen barrier waves each admit eight reads and
reject four/eight in every round. These observed counts match the existing
bounded admission mechanism; they do not demonstrate 80ms queue expiry,
fairness, cancellation or a need to increase capacity. Prior qualification of
those other behaviors remains accepted and is not rerun.

The nominally spaced cases have zero `busy` and zero zero-active-slot hints,
but are not perfectly periodic: actual adjacent starts range from 4.531us to
2421.848us across the spaced cases, and their per-wave spreads range from
3.381ms (size four) to 16.606ms (size sixteen). Barrier spreads range from
18.080us to 448.640us. Timer coalescing/runtime scheduling are retained; no
exact 1ms dispatch or synchronized launch is assumed. All wave spans and
all/valid/acquisition/reply distributions are in `probe-analysis.json`.

The eight sequential recovery reads have acquisition/reply/total p95
0.383/68.014/68.504us, zero errors and unchanged original Store. This tiny
provider-only observation is not a full HTTP recovery SLO or a comparison with
YT-171's 100/s recovery latency.

## Historical HTTP correlation and its limits

The retained JSON rejection is synthetic probe 7666, client start offset
15334ms. Eight requests start in that millisecond; its full call is 1ms and
HTTP duration 0.145062ms. The quantized start bucket permits between one and
eight other overlapping client full calls. It does not locate the provider's
instantaneous occupancy.

Eleven of SSR's nineteen rejected calls start at offsets 37–38ms. Other
rejections start at 13143, 15866, 16710, 16985, 18112, 18117 (two) and 18118ms.
The entire SSR phase peaks at nineteen starts per millisecond, while the JSON
phase peaks at eight. One SSR rejected request at 18112ms has only zero to
three possible/definite other full calls at its start bucket, despite a later
provider rejection during its 6ms client full call. This counterexample prevents
attributing every rejection to high occupancy at client-start time.

The client overlap bounds exclude the rejected call itself, treat equal-time
boundaries conservatively and use millisecond buckets. Client spans include
transport/oracle work and are not provider read/worker lifetimes. Iteration/rate
nominal offsets include JS/VU/startup effects and are not isolated generator
lateness. The exact twenty historical per-event rows, raw SHA-256/population
reconciliation and uncertainty notes remain in `retained-burst-analysis.json`.
The new direct provider experiment supports burst-sensitive gate behavior;
it does not prove the precise arrival/queue sequence of those old HTTP errors.

## Observation cost, resources and acceptance boundary

Bounded record append/lock/copy costs sum to 271537ns, mean 419ns and maximum
14226ns. This excludes monotonic clock/atomic/branch cost and can perturb
scheduling in this test build. No zero-cost observer, production latency or
causal performance improvement is claimed. Production has no probe state,
capture, timer or new feature/dependency/API; only equivalent private result
propagation and test hooks change in the source. Native compile/lint evidence
and final review cover that boundary; old passing tests are not replayed.

The admitted probe unit uses CPUs 12–13, two CPU quota, 512MiB/zero swap,
128 tasks, 512 FDs and Nice 10. Its charged memory peak including wrapper is
9.266MiB; Valkey's charged peak is 8.836MiB under its original one-CPU/128MiB
bound. Both sampled cgroups have zero CPU throttling and memory high/max/OOM
counters. The wrapper interval uses 88685us of cgroup CPU; Valkey's observed
interval uses 81741us. These short control-inclusive observations are not
per-request allocations or a physical saturation proof. Evidence is below
1MiB, well below the original 64MiB bound.

Actual cleanup is verified with the original failure receipt retained: one
native Rust invocation and wrapper waited/reaped, Store/driver drained, no
container/unit/listener/owned PID left, private IDs only in reaped memory and
cached Valkey image retained. No newly acquired image, disk credential/data
store, PostgreSQL/HTTP server, cloud resource or shared-service change exists.
The original YT-171 source/evidence and twenty failures remain intact; the
old Store source is retained exactly before the scoped cfg(test) diff.

The recommendation is to **keep the existing limits**. Before a separately
selected full-HTTP experiment or any provider change, measure request shape
and correlate provider boundaries at that layer if precise historical
causation is needed. Average req/s alone cannot define admission safety; these
small controlled cases do not set a deployment SLO or authorize pacing/retries
in the framework. No next item or implementation is activated here.

Final independent Reviewer
`/root/yt172_session_admission_results_final_review_20261025` returns **PASS,
zero issues/required followups**. Seven source files, 142 evidence files and eight
mapped inputs remain hash-identical throughout final review; native reviewer
role/model/effort match the resolved configuration. Only documentation status
and review references change afterward. The backlog is Done at bounded
provider-research scope; failed calls and raw startup/cleanup flags remain as
recorded. All new changes remain unstaged/uncommitted; YT-096/D1–D6 and YT-020
are unchanged.
