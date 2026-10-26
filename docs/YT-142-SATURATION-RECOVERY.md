# YT-142 — Bounded Starter saturation and recovery

Date: 2026-10-23. Status: **Done at bounded optional observation scope after final independent Reviewer PASS**.
Initial independent contract Reviewer PASS preceded implementation and execution.
The failed load point remains FAIL. Final independent Reviewer returned PASS,
zero issues and required followups; controller acceptance completes the bounded
reference without claiming maximum capacity.

## Authority and question

The user separately requested progressive isolated load after the accepted
[YT-141 reference](YT-141-PERFORMANCE-VALIDATION.md), to locate observed
saturation and recovery while retaining failures. YT-142 is a new bounded task;
YT-141, [YT-134](YT-134-RELEASE-STORAGE-CAPACITY.md) and
[YT-136](YT-136-TELEMETRY-COLLECTION.md) stay accepted at their original scopes.
Do not repeat their passing 100/500/s phases, faults, matrices or reviews.
[ADR 0001](adr/0001-separation-of-concerns.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md) remain unchanged.

The question is which **new offered rate above 500/s first fails** the declared
application bounds, whether the evidence attributes that failure to the app or
generator, and how the app behaves when arrivals immediately fall to 250/s.
Report a tested last-pass/first-fail interval, not an exact maximum. A bounded
run may reach a client/resource ceiling or finish without demonstrating app
saturation; report that limit honestly rather than escalating resources or
inventing capacity. No Core/runtime/dependency, admission, Auth/CSRF, policy,
optimization, cloud, public release, consumer SLO or recurring CI change follows.

## Reuse, fixture and finite budgets

Bind HEAD `103177912d86b87a86f8b07ded8a3d18c9496e84` and the current baseline.
Reuse the exact accepted executable at
`target/yt133-private-hw52es_r/build/release/yetlet-production-starter`, SHA-256
`1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`,
without rebuilding. The legacy 345-file receipt still has 339 matches and the
same six accepted unrelated YT-139/YT-140 Admin/Board differences; all 35 recorded
Starter-local inputs and relevant manifests match. Reuse the previous resolved
Cargo metadata identity; do not execute metadata again. `source-findings.json`
binds actual hashes, exceptions, lockfiles, manifests and original evidence.
Before/after checks must match this baseline and exact executable; a new
relevant difference stops execution instead of authorizing repair or rebuild.

Use a new private ignored `target/` directory, one native app, a new high
loopback port and ext4 `/dev/nvme0n1p2`; record mount, available space, host and
actual CPU topology. Copy unchanged Production/base settings, with only fixture
config/database paths/port, fresh private synthetic 64-hex token, metrics=true
and `TOKIO_WORKER_THREADS=2`. Use a clean environment without inherited `APP_*`,
proxy or tool overrides. Retain info logging, HTTP capacity 32/deadline five
seconds, SQL immediate capacity four/busy timeout two seconds and schema 2/WAL.
Starter has no rate limiter; still retain 429 outcomes. No authentication,
CSRF or admission bypass is permitted.

Seed the exact same 20 synthetic rows as YT-141 through real authenticated API
POSTs: IDs 1–20, title `YT141 seed NN` (01–20), body
`controlled-arrival read fixture`, status `open`, version 1. Require 20 status-201
acknowledgments without retries, complete pre-load rows/schema/status-index/
sequence 20 and a verified newest-first list body. A failed or ambiguous seed
stops measurement and requires reconciliation. No measured writes are selected.
Use the real single Bearer header and no Origin/cookie/redirect for API reads;
keep the token out of arguments, logs, metric tags, metadata and raw commands.

