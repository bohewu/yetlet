# Yetlet Backlog

**Status:** Initial backlog established 2026-10-08; updated as evidence becomes available.

**Source of work:** This document records MVP tasks and their dependencies. [Roadmap](ROADMAP.md) tracks milestone gates; [MVP](MVP.md) defines acceptance criteria. `Done` means verified evidence exists, not merely that code was written.

Priority: `P0` is the critical path, `P1` is required for the MVP, and `P2` covers later research. Sizes `S/M/L` indicate relative complexity, not promised delivery time. Update status with test results, commands, or a relevant change reference.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-001 | P0 | S | M0 | Repository docs / ADR / roadmap / backlog | — | `python3 scripts/check_docs.py` PASS (2026-10-08) | Done |
| YT-002 | P0 | S | M1 | Cargo workspace + native/WASM targets | YT-001 | Native workspace and wasm32 checks PASS (2026-10-08) | Done |
| YT-003 | P0 | M | M1 | Minimal shared `Component/View` model | YT-002 | `yetlet-core` and `yetlet-todo-ui` native/wasm32 compile PASS | Done |
| YT-004 | P0 | M | M1 | HTML renderer with safe escaping | YT-003 | Four `yetlet-html` escaping/security tests PASS | Done |
| YT-005 | P0 | M | M1 | Axum/Tower server adapter | YT-004 | Native `/` and JSON `/api/ping` routing tests PASS | Done |
| YT-006 | P0 | S | M1 | Static SSR demo at `GET /` | YT-005 | Bounded localhost HTTP smoke: 200, strict CSP, no scripts/WASM | Done |
| YT-007 | P0 | S | M1 | Native checks and CI baseline | YT-002 | Local fmt/clippy/tests/WASM PASS; CI YAML created but not executed remotely | Done |
| YT-008 | P0 | M | M2 | Browser WASM build pipeline | YT-003 | `build_browser.py`, hashed manifest, Chrome asset load PASS | Done |
| YT-009 | P0 | L | M2 | DOM mount and local state updates | YT-008 | Chrome two-click state/class/text update, no HTTP/WS PASS | Done |
| YT-010 | P0 | M | M2 | Shared TodoList across targets | YT-006, YT-009 | Same Rust `todo_list` source, SSR/browser outerHTML byte-identical PASS | Done |
| YT-011 | P1 | S | M3 | Tiny external JS loader and hashed assets | YT-008 | Loader and hashed assets; Chrome confirms no Search WASM before interaction | Done |
| YT-012 | P1 | M | M3 | Island mount boundary | YT-010, YT-011 | Chrome confirms initial SSR results preserved; input/heading nodes remain stable | Done |
| YT-013 | P1 | M | M3 | Search island at `GET /search` | YT-012 | SSR 200; browser case-insensitive search, empty/reset, no input-triggered HTTP/WS | Done |
| YT-014 | P1 | M | M4 | Typed actions and LocalExecutor | YT-003 | `yetlet-actions` unit tests + Browser local toggle, zero network requests PASS | Done |
| YT-015 | P1 | M | M4 | HttpExecutor / explicit remote boundary | YT-014, YT-005 | Chrome one typed Create POST; separate successful/error executor paths; no remote DOM event PASS | Done |
| YT-016 | P1 | M | M4 | `/api/todos` JSON handlers | YT-005, YT-015 | GET/POST tests: state/revisions, 400/403/404/413/415/422 typed errors PASS | Done |
| YT-017 | P1 | M | M3 | Island state schema/version/security | YT-012 | Bounded v1 JSON, HTML escaping, invalid state tests and Chrome fail-closed SSR fallback PASS | Done |
| YT-018 | P1 | M | M4 | Strict CSP and browser security verification | YT-013, YT-016, YT-017 | Headless Chrome/CSP, JSON errors, origin/media/body limits PASS at anonymous prototype scope | Done |
| YT-019 | P1 | L | M5 | Cross-target/E2E tests and performance baseline | YT-006, YT-010, YT-013, YT-016, YT-018 | [M5 verification](M5-VERIFICATION.md): A01–A10 local proof, release metrics, Chrome 155/Firefox 157 smoke | Done |
| YT-020 | P1 | S | M5 | Build/run docs and release readiness | YT-019 | [Release readiness](RELEASE-READINESS.md) prepared; public release blocked by license, MSRV/support/security decisions | Blocked |
| YT-021 | P2 | M | Later | Streaming SSR benchmark | YT-006 | Measured TTFB/throughput; ADR if selected | Deferred |
| YT-022 | P2 | L | Later | Serializable handler/resumability spike | YT-017 | Demonstrate stable handler identity and safe capture serialization | Deferred |
| YT-023 | P2 | L | Later | Independent WebGPU scene renderer spike | YT-009 | Reproducible frame/memory results, not a DOM/CSS rewrite | Deferred |
| YT-024 | P2 | M | Later | Custom action executor transport | YT-015 | Opt-in interface without UI/runtime/server coupling | Deferred |
| YT-025 | P2 | M | M7a | Static-template IR versus VDOM benchmark | YT-009 | [M7a evidence](M7A-VERIFICATION.md): 4 isolated strategies, equivalent output, identity/focus and release asset isolation measured | Done |

