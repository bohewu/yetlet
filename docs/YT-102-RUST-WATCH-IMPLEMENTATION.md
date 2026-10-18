# YT-102 — Settings-controlled Rust watch implementation

Date: 2026-10-18 (Asia/Taipei). Status: implementation and verification complete;
independent review PASS. Baseline: `78f3659f2b21cb80a41f563de0e47588ce3ee34a`.

The owner explicitly authorized integrating Rust watch/rebuild/restart into
`yetlet run`, using existing settings, prioritizing Watchexec and retaining Vite
across Rust restarts. This implements that bounded slice; the broader YT-102 DX
audit remains Todo. YT-103 remains independently unblocked. At verification
closeout, no commit or push had been performed; subsequent local commit
authorization is recorded in repository history.

## Configuration and behavior

```toml
[cli.rust]
watch = true
```

New scaffolds declare `watch = false`. Existing settings without the Rust section
continue to default to false; a provided section remains strict about keys and
types. Normal base/profile precedence and `APP_CLI__RUST__WATCH` apply through
the existing loader. No new CLI flag, Cargo metadata namespace or configuration
parser was introduced. Plain SSR retains its runtime-settings module opt-in.
Generated runtime settings accept the same compatible CLI namespace.

When enabled, separately prepared Watchexec must be available on PATH. Version
2.8.0 was exercised; the CLI never installs it. Watchexec runs only as a JSON
file-event source. The CLI owns the initial frozen/offline Cargo process and
each replacement, with separate persistent watcher and optional Vite ownership.
It stops/reaps the old Rust generation before starting another. Compile/runtime
exits are reported and wait for another edit; Vite continues running. User
cancellation, Vite exit or watcher failure ends and cleans the session.

`yetlet build` ignores watch execution and does not require Watchexec. CLI policy
is read once per invocation. A new Rust process reads backend settings afresh,
but changing CLI watch/Vite policy requires restarting the CLI itself.

Watch covers app-root `*.rs`, Cargo.toml and Cargo.lock, including build.rs. It
excludes the configured frontend root, target/node_modules/dist and an in-root
CARGO_TARGET_DIR. It does not follow symlinks. Outside-app local dependencies are
not watched; other Cargo-config output directories must be kept outside the
watch root or selected through CARGO_TARGET_DIR. The event pipe has bounded line
size and one pending change; malformed output/EOF fails the session.

## Implementation surfaces

- `src/settings.rs`, `src/scaffold.rs`, `templates/config/cli.rs` and
  `tests/selected_scaffold.rs` in `tools/yetlet-cli/`: strict compatible setting,
  explicit generated default and runtime/scaffold coverage.
- `src/app.rs`, `src/platform/process/mod.rs`, new `watch.rs`, `linux.rs`,
  `foreground.rs`, `windows.rs` and `unsupported.rs`: run-only selection,
  Watchexec event command/parser, and native supervision. Existing Linux
  process-group/subreaper/terminal ownership and Windows suspended-spawn,
  executable identity/ACL, handle allowlist and private Jobs remain in use.
  Source-change restarts do not set session cancellation or signal Vite.
- Existing `foreground_tests.rs` and `tests/windows_build.rs`: focused restart,
  failure recovery, Vite identity, cancellation and cleanup assertions.
- The CLI contract, CLI guide, selected generated README, this record and
  documentation index describe the new behavior and limits. Historical research
  and external-wrapper trial records remain unchanged.

The watcher receives no backend `APP_*` settings or secret-selector environment;
Rust retains its backend context and Vite keeps its established environment
filter. No new handler, router, dylib boundary or framework runtime is required.

## Verification

Evidence root: `artifacts/rust-watch-integration-20261018/`.

| Check | Result |
| --- | --- |
| Complete CLI test suite, offline/locked | 105 passed, 0 failed; four existing opt-in tests ignored by default |
| Explicit selected generated-runtime compilation test | PASS, including compatible strict Rust settings |
| CLI Clippy all targets, warnings denied | PASS |
| Windows MSVC cross-target tests compilation/Clippy | PASS; compilation evidence, not native MSVC execution |
| Real Linux generated Vite app build | Explicit offline dependencies, frontend production build and frozen Rust build exit 0 |
| Watch enabled during Rust-only build, without Watchexec on PATH | Build exit 0; frontend skipped through its independent setting |
| Watch run without Watchexec | Exit 125 before frontend/Cargo startup; no installation |

