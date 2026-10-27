# YT-153 — Document buffer and body temporary attribution

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded attribution/proposal scope after independent final Reviewer PASS.**

## Authority and unchanged product boundary

The user selects contract-first attribution of document buffer growth and body
temporary costs, reusing YT-150–152 evidence, minimum new measurement, unchanged
API/output/security, one minimum optimization proposal after observing hotspots,
no optimization implementation, no accepted-proof replay, retained failures and
independent Reviewer PASS before Done. No commit/push is authorized. Baseline is
clean `30edd16217f697706629c8c798b41f72ebfd5b5d`, in the current checkout.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). All product sources, public
APIs, Core representation, browser/default graphs, HTML validation/escaping/URL
policy, metadata/CSP/HTTP behavior, dependencies and root/sample locks remain
unchanged. Do not reserve, pre-size, fuse buffers, change temporary lifetimes,
introduce raw-HTML/destination-buffer APIs, copy a renderer, add a crate or
implement any optimization. Add only default-off measurement code to the
existing `examples/ssr-performance-baselines` sample.

Reuse [YT-150](YT-150-FULL-SSR-COMPARISON.md),
[YT-151](YT-151-VIEW-CONSTRUCTION-ATTRIBUTION.md) and
[YT-152](YT-152-FIRST-CHILD-CAPACITY.md) read-only. Current document observations
are 3 alloc / 21 realloc calls, 490,386 held and 507,337 peak logical bytes for
245,207 HTML bytes. View allocation work is separate; YT-152 rejected the
first-child policy and restored baseline product code. Do not reopen that
candidate, infer CPU shares from allocation counts, or promote prior elapsed
values into a new comparison. No old accepted command/test/build, Counter sanity
suite, browser/CSP/HTTP/provider/database or complete-page timing is replayed.
New evidence is under `target/yt153-document-buffer-iuwrao7_/evidence`.

## Source-derived adapter composition and finite observations

Add default-off `buffer-study = []` and private `src/buffer_study.rs`. Its
observed complete-document function copies only the small current adapter
composition sequence, adding lexical scopes and fixed stack observations.
It calls **actual unchanged** `yetlet_html::render_to_string` and `escape_text`;
no copied HTML renderer, validation, URL checker or optimized variant is used.
Bind the copied literals/order/calls to the actual adapter source by review.

Preserve exact String creation and append order: initial opening, language,
head/title opening, escaped-title temporary append, body opening, actual body
temporary append and final fourteen closing bytes. Keep title/body as borrowed
function-result temporaries in their original `push_str(&helper(...)?)`
statements. A helper may observe and return the owned String without cloning;
it must not extend destruction beyond the original statement. Do not change
capacity, perform a second length traversal or render earlier for counted work.

Capture nine named boundaries: initial, language appended, head appended,
title ready, title appended, body opening appended, body ready, body appended,
closed. Each records envelope length/capacity, any live temporary length/capacity
and global live/peak values (counted only). Title/body-ready helpers receive
the already-observed envelope length/capacity, record after the actual API
returns, then return its String to the original append expression. All records
are fixed stack data and allocate nothing.

Extend only the existing sample Counter with cfg-gated hooks into a fixed
64-event trace in this module. Record successful alloc/zeroed/realloc/dealloc,
failed alloc/realloc, old/new request bytes and post-event global live/peak.
Tag current call sites as body/title/envelope/other using the established
nonallocating atomic/lexical-restore pattern. Log frees under their **current
call site**, not a claimed owning bucket: title/body temporary destruction
occurs in envelope scope; final output release occurs outside it. Do not add
pointer/allocation IDs, owner tracking, generalized profiling or timing in the
allocator. Native selected work is single-threaded. A trace overflow remains
visible and stops interpretation; do not increase its bound to replace evidence.
Existing Counter fields/legacy hooks and all old study behavior stay unchanged.

## Counted population, reconciliation and interpretation

New `buffer-counts` runs **one fresh counted process, three document operations,
no warmups**, using the retained 1,024-row View/metadata prepared outside the
regions. Only the outer operation calls `counts::begin`. Capture global
start/held/released, the nine boundaries, the finite event trace/count/overflow
in stack records. Hold output at held and drop it before released. Finish the
trace after release. Print all raw observations before asserting conditions.

