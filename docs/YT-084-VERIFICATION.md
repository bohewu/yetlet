# YT-084 — Seven Rust Builders and Documentation/Catalog Checks

**State: Done** after independent source/parity review and a separately reviewed
prerequisite-documentation correction; zero remaining issues or required followups. YT-082 and the corrected YT-083 foundation already passed
their separate independent reviews. This item does not accept YT-085/YT-086.

## Independent review and scoped adoption correction

The first independent read-only Reviewer Job `wc_job_mo3glauOcGaSnPRI`
finished with a material **FAIL**: the live root and docs quickstarts still
listed Python as a browser-build prerequisite above their Rust commands.
The parent corrected only those two sentences, keeping Python requirements
for the unmigrated YT-085/YT-086 tools explicit. No code or build changed.
The original FAIL receipt remains at `target/m12/yt084/reviewer/result.json`
(SHA-256 `d2fec2eaed5e2cfc8c5931132fd9790012184c88e484cc7b26e1b3bbc46f2b76`).

Follow-up independent Reviewer Job `wc_job_IvwN3Xp57FjpNN20` returned
**PASS**, no issues or required followups. Exact thread
`01a13e33-eef2-7970-a818-0caa958c080b` recorded `gpt-6-astra`, effective
`high`, read-only; this is a local role-directed Job, not native agent-type
dispatch. Receipt: `target/m12/yt084/reviewer-adoption-fix/result.json`,
SHA-256 `e57c11f292b9995508f542d4a5007ef01d2f1af91807762fb846fe11da3ae931`.
Both executions ended with exit 0 and unchanged review inputs. The first
review independently checked all 128 golden hashes, 524 source-after hashes,
52 original Python hashes, per-builder results and negative test receipts.
The cached yanked XML pin is an explicit limitation, not a dependency safety
endorsement; its concrete pinned-CSS input path did not establish a blocker.
No builder or browser replay was needed for the documentation-only repair.

## Scope and source identity

Work stayed in `$HOME/repos/yetlet` on the recorded Ubuntu runner, original checkout at
`7cfac613f908c5c19fa581438288386809870b56`. Existing uncommitted YT-082/YT-083
work was accepted parent-owned input. The cancellation repair in
`process::final_report` and `doctor_with` was preserved; no YT-083 reviewer
receipt or verification document changed. No worktree, commit, push, global
install/settings, hosted action, GUI/browser or load test was used.

The explicit owner authorization covers YT-084 tooling and its direct negative
fixtures; tooling is the product. Contracts are the [plan](M12-RUST-TOOLCHAIN-PLAN.md),
[exact migration ledger](M12-PYTHON-MIGRATION.md), original seven Python builders,
`check_docs.py`, `check_samples.py` and all four `test_check_samples.py` cases.
Original Python files/tests remain unchanged. All repository documentation is
English. The tool stays unpublished with an independent lock/dependency graph.

Evidence directory: `target/m12/yt084/`. `source-before.json` captures selected
tracked source/assets plus accepted xtask inputs before implementation;
`parent-before.sha256`, `head.txt` and `status-before.txt` identify the checkout.
`golden/` and `golden-sha256.json` freeze actual legacy outputs, including asset
bytes, manifests, pages and CSS source/generated records. The two task-local
Python comparison scripts are explicitly **transitional differential tests**,
not canonical commands. Rust executes no Python/Node interpreter.

## Exact output parity

Each Python entry was built independently with the selected profile, then each
Rust entry independently with the same profile and original source. No combined
Cargo build unifies the entry features. `legacy-*.log` and `rust-*.log` retain
actual commands/cwd, tool paths, statuses and digests. `parity.json` and
`parity-final.log` record the comparison. All **128/128 files match exactly**:

| Builder | Debug files | Release files | Other matched files |
|---|---:|---:|---:|
| Browser: separate Todo/Search | 10 | 10 | 0 |
| Keyed bindings: Wasm plus native SSR | 7 | 7 | 0 |
| Multi-island Dashboard: separate Counter/Filter | 8 | 8 | 0 |
| Render benchmark | 6 | 6 | 0 |
| Scene: nested scene plus separate root Counter | 0 | 8 | 0 |
| Form CSS proof | 0 | 0 | 29 |
| Board assets | 0 | 0 | 29 |

CSS counts include the four original inputs and 25 generated files per fixture;
there are 23 externalized SVG occurrences and 22 hashed outputs per fixture.
The readonly comparisons match output bytes and inventories, not a new browser
acceptance. Pure-transform Rust tests separately compare all 25 expected outputs
per fixture against tracked legacy bytes and build/check disposable fixtures.

