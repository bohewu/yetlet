# YT-151 — View and metadata construction cost attribution

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded attribution/proposal scope after independent final Reviewer PASS.**

## Authority and preserved boundary

The user selected contract-first attribution of View/metadata construction,
reusing [YT-150](YT-150-FULL-SSR-COMPARISON.md), minimum new measurements,
distinguishing fixture formatting, String/Vec and Core node costs, preserving
existing APIs, and one minimum optimization proposal after evidence. Do not
implement an optimization. Retain failures, do not replay accepted verification,
and require independent Reviewer PASS before Done. No commit/push, external
resource or deployment is authorized. Baseline is clean
`4d9740cf670da1617b3b0a8d8dc54d70d1c348e0` in the current checkout.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md), and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). All product crates, public
APIs, representation, security/error behavior, browser/Core/adapter/default
graphs and root/sample locks/dependencies stay unchanged. No SmallVec/Cow,
capacity policy, template, arena, pool, new crate, generic profiler, validator
framework or new runtime is selected. Measurement is an opt-in extension of
`examples/ssr-performance-baselines` only.

YT-150's current construct population has 12,305 alloc / zero realloc calls,
1,641,666 held/peak logical bytes and balanced releases. Its actual raw snapshots
in `target/yt150-full-ssr-ds4q4056/evidence/counts-current-construct.stdout`,
source/feature bindings, timing observations, oracles and failures remain
read-only. Do not replay old tests/builds/commands, page timing/count groups,
HTTP/browser/CSP/provider workloads or old counter sanity checks. Historical
whole-construction time is context, not an additive CPU-share denominator.
New evidence belongs under `target/yt151-view-attribution-nlh1nqql/evidence`.

## Source-derived builder using real APIs

Add default-off `construction-study = []` and private
`src/construction_study.rs`. Its source-derived `row`/`page` preserve the existing
fixture's exact formats, values, metadata, evaluation order and iterator shape.
They call real `Element`, `View`, `DocumentMetadata` and `LanguageTag` APIs; do
not copy their implementation or alter product source for probes.

Use small lexical wrappers to stage `Into<String>` and `Into<View>` before the
actual insertion call. Original Core methods perform those conversions before
Vec push; owned String/View re-entry then moves without allocation. Keep
`children((0..rows).map(row))`, with nested row scopes overriding the parent
children-Vec scope while callbacks execute. No reserve, clone, reordering of
formats, changed payload lifetime or alternate node representation is allowed
in the attributed full builder. Source review must bind these transformations
to the original sample and Core method bodies; allocation reconciliation and
structural equality below are also required before interpretation.

Use seven disjoint **creation call-site** buckets:

| Bucket | Captured work |
|---|---|
| format | The exact five row `format!` calls, including their String allocation/growth |
| string | Other `Into<String>` ownership conversions for text/attribute values; moving an already-owned formatted String adds no allocation |
| attribute_vec | Actual Core attribute insertion after String conversion |
| children_vec | Actual Core child insertion/extension after node conversion; includes storage for inline View values |
| metadata | Actual title/LanguageTag construction and validation, including its owned Strings |
| node | Actual Element initialization and conversion to inline View variants, after String work is staged |
| other | Any creation event outside those scopes; retain rather than hiding it |

Formatted String buffers belong to format and metadata Strings to metadata,
not also to string: these are exclusive stages, not overlapping type totals.
Core's enum/struct nodes are inline source values; their Vec slot storage is
charged to the appropriate Vec bucket. A zero node allocation observation does
not mean zero node CPU/move/storage cost or zero Core cost.

Extend only the existing sample `Counter` alloc/alloc_zeroed/realloc hooks with
feature-gated calls to a new fixed seven-bucket module in construction_study.
Reuse the existing nonallocating atomic/lexical-restore pattern, without
refactoring existing serializer tags or changing their fields/behavior. Both
features may compile together; selected study builds enable no old study feature.
Tagging is explicitly single-threaded, counts only creation events, and records
alloc, zeroed subtotal, realloc, alloc request bytes, realloc old/new bytes and
failures. No allocation IDs, per-bucket frees, ownership, live/peak or physical
copy tracking is introduced. Restore parent tags on return/unwind; only the
outer operation calls the existing global `counts::begin()`.

