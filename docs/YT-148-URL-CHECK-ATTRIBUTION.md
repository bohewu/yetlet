# YT-148 — URL-check allocation attribution

Date: 2026-10-23. Status: **Done at bounded attribution/proposal scope after
independent final Reviewer PASS, zero issues and required followups.**

## Authority and preserved boundary

The user selected contract-first attribution of the remaining URL-check work,
reuse of accepted [YT-146](YT-146-SSR-SERIALIZER-ATTRIBUTION.md) and
[YT-147](YT-147-PRIVATE-ESCAPE-INTO.md) evidence, minimum new measurement, and
an evidence-led minimum optimization proposal only. Independent final Reviewer
PASS with zero unresolved required followups is required before Done. Baseline
is clean HEAD `91da058cf66922879e90c6c21b549807fb04a1f8` in the current checkout.
No commit, push, deployment or external environment change is authorized.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). This study changes no product
crate, public API, URL/security policy, renderer semantics, dependency, lock,
default feature graph or browser asset. No optimization candidate, alternative
checker, profiler, benchmark framework or validation infrastructure is selected.

Accepted evidence under `target/yt146-ssr-attribution-grvvqfm_/evidence` and
`target/yt147-escape-into-dc8ihdk3/evidence` remains read-only, including raw
observations, bindings, failures and independent reviews. Reuse its existing
whole-render allocation and elapsed results as historical context. Do not rerun
accepted builds, measurements, old tests, all-size output/hash checks, workspace,
browser, CSP, HTTP or dependency proofs. New checks below cover only the new
probe and its finite literal URL corpus; they are not new browser/CSP acceptance.

## Source-derived probe and exact corpus

Add sample feature `url-probes = []`, default off, to the existing
`examples/ssr-performance-baselines` package. Reuse its existing copied
`unsafe_scheme` and `URL_ATTRIBUTES` once: move those declarations, without
algorithm changes, from `shadow` into a small private sibling module inside
`attribution.rs`, and import them into the existing shadow and new probe.
Compile this seam for its existing consumers and `url-probes`; keep the shadow
renderer gated by its existing features. The new selected builds enable neither
`serializer-attribution`, `escape-comparison` nor `component-probes`. They do
not compile or invoke the old renderer as a current-render timing control.
Leave `counts.rs` unchanged and use only its existing global counters.

The copied checker must remain identical to the product checker: iterate
Unicode scalar values, remove only ASCII whitespace/control characters, apply
full `char::to_lowercase`, collect a temporary `String`, then check the exact
`javascript:`, `vbscript:` and `data:` prefixes. Preserve normalization,
allocation and temporary destruction inside each call. No URI/entity decoding,
Unicode-whitespace stripping, case folding or policy expansion is allowed.
Keep the exact URL-attribute list and product validation order. In particular,
`xlink:href` remains rejected by the existing identifier rule because of its
colon; its list membership does not make it a supported attribute.

Bind the copied declarations to the baseline source and YT-147's retained
`source-control-check.json` before interpreting measurements. The current
`crates/yetlet-html/src/lib.rs` SHA-256 is
`4bf49ed3b2a163c458efeeb3ad597d898f6dc802a43907818d6f851c268aa391`;
baseline `attribution.rs` is
`d05535b681c72018ba9ed9b11d10931aff611e226472902357c5e14724a52aae`.
Independent source review must confirm the moved declarations remain exact,
the old shadow changes only its imports, and product hashes/public declarations
are unchanged. Source binding and finite tests do not prove all Unicode inputs
or identical generated machine code.

Construct the existing `page(1024)` once per process. Before any timing/count
region, extract one borrowed `Vec<&str>` from that same retained `View`, visiting
element attributes in their stored order and then children in order, selecting
only names in the unchanged URL list. Retain the tree and vector through all
observations; perform no extraction, formatting, checking, input allocation or
input destruction inside a measured region. The exact large-page population is
1,024 `href` values, in row order, each `/records/NNNN?view=full&public=1`,
32 UTF-8 bytes each and **32,768 input bytes** total. Do not substitute a second
generated corpus for the borrowed workload.

One operation visits every borrowed value once and accumulates the returned
unsafe booleans into a primitive count, expected zero. Use `black_box` on the
corpus/input values, each checker result and final count to retain observable
work without changing the copied checker. Vector iteration, barriers and
primitive accumulation are inside both measured populations. Normalized String
creation, prefix checks and destruction complete before each call returns.
Input/output oracles, printing, hashing and process inspection stay outside.

