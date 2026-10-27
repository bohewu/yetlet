# YT-150 — Full SSR comparison including YT-149

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded full-SSR measurement scope after independent final Reviewer PASS, zero issues and required followups.**

## Authority and scope

The user requests a minimum complete SSR comparison including YT-149, with View
construction and render measured separately, contract before execution, no other
optimization, no replay of accepted verification, retained failures and independent
Reviewer PASS before Done. New changes have no commit/push authorization.
YT-149 was committed separately as `1b3c797431fcac369709da7be4580c70a802b7ae`;
that authorized commit does not authorize committing this study.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Preserve all product sources,
public APIs, HTML/security/error semantics, Core/browser/adapter/default graphs,
root/sample locks and dependencies. This item adds only opt-in measurement code
in the existing SSR sample and documentation. No buffer, View or second URL
optimization, generic runner, validator framework or new crate is selected.

Reuse [YT-145](YT-145-SSR-PERFORMANCE-BASELINES.md),
[YT-147](YT-147-PRIVATE-ESCAPE-INTO.md),
[YT-148](YT-148-URL-CHECK-ATTRIBUTION.md) and
[YT-149](YT-149-PRIVATE-URL-PREFIX.md) reports, raw data, output oracles, source
bindings, failures and review results read-only. Do not rerun their old commands,
test entry points, builds, HTTP/browser/CSP/database/provider workloads or output
programs. The new source-version comparison answers a new question; historical
PASS and isolated URL timing are not transferred to complete-render performance.

## Actual product versions and identical measurement code

Reference is pre-YT-149 `f01d981c9e924cecc0713b3494772628c6542954`; current is
`1b3c797431fcac369709da7be4580c70a802b7ae`. Both already contain YT-147 direct
escaping. Across compiled product inputs, only private `unsafe_scheme` differs:
reference HTML source SHA-256
`4bf49ed3b2a163c458efeeb3ad597d898f6dc802a43907818d6f851c268aa391`, current
`4568a493ac05d200581b27a5161be35bc8f13d98f80030e2bb10accffdfae496`.
Confirm the remaining compiled product files, manifests, locks and fixture inputs
are identical between these commits before interpretation.

Use two task-owned immutable `git archive` source snapshots under
`target/yt150-full-ssr-ds4q4056/source-reference` and `source-current`. These are
build input copies, without Git metadata; no checkout, detached worktree, branch
change or product mutation is needed. Each archive has about 12.4 MB of tracked
files. Overlay the same frozen standalone sample source/manifest/lock bytes onto
both copies after implementation; bind each overlay hash and every compiled
product input to its commit. The actual product libraries compile in each copy.
The same new sample calls their real `render_document_with_metadata` API: there
is no copied renderer or private-helper timing surrogate. Both copies lack Git
metadata and use the same settings. They are new builds of pinned sources, not
the exact historical executable; build path/layout/compiler variation remains.

Add default-off `page-comparison = []` and private `src/page_comparison.rs` in the
existing sample. Reuse unchanged `page`, `expected`, `verify`, `CASES` and the
existing `counts.rs`; preserve every old feature, command and test. Selected
builds enable only `default,page-comparison` or additionally `allocation-counts`,
with no old attribution/comparison feature. Four release binaries are necessary:
reference/current times ordinary/counted. Do not add a runtime variant selector
inside product code. CLI labels identify the pinned binary; they cannot change
its implementation.

## Minimum workload and phase lifetimes

Use only the existing **1,024-row** page for new timing/counting: body 244,970
bytes and complete document 245,207 bytes. This matches YT-149's 1,024 relative
URLs and avoids a larger workload matrix. All three existing sizes still receive
new output equivalence outside measurement. No new fixture or size parameter is
needed. Measure these phases separately:

