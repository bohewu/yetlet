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
python3 scripts/smoke_middleware_lab.py
cargo tree --locked -p yetlet-middleware-lab
python3 scripts/check_samples.py
```

This is a bounded anonymous local HTTP demonstration. It does not claim production instrumentation, distributed tracing, TLS/reverse-proxy trust, a separate middleware engine or zero runtime overhead. The streaming error route deliberately demonstrates an HTTP body failure, not a successful error recovery. See [M9b verification](../../docs/M9B-VERIFICATION.md).
