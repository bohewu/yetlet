# Fixed SSR cost observations

This unpublished, standalone optional sample implements the
[YT-145 contract](../../docs/YT-145-SSR-PERFORMANCE-BASELINES.md). It uses existing
Core/HTML/server-Axum APIs, without HTTP, browser code or optional providers.
Its private workspace/lock and default-empty features leave root consumers
unchanged. No performance threshold or CI gate is introduced.

Build normal timing and counted binaries separately:

```bash
CARGO_TARGET_DIR=target/yt145-ssr-cost-vpx_z1d5/build-normal cargo build --release --locked --manifest-path examples/ssr-performance-baselines/Cargo.toml
CARGO_TARGET_DIR=target/yt145-ssr-cost-vpx_z1d5/build-counted cargo build --release --locked --manifest-path examples/ssr-performance-baselines/Cargo.toml --features allocation-counts
```

`output small|medium|large EXISTING_DIRECTORY` writes independently checked
body/document HTML. `timing 1|2` is available only in the uninstrumented build;
it emits warmup and measured CSV rows for all fixed cases/phases. `counts SIZE
PHASE` exists only in the counted build and emits three start/held/released
snapshot sets; phase is `construct|body|document|combined`. Both executables use
System; only the counted build wraps it with nonallocating atomic bookkeeping.
No measured-work hashing, logging, output accumulation or tree clone is needed.

The contract defines exact lifetimes, fixed order, samples, resource limits and
raw retention. Run executable measurements under a recorded single-CPU affinity,
1 GiB address-space ceiling and external timeout. Keep ordinary timing separate
from instrumented counts. Counter fields are cumulative snapshots: derive
creation/release deltas relative to each operation's start/held points. The
region peak is reset to current logical live bytes, including retained inputs;
it is not process RSS or allocator physical overhead. Counted stderr records
whole-process startup/post-work VmRSS/VmHWM. Wrap each fresh counted process with
`/usr/bin/time -v` for a separate process peak; never subtract cumulative maxima.

Focused checks:

```bash
cargo test --locked --manifest-path examples/ssr-performance-baselines/Cargo.toml --features allocation-counts -- --test-threads=1
cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml -- --check
cargo xtask check docs
```

The one fixed-page test checks all sizes, dimensions and independent escaped
literal outputs; one direct known-layout test checks alloc/zeroed/realloc/free
counter semantics. Accepted historical workspace/browser/CSP measurements are
reused. Consult the contract for current results, limits and review status.

## YT-146 attribution extension

The [YT-146 contract](../../docs/YT-146-SSR-SERIALIZER-ATTRIBUTION.md) adds two
sample-only, default-off features without changing dependencies or the old
commands. `serializer-attribution` enables `allocation-counts` plus a pinned
source-derived serializer copy. `attribution-output EXISTING_DIRECTORY` checks
and writes all six new-copy outputs. `attribution-counts body|document` records
three large-page operations with global snapshots and creation-only call-site
buckets. The copy invokes the actual public escape function; its private
validation/traversal and adapter composition preserve the pinned source.
Buckets describe where allocation/reallocation requests happen, not ownership,
retained memory, frees, peak attribution or physical copies. Only outer regions
reset global peak; lexical tags restore on return, error and unwind.

`component-probes` in an ordinary build provides `component-timing 1|2` for the
large borrowed escape corpus and source-faithful envelope composition around
the accepted pre-rendered body. It rejects allocator-instrumented timing.
Each group records 5 warmups and 20 measured operations; repetition 2 reverses
probe order. These are standalone component observations, not whole-renderer
CPU shares, isolated memcpy measurements or speedups. Run from the repository
root so the retained YT-145 body file resolves correctly.

Only new tests selected by `yt146_` are needed for this extension. Use separate
`build-components`/`build-attribution` target directories, `--offline --locked`,
the explicit feature and `--test-threads=1`. The contract fixes CPU 0, 1 GiB
address space, finite deadlines and raw receipts. Preserve all accepted YT-145
artifacts and failures; do not replay its old test or measurement groups.
No optimized production implementation is included.

The retained YT-146 run matches all six output hashes and reconciles six new
allocation operations to YT-145. Escaping contributes 6,150 of 7,176 document
allocation calls; buffer request volume is a separate concentration. The
contract/report retains 80 ordinary component timings and 20 warmups, their
limits and one future private escaping proposal. No speedup or optimization
implementation is claimed; independent final review remains pending.


## YT-147 private escaping comparison

