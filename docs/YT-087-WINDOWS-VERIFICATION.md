# YT-087 — Additional Native Windows CLI Verification (2026-10-16)

**Current result: Native Windows GNU/MSVC compilation and expected fail-closed contract PASS after separately authorized environment provisioning; Windows scaffold/build/dev functionality remains unsupported.** The historical first attempt was **Blocked / Not verified** by missing prerequisites and is retained below. This is a two-runner Windows test of the *same uncommitted CLI source* independently accepted as an unpublished **Linux-only** developer preview. It does **not** reopen the bounded YT-087 Linux Done decision or establish Windows support. YT-020 production/public-release readiness remains Blocked.

## Source and runner identity

The authoritative source was the existing Ubuntu Yetlet checkout on `master`, HEAD `94cf8f555b554a26c2416842eab06110b229904d`, plus the uncommitted `tools/yetlet-cli/` source reviewed for YT-087a/b/c. The CLI itself has not been committed or pushed, so neither Windows checkout nor a remote branch contained the current CLI.

The source transfer used the authorized WebCodex project-file transfer interface, copying **15 ordinary CLI source, template, manifest and lock files individually** into task-owned Windows validation directories. Each Windows machine independently checked all 15 SHA-256 file digests against the Ubuntu source, with **zero mismatches**. The files were never merged into, or substituted for, either Runner's checked-out application sources. A compressed artifact placed under the Ubuntu `target/` directory could not be transferred because the artifact API denied that path; the blocked artifact was **not** exported or relocated to evade the restriction.

The GNU Runner had a clean but older Yetlet checkout, branch `maint/m10-qualification-20261010`, HEAD `3128d23591feee15ce976d7b83a56dcd587048d7`; source verification used the separately copied current CLI instead of that older checkout. The MSVC Runner used a task-owned directory within a local operations workspace, not a Yetlet Git checkout. Both machines were native Windows x86-64; this was not a Linux cross-compile or Wine emulation.

## Actual native Windows commands and results

All Cargo dependency-resolution attempts used `CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0`, `RUSTUP_SKIP_UPDATE_CHECK=1`, and an explicit Rust 1.99 target. They did not install crates, change the global toolchain default or contact an external registry.

| Evidence / test | Windows GNU | Windows MSVC |
|---|---|---|
| Toolchain | `1.99.0-x86_64-pc-windows-gnu` | `1.99.0-x86_64-pc-windows-msvc` |
| Exact source SHA-256 inventory | **15/15 matched** | **15/15 matched** |
| Native `rustc -Vv` | Exit 0 | Exit 0 |
| Cargo version | 1.99.0, exit 0 | 1.99.0, exit 0 |
| `cargo read-manifest --manifest-path ...` | Exit 0 | Exit 0 |
| `cargo fmt --manifest-path ... --check` | Exit **1**, toolchain missing `cargo-fmt.exe` | **PASS**, exit 0 |
| `cargo check --offline --locked --all-targets` | Exit **101**, uncached registry dependency `toml` | Exit **101**, uncached registry dependency `serde_json` |
| `cargo test`, Clippy, CLI binary/help/doctor/new/build/dev | **Not run:** compile unavailable | **Not run:** compile unavailable |
| Native Windows scaffold/process teardown/PTY/console | **Not verified** | **Not verified** |

**Interpretation:** Both Windows Runners parsed the identical Cargo manifest and had working Rust 1.99 compilers. Neither completed dependency resolution offline, so **no Windows CLI compilation or execution PASS exists**. Missing crates and the GNU formatter are environment prerequisites, not reproduced defects in the Yetlet CLI. In source, `new/build/dev` intentionally fail closed outside Linux; their *runtime* refusal was not exercised on Windows and must not be credited as a passing native test. Rustfmt MSVC PASS is formatting only, not functional compatibility.

A preflight script on the GNU machine initially failed at PowerShell parsing because its test helper was named `Do`; this was fixed in the verification harness before any Cargo operation. The result above comes from the corrected script, not a substituted outcome.

## Unexpected toolchain provisioning during preflight

