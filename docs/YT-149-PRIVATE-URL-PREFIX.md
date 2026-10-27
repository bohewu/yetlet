# YT-149 — Private URL prefix streaming and matched comparison

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded implementation/comparison
scope after independent final Reviewer PASS, zero issues and required followups.**

## Authority and retained evidence

The user selected contract-first implementation of the minimum private
prefix-streaming proposal from [YT-148](YT-148-URL-CHECK-ATTRIBUTION.md), removing
the complete normalized String while preserving the exact ASCII filter, full
Unicode lowercase expansion, URL/security policy, API and error order. Make
only the minimum matched comparison; retain failures and require independent
final Reviewer PASS before Done. New work has no commit, push, deployment or
external-environment authorization. Baseline is clean commit
`f01d981c9e924cecc0713b3494772628c6542954` in the current checkout. Its authorized
YT-148 commit does not authorize committing this item.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). This private HTML renderer
change introduces no architectural or public API decision. Preserve HTML-first
SSR, strict escaping and existing trust boundaries.

Reuse accepted [YT-146](YT-146-SSR-SERIALIZER-ATTRIBUTION.md),
[YT-147](YT-147-PRIVATE-ESCAPE-INTO.md) and YT-148 reports, source bindings,
literal/hash oracles, raw observations, failures and independent reviews without
replaying their accepted populations. They remain read-only, including YT-148's
source-comparison harness failure and blank-line-only correction. New evidence
belongs under `target/yt149-url-prefix-dch4xf5w/evidence`; its `baseline.json`
records tracked hashes and `reused-evidence.json` binds eight reused artifacts.
Do not rerun old test entry points, old builds, historical timing/count groups,
workspace/browser/CSP/HTTP/dependency proofs or accepted comparison programs.
New product checks below establish changed-source behavior; historical PASS
does not automatically cover the changed implementation.

## Exact product change and equivalence argument

Change only the body of private `unsafe_scheme(value: &str) -> bool` in
`crates/yetlet-html/src/lib.rs`. Create a lazy character iterator using the
existing operations in the existing order:

```text
value.chars()
    .filter(|character| !character.is_ascii_whitespace() && !character.is_ascii_control())
    .flat_map(char::to_lowercase)
```

Consume its first normalized character. For `j`, select static remaining text
`avascript:`; for `v`, select `bscript:`; for `d`, select `ata:`; otherwise return
false, including exhaustion. Match the selected remainder with
`expected.chars().all(|character| normalized.next() == Some(character))`.
Return immediately on mismatch or complete prefix match. Do not materialize the
normalized String, scan the unused tail, add a helper/public seam or introduce
an ASCII-only fast policy. Keep full `char::to_lowercase`, including expansions.

Let `N(value)` be the old filtered, fully lowercased character sequence. The new
iterator yields exactly that sequence because both filtering and lowercase
operations, and their order, are identical. Each blocked prefix has a distinct
first ASCII character (`j`, `v`, `d`). If the first yielded character matches
none, no blocked prefix can match. Otherwise checking every character of that
one prefix's remaining literal is equivalent to `N(value).starts_with(prefix)`:
exhaustion or mismatch is false; completion is true irrespective of the tail.
The other two prefixes cannot match that first character. Their disjunction
therefore equals the old Boolean for every valid Rust string. ASCII prefix
characters have the same character and UTF-8 byte boundaries, so character
comparison preserves the former String prefix semantics. Laziness changes work
and allocation, not the Boolean or any observable validation/error result.

Empty/all-filtered strings, partial prefixes, the required colon, arbitrary
tails and lowercase expansions retain these rules. Non-ASCII whitespace and
controls remain retained unless they satisfy the same existing ASCII methods.
Do not decode entities/URIs, perform Unicode case folding, broaden whitespace
removal or change policy. Keep the signature, `URL_ATTRIBUTES`, validation
sequence, all error variants/values, tag/attribute order, escaping, textarea
behavior, public `escape_text`, document composition, Core/browser/adapter code,
dependencies and locks unchanged. `xlink:href` still yields `InvalidAttribute`
under the existing identifier rule despite its membership in the URL list.

## Minimum source-derived comparison

Extend the existing sample with default-off `url-comparison = ["url-probes"]`.
Keep `url-probes` and its retained `url_check::unsafe_scheme` unchanged as the
**reference** algorithm. The old shadow must continue using that old helper and
retain all old escaping temporaries, traversal and lifetimes unchanged. Do not
rebuild a second whole SSR reference or select any old renderer feature.