## Fixed counted population and structural inventory

New `construction-counts` runs one fresh counted foreground process with **three
operations, no warmups**, on the existing 1,024-row page. Capture global
start/held/released snapshots and bucket start/held snapshots in preallocated
stack records. Hold metadata/View at held; drop both before released. Print raw
global and bucket observations only after capture, retaining all global fields.
Require balanced own-start release and zero failures, identical observations
across the three operations, and each bucket sum equal to the corresponding
global creation field. Do not invent a reduction target or require other/node
to be zero; unclassified events remain explicit.

Every new global held/released relative field must reconcile to every retained
YT-150 current construct operation (all nine pairs, twelve fields at each
point), before assigning the accepted construction cost to these buckets.
A mismatch stops causal interpretation: preserve it without retuning or replay.
This is new attribution of the same population, not re-execution of its old
acceptance command. Global heap is not a sum of bucket peaks; request bytes are
not copies, allocator overhead or RSS.

Outside measurement, inventory the resulting real View through public fields/
slices: element/text/attribute/edge counts, child-count histogram, public String
length/capacity totals, and target-specific `size_of` for View/Element/Attribute/
metadata. Do not add a capacity accessor or inspect private Vec fields. Vec
request bytes can explain slot storage using actual source types, but do not
claim a per-node capacity trace or stable ABI. Metadata getters expose lengths,
not private String capacity; its event bucket supplies allocation context.

## Separate ordinary component probes

Build ordinary `construction-study` separately from counted
`construction-study,allocation-counts`. Neither selects old attribution/page/URL
features. Ordinary must exclude the allocator wrapper; counted rejects timing.
No instrumented elapsed result is ordinary time.

New `construction-timing 1|2` runs two fresh ordinary processes. Repetition 1
visits format then assembly; repetition 2 reverses that order. Each group has
five warmups and twenty observations: **20 warmups and 80 measured operations**.
Preallocate records and retain integer ns/op, repetition/probe/order/population/
index/result size, all rows, count/mean/median/min/max/nearest-rank p95 per group.
Median averages sorted entries 10/11; p95 is entry 19/20. No outlier exclusion,
pooled-only result, significance or numerical timing gate is selected.

- Format probe calls the identical five formatting helpers for each of 1,024
  indices, black-boxes the resulting five owned Strings and destroys them before
  advancing to the next row. Formatting, String creation/destruction, loop and
  barriers are inside; no View is built. This differs from full construction,
  which retains the Strings and interleaves Core work, so no additive share follows.
- Assembly probe prepares one `Vec<Option<RowPayload>>` per operation before the
  timer, using those exact formatted Strings/capacities. Keep the container alive
  outside the timer. Inside, use Range-map callbacks to take/move each payload at
  its original insertion position into a real View, preserving the outer
  children-iterator allocation shape. Static String conversions, metadata, Core
  Vec work and inline node operations are inside. Stop with metadata/View held,
  then drop them and the empty payload container outside. Payload selection/
  bookkeeping/barriers are included: this is a prepared-input assembly probe,
  not pure Core CPU or a before/after optimization.

Do not subtract these probes from YT-150 time, add them into full construction,
or assign CPU percentages to the allocator buckets. They separate observed
formatting and prepared assembly boundaries; exact String-versus-Vec-versus-node
CPU shares remain unmeasured. Disclose cache/lifetime/compiler-placement limits.

## New checks, footprint and bounds

Compile new `yt151_` tests once in the counted configuration with offline locked
`--no-run --message-format=json`, then run only that emitted binary's filter with
`--test-threads=1`. Two focused tests cover structural metadata/View equality
against unchanged page at 8/128/1,024 rows, prepared assembly equivalence, and
nested tag restoration/actual allocator-hook dispatch. No old test entry point
or sanity suite is rerun. New `construction-output EXISTING_DIRECTORY` verifies
both structural equality and independent literal body/document output for all
three sizes, writes six new outputs, and compares their accepted YT-150/YT-147
hashes outside measurement. It also reports the large-tree inventory.

