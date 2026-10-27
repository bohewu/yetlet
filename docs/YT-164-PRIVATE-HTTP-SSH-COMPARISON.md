# YT-164 — Private direct HTTP versus SSH transport

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded isolated transport-comparison scope after Independent final Reviewer /root/yt164_transport_results_final_review_20261024 PASS, zero issues/required followups. Four comparable populations and owned cleanup complete; historical27drop cause unresolved.**

## Authority and question

The user activates the minimal isolated followup proposed by
[YT-163](YT-163-DROPPED-ARRIVAL-ATTRIBUTION.md), after its four documents are
committed separately. Compare private direct HTTP with the internal SSH-forward
path while preserving the existing workload, fixed load and response checks.
Remove all task-created test resources on success or failure, preserve failures,
and obtain independent Reviewer PASS before closing the work. New changes must
not be committed or pushed.

This asks whether a path-associated latency/drop difference appears in the new
bounded observations. It does not prove the unique historical cause of YT-162's
27drops, physical-frequency control, sustained capacity or a product defect.
Preserve all accepted results and failures in YT-162/163. Reuse their product,
browser, CSP, provider and platform qualifications without rerunning them.
Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). This stays optional
application/benchmark work, not a consumer hardware/SLO or release requirement.

## Smallest application adaptation

The accepted sample executable binds loopback and the caller addresses loopback;
neither can exercise actual private direct HTTP unchanged. Add only two explicit
sample overrides: YT164_BIND_IPV4 for the diagnostic SSR executable and
YT164_TARGET_IPV4 for the existing caller. Unset means127.0.0.1; explicitly set
values must be127.0.0.1 or a canonical RFC1918 IPv4 address. Reject unspecified,
public, hostname, port/path and malformed values before starting a request or
listener. The executable override is compiled only with ssr-stage-probe.
No mandatory dependency, Core API, renderer, response or default binding changes.

Build the current diagnostic executable once, offline/locked, into a new target
directory. The endpoint adapter changes binary identity; record its actual
source/lock/compiler/build hash rather than claiming the old executable is
unchanged. Use **this same executable for both paths and all four phases**;
this is a transport comparison, not a new reference/current product benchmark.
Reuse the accepted k6 executable unchanged. Caller changes must be limited to
address admission and URL host selection; keep its literal body/header/CSP
oracle, medium128rows/31063bytes, timing, metadata and VU settings unchanged.

Focused new-adapter verification consists of compilation, invalid-address
rejection and actual new-target listener/path admission. Existing semantic and
runtime qualifications are reused; every new workload response still executes
the unchanged full correctness checks.

## Finite isolated execution contract

