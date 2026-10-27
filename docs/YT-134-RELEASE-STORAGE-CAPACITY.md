# YT-134 — Optional release and persistent-storage capacity reference

Date: 2026-10-23. Status: **Done at bounded optional release/storage reference scope**.
Independent Reviewer acceptance: **PASS**, zero issues and required followups. This is an opt-in application
reference and developer workload guide, not a framework default or qualification
of a consumer's hardware, deployment, SLO or service commitments.

## Authority and bounded question

The user authorized YT-134 after the accepted [YT-133 reference](YT-133-STARTER-SINGLE-INSTANCE-CONTRACT.md):
first establish a release/persistent-storage workload contract, then perform
minimal isolated verification, reuse unchanged accepted evidence, and require
independent Reviewer PASS before Done. No Commit or Push is authorized.
The [M23 responsibility boundaries](PRODUCTION-DEPLOYMENT-PROFILES.md) apply.

The question is which tested load points satisfy predeclared investigation
thresholds for **this exact Starter release binary, schema-2 fixture and resource
budget** on local persistent storage, and whether a lower load recovers after
the highest tested concurrency. Report excluded points and measurement limits.
No universal requests/second claim, production hardware requirement, long soak,
power-loss durability, infrastructure installer or benchmark framework follows.

## Reuse and fixture

Use the YT-133 release executable without recompilation only after all 345
recorded build inputs and its SHA-256 match the accepted evidence. Expected
binary SHA-256:
`1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`.
Bind the exact current Git/source, lockfile, binary, copied configuration, schema,
dataset, filesystem/mount and resource identities before startup. Reuse YT-127
admission, YT-128 telemetry, YT-131 schema/compatibility and YT-133 private access/
lifecycle evidence at their original scopes. [YT-129](YT-129-LOAD-SOAK-RECOVERY.md)
remains historical debug/schema-1/tmpfs evidence, including its Step 8 FAIL;
this experiment neither reruns it nor retroactively qualifies it.

Use one isolated native application process, a new high loopback port and a
private disposable database directory under the repository's ignored `target/`
on its actual ext4 mount. Record mount source/type and free space; refuse tmpfs.
Use the existing Production settings with only fixture port/path, an ephemeral
64-hex API token, `APP_METRICS__ENABLED=true` and `TOKIO_WORKER_THREADS=2` explicitly
selected. Metrics is enabled only for measurement; unchanged default-off proof
is reused. No rate-limit middleware is installed in Starter. Do not add a
rate-limit bypass or new setting for this experiment.

Use an owned transient user unit: CPU quota 200% (one core is 100%), memory
maximum 2 GiB, TasksMax 128, LimitNOFILE 512, Nice 10, Restart=no and a 20-second
supervisor stop bound. Bound the owned client/monitor process as well, with at
most two CPU cores, 512 MiB and 64 tasks. These are reproducibility/resource
controls, not consumer hardware requirements. Do not alter existing services,
unit installations, global configuration, real credentials or retained data.

Measure direct loopback application HTTP, **excluding Nginx and SSH throughput**.
The accepted private edge remains an optional separate integration; repeating
its access/control-path tests would not answer this capacity question. Client,
monitor and server share the host, CPU scheduler, filesystem and page cache.
Record this interference and cgroup throttling. Persistent filesystem placement
does not prove stable-media flush latency, crash durability or production IOPS.
Retain the actual store pragmas/source semantics without treating an inspector's
SQLite version or per-connection settings as the server's measured engine state.

## Workload declaration

Seed exactly 20 short synthetic issues through the existing authenticated HTTP
API and warm one anonymous SSR Session before measured phases. Do not insert
rows directly or weaken Bearer/Origin/CSRF, SQL, deadline or admission policy.
Use the existing bounded list endpoint (at most 100 rows). Every worker repeats
ten operations in order: four authenticated API detail reads, three API list
reads, two SSR detail reads with the warmed cookie, then one authenticated API
create. Use fixed short synthetic fields and unique write markers; record their
exact lengths and seed identities before the measured phases. Workers begin
their cycles together, so synchronized bursts and deadline truncation affect
realized ratios; report actual counts, not just the nominal 40/30/20/10 mix.

