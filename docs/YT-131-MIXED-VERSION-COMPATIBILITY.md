# YT-131 — Starter v1 to v2 compatibility

Date: 2026-10-23. Status: **Done at bounded local C1 scope after independent Reviewer PASS and controller acceptance**. Original Phase 2 C1 follows accepted
[B2](YT-130-BACKUP-RESTORE.md). This is an application-owned status-filter view
and forward migration, with explicit refusal and recovery evidence. It does not
qualify rolling deployment or zero downtime.

## Declared change and boundaries

Before implementation, `observations/c1/scenario-before-implementation.md`
recorded the following choice. V2 adds bearer-protected
`GET /api/v2/issues?status=open|in_progress|closed`, returning the existing Issue
array, newest first, at most 100 matching rows. Missing, duplicate, unknown or
invalid query fields return bounded 400 errors. The separate route returns 404
on v1, so a new caller cannot mistake an ignored filter for a filtered result.
Existing `/api/issues` routes, form fields, statuses, issue JSON, conflict rules,
`Issue`, `Store::open`, `Store::list` and `web::app` remain compatible.
`Store::list_by_status` is additive. Filtering occurs before the row limit.

Schema 2 adds `issues_status_id ON issues(status, id DESC)` using app-owned
`0002_status_index.sql`. Historical `0001_init.sql`, all table constraints and
all issue data remain unchanged. Startup takes an immediate SQLite transaction,
verifies the historical table before upgrading schema 1, creates the index and
sets `user_version=2` atomically. New empty databases apply both migrations.
Schema-2 startup/readiness checks the exact historical table and index DDL.
Unsupported versions or malformed schemas are refused; a failed migration
does not advance the version or rewrite data. There is no down migration.

The existing native Axum/Tower authorization, no-Origin API policy, anonymous
Memory Session/CSRF rules, A1 admission/deadlines and A2 collection are retained.
No Core, browser, dependency, configuration, framework API, global registry or
security/data-policy change is introduced. This index supports the concrete new
read path; no throughput improvement is claimed without measurement.

## Version identities and reproduction

Branch `docs/yt113-123-opt-in-capability-plan`, HEAD
`5db8b8c61b6d319a641814dff4af493ce3e5d965`; the checkout already contained accepted
T0–T4 changes. HEAD alone does not identify either binary. Before editing, the
accepted A2/B1/B2 v1 executable bytes were copied, and 988 current tracked and
nonignored untracked files were archived without switching checkout or rebuilding
v1. `v1-source-manifest.json` contains individual SHA-256 values;
`v1-identity.json` records dirty state and the source archive hash.

| Retained artifact | SHA-256 |
|---|---|
| `starter-v1` | `4dc7dec913722dbcf86a8677790e69d20fa27a3676e855e31fd9f238a40fcb29` |
| `v1-source.tar.gz` | `bf384f6b96002016b5a3097cf7d428f8abd8926ce85eb09820f6bd0d166f1d38` |
| `starter-v2` | `fab124a026258d3a273b862ff01d9b4abb8b15e7022ecddb98dc5a08695cfde0` |

