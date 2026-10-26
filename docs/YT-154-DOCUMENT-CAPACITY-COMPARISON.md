# YT-154 — Private document tail-capacity comparison

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded implementation/comparison scope after independent final Reviewer PASS.**

## Authority and preserved boundary

The user selects the minimum private body-plus-closing capacity reservation
implementation and comparison across small/medium/large pages, contract first,
unchanged API/output/security and body release timing, no accepted-proof replay,
retained failures, independent Reviewer PASS before Done, and no new commit/push.
YT-153 was separately committed as `6643f4aaf5e8f8f9bc31230015ed49e8a4b80397`
with strict/builtin Guard PASS, zero findings, tree
`95e92bf995649de3861c3e03b3af3a96fa7ab807`; no push. Begin from that clean checkout.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Only the private capacity /
temporary handling inside existing `render_document_with_metadata` may change.
Public signatures, result/error types, title/language handling, HTML validation,
escaping, URL policy, CSP/HTTP behavior, renderer/Core/Vec representation,
browser/default dependency graphs, manifests/locks/dependencies stay unchanged,
apart from one default-off sample measurement feature. No body fusion, copied
renderer, destination-buffer/raw-HTML API, traversal, pool, new crate, alternate
optimization or body lifetime extension.

Reuse [YT-153](YT-153-DOCUMENT-BUFFER-ATTRIBUTION.md),
[YT-150](YT-150-FULL-SSR-COMPARISON.md) and the
[YT-152 rejection](YT-152-FIRST-CHILD-CAPACITY.md) read-only. Existing growth
attribution predicts reduced returned capacity, not CPU/peak improvement; the
large two-buffer peak might increase fourteen bytes. Do not run old accepted
command/test/build populations, Counter calibration, buffer-study probes,
browser/CSP/HTTP/provider/database or other completed verification. New evidence
is under `target/yt154-document-capacity-iwvsg010/evidence`.

## Minimum product candidate

Keep initial prefix/title/body-opening composition and actual body renderer
call order. Bind the body String returned by the same `render_to_string(view)?`
call, reserve enough additional envelope capacity for its known length and the
existing fourteen-byte closing suffix together, append the body, **explicitly
drop it before appending the suffix**, then return the existing String result.
Use standard String capacity checks/reservation, without new errors or API.
There is no new body-render pass, retained-body cache or per-node estimation.

The old borrowed-result temporary is dropped after its append statement. The
explicit drop must preserve that boundary rather than hold the new binding
until function return. Source review is authoritative for before-suffix order;
numeric trace also confirms the internal body-sized free before result-held.
Because the proposed suffix append may not allocate, allocator events alone
cannot mark that instruction or prove its relative drop timing. No observation
hook belongs in product code.

No numerical speedup/reduction gate, adaptive tuning or favorable rerun is
selected. Report every capacity/heap/elapsed tradeoff per size and repetition,
then recommend retaining or rejecting the candidate. A rejected candidate may
restore only this task's product method while preserving its immutable source /
binaries and results, as YT-152 did; no alternate strategy follows. Completion
means a correct bounded implementation/comparison and honest disposition, not
universal performance qualification.

## Actual versions and optional measurement consumer

Create two task-owned immutable `git archive` copies of `6643f4a`, without Git
metadata, checkout, branch or detached worktree changes. Overlay identical final
sample source/manifest/lock bytes on both; only current additionally receives
the candidate adapter lib. Bind archive hashes, original Git blobs/modes (symlinks
by link text), sample/product/lock hashes, feature fingerprints and executable
hashes/sizes. All other product inputs must match between copies. Compile and
call actual product libraries; no source-derived adapter copy is timed here.

Add default-off `document-capacity = ["buffer-study"]` and private sample
`src/document_capacity.rs`. Reuse the accepted finite allocator trace through
that dependency, with counted builds also selecting allocation-counts. The
buffer-study module/Counter and all old commands/tests remain unchanged and are
not executed. Ordinary builds exclude the allocator/atomic tracer. Source labels
reference/current identify the pinned binary; no runtime product-policy switch.

