# YT-086 — Native Measurement, Qualification and Release Candidate

**Final disposition (2026-10-15):** Done at bounded source/fixture migration scope. Final independent read-only Reviewer `wc_job_6UfDDdHnNCBXkMhA` returned `overall_status=pass`, zero issues and zero required followups. The original twelve P2 findings are closed. Prior in-progress checkpoints below are historical. Controlled-PATH source/filesystem commands invoked no Python/Node; the complete unexecuted runtime graph, hosted private rebuild, GPU, Docker/Redis, Windows cancellation and long-load qualifications remain Not verified. Full xtask test run retained 117 passing, four loopback `PermissionDenied` failures and two ignored entries; no full-suite PASS is claimed. The reviewer could not perform Cargo check in its read-only filesystem. YT-085 remains accepted; no commit, push, release or public CLI activation occurred.

**State: In progress.** This is one role-directed Generalist Job in the existing
`$HOME/repos/yetlet` checkout on `master`, starting at`7cfac613f908c5c19fa581438288386809870b56` with YT-082–YT-085 changes already
uncommitted. It is a WebCodex-launched execution, not native managed-role dispatch.
A separate independent Reviewer Job and PASS are required before Done. This Job
neither performs nor claims that review.

YT-085 remains Done. Its accepted separate Reviewer receipt is
`target/m12/yt085/reviewer-board-retention-20261015/result.json`, SHA-256
`251c3230e936c89294e15c3f15297eb5cb12363490cd2cd90d3463816df84fce`.
All prior runtime receipts, failures, environment boundaries and original Python
sources remain historical and unchanged.

## Exact twelve ledger rows and implementation

| Original oracle | Rust candidate | Evidence and acceptance boundary |
|---|---|---|
| `examples/lab-operations-board/qualify_local.py` | `qualify board` | Five original profile definitions, 10,000/90,000 seed, contention/unread-response/schema rollback, paced 25-client mix, ten reconnecting SSE readers, raw samples, strict gates and optional tracing. Long profiles not executed. |
| `examples/webgpu-scene-island/verify_dynamic.py` | `measure scene-dynamic` | Matched round/size/renderer order, three workloads, five warmups plus thirty samples, sustained/replacement lanes, pixels and final-state equivalence, 420/480-second bounds and nonzero partial results. Actual GPU measurement not executed. |
| `qualification/yt066b3/run.py` | `qualify redis-three-app` | Pinned disposable topology, CAS/retirement/TTL, unknown outcomes, bounded admission/drain, outage/restart and existing no-JS browser fixture. Redis integration not executed. |
| `scripts/measure_baseline.py` | `measure baseline` | Five HTTP warmups, measured first byte after connect, asset/hash/gzip facts, Chrome timing marks and lazy Search checks; raw timing arrays retained. Browser measurement not executed. |
| `scripts/measure_render_strategies.py` | `measure render-strategies` | Production-before capture and recheck, seeded existing browser workload, all four strategy identity/equivalence/focus/count assertions. Browser measurement not executed. |
| `scripts/measure_sample_dx.py` | `measure sample-dx` | Source/test boundary and async counts, application-package closure, normal/build/target dependencies, optional capability source. Actual source measurement executed. |
| `scripts/qualify_local_operations.py` | `qualify local-operations` | Memory isolation/restart, SQLite two-process CAS/audit, online backup deadline/no overwrite, exact restore/late-write exclusion and corrupt candidate native-exit check. Runtime attempt failed; retained separately. |
| `scripts/qualify_multi_instance.py` | `qualify multi-instance` | Exact project/name cleanup and rechecks, pinned constrained build/loopback topology, stateless survivors/rejoin and Memory rejection. Docker runtime not executed. |
| `scripts/release_checksums.py` | `release checksums create/verify DIRECTORY` | Flat bounded unsigned schema, duplicate keys/entries, no-follow ancestors and files, exclusive creation, fingerprints, redacted errors. Actual create/verify executed. |
| `scripts/test_local_operations.py` | Rust test `local_operations` plus shared process/server tests | Real WAL checkpoint/restore and deadline fixtures, report redaction/atomic write, typed private exit retention. Shared owned-process lifecycle cases remain in foundation/Server tests; complete local orchestration parity still needs review. |
| `scripts/test_qualify_multi_instance.py` | Rust test `qualify_multi_instance` | Source/topology/redaction/cleanup cases and finalization cancellation/report-write regressions. Cancellation fixtures are deterministic state transitions, not new Docker/OS signal runtime proof. |
| `scripts/test_release_checksums.py` | Rust test `release_checksums` | Sixteen original security/schema/CLI families, plus private unit tests for file replacement, symlink replacement and mutation races. |