## Fixed ordinary timing and counted populations

Build ordinary `url-probes` and counted `url-probes,allocation-counts` separately
with the same toolchain, locked inputs and default release settings. Verify the
ordinary feature fingerprint excludes the allocator wrapper and all old probe
features. Reject a URL timing invocation in an allocator-instrumented build;
never interpret instrumented elapsed time as ordinary timing.

New CLI `url-timing 1|2` performs exactly two fresh foreground process
repetitions, one per argument. Each has **five warmups and 20 observations** of
the same standalone URL corpus: **10 warmups and 40 measured operations total**.
Preallocate raw observation storage. Time the complete operation, including
checker temporary destruction; record integer `elapsed_ns`, repetition,
population, index, URL count, input bytes and unsafe count, then print after the
group. Keep every row and each repetition separate. Report count, mean, median
(average sorted entries 10/11), min/max and nearest-rank p95 (entry 19/20), with
explicit ns/op or microseconds/op. There is one current checker, no variant pair
or order-balancing claim, no outlier exclusion and no numerical performance gate.

New CLI `url-counts` runs one fresh counted foreground process, **three
operations and no warmups**. Preallocate three fixed stack snapshot records.
Use existing `counts::begin()` at each start and `counts::snapshot()` immediately
after the complete operation, before any formatting or assertion. Retain all
existing fields: allocation/zeroed/reallocation/deallocation calls, allocation
and deallocation request bytes, realloc old/new request bytes, failures, live
and region peak. There is no returned owned output: each normalized String has
already been freed at the end snapshot. Derive deltas and peak/live increments
against each operation's own start; require zero failures, zeroed calls zero,
end live equal start live, unsafe count zero and identical relative observations
across all three operations. Absolute process live baselines are not costs.

Reconcile each operation's event/request deltas to the accepted YT-146 **URL
bucket**, identical for body and document:

| Alloc calls | Realloc calls | Alloc request bytes | Realloc old bytes | Realloc new bytes |
|---:|---:|---:|---:|---:|
| 1,024 | 2,048 | 8,192 | 24,576 | 49,152 |

Use the retained raw bucket observations as well as their report, and record
the comparison before hotspot interpretation. YT-147's source binding establishes
that the product URL checker remained unchanged after escaping optimization;
its accepted current totals supply whole-render context. No new whole render
is needed. A mismatch stops causal interpretation and is reported for review;
do not tune the checker, replace evidence or rerun populations to force equality.
YT-146 did not attribute deallocation/live/peak to URL buckets: standalone values
for those fields are new observations, not a historical reconciliation claim.

Counters describe logical allocator events and requested sizes, not physical
copies, RSS, allocator overhead or retained per-bucket memory. Standalone timing
includes iteration/barriers and uses a source-derived private checker with
different placement/inlining opportunities. It establishes neither a whole-render
CPU share nor an additive component time. Do not subtract it from historical
render medians, infer a speedup or forecast a peak-heap/RSS reduction.

## Only new focused behavior checks

Add two sample tests selected by `yt148_`; filter all existing entry points:

- Corpus extraction/checker equivalence: verify all 1,024 borrowed values in
  exact row order against fixed row-format expectations, total bytes, no text or
  non-URL attributes, and every checker result false. Verify the declaration
  list and source-derived checker through source comparison, not a second
  checker implementation.
- Finite URL security results: use explicit literal booleans and literal
  `Result<String, RenderError>` expectations from actual public
  `yetlet_html::render_to_string`. Cover empty/relative/fragment/HTTPS URLs,
  escaped ampersand/quotes, multibyte content and lowercase expansion; the three
  blocked prefixes with mixed case and representative ASCII space, NUL, bell,
  tab, LF, CR and DEL obfuscations; and nearby nonmatching prefixes. Include
  retained non-ASCII whitespace, with literal expectations matching the current
  ASCII-only rule. Exercise `href`, `src`, `action`, `formaction`, a non-URL
  attribute carrying scheme text, and `xlink:href`'s `InvalidAttribute` result.
  Include duplicate/invalid/forbidden attribute and invalid-tag precedence over
  URL checking. Do not derive render expectations from the copied checker or
  replay the old general validation corpus.

