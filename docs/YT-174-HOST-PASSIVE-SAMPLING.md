# YT-174 — Host-side passive resource-sampling comparison

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded passive-observer
comparison scope; final independent Reviewer PASS, zero issues/followups.**

## Authority and preserved control

The user authorizes a host-side passive sampling comparison without
`docker exec` to examine observer interference: contract first, minimal new
measurements, unchanged limits/security, reuse passed validation, retain all
failures, clear test resources and obtain independent Reviewer PASS before Done.
New changes remain uncommitted; no Push is authorized.

The thirteen [YT-173](YT-173-HTTP-SESSION-TIMELINE.md) paths are committed as
`3180b4c` after strict/builtin Commit Guard PASS, zero findings, exact tree
`d4ae57bff581114194c9451ab9732cbf8d9d5e2a`; the worktree is initially clean.
Preserve its 12550 requests/12538 valid/twelve busy, complete per-request traces,
review/cleanup receipts, failed CLI/mapping attempts and uncertainty. Its 26
resource samples cost 3.182s, maximum 0.143s; eleven early rejections overlap
a Valkey container-proc command under explicit advisory clock assumptions,
while one later rejection does not. These facts do not prove observer causation.

Reuse YT-173 as a frozen **historical control**, without rerunning its accepted
qualification or live container-exec observer. Run one newly selected passive
condition using the exact accepted application binary, caller, SQL fixture and
instrumentation configuration. No runtime/provider/Core/API/default graph,
build/feature/dependency, limit, security, HTTP body/header or route change.
Keep ADRs [0001](adr/0001-separation-of-concerns.md),
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md).

## Passive observer and isolation boundary

All periodic resource observations read only host `/proc` and cgroup files for
the pre-admitted, exact-owned app/client/container main PIDs and cgroups. Cache
the verified PID/start-tick/cgroup binding before load and reject identity drift.
Keep the original sample-before/after and nominal post-sample one-second cadence;
record actual cadence, sample duration, CPU/charged memory/events and file-budget
cost. No container-side sampler, process, signal, ptrace, privilege change,
Docker command, SQL control query or active network probe runs inside a sample.

Mandatory readable process/cgroup identity and counters must be present; loss
stops new exposure and triggers cleanup. Optional host exe/FD details can be
permission-restricted across UIDs: record closed unavailable categories rather
than entering the container or escalating privilege. Do not call such fields
verified, substitute guessed numbers or turn optional details into a new gate.

The entire new task uses **zero Docker exec operations**, including setup.
Docker lifecycle/inspect/log operations occur outside measured traffic. Since
the host has no psql/psycopg, SQL setup/reconciliation uses short-lived,
exact-owned PostgreSQL-image client containers before/after the app workloads,
with the private Unix socket/password file, no network or published ports,
fixed UID and bounded CPU/memory/PIDs/FDs. They are removed on exit or by
ownership-verified fallback; none runs during offered load. Valkey readiness
uses its accepted log/host listener context, not a container-side command.
Version qualification is reused through immutable pinned-image identity.

The necessary SQL-client launch mechanism differs from the historical setup;
SQL/fixture/preconnection/bootstrap semantics remain unchanged. Disclose this
and sequential timing/warming/host-state differences; do not claim a randomized
or simultaneous paired experiment.

## Once-only new sampling condition

| Per-mode phase | Fixed selection, matching the historical comparison cohort |
| --- | --- |
| Sequence | JSON then private SSR, one fresh app/pool/eight Sessions per mode, shared new Valkey/PostgreSQL allocation |
| Observe | JSON 500/s for 5s; SSR 2000/s for 5s, fixed 128 VUs |
| Finite HTTP burst | Sixteen VUs, exactly one iteration each; retain actual server/client start spread |
| Post-load health | Eight sequential full authenticated reads on the original PID/Store/pool |
| Identity/oracle | Same synthetic-ID ranges, complete body/header/CSP oracle and common server/provider/worker clock as YT-173 |

