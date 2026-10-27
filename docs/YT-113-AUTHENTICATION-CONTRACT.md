# YT-113 — Authentication contract review

**Date:** 2026-10-22. **Source base:** `ab655be54f86f04a0a00b1d57785a14dff54c915`.
**Status:** Done at documentation-only research scope after independent Reviewer PASS. No authentication
crate, runtime, dependency or existing application is changed by this document.

The owner selected documentation-only YT-113 after the six-file M21 planning
commit. This decision refines [ADR 0005](adr/0005-opt-in-capability-modules.md)
within the existing [HTTP pipeline](HTTP-PIPELINE.md), [handler-first ADR](adr/0006-handler-first-mvc-optional.md)
and [threat model](THREAT-MODEL.md). YT-114 and YT-115 require separate owner
authorization even after this research passes review. The [M21 registration
review](YT-113-123-OPT-IN-CAPABILITIES.md#registration-review-2026-10-22) accepts
the earlier roadmap only; it is not this item's review.

## Existing consumers and reusable boundaries

| Inspected source | Existing behavior | Contract consequence |
|---|---|---|
| [Auth Notes](../examples/auth-notes/src/lib.rs), `current_user`, login and note handlers | Loopback-only synthetic codes; `Session<AuthSession>` holds an optional local user; login replaces/rotates the session; each note access checks ownership | A session is not identity verification. Preserve native hidden CSRF fields, application-owned login and per-resource checks; synthetic credentials remain demonstration-only |
| [Board domain](../examples/lab-operations-board/src/domain.rs), `Principal`, `LoginSession` | Application principal contains local name, project and write permission. Optional Valkey serialization stores the local name and reconstructs its permissions from the local allowlist | Do not copy this product permission model into a generic principal. Protected server-store DTOs are distinct from browser serialization |
| [Board handlers](../examples/lab-operations-board/src/lib.rs), `user`, `home`, mutation handlers | Session lookup precedes project/write checks; authentication failure and forbidden access differ; the home route deliberately redirects to login | Identity extraction must coexist with explicit application policy and native redirects |
| [Board OIDC](../examples/lab-operations-board/src/oidc.rs), `discover`, `finish`, `callback`; [Admin composition](../examples/admin-dashboard/src/oidc.rs) | Maintained `openidconnect` validates the flow; one configured issuer maps verified subjects to existing local users; SecretProvider is injected at startup | Keep protocol and cryptography in the maintained client. Admin reuses Board; these are not two independent OIDC implementations proving an adapter is needed |
| [Memory Session](../crates/yetlet-session/src/lib.rs), `open`, `Session`, `session_middleware` | Synchronous operations; absent/unknown/expired cookie may create a fresh default anonymous session; malformed cookie is rejected; middleware applies private headers | A new anonymous session must never restore a retired authenticated identity. Do not rewrite this provider to async or claim it reports every old-cookie reason distinctly |
| [Redis Session](../crates/yetlet-session-redis/src/lib.rs), `open`, `read`, `replace_and_rotate`, `destroy` | Header parsing is synchronous; authoritative operations are async and distinguish missing/expired/rotated/revoked from store failures | Reuse the existing provider without local fallback or an all-provider abstraction. Preserve operation uncertainty/conflict handling |
| [Board Session selection](../examples/lab-operations-board/src/sessions.rs) | App-local Memory/Valkey enum; only the login page explicitly ensures anonymous shared state; remote failures have separate mappings | This application convenience does not justify a mandatory framework store registry or changing Memory signatures |
| [RequestContext](../crates/yetlet-server-axum/src/context.rs), [middleware](../crates/yetlet-server-axum/src/middleware.rs), [App](../crates/yetlet-server-axum/src/app.rs) | Original request extensions, typed State, unconsumed body, native route/group layers and Router escape hatch | Use those seams; add no fixed User/Session/Cookies field or second middleware engine |

The [accepted OIDC evidence](YT-094-OIDC.md) remains bounded: discovery, keys,
credentials and subject mapping are startup snapshots; local logout is not IdP
logout. Memory permission snapshots and Valkey's local-name reconstruction do
not demonstrate dynamic account-disable propagation or immediate cross-node
revocation. This study does not rerun or extend their accepted qualification.

## D1 — Minimal principal and trust provenance

Select a separate, native-only opt-in `yetlet-auth` foundation for a later
YT-114 slice. It depends on the native Axum boundary, not the browser renderer;
core and server adapter must not depend on it. There is sufficient repeated
identity/extraction work to select this narrow boundary, but insufficient
evidence for an OIDC adapter or generic permission system.

`Principal` is an immutable request-local value with private fields and getters:

| Value | Meaning and source |
|---|---|
| `issuer` | Exact validated issuer, or an explicitly configured application identity namespace for a non-OIDC provider; namespaces must not collide |
| `subject` | Stable identifier within that issuer, obtained only after provider verification; never a submitted username, email or display name used as proof |
| `scheme` | Application-configured provider/validator identifier, retained through Session continuity; a routing/provenance label, not an authentication-strength claim |
| `tenant: Option<...>` | Optional tenant binding verified by the provider and/or authoritative local mapping; absence grants no tenant access |

Identity comparison uses the exact `(issuer, subject)` pair. Scheme is additional
provenance: a route may restrict which configured scheme it accepts. Tenant is
a separately checked scope, never a substitute identity or permission. No case
folding, trimming or URL canonicalization may merge distinct external identities.
Reject empty/control-bearing values and enforce documented finite size limits
before construction; YT-114 must specify those limits without truncating values.
OIDC-specific syntax/claim limits remain the adapter's responsibility.

Use an explicitly named verified-input constructor, such as
`Principal::from_verified`, with shape validation. This constructor asserts the
trusted application's prior verification; it does **not** validate a token,
cookie or issuer by itself. Private fields and a marker are not a sandbox
against malicious server code. The trusted computing boundary includes the
application, configured resolver, storage and installed middleware.

There are no roles, permission lists, project IDs, email, credentials, bearer
tokens, Session IDs, provider handles or arbitrary claim bags in this value.
Local account IDs, authentication time/expiry and authorization versions remain
typed application/provider state. No `Default`, serde or View conversion is
selected for Principal; Debug must redact identity values. Clone may copy the
immutable snapshot, without making it a durable authorization decision.

## D2 — Identity resolution and typed extraction

The semantic API below is selected for YT-114; it is **proposed, uncompiled
API**, not a claim that these symbols exist. Rust generic/lifetime spelling and
private storage layout belong to that implementation review.

| Surface | Selected obligation |
|---|---|
| Scoped authentication composition | One explicitly supplied resolver plus native failure-to-Response mapping, installed through Axum/Tower on selected routes; ordinary typed State/constructor injection |
| Resolver | Conceptually `resolve(&mut Parts, &S) -> Result<Option<Principal>, Response>` using normal async Rust; `Ok(None)` means positively resolved anonymous, `Ok(Some(...))` means verified, `Err(...)` stops the request |
| `Authenticated` extractor wrapping Principal | `FromRequestParts<S>`; returns the verified principal or the scope's explicit unauthenticated response; does not read or buffer the body |
| `Option<Authenticated>` | `OptionalFromRequestParts<S>`; returns None only for the installed resolver's anonymous outcome; errors never become anonymous |
| Authorization seam | An ordinary application function taking the principal plus the authoritative resource/action/tenant and returning native `Result<(), Response>`; only an explicit allow succeeds |

The layer records a private request-local resolved state, including anonymous
resolution. Extractors read that state, not raw `Extension<Principal>` values or
client identity headers. A missing layer/state is a configuration error (generic
500), including for optional extraction. Public handlers without Auth neither
install this layer nor request these extractors.

The resolver must replace/disregard preexisting raw principal assertions and
bind the private state to its selected scope. A second authentication resolver
on the same request is rejected as configuration error rather than silently
overwriting provenance. Separate sibling groups can use different providers;
nested authorization layers reuse the one resolved identity. No automatic
provider iteration, cookie-to-bearer fallback or process-global identity exists.
Trusted application middleware may intentionally issue identity through the
resolver; arbitrary client headers, form fields and hydration values cannot.

Resolution runs once before downstream handler extraction; a rejected request
does not poll its body. `Authenticated` precedes any Form/Json/body-owning
RequestContext parameter. Business authorization that needs body data must still
run before mutation, with ordinary route-selected body limits. Background work
must explicitly carry a bounded identity reference and reauthorize when it
executes; it cannot inherit an ambient current user.

## D3 — Anonymous, errors and authorization

| Condition | Required behavior |
|---|---|
| No credential, or a successfully resolved anonymous Session | Optional extraction is None; required extraction invokes the configured unauthenticated response |
| Presented credential definitively invalid/expired/revoked | No authenticated principal; reject on both required and optional routes, except a provider's documented anonymous replacement behavior described below |
| Verified identity but no local account mapping, disabled account, wrong tenant/resource, or absent/denied policy | Deny without protected data or mutation; normally 403; an explicitly selected existence-hiding 404 is application policy |
| Malformed/duplicate configured Session cookie | Preserve provider 400; do not retry another scheme |
| Provider/store/account-policy service unavailable or result uncertain | Generic 503; no anonymous downgrade, local store fallback, cached allow beyond its accepted validity, or login redirect |
| Missing auth composition, duplicate resolver or inconsistent scope | Generic 500; do not expose type names, keys or state |

The Memory provider may turn a well-formed retired/unknown cookie into a new
anonymous session before the bridge sees it. That anonymous replacement is
allowed on deliberately anonymous routes, and required routes must still deny
access. Redis's definite retired-credential errors remain errors. These are
documented provider differences, not grounds to alter either existing API.

Authentication establishes who; authorization independently checks whether that
identity may perform this action now. No mapping/decision means deny. UI hiding,
successful extraction, a tenant claim, user-supplied ownership, CSRF or a request
ID never grants permission. App-owned policy may perform synchronous checks or
await its own authoritative service; no policy DSL, role schema, mandatory DI,
Controller or service locator is selected.

For HTTP authentication failures, the selected mapper returns 401 with an
applicable configured `WWW-Authenticate` challenge; 403 represents a definitive
policy denial. No default Basic/Bearer challenge is invented for a cookie-only
flow. A native form application can instead explicitly select a 303 redirect to
a configured local login route, without replaying the original body. Redirects
must not turn forbidden access, provider outages or an attacker-supplied return
URL into login navigation. YT-114 must require explicit failure mapping; YT-115
must select the cookie consumer's response policy. Existing sample bare-401
responses are historical behavior, not proof of a compliant generic challenge.

All authenticated/personalized responses and authentication failures use
`Cache-Control: no-store`, including redirects and 4xx/5xx. Preserve Session's
Set-Cookie and Vary handling; shared caches must bypass private responses.
Messages and metrics use bounded categories, not identity values, credentials,
headers, authorization codes, tokens or provider error bodies.

## D4 — Session, CSRF and Cookie ownership

| Owner | Responsibility |
|---|---|
| Authentication provider/application | Verify login evidence; map identity; track validity and account availability; issue Principal only within those bounds |
| Existing Session provider | Opaque ID, authoritative state, expiry/capacity, atomic update, replacement/rotation, destruction, cookie parsing/emission and its actual store/error guarantees |
| YT-115 bridge | Translate successfully read server-owned identity references into request identity; distinguish absence/retirement from outage; preserve store and response semantics |
| Application mutation handler/guard | Validate CSRF/origin as applicable before mutation; enforce action/resource authorization and transactional consistency |
| Application/deployment | Select production Secure/HttpOnly/SameSite/path/domain/TLS policy, credential lifecycle and external operational controls |

Do not install Session when Auth is selected without a Session provider; do not
require Auth for anonymous Sessions. Reuse SecretProvider via typed injected
handles. Keep Memory methods synchronous and Redis operations asynchronous; no
universal async store trait, new cookie jar or duplicate storage engine is needed.
Current providers use SameSite=Lax and production-secure cookies by default;
the samples explicitly choose local insecure HTTP. Different cross-site cookie
policies need separate provider selection, not an implied new setter.

On login/privilege boundary change, replace and rotate Session identity and CSRF
before issuing authenticated continuity. Logout requires CSRF and revokes local
Session state; subsequent protected access must fail. Concurrent stale responses
must not revive or overwrite the replacement. Retain Redis conflict/unknown
operation semantics: do not retry an uncertain mutation as if it never happened.

Session-bound CSRF tokens deliberately rendered in the initiating user's
escaped, protected `no-store` native forms remain supported, including login and
logout without JavaScript. Validate before mutation; SameSite alone is not CSRF
verification. Never put tokens into URLs, generic hydration, logs, metrics or
shared caches. Principal and request extensions are never automatically rendered;
applications may separately select escaped non-secret display data for a view.
Protected server-store DTO serialization is allowed; it must not deserialize
browser-supplied roles or directly reconstruct a trusted Principal from input.

## D5 — OIDC mapping and authentication freshness

For an OIDC consumer, preserve the **verified** exact issuer/subject pair from
the maintained client through server-side Session continuity. The current Board
maps a subject under one configured issuer and then stores a local identity;
it does not already retain the proposed generic external-identity record. A
later bridge must carry an issuer/subject/scheme reference or a server-owned
binding to that exact pair, rather than inventing an issuer from a local name.

Resolve the pair through an explicit local mapping on every authorization
decision or through a deliberately bounded, invalidatable server-side snapshot.
No matching account means no business access. Email, preferred username,
display name and provider roles cannot create/link an account or grant access.
Different issuers with the same subject remain different identities. Pairwise
subjects across different clients are not automatically linked. A tenant must
be mapped and checked against the requested resource separately.

| Event or timing boundary | Selected contract and limit |
|---|---|
| Each new protected request | Resolve authoritative Session validity and evaluate current account/policy validity before allowing work; no timeless Principal cached across requests |
| Permission change/account disable | Application selects a live lookup or a bounded authorization-version/expiry check with invalidation. Unknown/stale validity denies; provider outage is 503. The interval and affected nodes must be explicit in YT-115's consumer evidence |
| Session logout/rotation/expiry | A subsequent lookup cannot authenticate using the retired record. Memory is process-local; Redis scope remains the accepted single-primary experimental scope |
| Concurrent in-flight request | A request-local snapshot cannot promise instant cancellation after revocation. Security-sensitive writes recheck validity and authorization at the transaction/commit boundary or use equivalent atomic version checks; streaming/long jobs need explicit revalidation/termination policy |
| IdP account/key/client/mapping change | The current OIDC sample requires restart for its configuration snapshots. Session lifetime is independent of original ID Token expiry. Document maximum local reauthentication/validity age; do not claim immediate IdP revocation or refresh |

No universal freshness timestamp, revocation bus, polling service or background
refresh task is introduced into Principal or the foundation. An unbounded/stale
allow is not a valid implementation of the bridge contract. Existing deployments
are not retrospectively certified by these future requirements.

OIDC remains application-owned for YT-114/115. [YT-116](YT-113-123-OPT-IN-CAPABILITIES.md#yt-116--optional-oidc-adapter-study-and-selected-integration)
must separately decide whether extraction has value. Keep discovery, HTTPS,
state/nonce/PKCE, ID Token signature/issuer/audience/time verification and
one-time callback handling in maintained library integration. Revisit the
existing [advisory applicability](YT-094-OIDC.md#dependency-audit-applicability--2026-10-21)
when dependencies change. No protocol, password/MFA system or JWT cryptography
is implemented here, and no live IdP or credentials are needed for this study.

## D6 — Dependency, cost and follow-on acceptance

Unselected Auth means no normal dependency from core/server/browser to Auth,
Session, OIDC, Redis or a password library; no middleware, State field, lookup,
allocation, network operation or background task is added to a baseline request.
This is an absence-of-installed-work contract, not a measured zero-overhead
claim for installed middleware. Workspace membership/building all optional
packages is distinct from an application's selected dependency graph.

Auth types and provider dependencies stay native-only and out of shared View
and browser WASM graphs. Do not add Auth to default features or core reexports.
YT-114 proves these properties with the existing baseline/graph checks; this
documentation change proves them only by making no manifest or runtime edits.

The following are focused future acceptance scenarios, **not executed tests**:

| Follow-on | Required evidence before its own Reviewer PASS |
|---|---|
| YT-114 | Required/optional extraction for anonymous, verified, invalid and unavailable outcomes; missing/duplicate resolver failure; forged headers/raw extensions; explicit allow/deny including unmapped/wrong-tenant identity; 401 challenge versus selected redirect and 403; sibling/nested scope, 404/405 behavior, no body polling on rejection; private response headers; unchanged baseline and browser dependency isolation |
| YT-115 | Anonymous/login/CSRF/logout without JS; rotation/fixation/replay/expiry and stale-response denial; Memory/selected async-provider error distinctions; cookie/private-cache behavior; permissions/account disable and documented freshness bound; outage/uncertain operation denial; independent review for the selected provider/consumer scope |

Use existing focused tests and real HTTP checks required by those backlog items;
no new validation infrastructure, production/cluster promise or broad rerun is
selected here. Redis/IdP execution and browser checks require the later selected
integration; the YT-113 documentation gate does not pretend to run them.

## References checked for this decision

Consulted on 2026-10-22 alongside the exact repository sources above:

- [Axum FromRequestParts](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html) and [OptionalFromRequestParts](https://docs.rs/axum/0.8.9/axum/extract/trait.OptionalFromRequestParts.html), documented as 0.8.9: body-free extraction and optional extraction that retains errors support D2.
- [Axum middleware](https://docs.rs/axum/latest/axum/middleware/) and [Router route_layer](https://docs.rs/axum/latest/axum/routing/struct.Router.html#method.route_layer), documented as 0.8.9: native Tower ordering and matched-route scopes support the existing pipeline rather than a new engine.
- [OpenID Connect Core, claim stability](https://openid.net/specs/openid-connect-core-1_0.html#ClaimStability): the stable external key is issuer plus subject; other profile claims do not replace it. The application-owned mapping/freshness rules above are Yetlet decisions, not claims that the protocol provides business authorization.
- [openidconnect client documentation](https://docs.rs/openidconnect/latest/openidconnect/), documented as 4.0.1: retain the existing maintained client and its async integration, subject to the repository's explicit feature/advisory limits.
- [RFC 9110, 401](https://www.rfc-editor.org/rfc/rfc9110.html#name-401-unauthorized) and [403](https://www.rfc-editor.org/rfc/rfc9110.html#name-403-forbidden): distinguish authentication challenges from refusal. Native login redirects and failure mapping remain explicit application decisions.

## Validation and review record

`cargo xtask check docs` PASS: 75 required documents and 123 backlog items;
English prose, existing internal-link/structure/whitespace and dependency DAG
checks pass. The count is the checker's fixed required-document inventory, not
the number of Markdown files in the repository. All seven changed documents
also passed direct local-link target, EOF and code-fence inspection, including
this new file; `git diff --check` and its explicit untracked-file
`git diff --no-index --check -- /dev/null docs/YT-113-AUTHENTICATION-CONTRACT.md`
check pass. No compiler/runtime/browser/CSP or IdP workload was rerun for this
documentation-only slice. Independent native Reviewer `/root/yt113_contract_review_20261022` returned
PASS with zero issues, zero required followups and zero optional notes. Local
child trace verified the reviewer role, `gpt-6-astra` model and effective `high`
effort against the healthy current workspace profile and shared resolver. The
seven review-target hashes matched through completion; only status/review
bookkeeping changed afterward. The reviewed body from the consumer inventory
through the references remains SHA-256
`c78b08da00618f5e57229378f1afe9e16d4cf143cd022d28dff6be62ee143267`.
The original M21 registration FAIL/PASS history is preserved separately.
No YT-114/115 implementation, research-result commit, push or release follows
automatically. YT-096 remains In progress and YT-020 remains Blocked.

## Subsequent implementation

The owner separately authorized [YT-114's native foundation](YT-114-IMPLEMENTATION.md)
after this reviewed research was committed. Its concrete API and validation
are recorded there; the reviewed contract body and prior review record above
are preserved. YT-115 remains unselected.
