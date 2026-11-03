# Development caches and verification evidence — YT-233

## Contract

This is repository tooling, not an SDK feature or runtime service. The owner
requests cache/evidence separation, space inventory and cleanup preview, including
WebCodex cross-platform runners. The initial preview delivery did not authorize
actual deletion, Commit or Push. The subsequent owner-authorized cleanup is
recorded below; it does not grant ongoing automatic cleanup authority. The helper
has no deletion command or apply flag. Existing product
qualification is reused; only the new tooling needs focused verification.

| Location | Purpose | Retention |
|---|---|---|
| Existing workspace `target/` directories | Stable daily builds; legacy roots may also contain verification records | Inventory only; inspect contents before future authorized cleanup |
| `.cache/yetlet/builds/<run>/` | New isolated rebuildable Cargo output | Preview only; marker binds the separate evidence directory |
| `artifacts/verification/<run>/` | Commands, logs, exit status, original failures, source identities and accepted artifacts | Preserve; excluded from cleanup candidates |
| Existing external evidence stores | Historical failures, receipts, manifests, signatures and frozen inputs | Preserve in place; never treat a whole store as a cache |

Keep old evidence at its recorded paths. Write new logs outside caches from the
start. Before a future authorized cleanup, stop owned builds, inspect its exact
scope and reconcile retained evidence. A marker declares layout; it cannot prove
that every caller placed logs correctly or that a build is idle.

Daily development reuses stable caches; allocate isolated caches only when source,
feature or package qualification needs one. Independent workspaces stay separate.
Existing scripts may read hardcoded `target/` outputs: do not redirect them without
checking consumers. No global Cargo config, debug profile, incremental setting,
qualification harness or automatic retention scheduler is changed.

## Prepare a new verification run

Requires Python 3.10 or later with only its standard library. From the checkout,
choose a unique lowercase run ID:

```sh
python3 scripts/dev_artifacts.py prepare --run my-check-001
```

Expected JSON gives `cargo_target_dir` and `evidence_dir`; neither contains the
other. The command creates only their directories and paired layout records.
Existing IDs, symlinks and Windows reparse points are rejected. Filesystem errors
can leave partial empty directories, which remain protected rather than being
automatically deleted. Use a new ID instead of replacing a failed run.

For Cargo commands with configurable output paths, the corresponding POSIX
example keeps settings scoped to a subshell and logs outside the cache:

```sh
(
  export CARGO_TARGET_DIR="$PWD/.cache/yetlet/builds/my-check-001"
  evidence_dir="$PWD/artifacts/verification/my-check-001"
  cargo check --locked >"$evidence_dir/cargo-check.log" 2>&1
)
```

On Windows, use the JSON paths and restore the prior setting:

```powershell
$layout = python scripts/dev_artifacts.py prepare --run my-check-001 | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Layout preparation failed' }
$previousTarget = $env:CARGO_TARGET_DIR
try {
    $env:CARGO_TARGET_DIR = $layout.cargo_target_dir
    cargo check --locked *> (Join-Path $layout.evidence_dir 'cargo-check.log')
    if ($LASTEXITCODE -ne 0) { throw 'Cargo check failed; retain its log' }
} finally {
    $env:CARGO_TARGET_DIR = $previousTarget
}
```

These are examples, not instructions to replay an accepted check. Keep original
failures alongside subsequent corrections and preserve required frozen artifacts.

## Inventory and cleanup preview

```sh
python3 scripts/dev_artifacts.py inventory
python3 scripts/dev_artifacts.py plan --json
```

Both are read-only. `plan` presents the same inventory with proposed
classifications; neither executes cleanup. Text output is sorted by size; JSON
includes full paths, file counts, errors and totals. Discovery finds in-checkout
Cargo `target/` roots without entering generated trees. Extra known legacy
targets or evidence stores can be selected explicitly:

```sh
python3 scripts/dev_artifacts.py plan --target-dir /path/to/old/target --evidence-dir /path/to/retained/evidence --json
```

Selected roots must not overlap. Missing optional roots are omitted. Inventories
contain local host metadata and are not public documentation-site inputs.

| Classification | Meaning |
|---|---|
| `rebuildable-preview` | New cache with matching ownership/evidence markers; review before future cleanup |
| `legacy-review` | Recognized old `debug` or `release` directory; inspect for mixed evidence first |
| `preserve` | Evidence, unknown output, unrecognized markers, redirection or incomplete measurement |

A name containing `target`, `cache` or a task ID is not enough to classify an old
run as disposable. Unrecognized target triples/custom profiles stay protected.
Symlinks, Windows junctions/reparse points and redirected ancestors are not
traversed; unreadable files, skipped links and discovery errors remain explicit.
Sizes are point-in-time logical bytes, not allocated disk usage or guaranteed
reclaimable space. Hard links are deduplicated within rows, not across rows.
The preview does not lock Cargo or establish freshness during concurrent builds.