The separately selected [YT-147 contract](../../docs/YT-147-PRIVATE-ESCAPE-INTO.md)
adds default-off `escape-comparison`. It reuses the existing shadow serializer
as the reference with fixed no-op tags and calls the changed actual libraries as
current. The reference still allocates through public `escape_text`; it is a
source-derived control using the current unchanged-semantic wrapper, not an
identical historical binary. Ordinary comparison excludes allocator bookkeeping;
counted comparison adds only `allocation-counts`, without serializer attribution.

New commands are `comparison-output EXISTING_DIRECTORY` (twelve files covering
both variants, three sizes and body/document), `comparison-timing 1|2` (large
page only, four groups per repetition, five warmups plus twenty measured entries
each; second repetition reverses order) and counted `comparison-counts
reference|current body|document` (three operations in each fresh process).
Timing rejects allocation-counted builds. Inputs are prebuilt/borrowed, output
is held at timing end and dropped outside; internal document body temporaries
remain inside. Counts retain global start/held/released snapshots, without tags.

Use only new `yt147_` focused tests, separate `build-normal`/`build-counted`
directories under the new run, offline/locked release builds, CPU 0, 1 GiB and
finite bounds/receipts from the contract. Do not run the old tests or commands
as YT-147 proof. Locks, dependencies, default features and `counts.rs` are unchanged.

The fixed new run passes all twelve hashes and reconciles reference allocation
snapshots to accepted YT-146 raw evidence. Current body/document each remove
6,149 allocation calls; held sizes are unchanged and document peak is unchanged.
Both ordinary repetitions have lower current medians, with variation retained
and no significance, stable speedup or production claim. The contract retains
all raw counts/timings, source/control caveats and cleanup. Independent final
review remains required; YT-147 is In progress.

## YT-148 standalone URL-check attribution

The [YT-148 contract](../../docs/YT-148-URL-CHECK-ATTRIBUTION.md) adds default-off
`url-probes`. Its private checker is the unchanged source-derived checker shared
with the existing shadow. The shadow keeps its original feature gates; selected
URL builds enable no old probe feature. Product crates, locks, dependencies and
`counts.rs` are unchanged.

`url-timing 1|2` in an ordinary build records five warmups and twenty observations
per fresh process. Counted builds reject timing. `url-counts` with
`url-probes,allocation-counts` records three operations without warmups, including
all start/end counter fields. Each operation borrows the same retained large
page's 1,024 relative ASCII URLs (32,768 bytes), checks each once and destroys
normalized temporaries before the region ends. Extraction, assertions and CSV
printing remain outside measured regions.

Build separate normal/counted targets with offline, locked default-release
inputs. Run only the two new `yt148_` tests, selected from the emitted test binary;
do not replay the historical checks above. The contract specifies CPU 0, 1 GiB,
fixed deadlines, exact populations and retained native receipts. Count deltas
must reconcile to both accepted YT-146 URL buckets before interpretation. New
standalone frees/live/peak have no historical per-bucket counterpart. Standalone
elapsed observations include iteration/barriers and different compilation
placement; they establish no whole-render CPU share or speedup. Finite security
checks are unmeasured and preserve the existing ASCII-only filtering rule.

YT-148 is Done at bounded attribution/proposal scope after independent final
Reviewer PASS, zero issues and required followups. A future optimization proposal
requires separate implementation authorization.

## YT-149 private URL prefix comparison

Status: **Done at bounded implementation/comparison scope after independent
final Reviewer PASS, zero issues and required followups**.
The default-off `url-comparison = ["url-probes"]` retains the legacy normalizer
and adds a source-derived copy of the changed private streaming checker. This
is timing of a sample copy, not an actual private product call or public API.
Existing probe/shadow commands remain unchanged. Build ordinary with
`--features url-comparison`; counted adds `allocation-counts` and rejects timing.
New commands are `url-comparison-output EXISTING_DIRECTORY`,
`url-comparison-timing 1|2` and, in counted builds,
`url-comparison-counts reference|current`.

All three new filtered tests, both fresh release builds and six changed-product
literal/hash outputs passed. Each timing repetition checks the same borrowed
1,024 row-ordered URLs / 32,768 bytes, five warmups and twenty measurements per
variant; repetition 1 orders reference/current and repetition 2 current/reference.
Exactly four new measurement processes ran once, retaining 20 warmups, 80 measured
and six counted operations. Every new reference counted operation matches every
retained YT-148 raw snapshot across all 12 relative fields. Reference per operation
has 1,024 allocations, 2,048 reallocations, 1,024 frees and 32-byte peak increment;
current records zero events/request bytes/live/peak increments on this corpus.
These are observations, with no numerical performance/allocation gate.