| Phase | Timed/counted work | Setup and lifetime boundary |
|---|---|---|
| construct | Create owned metadata/View, including row formatting, Strings and Vecs | Start without a tree; stop with the complete tuple held; drop outside the timer |
| document | Real complete-document API, including validation, escaping, URL checks, body temporary, outer buffer and body destruction | Build one retained metadata/View before the group; stop with document String held; drop output outside the timer |
| combined | Construct metadata/View, render complete document, destroy metadata/View, return document | No retained tree; input destruction is inside; stop with returned document held, then drop output outside the timer |

Black-box borrowed inputs and owned returned results. Branch/phase selection,
fixture/oracle checks, printing, snapshots' formatting, hashing and process
inspection stay outside timers/count regions. Preallocate observation storage.
The body's temporary destruction remains inside the actual document API.
Combined directly measures construction plus render plus input teardown; do not
add medians or subtract unrelated phases to estimate it. YT-145 combined timing
also dropped the final output inside its timer, so its combined medians are not
an identical-lifetime comparison. This new pair has identical boundaries.

## Fixed populations and interpretation

New ordinary CLI `page-timing reference|current 1|2` uses label metadata only.
Run four fresh foreground processes in order: reference/1, current/1, current/2,
reference/2. Repetition 1 phase order is construct/document/combined; repetition 2
reverses it. Each phase has five warmups and twenty measured operations: **12
groups, 60 warmups and 240 measured observations total**. Print each group after
measurement. Retain variant, repetition, phase/order, population/index, integer
ns/op and result length. Report each variant/repetition/phase separately: count,
mean, median (average sorted entries 10/11), nearest-rank p95 (entry 19/20),
min/max. Keep raw order, variation and all rows; no pooling-only result, outlier
removal, numerical improvement gate or significance/stable-speed claim.

New counted CLI `page-counts reference|current construct|document|combined`
runs one fresh process per variant/phase: **six processes, three operations each,
18 operations total**, no warmups. Capture existing start/held/released snapshots
on stack using `counts::begin` and `snapshot`; no formatting or assertions before
raw capture. For combined, metadata/View have already been released at held,
while returned document remains alive. After held, drop the region-owned result
and capture released. Require zero allocator failures, release to each own start
live and identical relative observations within each variant/phase. Retain all
12 fields, separate creation/release deltas, own-start live and peak increments.
Report all current values without an invented allocation target. Request bytes
are not physical copies; logical heap is not RSS or allocator overhead.

Reconcile reference construct observations against accepted YT-145 construct
raw data and reference document observations against accepted YT-147 current
actual-library document raw data, including all applicable relative event,
request, live and peak fields. Read their existing raw data without replay.
The new combined lifetime has no identical historical counterpart: report its
new balanced values without inventing a historical reconciliation. A mismatch
stops causal interpretation; retain it for review without retuning/replacing
populations. Construction product/fixture sources are identical across variants;
its timing differences show control variability, not a URL-induced construction
speedup. Paired changes use the same repetition/phase reference denominator.
No HTTP throughput, database, browser paint, production SLO, whole-process RSS
or universal page-speed claim follows from this one synthetic local page.

## New checks, footprint and execution bounds

A new focused `yt150_` sample test exercises the three operation result/lifetime
shapes and actual outputs against the independent literal oracle. Filter all old
test entry points. Compile the new test once in the current source copy with
`--no-run --message-format=json`, select that executable and execute only
`yt150_ --test-threads=1`. No old product tests or counter sanity suite is needed.
New `page-output reference|current EXISTING_DIRECTORY` checks actual body and
document bytes for 8/128/1,024 rows, writes six outputs per binary, then independently
compares all twelve SHA-256 values with accepted YT-147 values. This is new paired
binary/fixture binding, outside timing/count populations. No security/API source
change or new browser/CSP qualification is selected.

Exactly eight tracked paths may change:

```text
docs/YT-150-FULL-SSR-COMPARISON.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/page_comparison.rs (new)
```

