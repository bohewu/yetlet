# Lab Operations Board — M11-P

**Unpublished, loopback-only application; YT-073 locally complete.** Fourteen native tests, Chrome/Firefox journeys and bounded YT-074 fault/load checks pass. Full YT-074 acceptance remains Blocked on unverified real IME/spoken AT and the broader proposed envelope. This is not production certification or a newly promoted catalog sample. See [current qualification](../../docs/YT-074-QUALIFICATION.md), [step-by-step startup](../../docs/BOARD-START.md) and [actual screenshots](../../docs/screenshots/m11-board/README.md).

## Run locally

From the repository root:

```sh
python3 examples/lab-operations-board/build_assets.py --check
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

The application owns a finite embedded asset table and document wrapper. Bootstrap 5.3.8 CSS and MIT notices are retained under `assets/upstream/`; `assets/inventory.json` records source and generated hashes. Build-time externalization removes embedded SVG data URLs without relaxing CSP. `build_assets.py` is an application-local copy of the proven transform, using a separate `/board-assets/` prefix and independent local CSS/JS. There is no public stylesheet/head API, Bootstrap JS, remote font, CDN dependency or new CSS engine. This deliberate small duplication preserves separate app asset ownership and release inputs.

Routes: `/` redirects to `/board`; `/board` resolves login or the current project; GET/POST `/board/login`; POST `/board/logout`; GET/POST `/board/projects/{project}/items`; GET `.../items/new`; GET/POST `.../items/{id}`; GET `.../items/{id}/edit`; GET `/board/projects/{project}/events`; `/healthz` is stateless liveness; `/readyz` checks storage/shutdown; `/board-assets/{name}` serves only embedded bytes; `/favicon.ico` returns 204. Unknown paths/methods retain native 404/405 and outer security headers.

## Validation

```sh
cargo test --locked --offline -p yetlet-lab-operations-board --all-targets
cargo clippy --locked --offline -p yetlet-lab-operations-board --all-targets -- -D warnings
cargo fmt -p yetlet-lab-operations-board -- --check
cargo build --release --locked --offline -p yetlet-lab-operations-board
```

These canonical bounded journeys now pass at their stated scopes. They start only owned temporary processes/data and clean them up; do not rerun sufficient passing evidence merely for another screenshot:

```sh
python3 examples/lab-operations-board/verify_local.py
python3 examples/lab-operations-board/verify_firefox.py
python3 examples/lab-operations-board/qualify_local.py
```

The historical import collision and failed login attempts remain documented. The material login fix changes Referrer-Policy from no-referrer to same-origin so native POST Origin survives the browser policy; null/foreign Origin and bad CSRF still fail. The intentional 409 console message is handled only at the known negative-test boundary. The repaired Chrome journey includes actual restart, three short 1,000-record diagnostic rounds and five fresh screenshots. The separate sustained check uses 10,000 records/90,000 initial history, three 60-second rounds and a 300-second soak after warmup: 24,000 measured requests, zero unexpected responses. These are not the original proposed 62-minute experiment.

Real Chinese OS IME, spoken assistive technology, additional browsers/mobile, the broader proposed duration, cross-version deployment rollback, real identity and production rollout remain unverified. Known-good same-schema local recovery and screenshot inspection are separately recorded actual evidence. Current review is owner-permitted current-session self-review, not independent security assurance. Runtime/core/default browser dependencies remain separate; no GPU work is included.
