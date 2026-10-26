# YT-147 — Private escape-into implementation and matched SSR comparison

Date: 2026-10-23. Status: **Done at bounded implementation/comparison scope
after independent final Reviewer PASS, zero issues and required followups.**

## Authority and preserved contracts

The user selected the one minimum proposal from
[YT-146](YT-146-SSR-SERIALIZER-ATTRIBUTION.md): first define the contract, then
implement private `escape_into` while preserving public `escape_text`, exact
output, validation, errors and security/API semantics, and make a minimum new
matched comparison. Reuse accepted verification, retain failures, and require
independent final Reviewer PASS before Done. New changes must not be committed,
pushed, deployed or applied to external environments. Baseline is clean commit
`90f83867c77307c9aab1b4ca72afe7f20afb4dc1` in the current checkout. The prior
nine-file YT-146 commit does not authorize a commit of this work.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). This is an internal HTML
serializer change, without an architectural promotion. Preserve HTML-first SSR,
strict escaping, semantic HTML and all existing trust boundaries. No public
buffer API, raw HTML, URL-policy optimization, document-composition change,
Core/browser change, dependency, CI gate or benchmark framework is selected.

## Exact product change

In `crates/yetlet-html/src/lib.rs`, extract the existing character loop into
private `fn escape_into(input: &str, output: &mut String)`. Append in input
character order: `&` to `&amp;`, `<` to `&lt;`, `>` to `&gt;`, double quote to
`&quot;`, apostrophe to `&#39;`, and every other Unicode scalar unchanged.
Do not clear or replace an existing destination prefix. Empty input appends
nothing. No reservation heuristic, byte scanning or second optimization belongs
in this change.

Keep public `escape_text(input: &str) -> String`: construct
`String::with_capacity(input.len())`, call the private helper, and return that
String. This preserves its original initial-capacity behavior and character
algorithm. The ordinary `View::Text` branch appends through the helper directly
to the existing output. After the unchanged attribute validation sequence,
write the same space/name/equals/opening quote, append through the helper, then
write the closing double quote. Preserve tag/attribute ordering, quoting,
validation precedence, all constants and every `RenderError` variant/value.

**Textarea keeps its existing public-wrapper call.** Its text collection,
CRLF/CR normalization, initial-LF compensation and temporary lifetime remain
unchanged. URL normalization and document title escaping also remain unchanged.
The adapter continues materializing a body and appending it into an outer
String. These explicit limits implement the YT-146 proposal's smallest scope.

## Reused evidence and new question

[YT-145](YT-145-SSR-PERFORMANCE-BASELINES.md) and YT-146 are Done at their stated
scopes. Their reports, accepted commands, source bindings, raw measurements,
failures and six literal output hashes remain read-only evidence under
`target/yt145-ssr-cost-vpx_z1d5` and `target/yt146-ssr-attribution-grvvqfm_`.
Do not replay YT-145's 24 timing groups or YT-146's component probes, old counter
sanity tests, workspace/browser/CSP/HTTP/dependency certification or old builds.
New focused checks exercise the changed shared escaping and the new comparison;
they do not transfer historical acceptance to changed source.

YT-146 observed 6,149 body escape temporaries, plus one unchanged document-title
escape. This motivates the change; it is neither an expected compiler guarantee
nor a numerical acceptance threshold. Ask how the selected direct appends affect
ordinary elapsed time, allocation calls/request bytes and global logical heap
for the unchanged **1,024-row page**: body **244,970 bytes**, document **245,207
bytes**. No construction, HTTP, RSS, capacity ladder or component-corpus timing
is needed. Timing variation and regressions must be retained and reported.

All three unchanged sizes (8/128/1,024 rows) still receive new equivalence checks.
Reuse `page` and the independent already-escaped `expected` literal oracle from
the existing sample; do not make a second fixture or general escape oracle.
Accepted output SHA-256 values are:

