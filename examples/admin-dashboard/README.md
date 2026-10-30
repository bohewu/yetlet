# SSR administration reference — YT-094

YT-094 is Done at the selected reference-application scope. The
[closeout matrix](../../docs/YT-094-CLOSEOUT.md) links the accepted evidence and
limitations. Dated slice records below retain their original source/platform
scope; their earlier overall task status is superseded by that closeout.

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

The triage page also offers a read-only Chart.js status summary through its
existing Vite/Rust/WASM enhancement. Native Open/Active/Blocked/Closed counts
remain visible with JS off or if the chart fails. Totals cover all currently
loaded items (at most 12), including unsaved status changes; filtering does not
change their scope. Only an explicit Save writes to the server. See the
[YT-106 slice record](../../docs/YT-106-CHART-VERIFICATION.md) for ownership,
fallback, browser/CLI checks and measured package cost.

Description is enhanced with an app-owned CodeMirror plain-text editor on that
same page. Rust retains drafts and explicit Save; typing/undo/redo stays local,
with a 4,000-character limit and fresh undo history on item switch or Reset.
Only this HTML response receives a fresh style nonce. The native textarea stays
usable if the package/nonce/styles are unavailable, and JS-off item links still
reach the native editor. See the [editor record](../../docs/YT-106-CODEMIRROR-VERIFICATION.md)
for policy, lifecycle and bounded verification.

Sign in → filter/sort/paginate work items → detail → edit → history. Native POST
forms remain usable with JavaScript disabled. Validation and version conflicts
preserve drafts; permissions and CSRF checks run on the server. Closing keeps the
record and history. The existing opt-in live-status enhancement remains optional.

