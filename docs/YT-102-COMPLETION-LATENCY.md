# YT-102 — Bounded Linux natural-completion latency

Date: 2026-10-20. Base HEAD: `649be69` plus the already reviewed, uncommitted
YT-095/YT-102 research documents. After the [DX audit](YT-102-DX-AUDIT.md), the
owner selected the next bounded implementation: avoid the fixed natural-exit
wait without weakening process ownership or cleanup. Implementation and focused
verification pass. Independent Reviewer `/root/yt102_completion_latency_review_oct20`
PASS has zero issues or required followups. This bounded slice is complete;
no commit is included.

## Change and safety boundary

The Linux single-command foreground path now skips its final 500 ms grace only
when all of the following hold:

- `waitid(WNOWAIT)` has observed leader exit, without reaping that leader.
- The supervisor has exactly one thread.
- Its own bounded procfs child list contains exactly the retained leader and,
  when present, its retained terminal notifier. No other child is permitted.
- No cancellation is pending at the final cancellation check.

The helper reads at most 64 bytes. Additional children, multiple threads, an
unreadable/missing procfs interface, truncated data, parse failure or a pending
signal retains the existing grace path. It creates no process, global PID scan,
new thread, provider, settings key or command option.

The final group TERM/KILL, child/descendant reap, native completion status and
terminal restoration still run. The unreaped leader pins its identity through
the last group signal; this change never reaps it early to infer group absence.
Cancellation retains its existing waits. Pair supervision and Rust-watch/Vite
generation handling are unchanged; this is not a global removal of GRACE.

### Why this is not an arbitrary process-tree snapshot

