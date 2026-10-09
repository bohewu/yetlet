# Yetlet Roadmap — Evidence-Gated Milestones

**Status:** Draft, last reviewed 2026-10-08.

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

## Post-MVP research

| Track | Research question |
|---|---|
| Streaming SSR | Does the API need streamed suspense and a measured first-byte advantage? |
| Compile-time template IR | Are static slot macros necessary, or are direct reactive DOM bindings sufficient? |
| Resumability | Can stable IDs and serialized, constrained handler captures work in Rust? |
| Custom transport | What is the smallest truly pluggable action executor interface? |
| WebGPU renderer | Can a separate renderer measurably improve 10k–100k scene-node workloads? |
| Worker compute | Which computations can leave the main thread without compromising DOM correctness? |
| Tooling | When do HMR, bundler integration, and devtools justify their complexity? |

## Decision rule

For every proposed feature, identify its owner: kernel, renderer, transport, server adapter, or optional tooling. If ownership is unclear or responsibilities overlap, document the decision or run a bounded spike before adding core abstractions.
