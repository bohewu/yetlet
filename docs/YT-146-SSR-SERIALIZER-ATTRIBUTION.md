# YT-146 — SSR serializer allocation and copy attribution

Date: 2026-10-23. Status: **Done at bounded attribution-study/proposal scope
after independent final Reviewer PASS, zero issues and required followups.**

## Authority and boundary

The user requested reuse of [YT-145](YT-145-SSR-PERFORMANCE-BASELINES.md), a
contract before minimum new attribution measurements, and a minimum optimization
proposal only after hotspot evidence. This is a study with sample-local
instrumentation, not implementation of a production optimization. Baseline is
clean HEAD `61a20f7e7da36116ed64c9c26249dce93df9bb3c`. Independent contract
Reviewer PASS must precede stage 2; independent final Reviewer PASS is required
before Done. No commit, push, deployment or external environment change is
selected.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Core, HTML and Axum adapter
sources/public signatures, root manifests/lock, browser assets, security policy,
default feature graphs and accepted evidence remain unchanged. No new profiler,
benchmark dependency, framework, CI gate, renderer API, workload ladder,
optimized variant or architectural decision is needed.

## Reused evidence and question

YT-145's exact fixed pages, literal escaped oracle, six output SHA-256 values,
accepted source checks, two ordinary timing repetitions, 36 allocation
observations, memory limits and retained failures remain authoritative. Their
raw artifacts stay intact under `target/yt145-ssr-cost-vpx_z1d5/evidence` and
`target/yt145-ssr-cost-vpx_z1d5/rendered`. Do not rerun its accepted commands,
24 timing groups, workspace/browser/CSP/HTTP/database/streaming proofs or old
counter sanity test. New checks below exercise only the new instrumentation.

Use only the unchanged **large** 1,024-row page for new cost observations: body
244,970 bytes, document 245,207 bytes. Accepted large body creation/release totals
are 7,174 alloc / 5,136 realloc / 7,174 dealloc, allocation bytes 129,120 and
realloc old/new bytes 380,950/761,900, incremental logical peak 262,208 bytes.
Document totals are 7,176 / 5,142 / 7,176, allocation bytes 129,187 and realloc
old/new bytes 626,666/1,498,015, incremental logical peak 507,337 bytes.
Combined owned work has 19,481 allocations, 5,142 reallocations and a
2,149,003-byte incremental logical peak; this is reused context, not a new
construction measurement.

Source inspection identifies three possible allocation sources: public
`escape_text` creates temporary Strings; private URL validation collects a
normalized String for each URL attribute; output Strings grow during body and
envelope composition. The adapter materializes a body and appends it into a
second String. These observations alone establish neither a dominant cost nor
a regression. Ask where observed allocation requests occur and how much
standalone escaping/envelope work costs on the accepted large input.

## Selected minimum method and source binding

Installed `perf` is available; heaptrack and Valgrind are absent. Do not install
profilers or change kernel/host/perf capabilities. A sampling profiler is not
required to answer this bounded allocation-call question. Reuse the sample's
System allocator counter and add a feature-gated, source-derived tagged copy of
the current private serializer/document composition. Call the **actual public
`yetlet_html::escape_text`** under tags; do not duplicate or optimize it.
Copy only the otherwise inaccessible render traversal, identifier/URL checks
and constants, retaining their operation/validation order, formatting, temporary
lifetimes, textarea handling and exact `RenderError` variants. Use actual
`DocumentMetadata` and borrowed `View`. This copy is measurement code, not a
second supported renderer. An independent source comparison is part of final
review; byte equivalence alone does not prove all security behavior.

Pin these original source hashes before changes, together with the YT-145
report, sample manifest/lock and six retained output hashes in the run receipt:

| Original source | SHA-256 |
|---|---|
| `crates/yetlet-html/src/lib.rs` | `4f2222e74b69e231b4445f20dea696360badbad0e18fd7cda8e73a3f06242af3` |
| `crates/yetlet-server-axum/src/lib.rs` | `06c73c3c4e1bfee37c06e3d2b19e641c93832ee028056932b0f6ddd4cd37d370` |
| `crates/yetlet-server-axum/src/document.rs` | `8d1f2a55f3f49f7998e9895807ef0c7fb06270fe2f967d7d6d5f540f7f49491a` |
| `examples/ssr-performance-baselines/src/main.rs` | `008d82ca2fb01f6ecb4813b851282147ac37c2232e6ffcc71b997238e7df17be` |
| `examples/ssr-performance-baselines/src/counts.rs` | `402bae14eae3ef54e440b416a6241b94c93e97bc865b4e2f675ef41b977ffe73` |

