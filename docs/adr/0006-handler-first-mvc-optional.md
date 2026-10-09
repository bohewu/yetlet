# ADR 0006 — Handler-first application APIs; MVC is optional

- **Status:** M6a–M6e implemented and locally verified. APIs remain experimental; no mandatory MVC, DI or Controller layer.
- **Date:** 2026-10-08
- **Authority:** Maintainer decision to avoid ASP.NET Core MVC-style heavyweight scaffolding and unnecessary boilerplate.
- **Related:** [Architecture](../ARCHITECTURE.md), [Roadmap](../ROADMAP.md), [Backlog](../BACKLOG.md), [ADR 0004](0004-request-context-and-middleware.md), [ADR 0005](0005-opt-in-capability-modules.md).

## Context

Yetlet already uses Axum/Tower routing, request extraction, typed JSON responses, native SSR rendering, shared UI components, browser WASM and explicit middleware. Requiring an additional Controller hierarchy or MVC application convention to consume these capabilities would duplicate mature Rust/HTTP abstractions and burden small applications.

**MVC is an application organization pattern, not a prerequisite for routing, rendering or executing handlers.** Developers may organize modules into models/views/controllers or by feature; Yetlet must not require either arrangement.

## Decision

1. **Handler-first, component-first.** Register ordinary Rust functions as HTTP handlers. Use shared `View`/components for SSR and browser UI. A Controller object, `BaseController` inheritance, action class, controller-specific lifecycle, attribute routing, or mandatory "model-view-controller" directories are not required.
2. **Minimal common path.** A static SSR page or small JSON API should require only route registration, a handler and its response. Avoid scaffolding files, a global DI container, per-action service registration or a custom adapter class for each route. Do not make developers explicitly construct `RequestContext` when an ordinary handler needs no request state.
3. **Use existing typed bindings.** Prefer the underlying Axum extractors (`Path<T>`, `Query<T>`, `Json<T>`, `State<S>`) and the existing optional `RequestContext<S>` rather than implementing a competing MVC model binder. Validate inputs at the relevant trust boundary.
4. **Use lightweight results.** Preserve native `IntoResponse`/`Json<T>`, status and redirect semantics. Provide only small SSR/`View` and typed-result conveniences that eliminate repeated code without hiding response headers, CSP, escaping or error/status behavior. Avoid a universal `ActionResult` class hierarchy.
5. **App builder is a facade, not a second router.** Proposed `App`/route-group methods should compose with Axum/Tower without changing `Router`, `Next`, middleware ordering, 404/405 responses, streaming, or compatibility with existing native handlers. Keep direct Axum usage available.
6. **MVC-style organization is opt-in.** Applications may place handlers in `controllers/` and use `models/`/`views/` modules; the framework does not interpret those directory names or require inheritance. A future `yetlet-mvc` helper is neither a dependency nor a committed deliverable unless actual consumers justify it.
7. **Pay for what you use.** Session, Cookie Authentication, Caching, Tracing and other provider modules remain explicitly installed and independent as approved by [ADR 0005](0005-opt-in-capability-modules.md). A route handler does not implicitly enable them.
8. **Do not chase "zero boilerplate" via hidden magic.** Simple behavior should be concise, while authorization, CSRF, response caching, remote boundaries and failures must remain explicit. Macros such as `#[controller]`, `#[component]` or a `ui!` compiler are optional future research, not requirements for M6.

## Implemented M6 API

```rust
use yetlet_core::View;
use yetlet_server_axum::{App, Page, extract::Path, response::Json};

async fn home() -> View { View::text("Hello") }
async fn item(Path(id): Path<u64>) -> Page {
    Page::new("Item", View::text(format!("Item {id}")))
}
async fn status() -> Json<&'static str> { Json("ready") }

let app: axum::Router = App::new()
    .page("/", "Home", home)
    .get("/items/{id}", item)
    .group("/api", |api| api.get("/status", status))
    .into_router();
```

`page(path, title, handler)` supports no-argument async View functions. Parameterized or fallible handlers use ordinary `.get` and return Page or Result<Page, E>. Page delegates to escaped SSR, preserves strict CSP and returns a generic 500 for rendering failure. Json, Redirect, error/status/header tuples and typed bindings are native Axum, not Yetlet-specific class hierarchies. The working API patterns are covered by rustdoc and integration tests in [M6 evidence](../M6-VERIFICATION.md).

`configure`/`try_configure` accept typed setup functions, and an independent optional request-ID module demonstrates scoped/global registration. The Todo routes now use the facade and typed results/bindings; shared browser UI source and release assets remain unchanged. See [Extensions](../EXTENSIONS.md).

## M6 acceptance and anti-boilerplate checks

- **YT-030 / App builder:** One simple route and a grouped route work without a Controller class or a mandatory source tree layout. Existing Axum handlers, Tower layers and response errors remain compatible.
- **YT-031 / Handler/result ergonomics:** A `View`-returning SSR page, typed `Json<T>` API, typed path/query/body handler, redirect and explicit error/status response are demonstrated using small, direct functions. No duplicate model binder, response hierarchy or mandatory `RequestContext`.
- **YT-032 / Opt-in integration:** No Session/Cache/Auth dependency is required for a basic SSR app. Module integration remains typed/scoped and independent of handler signatures.
- **YT-033 / Extension proof:** One separately installed capability demonstrates zero registration work for an app that does not use it; test scoped middleware and no browser runtime dependency.
- **YT-034 / Demo regression:** Migrate the Todo example only after the public-facing facade is proven; preserve native/browser SSR/DOM behavior, local-vs-HTTP actions and security boundaries. Record measurements rather than claim speed improvements.
- **Maintainability:** Prefer existing Rust functions and feature modules. Any new abstraction must remove repeated application code or express a real safety boundary; complexity without evidence is not an accepted feature.

## Alternatives rejected as the M6 default

- Required ASP.NET Core MVC-like Controller/BaseController and action/result hierarchies.
- Automatic convention-based controller discovery and mandatory folder layout.
- A second HTTP router, filters pipeline or general-purpose model-binding system.
- Always-on DI container, server session, auth providers or runtime plugin registry.

**Scope:** M6 is complete at local prototype scope: thin App, small Page result, ordinary native handlers, typed setup functions, one optional provider and the migrated Todo. A general direct-View IntoResponse implementation, generic component macros, MVC helpers, production Session/Auth/Cache providers and public-release guarantees are not included.
