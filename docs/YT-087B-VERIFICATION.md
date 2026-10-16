# YT-087b — Frozen Build and Owned Foreground Dev Verification

**Date:** 2026-10-16. **Task:** YT-087b only. **Current result:** Linux implementation candidate with passing isolated Rust checks and direct process tests; the original Generalist Runner denied TCP binding and lacked a controlling foreground terminal, but a separate bounded host supplement later **passed actual HTTP and PTY Ctrl-C** (see final addendum). A subsequent distinct independent YT-087b Reviewer PASS accepted this bounded Linux preview after the focused P2 correction (see final addendum). YT-087 remains **In progress**; YT-087c is separate and YT-020 remains Blocked.

## Source, authority and role identity

Work used the original `$HOME/repos/yetlet` checkout, branch `master`, baseline HEAD `94cf8f555b554a26c2416842eab06110b229904d`, with the existing uncommitted M13 changes preserved. No worktree, clone, stash, clean, staging, commit, push, global installation, release, hosted trigger, browser launch or deployment occurred. Repository documentation is English.

This execution is the maintainer-supplied **WebCodex-launched `codex exec` Generalist role-directed Job**, applying `$HOME/.codex/agents-pipeline/agents/generalist.md`, `REASONING_POLICY.md` and `MATERIALITY_GATE.md`. Its child-side Codex thread/session ID is `01a1420a-4701-7712-a30c-55f6f7dd05a0`. The WebCodex Job ID and launcher model/effort telemetry are **not exposed in this child context** and require correlation by the launching observer; no ID is invented and no native managed `agent_type`, role match or effective-effort claim is made. No child agent was dispatched. This is distinct from both the older lost Job and the previous parent-session role emulation.

The original Generalist Job `wc_job__3DiWnfRkzAjXaJ-` remains **lost / outcome_unknown**. [YT-087a verification](YT-087A-VERIFICATION.md) retains its interrupted history, original Reviewer FAIL, fixture failures, resumed **15-test PASS**, and independent read-only Reviewer `wc_job_eWTKCMqqc5Ox5Bzn` **PASS**, zero issues/followups. That accepted Linux-only unpublished `new`/`doctor` scope is preserved. Its earlier external-app evidence in `target/m13/yt087a/local-vH7Tot/` includes explicit offline application lock generation, frozen compilation and HTTP 200 with CSP, escaped HTML and no script. Those historical results are not relabeled as a current 087b runtime PASS.

## Actual changes

| File | Change from the task's uncommitted baseline |
|---|---|
| `tools/yetlet-cli/src/lib.rs` | Add strict `build`/`dev` parsing, public help and direct parser tests; existing 087a tests retained |
| `tools/yetlet-cli/src/main.rs` | Dispatch only explicit build/dev to the application runner |
| `tools/yetlet-cli/src/app.rs` | New standalone app/lock preflight, literal frozen Cargo command, native final exit/signal propagation and direct tests |
| `tools/yetlet-cli/src/foreground.rs` | New Linux-only foreground/process-group ownership, signal forwarding, bounded cleanup and terminal handoff/restoration |
| `tools/yetlet-cli/src/foreground_tests.rs` | Native subprocess fixtures for exits, signals, owned descendants, orphan adoption and unaffected separate group |
| `tools/yetlet-cli/tests/commands.rs` | Direct CLI/native Cargo tests for build/dev exits and missing/stale lock/cache negatives |
| `tools/yetlet-cli/src/safe_dir.rs` | Only add the small `Anchor::working_directory` accessor: verify the existing chain and clone its held directory FD; no changes to creation/publication rules or accepted tests |
| `docs/README.md`, `docs/YT-087-CLI-PLAN.md` | Point to this candidate and its pending gates |
| `docs/YT-087B-VERIFICATION.md` | This exact result and handoff |

The CLI remains an unpublished isolated workspace. No dependency, Cargo manifest, root/xtask graph or lockfile change was needed. `doctor.rs`, `framework.rs`, `scaffold.rs`, generated templates, the 087a verification document, existing backlog/roadmap changes and prior receipts are preserved.

## Command and ownership contract

```text
yetlet build --manifest-path PATH [--release]
yetlet dev --manifest-path PATH
```

