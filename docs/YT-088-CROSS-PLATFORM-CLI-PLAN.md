# YT-088 — Linux/Windows Yetlet CLI Platform Parity Plan

**State (2026-10-16):** Engineering design independently reviewed PASS by read-only Job `wc_job_SPLMk3nKvrzHPwYV` (0 issues/followups). [YT-088a internal adapter extraction](YT-088A-VERIFICATION.md) is independently reviewed Done by Job `wc_job_cO62Qi9H1GTmD9WI` (zero issues/followups) after Linux 25+1 and native GNU/MSVC 7+7 passing regressions. YT-088b is also Done within its separately accepted native Windows filesystem/new and generated-SSR scope, following final independent read-only Reviewer `wc_job_6Y-nSautm2RhPMPa` PASS. YT-088c is Done at native Windows GNU/MSVC frozen CLI build scope after independent Reviewer `wc_job_8J7lc5L7tuyUwnWd` PASS (zero issues/followups). [YT-088d native foreground dev](YT-088D-VERIFICATION.md) has actual native GNU/MSVC Console Ctrl-C/Break, resistant process and full SSR HTTP source-bound verification, with final independent Reviewer `wc_job_w0O0AXkCGwQAT5nP` PASS (0 issues/followups). [YT-088e final cross-platform closeout](YT-088E-VERIFICATION.md) received independent read-only Reviewer `wc_job_IPC2lyaZyAz72KlT` PASS (zero issues/followups), after current-source Linux 25+1 and native GNU/MSVC 28+8 regression and documentation checks. The maintainer explicitly identified Windows/Linux CLI feature parity as an intended baseline and asked whether narrow operating-system abstraction is appropriate. YT-088 is Done at the bounded unpublished Linux/Windows CLI preview scope; preserve YT-087 Done only at its independently accepted unpublished **Linux** scope. Public release YT-020 remains Blocked.

**Design review boundary:** Reviewer `wc_job_SPLMk3nKvrzHPwYV` approved only the two narrow compile-time OS adapter boundaries, existing Linux preservation and the staged Windows proof requirements. It did **not** validate an implemented Windows rename/ACL/Job Object/console adapter or accept the later YT-088a source. YT-088a behavior-preserving extraction and YT-088b native filesystem/new have separate source-bound Reviewer PASS decisions. YT-088c build has its separate final independent PASS; YT-088d foreground dev has distinct GNU/MSVC native evidence and independent Reviewer PASS; 088e has now passed distinct independent final review; no public or production release is implied.

**YT-088a–e independently accepted at bounded scope:** [088b verification](YT-088B-VERIFICATION.md) preserves its accepted Windows filesystem/new and generated SSR evidence. [088c verification](YT-088C-VERIFICATION.md) preserves historical failed candidates, native GNU/MSVC 27 unit + 5 integration PASS, actual generated-app public CLI debug/release builds with unchanged explicit locks, and final independent Reviewer PASS. [088d verification](YT-088D-VERIFICATION.md) records independently accepted console-attached dev, targeted child-group BREAK, bounded private-Job cleanup, headless refusal, actual Windows GNU/MSVC SSR/Console journeys and resistant descendants. YT-088 is Done after 088e independent final integration review at unpublished Linux/Windows developer CLI scope.

## Problem and minimum support promise

YT-087's 15-file current-source verification on both Windows GNU and MSVC independently passed native `cargo fmt`, `cargo check`, Clippy, five applicable Rust tests and the **expected unsupported-platform refusals**. Those results verify build portability and honest fail-closed behavior, not a working cross-platform CLI. At the end of YT-087, `scaffold::create` and `app::run` explicitly returned Unsupported for non-Linux. YT-088b/c have since independently established Windows new and frozen build; 088d now independently accepts console-attached native foreground dev, and 088e final cross-platform closeout is independently accepted.

Minimum target: **Linux and Windows must both run the same CLI commands** (`doctor`, `new`, `build`, `dev`) with the same public arguments, generated app contract, security requirements and documented exit/failure semantics. Target actual Windows x86-64 MSVC and GNU runners separately, reusing their existing Rust 1.99 environments; do not treat cross-compilation or compile-only evidence as runtime proof. Additional OS platforms are deferred and must fail closed.

Preserve current handler-first/no-JS default, root/xtask Cargo graphs, root lockfiles, isolated unpublished CLI workspace, strict CSP/escaping, no Python/Node dependencies in the CLI implementation, and no release activation, host deployment, tag, Git commit or push.

## Architecture: two narrow internal platform boundaries

Keep parsing, validation, TOML/lock checks, templates, documentation and command contracts in the shared CLI domain. Add an **internal, statically selected platform implementation**, not a public plugin API or dynamically loaded runtime backend:

