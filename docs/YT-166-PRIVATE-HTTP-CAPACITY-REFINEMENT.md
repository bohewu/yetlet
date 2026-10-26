# YT-166 — Private HTTP capacity boundary refinement

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded refinement scope; independent contract/mapping/final Reviewer PASS, zero issues/required followups; no new-change commit/push.**

## Authority and question

The user activates only the new5k/6k/7k arrivals/s points after the separate
six-path YT-165 strict/builtin commit. Establish the contract, perform minimal
isolated validation with the same workload/response checks, preserve failures,
delete all test resources and obtain independent Reviewer PASS. New changes
must not be committed or pushed.

Reuse [YT-165](YT-165-PRIVATE-HTTP-CAPACITY-RECOVERY.md)'s accepted current
executable SHA25647d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65
and accepted k6 without rebuilding or rerunning qualifications. Preserve its
failed8k point, all prior failures and the accepted recovery evidence. Keep
medium128rows/31063bytes, the literal full body/header/CSP oracle, identity
encoding, HTTP1.1, connection reuse,8s request/graceful timeout,256fixed VUs
and every4096th bounded server-probe header with explicit server_probe_id joins.
No Core/runtime/API/output/dependency or default listener change follows.
Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md).

The old4k/8k observations are context, not same-host measurements of this new
allocation. Report actual guest identity and each new point. Do not merge
different allocations into a fictitious precise clean/pressure bracket. This
finite synthetic profile is not a universal maximum or consumer hardware/SLO.

## Minimal once-only execution

| Field | Selected bound |
| --- | --- |
| Ownership | One new task-tagged group, one allocation generation, same East Asia zone1/pinned Ubuntu24.04 image; no existing service modification, size/region fallback or repeated workload point |
| Lifetime |60min resource lifetime, stop new work by45min, reserve15min for cleanup |
| Platform | Same app Standard_D2s_v5 versus separate load Standard_D16s_v5; reuse18-vCPU quota admission and verified prior resource removal, with actual new-target admission before load |
| App resources | Same two workers, CPU0–1/quota200%,256MiB/128tasks/512FDs/Nice10, no restart; fresh app instance per offered point,45s maximum lifetime/5s stop |
| Generator | Same16CPU quota,8GiB/512tasks/8192FDs/2GiB per file;300s maximum batch,270s internal deadline reserving30s cleanup |
| Workload order | Fresh app plus125/s2s admission warmup, then5k/s10s; stop/reap it. Repeat once for6k, then7k on the same app VM with a fresh instance each time |
| Recovery/old points | No new1k recovery,2k/4k/8k or higher point. Reuse their accepted evidence; warmups admit the new app instances rather than repeat an old qualification |
| Capture/observation | Same finite50ms process/thread/guest/cgroup observations and server-side stage markers; raw JSONL in owned /dev/shm tmpfs,4GiB total raw/512MiB new local evidence, canonical ZIPs without persistent raw duplicates |
| Network/control | Private direct TCP39262 only, client-only TCP22/39262 rules and deny other inbound; accepted storage-first Blob GET/1KiB roundtrip, protected Run Command transfer and authenticated private SSH control |

Extend only the existing capacity caller's fixed plan with the six new warmup /
measured labels and warmup-label detection. Snapshot the accepted YT-165 caller
from its committed source before that extension. Keep the oracle, validation,
request/timing/metric code, concurrency, sparse sampling and old plan entries
identical; no duplicate caller or new benchmark framework is needed. Inspect
only the new declared plans and an invalid selector without HTTP. Old runtime,
browser/CSP/provider/build/endpoint qualifications remain reused.

Each fresh app, warmup and measured step must match the transferred manifest,
original PID for that point, actual resource limits and expected private path.
Normal stop confirms PID absence before admitting the next fresh instance.
The different new offered points are not replays of a failed point.

## Pressure and safety semantics

Reuse YT-165's descriptive pressure definition: any scheduled drop, HTTP error,
invalid/incomplete response population or native p95 at least50ms. The50ms
marker is a study rule, not a production SLO. Expected scheduling/latency/HTTP
pressure is retained as a failed offered-load observation. After the original
app remains identifiable and stops cleanly, continue to the next explicitly
requested new rate with a fresh app. Unlike YT-165's exploratory staircase,
expected pressure alone does not cancel the remaining5k/6k/7k points.

A200 body/header/CSP mismatch, app crash/restart, native command failure,
identity/observer/capture/resource-admission failure or deadline is a hard stop:
retain evidence and clean up, with no repair/replay of a completed workload,
automatic new allocation or substitution of a favorable result. A warmup
admission failure stops before its measured point. Do not silently raise VUs,
memory, hardware or sampling/capture limits in response to pressure.

