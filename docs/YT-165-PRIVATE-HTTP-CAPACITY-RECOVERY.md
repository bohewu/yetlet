# YT-165 — Private HTTP capacity, saturation and recovery

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded private-profile capacity/saturation/recovery scope after Independent final Reviewer /root/yt165_capacity_results_final_review_20261024 PASS, zero issues/required followups. Failed8k offered point preserved; precise interior clean boundary remains unmeasured.**

## Authority and preserved workload

The user activates a private direct-HTTP capacity study after the separate
seven-path YT-164 commit. Establish the contract, increase load progressively,
preserve failed results, delete all test resources and obtain independent
Reviewer PASS before closeout. New changes must not be committed or pushed.

Reuse [YT-164](YT-164-PRIVATE-HTTP-SSH-COMPARISON.md)'s exact current executable
SHA25647d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65
and accepted k6 binary without rebuilding or rerunning qualifications. Keep
medium128rows/31063bytes, the literal full body/header/CSP oracle, HTTP1.1,
identity encoding, connection reuse and8s request/graceful timeout. Preserve
all earlier results/failures and [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). No Core/runtime/API/output,
dependency, default listener or consumer hardware/SLO requirement changes.

Capacity here is conditional on this finite app/VM/network/client profile.
Distinguish a last clean offered point, first pressure point, supported server
saturation and a generator/VM/network observation limit; do not call every drop
a framework capacity ceiling. A clean final point provides a lower bound only.

## Finite profile and staircase

| Field | Selected contract |
| --- | --- |
| Ownership/window | One new task-owned Azure group, one allocation generation, East Asia zone1, pinned Ubuntu24.04 image;60min resource lifetime, stop new work by45min and reserve15min for cleanup; no region/size fallback |
| App | Same Standard_D2s_v5,2vCPU/8GiB guest; two workers, CPU0–1/quota200%,256MiB/128tasks/512FDs, no restart; one app PID for the entire staircase and recovery,240s hard lifetime/5s stop |
| Generator | Separate Standard_D16s_v5,16vCPU/64GiB guest; quota1600%,8GiB/512tasks/8192FDs,300s hard lifetime with270s internal batch deadline/30s cleanup reserve |
| Concurrency |256 preallocated/max VUs fixed before execution, no adaptive increase; deliberately different from the old64-VU path study so the new generator has declared headroom |
| HTTP/control | Direct private client-to-app TCP39262 only; authenticated private SSH for control, no data tunnel; client-only TCP22/39262 rules and deny other inbound; accepted storage-first Blob/Run Command transport |
| Warmup | Once125/s2s on the newly admitted app; not a replay of a historical population |
| Offered steps |2,000 /4,000 /8,000 /12,000 /16,000 arrivals/s,10s each, once in increasing order; stop increasing at the first pressure signal or safety boundary |
| Recovery | On the same app PID,1,000/s10s after the final stress invocation, then5s no-HTTP idle and another1,000/s10s; these exercise post-stress state, not the old1,000/s qualification |
| Capture | New capacity caller derived from the existing oracle; all native JSONL/failures preserved. Owned raw JSONL lives in existing /dev/shm tmpfs to avoid an OS-disk write ceiling; executable inputs remain on the normal task filesystem. Maximum4GiB raw output/2GiB per file,512MiB new local evidence, no persistent raw duplicate |
| Observation | Same finite50ms process/thread/guest observations, extended to the declared batch lifetime; add cgroup boundaries and server-side stage markers, retain cadence/cost/coverage and actual process identities |

A new-profile read-only quota admission returns regional and DSv5 usage0/20,
enough for18vCPUs. Native exit0 is retained. Its original offline normalization
failed because quota numbers arrived as strings; one smallest integer conversion
uses the retained response without another Azure query. The failure/correction
are preserved. Quota is not a guarantee of allocation; the old SKU-query timeout
remains unresolved, with no broad query or allocation fallback selected.

## Bounded caller adaptation and sampling

