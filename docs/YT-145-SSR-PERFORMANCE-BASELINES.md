# YT-145 — Small, medium and large SSR cost baselines

Date: 2026-10-23. Status: **Done at the fixed local SSR baseline scope after
independent final Reviewer PASS, zero issues and required followups. Independent
contract review preceded implementation and measurement.**

## Authority and question

The user requested a contract followed by minimum small/medium/large SSR
measurements of duration, memory and allocations; reuse accepted verification,
retain failed results and record discovered gaps for possible future work.
Necessary sample-local allocation instrumentation is explicitly authorized.
Baseline is clean HEAD `5a9140f064e009140cc07f813d46143cb0e861b1`, after the
separately authorized YT-144 commit. This task authorizes no new commit, push,
deployment, optimization or unrelated fix.

Measure current owned component/View construction, existing HTML rendering and
escaping, complete-document composition and combined owned-page work. This is
a local synthetic CPU/heap baseline, without HTTP, concurrent capacity, browser,
database, Auth, optional-provider or customer workload claims. No numerical
performance threshold, superiority requirement, linear-scaling gate, hardware
recommendation or consumer SLO is selected.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0006](adr/0006-handler-first-mvc-optional.md) and [Naming](NAMING.md).
Do not change Core, HTML, adapter APIs, security policy, browser assets, root
membership/manifest/lock, default features or CI. No escaping bypass, raw-HTML
API, mandatory instrumentation, generic benchmark runner or validator framework
is needed. A component here is an ordinary application function returning
`View`; there is no new Component trait or compiler.

## Reused evidence and actual source findings

- [YT-143](YT-143-AXUM-WORKLOAD-COMPARISON.md) exercises a 428-byte escaped SSR
  document through matched HTTP routes. Its HTTP latency and retained failed
  pair do not isolate construction, serialization or heap costs.
- [YT-144](YT-144-POSTGRES-WORKLOAD-COMPARISON.md) exercises SELECT-to-JSON;
  its accepted checks and failures remain intact, without SSR cost attribution.
