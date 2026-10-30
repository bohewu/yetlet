# Yetlet

HTML-first views and explicitly selected native capabilities. This checkout uses
unreleased `0.9.0-alpha.5`; it is not yet available from crates.io. Earlier release
tags retain their sixteen-package layout and original qualification evidence.

## Minimal integration

Use Rust 1.99+ and an explicit path to this checkout:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.5" }
```

```rust
use yetlet::{Element, View};
let view: View = Element::new("p").child(View::text("Hello <Yetlet>")).into();
assert_eq!(yetlet::html::render_to_string(&view).unwrap(), "<p>Hello &lt;Yetlet&gt;</p>");
```

Defaults select `html`. With `default-features = false`, the shared View model
has no external normal dependencies. Browser integrations are separately owned
by [yetlet-browser](../yetlet-browser/README.md), using this same View type.

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Shared values | Always available | [View](guides/core.md) |
| HTML serializer | `html` (default) | [HTML](guides/html.md) |
| Portable command contracts | `actions` | [Actions](guides/actions.md) |
| Local reactive graph | `reactive` | [Reactive](guides/reactive.md) |
| Native Axum integration | `server-axum` | [Server](guides/server-axum.md) |
| Request correlation | `request-id` | [Request ID](guides/request-id.md) |
| Memory Session | `session` | [Session](guides/session.md) |
| Authentication / Memory bridge | `auth` / `auth-session` | [Auth](guides/auth.md) |
| Async Redis/Valkey protocol Session / TLS | `session-redis` / `session-redis-tls` | [Provider](guides/session-redis.md) |
| Typed settings / provider injection | `config` / `secret-provider` | [Config](guides/config.md) |
| Secret lookup | `secrets` | [Secrets](guides/secrets.md) |
| Application signing | `signing` | [Signing](guides/signing.md) |
| Per-process rate limiting | `rate-limit` | [Rate limiting](guides/rate-limit.md) |

Feature selection compiles an API; startup registration and typed application
options decide activation. Settings loading remains optional. Native modules
and their networking/storage/crypto dependencies are absent on wasm32, even with
all features selected; shared serialization may still compile. No provider,
subscriber, cache, DI container or automatic retry is installed by importing it.