All tool dependencies stay in the unpublished isolated `xtask` workspace. Added
`serde` is already locked; SQLite's existing dependency enables its online-backup
feature. Root Cargo manifests/lock/toolchain and product dependency selection are
unchanged. Changes in YT-085 helper modules expose existing internal Board/scene
helpers only; no accepted scenario is replayed or marked undone.

## Invocation adoption

The isolated `qualification/yt066b/tests/cookie_fencing.rs` now launches the
prebuilt xtask directly with `fixture redis-cookie-browser BASE`, retaining
provider/server shutdown and replacement-session assertions. `YETLET_XTASK_BIN`
may select an absolute custom-build executable. There is no recursive Cargo test
and no Python fallback.

The three workflow files now call Rust implementations. Rust baseline retains
its MSRV/native/browser/security gates and adds the isolated tool tests/fmt/strict
Clippy. Scheduled qualification retains the OS matrix, browser prerequisite
failure, shell pipefail, concurrency, timeouts and artifact policies. Private
rebuild removes inline Python and bootstraps xtask into a runner-temp target
before its clean-source check. Its Rust body validates the immutable Git tree
and exact eighteen license aliases before GNU tar extraction, materializes
licenses, hashes selected inputs, uses pinned GNU 1.99 tools and fresh Cargo
home/target, locked fetch/offline package, rechecks inputs, requires exactly nine
archives and emits unsigned provenance/checksums. Manual-only dispatch,
read-only permission, immutable checkout, full-SHA actions, no persisted token,
25-minute bound and seven-day retention remain. No hosted workflow was triggered.

Live developer commands in root/sample/qualification READMEs and applicable
HTTP/provider/qualification/release guides now name the candidate Rust commands.
Historical verification files and receipts retain their original commands and
claims. Commands intentionally retain prebuilt-binary/browser/asset prerequisites.
Fresh qualification evidence under `target/m12/yt086/` protects old receipts.

## Validation and bounded runtime

Evidence lives at `target/m12/yt086/job/`. Exact final command results are recorded
below after verification. Earlier focused checks passed 16 checksum, 6 measurement,
4 local-operation and 9 Docker-helper cases; later finalization/race tests expand
that set. These counts are not a claim that every original orchestration assertion
has independently passed parity review.

A controlled PATH prepends failing `python`, `python3`, versioned Python, `node`
and `nodejs` sentinels before Cargo and system tools. Actual native sample DX and
checksum create/verify exited 0 with an empty interpreter invocation log. The
local qualification attempt exited 1, recorded zero owned processes and successful
private-directory removal, and remains a failed result. Help/preflight/source
inspection cannot substitute for executing unavailable browser/Docker branches;
complete runtime invocation-graph closure is not claimed.

The checksum fixture and SQLite online-backup tests are actual bounded filesystem
runtime scenarios. They do not prove application-server, GPU or distributed
qualification. Full Board load, physical IME/spoken AT, Windows persistent
Service, external Redis, private hosted rebuild and release activation remain
Not run or Not verified. Existing accepted YT-085 GPU-unavailable and default
Firefox fallback results retain their old source identities.

## Counters, limitations and next gate

The initial compile required closure type annotations; the second source batch
required imports, SQLite integer conversion and owned thread captures. The third
repair addressed strict Clippy findings and protected old report destinations.
The final bounded repair addresses observed SSE readiness, reporting/finalization,
source identity and file-race coverage gaps. Product repair budget is **4/4**;
no further implementation repair is authorized in this Job. Test-fixture macro
syntax and a report-error fixture path were corrected in place as separate
harness failures; neither was called a product regression or a passing test.
Operational retries remain bounded to two; final exact usage and signatures are
recorded with closeout evidence.