The MSVC machine did **not** initially have the named Rust 1.99 toolchain. A preflight invocation of `cargo +1.99.0 --version` was inadvertently made before disabling rustup's auto-install behavior. Rustup consequently **auto-installed the 1.99.0 MSVC toolchain**. This was an unintended host change outside the original no-install verification plan and is disclosed here; it was not silently reversed, and no further installation or dependency download was attempted. The user's existing default Rust toolchain was not deliberately changed.

## Evidence retention and next admission gate

Raw source-verification reports, Cargo logs and a SHA-256 summary of the twelve Windows receipts are retained under the ignored **Ubuntu** directory:

`target/m13/yt087-windows/{gnu,msvc}/` and `target/m13/yt087-windows/windows-results-summary.json`.

Windows task-only source copies and logs were used solely for this verification, and were subsequently removed after archiving the twelve reports on Ubuntu; the GNU Runner's original Git checkout was confirmed clean at its original branch/HEAD. The unintentionally installed MSVC Rust toolchain remains installed (it was not uninstalled). The previous accepted Linux YT-087 unit tests (24 + 1 integration), HTTP/CSP results, native PTY Ctrl-C and resistant-child process tests are not rewritten or rerun. No Yetlet source, Cargo dependency, Windows implementation, Git branch, lockfile, repository release state or root build graph was changed by these Windows verification attempts.

A future **real Windows compilation** requires explicit provisioning of the compatible Cargo registry cache and, for Windows GNU, rustfmt. After those prerequisites are available, repeat the source-hash check and offline locked `cargo check/test`, then verify the actual built executable's help/doctor and expected non-Linux `new/build/dev` fail-closed behavior with native exit codes and no side effects. Enabling `new/build/dev` as functioning Windows features would additionally require a separately reviewed Windows no-replace/reparse-safe scaffold, Job Object/process-group and console cancellation implementation. This report does not authorize such implementation, new installation, network dependency resolution, commit, push or publication.

**Final disposition for this addendum:** Evidence of a bounded **Windows verification attempt**, with source integrity and manifest parse confirmed, build/functional acceptance **Blocked** by missing offline prerequisites. Independent read-only Reviewer Job `wc_job_sHIm-kmGxeN7eHsE` returned **PASS, zero issues and zero required followups** for the *accuracy of this evidence and limitation classification only*. It explicitly retained `windows_outcome.status=BLOCKED`, no Windows binary/runtime tests and no Windows feature acceptance. The reviewer did not reproduce an executable, authorize package installation or change YT-087's Linux-only Done scope.

## Authorized Windows environment provisioning and native validation PASS (2026-10-16)

**Current supplemental disposition:** Both real Windows x86-64 Runners have now **PASS** for native Rust 1.99 CLI compilation, `cargo fmt`, `cargo check --offline --locked --all-targets`, `cargo test --offline --locked`, strict Clippy and the **expected non-Linux fail-closed CLI contract**. The preceding Blocked result and its separate evidence Reviewer PASS remain historical; they describe the *first attempt before the maintainer authorized provisioning*. These new passes **do not implement Windows `new/build/dev` functionality** and do not expand YT-087 beyond its already accepted Linux-only developer-preview feature scope.

The maintainer explicitly authorized necessary installations to unblock Windows verification. On **GNU**, Rust 1.99.0 rustfmt/Clippy components were installed with pinned `rustup component add`. On **both** Windows Runners, `cargo fetch --locked --target <native Windows target>` downloaded the missing cached crates over the network; all subsequent formal compilation and tests used `CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0` and unchanged source/lockfiles. On GNU, the Rust self-contained `dlltool.exe` lacked a runnable assembler, initially producing a test build exit 101; this failure and the attempted bundled-tool PATH correction are retained separately. An attempted large verified WinLibs archive download was deliberately stopped before extraction in favor of a smaller official MSYS2 x64 binutils package. **No full WinLibs/Chocolatey system installation took place.**