Stage 1 changes only four docs, performs canonical docs/whitespace checks and
independent contract review. Implementation/builds/tests/snapshots/measurements
start only after contract PASS. Stage 2 has one fixed **45-minute** UTC/monotonic
deadline, recorded before implementation and never renewed. Four source-version
release builds explain the longer preparation window; the new workload remains
small. Build/test commands have 600-second per-command ceilings within that
window. Use two build jobs, offline locked default release, Rust/Cargo 1.99.0,
System allocator, unchanged host/CPU/cgroup/settings and absent release/RUSTFLAGS
overrides. Executables use CPU 0 and **1 GiB** address space. Do not install tools,
change host configuration, affinity policy, quotas, LTO/PGO or security settings.

Persist each native environment/preparation result immediately before checking
later conditions, including `findmnt -T` raw outputs for actual evidence/source/
build paths. Keep profile/contract inputs distinct from parent-owned completion
metadata. Record exact argv, UTC/monotonic boundaries, native exit/timeout,
selected nonsecret environment, source/archive/overlay/lock/binary hashes/sizes,
Cargo feature fingerprints and all raw stdout/stderr. Use four separate build
directories under the same task root. Source archives and binary provenance are
part of interpretation, not a new reusable validation framework.

Run `cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml
--check`, the one new filtered test, and four serial offline locked release
builds with `--features page-comparison` or `page-comparison,allocation-counts`.
Run both new output commands outside the measurement batch. The **ten-process**
measurement batch (four timing, six counted) has a **60-second** monotonic ceiling
and **8 MiB** total stdout/stderr/receipt budget. Each timing/output command has
15 seconds and each counted command 10 seconds, bounded additionally by the
batch deadline. No automatic retry, observation replacement, tuning, fresh run
or workload expansion follows a failure. Preserve partial/failed/unknown results
and stop for materiality classification before any correction. A correction may
only follow the original requirement and global materiality policy.

Start no server, browser, watcher, service or container. Wait/join owned finite
foreground commands, terminate only owned children on timeout, and record cleanup
with no remaining owned process/session. Retain inert source/build/output/raw
artifacts under this task root. Product and historical tracked/evidence hashes
remain unchanged. Parent documentation/whitespace checks precede independent
final Reviewer PASS; only then mark Done. After PASS, allow status/reviewer-
reference-only transitions in the four docs and sample README, followed by docs/
whitespace checks. Rust, Cargo, binaries and observations stay frozen. There is
no new public API/architecture decision, optimization, commit, push or change to
YT-096/YT-020 production/public-release gates.


## Execution and observations — 2026-10-24

Independent contract Reviewer `/root/yt150_full_ssr_contract_review_20261024`
returned PASS, zero issues and required followups, before implementation and the
fixed 45-minute stage-2 clock. `contract-review-result.json` and
`stage2-clock.json` retain that gate and its unchanged deadline. No product
optimization or product source edit occurred in this study.

### Actual binaries and focused checks

The new sample feature/module and CLI call real product libraries in both pinned
archives. `source-snapshots.json` binds each archive, mode-aware Git blobs and
identical seven-file sample overlay; the only product difference is the private
URL checker. Symlink license blobs bind link-target bytes and resolved contents
separately. There is no `.git`, detached worktree, copied renderer or source
checkout switch. Compilation inputs stay frozen; later README result/status
prose is not executable input. All regular product source bytes remain exact.

One new `yt150_` test passed with the old entry point filtered; it exercises
owned construction, borrowed document rendering, combined output and the literal
oracle. Formatting, one offline locked JSON test compilation and all four serial
release builds passed. All twelve new actual-product body/document outputs match
both the independent literals and six accepted YT-147 hashes. No old command,
accepted test, build, output program or workload was replayed. Normal/count
features are exactly the selected default/page-comparison configurations;
`premeasurement-binding.json` retains binary hashes/sizes and Cargo fingerprints.
No `counts.rs`, `attribution.rs`, product, dependency or lock change occurred.

### Ordinary time, with construction as control

Every row has 20 measured observations and five retained warmups. Values below
are **microseconds per operation**, for the same 1,024-row page. Both the full
raw order and integer nanoseconds remain in the CSVs and
`measurement-summary.json`; nearest-rank p95 is sorted entry 19/20.

