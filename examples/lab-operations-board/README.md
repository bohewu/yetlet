# Lab Operations Board — M11-P

**Unpublished, loopback-only application; YT-073 locally complete.** Fourteen native tests, final Chrome/Firefox journeys, real Chewing IME and actual Firefox/Orca speech pass. YT-074 is Done at the owner-amended mixed-load scope: the full 62-minute run passed with 45,000 measured requests and zero errors, followed by fresh independent review PASS. The prior synchronized-burst write-p95 failure remains a limitation. See [qualification](../../docs/YT-074-QUALIFICATION.md), [desktop acceptance](../../docs/YT-074-DESKTOP-ACCEPTANCE.md) and [startup](../../docs/BOARD-START.md). This is not production certification or a newly promoted catalog sample.

## Run locally

From the repository root:

```sh
cargo xtask build board-assets --check
cargo run --release --locked -p yetlet-lab-operations-board
```

The default listener is `127.0.0.1:3012`, and the URL is `http://127.0.0.1:3012/board`. Startup prints three fresh **synthetic, terminal-only** login codes. Alice operates Project 1, Bob operates Project 2, and Viewer reads Project 1. These are not real identities, passwords or an OIDC integration. Do not retain their codes in reports. Use the exact printed IP/port authority; Host validation intentionally rejects an alternative host name.

`YETLET_BOARD_LISTEN` can select another loopback address; non-loopback addresses fail. `YETLET_BOARD_DB` selects a local SQLite file, default `target/lab-operations.sqlite3`. Its parent must exist. Database paths are trusted operator configuration, not request parameters. Existing symlink/non-file database paths and unsupported schema versions fail. Keep the database private; no external deployment is configured.

## Implemented workflow

The SSR product provides a filtered/sorted/paginated work-item list, create/detail/edit, status changes, assignment within the current project and the latest 100 history records per item. Lists use 50 items per page. Search is a bounded, case-sensitive title substring. Sort uses three fixed choices, not user SQL. Closing retains history; a closed record must be reopened before becoming Active or Blocked. Hard deletion, remote machine execution, scheduling, rich HTML and uploads are not provided.

Viewer mutations and all cross-project access are checked on the server. Session continuity is separate from synthetic authentication. Login rotates Session/CSRF; unsafe actions require the session token and an accepted Origin when supplied; exact Host checking also protects the local listener boundary. Memory Session expires absolutely after 30 minutes and is lost on process restart. SQLite business records/history persist independently. No shared-session, real IdP, durable Redis, distributed login or HA guarantee is claimed.

Title is 1–160 characters, description at most 4,000, with a separate 64 KiB encoded form-body limit. All stored content is plain text rendered through escaped Yetlet Views. False booleans are omitted; textarea initialization preserves a leading newline. Initial forms do not show validation errors prematurely. Submitted invalid drafts remain visible; version conflicts return 409 with the unsaved draft and a link to the current record. Successful saves use POST/Redirect/GET. Updates are compare-and-swap and append history in the same transaction; history failure rolls back the record. Create requests are not idempotency-keyed: after an unknown response, inspect the list rather than blindly submitting again. No automatic mutation retry is implemented.

The application bounds active SQL workers to four, admission waiting to one second and SQLite busy waiting to 250 ms. Started blocking work owns its permit even after its caller is cancelled. Shutdown closes stream/worker admission and drains admitted work; cancellation does not undo a committed transaction. The local demo caps business records at 10,000 and history rows at 100,000; reaching those caps rejects new changes. Qualification started at 10,000 items/90,000 history rows to leave write headroom; separate native tests verify exact-cap rejection. No history is deleted automatically.

## HTML, assets and optional live status

Default lists/forms require no JavaScript or WASM. `?live=true` explicitly adds one external application-owned module and a bounded native SSE endpoint. Every connection obtains a complete project-authorized count/revision snapshot, so reconnect does not depend on replaying an in-memory queue. Later changes update counts and mark the displayed record list stale with an explicit authoritative-refresh link. A connection lasts at most five minutes, checks Session on each iteration and occupies one of sixteen stream slots. No client command or raw UI event is sent to the server.