Each operation opens a fresh HTTP/1.1 connection, consumes the full body and
closes it, with an 8-second client timeout and **no automatic retries**. Workers
are closed-loop: the next request starts only after the previous response and
the declared pause. This measures achieved throughput at tested concurrency and
pacing, not a fixed offered arrival rate or an open-loop maximum capacity.

| Phase | Workers | Target seconds | Pause after each response |
|---|---|---|---|
| Baseline | 1 | 8 | 20 ms |
| Step 2 | 2 | 10 | 10 ms |
| Step 4 | 4 | 10 | 5 ms |
| Step 8 | 8 | 10 | 0 ms |
| Sustained reference | 2 | 20 | 10 ms |
| Recovery | 1 | 8 | 20 ms |

This is 66 scheduled seconds plus setup, in-flight completion, idle observation
and cleanup, not a long-duration soak. No forced database lock, old fault matrix,
browser/CSP, Windows, IdP, Valkey, backup/restore or deployment test is repeated.
The highest point may expose natural admission/resource saturation; if it does
not, report that no saturation boundary was established and do not escalate
load merely to manufacture a limit. Stop on unsafe resource growth or fixture
failure, retain partial evidence and do not tune thresholds after seeing results.

## Measurements and predeclared interpretation

Persist this contract's hash, exact command/settings/environment and the phase
schedule/thresholds **before starting the application or workload**. Retain raw
per-request phase/operation/status/full-body elapsed time without tokens, cookies,
CSRF values or request/response bodies. Keep acknowledged create IDs and unique
markers separately for final readback. Record transport errors and ambiguous
writes as failures requiring reconciliation; never retry a mutation blindly.

Approximately once per second collect private liveness/readiness, existing
bounded metrics, process CPU/RSS/thread/FD counts, process storage-I/O counters,
cgroup throttling and database/WAL sizes. Report timing and actual sample counts;
sampling may miss peaks. Health/metrics are excluded from workload counts but
can contribute to application counters and contend for resources. Record
before/after storage sizes and counters; cache/writeback prevents attributing
these directly to physical disk service time or a per-request fsync.

Report all-response and successful throughput using actual phase wall time
through last in-flight completion; nearest-rank p50/p95/p99 full-body client
latency over all responses, plus successful-only quantiles; 503/transport/
unexpected-status counts and rates; phase CPU consumption where one core is
100%; sampled peak RSS and observed SQL/HTTP gauges. Include writes separately.
Do not report successful-only latency as if it included rejected work.

Local investigation thresholds, fixed before execution:

- A passing load point has zero 503, transport or unexpected statuses,
  p95 at most 100 ms, p99 at most 250 ms, mean application CPU at most 200%,
  sampled peak application RSS at most 128 MiB and all sampled readiness 200.
- Baseline and recovery additionally achieve at least 20 successful requests/s;
  the two-worker sustained phase at least 50 successful requests/s. Other steps
  require at least 20 successful requests/s. These are fixture investigation
  choices, **not a consumer SLO or framework completion requirement**.
- Keep every failed point in the result table and exclude it from the observed
  passing range. The highest passing tested point is not a measured hard limit.
  A failed performance threshold alone does not authorize optimization, policy
  relaxation, a rerun or changing these thresholds.
- At completion reconcile the exact set of acknowledged writes against the
  stopped fixture's rows, unique markers, schema/index and sequence invariants.
  Record ambiguous/failed writes separately; zero silent loss or duplicate retry
  is required. Final active gauges must return to zero after clients stop.
- Observe five seconds idle, then require native SIGTERM exit 0 and `stopped`
  within the selected 20-second supervisor bound. Confirm owned units/processes
  exited, the port refuses connections and disposable credentials are removed.
  Preserve evidence before removing only this task's disposable data.

Framework/reference Done requires a usable contract, reproducible bounded
measurement and honest passing/failed ranges and limits, write reconciliation,
owned cleanup and independent Reviewer acceptance. It does not depend on a
particular consumer CPU, disk, request rate, SLO or production deployment.
Any material product failure is reported and repaired only within the original
requirement; no benchmark tooling or speculative optimization is authorized.

## Results and acceptance

The pre-execution contract is retained as `contract-before-execution.md`, SHA-256
`62929603edbaafdb2d8e7519f4e1309a2be227fc1502d12436c66a79cba01513`.
The schedule, payload declaration and investigation thresholds were retained
before startup in `preexecution.json`; no threshold was tuned after measurement.
The later document edits add results and acceptance records only.

