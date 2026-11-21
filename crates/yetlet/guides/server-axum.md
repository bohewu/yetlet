# yetlet::web

Build native JSON endpoints and optional HTML pages with a small facade over
Axum/Tower. The Web capability does not install browser, Session or configuration
services.

**Current-checkout API.** Select `web` for native HTTP and add `html` for SSR.
Published SDK `0.9.0-alpha.8` still uses the `server-axum` feature and
`yetlet::server_axum` imports; its artifact has not changed. The commands and
imports below target this checkout. Registry users should follow the
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
| `web` | `App`, native extractors/responses, RequestContext, Router/Tower integration and advanced `HttpWorker`; no View or HTML |
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

The advanced `yetlet::web::HttpWorker` is the explicitly selected owned-thread
alternative. `start` retains the loopback ephemeral-port default; `start_on`
takes an explicit address. Its HTTP 200 readiness probe is explicit, and the
caller retains the worker through stop, Pending and observation of the original
Join. Dropping its handle cannot prove completion. This low-level API retains
its existing worker contract; managed WebHost drivers and their default
readiness policy are separate work.

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
