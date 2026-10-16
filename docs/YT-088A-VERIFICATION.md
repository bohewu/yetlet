# YT-088a — Internal CLI Platform Seams

**Date:** 2026-10-16. **Final result:** Internal adapter extraction accepted by distinct independent Reviewer `wc_job_cO62Qi9H1GTmD9WI` PASS (zero issues/followups) after Linux and native Windows GNU/MSVC source-bound regression evidence; see final acceptance note. YT-088 remains **In progress**, YT-087 remains **Done at its accepted unpublished Linux-only scope**, and YT-020 remains **Blocked**.

## Authority and source

This bounded task implements only the two internal compile-time OS seams approved in the [YT-088 plan](YT-088-CROSS-PLATFORM-CLI-PLAN.md). Design Reviewer `wc_job_SPLMk3nKvrzHPwYV` returned PASS with zero issues/followups for the design, not this implementation. Work used the original `$HOME/repos/yetlet` checkout, branch `master`, HEAD `94cf8f555b554a26c2416842eab06110b229904d`. Existing uncommitted YT-087 source, docs and Windows verification receipts were preserved.

The task applies the canonical Generalist contract and reasoning/materiality policies as a role-directed Codex job. It is not native managed dispatch. Launcher Job ID and observed model/effort evidence are not available in this child context and must be correlated by the parent; no native role match, model, effort or independent-review identity is asserted. No agent was dispatched by this task.

## Actual changed files

All source paths below are relative to `tools/yetlet-cli/src/`.

| Source | Bounded change |
|---|---|
| `lib.rs` | Register the private `platform` module; remove root Linux module selection. Public modules, parsing, HELP bytes and parser/template tests are unchanged |
| `scaffold.rs` | Retain shared names/templates and check platform capability before framework inspection; publish through `platform::fs::create_project` |
| `framework.rs` | Retain absolute-path and framework identity policy; delegate file reads through `platform::fs` |
| `doctor.rs` | Use adapter capability/message constants; preserve diagnostic bytes and local version probes |
| `app.rs` | Retain public Failure/Completion/run contracts; share bounded UTF-8/TOML parsing, standalone manifest/lock policy and frozen literal Cargo argv/environment; delegate native operations |
| `platform/mod.rs` | Internal seams, one static entrypoint signature test, and one non-Linux refusal regression compiled only on non-Linux |
| `platform/fs/mod.rs` | Compile-time backend selection, early scaffold capability refusal, shared manifest-then-lock validation order |
| `platform/fs/linux.rs` | Existing protected-directory/file preflight split into a temporary AppFiles context and pinned App; retain ownership/mode checks, final full-chain identity verification and the three original app tests |
| `platform/fs/safe_dir.rs` | Byte-identical relocation of original `safe_dir.rs`, including all accepted tests, O_NOFOLLOW, renameat2 no-replace and identity-checked cleanup |
| `platform/fs/unsupported.rs` | Original non-Linux scaffold/app refusal strings and original limited doctor framework read; no secure Windows app support |
| `platform/process/mod.rs` | Internal literal-command/supervision entrypoint and shared exit finalization; keep native code and fallback values |
| `platform/process/linux.rs` | Relocate pre-exec fchdir and native final signal handling; use the existing foreground supervisor |
| `platform/process/foreground.rs` | Byte-identical relocation of original `foreground.rs`; subreaper, pinned leader, notifier, terminal restoration and cleanup deadlines unchanged |
| `platform/process/foreground_tests.rs` | Relocate original tests; change only the literal Rust test selector to `platform::process::linux::foreground::tests::owned_fixture` and its rustfmt layout |
| `platform/process/unsupported.rs` | Fail closed without spawning; preserve non-Linux final-status behavior |

The old root `src/safe_dir.rs`, `src/foreground.rs` and `src/foreground_tests.rs` paths were relocated, not discarded. Documentation changes are only this file and the [YT-088 plan](YT-088-CROSS-PLATFORM-CLI-PLAN.md). `main.rs`, templates, Cargo manifests/locks, existing CLI integration test and README, root README, backlog, roadmap, docs index and all YT-087 verification files retain their starting bytes. Root/xtask graph, lock, toolchain and Cargo configuration also match their committed HEAD blobs. No public trait, DI container, dynamic backend, dependency or framework-crate change was introduced. Target-specific source selection is contained under `src/platform/`; the existing Linux integration-test gate and Linux-only libc dependency remain unchanged.

## Linux verification

Evidence directory: `target/m14/yt088a/`. All Cargo commands use the already installed **1.99.0** toolchain with `RUSTUP_AUTO_INSTALL=0`, `RUSTUP_SKIP_UPDATE_CHECK=1`, `CARGO_NET_OFFLINE=true`; dependencies remain offline and locked.

| Actual final command | Native result | Receipt |
|---|---|---|
| `cargo +1.99.0 fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | Exit 0 | `fmt.log` |
| `cargo +1.99.0 check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | Exit 0 | `check.log` |
| `cargo +1.99.0 test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | Exit 0; **25 unit + 1 Cargo integration passed, 0 failed, 0 ignored**; binary unit/doctest cases: 0 | `test.log` |
| `cargo +1.99.0 clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | Exit 0; no lint suppression added | `clippy.log` |

