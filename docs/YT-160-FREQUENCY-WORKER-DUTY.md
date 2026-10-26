# YT-160 — Reported CPU frequency and worker activity context

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded research scope; independent final Reviewer PASS.**

## Authority and bounded question

Evidence: `target/yt160-frequency-duty-z1n_17fa/evidence`.

The user requests further research of the CPU-frequency / worker active-idle
question proposed after [YT-159](YT-159-HTTP-WORKER-THREAD-CPU.md). Continue with
contract-first minimum measurement and independent Reviewer before Done; preserve
accepted evidence, failures, API/output/security and all thirteen uncommitted
paths. No optimization, commit/push, host/governor/frequency/permission change,
package installation or existing-service change. Preserve
[ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md).

YT-159's sampled construction CPU p50 increases27.83% /36.80% current/reference,
while fixed empty-clock CPU work increases28.65% /33.03%. YT-158's single-thread
context remains distinct. This item asks whether new read-only reported frequency
and running/non-running context is consistent with those scaling observations.
It does not prove unique historical p95 causality, constant work or frequency-
normalized instruction counts. A bounded unresolved result is valid.

## Available surfaces and their limits

The host is Ubuntu kernel7.0.0-38-generic, intel_pstate active/powersave, EPP
balance_performance, with readable cpuinfo_avg_freq and scaling_cur_freq on
CPUs12/13. CONFIG_CPUFREQ_ARCH_CUR_FREQ is enabled. These are reported kernel
frequency values, not a direct timestamped per-request frequency trace.

