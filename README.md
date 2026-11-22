# Yetlet

**A small, composable Rust application framework with explicit capabilities.**

Yetlet separates application lifetimes, HTTP services, shared Views and rendering.
Choose the capabilities an application needs. The published Web/SSR SDK described
below renders useful HTML without JavaScript and adds browser interaction only
when selected. The [development checkout](#development-checkout) also provides
built-in Core lifetimes and optional Web, Native, UDP and renderer modules.

Build server-rendered pages and HTTP APIs with ordinary Axum handlers. Share
Rust Views with optional browser WASM when a screen needs local interaction.

## Why Yetlet

- **First-class HTML rendering.** Serve useful pages and native forms without
  JavaScript or a persistent server-side UI connection.
- **Keep native Rust HTTP.** Use Axum extractors, Tower middleware and typed state
  alongside a small `App`/`Page` facade.
- **Share Views across targets.** Reuse Rust UI values for SSR and opt-in browser
  DOM updates; keep local interactions and remote actions explicit.
- **Choose capabilities as needed.** Add Session, authentication, rate limiting,
  configuration or request correlation through explicit dependencies and setup.
- **Use bounded output policies.** HTML escaping and strict page CSP are built
  into the server rendering path; application authorization stays explicit.

## Current published releases

**Published SDK:** `0.9.0-alpha.8` (`yetlet` / `yetlet-browser`).
**Published CLI:** `0.4.1` (`yetlet-cli`).
The [SDK publication record](docs/ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
and [CLI publication record](docs/CLI-041-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-17)
retain exact GitHub/crates.io archives, signatures and consumer evidence. The SDK
includes typed Tag Builders, `App::serve()`, `App::serve_on()` and the optional
Axum adapter Prelude. The independent CLI resolves SDK versions, writes exact
app pins and locks, and generates simple native startup with verified SDK
alpha.8. Its opt-in native/browser Style templates use a private app CSS
generator. The Style walkthroughs select SDK alpha.7 explicitly.

The quickstart follows the verified Reader Edition in [site/pages.toml](site/pages.toml).
Each SDK or CLI publication includes a reviewed reader, README and Pages update
before release closeout. Development manifest changes alone do not promote it.

## Start with native SSR

**Experimental alpha.** This quickstart uses SDK `0.9.0-alpha.8` and the
independently versioned CLI `0.4.1`. The SDK is available
on crates.io:
[`yetlet`](https://crates.io/crates/yetlet/0.9.0-alpha.8) and
[`yetlet-browser`](https://crates.io/crates/yetlet-browser/0.9.0-alpha.8), version
`0.9.0-alpha.8`. The [matching GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.8)
contains the archives and verification material. To create a standalone app,
follow [Build your first Yetlet app](docs/FIRST-APP.md). The independently
published CLI `0.4.1` creates apps without a repository clone:

```sh
cargo +1.99.0 install yetlet-cli --version 0.4.1 --locked
yetlet --version
```

Expected output: `yetlet 0.4.1`. Cargo builds the CLI from source; see the
[native build prerequisites](docs/CLI-GUIDE.md#first-use-prerequisites),
including its OpenSSL dependency, before installing.

Use the pinned Rust/Cargo 1.99.0 toolchain. Native SSR needs no Node.js or WASM
toolchain. APIs may change; see [compatibility](docs/COMPATIBILITY.md).

From the repository root of the
[published alpha.8 source tag](https://github.com/bohewu/yetlet/tree/v0.9.0-alpha.8):

```sh
cargo run --locked -p yetlet --features server-axum --example server-axum-minimal
```

The development checkout uses the feature selections documented below; its
manifest version alone does not make it identical to this published source.

Open `http://127.0.0.1:7878/` for HTML and
`http://127.0.0.1:7878/api/status` for JSON. Stop the server with Ctrl+C.
The complete [published example](https://github.com/bohewu/yetlet/blob/v0.9.0-alpha.8/crates/yetlet/examples/server_axum/minimal.rs) is:

```rust
use yetlet::server_axum::prelude::*;

async fn home() -> View {
    View::text("Hello from Yetlet")
}
async fn status() -> Json<&'static str> {
    Json("ready")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .page("/", "Yetlet", home)
        .group("/api", |api| api.get("/status", status))
        .serve()
        .await?;
    Ok(())
}
```

Use `App::serve_on(address)` to choose a different bind address.
`App::into_router()` returns a native Axum Router for custom listeners or
graceful shutdown. For parameters or fallible handlers, use ordinary extractors
and `Page` or `Result<Page, E>`. See the [server guide](crates/yetlet/guides/server-axum.md).

## Development checkout

This source tree uses the next-alpha feature contract. It does not change the
published SDK or the Reader Edition above. Use this checkout's CLI when selecting
a feature-module SDK; the published CLI's exact compatibility limits and upgrade
or pin path are recorded in the [compatibility guide](docs/COMPATIBILITY.md).

`yetlet` has `default = []`. `HostGroup`, `Host` and `ResourceGroup` are built in;
Core adds no HTTP runtime, View layer, thread pool or process supervisor. SDK
publication remains the two packages `yetlet` and `yetlet-browser`.

| Application need | Explicit selection |
|---|---|
| Custom/local Host lifetimes | Core only, no feature |
| Shared HTML/DOM values | `view` |
| HTML serialization | `html` (enables `view`) |
| HTTP API | `web` |
| Server-rendered HTTP pages | `web` + `html` |
| One native event loop with multiple windows | `native-winit` |
| Bounded caller-driven UDP | `udp` |
| Bounded command/snapshot bridge | `command-bridge` |
| Independent native GPU renderer | `renderer-wgpu` |
| Browser DOM | `yetlet-browser` with `dom` or `dom-keyed` |

For this checkout's HTTP example:

```sh
cargo run --locked -p yetlet --no-default-features --features web,html --example server-axum-minimal
```

The existing example target name is retained; its source imports `yetlet::web`.
`App` configures one HTTP service. Convert it with `App::into_host()` when its
lifetime must be coordinated with other Hosts. Ordinary consumers compose
`HostGroup::new().with_host(...)`; advanced callers customize resources through
`ResourceGroup`. Constructors and `build()` do not acquire resources.

See the [current API map](docs/API-REFERENCE.md#source-only-checkout-additions),
[managed Web consumer](examples/external-consumers/web-hosts/README.md) and
[two-window/UDP/Web/custom-resource consumer](examples/external-consumers/native-hosts/README.md).
These examples describe source capabilities; GPU/platform and normal-archive
qualification require their own evidence.

## Choose your next step

| I want to… | Start here |
|---|---|
| Create my own standalone app | [Build your first Yetlet app](docs/FIRST-APP.md) |
| Create a browser-only WASM app | [Browser first-app walkthrough](docs/BROWSER-APP.md) |
| Write Views and reusable components | [View/component guide](docs/VIEW-COMPONENTS.md) |
| Use typed tags and optional Style authoring | [SDK/CLI API map](docs/API-REFERENCE.md#source-only-checkout-additions) · [Style Showcase](examples/style-showcase/README.md) (checkout sample) |
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
  errors, [how to read Rust API docs](docs/API-REFERENCE.md#reading-rust-api-documentation)
  and local rustdoc.
- [Troubleshooting](docs/TROUBLESHOOTING.md): symptoms, likely causes and linked fixes.
- [Opt-in quick reference](docs/OPT-IN-QUICK-REFERENCE.md): features, startup wiring,
  application settings and defaults.
- [HTTP pipeline](docs/HTTP-PIPELINE.md) and [opt-in composition](docs/EXTENSIONS.md).
- [CLI guide](docs/CLI-GUIDE.md): scaffolding, build/run/watch and selected
  configuration generators.
- [Documentation index](docs/README.md): further guides and retained history.

## Project and license

Planned work belongs in [Roadmap](docs/ROADMAP.md); tracked tasks belong in
[Backlog](docs/BACKLOG.md). Reporting and participation follow
[Contributing](CONTRIBUTING.md) and the [security policy](SECURITY.md).
Maintainers: [development/PR workflow](docs/DEVELOPMENT.md) and
[new-version release workflow](docs/RELEASING.md).

Licensed under either [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
