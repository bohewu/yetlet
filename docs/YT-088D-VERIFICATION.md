# YT-088d — Windows Native Foreground Dev and Console/Job Verification

**Date:** 2026-10-16. **Final status:** Native Windows GNU/MSVC foreground dev independently accepted after source-bound SSR HTTP, real Console cancellation and resistant Job cleanup tests; final Reviewer `wc_job_w0O0AXkCGwQAT5nP` PASS, 0 issues/followups. This does not mean YT-088 or the broader product release is Done.

## Scope and unchanged baselines

The task extends the CLI's existing internal `platform::process::windows` adapter to support foreground `yetlet dev --manifest-path APP/Cargo.toml`, while preserving the approved Linux process-group/PTY and Windows safe `new`/frozen `build` contracts. Design YT-088a and functional YT-088b/088c have separate independent Reviewer PASS receipts in [the M14 plan](YT-088-CROSS-PLATFORM-CLI-PLAN.md). YT-087 remains Done at its historical Linux preview scope; YT-020 release readiness is Blocked.

The **original** role-directed Generalist Job `wc_job_dcHahdxuKlCEQ8Yv` returned `lost / outcome_unknown` after Runner reconciliation; no terminal completion or independent acceptance is claimed for that Job. The parent inspected the same checkout, found no active Codex/Cargo processes or another active workspace job, and preserved its partial source/files. The parent then used bounded **current-session role emulation** for verification, a direct test enhancement and documentation. This does not substitute for a distinct read-only Reviewer.

Checkout: `$HOME/repos/yetlet`, `master`, HEAD `94cf8f555b554a26c2416842eab06110b229904d`. The full uncommitted M13/M14 state remains in the same worktree, with no staging, reset, stash, extra worktree, commit, push or publication.

## Windows process implementation

The existing native `windows.rs` path now supports both `cargo build --frozen` and `cargo run --frozen` via an explicit vetted executable, Windows Unicode argument/environment handling, owned inheritable standard-handle allowlist, `CreateProcessW(CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP)`, and assignment to a new private `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` Job **before** resuming its primary thread. App cwd, manifest, lock and Cargo executable identity/ACL remain pinned and checked; unsuccessful assignment/verification never launches an unowned child. Process and Job cleanup is bounded and owns only the handles/groups created by the CLI, never arbitrary process names.

For `dev`, an attached Windows console with processed Ctrl-C input is required, otherwise it fails closed before starting Cargo. `SetConsoleCtrlHandler` observes native Ctrl-C and Ctrl-Break on the CLI; it does not broadcast CTRL_C to a shared console group. The supervisor sends `CTRL_BREAK_EVENT` **only** to its own held child process group where applicable, waits for a bounded grace period and terminates/reaps the private Job on failure to cooperate. Previous console handler state and caller console mode are not deliberately overwritten. Native exit/status values remain Windows values (not fabricated POSIX signals); `0xC000013A` is observed on normal Ctrl-C of the generated SSR app. Intentionally escaped descendants and unsupported console/headless contexts are outside this bounded support envelope.

The parent added a focused adversarial case to `tools/yetlet-cli/src/platform/process/windows_tests.rs`: an app process **and** its descendant ignore both Console C and BREAK; the test uses a **private `CREATE_NEW_CONSOLE` fixture**, generates a real Ctrl-C, verifies bounded owned Job termination, native cancellation exit, reaped descendant, unrelated sentinel survival, and restoration of the previous handler. The descendant owns a wait handle and a waiter thread; strict Clippy remains `-D warnings`. The unchanged Linux and accepted Windows process implementation bytes were not edited by this test-only enhancement.

## Rust checks and native tests

The interrupted original Job had already recorded exit 0 for Linux and MSVC cross-target compile/Clippy. Following the test enhancement, the original Ubuntu checkout reran Rust 1.99 offline/locked `fmt`, `check --all-targets`, `test` (**25 unit + 1 Cargo integration PASS**), strict Clippy, and MSVC cross-target `check`/strict Clippy: all completed exit 0, no skip, test weakening or lint downgrade.

A frozen **28-file CLI source** after the regression was transferred to the two authorized Windows Runners: ZIP SHA-256 `3e4cbb65eb5a7a236cc36fbaeb62d4107e8fa236fbbdd1d43a2939f6eb4136be`. Each Runner checked 28/28 embedded file SHA-256 values before compilation. All native Windows Rust 1.99 full-suite runs completed:

| Native check | Windows GNU | Windows MSVC |
|---|---|---|
| `cargo fmt --check` | Exit 0 | Exit 0 |
| `cargo check --offline --locked --all-targets` | Exit 0 | Exit 0 |
| `cargo test --offline --locked` | **28 unit + 8 integration passed**, 0 failed/ignored | **28 unit + 8 integration passed**, 0 failed/ignored |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 | Exit 0 |
| Dedicated resistant-Ctrl-C/owned Job/handler test | PASS in real private Console | PASS in real private Console |

