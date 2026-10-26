# YT-155 — Small/medium latency attribution and stability

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded research scope; independent final Reviewer PASS.**

## Authority and unchanged product boundary

The user selects contract-first minimum research separating capacity-reservation
strategy costs from measurement variability, reusing YT-154, preserving API /
output/security, no optimization, no accepted-proof replay, retained failures,
independent Reviewer PASS before Done and no commit/push. Baseline is clean
`50decf86ebe628d7163c53c4fbb040a4fb955d08` in the current checkout.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). All product sources, public
APIs, body release order, HTML validation/escaping/URL policy, metadata/CSP/HTTP
behavior, Core/browser/default graphs, dependencies and root/sample locks remain
unchanged. Do not change reserve policy, fuse buffers, add APIs, instrument
product code, tune compiler/allocator/host settings or introduce an optimization.
Only the existing optional SSR measurement sample and four documents change.

[YT-154](YT-154-DOCUMENT-CAPACITY-COMPARISON.md) remains accepted at its own scope.
Small median changes were +98.96% / +2.38%; medium +46.65% / +3.52%. Their cause
is unmeasured. Lower returned capacity and fourteen-byte higher peak remain
read-only evidence, not timing causes proven by counters. Preserve all historical
raw populations, source/binary bindings, failures and review decisions. Do not
run YT-154's old commands, builds, tests, counts, output qualification, large-page
groups, Counter checks or browser/CSP/HTTP/provider/database verification.
New evidence belongs under `target/yt155-small-medium-latency-cr2njref/evidence`.

## Two complementary questions

1. Do small/medium actual-API differences persist with a declared minimum warmup,
   longer uninterrupted blocks and balanced source/group order, while retaining
   initial-call observations separately?
2. In one process/binary, what difference is observed between the original and
   reserved **tail-handling paths**, given identical preparation and alternating
   order, without timing the renderer?

These separate actual API behavior, local strategy-tail cost and variability.
They do not guarantee unique attribution to individual reserve instructions,
physical copies, allocator internals, compiler layout or hardware state. A
bounded inconclusive finding is valid if supported by retained observations;
do not tune/repeat until a preferred explanation appears.

## Immutable product inputs and minimum new consumer

Reuse YT-154's accepted `source-reference` and `source-current` product bytes,
bound by its `source-snapshots.json`. Make task-owned source copies; never edit
the originals. Verify all copied product files/manifests/locks against accepted
hashes, then overlay identical final measurement sample source/manifest/lock
bytes. Only the pre-existing adapter method differs between the product copies.
Current product bytes must also match the current checkout. Bind copied source,
overlay, executable hashes/sizes/features and actual library artifact manifests.
No new Git archive, checkout, detached worktree or product rebuild of an old
acceptance configuration is required.

Add default-off `latency-study = []` and private `src/latency_study.rs`. Two new
ordinary release binaries are needed for the new measurement consumer, one
against each accepted actual product source. Select only default/latency-study;
no allocation-counts, trace, previous study feature, new dependency or old CLI
entrypoint. Preserve unchanged fixture and independent literal oracle. Use only
small 8-row and medium 128-row pages; large results remain historical context.

### Actual API mode

New `latency-api reference|current 1|2` calls actual
`render_document_with_metadata` with View/metadata retained outside measurement.
For each size, record eight **initial** per-call observations, then a predeclared
unmeasured warmup of at least **50 ms**, bounded by **100,000 calls**. Record exact
warmup calls/duration/completion; failure to reach the duration within the bound
stops interpretation without an expanded warmup or retry. No sample-dependent
optimization or favorable-value convergence test exists.

Then collect **ten uninterrupted blocks of 32 observations**. Every observation
uses the same output-held timer boundary as YT-154: begin clock, actual API,
black-box returned String, stop clock, inspect length/capacity and destroy it
outside timing. Dispatch, setup, warmup, record allocation and printing stay
outside timers. Preallocate records, print the completed initial population before
warmup, then print steady records after the group; do not perform I/O between
steady blocks. Initial means first calls of that group, not a
guaranteed cold process/cache. Earlier group activity and context reads exist.

Run four fresh processes: reference/1, current/1, current/2, reference/2.
Repetition 1 visits small/medium, repetition 2 medium/small. This yields **64
initial observations and 2,560 steady API observations**. Warmups are unmeasured
operations with aggregate count/duration, not invented per-call samples.

### In-process strategy-tail mode