Do not use unqualified `cargo clean` on a mixed historical root: it removes the
entire target directory. See Cargo's official
[build-cache layout](https://doc.rust-lang.org/cargo/reference/build-cache.html)
and [clean behavior](https://doc.rust-lang.org/cargo/commands/cargo-clean.html).

## Verification checkpoint — 2026-11-03

Initial GNU execution: nine new focused tests PASS. The same source ran in memory
on the online Windows WebCodex runner: eight PASS, one test-only path-separator
assertion FAIL. A `Path` equality correction supplements only that affected case
on GNU and Windows, both PASS; the first failure remains recorded. Synthetic
fixtures are removed by `TemporaryDirectory`; no pre-existing source, cache or
evidence is deleted. No SDK/browser/MSVC compilation suite is replayed.

A real local layout demonstration creates a tiny new cache/evidence pair. The
local inventory records 433 entries and no discovery errors: 33,635,955,099
logical bytes of legacy profiles requiring review, 36,403,913,305 protected bytes
and 339 new-cache bytes. Root `target/debug` is 21,994,829,596 bytes,
`xtask/target/debug` 5,749,919,251 and CLI `target/debug` 4,773,751,251. These are
snapshot sizes, not safe-to-delete claims. The external evidence-store scan skips
two links and remains protected as an explicitly incomplete measurement.

The Linux runner is this local host, as the owner confirms; direct inventory
covers it even while its WebCodex connection is stale. The first online Windows
runner has no Yetlet-named entries in the selected repository, temporary and
home-cache parents; its conventional external evidence path is absent. This is
bounded observation, not a whole-disk absence claim. Prior
[qualification cleanup](ALPHA5-RELEASE-PLAN.md#actual-qualification-results--yt-212)
stays accepted without rerunning it.

After reconnection, mint-game's existing qualification checkout is inventoried
without updating its old branch or altering its existing untracked artifacts.
The same helper source measures ten entries with no discovery errors:
994,771,366 logical bytes in legacy `target/debug` and 18,918,178,464 protected
bytes, including 16,268,077,821 in the old mixed `artifacts/` tree. Its report is
appended as a new evidence file and read back with matching SHA256
`df45dd8994d21f3f90704c0539e1e4000fb9096a03bd269c026a8a708e625de0`.
An initial directory listing identifies nested historical `target`/`cargo-target`
locations beside logs and source archives. Those remain protected; no assumption
that all historical artifacts are disposable is made. A separate names-only
supplement loses the Runner connection. Its original Job reaches terminal
`lost_after_reconcile` with `command_execution_state=not_started`: the command
did not start, no output is accepted and no process cleanup is required for that
unstarted command. The failure is retained without retry or replay of the
successful main inventory. No remote cache deletion, environment setting change,
tool installation or Cargo build is performed.

Private reports, commands/results, original failure and supplemental Windows
result are retained outside the checkout under the existing evidence store's
`runs/yt233-dev-artifacts-20261103`. Initial independent implementation review is
PASS with zero issues/followups and eight unchanged source hashes. The registered
reviewer/high request is recorded, but the local trace helper cannot verify its
model or effective effort; no verified-model/effort claim is made. The first
three-document delta review also passes with unchanged implementation hashes.
Final terminal-observation delta review is PASS with zero issues/followups;
YT-233 is Done at the non-destructive tooling and scoped inventory boundary.
Successful inventories cover the local Linux host and both selected Windows
locations without claiming a whole-disk or live-cache cleanup proof. The lost
names-only supplement does not replace or invalidate those completed inventories.

## Owner-authorized cleanup — 2026-11-03

After accepting the preview, the owner explicitly requests completing actual
cleanup in the same task. Authority covers inactive, rebuildable Yetlet compiler
output on the local Linux host and the two Windows runners. It preserves source,
historical failures, frozen release inputs, final binaries, installed tools and
unknown output; it does not authorize Git operations or an automatic scheduler.

The operation selects only Cargo's internal `.fingerprint`, `deps`, `build` and
`incremental` directories underneath `debug`/`release` profiles with existing
Cargo lock markers, including nested old verification caches. It rejects
redirections, tracked files, unreadable entries and directories containing
potential embedded records or archives. It checks for active Rust builders and
holds existing profile file locks during deletion. It never removes an entire
historical `target/` or `artifacts/` root. Final binaries and their dependency
files stay in place; the next compilation may need to rebuild removed caches.

Linux cleanup removes 132 selected compiler-internal directories, representing
43,775,991,301 logical bytes. Observed filesystem free space increases by
42,378,137,600 bytes (39.47 GiB); this is a before/after filesystem observation,
not a promise of exact exclusive attribution during other host activity. The
247,167 retained files have unchanged size/mtime; bounded text records also have
matching SHA256. All tracked source hashes, HEAD and Git status stay unchanged
during cleanup. No selected directory remains, no deletion error occurs and no
active Rust builder is observed. Final source documentation updates follow the
operation and are separate from this source-preservation check.

The other Windows runner's fresh check finds no Yetlet-named entries in the
selected repository, temporary or home-cache parents, so it deletes nothing and
leaves other projects untouched. Mint-game's one cleanup Job completes with
exit0. It removes 15,970,171,122 logical bytes of selected compiler internals;
observed filesystem free space increases by 14,740,746,240 bytes (13.73 GiB).
Its 81,236 retained files have unchanged size/mtime and bounded text-record
hashes, all tracked source hashes/HEAD/Git status match, and no selected directory
or deletion error remains. Combined observed free-space increase is
57,118,883,840 bytes (53.20 GiB). No cleanup is retried.

The runner disconnects after completing the operation. A subsequent attempt to
read its full result file times out; that transport observation is retained and
does not restart deletion. The exact retained terminal Job output includes the
complete scalar result, all successful preservation assertions and reported
result SHA256 `4c159b95e2863728ee8080a8a7c9b9f5a6ae9796d303d57220821ccc275709f9`.
The bounded stdout projection truncates the start of the deleted-path array;
no independent full-file checksum readback or unobserved path-count claim is
made. Full manifests, journal and result remain at the runner's appended cleanup
evidence location. Operational independent review is PASS with zero issues or
required followups; cleanup is complete at the authorized compiler-internals scope.

Prior accepted qualification is not replayed. The private `cleanup-authorized`
subdirectory of the existing YT-233
evidence run retains the plan, preservation manifests, per-directory removal
journal and actual result. The earlier read-only acceptance and original Windows
test/transport failures remain intact.