| Owned resource | Fixed budget |
| --- | --- |
| App | Same CPU affinity 0/1 and CPUQuota 200%, MemoryMax 2 GiB, TasksMax 128, LimitNOFILE 512, Nice 10, Restart=no, native/supervisor stop bound 20 s |
| Client plus monitor | Separate allowed logical CPUs 2–7, CPUQuota 600%, MemoryMax 1 GiB, TasksMax 128, LimitNOFILE 512, Nice 10, Restart=no; no other services changed |
| k6 VUs | Load 64 preallocated=max; recovery 16 preallocated=max; at most 80 total in a paired invocation. Warmup uses 16 |
| Output | 512 MiB per file, 2 GiB total retained run output, at least 4 GiB free before startup; stop/gap on cap or sink failure |
| Time | Client transient unit RuntimeMaxSec 540 s; outer watchdog 600 s from client launch, including setup/cleanup; archive acquisition separately bounded to 120 s, total at most 720 s; no restart/retry/resource escalation |

Verify all selected CPUs remain allowed; stop if unavailable rather than change
placement after observing results. Record shared-host, logical-core/cache,
monitor and file-output interference. The larger client budget deliberately
differs from YT-141 to reduce generator limitation; historical 500/s is therefore
a prior reference, not a freshly matched lower bound under this client budget.

Reuse official maintained k6 2.3.0 and its accepted release metadata/checksum.
Only after contract PASS, fetch the same official Linux-amd64 archive privately,
SHA-256 `39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`,
verify executable identity/version and record acquisition command/exit/time/
stdout/stderr. No global installation or cloud service is selected. Extend only
the existing [script and README](../examples/production-starter/deployment/performance/README.md)
for explicit saturation phases/rates/VU settings and k6's two native scenarios;
preserve original YT-141 settings support, GET function and canonical metric names.
Do not duplicate scripts, implement a scheduler or create a general harness.

## Predeclared ladder and transition

Warm the app/cache once at a new 250/s for five seconds, excluded from load and
recovery results. Then run these candidate points in order, never repeat one:

| Load point | Offered arrivals/s | Load seconds | Nominal load arrivals | Paired recovery |
| --- | --- | --- | --- | --- |
| 1 | 1,000 | 10 | 10,000 | 250/s for 30 s |
| 2 | 2,000 | 10 | 20,000 | 250/s for 30 s |
| 3 | 4,000 | 10 | 40,000 | 250/s for 30 s |
| 4 | 8,000 | 10 | 80,000 | 250/s for 30 s |
| 5 | 16,000 | 5 | 80,000 | 250/s for 30 s |

The last load phase is five seconds to keep native raw JSON within its finite
file budget; YT-141 used roughly 4 KiB/attempt, so 160,000 attempts could exceed
512 MiB. Its different duration is explicit and cannot establish a ten-second
or sustained 16,000/s result. Maximum scheduled activity is 200 seconds,
268,750 nominal attempts including all five recoveries and warmup, plus bounded
drain/setup/gaps. Stop on the first failed, mixed or generator-limited load
point **after retaining its paired recovery**, unless unsafe state requires an
immediate stop. Continue only when both preceding load and recovery pass.
If every candidate passes, stop at 16,000/s; no saturation claim follows.
No midpoint is selected: adjacent new load points already bound an observed
transition within a factor of two. Do not rerun passed points to narrow it.

Each point is one k6 invocation with two `constant-arrival-rate` scenarios.
Load begins at zero; recovery begins at `startTime` exactly equal to load
duration. Each has `timeUnit=1s`, its fixed VU pool and eight-second graceful
stop; load calls may finish during the first eight recovery seconds. This
overlap is intentional: report carryover and separate scenario populations,
not an uncontended recovery that starts only after drain. The GET function
issues one complete `GET /api/issues` per iteration, with connection reuse
within each VU, eight-second timeout, zero redirects/retries and no sleep.
Recovery has its own pool; warmup does not prewarm later connections.
Use k6's documented [scenario start times and tags](https://grafana.com/docs/k6/latest/using-k6/scenarios/advanced-examples/)
and [graceful stop](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/graceful-stop/);
the accepted pinned tool supplies scheduling and interruption accounting.