These are measurements under a **different observer condition**, not repeated
qualification of the old runtime or passed burst/recovery proofs. Holding the
phase sequence avoids silently changing the prehistory before SSR. No old
tests/builds/Clippy/caller-plan qualification or direct-provider experiment is
rerun. Bootstrap only the credentials/dataset needed for the new condition.

Maximum exposure remains 12500 nominal observations, 32 burst and sixteen
post-load reads, plus finite boundary/bootstrap/control requests. No extra
round, duration extension, rate change, tuning, fault injection, active-slot
hold or retry if busy disappears or persists. Keep four active/eight total
Session slots, 80ms queue/400ms caller bounds, 300s TTL/256-byte payload; SQL
pool four, 250ms acquire/750ms query and original statements. Full response
validation and fail-closed identity/authorization behavior stay intact.

Ordinary pressure remains failed requests in its finite invocation. Native,
oracle, mandatory resource/identity, coverage/capture or deadline failure stops
new exposure, preserves partial raw/native/reap evidence and cleans up. Failed
post-load health stops the remaining mode; unstarted work is unqualified.
No favorable replay or replacement clock/run is allowed.

## Comparison and inference limits

Compare sampler per-sample/aggregate cost and actual cadence; verify from
source/command receipts that no Docker exec is used and no Docker operation or
SQL client overlaps measured traffic. Reconcile all client/server/call/worker
IDs, statuses, errors, all/valid timing populations, drop/window/tail counts,
SQL counters/logical equality, worker-release bounds and actual cleanup.
Keep optional inaccessible host fields explicit. No new validator/observer
framework or product metrics feature is selected.

For each new busy event reuse the conservative same-server-clock ownership
analysis, separating startup arrival/connection context, outstanding reply
episodes and sampling intervals. The admitted-to-reply lower bound and
post-drop upper bound do not become exact release timestamps. Client timing
and any advisory offset assumptions remain separate from direct server proof.

Removing container-exec work mechanically removes that observer path; lower
measured sample cost establishes overhead reduction. Fewer busy events in one
short historical-control comparison supports sensitivity but does not prove
that sampling caused every prior error. Persisting busy demonstrates that the
old observer is not necessary for those new events. Zero busy is a finite
observation, not a higher capacity/SLO guarantee. No significance/unique
historical-cause, physical bottleneck or production-equivalence claim is implied.
Report uncertainty and stop at the declared exposure on every outcome.

## Resource and cleanup budget

Reuse YT-173's pinned Valkey 9.1.2/PostgreSQL 18.6 identities and private topology:
app CPUs 12–13/two CPU/512MiB/zero swap/128 tasks/512 FDs/Nice 10/no restart;
PostgreSQL CPUs 14–15/two CPU/512MiB/zero swap/64 PIDs/1024 FDs/64MiB shm,
network none/private ext4 data; Valkey CPU 10/one CPU/128MiB/zero swap/64 PIDs/
1024 FDs/64MB noeviction/no persistence/UID 10001/owned tmpfs. Load/control
uses CPUs 16–19/four CPU/1GiB/zero swap/128 tasks/512 FDs/Nice 10.

Transient SQL clients use CPUs 16–19, one CPU/128MiB/zero swap/64 PIDs/512 FDs,
network none/read-only root/private tmpfs, UID matching the private socket.
No concurrent workload runs during those setup/reconciliation commands.
No host tool installation, new image type, cloud/shared service or host setting.

Allow 180s for missing pinned-image acquisition; no compilation budget is needed.
Use one original 600s service clock with 180s cleanup reserve, capped native
commands, unchanged application/unit bounds, no reset/replacement allocation.
Keep 128MiB per raw file/256MiB new evidence, at least 8GiB initial free disk
and at most 2GiB private PGDATA. Reuse exact ownership checks, native receipts
in finally, independent bounded cleanup actions and force-removal fallback.
One smallest canonical glue correction per distinct seam is bounded; repeated
infrastructure signatures stop without replaying completed work.

On every outcome drain/reap apps/handlers/Session drivers/pools/k6/controller,
remove both long-lived and transient owned containers/units/listeners/private
data/socket/password/cookie files and newly acquired unused image references.
Retain pre-existing caches. Verify actual absence and preserve all failures;
no active test resource may remain.

