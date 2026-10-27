# HTTP Middleware Pipeline and RequestContext

Compose native global, group and route middleware while keeping request-owned
parts/body separate from typed application state. Axum/Tower owns HTTP transport.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use the pinned Rust/Cargo toolchain and the existing server adapter in a native
Axum/Tower application. The example is Router wiring, not a standalone server:
serve the returned Router through native Axum as shown in the [server guide](../crates/yetlet-server-axum/README.md).
A body-owning RequestContext must be the last body-consuming extractor; use
`from_fn_with_state` when middleware needs application state.

## Minimal integration

### Minimal composition example

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

This is the **original native** Axum Router integration; M6a also implements a thin handler-first `App<S>`/group facade **over the same Router**, not a second router, MVC Controller runtime or middleware engine. Native `State`/`Request` handlers, `MethodRouter`, group-scoped `route_layer` and pre-routing `with_global` remain compatible. Use `App::into_router()` before adding the outer global layer. See [M6a verification](M6A-VERIFICATION.md) and [ADR 0006](adr/0006-handler-first-mvc-optional.md).

## Expected result

For the assembled Router, `GET /api/example` returns `explicit-request-value`
with `x-global: visited` and `x-group: api`. Global middleware runs before
application matching; response middleware sees the returned Response before a
streamed body finishes. The example does not open a listener by itself.

## Configuration and errors

### Public API and ownership

`RequestContext<S = ()>` implements Axum `FromRequest<S>` when S is Clone + Send + Sync. It is a body-owning extractor: place it last in handlers, or immediately before native `Next` in middleware. When middleware uses application state, construct it with `from_fn_with_state`.

Request-side operations:

- `method`, `uri`, `uri_mut`, `path`, `original_uri`, `headers`, `headers_mut`.
- `query::<T>()` and `path_params::<T>().await` decode on demand through Axum and preserve its rejection types. No duplicated parameter cache.
- `matched_path() -> Option<&str>`: None before application matching or on an unmatched fallback; Some in matched group/route/handler scope. `original_uri` retains the external URI through nested-prefix stripping.
- `extensions`/`extensions_mut` use the original request-local http::Extensions. Inserted types follow its Clone + Send + Sync + 'static rules. Mutable sharing across layers is an explicit application choice, never a global request dictionary.
- `state() -> &S` exposes application-level shared state, separately from Extensions. Use Arc-backed handles rather than cloning a large application object per extraction.
- `body`, `body_mut`, `into_body`, `into_request` preserve one owned streaming body. The context is not Clone or Serialize and performs no read/collect on construction.

Responses remain ordinary Axum responses; RequestContext contains no response builder, DI container, identity/session engine or renderer.

### Scope and execution order

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

### Built-in context versus optional RequestId

`RequestContext<S>` has three fields: original HTTP `parts`, the streaming `body`, and typed application `state`. The core does **not** automatically create `RequestId`, trace/span metadata, sessions, cookie jars, principals or cache handles. Existing request fields come from the incoming HTTP request; matched-route metadata becomes available only after routing; query/path parameters are decoded on demand.

The Todo example **explicitly installs** `RequestIds::new().global(app)` from the optional `yetlet-request-id` crate. A per-provider Arc<AtomicU64> starting at 1 generates a numeric RequestId extension and x-yetlet-request-id response header. M6 also supports group-scoped installation; the core has no dependency on this provider. The native home route intentionally selects only the number for an inert data-request-id attribute. No request extensions are automatically serialized.

The example counter resets for a new application instance and is not globally unique across server nodes. Applications needing trace correlation should opt into a provider with an intentional ID and trusted propagation policy. See [ADR 0005](adr/0005-opt-in-capability-modules.md).

### Existing demo integration

M6 moved numeric correlation into `crates/yetlet-request-id`; the Todo app installs it explicitly at finalization. The example's `pipeline::with_api_scope` remains an ordinary `App::configure` setup function applying the API marker/header only to matching API paths.

