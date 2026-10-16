# YT-088e — Linux/Windows Yetlet CLI Feature-Parity Final Verification

**Date:** 2026-10-16. **Final state:** Done at bounded unpublished Linux and Windows GNU/MSVC developer CLI feature-parity scope after independent final Reviewer `wc_job_IPC2lyaZyAz72KlT` PASS, zero issues and required followups. The developer CLI remains unpublished, not globally installed, and not production/release-qualified. YT-020 remains Blocked.

## Source identity and acceptance lineage

Use the original single checkout `$HOME/repos/yetlet`, branch `master`, HEAD `94cf8f555b554a26c2416842eab06110b229904d` plus the preserved uncommitted M13/M14 CLI implementation. There is no separate implementation worktree, source reset, commit, push or published crate.

| Milestone | Independent acceptance | Actual capability |
|---|---|---|
| YT-087 | Independent Linux 087a/b/c PASS, see [historical record](YT-087C-VERIFICATION.md) | Original unpublished Linux `doctor/new/build/dev` |
| YT-088a | Reviewer `wc_job_cO62Qi9H1GTmD9WI` PASS | Two narrow compile-time FS and process adapters; Linux code and tests preserved |
| YT-088b | Reviewer `wc_job_6Y-nSautm2RhPMPa` PASS | Windows GNU/MSVC safe no-overwrite NTFS `new`, real generated SSR HTTP/CSP, explicit offline app Cargo.lock |
| YT-088c | Reviewer `wc_job_8J7lc5L7tuyUwnWd` PASS | Windows native frozen Cargo `build` debug/release for actual generated Yetlet apps; owned Job/preflight/ACL and unchanged lock |
| YT-088d | Reviewer `wc_job_w0O0AXkCGwQAT5nP` PASS | Windows GNU/MSVC `dev` foreground Job Object + actual dedicated-console Ctrl-C, resistant descendants, HTML/CSP HTTP 200 and native exit/cleanup |

The original YT-088d role-directed Generalist execution `wc_job_dcHahdxuKlCEQ8Yv` was `lost/outcome_unknown`; its recovered changes were inspected after ensuring no competing process, validated through bounded current-session role emulation, then accepted by a distinct independent Reviewer. Historical earlier native failures and the original GNU task-parent ACL/linker observations remain unchanged in their respective receipts.

## Public CLI contract after parity

Both Linux and Windows x86-64 GNU/MSVC use the same binary command shape: `yetlet doctor [--framework-path PATH] [--json]`, `yetlet new NAME --framework-path PATH [--dir DIRECTORY]`, `yetlet build --manifest-path APP/Cargo.toml [--release]`, and `yetlet dev --manifest-path APP/Cargo.toml`. The CLI remains handler-first and native SSR by default; it does not add mandatory DI, MVC, JS/WASM, persistent event circuits, live reload or a browser runner.

`new` never downloads, compiles, installs or creates an application Cargo.lock. The user explicitly invokes `cargo generate-lockfile --offline --manifest-path APP/Cargo.toml` using already cached dependencies before using the CLI's `build`/`dev` `--frozen` operations. Neither build nor dev silently resolves/update locks or fetches dependencies. The new application uses explicit local paths to `yetlet-core` and `yetlet-server-axum` rather than a falsely published registry dependency. Both platforms preserve escaped View/HTML, strict CSP and loopback binding.

**Native platform restrictions:**
- Linux retains existing directory-handle/no-follow/no-replace security and owned process group/PTY cancellation semantics.
- Windows requires a local fixed NTFS target, verified owner/DACL and safe no-reparse traversal; unsafe, network and ambiguous parent security configurations fail closed. A trusted owner-protected `cargo.exe` must be selected on PATH for build/dev; unsafe rustup shim permissions are correctly refused. A real attached processed-input console is required for interactive dev; absent or incompatible console/nested Job conditions are explicitly not treated as working dev.
- Windows foreground supervision uses suspended Cargo creation, assignment to a private Job Object before thread resume, targeted child-group BREAK, bounded owned cleanup, and native `0xC000013A` console cancellation. This is not a secure sandbox and does not promise to contain deliberate session/job breakaway.
- The observed MinGW/GNU linker may refuse some Unicode **output EXE paths** despite the CLI's Unicode-safe Windows manifest/scaffold operations; use a compatible task-owned ASCII `CARGO_TARGET_DIR` if necessary. Windows MSVC retains separate Unicode generated-app build evidence. Linux vs Windows native error/exit codes are not assumed byte-identical.
- Other operating systems and unsupported filesystem, headless/console or application constraints fail closed. No production deployment, binary installer, crates.io publication, hosted release, performance qualification or supply-chain release gate is included.

## Evidence matrix and retained qualifications

Accepted Linux current-source regression from YT-088d: Rust 1.99 offline/locked **25 unit + 1 Cargo integration PASS**, fmt/check/strict Clippy PASS, plus MSVC cross-target check/Clippy. The corresponding historical YT-087 Linux real HTTP/PTY and resistant-descendant evidence remains retained; no new browser or GPU result is inferred.

