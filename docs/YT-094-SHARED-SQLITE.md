# YT-094 — two app instances sharing a local SQLite database

Date: 2026-10-19. Scope: owner-authorized business-storage slice after the
[settings/SecretProvider integration](SETTINGS-SECRET-PROVIDER.md) passed
independent review. Focused native and actual Compose HTTP checks pass;
independent Reviewer PASS with zero issues/required followups. No commit or push
was performed.

## Selected integration

Two real Admin app containers use the same local SQLite database directory.
The existing typed `database.path`/`APP_DATABASE__PATH`, `rusqlite` application
queries, transactions, version checks and error mapping are reused unchanged.
The [optional Compose override](../qualification/yt094-valkey/compose.shared-sqlite.yaml)
maps one named `shared-data` volume at `/data` for both replicas, including the
database, WAL and shared-memory files. No new setting, CLI command, datastore
provider, ORM, migration framework or database-engine implementation is added.
The base fixture keeps its original independent volumes unless this override
is explicitly selected.

Valkey still supplies shared sessions through the already qualified opt-in
settings/provider. The lifetime is the current 1800-second default. Each
application retains its own connection, process and app-owned admission gate.
The selected storage is a Linux single-host local volume; a multi-host shared
filesystem or distributed SQLite deployment is outside this slice.

SQLite provides ACID transactions; its engine capabilities are not the subject
of this verification. Checks target the application's reads/writes, optimistic
conflicts, busy error/recovery and restart assembly. They do not certify power
failure, filesystem durability, database performance, WAL correctness or HA.

## Verification

The actual private application image is
`sha256:0f1c7afaea47c9883b1e4ef25150ab6f82002bce79899939a89657bbef9128cc`.
It contains the current locked Valkey-enabled native binary, public settings,
matching runtime libraries and the unchanged frontend snapshot, using the
existing scratch-image preparation. Valkey is the previously pinned 9.1.2
image from the base fixture. Inspecting the running containers confirmed that
both use this application image and exactly the same `/data` volume.

| Application check | Actual result |
| --- | --- |
| Native shared-file test | Two separately opened Board connections share records; B updates are visible through A; stale draft gets 409 and retains the draft |
| Native concurrent edits | Two requests for version 2 produce 303/409; one committed item/history version advances to 3 |
| Native external writer lock | Existing write route returns 503 with its safe busy message; after rollback, version remains 3, retry succeeds and restart reads version/history 4 |
| Board regression | Default suite 18/18; OIDC/Valkey-enabled suite 26/26; strict all-target Clippy passed |
| Compose topology | Two app processes, shared local SQLite volume and shared Valkey; both `/readyz` endpoints returned 200 |
| Cross-replica business reads/writes | A creates version 1, B reads it; B updates to version 2, A reads the updated editor values |
| HTTP stale/concurrent edits | Stale form returns 409 with its draft; simultaneous same-version A/B posts return 303/409; authoritative editor reports version 3 |
| App restart | While A is stopped B returns 200; restarted A accepts the same browser-style HTTP cookie and reads the same version 3 |
| Shared logout regression | B logout retires the session; A returns 401 with that cookie |
| HTTP security | Actual native form posts use Origin/CSRF and receive CSP headers; this is HTTP evidence, not a new rendered-browser matrix |
| Lifecycle/cleanup | A/B/Valkey exit 0; three app-drain markers include A's restart stop and final A/B stops. Zero owned containers/volumes/networks, all three ports closed and the owned app image removed |

Native commands:

```sh
cargo test --offline --locked -p yetlet-lab-operations-board independent_boards_share_sqlite
cargo test --offline --locked -p yetlet-lab-operations-board
cargo test --offline --locked -p yetlet-lab-operations-board --features oidc,valkey
cargo clippy --offline --locked -p yetlet-lab-operations-board --features oidc,valkey --all-targets -- -D warnings
cargo build --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features valkey
```

The actual HTTP journey used the existing native HTML forms and synthetic
terminal-code login, then separate replica URLs with one private cookie jar.
No external IdP/client/account, cloud resource or browser was recreated.
Private logs and aggregate receipts remain outside Git; credentials, cookies
and CSRF values are excluded from repository evidence.

Independent Reviewer `/root/yt094_shared_sqlite_review_oct19` returned PASS with
zero issues/required followups after inspecting the actual eight review targets,
eleven runtime input hashes, current asset/image identity, native/HTTP results
and lifecycle/cleanup records.

## Boundaries and next work

The accepted initial Valkey/browser/outage results are preserved separately;
they are not relabeled as a new SQLite or browser run. The existing native
read-only/unavailable-storage tests remain the broader error-path evidence.
Shared local storage does not imply cross-host deployment, distributed SSE
delivery, SQLite backup/restore, shared provider refresh or production HA.
Migrations/rolling schema compatibility and simultaneous first-run migration
qualification remain outside the selected test cases. The subsequent
[YT-094 closeout](YT-094-CLOSEOUT.md) accepts these limits and marks the selected
reference-application scope Done. YT-104 remains independently unblockable.
Owner-waived platform/input matrices remain closed.