Use unchanged page/metadata/literal fixtures: small 8 rows, medium 128 rows,
large 1,024 rows. Measure these four groups only:

| Group | Actual API / lifetime boundary |
|---|---|
| small | Complete-document API with retained small View/metadata prepared outside; output held at stop, dropped outside timer |
| medium | Same with medium View/metadata |
| large | Same with large View/metadata |
| combined | Large page construction, complete-document API and input destruction; returned output held at stop, dropped outside timer |

Black-box actual inputs/returned Strings. Dispatch, preparation, observation
storage, printing and output length/capacity inspection are outside timers.
Combined directly measures construction/render/input teardown, not a sum of
phase medians. Existing body rendering and body release stay inside the API.

## New focused proof and fixed populations

One new sample `yt154_` test checks actual document/body outputs against the
independent literal oracle at all three sizes, title/language escaping and
finite nested unsafe-URL/forbidden-attribute/invalid-void-child errors. Compile
once in the current counted source copy, offline locked `--no-run
--message-format=json`; run only the emitted `yt154_ --test-threads=1` filter.
All old tests, including the two buffer-study tests, remain filtered. New
`document-capacity-output reference|current EXISTING_DIRECTORY` runs once per
ordinary binary, writes six body/document outputs each, checks literals and all
twelve accepted YT-150 output hashes. These bind the changed actual libraries.
Do not assert an invented capacity target in tests.

Build four new release executables: reference/current ordinary and counted.
New `document-capacity-timing reference|current 1|2` runs four fresh processes
in order reference/1, current/1, current/2, reference/2. Group order is
small/medium/large/combined in repetition 1, reversed in 2. Each group uses five
warmups and twenty measurements: **80 warmups / 320 measured operations**.
Retain variant/repetition/group/order/population/index, integer ns/op and returned
length/capacity. Report count, mean, median (average entries 10/11), nearest-rank
p95 (entry 19/20), min/max and paired same-group/repetition differences. Preserve
variation, no pooling-only result, outlier rejection or universal speed claim.

New `document-capacity-counts reference|current small|medium|large|combined`
runs eight fresh counted processes, **three operations each / 24 total**, no
warmups. Prepare the fixture and observe one actual body's length/capacity
outside measurement. Retain global start/held/released, returned length/capacity,
all twelve relative fields, failures and within-three-operation stability;
require release to own-start live. Print raw results before assertions.

For document groups only, begin/finish the existing fixed 64-event trace around
the actual API and output release. It observes alloc/realloc/free requests and
global live/peak under **other** call-site scope, since product code has no tags.
Require event/global byte/live/peak reconciliation, zero failures/overflow and
source-consistent internal temporary/output releases. It is not pointer/owner
tracking. Combined explicitly leaves tracing disabled: its construction events
would exceed the fixed bound, and global snapshots provide its required heap
evidence. Do not expand or change the existing trace to cover construction.

Reference large-document held/released must match every retained YT-153 operation
across all twelve fields (nine pairs); reference combined likewise matches
YT-150 current combined raw evidence. New small/medium counted values have no
claimed identical historical counterpart. A mismatch stops causal interpretation
without retuning/replay. Logical requests/live/peak are not physical copies,
allocator overhead, size classes or RSS. No stable page speed, HTTP capacity,
browser paint/memory or production SLO follows from short native synthetic runs.

## Bounded delivery, evidence and review

Exactly nine repository paths may change:

```text
docs/YT-154-DOCUMENT-CAPACITY-COMPARISON.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
crates/yetlet-server-axum/src/lib.rs
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/document_capacity.rs (new)
```

