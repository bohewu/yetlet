# Explicit Module Composition

Compose explicitly selected capabilities through ordinary setup functions, native
Axum/Tower middleware, typed State and request Extensions. No capability registry
or mandatory Settings/Auth/Session installation is involved.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use the pinned Rust/Cargo toolchain and a native Axum/Yetlet application. Select
each optional crate explicitly. Examples below are setup fragments; keep resource
handles in typed state and register routes before applying the intended scope.
For an existing complete app, use the [REST guide](../examples/rest-api/README.md).

## Minimal integration

### The baseline has no optional capability

```rust
use yetlet::view::View;
use yetlet::server_axum::App;

async fn home() -> View { View::text("Hello") }
let app: axum::Router = App::new().page("/", "Home", home).into_router();
```

This requires no `yetlet::request_id`, Session, Cookie Auth or Cache dependency. The runnable `crates/yetlet-server-axum/examples/minimal.rs` example uses only the server adapter and existing core. Cargo's normal dependency graph for the server adapter excludes the optional provider. Building `--workspace` intentionally tests all workspace members; that is different from application dependencies.

### The independent opt-in proof: request correlation

`yetlet::request_id` is an optional crate, depending on the server adapter, **not the reverse**. It extracts the old Todo demo's numeric ID behavior into an explicitly selected provider.

```rust
use yetlet::request_id::{RequestId, RequestIds};
use yetlet::server_axum::{App, extract::Extension};

async fn item(Extension(id): Extension<RequestId>) -> String { id.0.to_string() }
let app: axum::Router = App::new()
    .get("/health", || async { "ok" })
    .group("/api", |group| {
        group.get("/item", item)
            .configure(|group| RequestIds::new().scoped(group))
    })
    .into_router();
```

Only matching `/api/item` requests receive `RequestId` and `x-yetlet-request-id`; `/health`, unknown paths, and sibling groups do not. Register all group routes before calling `scoped`, because it delegates to native `Router::route_layer`. A matched path with an unsupported method still has native 405/Allow semantics. Do not install both global and scoped generators on the same request path unless replacing IDs is deliberate.

Global correlation instead **finalizes** a fully assembled `App<()>`:

```rust
use yetlet::request_id::RequestIds;
use yetlet::server_axum::App;
let router = RequestIds::new().global(App::new().get("/", || async { "ok" }));
```

This is the existing `with_global` pre-routing boundary. Supply application state and finish all routes first; adding routes to the returned outer Router can bypass that boundary. Global registration also observes 404, 405 and downstream short-circuit responses.

For validated configuration, use `RequestIds::try_new(first)` and propagate `InvalidStart` through `try_configure`. Starts must be 1 through `u64::MAX - 1`. An exhausted counter returns 503 rather than wrapping. Clones share one counter; fresh instances restart it. IDs are process/provider-local, not distributed trace IDs, persistent identities, secrets or authentication credentials. Inbound headers/extensions do not determine the generated value.

The provider adds the typed request extension and response header only. It does not read/clone/buffer the body or serialize request context into HTML. The Todo root explicitly selects the numeric ID using an optional native `Extension<RequestId>` extractor; no whole request is transferred to SSR.

## Expected result

The baseline builds a page Router without optional provider state. The selected
correlation example adds a server-generated numeric ID only to its assembled
`/api` scope; independent providers have independent counters. No setup function
implicitly installs Session, Auth, caches or configuration.

## Configuration and errors

### Setup functions, not plugin objects

`App::configure` accepts `FnOnce(App<S>) -> App<S>`, executes it once at setup, and returns the configured app. `App::try_configure` accepts the fallible version and returns the module's own typed error. Setup does not run on each request; only an explicitly installed middleware has per-request work.

A module can register routes, call `route_layer`, customize its native Router through `map_router`, and consume explicitly passed configuration. Shared resource handles belong in `with_state` (usually Arc-backed); request-local values belong in typed Extensions. There is no ambient service resolution or automatic initialization.

### Bounded opt-in Session provider

`yetlet::session` is another optional crate, depending on the server adapter rather than the reverse. An application supplies a validated `SessionConfig` and installs `SessionManager<T>` only on the routes that need anonymous server-side state:

```rust
use std::time::Duration;
use yetlet::server_axum::App;
use yetlet::session::{SessionConfig, SessionManager};

#[derive(Clone, Default)]
struct Cart;
let sessions = SessionManager::<Cart>::new(
    SessionConfig::new("cart", Duration::from_secs(1800), 1024)?,
);
let app = App::new().group("/cart", move |group| {
    group.get("/", page).configure(|group| sessions.scoped(group))
});
# Ok::<_, yetlet::session::SessionConfigError>(app)
```

The provider generates opaque IDs and CSRF tokens from the operating-system RNG, stores data and absolute expiry in a bounded memory map, rejects malformed/duplicate cookies, never adopts an unknown supplied ID, and exposes typed synchronous read/update/rotate/destroy operations. It applies `HttpOnly`, `SameSite=Lax`, `Path=/`, bounded `Max-Age`, `no-store`, `Vary: Cookie` and `nosniff`; `Secure` is the default. The localhost cart opts out of Secure explicitly because it runs without TLS.