Wait for the entire paired invocation to exit, then observe one second idle
before any next candidate. No app restart, policy change, forced SQL fault or
cooldown reset is allowed. Capture actual scenario anchor, load boundary,
individual starts/completions, first recovery start and last load completion;
report generator scheduling lag and actual carryover window. These are
millisecond wall-clock observations, not intended-arrival latency correction.

## Measurements, attribution and investigation bounds

Freeze the reviewed contract, exact script/tool/source/configuration identities,
full argv, clean non-secret environment, fixture, ladder, conditions and limits
before app startup. Reuse the corrected YT-141 direct execution/scrape seams:
inspect app environment after readiness; parse raw output sequentially and retain
only the required analysis populations in memory. Raw native JSON stays intact.
No extra validation infrastructure or old proof rerun is authorized.

Retain per-scenario planned, actual started, native completed, custom completed,
validated-200, dropped and interrupted counts, native boundary extras, actual
start/completion windows and achieved rates. Show completions within the
scheduled window separately from throughput through final completion. Record
429/503/504, timeout, transport, invalid-200 and unexpected outcomes. Publish
nearest-rank p50/p95/p99 for all completed attempts, HTTP responses and exact-body
200s separately. Dropped/never-started and interrupted calls have no completed
latency sample. Retain raw start/completion/scenario-anchor metadata and outcome
tags without bodies or credentials; preserve the existing canonical Trends.

Use non-overlapping existing bounded telemetry/readiness samples about once per
second, with actual timestamps/sample gaps, boundary snapshots and separate
probe accounting. Retain existing three-second scrape/five-second watchdog and
64 KiB metric limits. Report app and actual post-exec k6 PID CPU/RSS/FD/thread
samples, monitor and both cgroups' CPU/memory/throttling/events, storage sizes,
active gauges and registry deltas. Child wait4 RSS includes fork-before-exec
inheritance and is not isolated k6 RSS. Server HTTP timing ends before body
delivery; SQL permit-wait is not SQLite/blocking-pool wait. Sampling can miss
peaks; concurrent registry snapshots are not transactional.

A passing load point requires zero 429/503/504, timeout/transport/unexpected/
invalid-body outcome, drops or interruptions; all-completed p95 <=100 ms and
p99 <=250 ms; validated scheduled-window completion rate >=95% of offered rate;
mean app CPU <=200%, sampled peak app RSS <=128 MiB; readiness 200 throughout,
no collection gap and no client/output/resource limit. Publish successful-only
latency alongside all-completed values, never as a replacement. Missing samples
are unavailable, not a passing zero. These are local investigation bounds.

Classify every first failure rather than equating configured rate with capacity:

- **Application admission/deadline evidence:** workload 503 or 504 with the
  corresponding existing SQL-saturated, HTTP-capacity or HTTP-deadline delta,
  with workload and readiness populations distinguished. This demonstrates the
  existing app boundary at observed arrivals, not justification to weaken it.
- **Application resource saturation:** latency/95%-throughput bounds fail with
  app CPU >=180% and increasing app cgroup CPU throttling, while client generation
  is complete and client CPU/memory/output limits are absent. Report all signals;
  latency alone is a performance failure, not proof of CPU saturation.
- **Generator or mixed limit:** any dropped/interrupted work, client throttling,
  memory/task/output cap or missing generation evidence prevents an app-only
  offered-rate limit claim. App rejections may coexist: retain both findings and
  achieved arrivals without calling the configured target an exact app capacity.
- A first failure without attributable signals remains unexplained and stops
  escalation. A failed threshold authorizes neither tuning nor a replay.

Report the last **new passing** load rate and first failed rate with attribution.
If the first 1,000/s point fails, there is no new passing load rate above 500/s;
state that and retain historical YT-141 500/s separately, rather than presenting
it as a matched same-run bracket. If client/mixed limitations prevent a clean
app bracket, report the unresolved bound explicitly.