Exactly nine tracked paths may change:

```text
docs/YT-151-VIEW-CONSTRUCTION-ATTRIBUTION.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/counts.rs
examples/ssr-performance-baselines/src/construction_study.rs (new)
```

Stage 1 changes only four docs, checks them and obtains independent contract
review before implementation. Stage 2 records a fixed **35-minute** UTC/monotonic
deadline before edits; never renew it. Use separate normal/counted build targets
on ext4, Rust/Cargo 1.99.0, default release/System allocator, two build jobs,
offline locked inputs and unchanged selected flags/host controls. Executables
use CPU 0 and 1 GiB address space. Record current cgroup identity and effective
limits; new login-session IDs are context, while both new probes share the same
current resource controls. Do not install tools or alter host settings.

One format check, one counted test compilation/filter and two fresh release
builds precede the new output command. Build/test commands have 600-second caps
within the fixed deadline; executable output/timing/count commands have 15-second
caps. The **three-process measurement batch** (timing1, timing2, counts) has a
60-second monotonic ceiling and 8 MiB combined stdout/stderr/receipt budget.
Persist exact native argv/exits/timeouts/UTC/monotonic boundaries and raw results
immediately; bind source/contract/lock/binary hashes/sizes/features before analysis.
Retain failed/partial/unknown results, no retry, replacement, tuning or new run.
Classify a failure under the materiality policy before the smallest correction;
never rerun already-passed checks merely to recreate artifacts.

Wait/join all owned foreground commands, stop only owned children on timeout and
record absence/cleanup. Start no server/browser/watcher/service/container. Keep
inert artifacts under the task evidence root. Product/history hashes and default
graph remain unchanged. After evidence, identify the call/request-volume hotspot
and propose **one** separately authorized minimal optimization with explicit
tradeoffs, or explain why no candidate is justified. No optimization code or
representation/API change belongs here. Parent docs/whitespace checks and final
independent Reviewer PASS with zero required followups precede Done. Only
status/reviewer-reference changes in four docs and sample README, followed by
docs/whitespace, may follow PASS; Rust/Cargo/data/binaries stay frozen. No next
item, commit, push or release-gate change is automatically authorized.

## Accepted contract and new evidence

Independent contract Reviewer
`/root/yt151_construction_contract_review_20261024` returned **PASS**, zero issues
and required followups, before any sample implementation. Its four reviewed
hashes and enforced native role/model/effort observation are retained in
`contract-review-result.json`, `contract-review-files.json`,
`contract-review-trace.json` and `contract-review-observed-decision.json`.
Independent final Reviewer `/root/yt151_view_attribution_final_review_20261024` returned **PASS**, zero issues and required followups. All nine reviewed hashes match `final-review-files.json`; `final-review-result.json` retains acceptance and scope limits. The item is now Done at bounded attribution/proposal scope.

All following paths are relative to the evidence directory above. Raw native
stdout/stderr and JSON receipts are retained separately for each command.

| New check / observation | Result and receipt stem |
|---|---|
| Sample formatting | One write and one check, exit 0: `format-write`, `format-check` |
| New counted test compilation | Offline locked `cargo test --features construction-study,allocation-counts --no-run --message-format=json --jobs 2`, exit 0: `compile-new-tests` |
| New emitted test binary | `yt151_ --test-threads=1`: 2 passed, 0 failed, 2 old entrypoints filtered: `new-tests` |
| Ordinary release | Offline locked `cargo build --release --features construction-study --jobs 2 --message-format=json`, exit 0: `build-normal` |
| Counted release | Same with `construction-study,allocation-counts`, exit 0: `build-counted` |
| New all-size output/inventory | `construction-output`, exit 0: `new-output`; six literal/hash outputs match accepted YT-150 outputs |
| Fixed ordinary population | `construction-timing 1`, then `2`, both exit 0: `timing-1`, `timing-2`; 80 measured operations and 20 warmups |
| Fixed counted population | `construction-counts`, exit 0: `construction-counts`; three full-builder operations |