| Repetition | Variant (order) | Count | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | reference (1) | 20 | 256,753.50 | 250,092.0 | 322,856 | 205,403 | 339,349 |
| 1 | current (2) | 20 | 7,772.45 | 7,768.0 | 7,799 | 7,752 | 7,808 |
| 2 | current (1) | 20 | 26,525.00 | 26,432.0 | 26,538 | 26,419 | 28,038 |
| 2 | reference (2) | 20 | 363,035.80 | 348,281.5 | 498,336 | 250,449 | 524,009 |

The repetitions vary; these standalone copied-checker timings include barriers
and iteration. They establish neither stable/production speedup nor whole-render
CPU share, additive cost, RSS or whole-render peak reduction. No old accepted
population, whole-render timing/count, browser/CSP/HTTP test or old test entry
point was replayed. Product APIs, dependencies, locks, `counts.rs`, default
features and old shadow/reference remain unchanged.

[The full YT-149 contract and results](../../docs/YT-149-PRIVATE-URL-PREFIX.md)
retain proof, source bindings, six output hashes, raw paths, all timing statistics,
reconciliation, resource bounds, cleanup and failures. The initial metadata
binding assertion failed and received one parent-authorized harness correction;
a separate manual pre-execution counted-predicate correction is recorded as such.
Normal tests were compiled before that excluded counted-only edit; the exact
pre/post source hashes and unchanged exercised sections are disclosed. Initial
raw `findmnt` text was not persisted after the metadata assertion failure;
exact argv/zero exits/ext4 assertions are attested, with this receipt limitation
retained for review. No product repair, operational retry, commit or push occurred.


## YT-150 complete SSR source-version comparison

Status: Done at bounded full-SSR measurement scope after independent final
Reviewer PASS, zero issues and required followups.
See [the contract](../../docs/YT-150-FULL-SSR-COMPARISON.md). Default-off
`page-comparison` calls actual product APIs from two pinned source archives,
with identical frozen sample code. This covers the existing 1,024-row page,
separating owned View/metadata construction, complete-document render with
retained inputs, and combined construction/render/input destruction. The final
output is held at timer end and dropped afterward in every output-producing
phase. Combined therefore has a different final-output lifetime from YT-145.

New commands are `page-output reference|current EXISTING_DIRECTORY`,
`page-timing reference|current 1|2`, and counted-only
`page-counts reference|current construct|document|combined`. Labels identify the
source binary; they never change product behavior. Ordinary builds select only
`page-comparison`; counted builds add `allocation-counts` and reject timing.
The contract fixes order/populations, separate ordinary and logical allocation
accounting, exact source/binary/output bindings, resource bounds and cleanup.
Only new `yt150_` tests and new commands are selected; historical commands and
accepted proofs are not replayed. No product source, API, dependency, default
feature graph, lock, renderer copy or other optimization is introduced.


The completed fixed large-page comparison retains 240 measured times/60 warmups
and 18 counted operations. Reference construct/document observations reconcile
to YT-145/YT-147 raw data. Document medians change by −31.94%/−31.36% and combined
by −21.29%/−21.78% in the two repetitions; unchanged construction also varies,
so these are descriptive local observations, not stable production speedups.
Document allocation/reallocation calls change from 1,027/2,069 to 3/21; logical
held/peak bytes stay unchanged. This uses actual product APIs, not URL probe
subtraction. The report retains all raw populations, mode-aware source archives,
preparation failures, current-vs-historical login-session context, receipt
limitations and cleanup. No product optimization or old proof replay follows.

## YT-151 View and metadata construction attribution

Status: Done at bounded attribution/proposal scope after independent final
Reviewer `/root/yt151_view_attribution_final_review_20261024` PASS, zero issues and required followups. [The contract](../../docs/YT-151-VIEW-CONSTRUCTION-ATTRIBUTION.md)
fixes the new population and resource bounds. Default-off `construction-study`
uses a source-derived fixture builder calling actual unchanged Core/metadata APIs.
Build ordinary with `--features construction-study`; counted adds
`allocation-counts` and rejects timing. Existing study features remain excluded.
New commands are `construction-output EXISTING_DIRECTORY`,
`construction-timing 1|2` and counted-only `construction-counts`.

