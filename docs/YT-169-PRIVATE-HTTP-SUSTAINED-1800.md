# YT-169 — Sustained private HTTP capacity for thirty minutes

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded profile-observation scope; independent contract/mapping/final Reviewer PASS, zero issues/required followups; clean5k/1800s on this allocation; new changes uncommitted/unpushed.**

## Authority and question

After the separately authorized six-path YT-168 strict/builtin commit without
Push, the user selects one isolated **5000 arrivals/s for 1800 seconds** study.
Establish the contract and preservation budget first. Keep the existing workload
and complete response checks, reuse accepted evidence without replay, preserve
failures, delete every test resource and obtain independent Reviewer PASS before
reporting. New YT-169 changes must not be committed or pushed.

[YT-168](YT-168-PRIVATE-HTTP-SUSTAINED-5000.md) retains its clean five-minute
observation; [YT-167](YT-167-PRIVATE-HTTP-SUSTAINED-CAPACITY.md) retains the
failed 6k/300s exposure. Neither is rerun or rewritten. This study asks whether
the same selected profile remains clean for thirty minutes on one new target.
It cannot isolate duration effects across different VM allocations, establish
a universal capacity ceiling or qualify a consumer SLO or hours-long soak.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Reuse the accepted app
SHA256 `47d794df83e0545372356b1b2487cb1925f7493796d47be4e8a565dc3f5b3f65`
and accepted k6 without rebuilding. Medium128rows/31063bytes, literal complete
body/header/CSP oracle on every response, HTTP1.1 identity encoding, connection
reuse,8s request/graceful timeout,256fixed VUs and2server workers are unchanged.
No Core/runtime/API/dependency/output/default-listener optimization is selected.

## Preservation and diagnostic budget

Scaling YT-168's9.38GB logical JSONL by six estimates about56.3GB. This is a
planning estimate, not an assurance of compression or execution success.
Use the accepted k6's existing [native gzip JSON output](https://grafana.com/docs/k6/latest/results-output/real-time/json/)
to retain every metric record losslessly while writing directly to `.jsonl.gz`.
Do not discard samples, replace raw records with aggregates or persist an
uncompressed duplicate. Gzip adds generator work; report its observed resource
cost and do not claim a controlled duration-only comparison with older captures.

| Budget | Selected bound and handling |
| --- | --- |
| Load guest | Same16CPU quota,32GiB memory,512tasks/8192FDs; same16GiB raw tmpfs; gzip file limit4GiB; admit free space before load |
| Logical records | Estimated56.3GB; offline decompression ceiling80GiB; complete gzip EOF/CRC, JSON and population reconciliation required for acceptance |
| Evidence |6GiB maximum new local evidence, including canonical ZIPs, inputs and small mirrors; check declared size/free space before download and actual size afterward |
| Transfer | Store gzip members once in ZIP64 without recompressing; retain partial bytes on failure; no persistent raw extraction; hash-verified private Blob transfer |
| Privacy | Gzip metrics are produced only by the fixed synthetic caller under a clean non-secret environment; they contain no supplied credentials. Only its two exact raw filenames may pass through as binary. Existing text/log redaction and secret-file exclusions remain |
| Analysis | After cloud cleanup, one bounded streaming pass per raw member using compact numeric arrays and bounded pending joins; no per-request dictionary retained for9million responses; no framework or replay |

The unchanged server diagnostic bounds are ID20000 and1024records. Sampling
every4096requests for9million arrivals would exceed both. Only the new measured
plan samples every16384requests:550nominal records with largest ID19616,
plus one phase-distinct warmup record. Old plans retain4096; the server binary,
record format and explicit server_probe_id join remain unchanged. Every response
still receives the complete correctness check. Report this diagnostic-density
change explicitly; do not use sparse records as a complete tail population.

## Once-only execution bounds

Adapt the existing YT-168 task-local execution seams only as required for the
new task identity, duration, gzip preservation and finite diagnostics. Keep the
same East Asia zone1/pinned Ubuntu24.04 image, app Standard_D2s_v5 and separate
load Standard_D16s_v5, actual2/16core admission, standard SSD OS disks only,
private direct TCP39262, authenticated SSH control and protected Run Command/
Blob transport. No database, Valkey, data disk, public HTTP listener or tunnel.

One new owner-tagged group/nonce and allocation generation has a90minute
absolute lifetime from first creation,75minute new-work deadline and15minute
cleanup reserve. No deadline reset, replacement allocation, fallback or load
retry. Storage-first authenticated GET and1KiB roundtrip precede compute.
Admit concrete hashes, guest quotas, free space and one original server PID.

