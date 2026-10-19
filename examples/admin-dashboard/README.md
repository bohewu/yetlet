# SSR administration reference — YT-094 first slice

A responsive, local administration application built from
`yetlet new admin-dashboard --vite --tailwind --service --settings`, then composed
with the existing [Lab Operations Board](../lab-operations-board/README.md).
The standalone Cargo workspace is deliberate: the CLI operates on an application,
while the Board library remains a member of the repository workspace. Explicit
local Cargo paths reuse its routes, typed forms, sessions, authorization, SQLite
transactions and audit history. There is no second implementation of those rules.

The interface uses Tailwind CSS 4 through the official Vite plugin. The entry
scans shared Rust views for CSS and dynamically loads the Rust/WASM triage module
only on its selected page. The manifest supplies same-origin, versioned assets;
ordinary pages have no browser script. Desktop has a
project sidebar, filter toolbar and record table. Mobile uses horizontal project
navigation, compact filters and keyboard-focusable scrolling tables. Login,
validation, conflict and storage-error screens share the theme.

## Prepare and execute

Run from the repository root with the existing Yetlet CLI on PATH. Provision
Cargo and Node dependencies separately; the CLI never installs them. Install
the `wasm32-unknown-unknown` Rust target and matching `wasm-bindgen-cli` 0.2.129
before building. Build first to generate binding declarations for TypeScript:

```sh
npm ci --prefix examples/admin-dashboard/frontend --ignore-scripts --no-audit --no-fund
cargo generate-lockfile --offline --manifest-path examples/admin-dashboard/Cargo.toml
yetlet build --manifest-path examples/admin-dashboard/Cargo.toml
npm run typecheck --prefix examples/admin-dashboard/frontend
yetlet run --manifest-path examples/admin-dashboard/Cargo.toml
```

Open `http://127.0.0.1:3014/board`. Copy the matching identity's terminal-only
`LOCAL_LOGIN` code into the sign-in form. Alice writes Project 1; Bob writes
Project 2; Viewer only reads Project 1. Codes and memory sessions are replaced on
restart. Business records and their transactional history remain in
`data/board.sqlite3`. Do not save terminal codes in screenshots or reports.

Stop the foreground CLI with Ctrl-C. The app signals streams and drains admitted
SQLite work. `yetlet run` consumes an existing build; rebuild after changing Rust
or CSS. App-root Rust watch can be selected using the existing CLI settings or
`yetlet watch`; changes in the external shared Board dependency are outside that
watcher's app-root scope.

## Settings

The existing `yetlet-config::Loader` loads `base.toml`, the selected profile and
`APP_` overrides. `APP_ENVIRONMENT` defaults to Production; the three profiles
remain loopback-only. Examples:

```sh
APP_SERVER__PORT=3015 yetlet run --manifest-path examples/admin-dashboard/Cargo.toml
APP_DATABASE__PATH=/absolute/private/board.sqlite3 yetlet run --manifest-path examples/admin-dashboard/Cargo.toml
APP_CLI__VITE__AUTO_BUILD=false yetlet build --manifest-path examples/admin-dashboard/Cargo.toml
```

`cli.vite.auto_build=true` builds browser Rust/WASM, bindings and Tailwind/Vite
assets before the native Rust app. With `auto_build=false`,
prepare a matching frontend build separately. `cli.vite.auto_start=false` is
intentional: this application serves built frontend assets directly from Rust. The generated
anonymous-root dev bridge cannot forward this application's cookie-backed,
multiroute workflow, so its unused dev scripts were removed and the application
rejects `auto_start=true`. Authenticated frontend HMR is a separate decision.
Disabling `cli.vite.enabled` skips CLI orchestration; the application still needs
its prepared frontend snapshot.

`frontend/` can be renamed by moving it and changing `cli.vite.root`; keep its
relative Tailwind sources aligned with the Rust views. Public frontend dotenv
files stay within that directory. No backend dotenv discovery or required secret
is added. The existing Loader's explicitly selected Development-secret overlay
remains available for declared application settings; do not copy it into the
frontend. `YETLET_CONFIG_ROOT` selects an absolute settings directory; relative
SQLite paths follow that root. `YETLET_ASSET_ROOT` selects a trusted, immutable,
absolute frontend build directory. The default asset root follows this app's
build-time directory and selected frontend root, independently of the working
directory.

## Workflow and checks

Sign in → filter/sort/paginate work items → detail → edit → history. Native POST
forms remain usable with JavaScript disabled. Validation and version conflicts
preserve drafts; permissions and CSRF checks run on the server. Closing keeps the
record and history. The existing opt-in live-status enhancement remains optional.

After preparing frontend assets:

```sh
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets
cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets -- -D warnings
cargo test --offline --locked -p yetlet-lab-operations-board --all-targets
cargo clippy --offline --locked -p yetlet-lab-operations-board --all-targets -- -D warnings
cargo xtask check docs
```

This is the first local YT-094 slice, not production qualification. Synthetic
identities, loopback HTTP, local SQLite and memory sessions do not qualify OIDC,
production TLS, shared sessions, HA, backups, remote deployment or general
Windows/macOS browser support. The subsequent
[Windows native slice](../../docs/YT-094-WINDOWS-VERIFICATION.md) records source-matched
MSVC tests, CLI lifecycle and native HTTP separately from browser/OS input.
Service delivery from this edited application layout is not qualified. See the
[backlog](../../docs/BACKLOG.md) for remaining gates; YT-094 stays In progress.