The module supports stop and explicit reconnect, retains last-known counts on error, and attempts at most five consecutive backoff retries. Page retirement closes its connection/timer/listeners. Chrome and Firefox now verify actual snapshot/update/pause behavior, with Chrome explicit reconnect and ten long-lived native SSE readers renewing during sustained load. Native tests also verify logout termination.

The application owns a finite embedded asset table and document wrapper. Bootstrap 5.3.8 CSS and MIT notices are retained under `assets/upstream/`; `assets/inventory.json` records source and generated hashes. Build-time externalization removes embedded SVG data URLs without relaxing CSP. `cargo xtask build board-assets` reproduces the application-owned transform, using a separate `/board-assets/` prefix and independent local CSS/JS. There is no public stylesheet/head API, Bootstrap JS, remote font, CDN dependency or new CSS engine. Separate prefixes and inventories preserve app asset ownership and release inputs; the original Python transform remains a reference oracle.

Routes: `/` redirects to `/board`; `/board` resolves login or the current project; GET/POST `/board/login`; POST `/board/logout`; GET/POST `/board/projects/{project}/items`; GET `.../items/new`; GET/POST `.../items/{id}`; GET `.../items/{id}/edit`; GET `/board/projects/{project}/events`; `/healthz` is stateless liveness; `/readyz` checks storage/shutdown; `/board-assets/{name}` serves only embedded bytes; `/favicon.ico` returns 204. Unknown paths/methods retain native 404/405 and outer security headers.

## Validation

```sh
cargo test --locked --offline -p yetlet-lab-operations-board --all-targets
cargo clippy --locked --offline -p yetlet-lab-operations-board --all-targets -- -D warnings
cargo fmt -p yetlet-lab-operations-board -- --check
cargo build --release --locked --offline -p yetlet-lab-operations-board
```

These historical bounded journeys passed at their stated scopes using the original Python tools. Their retained receipts are not Rust qualification evidence; do not rerun sufficient passing evidence merely for another screenshot:

```sh
python3 examples/lab-operations-board/verify_local.py
python3 examples/lab-operations-board/verify_firefox.py
python3 examples/lab-operations-board/qualify_local.py
```

Current Rust commands (YT-085 smoke accepted; YT-086 qualification pending independent review) create fresh outputs in `target/m12/yt085/board-*-*/` and `target/m12/yt086/board-bounded-*/`:

```sh
cargo xtask smoke board-chrome
cargo xtask smoke board-firefox
cargo xtask qualify board
```

The historical import collision and failed login attempts remain documented. The material login fix changes Referrer-Policy from no-referrer to same-origin so native POST Origin survives the browser policy; null/foreign Origin and bad CSRF still fail. The intentional 409 console message is handled only at the known negative-test boundary. The repaired Chrome journey includes actual restart, three short 1,000-record diagnostic rounds and five fresh screenshots. The separate sustained check uses 10,000 records/90,000 initial history, three 60-second rounds and a 300-second soak after warmup: 24,000 measured requests, zero unexpected responses. These are not the original proposed 62-minute experiment.

The owner-amended `qualify_local.py --profile full` selects 120-second warmup, three 600-second rounds and 1,800-second soak at 0.5 requests/second/client. Five staggered cycle positions preserve each client's four-read/one-write pattern and produce 20 reads/five writes per nominal slot. Default `bounded` behavior remains unchanged. The synchronized full attempt failed write p95; its data is retained separately from the newly authorized mixed-load run. All numeric thresholds, SQLite FULL durability and hard caps are unchanged. The single newly authorized full execution passed all three rounds and the complete 30-minute soak; no replacement or automatic rerun is authorized.

The actual spoken journey covers page/navigation/form labels, validation discovery, detail/history, conflict draft, table caption/headers/links and live update/pause/reconnect/refresh. It does not prove automatic alert focus, general accessibility conformance or other screen readers. Windows Edge, additional browser/mobile platforms, cross-version deployment rollback, real identity and production rollout remain unverified. Final load and independent-review outcomes are recorded in qualification; no core/default browser dependency or GPU change is included.


## Opt-in commit-time write authorization — YT-181

