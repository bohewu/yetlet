# YT-247 — Supplemental Windows Verification

Date: 2026-11-10 (Asia/Taipei). **Windows execution completed; independent delta Reviewer PASS.**
The Owner requested Windows verification before moving to YT-248 and then
authorized cleanup of task-generated Rust test files/cache. This supplements the
[Phase B implementation record](YT-247-SDK-VERSION-RESOLUTION-IMPLEMENTATION.md)
and [accepted Contract](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md). It does not
start YT-248 or publish a CLI.

## Exact source and existing prerequisites

Production source is PR #69 merge
`9ec1ae69977f104fb3d3e1fe1ee730e005363057`. A Git archive containing the independent
CLI and existing artifact helper has SHA256
`030dfb30a055de3fb0b679ef7f6126185768b8460ed24a9a53302fe56c83682c`.
The receiving Runner verified that digest and recorded all 161 CLI input hashes.
All production inputs remain identical; one existing Windows test assertion is
corrected as described below. Its resulting SHA256 is
`81994a37c4305b777a77866ee261c2fe9cb3d20fd37d9a202e0ac7a8ff62d55e`,
matching the local delivery file.

The Runner hosting mint-game is `NCUT-BOHE-2024`, client
`ncut-bohe-c420c5de1cac4bd4`. It has Rust1.99 GNU/MinGW, without the needed GNU
OpenSSL headers/libraries or a complete MSVC C++ installation. Its old Yetlet
checkout and mint-game are preserved. The selected execution host is
`NCUT-BOHE`, client `ncut-bohe-244143825ae3488e`, Project `localtasks`, Session
`wc_sess_DLiKchgdBE4PdpK6`.

