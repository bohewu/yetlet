# YT-178 — Isolated HTTP admission=8 comparison

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded cap-eight comparison
scope; final independent Reviewer PASS, zero issues/followups.**

## Authority and frozen comparison

The user selects one minimal isolated HTTP admission **eight** workload after
[YT-177](YT-177-HTTP-ADMISSION-WORKLOAD.md): contract first, keep Session limits,
zero added queue/zero new retry, reuse passed evidence, retain failures, remove
test resources and obtain independent Reviewer PASS. New changes remain
uncommitted/unpushed.

The four YT-177 documents are committed as `140812f`, strict/builtin Commit
Guard PASS, zero findings, tree `251602f8a4a5cbd21d7c026e934b22e81a7de996`; no Push.
Its 12550 selected requests/12452 valid/98 checked HTTP-only 503, zero provider
pressure/SQL errors, complete captures and actual cleanup remain immutable.
Use those **retained cap-four results**, not a newly executed cap-four arm.
YT-174's no-outer-gate data remain secondary historical context. No prior
passing test/build/check/caller-plan/analysis/load is replayed.

Only this report and the Backlog/Roadmap/documentation index change. No product
code, provider/API/Settings/default/dependency/lockfile/instrumentation change.
Keep ADRs [0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md).

## Single new condition and unchanged artifacts

Reuse the exact YT-177 release binary, SHA-256
`b0d5164d113184200dd0cfb35bc6f7ec89ebacb126fad3466bb72b0d64059fa8`, its recorded
`auth-workload-observation,auth-workload-admission` feature selection and private
`--cfg yetlet_session_admission_probe`. Verify source/lock/build-receipt/binary
bindings without rebuilding. Reuse the exact accepted caller and empty k6
configuration, including its full 200 oracle and separate checked-503 metric
`yt177_well_formed_503`; that retained metric name does not select the cap.
Copying immutable inputs is not a new qualification run.

The only selected workload parameter change is startup
`YT176_HTTP_MAX_IN_FLIGHT=8`. Create a fresh owned allocation/ports/namespace,
new app/pool/eight signed Sessions per mode, JSON then SSR, as in YT-177.
Record the explicit clean app environment and binary/source hashes. Session
remains **four active/eight total**, 80ms active wait/400ms per-API deadline,
300s fixture TTL/256-byte payload, unchanged worker permit retention/reconnect/
reconciliation/drain. SQL remains four preconnected connections, 250ms acquire/
750ms query, unchanged prepared statements/schema/fixture. HTTP eight is a
local request-future bound, not eight Session workers or a distributed quota.

| Per-mode phase | Fixed cap-eight exposure, matching the retained cohort shape |
| --- | --- |
| Observe | JSON 500/s × 5s; SSR 2000/s × 5s; fixed 128 VUs |
| Burst | Sixteen VUs, one request each; max duration/grace 4s/4s |
| Recovery | Eight sequential full authenticated reads on the original PID/Store/pool, no retry |
| IDs/oracle | Same synthetic ranges and complete 200/503 body/header/privacy oracles; no cookie/CSRF/identity values in raw capture |

Keep all boundary/tail requests and failures. No extra warmup, gate-off/four
arm, cap/rate/duration adjustment, forced permit hold, SQL lock/fault injection,
second attempt or replacement run/clock if failure is absent or abundant.
Bootstrap creates only the credentials needed by this new condition, without
replaying old lifecycle/tenant/CSRF/outage qualification. No added HTTP queue,
waiter buffer, new deadline or automatic retry is introduced.

## Outcomes, attribution and comparison limits

