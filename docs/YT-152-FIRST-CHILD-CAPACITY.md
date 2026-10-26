# YT-152 — Private first-child capacity comparison

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded comparison/rejected-candidate scope after independent final Reviewer PASS.**

## Authority and preserved boundary

The user selected the minimum private `Element::child` first-capacity strategy
implementation and comparison, covering one child, multiple children and the
complete page. Establish the contract first, preserve APIs/output/security,
reuse accepted evidence without replay, retain failures and require independent
Reviewer PASS before Done. No new commit/push is authorized. YT-151 was separately
committed as `505eba79c90553649e63ac128fe4683ec08f93ea` with strict/builtin Guard
PASS, zero findings, tree `95aaa57aa5c64f1233f79e35687054643fe03adf`; no push.

Follow [Naming](NAMING.md), [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Retain Vec/View representation,
public signatures/traits, ownership and conversion/drop/order behavior. No new
dependency, unsafe code, crate, builder hint, capacity accessor, browser runtime,
template, arena or alternate optimization. HTML validation, escaping, URL policy,
metadata, transport, default graphs and root/sample locks stay unchanged.

Reuse [YT-151](YT-151-VIEW-CONSTRUCTION-ATTRIBUTION.md) and
[YT-150](YT-150-FULL-SSR-COMPARISON.md) raw observations, hashes, review results
and failures read-only. Their allocation attribution motivates this experiment,
not a claim that 68% of CPU is Core work. Do not run old accepted test entrypoints,
builds, workload/output commands or browser/CSP/HTTP/provider/database suites.
New evidence belongs under `target/yt152-child-capacity-rj_wkcft/evidence`.

## Minimum product change

Only change `Element::child`'s private allocation decision: convert the incoming
value to View first, as the original push argument evaluation does; if the
children Vec has zero capacity, request exactly one slot with `reserve_exact(1)`;
then push the owned View. Existing capacity and later normal Vec growth are used
unchanged. Leave `Element::children` and all other product methods unchanged.
Do not reserve before invoking a user-defined conversion. No stable capacity,
allocation count, target layout or allocator-size-class promise becomes public.

This may save spare slot storage for one-child Elements but add growth/realloc
and moves for multi-child Elements. Report both effects and all elapsed results.
No invented speedup threshold, outlier removal, tuning or follow-on optimization
is allowed. Completion means a correct bounded implementation and honest matched
comparison, not proof of universal improvement. The final recommendation must
state whether the memory/elapsed tradeoff justifies retaining this candidate;
inconclusive or worse elapsed results stay visible rather than becoming a
production performance claim.

## Actual product sources and fixed fixtures

Make two task-owned immutable `git archive` copies of baseline `505eba7` under
the task root, without Git metadata or checkout/worktree changes. Overlay identical
frozen sample source/manifest/lock bytes on both; only the current archive also
receives the changed Core lib. Bind original Git blobs/modes (symlinks by link
text), archive/source/overlay/lock hashes, compiler features and binary hashes/
sizes. Current lib includes its new cfg-test block; only the child method differs
in product behavior. Compile real libraries in both archives, with no copied Core
or renderer implementation. Native variant labels identify binaries only.

Add default-off `child-capacity = []` and private sample
`src/child_capacity.rs`. Select only this feature, plus allocation-counts for
counted builds. Reuse the unchanged System Counter and original complete-page
fixture/oracle. Do not enable construction-study or other old studies. Two new
simple fixtures use 1,024 section Elements, each with one or four identical owned
Text children, under one main root built by Range-map `children`. Text contains
HTML-sensitive characters. Both use the same existing title/language metadata;
there are no per-row formats/attributes in these simple fixtures. An independent
literal oracle constructs the exact expected body/document outside measurement.

| Group | Work and lifetime |
|---|---|
| single | Construct metadata/View for 1,024 one-child parents; stop with inputs held, drop outside timer |
| multi | Same with four children per parent; exposes the extra-growth tradeoff |
| page | Original unchanged 1,024-row complete-page metadata/View construction, including formatting |
| combined | Original page construction, real complete-document render and input destruction; stop with document held, drop output outside timer |

Black-box returned owned values and relevant inputs. Dispatch, fixture checks,
printing, hashing, snapshots' formatting and result inspection are outside time
regions. Combined includes real body temporary/input destruction and uses the
same output-held boundary as YT-150. No separate document-only replay is needed.

## New checks and populations

One new Core `yt152_` test covers custom conversion once, single/multi sibling
order, mixed child/bulk insertion, cloning and unwind propagation/drop behavior.
One new sample `yt152_` test checks the new simple fixtures against independent
literals, original all-size page literals, and finite nested-child escaping /
unsafe URL / forbidden-attribute / invalid-void-child error outcomes. These are
focused checks of the changed insertion path, not old test entrypoint replay.
Compile each new test binary once with offline locked `--no-run
--message-format=json`, then execute only its `yt152_ --test-threads=1` filter.
No allocator sanity suite or browser requalification is selected.

Build four separate offline locked default-release binaries: reference/current
ordinary and counted. Run new `child-output reference|current EXISTING_DIRECTORY`
once per ordinary binary. Each writes body/document for single and multi plus
the original three page sizes: ten outputs per binary, twenty total. New simple
outputs must equal their literal oracles and each other; the twelve original
page hashes must additionally equal accepted YT-150 values. Record byte lengths.

New ordinary `child-timing reference|current 1|2` runs four fresh processes in
order reference/1, current/1, current/2, reference/2. Group order is
single/multi/page/combined in repetition 1, reversed in repetition 2. Each group
has five warmups and twenty measurements: **80 warmups / 320 measured operations**.
Preallocate records, retain every integer ns/op and variant/repetition/group/
order/population/index/result size, and report per group count, mean, median
(average entries 10/11), nearest-rank p95 (entry 19/20), min/max. Do not pool away
repetitions, drop failed/outlying values or treat phase medians as additive.

New counted `child-counts reference|current single|multi|page|combined` runs one
fresh foreground process per variant/group: **eight processes, three operations
each, 24 operations**, no warmups. Stack records capture global start/held/
released, with each owned result held and then dropped. Print raw all twelve
fields before assertions; require release to own-start live, zero failures and
within-group stability. Retain allocation/zeroed/reallocation/free calls, request/
reallocation-old/new/deallocated bytes, live and peak increments. No expected
reduction or zero-realloc requirement is imposed on current. Requested bytes are
not physical copies, allocator overhead or RSS; node moves are not directly timed.

Before interpreting the paired results, reconcile reference page held/released
against all three retained YT-151 construct operations (all nine pairs, twelve
fields at both points), and reference combined against all three retained YT-150
current combined operations likewise. New simple fixtures have no historical
counterpart. A mismatch stops causal interpretation without retuning/replay.
Describe paired differences per repetition/group with the matching reference
denominator, plus raw variation and memory/reallocation tradeoffs. This native
synthetic comparison establishes no production throughput, browser memory/paint,
stable speedup or hardware/SLO qualification.

## Bounded delivery and review

Exactly nine repository paths may change:

```text
docs/YT-152-FIRST-CHILD-CAPACITY.md (new)
docs/BACKLOG.md
docs/ROADMAP.md
docs/README.md
crates/yetlet-core/src/lib.rs
examples/ssr-performance-baselines/Cargo.toml
examples/ssr-performance-baselines/README.md
examples/ssr-performance-baselines/src/main.rs
examples/ssr-performance-baselines/src/child_capacity.rs (new)
```

Stage 1 changes only four docs, checks them and obtains independent contract
review. After PASS, start one fixed **45-minute** UTC/monotonic stage-2 deadline
before implementation, never renewed. The task-local finite command receipt
helper reuses the accepted existing pattern; no general validation framework,
validator tests or profiling infrastructure is authorized. Record native argv,
UTC/monotonic boundaries, raw stdout/stderr, exits/timeouts and owned cleanup
immediately. Preserve failed/partial/unknown results. Classify any failure under
the materiality policy before the smallest necessary correction; no passed proof
replay, automatic retry, fresh run or workload expansion.

Use Rust/Cargo 1.99.0, ext4 task source/build/evidence paths, unchanged default
release/System allocator, two build jobs and offline locked inputs. Record
selected nonsecret flags and current cgroup identity/effective ancestor limits;
new login-session identity is context, not a historical literal-equality gate.
Do not install tools or alter host settings/LTO/PGO/quotas/governor. Native
executables use CPU 0 and 1 GiB address space. Build/test commands have 600-second
caps within the fixed deadline. Output/timing/count commands have 15-second caps.
The twelve-process measurement batch has a 60-second monotonic ceiling and
8 MiB combined stdout/stderr/receipt budget. Run one format check for changed
Core/sample Rust; use emitted test executables rather than recompiling tests.

Wait/join all owned foreground commands; kill only owned children on timeout.
Start no server/browser/watcher/container/external resource. Retain inert raw/
source/build artifacts under the task root and confirm no owned process remains.
Protect every other tracked path and retained evidence binding. Canonical docs/
whitespace checks and independent final Reviewer PASS with zero required
followups precede Done. After PASS, only status/reviewer references in the four
docs and sample README may change, followed by docs/whitespace checks; product,
sample Rust/Cargo, binaries and observations remain frozen. No commit/push,
release-gate expansion, next optimization or YT-096/YT-020 decision follows.

## Execution results and retention decision

Independent contract Reviewer
`/root/yt152_first_child_contract_review_20261024` returned **PASS**, zero issues
and required followups, with all four reviewed hashes matching before sample or
product edits. `contract-review-result.json`, `contract-review-files.json`,
`contract-review-trace.json` and `contract-review-observed-decision.json` retain
that gate and the enforced native role/model/effort observation. Independent final Reviewer `/root/yt152_first_child_final_review_20261024` returned **PASS**, zero issues and required followups. All nine reviewed hashes match `final-review-files.json`; `final-review-result.json` retains acceptance of the candidate comparison, default-policy rejection and disclosed final-checkout build boundary. The item is Done at bounded comparison/rejected-candidate scope.

**Decision: do not adopt the unconditional one-slot policy in default Core.**
The experiment was implemented and measured as contracted. It reduces logical
memory on single-child/full-page fixtures, but both repetitions have slower
medians for every group, and multi-child growth adds reallocations without a
held-memory benefit. This is evidence against general adoption here, not a
universal timing conclusion or a new numerical acceptance gate.

After preserving the candidate archive/binaries and all raw data, remove only
this task's child-method replacement from the checkout, restoring the exact
accepted baseline production Core code. Keep the new regression test, optional
comparison sample and report. There is no alternative implementation, tuning,
repeat workload or subsequent optimization. `retention-decision.json` binds the
candidate and restored file hashes. The actual compiled candidate is retained in
`source-current/crates/yetlet-core/src/lib.rs` under the task root, with this body:

```rust
pub fn child(mut self, child: impl Into<View>) -> Self {
    let child = child.into();
    if self.children.capacity() == 0 {
        self.children.reserve_exact(1);
    }
    self.children.push(child);
    self
}
```

Thus the final checkout has **no production Core capacity-policy change**;
its Core diff adds only the new cfg-test. The new test source is identical before
and after withdrawal. Executed test/build evidence binds the actual candidate;
the final production body matches the already accepted reference byte-for-byte.
The final checkout with its new test block was not separately rebuilt, and no
passed check is replayed to conceal that distinction. Variant `current` means
the archived experimental library, not the final default checkout policy.

### Sources, checks and output equivalence

All paths below are relative to the evidence directory unless stated otherwise.
`source-snapshots.json` records two baseline git archives, mode-aware original
blob checks (license symlinks use link text), identical sample overlays and all
product hashes. Only the archived current Core lib differs. The new fixture
calls real Core/metadata/render APIs; no copied implementation is timed.
`measurement-source-bindings.json` records premeasurement inputs;
`source-review.json` and `retention-decision.json` distinguish the intentional
final Core restoration from frozen candidate evidence. Counter, old studies,
page builders/oracles, dependencies/locks and every out-of-scope tracked path
remain unchanged. Seven retained historical bindings remain read-only.

| New check / evidence | Result / receipt stem |
|---|---|
| Changed sample/Core formatting | Two writes, one combined recursive format check; exit 0: `format-write-sample`, `format-write-core`, `format-check` |
| New Core test compilation/filter | Offline locked lib compilation then emitted `yt152_ --test-threads=1`: 1 passed, 5 old tests filtered; `compile-core-test`, `new-core-test` |
| New sample test compilation/filter | Same new filter against actual candidate archive: 1 passed, 1 old test filtered; `compile-sample-test`, `new-sample-test` |
| Four actual-library release builds | All offline locked default release, jobs 2, exit 0: `build-{reference,current}-{normal,counted}` |
| Twenty new literal/hash outputs | Both new `child-output` calls exit 0: `output-reference`, `output-current`; all ten pairs match, twelve original-page hashes match accepted values |
| Fixed ordinary population | All four `timing-{variant}-{repetition}` processes exit 0: 320 measurements / 80 warmups |
| Fixed counted population | All eight `counts-{variant}-{group}` processes exit 0: 24 operations |

Native argv/stdout/stderr and immediate exit/timing receipts are separate files.
`test-artifact-{core,sample}.json` binds emitted test binaries; four
`binary-{variant}-{normal,counted}.json` records bind source/features/executable
hashes and sizes. Ordinary selects only default/child-capacity, counted adds
allocation-counts. No old study feature or Counter calibration is selected.
`output-equivalence.json` retains all twenty sizes/hashes, accepted page hashes
and literal-check outcomes. Simple body/document lengths are 56,333 / 56,570
bytes (single) and 166,925 / 167,162 bytes (multi); page outputs remain unchanged.

### Ordinary elapsed observations

Each row contains twenty measured observations after five warmups, in the
contracted variant and reversed group orders. All warmups/raw integer values
remain in stdout; no outlier was discarded. Group `page` is construction only;
`combined` includes actual complete SSR and input destruction, output held.

| Variant | Rep | Group | Mean ns/op | Median ns/op | p95 ns/op | Min ns/op | Max ns/op |
|---|---|---|---:|---:|---:|---:|---:|
| reference | 1 | single | 40,390.45 | 38,449.0 | 43,676 | 36,641 | 66,091 |
| reference | 1 | multi | 88,372.10 | 88,329.0 | 99,665 | 77,233 | 104,781 |
| reference | 1 | page | 221,291.75 | 213,272.5 | 272,619 | 204,262 | 283,416 |
| reference | 1 | combined | 731,163.15 | 731,100.5 | 740,159 | 723,558 | 744,921 |
| reference | 2 | combined | 768,070.50 | 735,397.5 | 945,028 | 721,478 | 1,060,419 |
| reference | 2 | page | 209,513.90 | 207,714.5 | 215,036 | 204,799 | 226,137 |
| reference | 2 | multi | 44,376.20 | 44,354.0 | 45,318 | 43,474 | 45,467 |
| reference | 2 | single | 19,022.65 | 18,829.5 | 19,404 | 18,517 | 21,903 |
| current | 1 | single | 71,923.35 | 71,272.5 | 76,128 | 63,885 | 90,681 |
| current | 1 | multi | 182,029.75 | 169,420.0 | 276,108 | 117,778 | 306,623 |
| current | 1 | page | 344,211.00 | 343,013.5 | 351,590 | 341,151 | 358,241 |
| current | 1 | combined | 774,387.20 | 773,527.0 | 783,392 | 765,091 | 788,321 |
| current | 2 | combined | 768,961.75 | 768,289.5 | 782,114 | 759,497 | 782,299 |
| current | 2 | page | 241,937.65 | 240,508.0 | 250,800 | 238,024 | 252,655 |
| current | 2 | multi | 78,783.10 | 78,663.0 | 79,307 | 78,266 | 79,905 |
| current | 2 | single | 22,386.85 | 22,143.0 | 22,526 | 22,048 | 26,889 |

Paired median changes use the same repetition/group reference denominator:

| Rep | Single | Multi | Page construction | Complete SSR |
|---|---:|---:|---:|---:|
| 1 | +85.37% | +91.81% | +60.83% | +5.80% |
| 2 | +17.60% | +77.35% | +15.79% | +4.47% |

These are negative candidate observations. Large simple-fixture repetition
variation remains visible: reference single is 38.449 / 18.8295 microseconds and
candidate 71.2725 / 22.143. Do not claim stable regression percentages, statistical
significance, universal throughput or exact CPU costs of individual branches /
reallocations / moves. No new repeats were added to select a preferred outcome.
The complete SSR medians are nevertheless worse in both paired repetitions,
which supplies no elapsed benefit to justify a default Core policy change.

### Logical allocation observations

All three operations within each variant/group are identical, release to each
own-start live and have zero failed/zeroed calls. Every released byte balance
holds including realloc old/new request differences. All nine reference page /
YT-151 pairs and all nine reference combined / YT-150 pairs reconcile all twelve
held/released relative fields. `measurement-summary.json` retains every field
and operation, paired changes and all statistics; the task-root `analyze.py`
performs direct retained-CSV analysis, without invoking workloads.

| Variant/group | Alloc | Realloc | Alloc request bytes | Realloc old bytes | Realloc new bytes | Held live bytes | Region peak bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| reference-single | 2,051 | 0 | 344,106 | 0 | 0 | 344,106 | 344,106 |
| reference-multi | 5,123 | 0 | 393,258 | 0 | 0 | 393,258 | 393,258 |
| reference-page | 12,305 | 0 | 1,641,666 | 0 | 0 | 1,641,666 | 1,641,666 |
| reference-combined | 12,308 | 21 | 1,641,741 | 507,852 | 1,260,387 | 490,386 | 2,149,003 |
| current-single | 2,051 | 0 | 147,498 | 0 | 0 | 147,498 | 147,498 |
| current-multi | 5,123 | 1,024 | 196,650 | 65,536 | 262,144 | 393,258 | 393,258 |
| current-page | 12,305 | 1,026 | 853,890 | 65,664 | 262,656 | 1,050,882 | 1,050,882 |
| current-combined | 12,308 | 1,047 | 853,965 | 573,516 | 1,523,043 | 490,386 | 1,558,219 |

Single-child held/peak bytes fall 196,608 (57.14%) with unchanged alloc calls.
Multi-child held/peak bytes stay unchanged while adding 1,024 realloc calls;
looking only at its lower initial alloc request bytes would hide subsequent
growth. Full-page construction held/peak falls 590,784 bytes (35.99%) while
adding 1,026 realloc calls. Combined's returned document stays at 490,386 held
logical bytes; its region peak falls 590,784 (27.49%) and realloc calls increase
from 21 to 1,047. No allocation-call elimination occurs.

At combined held, input destruction has already freed 12,307 allocations in
both variants; all other construction groups have zero held frees. Releasing
the result gives 2,051 / 5,123 / 12,305 / 12,308 total frees respectively and zero
own-start live in every case. Realloc old/new byte accounting measures requests,
not actual copies or internally simultaneous allocator blocks. Peak/live values
are this wrapper's logical accounting, not RSS, allocator metadata/size classes,
physical-memory savings or per-node ownership measurements.

### Failure history, controls and cleanup

Two preparation errors are retained: an initial read-only lookup of an absent
historical `prepare.py` helper (exit 2, no verification run), and an atomic
implementation patch rejected because its main-match context differed from the
actual file. The latter changed no files; one inspected-context correction
applied the original implementation. `initial-inspection-note.json` and
`implementation-patch-failure.json` retain the errors/classification. Neither
created a fresh task/clock, product repair or proof/workload retry. No native
build/test/output/measurement failed or timed out. The intentionally caught
conversion unwind is a passing test mechanism, not failed verification. Negative
candidate timing/reallocation results remain raw evidence, not erased failures.

Rust/Cargo 1.99.0, ext4 evidence/source/build directories, default release/System
allocator, absent selected Rust/Cargo release overrides/LD_PRELOAD/MALLOC_CONF,
and current cgroup/affinity limits were captured before assertions in
`environment.json` / `env-*` receipts. Session-161/user ancestors have unlimited
CPU/memory maxima; executable commands still use CPU 0 and 1 GiB address space.
Host frequency/governor/load/allocator internals and physical memory are not
isolated or qualified. Source archives and paths can affect code layout; both
variants use identical frozen sample inputs and current resource controls.

The fixed twelve-process measurement batch takes 331,768,980 ns, retaining
32,700 combined stdout/stderr/receipt bytes, within its 60-second / 8-MiB bounds
(`measurement-batch.json`). All owned foreground commands are waited, report
leader absence and no timeout. No service, listener, browser, container or
external resource is created. Inert archives/binaries/raw results remain under
the task root; the single 45-minute stage-2 clock is never renewed. Parent docs /
whitespace checks PASS and independent final Reviewer PASS is recorded above; the permitted status/reference closeout marks Done.
No new commit/push, default Core adoption or additional optimization is selected.

Parent new-results checks PASS: `cargo xtask check docs` reports 75 documents /
152 backlog items, valid internal links/structure/whitespace/DAG; tracked
`git diff --check` and both new-file no-index whitespace checks have empty
diagnostics. No-index exit 1 denotes added content, not a failed check.
`final-review-files.json` freezes the nine final review paths, including the
restored production Core/new test; `candidate-core.patch` separately shows the
implemented, compiled and rejected candidate against baseline. Post-PASS changes
remain limited to status/reviewer-reference closeout already declared above.