Add only a separate sample capacity caller and its documentation; the accepted
path-study caller and server source/artifact remain unchanged. Admit only the
declared private endpoint, step/rate/duration and256-VU plan. Copy the exact
existing response oracle, recording its source equality. A200 response with
incorrect body/headers/CSP aborts immediately and is never accepted as overload.
Keep every non200/error/drop observation with its actual native exit.

The existing server probe has finite IDs/storage. To reuse it unchanged, send
its diagnostic header only every4096th iteration, assigning a unique existing
w:/m: ID from a64-sample block per declared step. Record server_probe_id
separately from each native request probe_id; never join them by coincidence.
The maximum declared run fits the existing ID/record limits. Unsampled requests
keep the same handler/output; probe density is a declared measurement change,
not a claim of identical diagnostic overhead to YT-164.

Focused verification is no-HTTP inspect of the new caller and source/oracle
bindings; no Rust build or accepted runtime/browser/provider test replay.
Runtime admission checks actual transferred hashes, app identity, private path,
limits, tmpfs capacity and observer availability before any load.

## Pressure, saturation and recovery interpretation

Stop increasing on any dropped iteration/HTTP error, p95 at least50ms, generator
or evidence bound, or a source/runtime integrity failure. The50ms value is an
explicit study pressure marker, not a user or production SLO. A response
correctness mismatch, app exit/restart, native command failure or deadline is a
hard stop; do not restart or replay the app to manufacture recovery evidence.
For expected scheduling/latency/HTTP pressure with the original app alive,
preserve the failed point and proceed only to the declared lower-load recovery.

Report configured arrivals, actual starts/completions inside the client-clock
half-open scheduled window, boundary starts, dropped attempts and post-window
drain separately. Full invocation counts divided by10s are not automatically
achieved in-window throughput. Do not assign response latencies to unstarted
arrivals or remove failures from scheduling outcomes.

Use app CPU/quota, queues, memory, process identity and waiting alongside client
CPU/quota, queues, capture and VU limits. Call server saturation supported only
when pressure/throughput evidence and server resource observations support it
without a demonstrated generator/capture ceiling. Otherwise retain a bounded
profile/generator/network limit. No inference of physical cycles or unique
frequency/crypto/kernel causality follows from guest counters.

Recovery reports the real delay from stress scheduled end, including its up-to8s
graceful drain, process handover and actual first recovery start. It is not an
instantaneous uninterrupted same-caller transition. As descriptive recovery
markers, identify the first three consecutive1s bins with zero drops/errors,
at least950 valid completions and p95 at most10ms, and the complete recovery
population outcomes. These are study-specific observations, not a production
SLO or a new qualification gate. Server resource envelopes include explicit
server-side marker/control/drain bounds; no cross-VM timestamp subtraction.

## Cleanup and acceptance

Reuse the accepted task-only storage-first transport and exact-owned cleanup;
record new script/artifact manifests before creation. Stop/reap client/app/
observer/control activity, retrieve available raw failure evidence, delete the
exact group, verify absence/zero owned resources and remove temporary credentials
on every outcome. Do not change existing services or host policy. One smallest
harness correction per distinct seam is allowed; repeated infrastructure failure
stops, with no new task/window or favorable measurement replay.

Independent contract review precedes implementation; review concrete execution
mapping before provisioning and actual results/cleanup before Done. Record all
limits honestly if the finite range does not establish a server upper bound or
recovery. Required new repository paths are this report, three docs indexes,
the new sample caller and sample README. Run documentation/diff checks for new
work; no benchmark/validator framework or broader qualification matrix.

## Concrete mapping before provisioning

Independent contract Reviewer /root/yt165_capacity_contract_review_20261024
returned **PASS, zero issues/required followups** before implementation. Private
scripts/manifests live in `target/yt165-private-capacity-20261024/evidence`.
The new caller copies the exact literal oracle and response-validation block;
the accepted caller/server sources and both executable hashes remain unchanged.
One added valid-response counter identifies non200 outcomes independently of
k6's default expected-status classification. Eight declared plan inspections,
one invalid-step rejection and three private endpoint/port rejections pass with
no HTTP. No old build/test/load qualification is repeated.

