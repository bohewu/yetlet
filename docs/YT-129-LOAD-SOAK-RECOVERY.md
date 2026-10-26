# YT-129 — Bounded local load, soak and recovery

Date: 2026-10-23. Status: **Done at bounded local B1 scope after independent Reviewer PASS, zero issues and required followups**.
Scope: original Phase 2 handoff Section 3 B1, canonical T3; accepted YT-127/A1
and YT-128/A2 precede this run. No runtime source, configuration, dependency,
security policy or application contract changed. This is local workload evidence.

## Source and environment

Branch `docs/yt113-123-opt-in-capability-plan`, HEAD
`5db8b8c61b6d319a641814dff4af493ce3e5d965`, with accepted T0/T1/T2 dirty work
preserved. Binary is the accepted A2 **debug** build:
`examples/production-starter/target/debug/yetlet-production-starter`, SHA-256
`4dc7dec913722dbcf86a8677790e69d20fa27a3676e855e31fd9f238a40fcb29`.
All Starter runtime/configuration/lock/test hashes matched A2 before execution
and remained unchanged afterward. Retained source/environment/settings records
include exact hashes, dirty state and pre-task patch; no checkout, staging,
commit, push, release or deployment occurred.

Host: Linux 7.0.0-38-generic x86_64, glibc 2.43, Intel i5-13600K, 20 logical CPUs
available, 66,072,690,688 bytes RAM, Python 3.14.4 and Rust 1.99.0. The repository
is on ext4; **the temporary database is on `/tmp` tmpfs**, so this does not measure
persistent-disk latency or durability. Client, monitor and server share the host.
No CPU quota or memory limit was imposed by this command. Full OS inventory is
retained. The server selects rusqlite 0.40.2 with bundled libsqlite3-sys 0.38.2
(vendored SQLite header 3.53.2). The raw `actual_SQLite.sqlite_version` field
is the separate Python inspection/lock-holder connection's SQLite 3.46.1, not
a query of the server's linked engine. Inspection found WAL, schema version 1
and 4096-byte pages.

Settings use existing `base.toml` plus `production.toml`, `APP_ENVIRONMENT=Production`,
an ephemeral `127.0.0.1` port, a fresh temporary absolute database path, a synthetic
64-character API token and `APP_METRICS__ENABLED=true`. Other settings remain
unchanged: HTTP capacity 32, SQL capacity 4, both permit queues zero, HTTP response
creation deadline 5 seconds, SQLite busy timeout 2 seconds, readiness deadline
1 second, shutdown grace 10 seconds, INFO JSON logs and process-local Sessions
with 1800-second lifetime / capacity 1024. Metrics default remains false outside
this explicitly enabled instance. No Session/Auth/CSRF/limits policy changed.

## Declaration made before execution

The environment, source/binary identity, schedule and thresholds were persisted
in `observations/b1/source-environment-settings.json` and
`observations/b1/predeclared-workload-thresholds.json` before server startup.
These thresholds are local investigation choices, not user guarantees. They
were not adjusted after results.

One worker cycles ten requests: 40% authenticated API detail, 30% authenticated
API list, 20% SSR detail and 10% API create, with fixed short synthetic values.
Each worker starts the same cycle; deadline truncation slightly changes realized
percentages, which are retained. Twenty rows are seeded once through HTTP. SSR
reuses one warmed Session cookie. The list endpoint returns at most 100 records;
write volume grows the database without increasing that response bound.

Every request opens a fresh loopback HTTP/1.1 connection, consumes the full body
and closes it; there is no connection reuse or automatic write retry. Closed-loop
workers pause after each response. Client timeout is 8 seconds. Health/metrics
and Linux `/proc` CPU/RSS/FD/thread/TCP samples run approximately once per second;
health probes are excluded from workload counts but included in app counters.

| Phase | Clients | Target seconds | Pause after each response |
|---|---|---|---|
| Baseline | 1 | 10 | 20 ms |
| Step 2 | 2 | 12 | 10 ms |
| Step 4 | 4 | 12 | 5 ms |
| Step 8 | 8 | 12 | 2 ms |
| Sustained | 2 | 45 | 10 ms |
| Forced overload | 16 plus four dedicated blocked writes | 6 | 10 ms |
| Recovery | 1 | 10 | 20 ms |

Normal baseline/sustained/recovery require zero rejections, transport errors or
unexpected statuses, p95 <=100 ms, p99 <=250 ms and all sampled readiness probes
200. Minimum throughput is 20 requests/second for baseline/recovery/steps and
50 for sustained. Steps use the same normal-load latency/error criteria to
identify excluded load levels. Mean server CPU must be <=200%, where one full
core is 100%; peak RSS <=128 MiB and sustained last-ten minus first-ten sample
mean RSS <=16 MiB. Sampled active SQL/HTTP must stay <=4/32.