The prework 1,025-file manifest is retained in
`target/yt146-ssr-attribution-grvvqfm_/evidence/baseline.json`; source/evidence
bindings are in `reused-source-bindings.json` beside it. A source/hash mismatch
must be reported before interpreting results; it cannot authorize product edits
or a rerun of accepted measurements.

## Allocation populations, buckets and lifetimes

Add sample features `serializer-attribution = ["allocation-counts"]` and
`component-probes = []`, both default off. The existing ordinary/default and
`allocation-counts` interfaces stay intact. Tagged code compiles only with
`serializer-attribution`; ordinary component probes compile without that
feature or any allocator wrapper. Reject a component timing invocation in an
allocator-instrumented build.

Keep fixed nonallocating atomic bucket counters alongside the existing global
counter. A lexical guard saves/restores the prior active tag on normal return,
error and unwind; no allocation, formatting, lock or panic belongs in allocator
bookkeeping or guard restoration. Nested tags are stack scoped, never inferred
from object pointers. A single foreground thread runs each workload; there is
no async runtime or concurrent use. Do not claim thread-safe attribution of
uncontrolled callers merely because counters are atomic.

| Bucket | Allocation/reallocation call sites included |
|---|---|
| text escaping | Actual public escape call for each body text value |
| attribute escaping | Actual public escape call for each attribute value |
| title escaping | Actual public escape call for document title |
| URL normalization | Exact current private `unsafe_scheme` normalization/check |
| body buffer | Body String construction and append/format/growth, after escaping tag restores |
| envelope buffer | Outer String construction and append/growth, after nested title/body tags restore |
| other | Any unclassified call; includes nonfixture textarea normalization/error allocations if exercised |

Wrap only the escape call, not the enclosing output write, so output growth
belongs to its buffer bucket. Body traversal runs under the body-buffer tag,
with nested escape/URL tags; document runs under envelope-buffer with nested
body/title tags. Preserve the original temporary drop boundary after each
append. Snapshot bucket deltas only for creation, not release. Report successful
alloc calls (zeroed as a subtotal), realloc calls, alloc-request bytes and
realloc old/new request bytes, plus failures. Bucket sums must equal global
creation deltas for each field. Report `other` explicitly, including zero.
Do not attribute deallocation, retained/live/peak bytes or RSS to these buckets:
a block can be freed after its allocation tag is restored. Realloc request
bytes do not measure actual physical moves or copies.

New CLI `attribution-counts body|document` runs one fresh process per phase,
three operations each: **6 total operations, no warmup**. Construct one large
metadata/tree before the first region and keep it alive through every release
snapshot. Take global and bucket start, held-output and global released
snapshots in fixed stack storage; reset only the existing region peak to
current live at start. Only the outer region calls `counts::begin`; nested
tags/snapshots never reset the global peak. Creation includes internal temporary destruction, but
excludes returned output destruction. Drop returned output before release;
retain the input until after all regions. No oracle, hash, printing, procfs read
or setup work is counted. Require three identical per-phase deltas, zero
allocator failures, global release live equal start live, and zero unclassified
creation calls on this fixed valid page.

Reconcile global calls, bytes, held/incremental peak and release deltas against
YT-145's accepted **body/document** raw snapshots, not only the table totals.
No old binary/phase rerun is needed. If tagging/compiler changes alter totals,
retain and expose the mismatch, stop causal attribution and request review;
do not force equality, silently exclude calls or tune product code. Even exact
reconciliation establishes this source-derived build's event attribution, not
guaranteed source-level allocation counts for every compiler/build.

## Copy inventory and separate ordinary component timings

Record source-level logical append volumes, not hardware memory traffic: the
adapter appends exactly 244,970 body bytes into the outer String; escape outputs
are separately materialized before append; URL normalization materializes its
own String. Record escape input/output byte totals and call counts from the
fixed corpus outside timers. Realloc old/new request totals are capacity
requests and cannot be relabeled bytes copied. No cache/memcpy bandwidth,
retained-bucket memory or whole-renderer CPU share is inferred.

Use a separate ordinary release build with `component-probes` and no global
allocator wrapper. New CLI `component-timing 1|2` visits just these two probes;
repetition 2 reverses their order. Each group performs 5 warmup and 20 measured
operations: **20 warmups and 80 measured rows total across four groups**.