The application feature `commit-write-auth` supplies the
[YT-181 SQL-owned reference](../../docs/YT-181-SENSITIVE-WRITE-IMPLEMENTATION.md).
It is default-off and adds no Core/Auth API or generic AOP. The existing Session,
CSRF, current file policy, item version, worker limits and response security
remain in effect. Its extra SQL authority covers only Alice/project-one item
writes, with item and history checked/committed together. It does not turn
Session logout or policy-file replacement into atomic SQL revocation.

Use a **new, nonexistent, task-private SQLite path** for the v2 reference; do
not point the selection at a real/v1 database or pre-create an empty file.
The first explicitly selected fresh file seeds Alice/project-one once. Opening
an existing v2 Store conservatively commits a deny and advances any enabled
generation before returning; opening another instance also invalidates old
allowed tickets. Denied rows are never reseeded. This intentionally requires
trusted current-authority confirmation and explicit re-enable after reopen;
it is not transparent restart availability or automatic restore qualification.
Stop/exclude all legacy writers; old binaries refuse v2 only at open.

Board consumers use `Board::open_with_options` (or
`open_with_oidc_and_options`) with `BoardOptions { write_authorization: true,
..BoardOptions::default() }`, supplying any selected frontend/Session fields.
[YT-185](../../docs/YT-185-OPT-IN-OPTIONS.md) records the intentional pre-public
constructor migration; Cargo features and existing external settings are unchanged.
Trusted application code can call
`board.set_project_write_permission("alice", 1, false).await?` to revoke, or
pass `true` to establish a fresh generation after current permission/recovery
confirmation. No HTTP control endpoint or automatic grant loop is installed.
Every request still needs ordinary identity, resource policy and CSRF allow;
SQL enable cannot grant another account/project or broaden those checks.

Admin additionally selects its existing Settings loader value
`write_authorization.enabled = true` (environment override
`APP_WRITE_AUTHORIZATION__ENABLED=true`) with the compiled `commit-write-auth`
feature and an explicitly new `database.path`. Base/default is false; absent
feature plus enabled selection fails settings validation. No hot reload or new
Settings-provider service is introduced. Do not interpret the parser/setting
as permission to enable v1 or to restore a backup over a live file.

One private Store-bound ticket is used per save attempt. SQL revoke advances its
generation, and re-enable advances again; stale tickets reject even after
re-enable. A writer already holding the IMMEDIATE transaction may finish before
revoke commits; a late save response does not change commit order. Busy, timeout
or disconnect never establishes rollback/revoke success or a safe blind retry.
The reference adds no retry, new queue, remote transaction or HA guarantee.


## Opt-in request and SQL-worker tracing — YT-182

[The application reference](../../docs/YT-182-TRACING-WORKER-IMPLEMENTATION.md)
uses native middleware and Rust `tracing`, with no generic AOP or tracing crate.
The `request-tracing` Cargo feature is default-off; compiling it does not
activate tracing. Embedded consumers use `Board::open_with_options` or
`open_with_oidc_and_options` with `BoardOptions.request_tracing = true`.
`write_authorization` is an independent options field; true requires its own
feature and fresh-v2 constraints. Default options disable both capabilities.
Board never initializes a global subscriber or reads ambient settings. Its
standalone binary retains defaults; the embedding host or Admin selects options.

Only server-generated, positive, provider-local request IDs and a finite HTTP
method enter root spans; worker records carry a fixed operation and `ok`/`error`.
No raw URI, query, credential, cookie, business body, SQL parameter or identity
is projected. The root has no ambient parent; scoped Dispatch and its selected
safe span are retained across blocking work and HTTP future destruction.
Unknown ambient spans/control calls are not forwarded. IDs reset with provider
recreation and are not distributed trace or durable audit identities.

`http_response_constructed`, `http_future_dropped` and `sql_worker_completed`
mean different things. Future drop does not cancel started SQL work, release
its permit early, imply rollback or authorize a retry. A constructed Response
is not completed body delivery; worker success is not a generic commit proof.
The existing four-worker limit, waits, Session/CSRF, SQL authority and drain
semantics remain. Outer Host/Origin refusals and Admin's separate Vite static
router are outside this trace scope; streams/body chunks are not instrumented.