The Session lock is never held across downstream `await`. `try_update` mutates a clone and commits only on application success. Session is still not authentication, durable storage, a distributed provider or a substitute for authorization and production security review. See [M8c evidence](M8C-VERIFICATION.md).

### Separately selected experimental Redis Session

M10g qualifies the opt-in `yetlet::session_redis` nested workspace at single-host
Docker multi-instance, single-primary and no-persistence scope. It uses explicit
async operations and native scoped Axum middleware; the synchronous Memory API
and Core/default-server/browser graphs remain unchanged. See the [provider
README](../crates/yetlet/guides/session-redis.md), [selected Cookie contract](YT-066B-COOKIE-ORDERING.md)
and [bounded closeout](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11).
The pure-SSR Cookie contract preserves retired-ID rejection but permits login
loss under extreme response reordering. This is not a Stable Candidate API,
authentication system, production durability/HA result or mandatory dependency.

### Separately selected Experimental scene renderer

`yetlet_browser::wgpu` is a first-party **Experimental opt-in** module, not a
runtime plugin registry or automatic DOM/CSS accelerator. Its
[nested crate](../crates/yetlet-browser/guides/wgpu.md) has dependency-free,
CPU-only defaults and an explicit wasm32 `browser` feature. Consumers
select it as a local path dependency and explicitly create a per-canvas
owner; device initialization is never caused by default SSR, HTTP,
HTML or ordinary DOM usage. The application retains the scheduler,
semantic HTML alternatives, trusted selection IDs and lifecycle
bridge. Browser errors are local to that capability; queue completion
is not FPS. Its actual support is limited to the
[documented Experimental configuration](../crates/yetlet-browser/guides/wgpu-support.md),
without a speedup or future-3D API claim. See the
[YT-081 cross-crate API audit](YT-081-API-CONTRACT-AUDIT.md) for typed
error and disposal-contract differences.

### Designing another module

The separately selected [secret provider](SECRET-PROVIDERS.md) is an opt-in
native contract: applications explicitly construct `SettingsSecrets` from
already-loaded fields, pass `Arc<dyn SecretProvider>` to ordinary service
constructors and optionally retain it in application-owned State. The module
adds no middleware, ambient service resolution or core dependency.

Use a normal setup function where it suffices. Add narrow traits only for genuine replaceable resource contracts (for example a cache store), not for a central plugin registry. Keep storage-specific dependencies in the module/provider crate. A handler that does not need context, auth or session should remain an ordinary function.

The M8c baseline supplied bounded Memory Session. The subsequent Auth foundation
resolves/extracts verified identity, and its optional Memory bridge maps Session
references through application-owned current-account policy. Business
authorization remains application-owned. Existing Session cookie handling is
implemented; a standalone `yetlet-cookies` crate or complete generic Cookie Auth
product is not selected. Cache providers remain unimplemented. Session is not
authentication; data cache is not HTTP output caching. Key rotation for protected
client-side data, distributed-store durability/HA, trusted proxy identity and
production qualification cannot be inferred from typed registration.

See [ADR 0005](adr/0005-opt-in-capability-modules.md), [ADR 0006](adr/0006-handler-first-mvc-optional.md), [HTTP pipeline](HTTP-PIPELINE.md), and [M6 evidence](M6-VERIFICATION.md).

## API reference

Use the [server facade](../crates/yetlet/guides/server-axum.md),
[request-ID API](../crates/yetlet/guides/request-id.md),
[current capability map](CAPABILITIES.md) and [current API reference](API-REFERENCE.md).
The registration/research/adoption sequence below retains its original decisions;
current example options live in the [Board](../examples/lab-operations-board/README.md)
and [Shop](../examples/modular-shop/README.md) guides.

<details>
<summary>Historical delivery context and evidence</summary>

**Status:** Composition, request correlation, bounded Session providers, native Auth/Memory bridge and per-process rate limiting are implemented at their separately reviewed scopes. The [current capability map](CAPABILITIES.md) separates framework features, application-owned integrations and research-only candidates.
**Contract:** Ordinary Rust setup functions, native Axum/Tower middleware, typed application state and request extensions. No runtime plugin manager, Controller registry or DI container.

### M21 opt-in capabilities

