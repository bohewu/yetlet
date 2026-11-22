# yetlet::web

Build native JSON endpoints and optional HTML pages with a small facade over
Axum/Tower. The Web capability does not install browser, Session or configuration
services.

Select `web` for native HTTP and add `html` for SSR. SDK alpha.8 uses the
`server-axum` feature and `yetlet::server_axum` imports. For that API, follow the
[first-app guide](https://github.com/bohewu/yetlet/blob/master/docs/FIRST-APP.md)
at their selected published revision.


## Prerequisites

Use this checkout with Rust/Cargo 1.99.0 and prepared locked dependencies, or allow
Cargo to fetch them. Run the example below from the repository root. No Node.js,
WASM target or optional provider is required for this path.

## Minimal integration

### Choose the required capability

| Checkout selection | Available surface |
|---|---|
| `web` | `App`, managed `WebHost`, native extractors/responses, RequestContext, Router/Tower integration and advanced `HttpWorker`; no View or HTML |
| `html` | Pure HTML serialization with View; no HTTP serving |
| `web,html` | Web plus `Page`, `App::page`, document/render helpers and View-related prelude imports |

An API-only application can use the Web prelude or explicit imports without
enabling HTML:

```rust
use yetlet::web::{App, response::Json};

async fn status() -> Json<&'static str> {
    Json("ready")
}

let app: App = App::new().get("/api/status", status);
```

Construction does not bind or start a runtime. Call `app.serve().await` from the
application's Tokio runtime when ready to serve it.

### Start a native page and JSON API

From the repository root, run the existing complete example:

```bash
cargo run --locked -p yetlet --features web,html --example server-axum-minimal
```

The checkout example uses this package's `App::serve()` and opts into the curated
`yetlet::web::prelude` for imports such as `App`, `View` and `Json`.
`App::serve()` binds exactly to `127.0.0.1:7878` for local plain HTTP, serving
`/` and `/api/status`. Use `App::serve_on(address)` to choose an explicit bind
address. All-interface addresses expose the listener on other network interfaces.
Both methods run on the application's Tokio runtime and return bind errors.

Use `into_router()` and native Tokio/Axum for listener ownership, direct TLS or
application-owned shutdown with `with_graceful_shutdown`. This checkout
explicit-listener example requires `web,html`.

```rust
use yetlet::view::View;
use yetlet::web::{App, response::Json};

async fn home() -> View {
    View::text("Hello from Yetlet")
}
async fn status() -> Json<&'static str> {
    Json("ready")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app: axum::Router = App::new()
        .page("/", "Yetlet", home)
        .group("/api", |api| api.get("/status", status))
        .into_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Expected result

The explicit-listener code above serves HTML at `http://127.0.0.1:3000/` and JSON
at `/api/status`. The checkout command uses `http://127.0.0.1:7878/`.
`Page` produces escaped complete documents with the default strict CSP. Ctrl+C
stops the foreground listener; a Router setup alone does not bind a listener.

## Execution ownership

`App::serve` and `serve_on` use the caller's Tokio runtime and create no dedicated
thread. Their consuming futures provide ordinary HTTP serving; cancelling one
does not promise graceful drain or framework-managed resource finalization.

### Manage multiple HTTP services

`App::into_host()` and `WebHost::new(router)` produce inert Host descriptions.
They use the built-in HostGroup coordinator for startup, admission, graceful
drain, original completion and final dependency release. Inside the
application's async entry point:

```rust
use yetlet::{HostGroup, web::App};

let public = App::new()
    .get("/api", async || "public")
    .into_host()
    .with_name("public")
    .with_address("127.0.0.1:8080");
let admin = App::new()
    .get("/status", async || "ready")
    .into_host()
    .with_name("admin")
    .with_address("127.0.0.1:8081");

let mut hosts = HostGroup::new()
    .with_host(public)
    .with_host(admin)
    .build()?;
let shutdown = hosts.shutdown_handle();
// Give shutdown to the application's shutdown policy.
if let Err(failure) = hosts.run_async().await {
    eprintln!("Host group failed: {failure}");
}
```

The [finite external consumer](https://github.com/bohewu/yetlet/blob/master/examples/external-consumers/web-hosts/README.md)
shows real requests, address observation and explicit terminal shutdown. Neither
construction nor build binds a listener or starts a thread. A WebHost defaults
to `127.0.0.1:7878`; use distinct addresses or port 0 for multiple services.
Capture `host.handle()` before moving a description into the group.
`WebHandle::local_addr()` reports the actual bound address, including port 0,
and remains historical metadata after stop. `is_ready()` and
`is_admitting()` report different stages; an address alone is not readiness.

### Select a driver only when needed

| Drive / selection | Execution |
|---|---|
| Fresh `hosts.run_async().await` or caller polling | Web uses the application's Tokio runtime, including a current-thread runtime. The application enables I/O and time and keeps that runtime alive through terminal shutdown. |
| Fresh `hosts.run()` | Web creates an owned service thread with its own current-thread Tokio runtime and a dedicated join observer. Core parks the calling thread between progress checks; Core itself creates no runtime, pool or helper thread. |
| `host.with_driver(WebDriver::OwnedThread)` | Explicit service-thread placement, including when the retained group is driven asynchronously. |
| `host.with_driver(WebDriver::CallerRuntime)` | Explicit caller-runtime placement. A blocking drive rejects this combination before any resource acquisition. |

Driver preparation runs for the complete plan before acquisition. The selected
mode is retained; resuming a cancelled async drive keeps its original driver.
Calling blocking `run()` on a still-live Async-selected owner returns
`HostRunError::ModeMismatch` without stopping, replacing or dropping that owner.
There is no automatic fallback. Core types for advanced customization live in
`yetlet::composition`.

The two Web drivers retain the same service implementation and use one Core
lifecycle. OwnedThread uses two dedicated threads per Host: the service thread
and its join observer. The observer waits for the original service Join,
including application thread-local destruction. It runs no Router, callbacks,
wakers or error-payload conversion; the retained caller interprets the raw result.
This keeps application cleanup off both async and blocking group polls. After
the service-body notification, async mode uses yielding timer rechecks and
blocking mode uses parked rechecks until the observer finishes. The caller then
observes only the observer's std-only exit tail. This is ordinary std completion
observation, not a hard latency guarantee. No shared pool is introduced.
Completion requires the original Join, not merely a readiness/completion signal.
The service-exit notification precedes owned-runtime destruction, so waiting for
application blocking work during teardown cannot suppress sibling shutdown.
Original Join still waits for runtime and thread-local cleanup before resources
can be released.

### Readiness and cancellation

Default readiness requires a bound listener and an actually initialized serving
driver. An API-only Router may return 404 at `GET /` and still be ready.
Ordinary admission opens only after every Host is ready. While closed, the
adapter returns 503 without entering the application Router.
If a ready service terminates while another Host is still starting, the group
keeps admission closed, stops the same services and waits for their original
completion. A closure notification does not replace the original failure or
permit early dependency release.

For an explicit startup check, configure
`with_http_probe(HttpProbe::new(method, path, expected_status).with_timeout(budget))`.
The exact method and path-and-query are the only startup admission exception;
the configured status must match before the retained absolute deadline. Resuming
a cancelled drive does not restart that deadline or accept a late buffered
response. Other work stays fenced, and shutdown removes the exception. Probe
failure rolls back the same retained owners.

Drain covers the HTTP requests/connections tracked by Axum's server.
Handler-spawned background tasks and upgraded/application-owned work need their
own managed completion obligations when they must share resource lifetimes.

Keep the built `hosts` outside a cancelled task or scope. Cancelling only
`run_async()` leaves acquisition, service progress, original completion and
cleanup state in that owner. Resume `run_async()`, or call
`shutdown.request_shutdown()` and drive it again to terminal completion.
A deadline or Pending result retains obligations; dropping the owner is not
graceful-completion evidence. Terminal errors are borrowed from the owner so
their original identity and later failures remain inspectable.

### Customize resource management

Custom Hosts return the same `ResourceGroup` used by built-in adapters. Start
with `Host::into_resources()`, append passive values with `with_resource`,
or register explicit acquisition/finalization with `with_managed_resource`.
`map_activity` can wrap the configured WebActivity to map `WebError` into the
application's common error type. Existing managed descriptors must use that
same error type; Core does not require it to be Clone, Send or Sync.
A custom activity's `open_admission` returns `AdmissionOutcome::Opened` or
`Closed`. Return `Closed` when termination has already fenced the gate, then
retain the actual execution outcome for `poll_join`. The group stops opening
later Hosts and rolls back without manufacturing an admission error.

Managed dependencies remain retained through all original Host completions.
After every original Host completion, the reverse final-release pass retires
each activity's dependency-using payloads and drops that activity before
releasing its earlier resources. The external consumer includes a real SQLite
resource example; databases remain an application choice and add no SDK dependency.

### Advanced low-level worker

The existing `yetlet::web::HttpWorker` keeps its separate low-level contract.
`start` retains the loopback ephemeral-port default; `start_on` takes an
explicit address. Its HTTP 200 probe is explicit and synchronous, and the
caller retains the worker through stop, Pending and observation of the original
Join. Its blocking probe/observation methods are not used by managed WebHost.
Dropping its handle cannot prove completion.

## Configuration and errors

### Native interoperability

`App<S>` is a thin facade; `into_router()` returns a native Axum Router. `.page(path, title, handler)` accepts a no-argument async View handler. Parameterized/fallible handlers use `.get`, native `Path`, `Query`, `State` or `Json`, and return `Page` or `Result<Page, E>`. Native JSON, redirects, statuses and headers retain Axum semantics. `configure`/`try_configure` execute ordinary typed setup functions during construction, without Controllers, DI, plugin discovery or a second model binder.

`RequestContext<S>` owns request parts/body and typed request extensions separately from application state. Native middleware remains usable directly; `with_global` wraps a fully assembled Router before routing. Follow [HTTP pipeline ordering/body semantics](https://github.com/bohewu/yetlet/blob/master/docs/HTTP-PIPELINE.md) when composing layers. SQL/storage remain application-owned choices.

### Documents and errors

`Page::new`, `render_document` and `render_page` use the legacy `zh-Hant` root language. For another language, validate `LanguageTag` and select `DocumentMetadata` with `Page::with_metadata` or the metadata-aware render functions. The contract is title/language plus fixed charset/viewport; no arbitrary head DSL is installed. Language validation is conservative ASCII shape validation, not a complete BCP 47 registry.

Views are escaped through `yetlet::html`. Rendering errors become a generic HTTP 500 through `Page`/`HtmlPageError`; HTML responses carry the strict default CSP. Full SSR buffers the document and needs no JS/WASM. Browser assets/loaders are explicitly application-owned build outputs. Application-owned streaming and HTTP deadline research remain app-owned opt-ins, without a core streaming/suspense/deadline API.

## API reference

Use the [source/rustdoc entry](../src/web/mod.rs) and [current API reference](https://github.com/bohewu/yetlet/blob/master/docs/API-REFERENCE.md).
Native upstream routing and extraction contracts remain Axum/Tower-owned.

### License and compatibility

Dual-licensed under [MIT](https://github.com/bohewu/yetlet/blob/master/LICENSE-MIT) or [Apache-2.0](https://github.com/bohewu/yetlet/blob/master/LICENSE-APACHE).
This guide describes the current checkout; published alpha.8 retains its
original feature/import contract. APIs remain Experimental. Read
[compatibility](https://github.com/bohewu/yetlet/blob/master/docs/COMPATIBILITY.md) and the
[current API reference](https://github.com/bohewu/yetlet/blob/master/docs/API-REFERENCE.md) at the matching revision.
The historical evidence below retains its original candidate and scope; it does
not qualify every later checkout or promise production support.

<details>
<summary>Historical delivery context and evidence</summary>

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet::server_axum` adapts Yetlet Views to ordinary Axum/Tower HTTP applications. Dependencies are Axum, Tower, serde, `yetlet::view` and `yetlet::html`; browser runtime, Session and request correlation are not installed.

</details>
