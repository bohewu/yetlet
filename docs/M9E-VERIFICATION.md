# M9e Verification — SQL-first SQLite CRUD

**Date:** 2026-10-09
**Task:** YT-054 / M9e
**Status:** Done. First reviewer P2 repaired, complete replacement validation PASS, repeat reviewer `wc_job_aVBprnEsYc8o3ubi` PASS and exact catalog entry promoted.

## Scope

`yetlet-sqlite-crud` is an API-only, loopback-only application using direct parameterized SQL through application-owned `rusqlite`. It owns one SQL migration, optimistic versions, explicit immediate transactions, an audit table and restart-persistent local state. It adds no Yetlet ORM, common data layer, repository abstraction, browser runtime, Session or framework database provider.

The dependency decision is documented in [M9e data-access comparison](M9E-DATA-ACCESS-COMPARISON.md). Synchronous SQLite calls run only inside a four-operation bounded `spawn_blocking` boundary. Startup migration is explicit and never runs per request.

## Acceptance commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo build --locked -p yetlet-sqlite-crud
cargo build --release --locked -p yetlet-sqlite-crud
python3 scripts/smoke_sqlite_crud.py
cargo tree --locked -p yetlet-sqlite-crud --edges normal
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Required evidence includes migration/user-version behavior, parameter binding, unique constraints, typed row mapping, committed restart persistence, deterministic temporary cleanup, explicit rollback after a failure occurring after a prior write, one-winner optimistic concurrency, native JSON/status behavior and absence of database/ORM dependencies from every unrelated server/browser package. Browser production assets must remain byte-identical or any difference must be explained.

## Targeted evidence

- The bounded dependency spike selected `rusqlite 0.40.2` over `sqlx 0.8.6` for this sample: 12 versus 117 resolved packages and 2,600,400 versus 4,365,424 raw release bytes in equivalent tiny fixtures. Clean offline rebuilds took 27.770 s and 30.460 s on this host; times are informational only.
- `yetlet-sqlite-crud` targeted format, strict Clippy, build and **9/9 tests** PASS. Tests cover migration/user-version and incomplete-schema rejection, Unix owner-only database creation, direct parameter binding, SQLite `NOCASE` uniqueness, explicit audit rollback after a prior update, stale/conflicting writes with exactly one concurrent winner, reopen persistence, API policy/statuses and loopback-only admission.
- A real localhost smoke starts and stops three separate processes against one temporary SQLite file. Create/update survives the first restart; two concurrent version-2 updates produce one 200 and one 409; versioned deletion survives a second restart. Cross-origin mutation is rejected and all temporary state/processes are cleaned.
- Startup owns the migration and WAL setup; request operations open one connection inside a four-permit `spawn_blocking` boundary. No database initialization is hidden per request and no async executor thread performs SQLite work.

## Full validation freeze

The main full validation run completed format, workspace strict Clippy, native tests/doctests, SQLite debug/release build, the three-process restart smoke, six-package wasm32 checks and production Browser/Search release rebuild. Its only failure was the virtual catalog documentation contract: the new README originally wrote HTTP method and path inside one code token, so exact `/healthz` and `/api/notes/{id}` tokens were not discoverable. The README was corrected without runtime changes, then the catalog/docs recovery and all remaining dependency, asset and Git gates passed.

| Check | Observed result |
|---|---|
| SQLite CRUD focused tests | 9/9 PASS |
| Workspace native tests | 153/153 PASS |
| Workspace doctests | 5/5 PASS |
| Workspace format and strict Clippy | PASS |
| SQLite debug/release build | PASS; repaired release binary 4,676,656 raw bytes |
| Three-process localhost persistence/concurrency smoke | PASS; all owned processes and temporary DB cleaned |
| Existing six WASM packages | PASS on `wasm32-unknown-unknown` |
| Production Browser/Search release rebuild | PASS; digest `faebb0709dccd863` |
| Retained production assets | 136/136 byte-identical to the pre-build snapshot |
| Documentation/backlog | 46 required documents / 59 task IDs, PASS |
| Existing sample catalog | Seven verified entries and 4/4 checker tests PASS |
| Candidate SQLite catalog | Exact virtual eight-entry catalog PASS; real promotion awaits review |
| Dependency isolation | 22 workspace package normal trees checked; `rusqlite`/`libsqlite3-sys` occur only in `yetlet-sqlite-crud`; no SQLx/Diesel/SeaORM |
| Git/worktree hygiene | Original master HEAD, one worktree, empty index, no later M9 sample, untracked whitespace PASS |

Retained validation logs, source hashes, dependency trees, exact candidate catalog entry and summaries are under Git-ignored `target/m9e-evidence/`. After the first reviewer finding changed native sample source, the **entire** format/Clippy/153-test/5-doctest/release/smoke/wasm/browser-release/docs/catalog/dependency/asset/Git matrix was rerun and passed. Browser E2E and renderer benchmarks were not repeated because no browser/renderer source changed; wasm32 checks, release rebuild and exact assets provide the bounded M9e isolation evidence. Complete cross-browser regression remains M9j.

## First independent reviewer finding and repair

Independent read-only reviewer Job `wc_job_Q3pBfUqPYGYNSIrD` returned one material P2. The first `verify_schema` revision counted only the `notes` and `note_audit` table names. A database still marked version 1 could rename `notes.body`, pass startup and `/healthz`, and fail later CRUD with an internal SQLite error. This violated the explicit incomplete-schema fail-closed acceptance.

The repair now compares the exact ordered required column names for both v1 tables through SQLite `pragma_table_info`: `notes(id,title,body,version,created_at,updated_at)` and `note_audit(id,note_id,action,note_version,title,created_at)`. Tests retain the missing-table case and add two version-1 fixtures that preserve both table names while renaming one required column in each table; `Database::open` rejects both with `SchemaMismatch`. Focused strict Clippy, 9/9 tests, build and real restart smoke pass after the repair. Replacement full validation then passed 153 workspace tests, five doctests, all 22 package dependency trees, six WASM packages, Browser/Search release rebuild and 136 byte-identical assets. Repeat reviewer `wc_job_aVBprnEsYc8o3ubi` independently confirmed the repair with no issues or followups.

## Reviewer gate

The first role-directed read-only reviewer Job `wc_job_Q3pBfUqPYGYNSIrD` completed at terminal exit 0 but returned `overall_status=fail` with the required-column P2 above. Runtime telemetry reported `gpt-6-astra`, high effort and a read-only sandbox. It did not modify the workspace.

After repair and complete replacement validation, a new role-directed read-only reviewer Job `wc_job_aVBprnEsYc8o3ubi` completed terminal exit 0 and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

Telemetry again reported `gpt-6-astra`, high effort and read-only. Both reviews were ordinary local `codex exec <reviewer prompt>` Jobs, **not** `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. All 151 reviewed source hashes remained unchanged during the repeat review. Only after PASS was the exact independently reviewed candidate stanza promoted to `examples/samples.toml`; runtime source was not changed. YT-054 is locally Done and YT-055 remains Todo.