- [M5 performance evidence](M5-VERIFICATION.md) and the
  [M7j repeated release baseline](M7J-VERIFICATION.md#repeated-release-baseline)
  record HTTP first-byte/browser timings, sizes and explicit sample limits.
  Current `xtask/src/measure/baseline.rs` uses native `Instant`; its historical
  Python counterpart is a frozen oracle, not the current command entry point.
- [YT-021](YT-021-STREAMING-SSR-RESEARCH.md) retains 100/10,000-row full versus
  incremental HTTP measurements. It explicitly leaves tree/serializer allocation
  and peak memory unmeasured. Do not rerun it or promote streaming here.

Current `yetlet-core::Element` owns attribute/child Vecs and dynamic Strings.
`yetlet_html::render_to_string(&View)` borrows the tree, validates names/URLs and
children, escapes dynamic values and allocates its result. Its `escape_text`
returns a temporary String. `render_document_with_metadata(&metadata, &view)`
creates an outer String, escapes the title, obtains a complete body String and
appends it before returning the full document. These are source observations,
not measured allocation counts or proof of a regression. The public static
SSR consumer in YT-143 has no serialized hydration state; adding such state
would introduce a different workload. This fixture therefore has none.

## Fixed pages and independent correctness oracle

Use exactly 8, 128 and 1,024 rows, IDs `0000` through `N-1` padded to four ASCII
digits. All cases use the same semantic, static-SSR report. Do not add an
arbitrary-size/seed/text-depth parameter framework. The only case selector is
`small|medium|large`.

The raw document title is `YT145 public report <&> "quoted" 'café'`, language
`en`. The tree in source order is:

```text
main id="ssr-report"
  header
    h1: SSR report <&> "title" 'café'
    p: Public synthetic rows
  section aria-label="Records"
    ul
      N × li data-row="IIII" title="Row IIII <&> \"quoted\" 'café'"
        h2: Record IIII
        p: Public <&> "value" 'café' IIII
        a href="/records/IIII?view=full&public=1": Open record
  footer
    p: End of public report
```

Here `IIII` denotes the fixed-width ID, not literal content. Attributes store
raw quotes; the diagram's backslashes only disambiguate the quoted title.
There are no whitespace text nodes. One page function invokes one row function
per row. Root `main` is depth 1, deepest text depth 6. Nodes count elements and
text, not metadata, attributes, document-envelope markup or Rust containers.
Owned payload bytes sum all View text/attribute String lengths plus metadata
title/language lengths; capacities, tags, Vec storage and stack bytes are
excluded. This count describes the input, not allocated heap usage.

| Case | Rows / row components | Elements | Text nodes | View nodes | Attributes | Owned UTF-8 payload bytes | Body bytes | Document bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| small | 8 | 40 | 27 | 67 | 26 | 1,074 | 2,146 | 2,383 |
| medium | 128 | 520 | 387 | 907 | 386 | 15,234 | 30,826 | 31,063 |
| large | 1,024 | 4,104 | 3,075 | 7,179 | 3,074 | 120,962 | 244,970 | 245,207 |

These are premeasurement literal arithmetic, subject to the exact independent
oracle below; any mismatch stops measurement until the contract is reconciled.
Freeze literal expected body/document fragments and their UTF-8 SHA-256 values
before implementation in run-local `contract-workload-arithmetic.json`.
The sample's focused test must independently assemble expected HTML from fixed
already-escaped literals and padded IDs, without calling Yetlet's renderer or
escape function. Use the existing adapter's exact doctype/head/envelope.
Compare every size byte-for-byte and assert the fixed counts/lengths above.
Retain actual rendered output and independently check its SHA-256 against the
frozen values outside measured/count regions; do not time hashing or oracle
construction.


| Case | Body SHA-256 | Document SHA-256 |
|---|---|---|
| small | `39008f753d68bc1cfa95af51281dd8182fbf5a25e1c73992e2d9cfdeb823ffaf` | `976f185de09dd60901b11a3a4325ea2d4d854c1077d2a26db6eb9df3a025dff6` |
| medium | `3ccd90de1b641bcdef854f4ca8a8f9ade8b4148be3ac1d80f5f2c57f20ba584e` | `b27ae2d7b741bc0b0b59a7b51d75d5270a2d1c3f7f6b1812f4875422b93097bb` |
| large | `25fedaaf88a65ec2c3c016fcd4545a58d345c5209640d37a2f3497cdd9557c56` | `3e01ffaa34dc450f5b648e7b3d64ebcaa4d2aa1fda6623753386a3d17ec58734` |

Dynamic `<`, `>`, `&`, double/single quotes and multibyte `é` occur in title,
text and attributes; the link query ampersand must become `&amp;`. The exact
oracle proves escaping and no injected markup or scripts for these inputs.
Preserve existing validation and default CSP; this fixture measures document
Strings, not HTTP response conversion or browser CSP enforcement. Previously
accepted security/CSP tests are reused, with no new browser claim.

## Phase boundaries and lifetimes

Generate dynamic row Strings and owned metadata inside construction from the
case's borrowed row count and fixed raw tokens. No row-data cache or unreported
clone substitutes for that work. A prebuilt tree is used only where stated.
Pass case inputs and borrowed trees through `std::hint::black_box`; consume
results through it as well. The [standard documentation](https://doc.rust-lang.org/std/hint/fn.black_box.html)
describes a best-effort optimization barrier, not a guarantee about source-level
allocation calls.

| Phase | Timed work | Setup outside timer | Release boundary |
|---|---|---|---|
| construct | Build owned metadata and View tree, including row formatting/owned Strings | Case selector/raw fixed tokens only | Stop timer with metadata/tree held; drop both afterward |
| body | `render_to_string(black_box(&view))` and consume result | Build one retained tree/metadata before warmup | Stop with output held; drop output afterward; retained input drops after case |
| document | `render_document_with_metadata(black_box(&metadata), black_box(&view))` and consume result | Same retained borrowed tree/metadata | Stop with document held; drop output afterward; retained input drops after case |
| combined | Construct metadata/tree, render document, consume output and explicitly drop document/tree/metadata | Case selector/raw fixed tokens only | All owned work and drops are inside timer |

No hidden tree clone, output accumulation, oracle comparison, printing, CSV
formatting or procfs read belongs in a timed region. Body/document creation
includes their internal temporary drops; their returned String drop is excluded
and reported as such. Construction excludes tree destruction. The combined
phase includes all three owned lifetimes; it is not inferred by adding phase
medians. Document minus body timing is not a pure envelope estimate because the
actual adapter renders a body and copies it into the outer String.

## Tool selection and timing population

Read-only primary-source comparison on 2026-10-23:

| Option | Fit for this task | Selection |
|---|---|---|
| [Criterion 0.8.2](https://docs.rs/criterion/0.8.2/criterion/) | Maintained statistical microbenchmark API, WallTime and explicit setup/batching; useful for later change comparisons | More machinery/dependencies than this finite descriptive baseline needs |
| [Divan 0.1.21](https://docs.rs/divan/0.1.21/divan/struct.AllocProfiler.html) | Integrated allocation profiling; its docs explicitly say counting affects timing and uncontrolled threads are uncounted | Do not mix instrumented timing with normal runtime |
| [stats_alloc 0.1.10](https://docs.rs/stats_alloc/0.1.10/stats_alloc/struct.Stats.html), [allocation-counter 0.8.1](https://docs.rs/allocation-counter/0.8.1/allocation_counter/) | Mature allocation totals/regions or live/peak counters | Neither documented public result alone supplies this contract's distinct alloc/realloc/dealloc and region-peak fields |
| Native `Instant` plus sample-local counted build | Existing repository timing pattern; fixed finite observations; necessary counter fields with no dependency | Selected minimum |

None of those measurement crates is locally cached. No crate is downloaded or
pinned for this selection. Use the installed Rust/Cargo 1.99.0 toolchain and
its default optimized release/System allocator build; record actual `-Vv`,
lock/checksum/feature/binary identities. No alternate allocator, PGO, LTO,
RUSTFLAGS, global toolchain or performance-governor changes. Freeze the standalone
resolved lock before build; preserve root/default dependency inputs.

Use one process per timing repetition. Repetition 1 visits small/medium/large,
with construct/body/document/combined in that order; repetition 2 reverses both
orders. Each of the 24 case/phase groups gets exactly 10 unreported-as-measured
warmup operations and 50 measured single operations. Retain warmup durations
with their separate label: 240 warmups and 1,200 measured rows total. Store each
integer `elapsed_ns` immediately after its operation in preallocated storage;
write raw CSV after the group. Include repetition, sequence, size, phase,
warmup/measured, index and output/node checksum or length as appropriate.
Output correctness is checked outside timing, before a group's workload.

Use [Instant](https://doc.rust-lang.org/std/time/struct.Instant.html) elapsed
wall time; retain timer/barrier overhead without subtraction or calibration
claims. It includes scheduling/preemption and platform clock limitations.
For each group report count, arithmetic mean, median (average sorted entries
25 and 26), min/max and empirical nearest-rank p95 (sorted entry 48 of 50),
in ns/op or explicitly converted µs/op. Keep both repetitions separate; no
best-run selection, outlier removal, p99, bootstrap confidence interval,
statistical-equivalence or cross-machine claim. These are single-operation
observations, not Criterion estimates, HTTP latency or a throughput ceiling.

## Allocation and memory semantics

After contract PASS, add one sample-only `allocation-counts` feature, default
off, compiling `src/counts.rs`. Build normal timing and counted executables in
separate target directories. The default binary has no global-allocator wrapper;
never use counted-build duration as ordinary runtime timing.

The counted build delegates `alloc`, `alloc_zeroed`, `realloc` and `dealloc`
directly to `std::alloc::System`, preserving pointer/Layout contracts. The
[GlobalAlloc safety contract](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)
requires nonallocating, nonunwinding bookkeeping. Use fixed atomic integer
counters, no logs, locks, formatting or allocation inside allocator methods.
Keep unsafe delegation entirely in the sample; introduce no Core allocator API.
This measures observed calls in this compiled program, not guaranteed counts
of source constructs; compiler optimization may remove allocations.

Record successful allocation calls (including a separate alloc_zeroed subtotal),
successful reallocations, deallocations and failed allocation/reallocation
calls. A failed realloc leaves the old live block intact. Record allocation
requested bytes, realloc old/new requested bytes separately and deallocated
requested bytes. Live requested bytes add successful allocations, change by
`new_size - old_size` on successful realloc and subtract deallocated layouts.
Realloc is one realloc event, not an invented alloc/dealloc pair; its logical
peak does not include an allocator's transient physical copying of both blocks.

Track process-global current logical bytes continuously, including before a
region. At each region start snapshot current live bytes/counters and reset
only the region high-water mark to that current live value. Never zero current
live bytes while retained tree/input allocations exist. Read start, held-result,
and after-release snapshots without heap allocation. Region peak minus its
start live value is the incremental maximum above that same region's baseline,
not a difference of cumulative process maxima. Capture separate creation and
release counter deltas. Construct's held result is metadata/tree; body/document
hold their borrowed input across all regions and hold one returned String at
the creation snapshot. Combined holds metadata/tree/document for a snapshot
then drops all before its end. Inputs that predate a render region must outlive
its release snapshot; their frees cannot pollute that region's deltas.

Run one fresh counted process per size/phase: 12 processes, each performing
three individual counted operations with region reset and complete per-operation
release. No count warmup, threads or async runtime. CLI/setup allocations,
oracle generation/comparison, procfs reads and output formatting stay outside
regions. Require creation/release balance relative to each region's own start,
zero failed allocator calls and identical observed counts across the three
operations, or retain and report the discrepancy without performance tuning.
One tiny direct known-layout alloc/realloc/dealloc sanity test is sufficient;
it is sample-local counter verification, not a harness certification project.

Record startup and post-work `VmRSS`/`VmHWM` from `/proc/self/status` outside
regions and `/usr/bin/time -v` maximum resident set for each fresh counted
process. The [Linux proc documentation](https://docs.kernel.org/filesystems/proc.html)
distinguishes current RSS and peak RSS and describes accounting limits. These
whole-process observations include executable/runtime, allocator overhead,
retained pages, instrumentation and startup. Report absolute values and the
startup baseline; never subtract two VmHWM/RSS maxima to attribute stage peak.
Logical requested heap excludes allocator metadata, fragmentation, stacks,
code/shared mappings and OS page accounting; String length is not capacity.
Use logical region peaks for phase attribution and RSS only for process context.

## Bounded execution, evidence and checks

Stage 2 may add exactly these files after independent contract PASS:

```text
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/Cargo.lock
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/counts.rs
```

Use `publish = false`, its own empty `[workspace]` and default-empty features.
Direct dependencies are path-only Core/HTML/server-Axum; output can be fixed
CSV/JSON without a new serialization or benchmark dependency. The four stage-1
docs may subsequently record actual results. No workspace registration, root
lock change, sample-catalog promotion or other product files are necessary.

All raw data, binaries/build output and receipts live under
`target/yt145-ssr-cost-vpx_z1d5` on actual ext4, verified with `findmnt -T` for
the evidence and both build directories before execution. Never use `/tmp`
for raw output. Preserve actual argv, selected nonsecret environment, start/end
UTC/monotonic time, native exit status, complete stdout/stderr and every failed
check/point before deriving summaries. Missing exits remain unknown. Freeze
source/contract/lock/binary hashes; retain original outputs and amend summaries
without replacing failed evidence.

Use one allowed logical CPU selected and recorded before measurement, unchanged
between cases, via `taskset`; record topology/affinity, kernel, CPU model, load,
memory and cgroup CPU/memory limits. Single-thread work needs no application
quota; verify an existing quota, if any, allows at least one full CPU. Do not
modify host limits. Inherit recorded host limits, add a 1 GiB per-command
address-space ceiling for measurement executables, and stop if the environment
cannot support these bounds. Build jobs are 2; each build/test command has a
600 s timeout. Each timing process has a 30 s timeout; each counted process a
10 s timeout. The complete measurement batch stops at 180 s monotonic elapsed
or 32 MiB raw output; there is no retry ladder, case-size change or extension
after an expired bound. Implementation/acquisition/build/check work has a
30 minute wall-clock ceiling from stage-2 start, recorded before work. Timeouts,
resource failures and partial output are retained rather than called results.

Minimum new verification is the sample's all-three-size exact output/count test
and the one counter sanity check, a normal and counted locked release build,
sample format check, `cargo xtask check docs` and `git diff --check`. Use actual
commands with `--manifest-path examples/ssr-performance-baselines/Cargo.toml`,
`--locked` and the selected feature explicitly; no old passing workspace,
browser, CSP, YT-143/144 or streaming suite is replayed. Prove protected source,
root lock/manifests/default feature graph inputs and browser assets unchanged
against `baseline.json` hashes; unchanged graph inputs reuse accepted graph
proof rather than rebuilding all consumers. Inspect standalone resolved graph
and feature metadata to confirm isolation, without a new validator.

All commands are finite foreground processes; start no server, browser, watcher,
container or service. Track command process identities and timeout outcomes,
join/terminate owned children on failure and verify no owned process remains.
Retain cleanup evidence separately from completed measurement. Do not delete
raw evidence as cleanup. Independent final read-only Reviewer must assess the
source-bound results, failures/limits, hashes and cleanup before YT-145 is Done.
Stage 1 stops at independent contract review; build and measurement are not
stage-1 documentation checks.

## Gap ledger and current outcome

| Observation / source | Impact | Confidence / classification | Possible separately selected future work |
|---|---|---|---|
| YT-143's fixed 428-byte HTTP SSR page; YT-144 is SQL-to-JSON | Existing HTTP comparisons cannot explain SSR construction/heap costs | High, evidence coverage gap; no proven regression | This authorized fixed-page measurement |
| `escape_text` returns temporary Strings and document composition materializes a body before copying | Possible allocation/copy cost; magnitude not measured yet | High source observation; performance impact unknown | Record measured evidence first; no optimization here |
| YT-021 does not measure peak memory or isolate row/serializer allocation | Streaming memory/cancellation claims remain unsupported | High, retained research limit | Separate realistic streaming-memory question only if requested |
| No current static-SSR consumer requires serialized hydration state in this fixture | Results do not cover island/state encoding, browser usability or optional modules | High, declared workload limit | Separate consumer-bound state/browser/module study if requested |

After measurement, append only observed gaps with raw source/measurement
reference, practical impact, confidence and future candidate. Distinguish
research limits from a demonstrated product regression. This ledger registers
possible future planning without new automatic gap IDs, task activation,
roadmap expansion or fixes. At stage 1 there was no measured result or proven regression. The results
below complete the accepted measurement; no product regression is established.
Next unblockable work is final independent review of this bounded delivery.

## Actual implementation and retained execution
Independent stage-1 contract review returned PASS with zero issues and required
followups before implementation. The accepted document is retained as
`contract-reviewed-document.md`, its verdict as `contract-review-final.json`
and trace as `contract-review-trace.json` in the evidence directory. That gate
accepts the contract only; final delivery review remains pending.
All five proposed sample files were added, without other product changes.
The standalone offline lock resolves 55 packages including the sample, with
path Yetlet 0.1.0 and Axum 0.8.9. Selected transitive versions include Tokio
1.53.3, Serde 1.0.229, Serde JSON 1.0.154, Tower 0.5.3 and libc 0.2.191.
Some standalone transitive versions differ from the root lock; exact versions,
registry checksums and resolved default metadata are retained. Root inputs
remain unchanged. The sample default feature is empty; `allocation-counts`
is absent from its resolved ordinary graph and enabled only in the separate
counted build. No new timing/counting crate was acquired.
Rust/Cargo 1.99.0, Linux 7.0.0-38-generic, x86_64, Intel Core i5-13600K;
CPU 0 affinity and default release settings were fixed. Actual cgroup leaf
and ancestors report unlimited CPU (`max 100000`) and memory (`max`); the root
has no `cpu.max`/`memory.max` file. The initial root-file probe errors are
retained in `stage2-host.stderr`, not presented as application failures. No
host settings changed. Each measured executable inherited the recorded limits
and received the 1 GiB address-space ceiling and stated external timeout.
Stage 2 started at 14:25:58 UTC with a fixed 14:55:58 UTC deadline. The
measurement batch ran at 14:30:21–14:30:22 UTC, taking 0.646 s wall time across
all 14 foreground command groups, under its unchanged 180 s budget. Raw
measurement/receipt files total 100,478 bytes under the 32 MiB ceiling. Both
repetitions and all 12 fresh counted processes exited 0; nothing was rerun or
discarded. Six independently frozen body/document hashes match actual rendered
files. Raw output/exit receipts were written before summary derivation.
Normal executable SHA-256: `9e981257a7c3f1bbea3bb0e913dc9ea43a50d5ae21324bf25c151f0af8fe2b95`.
Counted executable SHA-256: `16bf8cf06a86eed7b7749ff958ee2542e93bce703a3e8d384daa30e26d0f1631`.
Source, lock, accepted contract and binaries are bound in
`premeasurement-binding.json`; exact commands/environments and complete native
output are retained alongside it. Binaries are 537,464/544,512 bytes respectively;
that difference reflects instrumentation, not a Yetlet-versus-native comparison.
## Uninstrumented duration results
Each row contains all 50 measured operations; 10 additional warmup rows per
group remain separately retained. Units are **µs/op**, rounded here to three
decimals; integer nanoseconds remain in `timing-1.stdout` and `timing-2.stdout`.
`measurement-summary.json` contains exact derived statistics. No sample was
removed. Construction/body/document exclude their returned owned-object drop;
combined includes metadata/tree/document destruction.
| Repetition | Case | Phase | Median | Mean | Empirical p95 | Min–max |
|---|---|---|---:|---:|---:|---:|
| 1 | small | construct | 2.228 | 2.257 | 2.325 | 2.155–3.360 |
| 1 | small | body | 7.316 | 7.311 | 7.387 | 7.145–7.400 |
| 1 | small | document | 7.452 | 7.694 | 8.171 | 7.118–21.176 |
| 1 | small | combined | 11.488 | 12.308 | 16.216 | 11.321–18.014 |
| 1 | medium | construct | 40.971 | 40.692 | 66.010 | 26.770–67.626 |
| 1 | medium | body | 83.586 | 83.375 | 87.989 | 77.528–93.127 |
| 1 | medium | document | 84.710 | 84.897 | 88.278 | 81.828–90.122 |
| 1 | medium | combined | 125.672 | 126.099 | 130.248 | 122.552–131.684 |
| 1 | large | construct | 207.363 | 279.448 | 423.140 | 201.807–423.917 |
| 1 | large | body | 683.977 | 686.476 | 711.869 | 656.444–715.668 |
| 1 | large | document | 739.835 | 742.741 | 765.752 | 728.789–776.279 |
| 1 | large | combined | 1064.676 | 1065.505 | 1080.606 | 1045.714–1089.916 |
| 2 | small | construct | 1.895 | 1.901 | 1.989 | 1.825–2.086 |
| 2 | small | body | 6.418 | 6.412 | 6.498 | 6.256–6.545 |
| 2 | small | document | 6.413 | 6.617 | 8.593 | 6.253–9.945 |
| 2 | small | combined | 9.229 | 9.240 | 9.341 | 9.090–9.390 |
| 2 | medium | construct | 26.901 | 26.972 | 27.495 | 26.624–28.324 |
| 2 | medium | body | 84.037 | 84.347 | 87.336 | 82.018–87.782 |
| 2 | medium | document | 85.799 | 85.718 | 88.931 | 81.949–90.459 |
| 2 | medium | combined | 125.409 | 125.887 | 128.702 | 123.706–131.302 |
| 2 | large | construct | 208.937 | 208.547 | 213.401 | 203.312–217.521 |
| 2 | large | body | 678.239 | 677.348 | 703.164 | 643.509–705.971 |
| 2 | large | document | 737.689 | 739.267 | 762.046 | 723.088–770.182 |
| 2 | large | combined | 995.041 | 998.083 | 1025.993 | 973.129–1035.564 |

The order-reversed second repetition remains separate. For example, medium
construction medians are 40.971/26.901 µs and large construction means are
279.448/208.547 µs; this variation prevents treating one run as a stable cost.
Large combined medians are 1.065/0.995 ms for this 245,207-byte document. These
are empirical local observations with timer/barrier and scheduling overhead,
not confidence bounds, linear scaling or a stable per-row cost.
## Logical allocation and memory results
Every row below is identical across three operations in its fresh counted
process. Totals cover creation **and the separately captured explicit release**;
the raw CSV preserves start/held/released counters, allowing those boundaries
to be examined separately. Counts represent successful observed allocator
calls. `A bytes` is cumulative requested allocation bytes; realloc old/new
are separate request sizes, not extra alloc/dealloc calls or physical copying.
`Held +` and `Peak +` are logical requested bytes above that region’s own
start live baseline, with borrowed inputs kept alive. All units are bytes.
| Case | Phase | Alloc / realloc / dealloc | A bytes | Realloc old / new bytes | Held + | Peak + |
|---|---|---:|---:|---:|---:|---:|
| small | construct | 113 / 0 / 113 | 15,050 | 0 / 0 | 15,050 | 15,050 |
| small | body | 62 / 50 / 62 | 1,104 | 5,046 / 10,092 | 4,096 | 4,160 |
| small | document | 64 / 56 / 64 | 1,171 | 7,938 / 17,735 | 4,738 | 6,465 |
| small | combined | 177 / 56 / 177 | 16,221 | 7,938 / 17,735 | 19,788 | 21,515 |
| medium | construct | 1,553 / 0 / 1,553 | 207,170 | 0 / 0 | 207,170 | 207,170 |
| medium | body | 902 / 653 / 902 | 16,224 | 47,638 / 95,276 | 32,768 | 32,832 |
| medium | document | 904 / 659 / 904 | 16,291 | 79,210 / 188,959 | 62,098 | 63,817 |
| medium | combined | 2,457 / 659 / 2,457 | 223,461 | 79,210 / 188,959 | 269,268 | 270,987 |
| large | construct | 12,305 / 0 / 12,305 | 1,641,666 | 0 / 0 | 1,641,666 | 1,641,666 |
| large | body | 7,174 / 5,136 / 7,174 | 129,120 | 380,950 / 761,900 | 262,144 | 262,208 |
| large | document | 7,176 / 5,142 / 7,176 | 129,187 | 626,666 / 1,498,015 | 490,386 | 507,337 |
| large | combined | 19,481 / 5,142 / 19,481 | 1,770,853 | 626,666 / 1,498,015 | 2,132,052 | 2,149,003 |

All 36 operations return to their own start live baseline, with zero failed
alloc/realloc calls and zero alloc_zeroed calls. Allocation/deallocation counts
balance, and allocated bytes plus realloc growth equal total deallocated bytes.
These are operation-local logical balances, not a general process leak proof.
For large document rendering, start live is 1,642,377 bytes (including the
prebuilt tree/metadata), held live 2,132,763 and peak 2,149,714. Its incremental
peak is 507,337 bytes; dropping the returned document restores 1,642,377 while
keeping the input tree alive. Large combined starts/ends at 711 bytes and has
2,149,003 additional peak bytes. No pre-existing input free is counted in a
render region. Exact creation/release deltas remain in the machine summary.
Process observations below are **KiB**, with Linux `kB` labels interpreted as
1,024-byte units. `Startup RSS`, post-work RSS/high-water and external maximum
are absolute counted-process measurements; post-work still holds borrowed
inputs for body/document phases. Procfs reading/printing occurs outside counter
regions. They include runtime, mappings, allocator retained pages and
instrumentation; their small differences are not attributable phase costs.
| Case | Phase | Startup RSS | Post-work RSS | Post-work VmHWM | External max RSS |
|---|---|---:|---:|---:|---:|
| small | construct | 2,348 | 2,404 | 2,404 | 2,256 |
| small | body | 2,288 | 2,372 | 2,372 | 2,216 |
| small | document | 2,284 | 2,396 | 2,396 | 2,220 |
| small | combined | 2,288 | 2,392 | 2,392 | 2,220 |
| medium | construct | 2,296 | 2,560 | 2,560 | 2,524 |
| medium | body | 2,344 | 2,616 | 2,616 | 2,540 |
| medium | document | 2,336 | 2,668 | 2,668 | 2,564 |
| medium | combined | 2,284 | 2,640 | 2,640 | 2,528 |
| large | construct | 2,296 | 4,152 | 4,152 | 4,228 |
| large | body | 2,348 | 4,428 | 4,496 | 4,496 |
| large | document | 2,296 | 4,292 | 4,620 | 4,620 |
| large | combined | 2,348 | 4,300 | 4,620 | 4,620 |

No cumulative RSS/VmHWM maximum was subtracted. Raw `counts-*.stderr` and
`counts-*.time` retain startup high-water values and the full external process
accounting; allocator transient copying/metadata and OS RSS are separate from
logical region peaks. Counter-instrumented execution supplies no runtime timing
comparison.
## Observed gap additions and final verification boundary
| Measured observation / source | Practical impact | Confidence / classification | Future candidate, separately selected |
|---|---|---|---|
| Large combined: 19,481 allocations, 5,142 reallocations and 2,149,003-byte incremental logical peak for 245,207 HTML bytes; `counts-large-combined.stdout` | Owned tree, validation, escaping and output buffers create allocation work worth understanding if a real consumer is constrained | High local measurement; expected work/growth, no regression or budget violation proven | Profile that consumer before considering any semantics-preserving optimization |
| Large body/document incremental logical peaks are 262,208/507,337 bytes; corresponding raw count CSV, existing adapter source | Complete-document composition retains/materializes body and output Strings; result does not establish an isolated copy cost | High measurements/source observation; allocation tradeoff, not a new defect | A separately justified bounded serialization/copy investigation; no raw-HTML bypass |
| Order-reversed timing populations retain construction variation; both timing CSVs | A single repetition is insufficient for stable per-row/overhead claims | High observed variation; cause unmeasured and no regression comparison | More representative repeated profiling only if a concrete consumer question warrants it |
| Absolute counted-process RSS maxima range 2,216–4,620 KiB; count stderr/time receipts | Logical heap and process residency answer different questions; three balanced regions do not prove total-process leak freedom | High observed values; short-run/accounting limit | Separate long-lived application memory question, if requested |
No new gap ID, optimization, architectural promotion or optional-module/browser/
capacity workload follows. Existing research limits in the earlier ledger
remain separate from these observations.
Focused native checks passed: two serial sample tests (one all-size
output/dimension oracle test and one known-layout counter test), normal and
counted locked offline release builds, sample format, canonical docs and
whitespace checks. Normal/count builds use separate ext4 directories. Root
source/manifests/lock/default graph inputs and browser assets are compared
against the pre-work baseline; accepted historical tests/measurements were
reused. There is no new browser enforcement/CSP or HTTP compile-path claim.
All measured command leaders were waited/joined; cleanup inspects their
recorded sessions for remaining children. No server, browser, watcher, container
or service was started. Inert task-private binaries/build output and all raw
evidence are retained for review. Cleanup and final protected-input receipts
are recorded separately. Independent final Reviewer
`/root/yt145_ssr_final_review_20261023` returned **PASS with zero issues and
required followups**; all nine reviewed file hashes matched. The verdict and
reviewed sources are retained as `final-review-result.json` and
`reviewed-files.json` in the evidence directory. YT-145 is **Done** at the
fixed local SSR baseline scope; the measured limits and future gap candidates
remain unchanged.