These tests cover new extraction and a finite security boundary, not a product
change or exhaustive Unicode/URL verification. The measured valid page itself
contains only relative ASCII URLs; unsafe/security inputs are unmeasured.

## Footprint, execution limits and commands

Stage 1 changes only this contract and the three metadata docs below. Parent
checks documentation and obtains independent contract PASS before stage 2.
The expected complete tracked footprint is exactly these eight paths:

```text
docs/YT-148-URL-CHECK-ATTRIBUTION.md
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/attribution.rs
```

The small shared private module fits in `attribution.rs`; no new Rust file is
needed. Preserve all product crates, root/sample locks, dependencies,
`counts.rs`, existing entry-point behavior and historical tracked reports.
Record protected hashes against the prework `baseline.json` in
`target/yt148-url-attribution-unxqzbfb/evidence`. Retain new evidence there and
use separate `build-normal`/`build-counted` directories beneath the same run root.

After contract PASS, record UTC/monotonic start and a fixed **30-minute stage-2
deadline**, with no renewal. Verify actual ext4 paths using `findmnt -T`, Rust/
Cargo 1.99.0, the same CPU/host/cgroup identities, System allocator and absent
release/RUSTFLAGS overrides; stop if incompatible. Keep CPU 0, executable address
space **1 GiB**, build jobs **2**, offline locked/default-release inputs and
existing host settings. Do not install tools or change affinity policy, quotas,
allocator, LTO/PGO, host capabilities or security settings.

Build/test preparation commands use a 600-second timeout each. Build tests once
with `--no-run --message-format=json` and select the emitted test executable;
run only that new binary's `yt148_` filter at CPU 0/1 GiB. Build/test commands:

```text
cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml --check
cargo test --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --features url-probes --target-dir target/yt148-url-attribution-unxqzbfb/build-normal --jobs 2 --no-run --message-format=json
timeout 600 taskset -c 0 prlimit --as=1073741824 -- <new-test-executable> yt148_ --test-threads=1
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features url-probes --target-dir target/yt148-url-attribution-unxqzbfb/build-normal --jobs 2
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features url-probes,allocation-counts --target-dir target/yt148-url-attribution-unxqzbfb/build-counted --jobs 2
timeout 15 taskset -c 0 prlimit --as=1073741824 -- target/yt148-url-attribution-unxqzbfb/build-normal/release/yetlet-ssr-performance-baselines url-timing 1
timeout 15 taskset -c 0 prlimit --as=1073741824 -- target/yt148-url-attribution-unxqzbfb/build-normal/release/yetlet-ssr-performance-baselines url-timing 2
timeout 10 taskset -c 0 prlimit --as=1073741824 -- target/yt148-url-attribution-unxqzbfb/build-counted/release/yetlet-ssr-performance-baselines url-counts
cargo xtask check docs
git diff --check
```

The three-process measurement batch has a **60-second monotonic ceiling** and
**8 MiB combined raw stdout/stderr/receipt budget**. Capture exact argv, source/
contract/lock/binary hashes and sizes, selected nonsecret environment, feature
fingerprints, native exit, timeout and UTC/monotonic boundaries before analysis.
Retain warmups, observations, failures and partial/unknown exits. No observation
replacement, automatic retry, optimization tuning or workload expansion is
authorized. A failed check or bound stops work for classification/reporting under
the materiality policy; it does not authorize repair or a new population here.

Use finite foreground commands only. Start no server, browser, watcher, container
or service. Wait/join each owned process, terminate only owned children on timeout
and verify no owned command/session remains. Retain inert build/raw artifacts;
record cleanup before claiming completion.

## Acceptance and next action

Done requires prior independent contract PASS, the bounded sample-only probe,
the two focused tests, exact source/security and protected-input review, all
fixed new populations with count reconciliation and limits disclosed, retained
raw failures/receipts, owned cleanup and independent final Reviewer PASS with
zero unresolved required followups. Identify the allocation hotspot from the
reconciled events separately from the standalone elapsed observations. Only
then propose one minimum future optimization, or explain why none is justified.
A possible prefix-streaming approach is a proposal requiring separately
authorized implementation and equivalent-safety review; no candidate code or
second algorithm belongs in this item. No production speed, SLO, public release
or architectural promotion follows; YT-096/YT-020 retain their existing gates.

