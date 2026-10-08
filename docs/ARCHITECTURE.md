# Yetlet Architecture — v0 Design Baseline

**Status:** Evolving proposal with locally validated M0–M3 prototype slices (2026-10-08).
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
                       │  + signals + typed actions │
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

## 5. Proposed module boundaries

| Candidate module | Responsibility | Timing |
|---|---|---|
| `yetlet-core` | Renderer-neutral component/View and state primitives | M1 |
| `yetlet-html` | Deterministic HTML serializer and escaping | M1 |
| `yetlet-server-axum` | HTTP/UI/API routing integration, headers | M1 |
| `yetlet-dom` | Browser DOM mount and updates | M2 |
| `examples/todo-browser` | Browser WASM bootstrap/example | M2 |
| `examples/search-island` | Search-specific lazy island runtime | M3 |
| `yetlet-actions` | Typed commands and pluggable executor boundaries | M4 |
| `yetlet-macros` | Compile-time `ui!` DSL and template extraction | Future spike |
| `yetlet-gpu` | Independent retained scene renderer | Later research |

These are conceptual boundaries, **not** a promise that each one will become a public crate. Use the smallest validated interfaces across the two renderer targets before adding extra abstractions or splitting packages.

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

**Design hypothesis:** A future macro may extract static templates and dynamic slots, potentially reducing VDOM diff costs. For the MVP, a minimal `View`/node representation is sufficient to establish a measured baseline. Whether a template IR is warranted remains an open spike decision.

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
          ├─ LocalExecutor
          ├─ HttpExecutor
          └─ Future CustomExecutor (explicit opt-in)
```

The MVP does not forward raw DOM events. Remote actions need explicit input/output/error schemas; the server must validate permissions again. HTTP mutations must address CSRF, idempotency, replay, and retries. Keep future custom transports separate from renderer interfaces.

**M4 prototype evidence (2026-10-08):** `yetlet-actions` exposes `Command` and injected `ActionExecutor<C>` implementations: `LocalExecutor` runs a shared domain transition; `HttpExecutor` makes an explicit JSON POST to `/api/todos`. `examples/todo-ui` owns `TodoCommand`/`TodoSnapshot`; the server uses the same transition behind an Axum adapter with a per-router in-memory store. Browser E2E confirmed local toggle with no HTTP and a separate semantic Create POST. Typed errors are surfaced without executing untrusted HTML. See [M4 verification](M4-VERIFICATION.md).

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
| WASM bundle size and startup | Measure compressed sizes/mount time and split islands where useful |
| Over-abstracted DOM updates | Validate direct updates before comparing VDOM/static slots |
| Server-only dependencies in browser | Target-specific compilation and browser CI |
| Remote commands accidentally becoming a circuit | Semantic actions only, no default WebSocket |
| Exploding combinations of modes | MVP exposes only three tested route presets |
| GPU renderer not matching HTML semantics | Keep GPU outside MVP and provide a separate scene interface |

See [ADR 0001](adr/0001-separation-of-concerns.md), [ADR 0002](adr/0002-events-and-actions.md), and [ADR 0003](adr/0003-renderers-and-loading.md).
