# yetlet::actions

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet::actions` defines typed semantic commands and explicitly selected executors. It does not forward DOM events or create a persistent remote event circuit. The portable contracts select serde. Browser HTTP execution is separately selected by `yetlet-browser` feature `actions`; import `yetlet_browser::actions::HttpExecutor`.

## Choose an executor

Implement `Command` with an associated `Response`, then select an `ActionExecutor<C>`. `LocalExecutor::new(handler)` accepts a synchronous `Fn(C) -> Result<C::Response, ActionError>` and exposes it through async `execute`. Local commands and results need no serialization; the caller owns the handler's captured state. Futures intentionally have no `Send` requirement.

`HttpExecutor::new(path)` validates an explicit same-origin absolute path. Construction and `endpoint()` are available natively, but HTTP execution is implemented only on WASM. That implementation requires serde input and a deserializable response, sends a JSON POST, and decodes the result. The [shared Todo commands](../../../examples/todo-ui/src/lib.rs) and [browser dispatch caller](../../../examples/todo-browser/src/lib.rs) demonstrate both paths. Follow [Todo setup](../../../examples/todo-server/README.md), or run from the repository root with its browser prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/app`; local toggle and remote Create have separate owners and execution boundaries.

## Errors and application responsibilities

Invalid endpoints fail with `ActionError::InvalidEndpoint`. Query/fragment syntax, protocol-relative URLs, backslashes and ASCII controls are rejected. Fetch/decoding failures report `Network` or `InvalidResponse`; non-2xx results report `Remote { status, code }`, with the typed error envelope used when decodable. Local/domain errors retain their explicit variants.

No automatic retry, application timeout, cancellation protocol, authentication or CSRF token attachment is provided. The server must validate inputs, authentication, authorization and CSRF policy; serialization alone provides none of those guarantees. A retired UI lifetime must suppress late results explicitly, as the Todo caller does.

[Custom transport research](../../../docs/YT-024-CUSTOM-TRANSPORT-RESEARCH.md) proves one app-owned typed TCP adapter over this existing trait. It does not select a supported transport/plugin framework or change the default browser HTTP path.

## License and compatibility

Dual-licensed under [MIT](../../../LICENSE-MIT) or [Apache-2.0](../../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../../docs/COMPATIBILITY.md), [API classifications](../../../docs/API-CONTRACTS.md) and [developer guides](../../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