| Case | Body | Document |
|---|---|---|
| small | `39008f753d68bc1cfa95af51281dd8182fbf5a25e1c73992e2d9cfdeb823ffaf` | `976f185de09dd60901b11a3a4325ea2d4d854c1077d2a26db6eb9df3a025dff6` |
| medium | `3ccd90de1b641bcdef854f4ca8a8f9ade8b4148be3ac1d80f5f2c57f20ba584e` | `b27ae2d7b741bc0b0b59a7b51d75d5270a2d1c3f7f6b1812f4875422b93097bb` |
| large | `25fedaaf88a65ec2c3c016fcd4545a58d345c5209640d37a2f3497cdd9557c56` | `3e01ffaa34dc450f5b648e7b3d64ebcaa4d2aa1fda6623753386a3d17ec58734` |

## Selected reference and matched builds

Extend only the existing `examples/ssr-performance-baselines` sample with
`escape-comparison = []`, default off. Reuse its existing source-derived
`shadow` in `src/attribution.rs` as **reference**, and call the actual changed
HTML/document libraries as **current**. Do not duplicate the copied traversal,
validation or adapter composition again. Compile the shadow for this feature
as well as its old attribution feature; for comparison without
`serializer-attribution`, use a fixed local `Bucket`/inlinable no-op `tagged`
shim. It supplies no allocator tags, counters or general instrumentation API.
Leave the existing `counts.rs` unchanged.

The reference retains old temporary-String calls to public `escape_text` and
the original append/drop boundaries. The current public wrapper still uses
the original capacity/algorithm, so that call retains the old temporary behavior.
This is a **source-derived control rebuilt with the current wrapper**, not an
identical historical binary or a historical compiler comparison. Independent
source review must check all original branches/constants, operation order,
textarea lifetimes and document composition against baseline, including outside
the measured valid page. Byte equivalence alone does not prove that binding.

Within each build, both variants use the same binary, runtime, input tree,
metadata, System allocator and default release/locked dependency settings.
Build ordinary `escape-comparison` separately from counted
`escape-comparison,allocation-counts`; neither selected build enables
`serializer-attribution` or `component-probes`. Ordinary timing has no allocator
wrapper; counted runs share the existing global counter without per-bucket tag
overhead. Reject timing when `allocation-counts` is enabled.

The baseline 1,027 tracked hashes are retained in
`target/yt147-escape-into-dc8ihdk3/evidence/baseline.json`, with explicit authority
in `authority.json`. Bind that commit and these source hashes before interpreting
results:

| Original source | SHA-256 |
|---|---|
| `crates/yetlet-html/src/lib.rs` | `4f2222e74b69e231b4445f20dea696360badbad0e18fd7cda8e73a3f06242af3` |
| `crates/yetlet-server-axum/src/lib.rs` | `06c73c3c4e1bfee37c06e3d2b19e641c93832ee028056932b0f6ddd4cd37d370` |
| `crates/yetlet-server-axum/src/document.rs` | `8d1f2a55f3f49f7998e9895807ef0c7fb06270fe2f967d7d6d5f540f7f49491a` |
| `examples/ssr-performance-baselines/src/attribution.rs` | `b2908ea9eb719f40afd884689199dab49449831cda9fa64a07b75aa0bddbf233` |
| `examples/ssr-performance-baselines/src/counts.rs` | `2048ba2acd2bd77be6fdab3908997a2909d27f1e922e33c0ceb3d0933af987d0` |
| `examples/ssr-performance-baselines/Cargo.lock` | `cb6115cb54419d0b9c286b9c4d3df1f1749e82f78af44c0cbe1ad6d82baae676` |

## Fixed timing populations and lifetimes

New CLI `comparison-timing 1|2` runs four groups per repetition. Repetition 1
order is reference/body, current/body, reference/document, current/document;
repetition 2 exactly reverses that order. Each group has **5 warmups and 20
measured operations**: eight groups, **40 warmups and 160 measured rows total**.
No retries, order tuning, extra repetitions or outlier removal.

Prebuild one large metadata/tree before groups and retain it through all output
releases. Each operation borrows the same tree; neither construction, clones,
oracle/hash checks, procfs reads nor printing occur inside the timed region.
Black-box borrowed inputs and returned output. Stop with the full output String
held; record elapsed integer ns and output length, then drop output outside the
timer. In document operations, body temporary creation/append/destruction stays
inside the timer for both variants exactly as in the original composition.
Reference text/attribute temporary creation and destruction also stay inside.

