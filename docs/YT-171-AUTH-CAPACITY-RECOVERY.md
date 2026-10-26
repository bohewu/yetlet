# YT-171 — Auth/Session/PostgreSQL staged load and recovery

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded staged-research scope; contract, mapping and final independent Reviewer PASS, zero issues/required followups; exact-owned cleanup PASS. Failed load points remain failed.**

## Authority and preserved baseline

The user accepts the recommendation to add minimal pool-acquisition/query and
Session-error observations, increase the complete Auth/Session/PostgreSQL
workload through250/500/1000/2000requests/s, stop increasing at first pressure
and validate recovery on the same process. Existing opt-in, failure-retention,
no accepted-proof replay, exact-owned cleanup and independent Reviewer gates
continue. No Commit or Push is authorized.

[YT-170](YT-170-AUTH-SESSION-POSTGRES-WORKLOAD.md) remains independently accepted
at its two100/s20s reference points. Its ten uncommitted paths are preserved;
exact accepted source copies and hashes are retained before this work. HEAD
remains the YT-169 commit `d15b94aac64ff68adb2977dee37a7e82098f1843`.
Do not rewrite YT-170's original compile/mapping FAIL or accepted final review.
The new instrumented/profile observations are not a controlled comparison with
its old binary, VU count or warm dataset.

Preserve ADRs [0001](adr/0001-separation-of-concerns.md),
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md). No Core/Auth/Session provider API,
business authorization, SQL, schema, output, security/header/cookie/CSRF,
root/browser/default graph or Admin deployment change is selected.

## Minimal explicit observation

Extend only the optional application fixture with feature
`auth-workload-observation`, which explicitly selects `auth-session-postgres`.
Keep timing/recording code and a loopback-only `/fixture/observations` operator
endpoint behind that feature. Feature-off retains the existing fixture paths
and no timing/observation endpoint. No new crate, exporter, tracing framework,
generic metrics system, provider introspection or production endpoint follows.

Record three fixed aggregate distributions: authoritative Session read elapsed,
SQL pool acquire elapsed and SQLx query elapsed. Fixed atomic microsecond buckets
and count/sum/cumulative-max counters are bounded; no per-request identity,
cookie, CSRF, credential, statement text/value or raw provider error is emitted.
Query elapsed includes protocol/scheduling/decoding; it is not PostgreSQL's
server execution time. Session elapsed includes the selected provider's own
admission/queue/network work; it does not isolate each internal phase.

Separate bounded error categories for pool timeout/other acquisition failure,
query deadline/server statement timeout/other query failure, and Session
busy/queue/dispatch/unavailable/lifecycle/invalid-cookie/record/other outcomes.
Preserve the existing HTTP mapping and no anonymous fallback/retry semantics.
Record optional diagnostics' cost as part of this selected profile.

Use quiescent before/after snapshots outside Auth/SQL work for each new load
and recovery invocation. Histogram count/sum/bucket deltas must reconcile;
report interval upper bounds rather than inventing exact distribution quantiles.
Maxima are cumulative high-water context, not subtractable per-phase maxima.
Missing observations stop later work and remain unverified, not synthesized.
The endpoint binds only with the existing private fixture loopback listener,
returns aggregate non-secret counters with no-store and does not resolve identity
or create provider traffic. It is not a public/framework API.

## Once-only workload and stop semantics

Reuse YT-170's exact logical fixture: two accounts/sixteen documents, immutable
synthetic preverified sign-in assertion, authoritative Valkey Session reads,
fresh PostgreSQL account/tenant permission lookup and tenant-scoped document
lookup, full JSON/escaped Yetlet Page/header oracle. Business requests are
read-only. Reuse old CSRF/tenant/policy/outage/provider/browser qualification
without replay; do not inject another outage, mutate policy or change limits.

One dedicated PostgreSQL and Valkey allocation serves two sequential ladders:
JSON then private SSR. Each ladder starts one fresh application/SQL pool and
eight fresh Sessions, then keeps that original app PID for every point and its
recovery. Stop/drain that app before the other workload starts; no overlapping
app pools. Store credentials only in excluded private0600 files.