Known acceptance limits include unexecuted browser/Docker/private-rebuild paths,
local orchestration parity pending independent inspection, and Linux-qualified
checksum file handling. Gzip uses the isolated Rust compression backend; emitted
compressed byte counts describe that backend and are not historical Python/zlib
byte-parity evidence. No new framework architecture or public CLI is introduced.
YT-086 remains In progress. The next unblockable backlog gate is a separate
read-only Reviewer Job over this candidate and the explicitly incomplete evidence;
that Job must not mark Done without resolving material gaps and its own PASS.

## Final observed results and stop boundary

| Command / observation | Exact result |
|---|---|
| `cargo build --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; `job/build.log` |
| `cargo check --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; `job/check-final.log` |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Exit 0; `job/clippy-final.log` |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | Exit 101; 103 passed, four failed, two pre-existing child entry points ignored in enumeration. Four existing `server_status` tests fail at loopback allocation with `PermissionDenied`; subsequent smoke integration binaries/doctests are not reached. This is not a full-suite PASS. |
| New tool tests within that run | `local_operations`: 4/4; `measurements`: 6/6; `qualify_multi_instance`: 18/18; `release_checksums`: 16/16; checksum race unit tests: 3/3. Cancellation cases test state/finalization seams, not a real Docker/signal run. |
| `cargo check --offline --locked --manifest-path qualification/yt066b/Cargo.toml --tests` | Exit 0; `job/qualification-check.log`; no Redis fixture executed. |
| `cargo fmt --manifest-path xtask/Cargo.toml --check`; `cargo fmt --all --check` | Both exit 0. |
| `cargo fmt --manifest-path qualification/yt066b/Cargo.toml --check` | Exit 1; wrapping of the newly changed caller remains required. No additional repair after the 4/4 bound. |
| `cargo xtask check docs`; `cargo xtask check samples` (offline locked bootstrap) | Both exit 0; docs reports 75 required documents/86 backlog items and catalog reports 12 samples. Structural checker success does not validate every command example. |
| `git diff --check` | Exit 0 for tracked changes. |
| Controlled-PATH sample DX and checksum create/verify | Each exit 0; actual local source/filesystem operations; interpreter sentinel log is zero bytes. |
| Local operations diagnostic at final source | Exit 1; `local-operations-evidence-hoW8Rw/local-operations.json` explicitly records `loopback socket unavailable: PermissionDenied`, zero owned processes and removed private directory. First failed attempt remains at `local-operations-evidence-HaZ5gg/`. No socket permission bypass or further runtime retry. |
| Static active invocation search | No Python/Node invocation match in the three workflows or qualification Rust sources; `job/active-graph-search.log` empty, `job/invocation-edges.txt` records the 39 inspected new command/caller edges. Runtime branches that could not execute remain unproved. |
| Preservation | All 52 original oracle hashes pass; root Cargo manifest/lock/toolchain hashes unchanged. Accepted YT-085 final Reviewer hash still matches. No staged diff, commit, push, deployment, hosted trigger, clone or worktree. |
| Cleanup | No suite-owned server/browser/driver launched; both local attempts report zero owned processes and removed private directories. Process tests completed/reaped their owned fixtures. Final process-name inspection found no task application/xtask/browser process. No known owned resource leak. |

**Known material incompleteness:** The isolated caller's formatting fails. The
newly adopted checksum examples in `docs/RELEASE-SUPPLY-CHAIN.md:82` and `:83`
incorrectly retain ledger placeholders before the concrete create/verify
arguments; those examples need correction in a separately admitted followup.
The render seed parser currently narrows the original unrestricted integer
option to nonnegative 32-bit input. Dynamic input hashing and Board's final
source-unchanged assertion need parity inspection against the complete original
selected product sources. Traced Board cleanup is not runtime-proven. The local
and Docker orchestration test mappings do not yet establish every original
lifecycle/signal assertion merely by matching test-family names. None of these
is accepted as a complete mapping by this Job.

The byte hash of `.git/index` changed from
`5bf0b163b99b4c63157f9457feece754119c65c4db23257325eee44b22769ae4` to
`1f346bf13ce867ead5e775669a6a9d761cd5439232a30d5000fbbd78022fa4d7`.
No stage/index-writing command was issued; the final staged diff is empty and
index contents still match HEAD. The origin of byte-level index drift is not
established, so byte-for-byte index preservation is not claimed or restored.
Later observations use `GIT_OPTIONAL_LOCKS=0`.