Report arrivals, starts/completions inside each half-open client-clock window,
late starts, unstarted drops and graceful completion tails separately. Native
full-invocation counts divided by10s are not automatically delivered in-window
RPS. Keep completed-response quantiles separate from failed scheduling outcomes.
Use app and generator quota/queue/memory/capture observations to distinguish
server/profile saturation from a generator/network/capture bound; sparse and
50ms observations are not continuous peak or physical-frequency proof. Do not
subtract unrelated VM clocks or compare differently scoped resource envelopes.

## Cleanup and acceptance

Reuse the accepted storage-first exact-owned lifecycle, with manifests before
creation. Stop/reap all app/observer/client/control activity; retrieve available
failure records, delete the exact group, verify group absence/zero owned
resources and remove temporary credentials on every outcome. One smallest
harness correction per distinct seam may not erase failures, renew the window
or replay completed measurements; repeated infrastructure failure stops.

Independent contract review precedes implementation, concrete mapping review
precedes provisioning and final independent results/cleanup review precedes
Done. A finite study may establish an interior clean/pressure bracket or expose
remaining uncertainty; report it honestly without a release/SLO claim. Required
repository paths are this report, three docs indexes, existing capacity caller
and sample README. Run documentation/diff checks for new work; no old-proof
rerun, new crate/runtime, Core optimization or new-change Commit/Push.

## Concrete mapping before provisioning

Independent contract Reviewer /root/yt166_refinement_contract_review_20261024
returned **PASS, zero issues/required followups** before dependent work. Private
evidence is in `target/yt166-private-refinement-20261024/evidence`. The accepted
YT-165 caller's Git blob is snapshotted with its commit/blob/SHA binding. The
existing caller changes only six fixed plan entries and warmup-label detection;
everything from the CSP/oracle through the request, response validation, options,
metrics and sampling code remains byte-identical. Six new plan inspections and
one invalid-selector rejection pass without HTTP; no old plan is executed.

The accepted storage-first/capture/resource seams are adapted to three separate
app/observer units with strict5k/6k/7k ordering. Each point records original PID,
actual two workers/limits, per-stage resource markers and normal PID absence
before the next app starts. Expected measured pressure does not change the
remaining explicit rate plan. Raw reconciliation stays after all normal stops,
so it does not change measured behavior between stages. The existing16-CPU
generator/tmpfs limits and protected transfer/exact-owned cleanup remain.

A preparation-only manifest replacement initially leaves stale nested plan
entries and fails syntax checking before any resource creation. Preserve that
source and failure. One smallest in-place correction replaces the canonical
manifest plan line; focused syntax checking passes with the same task/nonce.
The seven successful caller inspections are retained without rerun. No product,
oracle or old accepted result changes to pass a harness check. Concrete mapping
review still precedes the only allocation.

The first execution-mapping review finds one P2: observer failure could be
recorded without preventing later load. Preserve that FAIL and both original
scripts. A single local correction checks app-observer readiness before warmup,
liveness/capture at stage boundaries and successful nonempty capture at normal
stop; the client also rejects premature observer exit, unsuccessful termination
or missing capture summary before later load. Failure receipts are written
before rejection and existing cleanup remains unconditional. The two corrected
scripts pass focused syntax checks; no workload, cloud allocation or successful
caller inspection is repeated. Independent mapping re-review returns **PASS,
zero issues/required followups** before provisioning. Retain the initial FAIL,
original scripts and correction evidence beside that accepted result.

## Actual once-only refinement

The accepted mapping precedes one storage-first allocation. Authenticated Blob
GET/1KiB roundtrip, transferred hashes, actual guest core counts, CPU/memory/
task/FD limits and tmpfs admission pass before load. The same new app VM serves
all three points, with three distinct original app PIDs and no restart. Each
warmup and measured step retain the matching PID; normal stop proves its absence
before the next app starts. All six native invocations exit0. No old point,
recovery, build or earlier qualification is repeated.

### Scheduling and completed-response latency

| Offered arrivals/s | Started/completed valid200 | Drops | Valid completions/s inside scheduled window | Valid HTTP p50 ms | Valid HTTP p95 ms | Valid HTTP p99 ms |
| --- | --- | --- | --- | --- | --- | --- |
| 5000 | 50001 | 0 | 4994.8 | 0.492989 | 0.970908 | 2.081565 |
| 6000 | 60001 | 0 | 5999.1 | 0.736553 | 2.688957 | 12.618009 |
| 7000 | 60416 | 9585 | 6015.7 | 42.531377 | 63.278223 | 69.732922 |

