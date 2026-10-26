# YT-167 — Sustained private HTTP capacity at 6k requests/s

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded research scope; independent contract/mapping/final Reviewer PASS, zero issues/required followups. The6k offered load remains failed; no new-change commit/push.**

## Authority and bounded question

The user authorizes the separate six-path YT-166 strict/builtin commit without
Push, followed by contract-first, minimal isolated longer-duration validation
at6000arrivals/s. Preserve the existing workload/response checks, accepted
verification and failed observations; delete all test resources and obtain
independent Reviewer PASS. New YT-167 changes must not be committed or pushed.

[YT-166](YT-166-PRIVATE-HTTP-CAPACITY-REFINEMENT.md) observes a clean6k point
for10s and pressure at7k. This study asks whether the same app profile maintains
its behavior through **one continuous300s exposure at6k**,30times the previous
duration. Five minutes is a selected bounded first sustained test, not an
hours-long soak, universal capacity, production SLO or hardware requirement.
No short6k point, other offered rate, recovery or accepted qualification is
replayed. Old observations from a different VM allocation remain context.

Reuse the exact accepted current app executable
SHA25647d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65
and accepted k6 without rebuilding. Preserve medium128rows/31063bytes, the
full literal body/header/CSP oracle on every response, HTTP1.1 identity
encoding, connection reuse,8s request/graceful timeout and256fixed VUs. No
Core/runtime/API/output/dependency/default listener change follows. Preserve
[ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md).

## Selected once-only execution

| Field | Bound selected before execution |
| --- | --- |
| Ownership | One new task-tagged group/nonce and allocation generation; same East Asia zone1/pinned Ubuntu24.04 image, no size/region fallback, existing-service modification or load retry |
| Resource window |60min from first creation; stop new work by45min and reserve15min for exact-owned cleanup |
| Guests | Same app Standard_D2s_v5 and separate load Standard_D16s_v5; actual2/16core admission, standard SSD OS disks only, no database/Valkey/data disk |
| App | Same2workers,CPU0–1/quota200%,256MiB/128tasks/512FDs/Nice10, no restart; one original app PID,360s maximum lifetime/5s stop |
| Offered workload | New125/s2s admission warmup on this app, then6000/s300s once; stop app/observer before offline raw reconciliation. No extra recovery/load phase |
| Generator | Same16CPU quota/256VUs/512tasks/8192FDs;32GiB memory and16GiB per-file/raw tmpfs cap selected for full-duration evidence,600s hard unit/570s internal deadline |
| Observation | Same50ms read-only process/thread/guest/cgroup records; finite420s/8400record ceiling and360s app-observer unit, recorded observer cost/gaps/errors |
| Capture | Full native request JSONL in owned tmpfs;16GiB total raw,2GiB new local evidence. Stream redacted lines into canonical ZIP without whole-file buffering or persistent raw extraction |
| Transfer | Existing storage-first authenticated Blob GET/1KiB roundtrip before compute, protected Run Command/private Blob,840s guest command bound and bounded upload/download |
| Data path | Private direct TCP39262; client-only22/39262 then deny other inbound; authenticated SSH is control only, no HTTP tunnel or public benchmark listener |

At the retained approximate raw rate,300s needs roughly11GB of request JSONL;
the old4GiB total/2GiB per-file limits cannot retain it. Only diagnostic/client
capture headroom and necessary finite duration limits increase, before review
and provisioning. App hardware/limits, request concurrency, body checking and
offered rate do not change. Admit actual tmpfs free space and memory limits
before load. Report the changed client capture envelope explicitly; do not
pretend every generator setting is identical to YT-166. Never enlarge limits
or allocate a replacement in response to failure.