| Stage per ladder | Selected behavior |
| --- | --- |
| New admission |125arrivals/s for2s once; blocks further load on pressure |
| New ladder |250,500,1000,2000arrivals/s,20s each, increasing in this order once |
| Pressure | Any unstarted drop, HTTP error/invalid population, incomplete started/completed population or native overall HTTP p95 at least50ms; descriptive study marker, not a consumer SLO |
| Stop | After first pressure point, retain the whole finite invocation and skip its higher points; do not resize, tune, retry or select favorable results |
| Recovery |100arrivals/s for10s once after the last attempted point, on the same PID; no deliberate idle or warmup. Report orchestration/grace/snapshot gaps |

A failed recovery ends remaining workloads and remains failed; an unstarted
mode is not qualified. If no point has pressure,2000/s is only the highest clean tested reference,
not a discovered saturation ceiling. If admission/native/oracle/resource/
deadline/observation integrity fails, stop new load and clean up; never call
an unstarted recovery or partial ladder a PASS. A200body/header mismatch
aborts immediately. Ordinary scheduling/latency/availability pressure triggers
the declared recovery instead of a favorable replay.

Use the accepted k6 constant-arrival-rate engine with fixed128preallocated=max
VUs on every new stage. Eight credentials are shared for read-only concurrent
requests; no measured mutation or same-ID CAS contention is selected. The
larger VU budget is declared before load to reduce the old8VU ceiling; VU slots
or client CPU may still limit results and must be separated from server limits.
Keep HTTP1.1 identity encoding, connection reuse,3s timeout/4s graceful stop and
complete response validation. Old YT-170 modes retain their original settings.
Only inspect new fixed caller plans without HTTP; do not replay old100/s points.

Maximum measured ladder exposure is150000nominal arrivals across both modes,
plus500admission and2000recovery arrivals. Before/after metrics and native
summaries are small control operations. Defer full raw parsing until the app/
services are stopped so parsing does not create a false recovery delay or
change the state between points. Native exits are retained before analysis.

Recovery success means all requests validate, no drops/errors/interruptions and
p95below50ms at the new post-load100/s point. Report all ten1s recovery bins,
first clean bin and first three consecutive clean-bin confirmation, plus tails.
This does not claim return to YT-170's old2.1ms result or instantaneous recovery
from the exact moment of overload. Report clock/quantization/control gaps.

## Fixed profile, budgets and attribution

Keep the accepted single-primary Valkey9.1.2 and PostgreSQL18.6 image digests,
private loopback/Unix-socket topology, shared read-only DB fixture, four min/max
SQLx connections,250ms acquisition/750ms query bounds, statement preparation,
300s Session lifetime and existing4admitted-calls/4waiters/256byte payload.
New namespaces and task-owned containers/files/units are mandatory; no shared
service, data, external IdP, Azure deployment or host tuning is selected.

| Resource | Fixed bound |
| --- | --- |
| Each app | CPUs12–13,2CPU quota,512MiB/0swap,128tasks/512FDs,Nice10,no restart,360s unit/5s stop |
| PostgreSQL | CPUs14–15,2CPU quota,512MiB memory/swap,64pids/1024FDs,64MiB shm,no restart; private ext4 data at most2GiB |
| Valkey | CPU10,1CPU quota,128MiB memory/swap,64pids/1024FDs,64MB noeviction,nonpersistent private tmpfs |
| Load/control | CPUs16–19,4CPU quota,1GiB/0swap,128tasks/512FDs,Nice10,no restart; retain finite approximately1s observation/cost |
| Capture |512MiB per raw output,2GiB total new evidence,at least8GiB initial free space; estimate153000arrivals at8KiB each is about1.17GiB, including margin |
| Time |180s missing pinned-input acquisition,600s new compilation/checks;900s absolute from first service startup, stop new work by720s and reserve180s cleanup; no deadline reset/replacement allocation |

Admit actual host/controller placement, quotas, matching-UID stable container
main-process/cgroup context, private mounts, socket length and transferred input
identities before load. After a complete ladder/recovery/app stop, reuse the existing raw-capture
fsync/fadvise seam before the other mode to release client file-cache pressure.
It does not parse raw data, evict DB pages or delay that ladder's recovery.
Preserve the YT-170 corrections for distinct host versus
controller affinity and exact newly acquired image cleanup on hard failure.
Acquire only missing pinned inputs after contract review; retain cached images
and remove exact newly acquired unused image references without force/prune.