Register YT-154 within M23; move the verbatim YT-153 row from after the update
rules beside its M23 predecessor, preserving its text/status/review history.
No Roadmap redesign or release-gate expansion follows.

Stage 1 changes only four docs, checks them and obtains independent contract
review. Stage 2 starts a fixed **45-minute** UTC/monotonic deadline before edits,
never renewed. Use Rust/Cargo 1.99.0, offline locked default release/System
allocator, two build jobs, ext4 source/build/evidence paths and unchanged selected
flags/host controls. Record current cgroup identity/effective limits/affinity;
session identity is context, not a literal historical-equality gate. No tool
installation, governor/quota/LTO/PGO/environment change or external resource.
Native executables use CPU 0 / 1 GiB address space. Build/test commands have
600-second caps, executable output/timing/count commands fifteen seconds, within
the fixed deadline. The twelve-process measurement batch has sixty seconds /
8 MiB combined stdout/stderr/receipt limits.

Use only the existing finite command-receipt pattern; no new runner/validator /
profiling framework or infrastructure tests. Persist argv, UTC/monotonic bounds,
native exits/timeouts, raw stdout/stderr, source/lock/binary hashes/features/sizes,
and cleanup immediately. Run one targeted changed-Rust format check. Preserve
failed/partial/unknown results; classify under materiality before the smallest
needed correction. No automatic retry, workload expansion, new task/clock or
passed-proof replay. Wait/join owned foreground commands, terminate only owned
children on timeout; start no service/browser/container. Retain inert snapshots /
binaries/raw artifacts and confirm no owned process remains.

Check unchanged protected sources/evidence and exact final footprint; canonical
docs/whitespace checks and independent final Reviewer PASS with zero required
followups precede Done. Only status/reviewer references in four docs and sample
README, then docs/whitespace checks, may follow PASS. Product/sample Rust/Cargo,
observations and binaries stay frozen. No new commit/push, subsequent optimization
or YT-096/YT-020 release decision is authorized.

## Execution and measured disposition

Independent contract Reviewer
`/root/yt154_document_capacity_contract_review_20261024` returned **PASS**, zero
issues and required followups, before product/sample edits. Four document hashes
match; the YT-153 row and its review/status text remain verbatim after the
recorded move into M23. Contract results, native role/model/effort observation and
failure/correction receipts remain in `contract-review-*.json` and
`backlog-row-move.json`. Independent final Reviewer `/root/yt154_document_capacity_final_review_20261024` returned **PASS**, zero issues and required followups. All nine reviewed file hashes, 32 data bindings, eight historical bindings and five executable hashes/sizes match. `final-review-result.json` retains acceptance and disclosed tradeoffs; the item is Done at bounded implementation/comparison scope.

**Recommendation: retain this minimum candidate in the uncommitted checkout as
a returned-capacity improvement, with size-specific time limits disclosed.**
All three sizes retain approximately half the output capacity and one fewer
reallocation, with identical output/error behavior and the original body release
boundary. All four groups have a fourteen-byte higher logical peak. Large
render/combined medians are lower in both repetitions; small/medium medians are
higher in both, with large first-repetition variation. Do not describe this as
universal acceleration. `retention-decision.json` records the disposition; there
is no additional tuning, repeat population, new strategy, commit or push.

### Actual implementation and proof

Only the existing adapter method's two body/closing append statements change:

```rust
let body = render_to_string(view)?;
let suffix = "</body></html>";
html.reserve_exact(body.len() + suffix.len());
html.push_str(&body);
drop(body);
html.push_str(suffix);
```

The actual renderer is called once in the original position; body is explicitly
dropped after append and before suffix. No public signature/error type or
renderer/metadata/security behavior changes. `source-review.json` confirms that
replacing this exact block with the original two statements reproduces the
entire baseline adapter lib byte-for-byte; every protected source, old Counter,
reused trace and original fixture/literal oracle stays unchanged. It also
separates source-authoritative before-suffix drop order from trace evidence of
internal release before result-held. No product observation hook is introduced.