## User-approved HTTP core extension

The middleware/RequestContext handoff adds these scoped tasks after M4 without renumbering existing tasks or replacing M5. Its acceptance checks are explicit user requirements, not an unrelated framework expansion.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-026 | P0 | M | M4.1 | RequestContext ownership and typed request/app state | YT-005 | Context/state/metadata/query/path/streaming tests PASS; browser graphs unchanged | Done |
| YT-027 | P0 | M | M4.1 | Native Tower global/group/route composition | YT-026 | Nested order, short-circuit, 404/405/Allow, error and third-party layer tests PASS | Done |
| YT-028 | P0 | M | M4.1 | API/SSR integration and request isolation/streaming proof | YT-027, YT-018 | 46/46 workspace tests, Chrome regression, 12 concurrent requests and byte-identical WASM PASS | Done |
| YT-029 | P1 | M | Later | Optional HTTP providers, deadlines and overhead measurements | YT-028 | Opt-in middleware providers and measured native costs; not P0 scope | Deferred |

## M6 — Handler-first framework APIs and opt-in extensions (locally complete)

**Approved architecture:** Handler-first, Component-first, MVC optional, minimal boilerplate. **M6a–M6e are implemented and locally verified.** MVC/Controller modules, large DI/filters/model-binding systems and macro-based discovery are not prerequisites. [ADR 0006](adr/0006-handler-first-mvc-optional.md) defines handler/results DX, and [ADR 0005](adr/0005-opt-in-capability-modules.md) defines opt-in modules. M6 does not depend on the separately blocked publication item `YT-020`.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-030 | P0 | M | M6a | Minimal App/group/route facade over Axum/Tower | YT-027, YT-016 | [M6a verification](M6A-VERIFICATION.md): 5 real-Router integration tests + rustdoc example; SSR/JSON, nested groups, state and scoped middleware PASS | Done |
| YT-031 | P0 | M | M6b | Handler/typed result ergonomics with minimal boilerplate | YT-030, YT-004 | [M6 evidence](M6-VERIFICATION.md): Page, async View route and native typed extraction/results; 5 targeted tests and compiled docs PASS | Done |
| YT-032 | P0 | M | M6c | Explicit opt-in capability registration and typed access contract | YT-030, YT-026 | configure/try_configure setup functions, 3 scoped/absence/error tests PASS; no registry or DI | Done |
| YT-033 | P1 | M | M6d | Verify one independent optional capability in isolation | YT-032 | Separate yetlet-request-id: 7 opt-in/global/scoped/error/concurrency tests PASS; normal dependency graphs prove isolation | Done |
| YT-034 | P1 | M | M6e | Todo demo migration and baseline regression | YT-030, YT-031, YT-033 | 66 native tests + 4 doctests, Chrome/Firefox PASS; release assets byte-identical; [measured M5 and before/after comparison](M6-VERIFICATION.md) retained | Done |

**M6 is locally complete at prototype scope**, not production or independent security certification. Session/Cookie Auth/Redis/Cache/Output Cache are candidate providers, not M6 deliverables. The request-correlation proof does not implement all deferred `YT-029` providers or operational controls.

## M7 — Component authoring, keyed DOM and local reactivity