Require zero failures/overflow, release to own-start live, within-three-operation
stability, event-count/byte/live/peak reconciliation with global counters, and
all nine new/YT-150-current-document pairs matching all twelve relative fields
at held/released. A mismatch stops causal interpretation without retuning or
replay. Bind observed String capacities/lifetimes to the captured source and
boundaries; do not assume which append grows or impose a numerical reduction.

Report creation calls/request volume by scope and the actual event growth sizes,
including envelope final append, body buffer growth and temporary destruction.
Report global overlap at body-ready/append/release boundaries and the returned
String length/capacity. Global peak is not a sum of scope peaks. Realloc request
bytes do not measure physical copying, allocator internals, size classes or RSS;
no allocation ownership or exact CPU cost follows from trace tags. Source-level
body append volume is logical payload, not measured hardware memory traffic.

## Separate ordinary append probes

Build ordinary `buffer-study` separately from counted
`buffer-study,allocation-counts`; neither selects any old study feature.
Ordinary excludes the global allocator wrapper, and counted rejects timing.
Prepare metadata/View and one actual rendered body once per timing process,
outside all timer groups. Keep that borrowed body alive throughout the probes.
Use the same source-derived prefix/append sequence, without reserve/pre-sizing.

| Probe | Inside timer | Outside timer |
|---|---|---|
| envelope | Prefix/title composition, borrowed-body append and closing append; finish with document held | Body rendering, final String inspection/destruction |
| body_append | Only appending the borrowed body to an identical prepared prefix; finish with prefix+body String held | Prefix/title setup and final inspection/destruction; no closing append is timed |
| suffix_append | Only appending the fourteen closing bytes to an identical prefix+body String; finish with document held | Prefix/title/body append setup and final inspection/destruction |

All returned Strings and relevant inputs are black-boxed. Dispatch, preparation,
record allocation and printing are outside timer regions. Prefix setup is rebuilt
per operation for the two append-only probes; no mutated buffer is reused.
These probes have a retained borrowed body, different allocation/cache lifetimes,
loop/barrier/clock/compiler placement and no body-render CPU inside. They are
source-derived sample observations, not actual private adapter timing or a
before/after optimization. Do not subtract/add medians against YT-150 complete
document time, assign CPU percentages or claim savings from these probes.

New `buffer-timing 1|2` runs two fresh ordinary processes, envelope/body_append/
suffix_append in repetition 1 and reversed in 2. Each group has five warmups
and twenty measurements: **30 warmups / 120 measured operations total**. Retain
every integer ns/op, probe/repetition/order/population/index/result length;
report count, mean, median (average entries 10/11), nearest-rank p95 (entry
19/20), min/max per group. Keep variation, no pooling-only result, outlier
removal, performance threshold or additional repetition after an unfavorable run.

## New checks, exact footprint and bounds

Two new counted-config `yt153_` tests cover observed composition/probe outputs
against actual APIs plus independent literals at 8/128/1,024 rows, finite
nested unsafe-URL/forbidden-attribute/invalid-child error outcomes, lexical tag
restoration including unwind, and actual allocator event dispatch/release.
These are tests of new measurement behavior, not old Counter calibration or
product regression entrypoints. Compile once offline locked with `--no-run
--message-format=json`, then run only the emitted `yt153_ --test-threads=1`
filter. Run one sample format check and two new offline locked release builds.

New ordinary `buffer-output EXISTING_DIRECTORY` checks observed/full actual
document and body/literal equality at all three sizes, writes six outputs and
matches six accepted YT-150 hashes outside measurement. Also verify the untimed
probe results against the same literal boundaries. Source review must bind
product/fixture/lock/default-feature inputs unchanged and preserve the older
study code and historical evidence hashes.

Exactly nine repository paths may change:

```text
docs/YT-153-DOCUMENT-BUFFER-ATTRIBUTION.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/counts.rs
examples/ssr-performance-baselines/src/buffer_study.rs (new)
```

Stage 1 changes only four docs, checks them and obtains independent contract
review. Stage 2 starts one fixed **40-minute** UTC/monotonic deadline before
implementation, never renewed. Use Rust/Cargo 1.99.0, offline locked default
release/System allocator, two jobs and separate ordinary/counted ext4 build
targets. Record selected nonsecret flags, current cgroup identity/effective
ancestor limits/affinity and native environment receipts before assertions.
No tool installation or host/governor/quota/LTO/PGO/settings changes. Executables
use CPU 0 and 1 GiB address space. Build/test commands have 600-second caps within
the fixed deadline; output/timing/count commands have fifteen-second caps. The
**three-process** measurement batch has a sixty-second monotonic ceiling and
8 MiB combined stdout/stderr/receipt budget.