Add one private source-derived **current** helper in `attribution.rs`, gated by
`url-comparison`, whose body is identical to the changed product helper after
indentation normalization. It is required because the product helper is private;
do not expose it for measurement. New focused tests and independent inspection
bind both algorithms: reference to baseline product/sample source and YT-148's
retained binding, current to changed product source. Check that deleting only the
product function body from before/after leaves the rest identical, including the
URL list and public declarations. Bind the new sample changes and unchanged old
shadow/entry points separately. No new validator framework or dependency is
authorized. The existing `yt148_` identity assertion currently compares the retained old
checker body to the current product body. Leaving that assertion unchanged would
make the existing feature test suite fail after the intentional product change.
Migrate only that reference binding to a portable, explicitly pinned legacy
normalizer literal in the existing sample test code; retain its corpus and URL-
list assertions. New `yt149_` checks bind the current copy to the changed product
and exercise the migrated reference binding without replaying the old test entry
point. No assertion may falsely equate the old and new algorithms by source text.
This is necessary compatibility with the existing sample tests, not permission to
weaken behavior checks or read run-local evidence from portable tests.

Baseline product SHA-256 is
`4bf49ed3b2a163c458efeeb3ad597d898f6dc802a43907818d6f851c268aa391`;
sample `attribution.rs` is
`83b991e56024e18a31ac68a3b22dbaa9b80e727bbfaaf3bf346b2e414a0e1291`.
Use `baseline.json` for other protected hashes. Keep `counts.rs` unchanged.

Use the existing `page(1024)` and `url_corpus` once per process, retaining the
same tree and borrowed vector across both variants. Extract outside measured
regions in the unchanged attribute-before-child order: exactly 1,024 row-ordered
`href` values `/records/NNNN?view=full&public=1`, 32 bytes each, **32,768 bytes**
total. Do not regenerate a substitute corpus. One operation checks every value
once and accumulates unsafe booleans into a primitive count, expected zero. Use
the same vector iteration, `black_box` barriers on inputs/results and primitive
accumulation for both variants; dispatch variants outside the timed/count region.
Reference normalization/prefix checks/String destruction complete inside every
call. The current iterator also completes before returning; unused input tails
are intentionally not visited. Construction/extraction, formatting, printing,
assertions, hashing, oracles and process inspection remain outside regions.

Both variants share a binary, inputs, toolchain, default release settings and
allocator within each build. The current helper is a source-derived copy, **not
timing of the actual product call or a public API**. Placement, inlining and
compiler optimization can differ from the product. Standalone iteration/barriers
are included. Even a measured ratio cannot establish whole-render CPU share,
additive elapsed cost, production speedup, RSS or whole-render peak reduction.
Accepted whole-render results provide historical context only; do not subtract
these standalone timings or add whole-render measurements to this item.

## Fixed new populations

Build ordinary `url-comparison` and counted
`url-comparison,allocation-counts` separately. Their feature fingerprints must
exclude `serializer-attribution`, `escape-comparison` and `component-probes`;
ordinary must exclude the allocator wrapper. Reject timing in a counted build.
Preserve existing commands/features and behavior assertions, with only the
reference-source binding migration above; add new commands:

- `url-comparison-timing 1|2`: exactly two fresh ordinary processes. Repetition 1
  runs reference then current; repetition 2 runs current then reference. Each
  variant gets five warmups and twenty measurements per repetition: **20
  warmups and 80 measured operations total**. Preallocate observation storage.
  Retain repetition, variant/order, population/index, integer `elapsed_ns`, URL
  count, bytes and unsafe count; print after measurement. Keep raw order and all
  rows. Report each repetition/variant separately: count, mean, median (average
  sorted entries 10/11), min/max and nearest-rank p95 (entry 19 of 20), with
  explicit ns/op or microseconds/op. No outlier removal, pooled-only result,
  significance claim or numeric timing gate is selected.
- `url-comparison-counts reference|current`: two separate fresh counted
  processes, one per variant, each three complete operations and no warmups:
  **six counted operations total**. Preallocate fixed stack snapshot storage.
  Use existing `counts::begin()` and immediate post-operation `snapshot()` before
  assertions/formatting. Retain every existing counter field and derive deltas
  from each operation's own start; peak increment is end peak minus start live.
  Require unsafe count zero, zero failures, balanced end live and identical
  within-variant relative observations across the three operations. Report
  observed current counts; do not invent a numerical allocation acceptance gate.

