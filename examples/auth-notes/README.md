# Auth Notes — Synthetic Authentication and Per-User Authorization

This loopback-only SSR sample separates four responsibilities:

1. **Authentication:** possession of an OS-random synthetic Alice or Bob login code printed only to the local terminal.
2. **Session:** the opt-in `yetlet-session` provider owns opaque cookie identity, absolute expiry and CSRF.
3. **Authorization:** every note read, update and delete compares the authenticated user with the server-owned note owner.
4. **Storage:** a bounded process-local note map; it is not a database or persistence example.

It does not implement passwords, password hashing, registration, recovery, MFA, OIDC, rate limiting or a production identity provider. No login code is compiled into the application, sent in SSR or stored in browser assets.

## Run

From the repository root:

```bash
cargo run -p yetlet-auth-notes
```

The app listens on `127.0.0.1:3010` by default and rejects non-loopback listeners. At startup it prints separate 256-bit lowercase-hex codes for `alice` and `bob` to the local terminal. For deterministic local automation, set **both** `YETLET_AUTH_ALICE_CODE` and `YETLET_AUTH_BOB_CODE`; setting only one fails startup. `YETLET_AUTH_LISTEN` overrides the loopback listener.

The YT-066a qualification fixture may additionally set `YETLET_AUTH_ISOLATED_CONTAINER_LAB=1` and bind `0.0.0.0` inside its private Docker bridge. This opt-in uses a separately named application entrypoint, does not weaken the default loopback rejection, and is valid only when app containers publish no host ports and the reverse proxy is bound to host loopback. It is not a production deployment mode.

Routes:

- `/healthz` — outside Session scope and never creates a cookie.
- `/auth/login` — GET login form and POST synthetic-code authentication.
- `/auth/notes` — authenticated per-user list and POST create.
- `/auth/notes/{id}` — authenticated read with server-side ownership check.
- `/auth/notes/{id}/update` — authenticated POST update.
- `/auth/notes/{id}/delete` — authenticated POST delete.
- `/auth/logout` — authenticated POST logout.

## Security contract

Login first requires the anonymous Session CSRF token and compares the supplied 64-character code in constant time against either the selected identity or a fixed-shape dummy. Successful login uses `replace_and_rotate`, changing both Session ID and CSRF; the anonymous cookie and token cannot authenticate or mutate. Logout verifies CSRF and rotates to a fresh anonymous Session; the old authenticated cookie cannot resume access.

Every create/update/delete/logout form requires the current Session CSRF token. Unknown, duplicate or oversized form fields fail closed. The body limit is 4 KiB. Session cookies are `HttpOnly`, `SameSite=Lax`, `Path=/`, bounded by absolute expiry and server-side capacity. `yetlet-session` is Secure by default; this sample explicitly disables Secure only after enforcing a loopback HTTP listener. A real HTTPS deployment must retain the Secure default.

Unauthenticated protected requests return **401**. A real note owned by the other synthetic user returns **403**, explicitly demonstrating that knowing an ID is not authorization. Unknown owned-resource IDs return **404**. This intentionally clear teaching behavior reveals that an ID exists; a production application may instead use an indistinguishable 404 policy after threat analysis.

Notes are bounded to 16 per user, 128 total, 80 title characters and 2,000 body characters. Ownership is never accepted from form input. HTML rendering escapes titles/bodies. Notes and sessions are process-local and disappear at restart. The sample provides no persistence, encryption at rest, distributed revocation, audit log, brute-force defense or proxy/TLS configuration.

## Verify

```bash
cargo test --locked -p yetlet-auth-notes --all-targets
cargo build --locked -p yetlet-auth-notes
cargo xtask smoke auth-notes
cargo tree --locked -p yetlet-auth-notes --edges normal
cargo xtask check samples
```

See [M9h verification](../../docs/M9H-VERIFICATION.md). This is a bounded authentication/authorization semantics fixture, not production security certification.
