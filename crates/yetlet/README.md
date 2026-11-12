# Yetlet

HTML-first views and explicitly selected native capabilities, version
`0.9.0-alpha.8`. See [GitHub releases](https://github.com/bohewu/yetlet/releases)
for published artifacts. Earlier tags retain their original package layout and
qualification evidence.

## Minimal integration

Use Rust 1.99+ and this exact registry dependency once this version is available:

```toml
[dependencies]
yetlet = "=0.9.0-alpha.8"
```

For local preparation, use an explicitly reviewed checkout at the matching version:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.8" }
```

```rust
use yetlet::{Element, View};
let view: View = Element::new("p").child(View::text("Hello <Yetlet>")).into();
assert_eq!(yetlet::html::render_to_string(&view).unwrap(), "<p>Hello &lt;Yetlet&gt;</p>");
```

Defaults select `html`. With `default-features = false`, the shared View model
has no external normal dependencies. Browser integrations are separately owned
by [yetlet-browser](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/README.md), using this same View type.

## Typed Tag Builder API

This release includes `yetlet::view::tags` with 101 ordinary HTML constructors
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

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Shared values | Always available | [View](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/core.md) |
| HTML serializer | `html` (default) | [HTML](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/html.md) |
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