Every new reference operation must reconcile to every retained YT-148 raw
operation before interpretation, including the following fields:

| Field | Reference per 1,024 URLs |
|---|---:|
| Alloc / zeroed / realloc / free calls | 1,024 / 0 / 2,048 / 1,024 |
| Alloc request bytes | 8,192 |
| Realloc old / new request bytes | 24,576 / 49,152 |
| Free request bytes | 32,768 |
| Failed alloc / realloc calls | 0 / 0 |
| End live / region peak increments | 0 / 32 bytes |

Retain the raw source paths/hashes and each comparison, rather than copying
report numbers alone. A mismatch stops causal interpretation; report it without
retuning/replaying populations. Current count deltas show the observed change on
this corpus, with logical events/request sizes distinguished from physical
copies, allocator overhead or RSS. Absolute process live baselines are not
per-operation costs.

## New focused behavior and output checks

Add only tests selected by `yt149_`, in the existing sample test code. Bind both
source algorithms and the borrowed population as above. Compare the source-bound
reference/current checker Booleans against explicit literals and each other on
a small focused corpus: all three mixed-case blocked prefixes; representative
ASCII whitespace/NUL/bell/tab/LF/CR/DEL before and within prefixes; empty and
all-filtered inputs; partial prefixes/missing colons, near-match suffixes and
safe tails; long safe/blocked inputs; retained non-ASCII whitespace and multibyte
characters; full lowercase expansion such as `İ` before/within a candidate.
Include prefixes followed by Unicode/long tails to cover early completion.
Finite cases complement the equivalence argument; do not construct a massive
generic matrix or claim exhaustive Unicode verification.

Exercise actual changed `yetlet_html::render_to_string` with literal
`Result<String, RenderError>` expectations from the accepted YT-148 URL corpus,
including all existing attribute-selection and duplicate/invalid/forbidden
attribute/invalid-tag precedence cases. Reuse that fixture through the smallest
shared helper if needed, preserving its old test entry point and literal behavior assertions; run it
through a new `yt149_` test only. Do not derive expected public results from
either checker. These checks exercise actual product security/error behavior,
whereas copied-checker timing does not.

Add `url-comparison-output EXISTING_DIRECTORY`: use unchanged `page`, `verify`
and independent already-escaped `expected` literal oracle for **all three**
8/128/1,024-row sizes through this new entry point. Compare actual product body
and document bytes to those literals, write all six outputs and compare SHA-256
to the six accepted values in YT-147. This is new changed-product equivalence,
not replay of an old output entry point. Large body/document remain
244,970/245,207 bytes. No whole-render timing/count population is needed. Compile
and finite security/output evidence are separate from browser/CSP evidence;
no new browser/CSP verification is selected or claimed.

## Footprint, limits and commands

Stage 1 changes only this contract and parent-owned Backlog/Roadmap/docs index.
Obtain independent contract PASS before stage 2. The complete expected tracked
footprint is exactly nine paths:

```text
docs/YT-149-PRIVATE-URL-PREFIX.md
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
crates/yetlet-html/src/lib.rs
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/attribution.rs
```

No additional tracked file, general harness/framework, dependency, validator
generator, CI gate or validation infrastructure is authorized. Root/sample locks,
`counts.rs`, Core/browser/adapter and all historical evidence remain unchanged.
Record protected hashes and all eight reused bindings against the prework
evidence; preserve unrelated work. Keep inert build/output/raw artifacts in
separate `build-normal`, `build-counted` and output directories under
`target/yt149-url-prefix-dch4xf5w`.

After contract PASS, record UTC/monotonic start and one fixed **30-minute stage-2
deadline**, without renewal. Verify ext4 build/evidence paths (`findmnt -T`),
Rust/Cargo 1.99.0, same host/CPU/cgroup identity as accepted evidence, System
allocator and no release/RUSTFLAGS overrides; stop on incompatibility. Keep
CPU 0, executable address space **1 GiB**, **two** build jobs and offline locked
default-release inputs. Do not install tools or change host settings, allocator,
affinity policy, quotas, LTO/PGO or security settings.

