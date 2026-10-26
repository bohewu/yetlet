# YT-141 — Controlled-arrival API performance reference

Date: 2026-10-23. Status: **Done at bounded optional local reference scope after final independent Reviewer PASS**.
Independent contract Reviewer PASS preceded the minimal implementation and new
workload below; it did not accept runtime results or mark the item Done.
This optional application reference adds no framework default, mandatory
dependency, consumer SLO, recurring CI gate or production qualification.

## Authority and new question

The user activated the [previously scheduled performance item](PRODUCTION-DEPLOYMENT-PROFILES.md#future-performance-validation),
requiring community-tool comparison before a minimal reproducible reference.
[ADR 0001](adr/0001-separation-of-concerns.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md) remain unchanged. No Core,
runtime, admission, authentication or configuration API changes are selected.

The new question is: for the exact accepted Starter release and a fixed small
dataset, what request outcomes, full-call latency and resource use are observed
when authenticated read-only API requests arrive at **100 and 500 requests/s**,
and does the following lower-rate phase satisfy the same investigation bounds?
Two fixed repetitions expose variation without searching for maximum capacity.
The rates are experiment inputs, not capacity claims or required service levels.

[YT-134](YT-134-RELEASE-STORAGE-CAPACITY.md) already measured closed-loop mixed
reads/writes with fresh connections. Its accepted points, rejected Step 8 and
Azure supplement retain their original scopes. This exercise instead uses
controlled arrivals and connection reuse within each phase, with no measured
writes. It does not repeat YT-134, compare fresh versus reused connections,
isolate the effect of connection reuse, or qualify SSR, another application,
proxy/SSH throughput, cold storage, durability, long soak or cloud capacity.

## Tool comparison and selection

The comparison uses primary documentation, consulted on 2026-10-23; alternatives
are not installed or benchmarked. No candidate load tool or Go compiler was
found locally. Building a new scheduler is unnecessary for this question.

| Candidate | Relevant capability and selection consequence |
| --- | --- |
| [k6 constant-arrival-rate](https://grafana.com/docs/k6/latest/using-k6/scenarios/executors/constant-arrival-rate/) | Independent iteration starts, explicit rate/duration and fixed preallocated VUs. [Dropped iterations](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/arrival-rate-vu-allocation/) expose insufficient client capacity. Custom metrics can retain application-valid successes separately. Selected |
| [Vegeta](https://github.com/tsenart/vegeta) | Fixed rate, worker bounds, connection reuse and granular results fit HTTP load. Application-body validation and separate success-filtered reporting still need processing; no smaller complete integration was identified for this selected question |
| [wrk2](https://github.com/giltene/wrk2) | Constant throughput and intended-arrival latency correction answer a related question. Its ten-second calibration and warning about short runs, plus Lua/application accounting, fit this short fixture less directly |
| Existing YT-134 driver | Its closed-loop scheduler cannot establish fixed offered arrivals. Reuse the accepted findings rather than expanding it into another load engine |

Select the official [k6 v2.3.0 release](https://github.com/grafana/k6/releases/tag/v2.3.0),
published 2026-09-21, non-prerelease. After contract PASS, download only the
official Linux-amd64 archive into the private task directory, verify SHA-256
`39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`,
and record the extracted executable hash/version. The archive is 30,847,764
bytes; official release metadata is retained in `k6-release-metadata.json`.
No system installation, container pull, hosted/cloud service or Cargo dependency
is selected. If unavailable or inconsistent, report an operational blocker;
do not silently substitute a tool or change the experiment.

The only planned repository implementation is a small copyable Starter-specific
k6 script and README under `examples/production-starter/deployment/performance/`.
Use k6's executor, [custom counters/trends](https://grafana.com/docs/k6/latest/using-k6/metrics/create-custom-metrics/)
and [granular JSON output](https://grafana.com/docs/k6/latest/results-output/real-time/json/).
The script validates this single response and records its outcomes; it is not
a generic benchmark harness, validator, scheduler or certification framework.
Fixture launch, sampling and cleanup commands stay run-local evidence.

## Exact source, settings and fixture

Contract inspection used HEAD `0b03ad43833448b4e9317d1fc8fb69b557d6b252`.
Reuse the executable at
`target/yt133-private-hw52es_r/build/release/yetlet-production-starter`, SHA-256
`1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`,
without rebuilding. Freeze this contract, the reviewed script, exact commands,
environment/settings, tool hashes and source receipts before any app startup.

The historical YT-134 conservative 345-file workspace envelope now has **339
matches and six known differences** from accepted [YT-139](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md)
and [YT-140](YT-140-OIDC-PROXY-REPLICAS.md):
Admin `tests/auth_valkey.rs` belongs to YT-139; YT-140 changed Admin
`src/config.rs`, `src/oidc.rs` and Board `src/lib.rs`,
`src/oidc.rs`, `src/oidc/tests.rs`. These separate applications are outside the
Starter's resolved local dependency closure. Read-only
`cargo metadata --offline --locked --manifest-path examples/production-starter/Cargo.toml --format-version 1`
exited 0; the closure contains Starter, config, core, HTML, server-axum,
request-id and Session, with all 35 corresponding recorded local inputs matching.
No full-345-match claim is made. Before/after execution must retain these exact
exceptions, match every relevant input and match the binary. Further differences
stop execution for review rather than authorizing a rebuild or old-proof rerun.

Both lockfiles match their original receipt: root `Cargo.lock` SHA-256
`2f5888dd8d60aca287b26a48a59e9c9b3f128623c24ecafc681af25e16b23b53`;
Starter `Cargo.lock` SHA-256
`ad3aae7584c61db19db936398e2d337b7dfdfd9539bc80687b26346658e7bc18`.
The old summary's lock hash describes the root file. `source-findings.json`
retains the exact paths/hashes, metadata identity, exceptions and accepted
YT-134/YT-136 receipts; the full current repository baseline is separate.

Use one native process, direct `127.0.0.1` HTTP on a new high port and a private
disposable directory under ignored `target/`, on the actual persistent ext4
mount. Record mount/source, free space, host/kernel/CPU and allowed CPU set;
refuse tmpfs. Copy unchanged `base.toml` and `production.toml`, select
`APP_ENVIRONMENT=Production`, fixture configuration root/database path/port,
fresh synthetic 64-hex Bearer token, `APP_METRICS__ENABLED=true` and
`TOKIO_WORKER_THREADS=2`. Retain logging level `info`. Record all effective
non-secret settings and environment names; do not inherit unrelated `APP_*`
overrides or proxy configuration. No feature comparison is selected.

Keep existing HTTP capacity 32 / five-second deadline, SQL immediate admission
capacity four / two-second SQLite busy timeout, startup WAL and schema 2.
Starter has no rate-limit middleware; record 429 even though none is expected.
Use the real Bearer-protected endpoint without Origin, cookies or redirects;
this preserves the API's existing browser-origin rejection. Do not weaken
authentication, CSRF, admission or data integrity. No browser/SSR mutation is
selected, so no SSR Session/CSRF fixture or bypass is needed.

Seed exactly 20 issues through authenticated `POST /api/issues`, outside timed
phases, without retries. For decimal IDs 01–20 use title `YT141 seed NN` and
body `controlled-arrival read fixture`; retain the exact UTF-8 payloads,
acknowledged IDs and full expected rows. Require 20 status-201 acknowledgments,
status `open`, version 1 and IDs/sequence 1–20. A failed or ambiguous seed stops
the workload and requires stopped-data reconciliation. Inspect the complete
schema/index/rows/sequence before measurement, and compare a valid authenticated
`GET /api/issues` response with those rows in newest-first order. Retain its
exact expected response bytes locally; no token/body is placed in load records.

Use uniquely owned transient user units, with no persistent unit installation:
app CPUQuota 200%, MemoryMax 2 GiB, TasksMax 128, LimitNOFILE 512, Nice 10,
Restart=no and 20-second stop bound; client plus monitor CPUQuota 200%,
MemoryMax 512 MiB, TasksMax 64 and LimitNOFILE 512. Select separate pairs of
currently allowed logical CPUs for app and client, record topology/affinity and
shared-host interference, and stop if this placement is unavailable. Bound
each output file to 64 MiB and total retained run output to 256 MiB, with an
explicit stop/gap on cap or sink failure. These are fixture controls, not
consumer hardware requirements. Do not change existing services or host limits.

## Fixed workload and populations

Every k6 iteration issues exactly one `GET /api/issues`, consumes the full body,
then ends. Use `constant-arrival-rate`, `timeUnit=1s`, 16 preallocated VUs and
`maxVUs=16`, an eight-second HTTP timeout, zero redirects and no retries, sleep
or additional request. Keep HTTP/1.1 connection reuse enabled within a phase;
retain actual response protocol and abort comparison if it differs. Tokens are
read from a private file, never command arguments, tags, raw metric metadata or
logs. Only the fixed loopback URL is accepted; disable HTTP debug output.

Run each row as a separate sequential k6 invocation. Allow `gracefulStop=8s`
to complete in-flight requests, wait for that process to exit, then observe
one second idle before starting the next invocation. Never overlap phases.
Record actual launch gaps, drain durations and interrupted iterations. Separate
processes do not preserve connection pools: warmup primes the application/cache,
and each phase includes its own connection establishment. This is not a fully
prewarmed-connection measurement or an immediate post-overload recovery bound.

| Order | Phase | Arrivals/s | Scheduled seconds | Planned arrivals |
| --- | --- | --- | --- | --- |
| 1 | Warmup, excluded from measured aggregates | 100 | 5 | 500 |
| 2 | Repetition 1 baseline | 100 | 10 | 1,000 |
| 3 | Repetition 1 higher rate | 500 | 10 | 5,000 |
| 4 | Repetition 1 recovery | 100 | 10 | 1,000 |
| 5 | Repetition 2 baseline | 100 | 10 | 1,000 |
| 6 | Repetition 2 higher rate | 500 | 10 | 5,000 |
| 7 | Repetition 2 recovery | 100 | 10 | 1,000 |

There are 65 scheduled active seconds and 14,000 planned measured arrivals,
plus 500 warmup arrivals, bounded drain, idle gaps, startup and cleanup.
Do not add a rate, extend a phase or repeat an accepted run to find saturation.
Sixteen VUs impose a generator limit: slow requests can exhaust them and drop
arrivals. Report that observation without attributing all drops to server
capacity. Fixed rates, finite duration and shared resources do not establish
a maximum, steady-state capacity or a statistically general distribution.

For each attempt retain phase, start/completion time, status, numeric k6 error
code, full-call elapsed milliseconds and body-valid boolean. A successful
attempt is status 200 with bytes exactly equal to the prechecked 20-row body.
Record invalid-200 separately. Use narrowly scoped counters/tags, excluding
headers, credentials and bodies. Raw JSON must preserve individual custom
Trend points and attempt outcomes, not merely final summary percentiles.

Measure full-call elapsed around `http.get` through receipt of the complete
body. Explicitly report its millisecond clock resolution and wall-clock limits.
This includes client connection/blocked time that k6's
[built-in HTTP timings](https://grafana.com/docs/k6/latest/using-k6/metrics/reference/)
do not all include in `http_req_duration`. Retain built-in timings separately.
Report nearest-rank p50/p95/p99 for all completed attempts, including timed-out
or failed calls with their elapsed time, and separately for validated 200s.
Also identify the HTTP-response population versus no-response transport errors.
Dropped/never-started and force-interrupted iterations have no completed
full-call latency sample: retain their counts and never treat them as fast
successes or claim intended-arrival-corrected latency as wrk2 measures it.

Report planned/offered rate, actual started attempts, dropped iterations,
completed attempts, validated successes and actual wall time. Show achieved
starts and completion/success throughput over the scheduled window separately
from completion/success throughput over the window through final drain. Do not
divide a count including drain completions by ten seconds without labeling it.
Retain warmup and both repetitions independently; no pooled-only result hides
a failed point. Reconcile planned arrivals against starts/drops and explain any
boundary or interrupted-iteration difference from native k6 counts.

## Sampling, thresholds and completion

Reuse accepted [YT-136 telemetry semantics](YT-136-TELEMETRY-COLLECTION.md) and
its bounded loopback snapshot command; do not rerun disabled/down/privacy or
SQL fault proofs. Take non-overlapping metrics/readiness samples about once per
second with the existing connect/total/watchdog and size bounds, including
before/after phase snapshots. Record actual sample count/time and failures;
failed samples are gaps, never stale evidence of current health. Keep probes
outside workload totals while accounting for their effects on HTTP/SQL counters.

Sample app and actual k6 PID CPU/RSS/threads/FDs, client/app cgroup CPU and
throttling, database/WAL sizes and final active gauges. Report CPU deltas with
one core equal to 100%, phase mean and sampled RSS peaks for both sides, and
generator throttling/monitor interference. Timing/peaks are sampled, not
continuous. Existing server HTTP timing stops at response creation, excluding
body delivery; SQL permit-wait is zero because there is no queue, not evidence
of zero SQLite/blocking-pool wait. Counter resets and nontransactional snapshots
retain their accepted limits. No cross-host clock alignment is involved.

Predeclared local investigation bounds, applied to each measured phase:

- Zero dropped or interrupted iterations, 429, 503, 504, timeout, transport
  error, invalid-200 or other unexpected status; expected response is 200.
- All-completed-attempt p95 at most 100 ms and p99 at most 250 ms; separately
  publish successful-only p50/p95/p99 and all-attempt p50. No samples means
  unavailable, not a passing percentile.
- Mean app CPU at most 200%, sampled peak app RSS at most 128 MiB, all sampled
  readiness 200 and no unexplained sampling/output gap. Client CPU/RSS and
  throttling remain explicit; observed client saturation or drops prevent an
  application-only capacity interpretation.
- Each recovery phase is assessed independently against these same bounds.
  Failure is retained and excluded from the passing observations. Natural
  saturation, if absent, is reported absent; it is not manufactured by more load.

A threshold failure alone does not authorize optimization, new middleware,
policy relaxation, a rerun or altered thresholds. Stop for unexpected fixture
mutation, unsafe resource growth, resource/output cap, source mismatch or
process failure; retain partial/failed observations and perform cleanup.

After clients stop, require final active HTTP/SQL gauges zero and observe five
seconds idle. Take SIGTERM through native shutdown; require exit 0 and `stopped`
within the 20-second supervisor bound. Inspect the stopped database and require
the exact same 20 acknowledged rows, schema-2 DDL/status index and sequence 20,
zero extra/missing/changed rows and zero unknown write outcomes. Report zero
measured writes explicitly; read-only load does not qualify mutation durability.
Do not substitute file-byte identity for logical no-write reconciliation.

Retain raw commands, release/config/script/tool/source hashes, request/metric
points, resource samples, reconciliation and stop results before removing only
owned disposable data, credentials, tool extraction/archive and transient units.
Reap every child, verify PIDs absent, descriptors closed, units inactive and
listener refused, and verify fixture/token removal. Preserve a failure's
evidence and cleanup even when no passing point is available.

Reference Done requires the minimal reusable script/docs, source-bound bounded
observations with failures and limitations, reconciliation, owned cleanup and
final independent Reviewer PASS. It does not require either rate to pass.
Compile, browser and CSP results remain historical; no new tests/builds,
browser/CSP matrix, SQL lock, backup, proxy, identity, Windows or release
qualification is selected. No commit, push or production operation is authorized.

## Evidence and current checkpoint

Run-local evidence root:
`/tmp/yetlet-yt141-performance-20261023-m5f1hvgz`.
`source-findings.json`, `starter-cargo-metadata.json`, `tool-comparison.json`
and `k6-release-metadata.json` bind the contract research. The exact accepted
release binary hash matched; the 339/345 receipt result and six unrelated
exceptions are explicit. Metadata inspection was read-only and exited 0.
No load tool was downloaded, app/service started, runtime workload executed,
package installed, source rebuilt or external environment changed in stage 1.
Independent contract Reviewer `/root/yt141_arrival_contract_review_20261023`
returned PASS with zero issues and required followups. Its six reviewed hashes,
`contract-review-final.json`, `contract-acceptance.json` and immutable
`contract-reviewed-document.md` preserve the pre-runtime contract. The later
implementation and observations below passed their separate final independent review.

## Actual bounded execution

The only implementation is the copyable [read-only k6 script and guide](../examples/production-starter/deployment/performance/README.md).
`preexecution.json` binds the initial exact commands/settings, reviewed contract,
source, tool and script before startup. The executed script SHA-256 is retained
per phase in `runtime-result.json`; it was unchanged through all seven phases.
The archive matched the official checksum; the extracted executable reported
`k6 v2.3.0 (commit/e088784614, go1.26.8, linux/amd64)`, SHA-256
`e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260`.
No system installation, rebuild, framework/runtime change or old workload rerun
occurred. Before/after release and source receipts match the frozen baseline,
including the same six disclosed unrelated YT-139/YT-140 exceptions.

Execution ran from 11:26:47.970 to 11:28:07.042 UTC on 2026-10-23, including
startup/seeding, idle gaps and stop, on Linux 7.0.0-38 / Intel i5-13600K. The
fixture used `/dev/nvme0n1p2` ext4, high loopback port 39213, app CPU affinity
0/1 and client/monitor affinity 2/3, with the declared transient-unit limits.
The separate CPU pairs share the same host, scheduler and storage/cache;
logical CPU placement is not separate-host isolation.

All **14,006 measured attempts** completed as exact-body HTTP 200, versus
14,000 nominal planned arrivals. Warmup separately completed 501 validated
responses versus 500 nominal. Native k6 starts/completed iterations and custom
completed points agree. Each phase has `planned - started - dropped = -1`;
its native boundary extra is retained, never rewritten to nominal. Achieved
starts per scheduled second were 100.1 or 500.1 for the measured phases.
There were zero dropped/interrupted iterations, 429/503/504, timeouts,
transport errors, invalid-200s or unexpected responses. All responses were
HTTP/1.1. No failed measured point was discarded or replayed.

| Measured phase | Offered/s | Actual starts / completed / valid 200 | All completed p50 / p95 / p99 ms | Valid-200 p50 / p95 / p99 ms | Investigation outcome |
| --- | --- | --- | --- | --- | --- |
| Repetition 1 baseline | 100 | 1,001 / 1,001 / 1,001 | 1 / 2 / 2 | 1 / 2 / 2 | PASS |
| Repetition 1 higher | 500 | 5,001 / 5,001 / 5,001 | 1 / 2 / 4 | 1 / 2 / 4 | PASS |
| Repetition 1 recovery | 100 | 1,001 / 1,001 / 1,001 | 1 / 2 / 2 | 1 / 2 / 2 | PASS |
| Repetition 2 baseline | 100 | 1,001 / 1,001 / 1,001 | 1 / 2 / 2 | 1 / 2 / 2 | PASS |
| Repetition 2 higher | 500 | 5,001 / 5,001 / 5,001 | 1 / 2 / 4 | 1 / 2 / 4 | PASS |
| Repetition 2 recovery | 100 | 1,001 / 1,001 / 1,001 | 1 / 2 / 2 | 1 / 2 / 2 | PASS |

Warmup p50/p95/p99 was 1/2/2 ms and is excluded from measured aggregates.
The all-completed, HTTP-response and validated-success populations coincide
here because there were no errors; the separate raw Trends remain retained.
These are nearest-rank full-call values at millisecond wall-clock resolution,
not high-resolution server latency or intended-arrival-corrected quantiles.
Raw `start_ms` is an individual actual call start; `scheduled_start_ms` is
`exec.scenario.startTime`, the **whole phase anchor**, not that request's
intended arrival. This meaning was recorded before app startup in
`metadata-semantics-before-start.json`. Native HTTP timing remains separate.

All attempts completed, but some completions lie beyond the scheduled window.
Scheduled-window completion/success rates count only completions inside the ten
seconds. The following separate denominator extends to the last completion;
both completion and success rates are equal in this run:

| Phase | Completed/s inside scheduled window | Window through last completion, s | Completed/s through last completion | App mean CPU / sampled peak RSS MiB | k6 mean CPU / sampled peak RSS MiB |
| --- | --- | --- | --- | --- | --- |
| Repetition 1 baseline | 100.00 | 10.003 | 100.07 | 8.96% / 8.21 | 11.50% / 45.82 |
| Repetition 1 higher | 500.00 | 10.001 | 500.05 | 37.75% / 8.58 | 40.46% / 47.50 |
| Repetition 1 recovery | 100.00 | 10.002 | 100.08 | 9.04% / 8.68 | 11.04% / 45.39 |
| Repetition 2 baseline | 100.00 | 10.002 | 100.08 | 9.14% / 8.69 | 10.96% / 45.76 |
| Repetition 2 higher | 500.00 | 10.003 | 499.95 | 37.36% / 8.70 | 39.98% / 47.36 |
| Repetition 2 recovery | 100.00 | 10.001 | 100.09 | 9.23% / 8.70 | 10.88% / 45.78 |

CPU uses one core = 100%. App CPU comes from process counter deltas; k6 CPU
comes from child user/system usage, each divided by invocation launch-through-
exit wall time (10.039–10.079 s for measured phases), including initialization/
completion overhead. Raw per-phase denominators are retained. Separate k6
processes reused connections within their own phase only. Both lower-rate
recovery observations passed, after the declared completion and idle gap.
No SQL/HTTP saturation, generator drops or app/client cgroup CPU throttling was
observed. No maximum rate, overload boundary or immediate recovery time was
established; the experiment was not escalated to manufacture one.

There were 81 successful fresh bounded metrics observations and 81 paired
sample readiness checks; each complete sample took 0.0256–0.0466 s, with zero
collection gaps. Including one successful startup readiness request, there
were 82 readiness HTTP 200s. One connection refusal while the app was starting
is retained outside workload counts. The final registry exactly accounts for
14,590 GET 200 = 14,507 workload (including warmup) + one fixture-list read +
82 readiness requests; 20 POST 201s are seeds. SQL acquired 14,610 permits for
these business/readiness operations. The 81 metrics scrapes exclude themselves
from HTTP counters while consuming client/app resources. No liveness workload
was added. Final sampled HTTP and SQL active gauges were both zero.

Sampled peak app RSS was 8.70 MiB, actual post-exec k6 RSS 47.50 MiB and monitor
RSS 161.01 MiB; the combined client cgroup peaked at a sampled 221.55 MiB under
its 512 MiB cap. Neither cgroup reported memory-limit or OOM events. App and
k6 sampled thread peaks were eight each; FD peaks were 27 and 24. Child `wait4`
high-water RSS also appears in the raw result, but includes fork-before-exec
inheritance from the Python monitor and must not be presented as isolated k6
RSS. All peaks are sampled except that differently scoped child high-water.
Sampled HTTP/SQL active peaks were one, which can miss brief bursts. Database
size was 16,384 bytes with zero WAL/SHM bytes at samples; this warm read-only
fixture does not establish physical IOPS, cold-cache behavior or flush latency.

All 20 seeds were acknowledged through the real Bearer API before measurement.
The full stopped rows, schema-2 DDL, status index and sequence 20 exactly equal
the pre-phase manifest and acknowledged synthetic rows. Measured writes were
zero; unknown, missing, extra or changed rows were zero. After 5.036 seconds
idle with active gauges zero, native SIGTERM produced `stopped` and exit 0;
the owned supervisor stop call took 0.00365 s, not an RTO commitment.

One **pre-workload harness failure** inspected the Python launcher's environment
before its `execve` into Starter completed. No HTTP request, seed or database
creation occurred. The owned process stopped; its PID/listener absence and
original runner/logs are retained in `preworkload-launch-inspection-failure/`.
One smallest correction moved that inspection after the existing readiness
wait. `harness-correction.json` and the exact diff bind the corrected runner
without replacing the original preexecution freeze. The same fixture, units,
script, settings, schedule and thresholds then executed once. No product repair
or measured-phase replay occurred; the initial failure line remains in the
combined unit log as well as the preserved failed attempt.

After native stop, all nine owned app/monitor/k6 PIDs were absent, descriptors
closed, both transient units inactive/unloaded with MainPID 0 and the listener
refused connections. The disposable database, configuration, home directory, token and downloaded
archive/executable directory were removed after evidence retention. The
synthetic token was absent from retained evidence. One redundant cleanup stop
of the already-unloaded app unit returned nonzero; its original inner exit/
stderr were not separately retained. The outer failure and this historical gap
are explicit; current unit/PID/port checks establish cleanup without retrying
that stop. No existing service or host configuration was modified.

`runtime-result.json`, per-phase raw JSON and native summaries/stdout/stderr,
`outside-workload-requests.json`, `samples.jsonl`, `collection/`, database
manifests, `reconciliation.json`, `reporting-projection.json`, source receipts,
`cleanup.json` and `implementation-summary.json` retain the exact commands,
identities, populations, resource observations, correction and terminal cleanup
observation. Raw download duration and the redundant-stop stderr were not
separately retained; no missing observation is synthesized. App logs are private,
not a new audit log. No new build/test, browser, CSP, cloud, proxy, identity,
Windows, durability or production qualification is claimed.

Final independent Reviewer `/root/yt141_performance_final_review_20261023` returned
**PASS**, zero issues and required followups. All eight input hashes and 1,000
outside-scope tracked files matched. The reviewer independently recomputed all
seven raw phase populations/quantiles and checked probe/resource accounting,
source/fixture equality, failure history and cleanup. `review-final.json`,
`review-input.json`, `reviewed-files.json` and `review.patch` preserve that
reviewed input. Only acceptance/status bookkeeping follows PASS; the executed
script is unchanged. YT-141 is **Done** at this bounded optional local reference
scope. No additional workload is needed. The remaining backlog gates are
YT-096 release-trust decisions and dependent YT-020; no new item is activated. Consumer deployment choices, YT-096 D1–D6,
YT-020 and immutable M20 evidence retain their separate status. No commit,
push, public release or production operation is authorized.