Independent contract review precedes acquisition/adaptation; concrete mapping
review precedes startup. Final results Reviewer PASS precedes Done. Update this
report/Backlog/Roadmap/index with actual evidence; no new-change Commit/Push,
further item, provider tuning or release-gate expansion. YT-096/D1–D6 and YT-020
remain unchanged.

## Concrete mapping before startup

Independent contract review returned PASS, zero issues/followups, with native
reviewer/model/effort verified. The accepted application binary SHA-256 is
`fd1a28433c0d3a78105bf9add93b757453b1a2680a45ba1bc4cce36760415afe`.
Task-local `execute.py`/`launch.py` adapt the accepted YT-173 scripts; the
timeline reducer and caller are reused. Explicit empty k6 configuration keeps
the accepted empty-config behavior without changing HOME. No compilation,
qualification test or old caller-plan inspection is rerun.

The sampler has no subprocess path. Required PID start-tick checks, host-only
optional exe/FD reads, existing resource/file guards and the original one-second
post-sample cadence remain. A controller guard forbids Docker exec entirely and
Docker commands during observe/burst/recovery. Six named, bounded SQL clients
cover original setup/reconciliation calls; the outer cleanup also knows their
exact ownership. PostgreSQL final readiness reads its native stderr PID 1 log
after stdout initialization completion. App/k6 native wait/reap receipts retain
finally coverage. These are setup/observer adaptations, not product changes.

Artifacts are under
`target/yt174-host-passive-sampling-20261025/evidence/`; immutable accepted
bindings, mapping manifest, native receipts, raw samples/traces and all review
results stay there. The newly acquired PostgreSQL reference is task-owned and
removed after use; the pre-existing Valkey cache is retained.

## Measured outcome

Contract review PASS precedes acquisition/adaptation. Initial mapping review
found one P2: launcher wait/stop exceptions could skip outer cleanup/native
receipts. Preserve its FAIL and original script. One canonical correction
catches the original failure, bounds controller stop/kill and launcher reap,
writes the native receipt in finally and continues independent cleanup. The
same independent Reviewer then returned PASS, zero issues/followups; all forty
mapping, thirty-four accepted and four source bindings matched before startup.
No services or measurements preceded that corrected PASS.

The sole launch exited 0; all four k6 invocations exited 0. Ordinary pressure
still counts as failed requests, not a successful response or a replay trigger.
There are **12550 selected requests, 12541 valid 200 and nine busy 503**, zero
drops/incomplete iterations/invalid 200/SQL errors/authorization denials. The
complete joins contain 12550 HTTP spans, 12550 provider calls and 12541 workers
(**37641 events**), with zero overflow or missing/duplicate selected IDs.

| New phase | Started/completed | Valid 200 | Busy 503 | Drops | Raw-request nearest-rank HTTP p95, all / valid (ms) |
| --- | ---: | ---: | ---: | ---: | ---: |
| JSON observe, 500/s × 5s | 2501 | 2501 | 0 | 0 | 1.318735 / 1.318735 |
| JSON sixteen-request burst | 16 | 8 | 8 | 0 | 1.358694 / 1.358694 |
| JSON serial post-load reads | 8 | 8 | 0 | 0 | Not a k6 population |
| SSR observe, 2000/s × 5s | 10001 | 10000 | 1 | 0 | 0.957283 / 0.957283 |
| SSR sixteen-request burst | 16 | 16 | 0 | 0 | 2.297649 / 2.297649 |
| SSR serial post-load reads | 8 | 8 | 0 | 0 | Not a k6 population |

The table reduces the native per-request metrics using nearest rank. Native
k6 summary percentiles interpolate: JSON/SSR burst p(95) is respectively
1.33722825/1.7876925ms; those summaries and the raw nearest-rank distributions
are both retained and must not be interchanged.

