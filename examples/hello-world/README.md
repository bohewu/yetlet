# Hello World — Native SSR

This is the smallest independently runnable Yetlet application: two ordinary Rust handlers, the minimal `App` builder, `View` and `Page`. It uses Axum's server directly and needs **no JavaScript, WASM, browser build, Session, Request ID, ORM or database**.

## Run

From the repository root:

```bash
cargo run -p yetlet-hello-world
```

The default listener is `127.0.0.1:3003`; override with `YETLET_HELLO_LISTEN` for local testing. Routes:

- `/` — fixed native SSR "Hello, world!" page.
- `/hello/{name}` — dynamic untrusted route text escaped by the shared HTML renderer. Try `http://127.0.0.1:3003/hello/Yetlet`.

Both routes return complete native SSR with strict CSP and no browser runtime. Unknown routes return native 404; unsupported methods return native 405. The sample is intentionally anonymous and has no persistence, authorization or production deployment hardening.

## View authoring sample (unreleased source)

This sample uses `yetlet::view::tags::{main, h1}` to construct the same
renderer-neutral `View` tree as the original `Element::new(...)` API.
For example, the `/` route constructs `main().child(h1().text("Hello, world!"))`,
while `/hello/{name}` uses `text(format!(...))` and the existing HTML
renderer escapes untrusted path input. These are Rust helpers, not JSX, CSS
generation or browser/WASM requirements.

The typed tag helpers are **source-only in this feature branch**, not part of
the published SDK `0.9.0-alpha.6`. Build this sample against the repository
path dependency shown in its `Cargo.toml`; do not copy it unchanged into an
alpha.6 registry-only application. The [View guide](../../docs/VIEW-COMPONENTS.md)
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
