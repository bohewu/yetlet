# Yetlet Roadmap — Evidence-Gated Milestones

**Status:** Draft, last reviewed 2026-10-10.

Milestones are sequenced by capability and dependencies rather than invented delivery dates. The [MVP specification](MVP.md) provides acceptance criteria. Each gate must retain real commands, results, and relevant architectural decisions.

| Milestone | Outcome | Backlog | Exit evidence |
|---|---|---|---|
| **M0 — Design baseline** | Specifications, architecture, ADRs, backlog, repository agent guidance | YT-001 | Linked docs, unique task IDs and dependencies, proposals distinguished from verified facts |
| **M1 — Native SSR foundation** | Cargo workspace, minimal shared components, safe HTML renderer, HTTP adapter | YT-002–YT-007 | `GET /` produces escaped HTML without JS; native tests pass |
| **M2 — Browser DOM/WASM** | Browser build, DOM mount/local state, shared components | YT-008–YT-010 | One component source compiles for both targets; local click updates DOM |
| **M3 — Selective islands** | External loader, asset manifest, island activation, bounded initial state | YT-011–YT-013, YT-017 | `/search` renders complete SSR, loads only the island on interaction, never forwards raw events |
| **M4 — API/action integration** | Typed actions, HTTP executor, JSON API, strict CSP/security evidence | YT-014–YT-016, YT-018 | Clear local/remote boundary, correct API errors, working browser/CSP E2E |
| **M5 — MVP validation/docs** | Cross-target regression, browser smoke, baseline measurements, examples | YT-019–YT-020 | Evidence for [A01–A10](MVP.md#3-acceptance-matrix) and public limitations |
| **M6 — Handler-first API and optional extensions (locally complete)** | App/Page APIs, explicit module functions, independent Request ID proof and migrated Todo | YT-030–YT-034 | Native/Chrome/Firefox contracts PASS; release assets unchanged; measured latency comparison, no mandatory MVC or DI |
| **M7 — Bounded keyed DOM and local reactivity (locally complete)** | Measured strategies, promoted low-level contracts, production Todo keyed graph and final decision freeze | YT-025, YT-035–YT-043 | Equivalent output, ownership/disposal, cross-browser lifecycle, measured bundle cost, Search isolation, complete regression and repeat reviewer PASS |
| **M8 — Sample-driven application experience** | Standardized sample contract, API-only REST app, opt-in Session cart, DX audit and evidence freeze | YT-044–YT-048 | Copyable applications, pay-for-what-you-use dependencies, security/lifecycle evidence and per-stage reviewer PASS |
| **M9 — Sample expansion and API contract qualification (locally complete)** | Nine canonical sample names plus three supplements, SQL-first persistence, contract classification | YT-049–YT-059 | Twelve samples, 170-export inventory, complete regression and repeat final reviewer PASS |
| **M10 — Release engineering and production-security qualification** | Licensing, governance, compatibility, threat model, supply-chain and operational qualification | YT-060–YT-067 | Maintainer decisions, hosted controls, reproducible release evidence and external security review |

## Critical path

```text
YT-001
  └─ YT-002 (workspace/toolchains)
       ├─ YT-003 (shared view)
       │    ├─ YT-004 (HTML renderer) → YT-005 (server adapter) → YT-006 (SSR demo)
       │    └─ YT-008 (browser build) → YT-009 (DOM renderer) → YT-010 (shared component)
       └─ YT-007 (native build/test CI)
YT-010 → YT-011 (loader) → YT-012 (islands) → YT-013 (search demo)
YT-003 → YT-014 (actions) → YT-015 (HTTP executor) → YT-016 (API integration)
YT-012 → YT-017 (state manifest)
YT-013 + YT-016 + YT-017 → YT-018 (CSP test)
YT-006 + YT-010 + YT-013 + YT-016 + YT-018 → YT-019 (E2E/performance)
YT-019 → YT-020 (release documentation)
YT-027 + YT-016 → YT-030 (thin App facade) → YT-031 (handler/results DX)
YT-030 + YT-026 → YT-032 (opt-in contract) → YT-033 (optional proof)
YT-030 + YT-031 + YT-033 → YT-034 (Todo handler-first demo)
# M6 is not blocked by separate public-release ticket YT-020.
YT-009 → YT-025 (M7a strategy benchmark) → YT-035 (authoring prototype)
YT-035 → YT-036 (local reactive graph) → YT-037 (keyed lifecycle)
YT-036 + YT-037 → YT-038 (SSR/WASM regression)
YT-038 → YT-039 (binding contracts)
YT-039 → YT-040 (local graph) + YT-041 (keyed DOM)
YT-040 + YT-041 → YT-042 (production Todo adoption) → YT-043 (final freeze)
YT-043 → YT-044 (sample contract) → YT-045 (REST) → YT-046 (Session cart)
YT-046 → YT-047 (DX audit) → YT-048 (M8 freeze)
YT-048 → YT-049 (review M9 plan) → YT-050 (Hello) → YT-051 (Middleware)
YT-051 → YT-052 (Forms) → YT-053 (Files) → YT-054 (SQLite)
YT-054 → YT-055 (SSE) → YT-056 (Multi-Island) → YT-057 (Auth)
YT-057 → YT-058 (Modular Shop) → YT-059 (API contract review/freeze)
YT-059 → YT-060 (license/policy) → YT-064 (release supply chain)
YT-059 → YT-061 (repository governance)
YT-059 → YT-062 (SemVer/document metadata) → YT-064
YT-057 + YT-059 → YT-063 (production threat model)
YT-059 → YT-065 (scheduled compatibility matrix)
YT-046 + YT-054 + YT-057 + YT-059 → YT-066 (provider operations)
YT-063 + YT-064 + YT-066 → YT-067 (external security review)
```

Prove native SSR before committing to an elaborate browser view IR. Avoid simultaneously designing DOM, SSR, islands, and GPU abstraction layers before the smallest dual-target implementation works.

## Gate checklist

### M0 — Documentation-only design baseline

- [x] `README.md`, `AGENTS.md`, and `docs/*` exist with valid relative links.
- [x] All backlog IDs are unique; every work item has acceptance evidence requirements and dependencies.
- [x] ADRs distinguish proposed choices from tested behavior.
- [x] `python3 scripts/check_docs.py` passes across tracked and untracked documentation, including links and whitespace.

**M0 evidence (2026-10-08):** `python3 scripts/check_docs.py` PASS (initially 9 documents, 25 backlog items, dependency DAG). `git diff --check` PASS for tracked changes; untracked documents were checked separately by the script.

### M1 — Native SSR foundation

- [x] `cargo test --workspace --all-targets --locked` passed at the M1 baseline (11 tests).
- [x] `cargo build -p yetlet-todo-server --locked` produced a binary started during an HTTP smoke run.
- [x] `GET /` returns escaped content with no mandatory browser runtime; strict CSP is active.
- [x] `yetlet-server-axum` uses the established Axum/Tower stack and `/api/ping` is a separate JSON route.

**M1 evidence (2026-10-08):** Rust 1.99, 11/11 tests, format/Clippy and native plus WASM shared UI compile PASS. A bounded localhost smoke returned 200 HTML for `GET /` and 200 for `/healthz`, without script/WASM references or unsafe-inline CSP. HTML was 423 bytes at that snapshot; **not** a performance baseline. CI YAML was created but had not run on GitHub at that time.

### M2 — Browser renderer

- [x] Browser WASM mounted and updated actual DOM in headless Chrome, not merely compiled.
- [x] The same `todo_list` Rust source rendered identical initial HTML in native SSR and browser DOM.
- [x] Two local clicks updated state without network requests or WebSockets.

**M2 evidence (2026-10-08):** 16/16 Rust tests and Clippy PASS at the M2 gate. `scripts/build_browser.py` generated a hashed manifest and browser modules. The headless Chrome `scripts/smoke_browser.py` confirmed byte-identical SSR/DOM TodoList, state changes, and no remote event circuit. See [M2 verification](M2-VERIFICATION.md). Remote GitHub Actions execution was not verified.

### M3 — SSR islands and bounded state v1

- [x] `GET /search` renders a complete usable SSR input and result list without eagerly importing Search WASM.
- [x] Small external `search-loader.js` and versioned assets activate Search WASM on local interaction.
- [x] Input events remain in the browser; initial SSR results/input/heading are preserved.
- [x] `YT-017`: Explicit allowlisted, size-bounded Search Island v1 JSON state, escaped as an HTML attribute. Invalid versions fail closed and preserve readable SSR markup.

**M3 evidence (2026-10-08):** Native/WASM builds, `scripts/build_browser.py`, and headless Chrome `scripts/smoke_browser.py` verified lazy loading, DOM preservation, local search/empty/reset, versioned state acceptance, and incompatible-state fallback. See [M3 verification](M3-VERIFICATION.md). YT-011–YT-013 and YT-017 are complete at **prototype** scope; generalized resumability is not claimed.

### M4 — Explicit actions, API, and security

- [x] `YT-014`: Typed `Command`/`ActionExecutor` and injected `LocalExecutor` with unit and browser event-boundary evidence.
- [x] `YT-015`: Explicit same-origin-path `HttpExecutor` sends only semantic JSON commands; no raw DOM-event forwarding or automatic mutation retries.
- [x] `YT-016`: `GET/POST /api/todos` in-memory snapshot API validates body, command, and error status; failed mutations preserve state.
- [x] `YT-018`: Prototype strict CSP and browser checks, typed HTTP 4xx responses, nonmatching Origin rejection, and bounded JSON body handling.

**M4 evidence (2026-10-08):** Native Cargo/Clippy, browser WASM build and headless Chrome E2E demonstrate the local/HTTP action separation, one typed Create POST, server snapshot rendering, visible typed 422 error, and regression of SSR/Islands. See [M4 verification](M4-VERIFICATION.md). This is an **anonymous single-origin HTTP demo**, not an authenticated production security review.

### M4.1 — User-approved HTTP pipeline integration

This bounded extension comes from the Middleware Pipeline & RequestContext handoff and preserves the existing implementation sequence. See [ADR 0004](adr/0004-request-context-and-middleware.md) and [HTTP pipeline](HTTP-PIPELINE.md).

- [x] `YT-026`: Request-only context, request-local typed extensions and distinct application state.
- [x] `YT-027`: Native Tower pre-routing global layer, nested groups, route layers and explicit 404/405/error semantics.
- [x] `YT-028`: Actual API/SSR integration, streaming/request-isolation tests and browser boundary regression.

**M4.1 evidence (2026-10-08):** 46/46 tests, strict Clippy/format, English docs, browser compile and one headless regression PASS. Two browser WASM artifacts remain byte-identical. Native overhead/production assurance remain unmeasured; see [HTTP pipeline](HTTP-PIPELINE.md).
- [ ] Deferred `YT-029`: Optional providers/deadlines and measured native overhead; not required to close P0.

### M5 — MVP release-evidence freeze

- [x] Reproducible **local** results for A01–A09, mapped to actual Rust/Chrome/Firefox tests.
- [x] Release-mode baseline measured for local SSR first-byte, HTML, gzip artifact estimates, Browser/WASM and Search Island readiness; environment and limitations recorded.
- [x] Documentation distinguishes locally tested prototype capabilities from production/public release gates.
- [ ] **Public-release authorization:** license, supported MSRV/API/browser matrix, authentication/security posture, and independent qualification remain open.

**M5 local evidence (2026-10-08):** [M5 acceptance and measurements](M5-VERIFICATION.md) documents A01–A10, release Rust/Chrome 155, Firefox 157 physical-keyboard E2E, 30 SSR timing samples, 5 Chrome navigation/activation samples, and asset sizes. `YT-019` is locally Done; `YT-020` is Blocked for publication, with explicit owners in [Release readiness](RELEASE-READINESS.md). GitHub CI has not executed the new M5 workflow changes.

### M6 — Handler-first developer API and opt-in modules (locally complete)

**Goal:** Let developers build a small SSR site or typed JSON API with ordinary Rust handlers and components, without mandatory MVC Controllers, source layouts, model binders, DI containers or result hierarchies. Expose a **thin Axum/Tower facade**, not a second router. Keep all optional capabilities explicitly installed and browser-safe. See [ADR 0006](adr/0006-handler-first-mvc-optional.md) and [ADR 0005](adr/0005-opt-in-capability-modules.md).

- [x] `YT-030 / M6a`: Thin `App<S>` facade supports ordinary GET/POST handlers, native `MethodRouter`, nested groups, scoped middleware, explicit state and raw Router escape hatches.
- [x] `YT-031 / M6b`: Page and async View route convenience; native typed inputs/results/rejections, redirects and errors; RequestContext remains optional.
- [x] `YT-032 / M6c`: configure/try_configure execute typed setup functions once; no plugin registry, service locator or mandatory provider dependency.
- [x] `YT-033 / M6d`: Independent yetlet-request-id crate proves opt-out, scoping, error/short-circuit/concurrent behavior and core/browser dependency isolation.
- [x] `YT-034 / M6e`: Todo migration preserves SSR, API and middleware contracts; Chrome/Firefox regression and release before/after measurement completed. No zero-overhead performance claim.

**M6a local evidence (2026-10-09):** Five App Builder integration tests and a compiling rustdoc example passed; the existing RequestContext/Tower integration remains intact. [M6a verification](M6A-VERIFICATION.md) documents API surface, native semantics and remaining gaps. This is neither the M6b result API nor a performance/production release claim.

**Complete M6 local evidence (2026-10-09):** 66 workspace tests and 4 doctests, strict Clippy, native/minimal/wasm32 builds, Chrome 155 and Firefox 157 regressions passed. Release asset hashes and HTML sizes are unchanged. The same-session native first-byte median changed from 0.019 to 0.033 ms; historical M5 was 0.034 ms. See [M6 verification](M6-VERIFICATION.md) for unrounded methodology/limits and [Extensions](EXTENSIONS.md) for the actual composition contract. Production/release gates remain separate.

**Deferred:** MVC-specific helpers are optional future choices, **not required** for M6; so are Controller inheritance, convention discovery, `#[controller]` macros, full template DSL, Session/Authentication/Redis/Cache providers, a dynamic Plugin Manager and a mandatory DI container.

### M7 — Component authoring, keyed DOM and local reactivity (locally complete)

- [x] `YT-025 / M7a`: Isolated release benchmark compares current subtree replacement, positional View patch, keyed direct DOM and static template slots with identical seeded workloads.
- [x] `YT-035 / M7b`: Isolated keyed/static-slot authoring API covers native SSR, identity preflight, typed text/attributes, one conditional/local event, keyed structure and explicit disposal.
- [x] `YT-036 / M7c`: Explicit owner plus weak handles/subscriptions, FIFO batching, bounded re-entrant passes, fail-closed errors and local keyed-render commits without remote events.
- [x] `YT-037 / M7d`: Controlled row input, keyed insert/remove/reorder, focus/selection and listener cleanup verified in Chrome 155 and Firefox 157.
- [x] `YT-038 / M7e`: Complete native/SSR/WASM/CSP/bundle/performance regression and freeze a no-decision API classification.
- [x] `YT-039 / M7f`: Promote renderer-neutral component/instance/key/slot and inert-attribute contracts into `yetlet-core`; retain fixed authoring as experimental.
- [x] `YT-040 / M7g`: Extract the proven synchronous strong-owner/weak-handle local graph into an optional production crate.
- [x] `YT-041 / M7h`: Implement a general keyed DOM reconciliation and explicit binding/listener ownership primitive.
- [x] `YT-042 / M7i`: Migrate the production Todo Browser local list path and prove Chrome/Firefox identity/lifecycle behavior.
- [x] `YT-043 / M7j`: Run the final full regression, bundle/performance comparison, architecture decision and evidence freeze.

**M7a local evidence (2026-10-09, retained report `2026-10-09T02:02:22+00:00`):** 200 rows, 500 updates, 10 measured rounds in Chrome 155. Median total round: subtree replacement 213.500 ms; positional View patch 41.700 ms; keyed direct DOM 2.000 ms; static template slots 1.900 ms. All strategies produced equivalent final states. Only keyed/slot prototypes preserved the untouched keyed row after reorder; whole replacement also lost owned input identity/focus. Production Browser/Search digest remained `3f3c5f074478c525`. See [M7a verification](M7A-VERIFICATION.md). This is architecture evidence, not a selected production renderer or framework comparison.

**M7b local evidence (2026-10-09):** Four native authoring/SSR tests, one compiling rustdoc example, wasm32/release build and an isolated Chrome 155 E2E passed. Mismatched instance/key or actual child-node/slot shape fails before binding; successful bind preserves SSR root/row/label/text identity, handles local typed events and conditional insertion/removal, preserves keyed rows across reverse/append/remove, and disposes listeners explicitly without stale handles affecting newer owners. Production digest remains `3f3c5f074478c525`. See [M7b verification](M7B-VERIFICATION.md).

**M7c local evidence (2026-10-09):** Thirteen reactive-core tests plus the existing four M7b tests and one doctest pass. The isolated Chrome fixture verifies a three-message/one-commit batch, deferred re-entrant dispatch, explicit observer/owner disposal, recoverable reducer failure, faulted renderer failure with keyed DOM identity preserved, and no HTTP/WebSocket circuit. Reviewer regressions prove nested subscription cleanup, caller iterator/state release and reducer/commit failure cleanup occur outside graph borrows without retaining replayable queue work; reducer-triggered owner disposal skips remaining messages/commit. See [M7c verification](M7C-VERIFICATION.md).

**M7d local evidence (2026-10-09):** The fixed keyed row adds one allowlisted controlled text input. Chrome 155 and Firefox 157 preserve its node identity, live value, focus and selection through keyed reorder; focused row removal and owner disposal remove input/click listeners so detached events cannot update the graph. Native SSR escaping/authoring checks and strict CSP remain intact. See [M7d verification](M7D-VERIFICATION.md).

**M7e local evidence (2026-10-09):** 86 workspace tests, 5 doctests, strict Clippy/format, four-package wasm32 checks, production Chrome 155 and Firefox 157 regressions, the complete M7a workload, and isolated M7b/M7c/M7d Chrome/Firefox release regressions pass. Production Browser/Search assets remain byte-identical at `3f3c5f074478c525`, and dependency graphs exclude both M7 experimental crates. The outcome is a no-decision evidence freeze: several small primitives are candidates for later extraction, but no current M7 API or renderer is promoted. See [M7e verification](M7E-VERIFICATION.md).

**M7f local evidence (2026-10-09):** `yetlet-core` now owns validated `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor` contracts. The isolated authoring prototype consumes those types while retaining its experimental fixed shape. Five core tests, seventeen prototype tests, one doctest, strict targeted Clippy and wasm32 compile pass. See [M7f verification](M7F-VERIFICATION.md).

**M7g local evidence (2026-10-09):** The exact M7c graph implementation and its thirteen ownership/batching/re-entrancy/failure/teardown tests now live in dependency-free `yetlet-reactive`. The keyed-binding fixture uses a compatibility re-export and retains four authoring tests plus one doctest. Strict targeted Clippy and native/wasm32 checks pass. See [M7g verification](M7G-VERIFICATION.md).

**M7h local evidence (2026-10-09):** Optional `yetlet-dom-keyed` now owns unique-order validation, keyed entry state, reconciliation and disposable listener lifetimes. The existing fixed-shape fixture consumes it and passes Chrome 155/Firefox 157 reorder, form-state and teardown regressions. Direct/feature-gated placement in `yetlet-dom` changed production bytes; the independent crate restored the canonical production digest `3f3c5f074478c525`. See [M7h verification](M7H-VERIFICATION.md).

**M7i local evidence (2026-10-09):** The production Todo Browser local list uses `yetlet-reactive` plus `yetlet-dom-keyed`, while the remote typed HTTP path and Search Island remain separate. Chrome 155 and Firefox 157 preserve local list/row identity, verify queued click→dispose retirement, ignore a delayed HTTP completion after disposal/remount, and retain SSR/CSP/API/Search regressions. The repaired production digest is `faebb0709dccd863`; the Browser runtime increases by 82,726 raw / 20,042 local-gzip bytes relative to M7e, while Search JS/WASM remain byte-identical after separate package builds. See [M7i verification](M7I-VERIFICATION.md).

**M7j final evidence (2026-10-09):** Replacement validation passes 91 workspace tests, 5 doctests, strict Clippy/format, six-package wasm32, debug/release Chrome 155 and Firefox 157 regressions including queued/in-flight lifetime races, the full M7a workload, isolated keyed lifecycle suites, dependency checks and a repeated release baseline. ADR 0007 selects only bounded identity/graph/keyed-DOM primitives and the Todo local-list adoption. Production digest is `faebb0709dccd863`; Search remains byte-identical. The first final reviewer found the lifetime P2 that this source repairs; repeat reviewer `wc_job_XZzdXEOIx6owj1o5` returned PASS with no issues or follow-ups. See [M7j verification](M7J-VERIFICATION.md).

### M8 — Sample-driven application experience

**Goal:** Validate that Yetlet's existing handler-first, component-first and opt-in capability model produces understandable applications with low accidental boilerplate. Samples are evidence, not excuses to add Controllers, DI, a macro DSL or a general client framework.

- [x] `YT-044 / M8a`: Standardize a versioned runnable-sample catalog, local READMEs and validation gate.
- [x] `YT-045 / M8b`: Add an API-only REST sample with no browser/WASM dependencies.
- [x] `YT-046 / M8c`: Add an SSR shopping cart with an explicit opt-in Session provider and bounded security contract.
- [x] `YT-047 / M8d`: Audit repeated boilerplate and dependency cost across the three applications.
- [x] `YT-048 / M8e`: Run the complete regression and freeze the M8 evidence/architecture decision.

**M8a local evidence (2026-10-09):** `examples/samples.toml`, sample-local READMEs and `scripts/check_samples.py` now validate repository-contained paths, exact launch commands, complete documented routes, companion/browser mappings and direct optional dependencies. The focused negative suite covers false declarations and malformed types; 10 Todo tests pass. Final reviewer `wc_job_vQ6pGDE0slh9gV8c` returned PASS with no issues. M8a changes no runtime behavior. See [M8a verification](M8A-VERIFICATION.md).

**M8b local evidence (2026-10-09):** The API-only Book CRUD sample passes 7 tests, strict Clippy, a real localhost smoke, catalog/docs gates and native dependency isolation. Independent reviews found routing-contract, authority-parser and no-Origin Host-validation defects; all material findings were repaired with focused regressions and complete M8b validation. Final reviewer `wc_job_d911MsTpbwT6fZY3` returned PASS with no issues or required follow-ups. See [M8b verification](M8B-VERIFICATION.md).

**M8c local evidence (2026-10-09):** The independent `yetlet-session` provider and native-only SSR cart implement random opaque identity, absolute expiry/capacity, fixation resistance, cookie lifecycle, session-bound CSRF, refresh persistence, bounded cart/form state and explicit absence from unrelated dependency graphs. Stage reviewers repaired atomic-rotation, stale-cookie and non-loopback-listener defects; M8e later found and repaired the stale-destroy removal-cookie variant. Final replacement validation passes 114 tests and 5 doctests. See [M8c verification](M8C-VERIFICATION.md).

**M8d local evidence (2026-10-09):** A reproducible audit compares application-owned Rust source, tests, routes, direct dependencies and optional capabilities across Todo, REST and Cart. Repeated patterns are either already handled, small native bootstrap or application/security-specific; no runtime API promotion is justified. Reviewer `wc_job_lZSDSBgMKWu1X5X7` returned PASS with no issues or required follow-ups. See [M8d verification](M8D-VERIFICATION.md).

**M8e final evidence (2026-10-09):** The complete workspace/native/browser/sample matrix, release artifact/dependency isolation, retained render/keyed fixtures, local baseline and M8d metrics pass. Final reviewer `wc_job_dospP8AM0wUmvHFd` found one stale-destroy cookie race; it was repaired with a deterministic two-request regression and 114-test replacement validation. Repeat reviewer `wc_job_J_MVNzFaVgTH1Gr1` returned PASS with no issues or required follow-ups. M8 is locally complete. See [M8e verification](M8E-VERIFICATION.md).

### M9 — Sample expansion and API contract qualification (implementation underway)

**Goal:** Challenge M8 application ergonomics using the original nine-sample catalog sequence plus three targeted supplements, then classify real API compatibility maturity. The [M9 implementation plan](M9-PLAN.md) is the authoritative scope and test matrix; M9p planning is complete, and implementation begins with bounded independently reviewed samples.

- [x] **YT-049 / M9p:** Documentation, unique task/dependency graph, SQL-first/no-ORM policy and independent read-only planning reviewer PASS before any M9 code.
- [x] **YT-050 / M9a:** Minimal native Hello World sample is cataloged and independently reviewed.
- [x] **YT-051 / M9b:** Middleware Lab with native global/group/route scopes, streaming/error validation and independent reviewer PASS.
- [x] **YT-052 / M9c:** Stateless SSR Forms Validation with field errors, native PRG, English language override and independent repeat reviewer PASS.
- [x] **YT-053 / M9d:** Bounded File Server, static/download headers, protected multipart, private storage, interrupted-write cleanup, dependency/asset isolation and independent reviewer PASS.
- [x] **YT-054 / M9e:** SQL-first SQLite CRUD using application-local `rusqlite`, complete regression and repeat reviewer PASS.
- [x] **YT-055 / M9f:** Bounded SSE Notifications with replay/cancellation/backpressure, complete isolation and independent repeat reviewer PASS.
- [x] **YT-056 / M9g:** Multi-Island Dashboard with separate bundles/state/lifetimes, missing/invalid fallback, Chrome/Firefox and final reviewer PASS.
- [x] **YT-057 / M9h:** Security-bounded Auth Notes with login/logout rotation, CSRF, per-user authorization/isolation and security-focused reviewer PASS.
- [x] **YT-058 / M9i:** Modular Shop explicit-module integration, transactional/idempotent orders, disabled-module behavior and repeat independent reviewer PASS.
- [x] **YT-059 / M9j:** [API contract inventory](API-CONTRACTS.md), Stable Candidate/Experimental/Internal classification, full native/browser/provider regression and repeat independent final reviewer PASS.

**Explicit data-access boundary:** No built-in Yetlet ORM, ORM runtime, mandatory repository/DbContext, query DSL, migration framework or hidden database provider. YT-054 evaluates existing SQL-first `rusqlite` or `sqlx`-style direct/typed query libraries as **app-selected optional dependencies**. Any third-party ORM is external and opt-in only, never a default dependency. See [M9 plan](M9-PLAN.md#3-sql-first-data-access-policy-no-yetlet-orm).

**Planning evidence (2026-10-09):** [M9 planning verification](M9-PLANNING-VERIFICATION.md) records 40 documents, 59 backlog items, the unchanged three-entry runnable catalog, SQL-first opt-in policy, complete task DAG and reviewer `wc_job_fr93PI1YLOtR3KjE` PASS with zero findings. At the planning freeze YT-049 was Done and YT-050–YT-059 were Todo; current per-stage progress follows below.
**M9a / YT-050 evidence (2026-10-09):** A distinct, cataloged native-only `yetlet-hello-world` proves deterministic SSR, an escaped dynamic route, strict CSP, native 404/405 and absence of optional providers or browser dependencies. Workspace tests 117/117, doctests 5/5, strict Clippy, real HTTP smoke, four-sample catalog and native release build pass; independent reviewer `wc_job_Htk-Quo_aFExMJs3` returned PASS. See [M9a verification](M9A-VERIFICATION.md).
**M9b / YT-051 evidence (2026-10-09):** The native-only Middleware Lab demonstrates global/group/method routing, short-circuit, typed RequestContext state, real third-party Tower interoperability and streamed/failing bodies. Workspace tests 124/124, doctests 5/5, strict Clippy, localhost smoke, five-sample catalog and optional-dependency isolation PASS; independent reviewer `wc_job_UnKflDaA8ZQHXCce` returned PASS with no issues. See [M9b verification](M9B-VERIFICATION.md).
**M9c / YT-052 evidence (2026-10-09):** Native SSR form validation and semantic error associations, escaped untrusted attributes, bounded 2 KiB bodies, strict CSP and no-side-effect 303 PRG pass 130 tests, five doctests and a real localhost smoke. First reviewer `wc_job_7D2rtHdGEfwW1Aet` found an inherited-language accessibility P2; a sample-owned `lang="en"` override on both main views and GET/error/success regressions repaired it. Repeat reviewer `wc_job_lGtk_ZIoAvK4F6PW` returned PASS with no required followups. No DB/ORM/Session/CSRF claim. See [M9c verification](M9C-VERIFICATION.md).

**M9d / YT-053 evidence (2026-10-09):** The bounded File Server passes 14 focused tests, 144 workspace tests, five doctests, full Clippy/format, six WASM packages and one localhost smoke including interrupted-upload cleanup. Browser/Search release digest remains `faebb0709dccd863`, all 136 retained assets byte-identical; file dependencies stay application-local. Independent read-only reviewer `wc_job_g8JSu80-Jseg6lmG` returned PASS with no findings, and runtime telemetry matched the saved `gpt-6-astra`/high configuration. The exact reviewed seventh catalog entry was promoted. See [M9d verification](M9D-VERIFICATION.md).
**M9e / YT-054 evidence (2026-10-09):** A bounded `rusqlite` versus compatible `sqlx` spike selected direct application-local `rusqlite`; explicit SQL/migration/transactions run behind a bounded `spawn_blocking` gate. Nine focused tests, 153 workspace tests, five doctests, three-process persistence/concurrency smoke, 22 dependency trees, six WASM packages and 136 byte-identical production assets pass. First reviewer `wc_job_Q3pBfUqPYGYNSIrD` found incomplete required-column validation; the fail-closed schema check and regressions were repaired, the full matrix rerun, and repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. The exact eighth catalog entry was then promoted. See [comparison](M9E-DATA-ACCESS-COMPARISON.md) and [M9e verification](M9E-VERIFICATION.md).
**M9f / YT-055 evidence (2026-10-10):** Native HTTP SSE uses an application-owned bounded Hub with finite IDs, a 16-event replay buffer, eight subscribers, four-event per-client queues, heartbeat comments and disconnect cleanup. Ten focused tests, 163 workspace tests, five doctests, real heartbeat/live/reconnect smoke, all 23 package trees, six wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control characters could be trimmed before validation; raw-input rejection and regressions repaired it, the full matrix reran, and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f verification](M9F-VERIFICATION.md).
**M9g / YT-056 evidence (2026-10-10):** Complete SSR hosts separately built Counter/Filter islands with independent manifests, versions, listeners and disposal. Six server tests, 169 workspace tests, five doctests, release Chrome/Firefox identity/focus/late-callback/failure-recovery and missing/malformed-manifest fallback pass. Eight wasm32 packages, all 26 dependency trees and 136 byte-identical production assets pass. Reviewers `wc_job_63A__nJFK3BgNWPk` and `wc_job_6pk1_UnsnH8WR4Ns` exposed missing browser fallback and retained-log evidence; both were repaired without runtime expansion, and final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS. See [M9g verification](M9G-VERIFICATION.md).
**M9h / YT-057 evidence (2026-10-10):** Loopback-only synthetic terminal codes authenticate Alice/Bob; `yetlet-session` separately owns opaque identity/CSRF; every note route enforces server-owned ownership. Login/logout rotate Session ID and CSRF. Twelve focused tests, 181 workspace tests, five doctests, a real two-user smoke, all 27 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. Security-focused reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS with no findings; the exact eleventh catalog entry was promoted. See [M9h verification](M9H-VERIFICATION.md).
**M9i / YT-058 evidence (2026-10-10):** Identity, catalog, Session cart and direct-SQL orders compose through ordinary Rust modules. Nine focused tests, 190 workspace tests, five doctests, a three-process restart/authorization/disabled-module smoke, 28 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_a_WnhFSftzNoYYgf` found a readiness-failure leak in the Python smoke harness; terminate/kill/reap/stream-close regressions repaired it, and repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. The exact twelfth catalog entry was promoted. See [M9i verification](M9I-VERIFICATION.md).
**M9j / YT-059 evidence (2026-10-10):** The [API inventory](API-CONTRACTS.md) classifies 170 public declarations/constants. Complete regression passes 190 tests, five doctests, 11 native smokes, eight wasm32 packages, three Chrome and three Firefox suites, 28 dependency trees and 136 byte-identical production files. First reviewer `wc_job_BzkzTPcKr4FKATTr` found omitted constants/errors and over-broad upstream reexport classification; both were repaired without runtime changes, and repeat reviewer `wc_job_QkQkovlBlvrhPxCE` returned PASS. Stable Candidates remain pre-SemVer evidence; YT-020 remains Blocked. See [M9j verification](M9J-VERIFICATION.md).

### M10 — Release engineering and production-security qualification

**Goal:** turn prototype evidence into explicit support, governance and production-security decisions without expanding Yetlet core or implying release approval.

- [x] **YT-060 / M10a:** Dual `MIT OR Apache-2.0` license, package metadata, security policy and maintainer-directed/no-external-PR contribution policy; independent reviewer PASS.
- [x] **YT-061 / M10b:** Collaborator-only PRs, protected `master`, required checks, SHA-pinned GitHub-owned Actions and Dependabot configured; unsupported private-state security services recorded; repeat reviewer PASS.
- [x] **YT-062 / M10c:** Minimal title/language document metadata, provisional compatibility policy, legacy generic compile fixtures, complete stable/MSRV validation and repeat reviewer PASS.
- [x] **YT-063 / M10d:** Product-neutral production edge/authentication threat model, private-cache/CSRF repair, documentation validation and repeat security-focused reviewer PASS.
- [x] **YT-064 / M10e:** Release design/preflight, actual license/source checker, 41 native CycloneDX SBOMs, 16 checksum tests and independent reviewer PASS; not signed/reproducible release activation.
- [ ] **YT-065 / M10f — In progress:** Scheduled/manual Ubuntu/Windows/macOS and Firefox workflow source plus four local Firefox suites and source reviewer PASS; new hosted/manual qualification pending.
- [ ] **YT-066 / M10g — In progress:** Actual local Session isolation/restart and two-process SQLite backup/restore/concurrency PASS with reviewer; distributed topology, HA, outage/load and production recovery remain unqualified.
- [ ] **YT-067 / M10h — Blocked:** External assessment handoff prepared; no appointed auditor, approved immutable release/deployment or completed distributed gate. Local document review does not satisfy an external audit.

**Post-M9 hardening evidence (2026-10-10):** [CI/security review](CI-SECURITY-REVIEW.md) records the first hosted CI failure, bounded browser-harness repair, full-SHA Action pins, read-only permissions, patched `RUSTSEC-2026-0009`, Rust 1.88 verification and the remaining GitHub/security/release gaps. GitHub Actions run `38024582602` completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`, including both required jobs and all Chrome release suites.

**M10 sequencing decision (2026-10-10):** The maintainer selected `MIT OR Apache-2.0` and a maintainer-directed model that does not accept public pull requests. YT-060/YT-061 now implement those policies without changing repository visibility. YT-062 retains the tiny/handler-first contract: title plus validated root language only, no arbitrary head manager, CSP DSL, DI, MVC or plugin subsystem. See [M10a](M10A-VERIFICATION.md), [M10b](M10B-VERIFICATION.md), and [M10c planning verification](M10C-PLANNING-VERIFICATION.md).

**M10a/M10b evidence (2026-10-10):** All 28 workspace packages carry `MIT OR Apache-2.0` and the canonical repository URL; all 19 examples are nonpublishable, and all nine library package archives contain both license texts. Reviewer `wc_job_MPF4l2t8JaJXBguv` returned PASS. GitHub remains Private with collaborator-only PR creation, squash-only merging, admin-enforced protected `master`, both required CI checks, GitHub-owned SHA-pinned Actions, read-only workflow tokens and Dependabot alerts/security fixes. First governance reviewer found missing check-run evidence and stale monitoring wording; both were repaired, and repeat reviewer `wc_job_lu2WynqvKYemLlmd` returned PASS. Public-only PVR/secret-scanning/CodeQL activation remains a visibility-transition checklist item.

**M10c / YT-062 evidence (2026-10-10):** Legacy page/render signatures remain; `LanguageTag`, `DocumentMetadata`, metadata-aware render functions and one compile compatibility fixture add explicit root-language ownership. Forms Validation adopts root `lang="en"` and removes its nested workaround. First reviewer `wc_job_VWOJoR-lV3PAyS_-` found a narrowed `Page::new` generic bound; the exact `Into<Cow<'static, str>>` contract and borrowed/owned generic regression were restored, complete stable/MSRV evidence reran, and repeat reviewer `wc_job_rz9Ci3gWOLfLKdqN` returned PASS. The bounded API remains Experimental; see [M10c verification](M10C-VERIFICATION.md).

**M10d / YT-063 evidence (2026-10-10):** Planning reviewer `wc_job__wg1bHF6DG3ztjjg` passed. The [Threat Model](THREAT-MODEL.md) defines a product-neutral TLS edge, forwarding-header, OIDC, Session/CSRF, authorization, rate-limit, CORS, secrets, telemetry and availability contract while keeping those capabilities external/opt-in. First completed reviewer `wc_job_UPiINEJpZ28tLFyl` found missing private-cache isolation and an overbroad CSRF-token prohibition; both were repaired, documentation validation reran, and repeat reviewer `wc_job_AlEmAzdabqC_tuHv` returned PASS. The document remains requirements evidence, not production certification; see [M10d verification](M10D-VERIFICATION.md).

**Remaining M10 execution evidence (2026-10-10):** [Approved plan](M10-REMAINING-PLAN.md), [release preflight](M10E-VERIFICATION.md), [compatibility matrix](M10F-VERIFICATION.md), [local operations](M10G-VERIFICATION.md) and [external audit handoff](M10H-VERIFICATION.md) keep executed evidence separate from unperformed hosted/distributed/external gates. No runtime dependencies, core subsystems or ORM were added.

## Post-MVP research

| Track | Research question |
|---|---|
| Streaming SSR | Does the API need streamed suspense and a measured first-byte advantage? |
| Compile-time template IR | M7b–M7e prove one fixed shape; M7f–M7i promote only low-level contracts and one production keyed-list path. General trees remain required before any macro decision |
| Resumability | Can stable IDs and serialized, constrained handler captures work in Rust? |
| Custom transport | What is the smallest truly pluggable action executor interface? |
| WebGPU renderer | Can a separate renderer measurably improve 10k–100k scene-node workloads? |
| Worker compute | Which computations can leave the main thread without compromising DOM correctness? |
| Tooling | When do HMR, bundler integration, and devtools justify their complexity? |

## Decision rule

For every proposed feature, identify its owner: kernel, renderer, transport, server adapter, or optional tooling. If ownership is unclear or responsibilities overlap, document the decision or run a bounded spike before adding core abstractions.