## Recovery and completion

Retain every paired 30-second recovery, including after the stopping point.
For each of its 30 scheduled one-second bins, classify attempts by actual start
relative to the recovery scenario anchor, and retain completed/validated counts,
drops/errors and all/success p50/p95/p99. Report carryover load requests as a
separate population using their load scenario tags. Native endpoint extras
outside the 30 half-open bins remain a separate boundary population.

A healthy recovery bin has at least 240 actual starts and all of them completed
as valid 200, zero dropped/interrupted/error outcomes, all-completed p95 <=100 ms
and p99 <=250 ms, plus latest readiness 200 observed no more than two seconds
before bin end and no intervening failed sample. That fresh snapshot must also
have SQL active <=4 and HTTP active <=17 (16 recovery VUs plus one readiness
probe); retain any still-in-flight load calls separately rather than count
them as recovery attempts. The 240 minimum permits small
wall-clock/bin scheduling variation around nominal 250 without concealing drops.
Require the first **ten consecutive healthy bins** within the fixed 30-second
window. Report the first healthy bin's start, its interval end and the end of
the first ten-bin confirmation window as distinct timestamps, each sample's
actual age, and recovery completion
time from the load-scenario boundary through the latest of the interval end,
its request completions and supporting probe completion. Do not backdate a
healthy interval on knowledge only available later. Maximum recovery observation
is 30 scheduled seconds plus at most eight seconds in-flight drain; a qualifying
interval must lie within the 30 scheduled seconds even if confirmation arrives
during drain. Whole recovery also reports
the load-point CPU/RSS/latency/error bounds, resource samples and missing bins;
no successful-only aggregate hides an unhealthy first second. Keep full-window
investigation PASS/FAIL separate from whether a later ten-bin recovery interval
was confirmed.

If no ten-bin interval qualifies, record recovery FAIL/right-censored at the
actual last observation; do not wait longer, restart or optimize. If all load
points pass, these are post-load transitions, not recovery from demonstrated
saturation. After final recovery/drain, require active HTTP/SQL gauges zero,
observe five seconds idle, and stop natively within 20 seconds with exit 0 and
`stopped`. Any unsafe mutation, resource/process/output failure triggers bounded
stop and partial evidence instead of continuing recovery for its own sake.

Reconcile the full stopped rows/schema/index/sequence with all 20 seed
acknowledgments and the pre-load manifest: zero measured writes, zero missing,
extra or changed rows and zero unknown outcomes. Retain evidence, reap every
child and observe actual owned unit state before attempting cleanup. Capture
every command's exact exit/stdout/stderr/time, including a nonzero command;
already-unloaded inactive/MainPID-0 units need no repeated stop. Verify PIDs/
descriptors/listener gone, then remove only owned fixture/config/token/tool
archive/extraction and transient units. No existing services are modified.

## Current checkpoint

Evidence root: `/tmp/yetlet-yt141-saturation-20261023-v23iozrv`.
`source-findings.json` binds unchanged executable/Starter inputs, reused metadata,
accepted tool/source receipts and historical corrected execution seams. Initial
independent Reviewer `/root/yt142_saturation_contract_review_20261023` returned
PASS with zero issues and required followups; reviewed identities and the
immutable `contract-reviewed-document.md` remain separate from later results.

## Actual observations

The existing script gained only explicit YT-142 phase/rate/VU support and native
paired scenarios; the GET function, metric names and original 16-VU YT-141 mode
remain source-compatible. No accepted 100/500/s workload was rerun. Exact script,
commands, settings, source and tool were frozen in `preexecution.json` before
startup. The official archive and extracted k6 executable matched their accepted
SHA-256 identities; acquisition command/exit/stdout/stderr/time are retained.
Before/after executable and source receipts match, with the same six disclosed
unrelated exceptions. No rebuild, runtime/policy change or cloud operation ran.