The [M21 batch plan](YT-113-123-OPT-IN-CAPABILITIES.md) registers
YT-113–YT-123 as Todo at registration. [YT-113 contract research](YT-113-AUTHENTICATION-CONTRACT.md)
is Done after independent Reviewer PASS. The separate [yetlet-auth foundation](../crates/yetlet/guides/auth.md)
is Done at [YT-114 scope](YT-114-IMPLEMENTATION.md) after its separate independent Reviewer PASS;
YT-115 is Done at Memory bridge scope and YT-116/119/120 are Done at independently reviewed research scopes; [YT-117 rate limiting](YT-117-RATE-LIMIT.md) is Done at the separately selected per-process scope after independent Reviewer PASS; [YT-118 Audit research](YT-118-AUDIT-STUDY.md) is Done after independent Reviewer PASS; [YT-121 Cache research](YT-121-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-122 Output Cache research](YT-122-OUTPUT-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-123 Tracing/Correlation research](YT-123-TRACING-CORRELATION-STUDY.md) is Done after independent Reviewer PASS. The plan separates the `yetlet::auth` verified-identity contract from application-owned authorization and separately selected Session integration. Existing app-owned OIDC and opaque Bearer capabilities do not establish generic adapters. A separate Cookie crate, generic audit event/sink capability,
data/output caches and tracing remain unselected candidates;
the existing native Axum/Tower and module setup APIs stay authoritative.
Registration does not provide these runtime capabilities or qualify
authenticated production deployments.

The separately selected [YT-115 Memory bridge](YT-115-SESSION-AUTH.md) adds an
explicit `yetlet-auth/session` feature and native HTTP proof with independent Reviewer PASS. Default Auth still has no Session dependency. Existing Memory sync and
Redis async provider APIs remain unchanged.

The separately selected [YT-117 rate limiter](YT-117-RATE-LIMIT.md) uses ordinary
Tower/Axum method scopes and independent per-process accounting. Application-owned
startup configuration defaults off and omits limiter construction and request
work while disabled. No Auth dependency, framework-wide policy registry, live
reload or distributed quota is selected. The bounded implementation is Done
after focused verification and independent Reviewer PASS. The owner later
selected the [existing Settings loader in its example](YT-117-RATE-LIMIT.md#settings-loader-follow-up--2026-10-22),
with separate focused verification and independent Reviewer PASS. The loader is
an example development dependency,
not a Core service or normal limiter dependency.

The owner-selected [YT-118 Audit study](YT-118-AUDIT-STUDY.md) compares Auth Notes,
Admin/Board transactional history, SQLite CRUD and diagnostic HTTP observation.
It recommends retaining application-owned event semantics and transaction/sink
composition; no audit crate, callback registry or automatic capture is selected.
Independent Reviewer PASS completes research only; existing history remains
application-owned and no new Audit API/runtime is selected.

The owner-selected [YT-121 Cache study](YT-121-CACHE-STUDY.md) compares
Starter/Shop business reads, SQLite/Board reads and static asset reuse. No
repeated data-cache adapter is demonstrated; retain typed app-owned composition.
Session, permission policy and limiter accounting remain authoritative state;
data caching does not authorize HTTP output reuse. Research is Done
after independent Reviewer PASS; that research selected no cache trait/crate or runtime.
The later separately authorized [YT-183 Shop reference](YT-183-CATALOG-CACHE-IMPLEMENTATION.md)
is an app-only default-off Moka integration accepted after independent Reviewer PASS
with zero issues/required followups;
it preserves Session/checkout authority and does not extract a framework cache API.

The owner-selected [YT-122 Output Cache study](YT-122-OUTPUT-CACHE-STUDY.md)
compares public assets and personalized SSR/API response lifecycles. Private
content stays uncached by default; Vary or a tenant/user key does not replace
no-store/current authorization. Retain existing header/static delivery; no
duplicated safe response store or generic output-cache API/runtime is selected.
Research is Done after independent Reviewer PASS.

The owner-selected [YT-123 Tracing/Correlation study](YT-123-TRACING-CORRELATION-STUDY.md)
compares RequestIds, selected SSR/API projection and Starter diagnostics with
native Rust tracing/Tower. Task and stream lifetimes, inbound-context trust
and sensitive-field projection remain explicit application decisions; no
repeated propagation adapter justifies extraction. Research is Done
after independent Reviewer PASS; no tracing crate/subscriber/exporter is selected.

### Multi-instance Auth follow-up

[YT-124](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md) is an owner-selected
documentation-only study, Done after independent Reviewer PASS. The Admin
reference already selects Valkey through `valkey` settings/features and the
compatible existing `yetlet::session_redis` package; its [two-app and TTL
evidence](YT-094-VALKEY-SESSIONS.md) is distinct from historical YT-066b3 Redis
execution. The new Auth crate currently bridges only Memory. The study defines
explicit async resolver composition, shared authority, lifecycle/freshness and
failure semantics; it adds no runtime feature, default provider or Core API.

The subsequent [YT-125 implementation slice](YT-125-VALKEY-AUTH-INTEGRATION.md)
selects an isolated application-owned async resolver, shared local authority and
two-process Valkey proof. It is Done after independent Reviewer PASS; existing framework/provider APIs
and default dependencies stay unchanged.

The owner subsequently selected [YT-126 Admin adoption](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md),
Done after focused verification and independent Reviewer PASS. It
selects application-owned native resolution/extraction with explicit feature
and runtime configuration and fresh shared local account policy. Existing
Memory/Valkey-only behavior and browser/default graphs remain separate. Native
and triage saves retain the documented pre-SQL consistency limit; no generic
bridge, framework API or cross-store commit fencing is selected.

</details>