The [proc child-list manual](https://man7.org/linux/man-pages/man5/proc_tid_children.5.html)
warns against enumerating an actively changing tree. This helper is deliberately
narrower: the single-threaded supervisor does not reap during the observation,
and its only accepted children are identities it is retaining unreaped. Exiting
children therefore remain present rather than being concurrently removed.
Any other direct child causes fallback, including a zombie or a child outside
the selected process group.

The existing subreaper adopts orphan branches. Linux performs child reparenting
before reporting the leader as a waitable zombie; the subsequent WNOWAIT check
therefore precedes this child-list test. If any descendant branch survives the
leader, an additional adopted root remains a child of this supervisor. A passed
check leaves only the known dead leader and the notifier, which never creates
children. The helper must not be reused for multithreaded or concurrently reaped
trees. See the [subreaper contract](https://man7.org/linux/man-pages/man2/PR_SET_CHILD_SUBREAPER.2const.html),
[Linux reparent/exit ordering](https://github.com/torvalds/linux/blob/master/kernel/exit.c)
and [proc child iteration](https://github.com/torvalds/linux/blob/master/fs/proc/array.c).

The changes are limited to
[foreground.rs](../tools/yetlet-cli/src/platform/process/foreground.rs) and one
focused [command integration test](../tools/yetlet-cli/tests/commands.rs), plus
this record and documentation references. Windows code, public contracts,
dependency locks, user settings and application source remain unchanged.

## Focused correctness evidence

With the already prepared OpenSSL prefix selected through `OPENSSL_DIR` and
`OPENSSL_STATIC=1`, these checks passed with exit 0:

```sh
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib foreground::tests -- --test-threads=1
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test commands natural_cli_exit_keeps_grace_for_adopted_children_and_reaps_them -- --exact --test-threads=1
cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib --bin yetlet --test commands -- -D warnings
cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check
```

The six existing foreground cases passed: native exit/signal attribution,
natural leader exit with an orphan, Ctrl-C forwarding/escalation and unrelated
group isolation, private-PTY Ctrl-C/terminal restoration, watch recovery and
cleanup, and the existing subprocess fixture. These fixtures are not a claim
that the single-thread fast path ran in every unit-test subprocess.

The new integration invokes the actual CLI and real frozen Cargo application.
Its root exits 17 after spawning a child. In one branch, that adopted child
handles TERM, takes 100 ms and writes a completion marker; the marker proves
that grace was retained. In the other, the child ignores TERM and must be
forcibly reaped. Both preserve exit 17 and verify the recorded child PID is
absent when the CLI returns. The existing three command cases passed separately,
covering frozen locks/cache failures, discovery and environment-only doctor.

The first command-suite attempt passed those three existing cases but rejected
the new fixture's group-writable temporary parent. The sole fixture correction
sets mode 0700, matching the existing protected-app fixture; the focused rerun
above passes. No product permissions were weakened. An initial incorrect test
filter selected zero cases and is not counted as verification; the corrected
foreground selection ran all six. Those outcomes are retained in
`initial-verification.json` and the tool transcript.

## Measured result

Evidence directory: `/tmp/yetlet-yt102-completion-20261020-bs5_yhvs/`.
`timing.json` and named per-command logs retain actual results. The app fixture
reuses the earlier CLI-created plain SSR source and its explicit compatible
local dependencies/offline lock. A separately prepared target was warmed before
the measured runs. CLI and direct Cargo alternate order across three pairs.
Both use the same app, debug profile and target, with non-TTY stdin; two extra
CLI builds use fresh owned controlling PTYs. Every build exits 0.

| Lane | Warm wall seconds |
| --- | --- |
| Earlier audit CLI build, one observation before this change | 0.565 |
| Current CLI build, three observations | 0.030, 0.034, 0.038 |
| Current direct Cargo build, three observations | 0.026, 0.025, 0.023 |
| Current CLI build in controlling PTY, two observations | 0.040, 0.038 |

These show removal of the observed fixed waiting on this host; they are not
percentiles, an SLO or a clean-machine benchmark. The old single observation is
not a simultaneous before/after pairing. No Vite, Windows, watch-feedback or
in-process hot-reload speed claim follows. The per-task build target was
removed; no preview/listener is started by these timing builds.

## Completion and limits

`cargo xtask check docs` and `git diff --check` passed with exit 0. Independent
review checked the actual code/test diff, timing logs, source identities and
cleanup evidence, including the kernel ordering behind the narrow procfs check.
At that Linux checkpoint, Windows, HMR and wider browser matrices were not rerun.
The separately requested Windows followup below adds native CLI evidence.
If procfs or the single-thread invariant is unavailable,
the existing bounded wait remains correct. This slice adds no automatic install,
deployment, release or new public process guarantee.

The next practical DX candidate is the audit's compact first-use prerequisite
guide, independently of this latency change. Prior YT-095/YT-102 research and
historical failures remain preserved; implementation here does not commit them.

## Windows native followup (2026-10-20)

The owner subsequently requested Windows verification. WebCodex project
`agent:ncut-bohe-244143825ae3488e:localtasks`, Session
`wc_sess_yBMwGevIOYkZ3yck`, ran the existing integration target on `NCUT-BOHE`,
Windows 11 Professional build 26300, native Rust/Cargo 1.99 MSVC. It reused the
previously prepared static OpenSSL prefix and protected Cargo copy with
process-local environment assignments. No installation or global ACL/PATH
change was needed. The first inspected GNU Runner lacked that OpenSSL prefix;
its unused transfer was removed and its original checkout/HEAD/status retained.

The task-owned root is `C:\repos\localtasks\yt102-windows-20261020`.
All 508 public source/manifest/lock/template/test files match the current Ubuntu
checkout at base `649be69d2be1a5a08512767149b54b87b927b90e` plus its existing
uncommitted CLI change, before and after execution. Archive SHA-256 is
`9d0a35a79f0433d335f8c5249d6ac9813772e8fafb651c2034a42bb248785cc2`;
the compiled Windows CLI SHA-256 is
`e99335097c3e62e5f55e24c49680800c967894e6631f040382cfed60b7e1f8c6`.
Linux-only code and its Linux-only command test are present in that inventory
but are not compiled/executed as Windows behavior.

```powershell
cargo test --offline --locked --target x86_64-pc-windows-msvc --manifest-path <source>/tools/yetlet-cli/Cargo.toml --test windows_build -- --test-threads=1 --nocapture
# After the single validation-environment correction described below:
cargo test --offline --locked --target x86_64-pc-windows-msvc --manifest-path <source>/tools/yetlet-cli/Cargo.toml --test windows_build build -- --test-threads=1 --nocapture
```

| Existing coverage | Native result |
| --- | --- |
| Debug/release build and frozen lock/cache policy | Successful builds 0; missing lock 1; stale lock and empty offline cache 101; lock retained |
| Unsafe Cargo and protected copy | Unsafe executable rejected at `stage=executable`, exit 125, before build output; protected copy builds, exit 0 |
| Preflight and executable selection | No app-local unsafe Cargo launch; existing refusal checks pass |
| Attached-console finite run | Application exit 23 retained; frozen/offline/headless refusals pass |
| Console Ctrl-C and Ctrl-Break | Both statuses `0xc000013a` (3221225786); all three pinned resistant descendants terminate and observe forwarded BREAK; unrelated sentinel survives without a forwarded event; lock unchanged |
| Console helper test | Pass; actual subprocess behavior is asserted by the preceding console cases |

The first invocation returned 101: four cases passed, two build cases failed
their expected-output-path assertions, and the existing Vite/watch cases were
ignored. This validation script had set `CARGO_BUILD_TARGET`, which moved fixture
artifacts under a target-triple directory even though both builds succeeded.
Removing only that process-local setting restored the fixtures' normal
`target/debug` and `target/release` paths. The focused `build` filter then ran
exactly the two failed cases: two passed, zero failed, six filtered out, exit 0.
Thus all six distinct default cases have successful evidence across two
invocations; this is not a claim of a single all-green suite invocation.
Neither product nor test source changed. The original failures remain retained.

The console cases use `CREATE_NEW_CONSOLE` and real `GenerateConsoleCtrlEvent`
events, not an injected stdin control character. They assert descendant handles
are signaled and the unrelated sentinel is alive before outer Job cleanup.
Initial wall time was 36.080 seconds including 27.50 seconds of compilation;
the focused rerun took 2.472 seconds. These are verification-command timings,
not CLI latency measurements or a Windows performance improvement claim.
The retained OpenSSL libraries emitted nonfatal LNK4099 missing-debug-PDB
warnings; linking and the behavioral checks completed.

Local evidence is in `artifacts/yt102-windows-20261020/`: `source-hashes.json`,
`preflight.json`, `environment.ps1`, both native logs, `test-result.json`,
`focused-result.json`, `final-result.json` and `cleanup.json`. The returned
`results.zip` SHA-256 is
`2b07decc2ad17cc77b7459cca489066b1fdf0ff7757f953d241902d7ce4fe016`.
WebCodex test Job `wc_job_oYzMQ19vaUUBFUjF` retains the original exit 101;
the focused rerun completed synchronously with exit 0.

Cleanup confirms zero task-root processes, the new build target removed, and
the global Cargo executable hash/DACL unchanged. The first cleanup wrapper
aborted before deletion on an unused invalid object-property assignment; its
premature artifact transfer found no file and changed nothing. Removing that
assignment allowed the bounded cleanup/export to complete; `cleanup.json`
preserves this operational failure separately from product evidence.
Old tools, cache, checkouts and verification records remain intact.

WebCodex's advisory closeout independently verified six output hashes and no
active Jobs. Its ledger still reports `task_outcome=fail` for the retained
initial cleanup-wrapper error, with `validation_source_unproven` for current
source freshness. The latest validation passed, with one resolved failure and
zero unresolved validation failures. This is not an all-green advisory ledger;
the explicit 508-file hash matching and native assertions above provide the
direct source/behavior evidence. `closeout.json` preserves these separate
outcomes; no test was rerun merely to clear the ledger.

Independent Reviewer `/root/yt102_windows_followup_review_oct20` PASS has zero
issues or required followups. Its bounded advisory addendum confirms the
retained fail/unproven ledger fields are factual reporting, not a material
native-verification gap; obtaining a green advisory ledger needs no rerun.
Windows process code is unchanged. The two explicitly ignored Vite/watch cases, browser/HMR checks and
Linux fast-path performance are outside this native followup. No commit or
push is included. The next DX candidate remains the compact prerequisite guide.

## Windows warm-build baseline (2026-10-20)

The owner subsequently requested actual Windows performance measurements, then
accepted retaining a baseline instead of repeating Windows performance checks
for every unrelated change when the overhead is small. The preceding native
behavioral evidence and historical failures remain accepted and were not rerun.

The same WebCodex Project/Session and retained 508-file source snapshot were
used on the same Windows 11 MSVC Runner, with an Intel Core i7-10700 CPU. A new
task-owned `performance/` directory contains an owner-protected app created by
the rebuilt current CLI: `yetlet new latency --parent <performance>`.
Only its two unpublished Yetlet dependencies were explicitly pointed at the
retained local source, and Cargo generated its offline lock separately. The
generated SSR application source was retained. No server, Vite or watcher was
started. CLI compilation, initial app compilation and one extra warmup per lane
are excluded from the samples.

The measured commands, both with the app as cwd, are:

```text
yetlet build --manifest-path Cargo.toml
cargo build --frozen --manifest-path Cargo.toml
```

Both use the same protected physical Cargo executable, cached dependencies,
debug profile, `app-target` directory and Rust 1.99 native MSVC toolchain, with
offline/no-auto-install environment assignments. A single PowerShell parent
uses literal `.NET ProcessStartInfo.ArgumentList` arguments and a monotonic
Stopwatch from immediately before `Process.Start` through `WaitForExit`. Stdin
is a pipe closed immediately; stdout/stderr are drained asynchronously and
persisted after the timer stops. WebCodex transport, PowerShell startup and log
file writes are excluded. This is a noninteractive build measurement.

| Lane | Three warm wall samples (ms) | Median (ms) |
| --- | --- | --- |
| Current `yetlet build` | 267.108, 265.255, 272.587 | 267.108 |
| Direct `cargo build --frozen` | 222.954, 225.420, 228.257 | 225.420 |
| Difference between the two medians | | 41.688 |

Order alternates: CLI/Cargo, Cargo/CLI, CLI/Cargo. All six samples and both extra
warmups return native exit 0. The warm logs report only Cargo's completed dev
profile, with no recompilation. App manifest/source/lock hashes and all 508
retained source hashes match before and after. The rebuilt CLI SHA-256 is
`89cdf02a98b28c91c38b9ae696c422f8d879b145575e4c558c8e0fe5cfe120e9`;
Cargo remains
`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
The separately rebuilt CLI uses a different build directory from the preceding
behavioral run; its binary hash is recorded separately, with matching sources.

On this host, the observed CLI overhead is about 42 ms. These samples show no
fixed half-second completion wait on this path. They do not identify which
preflight, process or filesystem operation contributes to the overhead. The
absolute Windows time is higher than the earlier Ubuntu observations, including
direct Cargo; different machines, OS environments and measurement wrappers do
not establish an x86-versus-x86 platform benchmark. Three samples are a bounded
baseline, not percentiles, an SLO or a cold-build/run/watch/PTY performance claim.
No Windows process-code optimization is justified by this measurement alone.

Following the owner's direction, retain this baseline for ordinary unrelated
changes. Revisit Windows timings when relevant executable selection, native
process/Job cleanup, console handling or toolchain behavior changes, or when a
measured regression is reported. A common x86 CPU architecture alone does not
prove equal OS process costs. Native behavioral checks remain scoped to the
affected contract; this decision does not introduce a mandatory benchmark gate.

Evidence is in `artifacts/yt102-windows-20261020/performance/`: `measure.ps1`,
`timing.json`, preparation records, the app manifest/lock/source, all warmup and
sample logs, and `cleanup.json`. The returned `performance-results.zip` has
SHA-256 `e41fc9589ca64c618680de0a16215ac686027e73f263ced3fbe6bee66cc8baa1`.
The first human-readable summary projection printed null fields because it
selected properties from an ordered hashtable; the canonical JSON contains
the numeric fields and all six complete rows. No measurement was rerun or
changed to correct that display. Cleanup confirms zero owned processes, both
new build targets removed, and unchanged global Cargo hash/DACL. Old native
evidence, tools and cache remain intact. Independent Reviewer
`/root/yt102_windows_perf_review_oct20` PASS has zero issues or required followups.
No product edit, commit or push is included.