New `latency-tail 1|2` runs only the current ordinary binary. It contains two
private source-derived tail copies: legacy appends body, releases it and appends
closing; reserved additionally reserves body plus closing before append. Both
release the owned body before closing. Copy only this small tail sequence, not
a renderer or complete adapter. Source review binds it to the accepted old/new
method; the new test below verifies composition.

Before **each** observation, outside its timer, construct the exact shared
prefix/title using the original literal/call order, then call the actual
unchanged body renderer. Pass fresh owned prefix/body Strings into the selected
tail, preserving their natural capacities. This preparation order matches the
full API up to the tail boundary. Dispatch is outside timing; the timer includes
tail reservation/growth/body append/body destruction/closing append, ends with
returned output held and drops it outside. No buffer/payload cache or reserve
outside the selected existing strategy is added.

After the same 50-ms/100,000-call bounded warmup, alternating both strategies,
collect ten blocks per size. Each block has sixteen four-call cycles, giving
32 observations per strategy. Repetition 1 uses legacy/reserved/reserved/legacy;
repetition 2 reverses that order. Small/medium group order also reverses. Two
fresh tail processes yield **2,560 steady tail observations**. Do not execute
tail commands in the reference binary or replay YT-153 append probes.

This same-process proxy reduces process/version-order confounding but has copied
tail functions, call/ABI/compiler placement, prepared inputs and different timing
boundaries. It is not an actual private adapter entrypoint, pure reserve-call
latency or a whole-document CPU fraction. Do not add/subtract it from API medians
or historical times. No counted build or new allocation population is selected;
reuse YT-154's logical allocation/peak evidence read-only.

## Records, context and interpretation

Run the four API processes followed by the two tail processes once: **six fresh
measurement processes, 5,184 per-call times** including the 64 initial values.
Retain mode, source/strategy, run, size/group order, phase, block/index, integer
ns/op, returned length/capacity; no row/outlier exclusion or pooling-only result.
Record per-group warmup totals and optional raw CPU-0 sysfs frequency context
before initial/warmup, after warmup and after steady measurement, outside timers.
Missing/unreadable frequency is null, not a reason to install tools or retry.
This reported context is not calibrated cycles or proof of a timing cause.
Capture native process CPU/context-switch/fault/RSS summaries with existing
`/usr/bin/time -v` if executable; otherwise retain native receipts without it.
No new resource-observation framework or OS control is introduced.

Report initial and steady API populations separately; per run/size/variant or
strategy report count/mean/median/min/max/nearest-rank p95. Median averages middle
entries for even counts; p95 uses sorted entry ceil(0.95*n). Also retain all ten
block medians/spreads, matched ordinal block deltas and the number of positive /
negative differences. Blocks are serial observations, not independent random
trials: no confidence interval, significance, universal regression or speedup
claim follows. Preserve both repetitions and initial values even if unfavorable.

Check lengths against accepted literals, returned capacities against retained
YT-154 observations as this native population's identity (not a public guarantee),
and raw schedule/count/warmup bounds. Source/body/metadata/allocator semantics
must match before interpreting differences. Historical +98.96%/+46.65% values
are context, not replaced denominators or a target for reproducing old proof.
Compare newly controlled actual-API and same-process tail evidence to state what
is reproduced, what local strategy cost is supported and what variability/cause
remains unresolved. Correlation of reported frequency/context with timing does
not prove that hardware, allocator or reserve uniquely caused the old result.

## New focused verification and bounded delivery

One new ordinary `yt155_` sample test checks both tail copies and preparation
against independent small/medium literal documents and the actual current API,
including finite invalid-tree preparation errors from the actual renderer.
Compile once offline locked with `--no-run --message-format=json`; run only its
emitted `yt155_ --test-threads=1` filter. No tests of a validator/warmup runner,
old entrypoints, product regression suite, new Counter checks or repeated output
qualification. Accepted YT-154 outputs/product identity are reused, not replayed.
Run one sample format check and two fresh ordinary release builds for the new
consumer. Freeze Rust/Cargo before tests/builds/measurements.

Exactly eight repository paths may change:

```text
docs/YT-155-SMALL-MEDIUM-LATENCY-STABILITY.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/latency_study.rs (new)
```

Stage 1 changes four docs, checks them and obtains independent contract review.
Stage 2 records a fixed **35-minute** UTC/monotonic deadline before edits, never
renewed. Use Rust/Cargo 1.99.0, offline locked default release/System allocator,
two build jobs and ext4 copied-source/build/evidence paths. Capture selected
nonsecret flags, current cgroup/affinity/effective limits before assertions.
Keep host/governor/frequency/quota/allocator/compiler/LTO/PGO settings unchanged.
No installation, service/browser/container, external resource or new profiling /
validation framework. Reuse only the existing finite native receipt helper.

