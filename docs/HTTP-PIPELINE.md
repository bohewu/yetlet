# HTTP Middleware Pipeline and RequestContext

**Scope:** User-approved middleware handoff, focused P0 extension to the existing M0–M4 implementation.
**Decision authority:** [ADR 0004](adr/0004-request-context-and-middleware.md).
**Status:** P0 integrated and locally verified on 2026-10-08. P1 providers/benchmarks and independent review are not claimed.

## Gap assessment and integration

| Handoff requirement | Previous implementation | Integration |
|---|---|---|
| HTTP transport/routing | Axum and Tower already present | Reused; no HTTP parser or routing engine replacement |
| Request-side context | Raw Request and State extractors only | Owning `RequestContext<S>` in the existing server adapter |
| Typed request state | Available via Axum/http but no Yetlet contract | Original request Extensions, distinct from typed application state |
| Global pre-routing middleware | No explicit API; Router::layer would run too late | `with_global` wraps the complete application Router |
| Group/route scope | Available in Axum but undocumented/untested here | `nest`, Router::route_layer, MethodRouter::layer, ServiceBuilder |
| API/SSR consistency | Shared server, mostly handler-local guards | One global pipeline; API group scope and header-only route guard |
| Error and stream semantics | IntoResponse, HtmlPageError, bounded API JSON | Preserved and tested across the middleware boundary |
| Browser separation | Browser and server crates already separate | No browser or shared View changes; no context serialization |

## Public API and ownership

`RequestContext<S = ()>` implements Axum `FromRequest<S>` when S is Clone + Send + Sync. It is a body-owning extractor: place it last in handlers, or immediately before native `Next` in middleware. When middleware uses application state, construct it with `from_fn_with_state`.

Request-side operations:

- `method`, `uri`, `uri_mut`, `path`, `original_uri`, `headers`, `headers_mut`.
- `query::<T>()` and `path_params::<T>().await` decode on demand through Axum and preserve its rejection types. No duplicated parameter cache.
- `matched_path() -> Option<&str>`: None before application matching or on an unmatched fallback; Some in matched group/route/handler scope. `original_uri` retains the external URI through nested-prefix stripping.
- `extensions`/`extensions_mut` use the original request-local http::Extensions. Inserted types follow its Clone + Send + Sync + 'static rules. Mutable sharing across layers is an explicit application choice, never a global request dictionary.
- `state() -> &S` exposes application-level shared state, separately from Extensions. Use Arc-backed handles rather than cloning a large application object per extraction.
- `body`, `body_mut`, `into_body`, `into_request` preserve one owned streaming body. The context is not Clone or Serialize and performs no read/collect on construction.

Responses remain ordinary Axum responses; RequestContext contains no response builder, DI container, identity/session engine or renderer.

## Minimal composition example

```rust
use axum::{Router, response::Response, routing::get};
use tower::ServiceBuilder;
use yetlet_server_axum::{RequestContext, middleware::{Next, from_fn, with_global}};

#[derive(Clone)]
struct RequestLabel(&'static str);

async fn global(mut ctx: RequestContext, next: Next) -> Response {
    assert!(ctx.matched_path().is_none());
    ctx.extensions_mut().insert(RequestLabel("explicit-request-value"));
    let mut response = next.run(ctx.into_request()).await;
    response.headers_mut().insert("x-global", "visited".parse().unwrap());
    response
}
async fn group(ctx: RequestContext, next: Next) -> Response {
    let mut response = next.run(ctx.into_request()).await;
    response.headers_mut().insert("x-group", "api".parse().unwrap());
    response
}
async fn guard(ctx: RequestContext, next: Next) -> Response {
    // Return a response here to short-circuit; this example just continues.
    next.run(ctx.into_request()).await
}
async fn handler(ctx: RequestContext) -> String {
    ctx.extensions().get::<RequestLabel>().unwrap().0.to_owned()
}
fn app() -> Router {
    let api = Router::new()
        .route("/example", get(handler).layer(from_fn(guard)))
        .route_layer(from_fn(group));
    with_global(Router::new().nest("/api", api),
        ServiceBuilder::new().layer(from_fn(global)))
}
```

This is the **currently implemented** Axum Router integration; it does not introduce a second router. M6 proposes a thin, handler-first App/group facade over this machinery—not mandatory MVC Controllers or a new middleware engine. Existing State/Request handlers must remain compatible. See [ADR 0006](adr/0006-handler-first-mvc-optional.md).

## Scope and execution order

For the documented APIs, a matched request enters:

```text
Global.before → OuterGroup.before → NestedGroup.before → Route.before → Handler
Global.after  ← OuterGroup.after  ← NestedGroup.after  ← Route.after  ← Response
```

Use ServiceBuilder for multiple middleware within one scope; first registered is outermost. Repeated Router::layer calls instead wrap previous layers, so last registered is outermost. Finish registering a group's routes before applying its route_layer.

| Case | Defined behavior |
|---|---|
| Valid matched request | All applicable scopes enter; response unwinds in reverse |
| Nested group | Parent wraps its child; sibling routes do not inherit the child's middleware |
| Short-circuit | Downstream scopes/handler do not run; already-entered outer scopes still see the response |
| Unmatched path (404) | Global wraps the application's fallback; matched-only group/route layers do not run |
| Known path, unsupported method (405) | Group and MethodRouter::layer run around the 405 response and preserve Allow |
| Handler returns an error response | Existing IntoResponse behavior; outer scopes can inspect/modify the 4xx/5xx |
| Streaming response | After-processing runs when the Response is returned, **not** when the body finishes streaming |
| Body stream failure | Remains a stream error; the adapter does not swallow it or rewrite already-sent headers |

