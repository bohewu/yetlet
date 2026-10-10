# Yetlet Architecture — v0 Design Baseline

**Status:** Evolving architecture with locally validated M0–M7 functionality and a frozen bounded M7 architecture (2026-10-09); the reviewer-found runtime-lifetime repair, replacement validation and repeat final review pass, while production release assurance remains open.
**Scope:** Native Rust server, HTML SSR, browser DOM/WASM, HTTP data APIs, optional islands.
**Related:** [MVP](MVP.md), [Roadmap](ROADMAP.md), [Backlog](BACKLOG.md), [ADRs](adr/0001-separation-of-concerns.md).

## 1. Problem statement

Yetlet aims to reduce the conceptual and runtime costs of fat web frameworks. Static SSR pages should not require an entire client framework, and routine browser input/click events should not travel to a backend via a persistent event circuit. Rust component code may be shared between native server and browser targets, while renderers, interactivity policies, and action execution locations remain independently selectable.

This is **not** a proposal to rebuild a browser, HTTP transport stack, WebSocket protocol, or CSS layout engine.

### Invariants

- Native SSR must produce meaningful initial HTML; non-interactive pages require **zero browser JS/WASM**.
- Interactive behavior must be activated through explicit islands or a full browser application.
- The browser owns DOM events; remote commands are semantic operations, not event replication.
- Server and browser may share Rust types and component source, but compile into distinct build artifacts.
- HTTP request handlers may render HTML, serialize bounded state, and reference build-time assets. They must never compile JS/Rust/WASM per request.
- HTTP routing and API middleware integrate with a mature server ecosystem, initially Axum/Tower.

## 2. Separate four dimensions

| Dimension | Options | Coupled to other dimensions? |
|---|---|---|
| Execution | Native server / browser WASM | No |
| Render backend | HTML stream / browser DOM / future GPU scene | No, subject to target capabilities |
| Interactivity | None / island / full browser / future resume | No |
| Remote action | None / plain form / typed HTTP / future custom executor | No |

**Important:** `render(server)` does **not** imply `events(server)`. An SSR route can use native form POST, and a CSR component can operate entirely offline.

## 3. Logical architecture

```text
                       ┌────────────────────────────┐
                       │ Rust component declarations│
                       │ + local state + typed actions│
                       └──────────────┬─────────────┘
                                      │ compiler/macros (future)
                           Component description / IR
                             (shape under evaluation)
                                      │
                    ┌─────────────────┴──────────────────┐
                    │                                    │
             Native server target                  Browser WASM target
                    │                                    │
              HTML renderer                         DOM renderer
                    │                                    │
            markup + asset refs                 client mount / updates
                    │                                    │
                    └──────────┬─────────────────────────┘
                               │
                      Local semantic action
                               │
                      Action executor port
                        │               │
                   Local/state       HTTP/fetch
                                        │
                             Axum/Tower adapter
                               ├─ UI routes
                               └─ API routes
```

Renderers must never implicitly establish a remote session or circuit. Action transport is an optional adapter. API requests follow ordinary server middleware and authorization boundaries.

## 4. Request paths

### 4.1 API route

```text
HTTP request → router → auth/middleware → typed API handler
                                             → JSON/status/body
```

UI SSR and API handlers do not need to share a URL path. Authentication, authorization, input validation, and rate limits belong at the server trust boundary.

### 4.1.1 RequestContext and middleware (P0 extension)

The server adapter now exposes a request-only `RequestContext<S>` and native Axum/Tower composition. It owns the original request head/body, retains typed request Extensions, and keeps shared application state separate. `with_global` wraps the completed Router before application route matching; groups/routes use native nested routers and layers. API and SSR pass through the same HTTP middleware, without coupling browser actions to server requests.

Scope order, 404/405 semantics, error responses, body ownership and the safe SSR data boundary are specified in [ADR 0004](adr/0004-request-context-and-middleware.md). The [implementation note](HTTP-PIPELINE.md) includes tested integration points, costs and P1 limits. No browser runtime dependency or general DI/session subsystem is introduced.

### 4.2 Static SSR route

```text
GET / → router → loader (native capability) → component
    → HTML renderer (context-aware escaping) → response
```

Without interactive components, HTML and CSS are sufficient. SSR may call application services directly without localhost HTTP loopback.

### 4.3 SSR + island route

```text
GET /search → native SSR → complete HTML + escaped state + hashed assets
                                  │
                          tiny external JS loader
                                  │ on focus/input
                          lazy Search Island WASM
                                  │
                          local browser DOM events
```

