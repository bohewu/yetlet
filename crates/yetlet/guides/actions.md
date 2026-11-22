# yetlet::actions

**Experimental SDK modules.** Requires Rust 1.99 or newer. This guide describes
the current checkout's neutral `actions` and optional `actions-http` capabilities.
Published SDK `0.9.0-alpha.8` retains its original contract: its `actions` feature
selects serde, HTTP types live in `yetlet::actions`, and executors have a fixed
`ActionError`. The source migration below does not change that registry artifact.
Use source and rustdoc at the revision you consume.

`yetlet::actions` defines typed semantic commands and explicitly selected
executors. It does not forward DOM events, create a persistent remote event
circuit or choose an application's transport.

## Select the capability

| Current-checkout selection | API and dependency boundary |
|---|---|
| `yetlet` feature `actions` | `Command`, `ActionExecutor` and `LocalExecutor`; no serde, View, Web or runtime dependency |
| `yetlet` feature `actions-http` | Enables `actions` and serde; adds target-neutral `actions_http::ActionError`, `ApiErrorBody` and `ApiErrorEnvelope` without a server or Fetch implementation |
| `yetlet-browser` feature `actions` | Enables `yetlet/actions-http` and target-specific browser serialization/Fetch dependencies; import `yetlet_browser::actions::HttpExecutor` |

Disable default features on the Browser dependency when selecting only action
transport. Keeping its DOM default also selects View. The new `actions-http`
feature belongs to this checkout; do not request it from a published alpha.8
registry dependency.

## Choose an executor

Implement `Command` with an associated `Response`, then select an
`ActionExecutor<C>`. Each executor declares its own associated `Error` and
returns `Result<C::Response, Self::Error>` from `execute`. The contract imposes
no serialization, `Send` or `std::error::Error` bound on that error type.

`LocalExecutor::new(handler)` accepts a synchronous
`Fn(C) -> Result<C::Response, E>` and exposes it through async `execute`, with
`Error = E`. Local commands and responses need no serialization. The caller owns
the captured state, including local `Rc` values; futures have no `Send`
requirement. A custom executor can implement asynchronous work under the same
associated-error contract.

A handler that only returns `Ok` may need an explicit error annotation:

```rust
use std::convert::Infallible;
use yetlet::actions::{Command, LocalExecutor};

struct Echo(String);
impl Command for Echo {
    type Response = String;
}

let executor = LocalExecutor::new(|command: Echo| {
    Ok::<_, Infallible>(command.0)
});
```

A handler with fallible domain operations can return its own error directly.
The [shared Todo domain](https://github.com/bohewu/yetlet/blob/master/examples/todo-ui/src/lib.rs)
uses `TodoError`; its native HTTP adapter explicitly maps that error to HTTP
status and the shared envelope. Its browser-local graph uses a local error type
for domain and rendering failures, while remote requests keep HTTP errors.

## Browser HTTP execution

`HttpExecutor::new(path)` validates an explicit same-origin absolute path.
Construction and `endpoint()` are available natively, but execution is
implemented only on wasm32. The executor's associated `Error` is
`yetlet::actions_http::ActionError`. Execution requires serde input and a
deserializable response, sends a JSON POST, and decodes the result.

The [browser dispatch caller](https://github.com/bohewu/yetlet/blob/master/examples/todo-browser/src/lib.rs)
demonstrates local and HTTP execution. Follow
[Todo setup](https://github.com/bohewu/yetlet/blob/master/examples/todo-server/README.md),
or run from the repository root with its browser prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/app`; local toggle and remote Create have separate
owners and execution boundaries.

## Errors and application responsibilities

Invalid HTTP endpoints fail with `ActionError::InvalidEndpoint`. Query/fragment
syntax, protocol-relative URLs, backslashes and ASCII controls are rejected.
Fetch/decoding failures report `Network` or `InvalidResponse`; non-2xx results
report `Remote { status, code }`, using the typed error envelope when decodable
and `remote_error` otherwise. These transport policies are unchanged.

The shared wire shape remains:

```json
{"error":{"code":"invalid_title","message":"Invalid command fields"}}
```

An HTTP adapter owns domain-to-status/code/message mapping; selecting
`actions-http` does not install a blanket conversion or an HTTP route.
The application-owned [custom TCP transport](https://github.com/bohewu/yetlet/blob/master/examples/custom-action-transport/src/lib.rs)
implements `ActionExecutor` with `TcpError`, including a remote application
code without an HTTP status. Its command/reply framing remains application-owned.
This experiment does not select a supported transport/plugin framework or change
the default browser HTTP path.

No automatic retry, application timeout, cancellation protocol, authentication
or CSRF token attachment is provided. The server must validate inputs,
authentication, authorization and CSRF policy; serialization alone provides
none of those guarantees. A retired UI lifetime must suppress late results
explicitly, as the Todo caller does.

## License and compatibility

Dual-licensed under [MIT](https://github.com/bohewu/yetlet/blob/master/LICENSE-MIT)
or [Apache-2.0](https://github.com/bohewu/yetlet/blob/master/LICENSE-APACHE). APIs
remain Experimental; publication does not establish production qualification or
API stability. See [compatibility](https://github.com/bohewu/yetlet/blob/master/docs/COMPATIBILITY.md),
[API classifications](https://github.com/bohewu/yetlet/blob/master/docs/API-CONTRACTS.md)
and [developer guides](https://github.com/bohewu/yetlet/blob/master/docs/README.md).
Repository links follow development and do not imply a new registry publication.