M7a–M7e completed the research evidence freeze. The maintainer then authorized a bounded continuation: promote only independently proven low-level contracts, adopt them in the production Todo Browser, and close M7 with a fresh decision/evidence freeze. Fixed-shape authoring, generic forms, a general VDOM/signals framework and macro DSL remain out of scope.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-035 | P1 | M | M7b | Keyed binding/static-slot component authoring prototype | YT-025 | [M7b evidence](M7B-VERIFICATION.md): compiled API, SSR identity preflight, typed slots/local events, keyed structure and disposal PASS in isolated Chrome fixture | Done |
| YT-036 | P1 | L | M7c | Browser-local reactive update graph | YT-035 | [M7c evidence](M7C-VERIFICATION.md): weak handles/subscriptions, one-commit batches, bounded re-entrancy, fail-closed errors, capture/listener disposal and isolated Chrome behavior PASS | Done |
| YT-037 | P1 | M | M7d | Keyed lifecycle, focus and form-state preservation | YT-036 | [M7d evidence](M7D-VERIFICATION.md): controlled input, keyed reorder/removal, focus/selection and input/click listener cleanup PASS in Chrome 155 and Firefox 157 | Done |
| YT-038 | P1 | M | M7e | SSR/WASM renderer regression and production bundle review | YT-036, YT-037 | [M7e evidence](M7E-VERIFICATION.md): 86 tests + 5 doctests, wasm32/Chrome/Firefox regressions, retained M5/M7 measurements, byte-identical production assets and no-decision API classification | Done |
| YT-039 | P1 | M | M7f | Renderer-neutral identity, key and safe slot contracts | YT-038 | [M7f evidence](M7F-VERIFICATION.md): core contract extraction, prototype reuse, 22 targeted tests + 1 doctest, strict Clippy and wasm32 PASS | Done |
| YT-040 | P1 | L | M7g | Extract synchronous local graph into an optional production crate | YT-039 | [M7g evidence](M7G-VERIFICATION.md): 13 graph tests, strong owner/weak handles, batching, bounded re-entrancy, failure/disposal semantics and native/wasm32 isolation PASS | Done |
| YT-041 | P1 | L | M7h | General keyed DOM reconciliation and binding ownership primitive | YT-039 | [M7h evidence](M7H-VERIFICATION.md): optional isolated crate, prevalidated unique keys, stable identity, listener disposal, Chrome/Firefox lifecycle and production asset isolation PASS | Done |
| YT-042 | P1 | L | M7i | Migrate production Todo Browser local list to promoted M7 primitives | YT-040, YT-041 | [M7i evidence](M7I-VERIFICATION.md): SSR parity, Chrome/Firefox keyed identity, queued/in-flight task retirement, disposal/remount, local graph dispatch, explicit HTTP separation, bundle delta and Search isolation PASS | Done |
| YT-043 | P1 | M | M7j | Final M7 regression, architecture decision and evidence freeze | YT-042 | [M7j evidence](M7J-VERIFICATION.md): 91 tests + 5 doctests, six-package wasm32, debug/release Chrome/Firefox including lifetime races, benchmark, bundle/performance and dependency freeze PASS; repeat reviewer `wc_job_XZzdXEOIx6owj1o5` PASS | Done |

## M8 — Sample-driven application experience

M8 validates ordinary application development before considering broader authoring abstractions. Each item must complete task-fit validation and an independent read-only reviewer before the next item begins.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-044 | P1 | S | M8a | Standardize runnable sample structure, catalog and documentation | YT-043 | [M8a evidence](M8A-VERIFICATION.md): repository-backed catalog, focused malformed/false-claim suite, 10 Todo tests and final reviewer `wc_job_vQ6pGDE0slh9gV8c` PASS | Done |
| YT-045 | P1 | M | M8b | API-only REST sample with typed CRUD and HTTP errors | YT-044 | [M8b evidence](M8B-VERIFICATION.md): 7 tests, localhost CRUD/policy smoke, native dependency isolation, material findings repaired and final reviewer `wc_job_d911MsTpbwT6fZY3` PASS | Done |
| YT-046 | P1 | L | M8c | SSR shopping cart with explicit opt-in Session provider | YT-045 | [M8c evidence](M8C-VERIFICATION.md): 114 tests + 5 doctests, lifecycle/CSRF/isolation, stage reviewer PASS and M8e stale-destroy follow-up repaired | Done |
| YT-047 | P1 | M | M8d | Developer-experience and boilerplate audit across Todo, REST and Cart | YT-045, YT-046 | [M8d evidence](M8D-VERIFICATION.md): reproducible source/dependency audit, 32 tests, no-runtime-API-change decision and reviewer `wc_job_lZSDSBgMKWu1X5X7` PASS | Done |
| YT-048 | P1 | M | M8e | Full M8 regression, dependency/bundle review and evidence freeze | YT-047 | [M8e evidence](M8E-VERIFICATION.md): 114 tests + 5 doctests, Chrome/Firefox/native/sample/artifact isolation PASS and reviewer `wc_job_J_MVNzFaVgTH1Gr1` PASS | Done |

## Ready queue

**First:** `YT-019` has reproducible local acceptance/measurement evidence. `YT-020` cannot close the public release gate without maintainer licensing and declared support/security decisions; see [release readiness](RELEASE-READINESS.md).

**Current implementation milestone:** YT-030–YT-048 are locally Done. M8 sample contract, REST, Session Cart, DX audit, complete regression and final reviewer PASS are frozen. Public publication (`YT-020`) remains separately blocked.

**Do not start before prerequisites:** Generalized hydration, arbitrary event forwarding, WebGPU, and resumability remain out of scope.

## Work-item template

```markdown
### YT-xxx — title
- Outcome:
- In scope:
- Out of scope:
- Dependencies:
- Acceptance commands / assertions:
- Risks / affected ADRs:
- Evidence (filled only after verification):
- State: Todo | In progress | Blocked | Done
```

## Update rules

1. Change status only after checking source changes and task-specific evidence.
2. Mark an item `Blocked` if a required build/toolchain/browser gate cannot run, and name the exact limitation; do not infer a passing result.
3. Record discovered must-have work under a new unique ID rather than silently expanding an existing task.
4. If MVP scope or the dependency graph changes, update this document and the Roadmap together.