```text
src/lib.rs + main.rs          # shared command parsing, exact args/errors/help
src/doctor.rs + scaffold.rs   # shared diagnostics, names, templates
src/app.rs                   # shared Cargo manifest/lock and --frozen contract
src/platform/mod.rs          # internal seams and compile-time contract test
  fs/mod.rs                  # cfg selection and shared app preflight ordering
  fs/linux.rs                # protected app files and pinned directory context
  fs/safe_dir.rs             # unchanged descriptor-relative safe_dir implementation
  fs/unsupported.rs          # current non-Linux refusal and prior doctor probe
  process/mod.rs             # shared supervision entry and final exit handling
  process/linux.rs           # pinned cwd and native signal operations
  process/foreground.rs      # unchanged owned foreground and SIGINT notifier
  process/foreground_tests.rs # retained process tests, relocated fixture selector
  process/unsupported.rs     # current non-Linux process refusal
```

`fs/windows.rs` contains accepted 088b creation and 088c app preflight; `process/windows.rs` retains native build ownership and independently accepted console-attached dev. Headless Windows dev and other OS process adapters fail closed before child startup. `app.rs` keeps TOML parsing, lock/manifest semantics and literal frozen Cargo arguments; `scaffold.rs` keeps name validation and templates. No adapter is exported from the public CLI library.

The boundary is capability-oriented:
- `FilesystemAdapter::create_project(parent, name, files)`: create one new project with no replacing destination, safe rollback and source identity checks.
- `FilesystemAdapter::open_app(manifest)`: return an owned/pinned context that safely reads the selected standalone Cargo.toml and Cargo.lock; never follow a symlink/junction outside intended checkout and never treat a stale lock as success.
- `ProcessAdapter::run_cargo(app, command)`: run one literal frozen Cargo `build` or `run` foreground tree, supervise only owned children, propagate native status/cancellation, restore console/terminal and enforce a bounded cleanup deadline.

These are **internal contract names**, not mandatory Rust `dyn Trait`/DI objects or exposed public APIs. Prefer Rust `#[cfg(target_os = "linux")]` / `#[cfg(windows)]` module selection and static functions/structs. Introduce test seams only when concrete tests need them. Do not duplicate command parsing, template rendering or Cargo policy in Windows-specific files. Maintain semantic alignment while allowing OS-specific native exit representation (POSIX signals versus Windows control events/exit codes).

## Platform filesystem contract

Common conditions: no `..` traversal, invalid device names, UNC/device-path ambiguity or untrusted reparse traversals; escaped Unicode and spaced paths work; existing file/dir/symlink/junction wins any creation race; only the CLI's exact temporary files can be deleted. A failed or ambiguous publication is nonzero; never blindly retry or remove a destination of uncertain identity.

**Linux:** preserve the independently accepted `O_NOFOLLOW`, directory-FD and `renameat2(RENAME_NOREPLACE)` design byte-for-byte where possible. No downgrade to a naive existence-check and `std::fs::rename`.

**Windows:** use OS-native directory/file handles, not a path-based check-then-rename approximation:
- Open directory handles with `CreateFileW` + `FILE_FLAG_BACKUP_SEMANTICS` / `FILE_FLAG_OPEN_REPARSE_POINT`; reject junctions/symlinks/reparse traversals and unsafe security/ACL or sharing conditions.
- Verify volume/file identity via `GetFileInformationByHandleEx(FileIdInfo)` or equivalent; pin and revalidate the complete selected parent identity before mutation/publication.
- Create the task-owned staging directory on the same supported filesystem under a parent that cannot be replaced by untrusted principals during creation; reject untrusted parent ACLs and unsupported network shares/filesystems unless a safe handle-based guarantee is demonstrated.
- Publish via a **documented non-replacing, destination-parent-handle-relative rename**. The independently tested YT-088b implementation uses `NtSetInformationFile(FileRenameInformation)` with a held parent directory handle and zeroed `ReplaceIfExists`, under strict local NTFS/ACL/reparse constraints; the originally proposed `SetFileInformationByHandle`/`FileRenameInfoEx` is not the selected implementation. No `MOVEFILE_REPLACE_EXISTING` or unverified check-then-rename fallback.
- On failures, remove only exact owned staging objects after identity comparison; never recursive-delete a foreign destination. Test parent junction swap, late target appearance and locked/in-use destinations before claiming Windows support.

These are testable design requirements, not an assertion that all named Win32 combinations are automatically safe; the Windows implementation must prove its selected API path on native Windows.

## Platform process/console contract

**Linux:** preserve accepted `fchdir`, process-group, subreaper, terminal notifier, bounded INT/TERM/KILL cleanup and native signal propagation.