Parity covers full JS/Wasm/CSS/SVG/page/generated Rust bytes, JSON formatting and
key order, schema/value meaning, source inventories, hash domains, sorted
filename/NUL/bytes order, full/truncated digests and asset URLs. Scene checks the
root and nested locks against the exact installed `wasm-bindgen 0.2.129` identity.
All builds retain locked Cargo execution, scene offline release, original output
roots and independent cache paths. Browser-side JS literals are frozen target
code in `xtask/src/build_templates.rs`, not an interpreter workaround. The
legacy render-benchmark inert `data:,` favicon remains unchanged, with no JS.
The generated Rust provenance comment still names `build_assets.py` solely to
preserve exact bytes; actual current instructions name the Rust builder.

## Validation and negative coverage

Commands/results and final digest identities are recorded in the completed
validation section below. The initial/final logs are retained separately.
Focused Rust tests preserve all four original sample test names and every
subcase: false command, exact route versus prefix, runtime/kind mismatch,
unknown direct dependency, traversal, boolean schema and malformed command,
kind/package arrays and package table. Additional direct fixtures cover all
field types, duplicate IDs/packages/canonical paths/companions/browser packages/
optional dependencies/routes, required/extra fields, build identity, source
root/marker safety, README exact commands, target build dependencies and symlink
escape. These fixtures never mutate the original catalog or triage state.

Docs fixtures cover required files, English prose including new Markdown,
excluded directories, links, EOF, trailing whitespace, fences, malformed/empty
backlog rows, duplicate IDs, unknown/self dependencies and cycles. CSS fixtures
cover malformed XML, disallowed tags/attributes/namespaces, active URLs,
imports/data URLs, unregistered CSS references, missing/extra/stale outputs,
source integrity, readonly checks, and unsafe input/output paths. Native tool
fixtures preserve child exit 37/cleanup status and reject a deceptive
`0.2.1299` version before building. Missing tools fail without installation.
Foundation cancellation and owned-child tests remain in the required full suite.

The two pinned HTTPS fetch URLs are implemented with no redirects, bounded
30-second acquisition and 1 MiB input size, CSS SHA-384 and license checks.
`--fetch` and `--check` are incompatible. Validation used existing cached assets;
**no network fetch was executed**. Actual online acquisition remains Not run.
No doctor path installs or downloads. Doctor retains its read-only foundation
capability; build-specific preflight executes via the same owned native runner.

## Adoption and remaining gates

After all 128 comparisons passed, current root/sample build instructions, sample
catalog, docs guide and the Todo missing-assets diagnostic switched to Rust.
The matching `rust.yml` docs/catalog checks and direct tests switch together
because the old checker cannot interpret the catalog's new Rust command
identity. This is the owner's explicitly allowed narrow early adoption;
unrelated CI, builds, smoke and release jobs remain unchanged. All whole-workflow
ledger rows remain Not migrated. Historical acceptance prose and Python sources
remain reference records with their original identity.

The independent review gate is now PASS. YT-085 is the next dependency-ready
backlog item; it was not implemented by this YT-084 delivery. YT-086
owns remaining CI/transitive callers and the final blocked-Python graph proof.
No runtime-framework API changed. Root feature graph/lock, retained M11 evidence
and all older operational gates remain at their original scope. Windows process
foundation evidence is retained from YT-083; this task does not claim a fresh
Windows builder run, browser/CSP journey, GPU qualification or load envelope.

## Completed command results and digests