Linux used the rebuilt CLI, actual Watchexec 2.8.0, Vite 8.3.4 and a newly
generated settings/Vite app. Vite PID **1191656** and watcher PID **1191615** stayed
unchanged across these observations:

| Stage | Rust PID / HTTP result |
| --- | --- |
| Initial run | 1191711; backend and `/@vite/client` HTTP 200 |
| Handler edit | 1192590; new SSR text HTTP 200, old Rust PID absent |
| Intentional compilation error | Rust absent and backend unavailable; compiler/CLI reported 101; Vite still HTTP 200 |
| Source correction | 1192973; recovered SSR text HTTP 200 with the same Vite/watcher |

Actual controlling-PTY Ctrl-C produced `CLI_CANCELLED`; all recorded owned PIDs
were absent and ports 43129/5173 closed. The execution adapter reported exit 1;
that value is retained in `linux-pty-stop.json` without inventing a native wait
status. The focused subprocess regression separately verifies watch cancellation
via SIGINT and watcher exit 31. Existing terminal ownership/restoration tests
also pass. The fixture's original Rust source and lockfile were restored.

The three restart/error observations, command preparation, actual compiler output
and cleanup are in `linux-result.json`, `linux-preparation.json`, `linux-pty.log`
and `linux-pty-stop.json`. The former external-wrapper trial's retained status
101 is historical: Watchexec now supplies events and does not own Cargo exits.

## Native Windows integration

WebCodex project `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`, Session
`wc_sess_lbIXmQXQjKcnV0oo`. Job `wc_job_J9-woNPEE2L8yFJd` completed exit 0:

```text
cargo test --offline --locked --manifest-path SOURCE/tools/yetlet-cli/Cargo.toml --test windows_build native_cli_rust_watch -- --ignored --exact --nocapture --test-threads=1
```

The selected native Windows GNU integration test ran all three existing-console
fixture cases, using real Watchexec/Vite, real frozen Cargo generations, a genuine
compile error and resistant Rust descendants. Every case asserts persistent Vite
through restart/error recovery, held Rust generation termination, lock retention,
frontend secret filtering and unrelated sentinel isolation before cleanup.

| Ending | Native result |
| --- | --- |
| Console Ctrl-C | `0xc000013a` / signed -1073741510; all owned trees gone, port released |
| Targeted Ctrl-Break | Same native cancellation status and cleanup |
| Vite exits 23 | CLI retains 23 and cleans watcher/Rust trees |

These cases generate OS console control events. They do not repeat the previous
[actual Windows keyboard-input trial](YT-102-WINDOWS-RUST-WATCH-VERIFICATION.md),
which qualified the separate external-wrapper workflow. Linux actual PTY input
above exercises this integrated mode; no fresh Windows keyboard-injection claim
is made here.

All **395** transferred source/workspace file hashes matched the returned tested
manifest at the original integration closeout. Its only subsequent runtime change
was the target-directory normalization correction described below; the later
`watch` command addition is recorded separately at the end of this document.
The native lifecycle adapters are unchanged. Returned ZIP SHA-256:
`857ade83702bf195117d5acdb1009bae498962ccebaf220d1c4533e18b412e47`.
`windows-results/` retains commands, case receipts, native output, tested manifest
and preservation. Original Windows checkout HEAD `3128d23591feee15ce976d7b83a56dcd587048d7`
and its artifacts-only status were preserved; no task process remains. Host
toolchain/PATH defaults were unchanged and no global installation occurred.

## Limits and preservation

This provides stop/rebuild/restart, including compilation downtime and recreated
Rust memory. It does not provide in-process replacement, automatic browser
refresh, preserved tasks/connections or graceful request draining. Frontend HMR
semantics and broader platform/toolchain matrices are outside this slice.

Unrelated local BACKLOG/ROADMAP edits and all earlier research/trial evidence were
preserved. Public release/production qualification remains open. Choosing any
additional watch settings or in-process hot replacement remains separate work.

Older settings-enabled generated apps may own a strict `Cli` Rust type without
the new field. Before enabling `[cli.rust]` in such an app, manually update that
app-owned type to accept the declaration, following the current generated
`Cli`/`Rust` types. The CLI does not rewrite existing app sources; leaving the
section absent preserves their previous behavior.