Executables use CPU 0 and 1 GiB address space. Build/test commands have
600-second caps; measurement commands fifteen seconds, within the fixed deadline.
The six-process batch has sixty seconds / 8 MiB combined stdout/stderr/receipt
limits. Persist exact argv, UTC/monotonic boundaries, raw results, native exits /
timeouts, source/lock/executable bindings and owned cleanup immediately. Preserve
failed/partial/unknown outcomes. Classify under materiality before any smallest
correction; no automatic replay, tuning, new task/clock or workload expansion.
Wait/join owned foreground commands, terminate only owned children on timeout,
and retain inert artifacts with no owned process left behind.

Parent docs/whitespace checks and independent final Reviewer PASS with zero
required followups precede Done. After PASS, only status/reviewer references in
four docs and sample README, then docs/whitespace checks, may change. All product /
sample Rust/Cargo, observations and binaries stay frozen. No optimization,
additional item, commit/push or YT-096/YT-020 release decision follows.

## Results and bounded attribution

Independent contract Reviewer `/root/yt155_latency_stability_contract_review_20261024`
returned PASS, zero issues/required followups, before sample implementation.
`contract-review-result.json` binds the four reviewed hashes; native trace and
shared resolver confirm the registered review contract. No product source changed.
The complete fixed population ran once: six exits 0, no timeout or missing group,
64 initial API + 2,560 steady API + 2,560 steady tail times. No observations were
removed and no failed outcome was replaced. The 12 warmup groups all reached the
declared minimum within the call cap. There were no task-native failures or retries;
historical failures remain untouched in the reused evidence.

### Actual API observations

Each steady cell contains 320 times, ten blocks of 32. Positive change means
current reservation is slower than reference. Ordinal block comparisons below
are descriptive pairs of separate-process observations, not simultaneous pairs
or independent trials.

| Size | Run | Reference median ns | Current median ns | Change | Positive / negative block deltas |
| --- | --- | --- | --- | --- | --- |
| small | 1 | 3,529.0 | 3,590.5 | +1.74% | 10 / 0 |
| medium | 1 | 44,400.0 | 46,101.5 | +3.83% | 10 / 0 |
| small | 2 | 3,603.0 | 3,660.0 | +1.58% | 10 / 0 |
| medium | 2 | 44,232.0 | 45,350.0 | +2.53% | 9 / 1 |

The small-page increase is 57.0–61.5 ns in the aggregate medians; medium is
1,118.0–1,701.5 ns. The direction remains positive in both repetitions and
39/40 ordinal block pairs, so discarding the entire remaining difference as
random noise is unsupported. These serial, separately compiled/process results
also do not prove a universal product regression or identify its instruction-level
cause. Within-run block median ranges are retained rather than only pooled times:

| Mode / size | Run | Reference / legacy block median range ns | Current / reserved block median range ns |
| --- | --- | --- | --- |
| api / small | 1 | 3,474.5–3,570.0 | 3,548.0–3,613.5 |
| api / medium | 1 | 43,735.5–45,118.5 | 45,691.5–46,718.0 |
| api / small | 2 | 3,579.0–3,622.0 | 3,640.5–3,672.0 |
| api / medium | 2 | 43,890.5–45,002.5 | 44,763.5–46,607.5 |
| tail / small | 1 | 72.0–81.0 | 60.0–71.5 |
| tail / medium | 1 | 485.0–489.0 | 475.0–484.5 |
| tail / small | 2 | 93.5–107.0 | 75.0–84.5 |
| tail / medium | 2 | 482.0–487.5 | 471.0–479.0 |

YT-154's large first-repetition increases (+98.96% small, +46.65% medium) are
not reproduced by this new warmed population. They remain valid historical
observations, not erased or relabeled as proven errors. Its smaller second-run
increases (+2.38% / +3.52%) are closer to this study's direction/magnitude, but
protocol/caller/features/build paths and sample populations differ. This is no
claim of a strict historical timing equivalence or a unique warmup cause.

### Same-process strategy-tail observations

Each strategy cell contains 320 times; both share one binary/process and fresh
preparation per observation. Positive means reserved is slower.

| Size | Run | Legacy median ns | Reserved median ns | Change | Positive / negative block deltas |
| --- | --- | --- | --- | --- | --- |
| small | 1 | 75.0 | 64.0 | -14.67% | 0 / 10 |
| medium | 1 | 487.0 | 480.0 | -1.44% | 0 / 10 |
| small | 2 | 98.0 | 77.0 | -21.43% | 0 / 10 |
| medium | 2 | 485.0 | 478.0 | -1.44% | 0 / 10 |