Preserve the [YT-177 oracle/classification contract](YT-177-HTTP-ADMISSION-WORKLOAD.md#oracle-classification-and-acceptance-meaning).
A checked 503 remains a failed request, separate from valid 200. Unexpected
status/body/headers, native/identity/capture/resource/deadline failure stops new
exposure and cleans up with partial raw/native evidence retained. Phase outputs
are saved before unexpected SQL/denial/Session-category stop checks. Provider
busy/queue timeout are ordinary measured pressure; recovery must validate eight
reads without error or remaining work stays unqualified. No favorable replay.

Adapt the task-local reducer only where task/cap/paths are bound. Preserve its
Session four-active/eight-total assertions. Join every client to one HTTP span;
provider calls remain a subset, not a mandatory call for each outer rejection.
Retain explicit provider busy/queue-timeout outcomes if observed. An HTTP-only
503 has no provider call/worker; classify and attribute it to outer admission
only under the fixed valid GET/source graph, verified feature/cap, full oracle,
zero-overflow capture and phase Session-read/call/SQL/error reconciliation.
The trace has no HTTP gate reason/admit/release event. Unreconciled cases stay
unattributed/failed rather than being silently reassigned to the outer gate.
Do not invent exact gate occupancy, owners, permit lifetime or clock alignment.

Report per phase attempted/completed/valid200/checked503, HTTP-only versus
Session errors, drops/incomplete calls, all/valid and failed-response client/
server timing populations, valid in-window throughput/tails, actual burst start
spread, SQL/Session counters, resources, sampler cost and same-PID recovery.
Keep raw nearest-rank percentiles separate from native interpolated summaries.
Zero observed provider errors is finite evidence, not a guarantee. A sixteen-
request burst can admit more than eight over time as permits are recycled.

The same binary/caller/sampler/limits remove the previous build/oracle-change
confound, but new allocation, sequential host/warming/connection/arrival state
and observer variability remain. Compare retained cap-four results numerically
without claiming randomized causality, statistical significance, monotonic cap
behavior, a physical maximum, production SLO or a universal recommended cap.
A higher HTTP cap may reduce outer rejections while exposing provider pressure;
both must remain visible. No lower latency or lower failure count is presumed.

## Isolation and cleanup budget

Reuse [YT-177's resource/cleanup contract](YT-177-HTTP-ADMISSION-WORKLOAD.md#isolation-resource-and-cleanup-budget)
and corrected controller/launcher. No new build budget is needed. Allow 180s
for missing pinned images, then **one original 600s service clock with 180s
cleanup reserve**, never reset. Use PostgreSQL 18.6/Valkey 9.1.2 pinned identities,
only new loopback/private-socket resources, no shared service, cloud allocation,
host change, package installation or off-host/TLS/ACL/HA qualification.

App: CPUs 12–13/two CPU/512MiB/zero swap/128 tasks/512 FDs/Nice 10/no restart,
180s unit bound. PostgreSQL: CPUs 14–15/two CPU/512MiB/zero swap/64 PIDs/1024
FDs/64MiB shm/network none/private ext4 data. Valkey: CPU 10/one CPU/128MiB/
zero swap/64 PIDs/1024 FDs/loopback/64MB noeviction/no persistence/UID 10001/
owned tmpfs. Load/control: CPUs 16–19/four CPU/1GiB/zero swap/128 tasks/512
FDs/Nice 10. Six transient SQL clients: CPUs 16–19/one CPU/128MiB/zero swap/
64 PIDs/512 FDs/network none/read-only root/private tmpfs/socket/password files,
only before traffic or after both apps stop.

Use zero Docker exec throughout. Host-only proc/cgroup samples retain exact PID/
start binding, explicit unavailable optional fields, original post-sample 1s
cadence and file guards. No task Docker or SQL control operation occurs during
load/recovery. Keep 128MiB per raw file/256MiB total new evidence, at least 8GiB
initial free disk and at most 2GiB private PGDATA. No new observer/validator
framework. One smallest correction per distinct proven harness seam is bounded;
repeated infrastructure signatures stop without replay or replacement evidence.

Every outcome must reap/drain app/k6/controller/owned Session/SQL work; remove
exact-owned service/transient-client containers, units, listeners, private data/
socket/password/cookie files and newly acquired unused image references. Keep
pre-existing caches and all raw/native/review/failure records. Verify actual
absence, including if preparation or review fails before service startup.

## Acceptance boundary

Independent contract review precedes acquisition/adaptation, mapping review
precedes the sole startup and final results Reviewer PASS precedes Done. Task-
local artifacts live under `target/yt178-http-admission-eight-20261025/`; retain
both old and new conditions separately. Reuse current compiled artifact and all
old qualification; new documentation checks cover only this delivery.

Done is bounded policy-behavior comparison, not an error-free load/capacity
qualification or production-default selection. Report actual source/Git state,
reconciled failures/limitations and cleanup. Keep all new documents uncommitted/
unpushed. YT-096 remains In progress with D1–D6 Pending; YT-020 stays Blocked.
No further cap, experiment, optimization, release or publication is automatic.

## Concrete mapping before startup

Contract Reviewer `/root/yt178_admission_eight_contract_review_20261025` returns
PASS, zero issues/followups, with native role/model/high effort verified. All
four contract sources and 73 retained bindings are stable through that review.

`artifact-reuse.json` binds the existing release binary and recorded YT-177 build
receipt without running a build. The copied caller is byte-identical, SHA-256
`9a401efc04e49cf451549ea1767b5e743477eb924682980cfe44fdf057555015`; empty k6
configuration and corrected native launcher are also byte-identical. No caller
syntax/plan check or old test/check/analysis runs.

The new owner/port/namespace plan selects HTTP cap eight. Controller changes
are task labels and the cap assertion; the existing app environment reads that
plan value. Reducer changes are only task identity, cap/launch-value checks and
reported verified cap. Session-specific four-active/eight-owner and fixture
eight-Session assertions remain unchanged. AST checks cover changed task-local
Python glue; `controller-reducer-diff.txt` retains the exact narrow differences.
The copied `build-native.json` is retained historical proof, not a new build
receipt. PostgreSQL is newly acquired and scheduled for exact unused-reference
removal; Valkey's pre-existing cache remains. No services/measurement started.

## Sole measured cap-eight outcome

Contract and mapping reviews PASS before their respective stages, zero issues/
required followups. The sole launch and all four k6 invocations exit 0; ordinary
503 responses remain failed requests, not successful responses or a retry cue.
There are **12549 selected requests, 12475 valid 200 and 74 checked HTTP-only
503**. No provider busy/queue timeout, SQL error, denial, other Session error,
generator drop, incomplete call, invalid 200 or capture overflow occurs. Both
original PIDs/Stores/pools serve eight valid serial recovery reads without retry.

| Phase | Retained cap 4: attempted / valid / failed 503 | New cap 8: attempted / valid / failed 503 |
| --- | --- | --- |
| JSON 500/s × 5s | 2501 / 2496 / 5 | 2500 / 2500 / 0 |
| JSON sixteen-request burst | 16 / 4 / 12 | 16 / 8 / 8 |
| JSON serial recovery | 8 / 8 / 0 | 8 / 8 / 0 |
| SSR 2000/s × 5s | 10001 / 9927 / 74 | 10001 / 9941 / 60 |
| SSR sixteen-request burst | 16 / 9 / 7 | 16 / 10 / 6 |
| SSR serial recovery | 8 / 8 / 0 | 8 / 8 / 0 |
| Total | 12550 / 12452 / 98 | 12549 / 12475 / 74 |

The new JSON observe executor starts 2500 rather than the old 2501 boundary
iterations, with zero drops/incomplete calls. Retain this actual population
difference; do not normalize it away or rerun it. Numerically the new cohort has
**24 fewer failures and 23 more successes from one fewer attempt**. These are
short sequential observations, not a randomized causal estimate. JSON observe
is clean in this finite cohort; SSR observe and both bursts still contain
failures. Do not relabel all load points as error-free.

### Complete joins and conditional classification

All IDs reconcile to **12549 HTTP spans, 12475 successful provider calls and
12475 workers: 37499 events**. Each 200 passes the same full response oracle.
Each of the 74 checked failures passes the unchanged generic/private/CSP 503
oracle with no cookie, redirect, challenge or retry hint. Each HTTP-only 503
has no provider call/worker; there is no invented zero-valued provider timing.

For every phase, Session-read attempts equal captured calls; account/document
queries equal valid200, query histograms equal twice that count, and all error/
denial categories are zero. No hidden pre-call Session failure explains missing
calls. The reused binary/features/cfg/caller/source hashes, actual clean launch
environment cap=8, valid fixed GET fixture, complete captures and counters agree.
Under these conditions the HTTP-only population is attributable to outer
admission before Session/Auth/SQL. This remains a conditional classifier: the
trace alone has no gate-reason/admit/release event and cannot generalize to
arbitrary routes or reconstruct exact slot owners, occupancy or permit times.

New JSON/SSR burst server-entry spreads are **0.280102ms / 1.423659ms**. The
HTTP limit is concurrent futures, not total successes in a sixteen-request
burst: eight JSON versus ten SSR successes are consistent with different
arrival spacing and permit recycling. No exact lifetime is inferred. Session
remains four active/eight total, independently of the HTTP cap eight.

### Timing and acceptance tradeoff

| Phase | Raw nearest-rank HTTP p95, all / valid (ms) | Raw full-call p95, all / valid (ms) |
| --- | --- | --- |
| JSON observe | 1.244038 / 1.244038 | 2 / 2 |
| JSON burst | 1.885702 / 1.885702 | 3 / 3 |
| SSR observe | 0.902209 / 0.857464 | 1 / 1 |
| SSR burst | 2.140189 / 2.140189 | 4 / 4 |

These are raw-request nearest-rank values, **not native k6 interpolated p(95)**.
Full native summaries and raw all/valid/p99/max/in-window/tail populations stay
separate in the evidence. Recovery is a different Python-client population and
is not mixed into k6 percentiles. JSON has 2500 valid in-window completions
(**500.0 valid/s**), none at/after the recorded end; SSR has 9938 in-window
(**1987.6 valid/s**) and three at the recorded end boundary. Integer-client
recorded drain is 0ms in both modes; this does not prove no sub-millisecond
lateness at that boundary.

Compared with cap four, observe HTTP p95 is numerically lower (JSON all
1.316030→1.244038ms; SSR all 0.966173→0.902209ms), but burst p95 is higher
(JSON all/valid 1.041342→1.885702ms; SSR 1.493250→2.140189ms). Burst sample
sizes and success/failure populations differ; this is a tradeoff observation,
not proof that increasing the cap always improves or worsens latency.

For the eight JSON and 66 SSR HTTP-only failures, server-span nearest-rank p95
is 7.513µs / 15.299µs, while client HTTP p95 is 1.100854ms / 7.793825ms.
The SSR failed full-call maximum is **54ms**, versus 27ms in the retained cap-
four cohort. Immediate server-side rejection does not guarantee a short client
tail. Keep native HTTP duration, full-call duration and server spans distinct;
no cross-clock subtraction or unique network/scheduler-cause claim follows.

Within this finite comparison, cap eight accepts more requests and still
shields the provider without observed Session pressure. **74 failures remain**,
and some burst/client tails are longer. The exact same binary/caller avoids
those change confounders, but sequential allocation/host/warming/connection/
arrival and observer variation remain. There is no monotonic-cap, statistical
significance, physical capacity, production SLO or universal default-cap claim.
No cap-four replay, new build or favorable retry is used.

### Resource record and actual removal

Twenty-six host-only samples cost 0.355951s in aggregate, maximum 0.026087s;
maximum sample-start gap is 1.031854s. This is recorded sample-body cost before
append/post-write guards, not total observer CPU or zero overhead. Mandatory
identity/cgroup counters remain valid; optional Valkey exe/FD details stay null/
permission_denied for all 26 samples. Recorded task-native commands and source
guards show zero Docker exec or task Docker/SQL-control overlap with traffic.
All six transient SQL clients finish with exit 0 outside the load windows.

JSON/SSR observe sample-bracket average CPU cores are app 0.253/0.425, client/
control 0.897/1.566, PostgreSQL 0.101/0.178 and Valkey 0.050/0.088. Sampled
cumulative charged-memory peaks are app 13914112 bytes, client 174456832,
PostgreSQL 86269952 and Valkey 6676480. Final trace-flush peaks may be missed;
these figures do not establish a physical bottleneck. No sampled OOM/max or
resource guard fires. Raw k6 files total 75834554 bytes; evidence at summary
is about 93.2MB, within 256MiB. Sampled PGDATA peaks at 48427266 bytes.

Stopped app counters reconcile to **12507 account queries** (12475 selected
valid plus 32 credential-bootstrap reads) and **12475 document queries**, with
zero Session/SQL errors or denials. Account/document rows, schema/indexes/
constraints remain equal; zero business writes or owner policy changes occur.
Both original apps acknowledge Session drain and pool closure.

Canonical cleanup **PASS**, no errors, completes **18.919s** after the one
original service clock starts; native launcher elapsed 19.648s, outer completion
19.729s. Both service containers and all six transient clients, app/k6/controller/
owned worker PIDs, units/listeners, private data/socket/password/cookie files
and newly acquired unused PostgreSQL reference are absent. Pre-existing Valkey
cache remains. Retain expected nonzero absence checks for already removed SQL
clients and stop/reset receipts for auto-unloaded units with verified final
absence/inactivity. No fallback allocation or clock reset occurs.

## Final acceptance boundary

Only four documents change. Product code, Session limits, zero added queue/new
retry, lockfiles/default graph and all 73 retained inputs remain unchanged.
No build, previous test/check/caller-plan/load/analysis is replayed; the copied
build receipt remains historical. The new reducer runs once after cleanup.
New docs checks pass (75 documents/178 backlog items) and diff whitespace passes.
All original failures, new 74 failures and complete native/raw/review/cleanup
receipts remain under the task-local evidence directory.

Final independent Reviewer `/root/yt178_admission_eight_final_review_20261025`
returns **PASS**, zero issues/required followups, with native role/model/high
effort verified. All four source, 350 evidence, 84 mapping and 73 retained
bindings remain stable through review; actual absence checks confirm cleanup.
Mark YT-178 Done at bounded cap-eight comparison scope only. Four documentation
status/review-reference updates follow PASS; all runtime/caller/controller/
reducer/raw data remain unchanged. `closeout.json` records final source hashes,
Git state, preserved failures and actual removal. The four new documents remain
uncommitted/unpushed. Completion is bounded comparison of
policy behavior, not an error-free capacity qualification or a default-cap
selection. A consumer must choose its acceptable failure/latency tradeoff before
separately authorizing a further cap or workload. No extra experiment, tuning,
implementation or release activation is automatic.