Use finite foreground commands. Preparation/build/test commands each have a
600-second timeout within the unchanged stage deadline. Test compilation occurs
once with `--no-run --message-format=json`; select the emitted new test binary
and run only `yt149_ --test-threads=1`. Commands below specify the new surfaces;
the executor records their exact resolved argv and applies timeout/CPU/address
space bounds rather than timing shell wrappers:

```text
cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml --check
cargo test --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --features url-comparison --target-dir target/yt149-url-prefix-dch4xf5w/build-normal --jobs 2 --no-run --message-format=json
timeout 600 taskset -c 0 prlimit --as=1073741824 -- <new-test-executable> yt149_ --test-threads=1
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features url-comparison --target-dir target/yt149-url-prefix-dch4xf5w/build-normal --jobs 2
cargo build --manifest-path examples/ssr-performance-baselines/Cargo.toml --offline --locked --release --features url-comparison,allocation-counts --target-dir target/yt149-url-prefix-dch4xf5w/build-counted --jobs 2
timeout 15 taskset -c 0 prlimit --as=1073741824 -- <ordinary-release-binary> url-comparison-output <existing-output-directory>
timeout 15 taskset -c 0 prlimit --as=1073741824 -- <ordinary-release-binary> url-comparison-timing 1
timeout 15 taskset -c 0 prlimit --as=1073741824 -- <ordinary-release-binary> url-comparison-timing 2
timeout 10 taskset -c 0 prlimit --as=1073741824 -- <counted-release-binary> url-comparison-counts reference
timeout 10 taskset -c 0 prlimit --as=1073741824 -- <counted-release-binary> url-comparison-counts current
cargo xtask check docs
git diff --check
```

Parent owns documentation/whitespace checks. The four-process measurement batch
has a **60-second monotonic ceiling** and **8 MiB combined raw stdout/stderr/
receipt budget**, with exact populations above, no retries, observation
replacement or outlier drops. Output verification occurs outside this batch.
Record source/contract/lock/binary hashes/sizes, feature fingerprints, selected
nonsecret environment, native argv/exits, timeout status and UTC/monotonic
boundaries before analysis. Retain failures, partial/unknown exits and all raw
rows. A failed check/bound stops for materiality classification and reporting;
it does not authorize extra workload, tuning, a fresh run or population replay.

Start no server, browser, watcher, service or container. Wait/join all owned
foreground processes, terminate only owned children on timeout and verify no
owned process/session remains. Record cleanup while retaining inert artifacts.

## Acceptance and next action

Done requires prior independent contract PASS; only the selected private product
body change and bounded sample extension; new focused source/differential/public
security/error tests; all-size literal/hash output equivalence; both fixed
ordinary repetitions and both counted processes with reference reconciliation;
protected-input bindings, retained raw failures, disclosed limits and owned
cleanup; parent documentation/whitespace checks; and independent final Reviewer
PASS with zero unresolved required followups. Match reviewed file hashes before
completion metadata changes. After PASS, permit only status/reviewer-reference
transitions in the four listed documentation files and sample README, followed
by parent documentation/whitespace checks. Rust, Cargo and measured evidence
remain frozen; these transitions stay within the same nine-path footprint.
Report observations without a numerical speedup or allocation gate and without
promoting production/public-release claims.

No implementation, build, test or measurement has run at this contract stage.
The next unblockable action is independent YT-149 contract review, followed on
PASS by the bounded implementation above. No public API or architectural choice
is opened. YT-096/YT-020 and production/public-release gates remain unchanged.

## Stage-2 results — 2026-10-24

**YT-149 remains In progress pending parent documentation checks and independent
final Reviewer PASS.** The initial checkpoint statements above describe the
pre-execution contract. Contract Reviewer
`/root/yt149_url_prefix_contract_review_20261024` returned PASS with zero issues
or required followups; `contract-review-result.json` binds the reviewed contract.
Stage 2 began `2026-10-23T16:16:04.190683+00:00`, monotonic
`139599190159907` ns. Its fixed, unrenewed deadline is
`2026-10-23T16:46:04.190683+00:00` / `141399190159907` ns.
All evidence paths below are relative to
`target/yt149-url-prefix-dch4xf5w/evidence`.

### Implementation and behavioral evidence

