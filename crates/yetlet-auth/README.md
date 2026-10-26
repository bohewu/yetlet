# yetlet-auth

Install native identity resolution only on selected routes. Authentication
establishes a verified identity; business authorization remains application-owned.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use Rust 1.99+ and this unpublished source checkout. Select `yetlet-auth`
explicitly in a native Axum application. Supply a resolver, its typed state,
an anonymous-response closure and resource authorization. The wiring below
assumes a fallible setup function; it is not a listener or a real login provider.
Enable `session` only for the separately documented Memory bridge.

## Minimal integration

### Native composition

Depend on this crate explicitly, then apply native Axum/Tower middleware only
to selected routes. Required extraction is `Authenticated`; optional extraction
is `Option<Authenticated>`. Both read the layer's private resolved state, without
consuming the body. State injected into the authentication layer is separate
from native handler State; ordinary application structs and Arc handles work.

This compiling wiring example deliberately resolves everyone as anonymous and
has a deny-by-default application policy. Replace the resolver with a maintained
provider's verified result and the policy with authoritative resource checks.
No submitted identity becomes trusted merely by calling the constructor.

```rust
use axum::{
    Router, http::{StatusCode, request::Parts},
    middleware::from_fn_with_state,
    response::{IntoResponse, Redirect}, routing::get,
};
use yetlet_auth::{
    Authenticated, Authentication, Principal, ResolveFuture, authenticate,
};

fn resolve<'a>(_parts: &'a mut Parts, _state: &'a ()) -> ResolveFuture<'a> {
    Box::pin(async { Ok(None) })
}
fn authorize(_principal: &Principal, _resource: &str) -> Result<(), StatusCode> {
    // No explicit application allow means denial.
    Err(StatusCode::FORBIDDEN)
}
async fn private_page(identity: Authenticated) -> Result<&'static str, StatusCode> {
    authorize(identity.principal(), "account")?;
    Ok("Private account")
}
let authentication = Authentication::new("application", (), resolve, || {
    Redirect::to("/login").into_response()
})?;
let app: Router = Router::new()
    .route("/account", get(private_page))
    .route_layer(from_fn_with_state(authentication, authenticate::<()>));
# Ok::<(), yetlet_auth::InvalidAuthenticationInput>(())
```

The same layer works with `App::route_layer`; no dependency on the server adapter
is needed in this crate's normal graph. Register routes before their layer;
sibling groups remain independent. Duplicate resolver layers on one request
produce a generic 500; nested authorization layers reuse the resolved identity.
Native Router 404/405 and middleware ordering rules remain authoritative.

## Expected result

The shown resolver returns `Ok(None)`: a request to the assembled `/account`
scope selects the explicit `/login` redirect, rather than granting access. The
example constructs no `/login` page. A real verified principal still needs the
application authorization check; the policy shown defaults to denial.

## Configuration and errors

### Principal and resolver

`Principal::from_verified(issuer, subject, authentication_name, tenant)` accepts trusted,
already-verified input and checks only shape. Nonempty, non-whitespace-only,
control-free values have these UTF-8 byte limits, without normalization or
truncation: issuer 2048, subject 255, authentication name 64, optional tenant 255.
Invalid inputs return value-free `InvalidAuthenticationInput`; the same error
covers invalid `Authentication::new` registration configuration. Provider-specific
protocol/claim rules are additional obligations. Getters expose only those four
fields; `same_identity` compares exact issuer+subject, independently of
authentication name/tenant.
Debug redacts identity. There is no serde, Default, View or implicit principal
extraction from public extensions, headers or browser state.

`authentication_name()` returns the application-configured authentication
registration/resolver label, with the byte bound `MAX_AUTHENTICATION_NAME_BYTES`.
It is not an HTTP `WWW-Authenticate` protocol scheme, issuer, storage-backend type
or permission. Matching uses the exact configured string, without normalization.
The fixed Display message is `invalid principal or authentication scheme`;
it includes no rejected values. Both constructors return `InvalidAuthenticationInput`.

`Resolver<S>` is a borrowed Parts/state function returning `ResolveFuture`:
`Result<Option<Principal>, Response>`. One boxed future is used per installed
request; configuration/state is shared through Arc. These are selected costs,
not a zero-overhead claim for installed authentication.

| Resolver result / condition | Effect |
|---|---|
| `Ok(Some(principal))` with the selected authentication name | Required/optional extraction succeeds; application authorization is still required |
| `Ok(None)` | Optional extraction returns None; required extraction calls the explicitly supplied anonymous-response closure |
| `Err(response)` | Stops the request; invalid credentials, malformed input and unavailable services must remain errors, even on optional routes |
| No installed layer, duplicate layer, mismatched principal authentication name | Generic 500, never anonymous or a handler call |