Boundary iterations are retained. JSON has 2500 valid completions inside the
five-second window and one post-window completion; SSR has 9998 valid inside
and two post-window completions. Both observed drains are 1ms. Full-call p95
(JSON/SSR observe) is 2ms/1ms; HTTP duration and full-call duration remain
different populations. All/valid complete timing distributions and tails are
in `timeline-analysis.json`, not inferred solely from native aggregate p95.

### Observer cost and cadence

| Same 26-sample selection | Retained YT-173 | New host-passive condition |
| --- | ---: | ---: |
| Aggregate recorded sample wall time | 3.182129s | 0.364719s |
| Mean sample | 122.390ms | 14.028ms |
| Maximum sample | 143.425ms | 27.054ms |
| Maximum actual sample-start gap | 1.139926s | 1.031995s |

Recorded aggregate sample wall cost falls **88.54%**. New JSON/SSR periodic
observe start-gap means are 1.024833s/1.022780s, consistent with the unchanged
post-sample one-second schedule rather than an exact 1Hz observer. This cost
is the existing sample body through resource/file-size reads, before JSON
append and post-write guards; it is not total observer CPU or zero interference.
Host file/counter reads and existing evidence/data-size walks remain. Their
individual costs were not separately attributed.

At the summary snapshot, the native-command audit has 58 argv receipts, including forty Docker commands
and six SQL clients that all exit 0. It finds **zero Docker exec** and zero
Docker command overlap with the four native traffic intervals or two serial
recovery intervals. Source also guards Docker during all traffic and contains
no subprocess path inside sampling. Inherited direct ownership/absence inspect
calls occur before exposure or during cleanup. SQL clients finish before load
or start after both apps stop. No SQL control query runs during traffic.

Mandatory host PID/start-tick/cgroup counters remain readable. PostgreSQL host
exe/FD data are readable; Valkey's different UID denies both optional fields in
all 26 samples. Values stay null with `permission_denied`; no container fallback
or privilege change occurs. No optional value is treated as verified.

### Busy attribution and comparison limits

Every new busy has exactly eight guaranteed and possible owners and four
guaranteed acquired owners throughout its 108–247ns rejection interval:
**four active plus four pending**, not an 80ms queue expiration. All guaranteed
owners have not yet replied; post-reply-only permit retention is unnecessary
for these events. These are conservative bounds, not exact release timestamps.

JSON burst IDs 20001/20002/20003/20005/20007/20009/20012/20014 reject. Its sixteen
server entries span **0.490664ms**, versus the historical 2.207377ms; the same
caller concurrency does not ensure the same arrival clustering. Eight requests
are admitted and eight reject while those owners remain outstanding. Their
queued acquisition waits are approximately 0.305–0.352ms, and acquired-to-reply
intervals approximately 0.262–0.524ms; limits are unchanged. All eight serial
reads subsequently validate on the original app/Store/pool.

SSR ID 6489 rejects about **3244.451ms after the first observe server entry**,
not in the historical early startup cluster. The eight owners have approximately
0.382–0.809ms pending waits and 0.576–1.071ms acquired-to-reply intervals. The
failed client uses an existing connection (`connecting = 0`), full-call rounded
0ms / HTTP 0.118223ms; do not confuse its millisecond-rounded full-call value
with a zero duration. The SSR finite burst and eight serial reads all validate.

The retained advisory endpoint construction gives consistent wall-origin
interval widths of 52.669µs (JSON) and 46.474µs (SSR). Under its explicit stable
wall/monotonic-rate and native metric-endpoint assumptions, none of the nine
feasible failure intervals overlaps a recorded sample. Sample endpoints are
expanded by 1µs; ISO fractional nanoseconds are parsed directly. This is
advisory cross-clock context, not an exact alignment or proof that no earlier
observer scheduling effect persists. Direct gate/owner conclusions do not
depend on this alignment. Native component metadata remain in
`busy-client-transport-context.json`.