Raw artifacts live under ignored
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/c1/`.
`v2-inputs.json` records inputs before the HTTP exercise; subsequent edits were
documentation only. `final-source.json` records final owned files, runtime input
equality and executable/evidence identities. `T5-owned.patch` isolates this task
from accepted earlier changes.

`exact-matrix-command.txt` retains the complete one-shot Python standard-library
HTTP/SQLite/subprocess command, including settings, assertions and cleanup.
It is a direct exercise record, not an installed validation harness. It uses
Production settings, a random token kept in memory, metrics enabled, existing
debug binaries and synthetic data in an isolated `target/yt131-c1-*` directory.
Ports are `127.0.0.1:45211` and `:45212`. Only the deliberate overlap uses both
listeners; other phases run serially. No external service or real data is used.
Reproduction requires unused ports and fresh output snapshot destinations:
the command deliberately refuses overwriting retained SQLite snapshots.

`matrix-command.log`, `requests.json`, five process logs, `events.json`,
`matrix-result.json` and `cleanup.json` retain the actual run. Response evidence
omits session cookies and redacts CSRF tokens. `held-v1-form.html` preserves the
old form markup with its ephemeral CSRF value redacted. The first shell invocation
had a here-document redirection syntax error before Python executed anything;
one operational correction succeeded. `operational-failure.txt` retains this
failure and classification; no product repair or qualification replay occurred.

## Observed compatibility matrix

The direct run recorded 26 HTTP responses, including deliberate failures.

| Combination | Actual observation | Operational meaning |
|---|---|---|
| New binary / existing v1 data | V2 startup and readiness 200; schema 1 → 2; all three complete records and sequence 3 identical | Forward migration preserves existing data |
| Old frontend / new handlers | Held v1 cookie + CSRF form submitted to v2 returned 403 with no data change; reload issued a fresh session, the same v1 form fields then returned 303 | Process-local sessions do not survive version/process change; reload is required |
| Stale form and command client / v2 | Repeated expected version 1 returned 409; actual status/version stayed closed/2 | Stale clients cannot silently overwrite a newer update |
| Old command API client / v2 | Existing list, PATCH and POST requests worked with unchanged issue JSON; new create returned ID 4 | Existing command wire contract remains usable |
| New status route / old binary | Authenticated request returned 404 | Use v2 routing or the existing bounded list; do not assume v2 capability from assets alone |
| Already-running v1 and v2 / schema 2 | V1 readiness 503 and liveness 200; v1 legacy GET and PATCH still worked; v2 rejected the competing stale PATCH with 409 | Readiness is not a request fence. Mixed serving is unsupported; remove and drain v1 before production migration |
| Old binary started / schema 2 | Exit 1, `starter failed: database_unavailable`, no listener, all schema/data/sequence unchanged | In-place binary rollback is refused |
| V2 restart / post-upgrade data | Readiness 200, filtered query 200, all four records and sequence 4 retained | Keep schema 2 and use v2-compatible forward recovery |
| V1 / isolated pre-upgrade backup | Readiness 200 and exact original three records restored; GET ID 4 returned 404 | Backup rollback works but loses subsequent acknowledged writes |

`before-v1.json`, `after-migration.json`, `after-mixed-writes.json`,
`after-v1-refusal.json` and `rollback-v1.json` contain full issue fields, complete
schema, sequence, integrity and foreign-key results. Every snapshot had integrity
`ok` and no foreign-key violations. Before migration, IDs 1–3 were open/version 1.
After mixed writes, ID 1 was closed/version 2, ID 2 in_progress/version 2,
ID 3 open/version 1, and new ID 4 open/version 1. Titles and bodies, including
Unicode, quotes and HTML-like text, matched exactly. The rollback excludes ID 4
and both status updates. HTTP success alone is not the data evidence.

The consistent `before-v1-backup.sqlite3` and `after-v2-writes.sqlite3` snapshots
are deliberately retained as private synthetic evidence. SQLite backup ran with
the sole synthetic writer paused; no live-main-file-only copy was used.

## Upgrade, rollback and forward-fix conditions

1. Identify the exact old/new executable and source, current schema and complete
   business-data manifest. Plan a maintenance interval: remove v1 from traffic,
   close admission, drain all writers and confirm clean stop. V1's readiness
   failure after migration is insufficient to stop direct requests. Do not use
   this deliberately overlapping experiment as a rolling deployment recipe.
2. Take a supported consistent SQLite backup, retain it separately and verify
   its integrity, schema 1, expected records and sequence before migration.
   The historical [B2 procedure](YT-130-BACKUP-RESTORE.md#operator-runbook) applies
   to this v1 backup. Retain operator secrets separately. A missing path may
   initialize an empty database: readiness alone never proves recovery.
3. Start v2 against the selected existing data, require readiness, check
   `PRAGMA user_version` equals 2, exact unchanged table DDL and
   `CREATE INDEX issues_status_id ON issues(status, id DESC)`, integrity,
   foreign keys and complete business records/sequence against the pre-upgrade
   manifest. Check an existing API read and the new status-filtered route before
   admitting writes. If any check fails, stop cutover and preserve the data.
4. For current v2 backup/restore, use B2's supported backup/fresh-destination
   mechanics but select a schema-2 manifest and v2 binary. Its embedded schema-1
   assertions describe the historical v1 exercise and must not be used as a v2
   acceptance claim. Compare full source/restored schema/data/sequence, verify
   schema 2 and the index, then check readiness and business data. No B2 timing
   or deployment qualification was rerun or transferred to v2.
5. Binary-only rollback on schema 2 is refused. Before any post-backup write,
   rollback may stop v2, restore the verified schema-1 backup to a fresh path,
   start retained v1 and check the complete manifest/readiness before switching.
   Keep schema-2 data and the backup until recovery is accepted. Never force
   `user_version=1`, remove the index to bypass refusal or copy over active data.
6. Once writes occur after the snapshot, restoring it loses acknowledged work;
   an issue ID may be reused from the older sequence. Preserve the schema-2
   snapshot and use a schema-2-compatible forward fix/restart. The run verified
   that v2 restart preserves all new writes, not an arbitrary future patch.
   Any emergency return to v1 after new writes needs a separate, explicit
   loss/reconciliation decision; no automatic downgrade or loss approval exists.
   Clients must refresh state and sessions after any restored snapshot.

The Starter has no JavaScript, WASM, lazy chunks or versioned frontend assets.
Its stale-client surface here is actual retained SSR markup and API requests.
[Accepted YT-103](YT-103-ROLLING-ASSET-VERIFICATION.md) remains the unchanged
evidence for retained hashed bytes, delayed browser loading and its bounded
rolling/rollback lab. It was not rerun. Asset availability does not establish
new API capability or persistence compatibility; preserve old assets under that
deployment's retention policy and choose API routing separately.

## Verification, cleanup and delivery

Commands from repository root, each exit 0:

```bash
cargo test --locked --offline --manifest-path examples/production-starter/Cargo.toml
cargo build --locked --offline --manifest-path examples/production-starter/Cargo.toml
cargo clippy --locked --offline --all-targets --manifest-path examples/production-starter/Cargo.toml -- -D warnings
cargo fmt --manifest-path examples/production-starter/Cargo.toml --check
cargo xtask check docs
git diff --check
```

Product tests: **13 unit and 12 HTTP passed**, including three new tests for
migration preservation/filter-before-limit, failed migration/index refusal and
v1/v2 wire/security compatibility. Existing tracing fixture and A1/A2 tests
passed unchanged. `product-tests.log`, `build.log`, `clippy.log`, `fmt.log`,
`docs-check.log` and `whitespace-check.log` retain results. HTTP tests cover
existing CSP/escaping; no browser, Windows, asset, capacity/soak, B2 recovery
timing, power-loss or release/deployment qualification was rerun.

All five started processes were reaped: four clean exits and the intentional
old-binary refusal exit 1. Both listeners closed; HTTP/SQLite connections and
log handles closed; the temporary directory was deleted. Only the explicitly
retained synthetic evidence snapshots remain. No commit, push, branch/worktree,
deployment, signing, publication or paid resource was used.

Changed files: Starter `src/store.rs`, `src/web.rs`,
`migrations/0002_status_index.sql`, `tests/http.rs`, `README.md`; this record;
the YT-131 entries in Backlog/Roadmap/Phase 2; and a minimal historical-v1 label
in the B2 runbook. Stale B2 acceptance text was reconciled to the independent
PASS already supplied by the controller. No new major decision remains open.
Independent Reviewer PASS has zero issues and required followups; controller
acceptance completes YT-131 at bounded local C1 scope. Records:
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T5/independent-review.json`
and `observations/T5/review-acceptance.json` under the same run root.
The [final five-capability matrix](PRODUCTION-READINESS-PHASE2.md#final-production-readiness-matrix)
consolidates accepted evidence; the next remaining decision is **YT-096 D1–D6**.
YT-096 remains In progress with D1–D6 Pending, YT-020 Blocked, and candidate
`ce56e08` unchanged.
