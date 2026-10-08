# Yetlet MVP — Vertical-Slice Specification

**Status:** Draft; M0–M5 A01–A10 locally verified at prototype scope; public release remains blocked by licensing and production-support decisions.
**Product goal:** Demonstrate that the same Rust component source can serve native SSR and browser WASM without mandatory client JavaScript on SSR-only routes or a persistent server-side event circuit.

This remains the acceptance contract for the **complete MVP**. M1 native SSR, M2 browser/WASM DOM, M3 lazy Search Island/state v1, M4 typed actions/API and M4.1 RequestContext/middleware passed local prototype tests. M5 provides a bounded [A01–A10 acceptance and performance baseline](M5-VERIFICATION.md) using Chrome 155 and Firefox 157. This does not establish generic resumability, production authentication/CSRF, complete IME/accessibility compatibility, or release approval. Consult [Release readiness](RELEASE-READINESS.md) before publishing.

## 1. Scenarios and scope

The demo is a minimal Todo/Search application with four application routes:

| Route | Initial output | Browser runtime | Required demonstration |
|---|---|---|---|
| `GET /` | SSR HTML | No JS or WASM | Readable shared-component HTML and working navigation |
| `GET /search` | SSR HTML | Tiny JS loader plus interaction-activated Rust/WASM island | Local search/filter; remote data API is a separate M4 feature |
| `GET /app` | Browser shell HTML | Full Rust/WASM DOM runtime | Client mount, local state updates, shared TodoList |
| `GET/POST /api/todos` | JSON | None required | Typed input/output/error and HTTP status semantics for browser commands |

An in-memory store and deterministic fixtures are sufficient. The MVP does not require a database, external identity provider, general router plugin, or multi-tenancy. Native form fallback may be implemented as progressive enhancement.

## 2. Normative requirements

### Functional (F)

- **F01:** The same Rust `TodoList`/`TodoItem` source must compile and render correctly for both native and browser targets.
- **F02:** `GET /` must not include mandatory JS/WASM references; the page must be complete and readable with JavaScript disabled.
- **F03:** `GET /search` must initially render usable SSR HTML. Only interactive islands load browser WASM, and input events are processed locally.
- **F04:** `GET /app` must mount a browser UI and perform at least one local state change (such as toggling completion).
- **F05:** Remote operations must use explicit `/api/todos` HTTP requests; raw DOM events must not be forwarded and no default persistent SignalR-like circuit may be created.
- **F06:** SSR initial state must have an explicit schema and version; invalid or incompatible state must not silently take over existing DOM.
- **F07:** Invalid API input must yield an appropriate 4xx status and typed error response. Unexpected errors must not leak internal stack traces or configuration.

### Nonfunctional (N)

- **N01:** Use Rust stable with a checked-in Cargo lockfile and reproducible native and `wasm32-unknown-unknown` builds.
- **N02:** Prove context-appropriate escaping for dynamic HTML text/attributes. Hydration state must not carry secrets or executable injection.
- **N03:** Operate under strict CSP without `unsafe-inline`; use external island/bootstrap JS assets.
- **N04:** Preserve keyboard focus, ordinary input behavior, and common IME/browser text interactions. Avoid intercepting native HTML behavior unnecessarily.
- **N05:** Tests must run in CI or through documented reproducible local commands, with required toolchains stated.
- **N06:** Establish a measured baseline for SSR TTFB, HTML bytes, required initial compressed JS/WASM size, and time until an island becomes interactive. **Do not report unmeasured performance claims.**

## 3. Acceptance matrix

| Test ID | Case | Expected |
|---|---|---|
| A01 | Native HTML escaping unit test | Dynamic text/attributes cannot break their intended HTML context |
| A02 | `GET /` with JavaScript disabled | TodoList HTML remains usable; no mandatory script/WASM |
| A03 | `GET /search` integration | Initial HTML, external asset references, well-formed versioned island state |
| A04 | `/search` browser typing E2E | Local input filtering; only explicit HTTP requests when remote data is requested |
| A05 | `/app` browser click E2E | Local state update; no remote-event circuit traffic |
| A06 | Native/browser shared component | Same fixture satisfies the declared SSR/DOM snapshot contract |
| A07 | Invalid API input integration | Typed 4xx error JSON without internal details |
| A08 | Browser CSP E2E | Required assets load with strict policy and no unsafe-inline |
| A09 | Native and browser builds | Both targets compile without server-only capabilities in browser output |
| A10 | Toolchain/performance report | Reproducible commands, environment, measured data, limitations; no invented metrics |

**Full MVP Definition of Done:** Real, inspectable evidence for A01–A09 plus a reproducible A10 baseline. Passing Rust unit tests alone does not prove browser/WASM or CSP behavior.

## 4. Proposed ergonomic API (illustrative, not implemented)

```rust
App::new()
    .page("/", HomePage::ssr())
    .page("/search", SearchPage::island())
    .page("/app", TodoApp::browser())
    .api("/api/todos", todos_api)
    .run()
    .await;
```

```rust
// Draft semantic actions; the exact contract will be validated in M4.
enum TodoAction {
    Toggle { id: u64 },
    Create { title: String },
}

// DOM events remain local; only explicitly typed action input crosses
// a remote boundary when a developer chooses an HTTP executor.
```

Start with explicit Rust builder/render calls and prove the capability before introducing `#[component]` or `ui!` macros. Public API cosmetics must not delay the first functional vertical slice.

## 5. Build and release envelope

- **Native:** Rust stable, Cargo workspace, executable tests, and one server binary using an established HTTP stack.
- **Browser:** `wasm32-unknown-unknown` plus packaging tooling, generating versioned WASM/JS assets.
- **Loader:** Small external JavaScript. The MVP **does not require** the loader itself to be compiled from Rust.
- **Documentation:** Accurate README build/run steps, per-route smoke procedures, and an observed compatibility matrix.
- **Development flow:** Server and browser assets may be built separately; final smoke verification must use a coherent reproducible set of deployed artifacts.

## 6. Explicit exclusions

LiveView/SignalR-like remote event replication, WebGPU acceleration of ordinary DOM, full CSS/layout engines, complete resumability, native desktop/mobile, multiple ORM providers, a custom HTTP parser, arbitrary magical RPC, a complete plugin ecosystem, and request-time WASM compilation on the server.

## 7. Evidence gates

Collect evidence in the order M1 → M2 → M3 → M4 per [Roadmap](ROADMAP.md). If the shared renderer model has a fundamental problem, revisit [ADR 0003](adr/0003-renderers-and-loading.md) and the component IR assumptions before expanding the crate architecture.
