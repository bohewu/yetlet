# M9d Verification — Bounded File Server

**Date:** 2026-10-09
**Task:** YT-053 / M9d
**Status:** Done. Full validation and independent reviewer `wc_job_g8JSu80-Jseg6lmG` PASS; reviewed catalog entry promoted and rechecked.

## Implemented scope

`examples/file-server` is a native application with a fixed public Tower HTTP static fixture, a bearer-protected multipart upload and an opaque-ID streaming download/HEAD route. It reuses native Axum, Tokio, Tower HTTP, tempfile, getrandom and subtle in the application only. No Yetlet core/server API, ORM, storage provider, browser runtime, Session or dynamic middleware engine was introduced.

The sample contract and safety boundaries are in its [README](../examples/file-server/README.md). The canonical [M9 plan](M9-PLAN.md) acceptance covers GET/HEAD/streaming, bounded authorized upload, traversal/symlink defense, time/size/backpressure and interrupted-write cleanup.

## Evidence to retain

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo build --locked -p yetlet-file-server
python3 scripts/smoke_file_server.py
cargo tree --locked -p yetlet-file-server --edges normal
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Native focused evidence: 14/14 sample tests pass after repairing an initial false assumption about tempfile's default directory mode. The directory builder now explicitly selects 0700 at creation on Unix. Tests cover the fixed static GET/HEAD/range and SSR language, credential/origin rejection, safe names and UTF-8, file and body size limits, multipart rejection, opaque download headers/identity, store capacity, upload deadline, concurrent cancellation cleanup, download backpressure and ownership through teardown. A Unix symlink replacement regression verifies that download never reads the replacement file.

The earlier compile error passing getrandom's error directly to `io::Error::other` was fixed by converting its diagnostic to a string. RNG errors still fail closed. No test expectation was weakened to bypass the directory-permission defect.

## Observed validation freeze

Full validation Job `wc_job_bbuqpnk-erQNr_8s` reached terminal exit 0. Retained command logs, package-specific dependency trees, source SHA snapshots and the complete summary are under `target/m9d-evidence/` (Git-ignored, not public release artifacts).

| Check | Observed result |
|---|---|
| File-server tests | 14/14 PASS |
| Workspace native tests | 144/144 PASS |
| Workspace doctests | 5/5 PASS |
| Workspace format and strict Clippy | PASS |
| File-server debug and release build | PASS; release binary 2,863,128 raw bytes |
| Six existing WASM packages | PASS on wasm32-unknown-unknown |
| Production Browser/Search release rebuild | PASS; digest `faebb0709dccd863` |
| Retained dist files | All 136 byte-identical to pre-M9d snapshot |
| Docs/backlog | 44 required documents / 59 task IDs, PASS |
| Existing sample catalog / focused tests | Six entries, 4/4 tests PASS |
| Candidate catalog | Seven-entry in-memory catalog validation PASS; actual promotion awaits review |
| Git whitespace | Tracked changes and 27 untracked files PASS |

A single owned localhost server run verified static GET/HEAD/ranges, English SSR, token-gated upload/repeated download/HEAD, correct content/disposition/security headers, auth and Origin rejection, filename/extension/binary/size/path/method rejection, and a real socket disconnected mid-upload. The partially written file disappeared and a subsequent upload succeeded. SIGINT graceful shutdown reaped the exact process and removed its private temporary root. No visible browser/UI or external deployment was used.

Independent package-specific normal-dependency trees for core, server adapter, existing native samples and all six WASM entries exclude file-server-only multipart/tempfile/signal dependencies. File Server excludes Browser/WASM/keyed/reactive/Session/Request ID and DB/ORM dependencies. All pre-existing runtime/source files are SHA-identical to the start-of-turn snapshot; only the workspace membership/lock and scoped documentation/checker updates changed. This is application-level opt-in, not a new framework API.

Chrome/Firefox E2E and renderer benchmarks were not repeated in this native-only slice: no browser/renderer runtime changed, WASM checks and independently rebuilt production bytes passed. Full browser/product qualification remains the later M9j gate. The current installed Rust toolchain was used; declared MSRV, other OS/browser combinations and production security are not newly certified.

## Catalog and review gate

The sample stayed outside the verified catalog until the independent reviewer returned PASS. The reviewer inspected the candidate stanza under `target/m9d-evidence/catalog-entry.toml` and validated the virtual seven-entry catalog. That exact stanza was then promoted before Forms Validation in `examples/samples.toml`; real seven-entry catalog validation and 4/4 catalog tests passed. The original sample order and all six earlier entries were preserved. YT-054 remains Todo.

## Independent reviewer

Independent reviewer Job **`wc_job_g8JSu80-Jseg6lmG`** completed at terminal exit **0** with `overall_status=pass`, `issues=[]` and `required_followups=[]`. The complete original JSON is retained in `target/m9d-evidence/reviewer-1-result.json`. No material defects or required fixes were reported.

The fresh workspace profile was premium/openai, healthy/current/eligible/trusted, with saved reviewer model `gpt-6-astra`. The shared reasoning resolver selected `high` for ad-hoc/adaptive non-strict review. Runtime thread **`01a120ff-d3ff-70a1-b506-f06719060f9f`** independently recorded `model=gpt-6-astra`, `effort=high`, `sandbox_policy=read-only` and this exact repository cwd. Model/effort matched the saved configuration and the resolver recheck had no conflict. Bounded runtime/config/policy records are retained under `target/m9d-evidence/reviewer-1-*`.

The Runner reported `coding_agent_providers: []`, so this used ordinary **`codex exec <reviewer prompt>`**, not `codex exec review`, native ACP or native `agent_type` dispatch. Observed model/effort do not prove a native role identity, formal assurance or production security qualification.

The reviewer inspected source, negative tests, retained command evidence and dependencies; it independently verified all 143 reviewed source hashes and 136 retained asset hashes, and ran read-only documentation/catalog/whitespace checks. It made no source edits, started no applications and did not repeat the already-passing socket smoke. Source hashes were checked again before promotion. After review only catalog promotion and status/evidence documentation were changed; no runtime source changed.

**Final state:** YT-053 / M9d Done at local prototype scope. YT-054 / M9e SQL-first SQLite CRUD is the next eligible item, not started. Same original master worktree and HEAD `742e0d4909cc9b132d81b78e12bbb66d5bf60f57`; no stage, commit, push, reset or new worktree.