Persist native argv, UTC/monotonic boundaries, exit/timeout, raw stdout/stderr,
source/lock/binary hashes/sizes/features and process cleanup immediately. Use
only the existing finite receipt-helper pattern; no generalized validator,
runner, tracker framework or validation-infrastructure tests are authorized.
Preserve failed/partial/unknown results and apply the materiality gate before
the smallest correction. No automatic retry, fresh task/clock, tuning or passed
proof replay. Wait/join owned foreground commands and kill only owned children
on timeout. Start no service/browser/container/external resource; retain inert
artifacts under the task root and confirm no owned process remains.

After evidence, identify the observed growth/temporary hotspot and propose
**one separately authorized minimum optimization**, with peak/lifetime/time
tradeoffs, or explain why no proposal is justified. Implement none of it.
Parent docs/whitespace checks and final independent Reviewer PASS with zero
required followups precede Done. After PASS, only status/reviewer references
in four docs and sample README may change, followed by docs/whitespace checks;
Rust/Cargo/product/data/binaries remain frozen. No new optimization/item,
commit/push or YT-096/YT-020 release-gate decision follows.

## Execution and observations

Independent contract Reviewer
`/root/yt153_document_buffer_contract_review_20261024` returned **PASS**, zero
issues and required followups, before implementation. All four reviewed hashes
and ten historical bindings match. `contract-review-result.json`,
`contract-review-files.json`, `contract-review-trace.json` and
`contract-review-observed-decision.json` retain the gate and enforced native
role/model/effort evidence. Independent final Reviewer `/root/yt153_document_buffer_final_review_20261024` returned **PASS**, zero issues and required followups. All nine reviewed file hashes, fourteen data bindings, ten historical bindings and three executable hashes match. `final-review-result.json` retains acceptance and scope limits; the item is Done at bounded attribution/proposal scope.

### Source and verification bindings

All evidence names below are relative to the evidence directory above.
`measurement-source-bindings.json` freezes product/sample/lock/contract inputs
before output/measurement. `source-review.json` confirms unchanged original
fixture/constants/literal oracle, every out-of-scope tracked source, and the
entire prior Counter after removing exactly seven new cfg-gated calls. Actual
adapter/HTML/Core/metadata sources remain unchanged. The observed adapter copy
preserves all literal/call/append order and statement-end temporary destruction;
its body/title helpers call the actual product APIs. It is sample observation
code, not an actual private adapter entrypoint or new public interface.

| New command / receipt stem | Result |
|---|---|
| `format-write`, `format-check` | Initial sample format write/check, exit 0 |
| `compile-new-tests` | One offline locked counted-feature test compilation, no-run JSON artifacts, exit 0 |
| `format-check-normal-edit` | One focused format check after a new ordinary-only source edit, exit 0 |
| `new-tests` | Emitted binary `yt153_ --test-threads=1`: 2 passed, 2 old tests filtered, exit 0 |
| `build-normal`, `build-counted` | Two offline locked default-release builds, jobs 2, exit 0 |
| `new-output` | New all-size actual/literal/probe checks and six accepted hashes, exit 0 |
| `timing-1`, `timing-2` | Fixed 120 times / 30 warmups, both exit 0 |
| `buffer-counts` | Three observed document operations with complete raw event/boundary/global observations, exit 0 |

Raw native stdout/stderr and JSON argv/exit/timing receipts are retained for each.
`test-artifact.json`, `binary-normal.json`, `binary-counted.json` bind emitted
executables/features/hashes/sizes; ordinary selects only default/buffer-study,
counted adds allocation-counts. No legacy study feature or Counter sanity entry
is selected. `output-equivalence.json` binds the six all-size body/document
outputs to accepted YT-150 hashes. No new renderer algorithm is copied or timed.

A manual source edit moved ordinary append-probe branch selection outside its
clock, before either ordinary build/output/workload. The counted test binary had
already compiled. `normal-timer-dispatch-edit.json` and the retained exact
`compiled-test-buffer_study.rs` bind pre/post hashes and prove that the only
changed block is excluded by `cfg(not(feature = "allocation-counts"))`; all counted
exercised sections are identical. The counted binary was used without redundant
compilation/test replay. Both release builds use the final source. One initial
whole-sample format check plus one focused changed-file check are retained rather
than claiming a single final-source check. No native check failed or was repaired.

### Allocation events, growth and temporary overlap