| Probe | Setup outside timer | Timed region and release |
|---|---|---|
| escape corpus | Borrow title, then depth-first attributes/text from one retained large page; prebuild a Vec of 6,150 borrowed slices, no clones | Invoke actual public `escape_text` for each slice, black-box output/length, sum lengths and drop each temporary inside timer; no output append or tree traversal |
| envelope composition | Read the accepted large body file, verify its accepted SHA-256 before execution, and retain metadata/body | Exact adapter outer String construction, language/title escaping, prefix/body/suffix appends, using borrowed pre-rendered body instead of rendering it; stop with output held, drop returned String after timer |

Both probes use the same title/language/escaping bytes and default release
settings as YT-145. The escaped-corpus setup follows the document call population
(3,075 text + 3,074 attributes + 1 title); tags/language are excluded. Precheck
literal expected escaped lengths/bytes and envelope equality outside timing.
Do not use a preescaped title in the envelope probe. Returning the body from a
retained file removes body rendering and its temporary body release from this
probe; the result measures **envelope composition including title escaping,
allocation and append**, not isolated memcpy or production document time.
Escape corpus timing includes temporary drops, traversal over the prepared
slice list and barriers; it excludes serializer writes, URL checks and View
walk. Neither is a causal share of whole-renderer elapsed time.

Black-box borrowed inputs/results and preallocate raw sample storage. Store
integer `elapsed_ns`, repetition, sequence, probe, population, index and result
length after the timer; print after each group. Keep every observation and
both repetitions separate. Report count, mean, median (average sorted entries
10/11), min/max and nearest-rank p95 (entry 19/20), in ns/op or explicit µs/op.
No subtraction of medians, outlier removal, p99, statistical significance,
whole-renderer CPU percentages or speedup claim is permitted. Tagged-wrapper
and noninlining overhead means attributed-build time is never ordinary timing.
YT-145 timings provide context only; these new probes are not before/after data.

## New equivalence checks and exact footprint

After contract PASS, change only these five sample files, plus these four
stage-1 documentation files for outcomes:

```text
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/counts.rs
examples/ssr-performance-baselines/src/attribution.rs (new)
```

The new module is necessary solely for source-derived attribution and the two
component probes. Keep the standalone lock, dependencies, root workspace and
default-empty feature configuration unchanged. No new script/tooling file,
generic tag framework, dependency download or optional-profiler integration.

Add only focused new tests, selected by the common `yt146_` filter:

- All three unchanged fixture sizes: tagged body/document equals actual library
  and existing independent literal oracle byte-for-byte. Emit six new shadow
  outputs outside regions; compare their SHA-256 to the six accepted YT-145
  values. This checks new copy equivalence, not a rerun of the accepted test.
- One finite differential corpus for copied validation: invalid/forbidden tags,
  invalid/duplicate/forbidden attributes (`onclick`, `style`, `srcdoc`), mixed
  case/control-obfuscated `javascript:`, `vbscript:` and `data:` URLs, void
  children, nested forms, invalid textarea/option/select children. Require exact
  same `Result<String, RenderError>` as the library. Include valid escaped
  attribute/text and textarea CR/LF/initial-newline cases. No browser/CSP claim.
- Lexical nested-tag restoration on normal and error return plus caught unwind;
  known tagged allocation/reallocation checks reconcile bucket/global deltas
  without rerunning the old known-layout test. Run tests serially.
- Ordinary component probe correctness: fixed corpus count/literal escaped
  values/length totals and full envelope output equal accepted literal oracle.

Final source review checks all copied branches/constants and temporary
lifetimes against the pinned originals, including paths outside the measured
fixture. No new production security acceptance or general serializer claim is
made. Existing security/CSP acceptance is reused.

## Fixed execution limits and receipts

Before stage-2 implementation, record a UTC/monotonic start and a fixed
**30-minute deadline**. Stop at it; do not renew bounds automatically. The
measurement batch has a separate **60-second** monotonic ceiling and **8 MiB**
raw stdout/stderr/receipt budget. No retries, workload expansion, old phase
reruns or size ladder follow a failed/partial observation. Retain failures and
unknown exits explicitly; a repair requires the materiality policy and review
of the unmet original requirement.