Execution used HEAD `94c0cb3a7635b9f1f01cc6c52f61fac30a714986` with this task's
documentation work preserved. All 345 build inputs and the accepted release
executable matched before and after; no rebuild, runtime change or previous
qualification rerun occurred. Linux 7.0.0-38/glibc 2.43, Intel i5-13600K with
20 visible logical CPUs, Python 3.14.4 and systemd 259.9 were recorded. The
private disposable database was on `/dev/nvme0n1p2` ext4, with about 347 GB free
before execution. The declared app/client resource caps were applied through
two uniquely owned transient user units; no existing service was modified.

The completed measurement ran from 07:14:41 to 07:15:53 UTC on 2026-10-23,
including setup/idle/stop. It issued **23,549 workload requests**: 17,829 status
200, 1,945 status 201 and 3,775 status 503, with zero transport or unexpected
status errors. Actual operation counts were 9,438 API detail, 7,065 API list,
4,698 SSR detail and 2,348 API create; phase truncation explains the small
nominal-mix differences. Full-body timings include unsuccessful responses.

| Phase | Actual s | Requests | All / successful req/s | All p50 / p95 / p99 ms | 503 rate | Mean CPU / peak RSS MiB | Declared outcome |
|---|---|---|---|---|---|---|---|
| baseline | 8.000 | 346 | 43.25 / 43.25 | 1.64 / 14.35 / 16.21 | 0.000% | 5.75% / 7.88 | PASS |
| step2 | 10.000 | 1,555 | 155.50 / 155.50 | 1.35 / 12.22 / 17.01 | 0.000% | 16.30% / 7.99 | PASS |
| step4 | 10.011 | 4,530 | 452.48 / 452.48 | 1.88 / 12.17 / 19.72 | 0.000% | 50.74% / 8.55 | PASS |
| step8 | 10.046 | 13,699 | 1363.57 / 987.81 | 2.61 / 11.30 / 27.94 | 27.557% | 104.12% / 8.96 | FAIL: 503 / readiness |
| sustained | 20.000 | 3,073 | 153.65 / 153.65 | 1.52 / 12.34 / 17.50 | 0.000% | 17.80% / 8.97 | PASS |
| recovery | 8.000 | 346 | 43.25 / 43.25 | 1.67 / 14.28 / 15.65 | 0.000% | 6.00% / 8.97 | PASS |

The four-worker point is the highest **tested passing point**, 452.48 req/s for
about ten seconds, not a maximum capacity or a sustained-rate guarantee. The
separate two-worker 20-second sustained phase passed at 153.65 req/s. The
unpaced eight-worker point failed the unchanged zero-rejection/readiness
thresholds: 3,775 workload 503 responses (27.5568%) and two sampled readiness
503 responses. Both following lower-load phases passed with no 503 and all
sampled readiness 200. This establishes observed recovery at those points, not
a production recovery-time commitment or an exact saturation boundary.

During Step 8 the SQL saturated counter increased by 3,777, accounting for
3,775 workload rejections plus two readiness probes; successful SQL operations
and probe populations differ from workload counts. No HTTP-admission saturation
was observed. This is evidence of the existing immediate SQL admission boundary,
not justification to remove it, add queuing or tune policy. Successful-only
Step 8 p95/p99 were 12.76/38.66 ms; these exclude rejections and must not replace
the all-response table above. Writes are slower than most reads:

| Phase | Acknowledged creates | Successful-create p95 / p99 ms |
|---|---|---|
| baseline | 34 | 18.56 / 28.25 |
| step2 | 154 | 19.41 / 20.50 |
| step4 | 452 | 20.74 / 39.51 |
| step8 | 965 | 63.56 / 135.28 |
| sustained | 306 | 19.67 / 26.69 |
| recovery | 34 | 16.50 / 28.81 |

There were 66 periodic samples and 82 total samples including phase boundaries;
probe/sample execution took about 0.0007–0.0249 seconds. Sampled RSS peaked at
8.97 MiB, FD count ranged 10–25 and threads 4–8. The app cgroup recorded zero
CPU throttling. **Client cgroup throttling was not sampled**, so the shared-host,
closed-loop generator remains part of the observed limit; these results do not
isolate an application-only maximum. Sampled active gauges and OS peaks may
miss shorter bursts.

