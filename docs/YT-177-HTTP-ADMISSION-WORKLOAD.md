# YT-177 — Isolated HTTP admission=4 Auth workload

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded enabled-gate policy-
behavior scope; final independent Reviewer PASS, zero issues/followups.**

## Authority and preserved baseline

The user selects one minimal isolated workload with the
[YT-176](YT-176-HTTP-ADMISSION.md) HTTP admission cap set to **four**. Establish
the contract first, retain Session limits/zero added queue/no new automatic
retry, reuse passed evidence, preserve failures, remove test resources and
obtain independent Reviewer PASS. New changes remain uncommitted/unpushed.

The eight YT-176 paths are committed as `d057dca`, strict/builtin Commit Guard
PASS, zero findings, tree `9d34964b22561dc61aa90de14fdccefc0329e12a`; no Push.
Preserve its six new test results, feature-off check, reviews and limits without
rerunning them. Retain [YT-171](YT-171-AUTH-CAPACITY-RECOVERY.md) through
[YT-175](YT-175-SESSION-BURST-OVERLOAD-POLICY.md), all failures and exact cleanup
records. YT-174 is a **historical control**, not a new gate-off arm.

No product/runtime/route/provider/API/dependency/default/Settings change is
selected. Use existing native Axum/Tower composition under ADRs
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md). Only this report and the three
planning/index documents change; task-local build/caller/controller/reducer
adaptations and raw evidence stay below the excluded
`target/yt177-http-admission-workload-20261025/` directory.

## One enabled-gate condition

Build the committed Auth reference once in release mode, offline/locked, with
`auth-workload-observation,auth-workload-admission` and the already established
private `--cfg yetlet_session_admission_probe`. This is a new feature combination,
not a replay of YT-176 tests or old qualification. Capture the source/lock/tool/
feature/binary hashes and native build exit. Keep the normal/default graph and
accepted historical artifacts unchanged. No new instrumentation is implemented.

Each app starts in an empty explicit environment with
`YT176_HTTP_MAX_IN_FLIGHT=4`; retain that non-secret launch receipt. Use the same
synthetic account/tenant fixture, prepared SQL, eight signed Sessions per mode,
three-second client bound and complete successful-response oracle. Credential
bootstrap is finite setup for the new condition, not replayed lifecycle proof.

| Per-mode phase | Fixed new exposure |
| --- | --- |
| Order | JSON then SSR, fresh app/pool/eight Sessions per mode, shared dedicated PostgreSQL/Valkey allocation |
| Observe | JSON 500/s for 5s; SSR 2000/s for 5s; fixed 128 VUs |
| Finite burst | Sixteen VUs, one request each, max duration/grace 4s/4s; retain actual start spread |
| Recovery | Eight sequential full authenticated reads on the original PID/Store/pool, without retry |
| IDs | Existing observe 1+, burst 20001+, recovery 30001–30008; no identity/cookie values in capture |

The finite cohort matches YT-174's shapes so it does not change prehistory before
SSR. It is a new condition with the real gate enabled. Keep all boundary/tail
requests and invalid/failed responses. No gate-off replay, extra warmup, duration
extension, cap/rate adjustment, forced slot hold, lock/fault injection, second
attempt or replacement clock/allocation if rejection is absent or abundant.
Keep HTTP admission four; Session four active/eight total, 80ms active wait and
400ms per-API deadline; SQL pool four, 250ms acquire/750ms query; original TTL,
payload, authoritative identity/tenant/permission/CSRF and worker-retention bounds.
No HTTP buffer, added wait queue or new automatic retry is installed.

## Oracle, classification and acceptance meaning