Report full started/completed/valid/status/error/drop/native populations,
all/valid HTTP/full-call quantiles, half-open scheduled-window throughput and
drain, all per-stage observation deltas and admission/recovery separately.
Retain server/account/document/Session operation counts, CPU/charged memory,
CPU throttle/memory/OOM events, sample gaps/cost, client/VU headroom and DB
statistics/logical reconciliation. Cumulative memory peaks are not allocations
or per-stage RSS. No client/backend/physical frequency bottleneck is inferred
solely from one aggregate CPU number.

Distinguish configuration/admission limits, pool wait, query elapsed, provider
errors, generator/VU pressure and app/DB/Valkey CPU evidence. These are this
profile's limits, not Yetlet's universal throughput ceiling; no automatic
permit/pool/cache/security change follows. If observations cannot separate a
cause, report it unresolved and retain the failed point.

## Acceptance and cleanup

Only new feature/counter-classification tests, focused new/changed branch
compile/lint/format, default-graph metadata comparison and the new staged
execution are needed. Reuse the old renderer oracle test and all accepted
lifecycle/load/provider proofs. No repeated old qualification or new validator/
observer framework. One smallest canonical glue correction per distinct seam
is bounded and cannot replay completed load; repeated infrastructure signatures
stop in the same task, preserving original budgets and history.

On every outcome stop/drain/reap app, Session manager, pool, k6 and observer/
control work; remove exact-owned containers/units, newly acquired unused image
references and private data/socket/password/cookie files; verify closed listeners
and absent owned processes. Preserve all raw failures and redacted evidence.
No active test resource may remain.

Independent contract review precedes implementation/acquisition; concrete mapping
review precedes the only startup; final results/failure-retention/cleanup Reviewer
PASS precedes Done. Done denotes completed bounded research, with any failed
load remaining failed. Update report/Backlog/Roadmap/index and sample usage;
all prior uncommitted changes remain preserved. No Commit/Push, production SLO,
Core optimization or further item is activated; YT-096/D1–D6 and YT-020 stay unchanged.


## Concrete implementation before startup

Independent contract Reviewer `/root/yt171_auth_capacity_contract_review_20261025`
returns **PASS, zero issues/required followups** before changes/acquisition.
The existing optional [fixture binary](../examples/axum-workload-comparison/src/bin/auth_postgres.rs)
selects its private [fixed observation module](../examples/axum-workload-comparison/src/bin/auth_postgres/observation.rs)
only with `auth-workload-observation`. Pool/SQL/Session timers and closed error
classes do not change responses, deadlines, permits, queries or authorization.
The operator endpoint is excluded with the feature off.

One new filtered concurrency/bucket/error-classification test passes; the old
renderer oracle test is filtered out. Locked offline observation build, strict
Clippy, feature-off compilation and changed-source format checks pass. A private
copy of the accepted build cache preserves the original YT-170 binary/artifacts.
The sample default normal graph remains byte-identical to accepted YT-170.

The existing [caller](../examples/axum-workload-comparison/auth-postgres-workload.js)
adds only explicit `YT171_STEP` selections and new fixed128VU settings/tags.
Its complete request function, CSP and JSON/HTML oracle bytes match the retained
YT-170 caller; old selections retain8VUs and original rates/durations. All
12new mode/step inspections pass without HTTP; no old selection is replayed.
Pinned image acquisition and concrete run-local plan/ownership/paths are retained
in `target/yt171-auth-capacity-20261025/evidence`. Reuse the corrected canonical
startup, matching-UID observation and exact-owned cleanup seams, with deferred
full raw analysis. Independent mapping Reviewer
`/root/yt171_auth_capacity_mapping_review_20261025` returns **PASS, zero
issues/required followups** before the sole startup. Fourteen concrete input
hashes and twenty-eight retained evidence/source bindings match before startup
and after execution; the accepted YT-170 baseline remains intact.

## Sole execution and retained pressure

The one local allocation completes on 2026-10-25. Native execution exits 0;
cleanup finishes in 152.972s from the original service clock, before the fixed
900s deadline. All ten caller invocations exit 0 and are waited/reaped. A caller
exit 0 means the finite invocation and complete oracle ran; it does not make a
pressure point pass. JSON 500/s and SSR 2000/s remain **load FAIL**.

