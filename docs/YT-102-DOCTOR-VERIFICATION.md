# YT-102 — Doctor permission repair and cross-platform verification

Date: 2026-10-18 (Asia/Taipei).

## Authority and source

The owner requested repair of the previously reported doctor refusal, followed
by another verification run, and explicitly allowed WebCodex cross-platform
verification. This is a follow-up to the [CLI implementation](YT-102-CLI-IMPLEMENTATION.md),
not release, installation, deployment or Git delivery authority.

Local checkout: `$REPO`, branch `feat/yt090-configuration`, HEAD
`a883469862c5be3d88b7fe22262e10be8b53b92a` plus the preserved dirty tree.
The doctor implementation and its safety checks were not changed. The repair
corrects the existing installation's path permissions.

Evidence: `artifacts/doctor-verification-20261018/`. The original doctor exit1
and JSON refusal remain in doctor-before.*. Source manifests and native command
logs are retained separately from historical implementation evidence.

## Minimal local permission repair

The selected stable Rustup settings/toolchain paths were current-user owned,
with ordinary ACL entries and group-write permissions. The probe correctly
refused them. Only group-write was removed from these five exact paths:

| Path | Before | After |
|---|---|---|
| `$HOME/.rustup` | 0775 | 0755 |
| `$HOME/.rustup/settings.toml` | 0664 | 0644 |
| `$HOME/.rustup/toolchains` | 0775 | 0755 |
| `$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu` | 0775 | 0755 |
| `$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin` | 0775 | 0755 |

Ownership, inodes, modification times and settings contents were preserved.
No recursive permission change, home/Cargo-directory change, tool installation,
runtime source change or weakened executable trust rule was performed. Before/after
ACL/mode/content records are in permission-before.json and linux-permission-*.

## Linux results

| Command/check | Actual result |
|---|---|
| Actual CLI `doctor --json` and human `doctor` | Exit0; completed=true, ok=true, Rust/Cargo1.99.0; repeated after tests |
| Focused native probe tests | 5 passed |
| CLI commands tests | 3 passed |
| CLI `cargo test --offline --locked --all-targets` | 100 passed,0 failed,4 ignored |
| Settings fixture after the Windows portability correction | 3 passed,0 failed |
| WebCodex Linux native doctor | Exit0, schema1, ok=true after explicit process-local Rust PATH |
| Preservation and cleanup | Original tracked hashes/status unchanged; no CLI runtime/listener left |

The Linux Runner initially lacked cargo/rustc on PATH; its first report
correctly failed as unavailable. A process-local PATH selected the existing
protected installation for the successful rerun. No Runner service environment,
profile, host default or installation was changed.

## Windows source and verification

WebCodex Project: `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`.
Session: `wc_sess_ol95jcfM1xSqJN8i`. Existing checkout:
`C:\repos\yetlet`, branch `maint/m10-qualification-20261010`, HEAD
`3128d23591feee15ce976d7b83a56dcd587048d7`, with existing artifacts only.

The existing checkout was not updated. A new task-owned source copy initially
contained 141 exact files for the current CLI, required crate sources and manifests.
Each file was SHA-256 checked before verification. Initial transfer archive:
`119ab31420af8599db5fc80e4b4df4ab1151d4272c3f5b2935a3de61f8a1881e`.
The full source archive supplied the remaining files required by the existing
integration tests: 393 files total, archive SHA-256
`e7ccde4c870a725f288767c507b1e24e1e97f83515515e15bdd7b7598ce71613`.
Remote evidence root: `artifacts/doctor-20261018-119ab314/`.

The already installed Rust1.99.0 Windows GNU toolchain and previously provisioned
GNU binutils are selected only in the child environment. Host Rust1.88 default
and original PATH are preserved. Initial missing-dlltool and missing-assembler
failures are operational evidence, not doctor product failures; their original
logs are retained. A wrapper's CP950 output-encoding failure is likewise separate
from the native Cargo exit.

The first native test run exposed two settings test fixtures that opened raw
temporary directories with inherited writable ACLs. This was a harness failure:
the runtime correctly rejected the unprotected directories. The one test-only
correction in `tools/yetlet-cli/src/settings.rs` uses the existing native
`create_project` helper and writes settings through the resulting application
root. Runtime validation and trust rules are unchanged. The final 393-file
manifest includes this single overlay, SHA-256
`3367bb5f3d7349f9534735d1aee9439b15d6cca9c13c37b41698001906cdc81e`.
Original source, failed logs and the initial 60-pass/2-fail unit result remain
available; no historical failure was replaced.

Final native Job: `wc_job_nGjk2_6swE1YL2pW`, terminal exit0.

| Command/check | Actual result |
|---|---|
| `cargo test --offline --locked --lib settings::tests` | Exit0; 3 passed |
| `cargo test --offline --locked --all-targets` | Exit0; 84 passed,0 failed,4 ignored: 62 unit and 22 integration tests; nested fixture invocations are not counted twice |
| Native `yetlet.exe doctor --json` | Exit0; schema1, completed=true, ok=true, Rust/Cargo1.99.0 |
| Native human `yetlet.exe doctor` | Exit0 |
| Native doctor with empty PATH | Expected exit1; schema1, completed=true, ok=false; no unavailable tool counted as pass |
| Preservation and cleanup | All 393 source hashes match after execution; existing checkout HEAD/status unchanged; no task executable remains |

Native receipts are copied back to `windows-results/` in the local evidence root.
The transfer archive `windows-results.zip` has SHA-256
`30b2289118e94c940967f37c9356d35751c81c945f2cb926ad6baed08daa2592`.
`result-repaired.json` records exact argument vectors, native exits, elapsed times,
and before/after checkout state; `cleanup-final.json` records process cleanup.
The Windows GNU result is actual native execution, separate from the earlier
MSVC compile-only evidence.

## Review and remaining scope

Independent Reviewer follow-up: **PASS**, no issues or required follow-ups.
The review independently checked the five exact permission changes, the test-only
fixture correction, runtime source preservation and terminal Linux/Windows receipts.
Role, configured model and effective reasoning effort matched the native child
trace; bounded review evidence is in review-result.json and review-agent-trace.json.

Final repository checks: `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check`,
`git diff --check` and `cargo xtask check docs` passed (75 documents,103 backlog items).
Local HEAD and the original index tree are preserved; final-source-preservation.json
records that only the test fixture differs from the transferred runtime source set.

Doctor remains environment-only; a passing result does not certify application,
frontend, console, generator, production or release readiness. Windows frontend
pair-supervisor qualification is recorded in the subsequent
[native Windows Vite follow-up](YT-102-WINDOWS-VITE-VERIFICATION.md). YT-102's broader DX research is
still Todo and YT-020 is Blocked. No commit, push or deployment was performed.