All three operations have 27 events / zero overflow, identical relative globals,
trace and boundary observations. Every event call/request/live/peak increment
reconciles to globals, and every new global held/released field matches every
retained YT-150 current-document operation: **nine pairs, twelve fields at each
point**. `measurement-summary.json` retains every normalized operation; the
original absolute globals/events/boundaries remain in `buffer-counts.stdout`
(JSON lines). The task-root `analyze.py` performs direct retained-data analysis
without invoking workloads.

Global held has 3 allocations / 21 reallocations / 2 frees, 75 alloc-request
bytes, 507,852 / 1,260,387 realloc old/new requested bytes and 262,224 freed
bytes. The returned output holds 490,386 logical bytes; peak is 507,337.
Release gives 3 frees / 752,610 deallocated bytes and zero own-start live.
All failures/zeroed calls are zero. These are observed values, not reduction gates.

| Current call site | Alloc | Realloc | Frees | Alloc request bytes | Realloc old/new bytes | Deallocated bytes |
|---|---:|---:|---:|---:|---:|---:|
| body | 1 | 15 | 0 | 8 | 262,136 / 524,272 | 0 |
| title | 1 | 1 | 0 | 40 | 40 / 80 | 0 |
| envelope | 1 | 5 | 2 | 27 | 245,676 / 736,035 | 262,224 |
| other | 0 | 0 | 1 | 0 | 0 / 0 | 490,386 |

The valid fixed fixture's body events form one 8-byte initial request and fifteen
successive doubling requests from 16 through 262,144 bytes. Title has a 40-byte
initial request and one 40-to-80 growth. Envelope has one 27-byte allocation and
five growth events: **27 → 54 → 134 → 268 → 245,193 → 490,386**. Scope totals are
current call-site accounting, not owners: the 80-byte title temporary and
262,144-byte body temporary are freed in envelope scope, and output is freed in
other scope. The source, lengths/capacities and exact event order establish the
fixed temporary lifetimes without pointer IDs or a generalized owner tracker.

| Boundary | Envelope len | Envelope capacity | Temporary len/capacity | Own-start live | Region peak |
|---|---:|---:|---:|---:|---:|
| initial | 27 | 27 | 0 / 0 | 27 | 27 |
| language | 29 | 54 | 0 / 0 | 54 | 54 |
| head | 134 | 134 | 0 / 0 | 134 | 134 |
| title_ready | 134 | 134 | 68 / 80 | 214 | 214 |
| title_appended | 202 | 268 | 0 / 0 | 268 | 348 |
| body_open | 223 | 268 | 0 / 0 | 268 | 348 |
| body_ready | 223 | 268 | 244,970 / 262,144 | 262,412 | 262,412 |
| body_appended | 245,193 | 245,193 | 0 / 0 | 245,193 | 507,337 |
| closed | 245,207 | 490,386 | 0 / 0 | 490,386 | 507,337 |

The body-ready point has a 262,144-capacity temporary plus the existing
268-capacity envelope. During body append, the envelope grows to 245,193 while
that temporary remains alive, reaching **507,337 bytes**. Its immediately
following free leaves 245,193 held. The **fourteen closing bytes** then grow the
envelope from 245,193 to 490,386, ending with 245,207 output bytes and **245,179
unused capacity bytes**. This late growth explains the large retained spare
capacity; it is not the operation's peak driver. Peak was already reached during
body/envelope overlap. No earlier accepted trace was replayed to locate this.

These are logical request/live/peak observations for one native fixture.
Scope peaks are not summed, and realloc old/new sizes are not physical copy
measurements. Static/stack tracing memory is outside allocator heap accounting;
ordinary timing excludes the allocator/atomic trace entirely. No allocator size
class, RSS, physical traffic or stable String/Vec ABI claim follows.

### Ordinary prepared-body append observations

Each row has twenty measured observations after five warmups. All raw integer
values and reversed orders remain in the timing stdout files; none are excluded.
Body-append ends before closing at 245,193 bytes; other probes end at 245,207.

| Rep | Probe (order) | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---:|---:|---:|---:|---:|
| 1 | envelope (1) | 7,985.15 | 7,698.0 | 8,786 | 6,417 | 12,805 |
| 1 | body_append (2) | 6,063.45 | 6,060.0 | 6,094 | 6,027 | 6,102 |
| 1 | suffix_append (3) | 80.50 | 79.5 | 91 | 53 | 105 |
| 2 | suffix_append (1) | 142.45 | 134.5 | 231 | 75 | 253 |
| 2 | body_append (2) | 11,004.15 | 10,717.0 | 10,798 | 10,625 | 16,529 |
| 2 | envelope (3) | 10,917.30 | 10,666.0 | 11,337 | 10,518 | 11,909 |