Copy the accepted caller into this task's evidence directory, preserving its
scheduling, IDs and full 200 body/header/UTF-8 oracle. Add only the new pressure
response oracle: a 503 must be HTTP/1.1, exact generic body/content type/length,
no-store, Cookie Vary, nosniff and strict CSP, with no Set-Cookie, Location,
authentication challenge, Retry-After, encoding or server header. Count
**well-formed 503 separately from valid 200**; a checked rejection remains a
failed request. Unexpected status/body/headers abort native exposure, preserve
partial raw/native results and trigger cleanup. Do not log response credentials
or claim that a 503 proves a write was rolled back.

Use complete existing HTTP/provider/worker captures and phase observation
counters, adapting only the task-local reducer. Join every selected client ID
to one HTTP span. Successful selected reads require a successful Session call
and worker. Provider busy has its explicit failed-call record. HTTP-only 503
has **no provider call**; do not force it into the old every-HTTP-has-a-call join.

The trace has no gate-specific reason/admit/release timestamp. Classify its
observable category as `http_only_503`; attribution to outer admission is
conditional on the fixed valid GET workload, exact committed route graph,
verified feature/cap, complete zero-overflow capture, response oracle and
reconciliation. In particular, Session-read attempts must equal captured calls,
with no hidden pre-call Session failure, SQL error, denial or finalizer failure
explaining the response. Otherwise retain it as unattributed/failed evidence,
not proof of an outer-gate rejection. Do not invent exact gate occupancy,
permit lifetimes, request owners or cross-clock timestamps from HTTP span ends.

Report per phase attempted/completed/valid200/checked503/HTTP-only503/provider
busy or queue-timeout/other errors, generator drops/incomplete calls, all/valid
client/server timing populations, valid in-window throughput/tails, SQL and
Session counters, resources and recovery. Keep nearest-rank raw percentiles
separate from native interpolated summaries. Compare retained YT-174 data with
explicit new-build/sequence/warming/arrival/observer and new-503-oracle costs;
no randomized causal improvement, universal capacity, zero-Busy or SLO claim.

Ordinary well-formed overload remains a measured failure, not an execution
abort or a reason to retry. Native/oracle/identity/capture/deadline/resource
failure stops further phases. After a phase, unexpected provider/SQL/denial
categories stop new exposure; preserve the phase's raw/summary first. Recovery
must validate eight reads without error or remaining work is unqualified.
A missing rejection condition is unobserved, not grounds for an extra run.
Done requires the bounded evidence/reconciliation/cleanup and final review;
failed requests and uncertainty remain explicit even if the research is Done.

## Isolation, resource and cleanup budget

Reuse YT-174's pinned PostgreSQL 18.6/Valkey 9.1.2 identities and topology. No
shared service, off-host TLS/ACL/HA qualification, host setting or installation.
One PostgreSQL container uses CPUs 14–15/two CPU/512MiB/zero swap/64 PIDs/1024
FDs/64MiB shm, network none/private Unix socket and ext4 data. Valkey uses CPU
10/one CPU/128MiB/zero swap/64 PIDs/1024 FDs, loopback only, 64MB noeviction,
no persistence/restart, UID 10001 and owned tmpfs. App CPUs 12–13/two CPU/
512MiB/zero swap/128 tasks/512 FDs/Nice 10/no restart; load/control CPUs 16–19/
four CPU/1GiB/zero swap/128 tasks/512 FDs/Nice 10. App runtime unit bound 180s.

Use the corrected YT-174 native controller/launcher cleanup seams. Six short-lived
owner-labeled SQL clients use CPUs 16–19/one CPU/128MiB/zero swap/64 PIDs/512
FDs, network none/read-only root/private tmpfs and private socket/password-file
mounts; they run only before load or after both apps stop. The entire task uses
zero Docker exec. Sampling reads host proc/cgroup only, with PID/start binding,
explicit optional cross-UID exe/FD unavailability and the existing post-sample
one-second cadence. No Docker/SQL control operation runs during offered traffic.

