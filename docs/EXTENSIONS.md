# Explicit Module Composition

**Status:** M6 composition and request correlation are verified; M8c adds a bounded opt-in Session provider.
**Contract:** Ordinary Rust setup functions, native Axum/Tower middleware, typed application state and request extensions. No runtime plugin manager, Controller registry or DI container.

## The baseline has no optional capability

```rust
use yetlet_core::View;
use yetlet_server_axum::App;

async fn home() -> View { View::text("Hello") }
let app: axum::Router = App::new().page("/", "Home", home).into_router();
```

This requires no `yetlet-request-id`, Session, Cookie Auth or Cache dependency. The runnable `crates/yetlet-server-axum/examples/minimal.rs` example uses only the server adapter and existing core. Cargo's normal dependency graph for the server adapter excludes the optional provider. Building `--workspace` intentionally tests all workspace members; that is different from application dependencies.

## Setup functions, not plugin objects

`App::configure` accepts `FnOnce(App<S>) -> App<S>`, executes it once at setup, and returns the configured app. `App::try_configure` accepts the fallible version and returns the module's own typed error. Setup does not run on each request; only an explicitly installed middleware has per-request work.

A module can register routes, call `route_layer`, customize its native Router through `map_router`, and consume explicitly passed configuration. Shared resource handles belong in `with_state` (usually Arc-backed); request-local values belong in typed Extensions. There is no ambient service resolution or automatic initialization.

## The independent opt-in proof: request correlation

`yetlet-request-id` is an optional crate, depending on the server adapter, **not the reverse**. It extracts the old Todo demo's numeric ID behavior into an explicitly selected provider.

```rust
use yetlet_request_id::{RequestId, RequestIds};
use yetlet_server_axum::{App, extract::Extension};

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
use yetlet_request_id::RequestIds;
use yetlet_server_axum::App;
let router = RequestIds::new().global(App::new().get("/", || async { "ok" }));
```

This is the existing `with_global` pre-routing boundary. Supply application state and finish all routes first; adding routes to the returned outer Router can bypass that boundary. Global registration also observes 404, 405 and downstream short-circuit responses.

For validated configuration, use `RequestIds::try_new(first)` and propagate `InvalidStart` through `try_configure`. Starts must be 1 through `u64::MAX - 1`. An exhausted counter returns 503 rather than wrapping. Clones share one counter; fresh instances restart it. IDs are process/provider-local, not distributed trace IDs, persistent identities, secrets or authentication credentials. Inbound headers/extensions do not determine the generated value.

The provider adds the typed request extension and response header only. It does not read/clone/buffer the body or serialize request context into HTML. The Todo root explicitly selects the numeric ID using an optional native `Extension<RequestId>` extractor; no whole request is transferred to SSR.

## Bounded opt-in Session provider

`yetlet-session` is another optional crate, depending on the server adapter rather than the reverse. An application supplies a validated `SessionConfig` and installs `SessionManager<T>` only on the routes that need anonymous server-side state:

```rust
use std::time::Duration;
use yetlet_server_axum::App;
use yetlet_session::{SessionConfig, SessionManager};

#[derive(Clone, Default)]
struct Cart;
let sessions = SessionManager::<Cart>::new(
    SessionConfig::new("cart", Duration::from_secs(1800), 1024)?,
);
let app = App::new().group("/cart", move |group| {
    group.get("/", page).configure(|group| sessions.scoped(group))
});
# Ok::<_, yetlet_session::SessionConfigError>(app)
```

The provider generates opaque IDs and CSRF tokens from the operating-system RNG, stores data and absolute expiry in a bounded memory map, rejects malformed/duplicate cookies, never adopts an unknown supplied ID, and exposes typed synchronous read/update/rotate/destroy operations. It applies `HttpOnly`, `SameSite=Lax`, `Path=/`, bounded `Max-Age`, `no-store`, `Vary: Cookie` and `nosniff`; `Secure` is the default. The localhost cart opts out of Secure explicitly because it runs without TLS.

The Session lock is never held across downstream `await`. `try_update` mutates a clone and commits only on application success. Session is still not authentication, durable storage, a distributed provider or a substitute for authorization and production security review. See [M8c evidence](M8C-VERIFICATION.md).

## Designing another module

Use a normal setup function where it suffices. Add narrow traits only for genuine replaceable resource contracts (for example a cache store), not for a central plugin registry. Keep storage-specific dependencies in the module/provider crate. A handler that does not need context, auth or session should remain an ordinary function.

M8c implements only the bounded memory Session provider described above. Cookie authentication, authorization and cache providers remain unimplemented. Session is not authentication; data cache is not HTTP output caching. Key rotation for protected client-side data, shared/distributed storage, login-time fixation rules, authorization, trusted proxy identity and production qualification cannot be inferred from typed registration.

See [ADR 0005](adr/0005-opt-in-capability-modules.md), [ADR 0006](adr/0006-handler-first-mvc-optional.md), [HTTP pipeline](HTTP-PIPELINE.md), and [M6 evidence](M6-VERIFICATION.md).
