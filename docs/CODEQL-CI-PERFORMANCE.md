# CodeQL CI input scope and performance

**Status (2026-11-10): minimal change implemented in an independent worktree;
local tests and workflow lint PASS. Keep this change out of the CLI release
checkout/branch and do not merge it until the current release is complete. No
hosted run was cancelled, restarted or changed for this investigation.**

## Observed cost

The current workflow pins `github/codeql-action` to
`33d7b85da1a04171e211befe2c739303b9a4c066`, uses `build-mode: none` and runs
Actions, JavaScript/TypeScript, Python and Rust for every non-Markdown diff.
Evidence is retained in
`artifacts/verification/codeql-input-scopes-20261110/`, separate from Cargo output.
The investigation starts from `5dc8a9a990ac083e07081ea3be008748239c8a58`;
[CLI candidate PR #71](https://github.com/bohewu/yetlet/pull/71) proceeds separately.

Read-only GitHub job metadata and timestamped original Rust logs give:

| Stage | Master run 42753822402, source 5dc8a9a | PR run 42760173208, source 68bd7e5 |
|---|---:|---:|
| Job queue: created to started | 2.0 s | 2.0 s |
| Extraction | 371.5 s | 280.8 s |
| Database finalization | 65.6 s | 50.1 s |
| Query stage, including query loading | 543.0 s | 404.8 s |
| Query evaluation within that stage | 537.3 s | 400.3 s |
| HTTP results upload | 1.0 s | 1.6 s |
| Explicit processing-status wait | 5.5 s | 5.4 s |
| Whole Rust job | 1046 s (17m26s) | 777 s (12m57s) |

Sources: [master job](https://github.com/bohewu/yetlet/actions/runs/42753822402/job/129623462918)
and [PR job](https://github.com/bohewu/yetlet/actions/runs/42760173208/job/129643816904).
Evaluation is a subset of the query stage; these rows must not be summed twice.
Whole-job time also includes checkout/tool initialization, result interpretation,
cleanup and post steps. In the master job 33.4 seconds of cleanup occur between
HTTP upload and the explicit wait; this is not queue time or a measured upload
transfer. Both observed jobs use CodeQL CLI 2.27.4 and Rust queries 0.1.46.
These different revisions are observations, not a controlled speed comparison.

The completed same-tree pair for CLI candidate PR #71 is PR `68bd7e5` and
master `7d213d1`: their Git trees are byte-identical. The
[master Rust job](https://github.com/bohewu/yetlet/actions/runs/42761678430/job/129648624082)
takes 1008 seconds (16m48s): queue 2.0s, extraction 342.8s, finalization 65.3s,
query stage 534.3s (evaluation 528.8s), HTTP upload 1.1s and explicit processing
wait 5.5s. Combined with the 777-second PR job, the duplicate pair consumes
1785 runner seconds (29m45s). This is observed duplicate runner time, not measured
CPU utilization or evidence that all non-Git analysis inputs match. Original logs,
phase timings and the tree-equality check are retained in `timings-complete.json`
and `same-tree-pr-master-cost.json`.

## Minimal scheduling change

Keep the existing events, runner image, permissions, timeout, pinned actions,
query suite, `build-mode: none`, categories and PR concurrency policy. Change only
language selection; every selected language still scans its full existing source
scope. No tests, examples, fixtures, vendors or query results are excluded.

The scope job checks the complete NUL-delimited Git diff with rename detection
disabled, so deletions and both sides of renames remain visible. It inventories
both base and head. A small standard-library Python selector replaces the inline
Markdown-only rule; its offline tests run in the existing scope job before the
selection is used. The analysis matrix contains only selected languages. The
summary explicitly distinguishes an unselected language from a new successful
scan; no fake SARIF, success receipt or reused result is produced.

| Input | Analysis selected |
|---|---|
| Rust sources anywhere, including tests/build scripts; Cargo manifests/locks, Rust toolchain and `.cargo` settings | Rust |
| Every tracked file below a Cargo package directory | Rust, plus its own recognized language; intentionally conservative for embedded/generated assets |
| Literal Rust `include!`, `include_str!`, `include_bytes!` targets, including outside the package | Rust even for Markdown, CSS, JSON, SQL or other data |
| Python source/stubs/notebooks and Python manifests, dependency locks and tool-version settings | Python; JSON notebook containers also keep JavaScript/TypeScript |
| JavaScript/TypeScript, supported template/data formats, package locks, Node/npm/Yarn settings | JavaScript/TypeScript; Rust too when inside a Rust package or embedded |
| Workflow/composite-action definitions | Actions plus JavaScript/TypeScript for YAML input coverage |
| CodeQL workflow/selector, discovery settings, query/model packs and their data, queries or CodeQL configuration | All four |
| Ordinary Markdown outside Rust package/include inputs | No analysis, preserving the existing docs optimization |
| Unknown data/tools, missing history, invalid/empty diff, unsupported inventory, schedule or manual run | Full applicable analysis; schedule/manual always all four |

Rust include discovery accepts parentheses, braces, brackets and split-line
invocations. Rust whitespace/comments are skipped at the invocation boundary;
unparsed arguments keep Rust mandatory. The focused regression exercises the
actual Git prefilter as well as external embedded-file selection.

The known Style Showcase build script reads its package and two other Rust
packages. A reviewed immutable Git blob identity binds this exception; all files
in those packages remain Rust inputs. A changed/new/custom build script or an
unresolved include makes Rust mandatory until its input boundary is reviewed.
Known `OUT_DIR` outputs are accepted only for that reviewed generator. Arbitrary
generator dependencies are not inferred from file extensions. This trades some
extra scans for coverage and leaves future unusual inputs on the full-scan path.

The branch protection readback requires `Rust 1.99 MSRV` and
`Stable native, samples, and Chrome`. Those jobs/workflows are untouched; no
required check is removed. Existing CodeQL language names are preserved whenever
selected. If language-specific CodeQL jobs later become required contexts, review
conditional-matrix semantics before changing policy; do not silently edit branch
protection or substitute a green placeholder.

## Database/result reuse decision

**Do not introduce Rust database caching or PR-to-master result reuse here.**
The pinned Action's
[overlay feature maps and language gate](https://github.com/github/codeql-action/blob/33d7b85da1a04171e211befe2c739303b9a4c066/src/config-utils.ts#L535-L600)
include C/C++, C#, Go, Java, JavaScript, Python and Ruby, with no Rust entry;
missing entries disable automatic overlay. Both retained Rust logs explicitly
report overlay mode `none` without caching and perform extraction plus queries.
The current official
[incremental CLI documentation](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis#minimum-cli-bundle-versions)
also lists per-language overlay versions without Rust. General incremental or
alert-filtering documentation is not proof of Rust database reuse support.
Cargo caches do not retain a CodeQL database/query result.

The PR and squash-merged master for CLI candidate PR #71 have identical Git trees,
but that alone is insufficient for safe reuse: PR diff-informed settings, default
branch alert coverage, commit/ref/category attribution and complete tool/query/
extractor/environment identities must also agree. Action SHA alone does not prove
all those inputs. Re-uploading PR SARIF as a master scan could misrepresent
coverage. Keep real full Rust scans for relevant PR and master changes; retain
weekly/manual full coverage. No cache key or cross-ref trust protocol is added.

## Validation and before/after claims

- Eleven standard-library regression tests PASS: complete diff/deletion inventory,
  literal/embedded inputs, manifests/locks/tool settings, query-pack data,
  unreviewed generators, uncertainty fallbacks and honest scope summaries.
- `actionlint` PASS for the modified workflow; whitespace and documentation checks
  are retained with the local evidence. No release workflow or Runner setting is
  changed.
- Actual historical diffs `bb414d1`, `864f2ab`, `1c0bb31` previously selected all
  four. The new selector selects Actions, JavaScript/TypeScript and Python for
  each, with Rust omitted; complete local selection takes about 1.2–1.4 seconds.
- Initial committed input inventory took about 0.62 seconds locally (51 Cargo
  packages, 171 literal include targets, no uncertain Rust input).
- A root Python-only change goes from four jobs to Python only. An ordinary docs
  change stays unselected; an embedded template Markdown change now correctly
  selects Rust. Rust source/lock changes still select Rust; policy/query/unknown
  changes keep full scans.

For unaffected Rust inputs the new schedule removes one Rust job per event, whose
observed cost above was 777–1046 seconds. This is a scheduling replay/projection,
**not measured hosted post-change wall time**. No new workflow was pushed or
executed while the current release CI runs; actual before/after hosted totals
remain to be measured after the release gate. Rust-relevant input changes are not
expected to get faster, and PR/master duplicate Rust work is deliberately retained.

## Delivery boundary

The review diff consists only of `.github/workflows/codeql.yml`,
`.github/scripts/codeql_scope.py`, `.github/scripts/test_codeql_scope.py` and this
record. Preserve the independent worktree for review; do not apply it to the
release checkout or merge before the Owner's release-completion condition.
SDK/CLI historical bindings, Reader Edition, signing/publication, Runner
permissions and branch protection remain unchanged. Future hosted timing and any
safe Rust database-reuse proposal require separate evidence, not a claimed cache
hit or a skipped required scan.