Allow 600s for the new release build (two jobs, CPUs 12–13, Nice 10), 180s for
missing pinned-image acquisition, then **one original 600s service clock with
180s cleanup reserve**, no reset. Keep 128MiB per raw file/256MiB total new
evidence, at least 8GiB initial free disk and at most 2GiB private PGDATA. Record
sampler cost and native deadlines. One smallest correction per distinct proven
harness seam is allowed; repeated infrastructure signatures stop without a new
run or replaying completed phases. No validation framework is created.

On every outcome stop/reap apps/k6/controller/owned workers, drain Session and
SQL, remove exact-owned service/transient-client containers/units/listeners,
private data/socket/password/cookie files and newly acquired unused image
references. Preserve pre-existing caches. Verify actual absence and retain every
failure/native receipt. Build/acquisition before startup must also leave no
active task resource; cleanup is required if preparation or review fails.

Independent contract review precedes build/acquisition, concrete mapping review
precedes the sole startup and final independent Reviewer PASS precedes Done.
No old test/build/caller-plan/browser/security/provider/lifecycle/load result is
rerun. YT-096 remains In progress with D1–D6 Pending; YT-020 stays Blocked.
No new-change Commit/Push or further workload is automatically activated.

## Concrete mapping before startup

Contract Reviewer `/root/yt177_admission_workload_contract_review_20261025`
returns PASS, zero issues/followups, with native role/model/high effort verified.
The new release build succeeds once in 63.469s, using two jobs/CPUs 12–13/Nice
10 and the exact selected features/cfg. Its SHA-256 is
`b0d5164d113184200dd0cfb35bc6f7ec89ebacb126fad3466bb72b0d64059fa8`.
No YT-176 or earlier test/check/build is replayed.

Task-local `execute.py`/`launch.py` preserve the corrected YT-174 host-only sampler,
PID/start binding, original deadlines and independent native cleanup fallbacks.
Only ownership/paths, new binary/caller, cap=4 app environment and response-class/
post-phase guards change. Every phase records valid200 and checked503 separately
before enforcing unexpected-backend stop rules. The copied caller preserves all
old scheduling/200 oracles, adds one closed checked-503 counter per request and
aborts unexpected responses without logging credentials. Its new syntax checks
pass; no old caller-plan inspection runs.

`analyze_workload.py` adapts the retained reducer for HTTP-only rows, with null
provider/worker fields, complete ID/metric/counter reconciliation, preserved raw
nearest-rank/native summaries and explicitly conditional attribution. It checks
the actual build/launch/cap/source receipt rather than creating gate timestamps.
It runs only after the sole complete workload and verified cleanup.

`enabled-condition.json`, exact owner/port plan, native build/acquisition records
and immutable source/input hashes are retained in the same evidence directory.
The newly downloaded PostgreSQL reference is task-owned and will be removed;
Valkey's pre-existing cache is preserved. No runtime services have started.

## Sole measured outcome

Contract and concrete mapping reviews both PASS with zero issues/required
followups before their selected stages. All source/input bindings remain stable.
The sole native launch and all four k6 invocations exit 0. These native exits
mean the finite experiment/oracles completed, **not that every request succeeded**.
There are **12550 selected requests, 12452 valid 200 and 98 checked 503**.
All four load/burst phases retain request failures; none is relabeled a clean
load point. No rate/cap change, retry, extra exposure or replacement clock occurs.

| New phase | Attempted/completed | Valid 200 | Checked 503 | HTTP-only 503 | Session busy / queue timeout | Drops / incomplete |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| JSON 500/s × 5s | 2501 | 2496 | 5 | 5 | 0 / 0 | 0 / 0 |
| JSON sixteen-request burst | 16 | 4 | 12 | 12 | 0 / 0 | 0 / 0 |
| JSON serial recovery | 8 | 8 | 0 | 0 | 0 / 0 | 0 / 0 |
| SSR 2000/s × 5s | 10001 | 9927 | 74 | 74 | 0 / 0 | 0 / 0 |
| SSR sixteen-request burst | 16 | 9 | 7 | 7 | 0 / 0 | 0 / 0 |
| SSR serial recovery | 8 | 8 | 0 | 0 | 0 / 0 | 0 / 0 |

