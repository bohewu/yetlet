# yetlet-server-axum

Build native HTML pages and JSON endpoints with a small facade over Axum/Tower.
The server adapter does not install browser, Session or configuration capabilities.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use this checkout with Rust/Cargo 1.99.0 and prepared locked dependencies, or allow
Cargo to fetch them. Run the example below from the repository root. No Node.js,
WASM target or optional provider is required for this path.

## Minimal integration

### Start a native page and JSON API

From the repository root, run the existing complete example:

```bash
cargo run --locked -p yetlet-server-axum --example minimal
```

Open `http://127.0.0.1:3000/` and `/api/status`. The code below is copied verbatim from [minimal.rs](examples/minimal.rs):

```rust
use yetlet_core::View;
use yetlet_server_axum::{App, response::Json};

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

The example serves HTML at `http://127.0.0.1:3000/` and JSON at `/api/status`.
`Page` produces escaped complete documents with the default strict CSP. Ctrl+C
stops the foreground listener; a Router setup alone does not bind a listener.

## Configuration and errors

### Native interoperability

`App<S>` is a thin facade; `into_router()` returns a native Axum Router. `.page(path, title, handler)` accepts a no-argument async View handler. Parameterized/fallible handlers use `.get`, native `Path`, `Query`, `State` or `Json`, and return `Page` or `Result<Page, E>`. Native JSON, redirects, statuses and headers retain Axum semantics. `configure`/`try_configure` execute ordinary typed setup functions during construction, without Controllers, DI, plugin discovery or a second model binder.

`RequestContext<S>` owns request parts/body and typed request extensions separately from application state. Native middleware remains usable directly; `with_global` wraps a fully assembled Router before routing. Follow [HTTP pipeline ordering/body semantics](../../docs/HTTP-PIPELINE.md) when composing layers. SQL/storage remain application-owned choices.

### Documents and errors

`Page::new`, `render_document` and `render_page` use the legacy `zh-Hant` root language. For another language, validate `LanguageTag` and select `DocumentMetadata` with `Page::with_metadata` or the metadata-aware render functions. The contract is title/language plus fixed charset/viewport; no arbitrary head DSL is installed. Language validation is conservative ASCII shape validation, not a complete BCP 47 registry.

Views are escaped through `yetlet-html`. Rendering errors become a generic HTTP 500 through `Page`/`HtmlPageError`; HTML responses carry the strict default CSP. Full SSR buffers the document and needs no JS/WASM. Browser assets/loaders are explicitly application-owned build outputs. [Streaming](../../docs/YT-021-STREAMING-SSR-RESEARCH.md) and [deadline](../../docs/YT-029-HTTP-DEADLINE-RESEARCH.md) research remain app-owned opt-ins, without a core streaming/suspense/deadline API.

## API reference

Use the [source/rustdoc entry](src/lib.rs) and [current API reference](../../docs/API-REFERENCE.md).
Native upstream routing and extraction contracts remain Axum/Tower-owned.

### License and compatibility

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../docs/COMPATIBILITY.md), [API classifications](../../docs/API-CONTRACTS.md) and [developer guides](../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.

<details>
<summary>Historical delivery context and evidence</summary>

**Experimental `0.9.0-alpha.4`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet-server-axum` adapts Yetlet Views to ordinary Axum/Tower HTTP applications. Dependencies are Axum, Tower, serde, `yetlet-core` and `yetlet-html`; browser runtime, Session and request correlation are not installed.

</details>
