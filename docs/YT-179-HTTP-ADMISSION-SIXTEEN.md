# YT-179 — Isolated HTTP admission=16 comparison

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded cap-sixteen comparison scope after independent
final Reviewer PASS; zero issues and required followups.**

## Authority and retained comparison

The user selects one minimal isolated HTTP admission **sixteen** condition:
contract first, preserve Session limits, zero added queue/zero new retry, reuse
passed evidence, retain failures, remove resources and obtain independent
Reviewer PASS. New changes remain uncommitted/unpushed.

The four [YT-178](YT-178-HTTP-ADMISSION-EIGHT.md) documents are committed as
`3ec72c1`, strict/builtin Commit Guard PASS, zero findings, tree
`aa32ccb4213f9112f1540659ad7fb728cd921d36`; no Push. Retain its cap-eight
12549 selected/12475 valid/74 HTTP-only 503 outcomes, longer burst/client tails,
reviews and actual cleanup. Also retain [YT-177](YT-177-HTTP-ADMISSION-WORKLOAD.md)
cap-four 12550/12452/98 and all earlier results/failures. Neither old condition
is rerun. No prior passing test/build/check/caller-plan/analysis is replayed.

Only this report and Backlog/Roadmap/documentation index change. No product,
provider/API/Settings/default/dependency/lockfile/instrumentation change. Keep
ADRs [0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md).

## One cap-sixteen cohort

Reuse the exact accepted release binary, SHA-256
`b0d5164d113184200dd0cfb35bc6f7ec89ebacb126fad3466bb72b0d64059fa8`, its retained
`auth-workload-observation,auth-workload-admission` features and private
`--cfg yetlet_session_admission_probe`. No rebuild. Reuse the byte-identical
caller SHA-256 `9a401efc04e49cf451549ea1767b5e743477eb924682980cfe44fdf057555015`,
empty k6 configuration, corrected native launcher and host-passive sampler.
The retained `yt177_well_formed_503` metric is intentionally unchanged and does
not select the HTTP cap. Retained build receipts are historical input proof,
not new build executions. Verify hashes and current unchanged runtime source.

Select only `YT176_HTTP_MAX_IN_FLIGHT=16` in each clean startup environment.
Use a fresh exact-owned allocation/ports/namespace, JSON then SSR, one new
app/pool/eight signed Sessions per mode and the existing fixed fixture/SQL.
Session remains **four active/eight total**, 80ms active wait/400ms per-API
caller deadline, 300s fixture TTL/256-byte payload and unchanged worker-owned
permits, reconnect/reconciliation/drain. SQL remains four preconnected
connections, 250ms acquire/750ms query. Do not change Session totals to sixteen.

| Per-mode phase | Fixed new exposure |
| --- | --- |
| Observe | JSON 500/s × 5s; SSR 2000/s × 5s; fixed 128 VUs |
| Burst | Sixteen VUs, one request each; max duration/grace 4s/4s |
| Recovery | Eight sequential full authenticated reads on original PID/Store/pool, no retry |
| IDs/oracle | Same synthetic ranges and complete 200/checked-503 body/header/privacy checks; no credential values in capture |

Keep actual boundary/tail populations, even when they differ by an executor
boundary iteration. No extra warmup, old four/eight arm, rate/cap/duration
adjustment, forced permit hold, SQL lock/fault injection, favorable retry,
second attempt or replacement run/clock. Bootstrap only the credentials needed
for this new condition; do not repeat lifecycle/tenant/CSRF/outage qualification.
No HTTP waiter queue/buffer, added deadline or new automatic retry is installed.
Existing provider/SQL waiting bounds remain; zero added HTTP queue is not a
claim that all downstream waiting disappears.

## Pressure classification and acceptance