The [YT-107 form/save guide](../../docs/YT-107-FORM-FEEDBACK.md#application-authoring-guide)
compares native forms, the WASM textarea/CodeMirror workspace and Todo, with
source links and recovery boundaries. Native **Restore page values** resets to
this document's initial values, including rejected values on a 422 page; its
latest-record link opens in another tab. WASM **Reset draft** restores the loaded
record. Access/conflict/unconfirmed WASM saves retain selectable draft values
and require comparison/reapplication in a fresh workspace. Copy edits before
leaving or reloading; drafts have no browser persistence or automatic retry.

After preparing frontend assets:

```sh
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets
cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets -- -D warnings
cargo test --offline --locked -p yetlet-lab-operations-board --all-targets
cargo clippy --offline --locked -p yetlet-lab-operations-board --all-targets -- -D warnings
cargo xtask check docs
```

The default synthetic identities, loopback HTTP, local SQLite and memory
sessions are a local demonstration. Separately selected OIDC, Valkey sessions
and shared-SQLite evidence is linked in the closeout; production TLS, HA,
backups, remote deployment and general all-OS support remain unqualified. The
subsequent
[Windows native slice](../../docs/YT-094-WINDOWS-VERIFICATION.md) records source-matched
MSVC tests, CLI lifecycle and native HTTP separately from browser/OS input.
Service delivery from this edited application layout is not qualified. See the
[closeout](../../docs/YT-094-CLOSEOUT.md) for accepted limits and the
[backlog](../../docs/BACKLOG.md) for separate follow-ups.

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
rebuild or workspace mutation occurred. At that first-slice review YT-094 remained
In progress, with OS IME/assistive-technology, external OIDC and broader host
coverage still open. Later evidence and owner decisions are reconciled in the
[closeout](../../docs/YT-094-CLOSEOUT.md); production TLS/deployment,
authenticated dev-server bridging and production storage/HA remain boundaries.

## Opt-in OIDC

Runtime `oidc.enabled=true` selects the existing Board's optional OIDC client and
disables terminal-code login. Keep real provider/client settings and credentials
outside Git. `YETLET_DEVELOPMENT_SECRET_FILE` explicitly selects a private TOML
overlay only with `APP_ENVIRONMENT=Development`; selected secret values flow
through the existing SecretProvider. Verified subjects need explicit application
permission mappings. See the [OIDC contract and validation boundary](../../docs/YT-094-OIDC.md).
The default `false` preserves the earlier demonstration and qualification.

The separately activated [YT-140 proxy/replica integration](../../docs/YT-140-OIDC-PROXY-REPLICAS.md)
first reviews the canonical origin/callback, trusted proxy and cross-instance
transaction/freshness contract, then selects the smallest new integration and
proof. Initial independent contract Reviewer PASS, zero issues/required
followups; minimum implementation and focused proof/cleanup are complete, final
independent Reviewer PASS, zero issues and required followups; Done at bounded
optional local integration scope. Existing direct
loopback and real-provider evidence retain their original scope.

`oidc.application_origin` is an optional, empty-by-default value loaded by the
existing Settings provider. `APP_OIDC__APPLICATION_ORIGIN` or a declared
profile/private overlay can select, for example, `http://127.0.0.1:31000`; both
backends must use that same origin and exact registered callback
`http://127.0.0.1:31000/board/oidc/callback`. The backend listener remains separate.
Only canonical loopback HTTP with a nonzero explicit port other than 80 is
accepted; paths/trailing slash, credentials, query/fragment, other hosts and
HTTPS are rejected. Empty or absent retains the direct listener; disabled OIDC
keeps its unused configuration inert.

The local static proxy preserves the canonical Host/Origin and strips forwarding
headers. Backends do not infer origin from those headers; supplied callback
Origin must be unique and exact, while absent Origin remains valid for normal
top-level redirects. Keep backend access within the trusted local topology and
follow the reference's shared namespace/DTO/cookie, current SQL/policy and
coordinated restart/re-login rules. This option is not a public HTTPS deployment
configuration or peer authentication.

The new live proof used two independent Board listeners/managers in one test
process and rejected missing-code callbacks after cross-instance transaction
consumption. Existing successful real-provider evidence is reused; this is not
a new successful token exchange or cross-process Admin OIDC qualification.

## Shared sessions (opt-in)

Build with Cargo feature `valkey`, then select `session.backend="valkey"` through
the existing settings. The default remains `memory`; a build without that
capability rejects selecting the remote backend. The application aliases and
reuses the existing provider, with Valkey as the selected shared-session server.
See the [contract and two-app results](../../docs/YT-094-VALKEY-SESSIONS.md) and
[local Compose fixture](../../qualification/yt094-valkey/README.md). Set
`session.valkey.ttl_seconds` to choose an absolute lifetime from 1 second to
30 days; the default is 1800 seconds (30 minutes), matching the local demo.
Reads do not extend it. This shares sessions,
not the replica-local SQLite business databases or production HA.
The separately selected [shared-SQLite slice](../../docs/YT-094-SHARED-SQLITE.md)
adds an optional Compose volume override for two app instances on one host.

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

The subsequent [Windows Rust/WASM verification](../../docs/YT-094-WINDOWS-WASM-VERIFICATION.md)
records the candidate's native/browser checks and returned operator-attested
IME/keyboard/Narrator journeys, with drain/exit-0 images and final cleanup. Those
results supersede the pending Windows input status above at their bounded scope;
production authentication/deployment and the other stated limits remain open.

Independent Reviewer `/root/yt094_wasm_review` follow-up PASS, zero remaining
issues/followups. The first review found a queued-save lifetime defect; the
pre-execution guard and independent zero-POST regression resolved it.

### Optional Auth adoption (YT-126)

Build with `--features auth-valkey` and select all of
`APP_SESSION__BACKEND=valkey`, the existing Valkey endpoint/namespace settings,
`APP_AUTH__ENABLED=true`, and `APP_AUTH__POLICY_FILE=/absolute/path/policy.json`.
Auth is off by default. Selecting it without the compiled capability, Valkey,
or an absolute policy path fails startup. This local authority is Linux-only;
other platforms are not qualified and selection fails startup.

The trusted operator owns the policy file and its parent directory. Replace the
regular JSON file atomically on the same host; do not edit it in place. Example:

```json
{"accounts":{"alice":{"enabled":true,"read":true,"write":true},"bob":{"enabled":true,"read":true,"write":true},"viewer":{"enabled":true,"read":true,"write":false}}}
```

Only these account keys and boolean fields are accepted. A missing/null account
or disabled account denies with 403. Read/write flags only restrict the compiled
account bounds: Alice remains project 1, Bob project 2, and Viewer cannot write.
Each protected resolution and subsequent authorization reads fresh policy without
caching, including pre-SQL checks and every SSE iteration. The file is limited to
4096 bytes and each read to 250 ms; missing, malformed, oversized or unavailable
files fail closed with 503. Opened handles must be regular files; Linux opens use
`O_NONBLOCK | O_NOFOLLOW`. Trusted parent directories and local filesystems are
required; the deadline bounds the request wait, not arbitrary filesystem work.

The resolver uses the existing typed Valkey Session DTO and `yetlet::auth`.
Handlers consume `Authenticated` through a private adapter and independently
recheck Session, account and project permission. Identity uses the application
namespace `urn:yetlet:admin:local-accounts`, scheme `admin-session`, the verified
local account name, and static `project-1`/`project-2` tenant. This is the local
binding established by the existing verified login, not an external OIDC identity.
OIDC subject mapping remains a startup snapshot; policy changes revoke local
account/read/write access without changing that mapping. Generic Auth principals
are never serialized into Session or hydration state.

Login creation, verified demo/OIDC login and CSRF logout are explicit lifecycle
routes outside required Auth. Disabled users can still log out, including during
a policy-file outage. Health and static assets remain public. Protected missing,
expired, rotated or revoked Sessions and explicit anonymous records redirect to
`/board/login` with 303. Invalid cookies remain 400, denied policy 403,
misconfiguration 500 and unavailable authority 503. JSON triage keeps its existing
safe error envelope when authorization fails inside its handler; resolver failures
before the handler use the native error response. Unknown mutations never report
success or emit a success cookie and are not automatically retried.

The final fresh principal is passed to the existing database save operation.
There remains a race between the last authorization decision, queued SQL work
and commit: a previously admitted write can commit after revocation. There is no
cross-store fencing or strict write-after-revocation guarantee. See
[YT-126 adoption record](../../docs/YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md) for review
status and the exact bounded evidence; this is not production/HA qualification.

The separately activated [YT-138 ordering research](../../docs/YT-138-AUTHORIZATION-COMMIT-ORDERING.md)
examines the native/triage save and independent Session/policy authorities,
distinguishing authorization snapshots, revocation acknowledgment and business
commit. Its accepted decision retains the existing pre-SQL race for this
application; independent Reviewer PASS, zero issues and required followups;
Done at research scope. This documentation-only work adds no
fencing, transactional authorization, runtime middleware or stronger guarantee.

The separately activated [YT-139 authority/recovery reference](../../docs/YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md)
selects a local two-Admin fixture sharing one SQLite business authority, current
trusted policy and optional single-primary Valkey Sessions. Independent
contract Reviewer PASS, zero issues/required followups; implementation of the
bounded stale-Session restoration/re-login proof and cleanup are complete; final
independent Reviewer PASS, zero issues and required followups; Done at bounded
optional local reference scope. It does not qualify cross-host shared
filesystems, production HA or stronger write-after-revocation ordering.

Recovery uses the existing Settings-provider value
`APP_SESSION__VALKEY__NAMESPACE`: choose one never-reused recovery namespace for
both stopped/restarted instances, outside the restored Session data. Keep the
current SQLite database and restrictive policy; follow the reference's
stop/drain and refusal rules before reopening service. There is no runtime
epoch allocator or automatic protection against rolling configuration back to
an unsafe old namespace.

The new focused Linux test also requires Python 3 with standard-library SQLite
and an explicitly caller-owned disposable Valkey. It restores one retired
Session record and pauses that whole instance; the caller owns service cleanup:

```sh
YT139_VALKEY_URL=redis://127.0.0.1:PORT/ cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --test auth_valkey -- --ignored --exact two_admin_processes_recover_with_new_session_epoch --nocapture
```

The focused actual-process integration test requires an operator-owned disposable
Valkey at `YT126_VALKEY_URL=redis://127.0.0.1:PORT/` and existing built frontend
assets. It pauses that entire disposable instance, starts two actual Admin child
processes, captures generated demo credentials only in memory, uses isolated
configuration/policy/SQLite data and drains/reaps its children:

```sh
cargo test --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey \
  --offline --locked --test auth_valkey -- --ignored --exact two_admin_processes_use_fresh_auth_policy --nocapture
```

The test does not start or stop Valkey. Its bounded pause/recovery coverage does
not qualify persistent storage recovery, failover, external IdP callbacks,
browser behavior, cross-host deployment or atomic revocation of business writes.


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

Select both the feature and existing Settings value to enable the Admin host:

```sh
APP_REQUEST_TRACING__ENABLED=true cargo run --locked \
  --manifest-path examples/admin-dashboard/Cargo.toml --features request-tracing
```

The Settings field is `request_tracing.enabled`, false in `base.toml`; the sole
environment override is `APP_REQUEST_TRACING__ENABLED`. Enabled without the
compiled feature fails configuration validation. Disabled does not install the
new request-ID layers, spans, worker capture or subscriber. No runtime reload
or RUST_LOG/EnvFilter selection is added. Existing Auth/Session feature/settings
can compose independently; this example does not enable them by implication.

The host initializes JSON output to synchronous stderr once, before serving.
Only the fixed `REQUEST_TRACING_TARGET` namespace at INFO and higher is admitted;
other dependency diagnostics stay OFF. Subscriber conflict is a redacted startup
failure. Ordinary writer errors do not rewrite business results; slow output
can delay requests/workers. The embedding host owns access/retention/output
capacity. No collector, exporter, output queue, performance guarantee or durable
audit delivery is installed. Existing terminal-only synthetic login-code output
is a separate local-demo channel; never collect it as tracing evidence.