| Command / evidence | Actual result |
|---|---|
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | PASS: 31 tests, zero failures; two test-only subprocess entry points ignored by discovery and invoked explicitly by their owning tests. `tests-adopted.log`. |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | PASS; `clippy-adopted.log`. |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | PASS; `fmt.log`. |
| `cargo fmt --all -- --check` | PASS; `root-fmt.log`. |
| `cargo run --offline --locked --manifest-path xtask/Cargo.toml -- check docs` | PASS: 75 required documents, 86 backlog items; English scan also includes new Markdown. `check-docs.log`. |
| `cargo run --offline --locked --manifest-path xtask/Cargo.toml -- check samples` | PASS: all 12 samples; `check-samples.log`. |
| `cargo run --offline --locked --manifest-path xtask/Cargo.toml -- doctor --json` | Exit 0, Rust/Cargo 1.99.0, both owned probes cleaned; `doctor.json`, `doctor.stderr`. |
| Seven builders / supported profiles | PASS: 11 comparisons, 128/128 exact files; `parity.json`, `parity-final.log`, individual legacy/Rust logs. |
| CSS default builds then readonly checks | Both PASS; all 58 files retain both SHA-256 and nanosecond modification times during checks. `form-build.log`, `board-build.log`, `css-before-check.json`, `css-after-check.json`, `*-readonly.log`. |
| Original Python docs/catalog/direct catalog tests before adoption | PASS, all four direct test cases; `legacy-docs.log`, `legacy-samples.log`, `legacy-test-samples.log`. |
| Original Python docs check after adoption | PASS: 75 documents, 86 backlog items; `python-docs-adopted.log`. The old catalog checker/tests were not rerun against the new command representation. |
| Root metadata / feature tree before versus after | Both byte-identical; `root-before.json`, `root-after.json`, `root-before.tree`, `root-after.tree`. |
| Root manifest/lock, process runner/platforms and original foundation test source | SHA-256 unchanged; all 52 tracked Python files unchanged. `source-before.json`, `source-after.json`, `parent-before.sha256`, `parent-after.sha256`, `isolation.json`. |
| Whitespace | `git diff --check` PASS, plus explicit changed-file EOF/trailing-whitespace checks including new files; `diff-check.log`, `changed-whitespace.log`. |
| Cleanup | Zero remaining xtask/fixture processes; no servers, browsers or watchers started. `cleanup.json`. Temporary bindgen directories and test fixtures were removed by their guards. |

| Manifest | Debug digest | Release digest |
|---|---|---|
| Browser | `5439967b104999cf` | `27597bad9c53b93e` |
| Keyed bindings | `d4be137516b1a447` | `6754025153168694` |
| Dashboard | `ee0c682c159bfee7` | `1eccb326477b59ae` |
| Render benchmark | `d2d8692a9f86fe68` | `819c4f83e8d38016` |
| Scene / Counter | Not supported | `f5584f407ee4f7b2` / `81c388455a88d98a` |

Root lock SHA-256 remains
`2e08330c49ff4e1740102a37b36fbe2fd044bd2033d8ea9a6aaf224a402866c3`.
Frozen golden inventory SHA-256 is
`f4f548e5459b01eca62bfa7ab24b04c195495b1f0aff9d08ec0649506bf501ef`;
`parity.json` SHA-256 is
`85f5ba92e1564a0cbf062fa7250c780e01d55ebd60e6d732f0a2036acd24487c`.
The full source/file digest inventories are retained rather than replacing old
M11 evidence. `changed-paths.json` lists the 45 actual YT-084 paths relative to
the accepted parent baseline: isolated xtask modules/lock/tests, current
instructions/catalog/diagnostic, narrow CI checks and migration status/evidence.

## Bounded failures and dependency limitation

Operational retries used: **1 of 2**. Offline resolution initially rejected the
only cached XML parser, `xml-rs 0.8.19`, because it is yanked. The isolated xtask
lock was seeded with the cached crate's verified SHA-256
`0fcb9cbac069e033553e8bb871be2fbdffcab578eb25bd0f7c508cedc6dcd75a`;
o source patch, install, network fetch or root-lock change was used. Locked
offline resolution and every subsequent compilation/test passed. This cached
yanked dependency is an explicit review limitation; no claim about why it was
yanked or an external dependency security audit is made. Other added dependencies
are the cached TOML, SHA-2, regex, temporary-directory, base64, percent-decoding
and HTTPS libraries needed by the approved port.

Product repair attempts used: **2 of 2**. First, required strict Clippy rejected
a complex return type and nested conditional; one bounded repair resolved both.
Second, tool discovery incorrectly canonicalized Cargo's proxy symlink into
`rustup`, losing argv-zero identity; one bounded repair preserves its filename
and adds a direct regression. The original failure retained native signal 6 and
`cleanup_ok=true`, never PASS. This repair also made missing lock package arrays
return an ordinary validation failure. Initial evidence remains in
`clippy-initial.log`, `parity.log` (the failed exit 134) and `tests.log`;
the successful per-builder logs describe the final comparison, with final
aggregate results in `parity-final.log`. No unresolved product failure remains.

Two separate harness-only corrections did not change product behavior: the new
catalog fixture now inserts an unsupported TOML key through its table API
instead of indexing a nonexistent key (`test-samples-fixture-rerun.log`), and the
source-accounting check uses the separately captured parent hashes for initially
untracked YT-082/YT-083 documents (`snapshot-final.py`, `isolation.log`). The
initial inventory covered tracked source and xtask, not those untracked parent
docs; this distinction is retained rather than silently rewriting the snapshot.
The materiality gate was applied before product repairs; no recovery dispatch,
new agent, framework, refreeze or unrelated repair occurred. Last unresolved
failure signature: empty.
