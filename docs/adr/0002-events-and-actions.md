# ADR 0002 — Local DOM events and typed actions across boundaries

- **Status:** M4 typed LocalExecutor/HttpExecutor prototype validated locally; custom transport and production security remain proposed
- **Date:** 2026-10-08
- **Related:** [Architecture](../ARCHITECTURE.md), [Backlog](../BACKLOG.md)

## Context

Serializing raw DOM click, input, and keyboard events to a server makes normal interaction network-dependent and introduces circuit state, event serialization contracts, and reconnect protocols. Yetlet does not use remote-event replication as its default interaction model.

## Proposal

1. A DOM event listener belongs to the browser component/local runtime.
2. Prefer to complete local UI state transitions in a browser-local executor.
3. When server data or mutation is necessary, issue an **explicit, validated, serializable, typed semantic command/action**, not a DOM event.
4. An `ActionExecutor` selects local or HTTP execution. Custom transports will require explicit opt-in.
5. Ordinary `<form method="post">` must remain usable without browser runtime; progressive enhancement must preserve native HTML semantics.

The M4 prototype implements this interface with browser-local, intentionally non-`Send` futures. **Purely local commands require no serialization**; only the HTTP executor requires serde input/output bounds:

```rust
trait Command {
    type Response;
}
trait ActionExecutor<C: Command> {
    async fn execute(&self, command: C) -> Result<C::Response, ActionError>;
}
```

The first implementation contains both `LocalExecutor` and a browser-only `HttpExecutor`. They share typed Todo commands and do **not** automatically serialize browser events to execute their listeners remotely. See [M4 verification](../M4-VERIFICATION.md).

## Security requirements

- Re-validate authentication, authorization, and inputs **on the server** for every remote action; browser state is untrusted.
- Cookie-based mutations require deliberate CSRF prevention and a SameSite policy. A typed API is not inherently secure.
- Define idempotency for retryable commands; do not automatically retry non-idempotent actions.
- Avoid leaking private fields in serialized state, logging, or HTML hydration metadata.
- Specify timeout, network, validation, server, and user-facing error behavior.

## Trade-offs

Developers must explicitly manage their service/action boundary rather than relying on arbitrary compiler-generated magic RPC. This costs some boilerplate in exchange for better visibility, HTTP semantics, testability, and security review.

## Verification

M4 locally verified both executor paths, typed successful/error HTTP responses, origin/content-type/body limits, and absence of a remote event circuit. This is an anonymous demonstration, not production-grade authorization or session CSRF protection.

- Browser-local clicks do not trigger remote requests.
- Remote mutations send only semantic input, never a `MouseEvent` or other raw DOM event.
- API validation failures, timeouts, and 5xx responses must not silently count as successful UI updates.
- No framework-specific persistent socket is established unless a future custom executor is explicitly installed.
