# yetlet-session

Add bounded, process-local Memory Session state, cookie lifecycle and explicit
CSRF checks to a native application. Session can be anonymous and is not authentication.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use this checkout with Rust/Cargo 1.99.0. The runnable cart selects native Session
explicitly and requires no JS/WASM or remote store. Run from the repository root;
allow locked dependency downloads unless the Cargo cache is already prepared.

## Minimal integration

### Use the native Session cart

From the repository root:

```bash
cargo run --locked -p yetlet-cart-server
```

Open `http://127.0.0.1:3002/cart`; forms work without JS/WASM. The [complete cart implementation](../../examples/cart-server/src/lib.rs) and [sample guide](../../examples/cart-server/README.md) demonstrate configuration, scoped registration, reads, fallible updates, CSRF validation, rotation and destruction. The demo explicitly permits insecure cookies only on loopback HTTP.

For your own app, validate `SessionConfig::new(cookie_name, ttl, max_sessions)`, construct `SessionManager::<T>::new(config)`, then apply `manager.scoped(app)` after registering the intended routes. Handlers extract `Extension<Session<T>>` or the typed request extension. `T` requires `Default + Clone + Send + 'static`, with no serde bound. Cloned managers share their store; independent managers do not. Session handles share request-local identity/cookie directives and must stay within the owning request's application lifetime.

## Expected result

At `http://127.0.0.1:3002/cart`, native forms update the selected Session without
browser code. The demo deliberately selects loopback-only insecure HTTP cookies.
Independent managers/processes do not share this Memory store; restart loses it.

## Configuration and errors

### Cookies, mutations and failures

Cookie defaults are Secure, HttpOnly, SameSite=Lax and Path=/, with no Domain. `insecure_for_local_http()` is an explicit demo override; production apps retain Secure and select their TLS/proxy policy. Missing/unknown/expired identities create a new default Session within capacity rather than adopting a caller-supplied ID. Expiry is absolute rather than sliding.

`read()` clones state. `try_update` mutates a clone and commits only on `Ok`; it distinguishes Session failures from application rejection. Callbacks run synchronously under store ownership: keep them short and non-blocking, and do not assume external effects roll back. `rotate` and `replace_and_rotate` retire the identity and rotate CSRF; `destroy` removes server state and expires the cookie. `csrf_token` and constant-time `verify_csrf` support explicit application enforcement; they do not install authorization.

Invalid configuration returns `SessionConfigError`; TTL is 1 second–30 days and capacity is 1–1,000,000. Session operations can fail with `Unavailable`, `Expired` or `Destroyed`. Middleware rejects malformed cookies with 400 and capacity/entropy/store failures with 503; handlers choose how to map their own operation/application failures.

### Deployment boundary

Session is not authentication or durable storage. Restart loses Memory state; multiple independent processes do not share it. Logout/cookie response reordering requires application-aware handling, and CSRF tokens must be checked on unsafe actions. Avoid serializing identities, tokens or server capabilities into browser state.

The [separate experimental Redis-protocol provider](../yetlet-session-redis/README.md)
has publication-prepared metadata and is included in the selected sixteen-package
candidate, but remains unpublished. Its async, single-primary semantics and
separately recorded Valkey evidence do not broaden Memory guarantees or imply
generic HA/failover/restore support. See [provider operations](../../docs/PROVIDER-OPERATIONS.md)
and the provider guide for its current limits; [M8c evidence](../../docs/M8C-VERIFICATION.md)
retains the earlier Memory scope.

## API reference

Use [Session source/rustdoc](src/lib.rs), the complete cart linked above and
the [current API reference](../../docs/API-REFERENCE.md).

### License and compatibility

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../docs/COMPATIBILITY.md), [API classifications](../../docs/API-CONTRACTS.md) and [developer guides](../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.

<details>
<summary>Historical delivery context and evidence</summary>

**Experimental `0.9.0-alpha.4`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet-session` is an opt-in, synchronous process-local Memory Session provider. It owns random opaque identities, bounded in-memory values, absolute expiry, cookie lifecycle and session-bound CSRF tokens. It depends on Axum, cookie, getrandom, subtle and `yetlet-server-axum`; no browser runtime or Redis dependency is added.

</details>