`test-artifact.json`, `binary-normal.json` and `binary-counted.json` bind emitted
features, executable hashes/sizes and compilation artifacts. Ordinary selects
only default/construction-study; counted adds allocation-counts, no old study
feature. The source and lock hashes in `measurement-source-bindings.json` are
frozen before output/measurement. `source-review.json` confirms the original
fixture constants/builders/literal oracle are byte-identical, the old Counter
source is identical after removing exactly six new cfg-gated hook calls, and all
tracked paths outside the nine permitted paths retain baseline hashes.
`reused-evidence.json` binds the six retained YT-150 inputs read-only.

### Exclusive creation observations

All three new operations are identical. Global held/released relative values
match **all nine new/historical pairs, all twelve fields at both points** from
the actual retained YT-150 raw construct output. There are 12,305 allocations,
zero zeroed/realloc/failed calls, 1,641,666 request/held/peak bytes, no held frees,
then 12,305 frees / 1,641,666 deallocated bytes and zero own-start live increment
after release. Every creation field also equals its seven bucket sums. The
raw global startup baseline is 679 live bytes; it is subtracted rather than
misreported as fixture memory. `measurement-summary.json` retains every relative
operation, bucket and descriptive statistic; `analyze.py` in the task root
records the direct CSV analysis, without re-executing workloads.

| Creation call site | Alloc calls | Alloc request bytes | Call share | Request-byte share |
|---|---:|---:|---:|---:|
| format | 5,120 | 186,368 | 41.6091% | 11.3524% |
| string | 1,029 | 11,352 | 8.3625% | 0.6915% |
| attribute_vec | 2,050 | 328,000 | 16.6599% | 19.9797% |
| children_vec | 4,104 | 1,115,904 | 33.3523% | 67.9739% |
| metadata | 2 | 42 | 0.0163% | 0.0026% |
| node | 0 | 0 | 0.0000% | 0.0000% |
| other | 0 | 0 | 0.0000% | 0.0000% |

Zero node/other events were observed, not required by a numerical gate. Element
construction starts with empty Vec values; View/Element values are inline. Node
move/initialization work remains real CPU work and node slot storage is already
included in children_vec. Metadata is only two allocations / 42 bytes on this
fixture; it is not a justified priority. Formatting is the largest call-count
bucket, but is fixture-owned `format!` work rather than a serializer/Core defect.
The combined Vec buckets account for 87.9536% of requested bytes, without
establishing CPU cost or copies.

The large-tree public inventory has 4,104 Elements, 3,075 Text values, 3,074
attributes and 7,178 child edges. Elements by child count are 0 / 3,077 / 1 /
1,025 / 1 for zero/one/two/three/four-plus children; the maximum is 1,024.
Thus 74.9756% of Elements have one child. Public text/attribute Strings total
120,920 length bytes and 197,720 capacity bytes; the latter equals the format
plus other-string request bytes here. The 76,800-byte gap belongs to this
fixture's formatting buffers, not metadata or a measured copy cost.
Target-specific sizes are View 64, Element 64, Attribute 40 and metadata 48 bytes.

The children_vec request total divided by this target's 64-byte View is 17,436
slots across all creation requests, versus 7,178 child edges. There are no
reallocations/frees at held, so these observed aggregate requests are consistent
with retained slot storage and spare space. This does not inspect private
per-Vec capacities or establish their distribution/ABI. Attribute requests
similarly include inline Attribute slots and must not be counted again as nodes.

### Ordinary component observations

Each table row is twenty measured observations after five warmups. The order
is reversed in repetition 2. Raw integer ns values, including every warmup,
remain in the two timing stdout files; no values were excluded.

| Rep | Probe (order) | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---:|---:|---:|---:|---:|
| 1 | format (1) | 132,036.30 | 131,583.0 | 135,573 | 130,054 | 138,384 |
| 1 | assembly (2) | 118,687.60 | 118,580.0 | 120,855 | 116,454 | 123,638 |
| 2 | assembly (1) | 118,430.20 | 117,898.0 | 125,552 | 114,949 | 126,946 |
| 2 | format (2) | 132,074.40 | 131,509.5 | 136,791 | 130,621 | 137,426 |

