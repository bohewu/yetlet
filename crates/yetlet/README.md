# Yetlet

A small, composable Rust application framework. This guide covers version
`1.0.0-rc.1`. See [GitHub releases](https://github.com/bohewu/yetlet/releases)
for published artifacts.

Core provides application lifetimes. Select View, HTML, Web, native windows, UDP,
rendering or auxiliary services through Cargo features.

## Minimal integration

Use Rust 1.99+ and this exact registry dependency for HTML rendering:

```toml
[dependencies]
yetlet = { version = "=1.0.0-rc.1", default-features = false, features = ["html"] }
```

For the current checkout, enable the same HTML capability explicitly:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=1.0.0-rc.1", default-features = false, features = ["html"] }
```

```rust
use yetlet::{Element, View};
let view: View = Element::new("p").child(View::text("Hello <Yetlet>")).into();
assert_eq!(yetlet::html::render_to_string(&view).unwrap(), "<p>Hello &lt;Yetlet&gt;</p>");
```

SDK alpha.8 enabled HTML by default and exposed View unconditionally. This
version has `default = []` with built-in Core composition, `HostGroup`, `Host`
and `ResourceGroup`. Select `view` for the optional View/Element/Attribute, tags
and binding APIs, or `html` for serialization; `html` enables `view`. Core-only
omits View, and View/HTML add no external normal dependencies or HTTP runtime.
Browser integrations are separately owned by
[yetlet-browser](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/README.md), whose DOM features enable the same View type.

## Core and managed Hosts

For Core lifetimes alone, select the package without optional features:

```toml
[dependencies]
yetlet = { version = "=1.0.0-rc.1", default-features = false }
```

`HostGroup` coordinates startup, observed readiness, admission, original
completion and final resource release. Ordinary code adds Hosts with
`HostGroup::new().with_host(...)`; `ResourceGroup` customizes owned or borrowed
resources. No default runtime, pool, process or universal state owner is created.

Enable `web` for an HTTP `App` or `WebHost`; add `html` for page helpers.
`App::into_host()` joins the shared lifecycle. Enable `native-winit` for one
root event loop with multiple windows, `udp` for bounded caller-driven datagrams,
`command-bridge` for bounded commands/snapshots, or `renderer-wgpu` independently
for native GPU rendering. Feature selection exposes APIs; the application must
still select a compatible driver and explicitly drive managed shutdown.

See the [mixed Host consumer](https://github.com/bohewu/yetlet/blob/master/examples/external-consumers/native-hosts/README.md)
and [lifecycle contract](https://github.com/bohewu/yetlet/blob/master/docs/M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md).

## Typed Tag Builder API

The `view` feature includes `yetlet::view::tags` with 101 ordinary HTML constructors
and `TagElement<T>` builders, producing the same owned `Element` / `View` tree.
Typed void tags have no child/text methods; selected setters are scoped to
applicable tags. `.id` and `.class` replace, `.add_class` merges distinct tokens,
and dedicated attribute setters upsert; generic `.attribute` remains append-only.
Renderer validation still applies. No inline-style API or template compiler is added.

```rust
use yetlet::view::{View, tags::{div, h1, input}};
let view: View = div().id("app").child(h1().text("Hello <Yetlet>"))
    .child(input().placeholder("Search")).into();
assert!(yetlet::html::render_to_string(&view).unwrap().contains("Hello &lt;Yetlet&gt;"));
```

## Native application startup

Select `web` and import `yetlet::web::prelude::*` for native
HTTP service configuration. API-only Web does not enable View or HTML; add `html`
for `Page`, page helpers and View-related prelude imports. `App::serve()` listens
for plain HTTP on `127.0.0.1:7878`; `App::serve_on(address)` takes an explicit bind
address. The application owns its Tokio runtime and can use `App::into_router()`
for native Axum listener and lifecycle control.

Published alpha.8 keeps the historical `server-axum` feature and
`yetlet::server_axum` imports. Those names describe the published artifact;
the current checkout uses `web` without an old-path compatibility alias.

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Core lifetimes | Built in, no feature opt-in | [Lifecycle design](https://github.com/bohewu/yetlet/blob/master/docs/M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) |
| Shared HTML/DOM values | `view` | [View](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/core.md) |
| HTML serializer | `html` (enables `view`) | [HTML](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/html.md) |
| Neutral commands and caller-selected executor errors | `actions` (no serde) | [Actions](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/actions.md) |
| Portable HTTP action errors/envelopes | `actions-http` (enables `actions` and serde) | [Actions](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/actions.md) |
| Local reactive graph | `reactive` | [Reactive](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/reactive.md) |
| Native HTTP/Router/Tower integration | `web`; add `html` for SSR | [Server](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/server-axum.md) |
| Native windows and root event loop | `native-winit` | [Mixed Hosts](https://github.com/bohewu/yetlet/blob/master/examples/external-consumers/native-hosts/README.md) |
| Caller-driven datagrams | `udp` | [Mixed Hosts](https://github.com/bohewu/yetlet/blob/master/examples/external-consumers/native-hosts/README.md) |
| Bounded commands and snapshots | `command-bridge` | [Lifecycle](https://github.com/bohewu/yetlet/blob/master/docs/M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) |
| Independent native GPU renderer | `renderer-wgpu` | [Renderer](https://github.com/bohewu/yetlet/blob/master/docs/M28-F2-WGPU-RENDERER.md) |
| Request correlation | `request-id` | [Request ID](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/request-id.md) |
| Memory Session | `session` | [Session](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/session.md) |
| Authentication / Memory bridge | `auth` / `auth-session` | [Auth](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/auth.md) |
| Async Redis/Valkey protocol Session / TLS | `session-redis` / `session-redis-tls` | [Provider](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/session-redis.md) |
| Typed settings / provider injection | `config` / `secret-provider` | [Config](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/config.md) |
| Secret lookup | `secrets` | [Secrets](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/secrets.md) |
| Application signing | `signing` | [Signing](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/signing.md) |
| Per-process rate limiting | `rate-limit` | [Rate limiting](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/rate-limit.md) |

Feature selection compiles an API; startup registration and typed application
options decide activation. Settings loading remains optional. Native modules
and their networking/storage/crypto dependencies are absent on wasm32, even with
all features selected; shared serialization may still compile. No provider,
subscriber, cache, DI container or automatic retry is installed by importing it.

APIs remain Experimental; this release does not establish production qualification
or stable API compatibility. Ordinary CSS is application-owned; there is no public
Style crate or required CSS generator.