App limits remain200%CPU/256MiB/128tasks/512FDs/Nice10/no restart, with1860s
app/observer unit ceilings. Offer one125/s2s admission warmup followed by the
single5000/s1800s exposure on that PID; warmup pressure blocks measurement.
Client unit/internal ceilings are1980s/1950s; guest command2400s. Same50ms
read-only observers have1920s/38400record ceilings. Record skipped slots,
read errors, CPU cost and finite coverage. Stop app and observers before result
transport. Move complete raw reconciliation to local analysis after cloud
removal; avoid an additional56GB guest parse inside the allocation window.

Only add two new fixed plan entries and the new measured sampling selection
to the existing caller. Removing those additions must recover the exact
committed YT-168 caller. Inspect only the new plans without HTTP. A single
no-HTTP synthetic iteration may verify the newly selected gzip format using
the accepted binary; this does not replay a passed workload qualification.
Reuse existing selector/private-endpoint guard, build, browser/CSP and provider
evidence without rerunning it.

## Failure semantics and reporting

Keep the descriptive pressure marker: any unstarted drop, HTTP error,
incomplete/invalid population or native overall p95 at least50ms. This is not
a production SLO. Retain the once-only exposure under ordinary scheduling/
latency pressure; never replay or resize for a favorable result. The unchanged
200-body/header/CSP mismatch aborts immediately. Native, PID/restart, observer,
capture, resource or deadline failures stop execution on detection and enter
cleanup. Partial exposure or an unreadable/truncated gzip is not sustained
success; preserve available failure bytes and distinguish execution from study
acceptance.

Report native summaries separately from raw nearest-rank quantiles. Reconcile
starts, completed/valid/invalid populations, unstarted drops, boundary starts,
half-open1800s completions/RPS and graceful tails. Drops have no response latency.
Report all sixty30s windows, first/last60s HTTP p95/p99, full-call timing,
app/client CPU and charged-memory trends, OOM/throttle events and bounded
server diagnostic samples. Gzip capture memory belongs to the load process
envelope; finite charged-memory growth does not prove an application leak.
Use only same-VM clocks for durations and retain observer gaps and missing data.

On every outcome stop/reap app, client, observers and control processes; retrieve
available evidence; delete the exact task-owned group; verify its absence and
zero owned resources; remove temporary credentials and reap the local session.
Capture failure never waives cleanup. Preserve old failures and bind accepted
sources/artifacts before provisioning. One smallest canonical harness correction
per distinct seam is allowed without replaying completed load; repeated
infrastructure failure signatures stop. No general validation infrastructure.

Independent contract review precedes adaptation, concrete mapping review precedes
the one allocation, and final results/failure-retention/cleanup Reviewer PASS
precedes Done. Done denotes completed bounded research even if load fails;
failed load remains failed. The report, three docs indexes, existing caller and
sample README are the six intended paths. Check new documentation and whitespace.
No new-change Commit, Push or further item is activated.


## Concrete mapping before provisioning

Independent contract Reviewer `/root/yt169_sustained1800_contract_review_20261025`
returns **PASS, zero issues/required followups** before adaptation. Task-local
inputs and evidence reside in `target/yt169-private-sustained-1800-20261025/evidence`.
The existing six canonical execution seams implement the declared duration,
lossless gzip capture, exact raw-filename passthrough and increased finite
observer/control deadlines. Existing text redaction and credential exclusions
remain. Complete raw reconciliation is deferred until cloud deletion.

Removing exactly the two new plan lines and new-only sampling selection
recovers the committed YT-168 caller byte-for-byte. New plan inspections pass
without HTTP, and one no-HTTP synthetic iteration verifies native gzip EOF/CRC
and JSON using the accepted k6 executable. The initial inspection invocation
omitted k6 inspect's explicit environment flags and failed before HTTP; its
original output is retained. One CLI-argument correction passes; no product
change, workload replay or allocation occurred for that failure.

Accepted sources, binary identities and historical failure artifacts are
hash-bound without rerunning their qualifications. Concrete independent mapping
Reviewer `/root/yt169_sustained1800_mapping_review_20261025` returns **PASS,
zero issues/required followups** before the only allocation. The six reviewed
source and eleven execution-input hashes match. The once-only execution, complete raw reconciliation and exact-owned cleanup
are now complete; final independent results review passes as recorded below.


## Actual once-only exposure and complete reconciliation

One declared storage-first allocation passes the authenticated1KiB Blob
roundtrip, actual core/quota/tmpfs admission and transferred hashes. Original
app PID1712 serves the251response admission and full1800s exposure without
restart. No prior workload, passing qualification or failed6k point is replayed.

| Population | Valid200 completions | Unstarted drops | Valid in-window responses/s | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms |
| --- | --- | --- | --- | --- | --- | --- |
|5000/s for1800s |9000001 |0 |4999.996667 |0.472333 |0.953755 |1.592884 |