The MVP uses progressive island activation/mounting, not full resumability. Only the island-owned subtree participates in client updates. The loader can be plain JavaScript rather than Wasm. Serialized state requires an explicit schema/version/component ID and may not contain credentials or server capabilities.

**M3 prototype evidence (2026-10-08):** `/search` uses a small external loader that imports Search WASM after a local focus/input event. Native SSR emits an explicitly allowlisted versioned JSON state, escaped as an attribute. Browser WASM validates the payload and updates only the results subtree. Invalid schema/version prevents activation while preserving the readable SSR page. This is **not** generalized component hydration or resumability.

### 4.4 Full browser route

```text
GET /app → shell HTML + compiled client artifacts
    → loader → browser WASM → DOM mount → explicit API if needed
```

Shared component source does not imply inclusion of server-side repositories or other privileged dependencies in browser artifacts. Browser data access is through capabilities that bind to explicit HTTP APIs.

**M7i production evidence (2026-10-09):** `/app` now preflights and binds its local Todo list through optional `yetlet-dom-keyed`, then drives typed `TodoCommand` values through one strongly owned `yetlet-reactive::LocalGraph`. Existing keyed rows are updated in place; the separate Create button remains an explicit `HttpExecutor` path. Chrome and Firefox verify identity, disposal, remount and no local request circuit. The Search Island does not acquire either dependency. See [M7i verification](M7I-VERIFICATION.md).

## 5. Proposed module boundaries

| Candidate module | Responsibility | Timing |
|---|---|---|
| `yetlet-core` | Renderer-neutral component/View and state primitives | M1 |
| `yetlet-reactive` | Optional explicitly owned synchronous local update graph | M7g |
| `yetlet-html` | Deterministic HTML serializer and escaping | M1 |
| `yetlet-server-axum` | HTTP/UI/API routing integration, headers | M1 |
| `yetlet-session` | Optional server-side session identity, memory storage, expiry and CSRF token | M8c |
| `yetlet-dom` | Browser DOM mount and updates | M2 |
| `yetlet-dom-keyed` | Optional keyed child reconciliation and listener ownership | M7h |
| `examples/todo-browser` | Browser WASM bootstrap; production keyed local Todo path | M2 / M7i |
| `examples/search-island` | Search-specific lazy island runtime | M3 |
| `yetlet-actions` | Typed commands and pluggable executor boundaries | M4 |
| `yetlet-macros` | Compile-time `ui!` DSL and template extraction | Future spike |
| `yetlet-gpu` | Independent retained scene renderer | Later research |

These are conceptual boundaries, **not** a promise that each one will become a public crate. Use the smallest validated interfaces across the two renderer targets before adding extra abstractions or splitting packages.

### Optional application capabilities (M6 composition; M8c Session proof)

Server features such as sessions, authentication, data caching, output caching, request tracing and rate limiting are **modular, opt-in, and pay-for-what-you-use**. Integrate them through independent Rust crates/providers, native Tower/Axum layers, typed request extensions and explicit application state. Do not add an always-on DI container or plugin manager. Application cache and HTTP output cache have different safety constraints; cookie-based authentication need not require a server session.

`App::configure` and `try_configure` now compose ordinary typed setup functions once, while providers reuse native Tower scopes, explicit state and Extensions. The independent `yetlet-request-id` crate is the M6 opt-in proof: the Todo app installs `RequestIds::new().global(app)`, and a scoped provider can be configured only on selected groups. The core has no dependency on this provider. `RequestContext` itself still generates no ID. No server extension or provider dependency is copied into Browser WASM/hydration. See [Extensions](EXTENSIONS.md), [ADR 0005](adr/0005-opt-in-capability-modules.md) and [M6 evidence](M6-VERIFICATION.md).

M8c adds the independent `yetlet-session` crate as a second, security-bounded proof. The cart explicitly installs `SessionManager<Cart>` only on its `/cart` group. IDs and CSRF tokens are OS-random, state and expiry remain server-side, unknown IDs are not adopted, mutations use a session-bound token, and clear/end demonstrate rotation and revocation. The server adapter, Todo, REST and Browser packages do not depend on Session. This is an anonymous process-local provider, not authentication or production qualification. See [M8c evidence](M8C-VERIFICATION.md).

