# YT-083 — Rust xtask foundation verification

**State: Done** after corrected Linux/Windows MSVC execution and a fresh
independent Reviewer PASS with zero issues or required followups. Earlier
failures below remain historical and do not qualify Windows GNU.
**Checkout:** `$HOME/repos/yetlet` on the recorded Ubuntu runner, HEAD
`7cfac613f908c5c19fa581438288386809870b56`. No commit, push, release, detached
worktree, global installation, profile edit or cross-host connection occurred.

## Scope and changes

The owner explicitly authorized validation tooling as the requested product and
assigned only the YT-083 foundation in the approved
[M12 plan](M12-RUST-TOOLCHAIN-PLAN.md). YT-084 through YT-086 ports and public CLI
YT-087 are not implemented. The [migration ledger](M12-PYTHON-MIGRATION.md) stays
byte-identical with all 52 files and three workflows Not migrated.

Changed files owned by this task:

- `.cargo/config.toml`: reviewed root-only locked xtask alias.
- `xtask/Cargo.toml`, `xtask/Cargo.lock`: unpublished isolated workspace and lock;
  cached `serde_json` 1.0.149, Unix-only `libc` 0.2.183 and Windows-only
  `windows-sys` 0.61.2. No framework runtime dependency.
- `xtask/src/lib.rs`, `xtask/src/main.rs`: native argument parsing, default help,
  read-only installed toolchain doctor and stable human/JSON error handling.
- `xtask/src/process.rs`, `xtask/src/process/unix.rs`,
  `xtask/src/process/windows.rs`: literal argv/cwd/env, bounded dual-stream
  capture, monotonic deadlines, cancellation, native child status, Unix process
  groups and Windows suspended-start Job ownership, checked cleanup and RAII.
- `xtask/tests/foundation.rs`: direct test-binary fixtures, not a shipped fixture
  command or a general test framework.
- `xtask/README.md`, this record and only the YT-083 backlog state/evidence and
  ready-queue text in `docs/BACKLOG.md`.

The parent-owned six YT-082 documents were already uncommitted at entry.
Except for the requested YT-083 backlog update, their bytes are preserved;
`target/m12/yt083/parent-backlog.patch` retains the original backlog diff.
Historical M11 source/review records, scripts, root manifest/lock and workflows
were not changed. No new ADR is needed for this isolated automation boundary.

## Executed Linux evidence

Installed Rust/Cargo: 1.99.0. Logs below are local ignored receipts under
`target/m12/yt083/`; they are not Windows or independent-review evidence.

| Actual command/check | Result | Receipt |
|---|---|---|
| `cargo generate-lockfile --offline --manifest-path xtask/Cargo.toml` | Exit 0, independent lock resolved from cache | `xtask/Cargo.lock` |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0: 11 checks passed; two ignored test-only fixture entry points are explicitly invoked by tests | `cargo-test.log` |
| `cargo fmt --check` | Exit 0, root defaults unchanged | `root-fmt.log` |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | Exit 0 | `xtask-fmt.log` |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Exit 0 | `clippy.log` |
| `cargo check --offline --locked` | Exit 0, ordinary root default build remains valid | `root-cargo-check.log` |
| `cargo xtask doctor --capability foundation --json` | Exit 0; policy/locks and installed Rust/Cargo 1.99.0 available, both child exits 0 and cleanup true | `doctor.json`, `doctor.stderr` |
| `cargo xtask doctor` | Exit 0, matching human status/version summary | `doctor-human.log`, `doctor-human.stderr` |
| `env PATH=/nonexistent xtask/target/debug/yetlet-xtask --help` | Exit 0 without tool/interpreter discovery | `help-without-tools.log`, `help-without-tools.stderr` |
| `python3 scripts/check_docs.py` | Exit 0: 75 documents, 86 backlog items; language/links/structure/whitespace/DAG PASS | `check-docs.log` |
| `git diff --check` | Exit 0 | `diff-check.log` |
| Root `cargo metadata --offline --locked --format-version 1` before/after | Byte-identical, including resolved graph/features and workspace membership | `root-before.json`, `root-after.json` |
| Root `cargo tree --offline --locked -e features` before/after | Byte-identical | `root-before.tree`, `root-after.tree` |
| SHA-256 baseline checks of root manifest/lock and five untouched parent docs | All OK; root lock remains `2e08330c49ff4e1740102a37b36fbe2fd044bd2033d8ea9a6aaf224a402866c3` | `parent-baseline.sha256`, `isolation.log` |

