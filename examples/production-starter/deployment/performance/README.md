# Optional controlled-arrival API reference — YT-141

This small script exercises the Starter's authenticated, read-only
`GET /api/issues` route. Read the [source-bound contract](../../../../docs/YT-141-PERFORMANCE-VALIDATION.md)
before using it. It adds no runtime dependency, required throughput, consumer
SLO or CI gate. Results apply only to the recorded fixture. Independent contract and final
reviews returned PASS, zero issues/required followups; YT-141 is Done at its
bounded optional local reference scope.

The selected tool is official k6 v2.3.0, obtained locally after independent
contract PASS. The Linux-amd64 archive SHA-256 is
`39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`;
verify it against the official release/checksum and record the executable hash.
Do not install a service or send results to a cloud backend. The documented
reference uses `--no-usage-report` and a clean process environment.

Use a disposable loopback Starter process with its real Bearer authentication,
unchanged admission policy, Production settings and metrics enabled. Seed
exactly 20 short synthetic issues through the API before measurement, reconcile
their acknowledged rows, then save a verified list response as the expected
body. Keep the 64-hex token in a private mode-0600 file. The script accepts only
a high loopback port; it does not accept a remote target. Do not put credentials
in shell arguments, output, tags or debug logs.

For **one** phase, select the previously verified executable and private file
paths. This example uses placeholders, not a credential value:

```bash
umask 077
env -i PATH=/usr/bin:/bin HOME="$PERFORMANCE_PRIVATE_HOME" \
  YT141_PORT="$PERFORMANCE_PORT" \
  YT141_RATE=100 YT141_SECONDS=10 YT141_PHASE=r1_baseline \
  YT141_TOKEN_FILE="$PERFORMANCE_TOKEN_FILE" \
  YT141_EXPECTED_FILE="$PERFORMANCE_EXPECTED_FILE" \
  "$K6_EXECUTABLE" --no-usage-report --address=127.0.0.1:0 run \
  --out "json=$PERFORMANCE_OUTPUT/r1_baseline.jsonl" \
  --summary-export "$PERFORMANCE_OUTPUT/r1_baseline-summary.json" \
  ./examples/production-starter/deployment/performance/read-api.js
```

Run under the contract's owned app/client CPU, memory, task, descriptor and
output limits. Freeze the script, source, binary, tool, settings and exact
commands before application startup. Do not use this command against existing
data or an unbounded target. Retain app logs privately and collect the existing
[bounded telemetry snapshots](../telemetry/README.md) separately from workload
counts. No automatic installation, fixture runner or cleanup framework is
provided by this reference.

Run seven sequential invocations in this fixed order:

| Phase | Rate/s | Seconds |
| --- | --- | --- |
| `warmup` | 100 | 5 |
| `r1_baseline` | 100 | 10 |
| `r1_higher` | 500 | 10 |
| `r1_recovery` | 100 | 10 |
| `r2_baseline` | 100 | 10 |
| `r2_higher` | 500 | 10 |
| `r2_recovery` | 100 | 10 |

Wait for each process to exit, including its eight-second graceful-stop bound,
then observe one second idle before launching the next. Connections are reused
within a phase; separate invocations start separate pools. Warmup primes the
app/cache, and its 500 planned arrivals are excluded from measured aggregates.
The six measured phases have 14,000 planned arrivals. Do not add load points or
rerun a failed point to find a favorable rate.

The executor schedules one GET per iteration at the chosen rate, using exactly
16 preallocated/max VUs, eight-second request timeout, no redirects or retries.
Slow requests can exhaust the generator's VUs. Preserve `dropped_iterations`,
started/completed/interrupted counts and actual timings; dropped work has no
completed latency sample. No intended-arrival latency correction is claimed.

`yt141_full_call_ms` stores one elapsed value per completed HTTP attempt,
including failures. Its raw JSON point includes start milliseconds as metadata,
completion timestamp, phase, status, numeric error code, response protocol and
body-valid flag. Metadata `scheduled_start_ms` names the entire scenario start
anchor from `exec.scenario.startTime`; it is not an individual intended arrival. `yt141_valid_200_ms` includes only exact-body status-200
responses; `yt141_started` counts attempts. Full-call timing uses `Date.now()`
at millisecond resolution around `http.get`, including receipt of the body;
it has wall-clock limits. Native `http_req_duration` excludes some connection/
blocked time and remains a separate population. No bodies or headers are
emitted. Preserve raw points rather than reporting only k6's final summary.