Seven exclusive creation-call buckets separate five row formats, other String
conversions, attribute Vec insertion, children Vec insertion/extension, metadata,
inline node construction and unclassified work. Formatted/metadata Strings are
charged to their own stages; inline View storage is included in Vec requests.
Buckets provide no per-owner frees/live/peak, physical copies or CPU percentages.
The three new full-builder counted operations must reconcile every global field
to retained YT-150 construct observations before interpretation.

Separate ordinary format and prepared-input assembly probes run in reversed
orders with five warmups and twenty observations per group in each repetition.
Formatting includes String destruction per row; assembly prepares exact owned
formatted Strings outside its timer and includes static String/metadata/Vec/node
work plus payload bookkeeping inside, ending with the tree held. These boundaries
have different lifetimes/cache/compiler placement from complete construction;
they are not additive costs or a before/after optimization. Two new filtered
tests and six literal/hash outputs check the attributed/prepared builders.
No product optimization, public API/dependency/lock change or old proof replay
is authorized. New evidence retains 80 ordinary times/20 warmups and three counted operations,
all exit 0. Two new filtered tests, both release builds and six literal/hash
outputs pass. All nine pairs of global counted observations reconcile to
retained YT-150 data across twelve held/released fields. Children Vec requests
are 1,115,904 bytes (67.9739%); five formats create 5,120 allocations (41.6091%
of calls); metadata is two allocations / 42 bytes. Zero node events describe
inline representation, not zero CPU/storage cost. Format medians are
131.583 / 131.5095 microseconds and prepared assembly 118.580 / 117.898;
these are nonadditive probes, not whole-construction CPU shares or speedups.
The report proposes only a future private first-child capacity experiment,
with explicit multi-child reallocation/move risks. No optimization is implemented;
final independent Reviewer PASS matches all nine reviewed file hashes.

## YT-152 private first-child capacity comparison

Status: Done at bounded comparison/rejected-candidate scope after independent
final Reviewer `/root/yt152_first_child_final_review_20261024` PASS, zero issues and required followups. [The contract](../../docs/YT-152-FIRST-CHILD-CAPACITY.md)
compares actual baseline/current Core sources with identical default-off
`child-capacity` sample code; counted adds `allocation-counts` and rejects timing.
No old study feature is selected. New commands are
`child-output reference|current EXISTING_DIRECTORY`,
`child-timing reference|current 1|2` and counted-only
`child-counts reference|current single|multi|page|combined`. Labels identify the
pinned library binary; no runtime product policy switch exists.

Fixed simple fixtures have 1,024 parents with one/four sensitive Text children,
and a Range-map bulk root. The original 1,024-row page remains unchanged.
Single/multi/page construction stops with metadata/View held; combined includes
complete-document rendering and input destruction, ending with output held.
Destruction of the returned result, inspection and printing are outside timers.
Four ordinary processes retain 80 warmups/320 measurements in reversed orders;
eight counted processes retain 24 operations with all global start/held/released
fields. Reference page/combined must reconcile to accepted YT-151/YT-150 data.
Two new filtered tests, four release builds and twenty literal/hash outputs cover
the changed child insertion path without replaying accepted entrypoints.

The archived experimental policy converts first, reserves one slot only at zero capacity, then
uses normal Vec growth. Bulk insertion, API/representation/output/security and
all dependencies/locks stay unchanged. Report extra multi-child reallocations,
logical heap and elapsed variation; no universal speedup or physical-copy claim
follows. No further optimization or new commit/push is selected.


New evidence passes both filtered tests, four release builds and twenty literal/
hash outputs; 320 measurements, 80 warmups and 24 counted operations are retained.
Reference page/combined snapshots match every retained historical counterpart.
The candidate reduces full-page construction logical held/peak bytes 35.99%,
but adds 1,026 reallocations; complete SSR medians worsen 5.80% / 4.47% and
multi-child medians 91.81% / 77.35% in this bounded pair. Variation is disclosed,
without a universal regression claim. **Do not adopt the candidate in default
Core.** The checkout restores the exact original production body and retains only
the new Core regression test. Executed candidate evidence remains immutable in
its pinned archive; final restored production source reuses accepted baseline
proof, without a separately rebuilt final-checkout claim. The report includes
the candidate method for reproducibility and all raw/source/binary bindings.
Two preparation errors and their smallest correction are retained; no native
proof/workload failed or was replayed. Final independent Reviewer `/root/yt152_first_child_final_review_20261024` PASS matches all nine reviewed file hashes and accepts the disclosed verification boundary.