M8d's measured three-sample audit retains these boundaries rather than introducing another runtime API: Todo pays for Browser/keyed/reactive capabilities, REST remains a native one-package API, and Cart remains a native one-package SSR application selecting Session. Shared candidates such as listener helpers, JSON errors, origin policy and forms remain application-specific until more examples converge. M8e freezes the complete artifact/dependency regression and final reviewer PASS without changing these boundaries. See [M8d evidence](M8D-VERIFICATION.md) and [M8e evidence](M8E-VERIFICATION.md).

### Handler-first application API (M6 implemented)

**Handler-first, Component-first, MVC optional, minimal boilerplate.** MVC is a code organization convention, not an application requirement. A simple SSR page or typed JSON endpoint uses ordinary Rust handler functions and shared `View`/components, with no Controller superclass, mandatory folder structure, custom model binder, broad `ActionResult` hierarchy or service locator.

M6a now implements `yetlet-server-axum::App<S>`, a thin facade over `axum::Router<S>`. `get`, `post`, `route`, `group`, `route_layer`, `with_state`, `map_router` and conversion to/from native Router preserve Axum/Tower handler, extractor, middleware, 404/405 and response semantics. `RequestContext<S>` remains optional. [M6a verification](M6A-VERIFICATION.md) records the native integration tests and reproducible commands.

M6 now adds `App::page(path, title, handler)` for no-argument async View functions and the small `Page` HTTP result for handlers with native typed extractors or `Result<Page, E>` errors. Json, Redirect and status/header results use native Axum semantics. The Todo server now uses App, Page, native State/Json and explicit provider registration. There is still no Controller runtime, discovery macro, custom model binder or full MVC framework. See [ADR 0006](adr/0006-handler-first-mvc-optional.md).

## 6. Components, templates, and state

### Shared component source

```rust
// Proposed DSL only; not a released Yetlet API.
#[component]
fn TodoList(items: &[Todo]) -> View {
    ui! { <ul>{ for item in items { <li>{item.title}</li> } }</ul> }
}
```

The native target outputs HTML; the browser target creates and updates DOM. Component boundaries and stable identifiers must be deterministic to avoid hydration mismatches.

**M7a measured evidence:** An isolated release benchmark compared current subtree replacement, a positional `View` patch, keyed direct DOM and runtime-cloned static slots over 200 rows × 500 updates × 10 rounds. The retained `2026-10-09T02:02:22+00:00` report recorded median round totals of 213.500, 41.700, 2.000 and 1.900 ms respectively on one local Chrome 155 environment. Whole replacement lost owned input/keyed-row identity; the positional patch preserved the input but not keyed identity after reorder; keyed/slot prototypes preserved both. Production Browser/Search assets remained byte-identical. See [M7a verification](M7A-VERIFICATION.md).

**M7b prototype evidence:** The isolated `yetlet-keyed-bindings` crate expresses one fixed keyed-list shape with native SSR, allowlisted typed text/attributes, one conditional slot and one local typed row action. Browser binding preflights component/instance/key/slot/shape identity before attaching listeners, preserves existing SSR nodes, reconciles keyed insert/remove/reorder, and supports explicit disposal. Production Browser/Search assets and dependency graphs remain unchanged. See [M7b verification](M7B-VERIFICATION.md).

**M7c prototype evidence:** The same isolated crate now contains one explicitly owned synchronous local graph. Strong ownership stays in `LocalGraph`; dispatch handles and subscriptions are weak. FIFO batches reduce into one draft and one commit, re-entrant messages run in a later bounded pass, reducer errors preserve the previous commit, renderer errors fault the graph, and disposal releases observers/listeners/captures. Chrome verifies the graph driving the M7b binding without HTTP or WebSocket traffic. See [M7c verification](M7C-VERIFICATION.md).

**M7d prototype evidence:** The fixed row now optionally owns one controlled text input. Model-aware live-value updates avoid resetting caret selection; successful keyed reconciliation captures/restores a connected focused input and selection, while focused row removal deliberately leaves the detached node unfocused. Chrome 155 and Firefox 157 verify input identity/value/focus/selection across reorder plus listener cleanup after row removal and owner disposal. See [M7d verification](M7D-VERIFICATION.md).

**M7e evidence freeze:** The complete native/wasm32/Chrome/Firefox regression, M7a benchmark rerun, production asset comparison and dependency isolation pass without changing production Browser/Search bytes. The evidence identifies small identity, slot, keyed reconciliation, ownership/disposal and weak-handle patterns as candidates for later extraction, but promotes none of the current APIs. See [M7e verification](M7E-VERIFICATION.md).