The reserved proxy is lower by 11/21 ns small and 7/7 ns medium, with negative
block deltas in all 40 pairs. Thus these observations do not support an intrinsically
slower complete reservation-tail path under this controlled preparation. They
also cannot explain the positive actual-API differences by addition/subtraction:
no-inline copied tail placement/ABI, allocator state, whole-function optimization
and process context differ. Very short tail timings include clock/wrapper cost;
no pure `reserve_exact` instruction cost or whole-page percentage is claimed.
The remaining actual-API difference is unresolved at this bounded scope.

### Initial phase, context and full distributions

The first group's eight initial-call medians vary sharply: current run-1 small
16,667.0 ns versus steady 3,590.5; run-2 medium reference/current 217,408.0 /
198,635.5 ns versus steady 44,232.0 / 45,350.0. Initial groups later in a process
need not be cold. Raw reported CPU-0 frequency before those three groups is near
1.1 GHz versus near 5.1 GHz after warmup. Reference run-1 small reports near
5.1 GHz before/after and has initial/steady medians 3,565.5 / 3,529.0 ns.
This correlation supports retaining explicit initial/steady separation and context;
it does not prove CPU frequency caused YT-154's observations. Raw sysfs values are
neither calibrated cycles nor continuous telemetry. All 36 readings and native
CPU/fault/context-switch/RSS records remain in `measurement-summary.json`.

The following table retains every per-run/size/variant/phase aggregate; units are
ns. Initial n=8 makes nearest-rank p95 equal max. Serial observations and short
populations are descriptive; no confidence interval or significance claim.

| Mode | Variant | Run | Size | Phase | n | Mean | Median | p95 | Min | Max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| api | current | 1 | medium | initial | 8 | 47,589.12 | 45,295.0 | 62,835 | 45,138 | 62,835 |
| api | current | 1 | medium | steady | 320 | 46,156.20 | 46,101.5 | 47,432 | 45,030 | 55,308 |
| api | current | 1 | small | initial | 8 | 19,345.38 | 16,667.0 | 35,295 | 16,381 | 35,295 |
| api | current | 1 | small | steady | 320 | 3,595.28 | 3,590.5 | 3,650 | 3,504 | 4,798 |
| api | current | 2 | medium | initial | 8 | 211,266.88 | 198,635.5 | 303,661 | 180,937 | 303,661 |
| api | current | 2 | medium | steady | 320 | 45,654.95 | 45,350.0 | 47,220 | 44,633 | 47,945 |
| api | current | 2 | small | initial | 8 | 3,906.38 | 3,734.0 | 4,581 | 3,680 | 4,581 |
| api | current | 2 | small | steady | 320 | 3,665.42 | 3,660.0 | 3,715 | 3,580 | 4,782 |
| api | reference | 1 | medium | initial | 8 | 48,486.12 | 45,988.0 | 66,144 | 45,718 | 66,144 |
| api | reference | 1 | medium | steady | 320 | 44,540.22 | 44,400.0 | 45,734 | 43,535 | 51,294 |
| api | reference | 1 | small | initial | 8 | 4,737.12 | 3,565.5 | 11,145 | 3,506 | 11,145 |
| api | reference | 1 | small | steady | 320 | 3,533.59 | 3,529.0 | 3,599 | 3,441 | 5,761 |
| api | reference | 2 | medium | initial | 8 | 226,332.25 | 217,408.0 | 284,958 | 205,299 | 284,958 |
| api | reference | 2 | medium | steady | 320 | 44,491.18 | 44,232.0 | 45,486 | 43,739 | 50,321 |
| api | reference | 2 | small | initial | 8 | 3,804.62 | 3,625.5 | 4,458 | 3,562 | 4,458 |
| api | reference | 2 | small | steady | 320 | 3,612.50 | 3,603.0 | 3,650 | 3,549 | 4,863 |
| tail | legacy | 1 | medium | steady | 320 | 487.35 | 487.0 | 503 | 465 | 538 |
| tail | legacy | 1 | small | steady | 320 | 80.14 | 75.0 | 115 | 69 | 135 |
| tail | legacy | 2 | medium | steady | 320 | 486.51 | 485.0 | 505 | 462 | 535 |
| tail | legacy | 2 | small | steady | 320 | 99.35 | 98.0 | 110 | 87 | 119 |
| tail | reserved | 1 | medium | steady | 320 | 479.72 | 480.0 | 495 | 457 | 519 |
| tail | reserved | 1 | small | steady | 320 | 67.27 | 64.0 | 80 | 56 | 92 |
| tail | reserved | 2 | medium | steady | 320 | 484.29 | 478.0 | 495 | 449 | 1,829 |
| tail | reserved | 2 | small | steady | 320 | 80.20 | 77.0 | 90 | 71 | 117 |