At this checkpoint, no new implementation, test, build or measurement has run.
The next unblockable action is independent YT-148 contract review. Its PASS
unblocks only the fixed sample probe, checks and observations above; proposal
implementation remains a separate owner decision.

## Stage-2 results — 2026-10-23

The initial checkpoint above is retained as the pre-execution contract record.
Independent contract Reviewer `/root/yt148_url_contract_review_20261023` returned
PASS with zero issues or required followups before implementation. Its four
reviewed file hashes are retained in `contract-review-result.json` under the new
evidence root. Stage 2 started at `2026-10-23T15:50:27.594495+00:00`, monotonic
`138062593960532` ns, with the unchanged deadline
`2026-10-23T16:20:27.594495+00:00` / `139862593960532` ns. No renewal occurred.

### Implementation and focused verification

Only the four sample files listed in the footprint changed: default-off
`url-probes`, CLI routing, the shared private source-derived checker, borrowed
corpus/operation and two new tests. The old shadow retains its feature gates and
changes only its import and moved declarations, plus formatting whitespace.
No product implementation, optimization, public API, dependency, lock,
`counts.rs`, browser code or default feature graph changed.

`source-control-check.json` binds the moved checker and URL list to baseline
`attribution.rs`, the unchanged product source hash above and the retained
YT-147 `source-control-check.json` (SHA-256
`c5ce860a7fd66a1cadeb6be95d745e6ec14ccc89e3e34188ff820b9af3664d92`).
Checker code is identical after indentation normalization; list formatting is
ignored. Product source and public declarations are unchanged. Independent final
source review remains required; neither this comparison nor the finite tests
proves all Unicode inputs or identical machine code.

The sample format check, one `--no-run --message-format=json` test compilation,
only the emitted test binary's `yt148_ --test-threads=1` filter, and both selected
release builds passed. Exactly two tests passed; the existing test was filtered
out. Corpus checks establish 1,024 borrowed URLs in exact row order, 32,768 bytes,
attribute-before-child order and exclusion of text/non-URL attributes. Literal
public renderer expectations cover the finite security inputs specified above,
including ASCII obfuscations, retained non-ASCII whitespace, lowercase expansion,
attribute selection and validation/error precedence. These inputs are unmeasured;
the measured page remains relative ASCII only. No browser/CSP/HTTP, old test,
old build, whole-render or all-size hash population was rerun.

`stage2-environment.json` records Rust/Cargo 1.99.0, ext4 evidence/build paths,
the same host/CPU/cgroup identities and limits as YT-147, System allocation and
absent selected release/RUSTFLAGS/allocator overrides. All executable observations
used CPU 0 and a 1 GiB address-space ceiling; builds used two jobs, offline locked
inputs and default release settings. `premeasurement-binding.json` retains source,
contract, lock and binary hashes/sizes and Cargo fingerprints. Ordinary features
are exactly `default,url-probes`; counted features add only `allocation-counts`.
No old renderer feature is selected; counted URL timing rejects instrumentation.

### Reconciled allocation events

All three fresh-process counted operations had the following identical deltas.
Every operation's event/request fields also match every retained URL-bucket row
for both YT-146 body and document, including zero failures and zeroed calls.
`measurement-summary.json` retains those raw rows, source paths, hashes and
comparisons before interpretation.

| Field | Per operation, 1,024 URLs |
|---|---:|
| Allocation calls | 1,024 |
| Zeroed calls | 0 |
| Reallocation calls | 2,048 |
| Deallocation calls | 1,024 |
| Allocation request bytes | 8,192 |
| Reallocation old request bytes | 24,576 |
| Reallocation new request bytes | 49,152 |
| Deallocation request bytes | 32,768 |
| Failed allocations / reallocations | 0 / 0 |
| End live increment | 0 bytes |
| Region peak increment over own start live | 32 bytes |
| Unsafe results | 0 |

Deallocation/live/peak are new standalone observations: YT-146 did not attribute
those fields to URL buckets. The absolute process baseline of 1,658,720 live
requested bytes includes retained inputs and is not a URL cost. Each normalized
String is destroyed before the end snapshot. Requested sizes and logical events
do not establish allocator overhead, copies, RSS or physical memory.