Keep source, binaries, raw outputs and receipts under
`target/yt146-ssr-attribution-grvvqfm_` on actual ext4. Verify evidence and both
new build directories with `findmnt -T`. Reuse recorded CPU 0 affinity, Rust/Cargo
1.99, default release/System allocator and existing host/cgroup limits; record
current identities/settings and stop if incompatible. Do not change CPU policy,
quota, allocator, RUSTFLAGS, LTO/PGO, kernel or host capabilities. Every test and
workload executable uses CPU 0 via `taskset` and a 1 GiB address-space limit;
build jobs are 2. Build/test commands time out at 600 s each; each of the two
component processes at 15 s and each of the two count processes at 10 s.

Use new separate target directories `build-components` and `build-attribution`;
never overwrite YT-145 binaries. Minimal command plan (all sample Cargo
commands use the manifest shown, offline and locked):

```text
cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml --check
cargo test --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --features serializer-attribution yt146_ -- --test-threads=1
cargo test --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --features component-probes yt146_ -- --test-threads=1
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features component-probes --jobs 2
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features serializer-attribution --jobs 2
<attribution-binary> attribution-output <existing-new-output-directory>
<attribution-binary> attribution-counts body
<attribution-binary> attribution-counts document
<component-binary> component-timing 1
<component-binary> component-timing 2
cargo xtask check docs
git diff --check
```

The new `attribution-output` command writes only the six shadow outputs for the
new equivalence check. Bind exact argv, selected nonsecret environment, toolchain,
source/contract/lock and binary SHA-256, sizes, start/end UTC/monotonic times,
native exit, full stdout/stderr and timeout status **before analysis**. Reuse
old raw count records and output files read-only. Inspect sample feature
metadata to establish ordinary probe exclusion of the allocator; compare all
protected tracked-file hashes to baseline rather than rebuilding consumers or
rerunning their old dependency proof. Logical global heap snapshots remain
separate from RSS; no new RSS run is needed for this allocation-source question.

Run finite foreground processes only. Start no server, browser, watcher,
container or service. Record/wait owned process identities, terminate only owned
children on timeout and verify none remain. Retain inert build/output/evidence
files; cleanup must not delete failures. Stage 1 permits only canonical docs and
whitespace checks and then stops for contract review.

## Acceptance, limits and next action

Completion requires reviewed contract, only the bounded new tests/builds and
observations above, all six new output hashes, explicit allocation reconciliation,
all raw/failed populations and cleanup, and independent final Reviewer PASS.
No numerical performance threshold or minimum hotspot magnitude is required.
A mismatch may produce an honest incomplete study; it never licenses silent
replacement evidence or a Done claim.

After evidence, rank the observed allocation-call/request-byte concentrations
separately from component timing. Propose **at most one** smallest follow-up
optimization only if supported by those observations. For example, measured
escape-temporary concentration could justify considering a private escape-into
helper with the existing public `escape_text` wrapper retained; this is not yet
a selected solution. URL validation or envelope work may warrant a different
proposal, or the evidence may justify none. State the exact source hotspot,
confidence, tradeoff and proposed future changed-behavior tests. Preserve all
escaping, validation, output bytes, error behavior, public signatures and
security boundaries. Do not implement the proposal or rerun its future tests
in this study. No regression, stable cost, CPU causal share, production SLO or
architecture promotion is established by these synthetic local observations.

At the stage-1 checkpoint there were no new measurements or optimization
recommendation; the next action was independent contract review. YT-145 remains
Done and YT-096/YT-020 release gates retain their existing status.


## Actual implementation and execution

Independent contract Reviewer
`/root/yt146_ssr_attribution_contract_review_20261023` returned **PASS, zero
issues and required followups** before implementation. The accepted hashes,
source/evidence bindings and verdict remain retained in the evidence directory.
Only the five planned sample files and four documentation files changed. The
standalone lock, dependencies and all protected root/Core/HTML/adapter/browser
inputs are unchanged. The ordinary resolved sample features are precisely
`component-probes` and empty `default`; its build excludes `counts` and the
allocator wrapper. Attribution alone enables `allocation-counts`. No optimized
variant or product code was added.

The new serial checks passed: three `yt146_` attribution tests (all-size exact
copy/oracle equivalence, finite security/error/textarea differential corpus,
nested normal/error/unwind tag restoration and tagged allocation reconciliation)
and one `yt146_` ordinary component-literal test. Old tests were filtered out.
Sample formatting and both offline locked release builds passed without repair.
The source-derived serializer retains all original branches/constants, with
lexical wrappers for event tagging; its escape calls use the real public helper.
Independent final source review passed, with zero issues and required followups.

