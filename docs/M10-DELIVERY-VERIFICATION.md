# M10 Delivery and Cross-Environment Verification

**Date:** 2026-10-10
**Scope:** Maintainer PR/CI delivery, native Windows and Docker qualification, and the outcome of the requested Codex Security scan.

## Maintainer decisions and source identity

The maintainer authorized branch/PR/required-CI/merge delivery, testing on the Windows host already used for mint-game, installing compiler prerequisites, and testing inside Ubuntu Docker. The maintainer replaced the expensive external-assessor requirement with a Codex-assisted internal security scan. No paid engagement, external-audit claim, public visibility, package publication, production deployment or developer CLI is authorized by this work.

Candidate `3128d23591feee15ce976d7b83a56dcd587048d7` was pushed through maintainer PR #1. Required GitHub Actions run `38042614403` completed successfully: `Rust 1.88 MSRV` and `Stable native, samples, and Chrome`, including all three Chrome release suites. The PR was normally squash-merged as `c38837835dac683b09194b475baab2d8eda5a875`. Git tree equality between the candidate and merge commit was verified. The original Ubuntu checkout was preserved and fast-forwarded; no detached development worktree, protection bypass, reset or clean was used.

The staged and outgoing strict/builtin commit guard exited zero with warning status, not zero findings: 18 known license symlink-target coverage warnings and one absolute-drive-path test warning. Two blocking entries were repaired before commit: a machine-specific path was removed from a plan, and a redaction test now uses an explicit noncredential error-message sentinel. The ten focused tests passed, and independent reviewer `wc_job_GiP1-EnLMgPXkOlb` accepted that minimal correction. Matching staged tree identity was checked before committing.

## Native Windows execution

Testing used a separate clean Yetlet checkout on the authorized Windows runner; the mint-game repository and its existing dirty sources were not edited. The host was Windows build 26300, using PowerShell 7.6.6. The runner was not elevated and had no C++ workload installed.

A checksum-verified official rustup installer installed Rust 1.88.0 for `x86_64-pc-windows-gnu` in the user's profile. Portable w64devkit 2.10.0 supplied GNU compiler prerequisites; its release archive SHA-256 was `18d0a4c71a166f8401ab6305781bec5882b40b5e06ba9807c61cb5f3b3c6325e`. No Visual Studio workload, system-wide PATH or persistent environment override was changed. Tools remain available user-locally for later use.

The first compile check passed, but test linking exposed missing GNU prerequisites. Resolving the bundled dlltool, its assembler dependency and the unwind library selection required process-local PATH entries and `RUSTFLAGS`/`RUSTDOCFLAGS` set to `-C link-self-contained=yes`. This uses the actual Rust-distributed linker inputs; no empty library stubs, product source changes or skipped tests were used.

Final WebCodex Job `wc_job_2fmSUCLQI2b9dOe9` reached terminal exit 0. The nine reusable crates passed **80 native tests and four doctests** on candidate `3128d23`; the Windows checkout remained clean. These are executed Windows GNU tests, not cross-compilation and not local MSVC or Windows-browser evidence. The separate hosted Windows lane is responsible for its actual default target. The source Windows logs/report remain in that checkout's ignored `target/qualification/` directory; cross-project transfer was refused by artifact policy and was not bypassed.

## Offline Docker execution

The actual compiler image was pinned to `rust@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`: Rust/Cargo 1.88 on Debian 12, Linux x86_64. A Git archive of exact candidate `3128d23` and locked vendored dependencies were mounted read-only. The container used UID/GID 1000, no external network, no host ports, a read-only root filesystem, dropped capabilities, no-new-privileges, bounded CPU/memory/process limits and separate writable build/evidence paths. No Docker socket or credentials were mounted.

WebCodex Job `wc_job_MUaVCKwQYsrSwseh` reached terminal exit 0:

- `cargo test --workspace --all-targets --locked --offline`: **196 tests PASS**;
- `cargo test --workspace --doc --locked --offline`: **five doctests PASS**;
- real loopback HTTP inside the container: native Hello World SSR, escaped untrusted text, CSP, 404 and 405 **PASS**;
- the owned server was stopped and the named disposable container was removed.

The image/vendor/build caches remain local development inputs. No deployment image or application container service was published or left running. Evidence lives under `target/m10-delivery/docker/`; this tests native behavior in one container environment, not container orchestration, distributed providers or production availability.