## YT-153 document buffer attribution

Status: Done at bounded attribution/proposal scope after independent final
Reviewer `/root/yt153_document_buffer_final_review_20261024` PASS, zero issues and required followups. [The contract](../../docs/YT-153-DOCUMENT-BUFFER-ATTRIBUTION.md)
adds default-off `buffer-study`; counted adds `allocation-counts` and rejects
timing. No old study feature is selected. New commands are
`buffer-output EXISTING_DIRECTORY`, `buffer-timing 1|2` and counted-only
`buffer-counts`. Product sources, APIs, locks, dependencies and default graphs
remain unchanged. No optimization is implemented.

The source-derived adapter composition calls the actual current HTML renderer
and title escaper, preserving append order and title/body temporary drop at the
original statement boundary. Nine stack boundary observations record String
length/capacity and counted global live/peak. A single-thread fixed 64-event
nonallocating trace records allocation/reallocation/free requests with current
body/title/envelope/other call-site tags, without pointers or owner tracking.
Temporary frees occur in envelope scope; output release occurs in other scope.
Three new full-document counted operations must reconcile all fields to retained
YT-150 current-document snapshots, including raw event/byte/live/peak integrity.

Separate ordinary probes measure envelope composition with a pre-rendered borrowed
body, body append into a prepared prefix, and closing append into prepared prefix
plus body. Two fresh reversed-order repetitions retain 30 warmups/120 times.
Setup, output inspection and destruction are outside timers. Borrowed-body lifetime,
cache/allocator/compiler placement and clock/barrier effects differ from actual
complete rendering: times are nonadditive and establish no CPU shares/speedup.
Two new filtered tests, two release builds and six new literal/hash outputs cover
measurement semantics. Source/binary/raw/bounds/cleanup/failure evidence and one
future minimum proposal will be recorded before final independent review.


YT-153 new evidence passes two filtered tests, both release builds and six
literal/hash outputs. Exactly 120 ordinary observations / 30 warmups / three
counted operations are retained, all native exits 0. All nine historical pairs
match every global held/released field; 27 events per operation, zero overflow,
and all event/byte/live/peak/boundary checks reconcile. The final fourteen-byte
suffix grows envelope capacity 245,193 to 490,386, leaving 245,179 spare bytes;
body/envelope overlap earlier establishes the 507,337-byte logical peak.
Envelope medians 7.698/10.666 microseconds, body append 6.060/10.717 and suffix
append 79.5/134.5 nanoseconds are separate nonadditive prepared-body probes.
The report proposes only a future private tail-capacity reservation comparison,
with no demonstrated CPU or peak benefit; arithmetic even permits fourteen more
peak bytes. No optimization is implemented. A manual normal-only timer edit
postdates counted-test compilation; exact hashes/excluded source and focused
format proof are disclosed, with no counted-test replay. Final independent Reviewer `/root/yt153_document_buffer_final_review_20261024` PASS matches all nine reviewed file hashes and data/history/executable bindings.

## YT-154 actual document tail-capacity comparison

Status: Done at bounded implementation/comparison scope after independent final
Reviewer `/root/yt154_document_capacity_final_review_20261024` PASS, zero issues and required followups. [The contract](../../docs/YT-154-DOCUMENT-CAPACITY-COMPARISON.md)
compares pinned actual baseline/current libraries with identical sample code.
Default-off `document-capacity = ["buffer-study"]` reuses the unchanged accepted
finite trace; counted adds `allocation-counts`, ordinary excludes the allocator.
Old buffer-study commands/tests are compiled but not executed. New commands are
`document-capacity-output reference|current EXISTING_DIRECTORY`,
`document-capacity-timing reference|current 1|2` and counted-only
`document-capacity-counts reference|current small|medium|large|combined`.
Labels identify source binaries, without a runtime product-policy switch.

Small/medium/large document groups call actual APIs with retained inputs; large
combined includes construction/render/input teardown. Returned Strings are held
at stop and dropped outside timers; length/capacity are inspected outside.
Four reversed-order ordinary processes retain 80 warmups/320 measurements.
Eight counted processes retain 24 operations. Document groups alone reuse the
64-event trace; actual APIs have other-scope events and source order proves
body drop before suffix (the suffix may not allocate). Combined leaves tracing
disabled and records global heap only. Reference large/combined reconcile to
accepted YT-153/YT-150 raw data, without replaying old acceptance commands.

