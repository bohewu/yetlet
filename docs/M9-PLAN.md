# M9 — Sample Expansion and API Contract Qualification Plan

**Planning task:** YT-049 / M9p
**Date:** 2026-10-09
**Status:** Planning independently reviewed (`wc_job_fr93PI1YLOtR3KjE` PASS); implementation proceeds through the per-stage gates recorded below and in the Backlog.
**Baseline:** M8 milestone commit `742e0d4909cc9b132d81b78e12bbb66d5bf60f57`, locally complete at prototype scope.
**Related:** [Roadmap](ROADMAP.md), [Backlog](BACKLOG.md), [Samples](SAMPLES.md), [Extensions](EXTENSIONS.md), [Release readiness](RELEASE-READINESS.md), [ADR 0005](adr/0005-opt-in-capability-modules.md), [ADR 0006](adr/0006-handler-first-mvc-optional.md), [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

## 1. Purpose and decision boundary

The M8 applications demonstrate that small native JSON, SSR/form and SSR-plus-WASM applications can be authored with ordinary Rust handlers and separately selected capabilities. M9 deliberately challenges those contracts with new application shapes **before** making any public stability commitment.

- **Planning is its own gate.** YT-049 writes this plan, verifies documentation/dependency consistency, and obtains an independent read-only reviewer PASS. No M9 implementation may begin before that reviewer passes and the planning evidence is recorded.
- **Implementation is not authorized by planning alone.** The subsequent YT-050–YT-059 items are scoped proposals to be executed separately, sequentially, with task-specific tests and an independent reviewer after **every** item.
- **Preserve the same worktree.** No detached worktree, reset, clean, branch switch, commit, push, Git tag or release without a separate explicit instruction. Existing M8 code and the verified sample catalog remain unchanged by this planning slice.
- **Prove friction rather than introduce magic.** Handler-first, component-first, SSR-first and Axum/Tower interoperability remain the default. No mandatory MVC/Controller, DI container, plugin registry, second router, hidden request/event transport, generic VDOM, signals/effects runtime or form engine.
- **Separate app, optional capability and framework boundaries.** A sample may bring an application-level dependency without creating a Yetlet crate. Promote a reusable API only after independently reviewed repeated use and a measurable ergonomic or safety benefit.
- **Do not confuse tests with release qualification.** M9 does not automatically satisfy the license, supported MSRV/browser/semver, authenticated-production-security or external CI gates in [Release readiness](RELEASE-READINESS.md).

## 2. Canonical user-facing sample list

The **display order** preserves the agreed nine-sample list. The three supplementary samples follow it. This is not the implementation order.

| Display | Sample ID | Current state | Demonstrates |
|---|---|---|---|
| 1 | `hello-world` | M9a verified, reviewer PASS; separate minimal server-adapter example remains | Smallest copyable SSR-only application, no browser or optional feature |
| 2 | `todo` | Verified in M8 catalog | Shared SSR/browser UI, typed actions, keyed local updates and Search Island |
| 3 | `rest-api` | Verified in M8 catalog | Native JSON API, validation, typed HTTP errors, no WASM |
| 4 | `session-cart` | Verified in M8 catalog | Explicit Session, strict SSR forms, CSRF and cookie lifecycle |
| 5 | `auth-notes` | Planned; security-sensitive | Authentication versus authorization, private notes and identity isolation |
| 6 | `middleware-lab` | M9b verified; reviewer PASS | Global/group/route Tower middleware, `RequestContext`, short circuit |
| 7 | `file-server` | M9d verified; independent reviewer PASS | Static files, safe upload/download, streaming and response headers |
| 8 | `sse-notifications` | Planned | Optional SSE, bounded event streams, disconnect and reconnection |
| 9 | `modular-shop` | Planned; integration reference app | Explicit feature modules, catalog/cart/orders, selected capabilities |
| 10 | `forms-validation` | M9c verified; reviewer PASS after accessibility repair | SSR-only form input, errors, accessibility and PRG boundary |
| 11 | `sqlite-crud` | M9e verified; repeat reviewer PASS after schema repair | Direct SQL, transaction/persistence behavior, opt-in DB driver |
| 12 | `multi-island-dashboard` | Planned supplement | Multiple independently loaded/mounted/disposed browser islands |

M8 catalogued `todo`, `rest-api` and `session-cart`; M9a adds independently reviewed, runnable `hello-world` (reviewer `wc_job_Htk-Quo_aFExMJs3` PASS). Merely discussing a planned sample **must not** mark it `verified` or insert a non-runnable catalog entry. `hello-world` must become a distinct, user-facing sample or be deliberately cataloged with a verified supported command; the existing `yetlet-server-axum --example minimal` is not automatically a verified catalog application.

## 3. SQL-first data access policy: no Yetlet ORM

**Decision for M9: Yetlet does not ship, build or require an ORM.** A `sqlite-crud` sample demonstrates persistence and application integration, **not** a framework ORM or a mandatory data access layer.

1. The application's SQL queries, parameters, migrations/schema fixture, transactions and row-to-struct mapping stay explicit and close to the application's feature code.
2. Select a mature external **opt-in** SQLite driver/query library **when implementing YT-054**, after a tiny comparative spike of integration ergonomics and synchronous versus asynchronous execution. Candidates include `rusqlite` (direct SQL/row mapping) and `sqlx` (SQL-first typed queries/query-as, with explicit runtime/feature selection); neither is preselected by this planning document.
3. A lightweight Dapper-like **query-to-typed-struct** experience is an acceptable *application-level goal*, not permission to implement a new ORM, active-record model, query DSL, migration engine, implicit `DbContext`, universal repository/Unit of Work or DI system inside Yetlet.
4. An existing third-party ORM may be chosen later by an individual application only with explicit maintainer acceptance. It remains **strictly opt-in and external**; it never enters `yetlet-core`, `yetlet-server-axum`, a simple SSR app or a browser/WASM dependency graph by default.
5. Use parameterized statements, explicit transaction/rollback and migration ownership. Validate that failed writes do not partially commit, and that app state/DB handles are passed via ordinary Axum `State` or a clearly owned native handle. Avoid hidden DB initialization per HTTP request.
6. If the selected driver is blocking, choose a bounded blocking-task/worker boundary instead of holding an async executor thread; if asynchronous, disclose its runtime and pool cost. Do not guess at the faster choice without measurements.
7. The database is a server-only, application-owned capability. No credentials, connections, pool handles, SQL code or schema internals are serialized into browser hydration state. No network or cloud database is required by the default sample.
8. Record cargo-tree and build/bundle isolation: no DB driver, ORM or new transitive feature in `hello-world`, existing Todo/REST/Cart, `yetlet-core`, server adapter or unrelated browser entries because of the SQLite sample.

This is a concrete architectural **non-adoption** decision for M9 planning. A future reusable data API requires a separately reviewed ADR and evidence from multiple genuinely convergent consumers. `modular-shop` may reuse an explicitly chosen application data adapter without promoting it to Yetlet core.

## 4. Proposed implementation and review sequence

Every row is a separate bounded work item; the sample display order in section 2 remains independent.

| Gate | Work item | Outcome | Required review |
|---|---|---|---|
| M9p / YT-049 | Plan and review M9 | Linked scope, task DAG, sample matrix, SQL-first opt-in rule | **Independent planning reviewer PASS before any code** |
| M9a / YT-050 | `hello-world` | Minimal cataloged SSR baseline and no optional dependencies | Independent read-only reviewer |
| M9b / YT-051 | `middleware-lab` | Validate native middleware/RequestContext composition | Independent read-only reviewer |
| M9c / YT-052 | `forms-validation` | Stateless SSR form validation and safe PRG demonstration | Independent read-only reviewer |
| M9d / YT-053 | `file-server` | Static and bounded upload/download boundaries | Independent read-only reviewer |
| M9e / YT-054 | `sqlite-crud` | Driver comparison, selected opt-in direct SQL, transactional CRUD | Independent read-only reviewer |
| M9f / YT-055 | `sse-notifications` | Native HTTP event stream, bounded consumers, cancellation | Independent read-only reviewer |
| M9g / YT-056 | `multi-island-dashboard` | Multiple independent SSR/browser islands and asset isolation | Independent read-only reviewer |
| M9h / YT-057 | `auth-notes` | Authn/authz, per-user isolation, validated session/cookie/CSRF | Independent read-only **security-focused** reviewer |
| M9i / YT-058 | `modular-shop` | Larger compositional app using proven optional boundaries | Independent read-only reviewer |
| M9j / YT-059 | Final API friction + compatibility audit | Public API inventory/classification, full regression and evidence freeze | Independent final reviewer |

The critical path is YT-048 → YT-049 → YT-050 → YT-051 → YT-052 → YT-053 → YT-054 → YT-055 → YT-056 → YT-057 → YT-058 → YT-059. YT-057 additionally relies on the existing M8c Session evidence; YT-058 also relies on the application composition and database/identity/browser work already reviewed. Tasks may be reprioritized with an updated, reviewed roadmap; do not silently skip a required reviewer.

## 5. Sample-specific acceptance contracts

### M9a / YT-050 — `hello-world`

- Copyable build/run command from the repository root; deterministic SSR route, escaped dynamic text and strict CSP without scripts or WASM.
- No `yetlet-session`, request-ID, DB, auth, keyed/reactive, browser tooling or provider startup. Baseline app binary/dependency tree, plus no unnecessary setup code.
- Native test and bounded HTTP smoke; catalog/README/source-marker/docs checks PASS.

### M9b / YT-051 — `middleware-lab`

- Demonstrate before/after `next`, reverse unwind, short-circuit skipping the handler, global versus group/route scoping and typed extension/state separation.
- Assert native 404/405 and streaming/failed-body behavior; show `RequestContext` request-only ownership and ordinary Tower `Layer` compatibility.
- No second middleware engine, default tracing/auth or implicit provider. Negative tests for leaked scope, double `next` assumptions and accidental body consumption.

### M9c / YT-052 — `forms-validation`

- No-JS SSR GET form + POST validation with field-level errors, escaped/untrusted submitted content, semantic labels/focus/error relationships and bounded form input.
- Demonstrate POST/Redirect/GET only for a successful **stateless/no-persistent-side-effect** action. Do not pretend CSRF is solved merely because there is no Session.
- If the design changes to mutate persistent state or use credentials, block and add a separately reviewed CSRF/session policy rather than silently widening the sample.

### M9d / YT-053 — `file-server`

- Directly compose mature static-serving/HTTP facilities. Safe GET/HEAD/download behavior with content type, disposition and `nosniff` headers.
- Bounded upload as an explicit application-owned, local-only or properly authorized endpoint: body/multipart limits, filename canonicalization, path traversal/symlink defense, size/time/backpressure and interrupted-write cleanup.
- No arbitrary filesystem read, open directory listing, executable upload or global mutable file-serving middleware. Test malformed paths, missing files, oversized bodies and partial cleanup.

### M9e / YT-054 — `sqlite-crud`

- Before choosing dependencies, compare a direct lightweight SQLite API and a mature SQL-first typed-query option against handler ergonomics, driver size, blocking/async behavior and deployability. Record the choice.
- SQL/schema/migration fixture and CRUD use parameterized SQL, explicit transaction/rollback, unique constraints and typed row mapping; no Yetlet ORM, code generator or mandatory repository abstraction.
- Prove restart persistence with an isolated temporary DB, concurrent/conflicting write policy, failed transaction rollback and deterministic fixture cleanup.
- Verify absence of all DB driver/query dependencies from unrelated server/browser packages. The driver is a dependency of this app only.

### M9f / YT-055 — `sse-notifications`

- Use ordinary HTTP/SSE primitives, explicit content type, finite event IDs, optional resumable `Last-Event-ID` policy, heartbeat, disconnect detection and resource cleanup.
- Bounded subscriber queues/replay buffer; document drop/backpressure behavior and a reconnect test. No global live connection circuit, no automatic DOM event forwarding or mandatory WASM.
- Verify that SSR-only and REST samples retain their existing dependency/runtime behavior.

### M9g / YT-056 — `multi-island-dashboard`

- SSR page is readable before JavaScript. Multiple isolated islands have separate bundle ownership, state versions, mount/activation/dispose lifetimes and independent failure fallback.
- Activation of one island must not fetch, mutate or dispose its siblings. Test Chrome and Firefox identity, input focus, teardown/remount, failed/late async actions and missing/invalid manifests.
- Preserve unrelated built asset bytes or record an explicitly justified change; do not promote general hydration, resumability or a framework-global event circuit.

### M9h / YT-057 — `auth-notes`

- Choose and document an isolated, explicit authentication strategy before implementation; demonstration may use test-only fixture identities. No production credential/IdP claims without real qualification.
- Distinguish authentication, authorization, Session and storage. Test unknown/unauthenticated access, per-user read/write isolation, IDOR prevention and clear 401/403 behavior.
- If using cookie credentials, require secure cookie defaults, login/logout fixation rotation, server-side authorization checks and CSRF on state-changing requests. Password-based flows must use an established password-hashing library, not custom cryptography.
- Keep real secrets out of source/SSR/browser bundles; use synthetic users/data, loopback-only defaults or an appropriately protected environment. Any proposed `yetlet-auth` provider is **not automatically approved** and requires its own rationale and reviewer.

### M9i / YT-058 — `modular-shop`

- Larger, coherent reference app that composes distinct catalog, cart, order and identity features through ordinary Rust modules/route groups and explicitly passed state.
- Build on reviewed SQLite, Session/auth and islands as needed, but every extra dependency stays opt-in. Order/place/rollback and authorization/cross-user tests must cover realistic failure paths.
- Demonstrate independent module ownership and a disabled-module case without a mandatory plugin registry, DI container, MVC layout or general ORM.
- Do not treat a mock checkout/payment flow as a real financial transaction gateway or production commerce deployment.

### M9j / YT-059 — API compatibility and evidence freeze

- Inventory the actual documented public Rust exports and exercised integration contracts. Classify each as **Stable Candidate**, **Experimental** or **Internal** with evidence and known break risks.
- Stable Candidate is **not** a stability/semver guarantee; a future release decision must separately set MSRV, API/semver support, browser targets, license and independent security expectations.
- Repeat all relevant native, wasm32, Chrome/Firefox, SSR/CSP, API, Session/CSRF, SQL transaction, middleware, file/SSE/island and auth isolation tests. Record production artifact hashes, executable/dependency sizes and any bundle delta.
- Perform an independent final reviewer gate and freeze task-level evidence; do not mark public-release YT-020 Done.

## 6. Evidence and negative-test matrix

| Sample / boundary | Positive evidence | Mandatory negative/regression evidence | Isolation evidence |
|---|---|---|---|
| Hello | Native SSR + catalog | Escaping, invalid route, zero JS | No optional provider/browser/DB |
| Middleware | Ordering, scoped results | Short-circuit, 404/405, body/streaming errors | No scope leakage |
| Forms | SSR validation / PRG | Invalid/malicious input, overlong body | No JS/Session/DB for stateless fixture |
| Files | Static GET/HEAD, bounded upload/download | Traversal, symlink, oversize, abort | Local-only or explicit authorization |
| SQLite | Typed direct SQL CRUD, restart and rollback | Constraint conflict, injection-like input, failed transaction | DB libraries only in the app |
| SSE | Typed/finite stream, reconnect | Disconnect, queue overflow, malformed Last-Event-ID | No default event circuit |
| Islands | Independent SSR/activation/teardown | Missing assets, stale callbacks, sibling mutation | Independent bundles/deps |
| Auth Notes | Login state and per-user notes | Unauthenticated, IDOR, CSRF, fixation, invalid logout | Credentials/server-only and opt-in |
| Modular Shop | Composition and order flow | Disabled feature, failed order, cross-user leak | No default ORM/plugin/DI |
| Final freeze | Full matrix, exact artifacts | Missing evidence, stale docs, compatibility gaps | Core/provider/browser trees |

For **each** sample: one catalog entry only after it is runnable; English README with exact command and limits; native tests; localhost/Browser smoke as relevant; Cargo-tree inclusion/exclusion; documentation and `git diff --check` plus untracked whitespace; independent role-directed **read-only reviewer** whose issues must be fixed and re-reviewed before moving on. Record PASS/FAIL as actually observed. A reviewer job is not an independent production security audit.

## 7. Planning document reviewer gate (YT-049)

The plan itself is a reviewed deliverable, not a self-approved spec.

**Pre-review checks:** `python3 scripts/check_docs.py`, `python3 scripts/check_samples.py`, current Roadmap/Backlog DAG, exact nine original sample names and three supplements, English documentation and whitespace, unchanged `examples/samples.toml`, unchanged source/Cargo manifests, same `master` worktree/HEAD and empty index.

**Independent reviewer must inspect:**
1. scope and truthful M8-versus-planned sample status;
2. preservation of all 12 named samples and separable display/implementation order;
3. unique YT-049–YT-059 IDs, prerequisites and review gates;
4. SQL-first rule **without a built-in or mandatory ORM** and strictly opt-in third-party DB/mapper choices;
5. concrete security and fail-closed boundaries, especially Auth/Session/CSRF, uploads, SSE and isolated browser lifetimes;
6. no hidden commitment to new core APIs, package publication or production stability.

**Pass condition:** independent role-directed reviewer result `overall_status=pass`, no material findings or required followups; save the real Job ID and issue/fix history in [M9 planning verification](M9-PLANNING-VERIFICATION.md). If review fails, repair docs, rerun documentation gates, and dispatch a **new** read-only reviewer. If review cannot run, mark YT-049 Blocked and do not start YT-050.

**After PASS:** mark only YT-049 Done, add reviewer proof, and leave the implementation tasks Todo. This plan authorizes neither an ORM nor any M9 sample implementation without a separate user instruction.