| Field | Selected bound |
| --- | --- |
| Ownership | One new task-tagged Resource Group, no existing service/resource modification; one allocation generation, no region/size/zone fallback |
| Lifetime |60min from first resource creation, stop new work by45min, reserve15min for cleanup |
| Platform | Two Standard_D2s_v5 VMs, East Asia zone1, pinned Ubuntu24.04 image from YT-162; same app VM and separate fixed client VM for all phases; Standard SSD OS disks only |
| Storage/control | Reuse accepted storage-first authenticated Blob GET/1KiB hash roundtrip before compute, Managed Run Command and protected private Blob transfer; no public app/SSH inbound rule or credential in repository/logs |
| Network | App listens on its owned private IPv4/39262 for both paths. Direct caller targets that exact endpoint. SSH caller uses127.0.0.1/39262 forwarded to the same private endpoint; authenticate the host key from Run Command. Explicit private client-to-app rules for TCP22/39262, deny other inbound traffic |
| Artifact | One current diagnostic server hash, one unchanged accepted k6 hash, one address-adapted caller hash; record exact manifests before any load |
| Order | SSH1 / direct1 / direct2 / SSH2; fresh app instance each phase, same two-worker executable and resource limits |
| Load | Each phase125/s2s warmup then1,000/s10s measured,64 fixed VUs/maxVUs,8s graceful/request timeout, existing full response oracle and connection reuse |
| App limits | CPU0–1/quota200%,256MiB/128tasks/512FDs,45s maximum lifetime,5s stop bound, no restart |
| Client limits | CPU0–1/quota200%,512MiB/64tasks/512FDs,180s maximum lifetime, internal170s deadline reserving cleanup, no restart |
| Observation | Existing modulo32 request-ID ingress/pipeline/CPU probes; finite read-only50ms external guest/process/cgroup observations on both VMs, including client SSH PID; no product instrumentation change. Record observer cadence/CPU cost, missing samples and coverage limits |
| Evidence | Retain all native exits, raw warmup/measured JSONL, tails/drops and resource/probe records; stream canonical returned ZIPs, no persistent raw duplicate,256MiB new evidence bound excluding compiler cache |
| Stop | Artifact/path/response integrity failure, app expiry, native nonzero, incomplete population or deadline stops later phases; drop-bearing points remain failed/unqualified. No favorable replay, VU/memory escalation or extra load matrix |
| Cleanup | Stop/reap owned client/tunnel/app/observer activity, retrieve available failure evidence, delete exact owned group; verify absence and zero task-owned resources, remove temporary keys/SAS material |

The accepted YT-162 SKU-query timeout and earlier Blob404 remain historical
limits; no new broad quota/SKU/qualification scan is required. Read-only account
and ownership checks bind the current session before creation. Storage/path and
ABI checks are admission of a new target, not a replay of accepted validation.
The earlier USD0.264/two-VM-hour compute estimate is historical, excludes other
charges and is not a cap or updated quote. Resource lifetime remains bounded.

## Interpretation and acceptance

Report each point's scheduling outcomes, HTTP/full-call distributions and native
components, then within-pair direct/SSH observations only when both points are
complete/comparable. Never assign latencies to unstarted arrivals, remove failed
points or promote sparse VU/resource sampling to continuous peak proof. The two
short balanced pairs cannot establish statistical equivalence or diagnose a
rare historical pause from its absence.

Use request IDs for cross-VM joins without subtracting unrelated clocks. Existing
server ingress and pipeline elapsed values describe sampled middleware entry to
response-ready work, not accept queues or body transmission. The external50ms
observer can miss a28ms burst; measured observer cost is additional context,
not proof of zero disturbance. Separate client scheduling, VM/SSH path and
instrumented server waiting as far as coverage permits, preserving unresolved
alternatives.

Independent contract review precedes adapter implementation; independent review
of the concrete execution mapping precedes resource creation. Final independent
review must cover actual measurements, failed points, adapter scope and verified
cleanup before Done. Run existing documentation checks and focused changed-code
checks only. No new benchmark/validation framework or qualification matrix.
Permitted repository paths are this report, the three documentation indexes,
the sample SSR source, caller and sample README. New changes remain uncommitted.

## Concrete adaptation before provisioning

Independent contract Reviewer /root/yt164_transport_contract_review_20261024
returned **PASS, zero issues/required followups** before implementation.
Private local scripts and evidence are retained in
`target/yt164-private-transport-20261024/evidence`.

The new diagnostic address-admission unit test passes; the offline/locked
diagnostic release build passes with its own source/lock/compiler/artifact
receipt. Sixteen no-HTTP k6 inspect cases admit the default/four valid private
or loopback inputs and reject eleven invalid inputs. Seven native invalid
server inputs exit before listening. The first inspect invocation omitted
its required environment-inclusion flag and failed fixed-workload admission;
that harness failure is retained. One smallest correction adds the inspect
flag with the already minimal synthetic environment, without changing the
caller, load or response checks to pass. No accepted test or workload is replayed.