Extend only the existing caller's fixed plan with new admission/sustained
labels. Preserve all old entries and the code from the CSP/oracle onward.
The existing sparse formula can serve a single300s measured phase: at nominal
1800000iterations it emits440measured records, largest ID16096, plus one
phase-distinct warmup record. These stay below the unchanged ID20000 and
1024record server limits; do not invent overlapping additional measured phases
or enlarge diagnostic capacity. Keep explicit server_probe_id joins distinct
from request probe_id. Inspect only the two new plans and one invalid selector,
without HTTP; prior endpoint/build/browser/CSP/provider checks remain reused.

## Outcome, pressure and failure semantics

Retain every outcome of the once-only300s exposure, including scheduling or
latency pressure. Reuse the descriptive pressure marker: any unstarted drop,
HTTP error, invalid/incomplete response population or native overall p95 at
least50ms. The marker is a study rule, not a production SLO. Pressure alone
does not trigger a load replay, resource increase or favorable-result selection.

A200 body/header/CSP mismatch aborts through the unchanged caller. App crash,
restart, native failure, lost identity, observer/capture/resource failure or
deadline hard-stops on detection and enters cleanup. Warmup admission failure
prevents the measured point. Retain partial/failed results; never mark an
incomplete exposure as a sustained success. A finite failed study can still be
documented as completed research with independent acceptance and its failed
load conclusion intact.

Report starts, completions within each half-open client-clock scheduled window,
late starts, unstarted drops and graceful tails separately. Total completions
divided by300s are not necessarily in-window RPS. Preserve native interpolated
summaries alongside exact raw completed-response quantiles. Keep failed
scheduling outcomes outside response-latency distributions.

Report ten successive30s windows, first/last60s latency populations and app/
client CPU/memory/queue context. An aggregate clean marker alone is not proof
of stationarity: retain localized tails/drops and any memory trend, with
coverage/observer cost stated. Memory observations include cumulative tmpfs
capture on the generator; do not call that app memory growth or an app leak.
Do not infer a leak from a finite increase or subtract unrelated VM clocks.
Near-app-quota activity can support profile saturation, not pure Core attribution
or proof that generator/network/diagnostic disturbance is absent.

## Safety, cleanup and acceptance

Bind accepted sources/executables and concrete scripts before the only
allocation. Reuse the exact-owned storage-first lifecycle and prior failure
history. The full new path must fit the original60min resource window; no
deadline reset, duplicate/uncertain execution or automatic new allocation.
One smallest canonical harness correction per distinct seam cannot erase
failures or replay completed load; a repeated infrastructure signature stops.

On every outcome, stop/reap app, observers, client and control activity, retrieve
available failure records, delete the exact task-owned group, verify group
absence/zero owned resources, remove temporary credentials and reap the local
session. Preserve full failed data in bounded canonical artifacts. Resource
cleanup remains mandatory if execution/capture fails.

Independent contract review precedes adaptation, concrete mapping review
precedes provisioning and final independent results/cleanup review precedes
Done. Required repository paths are this report, three docs indexes, the
existing capacity caller and sample README. Documentation/diff checks apply
to new changes; no old qualification replay, new benchmark framework, Core
optimization, consumer SLO gate or new-change Commit/Push.

## Concrete mapping before provisioning

Independent contract Reviewer /root/yt167_sustained_contract_review_20261024
returns **PASS, zero issues/required followups** before adaptation. Private
evidence is in `target/yt167-private-sustained-20261024/evidence`. Snapshot the
committed YT-166 caller with its commit/blob/SHA binding. Only two fixed plan
entries are added; all old entries and code from the CSP/oracle onward remain
byte-identical. Both new plan inspections and one invalid-selector rejection
pass without HTTP. No old build/test/endpoint/workload qualification runs.

Six canonical task-local seams adapt the accepted lifecycle, per-stage control,
observer and capture to one new300s point. Focused syntax checks pass. The
actual input mapping binds11files and74retained artifacts/history bindings.
Concrete app admission and stage boundaries keep original-PID/resource and
nonempty-live-observer checks; normal stop records PID absence and successful
observer completion. Client observers are stopped/waited with exit/capture
receipts before summaries are accepted. These accepted failure guards are
preserved; the earlier mapping FAIL and correction remain in retained history.

