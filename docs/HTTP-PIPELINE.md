# HTTP Middleware Pipeline and RequestContext

Compose native global, group and route middleware while keeping request-owned
parts/body separate from typed application state. Axum/Tower owns HTTP transport.

**Experimental SDK integration.** These modules are distributed in SDK
`1.0.0-rc.1` through `yetlet::web` and the `web` feature. Middleware and
`RequestContext` work without View or HTML. Add `html` for SSR Page/document
helpers. Dated measurements and commands retain their original source scope. [Release readiness](RELEASE-READINESS.md) records distribution and
its qualification limits.


## Application-owned deadlines, streaming and admission timing

**Tower deadline bounds only the inner service until it creates a
Response**, not the full response body or transport. In the selected
middleware lab, a 200ms awaited handler with a 100ms TimeoutLayer
and HandleErrorLayer returned 504; without the opt-in layer it
still returned 200. Dropping a pending handler future may stop its
remaining awaits, but does not roll back a preceding write or
cancel a started spawn_blocking worker or detached task. A Response
that already returned 200 can stream or stall beyond the deadline.
Whole-body/idle/connection budgets, safe retries and business
reconciliation remain application/server responsibilities. No
generic Yetlet deadline provider was added.

The 2026-10-12 native no-network study measured four million sequential
synthetic requests and recorded median ns/request **375.590** for a
direct Router, **538.303** with global envelope, **619.120** with
no-op request context, and **622.542** with deadline/error mapping.
Context and deadline compositions were alternatives, not stacked.
These are local aggregate synthetic costs, not production p95/p99,
concurrency, isolated middleware allocation or universal SLOs.
See the [middleware lab](../examples/middleware-lab/README.md).

Optional application-owned row-streamed SSR can emit first body bytes
earlier in some large/synthetic delayed workloads, but increases
total completion cost and cannot change a 200 already sent when a
later row fails. Keep complete escaped SSR as the default; no
browser progressive paint, TLS/HTTP2 or proxy flush promise follows.