Tests directly cover exact literal shell-metacharacter argv, native environment,
spaces/Unicode executable and cwd, non-UTF-8 Unix paths/environment, concurrent
stdout/stderr exceeding pipe capacity with bounded retained prefixes, exact
child exit 37, Unix SIGTERM identity, missing executables, nonzero timeouts and
cancellation, RAII fallback on unwind, unknown commands/flags/capabilities,
missing tools and executable identity/version-protocol failure. Scoped fixtures
cover timeout, successful leader exit and failed leader exit with live
pipe-holding descendants. Owned heartbeat stops, the group is reaped and gone,
and an unrelated instance of the same executable continues its heartbeat until
explicit test teardown. Windows has a gated full-DWORD status test; it was not
compiled or executed here.

The first suite retained one `EPERM` fixture setup failure when this environment
refused loopback bind (`cargo-test-initial.log`). Classified `harness_failure`:
one in-place fixture change used file heartbeats instead of sockets; the focused
same-test rerun passed (`scoped-cleanup.log`). No permission or host setting was
changed. A subsequent added RAII test used a nonexistent `Result::is_err_or`
method (`cargo-test-compile-failure.log`); its one-line correction to
`!is_ok_and` passed the focused library test (`raii-test.log`) and final full
suite. Conservatively charged one repair attempt for this compile failure.
Operational retries: 0; product/content repair attempts: 1 of 2. No failure is
left unresolved in the Linux checks.

Cleanup evidence: all owned fixture assertions passed, the unrelated child is
explicitly killed/waited by its test guard, and temporary fixture files are
removed by their guards. The final process-name inventory contains no fixture
process (`remaining-fixtures.log` is empty; the inventory filter exits 1 for no
matches). No server, browser, watcher or background task remains owned here.

## Independent review failure and focused cancellation repair

The first independent source/evidence review, Job
`wc_job_vA_y8Ov3V8dOZ4iA`, exited 0 but returned **FAIL** with one P2:
late cancellation during successful probe cleanup could leave doctor exit 0
and omit Cargo without an explicit reason. The original result remains at
`target/m12/yt083/reviewer/result.json`, SHA-256
`86ab735e0bc79895d3458689871b6dccbb39713f26819f7c2aaa3f088b6a2ed8`.
Exact thread `01a13e01-d931-7233-9dcc-b1b1cb957925` confirmed
`gpt-6-astra` / `high`; it was an independent local role-directed Job,
not native managed dispatch. No review evidence was silently changed to PASS.

The parent fixed only the cancellation paths in `xtask/src/process.rs` and
`xtask/src/main.rs`, adding three direct deterministic Rust regressions.
The runner observes cancellation again after cleanup and output draining,
retains native child status, and never turns cleanup/spawn failure into success.
Doctor reports cancellation as nonzero and explicitly lists remaining probes as
`not_run` with reason `cancelled`, without launching them. These tests use
an isolated atomic token; no flaky time-window search or global signal mutation.

The repaired source passed **14 Linux tests** and **13 Windows MSVC tests**;
the two test-only fixture entry points are still deliberately ignored by
discovery and invoked by tests. Strict Clippy, formatting and non-cancelled
doctor passed. Actual corrected Windows execution was Job
`wc_job_fQnJCnlH-12Cw3sV`, terminal exit 0, with 14/14 exact source hashes
unchanged, doctor exit 0 and zero owned processes remaining.
The corrected source archive SHA-256 is
`66636821fa6ab53ce1e8c944dac6af37c40d0cbc388493959d8172e9ee19c948`;
returned evidence archive SHA-256 is
`69be2762f47ef9deb4ce1e4f5d294c1516ab0e515976682eb359c254eaa52abe`.
Logs are in `target/m12/yt083/cancellation-fix/`, with exact Windows
logs/results/source hashes in `windows-return/`. Earlier 11/10 test results
below describe the pre-repair source, not the final corrected candidate.
A fresh independent review of this correction subsequently passed; see the final acceptance below.

