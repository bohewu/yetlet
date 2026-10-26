# YT-162 — Benchmark target selection and independent platform comparison

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded platform-reference study scope after Independent final Reviewer /root/yt162_azure_results_final_review_20261024 PASS, zero issues/required followups. Four populations retained; only pair one comparable, reference2 scheduled failure preserved.**

## Purpose and authority

The user asks to continue after [YT-161](YT-161-BENCHMARK-EXECUTION-PROFILE.md).
Preserve its accepted negative admission, all earlier measurements/failures and
the fifteen uncommitted paths. This item first selects an actual target and
states exactly which question that target can answer. No product optimization,
crate/runtime/API/output/security change, commit or push is authorized.

The earlier explicit permission to use Azure CLI for temporary tests and delete
all task-created resources is retained in [YT-134](YT-134-AZURE-REFERENCE.md).
It does not make a normal VM a controlled physical-frequency experiment.
The user accepted the recommended independent Azure platform comparison after
being offered that direction or waiting for physical-frequency test hardware.
The explicit reply is retained in evidence/direction-selection.json; no choice
is inferred from elapsed time. Independent contract review passed before
dependent resource creation or load. Physical-frequency causality stays outside
the selected claim.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). The reference is optional
application/benchmark work, not a consumer hardware/SLO or release requirement.

## Actual read-only inventory

Private local evidence: `target/yt162-target-selection-3l3dnyvk/evidence`.
No token, subscription/resource identity, credential value or operator network
address is copied into this repository document.

| Check | Actual result | Limit |
| --- | --- | --- |
| Current host | Reuse YT-161's nonwritable policy/capability evidence | No shared-host governor/control change is attempted |
| Noninteractive privilege inventory | sudo -n -l exits1: interactive authentication required | No interactive authentication or elevation follows |
| Azure account | Enabled, default AzureCloud account; native exit0 | Authentication does not prove permission for every future operation |
| Existing targets | VM and dedicated-host resource lists both empty, exit0 | No existing isolated target is available in the selected subscription inventory |
| East Asia quota | Regional cores0/20 and standardDSv5Family0/20, exit0 | Four proposed vCPUs fit this snapshot; quota is not allocation success |
| Pinned image | Canonical:ubuntu-24_04-lts:server:24.04.202610160, x64/Linux/V2, exit0 | Actual guest ABI and returned artifact hashes still need admission |
| SKU restrictions | az vm list-skus exceeds its45s bound | Preserve TimeoutExpired; current zone/restriction/capacity eligibility is Not verified |