All started iterations complete and pass the unchanged complete body/header/
CSP oracle. Native errors, mismatches, interrupted requests and scheduling drops
are zero. Overall native p95 equals the raw nearest-rank value here; their
populations remain separately retained. HTTP maximum is46.541509ms; full-call
1ms-quantized p50/p95/p99 are1/1/2ms, maximum49ms. This is a clean finite
profile observation, not a production latency guarantee.

There are9000001starts against9000000nominal arrivals. Four starts occur at/
after the scheduled end, including boundary execution, not four extra nominal
slots. The half-open1800s client-clock window contains8999997starts and
8999994valid completions; seven completions follow it. Observed HTTP drain is
1.233643ms. Dividing all completions by1800would give5000.000556 and conceal
that tail. Drops never receive a response latency.

### All sixty completion-clock intervals

Every30s window has zero drops and zero invalid responses. Retain the localized
150–180s tail: p95 1.916467ms and p99 13.668743ms, despite the clean overall
marker. First60s has299995valid completions, p95 1.074901ms/p99 1.873662ms;
last60s has299999, p95 0.894840ms/p99 1.478847ms. These populations show no
beginning-to-end deterioration; they do not prove every interval is identical,
stationarity, hours-long behavior or a causal explanation for the local tail.

| Window s | Valid completions | Drops | HTTP p95 ms | HTTP p99 ms |
| --- | --- | --- | --- | --- |
|0–30 |149995 |0 |1.111058 |3.043967 |
|30–60 |150000 |0 |1.038823 |1.616805 |
|60–90 |150000 |0 |1.029490 |1.656949 |
|90–120 |149999 |0 |1.032856 |1.719590 |
|120–150 |150000 |0 |1.002674 |1.577952 |
|150–180 |150001 |0 |1.916467 |13.668743 |
|180–210 |150000 |0 |1.006699 |1.589759 |
|210–240 |149999 |0 |0.978432 |1.562859 |
|240–270 |150001 |0 |1.002662 |1.694840 |
|270–300 |150000 |0 |0.946396 |1.454642 |
|300–330 |150000 |0 |0.976634 |1.546320 |
|330–360 |150000 |0 |0.960754 |1.517939 |
|360–390 |150000 |0 |0.994094 |1.648767 |
|390–420 |150000 |0 |0.999890 |1.688884 |
|420–450 |149999 |0 |0.976505 |1.645582 |
|450–480 |150000 |0 |0.914704 |1.454922 |
|480–510 |150001 |0 |0.917166 |1.479484 |
|510–540 |150000 |0 |0.939253 |1.470513 |
|540–570 |149999 |0 |0.923035 |1.486745 |
|570–600 |150001 |0 |0.908198 |1.451285 |
|600–630 |149999 |0 |0.927814 |1.505353 |
|630–660 |150000 |0 |0.940482 |1.643248 |
|660–690 |150001 |0 |0.928247 |1.535442 |
|690–720 |150000 |0 |0.933017 |1.504348 |
|720–750 |150000 |0 |0.945704 |1.574743 |
|750–780 |150000 |0 |0.958933 |1.642030 |
|780–810 |150000 |0 |0.966573 |1.748960 |
|810–840 |150000 |0 |0.870981 |1.408001 |
|840–870 |150000 |0 |0.907281 |1.505270 |
|870–900 |150000 |0 |0.896938 |1.450970 |
|900–930 |150000 |0 |0.928545 |1.564623 |
|930–960 |150000 |0 |0.878404 |1.406536 |
|960–990 |150000 |0 |0.913575 |1.543511 |
|990–1020 |150000 |0 |0.965990 |1.728731 |
|1020–1050 |150000 |0 |0.914055 |1.528533 |
|1050–1080 |149999 |0 |0.898272 |1.459527 |
|1080–1110 |150001 |0 |0.930229 |1.609939 |
|1110–1140 |150000 |0 |0.819257 |1.346310 |
|1140–1170 |150001 |0 |0.843215 |1.437358 |
|1170–1200 |149999 |0 |0.795762 |1.372973 |
|1200–1230 |150000 |0 |0.849023 |1.458692 |
|1230–1260 |150001 |0 |0.837333 |1.407550 |
|1260–1290 |149999 |0 |0.943132 |1.670966 |
|1290–1320 |150000 |0 |0.864789 |1.402576 |
|1320–1350 |150000 |0 |0.860011 |1.451545 |
|1350–1380 |150000 |0 |0.845118 |1.414101 |
|1380–1410 |150000 |0 |0.932916 |1.605287 |
|1410–1440 |150000 |0 |0.911926 |1.476378 |
|1440–1470 |149999 |0 |0.944787 |1.580705 |
|1470–1500 |150001 |0 |0.917235 |1.462419 |
|1500–1530 |150000 |0 |0.941695 |1.571369 |
|1530–1560 |150000 |0 |0.870924 |1.411306 |
|1560–1590 |150000 |0 |0.996793 |2.134177 |
|1590–1620 |150000 |0 |0.910379 |1.563703 |
|1620–1650 |150000 |0 |0.921317 |1.605079 |
|1650–1680 |150000 |0 |0.914658 |1.463861 |
|1680–1710 |150000 |0 |0.945078 |1.677206 |
|1710–1740 |150000 |0 |0.885433 |1.421987 |
|1740–1770 |149999 |0 |0.923879 |1.574404 |
|1770–1800 |150000 |0 |0.860568 |1.396504 |

