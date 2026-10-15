# SSR Session Cart Sample

This native-only application demonstrates an explicitly installed `yetlet-session` provider, server-rendered forms and bounded process-local cart state. It does not use JavaScript, WASM, a browser companion, Controllers or dependency injection.

## Run

From the repository root:

```bash
cargo run -p yetlet-cart-server
```

The default listener is `http://127.0.0.1:3002`; `YETLET_CART_LISTEN` may select another IPv4 or IPv6 **loopback** address only. The application rejects non-loopback listeners while the demo cookie is intentionally insecure.

Inspect:

- `http://127.0.0.1:3002/healthz` — outside the Session scope and does not create a cookie;
- `http://127.0.0.1:3002/cart` — strict-CSP SSR catalog and cart forms;
- `/cart/add`, `/cart/decrement`, `/cart/remove`, `/cart/clear`, `/cart/end` — POST-only form actions.

The provider defaults to `Secure` cookies. This direct localhost HTTP sample explicitly disables `Secure` so the cookie can round-trip without TLS, and its app constructor refuses every non-loopback listener. It still uses `HttpOnly`, `SameSite=Lax`, `Path=/`, an absolute 30-minute server-side expiry and a session-bound 256-bit CSRF token. Production deployment must retain `Secure` and define trusted-proxy/TLS policy.

Unknown session IDs are never adopted, expired records are replaced, clear rotates both session ID and CSRF token, and end removes server state and expires the cookie. The in-memory store is capped at 1,024 sessions. Each cart is limited to three distinct lines and quantity 20 per product.

## Verify

```bash
cargo test --locked -p yetlet-session -p yetlet-cart-server --all-targets
cargo build --locked -p yetlet-cart-server
cargo xtask smoke cart
```

This is a bounded anonymous-session example, not authentication, durable storage, multi-node session distribution or production security qualification. Retained evidence is in [M8c verification](../../docs/M8C-VERIFICATION.md).