Calculate nearest-rank p50/p95/p99 from raw completed-attempt and validated-200
points separately. Report response versus transport-error counts, 429/503/504,
timeouts, invalid-200s, drops and interruptions. Keep scheduled-window rates
separate from rates over the window through final completion. Report both
repetitions, client/server CPU/RSS/throttling, probe/sample gaps and recovery,
including every failed investigation bound from the contract. The k6 summary
exit alone does not establish that these bounds passed.

After the fixed phases, require idle active gauges zero, clean native shutdown
and exact stopped schema/index/sequence/full-row equality with the 20 seeded
acknowledgments. There are no measured writes. Reap owned processes, stop owned
transient units, verify the listener closed, retain evidence, then remove only
the disposable dataset/token/tool directory. A failing phase still requires
reconciliation and cleanup. Existing YT-134 and YT-136 proof is reused at its
accepted scope; no production, SSR, browser, durability or maximum-capacity
qualification follows.

## Separately activated saturation and recovery — YT-142

[YT-142's reviewed contract](../../../../docs/YT-142-SATURATION-RECOVERY.md)
adds a finite progressive experiment above the accepted YT-141 rates. The
original phase/rate settings still select the original 16-VU single scenario;
they are not rerun for YT-142. The same GET function, private inputs, timeout,
connection reuse and canonical metrics serve both references.

Select `YT141_PHASE=yt142_warmup`, `YT141_RATE=250`, `YT141_SECONDS=5` for the
new cache warmup. Then select the following paired invocations in order, stopping
after the first failed or generator-limited point and its recovery:

| `YT141_PHASE` | `YT141_RATE` | `YT141_SECONDS` |
| --- | --- | --- |
| `yt142_1000` | 1000 | 10 |
| `yt142_2000` | 2000 | 10 |
| `yt142_4000` | 4000 | 10 |
| `yt142_8000` | 8000 | 10 |
| `yt142_16000` | 16000 | 5 |

For example, substitute those three environment values in the one-invocation
command above and use unique output filenames. Each paired mode configures
native k6 `load` and `recovery` scenarios. Load has 64 fixed VUs; recovery starts
at the exact load duration at 250/s for thirty seconds, with sixteen fixed VUs.
Both have eight-second graceful stop. At most eighty VUs exist; load calls can
finish during early recovery. Native `scenario` tags distinguish the two
populations while `phase` identifies the invocation. Do not merge them or
replace scenario timing with a custom scheduler. Connections belong to each
VU pool; recovery has a fresh pool.

Use the contract's unchanged app 200% CPU budget and separate client/monitor
600% CPU, six logical CPUs, 1 GiB memory, task/FD limits and finite watchdog.
Output is bounded to 512 MiB/file and 2 GiB total with at least 4 GiB free.
Retain raw JSON, but stream analysis and keep only needed points in memory.
No midpoint, repeated rate, resource escalation, automatic retry, policy change
or optimization is selected. A dropped iteration, client cap or output gap
prevents an application-only offered-rate capacity claim.

Use timed samples **inside the load window** for load CPU/throttling and
admission attribution; a forty-second load-plus-recovery CPU average would
dilute the load. Publish the actual sampled interval and its boundary limits.
Keep all/successful/HTTP-response latency, native boundary extras, planned/
started/completed/dropped/interrupted counts, probes and carryover separate.

For recovery, publish all thirty one-second start bins and the first ten
consecutive healthy bins defined by the contract. Distinguish the first healthy
bin start, ten-bin window end and actual confirmation time when required
completions/probes are available, all relative to the load boundary. Retain
initial unhealthy seconds, full-window failures and the maximum thirty-second
scheduled/eight-second-drain observation. Do not extend a failed recovery.

The same twenty real API seed acknowledgments, full stopped logical-data
reconciliation, native shutdown and owned process/unit/listener/token/tool
cleanup remain required. Observe a transient unit's actual state before a stop;
an already-unloaded inactive unit needs no repeated stop. Preserve exact command
exits/stdout/stderr/time, every failed load point and any measurement limitation.
YT-142 is Done at bounded optional observation scope after final independent
Reviewer PASS, zero issues/required followups. The 1,000/s load point remains
FAIL; acceptance does not establish an exact maximum or a new passing bracket.