Preallocate raw observation storage. Record repetition, sequence, variant,
phase, warmup/measured population, index, integer `elapsed_ns` and result length;
print after each group. Keep every row and each repetition/group separate.
Report count, mean, median (average sorted entries 10/11), min/max and nearest-
rank p95 (entry 19/20), with explicit ns/op or microseconds/op. Paired descriptive
differences may be reported per phase/repetition, with denominator/sign stated;
no significance, stable-speedup, whole-renderer CPU-share or causal component
claim follows from these short local observations. Instrumented time is not
ordinary timing.

## Fixed allocation populations and reconciliation

New CLI `comparison-counts reference|current body|document` runs one fresh
foreground counted process per variant/phase: **four processes, three operations
each, 12 operations total, no warmup**. Retain one prebuilt large metadata/tree
through all start/held/released snapshots. Reset only the existing region peak
at start to current live. Returned output remains alive at held snapshot; drop
it before released snapshot. Setup, snapshots' formatting, printing, oracles,
hashing and input destruction are outside regions.

Retain all existing global start/held/released fields: alloc, zeroed, realloc,
dealloc, allocated/deallocated request bytes, realloc old/new request bytes,
failed alloc/realloc, live and peak. Derive creation/release deltas and held/
peak/released live increments relative to each operation's own start. Require
zero failures, released live equal start live and identical per-variant/phase
deltas across the three operations. Request bytes are not physical copies;
logical live/peak is not RSS or allocator overhead.

Before interpreting effects, reconcile reference output/errors/source and all
relative global creation/release/heap fields to the accepted YT-146 raw snapshots
(and their retained YT-145 reconciliation), without rerunning either old binary.
Explain compiler/control limits and preserve absolute baseline differences.
A mismatch stops causal interpretation and requires review; never silently
replace evidence or tune product behavior to force counts. Report current versus
reference call/request/heap changes, including output growth changes. There is
**no required numerical reduction in time, allocations or peak heap**.

## New focused behavior checks and footprint

Add tests selected only by `yt147_`; old entry points remain filtered out:

- In the HTML crate, directly test private append semantics on a nonempty prefix,
  empty input/destination, multibyte UTF-8 and all five escaped characters;
  compare the public wrapper to independent fixed expected literals and retain
  its signature/capacity initialization by source review.
- In the comparison sample, all three sizes compare reference and current body/
  document to the independent literal oracle. `comparison-output
  EXISTING_DIRECTORY` writes both variants' twelve outputs outside measurements;
  independently hash each against its corresponding accepted value above.
- Exercise a finite exact `Result<String, RenderError>` reference/current corpus:
  invalid/forbidden tags, invalid/duplicate/forbidden attributes (`onclick`,
  `style`, `srcdoc`), mixed-case/control-obfuscated `javascript:`, `vbscript:` and
  `data:` URLs, void children, nested forms, invalid textarea/option/select
  children, and competing-invalid-input precedence. Include valid escaped text/
  attributes, empty/multibyte input, textarea CR/LF/initial-newline cases and
  valid select/option. Reuse the existing YT-146 corpus through a small local
  fixture helper if needed; do not replay its old test entry point or create a
  second renderer/security framework.

Only these nine tracked paths may change after review (four docs now, five
implementation/sample paths later):

```text
docs/YT-147-PRIVATE-ESCAPE-INTO.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
crates/yetlet-html/src/lib.rs
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/attribution.rs
```

No new tracked file other than this contract is needed. Preserve the
sample/root locks and dependencies, root inputs, browser assets, Core, adapter,
`counts.rs`, default-empty features and all YT-145/146 history. Confirm public
signatures/exports and protected tracked hashes against baseline rather than
rebuilding old consumers or rerunning their dependency certification. Sample
feature metadata must show only the two selected comparison configurations.

## Stage gates, execution bounds and commands