The timed-out read-only SKU query creates no cloud resource and does not show a
Yetlet defect. No repeated SKU query or allocation fallback is selected here.
If the user selects Azure, retain this limitation: one declared allocation may
still fail; preserve that failure and clean up instead of switching regions,
sizes or restarting the experiment automatically. Even a successful SKU listing
would not guarantee allocation, as [Azure's SKU guidance](https://learn.microsoft.com/en-us/azure/azure-resource-manager/troubleshooting/error-sku-not-available)
explains.

## Target decision and claim boundary

| Direction | Candidate | Supported question | Unresolved prerequisite |
| --- | --- | --- | --- |
| Physical-frequency attribution | Suitable dedicated test hardware with explicit control and effective-frequency observation | Whether the version difference persists under the declared matched physical conditions | Actual target/access/topology/controls/observation remain Pending; no such target found |
| Independent platform comparison, user selected | Two temporary Standard_D2s_v5 VMs in East Asia, separate app/client | Whether reference/current latency and construction CPU differences recur on this new VM platform | Actual allocation/guest/artifact admission Pending; direction and contract accepted |

Azure's [Dsv5 specification](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/general-purpose/dsv5-series)
lists multiple possible Intel processor generations; record actual guest CPU
identity rather than selecting a supposed fixed GHz value from marketing text.
The VM separates this experiment from existing local services; it does not prove
exclusive physical cores, constant frequency or host P-state authority. Do not
change VM or shared-host frequency settings. A future [Dedicated Host](https://learn.microsoft.com/en-us/azure/virtual-machines/dedicated-hosts)
purchase is not selected: exclusive tenancy alone does not establish the required
control/observation capability, and it is unnecessary for the bounded VM question.

The [historical Azure compute-score page](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/compute-benchmark-scores)
is no longer maintained and recommends measuring the actual workload. Do not use
its historical CoreMark numbers to predict Yetlet capacity. YT-161 remains Not
admitted for controlled-frequency execution even if the VM comparison succeeds.

## Proposed finite Azure execution contract

The user selected this direction after independent contract PASS. The following
declaration remains distinct from actual execution or allocation success.

| Field | Proposed choice |
| --- | --- |
| Ownership/lifetime | One newly named/tagged task-only Resource Group, maximum60-minute resource lifetime from first creation; stop new work by45min and reserve15min for cleanup |
| Compute | Exactly two Standard_D2s_v5,2vCPU/8GiB each; same app VM for all reference/current phases, separate fixed load VM; East Asia zone1, no automatic size/region/zone retry |
| OS/storage | The pinned image above; Standard SSD OS disks only; no data disk, database, Valkey or persistent application data |
| Control/input/results | Reuse the accepted YT-134 Managed Run Command/private Blob transport pattern; task-only storage and protected short-lived SAS; no public SSH/inbound app rule, credentials in logs or repository |
| HTTP path | Reuse accepted internal authenticated SSH loopback forward between owned VMs, preserving unchanged loopback-only app/caller; report network/tunnel overhead explicitly |
| Binaries | Exact accepted YT-159 reference/current executables, accepted k6 and unchanged ssr-stage-probe.js; no rebuild, dependency download or old qualification replay |
| App | Two workers, CPU0–1/quota200%,256MiB/128 tasks/512FDs, no restart; each fresh app instance has45s maximum lifetime and5s stop bound |
| Load | Medium128 rows, reference1/current1/current2/reference2; each phase125/s2s warmup then1,000/s10s measured,64 fixed VUs; unchanged caller body/CSP/ID checks |
| Evidence/bounds | Exact per-target manifest before load; retain all native exits, populations/tails, errors/drops, app/client resources and sampled same-thread CPU records; finite256MiB task-local evidence/output, native load deadline180s including owned process cleanup |
| Stop | Any target/artifact/HTTP integrity failure or deadline stops later phases; no favorable replay, tuning, extra workload/rate matrix or automatic rebuild |
| Cleanup | Stop/reap owned app/client/tunnel activity, download bounded evidence, delete exact owned group and verify group absence plus zero owned resources; remove ephemeral credentials on success/failure |

Current published Linux on-demand compute pricing returned by Azure's public
[Retail Prices API](https://learn.microsoft.com/en-us/rest/api/cost-management/retail-prices/azure-retail-prices)
is USD0.132 per VM-hour for Standard_D2s_v5 in East Asia: two VMs for one hour
imply USD0.264 in compute only. The private response preserves the precise meter,
query/time and excludes Windows/Spot/Low Priority meters. This is an estimate,
not a total bill or spending cap: OS disks, storage/operations, public egress
addresses, network, taxes and account-specific charges are separate. Resource
creation/lifetime must remain bounded even when workload admission fails.

Before loading, verify transferred SHA256 against accepted artifacts, actual
Linux/x64/loader/libraries/CPU compatibility, free resources, owned loopback path
and the native two-worker process. These are new-target admission checks, not
replays of existing unit/browser/CSP/provider/build qualifications. If an
accepted binary cannot run, stop and preserve the reason; rebuilding requires a
separate material decision. Record actual CPU topology, guest steal/throttling
and exposed frequency metadata with their limitations. Missing physical counters
do not authorize fabricated normalized cycles or frequency causality.

Interpret p50/p95/p99 and CPU/resource populations within each matched pair on
the same app VM; retain both pairs, raw tails and sparse-sample limitations.
The fixed short phases are a platform observation, not a statistical proof of
equivalence, saturation test, universal benchmark rank or production SLO.
Persistence or disappearance of a difference cannot retrospectively identify
the unique cause of the original local HTTP p95 observation.

### Native execution mapping

Reuse the accepted YT-134 transport with task identities/paths, no data disk and
bounded command handling. All task-local scripts and exact transferred file
hashes are recorded before provisioning and again in the per-target manifest
before load. No repository runtime/sample script changes are required.

Azure Run Command performs guest admission/setup and retrieves results. The one
load-VM batch uses authenticated internal SSH to start/stop each exact app unit
locally; Azure control-plane polling therefore does not consume its45s lifetime.
One loopback-only SSH forward remains owned by that batch. Both existing
executables retain their two-worker runtime and unchanged sampled CPU logging;
no observer or counter is added to the product. Each phase retains new-target
systemd/cgroup/process snapshots, raw caller populations and original PID absence.

The load VM's batch uses at most two CPUs/512MiB/64 tasks/512FDs, no restart,
180s RuntimeMaxSec and5s stop bound, with its internal170s deadline reserving
cleanup. Guest commands have a240s bound. Stop on invalid response populations,
errors/drops, incomplete phases, lost worker/CPU identity or deadline; don't
replay a partial population. Native receipt success, returned bundle SHA and
owned cleanup are all required; Azure provisioning success alone is insufficient.
An independent execution-mapping review checks this finite adaptation before
provisioning. It does not reopen accepted product qualifications.

## Acceptance and next step

Only this report and docs/BACKLOG.md, docs/ROADMAP.md and docs/README.md change;
prior sources, accepted studies and failures remain unchanged. Run existing
cargo xtask check docs and git diff --check for documentation changes. No new
benchmark framework, validator or qualification matrix is authorized.

Independent contract review precedes any conditional VM execution. YT-162 stays
In progress while direction/admission/execution is Pending; a reviewed contract
alone cannot mark an unperformed platform comparison Done. If Azure is selected,
final independent review must accept actual results, failures and resource
cleanup before Done. If physical-frequency attribution is selected instead,
report the missing actual target without creating VM resources that cannot
answer it. No Commit/Push or next product optimization follows automatically.

## Actual execution and retained failures in the first window

Private execution evidence is under evidence/azure. Independent contract Reviewer
/root/yt162_platform_comparison_contract_review_20261024 passed before execution.
The separate execution-mapping Reviewer
/root/yt162_platform_execution_mapping_review_20261024 initially found three
material harness gaps: shared deadline enforcement, an auxiliary inventory
failure bypassing deletion, and missing app-evidence retrieval after client
failure. The smallest changes to the existing transport/session closed those
paths; the same Reviewer returned **PASS, zero issues/required followups**.
The original FAIL, scripts and changes remain retained. No Yetlet product code
or accepted qualification was changed or replayed.

### First provisioning generation: worker-name admission failure

Two declared VMs and the task-only storage were allocated successfully.
App transfer hashes and native loader/library checks passed; the guest reports
Linux x64,glibc2.39. Protected Run Command/Blob transfer and authenticated internal
SSH operated. The app began listening, but the phase-start helper failed its
worker-name predicate before returning admission to the client.

The helper tested startswith('tokio-runtime'). The accepted Cargo compiler
receipt binds Tokio1.53.2; that exact local crate's builder source uses the
default name tokio-rt-worker, which cannot match the predicate. The original
client traceback, command receipts and server log are retained. The server
gracefully stopped with record_count0,invalid_ids0,invalid_thread_records0 and
overflow0. The client exited1 before its first k6 invocation; no warmup/measured
JSONL or scheduled HTTP population exists. This is a harness admission failure,
not evidence of poor Yetlet throughput or an HTTP/CPU regression.

The failed-instant worker snapshot was not saved before the assertion, so its
observed task count and original app PID remain **Not verified**. Do not convert
source configuration into an observed two-worker runtime claim. Client cleanup
records the tunnel reaped and app cleanup; subsequent full VM/resource deletion
closes the remaining lifecycle boundary.

The exact owned group and its16 inventoried resources were removed; absence and
zero owned resources were verified. Local temporary credentials were removed.
That generation completed cleanup in588.497329473s. Its152 original files are
preserved byte-for-byte under attempt1-worker-name-failure with archived bindings.
Only temporary identical input staging copies/completed upload archives were
deduplicated locally; accepted originals, input member/hash records and all
returned outputs/failures remain retained.

### Same-task correction and original-clock continuation

One smallest in-place correction changes the predicate to tokio-rt-worker and
saves its existing snapshot before the assertion. It preserves the two-worker
requirement, binaries/caller, limits, workload and failure history. The exact
Tokio source/hash and compiler binding are in worker-name-diagnosis.json.
Independent worker-correction review returned **PASS, zero issues/required
followups**, covering only one generation-two continuation for the four
populations that had never started.

The continuation retains the same task, owned group/owner, region/zone/size and
**original clock:06:40:01.684042–07:40:01.684042UTC on2026-10-24**, with new work
cutoff07:25:01.684042UTC. Prior verified deletion precedes re-creation; generations
do not overlap. It is not an allocation-failure fallback, favorable workload
replay, new task or renewed budget. No third generation was activated within
that original window.

### Second generation: Blob data-plane failure before guest setup

VM deployment and storage-account creation returned success. The subsequent
Blob container-create command returned native exit3 / ResourceNotFound at
06:56:25UTC. Its non-secret native diagnostic and command timestamps are
retained. No second-generation guest bootstrap, worker admission, k6 invocation
or workload ran. The corrected worker predicate therefore remains **not runtime
verified**. Existing root mirror-app/mirror-load and preexecution records from
generation one must not be relabeled as second-generation results.

The exact cause of ResourceNotFound remains unresolved. Management-plane success
does not establish a successful Blob operation; do not assert a credential,
DNS, account-name reuse or provider-readiness cause without new evidence.
The failure stops execution and starts owned cleanup. No different
account/region/size, third generation, new deadline or fabricated performance
comparison was activated within that closed window.

The second group's11 resources were removed; native absence plus the unique
owner/resource-group inventory reports zero remaining resources. Temporary local
credentials were removed, both local session processes were waited/reaped, and
both native session exits remain1. Cleanup completed at06:58:35.350407UTC:
221.115426779s from generation-two creation and1113.666365147s from the original
clock start. Both cleanup receipts are PASS within the unchanged3600s bound;
neither makes the unperformed comparison pass.

## Conclusion at the first window closeout

This section preserves the accepted06:58UTC closeout. The later selected
storage-first continuation and its new observations are reported below.

**Zero new k6 populations and no measured performance result.** The intended
four-phase independent-platform comparison is unperformed; YT-162 stays
In progress even if independent review accepts this report's failure/cleanup
evidence. YT-161's physical-frequency admission remains Not admitted and the
original local p95 cause remains unresolved.

The next material followup is a bounded investigation of the Blob data-plane
failure/readiness boundary before any separately selected live continuation.
Reuse the retained native diagnostics and corrected worker predicate; do not
replay accepted product qualifications or infer a Yetlet optimization target
from these operational failures. No Core/runtime/sample/API/output/security,
release-gate, commit or push change follows.

## User-requested Blob diagnosis and narrow continuation

The user asks to investigate the Blob failure and restore the four unstarted
comparison populations. Existing temporary-Azure/all-owned-cleanup permission
persists; no Commit/Push or product change is authorized. Preserve the accepted
failure report and its original completed resource window. At the new inventory
time08:20UTC, the old07:40UTC deadline had already expired: that window cannot
resume and is not renewed. This is a separately selected narrow continuation
with the concrete strategy change below, not a replay of completed experiments.

### Confirmed diagnosis and remaining uncertainty

Both stored ARM responses identify the same intended standard Blob endpoint,
Succeeded provisioning, available primary state, allowed shared-key access and
network rules with defaultAction Allow. Both container calls occurred about23s
after the respective account creation time; the first succeeded and the
same-name re-creation failed with ResourceNotFound. These facts rule out neither
provider routing/readiness nor historical DNS behavior. The former request's
resolved/connected address and HTTP trace were not captured; the later
cache-only query has no usable retained response. Current storage environment
selector variables are absent, which is not proof of their historical state.
Do not claim a uniquely established DNS, credential or provider root cause.

The concrete missing admission condition is an authenticated Blob data-plane
read before compute allocation. ARM provisioning success alone did not prove
that the subsequent data-plane call would work. Azure documents
[ResourceNotFound as HTTP404](https://learn.microsoft.com/en-us/rest/api/storageservices/common-rest-api-error-codes),
and [Get Blob Service Properties](https://learn.microsoft.com/en-us/rest/api/storageservices/get-blob-service-properties)
as an authenticated GET returning200 on success. Use the installed Azure CLI's
[blob service-properties show](https://learn.microsoft.com/en-us/cli/azure/storage/blob/service-properties?view=azure-cli-latest)
for that read; container-exists false is insufficient because it does not prove
a functioning account-level data service. Azure's
[endpoint guidance](https://learn.microsoft.com/en-us/azure/storage/common/storage-account-overview#storage-account-endpoints)
also warns that underlying addresses/CNAMEs can change. No host DNS/cache or
network configuration is modified to test that possibility.

### Finite storage-first continuation declaration

Evidence: evidence/blob-diagnosis and evidence/azure-ready. Independent review
must accept this mapping before any new resource is created.

1. Use one newly identified/tagged group and fresh account name, avoiding the
   prior deleted-name reuse as an experimental confounder without declaring it
   the proven cause. Keep East Asia, the selected image/twoD2s_v5, original
   executables/caller, workload, app/client limits and private transport.
2. Start one fixed60-minute resource window at first creation, stop new work
   by45min, reserve15min for owned cleanup. Preserve the old window/counters.
   No second allocation generation or region/size/zone fallback is selected
   within this continuation.
3. Create storage before compute. Record its actual primary Blob endpoint and
   DNS observation; require authenticated service-properties success. At most
   two native GET attempts within60s, each at most20s; only the known
   ResourceNotFound permits one10s wait/recheck. A repeated signature or any
   other failure stops without VM allocation.
4. Create the private container once and round-trip exactly1KiB of synthetic
   bytes, checking SHA256/byte identity. These are new-target admission checks,
   not accepted product-qualification replays. Only after both gates succeed
   allocate the declared two VMs and run the four still-unstarted populations
   once. Every failure still stops later phases and triggers exact cleanup.
5. Keep all raw JSONL in the returned hash-verified ZIP. Extract only one
   population temporarily for existing analysis, then remove that redundant
   extraction; retain the canonical ZIP and all failures. This preserves the
   existing256MiB total local evidence/output bound alongside immutable prior
   history. Temporary identical input copies/upload archives may be removed
   after upload; their original sources/manifests/target hash receipts remain.
6. Verify owned group absence, zero owned resources, removed credentials and
   reaped processes on success/failure. Independent final review is still
   required before Done. A successful fresh-account gate is bounded new
   transport evidence, not a retrospective explanation of the earlier404 or
   physical-frequency causality.

## Storage-first continuation results

Independent continuation Reviewer
/root/yt162_blob_readiness_continuation_review_20261024 returned **PASS, zero
issues/required followups**, before resource creation. The old window and all
359 retained evidence bindings remain unchanged. The new window started at
08:40:07.680308UTC with its own fixed09:40:07.680308UTC deadline; it did not renew
the expired earlier window.

### Blob path restored before compute

The first authenticated service-properties GET succeeded without recheck. The
private container was created once; the1024-byte upload/download matched exactly,
SHA256785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9.
Only afterward did the declared two VMs allocate. The accepted Run Command /
protected Blob / internal SSH path transferred exact accepted artifacts and
returned hash-verified evidence. This is positive admission of this fresh target,
not proof that DNS or readiness uniquely caused the earlier404.

The corrected worker predicate now has actual runtime evidence: all four app
start snapshots contain exactly two tokio-rt-worker tasks. The same reference /
current binaries, k6, caller, four-phase order, workload, response/CSP checks
and app/client limits remain unchanged. No accepted build/test/qualification
or completed workload population was replayed.

### Four once-only measured populations

All completed requests pass the unchanged body/header/CSP checks with HTTP200;
there are zero HTTP errors and zero interrupted started iterations. Warmups
retain1003 valid200 responses. Measured totals are39977 valid200 plus27 scheduled
drops, or40004 scheduling outcomes against nominal40000. Each measured point
retains one start at/after its nominal end, accounting for the four boundary
starts. Native raw points and completion tails remain intact without filtering.

| Phase | Valid completed200 | Scheduled drops | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms | Sampled construction CPU p50 microseconds | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| reference1 | 10001 | 0 | 0.725295 | 4.762528 | 11.882318 | 75.485 | Comparable first pair |
| current1 | 10001 | 0 | 0.686209 | 3.882626 | 10.614350 | 74.495 | Comparable first pair |
| current2 | 10001 | 0 | 0.684911 | 4.209372 | 10.512432 | 77.087 | Retained; paired reference fails |
| reference2 | 9974 | 27 | 0.733113 | 5.455808 | 15.989153 | 75.091 | Failed scheduled point; exclude pair two ratios |

Quantiles use the reused nearest-rank method on completed valid responses, not
latencies assigned to unstarted/dropped requests. k6's exported interpolation
can differ slightly; both its original summaries and every raw point remain.
HTTP duration includes this VM network/SSH path. Integer-millisecond full-call
latency is a separate population; its p95 values are5/4/4/6ms in phase order.
The failed reference2 completed-response maximum is121.683587ms; it is retained.

Only pair one supports the declared within-pair observation: current HTTP p95 is
18.48% lower (ratio0.815245), while synchronous construction CPU median is1.31%
lower (ratio0.986885). CPU means are81.613/81.859microseconds and CPU p95 is
114.844/118.890microseconds: do not turn a small median difference into a
general CPU speedup. This one short qualified pair does not reproduce the
earlier local median CPU increase, but cannot establish its unique cause,
statistical equivalence, universal performance rank or capacity.

The sampled populations contain1283 phase-scoped unique correlated records and2566 valid
same-thread clock measurements, with no invalid clock/identity/CPU-affinity
record. CPU samples remain synchronous construction/empty brackets; no CPU
duration is taken across await. Sparse HTTP-tail sample counts are18/14/15/15;
neither those cohorts nor the signed gaps are sufficient for a unique mechanism.

### Failed reference2 and observation limits

Native k6 exits0 in all eight invocations. The controller deliberately exits1
after reference2 reports27 dropped_iterations and warns that its64-VU ceiling
was reached. There is no further phase or replay. The final regular stop call
is skipped by that failure path; the owned cleanup stops the app/tunnel and
retains its server dump. The normal phase-end resource snapshot and direct
original-PID absence observation for that last app are therefore Not verified;
later MainPID0/inactive and full VM deletion close the lifecycle boundary.

Measured client-group mean CPU is108.91–109.82% of one logical CPU. Native
boundary deltas report no CPU throttling, memory.max/oom/oom_kill events, or
interrupted started iterations; cumulative observed memory peak grows from
112.76 to275.24MiB. These are invocation-boundary observations, not proof of
ample headroom at every instant. The warning and transient tails do not identify
whether client scheduling/GC, VM/SSH/network delay or server waiting exhausted
the active VUs. Do not label27 drops as HTTP errors or prove a Yetlet capacity
ceiling from them. Pair two has no qualified ratios.

One minimal offline analysis correction records the real failed point and
cleanup-only stop instead of asserting all points pass. Its prior source and
repair authority are retained; actual native failure, thresholds, binaries,
caller and workload are unchanged. Analysis executes after cloud deletion,
exit0 in3.470303s. It does not rerun the load or subtract failed observations.

### Retention, cleanup and acceptance boundary

The canonical hash-verified returned ZIP retains all eight raw JSONL populations,
including the failed one; only one population is temporarily extracted for
analysis and that duplicate extraction is removed. Raw tails, summaries, native
exits, worker/clock/resource bindings and all older failures remain. Total local
evidence including immutable history is about83MB, below256MiB.

The exact group and all16 inventoried resources are removed; native group
absence and unique-owner inventory verify zero remaining resources. Temporary
credentials are removed; the local session process is waited/reaped with exit1.
Cleanup finishes at08:47:36.587384UTC,448.907076s after first creation, within
the fixed3600s window. Three normal app stops record original-PID absence; the
last app is closed through cleanup and VM removal. Cleanup PASS does not change
the failed scheduled point or native controller exit.

Final independent review may accept **bounded platform-reference study scope**:
one comparable pair plus the preserved failed second pair, with no product
optimization and no claim that all four points passed. No additional cloud
execution is selected by this closeout. YT-161 physical-frequency admission and
the original local p95 causality remain unresolved at their existing scopes.

## Remaining gap and followup

[YT-163](BACKLOG.md) is a future opt-in attribution item for the retained
reference2 dropped arrivals. Start with the raw start/completion/drop timeline,
64-VU warning, client-group boundaries and server CPU/probe records; distinguish
load-generator/VM/tunnel scheduling from server waiting before selecting a
change. Preserve the failed point; do not raise VUs, memory, hardware or workload
silently to manufacture a passing number, and do not make consumer hardware/SLOs
a framework gate. New live verification needs its own bounded contract and user
activation. It is not activated by this study or a commit operation.