## Review correction and focused verification

The first independent review found one P2: a relative CARGO_TARGET_DIR such as
`./build-output` produced `/./build-output/**`, which Watchexec interpreted as
a literal dot prefix. Generated Rust output could then trigger another restart.
The correction lexically normalizes the target relative to the protected app
before constructing the ignore. It does not require creating the directory or
change native process supervision.

Three focused event/target-normalization tests pass, including dot/parent
components returning to an in-app output directory. A direct real Watchexec
2.8.0 regression changed both `build-output/generated.rs` and application
`main.rs`: only the application source event was emitted. The watcher terminated
and was reaped normally. Evidence is `target-exclusion-result.json` and
`target-exclusion-probe/`. `post-review-source-delta.json` identifies the single
changed runtime file against the earlier 395-file native snapshot. The original
failed review is retained; lifecycle tests were not repeated for this isolated
filter correction.

Independent reviewer `/root/integrated_rust_watch_review` returned **PASS** on
follow-up, with zero remaining issues or required followups. It verified the P2
correction, focused and real-tool regressions, final hashes and the unchanged
native lifecycle adapters. Its matching role/model/effort trace, initial finding
and final decision are retained as `review-initial.json`, `review-final.json` and
`review-final-trace.json`. At that closeout, only review-status bookkeeping
changed afterward. The command addition below has its own validation and review.

Final focused tests, Clippy, formatting, Windows cross-target compilation,
documentation and whitespace checks pass. Detailed outcomes are retained in
`validation.json`. WebCodex verified the three native output artifacts with no
active Jobs or unresolved failures; its automatic source-validation status is
`unproven`. Source correspondence and the subsequent one-file repair are therefore
reported explicitly above rather than claimed as automatic certification.

## Additive watch command

The owner subsequently authorized `yetlet watch [--manifest-path PATH]` as
`run` plus an invocation-only Rust watch override. `run` continues to honor
`cli.rust.watch`; `watch` validates the same app/settings and then enables watch
even when that setting is false or absent. Malformed settings still fail before
effects. Vite selection remains `enabled && auto_start`; Rust restarts retain
the same frontend process. Settings files are not rewritten and build is unchanged.

The addition changes `tools/yetlet-cli/src/lib.rs`, `src/main.rs`, `src/app.rs`,
`tests/commands.rs` and `tests/frontend_process.rs` (paths after the first are
relative to `tools/yetlet-cli/`). It is confined to CLI parsing/help, executable
dispatch and shared app selection, with focused command tests. It changes no native process adapter,
event parser, dependency, scaffold settings schema or restart lifecycle. The CLI
guide, contract and generated README describe both execution choices. Earlier
native Windows evidence above qualifies the shared supervisor; it does not
claim a fresh native Windows execution of the added command.

| Command-addition verification | Result |
| --- | --- |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | 107 passed, 0 failed, four existing opt-in tests ignored |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --target x86_64-pc-windows-msvc --tests` | PASS; compile-only |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check` | PASS |
| `cargo xtask check docs` and `git diff --check` | PASS |

Focused command coverage demonstrates the absent/false watch setting, a false
environment override, unchanged non-watch `run`, disabled Vite auto-start,
unchanged settings bytes, rejection of malformed settings/environment before
startup, and owned backend/descendant/watcher cleanup after SIGINT. Help works
with no project or tools; watch accepts the existing literal/native manifest
forms and rejects duplicate or misplaced arguments. An initial fixture-only
correction replaced an EOF-status assumption with explicit cancellation.

Independent review found no product blocker but identified a P2 in the new
fixture: Cargo readiness did not prove that the watcher had finished publishing
its PID before cancellation. A bounded wait now requires a valid positive watcher
PID before sending SIGINT and retains that PID for cleanup checks. Only this
focused test was rerun after the fixture correction: one passed, zero failed;
formatting and whitespace checks pass. The full-suite, Clippy and MSVC results
above preceded this test-only correction; runtime sources did not change.

Prior planning, settings/scaffold and native supervisor sources remain
byte-identical to their state before this additive command. No commit or push
was performed during verification.

Independent reviewer `/root/watch_command_review` returned **PASS** on follow-up,
with zero remaining issues or required followups. Native role/model/effort
evidence matched the dispatched reviewer. The broader YT-102 audit remains Todo;
YT-103 remains independently unblocked.