Observed database/WAL/SHM maxima were 192,512 / 688,072 / 32,768 bytes. Process
storage-I/O counters and their phase deltas are retained; they include process
activity and writeback accounting, not database-only physical I/O or per-request
fsync timings. The app explicitly selects startup WAL and per-operation two-second
busy timeout; its synchronous mode and linked SQLite runtime version were not
measured. Inspector SQLite 3.46.1 / WAL / integrity observations describe that
inspection connection only. Ext4 placement and warm page-cache observations do
not qualify power loss, cold cache, media durability or consumer disk latency.

The stopped fixture contained exactly **1,965 rows = 20 seeds + 1,945 acknowledged
creates**, with matching full synthetic rows, unique IDs/markers, schema-2 DDL
and status index, sequence 1,965 and inspector integrity result `ok`. All 403
create attempts that received 503 were absent; there were no ambiguous transport
outcomes, missing acknowledgments, extra rows or automatic retries. Final
HTTP/SQL gauges were zero. After 5.005 seconds idle, native SIGTERM yielded
exit 0 and `stopped`; the supervisor stop command took 0.0053 seconds in this
fixture, not an RTO claim. Both owned units became inactive with MainPID 0 and
exit 0; both process IDs were absent, the port refused connections, and the
owned disposable fixture and credentials were removed after evidence retention.

One pre-workload harness failure occurred before any HTTP request: a local
function shadowed the imported HTTP client module. The app stopped cleanly and
all owned resources were cleaned. A single import-alias correction preserved
the original script/output/declaration and resumed the same task and unchanged
schedule. One failed-state reset of only the owned client unit enabled its
same-name launch. The correction hash was recorded before that launch; the
original script and unified diff were reconstructed afterward and the original
hash verified. These events are retained in `failure-history.json` and
`attempt1-preworkload-harness-failure/`; no product repair or measured workload
replay occurred.

Evidence root: `/tmp/yetlet-yt134-capacity-20261023-52q8_bc_`.
The exact direct workload and app/client argv/settings are retained in
`measure.py`, `commands.json`, `preexecution.json`, protected-environment summaries
and unit-property records. `requests.jsonl`, `writes.jsonl`, `samples.jsonl`,
`metrics-*.prom`, `result.json`, `reconciliation.json` and
`reporting-projection.json` retain observations and interpretation.
`cleanup-external.json` records independent post-client process/unit/port/fixture
checks. Before/after reuse hashes and original failure/correction evidence remain
separate. Per-request records contain no tokens, cookies, CSRF values or bodies;
known synthetic inputs and stopped-row reconciliation are retained privately.

Independent Reviewer `/root/yt134_release_storage_independent_review_20261023`
returned **PASS**, zero issues and required followups. All six reviewed source
hashes matched before and after review; 991 other baseline paths and all 345
build inputs remained unchanged. Controller acceptance records Done at bounded
optional reference scope. The passing and failed measured points are both part
of the deliverable; their thresholds are
not a new framework or consumer completion gate. No SSH/proxy throughput,
open-loop maximum, long soak, crash/restore, browser, CSP, Windows, IdP, Valkey,
production capacity or production deployment qualification is claimed.

YT-134 is Done after independent Reviewer PASS and controller acceptance. The
next optional item is YT-135 current-schema backup/restore/upgrade reference;
YT-135–YT-140 remain Todo and require explicit activation. YT-096 remains In progress
with D1–D6 Pending; YT-020 remains Blocked; immutable M20 candidate `ce56e08`
and its original nine libraries are unchanged. No signing, release or production
qualification is implied.

## Optional Azure supplement

After this local acceptance, the user separately authorized an ephemeral
[Azure reference](YT-134-AZURE-REFERENCE.md): separate app/load VMs, managed data
disk and one ordinary VM reboot/readback, followed by deletion of all task-owned
resources. Its 34,329-request workload, exact write reconciliation, ordinary
reboot/readback and full resource deletion are complete, with separate independent
Reviewer PASS, zero issues and required followups. Both failed cloud load points
remain retained; the original local PASS keeps its original environment scope. This does not change the local Done status,
make Azure mandatory or activate YT-135–YT-140.