The forced fault must produce 503, reach four active SQL operations, keep metrics
and liveness 200, make saturated readiness 503, then regain readiness within
3 seconds after lock release. Recovery must pass the normal criteria. Final
active gauges must be zero; after 12 seconds idle, FD/thread counts must be
within +4 of the warmed baseline. Shutdown must exit 0 within 10 seconds, with
all owned processes/clients/pools/sampler/data/listener cleaned.

## Measured results

One execution ran from 2026-10-23T04:16:43.124856+00:00 to 2026-10-23T04:18:42.689387+00:00,
including startup, warmup, load, idle observation and shutdown. There were 42,043
workload requests. No operational retry, product repair or duplicate workload
run was needed. Execution exit 0 means the command completed; the Step 8 threshold
failures below remain failures.

Quantiles are nearest-rank client full-body round-trip milliseconds, excluding
intentional pauses, over all workload responses including 503. Successful-only
quantiles and every request's latency/status are separately retained. CPU is the
mean of server process CPU deltas between periodic samples within the phase;
RSS/active peaks are sampled and may miss shorter spikes.

| Phase | Actual seconds | Requests | All / successful rps | p50 / p95 / p99 ms | 503 rate | Mean CPU | Peak RSS MiB | Declared outcome |
|---|---|---|---|---|---|---|---|---|
| baseline | 10.01 | 437 | 43.64 / 43.64 | 2.87 / 3.68 / 4.05 | 0.000% | 12.75% | 11.66 | PASS |
| step2 | 12.01 | 1,853 | 154.26 / 154.26 | 2.62 / 4.39 / 6.00 | 0.000% | 43.02% | 12.48 | PASS |
| step4 | 12.01 | 5,607 | 467.01 / 467.01 | 3.26 / 5.90 / 6.80 | 0.000% | 146.53% | 13.43 | PASS |
| step8 | 12.00 | 20,470 | 1705.20 / 1679.37 | 2.28 / 5.02 / 6.56 | 1.514% | 341.73% | 15.45 | FAIL: rejection / CPU |
| sustained | 45.01 | 6,890 | 153.08 / 153.08 | 2.67 / 4.53 / 5.80 | 0.000% | 43.45% | 15.45 | PASS |
| overload | 6.18 | 6,359 | 1029.50 / 22.02 | 2.25 / 4.43 / 6.64 | 97.861% | 153.89% | 15.45 | Expected fault |
| recovery | 10.02 | 427 | 42.60 / 42.60 | 3.22 / 4.89 / 5.24 | 0.000% | 14.13% | 15.79 | PASS |

All phases had zero transport errors and zero unexpected statuses. Normal
baseline, Steps 2/4, sustained and recovery had zero rejections and all sampled
readiness 200. Step 8 had 310 HTTP 503 responses (1.514%) and mean CPU 341.73%,
failing both the zero-rejection and <=200% CPU criteria. Its p99 passed; latency
alone would hide saturation. Snapshot deltas show 310 SQL saturated admissions,
zero unavailable SQL completions and zero HTTP-capacity rejections during Step 8.
These are SQL admission outcomes, not rate limiting.

Across the run, sampled SQL active peaked at 4 and HTTP active at 6. The HTTP
32-permit boundary was not reached or capacity-qualified here. Sampled RSS peaked
at 15.79 MiB. Sustained last-ten minus first-ten sample mean RSS was +0.003125 MiB,
below the 16 MiB threshold; this short interval does not establish long-term leak
freedom. The Step 8 CPU failure remains visible despite passing memory thresholds.

### Forced overload and recovery

A separate Python SQLite connection held `BEGIN IMMEDIATE`. Four dedicated API
creates filled all four SQL permits; the saturation snapshot confirmed SQL active
4, metrics/liveness 200 and readiness 503. The sixteen-client mixed phase then
ran with that lock held. It returned 6,223 workload 503 responses out of 6,359
requests (97.861%); 136 reads succeeded, and no create succeeded. All four dedicated
writes also returned 503. Metrics/liveness remained available and all six sampled
fault readiness results were 503.

The overload snapshots report 6,221 saturated SQL admissions and 12 unavailable
SQL completions, including dedicated writes and control probes beyond workload
counts; these are different populations. HTTP-capacity rejection delta was zero.
Client overload p99 was only 6.64 ms because most rejections were immediate; the
maximum request took 2008.28 ms, consistent with admitted writes waiting on the
separate two-second SQLite busy timeout. SQL permit wait sum remained **zero**
throughout because permit admission never queues. It is not SQLite lock wait,
blocking-pool scheduling, or proof of no storage contention. No standalone lock
wait histogram is provided by A2; the injected lock, operation outcomes and raw
request timing establish this fault's distinction.