All 160 steady block distributions, ten block medians and ordinal differences
per comparison, sign counts and median-of-block-medians remain in the summary;
all 5,184 original samples remain in the six `.stdout` files. The reserved medium
tail run-2 maximum 1,829 ns is preserved; its mean 484.29 ns is closer to legacy
486.51 ns than their medians, illustrating why neither extrema nor medians alone
establish a general speedup. No outlier filtering was applied.

### Verification, identity and decision

Native command receipts retain the exact argv and boundaries. Commands used the
existing finite wrapper; these are new consumer configurations, not accepted-proof
replays:

- One `cargo fmt --manifest-path examples/ssr-performance-baselines/Cargo.toml
  --check`: exit 0.
- One offline/locked `cargo test --features latency-study --no-run
  --message-format=json --jobs 2` against the copied current sample, then its
  emitted executable with `yt155_ --test-threads=1`: 1 passed, 1 older test
  filtered out. The focused test covers new proxy composition/literal outputs
  and actual-renderer invalid URL/void-child error propagation.
- Two offline/locked ordinary `cargo build --release --features latency-study
  --message-format=json --jobs 2` commands against the copied source variants:
  exits 0; emitted executable and actual library artifacts select only
  default/latency-study. No counted/old-study build or old entrypoint was run.
- Four `latency-api` and two `latency-tail` foreground commands, CPU 0,
  1 GiB address space and native `/usr/bin/time -v`: all exits 0, waited and
  owned leaders absent. No owned server/container/background worker exists.

The six-command batch took 0.993230881 s and retained 1,000,408 bytes
of combined stdout/stderr/receipts, below 60 s / 8 MiB. The 12 warmups totaled
83,860 unmeasured operations, each 50.000287–50.102268 ms,
with exact per-group values retained. They are not part of the 5,184 per-call
observations. GNU time reports 0.11–0.13 user seconds/process, zero major faults,
1 voluntary and 1–2 involuntary context switches, max RSS 2,580–2,704 KiB.
These process-wide summaries are context, not per-timer attribution.

All returned lengths and capacities match accepted YT-154 population identity:
small 2,383 output bytes with reference/current capacities 4,738/2,383; medium
31,063 bytes with capacities 62,098/31,063. The tail copies match corresponding
strategies. Capacity is not an API promise. No allocation counters or previous
large/combined output qualification were rerun. Source bindings confirm unchanged
products, fixture/oracle, locks and all 43 reused history bindings; new sample
Rust/Cargo bytes stayed frozen through test/build/measurement and final review.

Recommendation: retain the accepted YT-154 capacity policy and its documented
memory/timing tradeoff. Do not implement a reserve-policy change or a special
small/medium branch from this evidence: the controlled tail is not slower and the
small positive actual-API difference has no uniquely established fix. For future
comparisons retain separate initial/declared-warm populations, balanced order and
block distributions; use actual API timing for product claims and label proxies.
If a future concrete workload makes this small difference material, separately
scope compiler/caller/layout or allocator-context attribution before optimization.
No new item or optimization is activated here. Public API/output/security and
body release timing remain unchanged. YT-096's D1–D6 remain Pending, YT-020
Blocked, original immutable release-candidate scope unchanged.

Artifacts: `target/yt155-small-medium-latency-cr2njref/evidence/` contains
`authority.json`, `stage2-clock.json`, `reused-evidence.json`, contract review,
environment/native receipts, `source-copies.json`, `source-review.json`,
`measurement-source-bindings.json`, `binary-{reference,current}.json`,
`test-artifact.json`, `measurement-batch.json`, raw observations and
`measurement-summary.json`; the task root holds the finite receipt helper and
one-shot arithmetic `analyze.py`. Final independent Reviewer `/root/yt155_latency_stability_final_review_20261024`
returned PASS, zero issues/required followups, after independently reconciling
all raw/aggregate/block values and frozen source/data/history/executable bindings.
`final-review-result.json` retains acceptance; `closeout.json` binds the permitted
status/reference-only transition. No runtime, observation or binary changed after
review. Done applies only to this bounded research; full-API cause remains unresolved.
No commit or push has been performed.