Execution ran from 11:55:33.673 to 11:56:27.021 UTC on 2026-10-23 with the
reviewed app/client limits, CPU affinities 0/1 and 2–7, local ext4 fixture and
loopback port 45367. New warmup and **only the first load/recovery pair** ran:

| Population | Nominal / actual starts / completed | Valid 200 / HTTP 503 | All completed p50/p95/p99 ms | Valid-200 p50/p95/p99 ms | Investigation outcome |
| --- | --- | --- | --- | --- | --- |
| New 250/s, five-second warmup | 1,250 / 1,251 / 1,251 | 1,251 / 0 | 1 / 2 / 2 | 1 / 2 / 2 | PASS; excluded from load |
| 1,000/s, ten-second load | 10,000 / 10,001 / 10,001 | 9,996 / 5 | 1 / 1 / 2 | 1 / 1 / 2 | **FAIL: five SQL-admission 503s** |
| Immediate 250/s, thirty-second recovery | 7,500 / 7,501 / 7,501 | 7,501 / 0 | 1 / 2 / 2 | 1 / 2 / 2 | Full window PASS; ten-bin recovery confirmed |

Native completed iterations and custom completed points agree. Each population
has a retained native boundary extra, `nominal - started - dropped = -1`.
There were zero dropped/interrupted iterations, 429/504, timeouts, transport
errors, unexpected status or invalid-200 responses. The five load failures have
k6 code 1503 (HTTP 503), not transport failure. All completed load attempts and
HTTP-response attempts contain 10,001 values; successful-only latency excludes
five responses. Their quantiles happen to coincide; their populations differ.
All full-call percentiles use the declared nearest-rank, millisecond wall clock.

The load rejection fraction is **5/10,001 = 0.049995%**. Load-window registry
samples show exactly five SQL `saturated` admissions and five GET 503s, no
HTTP-capacity/deadline rejection, and readiness 200. This is observed brief
exhaustion of the existing four SQL permits, not a CPU or sustained throughput
maximum. No generator drop, CPU throttling or client resource limit occurred.
The ladder stopped as declared: 2,000/4,000/8,000/16,000/s did not run. There is
**no new passing load point above 500/s and no same-run last-pass/first-fail
bracket**. Accepted historical YT-141 500/s remains a separate reference under
its different client budget; it is not silently promoted to a matched bound.

Achieved load starts were 1,000.1/s. Counting only completions inside the ten
scheduled seconds gives 999.9 completed/s and 999.4 valid/s. Through the last
load completion, 10.001 seconds, these are 1,000.00 and 999.50/s respectively.
Recovery starts were 250.033/s; scheduled-window completion/success were both
250.00/s, versus 250.017/s through the final completion at 30.002 seconds.
Raw counts and distinct denominators remain retained; none substitutes a
successful-only rate for all attempted work.

Load attribution uses ten resource samples within the native load window:
11:55:41.189835–11:55:50.564759 UTC, **9.374925 seconds**, excluding startup and
recovery. Mean app CPU was 49.49%, actual k6 PID CPU 51.31%, and sampled app/k6
RSS peaks 9.93/69.21 MiB. The corresponding registry snapshots span
11:55:41.185796–11:55:50.560847 and contain all five SQL rejections. The enclosing
snapshot interval also includes boundary slivers and is labeled separately;
no forty-second paired average is used for saturation attribution. One core is
100%. Neither app nor client cgroup recorded throttling. The finite snapshots
can miss peaks and do not establish a hard server-only rate limit.