Both require explicit `Cargo.toml`, a standalone `[package]` plus `[workspace]` without inherited workspace membership, and a pre-existing regular application `Cargo.lock` in the same directory. They reject traversal, symlink path components/files, unsafe cwd ownership/permissions, wrong manifest names, invalid TOML and unsupported/duplicate/misplaced flags. The app cwd must belong to the current user and disallow group/world writes. Manifest/lock ownership is required; group/world-write bits are allowed only behind a private app directory that denies all group/other access. This admits Cargo's ordinary umask-derived lock permissions inside the accepted scaffold's 0700 directory without changing host settings.

The app cwd is pinned by a directory FD; an explicit command cwd and a pre-exec `fchdir` select that held identity. Commands use `Command::new("cargo")`, literal `build` or `run`, `--frozen`, `--manifest-path`, `Cargo.toml`, and only build's optional `--release`. `CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0` and `RUSTUP_SKIP_UPDATE_CHECK=1` prevent Cargo dependency-network fallback and rustup auto-install/update probes. There are no Python, Node or shell command wrappers in the product execution path, no CLI lock generation, watch loop, browser or implicit app launch from help/doctor/new. Ordinary Cargo compilation can execute the explicitly selected application's trusted build code and Cargo configuration.

Linux supervision owns one foreground Cargo child in its own inherited process group. A process-local child subreaper permits adopted descendants to be reaped. `waitid(WNOWAIT)` reserves the leader PID until the last group signal; no executable-name kills, unowned group signals or global child waits are used. SIGINT/SIGTERM to the CLI retain the first cancellation identity and are forwarded to that group. Cancellation receives a 500 ms grace interval, followed by TERM/500 ms and KILL; natural Cargo exit also triggers TERM/500 ms then KILL for remaining descendants. Final wait/reap is bounded to three seconds per cleanup attempt, with a bounded Drop fallback on errors. No further signals are sent after leader reaping. Cleanup or terminal-restoration errors return 125 and cannot become success.

With a valid controlling foreground terminal, its ownership is transferred to Cargo and restored after cleanup, with SIGTTOU blocked only during the handoff. A console without a valid foreground owner fails closed. Child numeric exits are preserved; native child signals and CLI cancellation are re-raised after verified cleanup where representable. Preflight errors use 1, usage errors 2, spawn/supervision/cleanup errors 125. Applications must retain the inherited process group; deliberate daemonization/session or group escape is unsupported. This is local preview supervision, not a containment sandbox.

## Fresh isolated validation

Toolchain: `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`, Linux. All listed final CLI commands returned native exit 0:

| Command | Result |
|---|---|
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | PASS |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | **23 unit tests + 1 native Cargo integration test passed**, zero failed/ignored; includes all prior 15 tests |
| `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | PASS |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | PASS |

Direct native results: process fixture exit **23** and signal **SIGUSR1** retained; cancellation retained native **SIGINT** after both parent and leaf observed the forwarded interrupt; a leaf deliberately ignoring INT/TERM was KILLed and reaped. A separate owned test sentinel in another group survived unchanged and was then explicitly killed/waited by its test owner. Normal leader exit **17** also cleaned and reaped its owned orphan. PID and group absence assertions passed. The subprocess fixture itself returns immediately in an ordinary test invocation; it is not an ignored test or a new testing framework.

The CLI/native Cargo fixture built successfully with a literal spaces/Unicode/metacharacter path; `dev` retained the app/Cargo exit **23**. Missing lock failed **1** for build and dev without creating a lock. Stale lock failed **101** for both with unchanged lock bytes. A separate explicit `cargo generate-lockfile --offline` fixture setup created the dependency-bearing app lock; an empty task-local `CARGO_HOME` then made both frozen commands fail **101** for unavailable offline cache, with unchanged lock bytes. No dependency was downloaded. Release-profile argv is directly asserted; an actual optimized application build was not repeated for this bounded slice.

Logs are under `target/m13/yt087b/`: `cli-test.log`, `cli-check.log`, `cli-clippy.log`; initial failures are retained separately. Docs/sample and final whitespace/preservation results are recorded below.

## One bounded generated-application attempt

The earlier 087a external app had already been removed, so this task used the accepted `new` implementation once to create `/tmp/yetlet-yt087b-RHdT4V/yt087bprobe`. No template was reimplemented. Its source/lock receipt is retained with this task's ignored evidence; no additional application scaffold or visible browser probe was added.

1. `yetlet new yt087bprobe --framework-path $HOME/repos/yetlet --dir /tmp/yetlet-yt087b-RHdT4V`: **exit 0**; no app lock existed afterward.
2. Separate explicit `cargo generate-lockfile --offline --manifest-path /tmp/yetlet-yt087b-RHdT4V/yt087bprobe/Cargo.toml`: **exit 0**, 54 packages resolved from the existing local cache. This successful bootstrap was not repeated.
3. `yetlet build --manifest-path .../Cargo.toml` with task-local `CARGO_TARGET_DIR`: **exit 0** after the permission compatibility repair described below; Cargo executed `build --frozen` and finished in 9.48 s. An earlier preflight exit 1 had launched no Cargo and is retained separately.
4. One PTY `yetlet dev` attempt returned **125** before Cargo output. The supplied console was `/dev/console`; a bounded inspection reported `PGID=1`, `TPGID=-1`, confirming that no valid controlling foreground group was available. This is a fail-closed terminal capability limit, not successful terminal/Ctrl-C evidence.
5. One operational retry used foreground inherited non-TTY stdio with `YETLET_PORT=0` and the same app/target. Cargo executed `run --frozen`, finished the build in 0.01 s and launched the actual generated binary. The app returned `PermissionDenied: Operation not permitted` when binding its loopback listener; the CLI retained native **exit 1**, rather than reporting cleanup failure or success.

The sandbox also denied netlink socket inspection. No TCP listener was successfully established, no HTTP request could be made, and no browser was launched. Actual generated-app HTTP GET and terminal-generated Ctrl-C remain **Not verified on this Runner**; the historical 087a HTTP result and the fresh native descendant/cancellation tests above remain distinct evidence. No online/network-permission workaround, environment reconfiguration or second runtime probe was attempted.

## Bounded repairs and limits

`repair_budget=3`, **repair_attempts_used=2**:

1. `product_failure`: initial CLI compile exit 101, `E0282` for the supervision closure result. A single `io::Result<()>` annotation closed the compile blocker; final tests/check/Clippy passed.
2. `product_failure`: the generated app's separate Cargo lock step produced 0664 under host umask 0002, while its accepted app directory was 0700. Initial overstrict file-mode preflight rejected it with exit 1. The smallest compatibility repair recognizes file protection by private cwd, retains ownership/no-symlink checks and adds a direct regression assertion. The same generated app then built successfully; no chmod workaround or host umask change was used.

**operational_retries_used=1** of limit 2: non-TTY dev after unusable PTY. The remaining runtime signature is `PermissionDenied: Operation not permitted` on generated-app TCP binding. No repeated infrastructure failure was retried. There was no harness repair, capability recovery or new dispatch.

Windows and other non-Linux execution are **Not verified**. Both build/dev fail closed there before any child is started; no native Windows job ownership, console control, filesystem/no-replace, compile or runtime PASS is claimed. Actual Linux terminal handoff/restoration, generated-app HTTP on this Runner, daemonizing applications and broader environment/release qualification are not accepted by these results.

## Final checks and next gate

- `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo xtask check docs`: **exit 0**, 75 catalog documents and 87 backlog items; docs/internal links/Markdown/whitespace/DAG PASS. The checker uses its existing required-document catalog plus recursive English checking; it is not a new exhaustive link checker for every untracked document.
- `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo xtask check samples`: **exit 0**, 12 sample entries; routes/dependencies/READMEs/evidence PASS. Receipts are `docs-check.log` and `samples-check.log`.
- `git diff --check` and focused `git diff --no-index --check -- /dev/null PATH` for this slice's new/edited untracked files: no whitespace diagnostics. No-index exit 0 or 1 with no diagnostics denotes an expected content difference; a broader untracked sweep instead reported exit **3** for the **pre-existing unchanged** `docs/YT-087A-VERIFICATION.md:74` blank line at EOF. That inherited file's bytes/hash were preserved, not repaired or hidden.
- Root, xtask and CLI Cargo.lock SHA-256 checks match `preserved-sha256.txt`. The same receipt confirms unchanged 087a scaffold/doctor/framework/templates/document and existing backlog/roadmap bytes. Removing only new accessor lines 102–107 from a copy of `safe_dir.rs` reproduces its exact starting SHA-256 `503f6c3838b9f2bbfc0d29a3796c1eab1d58e0cb1d6a38649be4010174ffef16`; all accepted tests and safety logic are unchanged. This reconstruction is retained as `safe_dir-before-accessor.rs` in the ignored evidence directory.
- Generated source matches the accepted template byte for byte. Source/manifest/lock receipts and hashes are retained at `target/m13/yt087b/generated-app/` and `generated-app-sha256.txt`; the exact owned external app and its temporary parent were removed and verified absent. All task command sessions completed. Fresh native process tests verified PID/group absence; the actual non-TTY dev completed with native exit 1 only after its supervisor's owned-group absence check. No listener was established. This is evidence within the Runner, not host-wide inspection outside its namespace.
- Final checkout is still original `master` at `94cf8f555b554a26c2416842eab06110b229904d`, with no staged changes. `baseline-status.txt` and `final-status.txt` preserve the uncommitted paths. This task added only the new 087b document and four Rust source/test files, and edited only the existing paths listed above. No task resource or server remains running.

The next unblockable action is a separate **read-only independent YT-087b implementation review** of this source and these bounded receipts, preserving the runtime/Windows limitations. If an accepted generated-app HTTP/terminal gate is required before that verdict, execute only the missing check on a capable authorized local Runner. YT-087c remains a separate later task; the unchanged generated README still uses its valid explicit Cargo bootstrap/run instructions and retains its original deferred-087b sentence for that later documentation slice. Do not mark YT-087 Done from this execution.
## Host runtime acceptance supplement (2026-10-16)

Following the role-directed Generalist Job `wc_job_7YGhZBnrLx3Ms6Lc` ending with **exit 0 but a correctly reported `partial` task result**, the parent ran **one bounded host-side** acceptance check for the exact missing Linux runtime/terminal capability. The Generalist's earlier PTY failure (125) and TCP `PermissionDenied` remain preserved historical sandbox results; the parent did not grant sandbox permissions or relabel those commands as successful.

Evidence: `target/m13/yt087b/host-runtime-wljaa6tw/result.json` and associated `new.log`, `explicit-offline-lock.log`, `frozen-cli-build.log`, `pty-dev.log`, `response.html`. One fresh owned external project `yt087bhost` was produced by the accepted `new` command and removed after completion. A one-off host Python standard-library PTY driver (validation harness only, **not** a dependency or child of the Yetlet CLI's active Cargo invocation graph) provided an actual controlling terminal to the native Rust CLI. It did not alter repository source, dependencies or locks. Observed native exit codes: `new=0`, explicit application `cargo generate-lockfile --offline=0`, `yetlet build --frozen=0`, actual terminal-generated Ctrl-C to `yetlet dev` terminated the CLI by **SIGINT 2** (harness return `-2`); this is not a synthetic test-only cancellation callback. Before Ctrl-C the generated application's `GET /` on `127.0.0.1:33305` returned HTTP **200** with CSP, escaped `Hello &lt;Yetlet&gt; &amp; Rust!` content and no script tags. After Ctrl-C, the listener refused connections; the exact CLI child was reaped and the owned external project removed. Host supplemental result: **PASS**.

This newly observed host result closes the two specific runner-unavailable runtime conditions without repeating unrelated 087a/browser/GPU tests. It does not confer Windows process/console support, an ability to manage intentionally daemonized descendants, production readiness or publication approval. The child Generalist Job was WebCodex-launched role-directed execution, not managed agent-type dispatch. YT-087b source and bounded Linux acceptance evidence are now ready for a **separate independent read-only implementation review**; YT-087 remains In progress and 087c stays pending. No commit/push/install/hosted trigger or additional source mutation occurred for this supplement.


## Focused P2 terminal-cancellation repair (2026-10-16)

**Result:** Corrected Linux implementation candidate, focused direct PTY and immediate foreground regressions PASS; **YT-087b remains pending independent re-review**. The distinct independent Reviewer Job `wc_job__31pU8wqklsFtFAz` returned **FAIL with one P2** against the previous candidate. The earlier Generalist Job `wc_job_7YGhZBnrLx3Ms6Lc` ended exit 0 with `task_status=partial` and two of three product repairs consumed. Neither that result nor the host supplement is overwritten by this correction.

The finding is material under the explicit maintainer handoff: after `tcsetpgrp` transfers the controlling terminal to Cargo's owned group, a terminal-generated SIGINT reaches only that group. An application retaining its group but ignoring SIGINT could leave the CLI's cancellation flag unset indefinitely. The prior tests signalled the CLI PID, and the successful host generated-app receipt exercised a responsive application; neither covered this resistant-application path. The smallest correction preserves the owned group and existing bounded escalation, adding an internal cancellation notifier in that same group.

### Changes and ownership

Only `tools/yetlet-cli/src/foreground.rs`, `tools/yetlet-cli/src/foreground_tests.rs` and this append-only verification addendum changed from this repair's starting snapshot. The original checkout remains `master` at `94cf8f555b554a26c2416842eab06110b229904d`. This is a **WebCodex-launched role-directed `codex exec` Generalist execution**, applying the canonical Generalist and Materiality Gate; it is not native managed dispatch. This continuation's launcher Job ID and model/effort telemetry are not exposed here, so no new ID or verified model/effort is asserted. No subagent was dispatched.

For a controlling terminal only, the CLI forks one internal notifier after spawning Cargo. The parent joins this unreaped direct child to Cargo's pinned PGID and waits at most three seconds for its private pipe readiness byte **before** transferring the terminal. The notifier uses only async-signal-safe operations after fork; its SIGINT handler writes a single byte to the private nonblocking pipe and exits. It never signals a captured parent PID. TERM is ignored by the notifier so ordinary TERM cleanup cannot masquerade as cancellation. The existing supervisor loop reads this pipe, never terminal input, and retains the first cancellation identity. Unexpected notifier exit, startup, join or pipe failure returns a supervision error after owned cleanup.

Cancellation still uses INT/500 ms, TERM/500 ms, then KILL and bounded reaping/terminal-group restoration. The notifier's exact unreaped direct-child PID is retained even on a failed PGID join, killed and reaped within the same cleanup deadline. Cargo's leader remains pinned through the last group signal; no signals target an unowned group, and there are no group signals after leader reaping. Non-TTY supervision is unchanged. No external executable/helper, dependency, manifest/lock change, watcher, HMR, raw-terminal polling or OS configuration change was introduced. See the Linux [async-signal-safe operation contract](https://man7.org/linux/man-pages/man7/signal-safety.7.html), [process-group rules](https://man7.org/linux/man-pages/man2/setpgid.2.html) and [terminal foreground ownership](https://man7.org/linux/man-pages/man3/tcsetpgrp.3.html).

### Direct regression and bounded validation

The one new Rust regression, `foreground::tests::terminal_ctrl_c_cleans_resistant_group_and_restores_tty`, allocates its own Linux PTY and uses a fixture session leader with `setsid`/`TIOCSCTTY`. The application fixture and its descendant retain the inherited PGID and install actual `SIG_IGN` dispositions for both INT and TERM. Readiness requires the application to observe that its group owns its controlling terminal. The test verifies that the application and notifier share that owned group while the supervisor remains separate, then writes the configured VINTR byte through the PTY master. No test code signals the CLI PID on this successful path.

Assertions prove native CLI termination by **SIGINT**, completion within five seconds (the entire focused test finished in **1.03 s**), terminal foreground-group restoration before final CLI signal propagation, no unreaped supervisor children, absent application/descendant/notifier PIDs and PGID, and survival of a separate-group sentinel without an interrupt observation. The sentinel is explicitly killed/waited by its owner and the temporary files/PTY handles are dropped. These are direct product fixtures in the existing test module, not a test orchestration framework. This Runner allowed the owned PTY; no host permission or terminal configuration workaround was needed.

Evidence directory: `target/m13/yt087b/p2-tty-repair/`. All final commands below returned native exit **0**:

| Command | Result / log |
|---|---|
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib foreground::tests::terminal_ctrl_c_cleans_resistant_group_and_restores_tty -- --exact --nocapture` | **1 passed**, zero failed/ignored; `targeted-pty-rerun.log` |
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib foreground::tests:: -- --nocapture` | **5 passed**, zero failed/ignored, 19 filtered; native exits/signals, CLI-directed interrupt, orphan cleanup and new terminal regression; `foreground-regression.log` |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | PASS; `fmt.log` |
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | PASS; `check.log` |
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | PASS; `clippy.log` |
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo xtask check docs` | PASS; 75 catalog documents, 87 backlog items; `docs-check.log` |