The known raw directory remains tmpfs. Line streaming with the existing
redaction function into ZIP64 avoids buffering a300s raw file in guest memory.
The local controller checks declared result bytes against remaining2GiB
headroom before download and total retained bytes after extraction; request
JSONL remains only in canonical verified ZIPs. Larger capture bounds and
840s guest/600s client/360s app durations match the selected contract. Exact-owned
cleanup and absolute resource-window admission remain. Independent mapping
Reviewer /root/yt167_sustained_mapping_review_20261024 returns **PASS, zero
issues/required followups** before the only allocation.

## Actual once-only sustained exposure

One storage-first allocation passes the authenticated Blob GET/1KiB roundtrip,
actual guest core/resource/tmpfs checks, protected transfers and exact binary/
input hashes. One original app PID serves the admission warmup and full300s
measured exposure with no restart. Both native calls finish, all started
iterations complete, and raw reconciliation matches their summary populations.
No short point, recovery, earlier qualification or failed load is replayed.

### The6k offered load fails the clean marker

| Population | Valid200 completions | Unstarted drops | Valid in-window responses/s | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms |
| --- | --- | --- | --- | --- | --- | --- |
|6000/s for300s |1765706 |34302 |5884.83 |43.115404 |63.332836 |68.435924 |

Admission adds251valid200 and zero drops. Every started request passes the
unchanged full body/header/CSP oracle; HTTP errors, response mismatches and
interrupted iterations are zero. Nonetheless34302unstarted arrivals represent
1.9057% of nominal1800000arrivals, and native overall p95 is63.332823ms.
This is a **failed offered-load observation**, not sustained clean6k capacity.
The study completes while preserving that failure; no resource increase,
replacement allocation or favorable replay follows it.

The table uses raw nearest-rank completed-response quantiles. The runtime
pressure rule uses the separately retained native interpolated summary.
Starts plus drops total1800008against nominal1800000; late execution and
boundary accounting must not become extra capacity. Five starts occur at/after
the scheduled end. Inside the half-open300s client-clock window1765449valid
responses complete;257finish later, with42.948786ms observed HTTP drain.
Dividing all completions by300would instead report5885.6867and hide that tail.
Full-call1ms-quantized p50/p95/p99 are43/63/69ms, a distinct timing population.

### Pressure is present throughout the observed windows

| Client completion window s | Valid completions | Drops timestamped in window | Valid responses/s | HTTP p95 ms |
| --- | --- | --- | --- | --- |
|0–30 |176267 |3479 |5875.57 |62.631487 |
|30–60 |176734 |3267 |5891.13 |62.901285 |
|60–90 |176898 |3101 |5896.60 |63.465727 |
|90–120 |177068 |2933 |5902.27 |63.506587 |
|120–150 |177287 |2713 |5909.57 |63.057865 |
|150–180 |174600 |5459 |5820.00 |64.825993 |
|180–210 |177130 |2814 |5904.33 |62.798952 |
|210–240 |176566 |3435 |5885.53 |63.317165 |
|240–270 |176050 |3948 |5868.33 |63.951200 |
|270–300 |176849 |3153 |5894.97 |62.834156 |

All ten windows retain drops and p95 above the descriptive50ms marker. The
first/last60s completed-response populations have353001/352899responses and
p95 62.775249/63.379719ms. Pressure already appears in the first30s window;
these observations do not show an initially clean period followed only by a
late collapse. The150–180s interval is worse, and every interval remains.

The old clean10s6k result and this new exposure use different VM allocations.
Keep the accepted old result intact. This run rejects a clean6k claim on its
own target, but cannot isolate elapsed duration as the cause of the difference
or establish that every allocation fails at6k. There is no within-target short
baseline replay and no unique physical-frequency/VM/kernel explanation.

