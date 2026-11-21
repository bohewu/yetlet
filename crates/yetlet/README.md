# Yetlet

A small, composable Rust application framework with built-in Core lifetimes
and explicitly selected View, HTML and integration capabilities in this checkout.
Published SDK `0.9.0-alpha.8` retains its original feature contract; see
[GitHub releases](https://github.com/bohewu/yetlet/releases) for published artifacts.
Earlier tags retain their package layout and qualification evidence. The source
feature changes below do not change the contents of a published artifact.

## Minimal integration

Use Rust 1.99+ and this exact registry dependency for HTML rendering:

```toml
[dependencies]
yetlet = { version = "=0.9.0-alpha.8", default-features = false, features = ["html"] }
```

For the current checkout, enable the same HTML capability explicitly:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.8", default-features = false, features = ["html"] }
```

```rust
use yetlet::{Element, View};
let view: View = Element::new("p").child(View::text("Hello <Yetlet>")).into();
assert_eq!(yetlet::html::render_to_string(&view).unwrap(), "<p>Hello &lt;Yetlet&gt;</p>");
```

The explicit `html` selection works with both published alpha.8 and this checkout.
Published alpha.8 retains its HTML default and always-available View API. This
checkout has `default = []` with built-in Core composition, `HostGroup`, `Host`
and `ResourceGroup`. Select `view` for the optional View/Element/Attribute, tags
and binding APIs, or `html` for serialization; `html` enables `view`. Core-only
omits View, and View/HTML add no external normal dependencies or HTTP runtime.
Browser integrations are separately owned by
[yetlet-browser](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/README.md), whose DOM features enable the same View type.

## Typed Tag Builder API

Published alpha.8 and this checkout include `yetlet::view::tags` with 101 ordinary HTML constructors
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

This release includes simple startup with the opt-in `server-axum` feature.
`App::serve()` listens for plain HTTP on `127.0.0.1:7878`.
`App::serve_on(address)` takes an explicit bind address. The application owns
its Tokio runtime and can use `App::into_router()` for native Axum listener and
lifecycle control. Import `yetlet::server_axum::prelude::*` to opt into the
curated server imports.

## Current-checkout features and guides

| Capability | Feature | Guide |
|---|---|---|
| Core lifetimes | Built in, no feature opt-in | [Lifecycle design](https://github.com/bohewu/yetlet/blob/master/docs/M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) |
| Shared HTML/DOM values | `view` | [View](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/core.md) |
| HTML serializer | `html` (enables `view`) | [HTML](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/html.md) |
| Portable command contracts | `actions` | [Actions](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/actions.md) |
| Local reactive graph | `reactive` | [Reactive](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/reactive.md) |
| Native Axum integration | `server-axum` | [Server](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/server-axum.md) |
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