`source-snapshots.json` binds the two git archives, original blob/mode checks
(including license symlink link text), identical sample overlays and every
product hash. Only the current adapter lib differs. Four
`binary-{variant}-{normal,counted}.json` records bind emitted sample and three
actual library artifacts/manifests, source/features/executable hashes/sizes.
Ordinary selects default/buffer-study/document-capacity; counted also selects
allocation-counts. The old study supplies unchanged trace code; its commands and
tests are not replayed. Both versions call real libraries, not a copied adapter.
`measurement-source-bindings.json` freezes current product/sample/lock inputs.

| New proof / receipt stem | Result |
|---|---|
| `format-write-sample`, `format-write-adapter`, `format-check` | Two formatting writes and one changed-Rust check, all exit 0 |
| `compile-new-test`, `new-test` | One offline locked counted current-archive test compilation; emitted `yt154_ --test-threads=1`: 1 passed, 4 old tests filtered, exit 0 |
| `build-{reference,current}-{normal,counted}` | Four fresh offline locked default-release builds, jobs 2, exit 0 |
| `output-reference`, `output-current` | Twelve actual body/document literal/hash outputs match accepted YT-150 bytes at all three sizes, exit 0 |
| `timing-{variant}-{repetition}` | Four new native processes: 320 measured times / 80 warmups, all exit 0 |
| `counts-{variant}-{group}` | Eight new native processes: 24 counted operations, all exit 0 |

Native stdout/stderr and exact argv/exits/UTC/monotonic/resource receipts are
retained. The test and both current release builds use the same frozen final
source; no postcompilation Rust edit occurred. `test-artifact.json` and
`output-equivalence.json` retain test executable and twelve output bindings.

### Returned capacity and logical heap

Every variant/group has three identical balanced count observations, zero
failed/zeroed calls, zero trace overflow and own-start live restored on release.
Every traced document event/request/live/peak reconciles to global snapshots;
its output capacity equals held logical bytes. All reference-large / YT-153 and
reference-combined / YT-150 pairs match all twelve held/released relative fields
(nine pairs per group). New small/medium logical results have no substituted
historical acceptance claim. `measurement-summary.json` retains all fields and
operations; the task-root `analyze.py` reads raw JSON-lines/CSV without invoking
workloads.

| Group | Output bytes | Reference returned capacity | Current capacity | Capacity change | Logical peak change | Realloc change |
|---|---:|---:|---:|---:|---:|---:|
| small | 2,383 | 4,738 | 2,383 | -2,355 (-49.70%) | +14 | -1 |
| medium | 31,063 | 62,098 | 31,063 | -31,035 (-49.98%) | +14 | -1 |
| large | 245,207 | 490,386 | 245,207 | -245,179 (-50.00%) | +14 | -1 |
| combined | 245,207 | 490,386 | 245,207 | -245,179 (-50.00%) | +14 | -1 |

| Variant/group | Alloc | Realloc | Alloc request bytes | Realloc old/new bytes | Held live bytes | Region peak bytes |
|---|---:|---:|---:|---:|---:|---:|
| reference-small | 3 | 15 | 75 | 6,980 / 15,819 | 4,738 | 6,465 |
| reference-medium | 3 | 18 | 75 | 64,332 / 159,203 | 62,098 | 63,817 |
| reference-large | 3 | 21 | 75 | 507,852 / 1,260,387 | 490,386 | 507,337 |
| reference-combined | 12,308 | 21 | 1,641,741 | 507,852 / 1,260,387 | 490,386 | 2,149,003 |
| current-small | 3 | 14 | 75 | 4,611 / 11,095 | 2,383 | 6,479 |
| current-medium | 3 | 17 | 75 | 33,283 / 97,119 | 31,063 | 63,831 |
| current-large | 3 | 20 | 75 | 262,659 / 770,015 | 245,207 | 507,351 |
| current-combined | 12,308 | 20 | 1,641,741 | 262,659 / 770,015 | 245,207 | 2,149,017 |