This host already has VS2022 MSVC, an OpenSSL3.5.9 static prefix and a task-owned
protected copy of installed Rust1.99.0 from the accepted YT-243 preparation.
The named/stable Rust compiler commit is
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`. The existing compiler/Cargo hashes
were checked against that preparation receipt. Only child-process environment
selects these inputs. No installation, original ACL change, global environment
change or Runner permission change is made. The earlier Phase B attempt did not
select this documented OpenSSL prefix; its prerequisite failure is retained.

## Actual checks and original failures

Evidence root: `artifacts/verification/yt247-resolver-20261110/` (ignored), with
the verified Windows package extracted under `windows-merged-evidence/`.
The original Linux **182 PASS / 8 ignored** result remains Linux evidence.
Windows-only cases were zero cases in that run.

| Windows check | Actual result |
|---|---|
| `cargo test --locked --lib --no-run` with selected MSVC/OpenSSL | PASS; CLI test code compiled and linked. Existing OpenSSL missing-PDB warnings do not prevent linking. |
| Initial `cargo test --locked --offline --lib --test windows_new --test release_interface -- --test-threads=1` | Library: **102 PASS / 6 FAIL / 3 ignored**; Cargo stopped before integrations. Original failures retained. |
| The six failing library cases selected by exact names, with a private task TEMP and a new processed-input console | **6 PASS**, 105 filtered, no ignored/failed cases. The assertion correction below is the only source delta. |
| `--test release_interface --test windows_new`, locked/offline | **2 + 3 PASS**. No zero-case Windows claim. |
| Actual binary `doctor --json` | PASS using the existing protected compiler/Cargo copy. |
| Actual binary native alpha.6, browser alpha.7, native-style alpha.7 and browser-style alpha.7 creation | PASS, online registry refresh, exact pins, SDK lock source/checksums and creation receipts. |
| Actual binary default native creation | PASS, selects reviewed alpha.7 and reports registry refresh/prerelease policy. |
| Actual binary `new --offline` after cache preparation | PASS, selects alpha.7 from `offline-cache`; proxy input points to a closed local endpoint. Fixture tests independently assert zero metadata-provider calls offline. |
| Repeated destination with the closed proxy input | Expected exit1; app manifest/lock/receipt and metadata-cache hashes unchanged. |
| Explicit alpha.7 offline with an empty Cargo home | Expected exit1, `SDK_OFFLINE_CACHE_MISS`, no published app or sibling staging residue. |

There are **108 different passing library cases across the initial and focused
corrective runs**, plus five integration cases. This is not a single fresh
all-library-green execution. The three ordinary ignored cases remain recorded;
the separate real binary creation checks exercise actual registry/Cargo behavior.

Four initial failures use a general TempDir whose inherited ACL includes
untrusted writable SIDs. The product correctly rejects it as a creation parent.
A task-private TEMP/TMP inherited from the existing protected app root supplies
the required fixture precondition without changing global TEMP or production
checks. The deadline case needs processed console input; a task-owned new console
provides it without changing the Runner's console mode.

The remaining test compared the dev command against a lower-level Cargo command
without accounting for the existing `YETLET_STYLE_DEV=1` contract. Production
already sets that flag before YT-247. The correction changes only
`src/platform/process/windows_tests.rs` to include it in expected environment;
production process behavior is unchanged.

Original setup/assertion failures remain visible: an empty RUSTC child override,
an incorrect PowerShell executable name, a Style lock assertion using `style/`
instead of actual `stylegen/`, and an evidence read assuming browser dependencies
were top-level instead of target-specific. Each correction continues the same
work/evidence root. Successful tests/apps are reused; no failed record is deleted
or converted into a passing command.

The six generated projects were independently read with standard-library TOML/
JSON parsers. Every selected SDK manifest pin is exact, browser SDK pairs match,
SDK lock checksums equal resolved metadata, all registry lock checksums remain,
the receipt lock digest matches, and both nested Style locks match the template.
These assertions, source hashes and the actual CLI binary are retained.

## Evidence, cleanup and limits

The Windows evidence ZIP is 5,494,565 bytes, SHA256
`f2ab86cee870aa3e9d0d91d9a9b3333c621969469150dda72002323e19993e74`.
Transfer to the local checkout and independent local digest verification pass.
It retains original failures, exact commands, six generated projects, metadata
snapshots and the accepted CLI executable (SHA256
`5865dd016cdeab7437e59d910561abbfb657ae4b0b0d04bda84df575be11eaae`).

After all owned children exited, Owner-authorized Windows cleanup removed:

- MSVC task cache: **1,348,294,544 logical bytes / 2,102 files**.
- Six temporary generated projects and empty offline home: **303,069 bytes / 105 files**.
- The task-private TEMP directory, empty at cleanup.
- The task-created resolver metadata namespace, after preserving matching copies.
- The earlier mint-game-host task cache: **13,368,262 bytes / 177 files**.

Cache ownership/layout markers, absence of active Rust builders, Cargo cache locks
and absence of reparse descendants were checked before deletion. The evidence
and accepted CLI binary were preserved first. All 35 retained MSVC evidence file
hashes and three original mint-host evidence hashes remain unchanged. The old
Yetlet checkout HEAD/status, mint-game, qualified tools and shared Cargo cache
remain unchanged. Cleanup receipts are transferred separately because the ZIP
predates deletion. Local Linux cleanup removes **2,892,051,762 logical bytes /
4,906 unique files** from the paired task cache, after preserving its accepted
CLI executable. All 113 retained local evidence-file hashes, HEAD and Git status
match across deletion. The three compiler caches total **4,253,714,568 logical
bytes** (about 3.96 GiB); these are logical-size observations, not a promise of
exclusive physical free space. Shared Cargo storage and the GPUI worktree remain
untouched.

WebCodex closeout observes zero active jobs, clean evidence integrity and verified
output digests. Its Session ledger still reports `fail`/`source_unproven` for
retained harness failures and the non-Git staged source. That advisory result is
not relabeled PASS. The bounded acceptance here uses actual command logs,
independent source/output hashes and the delta review; no whole-Session-green
or broader platform certification claim is made.

Independent read-only delta Reviewer `/root/yt247_resolver_review_20261110`
returns PASS with zero issues/followups. All seven reviewed file hashes remain
unchanged before these factual status annotations. Native trace matches role
`reviewer`, model `gpt-6-astra` and effective effort `high`; shared-resolver
observation is enforced without degradation. The result and bounded trace are
retained as `windows-review-result.json` and `windows-review-trace.json`:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

Guarded Git delivery and exact-head/master CI receipts follow in the same ignored
evidence root. This precommit record cannot contain its own future commit hash.

This verifies Windows MSVC CLI compilation and the changed resolver/creation/
locking path with existing console/tool prerequisites. No Windows native server,
browser GUI, Vite build, WASM runtime, browser CSP journey or GNU execution is
claimed by this supplement. YT-248 retains its separate installed-consumer/Style
runtime acceptance and remains unstarted pending Owner instruction.