Format returns 109,568 formatted payload bytes per operation; assembly returns
7,179 metadata-held tree nodes (the result count is tree nodes, not metadata
objects). Approximately 132 microseconds for formatting/destruction and 118
microseconds for prepared assembly are separate local boundary observations.
They cannot be added/subtracted from the retained YT-150 construction medians
208.5275 / 206.9655 microseconds, or treated as exact String/Vec/node CPU shares.
Preparation, allocation reuse, different String lifetimes, barriers/bookkeeping,
source-derived call placement and compilation can alter behavior. The probes
show that fixture formatting is material and assembly still costs time; they
are not an optimized-vs-original comparison or stable production throughput.

### One minimum future proposal

Prioritize a **private first-child allocation-capacity experiment in
`Element::child`**, preserving Vec/View representation and all public APIs,
output, security and insertion order. Explore requesting one slot when inserting
into an empty children Vec, then retain normal Vec growth for subsequent children.
The motivation is the 67.9739% children-Vec request share and 3,077 one-child
Elements, rather than a claim that Core CPU is 68% or nodes are free. Keep the
bulk `children` Range-map/extend path unchanged.

This may reduce spare requested storage for one-child Elements while retaining
one allocation each. Conversely, the 1,026 two/three-child Elements could require
an extra growth/reallocation and move initialized View values; allocator size
classes, branch cost, total construction time and peaks could worsen. No actual
saving or speedup is accepted here. A separately authorized bounded comparison
would need both one-child and multi-child shapes plus the retained full fixture,
actual output/semantics checks and ordinary/global-allocation evidence. Reject
the change if the tradeoff is unfavorable. No capacity-policy code, SmallVec,
public builder hint, template, arena, dependency or alternative proposal is
implemented or selected in this item. Fixture formatting remains separately
identified application work; changing it is outside this framework proposal.

### Resource context, failures and cleanup

Rust/Cargo 1.99.0, ext4 evidence/build targets and unchanged default release/System
allocator were recorded in `environment.json` and `env-*` raw receipts. Selected
RUSTFLAGS, Cargo target/build flags, release LTO/codegen/optimization overrides,
LD_PRELOAD and MALLOC_CONF are absent. Current session-161 and its recorded user
ancestor cgroups have unlimited CPU/memory maxima; executables are nevertheless
pinned to CPU 0 with 1 GiB address-space caps. No host governor, CPU frequency,
background load, allocator internals or physical-memory isolation claim follows.
Both new probes share the same captured current controls. Session identifiers
are context, not a literal equality gate against historical sessions.

The fixed measurement batch completes in 38,656,316 ns, retains 7,439 combined
stdout/stderr/receipt bytes, and starts exactly three foreground workload
processes; `measurement-batch.json` records the 60-second/8-MiB bounds. All native
commands are waited and report leader absence, zero exits and no timeout. No
server, listener, container, provider or external resource is started; inert
artifacts remain in the task root. Stage 2's fixed unrenewed 35-minute clock
remains in `stage2-clock.json`.

There are no failed/partial native commands, assertion failures, workload retries
or product/harness repairs in this task at this point. The nested-restore test
intentionally catches an unwind; it passes and is not a failed verification.
Historical YT-150 preparation failures and accepted proofs remain unchanged.
No old accepted test/workload, browser/CSP/HTTP/Windows/Valkey/IdP check or counter
sanity command was replayed. Product source, APIs, default features, dependencies
and locks remain unchanged. Final independent Reviewer PASS and the permitted status/reference closeout are recorded above;
no commit/push or optimization implementation is authorized.

Parent results checks PASS: `cargo xtask check docs` reports 75 documents / 151
backlog items, valid links/structure/whitespace/DAG; `git diff --check` is clean.
Both new untracked files have empty `git diff --no-index --check` diagnostics
(exit 1 means added content), recorded in `results-new-doc.json` and
`results-new-rust.json`. These checks concern new changed content, not old
accepted runtime proof. `final-review-files.json` will freeze all nine paths for
independent read-only final review; post-PASS changes are limited to the status/
reviewer references already declared above.