## Platform boundary and remaining gates

The first Windows host's installed Rust 1.99 GNU build stopped **before tests**
with native exit 101 at `dlltool` import-library generation (`CreateProcess`
failure). No doctor/test PASS was inferred, no compiler/linker was installed,
and its original `3128d23` Git checkout stayed clean. This exact failed
observation remains in `target/m12/yt083/yt083-windows-gnu-failure/`.

A second already-authorized Windows Runner had installed **Rust/Cargo 1.99.0
MSVC, Visual Studio native linker and Windows SDK**. The same exact source
and cached dependency archive was transferred to a task-owned non-Git fixture
under `localtasks/target/m12/yt083/source with spaces`; this was not a new
worktree or replacement of any original checkout. No global configuration,
installation, dependency download, GUI or browser action occurred.

The source archive SHA-256 was
`211e2f2c1f92ef78f8fb54be248ce8f93bc9399f410c419aed71181e07b1f5b2`
(14 source/manifest/lock/policy files, plus the cached vendored dependencies).
All 14 source hashes matched before and after actual Windows compilation, and
matched original Ubuntu bytes when returned. Process-local `RUSTUP_TOOLCHAIN=stable`
selected the installed 1.99.0 MSVC compiler; Cargo used `--offline --locked`
and a command-line vendored-source override. No new root Cargo config was written.

**Actual Windows MSVC result: PASS**, WebCodex Job
`wc_job_RkKC_Kt8XhM-2IWK`, exit 0. Ten tests passed (two fixture entry points
are intentionally ignored by discovery and explicitly invoked by parent tests).
The Windows-only full DWORD test retained `0xc000013a`; paths with spaces and
Unicode, exact failures, bounded dual-stream capture, timeout/cancellation,
RAII and owned descendants after successful/failed leader exits all passed.
`doctor --capability foundation --json` exited 0, correctly reported the installed
1.99.0 rustc/Cargo and `cleanup_ok=true` for both probes. No owned process remained.

The returned archive SHA-256 is
`70f396a40ba38b6a5b6e12d49fe2059d36f032ddaa49f0ad4711424f33b517a6`.
Verified original logs, source hashes, doctor and result JSON are retained at
`target/m12/yt083/yt083-windows-msvc-results/` on Ubuntu. The separate GNU failure
archive SHA-256 is
`f938de50840774de45df6195719baedee317f70bc662ab03e8922207497f4710`.
This establishes the selected Windows **MSVC** foundation, not a GNU toolchain
PASS or arbitrary Windows environment support. No browser or future port is
qualified by these process tests.

The supported Unix scope is descendants that remain in the owned process group;
intentional daemonization/setsid escape and cleanup after supervisor SIGKILL are
not claimed. Windows Job assignment/resume must succeed on the actual runner or
fail nonzero. Other Unix systems have not been exercised. Browser/CSP suites,
hosted workflows, M11 workloads, port parity and production qualification were
not run or newly accepted.

## Final independent acceptance

The separate follow-up review, WebCodex Job `wc_job_yPSca7ozsVyxWPhR`,
completed with exit 0 and **PASS**, issues `[]`, required followups `[]`.
Exact thread `01a13e0b-93c7-78b2-a64c-b31e9366465e` runtime telemetry
confirmed `gpt-6-astra` / `high`, in the original checkout. The review was
read-only and role-directed local Codex execution, not native managed dispatch.
It rechecked the cancellation repair, deterministic tests, Linux 14/Windows
MSVC 13 results, all 14 Windows source hashes, isolation and cleanup. Reviewed
source/document hashes were unchanged during review. The retained result is
`target/m12/yt083/reviewer-after-cancellation/result.json`, SHA-256
`ceb27bfdccffe8f9d8f65e64b99941e375e5ee601d2cd657a1d081f569c65aae`.
No new browser/process fixture, build or host change was performed by the reviewer.

YT-083 is Done at this foundation scope. YT-084 is now the next authorized
implementation item; it still needs its own parity and independent review.
None of the 52 Python entry points is migrated by this foundation alone.
No commit, push, publication or YT-020 activation occurred.