**Windows:** use a real owned **Job Object** and native console semantics:
- Create one **suspended** Cargo child, ensure assignment to a uniquely owned Job Object **before resuming the initial thread**, and fail/terminate safely on any spawn/assignment error. Job must retain ordinary Cargo descendants; use `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and do not allow silent child breakaway. Account for the possibility of an existing parent Job Object/nested Job on Windows runners; fail closed when exclusive ownership cannot be established.
- Do not implement `CREATE_SUSPENDED` through `std::process::Command` unless the primary thread handle can actually be resumed and assignment ordering is proven. Prefer documented `CreateProcessW`/thread handle with correctly quoted Unicode arguments and inherited stdio, or a demonstrated equivalent OS API.
- Foreground console Ctrl-C/Break: `SetConsoleCtrlHandler` on the supervising CLI; when appropriate use `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child-process-group-id)` for a console-attached group. Windows `CTRL_C_EVENT` is **not reliably group-targeted**: do not broadcast it to unrelated console processes. After a bounded graceful period, terminate only the owned Job and wait/reap its members. For headless/no-console callers, preserve noninteractive owned lifecycle and fail closed if the required cancellation guarantee is unavailable.
- Preserve native Windows child exit statuses, distinct preflight vs spawn/supervision errors, and explicit cancellation semantics. Do not claim that Windows Ctrl+C has a POSIX signal number or invent identical native exit codes between OSes.
- The CLI is not a process sandbox; intentionally escaped processes/session groups remain unsupported until separately specified.

## Work slices and independent evidence gates

| Slice | Deliverable | Evidence required before independent PASS |
|---|---|---|
| YT-088a | Refactor into the narrow internal `platform::fs` and `platform::process` boundaries with no behavior changes | [Local evidence](YT-088A-VERIFICATION.md): Linux original 24+1 CLI tests preserved, now 25+1 with a static seam test; fmt/check/strict Clippy pass; actual retained HTTP/PTY receipts preserved; root/xtask graph unchanged. Current-source Windows MSVC/GNU compile/test and independent Reviewer PASS still required |
| YT-088b | Actual safe Windows `new` and cross-platform doctor platform checks | Native MSVC/GNU scaffolding, no-replace / junction/symlink / parent race / foreign-data preservation / rollback, spaced-Unicode paths; actual generated independent Cargo project gets its own explicit offline lockfile, compiles `--frozen`, HTTP GET 200 escaped SSR/CSP/no JS on loopback; separate Reviewer PASS |
| YT-088c | [Accepted frozen Cargo build](YT-088C-VERIFICATION.md) with secure app preflight, pinned cwd/files and Job ownership | Done: Windows GNU/MSVC 27 unit + 5 integration PASS, actual generated-app CLI debug/release builds and lock hashes preserved; final independent Reviewer `wc_job_8J7lc5L7tuyUwnWd` PASS, zero issues/followups |
| YT-088d | [Accepted Windows foreground dev](YT-088D-VERIFICATION.md), private Job and console cancellation | Native GNU/MSVC real `cargo run --frozen` SSR HTTP/CSP/escaped HTML, Ctrl-C/Break, resistant descendant, native Windows exit, private Job/listener cleanup, preserved Linux checks; final independent Reviewer `wc_job_w0O0AXkCGwQAT5nP` PASS |
| YT-088e | Cross-platform docs, final status and independent review | All four commands honestly available on Linux+Windows; MSVC/GNU fmt/test/check/Clippy and actual dev journeys source-bound, docs/sample checks, preserved raw historical YT-087 and Windows verification receipts; final separate Reviewer PASS |

Execution order may be refined after the YT-088 design Reviewer. No step may be skipped or marked Done using a compile-only result, success-shaped stub, Windows expected-refusal test, or cross-OS exit-code assumption. Use original Ubuntu checkout, existing two Windows WebCodex Runners, separate task-local test outputs, no detached worktrees, and no automatic unapproved commit/push.

## Historical YT-088a initial evidence and next gate (superseded)

The original `master` checkout remains at `94cf8f555b554a26c2416842eab06110b229904d`, preserving the extensive uncommitted YT-087 source, documentation and Windows receipts. Rust 1.99.0 offline/locked isolated CLI checks pass with **25 unit + 1 Cargo integration tests, zero failed/ignored**. `safe_dir.rs` and `foreground.rs` are byte-identical relocations; the foreground test file changes only its exact fixture module selector and formatting. Existing application preflight, literal command, exit/signal, resistant foreground PTY, descendant/orphan cleanup and unrelated-group assertions remain active. Exact commands, repair history, source manifest, docs/sample results and limits are in [YT-088a verification](YT-088A-VERIFICATION.md).

This is local implementation evidence from a role-directed Codex task, not native managed dispatch or independent acceptance. The parent WebCodex must bind the final source manifest to native GNU and MSVC checks with expected unsupported-command results, then arrange a separate read-only Reviewer. Historical Windows receipts attest only the old YT-087 source. No Windows checkout was accessed by this Linux task; no new generated-app HTTP journey was required or claimed. YT-088b is the next implementation slice only after those YT-088a gates pass.

## Grounded Windows OS API references

- Microsoft `FILE_RENAME_INFO` documents `ReplaceIfExists=false` failure when destination exists, and a `RootDirectory` handle for relative renames: https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info
- Microsoft `CreateFile` documents directory handle and reparse flags: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
- Microsoft Job Objects describes descendant membership, breakaway behavior and kill-on-last-handle-close: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
- Microsoft `GenerateConsoleCtrlEvent` explicitly warns that `CTRL_C_EVENT` cannot be reliably limited to a selected process group: https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent

The first two OS implementations and their Win32 API usage must be reviewed against the **current** platform SDK as code is written; this plan does not certify them. Do not add a general-purpose cross-platform runtime abstraction to the Yetlet framework crates: this is the **developer CLI only**.