| Measured point (20s each) | Started / completed | Valid 200 / 503 | Valid in-window req/s | Native overall HTTP p95 ms | Result |
| --- | --- | --- | --- | --- | --- |
| JSON 250/s | 5001 / 5001 | 5001 / 0 | 250.00 | 1.560449 | Clean tested point |
| JSON 500/s | 10001 / 10001 | 10000 / 1 | 499.95 | 1.159143 | FAIL: Session busy |
| SSR 250/s | 5000 / 5000 | 5000 / 0 | 249.85 | 1.646823 | Clean tested point |
| SSR 500/s | 10001 / 10001 | 10001 / 0 | 500.00 | 1.192094 | Clean tested point |
| SSR 1000/s | 20001 / 20001 | 20001 / 0 | 1000.00 | 1.163277 | Clean tested point |
| SSR 2000/s | 40002 / 40002 | 39983 / 19 | 1998.90 | 0.955089 | FAIL: Session busy |

JSON stops after its first pressure at 500/s: one 503 out of 10001 completed
requests (0.009999%). Its 1000/s and 2000/s selections are **not started**, not
qualified or inferred from SSR. SSR has nineteen 503s out of 40002 at 2000/s
(0.047498%). Every 503 matches exactly one Session `busy`; no other Session
error class, pool/query error or business denial occurs in any new phase.

All phases have zero generator drops, interrupted iterations and invalid 200
bodies/headers. The measured population is 90006 started/completed, 89986 valid
200 and twenty 503. Including the two new admissions and two recoveries:
92510 started/completed, 92490 valid 200 and the same twenty 503. No failed
point is repeated or discarded.

Admissions remain separate: JSON/SSR each completes 251 valid 200 at 125/s for
2s, with zero errors/drops and native p95 1.901232/1.843083ms. Both pass the
new admission; neither replays YT-170's accepted 100/s measurement.

All raw/native populations, SHA-256 identities, per-phase histogram deltas,
window/tail counts and separate all/valid timings are retained in
`target/yt171-auth-capacity-20261025/evidence/workload-results.json` and the
`<mode>-<step>-analysis.json`, `-summary.json`, `-native.json` and `.jsonl` files.
Ten raw files total 528991993 bytes; the largest is 229535270 bytes, below the
512MiB per-file bound. No raw parsing occurs between load and recovery;
post-cleanup analysis exits 0 in 8.637s.

The following timings use raw nearest-rank quantiles; native summaries use
interpolation and remain the declared pressure criterion. Full-call values
include the complete response oracle and have client millisecond quantization.
Cells are p95 / p99 in milliseconds, rounded to three decimal places.

| Phase | HTTP all | HTTP valid | Full call all | Full call valid |
| --- | --- | --- | --- | --- |
| json-admission | 1.914 / 3.010 | 1.914 / 3.010 | 3.000 / 5.000 | 3.000 / 5.000 |
| json-load-250 | 1.560 / 1.958 | 1.560 / 1.958 | 2.000 / 3.000 | 2.000 / 3.000 |
| json-load-500 | 1.159 / 1.751 | 1.159 / 1.751 | 2.000 / 3.000 | 2.000 / 3.000 |
| json-recovery | 2.194 / 2.655 | 2.194 / 2.655 | 3.000 / 3.000 | 3.000 / 3.000 |
| ssr-admission | 1.845 / 2.619 | 1.845 / 2.619 | 3.000 / 5.000 | 3.000 / 5.000 |
| ssr-load-250 | 1.647 / 1.859 | 1.647 / 1.859 | 2.000 / 3.000 | 2.000 / 3.000 |
| ssr-load-500 | 1.192 / 1.999 | 1.192 / 1.999 | 2.000 / 3.000 | 2.000 / 3.000 |
| ssr-load-1000 | 1.163 / 2.465 | 1.163 / 2.465 | 2.000 / 3.000 | 2.000 / 3.000 |
| ssr-load-2000 | 0.955 / 3.090 | 0.955 / 3.095 | 1.000 / 3.000 | 1.000 / 3.000 |
| ssr-recovery | 2.526 / 2.820 | 2.526 / 2.820 | 3.000 / 4.000 | 3.000 / 4.000 |