Accepted Windows GNU/MSVC YT-088d source-bound evidence at `target/m14/yt088d/windows/`: each native Rust 1.99 toolchain ran **28 unit + 8 integration PASS**, fmt/check/strict Clippy exit 0, with actual CREATE_NEW_CONSOLE fixture for Ctrl-C/BREAK and a resistant child+grandchild ignoring console controls. An external test-only Rust driver started the public CLI `dev` against an actual generated Yetlet SSR app; native loopback HTTP 200, CSP, escaped meaningful text, no script, normal private-console Ctrl-C `0xC000013A`, reaped CLI and refused listener afterward, unchanged app Cargo.lock. The driver/Job evidence explicitly separates tested private consoles from arbitrary Windows console hosts. The current-source 088e CLI HELP and README/template refresh has passed separate native GNU/MSVC source-bound regression checks; the unchanged 088d production runtime source underlies the previous real SSR E2E; see the supplement below.

Retained raw 088a/b/c/d independent reviews, native GNU/MSVC actual generated-app logs and unchanged root/xtask manifests/lock/toolchain gates must be reconciled before closure. The initial Windows verification that **only** passed expected refusals remains history, not a substitute for actual Windows positive new/build/dev journeys.

## Final closeout gate

**Completed and independently accepted:** latest-source Linux and native GNU/MSVC Cargo fmt/check/test/strict Clippy, Windows help/doctor checks, docs/sample checks, root/xtask integrity and source/evidence reconciliation. Independent final Reviewer `wc_job_IPC2lyaZyAz72KlT` returned PASS, zero issues/followups, permitting YT-088 Done only at the bounded Linux + Windows CLI preview scope. YT-020 remains Blocked and no staging, commit or push is authorized.

## Latest-source acceptance evidence (2026-10-16)

The complete final 28-file CLI snapshot `artifacts/yt088-finish-20261016/cli-088e-final-source.zip` has SHA-256 `ca51ba0e31925da68005792076003c2a77130422b99fcd264579883b2572b578`. Each real Windows GNU and MSVC Runner independently verified all 28 file digests and ran Rust 1.99 offline/locked fmt, all-targets check, tests, strict Clippy (`-D warnings`) and the actual executable's `--help` and `doctor --json` with **native exit 0**. Each Windows suite passed **28 unit and 8 integration tests**, zero failed/ignored. Help identifies Linux/Windows correctly; doctor reports `new-platform=pass`.

The Ubuntu checkout simultaneously passed offline/locked fmt/check/test/strict Clippy (**25 unit + 1 integration, zero failed**), MSVC cross-target check, `cargo xtask check docs` (75 required documents / 88 backlog items), `cargo xtask check samples` (12 samples), and `git diff --check`. Committed root/xtask manifests/locks/toolchain matched HEAD; no explicit Python, Node, npm or npx interpreter process is launched by native CLI source.

The exact latest-source Windows receipts are retained at `target/m14/yt088e/windows/{gnu,msvc}/`, with raw hashes in `summary.json`. Comparing the latest and the accepted YT-088d E2E source archives proves that **only `README.md`, `src/lib.rs` (HELP wording) and `templates/README.md`** changed; production Windows/Linux filesystem/process, Cargo/lock semantics and `templates/main.rs` are byte-identical. The existing two-runner real SSR HTTP 200, CSP, escaped HTML, dedicated-console Ctrl-C, resistant-descendant and closed-listener receipts therefore remain historical proof for unchanged runtime code. No new HTTP test was falsely claimed for the changed help/documentation bytes.

**Final independent YT-088e acceptance:** Read-only Reviewer Job `wc_job_IPC2lyaZyAz72KlT` completed with `overall_status=pass`, `issues=[]`, and `required_followups=[]`. The reviewer independently matched the complete final 28-file source archive and working-tree bytes, Windows GNU/MSVC final logs and toolchain evidence, the source delta since YT-088d (only CLI README, HELP, generated README), accepted 088a–d real SSR/console/process evidence and preserved historical failures. It verified that generated-app instructions and Windows NTFS/ACL, trusted Cargo executable, processed-input console/nested-Job and GNU linker limits are honest. Reviewer performed no writes, tests, builds or deployments. **YT-088e and YT-088 are Done at bounded unpublished Linux + Windows x86-64 GNU/MSVC developer CLI scope only.** YT-020 remains Blocked; no commit, push, hosted release or public publication was authorized.

**Post-review temporary artifact disposition:** After verifying that GNU/MSVC source-bound receipts are preserved in the original Ubuntu `target/m14` evidence directories, the three task-owned YT-088d/e scratch directories on the GNU Runner were removed by their exact paths following archive checksum verification. An equivalent MSVC scratch cleanup request was blocked by the execution environment's safety controls; it was not retried or bypassed, so those isolated MSVC task copies remain. Previously installed Rust components, GNU user-local binutils and Cargo registry caches remain available. No source or acceptance claim was changed by this housekeeping.