**HTTP Session rejection timing is a different measurement layer
from direct provider barrier bursts.** Twelve SSR Busy/503 failures
were observed with four active and four admitted-but-pending Session
owners at each exact server-side rejection interval. Rejected
requests did not dispatch provider reads or enter SQL. Distinct
sixteen-VU JSON/SSR HTTP bursts had zero Busy, with 2.207ms/3.984ms
server-entry spreads rather than synchronized provider arrivals.
Client connect/setup, response completion, Redis work and shared
resource instrumentation clocks cannot be added as identical spans.
Eleven rejected intervals plausibly overlapped an external Valkey
resource sample under an *advisory* clock estimate, not proof of
observer causation; the other failure occurred outside its interval.
Exact permit-release times and global HTTP capacity are unproven.
See [Provider Operations](PROVIDER-OPERATIONS.md#bounded-provider-bursts-and-http-attribution).
## Opt-in application HTTP admission contract

The Auth/PostgreSQL reference's **application-owned**, opt-in
`auth-workload-admission` feature is not a Core/default limiter.
With that feature compiled, its existing startup environment key
`YT176_HTTP_MAX_IN_FLIGHT` is read once **before backend setup**:
absent means no gate; ASCII digits 1–1024 select a cap; zero,
non-Unicode, invalid or out-of-range input fails startup.
Feature-off builds omit this module and env lookup entirely.
The cap counts protected **HTTP request futures per process**,
not RPS, connections, Session operations or distributed quota.

One shared Arc semaphore surrounds the Session/Auth/SQL scoped
API, private SSR, login/logout routes, inside the outer privacy/CSP
layer. Health and optional fixture observations stay outside.
A full gate uses immediate `try_acquire_owned`: **no HTTP
wait queue, timer, automatic retry or downstream call**.
The bounded private generic **503** carries no-store, Vary: Cookie,
nosniff and strict CSP; it carries no Set-Cookie, redirect,
authentication challenge or fabricated Retry-After. Distinguish
overload 503 from keyed rate-limit 429, expired identity and CSRF
or permission denial. Keep successful inner response headers intact.

Admitted permits remain held through downstream response creation
**including Session finalization**, ending at ResponseReady or
future cancellation. They do **not** span response-body/socket
delivery; cancellation releases only the HTTP permit, never
any already dispatched, worker-owned Session/backend permit.
Prior business or Session writes might have committed even
if the HTTP caller times out or sees a finalizer 503. No retry,
rollback, distributed admission or fallback to stale identity
is implied. Provider admission remains **four active/eight total**,
with 80ms active-permit wait and 400ms per-Session-operation
deadline, independent of the four-connection SQL pool.

Original integration tested **six native in-process Axum cases PASS**
and separately compiled feature-off configuration, including
shared rejection, a held finalizer, cancellation and recovery;
those tests did **not** execute a new live Redis/SQL workload.
Three subsequently accepted isolated Auth workloads are described
in [Qualification Matrix](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts),
with application decisions and exact latency populations in the
[workload example](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection).
Complete original commands, source and Reviewer evidence survive
in **local Git** `619b0d9d1e05c86ea0193efb5fc91a198ec930a8` (public SHA reachability unverified).
## HTTP completion, sensitive-write and worker lifetime boundaries

The HTTP `Response` is constructed before its full body is sent;
a disconnected/timed-out caller does not thereby cancel a previously
dispatched SQL closure, revoke its own write permission, prove a
business commit or permit a blind retry. This applies to the native
Starter admission/deadline path and to application-selected Lab
Board opt-ins. Current request admission, CSRF/Session, SQL Busy
and handler cleanup remain consumer-owned.

The optional Board **commit-write-auth** protects only its selected
SQL business-write effect with a private Store-owned permission
ticket checked while holding the real `BEGIN IMMEDIATE` transaction.
A revoke ordered after that transaction may not preempt its commit;
a fresh revoke/re-enable generation invalidates older tickets.
This is **not** a generic framework authorization middleware,
distributed lock or guarantee that Session/IdP revocation and SQL
commit are atomic. Refusing missing/legacy/malformed authority on
reopen is a deliberate fail-closed application policy.

Optional **request-tracing** uses a server-origin process-local
request ID and selected safe HTTP method into a native root span;
the approved worker span/Dispatch survives an HTTP future drop
only for the real owned work. It must not propagate user-supplied
ambient trace context, URI/query, identity, cookie, SQL parameter
or secrets. Response construction, future drop and SQL worker
completion are different events; a worker's `ok` result is not
a universal durable business-commit receipt. See
[Provider Operations](PROVIDER-OPERATIONS.md#optional-sensitive-writes-tracing-and-catalog-consistency)
and [Board source](../examples/lab-operations-board/README.md#opt-in-request-and-sql-worker-tracing--yt-182)
for exact opt-in activation and admitted scope.

## Authentication, Session and quota HTTP failure ordering

A guarded HTTP integration must select middleware and admission
order intentionally: a bounded **application request gate** before
expensive Auth/Session/SQL can shed calls early, but is not installed
merely by enabling `yetlet::auth`, rate limiting or Redis Session.
The Session backend's own **four active / eight total** operations,
immediate total-admission Busy, 80ms active-permit wait and 400ms
**per API operation** timeout are separate from an HTTP request
deadline or route-level quota. A single HTTP request may resolve
Session more than once and finalize/cancel after a handler.
Timeout/drop does **not** reclaim already running backend permits,
guarantee rollback, authorize blind mutation retry or guarantee
that no Set-Cookie raced after a different replica's rotation.

| Source decision | HTTP/application consequence |
|---|---|
| Verified missing/expired/revoked identity | Preserve the selected native unauthenticated/redirect handling and required authentication challenge; do **not** infer a storage outage |
| Verified Auth principal but resource/role denied or CSRF invalid | Preserve current 403/CSRF application refusal |
| Session/Valkey/policy unavailable, Busy, invalid record, over-capacity or unknown mutation result | Fail closed, private/no-store redacted **503**, never anonymous allow, 401/403 or successful Cookie rotation |
| Actual rate quota denied under a bounded key/time policy | Existing no-store **429** with measured quota `Retry-After`; internal limiter/storage failure remains 503 |
| An application-selected separate HTTP response-creation deadline | Keep that application's existing mapping (Starter uses 504), documenting that dispatched SQL work may still commit after HTTP response/cancellation |

Do not derive an HTTP Retry-After from an 80ms Session slot wait
or turn rate-budget `429` into general infrastructure `503`.
The original YT-175 research recommends retaining the provider
guard and selecting a bounded *outer* gate only for an application
that needs overload protection; it **did not implement** new
middleware, platform settings, distributed quotas or hidden retries.
A queue consumes waiting memory/deadline and cannot promise burst
acceptance; distinct manager processes/replicas multiply admission
rather than sharing a global limit. See
[Provider Operations](PROVIDER-OPERATIONS.md#provider-overload-and-quota-distinctions)
and [Threat Model](THREAT-MODEL.md) for trust/cookie/forwarded-header,
user and credential privacy boundaries.

## Prerequisites

Use the pinned Rust/Cargo toolchain and the existing server adapter in a native
Axum/Tower application. The example is Router wiring, not a standalone server:
serve the returned Router through native Axum as shown in the [server guide](../crates/yetlet/guides/server-axum.md).
A body-owning RequestContext must be the last body-consuming extractor; use
`from_fn_with_state` when middleware needs application state.

## Minimal integration

### Minimal composition example

```rust
use axum::{Router, response::Response, routing::get};
use tower::ServiceBuilder;
use yetlet::web::{RequestContext, middleware::{Next, from_fn, with_global}};

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

This is the **original native** Axum Router integration; M6a also implements a thin handler-first `App<S>`/group facade **over the same Router**, not a second router, MVC Controller runtime or middleware engine. Native `State`/`Request` handlers, `MethodRouter`, group-scoped `route_layer` and pre-routing `with_global` remain compatible. Use `App::into_router()` before adding the outer global layer. See [M6a verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) and [ADR 0006](adr/0006-handler-first-mvc-optional.md).

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

The Todo example **explicitly installs** `RequestIds::new().global(app)` from the optional `yetlet::request_id` module. A per-provider Arc<AtomicU64> starting at 1 generates a numeric RequestId extension and x-yetlet-request-id response header. M6 also supports group-scoped installation; the core has no dependency on this provider. The native home route intentionally selects only the number for an inert data-request-id attribute. No request extensions are automatically serialized.

The example counter resets for a new application instance and is not globally unique across server nodes. Applications needing trace correlation should opt into a provider with an intentional ID and trusted propagation policy. See [ADR 0005](adr/0005-opt-in-capability-modules.md).

### Existing demo integration

M6 moved numeric correlation into `crates/yetlet-request-id`; the Todo app installs it explicitly at finalization. The example's `pipeline::with_api_scope` remains an ordinary `App::configure` setup function applying the API marker/header only to matching API paths.

The header-only `action_headers` middleware remains before handler body extraction. M6 uses native State/Json handler parameters and an explicit route-level DefaultBodyLimit::max(4096). Json consumes the body once; rejection mapping preserves the original error envelope/status. This is route-selected JSON buffering, not unconditional framework buffering.

The SSR root now uses Option<Extension<RequestId>> and returns Page, selecting only the numeric ID for its data attribute rather than receiving or serializing the whole RequestContext. Existing tests still prove raw cookies/private headers are not copied. SearchStateV1 and browser source remain untouched. M4 authentication/origin limitations still apply; correlation is not a production identity system.

### Why this remains a small integration

Two production modules are added to the existing server adapter. They reuse http request parts, Body/Extensions, Axum extractors/Next, and Tower Layer/Service. There is no custom dynamic middleware registry, new crate family, duplicate response model, general dependency injection, persistent session, or browser event channel.

This is **not zero overhead**: S is cloned during extraction, native middleware has its own future/service costs, and with_global adds a fallback-only router envelope. RequestId/scope middleware are demo behavior with their own small per-request work. Native latency, allocation counts and throughput were not benchmarked at this P0 checkpoint. The browser build comparison below is separate from those unmeasured native costs.

## API reference

Use [Web source/rustdoc](../crates/yetlet/src/web/mod.rs),
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

The counts and byte comparisons below record the original P0 checkpoint. Complete M6 integration and its migrated Todo results are recorded separately in [M6 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements); the original evidence is retained, not relabeled as a new run.

Focused commands observed passing during implementation:

```bash
cargo test -p yetlet --features server-axum --all-targets --offline
cargo test -p yetlet --features server-axum -p yetlet-todo-server --all-targets --offline
```

The new 14 core integration tests cover scoped ordering, reverse unwind, early response, metadata, query/path failures, pre-routing rewriting, 12 simultaneous request contexts, error responses, native tower-http layer composition, deferred body polling, preserved trailers, stream errors and unread short-circuited bodies. Two demo integration tests cover actual API/SSR paths, safe render selection and outer response headers on 404/405/415.

The following commands are the original historical closeout. Current Rust
equivalents are `cargo xtask check docs`, `cargo xtask build browser` and
`cargo xtask smoke browser-chrome`; fresh smoke evidence uses
`target/m12/yt085/browser-chrome-*/`. Their migration acceptance is recorded
separately in [YT-085 verification](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-085-VERIFICATION.md).

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
- [YT-029 deadline/overhead research](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing) now records an opt-in application-owned native Tower timeout, cancellation/body limits and matched in-process release measurements. This completes the bounded research item, not a supported core API or production benchmark. General provider policy and additional streaming/backpressure qualification remain deferred.
- Request-local extensions do not prevent a developer from deliberately inserting the same Arc into every request; the framework only avoids doing so implicitly.
- M5 and M6 record local measurements, not a production performance/security certification. Public release and remaining operational-provider gates remain separate.

</details>

## Handler and typed action boundaries

Local `Command`/`ActionExecutor` execution need not implement serialization.
Only the HTTP executor requires serializable semantic input/output. Raw DOM
events stay local, and non-idempotent commands are not automatically retried.
The original anonymous Todo API used a per-router Mutex store, strict command
fields, a 4 KiB JSON body cap, up to 100 items and trimmed 1–100-character
non-control titles. Its verified errors were 400 unknown fields, 403 Origin,
404 unknown ID, 413 body limit, 415 media type and 422 invalid title; failed
commands did not advance revision. Origin comparison assumed direct HTTP and
was neither authentication nor credentialed CSRF protection. Do not transfer
that demo's policy to a trusted TLS proxy without explicit host/scheme policy.

M6 preserves native State/Path/Query/Json, redirects, response tuples and
`IntoResponse` errors. `Page` provides escaped SSR; it does not add HTTP imports
to shared View. `route_layer` affects already-registered matched routes;
unmatched paths need an explicit global wrapper. RequestContext owns request
parts/body and typed request-local extensions, separately from shared app state.
It never serializes cookies, arbitrary extensions or server handles into HTML.
The [sample acceptance sections](SAMPLES.md#sample-acceptance-and-source-boundaries)
distinguish REST's native 404/405 header boundary, middleware body errors,
stateless forms, file uploads and bounded SSE delivery.