Each measured point lasts10s. The three admission warmups add753valid200.
Measured totals retain170418valid200 and9585unstarted arrivals. Every started
iteration completes with the full literal body/header/CSP oracle, zero HTTP
errors and zero response mismatches. The7k offered point fails the descriptive
pressure rule:9585drops,13.6929% of nominal70000arrivals, and native k6 p95
63.278112ms. It remains a failed offered-load observation, without replay or
increased VUs/resources. Clean5k/6k have zero drops/errors and native p95 below
the50ms study marker;6k already has a larger p99 tail than5k.

The table uses nearest-rank raw completed-response quantiles; the runtime rule
uses separately retained native interpolated summaries. Scheduling failures
never receive a response latency. Measured starts plus drops total180003
against nominal180000. Starts at/after each scheduled end are6/3/5, including
late execution of nominal slots; do not label every late start an extra slot.
Post-window completions are53/10/259, with observed HTTP drain
17.088403/1.481828/71.697730ms. Thus7k delivers6015.7valid in-window responses/s,
not6041.6from dividing all completions by10s. Full-call1ms-quantized p95 values
are1/3/63ms; they are a separate population from native HTTP timing.

### Saturation and observation limits

The new-host observations establish6k as the highest clean tested offered rate
and7k as the lowest pressure tested rate: an observed6k–7k bracket for this
finite profile. They do not identify an exact maximum or a sustained production
capacity. Old4k/8k are not endpoints of this same-allocation bracket.

At5k/6k/7k, app resource envelopes average126.79/173.90/179.12% of one CPU out
of200%, including native startup/control/drain. Server-side before-marker
50ms intervals at6k/7k have median190.21/195.78% of one CPU, with199/218 and
196/218 intervals at least180%. At7k the HTTP waiting median/p95 are
42.461065/63.213708ms. Near-two-CPU app activity, the flat delivered rate and
increased waiting support **server/profile CPU saturation**. Sparse middleware
probes omit accept/runtime queues and transmission; this does not identify
a particular Core instruction, kernel or physical-frequency mechanism.

Client envelopes average477.45/563.87/561.43% of one CPU,29.84/35.24/35.09% of
the declared16-CPU quota. Readable k6 process means are477.10/564.42/561.92%.
Both app and client record zero quota-throttling and memory max/OOM events.
The client's cumulative charged cgroup memory peak reaches1207.14MiB below8GiB;
fresh app cgroup peaks stay below12MiB. Raw request capture totals1060787719
bytes, below4GiB. No declared client/capture bound is reached. These observations
support the server/profile interpretation without proving that generator
scheduling, diagnostics or network variability have zero effect.

Nine finite observer populations retain1629records. One skipped nominal app
observer slot is retained; process-read error records are zero. Largest sampled
gap is53.079119ms. App observers cost2.13–2.20% of one CPU; measured client
observers cost3.54–3.88%. They are not free or continuous peak traces. The
unchanged server diagnostics retain46phase-scoped unique sparse records and
92valid same-thread clocks, zero invalid IDs/clock records/overflow, within
the original limits. Explicit server_probe_id joins remain separate from
per-request probe_id. Native request windows and server marker envelopes use
their own clocks, without cross-VM subtraction or differently scoped CPU claims.

### Retention, cleanup and remaining scope

Preserve the preparation SyntaxError, mapping-review P2 FAIL, original scripts,
single local corrections and subsequent PASS; preserve the failed7k offered
point, native logs, full raw request JSONL and observer/marker evidence. The
canonical returned ZIPs retain approximately1.06GB logical request JSONL without
persistent extraction. New local evidence is approximately103MB, below512MiB.
Three bounded offline calculations run only after resource removal and exit0;
raw reconciliation matches all six native invocation populations.

Normal stop confirms all three original app PIDs absent. App/observer units,
per-invocation observers and control activity close; the local session process
is waited/reaped. The exact group and all16inventoried test resources are
deleted. Group absence, zero owned resources, removed local credentials and
unchanged accepted bindings are verified. Resource lifetime is457.122036s
against3600s; native controller elapsed is459.113952s, exit0. That exit records
completion of the bounded study, including failed7k load, rather than success
at every offered rate. Existing services and Core/runtime/API/output semantics
remain unchanged. No new changes are staged, committed or pushed.

The useful observed boundary is now6k clean versus7k pressure, with increasing
tail latency already visible at6k. Long-duration behavior and the exact boundary
remain unmeasured; consumer/business workloads and a production SLO remain
outside this study. A separately activated sustained-profile study could ask
about stability at a distinct duration, but no further allocation, workload,
optimization or item is activated here. Independent final Reviewer
/root/yt166_refinement_results_final_review_20261024 returns **PASS, zero
issues/required followups**. The reviewed source/evidence bindings and exact
pre-closeout copies are retained. Only status/review-reference lines change
after that acceptance; no workload or accepted qualification is repeated.