Execution used Rust/Cargo 1.99.0, Linux 7.0.0-38-generic, x86_64 Intel Core
i5-13600K, CPU 0 and a 1 GiB per-command address-space ceiling. Evidence and
both fresh build directories are on ext4. Cgroup leaf/ancestors retain unlimited
CPU/memory where controllers exist; no host settings changed. The first new
attribution test used recorded POSIX `sched_setaffinity` pre-exec setup rather
than the contract's literal `taskset` invocation; effective affinity was CPU 0
and the same 1 GiB bound. This execution-form deviation is disclosed, and that
passing test was not replayed. The component test and all workload commands
used `taskset -c 0` explicitly. Build jobs remained 2; default release and System
allocator settings were retained, without RUSTFLAGS/PGO/LTO changes.

Stage 2 began at **14:57:01 UTC**, with the fixed **15:27:01 UTC** deadline.
The five foreground output/count/timing commands ran once at 15:02:31 UTC,
taking **0.060 s** total, with **12,254 bytes** of raw stdout/stderr/receipts
under the unchanged 60-second/8 MiB bounds. All native exits were 0, without
timeouts, partial groups or retries. Raw source/binary/argv/environment/time/exit
receipts preceded reconciliation and summary derivation. No sample was removed.
No old accepted test, measurement group or binary was replayed. Original YT-145
artifacts and failures remain intact.

Ordinary component binary SHA-256: `19a6274734749c7edb3159ceb821aaf3ff61e16ed6f33531973fdb0fc3806e7a`
(555080 bytes). Attribution binary SHA-256:
`b9d73a7963768db60a79bc19335306218500c8460ff650858444e3293b32092e`
(599576 bytes). These sizes reflect different sample feature/instrumentation
contents; they are not product performance comparisons. `premeasurement-binding.json`
records the contract/source/lock/binary and reused-output identities.


## New allocation-call attribution results

All six new shadow outputs match the six accepted YT-145 SHA-256 values.
Each body/document count process performed three operations. Every creation
and release counter delta, held/live increment, region peak increment and
released-live increment matches its corresponding accepted YT-145 raw snapshot.
Absolute process live baselines differ because CLI/setup differs; they were
not compared as phase costs. All three operations per phase are identical,
with zero failures, zero zeroed allocations and zero unclassified creation
calls. All releases return to their own start live baseline. The independent
raw comparison is retained in `reconciliation-body.json` and
`reconciliation-document.json`; no old count command was needed.

The table gives **document creation call-site** deltas, identical across three
operations. Body has the same text/attribute/URL/body rows, with no title or
envelope events. `A bytes` and realloc old/new bytes are cumulative requested
sizes, not physical bytes copied, retained memory or per-bucket peak usage.

| Bucket | Alloc | Realloc | A bytes | Realloc old bytes | Realloc new bytes |
|---|---:|---:|---:|---:|---:|
| text | 3,075 | 1,025 | 54,343 | 31,774 | 63,548 |
| attribute | 3,074 | 2,048 | 66,577 | 62,464 | 124,928 |
| title | 1 | 1 | 40 | 40 | 80 |
| url | 1,024 | 2,048 | 8,192 | 24,576 | 49,152 |
| body | 1 | 15 | 8 | 262,136 | 524,272 |
| envelope | 1 | 5 | 27 | 245,676 | 736,035 |
| other | 0 | 0 | 0 | 0 | 0 |
| Total | 7,176 | 5,142 | 129,187 | 626,666 | 1,498,015 |

Escaping accounts for **6,150 allocation calls (85.70%)** and **3,074 realloc
calls (59.78%)** in document creation: 3,075 text escapes, 3,074 attribute escapes
and one title escape. URL normalization contributes 1,024 allocations and
2,048 reallocations. Output buffers have just two allocations and 20 reallocs,
but account for **1,260,307 realloc-new requested bytes (84.13%)**. This ranks
call frequency separately from request-volume concentration; neither ranking
establishes CPU cost, physical movement, retained-bucket memory or a regression.

Body global held/peak increments remain **262,144/262,208 bytes**; document
increments remain **490,386/507,337 bytes**. These are global operation-local
logical heap values, not sums of bucket peaks or RSS. The envelope's five
reallocations carry 736,035 new-request bytes. Source order appends the final
14 closing bytes after the full body; aggregate requests are consistent with
outer-buffer growth at that point. No per-realloc trace was recorded, so these
totals do not locate each individual growth or measure its physical copy cost.