Windows are half-open and based on client `Date.now` milliseconds. SSR 250/s
has 5000 valid total but only 4997 in-window; three quantized completions land
at the end boundary. SSR 2000/s has 39983 valid total, 39978 in-window, five
post-window completions and a 4ms drain. These boundary/tail populations are
retained, not converted to drops or used to inflate window throughput. Per-phase
start-at/end and drain details remain in the analysis records.

## Pool, query and Session observations

All quiescent delta reconciliations pass. Across the ten phases, 92510 Session
reads precede 92490 successful account and 92490 document lookups; failed
Session admissions never reach SQL. The two SQL histograms each contain 184980
attempts, and their count/bucket/error relationships match. Bootstrap adds
sixteen account lookups per app outside phase snapshots; final stopped totals
are 92522 account and 92490 document lookups, zero DB errors/denials and twenty
Session errors. Logical accounts/documents, schema/indexes/constraints compare
equal; there are no business writes or owner policy edits.

Each timing cell below is **mean microseconds / p95 bucket upper bound**.
Bounds describe the recorded histogram interval, not exact p95 values.

| Phase | Pool acquire | SQL query | Session read | Session busy |
| --- | --- | --- | --- | --- |
| json-admission | 4.161 / 100 | 186.884 / 500 | 690.837 / 1000 | 0 |
| json-load-250 | 3.730 / 100 | 145.377 / 500 | 494.054 / 1000 | 0 |
| json-load-500 | 3.351 / 100 | 112.540 / 250 | 321.792 / 500 | 1 |
| json-recovery | 4.513 / 100 | 224.484 / 500 | 728.078 / 1000 | 0 |
| ssr-admission | 3.980 / 100 | 226.751 / 500 | 663.351 / 1000 | 0 |
| ssr-load-250 | 3.689 / 100 | 168.466 / 500 | 527.846 / 1000 | 0 |
| ssr-load-500 | 3.416 / 100 | 112.194 / 250 | 316.939 / 500 | 0 |
| ssr-load-1000 | 2.497 / 100 | 90.154 / 250 | 256.579 / 500 | 0 |
| ssr-load-2000 | 2.056 / 100 | 56.747 / 250 | 170.023 / 500 | 19 |
| ssr-recovery | 4.769 / 100 | 230.056 / 500 | 687.023 / 2000 | 0 |

The reachable failure class is the existing provider admission gate:
[`call`](../crates/yetlet-session-redis/src/store.rs) returns `Busy` when its
fixed eight total admission slots (four active plus bounded waiting) are
occupied. This is authoritative error attribution, not proof of exhausted
physical CPU or a universal 500/s or 2000/s ceiling. SQL pool p95 is bounded by
100us in every phase and SQL errors are zero; no pool-size change is justified
by this evidence. Session elapsed still combines scheduling, admission, queue
and Valkey work; the exact short-burst timing behind the gate remains unresolved.

JSON and SSR are sequential, have different fresh app/session states, and use
a shared warming database. One JSON rejection at 500/s alongside a clean SSR
1000/s point therefore does not establish monotonic capacity or faster SSR.
Likewise, decreasing aggregate latency at higher offered load is not a measured
optimization. No matched randomized comparison, arrival-burst trace, PG internal
execution timing or long-run guarantee is claimed.

## Same-process recovery

JSON preserves PID 3033124 throughout its ladder/recovery; SSR preserves PID
3037026. Neither app restarts, retunes permits, replaces Sessions, reloads its
pool or inserts a recovery warmup. Each 100/s 10s recovery completes 1001 valid
200, including one post-window completion, exactly 1000 in-window and 100 req/s;
zero drops, errors or interruptions. Native HTTP p95/p99 is
2.194152/2.655216ms for JSON and 2.525776/2.819518ms for SSR. Both recoveries
pass the predeclared finite-window criterion.

Every bin below has 100 valid responses, zero invalid responses/drops and p95
below 50ms; p95 is raw nearest-rank milliseconds.

