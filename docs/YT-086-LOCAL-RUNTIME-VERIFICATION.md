# YT-086 — Local Loopback Runtime Verification

**Date:** 2026-10-15 (Asia/Taipei). **Result:** PASS for the requested local
default xtask verification scope. This supplements
[YT-086 verification](YT-086-VERIFICATION.md); it does not replace historical
failures or expand environment qualification.

## Environment and preservation

- Checkout: `$HOME/repos/yetlet`, branch `master`, HEAD
  `7cfac613f908c5c19fa581438288386809870b56`, before and after verification.
- Ubuntu 26.04.1 LTS, Linux `7.0.0-38-generic`, x86_64; repository-selected
  `rustc 1.99.0 (b940084d7 2026-09-28)` and
  `cargo 1.99.0 (5f94df478 2026-08-27)`. The toolchain, rustfmt and Clippy were
  already installed. PATH prepended the existing `$HOME/.cargo/bin`.
- Commands used `RUSTUP_AUTO_INSTALL=0`, `CARGO_NET_OFFLINE=true` and
  `GIT_OPTIONAL_LOCKS=0`. Test-only `TMPDIR` selected the new evidence directory's
  `tmp/` so temporary-resource removal could be checked directly.
- App chat inventory and local process inspection found no competing checkout
  execution. Before/after source hashes provide the preservation check.
- Evidence directory:
  `target/m12/yt086/local-loopback-20261015T133023Z-860422f0/`.
  It contains the environment, before/after Git status, command files, complete
  merged stdout/stderr logs, native exit codes, test counts and cleanup records.
- All 567 pre-existing tracked/untracked source files and 8,089 pre-existing M12
  evidence files retain their hashes. All 52 original Python oracles match HEAD.
  Root and isolated manifests/lockfiles, toolchain selection and the staged index
  are unchanged. The sole new source-tree file is this document.
- YT-085's accepted Reviewer receipt remains SHA-256
  `251c3230e936c89294e15c3f15297eb5cb12363490cd2cd90d3463816df84fce`.
  Its acceptance and YT-086's final independent Reviewer PASS
  `wc_job_6UfDDdHnNCBXkMhA` remain unchanged. No implementation correction was
  necessary, so this verification did not require a fresh independent review.

## Loopback and focused lifecycle tests

The active local execution context successfully bound and listened on
`127.0.0.1:53243`; after closure, connection returned `ECONNREFUSED` (111).
No sandbox bypass, privilege escalation or host configuration change was used.
The previously recorded WebCodex `PermissionDenied` is consistent with that
restricted environment, and did not reproduce as a local code defect.

`cargo test --offline --locked --manifest-path xtask/Cargo.toml --test server_status -- --nocapture`
exited **0: 5 passed, 0 failed, 0 ignored**, including the existing subprocess
fixture and all four previously failing scenarios:

| Scenario | Observed assertion contract |
|---|---|
| `early_exit_keeps_native_code_and_receipt` | Native exit 23 and readiness error retained; readable exit receipt. |
| `failed_graceful_exit_writes_receipt_before_returning_error` | Native exit 17 retained; receipt exists before the failure return. |
| `early_zero_is_not_a_successful_readiness` | Native exit 0 retained; readiness still fails with code 1. |
| `signalled_exit_retains_signal_and_portable_status` | SIGTERM 15 retained, native code null, portable error 143. |

Each scenario also asserts `listener_closed=true`, `stream_closed=true`,
`owned_tree_reaped=true`, null `cleanup_error`, and the closed Service stream.
The full suite additionally exercises receipt/cleanup failures returning 125,
owned descendants, readiness-error preservation and escalation after the grace
deadline. Assertions were not changed or weakened.

## Complete default xtask suite

`cargo test --offline --locked --manifest-path xtask/Cargo.toml` exited **0**.
Complete output is in `full-suite.log`; `test-counts.json` records every result.

| Test binary / target | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Library unit tests | 30 | 0 | 1 |
| Binary unit tests | 3 | 0 | 0 |
| `builders` | 7 | 0 | 0 |
| `check_docs` | 2 | 0 | 0 |
| `check_samples` | 8 | 0 | 0 |
| `foundation` | 9 | 0 | 1 |
| `local_operations` | 11 | 0 | 0 |
| `measurements` | 9 | 0 | 0 |
| `qualify_multi_instance` | 23 | 0 | 0 |
| `release_checksums` | 17 | 0 | 0 |
| `server_status` | 5 | 0 | 0 |
| `smoke_browser_harness` | 4 | 0 | 0 |
| `smoke_modular_shop` | 3 | 0 | 0 |
| `smoke_protocol` | 10 | 0 | 0 |
| Doctests | 0 | 0 | 0 |
| **Total** | **141** | **0** | **2** |

Both ignored entries are the existing test-only subprocess entry points,
explicitly invoked by their owning tests. Later integration binaries and the
doctest target were reached. The separate focused run is not added to this total.

## Regression checks and cleanup

| Exact command | Native exit |
|---|---:|
| `cargo check --offline --locked --manifest-path xtask/Cargo.toml --all-targets` | 0 |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | 0 |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo xtask check docs` | 0 |
| `cargo xtask check samples` | 0 |
| `git diff --check` | 0 |

Focused, full-suite and final inspections found no remaining test process, new
listener or entry in the task's temporary directory. Existing system listener
addresses were unchanged. Tests directly verified stream closure and readable
receipts before their temporary directories were removed. No cleanup error was
accepted as success. Preservation results and final status are in `result.json`
and `preservation.json`; original logs and failures remain in their old locations.

No production source, test, assertion, dependency or Git history changed. No
installation, worktree, commit, push, deployment or hosted trigger occurred.
This is local Linux default-suite evidence only; optional GPU/browser journeys,
Docker/Redis integration, Windows persistent-Service/cancellation, long-load
qualification and hosted private rebuild remain at their previously documented
limits. There is no new architecture decision. No further backlog execution is
admitted by this follow-up: YT-087 remains Deferred and YT-020 remains Blocked.
