# YT-159 — HTTP worker construction wall and thread CPU

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded attribution scope; independent final Reviewer PASS.**

## Authority and bounded question

Evidence: `target/yt159-http-thread-cpu-rw8q72tn/evidence`.

The user asks to continue the proposed actual HTTP worker construction wall /
thread CPU study after [YT-158](YT-158-RENDER-HISTORY-CONTEXT.md). Preserve
API/output/security, accepted evidence and failures; contract first, minimum new
measurement, no optimization and independent final Reviewer before Done. No
commit/push is authorized. Snapshot the eleven existing uncommitted paths first;
do not change their accepted historical reports, sources or data in task artifacts.

[YT-157](YT-157-MEDIUM-HTTP-TAIL-ATTRIBUTION.md) observed higher current construction
wall clocks on actual workers, while YT-158's synchronous single-thread experiment
observed slightly lower current construction CPU and small render-history effects.
Neither identifies the original HTTP p95 cause. This item asks whether a new
actual-worker construction difference is CPU-visible or mostly outside the
calling-thread CPU interval. A bounded unresolved outcome is valid; do not force
unique allocator/scheduler/compiler/transport causality or a favorable replay.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Product libraries, dependencies,
locks, public APIs, browser/default graphs, normal fixture/handler/oracle/headers,
escaping/URL validation, document reservation and body release remain unchanged.
No runtime optimization, allocator swap/tuning, telemetry package, profiler
installation, host/governor change, new transport or reusable benchmark framework.

## Minimum feature-gated sample observer

Only the standalone HTTP sample gains default-off
`ssr-thread-cpu-probe = ["ssr-stage-probe"]`. Extend private stage_probe.rs through
cfg-only fields/branches and private `src/bin/ssr/thread_cpu.rs`; the unextended
YT-157 feature retains its original observer behavior and caller. Existing
ssr-context-study incompatibility still applies. This new mode requires Linux
x86_64 GNU; give a clear unsupported diagnostic elsewhere, with no portability
claim or new dependency. No change to src/bin/ssr.rs is needed.

Reuse the exact caller ssr-stage-probe.js, private X-Yt157-Id protocol and modulo32
selection, bounded1,024 records and post-drain dump. New diagnostic populations
keep their own task paths/features/binary identities despite reusing metric names.
Do not relabel them as accepted YT-157 requests. Unselected requests execute no
new CPU clocks, thread/CPU queries or empty probes.

For each selected request, measure only the **synchronous** existing page(rows)
plus Page::with_metadata block, with no await between clocks. Outer Instant wall
start precedes pthread_self/sched_getcpu and thread CPU start; after construction,
read thread CPU end and thread/CPU identity, then outer wall end. Keep the actual
Page alive until all clocks stop, return actual Page unchanged and preserve native
IntoResponse/lifetimes. Never measure thread CPU across middleware Next::run.await;
that pipeline wall clock remains its separate original boundary.

Record wall_ns, thread_cpu_ns, signed wall_minus_cpu_ns, opaque pthread identity
at both ends, observed logical CPU at both ends and a finite status. Clock return
failure/reversal, invalid identities/CPU or different start/end pthread identity
is invalid evidence, not a zero-valued measurement. Preserve invalid records and
stop further live phases after drain/cleanup. Thread identity is local correlation,
not a kernel TID or public interface. Observed logical CPU changes are retained;
same thread can migrate between the two allowed CPUs without invalidating its
thread CPU clock. Equal endpoints cannot prove there was no intermediate migration.