The source-bound URL normalization is an allocation-event hotspot: its 1,024
allocation calls compare with accepted YT-147 current body/document totals of
1,025/1,027, and its 2,048 reallocations compare with 2,063/2,069. Those are event
concentrations, not elapsed shares or additive cost estimates. Historical current
body/document realloc-new request totals are 573,424/1,309,539 bytes, so call
concentration alone also does not establish request-volume concentration.
No current whole render was needed or performed.

### Ordinary standalone elapsed observations

Each operation visits all 1,024 URLs once. Values below are integer-derived
**ns/op**, with each repetition reported separately. Median averages sorted
entries 10/11; nearest-rank p95 is entry 19 of 20. Both processes retain all five
warmups and twenty measured rows, with no exclusions or replacements.

| Repetition | Measured count | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 20 | 396,896.85 | 404,745 | 482,284 | 292,035 | 485,337 |
| 2 | 20 | 284,066.55 | 271,423 | 373,456 | 212,920 | 380,556 |

The first repetition generally rises and the second generally falls; raw order
and variation remain visible. There is no paired variant, significance test,
order-balancing claim or numerical gate. Iteration, barriers, primitive result
accumulation, normalization, prefix checking and temporary destruction are inside
these regions. Extraction, input lifetime, assertions and printing are outside.
The source-derived private checker has different compilation placement/inlining
opportunities. These observations establish neither whole-render CPU share nor
speedup; they must not be subtracted from historical renderer medians.

### Retention, failures, limits and next action

The three-process batch ran once from `2026-10-23T15:55:05.191669+00:00` to
`2026-10-23T15:55:05.226822+00:00` (35,053,673 monotonic ns), within its 60-second
ceiling and the stage deadline. Combined measurement stdout/stderr/receipts total
14,910 bytes, below 8 MiB. Native exits were `0,0,0`; all 10 warmups, 40 measured
operations and three counted operations are retained in `url-timing-1.stdout`,
`url-timing-2.stdout`, `url-counts.stdout`, their stderr and receipt files.
`measurement-batch-start.json`, `measurement-batch-result.json` and
`measurement-summary.json` retain boundaries, exact argv, binary/source/feature
bindings and raw-derived summaries. No observation was retried or replaced.

One pre-build direct source-comparison assertion failed because a removed
function left an extra blank line in the expected shadow text while formatting
collapsed it. Checker/list comparisons had already passed; no product behavior
was disproved. Work stopped and the parent authorized one smallest harness
correction: ignore blank lines only in the old-shadow comparison. The focused
check then passed without a product edit, new run or population replay.
`source-binding-initial-failure.json` and `source-control-check.json` retain the
failure and correction. Product repair attempts and operational retries are zero.
All preparation and measurement commands are finite foreground processes; all
owned process leaders were waited and absent at cleanup, with no remaining owned
run command. No server, browser, watcher, service or container was started.
`protected-inputs.json` and `cleanup.json` retain protected hashes, all seven
immutable evidence bindings and cleanup. Documentation/whitespace checks are
owned by the parent after metadata updates and remain pending at this checkpoint.

One minimum future proposal is justified by the reconciled event concentration:
consider removing only the temporary normalized String through a private
prefix-streaming check. This is a proposal, with no candidate or second checker
implemented here. A separately authorized implementation must preserve Unicode
scalar traversal, ASCII-only whitespace/control filtering, full lowercase
expansion and exact three-prefix semantics, plus URL-list and validation/error
order. Non-ASCII whitespace must stay retained. It needs equivalent-safety review
and new bounded evidence before any adoption. The 32-byte standalone peak does
not forecast a whole-render peak/RSS reduction, and these timings forecast no
production speedup.

**YT-148 is Done at bounded attribution/proposal scope.** Independent final
Reviewer `/root/yt148_url_attribution_final_review_20261023` returned **PASS,
zero issues and required followups**. All eight reviewed file hashes match;
the verdict and source/result bindings are retained in `final-review-result.json`.
Parent documentation/whitespace checks passed before review. Only these completion
metadata/reviewer references and the sample README status are updated after PASS,
followed by documentation/whitespace checks; Rust, Cargo and measured evidence
remain unchanged. No further validation or optimization implementation follows.
The next proposed work is a separately authorized private prefix-streaming
contract/implementation; no API or architecture decision remains open in this
study. No commit or push is authorized, and YT-096/YT-020 and public-release/
production gates remain unchanged.
