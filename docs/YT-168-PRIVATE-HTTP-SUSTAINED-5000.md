# YT-168 — Sustained private HTTP capacity at 5k requests/s

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded profile-observation scope; independent contract/mapping/final Reviewer PASS, zero issues/required followups; clean5k/300s on this allocation, no new-change commit/push.**

## Authority and question

After the separate six-path YT-167 strict/builtin commit without Push, the user
authorizes contract-first, minimal isolated validation at **5000arrivals/s for
300s**, preserving the existing workload/response checks, accepted verification
and failed results. Delete all test resources and obtain independent Reviewer
PASS before reporting. New YT-168 changes must not be committed or pushed.

[YT-167](YT-167-PRIVATE-HTTP-SUSTAINED-CAPACITY.md) retains the failed6k/300s
observation. [YT-166](YT-166-PRIVATE-HTTP-CAPACITY-REFINEMENT.md) retains the
clean5k/10s observation on a different allocation. Neither is overwritten or
replayed. This new exposure asks whether the unchanged app profile has usable
headroom at5k for five minutes. A clean result would cover this finite profile
and target, not hours-long stability, production SLOs or universal capacity.
Different allocations prevent a controlled rate-only or duration-only causal
comparison with earlier results.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Reuse the exact accepted app
SHA25647d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65
and accepted k6 without rebuilding. Keep medium128rows/31063bytes, the full
literal body/header/CSP oracle on every response, HTTP1.1 identity encoding,
connection reuse,8s request/graceful timeout and256fixed VUs. No Core/runtime/
API/output/dependency/default listener change follows.

## Once-only execution and resource bounds

Reuse YT-167's accepted execution/capture/failure/cleanup seams. Change only
task ownership/network namespace and the new fixed5k plan labels/rate; do not
increase any admitted resource, capture, diagnostic or timing bound.

| Field | Selected bound |
| --- | --- |
| Ownership | One new task-tagged group/nonce and allocation generation; East Asia zone1/pinned Ubuntu24.04 image, no size/region fallback, load retry or existing-service modification |
| Resource window |60min from first creation; stop new work by45min, reserve15min for exact-owned cleanup |
| Guests | Same app Standard_D2s_v5 and separate load Standard_D16s_v5; actual2/16core admission, standard SSD OS disks only, no database/Valkey/data disk |
| App | Same2workers,CPU0–1/quota200%,256MiB/128tasks/512FDs/Nice10, no restart; one original PID,360s maximum lifetime/5s stop |
| Offered workload | New125/s2s admission warmup, then5000/s300s once on that app; stop app/observer before offline raw reconciliation; no extra rate/recovery phase |
| Generator | Same16CPU quota/256VUs/32GiB memory/512tasks/8192FDs,16GiB file/raw tmpfs cap,600s hard unit/570s internal deadline |
| Observation | Same50ms read-only process/thread/guest/cgroup records,420s/8400record ceiling and360s app-observer unit; retain cost/gaps/errors |
| Capture | Full native request JSONL in owned tmpfs,16GiB total raw/2GiB new local evidence; existing redacted streaming ZIP64, verified canonical ZIPs without persistent raw extraction |
| Transfer | Storage-first authenticated Blob GET/1KiB roundtrip before compute; protected Run Command/private Blob,840s guest command and bounded result upload/download |
| Data path | Private direct TCP39262; client-only22/39262 then deny other inbound, authenticated SSH control only, no HTTP tunnel/public benchmark listener |

Admit actual guest limits, tmpfs free space, transferred hashes and original
PID before new load. Extend only the existing caller's fixed plan with new
admission/sustained labels. Removing those two entries must recover the exact
committed YT-167 caller; the oracle, old entries, request/timing/options and
validation remain unchanged. Inspect only the two new plans without HTTP.
Reuse the unchanged selector/private-endpoint guards' accepted rejection
evidence; no old invalid-selector, build, endpoint, browser/CSP/provider or
workload qualification is rerun.

Keep the unchanged every4096th sparse formula and explicit server_probe_id
joins distinct from request IDs. Nominal1500000iterations emit367measured
records, largest ID13760, plus one phase-distinct warmup record, below the
unchanged ID20000 and1024record server bounds. One measured phase avoids
overlapping diagnostic blocks; do not enlarge capacity or add phases.

## Outcome and failure semantics

Retain the full once-only exposure even if scheduling/latency pressure appears.
Use the same descriptive marker: any unstarted drop, HTTP error, invalid/
incomplete response population or native overall p95 at least50ms. This is a
study marker, not a production SLO. No replay, resizing or favorable-result
selection follows pressure. A200 body/header/CSP mismatch aborts through the
unchanged caller. Native, PID/restart, observer/capture/resource or deadline
failure hard-stops on detection and enters cleanup. Warmup admission failure
blocks the measured point. Never claim a partial exposure as sustained success.

