# YT-158 — Render history and construction CPU context

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded attribution scope; independent final Reviewer PASS.**

## Authority and bounded question

Evidence: `target/yt158-render-context-8x1j_5g7/evidence`.

The user asks to continue the proposed cross-request construction/allocation-state
versus runtime/OS scheduling research after [YT-157](YT-157-MEDIUM-HTTP-TAIL-ATTRIBUTION.md).
Preserve accepted evidence and failures, API/output/security, contract-first
minimum measurement and independent Reviewer before Done. No optimization,
commit or push is authorized. The nine accepted, uncommitted YT-157 paths are
snapshotted before new edits; their report, stage observer and caller stay intact.

YT-157's actual medium construction code is identical between product copies,
yet sampled construction p50 is about 0.112 ms reference versus 0.146–0.149 ms
current. That is an observation, not an allocation count or cause. Its middleware
wall clock cannot distinguish CPU work from waiting. This item asks whether
construction remains sensitive to preceding render history on a synchronous
single thread, and whether new construction wall-time changes accompany thread
CPU-time changes. It does not reproduce the HTTP p95 or prove allocator causality.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Product libraries, dependencies,
locks, default/browser graphs, normal route/handler/fixture/oracle, escaping, URL
validation, response headers, document capacity and body release remain unchanged.
No allocator swap/tuning/flush, unsafe product change, profiler installation,
host setting, new runtime feature, provider, cache or benchmark framework.

## Minimum new diagnostic boundary

Only the existing standalone HTTP sample gains default-off
`ssr-context-study = []` and private `src/bin/ssr/context_study.rs`. With this
feature, `ssr reference|current 1|2` executes a synchronous local experiment and
never starts a server or Tokio runtime during measurements. It is mutually
exclusive with `ssr-stage-probe`; normal/default and YT-157 code paths remain
unchanged. Reuse the same byte-identical `page(128)` fixture, actual
`Page::with_metadata`, actual `Page::into_response` and System allocator.

Two predecessor contexts, each fully released before the measured construction:

- **construct-only**: construct a fresh medium Page, then drop it without render.
- **rendered**: construct a fresh medium Page, convert it through the actual
  IntoResponse, then drop the complete response. This naturally releases the
  original Page/tree inside IntoResponse and body on response drop.

Immediately after either predecessor, measure fresh `page(128)` plus
`Page::with_metadata` only; retain the resulting Page until clocks stop, then
drop it. No prebuilt View, extra serialization, observation allocation, printing,
sleep, runtime, HTTP or body drain occurs inside that measured boundary.
Each cycle therefore uses two fresh Pages. These contexts change allocation,
cache, instruction and lifetime history together; a difference is **render-history
sensitivity**, not proof of which allocator bin/cache/compiler mechanism caused it.

Use nested clocks on the same synchronous thread: outer Instant wall start,
`clock_gettime(CLOCK_THREAD_CPUTIME_ID)` start, construction, thread CPU end,
outer wall end. Return codes and nonnegative deltas are checked. The outer-minus-
inner gap includes two clock calls/observer work as well as possible descheduling;
it is not exact off-CPU time. Keep signed gaps, do not clamp/exclude samples or
subtract marginal percentiles. Store 128 empty-bracket observations per process
as a clock-overhead context, without automatic subtraction or clock certification.

This Linux x86_64 GNU-only diagnostic uses small private C-ABI declarations for
thread CPU clock/resolution, `sched_getcpu` and read-only `mallinfo2`. Match installed
headers; no new dependency. Other targets get a clear unsupported diagnostic.
Outside measured groups record current CPU, thread `/proc` status/schedstat and
allocator arena/free-block/allocated/free-byte counters before/after each group.
These coarse allocator counters do not expose tcache ownership, allocation-call
latency, CPU frequency or other arenas and cannot prove reuse or a leak.
Do not reset allocator state. These observations themselves perturb subsequent
history; identical code and fixed counterbalancing limit, but do not remove, it.