Conservative operational accounting is **3 corrections/reruns against a limit of
2**: two read-only discovery corrections (incorrect source paths and an unmatched
shell glob), plus the diagnostic runtime rerun. This exceeded the requested
limit by one; it is reported as a process failure, not reset or hidden. The
product repair count remains **4/4**. The final operational failure signature is
`loopback socket unavailable: PermissionDenied`; final unmet repository checks
also include isolated caller formatting and the documented command/parity gaps.
Work stops with a blocked/partial candidate, not an acceptance recommendation.
The exact 58 changed files and machine-readable result are retained under
`job/changed-files.json` and `job/result.json`.

## Owner-authorized bounded correction after independent FAIL

The independent read-only Reviewer Job `wc_job_9ec58HyahCCkPjcO` returned
**FAIL with twelve P2 findings**. That FAIL and the preceding terminal accounting
above remain historical. A new explicit owner instruction admitted only these
twelve corrections in the same checkout at
`7cfac613f908c5c19fa581438288386809870b56`. This is a role-directed execution,
not native managed dispatch, and it does not constitute the subsequent review.
YT-086 remains **In progress**. YT-085's separate PASS is unchanged.

Fresh evidence is under `target/m12/yt086/correction-20261015-cXpaJW/`;
`result.json` records command results and the exact 21 correction paths. None of
the earlier `job/`, `host-validation/` or runtime receipts was overwritten.

| Finding | Bounded correction and observed evidence |
|---|---|
| 1 — checksum overflow | Validate the per-file limit before accumulation. The two-entry `1, u64::MAX` fixture rejects both in-process and through the executable with redacted exit 1 and no panic. |
| 2 — selected source identities | Board retains the original Git-selected Rust/manifests, root lock/toolchain/helper and non-Markdown application inputs, plus tool inputs. Dynamic retains scene JS/Python/manifests/lock/source, binary, built assets and root lock, plus tool inputs. Both regenerate the same inventory at completion. Temporary-repository application/lock/style mutation fixtures detect drift; no application source was mutated in this checkout. |
| 3 — local ownership and receipts | Split the existing Server spawn/readiness boundary without changing its ready-start behavior. Register each successfully spawned handle before readiness, retain failed readiness, and allocate a fresh numbered attempt directory before spawn. Seven actual fixture launches on the same deliberately non-listening port retain seven distinct native exit records, including the corrupt-start equivalent. Final report counts are derived from all owned handles and collected receipts. |
| 4 — CI artifact paths | Rust failure uploads select the three current Chrome suite directories below `target/m12/yt085/`; scheduled qualification selects the current forms suite's sanitized `forms-accessibility-firefox.json` only, alongside its existing tee logs. Source-to-upload audit is in `artifact-path-audit.log`. Failure/always conditions and seven-day retention are preserved. No hosted run occurred. |
| 5 — Redis persistence control | Query `redis-cli --raw CONFIG GET appendonly` and `save` before emitting `redis_persistence=none`. Focused command-response fixtures reject enabled AOF and nonempty snapshot schedules. No Redis service was launched. |
| 6 — dynamic failure classification | Hard adapter/pixel/state errors remain `failed`, even with omitted lanes. Only deadline/cutoff outcomes can become `bounded_partial`; cleanup/input-integrity failures remain failed. Focused fixtures cover these distinctions without GPU execution. |
| 7 — Board tracing | Restore application PID, tracer PID, trace filename and port for each traced ready invocation; retain phase `started_unix`/`finished_unix`, including warmup. The bounded trace metadata fixture passes. No traced load was run. |
| 8 — direct lifecycle mappings | Runtime local/Docker paths and tests now use the same scenario-to-cleanup-to-report orchestration boundaries. Local cases launch/reap actual Rust child fixtures and exercise readiness failure, timeout/cancellation, spawn failure redaction, all-cleanup after receipt failure, count/receipt retention and directory removal before reporting. Docker cases inject command responses through the existing exact-owner cleanup function, remove actual work directories and drive real SIGTERM/SIGINT in isolated test subprocesses at start, transition, cleanup and report-write stages. Repeated signals, failed rewrite and actual work-removal failure cannot yield PASS. These are direct fixture results, not actual Docker qualification. Existing shared escalation/descendant tests are reused. |
| 9 — render seed | Keep signed arbitrary-width decimal integers as normalized JS literals, including signs, leading zeros, underscore separators and Unicode decimal digits. Number conversion and the benchmark's downstream u32 conversion still occur in the browser. Tests include `-1`, `4294967296`, above-exact-Number and 400-digit seeds; existing row/update/round bounds remain. |
| 10 — checksum examples | Remove duplicate ledger placeholders; examples use `cargo xtask release checksums create DIRECTORY` and `verify DIRECTORY`. Actual fresh checksum create/verify also pass under interpreter sentinels. |
| 11 — provenance and destinations | Restore original commands describing historical execution in Board/scene READMEs and HTTP pipeline documentation. Current Rust commands, their separate acceptance state and fresh M12 output directories are identified explicitly. Provider documentation separates current local/Docker destinations from historical Python receipts. All earlier observations remain attributed to their original runs. |
| 12 — isolated caller format | Apply the caller's wrapping correction. `cargo fmt --manifest-path qualification/yt066b/Cargo.toml --check` exits 0. |