The execution mapping adapts the accepted YT-162 storage-first transport and
cleanup helpers in this task's private directory. One new owner/group/storage
identity and private subnet replace the old identities; the old scripts and
evidence remain intact. The app executable binds its private endpoint in all
four phases. The same authenticated SSH forward stays owned by the client
batch; direct phases bypass it. The network rules allow only the private load
VM to reach app TCP22/39262 and deny other inbound traffic, including public
inbound traffic. Run Command remains the operator control/result path.

Each app phase has a separate bounded observer unit, stopped during phase stop
and failure cleanup. Each k6 invocation has an owned observer process, terminated
and reaped after native completion/failure. The app observer also records existing
sshd processes; the client observer records k6 and its exact SSH PID. The same
read-only50ms process/thread schedstat, guest CPU/pressure/network observations
are used on both paths. Observer summaries retain actual cadence, skipped slots,
CPU time and memory; no claim of scheduler tracing or zero observer overhead.
Their45s/900-sample limits and owned lifecycle remain distinct from the actual
workload units. Failure still stops later phases and invokes available app
evidence retrieval plus exact group deletion within the original fixed window.

## Actual once-only isolated results

Independent execution-mapping Reviewer
/root/yt164_transport_mapping_review_20261024 returned **PASS, zero
issues/required followups** before any resource creation. The single declared
storage-first allocation proceeds without retry/fallback: first authenticated
Blob GET succeeds,1KiB roundtrip matches, then the two declared VMs deploy.
Guest hashes/ABI, the private binding, authenticated SSH path and actual
two-worker runtime are admitted. All four app instances use the same new
current executable, SHA256
47d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65.
The accepted k6 hash is unchanged; the caller's only source difference is
private-address admission and URL host selection.

All four measured populations finish with10001valid200 responses each,
40004total; the four warmups retain1004valid200. Native exits are zero, with
zero HTTP errors, drops or interrupted started iterations. All responses execute
the unchanged complete body/header/CSP oracle. Each point retains its one
nominal-end boundary start, with completion tails preserved. There is no
favorable replay or additional load point. Both within-pair comparisons qualify.

### HTTP and sampled server observations

| Measured phase | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms | HTTP maximum ms | Sampled pipeline p50 ms | Sampled construction CPU p50 microseconds |
| --- | --- | --- | --- | --- | --- | --- |
| SSH1 | 0.717482 | 5.489663 | 12.829500 | 23.889308 | 0.190993 | 79.075 |
| direct1 | 0.348023 | 2.759642 | 9.052095 | 17.766869 | 0.175004 | 68.256 |
| direct2 | 0.350822 | 2.484601 | 8.482415 | 21.821115 | 0.175404 | 69.077 |
| SSH2 | 0.703053 | 5.334995 | 13.343379 | 26.655051 | 0.188829 | 79.222 |

Nearest-rank quantiles use every completed valid measured request; original k6
interpolated summaries and raw points are retained separately. Within pair one,
direct/SSH HTTP p95 ratio is0.502698 (49.73% lower); within reversed pair two
it is0.465718 (53.43% lower). Median ratios are0.485062/0.498998. These are
path-associated observations using the same current product, not an SSR
optimization or a product-version speedup.

Waiting means are1.342282/0.649430/0.625430/1.312034ms in phase order;
waiting p95 is4.708784/2.300664/2.075413/4.568488ms. Receiving means are
0.094265/0.072028/0.067862/0.093675ms. Blocked/connecting remain separate from
HTTP duration and are retained. The integer-millisecond full-call p95 is6/3/3/6ms;
its1ms clock cannot resolve submillisecond differences reliably.

The four server dumps retain1284samples,313measured plus8warmup each, with
2568valid same-thread construction/empty clocks and zero invalid IDs, clock
records or overflow. Pipeline p95 is0.280350/0.269755/0.275801/0.248162ms.
These short sampled brackets cannot account for the entire HTTP waiting path;
they omit pre-middleware scheduling and body transmission. CPU medians also vary
with path/context, so do not subtract marginal HTTP/pipeline quantiles as exact
wire overhead or attribute the difference uniquely to encryption. No physical
frequency control or cross-VM timestamp subtraction is introduced.