Envelope medians are 7.698 / 10.666 microseconds; body append 6.060 / 10.717;
closing append 79.5 / 134.5 nanoseconds. The low closing-probe times do not prove
in-place growth, zero physical copies or a universal cheap-realloc property.
This experiment does not inspect allocator internals or record pointers.
The overlapping/nonadditive observations, including body-append exceeding full
envelope in repetition 2, retain setup/cache/allocator/clock/barrier/compiler
variation rather than assigning CPU shares. Prepared borrowed body lifetime is
different from the actual full operation's short-lived owned temporary. Do not
subtract these probes from retained YT-150 document time or add their medians.
The measured memory-growth issue supplies no demonstrated whole-render CPU gain.

### One minimum future optimization proposal

Propose only a **private adapter tail-capacity reservation experiment**: once the
actual renderer returns the body's known length, reserve enough additional space
for that body **and the closing suffix together**, append the body, explicitly
release its owned temporary at the existing statement-equivalent boundary, then
append the closing suffix. Use existing String operations such as reserve_exact,
without a new public API, renderer copy, second tree traversal or body fusion.
Preserve all validation/errors/escaping/URL/output order and metadata semantics.
No implementation or optimized variant is included in this item.

The intended target is returned-output spare capacity and one late growth,
not a promised CPU speedup or elimination of the body temporary. Source-derived
arithmetic (not a measured candidate) says that if capacity were exactly the
known 245,207 final bytes, spare capacity could fall by 245,179 bytes and the
late doubling could be avoided. However, body plus that envelope would overlap
at 262,144 + 245,207 = **507,351 bytes**, fourteen more than the current observed
507,337 logical peak. The proposal therefore supplies **no peak-memory reduction**
and may slightly increase that peak; changing body lifetime could make it worse.
The independent closing append is already very short in this local probe, while
an explicit reserve call and changed allocator reuse can also cost time.

A separately authorized minimum actual-library comparison must decide whether
retained capacity and elapsed tradeoffs justify adoption, including small/medium/
large outputs and exact lifetime/security checks. Preserve unfavorable results
and reject the candidate if the tradeoff is not useful, as YT-152 demonstrates.
No further optimization or broad architecture change is selected here.

### Controls, failures and cleanup

`environment.json` and `env-*` native receipts capture Rust/Cargo 1.99.0, ext4
build/evidence paths, default release/System allocator and current cgroup limits /
affinity. Selected Rust/Cargo target/build/release overrides, LD_PRELOAD and
MALLOC_CONF are absent. Executables use CPU 0 and 1 GiB address-space caps;
no host control, governor, frequency, quota, tool installation or external
resource changes occur. Shared-host load, allocator internals and physical memory
are not isolated. Current login-session identity is contextual, not a literal
historical equality gate.

The fixed three-process batch takes 31,323,787 ns and retains 28,606 combined
stdout/stderr/receipt bytes, within sixty seconds / 8 MiB. All native processes
are waited, report absent leaders, exit 0 and no timeout. The single stage-2
forty-minute deadline is never renewed. No service, listener, watcher, browser,
container or external resource is started; inert evidence/build/output artifacts
remain under the task root. No old accepted workload/test/build or old Counter
entrypoint is replayed.

No failed/partial native proof, assertion, workload retry or product/harness repair
occurs. The manual ordinary-only source edit and its compile/format provenance
are disclosed above. The intentionally caught tag unwind is a passing test
mechanism. Historical YT-150–152 failures, rejected candidate and review history
remain intact. Parent docs/whitespace checks PASS and independent final Reviewer `/root/yt153_document_buffer_final_review_20261024` PASS is recorded above; the permitted status/reviewer-reference closeout marks Done. No commit/push or product
optimization is authorized.

Parent results checks PASS: `cargo xtask check docs` reports 75 documents / 153
backlog items and valid links/structure/whitespace/DAG; `git diff --check` is
clean. Both new untracked files have empty no-index whitespace diagnostics
(exit 1 denotes added content). `final-review-files.json` freezes the nine
review paths, with source copies and tracked patch; `final-review-data.json`
binds all new native observation stdout, summary and output hashes before final
review. Only the declared status/reviewer-reference closeout may follow PASS.