The minimum candidate reserves body plus suffix after the actual renderer returns,
appends body, explicitly drops it at the original boundary and appends suffix.
No API/renderer/security/default dependency/lock change is selected. One new
filtered test, four release builds and twelve new literal/hash outputs cover the
changed real adapter. Retained-capacity/peak/elapsed tradeoffs determine the
recommendation; no speedup or peak reduction is assumed. No further optimization
or new commit/push follows.


New YT-154 evidence retains one new passing filtered test (four old tests
filtered), four passing release builds, twelve literal/hash outputs, 320 measured
times/80 warmups and 24 balanced counted operations. Both historical groups match
all nine pairs across twelve fields; document traces reconcile all events and
internal releases, combined tracing stays off. Output capacities change
4,738→2,383 / 62,098→31,063 / 490,386→245,207; each group removes one realloc,
while logical peak increases fourteen bytes. Body source order and trace evidence
preserve its release boundary. Keep this minimum candidate for returned-capacity
benefit, with no universal speed claim: large render/combined paired medians are
lower, small/medium are higher and first-repetition variation is substantial.
All raw negative results remain in the report. One contract EOF whitespace failure
and its smallest correction are retained; no runtime proof/workload failed or
was replayed. Final independent Reviewer `/root/yt154_document_capacity_final_review_20261024` PASS matches all nine reviewed file hashes and data/history/executable bindings; no new commit/push.

## YT-155 small/medium latency attribution and stability

Status: Done at bounded research scope after independent contract and final
Reviewer `/root/yt155_latency_stability_final_review_20261024` PASS, zero issues and
required followups. [The contract](../../docs/YT-155-SMALL-MEDIUM-LATENCY-STABILITY.md)
adds default-off `latency-study = []` with no previous study/counter dependency.
Build two new ordinary binaries against YT-154's accepted reference/current
product bytes and identical measurement sample overlay. Product sources, locks,
APIs, output/security and body release order remain unchanged; no optimization.
New commands are `latency-api reference|current 1|2` and `latency-tail 1|2`.
Counted configurations reject both commands.

Actual API mode retains inputs, records eight initial calls per size, performs
50-ms unmeasured warmup bounded by 100,000 calls, then ten uninterrupted blocks
of 32 per-call output-held timings. Four source/order-balanced fresh processes
retain 64 initial / 2,560 steady observations. Initial does not mean cold cache.
Tail mode runs the current binary twice with fresh prefix/body preparation outside
each timer, comparing two source-derived legacy/reserved tail copies in same-process
ABBA/BAAB order. Each size gets the same bounded warmup then ten blocks with 32
observations per strategy, 2,560 tail times total. Body destruction is inside the
tail timer before closing; returned output destruction is outside all timers.

The six-process population retains 5,184 raw times, block distributions and
optional reported CPU-0 frequency/resource context. Reuse accepted capacity /
allocation/output evidence, without old command replay. Tail copies are private
measurement proxies, not actual private adapter timing or pure reserve-call
cost; do not add/subtract their medians into API time or assign CPU percentages.
Serial blocks and kernel-reported context establish no independent trials or
unique hardware/allocator cause. One new filtered test verifies preparation and
tail outputs/errors; no warmup/validator infrastructure tests are introduced.
Final independent review PASS precedes Done; no commit/push.

New fixed population completed once: 64 initial + 2,560 steady API + 2,560
steady tail times, six native exits 0; all 12 warmups completed, no exclusions
or retries. Warmed actual-API medians increase 1.58–1.74 percent small and
2.53–3.83 percent medium (39/40 positive ordinal block deltas). Same-process
reserved tail medians decrease 14.67/21.43 percent small and 1.44/1.44 percent
medium (40/40 negative block deltas). Tail proxies cannot be subtracted from
actual API times; full-API cause remains unresolved. Initial observations vary
sharply and correlate with reported frequency context, without proving the old
YT-154 cause. Original unfavorable evidence remains intact.

One new focused test passes (one old test filtered), format and two new ordinary
release builds pass. Full distributions, all blocks/initial values/context,
capacity identity and limitations are in the linked report and
`target/yt155-small-medium-latency-cr2njref/evidence/measurement-summary.json`.
Retain YT-154's capacity policy; no optimization follows. Final independent
review PASS verifies all eight hashes and data/history/source/executable bindings;
only status/reviewer references changed after acceptance. Product sources/default
graphs and locks are unchanged.