Recovery's native scenario anchor and first actual request were both
**11:55:51.178 UTC**, exactly the declared load boundary. Two load calls completed
after it, with last completion at +1 ms; their native `load` population remains
separate. All thirty recovery bins were healthy, with 249–251 starts/bin and a
separately retained one-request endpoint population. Supporting readiness was
200, maximum age 1.076 seconds, and the declared active-gauge bounds held.
The first complete **[0,1) second bin** was healthy. The first ten-bin interval
was [0,10), with its end and required-completion/probe confirmation at
11:56:01.178, **+10.000 seconds**. This confirms that observed interval; it does
not prove instantaneous recovery or that the application required ten seconds
to recover. No unhealthy initial bin was omitted, and no recovery was extended.

Recovery resource samples separately span 11:55:51.595251–11:56:20.855380 UTC,
29.260129 seconds; app/k6 mean CPU was 20.92%/22.80%. There were 50 fresh bounded
metric observations, paired readiness probes and zero collection gaps; samples
took 0.0261–0.0452 seconds. Including startup, readiness returned 51 HTTP 200s;
one initial connection refusal remains outside workload counts. Final counters
account for GET 200 **18,800 = 18,748 valid workload (including warmup) + one
fixture-list read + 51 readiness**; GET 503 five; POST 201 twenty seeds. SQL
acquired 18,820 permits and rejected five. Fifty metrics scrapes exclude
themselves from those HTTP counters while consuming resources.

Whole-run sampled peaks were app RSS 10.13 MiB, actual post-exec k6 RSS
70.77 MiB, monitor RSS 135.96 MiB and combined client-cgroup memory 198.32 MiB.
No memory-limit/OOM event or CPU throttling occurred. App/k6 thread peaks were
9/12 and FD peaks 90/88. Child wait4 high-water includes inherited pre-exec
memory and is separately labeled, never treated as isolated k6 RSS. Sampled
storage maxima were database 16,384 bytes, WAL zero and SHM 32,768 bytes; no
physical I/O, durability or cold-cache conclusion follows.

All twenty authenticated seed acknowledgments exactly match the stopped full
rows, schema-2 DDL, status index and sequence 20. No measured writes, unknown
outcomes, extra/missing/changed rows occurred. Final HTTP/SQL active gauges were
zero; after 5.036 seconds idle, native SIGTERM returned exit 0 with `stopped`.
The owned supervisor stop call took 0.003566 seconds, not an RTO commitment.
All six owned app/monitor/k6/outer-watchdog PIDs and their descriptors are gone,
both transient units inactive/unloaded with MainPID 0, port refused, and fixture,
credentials and downloaded tool archive/extraction removed. Actual unit state
was observed before cleanup; every cleanup command exited 0. Existing services
were untouched. No setup/harness/operational failure, correction, product repair
or workload replay occurred in YT-142; the five planned investigation failures
remain evidence rather than an implementation defect.

`runtime-result.json`, raw native JSON/summaries/stdout/stderr, timestamped
`samples.jsonl`/`collection/`, outside-workload request records, database manifests,
`reconciliation.json`, `reporting-projection.json`, command histories,
`cleanup.json` and `implementation-summary.json` retain identities, outcomes,
intervals, per-second recovery and cleanup. Final independent Reviewer
`/root/yt142_saturation_final_review_20261023` returned **PASS**, zero issues and
required followups. All eight input hashes and 1,001 outside-scope tracked files
matched; the reviewer independently recomputed the load/recovery populations,
latency, rates, recovery bins, inner load attribution and native counters, and
checked source/data equality and cleanup. `review-final.json`, `review-input.json`,
`reviewed-files.json` and `review.patch` preserve the reviewed input. Only
acceptance/status bookkeeping follows PASS; the executed script is unchanged.
YT-142 is **Done** at bounded optional observation scope, while the load point
remains **FAIL**. No additional workload is selected. The remaining backlog
gates are YT-096 release decisions and dependent YT-020; no new task is activated. Build/browser/CSP, consumer SLO, exact maximum, sustained CPU saturation
and production qualifications remain unclaimed. No stage, commit, push or
cloud operation is authorized by this reference.