The first focused invocation returned **101**, preserved as `targeted-pty.log`: the fixture read `/proc/PID/task/PID/children`, but Rust's test worker thread owned the children, so the assertion saw zero instead of two. Classified **harness_failure**, with one smallest in-place correction: record/use the supervising thread's TID for both pre-interrupt and post-cleanup observations. The same focused check then passed. No product edit followed that test failure. An initial patch application for this fixture correction failed to match formatted context and changed no file; one corrected patch application succeeded. This was an **operational_failure**, not a product repair or capability recovery.

### Counters, preservation and next gate

- **Product repair attempts:** this continuation **1/1** remaining; cumulative **3/3**, **0 remaining**. The single repair addresses only the reviewer's terminal-cancellation P2. Prior compile/type and private-directory lock-permission repairs remain recorded above.
- **Operational retries:** this continuation **1/2** (patch-context correction); inherited runtime retry **1**, cumulative **2**. **Harness corrections:** **1**; no reasoning/model recovery, redispatch or additional product repair. Last encountered failure signature: `operational_failure: apply_patch formatted-context mismatch`, resolved; preceding harness signature: `children=0 expected=2 on main TID`, resolved. No unresolved local check failure remains.
- `baseline-sha256.json`, `baseline-status.txt`, retained before-file copies, final `preservation.json`, `final-status.txt` and `result.json` record source and checkout preservation. All other starting source files, accepted 087a bytes, root/xtask/CLI locks and prior 087b source/host receipts are retained. The pre-repair verification document is an exact byte prefix of this final file. Whitespace checks cover the two changed untracked Rust files and this document individually; expected no-index difference exit 1 is accepted only with empty diagnostics. The inherited 087a EOF whitespace finding is unchanged.
- The previously successful generated-app HTTP/PTY E2E at `target/m13/yt087b/host-runtime-wljaa6tw/result.json` was **not rerun**. Its PASS remains scoped to the responsive generated app. No new application, server, listener, browser or external environment was started. All test command sessions completed, with owned cleanup and reaping assertions passing. No stage/commit/push, worktree, global install, publication or host permission change occurred.