Stage 1 changes only the four documentation paths, runs canonical docs and
whitespace checks, saves hashes/patch/native receipts, then **stops for independent
contract review**. No product implementation, test, release build or measurement
precedes contract PASS. Stage 2 starts only after that PASS. Record UTC and
monotonic start and a fixed **30-minute deadline**, never automatically renewed.

Use `target/yt147-escape-into-dc8ihdk3` on ext4, with new separate `build-normal`
and `build-counted` directories; verify evidence/build paths with `findmnt -T`.
Record current toolchain, CPU/cgroup/host identities and selected nonsecret
environment, preserving CPU 0, Rust/Cargo 1.99, System allocator, default release,
1 GiB executable address-space ceiling and build jobs 2. Stop if incompatible.
Do not change affinity policy, CPU quota, allocator, RUSTFLAGS, LTO/PGO, kernel,
host capabilities or install tools. Every test/workload executable uses
`taskset -c 0` and the 1 GiB limit. Builds/tests time out at 600 s each; timing
processes at 15 s, counted processes at 10 s. The finite measurement batch has a
**60-second monotonic ceiling** and **8 MiB raw stdout/stderr/receipt budget**.
The output equivalence command is outside measured populations, at 15 s.

Minimal stage-2 commands (sample Cargo commands use the shown manifest,
`--offline --locked`, explicit target directory and jobs 2):

```text
rustfmt --edition 2024 --check crates/yetlet-html/src/lib.rs
cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml --check
cargo test -p yetlet-html --lib --offline --locked yt147_ -- --test-threads=1
cargo test --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --features escape-comparison yt147_ -- --test-threads=1
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features escape-comparison --jobs 2
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features escape-comparison,allocation-counts --jobs 2
<ordinary-binary> comparison-output <existing-new-output-directory>
<ordinary-binary> comparison-timing 1
<ordinary-binary> comparison-timing 2
<counted-binary> comparison-counts reference body
<counted-binary> comparison-counts current body
<counted-binary> comparison-counts reference document
<counted-binary> comparison-counts current document
cargo xtask check docs
git diff --check
```

Bind exact argv, contract/source/lock/binary SHA-256 and sizes, selected environment,
UTC/monotonic start/end, native exit, stdout/stderr and timeout status **before
analysis**. Keep source/binary/output snapshots, all rows including warmups,
failures, partial/unknown exits, review decisions and cleanup receipts in the
run directory. No failed population is discarded or replaced with a tuned retry.
A material product failure permits at most two in-task repair cycles after
classification under the materiality policy; operational retry limit is two.
Repeated harness/infrastructure signature twice stops work. Bounds or missing
review never authorize a new framework, second candidate or old-proof replay.

Use finite foreground processes only. Start no server, browser, watcher,
container or service. Wait/join every owned process, terminate only owned children
on timeout, and verify no owned command/session remains. Retain inert raw/build
files and failures. Cleanup evidence is required before a completion claim.

## Acceptance and next action

Done requires independent contract PASS before implementation, the exact private
change with preserved public/security/output behavior, the bounded new focused
checks, twelve output hashes matching the six accepted values, source/reference
count reconciliation, all fixed timing/count populations with failures/variation
retained, protected-input/API checks, cleanup and independent final Reviewer
PASS with zero unresolved required followups. Report actual changed files,
commands/results, source/binary limits and any open decision. A timing regression
does not itself fail this behavior contract; disclose it without another tuning
round or a numerical performance gate. No production SLO, architectural promotion
or release qualification follows. YT-096/YT-020 retain their current gates.

At the stage-1 checkpoint no implementation, test, build or new measurement had run.
The next unblockable action is independent YT-147 contract review; after its
PASS, the bounded implementation and new comparison are the next work. No
additional optimization or follow-on task is activated by this contract.


## Actual implementation and new verification

Independent contract Reviewer `/root/yt147_escape_into_contract_review_20261023`
returned **PASS, zero issues and required followups** before stage 2. The reviewed
four hashes and verdict are retained in `contract-review-result.json`. Stage 2
started **15:24:21.989 UTC**, with fixed deadline **15:54:21.989 UTC**. No deadline
renewal, implementation repair, failed product check or operational retry occurred.
The initial stage-1 root-versus-docs README interpretation was corrected before
contract review; both snapshots and the scope-correction record remain retained.
The new-file no-index whitespace native exit 1, with empty diagnostics, remains
recorded rather than hidden as a command exit 0.