Report half-open client-clock window starts/completions, late starts, unstarted
drops, graceful tails and in-window valid RPS independently of full-invocation
counts. Keep native interpolated summaries and raw nearest-rank completed-
response quantiles separate; drops never receive a response latency. Report
ten successive30s windows and first/last60s latency populations. Retain app/
client CPU/memory/queue context, finite observer costs and missing observations.
Generator memory includes cumulative tmpfs capture and is not app memory growth.
Do not infer a leak from a finite increase, align unrelated VM clocks or claim
pure Core causation from profile saturation.

## Cleanup and acceptance

Bind accepted sources, artifacts and prior failed results before the only
allocation. The entire new control/guest/result path must fit the original
60min window; no deadline reset, duplicate uncertain execution or replacement
allocation. Preserve failures. One smallest canonical harness correction per
distinct seam cannot replay completed load; repeated infrastructure signatures
stop. No new benchmark/validator framework or unrelated qualification matrix.

On every outcome, stop/reap app, observers, client and control activity; retrieve
available failure evidence; delete the exact task-owned group; verify group
absence/zero owned resources; remove temporary credentials and reap the local
session. Capture failure cannot waive resource cleanup. Preserve full raw
results in the bounded canonical artifacts.

Independent contract review precedes adaptation, concrete mapping review
precedes provisioning and final independent results/failure-retention/cleanup
review precedes Done. Done means accepted completed bounded research; failed
load remains failed. Required repository paths are this report, three docs
indexes, the existing capacity caller and sample README. Run documentation/
diff checks for new work. No Core optimization, consumer hardware/SLO gate,
new-change Commit or Push is authorized.

## Concrete mapping before provisioning

Independent contract Reviewer /root/yt168_sustained5000_contract_review_20261024
returns **PASS, zero issues/required followups** before adaptation. Private
evidence is in `target/yt168-private-sustained-5000-20261024/evidence`. Snapshot
the committed YT-167 caller with its commit/blob/SHA binding. Removing exactly
two added plan entries recovers that entire accepted file, including all old
plans, guards, oracle, timing, validation, options and sparse sampling. Both
new plans inspect successfully without HTTP; accepted guard-rejection and
qualification evidence is reused without replay.

Six canonical task-local execution seams change only the task identity/private
network namespace and fixed5k plan labels/rate. Focused syntax checks pass.
Eleven concrete inputs and85retained source/history/artifact bindings protect
the accepted implementation and failed6k raw evidence. App/observer readiness,
original-PID/resource markers, successful nonempty observer stop receipts,
raw reconciliation after normal app stop and exact-owned cleanup remain.
The existing redacted streaming ZIP64,16GiB tmpfs/32GiB client memory/2GiB
local evidence checks and360s/600s/840s unit/guest bounds remain unchanged.
Independent mapping Reviewer /root/yt168_sustained5000_mapping_review_20261024
returns **PASS, zero issues/required followups** before the only allocation.

## Actual once-only sustained exposure

One declared storage-first allocation passes authenticated Blob GET/1KiB
roundtrip, actual guest core/resource/tmpfs admission and transferred hashes.
One original app PID serves the warmup and full300s exposure without restart.
Both native invocations finish; raw reconciliation matches their populations.
No prior short point,6k long point, recovery or qualification is replayed.

### Clean observation at the selected finite scope

| Population | Valid200 completions | Unstarted drops | Valid in-window responses/s | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms |
| --- | --- | --- | --- | --- | --- | --- |
|5000/s for300s |1500001 |0 |4999.983333 |0.456975 |0.893961 |1.615417 |

Admission adds250valid200 with zero drops. Every started iteration completes
and passes the unchanged full body/header/CSP oracle. HTTP errors, response
mismatches, interrupted iterations and scheduling drops are zero. Native
overall p95 is0.893961ms, below the inherited descriptive50ms pressure marker.
This is a **clean5k/300s observation on this allocation**, not a universal or
production capacity guarantee. Preserve every tail and the old failed6k data.

The table uses nearest-rank raw completed-response quantiles; native
interpolated summaries remain separately retained, even where values coincide.
There are1500001starts against nominal1500000arrivals. Three starts occur
at/after the scheduled end, including late boundary execution; do not count
them as three extra nominal slots. The half-open300s client-clock window
contains1499995valid completions, with six later completions and0.960032ms
observed HTTP drain. Dividing all completions by300would instead give
5000.003333and hide that tail. Full-call1ms-quantized p50/p95/p99 are1/1/2ms,
a distinct population. Maximum HTTP/full-call latency is32.735919/33ms;
submillisecond measurements and quantized zeros are not zero-cost calls.

### Segment and beginning/end observations