## First-slice local evidence — 2026-10-19

Base checkout `7e7b4f168cd40920966a8b1ed2eb21ada07ba54a`, Linux x86_64,
Node 24.21.0, Chrome 155.0.8059.84. Browser plugin not available; bundled
Playwright used system Chrome. These are working-tree checks, not release or
cross-platform qualification. Temporary scripts, screenshots and synthetic SQLite
fixtures are outside the repository in `/tmp/yetlet-yt094-20261019-4te8qtu_`.
They contain no saved login codes.

| Check | Result and boundary |
|---|---|
| CLI creation | Selected Vite, Tailwind, service and settings options; generated into a protected temporary parent, then moved into `examples/`. The shared `examples/` parent is group-writable and correctly failed CLI publication preflight; the CLI trust rule was preserved |
| Dependency preparation | Separate offline npm installation and Cargo lock preparation succeeded |
| CLI build | `yetlet build --manifest-path examples/admin-dashboard/Cargo.toml` exited 0; built Vite CSS then Rust. With `APP_CLI__VITE__AUTO_BUILD=false`, exited 0 and explicitly skipped frontend compilation |
| Native tests | Standalone app 13/13; existing Board 15/15. Covers settings, stylesheet selection/path rejection, route composition, storage failure/rollback, authorization, CSRF, optimistic concurrency and restart |
| Static checks | Both packages passed strict Clippy and format checks; frontend TypeScript check passed; `cargo xtask check docs` passed |
| No-JS browser workflow | Login, 53-record pagination (50 + 3), filter/empty states, create/detail/edit/history, preserved 422 validation draft and 409 conflict draft passed. The subsequent new record brought the fixture total to 54 |
| Input and layout | Desktop 1440×1000 and mobile 390×844 rendered with selected CSS, no body overflow. Native controls, visible keyboard skip/focus and a Chrome IME composition/commit of Chinese text passed. This is browser-protocol IME evidence, not an OS IME or assistive-technology qualification |
| Security and storage | Viewer writes, cross-project access and forged CSRF denied; escaped markup remained plain text. Disposable SQLite table outage produced 503 guidance and recovered after restoration |
| Bounded workload | 40 authenticated detail reads at concurrency 8 passed; approximately 23 ms locally, not a capacity or latency commitment |
| Lifecycle | All application startup/shutdown used `yetlet run`. Two Ctrl-C stops printed the app drain marker and closed the port; the CLI reported SIGINT. Restart preserved records/history and retired in-memory sessions; logout invalidated the session |
| Visual inspection | Matching three-record baseline and new desktop view were inspected. List, detail, validation, conflict, empty, viewer and mobile edit/error states were captured. The initially unstyled storage failure was corrected to the selected theme; mobile filters were compacted and record titles given a readable minimum width |

The first browser attempt exposed a real Router composition defect: installing a
new fallback replaced the Board's globally wrapped routes and returned 404 for
login. The application now mounts assets before the intact Board fallback; a
native regression test proves login, its security headers and wrong-Host denial.
The initial failed result is retained beside the successful browser evidence.
No uncaught page errors were observed in the no-JS browser journey; expected
HTTP 422/409/403/503 responses were exercised deliberately.

Independent Reviewer `/root/yt094_slice_review` passed this first slice with
zero material issues and required followups. The reviewer inspected the source,
browser assertions and desktop/mobile/error screenshots, and independently ran
the existing compiled app and Board test binaries (13/13 and 15/15); no reviewer
rebuild or workspace mutation occurred. YT-094 remains In progress; next
qualification should select native OS IME/assistive-technology and broader host
coverage before treating this reference as fully qualified. External OIDC,
production TLS/deployment, authenticated dev-server bridging and production
storage/HA remain explicit boundaries.

## Opt-in OIDC

Runtime `oidc.enabled=true` selects the existing Board's optional OIDC client and
disables terminal-code login. Keep real provider/client settings and credentials
outside Git. `YETLET_DEVELOPMENT_SECRET_FILE` explicitly selects a private TOML
overlay only with `APP_ENVIRONMENT=Development`; selected secret values flow
through the existing SecretProvider. Verified subjects need explicit application
permission mappings. See the [OIDC contract and validation boundary](../../docs/YT-094-OIDC.md).
The default `false` preserves the earlier demonstration and qualification.

## Rust/WASM triage workspace

Operators can select Triage workspace for the latest 12 project items. Filter,
reorder and switch between local drafts without HTTP; Save selected sends one
authenticated, CSRF-protected, version-checked command. Conflicts retain drafts.
Without WASM, item links still reach the native detail/editor pages. Other pages
remain script-free. Drafts are tab-local and are discarded on navigation/reload.

The frontend build now also needs the `wasm32-unknown-unknown` Rust target and
`wasm-bindgen-cli` 0.2.129. `WASM_BINDGEN` can select that executable; the existing
user-local Yetlet tools directory or PATH is the fallback. The existing settings
and CLI build/run/stop workflow are unchanged. Generated bindings are ignored.
See the [slice verification](../../docs/YT-094-WASM-TRIAGE.md) and
[API observations](../../docs/YT-094-API-OBSERVATIONS.md). The earlier Windows
native evidence does not qualify this changed slice; Windows browser and real
OS input/assistive technology remain deferred to final qualification.

Independent Reviewer `/root/yt094_wasm_review` follow-up PASS, zero remaining
issues/followups. The first review found a queued-save lifetime defect; the
pre-execution guard and independent zero-POST regression resolved it.