[CPUFreq documentation](https://docs.kernel.org/admin-guide/pm/cpufreq.html)
describes hardware-feedback averages and warns that scaling_cur_freq may differ
from actual hardware frequency. [intel_pstate documentation](https://docs.kernel.org/admin-guide/pm/intel_pstate.html)
explains active-mode policies/HWP; powersave is not a fixed minimum-frequency
setting. These labels alone cannot explain a timing change.

Upstream v7.0 [CPUFreq sysfs code](https://github.com/torvalds/linux/blob/v7.0/drivers/cpufreq/cpufreq.c)
and [x86 frequency code](https://github.com/torvalds/linux/blob/v7.0/arch/x86/kernel/cpu/aperfmperf.c)
show both average/current surfaces using the architecture accessor on this
configuration. It reads recent APERF/MPERF-derived scheduler samples, with a
roughly20ms age boundary and a fallback when no usable recent sample exists.
Fallback/freshness is not exposed per sysfs read; the two fields are not
independent confirmations. This is an upstream reference, not attestation of
the running Ubuntu kernel binary or every vendor patch. Do not identify a
fallback as a measured idle frequency, normalize request duration with it, or
claim the observer is nonperturbing. No forced MSR/perf sampling is introduced.

perf/turbostat are installed, but perf_event_paranoid=4, effective capabilities
are empty and MSR devices are unreadable to this task. Use already readable
sysfs/proc surfaces; no privileged invocation, sudo, module, sysctl, device or
permission change. An optional local upstream-source download returned HTTP429;
retain it without retry. Already inspected primary URLs remain sufficient for
the stated limitations; do not promote missing local copies to kernel attestation.

## Reuse all runtime code and accepted verification

Reuse the **exact** accepted YT-159 reference/current ssr-thread-cpu-probe
executables, compiler/library/source/lock identities and untouched caller
ssr-stage-probe.js. Reuse its exact k6 binary/acquisition/version proof. No
rebuild, test, format, feature/crate/runtime/sample-code or caller change. Verify
the reused identities before and after execution; keep original artifacts intact.
Only this report and docs/BACKLOG.md, docs/ROADMAP.md, docs/README.md may change.
The combined worktree will have fourteen paths, staged0.

Extend the accepted finite task-local monitor solely to collect the missing
frequency/activity observations. This is a bounded observer for the requested
workload, not a runner/validator/proof framework. Preserve all original native
lifecycle, body/header/status, resource, request-ID and stage-clock handling.
New populations differ by their external observation and baseline phase; they
are not an uninstrumented replay or replacement of accepted YT-159 results.

## Fixed new observation sequence

Exactly four fresh app phases: reference1, current1, current2, reference2. Each
has one exact medium preload, **two seconds of post-preload no-HTTP baseline**,
two-second125/s warmup, one-second no-HTTP idle, then medium128-row1,000/s for
ten scheduled seconds. Use the original64 fixed VUs,8s timeout/graceful stop,
HTTP/1.1 loopback keep-alive, identity encoding and exact response/CSP oracle.
Nominal40,000 measured /1,000 warmup arrivals; preserve actual extras/failures.
No additional rate, small/large page, busy loop, fixed-frequency control,
uninstrumented control, repeated point or tuning to obtain a favorable result.

Keep server two Tokio workers, CPUAffinity12–13/CPUQuota200%/MemoryMax256MiB/
TasksMax128/NOFILE512/Nice10/no restart/RuntimeMax45s/native stop≤5s. Keep client
plus monitor CPUs14–19/600%/1GiB/TasksMax128/NOFILE512/Nice10/no restart/
RuntimeMax600s. Affinity is not exclusive ownership: other host work/package
power/thermal behavior can affect CPUs. No cpuset controller is delegated.

Within the existing monitor loop, target **20Hz** outside server workers during
the two no-HTTP periods and each k6 invocation. Keep original roughly1Hz resource
sampling separate. For each new observation record epoch and monotonic start/end,
read duration, phase segment, cpuinfo_avg_freq for CPUs12/13, per-CPU /proc/stat
counters and every owned app thread's kernel TID/starttime/comm/state/last CPU /
schedstat. Use read-only paths and bounded output. Do not query unrelated PIDs.
Read errors retain path/errno and stop further execution with owned cleanup;
do not silently substitute another frequency source or replay a phase.

Native /proc worker IDs are independent of YT-159's opaque pthread identities;
**do not invent an ID mapping**. Main TID equals PID; preserve all other observed
TIDs/names and verify their lifecycle. Source/runtime has two workers and no
new thread. Calculate per-thread running-time delta / elapsed time from schedstat,
and aggregate workers against two-thread capacity. The remainder is non-running,
not measured hardware C-state idle; it also contains blocking/descheduling.
Sampled R/S states are sparse endpoints, not time residency. Kernel runqueue
counters are context, not per-request waits or proof that none occurred.

[Scheduler counter semantics](https://docs.kernel.org/scheduler/sched-stats.html)
support deltas. Per-CPU /proc/stat is system-wide tick accounting, not app CPU;
retain idle and iowait separately, avoid guest double counting and report any
counter reversal rather than clamping. Use at least one-second bins for duty
arithmetic; a50ms interval has coarse USER_HZ resolution. Record SC_CLK_TCK.

## Temporal alignment and interpretation

Report actual observation counts/intervals/read spans/errors for each segment.
Measured context comparability requires≥150 successful observations per10s,
maximum gap≤150ms, maximum read span≤25ms and stable owned TID/starttime sets.
Keep ordinary comparability failures in the four-phase sequence without tuning;
only valid complete pairs get new source ratios. Body/source/clock/observer /
required-read/resource failures stop further live work and retain partial data.
Use existing HTTP comparability: native success/exact output/no errors/drops /
interruption,≥95% nominal valid window completions, valid IDs/clocks,≥2 inner
resource samples/gap≤2s, client intervalCPU<480%, cumulative memory<819.2MiB
and zero observed throttle/memory/task/FD-limit events. Retain sampling slivers.

Align reported frequency, aggregate/per-thread activity and selected construction /
empty CPU distributions by the same **one-second client scenario bins**, using
recorded wall/monotonic read spans and request IDs. Keep boundary samples separate.
Use CPU12/13 readings as coarse concurrent context, not a nearest-value exact
frequency assigned to an individual request. Report both CPUs and all bins,
not a favorable subset or assumed equality of policy/current-frequency surfaces.

Retain complete new HTTP/native/stage/empty timing populations and selected
worker/tail limits. Use nearest-rank p50/p95/p99/extrema and no exclusions,
clamping or independent-percentile subtraction. Frequency/CPU/duty associations
are descriptive serial observations with unobservable freshness/fallback and
observer effects; no IID/significance, normalized cycles/instructions, causal
percentage attribution, unique historical cause, capacity/SLO or production claim.
If lower reported frequency accompanies longer CPU/empty timings, say consistent
with that context; if evidence is mixed or unavailable, preserve that outcome.

## Bounds, cleanup and acceptance

Before observer implementation set one fixed **40-minute** UTC/monotonic deadline,
never renewed. New runtime/analysis/cleanup≤600s inside it. Task-owned ext4 with
≥8GiB free;≤128MiB/file /512MiB total new evidence. Accepted raw data stays outside
the cap and untouched. Reuse finite receipts, native unit controls and direct
arithmetic. No new calibration/test matrix or validation infrastructure. Classify
failures; one smallest correction per distinct seam, same consecutive signature
stops. Keep all failed results and original clocks; no favorable workload replay.

Stop/reap only owned apps/client/monitor. Retain native exits/stage dumps, verify
units inactive/MainPID0, original PIDs/cgroups absent and listener closed; no
root-cgroup fallback when final ControlGroup clears. Compare pre/post readable
policy settings to document that no task tuning occurred; coincident external
changes remain explicit. Retain inert artifacts. Required repository checks are
cargo xtask check docs and git diff --check after documentation changes; accepted
Rust/test/build/browser/provider/security qualifications are reused without replay.

Independent contract PASS precedes observer implementation; final Reviewer PASS0
precedes Done at bounded research scope. Freeze four file/data/history/source /
executable identities. After PASS only status/reviewer references in these four
documents and required docs/whitespace checks may change. No next item, product
optimization, commit/push, YT-096 Pending decision, YT-020 Blocked status or
immutable release-scope change is activated here.

## Results and bounded interpretation

Independent contract Reviewer `/root/yt160_frequency_duty_contract_review_20261024`
returned PASS0 before observer implementation. Final independent Reviewer
`/root/yt160_frequency_duty_final_review_20261024` returned PASS, zero issues/required
followups, after raw arithmetic/source/identity/cleanup review. YT-160 is Done
at bounded research scope; historical HTTP-tail causality remains unresolved. No product/sample/runtime source, feature, lock,
caller, executable or accepted verification was changed or replayed.

### Four new externally observed populations

All four phases ran once in reference1/current1/current2/reference2 order, using
the exact accepted YT-159 executables and caller. All eight k6 invocations and
four native app stops exit0. The new observations retain **40,004 measured +
1,003 warmup valid200** responses, zero errors/drops/interruptions/invalid200s,
and four exact medium preloads. Both complete source pairs satisfy HTTP and
context comparability. All1,284 selected IDs /2,568 same-thread construction
and empty measurements remain valid and uniquely correlated, without observer
loss or invalid clock. These are new observation populations with an added
baseline and external reads, not replacement historical benchmark results.

The external observer retains **1,176** context records:156 baseline,160 warmup,
80 inter-invocation idle and780 measured-invocation records. All app thread sets
remain stable with one main TID and two tokio-rt-worker TIDs, including original
starttimes. All required reads succeed. Of780 measured-invocation observations,
**769** lie wholly inside the client scenario window;11 startup/drain boundary
records remain separate. No retained window observation straddles a one-second
bin in this population. Native TIDs are never mapped to opaque pthread IDs.

### Lower reported frequency accompanies higher CPU duration

Frequency is the unweighted median of observed cpuinfo_avg_freq readings, shown
in GHz. It is coarse kernel-reported feedback with unobservable freshness /
fallback, not each request's execution frequency. Construction/empty CPU values
are microsecond medians of313 selected measured requests per phase. Worker
running share divides observed schedstat runtime by two-worker elapsed capacity.

| Execution order | Window context n | CPU12 / CPU13 reported GHz | Construction CPU µs | Empty CPU µs | Worker running share % |
| --- | --- | --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 193 | 1.906 / 1.900 | 111.601 | 0.544 | 23.90 |
| 02-r1-medium-1000-current | 191 | 1.301 / 1.478 | 145.332 | 0.710 | 23.53 |
| 03-r2-medium-1000-current | 192 | 1.397 / 1.500 | 143.157 | 0.715 | 22.86 |
| 04-r2-medium-1000-reference | 193 | 1.800 / 2.000 | 109.064 | 0.543 | 23.52 |

Current/reference reported frequency medians are lower **31.73%/22.22%** on
CPU12/13 in pair1 and **22.39%/25.01%** in pair2. Construction CPU p50 is higher
**30.22% /31.26%**, while fixed empty-clock CPU p50 is higher **30.51% /31.68%**.
The direction is consistent across both pairs and both observed CPUs, and
supports frequency/execution context as a credible contributor to this new
construction-clock scaling. It does not measure fixed work, effective cycles,
a causal percentage or the historical full HTTP-tail cause. Do not multiply
independently sampled frequency and duration to manufacture normalized cycles.

Worker running share is similar, about22.86–23.90% of two-worker capacity, with
source differences below0.67 percentage points. Thus aggregate running/non-running
time alone does not identify why reported frequency differs. Per-worker balance
also varies: one worker may carry materially more execution in a phase. Snapshot
R/S counts and the non-running remainder cannot establish hardware idle residency
or wakeup-transition behavior. No C-state or privileged hardware-counter trace
was collected.

Each planned two-second post-preload baseline contributes39 observations and
each one-second inter-invocation idle contributes20. Their frequency medians
are800MHz on both CPUs, with zero worker runtime delta across their **covered
sample spans**. Those readings may include the architecture fallback; they do
not prove a hardware idle frequency or complete-interval zero activity. No task
HTTP requests are issued during these two intervals.

All ten one-second bins per source, both CPU distributions, worker runtime /
runqueue deltas and construction/empty distributions remain in
frequency-duty-summary.json. Read spans are wholly contained in bin membership;
request grouping uses client start-time bins rather than exact server execution
windows. No nearest frequency is assigned to individual request IDs. Some bins
contain800MHz reported medians even while requests execute; the data is not
forced into a monotonic per-bin frequency/duration explanation.

### Accounting and precision limits

Measured context counts are193/191/192/193. Actual interval maxima are
57.962/57.491/56.100/56.667ms, within150ms. Read-span maxima are
2.225/2.276/3.591/2.323ms, within25ms. These are serial, sparse observations
with read/monitor/host effects, not independent trials or exact per-request
frequency timestamps. The added observer can change shared-package load.
No unique frequency, compiler, cache, allocator or historical p95 cause follows.

Raw /proc/stat counters are retained with idle/iowait separate and guest fields
excluded from the first-eight-field total. There are no observed reversals.
However, totals expressed at SC_CLK_TCK=100 span **8.28–10.44 tick-seconds**
against approximately9.92–9.96s elapsed for the matching endpoint windows.
Those coarse/system-wide accounting populations do not fully reconcile to
elapsed wall time or owned-worker runtime. Preserve that limitation; do not
turn their idle-counter shares into exact core residency or app idle time.
Worker schedstat and resource samples remain separate observations; this item
does not diagnose the kernel-accounting discrepancy.

Upstream source inspection explains potential caching/fallback; it does not
attest the running Ubuntu kernel implementation. Both sysfs names can use the
same accessor, so this study samples only the declared average field at20Hz.
No perf/MSR/turbostat privilege or permission bypass is used. Initial metadata
reads of the second field are capability inventory, not a second live frequency
confirmation. Neither hardware maximum nor governor labels substitute for
observed execution conditions.

### Full HTTP results remain mixed and unfavorable tails stay

HTTP values below are milliseconds; app CPU/RSS are the existing inner resource
windows, separate from schedstat context endpoints.

| Execution order | HTTP p50 / p95 / p99 ms | App CPU % | Sampled RSS MiB | Selected HTTP-tail n |
| --- | --- | --- | --- | --- |
| 01-r1-medium-1000-reference | 0.577546 / 1.270386 / 2.645447 | 47.59 | 7.18 | 11 |
| 02-r1-medium-1000-current | 0.606083 / 1.438123 / 3.312209 | 46.26 | 6.86 | 19 |
| 03-r2-medium-1000-current | 0.599650 / 1.347562 / 3.056086 | 45.62 | 7.13 | 21 |
| 04-r2-medium-1000-reference | 0.571609 / 1.110633 / 2.593089 | 46.59 | 7.03 | 11 |

Current/reference HTTP p95 is higher **13.20% /21.33%** in this new observer
context. Both current app CPU means are slightly lower, but no universal speedup,
capacity claim or causal HTTP budget is inferred. Complete HTTP maxima in order
are5.597333/11.794108/10.837366/10.703222ms. All native components, floating HTTP
versus integer-ms full-call populations, tails, boundary events, per-worker stages
and warmups stay retained without exclusions or favorable replay. Sampled tail
cohorts are only11/19/21/11; deterministic sampling is not random-IID evidence.

### Decision and remaining question

The observed association makes **frequency/execution context a stronger candidate
than an unsupported claim of extra construction instructions**. Current reported
frequency is lower while construction and fixed empty CPU durations rise together.
Aggregate worker activity is broadly similar; this does not establish the idle /
wakeup mechanism or uniquely attribute historical HTTP p95. A causally controlled
frequency comparison was deliberately not performed on this shared host.

Retain the accepted product optimizations and current APIs. No safe product-code
optimization or rollback is selected. If separately requested, the next useful
decision is a reproducible benchmark execution profile on an explicitly isolated
environment, with declared frequency-policy controls and observation limits.
Do not keep adjusting Core/serializer code to chase this shared-host tail or
change the current host policy under this item. No such next task, environment
or tuning action is started here; original HTTP-tail causality remains open.

### Reused verification, failures, bounds and cleanup

The exact accepted reference/current executables, source snapshots, library /
compiler/lock identities, caller, k6 acquisition/version and test/build/format
results are reused. No Rust or runtime feature/source/build/test changed or ran.
Repository verification uses cargo xtask check docs and git diff --check after
this report/index update. Four documentation paths are the whole new repository
scope; all prior sample/product files and historical reports remain unchanged.

Two preparation failures are retained. The optional local upstream-source copy
returned HTTP429; no retry, installation or privilege recovery was attempted,
and primary URLs already inspected through the web remain referenced. The
finite observer adaptation initially matched a clock-initialization substring
twice, raising AssertionError before any observer file/live work. One smallest
line-boundary correction completed preparation. The before-script, original
exit1 facts and corrected native receipt remain separate. No live workload /
accepted test/build/arithmetic was replayed and the original task clock stays.
There are no new failed live, response-validation or arithmetic invocations.

Native runtime takes64.382s; direct arithmetic and owned cleanup complete
242.876s after its start, within600s and the original unrenewed40-minute deadline.
HTTP raw output totals **225,030,496 bytes**, context output **1,314,265 bytes**;
max HTTP file **55,054,063 bytes**, within512MiB total /128MiB per-file limits.

Measured valid window completions are39,996; five task-start timestamps fall
outside the half-open schedule and eight completions occur at/after its end.
Nominal-minus-started-minus-dropped remains−4. Warmup actual1,003 against1,000
nominal is retained; window/drain rates remain distinct from capacity.

Client peak interval CPU is152.45% (<480%), cumulative charged memory peak
116.61MiB (<819.2MiB), maximum resource-sampling gap1.013s.
Observed app/client throttle/memory/task/FD-limit events are zero; all measured
HTTP and context comparisons meet the declared conditions. Boundary slivers
and potential short peaks are not claimed absent.

Cleanup confirms **five inactive owned units,13 absent workload PIDs, absent
original cgroups and closed listener**. Native app stops/client/controller exit0
are retained; cleared final ControlGroup means endpoint unavailable, not root
cgroup or relabeled last counters. Pre/post readable governor/EPP/frequency limits,
pstate and perf policy match. Task code performs only reads of those settings;
matching snapshots do not exclude coincident transient external changes.
All139 runtime frozen inputs match at completion. Inert artifacts stay for
independent review; no service, host setting or external resource was changed.

Final independent Reviewer `/root/yt160_frequency_duty_final_review_20261024` PASS0
supports Done at bounded research scope and the permitted four-document status/
reviewer-reference closeout. The combined worktree is fourteen paths, staged0,
with no commit/push, optimization, YT-096 decision or immutable release change.