## Original Codex-assisted security scan attempt: incomplete

The installed `security-scan` skill from Codex Security 0.1.32 was invoked for an earlier then-current repository commit `3128d23591feee15ce976d7b83a56dcd587048d7`, not a diff or Deep scan. Coordinator Job `wc_job_oCTu5__mlpWYcsLr` completed as a process, but its semantic outcome was **incomplete**, not PASS.

The authoritative scan ID is `580f6c9a-ded9-4f69-a3c9-16bbcc29ab97`. Capability preflight was ready. The coordinator reported ten fully reviewed files and terminated source-review workers; repository coverage and findings/severity totals were not finalized. The first canonical checkpoint failed because an ancestor of the scan output directory was group-writable without a sticky bit. After verifying ownership, the main session removed only the group-write bit from the owned Codex home directory (0775 to 0755); no credential or configuration file was edited.

A continuation of the **same** scan was then rejected by the platform safety check before returning an execution/Job identity. It was not retried with different wording/tools, emulated, replaced by another scan, or converted into a handwritten report. There is no completed canonical report and no valid repository-wide clean-result or vulnerability count. The original coordinator is terminal; the registered incomplete scan is not evidence of active background workers.

Under the maintainer's revised scope, YT-067 concerns internal automated/Codex-assisted source assessment only. At the time of this original attempt it remained Blocked because that scan had no final report, not because an external auditor was unappointed. Neither successful CI nor a document reviewer closes such a scan gap. An external engagement is optional future work and would need separate authorization.

## Later current-source Standard scan

A new official Codex Security 0.1.32 Standard scan was admitted for the independently updated, clean `master` source `25bff77ccc5a3641b30be27c4ca5d862c06f535d`. Scan `fc4d4af8-5864-4ace-b098-23ebfe249b31` reached terminal semantic completion, generating all canonical metadata and `report.md`; it reviewed 227/227 regular tracked files and 18 license symlinks and reported one Low finding (medium confidence), no Critical/High/Medium findings. The initial older-source scan remains incomplete and was neither resumed nor retroactively finalized. See [M10h verification](M10H-VERIFICATION.md) for the bounded `HttpExecutor` validation patch and actual regression/reviewer evidence. The detailed report remains in private scanner-owned state, not in the repository. Neither scan is an external audit or certification.

## Hosted qualification evidence

Qualification run `38043565238` reached terminal **success** on merge commit `c38837835dac683b09194b475baab2d8eda5a875`. All four jobs passed, and their named artifacts were downloaded and parsed rather than inferred from job titles:

| Hosted lane | Observed identity | Executed result |
|---|---|---|
| Ubuntu 24.04 native | Rust 1.99, x86_64-unknown-linux-gnu; image 20261004.327.1 | Nine crates, 80 tests/four doctests PASS |
| Windows 2025 native | Rust 1.99, x86_64-pc-windows-msvc; image 20260925.250.1 | Nine crates, 80 tests/four doctests PASS |
| macOS 15 native | Rust 1.99, aarch64-apple-darwin; image 20260907.0337.1 | Nine crates, 80 tests/four doctests PASS |
| Linux Firefox | Firefox 157.0, geckodriver 0.37.1, Python 3.12.15 | Todo/Search, keyed, multi-island and forms keyboard suites PASS; report cleanup PASS |

The weekly workflow is now available on the default branch. Its first recorded result is this manual dispatch; future scheduled results are not preclaimed. The separate merged-source baseline run `38043547271` also reached terminal success with both required jobs, including all Chrome release suites. The watcher Job `wc_job_6BYGF4yZfjh7aLe-` observed both runs to terminal exit 0; neither hosted result is inferred from local tests.

Real OS IME, screen-reader output, WebKit/mobile, zoom/conformance, production SLOs and actual distributed provider operations are not established by the automated lanes. Existing YT-066 local Session/SQLite results are not a distributed-store qualification. No developer CLI or new runtime dependency was added.

## Documentation closeout review

The first documentation-only reviewer `wc_job_Ml51hE5u64rltNhd` found a P2: three older CI/security-summary statements still described completed hosted qualification as pending. Those statements were corrected to reference actual run `38043565238` while retaining unverified manual and distributed scopes. No platform test or source scan was rerun for this prose repair. A repeat document review checks evidence accounting only; it was not a substitute for an actual security source scan. The later completed current-source scan and local finding remediation are independently recorded in M10h.
