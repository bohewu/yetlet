# Hello World — Native SSR

This is the smallest independently runnable Yetlet application: two ordinary Rust handlers, the minimal `App` builder, `View` and `Page`. Its entry point wraps the existing Router with `App::from_router(...).serve_on(address)` and needs **no JavaScript, WASM, browser build, Session, Request ID, ORM or database**.

## Run

From the repository root:

```bash
cargo run -p yetlet-hello-world
```

The default listener is `127.0.0.1:3003`; override with `YETLET_HELLO_LISTEN` for local testing. Routes:

- `/` — fixed native SSR "Hello, world!" page.
- `/hello/{name}` — dynamic untrusted route text escaped by the shared HTML renderer. Try `http://127.0.0.1:3003/hello/Yetlet`.

Both routes return complete native SSR with strict CSP and no browser runtime. Unknown routes return native 404; unsupported methods return native 405. The sample is intentionally anonymous and has no persistence, authorization or production deployment hardening.

The entry point uses the optional native Prelude from this checkout. Explicit
imports remain supported. Startup methods and Prelude are source-only additions
after published SDK alpha.7. Port `0` keeps a native `TcpListener` so the banner
reports the actual bound address. The non-catalog SDK minimal example demonstrates
`App::new().page(...).serve()` at `127.0.0.1:7878`; this sample retains port `3003`.

## View authoring sample (typed tags in published SDK alpha.7)

This sample uses `yetlet::view::tags::{main, h1}` to construct the same
renderer-neutral `View` tree as the original `Element::new(...)` API.
For example, the `/` route constructs `main().child(h1().text("Hello, world!"))`,
while `/hello/{name}` uses `text(format!(...))` and the existing HTML
renderer escapes untrusted path input. These are Rust helpers, not JSX, CSS
generation or browser/WASM requirements.

The typed tag helpers are **included in the published SDK `0.9.0-alpha.7`**.
This repository sample still builds against the existing path dependency in
its `Cargo.toml`; a registry-only application using SDK alpha.6 must not copy
the typed constructor calls unchanged. Registry consumers choosing SDK alpha.7
can use the same typed Tag Builder. The [View guide](../../docs/VIEW-COMPONENTS.md)
shows class helpers and broader tag coverage.

## Verify

```bash
cargo test --locked -p yetlet-hello-world --all-targets
cargo build --locked -p yetlet-hello-world
cargo xtask smoke hello-world
cargo tree --locked -p yetlet-hello-world
cargo xtask check samples
```

See [M9a verification](../../docs/M9A-VERIFICATION.md) for reproducible evidence and independent reviewer outcome.