After all workload and dedicated clients completed, the lock was rolled back.
The first readiness probe returned 200 in 0.760 ms (below the declared 3 seconds),
and the ten-second recovery phase passed with zero errors/rejections. HTTP/SQL
active gauges returned to zero. Fresh inspection found exactly 3,554 rows,
equal to 20 seeds plus observed successful creates, with schema version 1. This
readback checks this synthetic scenario; HTTP success and metric/span completion
are not a general durable-write acknowledgment. No unknown-result write was retried.

## Capacity interpretation and limits

The tested passing normal points span **43.64 to 467.01 requests/second** at
one, two and four clients with their stated pacing. The 45-second sustained
point is **153.08 requests/second at two clients**, p99 5.80 ms. The eight-client
point at 1705.20 total / 1679.37 successful requests/second fails the declared
normal rejection/CPU thresholds. There is no measured guarantee between these
points and no established hard upper capacity bound. Concurrency and pacing
both change across steps; throughput is achieved closed-loop traffic, not a
proven open-loop arrival-rate limit or production sizing recommendation.

This is a short local soak on a debug binary and tmpfs with small synthetic
records and a bounded list response. Release optimization, persistent disk/fsync
latency and failures, larger records, cold storage, TLS/proxies/WAN, connection
reuse, slow clients/bodies, transport exhaustion, multi-hour leaks, multi-host
HA and real deployment contention remain uncovered. No HA/SLA, persistent-disk
qualification or blanket production-readiness claim follows. No new architecture,
security/data policy or external resource decision was required.

## Resource release and verification

After 12 seconds idle, the process had 10 FDs (warmed baseline 10), 26 threads
(warmed baseline 22; +4 meets the declared bound), zero server-established TCP
connections and zero active HTTP/SQL operations. Retained runtime threads are
not assumed to disappear while the process lives. Approximate one-second sampling
and OS counters cannot establish every transient connection/task lifetime.
Per-request connections close; worker pools join before phase completion. The
Python inspection connections terminate with the one-shot client process; none
survive its successful exit. No standalone monitor/collector/exporter was started.

SIGTERM completed cleanly in 3.17 ms, exit 0; logs contain `started`,
`shutdown_requested`, `stopped`. PID 1350506 was reaped and absent. The sampler
joined, all client pools joined, temporary SQLite/WAL/SHM directory was removed
and the loopback listener refused a final connection. Kernel TIME_WAIT sockets
from intentionally fresh connections are not retained server FDs or active tasks.

All raw evidence is under
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/b1/`:

- `source-environment-settings.json`, `predeclared-workload-thresholds.json`,
  `before.patch`, `owned-files-before.json`: pre-execution identity/declaration
  and the pre-edit document snapshot.
- `exact-workload-command.txt`: exact one-shot Python standard-library/HTTP/OS
  command, runnable from the repository root with Python 3; no install or new
  executable harness/framework. A reproduction creates fresh synthetic data
  and a different ephemeral port; do not rerun merely to restate this evidence.
- `workload.log`, `requests.jsonl`, `os-health-samples.jsonl`, `server.jsonl`,
  `metrics-*.prom`, `phase-results.json`, `run-result.json`: raw observations,
  phase aggregates, business readback and teardown.
- `threshold-analysis.json`: unchanged declared thresholds evaluated against raw
  results, retaining Step 8 rejection and CPU failures.

`cargo xtask check docs` and `git diff --check` results are recorded in T3's task
result and retained logs. Runtime fmt/Clippy/build/product tests are **reused from
accepted A2**, not rerun: source/configuration/lock/test/binary hashes were equal
before/after this docs-and-observation-only task. A2's 11 unit tests, corrected
11-thread HTTP suite, strict Clippy, build, format and boundary checks retain
their original scope. No new compile, browser, CSP, Windows or historical provider/
asset qualification was performed. The live workload adds this source-bound
local HTTP capacity/recovery evidence only.

Changed repository files: this record, `examples/production-starter/README.md`,
and YT-129 entries in `docs/BACKLOG.md`, `docs/ROADMAP.md` and
`docs/PRODUCTION-READINESS-PHASE2.md`. The README also reconciles its stale A2
pending sentence with parent-confirmed independent acceptance; no A2 behavior
or evidence was changed. YT-129 is Done after independent review and controller acceptance. YT-130 and
YT-131 have subsequently been accepted at their bounded scopes. The next remaining
decision is **YT-096 D1–D6**, Pending; YT-096 remains In progress, YT-020 Blocked
and candidate `ce56e08` unchanged.

Controller acceptance follows independent Reviewer PASS, zero issues and required followups, recorded at `.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T3/independent-review.json`. Reviewed sources stayed unchanged; subsequent status edits record acceptance only.