The header-only `action_headers` middleware remains before handler body extraction. M6 uses native State/Json handler parameters and an explicit route-level DefaultBodyLimit::max(4096). Json consumes the body once; rejection mapping preserves the original error envelope/status. This is route-selected JSON buffering, not unconditional framework buffering.

The SSR root now uses Option<Extension<RequestId>> and returns Page, selecting only the numeric ID for its data attribute rather than receiving or serializing the whole RequestContext. Existing tests still prove raw cookies/private headers are not copied. SearchStateV1 and browser source remain untouched. M4 authentication/origin limitations still apply; correlation is not a production identity system.

### Why this remains a small integration

Two production modules are added to the existing server adapter. They reuse http request parts, Body/Extensions, Axum extractors/Next, and Tower Layer/Service. There is no custom dynamic middleware registry, new crate family, duplicate response model, general dependency injection, persistent session, or browser event channel.

This is **not zero overhead**: S is cloned during extraction, native middleware has its own future/service costs, and with_global adds a fallback-only router envelope. RequestId/scope middleware are demo behavior with their own small per-request work. Native latency, allocation counts and throughput were not benchmarked at this P0 checkpoint. The browser build comparison below is separate from those unmeasured native costs.

## API reference

Use [server source/rustdoc](../crates/yetlet-server-axum/src/lib.rs),
[composition](EXTENSIONS.md) and the [current API reference](API-REFERENCE.md).
Current middleware contracts above preserve native routing, errors and streaming.
The old P0/P1 delivery assessment below is a dated checkpoint, not a current Todo list.

<details>
<summary>Historical delivery context and evidence</summary>

**Scope:** User-approved middleware handoff, focused P0 extension to the existing M0–M4 implementation.
**Decision authority:** [ADR 0004](adr/0004-request-context-and-middleware.md).
**Status:** P0 integrated and locally verified on 2026-10-08. P1 providers/benchmarks and independent review were not claimed at that checkpoint. Subsequent bounded YT-029 deadline/overhead evidence is linked below.

### Gap assessment and integration

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

### Validation and evidence

The counts and byte comparisons below record the original P0 checkpoint. Complete M6 integration and its migrated Todo results are recorded separately in [M6 verification](M6-VERIFICATION.md); the original evidence is retained, not relabeled as a new run.

Focused commands observed passing during implementation:

```bash
cargo test -p yetlet-server-axum --all-targets --offline
cargo test -p yetlet-server-axum -p yetlet-todo-server --all-targets --offline
```

The new 14 core integration tests cover scoped ordering, reverse unwind, early response, metadata, query/path failures, pre-routing rewriting, 12 simultaneous request contexts, error responses, native tower-http layer composition, deferred body polling, preserved trailers, stream errors and unread short-circuited bodies. Two demo integration tests cover actual API/SSR paths, safe render selection and outer response headers on 404/405/415.

The following commands are the original historical closeout. Current Rust
equivalents are `cargo xtask check docs`, `cargo xtask build browser` and
`cargo xtask smoke browser-chrome`; fresh smoke evidence uses
`target/m12/yt085/browser-chrome-*/`. Their migration acceptance is recorded
separately in [YT-085 verification](YT-085-VERIFICATION.md).

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

### Deferred P1 and remaining compatibility limits

- Reusable tracing/authentication/CORS/session/rate-limit providers remain separate opt-in components, not P0 defaults.
- Native infallible Tower layers are supported and an existing tower-http layer is tested. Fallible services must use an explicit error-to-response adapter. The thin App facade is implemented in M6; arbitrary fallible-service convenience APIs remain deferred.
- [YT-029 deadline/overhead research](YT-029-HTTP-DEADLINE-RESEARCH.md) now records an opt-in application-owned native Tower timeout, cancellation/body limits and matched in-process release measurements. This completes the bounded research item, not a supported core API or production benchmark. General provider policy and additional streaming/backpressure qualification remain deferred.
- Request-local extensions do not prevent a developer from deliberately inserting the same Arc into every request; the framework only avoids doing so implicitly.
- M5 and M6 record local measurements, not a production performance/security certification. Public release and remaining operational-provider gates remain separate.

</details>