The accepted storage-first helpers are adapted to one app unit and one bounded
continuous app observer. The load VM changes to the declared16-vCPU SKU; actual
guest core counts, quota/memory/task/FD limits and tmpfs capacity are admitted
before requests. Binaries/scripts remain in the normal task directory; only
raw request JSONL goes to its owned /dev/shm directory. The result wrapper adds
that known raw directory to its returned ZIP, rejecting symlinks, without raw
duplication or a host mount/configuration change.

Server-side before/after stage markers and cgroup snapshots retain their own
clock bounds and original PID. The controller evaluates pressure from native
k6 summaries and the valid-response count, retaining the failed offered point.
All raw reconciliation is deferred until after both recovery points and app
stop, so it does not insert a large quiet gap before recovery. Native graceful
drain, actual caller startup and SSH control handover still remain in the
reported recovery delay. App and observer identities/limits are checked before
load; normal stop and failure cleanup retain original-PID absence and unit
closure. Concrete mapping review is required before the single allocation.

## Actual once-only staircase and recovery

Independent mapping Reviewer /root/yt165_capacity_mapping_review_20261024
returned **PASS, zero issues/required followups** before creation. One declared
storage-first allocation admits the first authenticated Blob GET/1KiB roundtrip,
the two exact guest core counts, transferred artifacts, private direct path,
actual cgroup/task/FD limits and tmpfs. The accepted executable/k6 hashes remain
unchanged; no build or earlier qualification is repeated. One app PID serves
warmup, all stress points and both recovery points without restart.

### Scheduling and completed-response latency

| Stage | Offered arrivals/s | Started/completed valid200 | Drops | Valid completions/s inside scheduled window | Valid HTTP p50 ms | Valid HTTP p95 ms | Valid HTTP p99 ms |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2k | 2000 | 20000 | 0 | 1999.8 | 0.512838 | 0.751370 | 1.550582 |
| 4k | 4000 | 40001 | 0 | 3999.5 | 0.566509 | 0.874150 | 1.544828 |
| 8k | 8000 | 59550 | 20450 | 5929.6 | 43.242377 | 63.445049 | 68.353842 |
| Immediate recovery | 1000 | 10001 | 0 | 999.9 | 0.635688 | 0.814826 | 1.257392 |
| Recovery after idle | 1000 | 10001 | 0 | 1000.0 | 0.631085 | 0.791425 | 1.203048 |

The once-only warmup adds251valid200. Measured totals retain139553valid200
and20450unstarted arrivals; all started iterations finish and pass the full
oracle, with zero HTTP errors or response mismatches. The8k point is a failed
offered-load point, not a clean capacity success. Its drops are25.5625% of
nominal80000arrivals. The runtime pressure trigger sees native k6 p95
63.444610ms as well as drops;12k/16k are deliberately not started. No replay,
retry or increased VU/memory limit follows the failed point.

Raw nearest-rank completed-response quantiles are shown above; native k6's
interpolated summary used by the runtime trigger is separately retained.
Failed scheduling outcomes are never discarded or assigned a response latency.
The measured starts plus drops total160003against nominal160000. Start counts
at/after nominal end are0/4/0/1/1 in stage order; some nominal slots execute late,
so they are not all extra nominal-boundary slots. Completion tails also remain:
the8k stage has254post-window completions and52.531596ms observed HTTP drain.
Dividing all59550completed responses by10s would hide that drain; the in-window
rate is5929.6, not5955. The1ms caller full-call population is separate: its
8k p95 is64ms and recovery p95 is1ms. Submillisecond zeros/phase quantization
must not be interpreted as zero HTTP latency or negative caller overhead.

### Saturation versus a generator/capture ceiling