Use the same [oracle/classification contract](YT-177-HTTP-ADMISSION-WORKLOAD.md#oracle-classification-and-acceptance-meaning).
A well-formed 503 is still a failed request, separate from valid 200. HTTP
sixteen exceeds the Session total eight: fewer outer rejections can expose
**provider Busy or QueueTimeout**. Record those outcomes explicitly; neither
provider-pressure zero nor a monotonic improvement is an acceptance criterion.
Ordinary outer/provider pressure does not trigger a favorable retry or stop the
fixed cohort. Unexpected native/oracle/identity/capture/deadline/resource/SQL/
denial/other Session-category failure stops new exposure after preserving raw/
phase/native evidence, followed by cleanup. Recovery must validate eight reads
without error; unstarted or unreconciled work remains unqualified.

Adapt task-local task/cap/paths only. Preserve the reducer's **Session**
four-active/eight-admission ownership bounds and eight fixture/recovery records.
Join each selected client to one HTTP span; provider calls remain a subset.
Successful selected reads require a successful call/worker. Explicit Busy and
QueueTimeout calls are provider failures, not outer-gate failures. HTTP-only
503 has no call/worker and is attributed to outer admission only with the fixed
valid GET/source graph, actual feature/cap, full response oracle, complete
zero-overflow capture and phase Session-read/call/SQL/error reconciliation.
The trace has no HTTP gate reason/admit/release marker. Unreconciled outcomes
stay unattributed/failed; do not invent exact gate occupancy, request owners,
permit lifetimes or cross-clock timestamps.

Report per phase attempted/completed/valid200/checked503, **outer versus provider
busy/queue timeout**, drops/incomplete calls, all/valid/failed client and server
timings, in-window valid throughput/tails, actual burst start spread, counters,
resources, sampler cost and same-PID recovery. Preserve raw nearest-rank and
native interpolated percentile definitions, clock boundaries and integer-client
timing uncertainty. Sixteen is a concurrent-future cap, not a promise of sixteen
successful requests in a burst or sixteen backend workers.

Compare retained cap-four/eight numbers with the new cohort without rerunning
or normalizing away any failure/boundary difference. Same binary/caller removes
those change confounders; fresh allocation, sequential host/warming/connection/
arrival and observer variation remain. Do not claim randomized causality,
statistical significance, monotonic cap behavior, physical capacity, production
SLO, error-free load or a universal default cap. An unobserved rejection category
is unobserved, not a reason for extra exposure.

## Isolation and cleanup budget

Reuse the [YT-177 resource/cleanup contract](YT-177-HTTP-ADMISSION-WORKLOAD.md#isolation-resource-and-cleanup-budget)
and corrected controller/launcher. No new build budget or qualification matrix.
Allow 180s for missing pinned PostgreSQL 18.6/Valkey 9.1.2 images, then **one
original 600s service clock with 180s cleanup reserve**, never reset. No shared
service, cloud allocation, host change, package installation or off-host/TLS/
ACL/HA qualification.

App: CPUs 12–13/two CPU/512MiB/zero swap/128 tasks/512 FDs/Nice 10/no restart,
180s unit bound. PostgreSQL: CPUs 14–15/two CPU/512MiB/zero swap/64 PIDs/1024
FDs/64MiB shm/network none/private ext4 data. Valkey: CPU 10/one CPU/128MiB/
zero swap/64 PIDs/1024 FDs/loopback/64MB noeviction/no persistence/UID 10001/
owned tmpfs. Load/control: CPUs 16–19/four CPU/1GiB/zero swap/128 tasks/512
FDs/Nice 10. Six transient SQL clients: CPUs 16–19/one CPU/128MiB/zero swap/
64 PIDs/512 FDs/network none/read-only root/private tmpfs/socket/password files,
only before traffic or after both apps stop.

Use zero Docker exec throughout. Host proc/cgroup-only sampling retains exact
PID/start binding, optional inaccessible fields explicitly unavailable, original
post-sample 1s cadence and file guards. No task Docker or SQL control operation
runs during load/recovery. Keep 128MiB per raw file/256MiB total new evidence,
at least 8GiB initial free disk and at most 2GiB private PGDATA. No new observer/
validator framework. One smallest correction per distinct proven harness seam
is bounded; repeated infrastructure signatures stop without replay/replacement.

Every outcome must reap/drain app/k6/controller/owned Session/SQL work; remove
exact-owned service/transient-client containers, units, listeners, private data/
socket/password/cookie files and newly acquired unused image references. Keep
pre-existing caches and every raw/native/review/failure record. Verify actual
absence even if preparation or review fails before service startup.

## Acceptance boundary

Independent contract review precedes acquisition/adaptation, mapping review
precedes the sole startup, final results Reviewer PASS precedes Done. Artifacts
live under `target/yt179-http-admission-sixteen-20261025/`; keep all conditions
separate. New documentation checks apply to this delivery; all previous
qualification/analysis remains reused, not replayed.

Done is a bounded policy-behavior comparison, not a zero-error capacity gate or
default selection. Report actual changed files/Git state, outer/provider
failures/limitations and verified cleanup. All new documents stay uncommitted/
unpushed. YT-096 remains In progress with D1–D6 Pending; YT-020 stays Blocked.
No further cap, experiment, optimization, release or publication is automatic.

## Concrete mapping prepared

Contract Reviewer `/root/yt179_admission_sixteen_contract_review_20261025`:
**PASS**, zero issues/followups; four document hashes and 94 retained bindings
matched before/after. Native reviewer role/model/effort trace matched the saved
configuration and resolver. Its result remains contract-only.

The task-local plan selects HTTP cap 16. The controller changes task identity
and its cap assertion; the reducer changes only task identity, condition cap,
launch-environment assertion and reported verified cap. Provider ownership,
eight fixtures/recoveries and all original failure guards remain. Caller,
empty k6 configuration and corrected launcher are byte-identical to YT-178;
original YT-177 build receipt/binary are reused. Both scripts parse without
executing. The pinned PostgreSQL image was newly acquired (native pull exit 0,
5.52s); Valkey was pre-existing. Their ownership/acquisition receipt binds
cleanup. Neither service has started. Independent mapping review must pass
before the sole startup.

## Sole measured cap-sixteen outcome

Independent mapping Reviewer
`/root/yt179_admission_sixteen_mapping_review_20261025`: **PASS**, zero issues/
required followups. Four document and 105 mapping hashes, including all 94
retained bindings, match before/after. Native role/model/high-effort evidence
matches the saved configuration/resolver. The sole launcher and all four k6
invocations exit 0; the new reducer runs once after cleanup and exits 0. No
build, previous test/check/caller-plan/load/analysis is replayed.

There are **12552 selected requests, 12540 valid 200 and 12 checked 503**:
**seven HTTP-only outer-admission rejections and five provider Busy failures**.
No provider QueueTimeout, other Session error, SQL error, denial, generator
drop, incomplete call, invalid 200 or capture overflow occurs. Checked 503
remains a failed request. Both original PIDs/Stores/pools validate eight serial
recovery reads without retry.

| Phase | Retained cap 4: attempted / valid / 503 | Retained cap 8: attempted / valid / 503 | New cap 16: attempted / valid / 503 | New 503 outer / Busy / QueueTimeout |
| --- | --- | --- | --- | --- |
| JSON 500/s × 5s | 2501 / 2496 / 5 | 2500 / 2500 / 0 | 2503 / 2503 / 0 | 0 / 0 / 0 |
| JSON sixteen-request burst | 16 / 4 / 12 | 16 / 8 / 8 | 16 / 14 / 2 | 0 / 2 / 0 |
| JSON serial recovery | 8 / 8 / 0 | 8 / 8 / 0 | 8 / 8 / 0 | 0 / 0 / 0 |
| SSR 2000/s × 5s | 10001 / 9927 / 74 | 10001 / 9941 / 60 | 10001 / 9991 / 10 | 7 / 3 / 0 |
| SSR sixteen-request burst | 16 / 9 / 7 | 16 / 10 / 6 | 16 / 16 / 0 | 0 / 0 / 0 |
| SSR serial recovery | 8 / 8 / 0 | 8 / 8 / 0 | 8 / 8 / 0 | 0 / 0 / 0 |
| Total | 12550 / 12452 / 98 | 12549 / 12475 / 74 | 12552 / 12540 / 12 | 7 / 5 / 0 |

Retain the actual 2503 JSON observe iterations, three more than cap eight and
two more than cap four. Zero drops/incomplete calls does not make the boundary
populations identical. JSON records 2499 valid in-window completions
(**499.8 valid/s**), four at/after the recorded end and integer-client drain
6ms; SSR records 9989 (**1997.8 valid/s**), two at/after the end and drain 1ms.
The extra JSON boundary work is not throughput above the configured 500/s.
No replacement run or population normalization is used.

Numerically cap sixteen has **62 fewer checked failures and 65 more successes
from three more attempts than cap eight**. Its failures include downstream
Busy, unlike the all-HTTP-only failures in the retained cap-four/eight cohorts.
This is a finite sequential comparison with different arrival/boundary
populations, not a causal estimate or proof of an optimal/default cap.

### Complete joins and separate provider pressure

The complete capture contains **12552 HTTP spans, 12545 provider calls and
12540 workers: 37637 events**. Successful calls have one worker each; all five
Busy calls have explicit failed-admission intervals with no admitted/acquired
marker and no worker. Every Busy interval has eight guaranteed Session holders
and four guaranteed acquired holders, within the unchanged ownership bounds.
This demonstrates measured provider admission pressure; it does not infer
exact HTTP slot ownership or permit-release timestamps.

Every phase reconciles Session-read attempts to calls, successful SQL account/
document queries to valid200, two SQL query observations per valid200, and
Session Busy categories to the five failed calls. The seven SSR HTTP-only 503s
have no call/worker and no hidden pre-call Session failure in the counters.
Actual feature/cfg/cap=16/source/binary/caller identities, fixed valid GETs,
full 200/private generic CSP 503 oracles and zero-overflow capture agree.
Under these conditions only that seven-request population is conditionally
attributable to the outer gate before Session/Auth/SQL. Do not relabel the five
provider failures as outer rejection or invent zero provider duration for
HTTP-only records. Trace absence of gate reason/admit/release remains a limit.

JSON/SSR burst server-entry spreads are **1.588369ms / 2.448203ms**, versus
0.280102ms / 1.423659ms in retained cap eight. Fourteen versus sixteen new
successes are consistent with different spacing and permit recycling, not a
simultaneous sixteen-worker capacity claim. Session stays four active/eight
total; the downstream queue is still bounded by its existing 80/400ms contract.
No HTTP queue, new retry or provider-limit adjustment is introduced.

### Timing populations and tradeoff

| Phase | Raw nearest-rank HTTP p95, all / valid (ms) | Raw full-call p95, all / valid (ms) | Raw server-span p95, all / valid (µs) |
| --- | --- | --- | --- |
| JSON observe | 1.259762 / 1.259762 | 2 / 2 | 897.842 / 897.842 |
| JSON burst | 1.224517 / 1.224517 | 2 / 2 | 1026.035 / 1026.035 |
| SSR observe | 0.908322 / 0.896751 | 1 / 1 | 523.957 / 524.036 |
| SSR burst | 1.988075 / 1.988075 | 3 / 3 | 921.208 / 921.208 |

These are **raw nearest-rank** percentiles. Native k6 interpolated HTTP p(95)
remains separate (JSON observe/burst 1.2596391/1.1629345ms; SSR
0.908322/1.87812275ms). Recovery uses a separate Python-client population.
All/valid/failed p50/p95/p99/max and in-window/tail populations remain in raw
captures, `timeline-analysis.json` and `results-summary.json`; neither client
epoch nor integer full-call timing is subtracted from the server clock.

| Failed population | n | Server p95 (µs) | Client HTTP p95 (ms) | Client full-call maximum (ms) |
| --- | --- | --- | --- | --- |
| JSON provider Busy | 2 | 15.523 | 0.272082 | 1 |
| SSR provider Busy | 3 | 134.032 | 2.172856 | 4 |
| SSR HTTP-only 503 | 7 | 12.572 | 4.089543 | 5 |

No JSON outer rejection or QueueTimeout is observed; their timing populations
are empty, not zero-valued latency. Raw failed phase/population distributions
are preserved separately. Compared with cap eight, observe all-request p95 is
slightly higher (JSON 1.244038→1.259762ms; SSR 0.902209→0.908322ms), while burst
p95 is lower (JSON 1.885702→1.224517ms; SSR 2.140189→1.988075ms). The new SSR
outer-failure full-call maximum is 5ms versus 54ms previously, with seven versus
66 observations. Different success/failure populations, arrival spread and
fresh sequential host/connection/warming/observer variation prevent statistical,
monotonic-cap or unique network/scheduler-cause conclusions.

**Cap sixteen accepts more requests in this cohort but exposes Session Busy.**
Cap eight previously kept observed provider pressure at zero with more outer
failures. Neither condition establishes error-free capacity, a production SLO,
a universal default or a consumer's acceptable failure/latency budget. Do not
increase the cap or tune Session automatically.

### Resources, failure history and actual removal

Twenty-six host-only samples cost 0.387694s in aggregate, maximum 0.026017s;
maximum sample-start gap 1.029603s. This is sample-body wall time before append/
post-write guards, not total observer CPU or zero overhead. Mandatory PID/start/
cgroup bindings hold; optional Valkey exe/FD fields are unavailable with explicit
permission_denied in all 26 samples. Forty-two task Docker native receipts plus
source guards show **zero Docker exec or task Docker/SQL-control overlap with
traffic/recovery**. All six transient SQL clients exit 0 outside those windows.

JSON/SSR observe sample-bracket average CPU cores are app 0.248/0.429, client/
control 0.887/1.574, PostgreSQL 0.101/0.180 and Valkey 0.048/0.087. Sampled
cumulative charged-memory peaks: app 14245888 bytes, client 179884032,
PostgreSQL 86032384 and Valkey 6758400. Final trace-flush peaks may be missed;
these samples do not identify a physical bottleneck. No sampled memory/OOM or
resource guard fires. Raw k6 files total 75839942 bytes, evidence at summary
about 93.3MB, within 256MiB; sampled PGDATA peaks at 48427266 bytes.

Stopped counters reconcile **12572 account queries** (12540 selected valid plus
32 credential-bootstrap reads), **12540 document queries**, five Session Busy,
zero SQL errors/denials. Logical account/document rows, schema/indexes/constraints
remain equal; no business writes or owner policy edits. Both original apps
acknowledge Session drain and pool closure.

One pre-start receipt-writing harness failure is preserved in
`prestart-harness-failure.json`: the parent read a nonexistent resolver
`enforcement` field. The smallest correction reads the existing
`enforcement_status`; runtime/controller/reducer and immutable bindings do not
change. No service or measurement had started, no reviewer redispatch occurred
and no exposure was repeated. All earlier failures and all 12 new failures stay
retained. There is no runtime, oracle, reducer or cleanup failure.

Canonical cleanup **PASS**, zero errors, completes **18.853s** after the one
original service clock starts; native execution elapsed 19.743s and outer
completion 19.850s. Exact-owned service containers and all six transient SQL
clients, original app/k6/controller/owned worker PIDs, units/listeners, private
data/socket/password/cookie files and the newly acquired unused PostgreSQL
reference are absent. Pre-existing Valkey cache remains. Expected nonzero
absence checks and stop/reset receipts for already unloaded units are retained
with final absence/inactivity verified. No fallback allocation or clock reset.

## Final acceptance boundary

Final independent Reviewer
`/root/yt179_admission_sixteen_final_review_20261025`: **PASS**, zero issues/
required followups. Native reviewer role/model/high-effort trace matches the
saved configuration/resolver. All four document, 352 evidence, 105 mapping
and 94 retained bindings match before/after review. Read-only direct checks
confirm absence of all eight containers, both units/listeners, ten recorded
PIDs, private work and newly acquired PostgreSQL reference; pre-existing Valkey
remains. No test, build, workload or reducer is replayed by review.

Mark YT-179 Done only at this bounded comparison scope. Only this report and
three indices change; product/API/default graph, source hashes, Session limits,
zero added queue/new retry and retained accepted inputs remain unchanged. New
documentation checks pass (75 documents/179 backlog items), as do whitespace
checks. Completion-status/review-reference metadata is updated after PASS;
runtime/caller/controller/reducer/raw/native inputs stay immutable. Local
`closeout.json` records final source hashes, Git state, all failures, reused
proof and actual cleanup. The four changed documents stay unstaged/uncommitted/
unpushed. No extra experiment, optimization, release activation or publication
follows completion. No additional implementation/performance item is activated;
YT-096 D1–D6 remain Pending and YT-020 remains Blocked.