Exactly the nine listed delivery paths changed. The product change contains
only the extracted private character loop, public wrapper delegation and normal
text/attribute direct appends, plus the new focused literal test. Textarea,
identifier/URL policy, constants, public declarations and adapter source are
unchanged. The reference serializer/validation/document-composition body is
byte-identical to its committed YT-146 source; only module availability and the
fixed no-op tag shim changed. `source-control-check.json` records these checks.
The existing finite validation corpus was extracted into a local test fixture
helper for reuse, without another renderer copy or an old test invocation.

New checks passed: one HTML private-append/public-literal test (seven old tests
filtered out) and two sample tests for all-size independent literals and exact
reference/current errors/control content (the old sample test filtered out).
The finite corpus includes all five escapes, multibyte/empty content, forbidden
attributes, obfuscated unsafe schemes, control children and precedence cases.
Product/sample formatting and both default-release offline locked builds passed.
These are changed-source native checks; no browser, CSP or dependency-certification
command was rerun and no new browser/CSP acceptance is claimed.

The ordinary compiled feature fingerprint contains precisely empty `default`
and `escape-comparison`, without the counter module/global allocator wrapper.
The counted fingerprint adds only `allocation-counts`; both exclude serializer
attribution and component probes. Root/sample manifests' dependency inputs,
locks, default graph, Core, browser, adapter and `counts.rs` remain protected.

Ordinary binary SHA-256: `5d4bf3c6a13e13d0701a778b3c525fdd36e0f3faf00c11bbf19b2fbc33f2203b` (567464 bytes).

Counted binary SHA-256: `6442d8c18ca306f984b67f13425f9fe91876747a94f34a862131a749e1525b2a` (583648 bytes).

Execution retained Rust/Cargo 1.99.0, Linux 7.0.0-38-generic, x86_64 Intel Core
i5-13600K, System/default release, build jobs 2 and ext4 evidence/build paths.
Current cgroup ancestors retained unlimited CPU/memory limits where controllers
exist; RUSTFLAGS and release overrides were absent. Tests and workload commands
used literal `taskset -c 0` and `prlimit --as=1073741824`; no host setting changed.
Native source/binary/feature/argv/time/exit/raw receipts preceded analysis.

The output command wrote twelve new reference/current files; every file matches
its corresponding one of the six accepted hashes. The seven foreground commands
(output, two timing, four counted) all exited 0, with no timeout/partial group.
The measurement batch completed at **15:27:28.376 UTC**, taking **0.155 seconds**;
raw stdout/stderr/receipts total **18,177 bytes**, below 60 seconds/8 MiB. All
160 measured timing rows, 40 warmups and 12 allocation operations are retained.
No old accepted measurement or proof was replayed. No observation was removed,
replaced, retried or used to tune a second optimization.

## Allocation and global logical heap effects

Before effect interpretation, every reference held/released relative global
field matches all three corresponding accepted YT-146 snapshots for each phase,
including creation/release calls/request bytes, failures, held/peak and released
live increments. Absolute process live baselines are not compared as work costs.
Each variant/phase has three identical operations, no allocation/reallocation
failure or zeroed allocation, and release live equals its own start live.
Full snapshots and reconciliation remain in `measurement-summary.json`.

| Phase / variant | Alloc calls | Realloc calls | Alloc request bytes | Realloc old/new request bytes | Held live increment | Peak increment |
|---|---:|---:|---:|---:|---:|---:|
| body / reference | 7,174 | 5,136 | 129,120 | 380,950 / 761,900 | 262,144 | 262,208 |
| body / current | 1,025 | 2,063 | 8,200 | 286,712 / 573,424 | 262,144 | 262,176 |
| document / reference | 7,176 | 5,142 | 129,187 | 626,666 / 1,498,015 | 490,386 | 507,337 |
| document / current | 1,027 | 2,069 | 8,267 | 532,428 / 1,309,539 | 490,386 | 507,337 |