| Client completion window s | Valid completions | Drops | Valid responses/s | HTTP p95 ms |
| --- | --- | --- | --- | --- |
|0–30 |149996 |0 |4999.87 |0.962651 |
|30–60 |150000 |0 |5000.00 |0.899940 |
|60–90 |149999 |0 |4999.97 |0.860269 |
|90–120 |149997 |0 |4999.90 |0.891004 |
|120–150 |150003 |0 |5000.10 |0.784259 |
|150–180 |150000 |0 |5000.00 |1.384424 |
|180–210 |150000 |0 |5000.00 |0.794458 |
|210–240 |150001 |0 |5000.03 |0.841446 |
|240–270 |150000 |0 |5000.00 |0.767480 |
|270–300 |149999 |0 |4999.97 |0.791016 |

All ten windows have zero invalid responses/drops and p95 below1.385ms.
Completions crossing internal boundaries explain slight per-window variation
around5000responses/s; it is not extra offered load or a new capacity limit.
The first/last60s completed populations contain299996/299999responses, with
p95 0.932337/0.777847ms. No beginning-to-end latency degradation appears in
these finite populations. The150–180s interval has the largest p95 and remains
in the report; neither a global average nor a favorable subset replaces it.

### Resource and memory context

The app resource envelope averages133.09% of one CPU,66.55% of its two-CPU
quota, including native startup/control/drain. Server-side before-marker50ms
intervals have median133.53% and p95 141.89%, with four of6042intervals at
least180%. Waiting median/p95 are0.412248/0.804523ms. These observations show
headroom within the selected exposure; they do not justify linear extrapolation
to a higher clean rate, especially across VM allocations. Sparse middleware
probes omit accept/runtime queues and transmission; there is no pure Core or
unique physical-frequency/VM/kernel attribution.

App charged cgroup peak is10.92MiB against256MiB. Within the server's own
marked observation population, first/last60s mean charged memory is
10.19/10.30MiB, with maximum sampled memory approximately10.70MiB. These are
finite cgroup observations, not heap-only data or proof of leak freedom;
their clock bounds differ from client completion windows. App quota-throttling
and memory max/OOM events are zero.

The measured client envelope averages380.08% of one CPU,23.76% of its16-CPU
quota; the readable k6 process mean is376.52%. Its measured cumulative charged
peak is9.21GiB against32GiB, including tmpfs capture. Client throttling and
memory max/OOM events are zero. Logical raw capture is9.38GB, below16GiB.
No declared generator/capture cap is reached; this does not prove every
generator/GC/network/diagnostic cost is absent or establish higher-rate capacity.

Three finite observer populations retain12184records, zero skipped nominal
slots and zero process-read errors. Largest observed gap is54.643767ms. The
app observer costs1.83% of one CPU and measured client observer3.57%; neither
is free. The unchanged server diagnostics retain368unique sparse records and
736valid same-thread clocks, zero invalid IDs/clock records/overflow. Explicit
server_probe_id joins remain separate from request IDs. Sampling/accounting
intervals are not continuous peak traces, and unrelated VM clocks are not
subtracted to create causal precision.

### Retention, termination and exact cleanup

Full native populations, raw JSONL, summaries, marker/observer/termination
receipts, tails and all prior failed results remain. Canonical verified ZIPs
retain9379390268logical raw request bytes, without persistent extraction or
whole-file guest buffering. New local evidence is approximately342MB against
2GiB. All three bounded offline calculations run only after cloud removal,
exit0 and reap their processes; raw calculation takes161.64s.

Normal app stop confirms original-PID absence; app/observer units, client
observers and control activity close. The whole client batch, including
post-stop reconciliation, finishes in399.819868s against600s; its extra work
is outside the300s load and app lifetime. The exact group and all16inventoried
test resources are deleted.
Group absence, zero owned resources, removed temporary credentials and unchanged
accepted bindings are verified. Resource lifetime is938.036623s against3600s;
native controller elapsed is940.071829s, exit0, and the local session is
waited/reaped. Existing services and Core/runtime/API/output semantics remain
unchanged. No new changes are staged, committed or pushed.

## Conclusion and remaining limits

This allocation sustains the selected5k/300s profile with zero drops/errors,
approximately5000valid in-window responses/s,0.894ms HTTP p95 and observed app
CPU headroom. It supplies a bounded five-minute clean reference. It is not
an hours-long soak, a consumer/business workload qualification or a production
SLO. The old5k short pass and6k long failure remain unchanged. Different
allocations prevent a precise same-target5k–6k capacity bracket, controlled
rate-only/duration-only causation or a universal ceiling.

Any longer soak, actual-consumer workload or further upper-bound question
requires a separately activated contract; no new allocation, rate, optimization
or backlog item is activated here. Independent final Reviewer
/root/yt168_sustained5000_results_final_review_20261024 returns **PASS, zero
issues/required followups**. Exact reviewed copies and source/evidence bindings
remain; only status/review-reference lines change after acceptance. New changes
remain unstaged and uncommitted, with no Push.