Allocation calls remain three for document and 12,308 for large combined; one
reallocation is removed in every group. Held frees remain two for document and
12,307 for combined; all input/body/title bytes are released at the same phase
boundary. Final release gives three / 12,308 total frees respectively and zero
own-start live. Full released request/free fields remain in raw data/summary.

The large document overlap is now 262,144 body capacity plus 245,207 envelope
capacity = **507,351 bytes**, versus 507,337 reference. Large combined similarly
has 2,149,017 versus 2,149,003 peak. Saving 245,179 returned-capacity bytes thus
does not reduce peak; the measured fourteen-byte increase matches the conditional
YT-153 arithmetic. String capacity observations are private target-specific
results, not new API guarantees or allocator usable-size/RSS/physical-copy proof.

### Body release and unchanged finite trace

Reference/current event counts are small 21/20, medium 24/23 and large 27/26,
all below the unchanged 64-event limit. Actual API calls have other-scope tags;
no pointer identities or fabricated owner buckets are recorded. Setup observes
actual body length/capacity outside each count region. Both variants show the
same internal body-sized free before result-held: 4,096 / 32,768 / 262,144 bytes,
preceded by the title-sized free; the final free matches returned capacity.
Source and sizes/order consistently identify these temporary lifetimes for this
fixture. Source's explicit `drop(body)` proves before-suffix order, because a
nonallocating suffix has no allocator event marker. No claim of instruction-level
release timing is inferred from trace alone. Combined records no events by
contract; its global heap observations remain complete.

### Ordinary actual-library elapsed observations

Each row has twenty measurements after five warmups; raw order, warmups and
returned length/capacity remain in stdout. All rows are retained. Group order
and variant order reverse as contracted. Times are native actual complete APIs,
not isolated copy-probe subtraction or added phase medians.

| Variant | Rep | Group | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---|---:|---:|---:|---:|---:|
| reference | 1 | small | 6,009.45 | 5,905.5 | 6,338 | 5,819 | 7,343 |
| reference | 1 | medium | 83,751.25 | 83,946.5 | 89,107 | 76,026 | 92,453 |
| reference | 1 | large | 464,442.80 | 424,171.0 | 602,899 | 408,487 | 707,410 |
| reference | 1 | combined | 746,842.75 | 747,083.5 | 760,371 | 735,917 | 761,566 |
| reference | 2 | combined | 770,400.40 | 736,778.0 | 947,705 | 727,616 | 1,047,526 |
| reference | 2 | large | 412,920.20 | 412,289.5 | 419,798 | 405,685 | 427,311 |
| reference | 2 | medium | 43,838.95 | 43,563.5 | 44,429 | 43,230 | 47,531 |
| reference | 2 | small | 3,477.20 | 3,470.0 | 3,510 | 3,440 | 3,541 |
| current | 1 | small | 12,089.50 | 11,749.5 | 12,231 | 11,666 | 17,754 |
| current | 1 | medium | 123,902.00 | 123,106.0 | 138,218 | 113,647 | 138,677 |
| current | 1 | large | 396,145.40 | 357,083.5 | 540,225 | 344,019 | 540,703 |
| current | 1 | combined | 663,703.65 | 660,900.0 | 680,073 | 649,150 | 682,051 |
| current | 2 | combined | 702,005.80 | 669,706.5 | 871,754 | 655,406 | 970,230 |
| current | 2 | large | 348,714.00 | 348,119.0 | 355,980 | 344,077 | 358,463 |
| current | 2 | medium | 44,920.55 | 45,098.5 | 45,666 | 43,831 | 46,470 |
| current | 2 | small | 3,548.35 | 3,552.5 | 3,588 | 3,495 | 3,604 |