Seven other executable tests were already covered in 088b/c; the latest 8 integration tests continue to include actual scaffold, frozen build, lock/cache/path/ACL and failure cases. The Windows native `dev_native_console_exit_identity_and_handler_restoration` test exercises genuine console events, including its extended resistant process/descendant case, not an artificial PID signal or skipped fixture.

## Actual generated application end-to-end runtime

A separate task-only **external Rust 1.99 validation driver** was compiled natively on each Windows Runner from `artifacts/yt088-finish-20261016/yt088d-e2e-console.rs` (SHA-256 `a81f74d8e2625ea42c2a265aa3401a4775479b1a64d8e3236fefd5d0219b5943`). It is **not** a CLI dependency, a framework crate, part of the active Rust toolchain, or a public API. Its only purpose is to launch a new isolated Windows `CREATE_NEW_CONSOLE`, spawn the genuine CLI `dev` against the *previously generated, full-source local Yetlet SSR app* from accepted 088c, poll an actual loopback listener, fetch real HTML, send terminal Ctrl-C **inside that private console**, and check native exits/cleanup. It never broadcasts console events to the user's console or other applications.

The generated apps have local `yetlet-core`/HTML/server-Axum path dependencies, their own explicit `Cargo.lock` and a pre-existing trusted owner-only Cargo proxy, using the existing Windows GNU/MSVC Rust 1.99 local toolchains. The CLI runs `cargo run --frozen`, not a direct app-process substitution. All measured checks:

| Source-bound runtime observation | Windows GNU | Windows MSVC |
|---|---|---|
| Native `yetlet dev` reached the generated `realapp.exe` | PASS | PASS |
| Loopback HTTP 200; Content-Security-Policy | PASS | PASS |
| Escaped `Hello &lt;Yetlet&gt; &amp; Rust!` and no `<script` | PASS | PASS |
| Dedicated private-console Ctrl-C delivered to CLI | PASS | PASS |
| Native exit on Ctrl-C | `0xC000013A` | `0xC000013A` |
| Exact owned CLI process reaped | PASS | PASS |
| App listener refuses new connections after cancellation | PASS | PASS |
| App `Cargo.lock` SHA-256 before/after | Identical | Identical |
| Test-only console isolated from existing user console | PASS | PASS |

Actual ephemeral ports were **60359** (GNU) and **30894** (MSVC). The bounded outer test driver retains the spawned helper handle and force-kills **only** its owned helper on timeout; each helper owns the exact CLI child and uses the CLI's own private Job cleanup. There is no process-name-based kill, background daemon or host console mode mutation.

Importantly, **only `src/platform/process/windows_tests.rs` changed between the first successful external SSR E2E snapshot and the final revalidated source**; every production CLI source file and Cargo.lock hash is byte-identical. The post-test current-source full Windows suite therefore validates the same production implementation used by the HTTP/CSP/terminal journey. This source binding is checked in both native `source-bound-summary.json` receipts, rather than silently attributing an old run to a changed binary.

## Evidence, limits and review gate

Original real-console E2E, all current-source GNU/MSVC Rust checks, the resistant process/descendant regression and SHA-256 inventories are retained at `target/m14/yt088d/windows/{gnu,msvc}/`, with `target/m14/yt088d/windows/summary.json`. Linux results and original interrupted-Job diagnostic logs remain under `target/m14/yt088d/`. The external driver source and frozen CLI archives remain in the task-only `artifacts/yt088-finish-20261016/` area until final evidence reconciliation.

The Runner-specific GNU linker caveat for arbitrary Unicode **EXE output directories** from 088c remains; 088b demonstrates Windows Unicode file/manifest paths and MSVC native build, but arbitrary MinGW linker encoding is not claimed as a Yetlet-controlled guarantee. This testing does not certify every Windows ACL, nested Job variant, console host, detached child or unsupported headless scenario, nor production release, public install or CI hosting.

**Next required gate:** independent read-only YT-088d Reviewer PASS on this exact source, the resistant native fixture and both actual CLI `dev` runtime receipts, zero blocking findings. Only after that may 088e cross-platform docs and final acceptance be marked complete. **YT-088 remains In progress; YT-087 Done at historical scope; YT-020 Blocked.**

## Independent final YT-088d acceptance

Distinct read-only Reviewer Job `wc_job_w0O0AXkCGwQAT5nP` returned **PASS**, `issues=[]`, `required_followups=[]` after inspecting original and updated 28-file source hashes, private suspended Cargo creation/Job assignment, target-group Console BREAK and Ctrl-C handling, handler restoration, bound cleanup, Linux preservation, the real GNU/MSVC SSR E2E receipts and the resistant-owned-descendant/sentinel regression. No code was modified or tests were run by the Reviewer. It explicitly retained the original lost/outcome-unknown Generalist Job, the Windows headless/nested Job limitations and the historical YT-087 acceptance. **YT-088d is Done at the bounded Windows GNU/MSVC attached-console preview scope**. Only YT-088e final closeout remains; YT-020 stays Blocked. No commit or push was authorized.