The historical control has JSON observe/burst 2501/16 valid, SSR observe
9989 valid + twelve busy and SSR burst sixteen valid. The new condition has
one SSR observe busy but eight JSON burst busy. Do not collapse that change
into a causal error-rate improvement. Observe HTTP p95 is slightly higher
(JSON 1.284469→1.318735ms; SSR 0.941433→0.957283ms), despite lower sampler cost.
Burst latency also cannot be a speedup claim when half the new JSON burst
fails fast. No significance, throughput increase, capacity/SLO uplift or
production equivalence is established.

Removing exec mechanically removes that observer path and lowers recorded
sampler overhead. Busy still occurs without it: container-exec observation
is **not necessary** for these new rejections. One short historical-control
comparison cannot establish the precise cause of the old twelve/twenty events.
Sequential warming/host state, actual arrival distribution and the disclosed
SQL setup launch change remain confounders. No rate/limit change, favorable
retry, extra exposure or claim of a product regression/fix follows.

### Resources, reconciliation and actual removal

During JSON/SSR observe, sample-bracket average CPU cores are app
0.248/0.431, client/control 0.915/1.553, PostgreSQL 0.103/0.185 and Valkey
0.054/0.093. These include the bracket/control work, not just the exact
five-second interval. Sampled cumulative charged-memory peaks are app
14741504 bytes, client/control 164061184, PostgreSQL 86462464 and Valkey
6963200. They can miss final trace-flush peaks and do not identify a physical
bottleneck. No sampled OOM/max event or mandatory resource guard fires.

Stopped app counters reconcile to 12573 account queries (12541 selected valid
plus 32 credential-bootstrap reads) and 12541 document queries, nine Session
busy, zero SQL errors/denials. All account/document rows and schema/index/
constraint output remain equal; zero business writes or policy edits occur.
Each app reports Session drain and SQL pool closure on its original PID.
Raw k6 files total 71055007 bytes; evidence at summary is about 88.1MB, below
the 256MiB budget, and sampled private PGDATA peaks at 48427266 bytes.

Canonical cleanup PASS has no errors and completes 18.746s after service-clock
start; native launcher elapsed 19.271s and outer completion 19.358s, within the
original clock. Both service containers and all six transient clients, owned
app/k6/controller/worker PIDs, transient units/listeners, private data/socket/
password/cookie files and newly acquired unused PostgreSQL reference are gone.
The pre-existing Valkey cache remains. Native absence checks for auto-removed
SQL clients return 1, and reset/stop calls for already auto-unloaded units
return nonzero; retain those receipts with the final verified absent/inactive
states. No fallback resource allocation or restarted service clock is used.

## Acceptance and next boundary

Changed repository paths are only this report, `docs/BACKLOG.md`,
`docs/ROADMAP.md` and `docs/README.md`; product source and all accepted evidence
remain hash-identical. Initial mapping FAIL, corrected PASS, original launcher,
raw failed requests, all native nonzero receipts and old studies remain. No old
build/test/caller qualification, browser/CSP, IdP/Valkey qualification or cloud
capacity result is rerun. New documentation checks pass (75 documents / 174
backlog items) and `git diff --check` passes. Final independent results Reviewer
`/root/yt174_host_passive_results_final_review_20261025` returns **PASS**, zero
issues/required followups, with native reviewer/model/high effort verified.
Mark YT-174 Done only at this bounded comparison scope; the four new document
changes remain uncommitted/unpushed.

Preserve the initial results-review FAIL: one P2 identified a nearest-rank p95
heading mislabeled as native k6 p95. One report-only correction distinguishes
raw nearest-rank distributions from native interpolated summaries; no measurement
or analysis is rerun. Corrected four-source, 355 evidence, forty mapping and
thirty-four accepted bindings remain stable through re-review. Contract, mapping
FAIL/corrected PASS, results FAIL/corrected PASS and all raw outcomes remain
under the same evidence directory; `closeout.json` records final status/source
hashes and the status-only post-review documentation updates.

The next useful boundary is to retain host-side observation for separately
authorized future performance studies, with its remaining overhead disclosed.
Existing four-active/eight-total fail-fast admission behaves as designed under
the observed clustered arrivals. An application that needs different burst
acceptance would first choose its latency/resource/overload policy; no automatic
provider limit increase, new tuning item or additional experiment is activated.