### Resource and observation limits

The app before/after envelope averages144.72% of one CPU,72.36% of its200%
quota. App charged peak is11.14MiB against256MiB. Server own-clock marker
samples average10.23MiB in their first60s and10.36MiB in their last60s;
these are charged-memory observations, not exactly client-window-aligned heap
measurements or leak proof. App observed50ms CPU interval median/p95 are
145.34/150.68%;23of36146intervals reach180%. Finite interval counters are
not continuous peaks or evidence for linear extrapolation to a higher rate.

Client envelope mean349.57% of one CPU uses21.85% of its16CPU quota, including
native gzip and observer work; the readable k6-process mean is345.49%.
Client charged peak is3.66GiB against32GiB and includes accumulated compressed
raw capture. App/client cgroup throttle, memory-max and OOM event deltas are
zero. No declared capture/resource ceiling is reached. New allocation, native
gzip and changed diagnostic density prevent attributing differences from older
runs solely to duration, compression or Core.

Three finite observer populations retain72395records, zero skipped nominal
slots and one process-read error. App and measured-client observer CPU cost are
2.24/3.97% of one CPU; largest observed gap is70.201950ms. The sole
FileNotFoundError occurs in the penultimate measured-client record,117.808ms
before its native after-resource marker; the next record reads the same PID/
start identity successfully. The exact missing proc/thread file was not
recorded, so no finer cause is asserted. Preserve that missing observation;
nonempty capture, successful observer termination and all request records
remain accepted. This is a finite observation limit, not a reason to replay.

The unchanged server emits551unique sparse records and1102valid same-thread
clocks, with zero invalid IDs, invalid clocks or overflow. The new measured
16384interval yields550records; the warmup yields one phase-distinct record.
Explicit server_probe_id joins are separate from per-request IDs. Sparse
records do not explain every tail and clocks from separate VMs are not
subtracted to manufacture causal precision.

### Retention, native termination and cleanup

All55575006662logical raw bytes are retained losslessly as1777858912gzip
bytes inside the canonical verified ZIP64. No uncompressed duplicate persists.
Gzip EOF/CRC, every JSON record and complete started/completed/valid/drop
populations reconcile. The compact streaming analysis peaks at175pending
joins, completes in571.04s and peaks at1077008KiB process RSS. It starts
only after cloud deletion. All three offline calculation processes exit0 and
are waited/reaped; no HTTP request is generated by analysis.

The actual result ZIP is1798150584bytes, below the local6GiB budget; total
new local evidence is approximately1.75GiB. Native client batch finishes in
1814.077633s against1980s. App/observer/client stop receipts confirm original
PID absence, no restart and bounded termination. All16inventoried test
resources are deleted; group absence, zero owned resources, removed temporary
credentials and unchanged retained bindings are verified. Resource lifetime
is2561.163671s against5400s. Native controller exits0 after2564.390764s,
is waited/reaped and its PID is absent. No active test resources remain.

The initial no-HTTP inspect CLI failure and one finite observer read gap are
retained alongside all older failed6k/transport/preparation evidence. No failed
result is replaced, no old accepted verification is replayed, and no Core,
runtime, API, dependency, output or default listener changes occur.

## Acceptance boundary

This allocation supplies a clean5k/30minute medium-SSR reference with complete
response checks, bounded resource headroom and no end-to-end latency decline
in the stated first/last populations. Preserve the150–180s local tail and
finite observer gap. It is not an hours-long soak, maximum-capacity guarantee,
controlled comparison across allocations or Auth/Session/database consumer
qualification. No optimization or further item is activated.

New documentation checks pass75documents/169backlogitems, including internal
links, Markdown, whitespace and dependency DAG; tracked/new-report diff checks
pass. Independent final results/failure-retention/cleanup Reviewer
`/root/yt169_sustained1800_results_final_review_20261025` returns **PASS, zero issues/required
followups**. YT-169 is Done at this bounded profile-observation scope. All six
new paths remain unstaged and uncommitted, with no Push; YT-096/D1–D6 and
YT-020 are unchanged. Actual Auth/Session/database workload research is a
possible next question requiring separate activation, not a new item here.