The last clean offered point is4k; the first observed pressure point is8k.
This establishes a coarse clean/pressure bracket, not an exact maximum clean
rate between them and not proof that roughly5.9k remains clean under sustained
load. At8k the app resource envelope averages179.23% of one CPU, including
caller startup/control/drain. During server-side before-marker observations,
50ms CPU-accounting intervals have median195.93% of one CPU out of200%, with
198of218intervals at least180%. That sustained near-two-CPU activity, queue-like
waiting and failed offered delivery support **server/profile CPU saturation**.
The8k waiting median/p95 are43.137826/63.342378ms. Sparse middleware probes omit
accept/runtime queues and body transmission; do not call this pure Core
instruction attribution or identify one frequency/kernel mechanism.

The16-CPU client envelope averages624.59% of one CPU,39.04% of its declared
quota; its observed k6 process average is626.76% over readable samples. Client
quota throttling and memory max/OOM events are zero. The RAM capture totals
867633581raw bytes, below4GiB; cumulative client cgroup memory peak reaches
989.49MiB, below8GiB. No declared generator/capture bound is reached. This and
the server observations support the saturation interpretation, without proving
that client scheduling or diagnostic overhead is absent at every instant.

The app records one34microsecond quota-throttling event at8k, zero memory
max/OOM events and a cumulative charged cgroup memory peak of12.21MiB. The
single tiny throttling event is not itself an explanation for63ms response
tails. Cgroup memory peak is not a claim of total process/VM memory footprint.
After pressure, app CPU envelopes return to20.34/19.84% of one CPU.

### Recovery on the original process

The first recovery request starts1.226909s after the8k scheduled window ends.
That gap includes the observed drain, actual native exit/startup and SSH control
handover; it is not a hidden raw-parsing pause or an uninterrupted offered-rate
switch. All ten1s bins in each recovery point satisfy the declared descriptive
marker, with zero errors/drops and near1000valid completions per bin. The first
three-bin marker is established3s after each recovery anchor: by4.224s after
stress scheduled end for immediate recovery, and20.277s for the later recovery
after the explicit5s idle and intervening handovers. This bounds the observed
restored behavior; it is not the exact instant the service recovered.

Original PID identity holds at every before/after marker and normal stop, with
no restart. Both recovery populations remain below1ms HTTP p95. No request,
observer or app replay is used to turn the failed8k offered point into a pass.

### Observation, retention and cleanup

Seven finite observer populations retain2356records, zero skipped nominal slots
and largest sampling gap53.996247ms. Two client process-read error records are
retained; per-process CPU means use readable first/last intervals. Native request
populations and cgroup boundary receipts remain available independently of those
missing observer reads. The app observer averages2.02% of one CPU; measured
client observers average3.35–4.41%, not zero disturbance. The unchanged server
dump retains37phase-scoped unique sparse records/74valid same-thread clocks,
zero invalid IDs/clock records/overflow, within its original limits. Explicit
server_probe_id joins avoid confusing these IDs with request iteration IDs.

All native logs, failed offered-load outcomes, pressure decisions, full raw
JSONL, marker/observer records and earlier failures remain. Canonical returned
ZIPs retain867.6MB logical request JSONL without persistent raw extraction;
new local evidence is about99MB, below512MiB. Offline analysis starts only after
cloud removal; its three bounded calculations exit0. The earlier quota
normalization failure/correction remains distinct from the expected8k pressure.

Normal app stop confirms original-PID absence; observer units, per-invocation
observers and control activity close. The controller exits0 because the bounded
study, including the failed pressure point and recovery, completed; it does not
mean every offered point passed. The local session process is waited/reaped.
The exact group and all16inventoried task resources are deleted, with group
absence, zero owned resources and removed local credentials verified. Resource
lifetime is539.693837s versus3600s; native session elapsed is541.503700s.
Existing services, Core source/APIs and accepted artifacts remain unchanged.

## Remaining precision and next proposal

For this finite synthetic profile,4k is verified clean and8k is saturated with
approximately5.93k in-window valid completions/s. The exact clean boundary in
between, long-duration behavior and production/business workloads remain
unmeasured. A future separately activated study could add a few new interior
offered rates, preserving these failed observations and avoiding repetition of
accepted points. No new load, Core optimization, consumer SLO or release scope
is activated by this proposal. Final independent review must accept the actual
results, limitations and cleanup before Done.