All successful responses pass the unchanged complete 200 oracle. Every 503
passes the additional exact-body/private-header oracle, without cookies,
redirects, challenge or retry hint. There is no unexpected status, invalid 200,
SQL error, authorization denial, other Session error or capture overflow.
Both original PIDs/Stores/pools serve eight valid post-load reads without retry.

### Reconciliation and conditional gate attribution

All selected IDs join exactly one HTTP span: **12550 HTTP, 12452 provider calls
and 12452 workers, 37454 events**. All calls succeed; their 200 responses have
one worker each. Exactly 98 checked 503 rows have no provider call or worker,
with null provider fields rather than fabricated timing. Per phase, recorded
Session-read attempts equal captured calls; account/document queries equal the
valid200 count, query histograms equal twice that count, and error/denial
categories are zero. No hidden pre-call Session failure explains a missing call.

The compiled feature/cfg/binary hashes, startup cap=4 explicit environment,
accepted router/valid-GET source graph, known eight-Session fixture, complete
zero-overflow trace and checked response/counter populations agree. Under those
fixed conditions the HTTP-only 503 population is attributable to the outer
admission boundary, before Session/Auth/SQL work. The trace alone has no gate
reason marker; retain the conditional basis rather than generalizing this
classifier to arbitrary routes, inputs or future middleware. It does not measure
exact gate admission/release instants, occupancy peaks or individual slot owners.

JSON burst server entries span **0.254263ms**; SSR entries span **1.639459ms**.
A cap of four limits concurrent request futures, not the number of successes
over an entire sixteen-request burst. Different arrival spacing and recycling
of permits can produce four versus nine successes without changing the cap.
No exact permit lifetime is reconstructed from those span bounds.

### Timing populations and historical comparison

| Phase | Raw nearest-rank HTTP p95, all / valid (ms) | Raw full-call p95, all / valid (ms) |
| --- | --- | --- |
| JSON observe | 1.316030 / 1.325102 | 2 / 2 |
| JSON burst | 1.041342 / 1.041342 | 2 / 2 |
| SSR observe | 0.966173 / 0.931092 | 2 / 2 |
| SSR burst | 1.493250 / 1.493250 | 3 / 3 |

These are raw-request **nearest-rank** reductions, not native k6 interpolated
summary p(95). Both complete native summaries and raw all/valid/p99/max/tail
distributions remain in the evidence; recovery uses its separate Python caller
population and is not mixed into k6 percentiles. JSON has 2495 valid completions
in-window (**499.0 valid/s**) and one post-window completion; SSR has 9925
in-window (**1985.0 valid/s**) and two post-window completions. Both observed
drains are 1ms, with all boundary requests retained.

For the 17 JSON and 81 SSR HTTP-only 503 responses, server-span nearest-rank p95
is respectively 44.877µs/25.978µs, while client HTTP p95 is 0.761770ms/10.335525ms.
SSR failed full-call maximum is 27ms. Immediate server-side try-rejection does
not promise immediate end-to-end completion. Client/server clocks and
populations stay separate; do not subtract them to invent network, scheduling
or gate-wait attribution.

The retained YT-174 cohort has **12541 valid and nine Session-busy failures**;
the new cap-four cohort has **12452 valid and 98 pre-Session failures**, with
zero observed Session busy/queue timeout. The backend is shielded in this
finite experiment, but more requests are rejected overall. JSON/SSR observe
failures rise from 0/1 to 5/74; burst failures change from 8/0 to 12/7. These
are retained outcomes, not a causal throughput improvement or an error-free
capacity qualification.

