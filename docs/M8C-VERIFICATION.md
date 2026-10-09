# M8c Verification — Opt-in Session and SSR Cart

**Date:** 2026-10-09
**Task:** YT-046 / M8c
**Status:** Locally complete. Source, replacement validation and final independent reviewer PASS are frozen.

## Outcome

M8c adds two bounded, native-only workspace members:

- `yetlet-session`, an explicitly installed process-local server-side Session provider;
- `yetlet-cart-server`, a strict-CSP SSR shopping cart that uses ordinary handlers, native forms and the provider.

Neither Yetlet core nor the server adapter depends on Session. The Todo, REST and browser packages remain Session-free unless an application adds the crate and installs its middleware.

## Session provider contract

`SessionManager<T>` integrates through native Axum/Tower route middleware and a typed `Extension<Session<T>>`. It does not add a fixed field to `RequestContext`, create a DI container or hold the store lock across `await`.

The bounded in-memory provider supplies:

- 256-bit OS-random opaque session IDs and independent 256-bit CSRF tokens;
- strict lowercase-hex cookie values and duplicate/malformed-cookie rejection;
- no adoption of unknown caller-supplied IDs;
- absolute expiry, expired-record purge and configured capacity limits;
- `read`, rollback-safe `try_update`, atomic `rotate`, `replace_and_rotate` and `destroy` operations;
- Secure cookies by default, with `HttpOnly`, `SameSite=Lax`, `Path=/` and bounded `Max-Age`;
- personalized-response `Cache-Control: no-store`, `Vary: Cookie` and `nosniff` headers;
- explicit scoped installation, leaving sibling routes and dependency graphs untouched.

The localhost sample deliberately calls `insecure_for_local_http()` so its cookie can round-trip without TLS. Its app constructor rejects every non-loopback listener before binding. This exception is named, local and bounded; production deployment must keep the Secure default.

## Cart behavior

The sample exposes:

- `GET /healthz` outside the Session scope;
- `GET /cart` for the SSR catalog and current cart;
- POST-only add, decrement, remove, clear and end-session actions.

Every mutation requires a session-bound synchronizer token rendered into a hidden form field. Invalid tokens return 403 before state mutation. Clear replaces the cart and rotates both ID and CSRF token. End removes the server record and expires the cookie. Refreshing `GET /cart` with the same cookie preserves state.

Application state is bounded to three distinct lines and quantity 20 per product. Form bodies are capped at 2 KiB. The response contains no script, browser runtime, generated asset or persistent event circuit; Yetlet's default CSP retains `form-action 'self'`.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo build --locked -p yetlet-cart-server
python3 scripts/smoke_cart.py
cargo tree --locked -p yetlet-cart-server
cargo tree --locked -p yetlet-todo-server
cargo tree --locked -p yetlet-rest-api
cargo tree --locked -p yetlet-todo-browser --target wasm32-unknown-unknown
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

The cart tree must contain `yetlet-session` and exclude browser/WASM crates. The Todo, REST, server-adapter and browser dependency trees must exclude `yetlet-session`, `cookie`, `getrandom`, `subtle` and `time` introduced by this provider.

Final replacement validation passes **114/114 workspace tests**, **5/5 doctests**, strict workspace Clippy/format, the six-package wasm32 check, **10/10 Session tests**, **6/6 cart tests**, the real localhost smoke, three-sample catalog/docs gates and all declared dependency-isolation checks. Twenty-four untracked files pass whitespace validation and no cart server process remains.

## Limits

This is anonymous process-local state, not authentication. It is not durable, replicated, distributed, encrypted at rest or suitable for multi-node deployment without a shared provider. Absolute expiry is not sliding. Clock rollback, key rotation for protected client-side data, trusted-proxy reconstruction, login-time fixation rotation, authorization and production CSRF/security qualification remain outside this bounded sample.

## Reviewer gate

First independent read-only reviewer Job `wc_job_3t3FBXQ1LJnaVDWY` used workspace model `gpt-6-astra`, high effort and returned three material P2 findings:

1. `rotate()` could read and replace in separate store critical sections, losing a successful concurrent update;
2. a stale request invalidated by another request's rotation could emit a removal cookie and delete the replacement browser identity;
3. the insecure localhost cart accepted a configured non-loopback listener.

All findings were accepted. Rotation now snapshots the latest value and replaces identity under one store lock; a concurrency regression asserts that any successful competing update is preserved. Missing/expired identities in stale requests no longer emit a removal cookie; a focused regression verifies the replacement cookie and state remain usable. The cart app requires an IPv4 or IPv6 loopback listener before enabling its named insecure-cookie exception.

Final repeat independent reviewer Job `wc_job_ck1QISavWWiI86g8` used `gpt-6-astra`, high effort and a read-only sandbox. It made no repository changes and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "Repairs A–C verified against source and regression tests; no concrete P0–P2 finding remains."
  ]
}
```

The reviewer confirmed the unchanged `master` HEAD, empty index, cleanup and absence of M8d implementation. YT-046 was closed at that stage.

During the M8e final review, Job `wc_job_dospP8AM0wUmvHFd` found one additional stale-destroy variant: after another request rotated an identity, `destroy()` could still queue a removal cookie even though it removed no server record. The repair now emits the removal cookie only when the exact record was removed; an already-retired identity returns `SessionError::Destroyed` with no cookie directive. `stale_destroy_after_rotation_does_not_clear_replacement_cookie` verifies prior CSRF validation, concurrent rotation, no stale `Set-Cookie` and continued replacement-state usability. Focused Session/Cart, complete workspace/doctest, Cart smoke, dependency and production-asset checks all pass. M8e repeat reviewer `wc_job_J_MVNzFaVgTH1Gr1` verified the repair and returned PASS with no issues or required follow-ups.