### Load generator, SSH and VM context

The following CPU means come from each process's first/last readable observer
samples, with guest clock-tick quantization. Client intervals cover their
measured k6 invocation; app intervals cover the whole warmup/measurement/control
phase and are not a synchronized measured-only cross-VM population.

| Phase | Client k6 CPU % of one CPU | Client SSH CPU % of one CPU | Active app-side sshd CPU % of one CPU | App SSR CPU % of one CPU |
| --- | --- | --- | --- | --- |
| SSH1 | 90.490 | 19.412 | 13.158 | 18.571 |
| direct1 | 90.491 | 0.000 | 0.000 | 16.541 |
| direct2 | 89.804 | 0.000 | 0.000 | 16.629 |
| SSH2 | 90.000 | 19.412 | 12.884 | 17.903 |

The same SSH forward remains alive in direct phases; it is bypassed and its CPU
delta is below the tick resolution, not proven literally cost-free. k6 CPU is
similar across paths, while active forwarding adds observable client/server
process work. Group-boundary mean client CPU including the observer is
111.54/92.71/91.69/110.90% of one CPU within the unchanged two-CPU quota.
There are no recorded client throttling or memory max/OOM events; cumulative
client memory peaks are125.22/178.32/234.51/289.82MiB. Guest boundary steal deltas
are zero, not a guarantee against every hypervisor/guest scheduling pause.

The12finite observer populations retain2066records, zero skipped nominal slots
and no process-read error records. Their largest observed sampling gap is
52.709748ms. Measured client observer CPU averages1.94–2.13% of one CPU;
app observers average1.87–1.92%, with no claim of zero disturbance. Scheduler
counters are nonzero/readable. In SSH1, one app sshd thread's largest
between-sample runqueue-counter increment is21.299697ms and the SSR worker's
is9.365362ms; these counter increments are not exact trace timestamps or an
identified mechanism. Other phases also show scheduling context: direct caller
start gaps reach11.114381/13.848124ms without dropping an arrival. Thus the new
data supports measurable SSH-path cost alongside guest/client variability,
not an assertion that SSH uniquely caused every tail.

Native reconstructed peak HTTP overlaps are20/13/17/20, below64in these finite
populations. The earlier64-overlap/27drop burst does not recur on either path.
Its absence cannot establish that the old failure was fixed, exclude a rare VM
or client stall, or retroactively qualify YT-162's failed second pair.

### Retention, resource removal and interpretation

All raw warmup/measured JSONL, observer populations, server dumps, native receipts
and original summaries remain in hash-verified returned ZIPs. Offline analysis
streams the canonical files without raw extraction and starts only after cloud
cleanup. New evidence is about78MB, within256MiB; compiler cache is separate.
All historical source/artifact/failure bindings remain unchanged. The initial
inspect harness failure is preserved alongside the focused corrected result;
there is no live workload failure or retry in this allocation.

Four normal app stops record original PID absence; observer units and all client
observer processes close, and the authenticated SSH process is terminated/reaped
with PID absence. The controller exits0 and its local session process is
waited/reaped. The exact group and all16inventoried task resources are removed,
with group absence, zero owned resources and removed local credentials verified.
Resource lifetime is515.125192s versus the3600s bound; native session elapsed is
517.142539s. No test resource remains and no existing service was modified.

For later framework HTTP/capacity measurements, prefer the isolated private
direct path and retain SSH for control/artifact transfer. Label historical
SSH-path results with their transport context and keep them unchanged. Any
further capacity/load study still needs its own bounded activation; this result
does not authorize extra points, a default product listener change or a Core
optimization. The unique historical27drop cause and physical-frequency question
remain open at their accepted scopes. Final independent review must accept the
actual results, scope and cleanup before this item is marked Done.