An HTTP permit covers Session plus downstream SQL/render/finalization, whereas
a Session permit covers a provider operation: equal numeric limits have
different units. This source-based distinction explains why the new policy can
be stricter; it does not establish a unique cause for every timing difference.
New binary/features, sequential host/warming/arrival state and the stricter
caller-oracle overhead differ from the historical run. No randomized matched
comparison, universal capacity/SLO or production-default cap is established.

### Resources and actual cleanup

There are 26 host-only samples: recorded wall cost sum **0.374799s**, maximum
0.026946s, maximum sample-start gap 1.032736s. This is the existing sampled body
through resource/file-size reads before append/post-write guards, not total
observer CPU or zero interference. Optional Valkey exe/FD fields remain null/
permission_denied in all 26 samples; mandatory identities/counters remain valid.
The task command audit confirms zero Docker exec and no recorded task Docker
command/SQL client overlap with native load or recovery windows. All six finite
SQL clients exit 0, outside traffic; source guards retain the same exclusion.

JSON/SSR observe sample-bracket average CPU cores are app 0.249/0.428, client/
control 0.917/1.588, PostgreSQL 0.101/0.179 and Valkey 0.054/0.089. Sampled
cumulative charged-memory peaks are app 13688832 bytes, client 171925504,
PostgreSQL 87986176 and Valkey 6868992; they can miss final trace-flush peaks and
do not identify a physical bottleneck. No sampled OOM/max or resource guard fires.
Raw k6 files total 75844353 bytes; evidence at summary is about 93.2MB, within
256MiB. Sampled PGDATA peaks at 48427266 bytes, within the 2GiB bound.

Stopped counters reconcile to **12484 account queries** (12452 selected valid
plus 32 credential-bootstrap account reads) and **12452 document queries**, zero
Session/SQL errors or denials. Logical account/document rows, schema, indexes
and constraints remain equal; zero business writes or owner policy edits occur.
Both apps acknowledge Session drain and pool closure on their original PIDs.

Canonical cleanup is **PASS**, no errors, completing **18.752s** after the
original service-clock start. Native launcher elapsed is 19.273s; outer
completion 19.339s. Both service containers and all six transient clients,
app/k6/controller/owned worker PIDs, units/listeners, private data/socket/
password/cookie files and the newly acquired unused PostgreSQL reference are
absent. Pre-existing Valkey cache remains. Preserve expected nonzero absence
checks for auto-removed SQL clients and reset/stop receipts for auto-unloaded
units together with verified final absence/inactivity. No fallback allocation
or restarted clock is used.

## Final acceptance boundary

Only this report, Backlog, Roadmap and documentation index change; product code,
provider bounds, lockfiles/default graph and all 52 retained input bindings stay
unchanged. Build/caller/native/controller/reducer/raw/outcome/cleanup receipts
remain under the task evidence directory. The full reducer runs once only after
cleanup; no previous test/qualification/analysis is rerun. New docs checks pass
(75 documents/177 backlog items) and `git diff --check` passes.

Final independent Reviewer `/root/yt177_admission_workload_final_review_20261025`
returns **PASS**, zero issues/required followups, with native role/model/high
effort verified. All four source, 359 task-evidence, 62 mapping and 52 retained
bindings remain stable through review. Mark YT-177 Done only at bounded enabled-
gate policy-behavior scope. Only four documentation status/review-reference
updates follow PASS; all runtime inputs/caller/controller/reducer/data remain
unchanged. `closeout.json` preserves final source hashes, Git state, accepted
failures and actual cleanup. The four new documents remain uncommitted/unpushed. The integration can be accepted at
bounded policy-behavior scope while retaining all 98 failed requests; cap four
is demonstrably not an error-free recommendation for these offered workloads.
If a consumer needs fewer rejections, it must first choose its acceptance/latency
tradeoff before separately authorizing another cap or workload. Keep the current
cap/Session/zero-queue/no-new-retry selection here; no further tuning, benchmark,
implementation or release activation is automatic.