Primary semantics: [thread CPU clock](https://man7.org/linux/man-pages/man3/clock_gettime.3.html)
counts calling-thread CPU time; [mallinfo2](https://man7.org/linux/man-pages/man3/mallinfo.3.html)
has documented visibility limitations. Preserve observed resolution separately
from accuracy; no CPU-frequency normalization or inference from a governor name.

## Fixed population and sources

Copy the accepted YT-157 reference/current source directories into task-owned
directories, retaining product refs `90f83867` / `0580a0a`; current products match
HEAD `25a0cc6`. Overlay identical final sample bytes, verify source/lock/binary
identities and preserve originals. No worktree, checkout/reset or accepted-proof
replay. Two new offline/locked default-release builds select only the new feature,
two jobs, unchanged Rust/Cargo1.99.0 and default System allocator/profile.

Exactly **four new processes**: reference1, current1, current2, reference2. Each
process has eight blocks, each with the two contexts; reverse context order on
odd blocks and reverse the initial order in run2. Before each of the sixteen
measured groups execute exactly256 unrecorded warmup cycles of that context,
then record exactly256 cycles. Preserve group start/end context and all block
distributions, including unfavorable tails. Allocate all sample/context storage
before warmup; print results only after every measured group completes.

Total: **16,384 measured constructions, 16,384 warmup cycles, 512 empty brackets**
across four processes; each measured/warmup cycle has one preceding Page and one
target Page. Fixed call count is priming, not proof of thermal/allocator equilibrium.
No new offered-rate, HTTP, small/large, capacity or existing accepted benchmark.

Use one allowed logical CPU12 per process, sequential execution, task-owned
transient user units with CPUAffinity12/CPUQuota200%/MemoryMax256MiB/TasksMax32/
NOFILE128/Nice10/no restart/RuntimeMax60s. One CPU permits at most100% execution;
200% quota avoids intentionally creating a quota stall. Do not alter host policy,
SMT sibling, frequency, existing services or global caches. Record native unit
exits and limits, actual CPU/thread/status/resource endpoints and ancestor context.
No external network or resource. Retain partial failures; stop on invalid output,
source drift, missing required data or resource limit. Timing direction alone
never authorizes tuning, exclusions or replay.

Before implementation set one fixed **35-minute** UTC/monotonic deadline, never
renewed. Each build/test ≤600s, all four runtime groups plus analysis/cleanup
≤600s inside it. Task-owned ext4 requires≥8GiB free; evidence≤32MiB and≤8MiB/file.
Use existing finite command receipts/native unit controls and direct arithmetic;
no new runner/validator/proof framework. Classify materiality before one smallest
correction to a distinct failed seam; same consecutive failure stops. Never
restart the task clock or replay passed work to obtain a favorable outcome.

## Reporting and acceptance

Direct Page conversion exposes an exact body size hint; HTTP routing/transport
adds Content-Length later. The diagnostic checks that actual direct-Page boundary,
not a synthetic HTTP header. The host delegates cpu/memory/pids but no cpuset
controller; native CPUAffinity and observed CPU/allowed-list checks enforce CPU12.

One new focused test checks actual medium Page/body/strict CSP/headers against
the existing independent literal, both predecessor paths and positive CPU/wall
records. Compile with offline/locked `--no-run --message-format=json --jobs 2`
and execute only emitted `yt158_ --test-threads=1`; exclude accepted normal and
YT-157 tests by cfg. One rustfmt check of changed Rust and repository docs/whitespace
checks. No security/browser/platform/provider requalification or old load replay.

Report exact counts, warmups, all wall/CPU/gap distributions, per-process empty
brackets, per-block context/tails/extrema and group CPU/scheduler/allocator deltas.
Only compare source ratios with complete/native-successful populations and
matching source/build/context. Use nearest-rank percentiles. Never add marginal
percentiles, treat serial observations as independent trials, claim CPU differences
are instruction counts or infer unique allocator/scheduler/HTTP causality.
If CPU tracks wall changes while gap is small, only say the synchronous cost is
CPU-visible in this diagnostic. If history matters, say history-sensitive; if
evidence is mixed/unresolved retain that outcome. Do not subtract unrelated
historical/probe clocks or replace YT-157 populations with these new ones.

Done means completed bounded attribution, not unique-cause resolution. Keep
current products and propose only an evidence-supported next question; no
optimization or additional task starts here. Independently reviewed results and
source/evidence bindings precede Done. After final PASS only status/reviewer
references in this report and the indexes/sample README, then docs/whitespace
checks, may change. Eight YT-158 paths: this report, docs/BACKLOG.md,
docs/ROADMAP.md, docs/README.md, sample Cargo.toml, README.md, src/bin/ssr.rs and
new src/bin/ssr/context_study.rs. They overlap six existing dirty paths, so
the combined uncommitted YT-157/YT-158 union is **eleven paths**. Preserve
YT-096 Pending decisions, YT-020 Blocked and immutable release scope.

## Results and remaining question

Independent contract Reviewer `/root/yt158_render_context_contract_review_20261024`
returned PASS0 before implementation and separately PASS0 for the direct-Page
header/CPU-affinity adaptations before any measured process. Both decisions,
original/corrected source identities and the initial failed test remain retained.
Final independent Reviewer `/root/yt158_render_context_final_review_20261024` returned PASS,
zero issues/required followups. YT-158 is Done at bounded attribution scope;
the original HTTP-tail cause remains unresolved.

### Complete fixed population

All four native processes completed once in the planned reference1/current1/
current2/reference2 order, exit0. All **16,384 measured constructions, 16,384
warmup cycles, 512 empty brackets and64 groups** are retained, with exact block/
context/index order. Each measured/warmup cycle constructs two fresh Pages; no
View is reused. All64 before/after pairs observe one application thread, CPU12
and allowed-list12. Every process independently verified the actual direct Page
body, exact31,063 bytes, size hint, content type, CSP and forbidden headers before
measurements. No HTTP server or runtime ran during the measured groups.

Raw output totals **2,180,673 bytes**. Native runtime control took **7.179s**;
analysis and owned cleanup completed **80.266s** from its start, inside the600s
allowance and original unrenewed35-minute stage deadline. No old test/build/HTTP
load, accepted performance population or failed load was replayed.

### Construction wall and CPU time

All values below are microseconds, nearest-rank percentiles; each row has2,048
measured constructions. The target boundary stops before dropping its Page.
No outliers are excluded, negative gaps clamped or clock corrections subtracted.

| Process | Predecessor | Wall p50 / p95 / p99 | CPU p50 / p95 / p99 | Gap p50 / p95 | Wall max |
| --- | --- | --- | --- | --- | --- |
| 01-reference-r1 | construct-only | 47.234 / 47.803 / 49.632 | 46.888 / 47.428 / 49.215 | 0.347 / 0.399 | 68.566 |
| 01-reference-r1 | rendered | 47.468 / 48.039 / 50.001 | 47.095 / 47.656 / 49.579 | 0.370 / 0.428 | 55.887 |
| 02-current-r1 | construct-only | 46.975 / 47.625 / 49.395 | 46.630 / 47.223 / 48.977 | 0.346 / 0.402 | 155.661 |
| 02-current-r1 | rendered | 47.232 / 47.821 / 49.788 | 46.860 / 47.425 / 49.379 | 0.372 / 0.426 | 60.409 |
| 03-current-r2 | construct-only | 46.838 / 47.988 / 49.599 | 46.481 / 47.581 / 49.214 | 0.351 / 0.401 | 67.059 |
| 03-current-r2 | rendered | 47.837 / 48.646 / 50.443 | 47.457 / 48.228 / 49.938 | 0.376 / 0.436 | 53.590 |
| 04-reference-r2 | construct-only | 48.325 / 49.039 / 50.751 | 47.966 / 48.516 / 50.331 | 0.358 / 0.413 | 51.942 |
| 04-reference-r2 | rendered | 48.703 / 49.456 / 51.200 | 48.309 / 48.974 / 50.759 | 0.387 / 0.447 | 81.039 |

Both complete source pairs meet the declared native/count/context/resource
conditions. Current/reference construction CPU p50 changes are **−0.55%/−3.10%**
after construct-only predecessors and **−0.50%/−1.76%** after rendered predecessors;
CPU p95 changes are −0.43%/−1.93% and −0.48%/−1.52%, respectively. Wall ratios
are similar. These are two serial, counterbalanced source observations, without
independent-trial confidence or universal performance claims. They do not replace
YT-157's actual HTTP populations or establish that a construction difference
cannot occur on two concurrent Tokio workers.

Within each process, rendered/construct-only CPU p50 increases are **0.44%,
0.49%, 2.10%, 0.72%** in execution order; p95 increases0.48%,0.43%,1.36%,0.94%.
Thus a small CPU-visible history sensitivity appears in this intervention, with
substantial process/order variation. The current/reference ratio of those
history ratios is1.0005 /1.0137 for CPU p50. This is not a causal decomposition
of the old approximately30% construction-clock difference: rendering changes
allocator, cache, instruction and lifetime state together, and no allocation
call count, bin ownership, instruction count or frequency trace was collected.

Per-block CPU p50 ranges (construct-only / rendered) are retained rather than
collapsed into a favorable aggregate:

| Process | Eight block p50 ranges, µs | Group involuntary / voluntary switches | Group runqueue-wait sum, µs |
| --- | --- | --- | --- |
| 01-reference-r1 | 46.615–47.073 / 46.835–47.312 | 9 / 0 | 143.145 |
| 02-current-r1 | 46.407–46.742 / 46.661–46.923 | 7 / 0 | 172.674 |
| 03-current-r2 | 46.221–47.313 / 47.255–47.721 | 9 / 0 | 92.159 |
| 04-reference-r2 | 47.840–48.092 / 48.162–48.425 | 9 / 0 | 103.811 |

### Clock, scheduler and allocator limits

Thread clock reports1ns resolution, not1ns accuracy. Empty-bracket wall p50 is
534–537ns, CPU p50 is258–260ns and outer-minus-inner gap p50 is276–277ns.
Construction gaps have p50 346–387ns and p95 399–447ns across complete contexts,
with zero negative gaps. The bracket includes observer work; these small central
gaps cannot certify that scheduling is absent.

The retained current1 construct-only maximum has wall155.661µs, CPU52.769µs
and gap102.892µs in the **same** observation. That group has one involuntary
switch, but counters cover all256 predecessors/targets plus context work, so
they cannot assign a switch to that individual sample. Other extrema, including
reference2 rendered81.039µs, remain in the population. All64 group distributions,
empty brackets, joint per-record wall/CPU/gap and every context endpoint remain
in the raw data and `context-statistics.json`. Do not subtract marginal maxima
or percentiles. [Kernel scheduler counters](https://docs.kernel.org/scheduler/sched-stats.html)
are coarse group context, not measured target-only CPU/wait or a historical trace.

All allocator context snapshots report arena372,736 bytes and mmap space331,776
bytes. Within every measured group, arena, allocated-byte and free-byte endpoint
deltas are zero; allocated-space endpoints vary28,976–31,904 bytes across
process/history contexts. That is compatible with differing retained process
state, but does not identify reused blocks, tcache state, cumulative allocation
requests or a leak. A stable endpoint does not imply zero allocation work.
VmRSS observations range3,520–3,976KiB and rise as preallocated observer storage
is touched/filled; do not attribute that rise to retained product Pages.

Sampled charged-cgroup memory peak is2MiB in each process, one sampled task
and at most three sampled FDs. Observed cpu throttle / memory-limit counters
are zero. These are cgroup accounting/sampled-context values, not whole-process
RSS or proof that no boundary event occurred. The units clear ControlGroup and
the original cgroups disappear at exit, so final endpoint counters are explicitly
unavailable; the last observed values are not relabeled as final counters.

### Bounded conclusion and next possible question

The fixed synchronous diagnostic does **not** show a slower intrinsic current
construction path: current CPU and wall distributions are slightly lower in
both source pairs. Render history has a small CPU-visible effect here, without
a stable large current penalty matching YT-157. This narrows the explanation but
**does not resolve the original HTTP p95 cause** or exonerate accepted products.
Different worker/thread arenas, overlap of response lifetimes, dispatch/caller
placement, host/frequency and observer interaction remain outside this intervention.

Retain current product code. No safe optimization target is established and no
allocator change, capacity tweak or runtime tuning is selected. If separately
requested, the next focused measurement is **thread CPU versus wall at the
existing sampled synchronous construction boundary on actual HTTP workers**,
with explicit same-thread validity and observer overhead. Reuse this study
instead of repeating its four processes. No such next item/load starts here.

### Verification, retained failures and cleanup

One new filtered test now passes: `cargo test --bin ssr --features
ssr-context-study --offline --locked --no-run --message-format=json --jobs 2`,
then the emitted executable `yt158_ --test-threads=1`: **1 passed,0 failed,
0 ignored/filtered**. Initial direct-Page Content-Length assertion exit101 is
preserved in `focused-test.*`, with before-source and materiality record. One
smallest correction checks actual absent direct header plus exact size hint/body.
The shared premeasurement verification source changed, so only new-feature test/
release artifacts were refreshed under distinct corrected receipts; initial
builds, identities and original executables are preserved. No product or accepted
old proof was reopened. Both corrected release builds pass without warnings;
features are default/ssr-context-study, actual library artifacts bound to copies.

Focused Rust formatting passes. The original source-edit patch preparation failed
because its expected constant was multiline while the source uses one line; one
exact-token edit completed the original change. A manager query initially parsed
the dash-prefixed slice name as an option; one `--` correction passed. Native
controller inspection then established that cpuset is not delegated; reviewed
CPUAffinity plus actual thread checks enforce CPU12 without host changes. All
preparation/verification failures and decisions remain separate, with no renewed
clock, favorable replay or source change after measurement. Required documentation/
whitespace checks follow the report update.

Cleanup reconfirms four inactive owned units, four absent application PIDs and
absent original cgroups. Native runtime controller exits0, waited with leader
absent. No listener/server, external resource or existing-service change occurred.
Sources, runtime code, binaries, raw observations and retained failures stay
frozen with accepted final review; only the permitted five-document status/
reviewer-reference closeout and documentation checks follow PASS.
The combined worktree still has eleven uncommitted paths (nine accepted YT-157
plus two new paths), staged0. No commit/push or release-decision change.