Axum MethodRouter::route_layer (different from Router::route_layer) skips its method fallback; using it intentionally changes 405 behavior. Custom group fallbacks likewise require an explicit policy instead of assuming the default unmatched-path contract.

`with_global` uses an outer fallback-only Router to preserve `Router` and `axum::serve` compatibility. Add all application routes before calling it; adding a route to the returned outer wrapper bypasses its fallback/global service. The global layer sees no *application* match metadata. Capture values needed for after-processing before moving the request to Next; response Extensions do not automatically inherit request Extensions.

## Built-in context versus optional RequestId

`RequestContext<S>` has three fields: original HTTP `parts`, the streaming `body`, and typed application `state`. The core does **not** automatically create `RequestId`, trace/span metadata, sessions, cookie jars, principals or cache handles. Existing request fields come from the incoming HTTP request; matched-route metadata becomes available only after routing; query/path parameters are decoded on demand.

The Todo example **explicitly registers** `assign_request_id` as a global middleware. A per-app `Arc<AtomicU64>` initialized to 1 generates a numeric identifier, inserts `RequestId(u64)` into request-local typed extensions, and adds `x-yetlet-request-id` to the response. The native home SSR route intentionally selects only that numeric value for an inert `data-request-id` attribute. No request extensions are automatically serialized.

The example counter resets for a new application instance and is not globally unique across server nodes. Applications needing trace correlation should opt into a provider with an intentional ID and trusted propagation policy. See [ADR 0005](adr/0005-opt-in-capability-modules.md).

## Existing demo integration

`examples/todo-server/src/pipeline.rs` assigns a server-generated numeric RequestId from a per-app Arc<AtomicU64>, stores it in the request, and adds `x-yetlet-request-id` after processing. The API group's middleware adds its own marker and `x-yetlet-scope: api` only to matching API paths.

The existing POST header checks moved from `mutate_todos` to `action_headers`, a header-only route middleware. Status codes and error envelopes remain unchanged. The mutation handler alone explicitly chooses the existing 4 KiB JSON buffer; it consumes the body once. This is not unconditional framework buffering.

The SSR root constructs a separate `RenderRequest` containing only the numeric RequestId, and emits a corresponding data attribute. Tests prove raw cookies/private headers are not copied. SearchStateV1 and the browser/source dependency graph remain untouched. Existing M4 authentication/origin limitations still apply: this work does not turn the anonymous demo into a production identity system.

## Why this remains a small integration

Two production modules are added to the existing server adapter. They reuse http request parts, Body/Extensions, Axum extractors/Next, and Tower Layer/Service. There is no custom dynamic middleware registry, new crate family, duplicate response model, general dependency injection, persistent session, or browser event channel.

This is **not zero overhead**: S is cloned during extraction, native middleware has its own future/service costs, and with_global adds a fallback-only router envelope. RequestId/scope middleware are demo behavior with their own small per-request work. Native latency, allocation counts and throughput are not benchmarked here. The browser build comparison below is separate from those unmeasured native costs.

## Validation and evidence

Focused commands observed passing during implementation:

```bash
cargo test -p yetlet-server-axum --all-targets --offline
cargo test -p yetlet-server-axum -p yetlet-todo-server --all-targets --offline
```

The new 14 core integration tests cover scoped ordering, reverse unwind, early response, metadata, query/path failures, pre-routing rewriting, 12 simultaneous request contexts, error responses, native tower-http layer composition, deferred body polling, preserved trailers, stream errors and unread short-circuited bodies. Two demo integration tests cover actual API/SSR paths, safe render selection and outer response headers on 404/405/415.

Final closeout commands executed successfully:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
python3 scripts/check_docs.py
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
git diff --check
```

**Observed:** 46/46 Rust tests passed, including the new 14 core and 2 demo integration tests. Formatting, Clippy with warnings denied, browser compilation, existing headless Chrome regression and all 14 English documents / 29 backlog items passed. The previous typed 422 and incompatible island-state negative scenarios retain their expected failure behavior.

`cargo tree --edges normal --target wasm32-unknown-unknown` for both browser packages excludes yetlet-server-axum, Axum, Tower and Hyper. Neither shared UI source nor browser runtime source changed in this integration. Build comparison against the pre-integration artifacts:

| Debug artifact | Before bytes | After bytes | SHA-256 before = after |
|---|---|---|---|
| Full browser WASM | 754,790 | 754,790 | `66328e5de5f50279b17b86aa07e9be605162f9113cb39f86829b6f8ef6813e27` |
| Search Island WASM | 602,056 | 602,056 | `2472829a118a88304d738a42376d86b416f338393a926ca45a9d1f6c8c423172` |

The two new server adapter modules contain **142 lines including comments/blank lines** (context.rs: 114; middleware.rs: 28), excluding demo/test/documentation changes. This is a source-size observation, not a latency/throughput benchmark. Existing debug WASM sizes are not production compressed transfer measurements. No independent reviewer or formal assurance claim is implied.

## Deferred P1 and remaining compatibility limits

- Reusable tracing/authentication/CORS/session/rate-limit providers remain separate opt-in components, not P0 defaults.
- Native infallible Tower layers are supported and an existing tower-http layer is tested. Fallible services must use an explicit error-to-response adapter; arbitrary fallible-service convenience APIs and a general App facade are deferred.
- Deadline/cancellation policy, timeout layers, more streaming/backpressure edge cases and native overhead benchmarks remain follow-up work.
- Request-local extensions do not prevent a developer from deliberately inserting the same Arc into every request; the framework only avoids doing so implicitly.
- M5 performance/release evidence remains open; no rollout, commit, push or production security certification occurs in this integration.