The resolver must validate credentials, enforce its validity/freshness bounds,
return redacted responses (normally 400 malformed, 401 invalid credentials,
503 unavailable), and never fall back to a weaker provider. Authentication
errors and application policy errors are native responses; no result hierarchy
or policy engine is added. A raw Principal extension is removed before
resolution. This private-state boundary protects against untrusted request
assertions; it cannot sandbox malicious application/provider middleware.

Failure mapping is explicit. An HTTP 401 needs a nonempty applicable
`WWW-Authenticate` challenge; a missing/empty challenge fails closed to generic
500. The application owns complete challenge syntax and applicability: the
crate does not implement an HTTP authentication scheme. A cookie/form consumer
may explicitly select a local 303 login redirect instead. No Basic/Bearer
challenge, redirect, identity-header parser or permission is installed by default.
Never redirect outages, policy denial or attacker-controlled return URLs.

Every response inside the selected scope, including errors and redirects, is
`Cache-Control: no-store`; existing Set-Cookie, Vary and other headers survive
ordinary processing. Response bodies remain streaming. Explicitly choose
escaped display data for SSR; never render Principal or credentials. The
application/provider owns error/log redaction, authorization and any CSRF,
cookie, revocation or policy service; successful extraction grants no business
access. An uncertain/failed policy lookup must not turn into an allow.

### Boundaries

YT-114 tests a synthetic resolver, not a production login protocol. Distributed Session
bridging, real OIDC, cookie changes, account storage, refresh/revocation services
and provider-specific operational qualification are outside this crate's
current scope. [YT-115](../../docs/YT-113-123-OPT-IN-CAPABILITIES.md#yt-115--optional-session-and-cookie-authentication-bridge)
was separately selected at the Memory bridge scope. Existing Session APIs and
no-JS form support are preserved; asynchronous Redis bridging remains unselected. The release candidate's original supported package set is unchanged;
this crate remains unpublished with publication-prepared metadata and intentionally refuses wasm32 compilation.
Baseline apps do not depend on it, initialize its state or execute its layer.

### Optional Memory Session bridge

Enable `features = ["session"]` explicitly. Compose existing Session middleware
**outside** `authenticate`: the Session extension must exist before resolution.
The `session::memory` helper reads the server-owned reference synchronously and
calls your identity mapping; no Session ID or Principal is automatically stored
in browser state. Unknown/expired cookies may already have become new anonymous
records according to Memory's existing contract. An in-flight retired record
uses your selected failure response; provider unavailability is always 503 and
a missing Session layer is 500, including on optional routes.

```rust
# #[cfg(feature = "session")]
# fn example() {
use axum::{http::{StatusCode, request::Parts}, response::{IntoResponse, Redirect}};
use yetlet_auth::{Principal, ResolveFuture, session};
#[derive(Clone, Default)]
struct IdentityRef { account: Option<String> }
fn resolve<'a>(parts: &'a mut Parts, enabled: &'a bool) -> ResolveFuture<'a> {
    Box::pin(async move {
        session::memory::<IdentityRef>(parts, |record| {
            let Some(subject) = record.account else { return Ok(None) };
            if !*enabled || subject != "account-one" {
                return Err(StatusCode::FORBIDDEN.into_response());
            }
            // This reference came from protected server-owned Session state.
            // A real app performs current authoritative account mapping here.
            Ok(Some(Principal::from_verified("urn:app:accounts", &subject, "memory", None).unwrap()))
        }, || Redirect::to("/login").into_response())
    })
}
# }
```

Login verification, CSRF before mutations, replace-and-rotate on login and
revocation on logout stay explicit. Authorize resources independently and
recheck current Session/account policy at sensitive business commit boundaries.
The qualified HTTP fixture uses live process-local account policy and serializes
logout/writes; it supplies no cluster-wide revocation or production IdP guarantee.
No async rewrite or Redis dependency is added to the foundation.

## API reference

See [Auth source/rustdoc](src/lib.rs), the [Memory helper](src/session.rs) and
the [current API reference](../../docs/API-REFERENCE.md). Rustdoc setup context
lines beginning with `# ` support example compilation; embed the actual wiring
in your own fallible setup function instead of copying them as Rust statements.

<details>
<summary>Historical delivery context and evidence</summary>

Unpublished, experimental, explicitly selected **native** identity foundation.
It implements the [YT-113 contract](../../docs/YT-113-AUTHENTICATION-CONTRACT.md)
at the bounded [YT-114 scope](../../docs/YT-114-IMPLEMENTATION.md). It verifies no
credentials itself and installs no default Session, Cookie, OIDC or Bearer provider.
The separately enabled `session` feature adds a bridge to existing Memory Session;
see [YT-115 evidence](../../docs/YT-115-SESSION-AUTH.md).

</details>