### Correction validation and explicit limits

| Command | Result and retained log |
|---|---|
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | Exit 0; `xtask-fmt.log`. |
| `cargo fmt --all --check` | Exit 0; `root-fmt.log`. |
| `cargo fmt --manifest-path qualification/yt066b/Cargo.toml --check` | Exit 0; `qualification-fmt.log`. |
| `cargo check --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; `check-final.log`. |
| `cargo check --offline --locked --manifest-path qualification/yt066b/Cargo.toml --tests` | Exit 0; `qualification-check.log`; no external fixture executed. |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Exit 0; `clippy.log`. |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml --test measurements --test local_operations --test qualify_multi_instance --test release_checksums` | Exit 0; **57 passed**: 9 measurement, 10 local, 21 Docker helper/orchestration and 17 checksum cases; `focused-tests-repair-2.log`. |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | Exit 101; **117 passed, four failed, two existing ignored entries** before stopping at `server_status`. The new trace metadata unit test passes in this run. Four existing tests fail at loopback allocation with `PermissionDenied`; subsequent integration binaries/doctests are not reached. `xtask-tests.log`; no full-suite PASS or permission workaround. |
| `CARGO_NET_OFFLINE=true cargo xtask check docs` | Exit 0; 75 required documents, 86 backlog items; `docs.log`. |
| `CARGO_NET_OFFLINE=true cargo xtask check samples` | Exit 0; 12 samples; `samples.log`. |
| Controlled PATH: `cargo xtask measure sample-dx --out ...`; `cargo xtask release checksums create ...`; `verify ...` | All exit 0 at fresh destinations; `sentinel-*.log`. Failing Python/Node sentinels record zero invocations. This covers these executed source/filesystem branches only. |
| Preservation and whitespace | Root manifest/lock/toolchain and accepted YT-085 Reviewer SHA-256 match; all 52 original Python files remain identical to HEAD; staged diff remains empty; `git diff --check` passes. |

Two implementation corrections were required: local scenario extraction first
shadowed a function name, then used overlapping report borrows. Both failed
compiler logs are retained; the correction's product repair counter is **2/2**.
The earlier Job's **4/4** repairs and operational overrun remain unchanged.
Conservative operational accounting records **six inspection/edit-tool errors**
(one connector argument shape, two unmatched shell globs, one missing source
path and two rejected patch contexts). Their corrections exceed the role's
default operational limit of two. This is a process-bound deviation, not a
successful retry policy or a reset of the earlier counters. No further product
repair, loopback retry, install or capability workaround is performed.

The final validation blocker is `loopback socket unavailable: PermissionDenied`.
No GPU measurement, Docker/Redis integration, private rebuild, hosted CI,
long-load qualification, physical IME/AT, release, commit, push or deployment is
claimed. Owned fixture children were reaped; successful fixtures removed their
private directories, and deliberately failed removal fixtures were contained in
test-owned temporary parents. No suite-owned service/browser was started.
The next unblockable gate is a **subsequent independent read-only Reviewer**
over these corrections and limits. Only its PASS can permit YT-086 Done;
YT-087 stays Deferred and YT-020 Blocked.

## Owner-authorized repair of the two remaining P2 findings