The next unblockable gate is a distinct **read-only independent YT-087b re-review** of this exact correction and its focused receipts. No independent PASS is claimed here. YT-087 remains In progress; 087c remains separate. Windows/non-Linux ownership, deliberately escaped process groups/sessions, and public/production qualification remain Not verified or unsupported as previously documented.
## Independent YT-087b acceptance (2026-10-16)

Distinct read-only Reviewer Job `wc_job_Ch4EM14EGXEcO8eU` completed with `overall_status=pass`, `issues=[]` and `required_followups=[]`, validating the single P2 resistant-application terminal SIGINT repair, actual direct PTY regression and source-bound cleanup/ownership semantics. It confirmed the notifier is joined to Cargo's pinned owned group before terminal handoff, that the parent detects private-pipe SIGINT without relying on the application's response, and that bounded group cleanup, native exit/signal and terminal restoration are preserved. The five targeted process regressions and prior host generated-app HTTP/CSP/escaping and ordinary terminal Ctrl-C evidence remain separate; no redundant E2E was performed by the Reviewer. The original first Reviewer FAIL and sandbox-denied role-runtime evidence stay historical.

**YT-087b is Done at bounded Linux unpublished CLI preview scope.** This reviewer PASS does **not** accept Windows/non-Linux process ownership, intentionally detached groups, hosted/production environments, published installation or release. YT-087 as a whole remains In progress pending 087c final docs/consistency/independent review. Neither the reviewer nor this bookkeeping update stages, commits, pushes or deploys the source.