**M7f promoted contract:** `yetlet-core::binding` now owns renderer-neutral `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor` types. The fixed M7b authoring prototype consumes these contracts, eliminating duplicate validation while remaining experimental. This promotion selects only the shared trust-boundary vocabulary; it does not select a template IR, DOM reconciler, scheduler or hydration protocol. See [M7f verification](M7F-VERIFICATION.md).

**M7g optional graph:** The exact synchronous graph proven in M7c now lives in dependency-free `yetlet-reactive`. It retains one strong owner, weak dispatch/subscription handles, FIFO batching, bounded later-pass re-entrancy, recoverable reducer errors, faulted commit errors and deterministic disposal. It is an explicit local message/commit primitive, not field-level signals, automatic effects or an async scheduler. See [M7g verification](M7G-VERIFICATION.md).

**M7h optional keyed DOM:** `yetlet-dom-keyed` now owns validated keyed order, element-plus-state entries, insert/remove/reorder reconciliation and disposable event-listener ownership. The M7b–M7d fixture consumes the primitive and retains its Chrome/Firefox identity, controlled-input and cleanup evidence. Keeping it independent from `yetlet-dom` is required for pre-adoption production asset isolation; attempts to place or feature-gate it inside the existing renderer changed compiled Browser/Search bytes. The update/DOM mutation failure boundary remains explicit and non-transactional. See [M7h verification](M7H-VERIFICATION.md).

**M7i production adoption:** Todo Browser now binds its local list to the promoted keyed owner and uses a graph-backed adapter at the existing `ActionExecutor<TodoCommand>` boundary. Route runtime ownership is explicit; disposal invalidates the runtime lifetime before dropping local/remote listeners, so queued actions cannot start and in-flight HTTP completions cannot update retired DOM. Readable DOM remains, and remount creates exactly one fresh action path. The remote result list and Search results retain their prior bounded subtree strategy. Separate Cargo builds prevent Todo's expanded browser features from perturbing Search assets. See [M7i verification](M7I-VERIFICATION.md).

**M7j architecture freeze:** [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md) accepts the renderer-neutral binding vocabulary, optional explicitly owned local graph, separate optional keyed DOM owner and one production Todo local-list adoption. It retains direct subtree mounting for simpler/independent surfaces and explicitly rejects inferring a general VDOM, template DSL, signals/forms framework or resumability design from the benchmark. The first final reviewer found one queued/in-flight disposal race; explicit lifetime invalidation plus Chrome/Firefox debug/release regressions repair it. Replacement validation and repeat reviewer Job `wc_job_XZzdXEOIx6owj1o5` PASS are recorded in [M7j verification](M7J-VERIFICATION.md).

M7a–M7e made the keyed/static-slot/local-ownership direction concrete and ended in a no-decision freeze. M7f–M7h promote only independently proven low-level contracts; M7i adopts them for one production Todo local-list path; M7j freezes the bounded decision, repaired runtime lifetime, complete regression evidence and repeat reviewer PASS. The fixed authoring shape, current text-input API, focus policy and conditional-slot shape remain experimental. No public VDOM, dependency-tracked signals/forms runtime, macro DSL or general template compiler is selected.

### M9 compatibility classification

M9's twelve independently reviewed applications provide enough evidence to classify—but not stabilize—the actual public Rust surface. [Public API Contract Classification](API-CONTRACTS.md) records renderer-neutral `View`/binding values, escaped HTML, handler-first `App`/`RequestContext`/Tower composition and bounded Request ID/Session application APIs as **Stable Candidates**. The complete-document `Page`/language/head contract and browser Actions/DOM/keyed/reactive surfaces remain **Experimental**. Adapter constructors, direct provider middleware and generated implementation details remain **Internal/support-only**.

This classification does not change crate versions or establish SemVer compatibility. M9j froze 190 tests, native/browser/provider/dependency and exact-asset evidence; after reviewer-discovered inventory/reexport corrections, repeat reviewer `wc_job_QkQkovlBlvrhPxCE` accepted the classification. Licensing, historical MSRV, browser/support policy, the root document metadata design, external CI and production security expectations remain release blockers.

### Data-access capabilities

```rust
// Conceptual interface; the browser target cannot depend on server repositories.
trait TodoSource {
    async fn list(&self) -> Result<Vec<Todo>, TodoError>;
}
```

A native implementation can call domain/repository services directly. The browser implementation uses explicit HTTP JSON endpoints. Do not hide arbitrary RPC generation inside a compiler macro.

### State transfer

Native HTML may carry serializable initial state, subject to all of the following:

- Explicit component and instance identity, schema version, and asset manifest version.
- Context-sensitive HTML/attribute/JSON escaping; avoid `</script>`, ampersand, or Unicode separator injection.
- Payload size limits, browser-side version validation, and a deterministic safe fallback.
- Never serialize server request context, access tokens, passwords, database connections, or privileged closure captures.

M3 demonstrates these constraints for the Search Island's bounded v1 payload. A generalized state protocol remains outside the current proof.

## 7. Events and commands

```text
pointer/keyboard/input DOM event
          │ local browser listener
          ▼
  component-local state transition
          │ when remote work is needed
          ▼
 typed command / explicit HTTP form
          │
          ├─ LocalExecutor / LocalGraph adapter
          ├─ HttpExecutor
          └─ Future CustomExecutor (explicit opt-in)
```

The MVP does not forward raw DOM events. Remote actions need explicit input/output/error schemas; the server must validate permissions again. HTTP mutations must address CSRF, idempotency, replay, and retries. Keep future custom transports separate from renderer interfaces.

**M4 prototype evidence (2026-10-08):** `yetlet-actions` exposes `Command` and injected `ActionExecutor<C>` implementations: `LocalExecutor` runs a shared domain transition; `HttpExecutor` makes an explicit JSON POST to `/api/todos`. `examples/todo-ui` owns `TodoCommand`/`TodoSnapshot`; the server uses the same transition behind an Axum adapter with a per-router in-memory store. Browser E2E confirmed local toggle with no HTTP and a separate semantic Create POST. Typed errors are surfaced without executing untrusted HTML. See [M4 verification](M4-VERIFICATION.md).

**M7i local adapter evidence (2026-10-09):** Todo Browser's local executor now dispatches the same typed command through a weak graph handle and returns only the last successfully committed snapshot. Graph ownership, rendering and listener lifetime remain local to the route runtime; `HttpExecutor` remains the only remote path. This changes local scheduling/DOM ownership, not the command or transport contract.

**Security boundary:** This prototype is anonymous and listens directly over HTTP. Content-type/Origin restrictions and JSON body limits are demonstrated, but they do not implement user authentication, credentialed CSRF protection, or HTTPS reverse-proxy identity. These remain requirements for any real deployment.

## 8. Security, accessibility, and operations

- Strict CSP as the default: no inline `on*` handlers and no `eval`. Bootstrap scripts are static external assets.
- Context-correct SSR escaping. Treat hydration metadata and client state as hostile.
- Prefer native semantic HTML with labels, keyboard focus, IME, and built-in scrolling.
- Browser builds cannot import server-only secrets/capabilities; invalid target imports should fail at compile time.
- Instrument by mode: SSR render time, HTML bytes, compressed WASM payload, mount time, and request count.
- Distinct error semantics: server renderer errors produce a status code; client actions expose typed errors; failed islands leave usable SSR content.
- Keep CSP, XSS, and dependency checks alongside performance work.

## 9. GPU and parallelism boundary (post-MVP)

WebGPU cannot accelerate ordinary DOM/CSS layout directly. A future WebGPU target must be a separate **Canvas/scene renderer**, which carries its own scene graph, focus, IME, text, and accessibility responsibilities. Workers may move computation away from the main thread, but DOM commits still occur on the main thread. Measure scene-node count, frame time, and memory in a separate spike; do not make GPU development an MVP gate.

## 10. Risks and experiments

| Risk | Initial test / mitigation |
|---|---|
| Rust UI macro limits and compile complexity | Prove shared View plus dual renderer first; decide on IR/`ui!` later |
| SSR/client state mismatch | Deterministic IDs and SSR/hydration snapshot assertions |
| WASM bundle size and startup | Repaired M7i/M7j records a +82,726 raw / +20,042 local-gzip Browser runtime delta; retain the final baseline and split optional islands where useful |
| Over-abstracted DOM updates | M7i adopts low-level keyed ownership for one Todo list only; remote/Search subtree paths and fixed authoring remain independent |
| Server-only dependencies in browser | Target-specific compilation and browser CI |
| Remote commands accidentally becoming a circuit | Semantic actions only, no default WebSocket |
| Exploding combinations of modes | MVP exposes only three tested route presets |
| GPU renderer not matching HTML semantics | Keep GPU outside MVP and provide a separate scene interface |

See [ADR 0001](adr/0001-separation-of-concerns.md), [ADR 0002](adr/0002-events-and-actions.md), [ADR 0003](adr/0003-renderers-and-loading.md), and [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).