| Recovery seconds | JSON p95 | SSR p95 | Result |
| --- | --- | --- | --- |
| 0–1 | 1.908621 | 1.889997 | Both clean |
| 1–2 | 1.999806 | 2.743777 | Both clean |
| 2–3 | 2.433916 | 2.225137 | Both clean |
| 3–4 | 2.110903 | 2.400055 | Both clean |
| 4–5 | 1.967261 | 2.349280 | Both clean |
| 5–6 | 2.655216 | 2.297939 | Both clean |
| 6–7 | 2.206581 | 2.328992 | Both clean |
| 7–8 | 2.285096 | 2.635107 | Both clean |
| 8–9 | 1.966423 | 2.710596 | Both clean |
| 9–10 | 2.131529 | 2.576851 | Both clean |

The first observed clean bin starts at recovery second 0 and three consecutive
clean bins confirm at second 3 for both. Native-exit-to-next-spawn gaps are
0.258s/0.276s; prior-window-end-to-recovery-anchor gaps are 331ms/388ms for
JSON/SSR (last-completion-to-anchor 331ms/384ms). Grace, snapshots and client
startup are included. These observations do not measure instantaneous recovery
at the first rejection, nor guarantee return to an earlier binary's latency.

## Resource context, cleanup and next boundary

CPU figures below are cgroup CPU-seconds per elapsed second over each sampled
before/after interval, including finite control overhead; quotas remain app 2,
client/control 4, PostgreSQL 2 and Valkey 1 cores.

| Pressure point | App cores | Client/control cores | PostgreSQL cores | Valkey cores |
| --- | --- | --- | --- | --- |
| json-load-500 | 0.221 | 0.857 | 0.128 | 0.068 |
| ssr-load-2000 | 0.396 | 1.525 | 0.205 | 0.113 |

Observed cumulative charged-memory peaks are app 10.832MiB, client/control
519.715MiB, PostgreSQL 85.441MiB and Valkey 11.906MiB. These include cgroup
file/kernel memory and are not Rust allocations or stage RSS. All sampled
CPU-throttling and memory high/max/OOM counters remain zero; maximum task
counts are 3/11/13/8, within the respective bounds. Session/SQL time and
rejection counts, rather than these average CPU values alone, identify the
visible admission failure. Peak k6 `vus` gauges are at most three against 128
fixed slots, but sparse gauges cannot exclude subsecond bursts; zero scheduling
drops and no quota throttling are the stronger limited client observations.

The 152 finite resource samples cost 19.762s in aggregate, at most 0.172s each,
with largest sample-start gap 1.182s. Their cost is part of the selected local
profile; no zero-observer-overhead or host-isolation claim follows. PGDATA peaks
at 48653746 bytes; sampled evidence peaks at 531630437 bytes, within the
original 2GiB budgets. The retained `results-summary.json` holds all per-phase
resource intervals and delta detail; it derives only from the completed records.

Exact-owned cleanup PASS: both app instances stop with Session manager drained
and SQL pool closed; ten k6 processes and the controller/native launcher are
waited/reaped; both containers and original main PIDs are absent. Loopback
listeners close, private data/socket/password/cookie files are removed, and the
newly acquired PostgreSQL image reference is absent. The pre-existing Valkey
image cache is retained. Both transient units have `LoadState=not-found`,
`ActiveState=inactive`, `MainPID=0`. No Azure/shared service/host configuration
was changed. No active test resource remains.

Contract/mapping PASS and final review are separate from failed load results.
Final independent Reviewer
`/root/yt171_auth_capacity_results_final_review_20261025` returns **PASS, zero
issues/required followups**. The twelve source paths and 1264 evidence files
remain hash-identical throughout review; native role/model/effort evidence
matches the resolved reviewer selection. Only documentation status/review
references change afterward. The backlog is Done at bounded staged-research
scope. Fourteen mapped inputs and twenty-eight accepted source/evidence bindings
remain unchanged. New focused checks are reused, accepted YT-170 and earlier passing
proofs are not replayed. Source/records remain uncommitted and unstaged.

The next useful bounded question is Session admission burst/queue behavior
under this full workload, with a separate contract before any permit/provider
change. Keep SQL pool size and Core/API/security semantics fixed unless new
evidence supports a change. No next item or implementation is activated here;
the present result is an independently reviewed staged research reference,
not a consumer SLO, production qualification or physical saturation ceiling.
