# Yetlet

**Rust UI, HTML first.**

Build server-rendered pages and HTTP APIs with ordinary Axum handlers. Share
Rust Views with optional browser WASM when a screen needs local interaction.
Yetlet is for Rust teams that want to keep UI composition, server behavior and
explicit browser enhancements in one language.

## Why Yetlet

- **Start with HTML.** Serve useful pages and native forms without JavaScript
  or a persistent server-side UI connection.
- **Keep native Rust HTTP.** Use Axum extractors, Tower middleware and typed state
  alongside a small `App`/`Page` facade.
- **Share Views across targets.** Reuse Rust UI values for SSR and opt-in browser
  DOM updates; keep local interactions and remote actions explicit.
- **Choose capabilities as needed.** Add Session, authentication, rate limiting,
  configuration or request correlation through explicit dependencies and setup.
- **Use bounded output policies.** HTML escaping and strict page CSP are built
  into the server rendering path; application authorization stays explicit.

## Start with native SSR

**Experimental alpha.** The two-package SDK is available on crates.io:
[`yetlet`](https://crates.io/crates/yetlet/0.9.0-alpha.5) and
[`yetlet-browser`](https://crates.io/crates/yetlet-browser/0.9.0-alpha.5), version
`0.9.0-alpha.5`. The [matching GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.5)
contains the archives and verification material. To create a standalone app,
follow [Build your first Yetlet app](docs/FIRST-APP.md); the optional CLI is a
separately source-installed preview.

Use the pinned Rust/Cargo 1.99.0 toolchain. Native SSR needs no Node.js or WASM
toolchain. APIs may change; see [compatibility](docs/COMPATIBILITY.md).

From the repository root:

```sh
cargo run --locked -p yetlet --features server-axum --example server-axum-minimal
```

Open `http://127.0.0.1:3000/` for HTML and
`http://127.0.0.1:3000/api/status` for JSON. Stop the server with Ctrl+C.
The complete [running example](crates/yetlet/examples/server_axum/minimal.rs) is:

```rust
use yetlet::view::View;
use yetlet::server_axum::{App, response::Json};

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

`App::into_router()` returns a native Axum Router. For parameters or fallible
handlers, use ordinary extractors and `Page` or `Result<Page, E>`; see the
[server guide](crates/yetlet/guides/server-axum.md).

## Choose your next step

| I want to… | Start here |
|---|---|
| Create my own standalone app | [Build your first Yetlet app](docs/FIRST-APP.md) |
| Write Views and reusable components | [View/component guide](docs/VIEW-COMPONENTS.md) |
| Build an SSR page or JSON API | [Hello World](examples/hello-world/README.md) · [REST API](examples/rest-api/README.md) |
| Add browser interaction | [Todo and Search Island](examples/todo-server/README.md) · [browser DOM guide](crates/yetlet-browser/guides/dom.md) |
| Handle forms and Session state | [Forms](examples/forms-validation/README.md) · [Session cart](examples/cart-server/README.md) |
| Resolve identity and protect routes | [Authentication guide](crates/yetlet/guides/auth.md) · [Admin reference](examples/admin-dashboard/README.md) |
| Set endpoint or group rate limits | [Rate-limit guide](crates/yetlet/guides/rate-limit.md) |
| Load typed settings and secrets | [Configuration](docs/CONFIGURATION.md) · [Secret providers](docs/SECRET-PROVIDERS.md) |
| Explore a modular SQL-backed application | [Modular Shop](examples/modular-shop/README.md) · [Issue Tracker starter](examples/production-starter/README.md) |

The [sample catalog](docs/SAMPLES.md) lists existing applications, routes and
limits. Application guides distinguish synthetic login from a real identity
provider and authoritative state from optional display caching.

Browser examples additionally need the WASM target and a wasm-bindgen CLI matching
`Cargo.lock` (0.2.129). Prepare them once in a POSIX shell, then follow the Todo guide:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"
export PATH="$HOME/.local/share/yetlet-tools/bin:$PATH"
```

## Documentation

- [Current API reference](docs/API-REFERENCE.md): packages, features, ownership,
  errors and local rustdoc.
- [Troubleshooting](docs/TROUBLESHOOTING.md): symptoms, likely causes and linked fixes.
- [Opt-in quick reference](docs/OPT-IN-QUICK-REFERENCE.md): features, startup wiring,
  application settings and defaults.
- [HTTP pipeline](docs/HTTP-PIPELINE.md) and [opt-in composition](docs/EXTENSIONS.md).
- [CLI guide](tools/yetlet-cli/README.md): scaffolding, build/run/watch and selected
  configuration generators.
- [Documentation index](docs/README.md): further guides and retained history.

## Project and license

Planned work belongs in [Roadmap](docs/ROADMAP.md); tracked tasks belong in
[Backlog](docs/BACKLOG.md). Reporting and participation follow
[Contributing](CONTRIBUTING.md) and the [security policy](SECURITY.md).
Maintainers: [development/PR workflow](docs/DEVELOPMENT.md) and
[new-version release workflow](docs/RELEASING.md).

Licensed under either [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