Immediately after the construction clocks, while Page remains held and before
observer locking, collect one empty bracket with the same clock/query sequence.
This is per-selected-request overhead context, outside construction but inside
pipeline/HTTP. Keep it separate; do not subtract it automatically, clamp signed
gaps or exclude outliers. Both versions use identical observer code. These clocks,
queries and record writes can change scheduling/cache/frequency and the HTTP tail.
Resolution is recorded separately from accuracy. Private C-ABI layouts/constants
must match installed headers. Primary semantics:
[thread CPU clocks](https://man7.org/linux/man-pages/man3/clock_gettime.3.html),
[pthread identity](https://man7.org/linux/man-pages/man3/pthread_self.3.html) and
[logical CPU query](https://man7.org/linux/man-pages/man3/sched_getcpu.3.html).

## Source identity and fixed new population

Copy accepted YT-158 reference/current source directories, verify their31 product
bindings/links and overlay identical final sample bytes. Actual product commits
remain90f83867 /0580a0a; current products match HEAD25a0cc6. Only accepted HTML /
document changes and Core test-only code differ; do not resurrect rejected child
capacity changes. Originals stay untouched; no checkout/reset or detached worktree.

Two new offline/locked default-release builds select only ssr-thread-cpu-probe
(which enables ssr-stage-probe), two jobs, unchanged Rust/Cargo1.99 and System
allocator/profile. Record features, source/lock/library artifacts and executables.
One new filtered test verifies actual medium route, exact independent31,063-byte
body, content/security headers, selected same-thread construction/empty records
and unselected no-record behavior. Exclude the accepted old test by cfg, compile
with --no-run JSON and execute emitted yt159_ --test-threads=1. One focused
rustfmt check on changed Rust. Do not replay old tests or build old features.

Reuse the exact retained k6 2.3.0 executable and acquisition/version proof:
SHA256 e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260.
No download, version execution or script change. Exactly four **new-feature**
phases: reference1, current1, current2, reference2. All use medium128 rows,
1,000 offered req/s for ten scheduled seconds,64 fixed VUs,8s timeout/graceful
stop; each fresh app gets one medium preload, two-second125/s warmup, drain and
one-second idle. Nominal40,000 measured plus1,000 warmup arrivals; retain actual
counts/boundary extras. No small/large/250/s, uninstrumented control or repeat.

Reuse YT-157's finite native lifecycle/monitor and direct component arithmetic.
Server: two Tokio workers, CPUAffinity12–13/CPUQuota200%/MemoryMax256MiB/
TasksMax128/NOFILE512/Nice10/no restart/RuntimeMax45s/native stop≤5s. Client and
monitor: CPUs14–19/600%/1GiB/TasksMax128/NOFILE512/Nice10/no restart/RuntimeMax600s.
The host delegates cpu/memory/pids, not cpuset; use native CPUAffinity, preserve
observed affinity/thread/source context and ancestor limits. No existing service
or host policy changes, external resources, global caches or allocator tuning.
Only completed task-owned raw-file cache advice is allowed, as in accepted evidence.

Before implementation set one fixed45-minute UTC/monotonic deadline, never
renewed. Each build/test≤600s; runtime/analysis/cleanup≤600s within it. Require
task-owned ext4 with≥8GiB free and≤128MiB/file /512MiB total new evidence.
Keep accepted raw data outside this new cap and untouched. No new runner, validator,
proof framework, startup retry matrix or new budget after a fault. Preserve
failures, classify materiality and allow only one smallest correction per distinct
failed seam; same consecutive signature stops. Never replay passed work or loads
to obtain a better result.

## Joins, inference and acceptance

Use existing native timing/status/body validity and unique phase/request IDs.
All native/custom HTTP components and IDs reconcile; modulo32 server samples
must match expected IDs exactly without missing/extra/duplicates/overflow. Each
construction_ns equals its new outer wall_ns; CPU/gap/status/identity fields are
validated. Warmup and measured remain separate. No invalid clock record may seed
a source ratio. Record observed threads/CPUs, every selected joint HTTP/pipeline /
construction/CPU/empty observation, all one-second bins and HTTP-tail/non-tail
cohorts; retain sparse deterministic tail counts rather than implying random IID.

Report nearest-rank p50/p95/p99/extrema for complete HTTP/native components and
selected construction wall/CPU/signed-gap/empty distributions, both repetitions,
per-worker partitions and source ratios when comparable. Outer-minus-inner gap
includes observer work plus possible descheduling; it is not pure off-CPU time.
CPU-visible changes can still arise from frequency/cache/allocator/compiler /
worker context and are not instruction counts or fixed algorithm causality.
Never subtract marginal percentiles, cross-process clocks or independent old/new
populations to assign the original HTTP p95 budget. Preserve original mixed
results and explicit unresolved causes; no significance/capacity/SLO/production
or unique historical-cause claim.

Retain actual nominal/started/completed/valid200/drops/interruptions/status/errors,
window/drain throughput, boundary starts and all-error versus valid200 timings if
errors arise. Reuse measured comparability: exit0/exact output/no errors/drops /
interruption,≥95% nominal valid window completions, consistent joins,≥2 inner
resource samples/gap≤2s, client peak intervalCPU<480% and cumulative memory
<819.2MiB, zero observed throttle/memory/task/FD-limit events. Missing body/source /
clock/observer/resource required evidence stops further execution; ordinary
timing/comparability failures remain in the fixed sequence without tuning/replay.
Coarse resource counters omit boundary slivers. Cleared final ControlGroup means
endpoint unavailable, never root cgroup or relabeled last sample.

Stop/reap only owned apps/client/monitor; preserve native exits/stage dumps and
verify units inactive/MainPID0, original PIDs/cgroups absent and listener closed.
Retain inert artifacts. Repository docs/whitespace and independent final Reviewer
PASS0 precede Done at bounded research scope, not unique-cause resolution.
After PASS only status/reviewer references in five documents and docs/whitespace
checks may change. Eight task paths: this report, docs/BACKLOG.md, docs/ROADMAP.md,
docs/README.md, sample Cargo.toml, README.md, src/bin/ssr/stage_probe.rs and new
src/bin/ssr/thread_cpu.rs. Combined with prior work: thirteen uncommitted paths.
No further task/optimization/commit/push, YT-096 decision or release-scope change.

## Results and bounded attribution

Independent contract Reviewer `/root/yt159_http_worker_cpu_contract_review_20261024`
returned PASS, zero issues/required followups, before code. Final independent
Reviewer `/root/yt159_http_worker_cpu_final_review_20261024` returned PASS,
zero issues/required followups after inspecting actual targets, bindings, raw joins /
statistics, verification and cleanup. YT-159 is Done at bounded attribution scope;
the original HTTP-tail cause remains unresolved. Accepted YT-157/158 results and
failures remain separate and unchanged.

### Populations, same-thread validity and output

All four new-feature phases ran exactly once in the planned order. Native
results retain **40,002 measured +1,003 warmup valid200** responses, zero errors,
drops, interruptions or invalid200s. All eight k6 exits and four native app
stops are0; four preload bodies/headers match. Both measured pairs satisfy the
reviewed comparability conditions; all warmups are valid priming operations,
with no resource-comparability flag in this population.

All **1,284** expected modulo32 records correlate uniquely:313 measured and8
warmup per app,1,252 measured/32 warmup total. All **2,568** construction/empty
clock measurements have valid positive thread CPU, equal live pthread identities
at both ends, valid CPU12/13 endpoints and exact signed wall-minus-CPU arithmetic.
Construction wall matches construction_ns and stays nested inside pipeline wall.
There is no missing/duplicate/extra sample, invalid clock/ID, overflow, negative
gap or construction-to-empty thread change. No CPU clock crosses an await.
Reported clock resolution is1ns, without an accuracy claim.

Each app samples two process-local worker identities. Each identity is observed
on both logical CPUs across requests; no construction shows differing CPU
endpoints. This does not exclude intermediate migration or descheduling. Private
identities never enter responses; source review preserves actual Page ownership,
IntoResponse, body release and exact content/CSP/header behavior.

### Complete new HTTP populations

HTTP timings below are milliseconds; CPU/RSS use the declared inner resource
windows. These new instrumented populations do not replace historical YT-156/157
data or reproduce its percentage under unchanged observation.

| Execution order | n | HTTP p50 / p95 / p99 ms | App mean CPU % | Sampled app RSS MiB |
| --- | --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 10,000 | 0.587393 / 1.303093 / 2.718116 | 49.13 | 6.82 |
| 02-r1-medium-1000-current | 10,001 | 0.596543 / 1.383613 / 3.124457 | 43.99 | 6.75 |
| 03-r2-medium-1000-current | 10,000 | 0.616309 / 1.454924 / 3.107392 | 48.68 | 6.71 |
| 04-r2-medium-1000-reference | 10,001 | 0.575416 / 1.298417 / 2.856944 | 48.06 | 6.82 |

Current/reference HTTP p95 increases **6.18% /12.05%**. Sampled pipeline p95
increases16.96% /19.92%, while pipeline p50 is lower6.75% /2.03%. Mean app CPU
is lower in run1 and slightly higher in run2; no universal CPU reduction or
throughput/capacity speedup follows. Native same-response sending+waiting+
receiving reconciles with HTTP duration to maximum error8.89e-16ms, below1e-6ms.
All native/custom timings agree per request ID. Full-call integer-ms timings,
all native components, error/status/count/window data and every tail remain
retained. Marginal percentiles are never added to apportion HTTP cost.

### Synchronous construction CPU tracks wall

Each row has313 deterministic measured samples. Values are microseconds. CPU
clocks enclose construction plus their bounded observation overhead; the signed
outer-minus-inner gap includes observer work and possible descheduling.

| Execution order | Construction wall p50 / p95 / p99 | Thread CPU p50 / p95 / p99 | Gap p50 / p95 / max |
| --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 115.274 / 173.717 / 290.083 | 113.444 / 170.811 / 284.200 | 2.215 / 3.599 / 6.277 |
| 02-r1-medium-1000-current | 147.871 / 267.453 / 305.888 | 145.012 / 262.320 / 299.583 | 2.706 / 5.264 / 19.665 |
| 03-r2-medium-1000-current | 154.179 / 261.527 / 296.805 | 151.387 / 256.839 / 290.568 | 2.838 / 5.247 / 14.805 |
| 04-r2-medium-1000-reference | 112.588 / 166.474 / 224.230 | 110.666 / 163.232 / 219.534 | 2.102 / 3.315 / 8.696 |

Current/reference construction CPU p50 increases **27.83% /36.80%**, close to
wall p50 increases28.28% /36.94%; CPU p95 increases53.57% /57.35%, close to wall
p95 increases53.96% /57.10%. No marginal-percentile subtraction is used. The
small central within-request gap compared with the complete construction
interval supports **CPU-visible construction timing differences in this new
actual-worker diagnostic**, rather than explaining that new construction
difference mainly as outside-thread waiting. This does not prove extra
instructions, an intrinsic construction algorithm regression, or the cause/share
of the historical full HTTP p95.

### Empty work, worker partitions and temporal context

Empty brackets execute the same clock/identity/query sequence after construction
while Page stays held. They are separate overhead observations inside pipeline /
HTTP, not values subtracted from construction. Values below are microseconds.

| Execution order | Empty wall p50 / p95 | Empty CPU p50 / p95 | Empty gap p50 / p95 |
| --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 1.152 / 1.736 | 0.548 / 0.838 | 0.605 / 0.923 |
| 02-r1-medium-1000-current | 1.513 / 2.925 | 0.705 / 1.334 | 0.806 / 1.546 |
| 03-r2-medium-1000-current | 1.569 / 2.682 | 0.729 / 1.239 | 0.833 / 1.464 |
| 04-r2-medium-1000-reference | 1.139 / 1.661 | 0.548 / 0.782 | 0.592 / 0.891 |

Even this fixed empty work has current/reference CPU p50 increases **28.65% /
33.03%**, and wall p50 increases31.34% /37.75%. That makes shared execution /
clock/cache/frequency context a concrete next question. It does **not** measure
frequency or establish it as the cause: observer/caller placement, system-call
cost, compiler effects, allocator history and worker activity also remain. The
record storage is larger in this new feature, changing sampled allocation context.
No hardware counter, cycle/instruction count or actual frequency trace was taken;
the recorded powersave governor is a policy label, not an observed clock rate.

Within each app, label its two opaque worker IDs A/B in ascending local-ID order;
labels have no cross-process identity. CPU medians from both workers and one-second
bin ranges preserve context variation instead of selecting a favorable worker.

| Execution order | Worker A/B n | Worker A/B construction CPU p50 µs | Sampled HTTP-tail n | Ten-bin construction CPU p50 range µs |
| --- | --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 180 / 133 | 112.720 / 113.904 | 19 | 110.444–116.760 |
| 02-r1-medium-1000-current | 191 / 122 | 145.529 / 142.054 | 14 | 58.607–148.392 |
| 03-r2-medium-1000-current | 156 / 157 | 152.345 / 150.915 | 15 | 140.002–156.731 |
| 04-r2-medium-1000-reference | 168 / 145 | 110.746 / 110.559 | 15 | 109.420–111.363 |

The current1 low bin remains in its aggregate; the process is not declared
stationary. Sampled HTTP-tail cohorts contain only14–19 requests and3–11 per
worker, so no random-IID/precision or unique-tail inference follows. All ten
bins, warmups, boundary populations, per-worker CPU endpoint counts, tail /
non-tail distributions and selected joint observations remain in
`thread-component-summary.json` and native raw records.

The largest measured construction gap is current1 m:6432: wall304.114µs,
CPU284.449µs, gap19.665µs within that same sample; it is not an HTTP-tail request.
The current2 selected tail retains m:3488 wall201.366µs, CPU186.561µs, gap14.805µs.
Full HTTP maxima5.626329/10.789700/6.184715/8.189467ms in execution order remain
even when not selected by modulo32. Do not subtract client/server or independent
percentiles to invent an exact transport, scheduler or construction HTTP budget.

### Decision and next possible question

Retain current products and all unfavorable HTTP results. The new same-thread
probe narrows the new construction difference to CPU-visible execution context;
it does not identify a safe code optimization or resolve the **original HTTP
p95 cause**. CPU time measures time running, not fixed work at a fixed frequency.
YT-158 remains accepted in its distinct single-thread/no-HTTP context; neither
study invalidates the other or substitutes a favorable population.

If separately requested, prioritize read-only **actual CPU frequency / worker
active-idle context** before another allocator or serializer change. The empty
clock and construction distributions move together, making that a more focused
next discriminator than assuming extra construction instructions. Any such
measurement needs its own bounded contract and observer limits; no governor /
frequency tuning, new workload or next item is activated here.

### Verification, bounds, failures and cleanup

New verification passes without a failed invocation: focused rustfmt on the two
changed Rust files; one offline/locked test compilation with --no-run JSON, then
emitted `yt159_ --test-threads=1`: **1 passed,0 failed/ignored/filtered**; two
offline/locked release builds of ssr-thread-cpu-probe, two jobs, no warnings.
Features are default/ssr-stage-probe/ssr-thread-cpu-probe; actual Core/HTML/adapter
artifacts and binaries match31 product bindings and14 identical sample overlays
per copy. Dependencies/locks and default/browser/product paths are unchanged.
No old test/build/load, security/browser/platform/provider qualification or tool
version/download was rerun. Initial documentation/whitespace checks pass; report
updates receive required checks before final review.

Runtime is56.678s; arithmetic and owned cleanup complete
142.489s after its start, within600s and the
original unrenewed45-minute stage deadline. Raw output totals **225,020,511
bytes**, max file **55,054,188 bytes**, below512MiB/128MiB.

Measured window valid completions are39,998; three starts
fall outside the half-open schedule and four completions occur at/after its end.
Nominal-minus-started-minus-dropped is−2, not normalized to zero. Started/completed/
valid counts, each boundary timestamp and scheduled/drain throughput stay distinct;
warmups have251/251/251/250 valid responses against1,000 nominal total.

Client peak interval CPU is155.42% (<480%); cumulative
charged memory peak is114.40MiB (<819.2MiB).
Inner measured windows span8.219–8.238s, max sampling gap1.034s.
Observed client full-invocation and app inner throttle / memory-limit events
are zero, with task/FD headroom. Sampling omits boundary slivers and possible
short peaks; these counters do not prove absence of scheduling/frequency effects.

Cleanup verifies **five inactive owned units,13 absent workload PIDs, absent
original cgroup paths and closed listener**. Four native stopped events, client
exit0 and waited root-control exit0 are retained. Cleared final ControlGroup is
explicitly unavailable; no root cgroup or last sampled counter is called a final
owned endpoint. Inert sources/builds/raw artifacts remain; no existing service,
host policy or external resource changed. All139 frozen runtime inputs matched
at phase completion. Only root report/index/sample-README results change afterward.

There are no new failed test/build/runtime/arithmetic invocations. One report-
preparation SyntaxError occurred in a nested escaped f-string before any report
write. The before-script and exit1 record are retained in
report-preparation-failure.json; one smallest separate-value formatting correction
completed the report. No source, clock, arithmetic, test/build/load or accepted
proof was replayed or changed.
Accepted YT-157/158 denominator/assertion/preparation failures remain in their
original evidence, with read-only bindings. No failure record or historical
review is overwritten. Eight new-scope paths overlap six prior paths; the worktree
now has thirteen combined uncommitted paths, staged0. No optimization/commit/push,
YT-096 Pending decision, YT-020 Blocked status or immutable release-scope change.