### Resource and memory context

The app before/after resource envelope averages193.99% of one CPU out of200%,
including native startup/control/drain. Server-side before-marker50ms CPU
intervals have median195.87%, with5980of6042intervals at least180%. Waiting
median/p95 are43.057563/63.277010ms. Persistent near-two-CPU app activity,
waiting and limited delivered rate support **server/profile CPU saturation**.
Sparse middleware clocks omit accept/runtime queues and transmission; this
does not identify a particular Core operation or exclude all network/client
effects. Finite non-atomic accounting intervals are not continuous peak traces.

The app records21quota-throttling events totaling2546microseconds, with zero
memory max/OOM events. This small cumulative throttling count alone does not
explain63ms tails. App charged cgroup peak is12.36MiB against256MiB. Within
the server's own marked observation population, first/last60s mean charged
memory is11.20/11.33MiB; maximum sampled memory stays below12MiB. These finite
samples do not establish an unbounded growth trend or prove leak freedom.
They are not exactly aligned with client completion windows or heap-only data.

The measured client envelope averages459.62% of one CPU,28.73% of the16-CPU
quota; the readable k6 process average is455.80%. Its measured cumulative
cgroup peak is10.86GiB against the selected32GiB. Client quota-throttling and
memory max/OOM events are zero. The11.06GB total logical raw request capture
stays below16GiB. Cumulative generator memory includes tmpfs evidence, not app
memory growth. No declared generator/capture cap is reached; this does not
prove serial generator/GC/network/diagnostic costs are absent.

Three finite observer populations retain12193records, zero skipped nominal
slots and zero process-read errors. Largest observed gap is90.198043ms. The
app observer costs2.38% of one CPU; the measured client observer costs3.83%.
Report that disturbance rather than treating observation as free. The server
retains433unique sparse records/866valid same-thread clocks, with zero invalid
IDs/clock records/overflow, below unchanged diagnostic bounds. Explicit
server_probe_id joins remain distinct from request IDs; VM clocks are not
subtracted or aligned to invent causal precision.

### Retention and exact cleanup

The full failed offered-load population, native logs, summaries, raw JSONL,
markers, observer/termination receipts and prior failures remain. Canonical
verified ZIPs retain11060108286logical raw request bytes without persistent
extraction or whole-file guest buffering. New local evidence is approximately
405MB against2GiB. All three bounded offline calculations run after cloud
removal, exit0 and reap their processes; raw calculation takes188.04s.

Normal stop confirms the original app PID absent; app/observer units, client
observers and control activity close. The client unit also completes its
post-stop reconciliation within416.658697s, below600s; that extra processing
is outside the fixed300s load and app lifetime. The exact group and all16
inventoried test resources are deleted, with group absence, zero owned
resources, removed temporary credentials and unchanged accepted bindings
verified. Resource lifetime is1055.575621s against3600s. Native controller
elapsed is1057.020178s, exit0, and its local process is waited/reaped. That exit
means the bounded study/cleanup completed, including the failed offered load.

## Conclusion and remaining gap

This target delivers approximately5885valid in-window responses/s under a
6k offered rate, with failed scheduling and63.33ms p95 throughout the observed
windows. It provides **no clean sustained6k qualification**. Neither the old
clean short result nor this failure is overwritten. CPU saturation is supported
at profile scope; duration-only causation, a universal ceiling, production SLO,
hours-long soak and consumer/business workloads remain unqualified.

A separately authorized longer test at a lower offered rate could seek usable
headroom. No new rate, allocation, Core optimization or backlog item is
activated here. Independent final Reviewer
/root/yt167_sustained_results_final_review_20261024 returns **PASS, zero
issues/required followups**, accepting completed bounded research with the
failed offered load retained. Reviewed source/evidence hashes and exact copies
remain; only status/review-reference lines change after acceptance. New changes
remain unstaged, uncommitted and unpushed.
