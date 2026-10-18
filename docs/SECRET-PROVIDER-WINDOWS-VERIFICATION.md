# SecretProvider native Windows verification

Date: 2026-10-18. Task: `secret_rotation_windows`.
Related scope: the separately authorized YT-090 SecretProvider and explicit
settings-rotation follow-up described in [the provider guide](SECRET-PROVIDERS.md)
and [rotation verification](SECRET-ROTATION-VERIFICATION.md).

Status: native Windows checks passed. This record covers the opt-in string
provider, its settings replacement, ordinary service/State composition and
existing Loader integration. Independent review and actual Azure operations
are recorded separately by the coordinator. This run made no product changes,
commit, push or deployment.

## Source and environment

The source checkout was the repository root, based on HEAD
`b627d291696fe1aaa4a3db5040da07e2ad44da04`, with the new crate uncommitted.
The supplied `windows-source.zip` SHA-256 was
`bd15490c6c2fe205ef6d041ed6f7e34d4d19588991a4e2bd72b9b03ca9930140`.
All 208 imported files matched the supplied per-file inventory before execution
and still matched after every native command, including both Cargo manifests
and lockfiles. The coordinator subsequently changed only the crate README's
qualification links; that documentation-only change is outside this tested
snapshot and does not change its runtime, tests, manifests or lockfiles.

The native WebCodex project was
`agent:ncut-bohe-244143825ae3488e:localtasks`, Session
`wc_sess_v49VHB4rF9p5ckzy`. The new owner-protected task root was
`C:\repos\localtasks\windows-secret-rotation-20261018`, with imported source
under `source` and build output under `target`.

- Windows 11 Professional, native version `10.0.26300.0`, x86-64 MSVC.
- PowerShell `7.6.6`; Cargo `1.99.0` and Rust `1.99.0`.
- Clippy `0.1.99`; rustfmt `1.10.0-stable`.
- Visual Studio 2022 Professional `vcvars64.bat` initialized only the command's
  process environment.
- Cargo was the existing protected executable at
  `C:\repos\localtasks\windows-cli-125-20261018\tools\cargo.exe`, SHA-256
  `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
- `CARGO_HOME` reused that previous task's `cargo-home`; `CARGO_TARGET_DIR`,
  `TEMP` and `TMP` pointed into the new task root. `RUSTC` and `RUSTDOC` selected
  the physical `1.99.0-x86_64-pc-windows-msvc` toolchain binaries.

## Native commands and results

Commands ran from the imported `source` directory with the protected Cargo
executable. Each command's exact executable, argv, working directory, UTC start
and finish times, and native exit code are retained in `native-results.json`.
The verification Job `wc_job_dg3H7A3qqxAhZevz` reached `completed`, exit 0.

```text
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path crates/yetlet-secrets/Cargo.toml --check
cargo run --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --example state
```

| Check | Native result |
|---|---|
| All targets | Exit 0; 13 passed, 0 failed: 1 library, 4 provider, 5 state/Loader example and 3 Azure parser tests |
| Rustdoc | Exit 0; 1 passed, 0 failed |
| Strict Clippy | Exit 0; no warnings |
| Formatting | Exit 0 |
| Configured state example | Exit 0; stdout exactly `Example request status: 200 OK` plus newline |

The example used a process-local synthetic Production token. Its plaintext was
absent from exported logs. The request ran through `Router::oneshot`; no socket
listener was necessary or started.

Native tests exercise complete snapshot replacement through an existing shared
provider, removed/empty/missing names, owned values surviving replacement,
redacted diagnostics and poisoned-lock failures. The Loader example explicitly
rewrites its temporary Development file, reloads through the existing Loader,
and replaces selected values while retaining the original service handle.
Fresh lookups see the reloaded value; the previously returned owned value keeps
its old value. Existing loader precedence, ignored Development-secret selection
in Staging/Production, custom providers, State injection, escaped SSR and CSP
also passed. The three Azure parser tests require no Azure CLI or credentials;
no actual Azure request ran on Windows.

## Retained failures and bounded corrections

The initial offline all-targets command exited 101 before compilation because
the retained cache lacked `signal-hook-registry` index metadata. This was a
dependency-cache operational failure and disproved no product behavior. The
coordinator authorized the exact locked dependency fetch:

```text
cargo fetch --locked --target x86_64-pc-windows-msvc --manifest-path crates/yetlet-secrets/Cargo.toml
```

Fetch exited 0. The same source then passed all requested commands offline and
locked. Initial command/results and cache-miss logs remain alongside the final
logs; no source or lockfile was repaired to resolve the cache.

A preliminary PowerShell `-Command` request was rejected by `run_process`
before starting; typed `run_script` was used instead. The optional
`cargo-fmt.exe -V` probe returned 2 because that short version flag is
unsupported; `rustfmt -V` and the actual formatting check succeeded.

The artifact exporter produced a native ZIP reference, but this host's
`resources/read` returned `Unknown resource`. Public text files were therefore
copied through bounded text transport. One oversized text batch and the
single-item JSON object/array envelope were corrected without rerunning product
checks. All 25 copied files match their native SHA-256 hashes byte for byte.
The original native ZIP remains at the task root, SHA-256
`dd3b67897a0974ec31e1432be20a0d4b6ae75a30f3e57afbf14ed2263b1503b7`.

## Evidence and cleanup

Public logs are retained under
`artifacts/secret-rotation-20261018/windows-native-logs/`. Primary receipts are
`native-results.json`, `all-targets.stdout.log`, `doc.stdout.log`,
`state.stdout.log`, `source-verification.json`, `source-hashes-final.json`,
`previous-source-preservation.json`, `environment.json` and `cleanup.json`.
`windows-exported-log-hashes.json` lists the exact native hash and byte size of
every copied file.

The final check found 208 matching source files, zero remaining owned secret
fixtures, zero active task executables and zero synthetic payload matches in
public logs. Tests removed their private fixtures, and the empty task temporary
directory was removed. The source, build output and public evidence remain in
the protected task directory for review. No server, browser, watcher or
background process was started. All execution commands reached terminal state.

The previous CLI source tree retained all 196 files with inventory SHA-256
`dbb806809aac65ddea1fec0b722c7dd00e6198091f72ea634253b80d29c3633f`.
The previous HTTPS source tree retained all 197 files with inventory SHA-256
`b71ae726be5e261f2b92bc304ca28e7003402855ccf0884aed95de37def3581c`.
Those inventory digests hash UTF-8 sorted relative paths, one space, each
lowercase file SHA-256, and LF-terminated lines. Existing Cargo configuration
and credential-file presence/hashes were unchanged. Only the new task root's
ACL was restricted; no global tool installation, trust-store or ACL change
occurred.

This Windows slice changed this record and its `windows*` evidence only. It
leaves cloud service behavior, credential retirement at an external system,
retained-client reconfiguration, browser qualification and production readiness
to their separately authorized work. It advances no backlog status. The next
unblocked coordinator action is final independent review and closeout; YT-094
remains the next separate core discussion.