Only the private product `unsafe_scheme` body changed. It preserves the old
ASCII whitespace/control filter and full `char::to_lowercase` expansion order,
selects a static remainder by the first normalized character, and stops on
mismatch, exhaustion or full prefix completion. The equivalence argument above
covers arbitrary tails and expansions; the finite tests complement that argument.
`source-control-check.json` proves that deleting only the function body leaves
all other product bytes identical, including public declarations, URL list,
validation order, errors and escaping. The old sample checker, old shadow,
old URL operation and old CLI routes remain unchanged. No dependency, lock,
`counts.rs`, default feature, browser, Core or adapter change was made.

The default-off `url-comparison = ["url-probes"]` adds the current source-derived
copy and three commands. The current checker body matches the product after
indentation normalization; its timing is **a sample copy, not an actual private
product call or public API**. The reference remains the old source-bound
normalizer. Its old source assertion now compares against an explicitly pinned
portable legacy body from baseline `f01d981c9e924cecc0713b3494772628c6542954`.
The existing corpus/list assertions and all literal public rendering/error
expectations remain intact in small shared test helpers; no portable test reads
a target directory or run-local evidence.

The format check, one offline locked JSON `--no-run` test compilation, only the
emitted binary's `yt149_ --test-threads=1` filter, and both fresh default-release
builds passed. Exactly three new tests passed; three old entry points were
filtered out. They bind the migrated legacy/current source, borrowed row-ordered
1,024 URLs / 32,768 bytes and attribute-before-child selection; compare explicit
finite prefix, control, Unicode expansion and long-tail results; and exercise
actual `yetlet_html::render_to_string` against unchanged literal security,
attribute-selection and error-precedence results. Browser/CSP/HTTP validation
was not selected or claimed. No old accepted test/build/population was replayed.

Normal test compilation preceded the manually corrected counted-only predicate.
The changed counted function is excluded from that normal build. Its embedded
`include_str!` whole-file bytes differ from the final source, while the exercised
test functions and extracted reference/current binding sections and product
inputs are unchanged. `counted-only-edit.json` retains both source hashes and
the exact delta; both later release builds use the final source. No stale
whole-file identity claim or repeated test compilation is made.

The new `url-comparison-output` entry point calls unchanged `verify`, `page` and
independent already-escaped literal oracles for 8/128/1,024 rows. All six new
body/document outputs match the six accepted YT-147 SHA-256 values, recorded in
`output-equivalence.json`; large lengths are 244,970/245,207 bytes. This is
changed-product output evidence outside the measurement batch.

### Matched observations and reconciliation

`premeasurement-binding.json` records source/contract/lock/binary hashes and
sizes before analysis. Features are exactly `default,url-comparison,url-probes`
for ordinary and additionally `allocation-counts` for counted; excluded old
renderer/component features remain absent. The counted build rejects timing.
Rust/Cargo 1.99.0, host/CPU model, cgroup membership/limits, selected environment,
System allocator, offline locked inputs and default release settings match
accepted YT-148 evidence. Executable runs use CPU 0 and 1 GiB address space;
builds use two jobs. No host/environment settings changed.

One process per ordinary repetition retains the same borrowed page/corpus across
both variants. Repetition 1 orders reference/current; repetition 2 orders
current/reference. All **20 warmups and 80 measured operations** are retained.
Each ns/op observation covers 1,024 URLs, including identical vector iteration,
barriers, primitive accumulation and complete checker temporaries. Input
construction/extraction, dispatch, formatting/assertions/output stay outside.
Median averages sorted entries 10/11; nearest-rank p95 is entry 19 of 20.

| Repetition | Variant (order) | Count | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | reference (1) | 20 | 256,753.50 | 250,092.0 | 322,856 | 205,403 | 339,349 |
| 1 | current (2) | 20 | 7,772.45 | 7,768.0 | 7,799 | 7,752 | 7,808 |
| 2 | current (1) | 20 | 26,525.00 | 26,432.0 | 26,538 | 26,419 | 28,038 |
| 2 | reference (2) | 20 | 363,035.80 | 348,281.5 | 498,336 | 250,449 | 524,009 |


The two fresh counted processes each retain three operations without warmups.
All three relative observations within each variant are identical and balanced,
with unsafe count and allocator failures zero. `reference-reconciliation.json`
compares every new reference operation to every retained YT-148 raw operation:
**all nine pairs match all 12 fields**, with source paths/hashes and both raw
snapshots retained before interpretation.