The prior 24 unit + 1 integration contracts remain active; one static seam signature test raises the Linux unit count to 25. Existing tests exercise no-replace races/foreign-data preservation, manifest/lock security and exact argv, native child exit 23, stale/missing lock/cache failures, native signals, owned descendants/orphans, resistant terminal Ctrl-C, terminal restoration and unrelated process survival. No new generated-app HTTP or browser test was run. The original accepted [087a HTTP](YT-087A-VERIFICATION.md) and [087b host HTTP/PTY](YT-087B-VERIFICATION.md) receipts remain historical and source-scoped; their actual files remain present at `target/m13/yt087a/local-vH7Tot/`, `target/m13/yt087b/host-runtime-wljaa6tw/` and `target/m13/yt087b/p2-tty-repair/`. Relocation does not turn them into new YT-088 runtime results.

`cargo +1.99.0 xtask check docs` and `cargo +1.99.0 xtask check samples`, under the same offline/no-install environment, both returned **exit 0**. The existing checker reported **75 catalog documents, 88 backlog items and 12 sample entries**; receipts are `docs.log`, `samples.log` and `documentation-checks.json`. Source checks above attest compilation/native process behavior, not a new SSR/CSP/browser qualification.

`git diff --check` returned **exit 0**. Focused `git diff --no-index --check -- /dev/null PATH` covered all 16 CLI Rust files and both YT-088 documents, with **empty whitespace diagnostics**; native exit 1 denotes the expected untracked content difference, not a failure. Exact results are in `whitespace.json`. Baseline snapshots and SHA-256 records are in `baseline/` and `baseline-sha256.json`; `preservation.json` confirms **22 starting files unchanged**, byte-identical safe_dir/foreground relocations, the exact limited fixture change, unchanged public HELP/parser tests, and all six root/xtask graph/lock/toolchain/config files matching committed HEAD. Existing Windows verification document bytes are unchanged.

## Bounded repair history

`repair_attempts_used=2` of limit 2; no further product repair was attempted:

1. `product_failure`: first all-target check exited 101 with E0599 because the extracted shared parser lacked the Read trait import. Added that import and removed the now-unused moved-test import. The next check and all 25+1 tests passed.
2. `product_failure`: first strict Clippy exited 101 for items-after-test-module and the explicit seam function-pointer type complexity. Moved the framework-read function before tests and named the test-local function-pointer type. No assertion, runtime behavior or lint policy was weakened. Final fmt/check/test/strict Clippy passed.

`operational_retries_used=1`: a one-off local editing command stopped on a substring lookup before the first build. Its partial edits were inspected, then the remaining authorized extraction was completed in place. No new checkout, dispatch or verification run was created. One preservation assertion also classified as `harness_failure`: its whitespace normalization omitted rustfmt's added trailing array comma after the relocated fixture selector. The exact diff was inspected and that single comparison corrected; no product source or test assertion changed for it. Original compile/lint failure receipts remain in `check-initial.log`, `clippy-initial.log` and the initial check JSON files. The last product failure signature was `items_after_test_module + type_complexity`, resolved by the final strict Clippy exit 0.

## Functional limits and parent handoff

Windows `new`, `build` and `dev` remain unsupported and fail closed. The non-Linux backend refuses before framework inspection/template rendering for new, and before app file reads or child startup for build/dev. Public command names/arguments, error text, diagnostic text and exit mappings remain unchanged. The prior Windows doctor probe remains a limited path-based diagnostic; it is not a reparse-safe app context or a Windows security claim. Native directory-handle/ACL/no-replace work belongs to YT-088b/c; suspended Cargo, Job Object assignment and console cancellation belong to YT-088d. Other OS functionality is deferred.

The parent must transfer/compare the **22-file final source manifest** `target/m14/yt088a/cli-source-sha256.txt` (SHA-256 `3c623af340bb20aa94f779b4f45415aa94e3254a8ff0e3e18dbc37dca9be2383`) before native GNU and MSVC validation. It must also account for the three relocated old paths, using a task-scoped source overlay rather than unrelated checkout cleanup. On each existing native Windows Rust 1.99 runner, run the same pinned fmt/check/test/strict Clippy commands, collect actual source hashes, native exit codes/test counts and exact expected unsupported command results. Seven unit cases are expected there (the original five shared tests plus static seam and non-Linux refusal tests); this is an expectation, **not an executed Windows PASS**. The Linux-only Cargo integration remains inapplicable. Old [Windows receipts](YT-087-WINDOWS-VERIFICATION.md) do not validate these changed source files. This task did not access a Windows checkout or run cross-compilation.

After native checks, the parent arranges a distinct read-only independent implementation Reviewer against these sources and receipts. This implementation/verification is not self-review acceptance. The immediate next unblockable action is parent-side source-bound Windows validation; **YT-088b** is the next implementation slice only after YT-088a's required native checks and independent PASS. No new architecture decision is needed for this extraction; future Windows native API choices remain separately reviewed open work.

