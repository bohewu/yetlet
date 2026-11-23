# Sample Applications

**Status:** Thirteen catalog applications are locally verified at bounded prototype scope. The original twelve-application M8/M9 compatibility/evidence freeze remains historical; YT-241 Style Showcase was independently reviewed and then promoted separately.

Yetlet samples exist to validate application ergonomics, dependency isolation and security boundaries. They are not a parallel tutorial framework and do not authorize new core abstractions merely to make a demo look shorter.

## Choose a starting point

Begin with [Hello World](../examples/hello-world/README.md) for native SSR or [REST API](../examples/rest-api/README.md) for HTTP-only development. Use [Todo](../examples/todo-server/README.md) for shared SSR/WASM Views and [Session cart](../examples/cart-server/README.md) for an explicit server capability. Use [Style Showcase](../examples/style-showcase/README.md) to compare native previews with real Rust View, Style and generated CSS sources. Each sample-local README provides exact commands and limitations. The [documentation index](README.md) separates current guidance from historical qualification.

## Startup paths in this checkout

The current checkout uses `yetlet::web::App` with explicit `web,html`
features for the minimal SSR example below. Its `new().page(...).serve()` and
`from_router(...).serve_on(address)` startup paths retain the address and runtime
contracts described below. The [published SDK `0.9.0-alpha.9` startup API](API-REFERENCE.md#published-application-startup--sdk-alpha9)
uses `App` through `web`; its SSR example below also selects `html`.

The non-catalog SDK minimal example uses the fixed `127.0.0.1:7878` HTTP default.
Run it from the repository root with the following command.

```bash
cargo run --locked -p yetlet --features web,html --example server-axum-minimal
```

The thirteen catalog applications preserve their own environment/port contracts.

| Sample | Environment variable | Default HTTP address | Startup |
|---|---|---|---|
| Hello World | `YETLET_HELLO_LISTEN` | `127.0.0.1:3003` | `from_router(...).serve_on(address)` |
| REST API | `YETLET_REST_LISTEN` | `127.0.0.1:3001` | `from_router(...).serve_on(address)` |
| Session Cart | `YETLET_CART_LISTEN` | `127.0.0.1:3002` | `from_router(...).serve_on(address)` |
| Middleware Lab | `YETLET_MW_LISTEN` | `127.0.0.1:3004` | `from_router(...).serve_on(address)` |
| Forms Validation | `YETLET_FORMS_LISTEN` | `127.0.0.1:3005` | `from_router(...).serve_on(address)` |
| File Server | `YETLET_FILES_LISTEN` | `127.0.0.1:3006` | Native listener and graceful shutdown |
| Auth Notes | `YETLET_AUTH_LISTEN` | `127.0.0.1:3010` | Native listener and graceful shutdown |
| Modular Shop | `YETLET_SHOP_LISTEN` | `127.0.0.1:3008` | Native listener and graceful shutdown |
| SSE Notifications | `YETLET_SSE_LISTEN` | `127.0.0.1:3008` | Native listener and graceful shutdown |
| SQLite CRUD | `YETLET_SQLITE_LISTEN` | `127.0.0.1:3007` | Native listener and graceful shutdown |
| Todo | `YETLET_LISTEN` | `127.0.0.1:3000` | `from_router(...).serve_on(address)` |
| Multi-Island Dashboard | `YETLET_DASHBOARD_LISTEN` | `127.0.0.1:3009` | Native listener and graceful shutdown |
| Style Showcase | `YETLET_SHOWCASE_LISTEN` | `127.0.0.1:3011` | `from_router(...).serve_on(address)` |

The five basic HTTP samples, Todo and Style Showcase retain a native listener
branch for port `0`, allowing the banner to print `local_addr()` after binding.
The five lifecycle-sensitive capability samples and Dashboard keep native
`TcpListener`, `axum::serve` and `with_graceful_shutdown` visible. `into_router()`
and direct Axum/Tower composition remain first-class. Prelude imports are optional.
Modular Shop and SSE share port `3008`, so concurrent runs require explicit
separate addresses. Each local README retains its exact root-level catalog command.

Todo and Dashboard use explicit browser build commands for their WASM journeys.
Todo also supports native-only SSR with `cargo xtask build todo-css` before
`cargo run --locked -p yetlet-todo-server`. Style Showcase's catalog command is
native-only, with `browser_runtime = false`; its optional companion is built
separately with `cargo xtask build showcase-browser`.

`serve()` and `serve_on()` use plain HTTP with caller-owned Tokio runtime and
explicit TCP addresses. HTTPS/TLS requires an explicit secure edge or native TLS
integration. The methods never infer TLS, create certificates or start a hidden
runtime. The minimal default uses a fixed port and returns bind errors.

## Catalog contract

`examples/samples.toml` is the machine-readable catalog for user-facing runnable applications. `cargo xtask check samples` validates the catalog with the isolated Rust tool. The browser build field names an exact supported Rust command; package identity, repository source, companions and README commands remain checked.

Each entry declares:

- a stable sample ID, title, workspace package and directory;
- one copyable root-level run command;
- routes plus sample-local source markers that establish each route;
- whether a browser runtime is required, its companion packages and explicit build script;
- optional Yetlet crates deliberately selected by the primary or declared companion packages;
- retained evidence supporting its current status.

Every catalogued directory also owns a local `README.md`. Internal packages such as browser entry crates, shared component crates and research fixtures remain ordinary workspace members but are not required to appear in the user-facing catalog.

Catalog paths, route sources, build scripts and evidence must resolve inside the repository. Commands must exactly match a README line and launch the declared primary package. Route declarations carry source markers, and optional dependencies must appear directly in the primary or declared companion manifests.

## Structure rules

1. Keep one runnable application package per catalog entry.
2. Use ordinary Rust handlers, Axum extractors and narrow Yetlet values; do not add mandatory Controllers, DI or source-layout conventions.
3. Keep browser builds explicit. An API-only sample must not require `wasm32`, wasm-bindgen, generated browser assets or a browser dependency.
4. Declare optional capabilities in the primary package or an explicit companion package. The Todo server owns request correlation, while its browser companion owns keyed/reactive dependencies. Applications that do not use Session, request correlation or browser updates must not select those crates.
5. Include task-specific native tests and a bounded localhost smoke test when network behavior is part of the sample.
6. Record limitations and distinguish local prototype verification from production assurance.

## Current catalog

| ID | Package | Kind | Browser runtime | Purpose |
|---|---|---|---:|---|
| `hello-world` | `yetlet-hello-world` | Server rendered | No | Minimal native SSR, HTML escaping, strict CSP and zero optional providers |
| `todo` | `yetlet-todo-server` | Full stack | Yes | SSR, Browser/WASM, Search Island, typed HTTP actions and keyed local updates |
| `rest-api` | `yetlet-rest-api` | API | No | Typed Book CRUD, RequestContext policy, structured errors and request correlation |
| `session-cart` | `yetlet-cart-server` | Server rendered | No | Opt-in server-side Session, native forms, CSRF boundary and refresh persistence |
| `auth-notes` | `yetlet-auth-notes` | Server rendered | No | Synthetic authentication, Session rotation, CSRF and per-user note authorization |
| `modular-shop` | `yetlet-modular-shop` | Server rendered | No | Explicit modules, Session cart and transactional/idempotent direct-SQL orders |
| `middleware-lab` | `yetlet-middleware-lab` | API | No | Native Tower middleware scopes, request extensions, short-circuit, streaming |
| `file-server` | `yetlet-file-server` | Server rendered | No | Fixed static fixture, bounded protected multipart upload, opaque-ID streaming download and cleanup |
| `sse-notifications` | `yetlet-sse-notifications` | API | No | Bounded SSE replay, heartbeat, disconnect cleanup and lagging-client policy |
| `forms-validation` | `yetlet-forms-validation` | Server rendered | No | Stateless SSR forms, escaped accessible errors and no-side-effect 303 PRG |
| `sqlite-crud` | `yetlet-sqlite-crud` | API | No | Direct parameterized SQLite, explicit transactions, restart persistence and optimistic versions |
| `multi-island-dashboard` | `yetlet-multi-island-dashboard` | Full stack | Yes | Complete SSR plus independently built/lazy Counter and Filter islands |
| `style-showcase` | `yetlet-style-showcase` | Server rendered | No; optional WASM | Four-category style gallery, actual Rust/CSS disclosures and optional local class switching |

YT-241 Style Showcase passed independent implementation Reviewer `wc_job_SjBc9esAHAREdWS7`
before its separate catalog promotion. The unpublished checkout sample runs with
native Rust/Cargo. Its optional `yetlet-style-showcase-browser` companion has a
separate build command. Todo and Style Showcase report their own bounded Linux
Chrome/Firefox results. [Todo CSS usage](../examples/todo-server/README.md#app-owned-rust-styles-and-typed-tag-builders),
[original Todo browser scope](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-240-TODO-STYLE-QUALIFICATION.md#platform-matrix-and-acceptance-boundary),
[Showcase commands and verification](../examples/style-showcase/README.md),
[published CLI Style evidence](CLI-GUIDE.md#style-release-and-platform-evidence)

## Research fixtures

The five [completed research studies](README.md#completed-research-prototypes) are bounded application-owned prototypes, separate from this thirteen-application catalog. Use their fixture READMEs and retained measurements to understand the decisions and limitations; they do not select additional supported core APIs or change application qualification.

The [Physics Lab](../examples/physics-lab/README.md) is a separate nested workspace with native SSR and explicitly selected Rapier/WebGPU assets; it is **not** a catalog or supported GPU engine. The [Experimental GPU support guide](../crates/yetlet-browser/guides/wgpu-support.md) gives the exact qualified Firefox/CPU/Wasm source conditions. Browser picking, timed draw submissions and original Firefox/WebGPU measurements must not be generalized into production performance or cross-browser support.

Run the existing navigation checks:

```bash
cargo xtask check samples
cargo xtask check docs
```

## Sample acceptance and source boundaries

These dated acceptance sections retain the catalog's M8/M9 evidence after
consolidation. They describe the **original frozen source on the local Linux
host**, not a fresh run of today's SDK. Rust/Cargo 1.99.0, Python 3.14.4, Chrome
155.0.8059.39 and Firefox 157.0/geckodriver 0.37.1 were the recorded tools.
Native-only stages did not each repeat browser suites; they checked wasm32,
normal dependency trees and exact rebuilt Todo/Search assets, followed by the
actual cross-product M9j browser freeze. Original Markdown and headings are
recoverable at local Git `ac98cdc127ccd299cb1150fef922b0560453a1d4`; remote
GitHub reachability is unverified. Raw tests, source snapshots and reviewer logs
remain separately retained ignored/external evidence, not committed Git payloads.
The [qualification matrix](QUALIFICATION-MATRIX.md#sample-and-api-freeze-evidence)
records aggregate scope, bundle costs and failure/replacement populations.

### Catalog and planning acceptance

M8a / YT-044 (2026-10-09) introduced repository-backed paths, exact README commands,
route source markers, companion/build mappings and direct optional dependencies.
Shape-only validation initially accepted false command/route/browser claims and
escaped evidence. Three failed independent reviews additionally found a task
edge, API-kind isolation, route prefix/suffix and malformed package/type gaps.
Those counterexamples were repaired before final reviewer
`wc_job_vQ6pGDE0slh9gV8c` PASS, which exercised 32 adjacent counterexamples. That
catalog contract was not weakened for this consolidation.

M9p / YT-049 planned the original nine names in display order (Hello, Todo, REST,
Cart, Auth, Middleware, Files, SSE, Shop) plus Forms, SQLite and Dashboard. Display
order differs from dependency/security implementation order. Planning did not
authorize implementation or catalog promotion. Reviewer `wc_job_fr93PI1YLOtR3KjE`
passed the eight-file documentation-only gate (40 docs/59 task IDs, existing
three-entry catalog/four checker tests). A mistaken normalized-string harness
assertion was corrected without product failure. YT-050–059 then had separate
sequential implementation/review gates; no unbuilt proposal became verified.

### Hello World acceptance

M9a / YT-050, 2026-10-09: three native tests, real loopback smoke, complete
117 workspace tests/five doctests, strict Clippy and debug/release builds PASS.
`/` and escaped `/hello/{name}` retained strict script-free CSP and native
404/405, with no cookie/provider/browser/DB dependencies. It is distinct from
the adapter's internal minimal example. Local unstripped debug/release binaries
were 26,737,520/1,857,904 bytes, not memory or wire size. A catalog negative fixture
was repaired to select Todo by stable ID rather than item zero. Independent
read-only reviewer `wc_job_Htk-Quo_aFExMJs3` PASS preceded fourth-entry promotion.
Anonymous loopback development is not an exposed hardened service.

### Todo acceptance

M2–M7j (2026-10-08/09) proved script-free SSR, shared SSR/DOM TodoList byte parity,
lazy bounded Search state, local toggles with zero requests and explicit semantic
HTTP create/error handling. M6 preserved native handler/middleware/API behavior.
M7j finished with 91 workspace tests/five doctests, six wasm32 packages and
Chrome/Firefox debug/release plus isolated keyed lifecycle tests. Initial M7b
review found preflight/reserved-attribute/stale-owner gaps; the first final M7j
review then found queued/in-flight retirement races despite M7i's earlier PASS.
Token lifetime repairs passed replacement validation and repeat final reviewer
`wc_job_XZzdXEOIx6owj1o5`. Native HTML/CSP, no raw-event circuit, detached listeners
and duplicate-free remount were checked; cleanup's ancestor-matching false
positive was resolved separately after product PASS. [Ownership and bounds](BROWSER-APP.md#keyed-ownership-and-runtime-retirement)
remain explicit. Only the local list adopts keyed/reactive; Search and remote
lists retain replacement. Browser bundle grew 82,726 raw/20,042 local-gzip bytes;
Search stayed byte-identical after separate entry builds. The in-memory API is
anonymous, not auth/credentialed CSRF, general forms or real IME qualification.

### REST API acceptance

M8b / YT-045, 2026-10-09: seven tests, native build/strict Clippy and real loopback
Book CRUD/policy smoke PASS; normal trees excluded every browser/DOM/reactive
runtime. Optimistic versions, validation, 8 KiB JSON limit and no-store/nosniff/API
version headers are application policy. Registered method fallback is empty 405
with route/global headers; unmatched `/api/missing` is empty 404 with global
correlation only. Do not promise JSON or route headers for every fallback.

Initial reviews found duplicate/malformed Origin/Host bypass and overstated
fallback coverage; a repeat found Authority accepted commas/bad ports. Three
later reviewer executions failed before model output (`workspace routing
discovery failed`), providing **no review result**. Self-review fixed duplicate
Content-Type but did not satisfy independence. Recovered review found supplied
Host validation could be skipped by no-Origin. Final policy validates any
supplied Host first, requires single syntactically valid direct-HTTP authorities,
handles bracketed IPv6/ports and rejects ambiguous values before mutation.
Complete validation was rerun; final `wc_job_d911MsTpbwT6fZY3` PASS closed that
scope. No-Origin clients are allowed, but this is neither authentication nor a
TLS reverse-proxy trust policy, persistence or distributed-concurrency contract.

### Session Cart acceptance

The catalog deliberately retains the unchanged [M8c record](M8C-VERIFICATION.md).
It proves native SSR/forms with explicitly selected bounded Memory Session,
opaque cookie/CSRF identity, absolute expiry/capacity, rotation/revocation and no
browser dependency. It is an anonymous single-node cart, not authentication,
durable/distributed storage or production CSRF qualification. M8e's later
stale-destroy P2 and repaired 114-test freeze remain recorded in the
[qualification matrix](QUALIFICATION-MATRIX.md#sample-and-api-freeze-evidence).

### Middleware Lab acceptance

M9b / YT-051, 2026-10-09: seven focused tests, 124 workspace tests/five doctests,
strict Clippy, native builds and real ephemeral loopback smoke PASS. Tests prove
global/group/method entry and reverse unwind, 403 short circuit without body
polling, 404/405/Allow, request extension versus Arc state isolation, Tower HTTP
headers and preserved mid-stream body failure. A stream failing after headers
is not successful recovery. Runtime tree excluded providers/browser/DB; local
release binary was 2,101,760 raw bytes. Terminal exit-0 independent reviewer
`wc_job_UnKflDaA8ZQHXCce` PASS preceded fifth-entry promotion. There is no second
middleware engine, distributed tracing, proxy/TLS or zero-overhead claim.

### Forms Validation acceptance

M9c / YT-052, 2026-10-09: six tests, 130 workspace tests/five doctests, native
strict Clippy/build and real HTTP smoke PASS. Invalid SSR is escaped 422 with
labels, described errors, invalid fields and first-error focus; valid submission
uses stateless/no-side-effect 303 PRG. No JS, cookies, Session or DB is required.
Native Form unknown-field rejection is 422, not the initially expected 400.
An initial smoke used a valid name while expecting a name error; separate
hostile-escaping and invalid-name cases repaired that evidence. First reviewer
`wc_job_7D2rtHdGEfwW1Aet` failed English content inheriting the then-default
zh-Hant document language. Sample-owned main language overrides repaired all
three response paths; later metadata work owns the current root-language API.
Repeat `wc_job_lGtk_ZIoAvK4F6PW` PASS accepted the source/evidence, explicitly
without independent verification of effective model/effort beyond requested
selectors. This is no CSRF claim: credentials or persistent mutation require
separate security design; email validation is illustrative.

### File Server acceptance

M9d / YT-053, 2026-10-09: 14 focused tests, 144 workspace tests/five doctests,
native builds, six wasm32 checks and all 136 unchanged production dist files PASS.
Real loopback sockets proved static GET/HEAD/range, bearer-protected bounded
multipart upload, opaque repeated download/HEAD, content/disposition/nosniff,
Origin/name/extension/binary/size/path/method rejection and mid-upload disconnect
cleanup followed by successful upload. Shutdown reaped the server and private
root. Unix symlink replacement never exposed the substituted file; deadline,
capacity, cancellation/backpressure and store lifetime were tested. Initial
getrandom-to-io error conversion failed to compile; conversion preserves
fail-closed RNG behavior. A failing test exposed tempfile directory mode;
explicit Unix 0700 at creation repaired the defect rather than weakening it.

Independent `wc_job_g8JSu80-Jseg6lmG` PASS checked 143 source/136 asset hashes;
seven-entry catalog promotion then passed. Runtime trace verified its requested
model/effort, but did not establish native role identity or formal security
assurance. Release binary was 2,863,128 local raw bytes. File libraries stay
app-local; no arbitrary root/directory listing, executable uploads, browser upload
form, default storage feature or production exposure is authorized. Browser E2E
was not replayed in this native-only stage; M9j provided the later full matrix.

### SQL-first data access and SQLite acceptance

**No Yetlet ORM is supplied, built or required.** SQL/parameters, migration,
transactions and row mapping belong to the app, through ordinary State/owned
native handles. No query DSL, implicit DbContext, generated models, migration
engine, mandatory repository/Unit of Work or DI layer was adopted. External
SQLx/mappers/ORMs remain explicit future app choices; no DB handles, secrets,
SQL/schema internals enter browser hydration state.

M9e's 2026-10-09 two disposable in-memory fixtures created a table, inserted a
parameterized row and mapped it into Rust under installed Rust 1.99. The then
unverified 1.85 baseline excluded SQLx 0.9's declared 1.94 requirement, so the
comparison used SQLx 0.8.6, not a universal MSRV conclusion.

| Selected fixture | Features | Packages | Raw release bytes | Initial / clean offline build |
|---|---|---:|---:|---:|
| rusqlite 0.40.2 | no defaults, bundled | 12 | 2,600,400 | 28.800 / 27.770 s |
| sqlx 0.8.6 | no defaults, runtime-tokio/sqlite/macros | 117 | 4,365,424 | 31.837 / 30.460 s |

SQLx had 109 packages absent from the rusqlite fixture and supplies async pooling,
multiple databases and derives not needed here. Single-host build times are
informational, not a comparative speed claim. Direct `Row::get` and a bundled
single-file DB selected app-local rusqlite. Four-permit `spawn_blocking` bounds
synchronous operations; startup alone owns migration/WAL, then each bounded
operation opens a connection. This is no universal anti-pooling decision.

M9e / YT-054 passed nine focused tests, 153 workspace tests/five doctests,
three-process restart smoke (one 200/one 409 concurrent winner, durable update
and deletion), rollback after prior write, parameter/injection-like values,
NOCASE uniqueness, Unix owner-only creation and explicit schema/version checks.
The first catalog check failed README route token discoverability; docs repaired
it without runtime change. First reviewer `wc_job_Q3pBfUqPYGYNSIrD` then found
version-1 renamed columns passed table-name-only validation. Exact ordered columns
for notes and audit are now checked; both renamed-column fixtures fail startup
with SchemaMismatch. Complete replacement validation, 22 dependency trees, six
wasm32 and 136 unchanged production assets passed; repeat
`wc_job_aVBprnEsYc8o3ubi` PASS preceded eighth-entry promotion. Browser E2E was
not repeated here. Release sample binary was 4,676,656 local raw bytes. Loopback
SQLite is application persistence, not a framework data provider.

### SSE Notifications acceptance

M9f / YT-055, 2026-10-10: ten tests, 163 workspace tests/five doctests, native
builds, six wasm32, 23 package trees and 136 unchanged production assets PASS.
An app Hub limits replay to 16 events, subscribers to eight and queued events to
four each; one-second comment heartbeats and finite monotonic IDs are explicit.
Fresh clients get future events only; Last-Event-ID replays greater retained IDs.
Old-gap/future/malformed/duplicate cursors fail closed. Full queues disconnect
laggards without blocking publishers; queued events drain before reconnect.
This is bounded at-least-once, not durable/exactly-once messaging or a global UI
circuit. Mutation rejects every Origin, requires one JSON Content-Type and caps
body at 2 KiB; streams set text/event-stream/no-store/nosniff/no-buffering.

Live smoke observed heartbeat, IDs 1–3, disconnect cleanup, offline 4–5 then replay
from 3, malformed/future cursors and Origin rejection. Initial harness stopped
before gates because Cargo was missing from PATH; corrected environment ran all
gates. First reviewer `wc_job_LF-i2IsQxFXO-obd` failed trim-before-control-check:
leading newline/trailing tab were silently normalized. Raw payload control checks
now precede space trim; 422 cases leave latest retained ID unchanged. Complete
replacement validation and `wc_job_fa8KJiTxGRuWIEDF` PASS preceded ninth-entry
promotion. No browser E2E replay here; release binary was 2,292,888 local raw bytes.

### Multi-Island Dashboard acceptance

M9g / YT-056, 2026-10-10: six server tests, 169 workspace tests/five doctests,
eight wasm32, 26 trees, release Chrome/Firefox and 136 unchanged Todo/Search
files PASS. Each island owns its versioned state/boundary, external loader,
module/WASM, listeners and mount/dispose lifetime. Server validates hashed
same-origin manifest URLs and file existence independently. Missing/malformed
whole manifest means strict script-free SSR; missing Counter WASM/invalid Filter
loader drops only that enhancement. Sibling DOM and readable panels survive.

Browser checks prove no initial WASM request, independent Counter then Filter
activation, root/input/value identity, Filter focus, delayed Counter retirement,
detached inertness and one-listener remount. Invalid Filter version fails locally;
restoring it binds from live input. Early harness fixes addressed synthetic focus,
`wasm` versus `WebAssembly` query text, favicon 404 and Firefox expression wrapping;
deterministic activation uses the documented local yetlet-activate event before
real focus/input assertions. First reviewer `wc_job_63A__nJFK3BgNWPk` failed missing
whole-manifest **browser** evidence despite native tests. Added missing/malformed
server runs proved HTTP200/default CSP/both panels/zero scripts/globals/requests
in both browsers. Second `wc_job_6pk1_UnsnH8WR4Ns` required actual retained new
exit-0 logs, not scripts alone. Exact unchanged scripts reran, logs retained;
final `wc_job_OMstvGrtDiKZ5p6C` PASS preceded tenth-entry promotion.

Dashboard digest `1eccb326477b59ae`: Counter 166,999 raw/63,710 gzip, Filter
177,694/68,753, CSS 416/279; server 2,635,496 raw bytes. Gzip is local estimate.
Separate companion builds never overwrite production dist. Native server excludes
browser companions; companions exclude server/Session/DB. No common state,
general hydration/resumability, RPC or browser-support guarantee is implied.

### Auth Notes acceptance

M9h / YT-057, 2026-10-10: twelve focused tests, 181 workspace tests/five doctests,
27 trees, eight wasm32 and 136 unchanged production files PASS. Two cookie-jar
localhost smoke exercised login/rotation, per-user CRUD/403 IDOR, CSRF and logout;
server was reaped. Synthetic OS-random 256-bit codes are printed only to the
local terminal, never SSR/browser/Debug secrets. Fixed-shape constant-time compare
uses a dummy unknown-user value. Login verifies anonymous CSRF and replaces/
rotates ID/CSRF/state; logout rotates to fresh anonymous state. Every handler
checks current server Session user and server-owned note ownership. Note storage
is bounded process-local, separate from Session; unknown client IDs are not adopted.
Secure-cookie defaults remain, with insecure HTTP allowed only after loopback
admission. No passwords, IdP, brute-force protection or reusable auth provider is
qualified. Credentials and persistent security require their own production design.

Initial Clippy unused code was removed; an apostrophe test corrected its expected
escaped `&#39;`. An oversize test exposed collapsed BytesRejection status; source
preserves 413 instead of weakening it. Evidence initially expected literal codes
in Python too; correction recorded dynamic smoke construction and absence of test
fixtures from optimized binary. Security-focused reviewer
`wc_job_EgSLng_VafIvs3ju` PASS preceded eleventh-entry promotion. Native HTTP
cookie/form evidence was used, not new browser E2E or penetration testing.
Release binary was 2,547,064 local raw bytes; effective scoped source review is
not production security assurance.

### Modular Shop acceptance

M9i / YT-058, 2026-10-10: nine tests, 190 workspace tests/five doctests, 28 trees,
eight wasm32 and 136 identical production files PASS. Explicit Identity/Catalog/
Cart/Orders modules use synthetic-code Session rotation/CSRF and app-owned direct
rusqlite. BEGIN IMMEDIATE atomically writes inventory/lines/order; Session-CSRF
plus cart-revision idempotency prevents a committed checkout from decrementing
twice if Session cleanup fails. Tests cover rollback after line write failure,
last-stock one-winner concurrency, persistence, owner 403 and stale cart.
Disabling Orders removes routes/navigation/checkout; invalid module combinations
fail construction. Three-process loopback smoke verifies checkout/restart/Bob403
and disabled Orders. There is no plugin/DI/Controller/ORM, real payment gateway,
production commerce, backup/encryption or multiprocess inventory claim.

An evidence allowlist initially forgot SQLite CRUD is another intended DB
consumer; correcting that assertion did not change runtime. First reviewer
`wc_job_a_WnhFSftzNoYYgf` reproduced readiness-failure child/log leakage in smoke
start(). Cleanup now terminates/reaps, bounded-kills/reaps if needed, closes the
stream and rethrows the original exception. Two mocked regressions and real
three-process smoke passed; unchanged Rust's retained 190-test matrix remained
valid. Repeat `wc_job_zbWS7OSbUtaKzYcn` PASS preceded twelfth-entry promotion.
Browser E2E was not replayed in this native-only stage; server release binary was
5,062,464 raw local bytes.

### DX audit and final freeze

M8d / YT-047 (2026-10-09) measured Todo/REST/Cart source shape, not complexity,
quality or performance: application packages 4/1/1, production nonblank Rust lines
1,102/551/424, tests 536/417/200, async functions 10/8/7, routes 4/3/7, direct
dependency names 16/6/6. Todo includes transitive app-owned UI. Optional provider
implementation was counted separately, not charged as application authoring.
32 sample/provider tests and four checker tests passed; reviewer
`wc_job_lZSDSBgMKWu1X5X7` PASS supported **no runtime API change**. The 11–14-line
native bootstraps, differing JSON/security policies and Cart-only forms did not
justify a hidden run helper, generic error/form engine, shared Origin policy,
Controllers, DI or a required layout. Future promotion needs convergent consumers
and explicit TLS/shutdown/security requirements, not line-count rank.

M8e's native-only stale-destroy repair and M9j's actual full twelve-sample,
170-declaration final freeze are in the [qualification matrix](QUALIFICATION-MATRIX.md#sample-and-api-freeze-evidence)
and [API review scope](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits).
Original independent reviews were role-directed local Codex Jobs, not native
managed agent dispatch or formal assurance. Each FAIL/operational no-result remains
separate from subsequent PASS. Historical release blockers did not disappear
because samples passed; today's accepted SDK/CLI publication records govern
current distribution, while production qualification remains open.
