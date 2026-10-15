# Middleware Lab — RequestContext and Native Tower Pipeline

This independently runnable HTTP lab demonstrates Yetlet's thin `App`, request-owning `RequestContext`, Axum `Next`, group/MethodRouter layers and a real third-party Tower HTTP response-header layer. It adds no new Yetlet middleware system, Session, authentication, DB layer or browser runtime.

## Run

From the repository root:

```bash
cargo run -p yetlet-middleware-lab
```

The default listener is `127.0.0.1:3004`; `YETLET_MW_LISTEN` overrides it for local testing. Routes:

- `/outside`: global middleware only; shows application state without group/route instrumentation.
- `/lab/plain`: global plus group middleware, request-local `x-lab-user` label and shared application state.
- `/lab/guarded`: global plus group and method-route middleware; send header `x-stop: 1` to short-circuit with HTTP 403 before the handler.
- `/lab/stream`: chunked `first|second` body, returned without middleware buffering.
- `/lab/broken-stream`: demonstrative mid-body error, intentionally not recoverable as another HTTP response after headers are sent.

Each response includes `x-mw-order` recording before/after unwinding and `x-native-tower: tower-http`. The global layer observes native 404s, while a recognized route with the wrong method yields native 405/Allow. Group/route instrumentation cannot leak to siblings. `RequestContext` extracts typed state/headers/extensions without eagerly reading the request body.

## Verify

```bash
cargo test --locked -p yetlet-middleware-lab --all-targets
cargo build --locked -p yetlet-middleware-lab
cargo xtask smoke middleware-lab
cargo tree --locked -p yetlet-middleware-lab
cargo xtask check samples
```

This is a bounded anonymous local HTTP demonstration. It does not claim production instrumentation, distributed tracing, TLS/reverse-proxy trust, a separate middleware engine or zero runtime overhead. The streaming error route deliberately demonstrates an HTTP body failure, not a successful error recovery. See [M9b verification](../../docs/M9B-VERIFICATION.md).

## Opt-in deadline research — YT-029

The normal listener and `app()` retain their existing behavior. The separate example-owned `deadline::with_deadline` composes native Tower `TimeoutLayer` inside Axum `HandleErrorLayer`, using Yetlet's existing global envelope. Explicit installation after route registration is required:

```rust
let router = yetlet_middleware_lab::deadline::with_deadline(
    yetlet_middleware_lab::app(),
    std::time::Duration::from_millis(100),
);
```

The example maps elapsed service execution to HTTP 504. Its cooperative timeout drops the pending service future; earlier completed effects are not rolled back. Detached tasks, started blocking work and externally dispatched operations need their own cancellation/transaction policy. The timer ends when a Response is returned: later body streaming has no whole-body deadline. Non-yielding work can delay or defeat timely timeout observation.

Run the bounded tests and in-process release comparison without a listener:

```bash
cargo test --locked --offline -p yetlet-middleware-lab --all-targets
cargo build --release --locked --offline -p yetlet-middleware-lab --bins
target/release/deadline-bench
```

The comparison uses the same empty GET and two-byte response for a plain Router, global envelope, no-op RequestContext middleware and explicitly installed deadline. It prints ten raw batch samples after warmup; it does not measure network latency or production load. See [YT-029 research and adoption decision](../../docs/YT-029-HTTP-DEADLINE-RESEARCH.md) for measurements, cancellation/body limits, source identity and validation. The helper is an application-owned research example, not a supported core deadline/provider API.
