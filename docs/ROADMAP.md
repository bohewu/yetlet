# Yetlet Roadmap — Evidence-Gated Milestones

**Status:** Draft, last reviewed 2026-10-09.

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