| Rep | Source | Phase | Mean µs | Median µs | p95 µs | Min–max µs |
|---|---|---|---:|---:|---:|---:|
| 1 | reference | construct | 249.027 | 234.500 | 298.851 | 215.165–331.182 |
| 1 | reference | document | 599.139 | 597.865 | 607.699 | 588.846–621.549 |
| 1 | reference | combined | 917.490 | 915.711 | 931.346 | 905.742–938.695 |
| 1 | current | construct | 210.740 | 208.528 | 219.596 | 206.841–227.042 |
| 1 | current | document | 408.151 | 406.894 | 417.227 | 400.284–417.237 |
| 1 | current | combined | 721.131 | 720.761 | 729.023 | 711.638–733.617 |
| 2 | current | combined | 742.770 | 721.750 | 878.500 | 708.065–937.390 |
| 2 | current | document | 409.319 | 408.045 | 417.178 | 402.187–421.601 |
| 2 | current | construct | 208.776 | 206.965 | 219.045 | 203.981–221.571 |
| 2 | reference | combined | 925.436 | 922.715 | 951.031 | 906.965–956.236 |
| 2 | reference | document | 597.813 | 594.506 | 617.344 | 589.241–624.074 |
| 2 | reference | construct | 218.358 | 218.450 | 221.562 | 214.805–222.190 |

Paired median changes use `(current - reference) / reference` for the same
repetition and phase. Document render observations are **−31.94% / −31.36%**;
combined observations are **−21.29% / −21.78%**. This is a new actual-library
complete SSR comparison, including input teardown in combined. It is not
inferred by subtracting URL probes or adding phase medians.

Construction source and logical counts are identical, yet its medians differ
**−11.08% / −5.26%**. These differences are control/compiler/layout/scheduling
variability, not a URL-caused construction improvement. They limit causal
precision for elapsed percentages. Two short repetitions on one synthetic page
establish no confidence interval, stable universal speedup, HTTP capacity,
production SLO or browser loading benefit. No observation was removed/replaced.
Historical timing is context only and is not this pair's denominator.

### Allocation, retained output and peak

Each variant/phase has three identical balanced observations. Reference construct
matches every accepted YT-145 construct operation and reference document matches
every accepted YT-147 current document operation: all nine pairs per phase match
all twelve relative held and released fields. Raw paths/hashes and those
snapshots are in `measurement-summary.json`. Combined uses the new input-teardown
boundary, so it has no claimed identical historical reconciliation.

The table gives creation-region calls/request bytes and held/peak increments;
release fields are retained separately in raw data. All releases return to their
own start live, and all allocation/reallocation failure counters are zero.

| Source / phase | Alloc | Realloc | Alloc request bytes | Realloc old/new requested bytes | Held live + | Peak + |
|---|---:|---:|---:|---:|---:|---:|
| reference-construct | 12,305 | 0 | 1,641,666 | 0 / 0 | 1,641,666 | 1,641,666 |
| reference-document | 1,027 | 2,069 | 8,267 | 532,428 / 1,309,539 | 490,386 | 507,337 |
| reference-combined | 13,332 | 2,069 | 1,649,933 | 532,428 / 1,309,539 | 490,386 | 2,149,003 |
| current-construct | 12,305 | 0 | 1,641,666 | 0 / 0 | 1,641,666 | 1,641,666 |
| current-document | 3 | 21 | 75 | 507,852 / 1,260,387 | 490,386 | 507,337 |
| current-combined | 12,308 | 21 | 1,641,741 | 507,852 / 1,260,387 | 490,386 | 2,149,003 |

The actual document API now observes **3 allocations / 21 reallocations** versus
1,027 / 2,069 in reference, removing the 1,024 normalized URL temporaries and
2,048 growth events on this page. The combined allocation count falls from
13,332 to 12,308, while unchanged View/metadata construction contributes 12,305.
Construction includes sample formatting/owned Strings/Vecs; these counts do not
establish a Core defect or a CPU share.

