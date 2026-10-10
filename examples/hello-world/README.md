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

## Verify

```bash
cargo test --locked -p yetlet-hello-world --all-targets
cargo build --locked -p yetlet-hello-world
python3 scripts/smoke_hello_world.py
cargo tree --locked -p yetlet-hello-world
python3 scripts/check_samples.py
```

See [M9a verification](../../docs/M9A-VERIFICATION.md) for reproducible evidence and independent reviewer outcome.