Paired median changes use the same repetition/group reference denominator:

| Rep | Small document | Medium document | Large document | Large combined |
|---|---:|---:|---:|---:|
| 1 | +98.96% | +46.65% | -15.82% | -11.54% |
| 2 | +2.38% | +3.52% | -15.56% | -9.10% |

Small/medium negative results are not discarded: small is 5.9055 → 11.7495
microseconds in repetition 1 and 3.4700 → 3.5525 in 2; medium is 83.9465 →
123.1060 and 43.5635 → 45.0985. These do not establish a small/medium speedup.
Substantial between-repetition differences remain, with cause unmeasured; do
not attribute them to a measured frequency/cache change or replace the first
population. The approximately 2.38% / 3.52% second-repetition increases also stay
visible as an unresolved size-specific performance limitation.

Large document medians decrease 15.82% / 15.56%, large combined 11.54% / 9.10%
in this pair. These descriptive local observations justify no stable universal
speed/regression percentage, statistical significance, production req/s/SLO,
browser paint/memory or concurrent HTTP qualification. Retaining the minimum
candidate targets demonstrated returned-capacity reduction; it is not acceptance
of universal acceleration. No numeric performance threshold was retrofitted,
and no extra repetition, tuning or alternate implementation follows.

### Failure history, resource context and cleanup

The first contract-stage `git diff --check` failed exit 2 for one redundant blank
line at Backlog EOF after moving the unchanged YT-153 row. Original
`contract-whitespace.stdout/.json` and `contract-whitespace-correction.json`
retain that failure and before/after hashes. Removing only that blank line and
rerunning only the failed whitespace check passed (`contract-whitespace-corrected`).
The already-passed initial cargo docs check was not replayed then; its semantic
links/items were unchanged. Contract Reviewer inspected this boundary before
implementation. No runtime/build/test/output/workload failure or repair occurred;
negative time/peak observations remain unfiltered results, not erased failures.

`environment.json` / `env-*` receipts record Rust/Cargo 1.99.0, ext4 evidence /
source/build paths, absent selected Rust/Cargo target/build/release overrides,
LD_PRELOAD and MALLOC_CONF. Default release/System allocator, two build jobs and
CPU 0 / 1 GiB native bounds are unchanged. Current session-161/user ancestor
cgroup maxima are unlimited; identity is context, not a historical literal gate.
No host/governor/frequency/quota/tool/allocator settings change. Shared-host load,
allocator internals and physical memory are not isolated; source paths/compiler
layout can affect timing despite identical consumer inputs. Tracing is counted
only; its fixed stack/static memory is outside logical heap accounting.

The twelve-process batch completes in 324,143,241 ns, retaining 141,048 combined
stdout/stderr/receipt bytes, within sixty seconds / 8 MiB. All owned commands
are waited and record absent leaders; no timeout, server, listener, browser,
watcher, container or external resource occurs. Inert archives/binaries/outputs /
raw results remain in the task root. The single forty-five-minute stage-2 clock
is never renewed. Protected sources, old trace/Counter/commands and eight
historical bindings remain unchanged. Parent docs/whitespace checks PASS and independent final Reviewer `/root/yt154_document_capacity_final_review_20261024` PASS is recorded above; the permitted status/reviewer-reference closeout marks Done. No new commit/push, subsequent optimization
or release-gate expansion is authorized.

Parent results checks PASS: `cargo xtask check docs` reports 75 documents / 154
backlog items, valid links/structure/whitespace/DAG; tracked and both new-file
whitespace checks have empty diagnostics. No-index exit 1 denotes added content.
`final-review-files.json` freezes all nine review paths, with source copies /
tracked patch; `final-review-data.json` freezes raw observations, summary, output
hashes, retention/source records and direct analysis before independent final
review. Only the declared five-doc status/reviewer-reference closeout may follow
PASS; all Rust/Cargo/product/data/binaries stay frozen.