The fixed corpus contains **120,960 raw UTF-8 bytes** and **182,456 escaped
fragment bytes** over 6,150 calls. Source-level appends also submit **244,970
body bytes** into the outer String. These are logical payload volumes; they
exclude allocator-internal moves and do not combine into a hardware-memory
traffic measurement. URL normalization is an additional temporary String, not
part of the escape corpus timing. RSS remains reused YT-145 context only.

## Separate ordinary component observations

All **80 measured** observations and **20 warmups** remain in the two raw CSVs;
there are 20 measured entries and five warmups in each group. Repetition 1
visits escaping then envelope; repetition 2 reverses order. Values below are
**µs/op**, with integer ns/op retained in raw data and `measurement-summary.json`.
No subtraction, outlier filtering or aggregation across repetitions occurred.

| Repetition | Probe | Median | Mean | Empirical p95 | Min–max |
|---|---|---:|---:|---:|---:|
| 1 | escape-corpus | 238.875 | 241.925 | 254.308 | 234.447–256.980 |
| 1 | envelope | 3.721 | 3.955 | 4.285 | 3.668–5.180 |
| 2 | escape-corpus | 252.487 | 253.128 | 259.076 | 244.412–269.656 |
| 2 | envelope | 3.955 | 4.219 | 4.239 | 3.908–8.365 |

Escape corpus includes the real helper's temporary creation/destruction and
slice iteration/barriers; it excludes View traversal, URL checking and output
writes. Envelope includes title escaping, allocation and append around the
borrowed pre-rendered accepted body, with the returned String drop outside the
timer. Its retained body and cache context differ from production document
composition. Neither probe measures isolated memcpy, whole-renderer causal
shares or a before/after speedup. Instrumented elapsed time was not used.
The two short order-reversed repetitions remain descriptive local observations,
without confidence, stable per-row, HTTP capacity or production-SLO claims.

## One minimum future proposal, without implementation

The measured escape allocation-call concentration supports considering one
bounded follow-up: a **private `escape_into(input, output)` helper in
`yetlet-html`**, retaining the existing public `escape_text(&str) -> String`
wrapper and exact character mapping. Only internal serializer text/attribute
writes would append escaped content directly to the existing body buffer;
textarea normalization, validation/order/errors and the adapter's public escape
call remain unchanged. This targets the **6,149 body escape temporaries**
observed here, without a new public API or cross-crate destination-buffer
plumbing. The data supports investigating those temporaries, not a promised
reduction in elapsed time, RSS, output capacity or global peak. URL-normalization
allocations and document/body materialization would still exist.

The tradeoff is helper reuse and possibly different buffer-growth behavior
instead of per-value intermediate Strings. If separately activated, future
acceptance should check exact text/attribute/textarea escaping for empty,
multibyte and all five metacharacters; unchanged invalid-name/URL/children/error
behavior; unchanged public `escape_text` output/signature; and all six fixed
body/document byte oracles. A new focused matched ordinary body/document
comparison and allocation observation could then establish actual effects on
calls, output growth and time, without assuming this study's component timings
predict them. Those future change tests/measurements were **not run here**.
No optimization code, second candidate, new task activation or architecture
change follows from this proposal.

## Completion boundary and evidence

The run-local evidence directory is
`target/yt146-ssr-attribution-grvvqfm_/evidence`. It retains source/binary bindings,
all native command stdout/stderr/receipts, six-output comparison, both raw count
populations, both raw timing populations including warmups, per-phase accepted
snapshot reconciliation, derived summary and original contract review. There
were no failed new builds/tests/workloads or repair attempts; the caught panic
in the tag-restoration test is intentional test input, not a discarded failure.
The stage-1 native no-index new-file exit 1 remains recorded with empty
whitespace diagnostics. Historical failures were not replaced or reclassified.

All commands are finite foreground work. No server, browser, watcher, container
or service was started. Recorded leaders are waited/joined; cleanup checks for
remaining owned sessions, separately from the measured results. Inert build and
raw evidence files remain retained. Final tracked-file comparison, canonical
docs/whitespace checks and cleanup receipts accompany `implementation-summary.json`
and `final-files.json`. Independent final Reviewer
`/root/yt146_ssr_attribution_final_review_20261023` returned **PASS, zero issues
and required followups**, after checking copied source, results, failures/limits,
protected inputs and cleanup. All nine reviewed file hashes matched. The verdict
and reviewed snapshots remain in `final-review-result.json` and
`final-reviewed-files.json`. **YT-146 is Done at bounded attribution-study/proposal
scope.** The next possible work is the separately selected private escaping
optimization; no implementation, task activation, commit or push follows.