| Field | Reference per operation | Current per operation |
|---|---:|---:|
| Allocation / zeroed / reallocation / free calls | 1,024 / 0 / 2,048 / 1,024 | 0 / 0 / 0 / 0 |
| Allocation request bytes | 8,192 | 0 |
| Reallocation old / new request bytes | 24,576 / 49,152 | 0 / 0 |
| Free request bytes | 32,768 | 0 |
| Failed allocation / reallocation calls | 0 / 0 | 0 / 0 |
| End live increment / region peak increment | 0 / 32 bytes | 0 / 0 bytes |

Current zeros are observations on this relative-ASCII corpus, not an invented
numerical allocation gate. Logical allocator events/request sizes are distinct
from physical copies, allocator overhead and RSS; absolute process live
baselines are not per-operation costs. The ordinary repetitions vary substantially.
No outlier was removed or replaced, and no significance, stable speedup,
whole-render CPU share, additive elapsed cost, production improvement or
whole-render peak/RSS reduction is claimed. Compiler placement/inlining may differ
between the copied helper and product. Historical whole-render data is context
only; no renderer subtraction or new whole-render timing/count population ran.

### Retained execution history, bounds and limitations

The single measurement batch ran `2026-10-23T16:23:19.819705+00:00`
to `2026-10-23T16:23:19.900665+00:00` for `81013611` monotonic ns,
within 60 seconds and the fixed stage deadline. Its four native exits are all
zero; stdout/stderr/individual receipts total **8,112 bytes**,
below 8 MiB. Exact argv, UTC/monotonic boundaries, all raw order/populations and
receipts are retained in the four `url-comparison-timing-*` /
`url-comparison-counts-*` records. There were no measurement retries,
replacements, outlier drops or product repair attempts.

One initial pre-edit binding assertion failed because parent-authorized
review-reference/status updates changed three metadata hashes after contract
review. The reviewed contract and all protected/reused inputs matched. Work
stopped; the parent authorized one smallest binding correction distinguishing
those three exact approved transitions. The focused binding assertion then
passed without replaying passed environment checks or changing source/contract.
`stage2-environment-initial-failure.json`,
`parent-contract-status-transition.json` and `preedit-binding-correction.json`
retain the failure and correction. This was a harness failure; no product
behavior was disproved.

A separate manual pre-execution inspection found that copying the old counted
command had included its zeroed-call numerical predicate in the new command.
This was **not a failed test, build, product check or workload**. The parent
authorized removing only that new predicate, preserving zero failures, balanced
live, within-variant stability and every raw field; the old command is unchanged.
`counted-gate-inspection-failure.json` retains the originally named record and
`inspection-history-note.json` clarifies its manual status. One format check
followed the actual counted-only edit; no compiled or measured population was
replayed. `counted-gate-direction.json` and `counted-only-edit.json` bind it.
Operational retries and product repair attempts are zero; no recovery/fresh run
occurred.

The initial environment command's raw `findmnt` stdout was held in memory when
the later metadata assertion failed and was not persisted. The three exact argv,
zero exits and successful ext4 assertions are recorded as an execution
attestation in `stage2-environment.json`, rather than represented as retained raw
text. Toolchain/host/model/cgroup/environment diagnostics are retained. No passed
environment proof was replayed to replace that missing raw text. This receipt
limitation is disclosed for final review; no incompatible filesystem was observed.

Protected tracked inputs and all eight reused artifacts are bound in
`protected-inputs.json`. Finite foreground process leaders were waited and absent;
no server, browser, watcher, service or container was started. `cleanup.json`
records cleanup while retaining inert artifacts. Parent documentation/whitespace
checks passed before final review. Independent final Reviewer
`/root/yt149_url_prefix_final_review_20261024` returned **PASS, zero issues and
required followups**; all nine reviewed file hashes matched. The independent
verdict and bindings are retained in `final-review-result.json`. Both disclosed
evidence limits are nonblocking; no required work remains in this slice.

**YT-149 is Done at bounded implementation/comparison scope.** Only completion
status/reviewer references in these four docs and the sample README are updated
after PASS, followed by documentation/whitespace checks. Rust, Cargo and measured
evidence stay frozen. New changes are uncommitted, and no push occurred. The next
action is owner review of this completed slice; no next performance item is
automatically activated, and no API or architectural choice is open. YT-096/
YT-020 and production/public-release gates remain unchanged.