## Resource and checkout closeout

All task Cargo command sessions completed. The existing native process tests passed their owned PID/PGID absence, descendant reaping, terminal restoration and sentinel cleanup assertions; their temporary app/cache directories use the existing RAII fixtures. A fresh `ps -eo pid,ppid,pgid,comm` inspection found no remaining Cargo, Rust test fixture, Yetlet server, watcher or browser process in this task's namespace. No listener/server/browser was started for a new journey. This is task-namespace evidence, not a host-wide process claim.

Final checkout remains original `master` at `94cf8f555b554a26c2416842eab06110b229904d`, with an empty staged index and the extensive pre-existing uncommitted YT-087 changes retained. No stash, reset, clean, detached worktree, clone, stage, commit, push, global installation, publication, hosted CI, deployment or Windows access occurred. No task process remains running; ignored local evidence is intentionally retained for the parent.

## Parent-arranged current-source Windows GNU/MSVC validation (2026-10-16)

A frozen ZIP of the actual **22-file post-extraction CLI source** (SHA-256 `fbc3b6ab56522c2cd93f38f74568400ea9128acd19424ceb0433443a5fbefbdb`) was transferred through the authorized project-artifact channel into two isolated native Windows Runner test folders. Both machines verified every source file's SHA-256 against the embedded manifest before running tools. The original old Windows Git checkout was not substituted for the uncommitted Ubuntu implementation; source selection remained under the task-only artifacts directory.

| Actual check (Rust 1.99; offline/locked) | Windows GNU | Windows MSVC |
|---|---|---|
| Source snapshot identity | **22/22 PASS** | **22/22 PASS** |
| `cargo fmt --check` | Exit **0** | Exit **0** |
| `cargo check --all-targets` | Exit **0** | Exit **0** |
| `cargo test --offline --locked` | **7 passed, 0 failed**, exit **0** | **7 passed, 0 failed**, exit **0** |
| `cargo clippy --all-targets -- -D warnings` | Exit **0** | Exit **0** |
| Native `yetlet --help` | Exit **0** | Exit **0** |
| Native `yetlet doctor --json` | Exit **1**, expected `new-platform=fail` | Exit **1**, expected `new-platform=fail` |
| Native `yetlet new/build/dev` | Each exit **1**, expected unsupported; no project/child created | Each exit **1**, expected unsupported; no project/child created |
| Unknown command | Exit **2**, usage | Exit **2**, usage |

The seven Windows test cases are the original five shared tests plus the new seam signature and non-Linux refusal regression. The Linux-only test cases and Cargo integration target are correctly absent on Windows. This is **passing 088a behavior-preserving Windows compilation and refusal semantics**, **not** positive Windows functionality. The previously installed local Windows GNU binutils were activated only for the task-local PowerShell process; no additional components or dependencies were installed by this validation.

Original runner logs and byte-verified copies are retained at `target/m14/yt088a/windows/{gnu,msvc}/`, with SHA-256 manifests in `target/m14/yt088a/windows/summary.json`. The source snapshot and expected Linux unchanged-manifest claim are preserved separately. No Windows application build, HTTP listener, new native Job Object or Windows no-replace directory implementation was launched for this slice. This source-bound Windows evidence is now ready for a **separate independent 088a implementation Reviewer PASS**, not an assertion that the reviewer gate has already passed.

## Final independent YT-088a acceptance (2026-10-16)

Separate read-only implementation Reviewer Job `wc_job_cO62Qi9H1GTmD9WI` returned **PASS**, `issues=[]`, `required_followups=[]` after examining the actual moved/rewired CLI source, all retained preservation hashes, the Linux 25+1 suite, the 22-file frozen source identity and **both native Windows GNU/MSVC 7/7 regressions**. It independently confirmed that original Linux safe-directory/foreground implementations were moved byte-identically; shared Cargo TOML, lock and argument handling, native statuses and error strings remained equivalent; Windows non-Linux operations still fail before side effects. No additional dependencies or public API were introduced. The Reviewer performed no edits, fresh builds or runtime journeys.

**YT-088a is Done at bounded behavior-preserving internal adapter extraction scope.** This does not establish successful Windows `new`, `build` or `dev`; 088b–088e remain to implement and review. YT-088 remains In progress, YT-087 maintains its separately accepted Linux preview Done status, and YT-020 remains Blocked. The original source checkout stays at HEAD `94cf8f555b554a26c2416842eab06110b229904d` with an empty staged index; no commit/push was authorized or performed.

**Final resource cleanup:** After both native Windows receipts were transferred, byte-verified, retained, and independently reviewed, task-only `artifacts/yt088a-windows/` source/build/output directories were removed on both Windows Runners. The GNU Runner's maintenance checkout remained at its original branch/HEAD with tracked and staged diffs empty; the pre-existing user-local GNU binutils toolchain was retained. The Ubuntu `target/m14/yt088a/windows/` original log/hash evidence and original CLI checkout were preserved. No public install, Git commit or push occurred.