The fresh independent read-only Reviewer Job `wc_job_uEQye7iAKqzfoe59`
resolved ten of the previous twelve P2 findings and retained exactly two
blockers. The owner explicitly authorized only these two repairs in the existing
checkout at `7cfac613f908c5c19fa581438288386809870b56`. The ten accepted
corrections and YT-085 acceptance remain unchanged. Previous failures and repair
and operational accounting above remain historical; this is a separately
authorized bounded repair, not a new acceptance review. **YT-086 remains In
progress.**

1. **Corrupt-start production decision:** `qualify/local.rs` now calls
   `verify_corrupt_start`, which requires failed readiness and consumes recursive
   per-attempt receipts through the existing `expected_startup_failure` and
   `exit_receipts` helpers. The immediate-directory JSON precondition is removed.
   A filesystem-only fixture calls this exact production decision with
   `corrupt-start/attempt-0007/server-0.exit.json`: native exit 23 passes;
   successful startup, missing receipt, zero/unknown/invalid exit code, incomplete
   cleanup and malformed JSON cannot pass. The existing seven-child fixture now
   also uses the nested seventh-attempt layout and the production decision.
2. **First cancellation identity:** the Unix handler adds an atomic first-signal
   record while retaining the existing cancellation token and ownership behavior.
   Docker orchestration carries that identity into the existing report boundary,
   including cancellation during report writing: SIGTERM reports `signal=15` and
   `failure_type=SIGTERM`; SIGINT reports `signal=2` and
   `failure_type=KeyboardInterrupt`, as in the original Python oracle. Later
   mixed signals cannot replace the first identity. Existing isolated subprocess
   fixtures assert both in-memory and persisted report fields, nonpassing status,
   completed exact-owner cleanup and work removal. Added report-write cases cover
   SIGINT and both mixed orders; failed rewrite still removes provisional output.

Only seven source/documentation paths changed in this repair:
`xtask/src/qualify/local.rs`, `xtask/src/qualify/docker.rs`,
`xtask/src/process.rs`, `xtask/src/process/unix.rs`,
`xtask/tests/local_operations.rs`, `xtask/tests/qualify_multi_instance.rs` and
this verification record. The shared finalizer's cancellation callback can now
attach identity to the report at each existing cancellation check; local-only
callers retain their boolean cancellation behavior.

Evidence is retained separately under
`target/m12/yt086/two-p2-20261015-18y5gdp8/`, including pre-edit copies, a
567-file input hash inventory and command results in `validation.json`.

| Command | Observed result |
|---|---|
| `cargo check --offline --locked --manifest-path xtask/Cargo.toml --all-targets` | Exit 0; `check.log`. |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml --test local_operations --test qualify_multi_instance` | Exit 0; 34 passed (11 local, 23 Docker helper/orchestration), zero failed; `focused-tests.log`. |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Initial exit 101 for an unused `Ordering` import after the callback change; the one-line import removal passes with exit 0. Both `clippy.log` and `clippy-final.log` are retained. |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | Exit 0; `fmt.log`. |

Final documentation, whitespace and preservation checks are recorded alongside
these results in `validation.json`. One read-only inspection included a nonexistent
`xtask/tests/process_service.rs` path; the actual test inventory was available and
no retry or resource operation followed that diagnostic. No earlier receipt was
overwritten. An optional per-file whitespace wrapper treated `git diff
--no-index --check`'s difference exit 1 with empty diagnostics as failure. Its one
bounded harness correction using `--no-exit-code` returned the same signature;
both attempts are retained and that wrapper was stopped without a PASS claim or
further retry. The required ordinary `git diff --check` passed. Product tests
were not replayed for this reporting-only failure.

These fixtures do not execute the full application or Docker qualification.
The saved full-suite result in `correction-20261015-cXpaJW/xtask-tests.log`
remains 117 passed, four existing `server_status` failures at loopback allocation
with `PermissionDenied`, and two ignored entries; later integration
binaries/doctests were not reached. That suite was not retried. Browser/GPU,
Docker/Redis, full load, hosted CI and private rebuild execution evidence is
reused with its existing limits. Windows console cancellation retains its
SIGINT-compatible mapping but was not executed in this Linux repair. No install,
public CLI activation, worktree, commit, push or deployment occurred. The next
unblockable gate is an independent read-only review of these two repairs;
only its PASS can permit YT-086 Done. YT-087 remains Deferred and YT-020 Blocked.
