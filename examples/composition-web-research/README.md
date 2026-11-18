# F0-B2 — Private Pure Web Composition Consumer

This is an independent unpublished, opt-in Cargo workspace using the
existing Yetlet `server-axum` feature and native Axum/Tower. It does not
modify the root workspace, Yetlet SDK public API, CLI templates, default
features or direct `App::serve()` path.

The concrete composition root is an ordinary `configure_router(SiteState)`
function. It uses the already supported `App::<SiteState>::new()`,
`try_configure`, `with_state::<()>` and `into_router()`, installs one
SSR Page at `/` and one JSON API route at `/api/status`. These routes
are **not** separate Hosts. They access ordinary native Axum state with no
Actor, fixed tick, mailbox, game StateOwner, worker or shared Host trait.

The evidence intentionally distinguishes:

1. **Router configured:** fallible setup succeeded; no socket exists.
2. **Listener bound:** the caller explicitly binds a loopback
   `TcpListener`, including port 0; this is not serving readiness.
3. **Serving observed:** a real localhost HTTP response was received
   while the caller's Tokio runtime polls the native Axum serve future.
4. **Drain complete:** the caller signals native graceful shutdown,
   retains the exact serving task across an intentionally timed-out
   observation while an in-flight API request is held, releases the
   request, then awaits the task's actual completion and rebinds the
   original port. The caller-owned runtime remains usable afterward.

The in-flight request gate is compiled only into tests; normal handler
paths have no gate. The code deliberately uses native
`axum::serve(...).with_graceful_shutdown(...)`, not a new Yetlet
serve method or duplicated HTTP engine.

## Focused verification

From repository root after the private lockfile is generated using
Cargo's local dependency cache:

```sh
cargo test --manifest-path examples/composition-web-research/Cargo.toml --locked --offline
cargo clippy --manifest-path examples/composition-web-research/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo fmt --manifest-path examples/composition-web-research/Cargo.toml --check
```

Only the new private Web consumer is selected. Earlier B0–D3/F0-B1
acceptance is reused, not replayed, because public Yetlet Web/API and
feature definitions remain unchanged.

This does **not** qualify Windows/MSVC, external LAN, TLS, global
server readiness, a generic Host lifecycle, detached handler tasks,
or real-world throughput. ADR 0013 remains Proposed. Original physical
D4 LAN remains Not verified, distinct from the earlier VM relay proof.