The final GNU assembler was installed only to the user's `$HOME/.local/share/yetlet/windows-gnu-binutils-2.47-3/`, using official MSYS2 MinGW-w64 binutils 2.47-3 (archive SHA-256 `827363748ce3320683319d860ee4fcfcdbb36baf66aac6dab0ec3822be2d4ff7`) and its individually verified `gettext-runtime`, `libwinpthread`, `zlib`, `zstd` and `libiconv` runtime packages. Its SHA-256-checked `as.exe` reports GNU Binutils 2.47.20260726. The per-user `Activate-GNU.ps1` prepends only the current PowerShell process PATH and pins Rust 1.99 GNU; a fresh activation plus `cargo check --offline --locked` returned exit 0. No user-wide/system-wide PATH edit or default Rust toolchain switch was made. GNU rustfmt/Clippy and Cargo cache installations remain available on the Runner. On MSVC, the already-present Rust 1.99 toolchain was used, with the newly fetched local Cargo cache; the earlier unintended MSVC toolchain auto-install remains disclosed above.

| Source-bound native verification | Windows GNU (Rust 1.99) | Windows MSVC (Rust 1.99) |
|---|---|---|
| Exact 15-file source SHA-256 identity | **15/15 PASS** | **15/15 PASS** |
| `cargo fmt --check` | Exit **0** | Exit **0** |
| `cargo check --offline --locked --all-targets` | Exit **0** | Exit **0** |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit **0** | Exit **0** |
| `cargo test --offline --locked` | **5 passed / 0 failed**, exit **0** | **5 passed / 0 failed**, exit **0** |
| Native `yetlet --help` | Exit **0**, PASS | Exit **0**, PASS |
| Native `yetlet doctor --json` | Exit **1**, correct `new-platform=fail` | Exit **1**, correct `new-platform=fail` |
| Native `yetlet new/build/dev` | All exit **1**, correct unsupported-platform refusal; no scaffold | All exit **1**, correct unsupported-platform refusal; no scaffold |
| Unknown CLI command | Exit **2**, usage error | Exit **2**, usage error |
| Windows implementation of scaffold or foreground process ownership | **Not implemented / unsupported** | **Not implemented / unsupported** |

Only the five Windows-applicable parser, diagnostic and manifest tests execute per native target; the 24 Linux unit tests, Linux-specific Cargo integration test and native HTTP/PTY process lifecycle remain separately accepted on Linux. **Do not count a Windows refusal as a successful Windows scaffold, build, dev or doctor capability.** The Windows CLI `doctor` exits nonzero by design because it reports the unsupported `new-platform` capability. This is a *PASS of the expected failure contract*, not a positive doctor verdict.

Final toolchain and test receipts were transferred from the Windows Runners to the original Ubuntu workspace with byte-verifying project-artifact transfer and retained at `target/m13/yt087-windows/provisioned/{gnu,msvc}/` with `windows-provisioned-summary.json` (SHA-256 for each retained log). The original GNU Yetlet checkout remains on its prior maintenance branch/HEAD with tracked/index diffs empty; MSVC validated a task-owned copy in a non-Git operations workspace. The original Ubuntu YT-087 sources, root/xtask lockfiles and Git HEAD were not mutated. No commit, push, publication or deployment was performed.

**Current feature-support boundary:** Windows **compiles and passes native tests for the existing CLI**, but `new/build/dev` deliberately remain unsupported on Windows. Enabling them as positive Windows features needs a distinct reviewed Windows directory no-replace/reparse implementation and owned Job Object/console cancellation logic; neither is implied by these results. YT-087 remains Done only at Linux preview feature scope, and YT-020 remains Blocked. Independent read-only Reviewer Job `wc_job_mJX2-BYoH3e8CC4Y` completed with **PASS, zero issues and zero required followups** after independently checking source SHA-256 identity, all 20 retained native Windows receipts, exact test/CLI exit statuses, the historical failures and the Linux-only feature boundary. This Reviewer accepted **native GNU/MSVC verification and accurate scope classification only**, not positive Windows `new/build/dev`, deployment, public release or Windows process/console support.

After this review, the two Runners' task-owned temporary CLI source copies, build outputs, downloaded-package staging files and local duplicate logs were removed. All 20 byte-verified receipts remain in the Ubuntu ignored evidence directory above; the GNU runner's previous Yetlet checkout is clean at its unchanged maintenance branch/HEAD. The installed user-local GNU binutils and its activation script, pinned Rust components and each Runner's Cargo dependency caches were deliberately retained for future native verification. No global Rust default or user/system PATH was changed.