Document held logical bytes/peak remain **490,386 / 507,337**; combined held
output/peak remain **490,386 / 2,149,003**. Lower allocation frequency therefore
does not demonstrate a lower retained-output or peak requirement here. Realloc
request bytes are not measured copies and logical heap is not process RSS.
The still-materialized body/outer buffer and owned View work remain possible
future attribution questions; no further optimization or new item is selected.

### Bounds, context changes and preserved preparation failures

The ten-process batch completed in **297,184,537 ns**, retaining **27,256 bytes**
of stdout/stderr/native receipts. All ten native exits are zero, below the
60-second/8-MiB bounds and original stage deadline. All **240 measured timings,
60 warmups and 18 allocation operations** are retained; no workload retry or
replacement occurred. Both output commands ran separately outside that batch.

Raw native environment outputs were persisted immediately, including every
`findmnt` result. Rust/Cargo 1.99.0, kernel/CPU model, default System allocator,
selected flags and CPU/memory controls match the retained context. The current
login cgroup is `session-161.scope`, while earlier evidence used
`session-6.scope`. Both new binaries run under the same current session and
CPU 0/1-GiB executable limits; corresponding leaf/user ancestor CPU and memory
limits are unlimited in both records. No host setting changed. This session-ID
change is explicit; historical elapsed results are not a before/after control.
`environment.json`, raw native receipts and `environment-comparison.json`
retain the current identity and this interpretation.

Three preparation/inspection issues are retained:

- The initial sample-overlay cardinality assertion expected eight files; the
  exact overlay had seven. Only that static expectation was corrected. The
  preceding format/environment checks were not replayed and no archive had yet
  been created (`snapshot-preparation-initial-failure.json`).
- A subsequent Git-blob comparison read symlinked license contents instead of
  the link-target blob. All 18 mismatches were license symlinks, not compiled
  source changes. The binding became mode-aware, with both target and resolved
  content recorded. The already-created reference archive was reused, not
  regenerated; current archive was created once. Both failures and the focused
  correction remain in snapshot binding records. No product behavior failed.
- Reading the older environment record first assumed object-shaped fields,
  encountering a TypeError for its paired-list format. The next inspection
  exposed a KeyError because the old literal login-session path is absent now.
  The literal identity-check lane was stopped, not repeatedly repaired or
  presented as PASS. The recorded session difference and equivalent resource
  controls above explain paired comparability without rerunning native checks.
  `environment-comparison-initial-failure.json` and
  `environment-comparison.json` preserve both diagnostics and the decision.

These are preparation/inspection limitations, not failed product tests, release
builds or workloads. No fresh run, capability recovery, product repair or
observation replay followed. Initial inline preparation/inspection command
boundaries were not captured as separate native receipts; their exact assertion
or exception diagnostics and successfully completed native prefixes are retained.
Every actual test/build/output/measurement command has its native receipt.

`protected-inputs.json` binds unchanged product/history inputs and the exact
eight-path footprint. `cleanup.json` records joined, absent native leaders and
completed tool sessions; no server, browser, watcher, service or container was
started. Source snapshots, binaries, outputs and raw evidence remain inert.
Parent documentation/whitespace checks passed before review. Independent final
Reviewer `/root/yt150_full_ssr_final_review_20261024` returned **PASS, zero issues
and required followups**. All eight reviewed file hashes matched; the source/
result bindings and verdict remain in `final-review-result.json`. The disclosed
preparation/inspection and session-context limits are nonblocking.

**YT-150 is Done at bounded full-SSR measurement scope.** Only completion status/
reviewer references in four docs and the sample README change after PASS, followed
by documentation/whitespace checks. Rust, Cargo, source archives, binaries and
observations remain frozen. No other optimization or subsequent performance item
is activated. The next action is owner review of this completed study; no open
API/architecture decision remains. New changes are uncommitted and no push is
authorized; YT-096/YT-020 gates retain their existing scope.