The measured difference is **6,149 fewer allocation calls and 3,073 fewer realloc
calls** for both body and document. Allocation request bytes fall by 120,920;
realloc old/new request totals fall by 94,238/188,476 bytes. This agrees with
removing the normal body text/attribute temporaries in this compiled fixed-page
population. Existing URL normalization, title escaping and output growth remain.
These are request/event differences, not physical bytes copied or causal CPU
shares. Body/document held output live increments are unchanged. Body global
peak falls only **32 bytes** (262,208 to 262,176); document peak remains **507,337
bytes**. Fewer short-lived allocations therefore do not establish a substantial
peak-heap improvement, a retained-memory reduction or an RSS improvement.

## Ordinary elapsed observations and limits

Every row below has 20 measured operations; values are **microseconds/op**.
Each group also retains its five warmups. Repetition 2 reverses all four group
orders; no aggregate across repetitions or outlier removal is used.

| Repetition | Phase / variant | Median | Mean | Empirical p95 | Min–max |
|---|---|---:|---:|---:|---:|
| 1 | body / reference | 708.205 | 743.925 | 918.070 | 695.537–1013.996 |
| 1 | body / current | 527.760 | 526.966 | 535.447 | 516.413–536.208 |
| 1 | document / reference | 751.979 | 756.661 | 772.715 | 748.712–776.088 |
| 1 | document / current | 589.938 | 589.896 | 595.793 | 584.365–602.423 |
| 2 | body / reference | 677.470 | 681.980 | 702.093 | 673.145–722.804 |
| 2 | body / current | 527.728 | 527.462 | 535.484 | 517.295–541.059 |
| 2 | document / reference | 741.633 | 743.671 | 761.680 | 736.211–764.259 |
| 2 | document / current | 600.154 | 600.810 | 612.747 | 590.512–622.296 |

Current-minus-reference median differences, divided by the **same repetition and
phase reference median**, are body **−25.48% / −22.10%** and document **−21.55% /
−19.08%** for repetitions 1/2. These are descriptive differences in this fixed
source-derived matched comparison. Reference body medians vary from 708.205 to
677.470 microseconds, while current document medians rise from 589.938 to
600.154; the first reference body group retains a 1,013.996-microsecond maximum.
All this variation remains visible in raw data. The short two-repetition design
establishes no significance, stable speedup, HTTP capacity or production SLO.

Both variants borrow the retained tree and hold output at timer end; output
drop is outside the timer. Document's internal body temporary remains inside.
The reference is rebuilt source with the current semantically unchanged public
wrapper, not the old historical binary. Source placement, inlining and compiler
choices can still influence this comparison. Count reconciliation strengthens
the allocation-control binding without proving identical machine code or timing.
No instrumented elapsed time, old component timing or subtraction from unrelated
historical medians supports these numbers.

## Current completion boundary

Evidence is retained under `target/yt147-escape-into-dc8ihdk3/evidence`, including
contract review, source/binary snapshots and feature fingerprints, exact native
receipts, twelve output hashes, every timing/count population, full reference
reconciliation, baseline-protected checks and final cleanup. The initial scope
correction and all historical failures remain preserved. No server, browser,
watcher, container or service was started. Every owned foreground leader was
waited; workload PID absence and completed command sessions are recorded. No
live owned resource remains; inert source/build/raw files are retained.

**YT-147 is Done at bounded implementation/comparison scope.** Independent final
Reviewer `/root/yt147_escape_into_final_review_20261023` returned **PASS, zero
issues and required followups**; all nine reviewed file hashes matched. The
verdict and reviewed snapshots remain in `final-review-result.json` and
`final-reviewed-files.json`. All requested implementation and bounded new
observations are complete; no second optimization or broader validation is
justified. Remaining URL normalization, unchanged title/textarea allocations
and body/document materialization are limitations, not newly activated tasks.
Further optimization requires separate selection. There is no open API or
architecture decision, new commit/push, browser/CSP qualification or change to
YT-096/YT-020 gates.
