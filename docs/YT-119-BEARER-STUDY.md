# YT-119 — Bearer authentication adapter feasibility

**Date:** 2026-10-22. **Status:** Done at research scope after independent Reviewer PASS.
**Decision:** Keep current opaque capabilities application-owned. Do not add a
Bearer/JWT feature, provider or crate; later implementation needs explicit
selection of a real identity-aware API and its token lifecycle.

## Real consumers and identity gap

The repository already has two real HTTP API consumers; this study does not
assume bearer use is absent:

| Consumer | Source-backed behavior | Reuse limit |
|---|---|---|
| [File Server](../examples/file-server/src/lib.rs), `FileStore::new/protected` | Configured 64-hex development capability; exactly one Authorization value, bounded constant-time comparison, explicit Bearer challenge on failure and Origin rejection | One app-wide capability over bounded file operations, not a stable per-user issuer/subject or delegated audience/scope model |
| [Production Starter](../examples/production-starter/src/web.rs), `app/authorize_api` | Separate bearer API and anonymous browser-CSRF routes; configured protected token, duplicate-header rejection, constant-time comparison, native API errors and no-store observation | App-wide operator capability; no identity authority, JWT/JWKS or introspection consumer. Existing bare 401 error needs an applicable challenge if migrated to the new Auth contract |

These loopback/reference-app capabilities do not establish production token
issuance, expiry, audience, per-user revocation or client delegation. Never use
the token itself as Principal subject or persist it in SSR/hydration/logs. An
application could explicitly bind a verified capability to a configured local
service account, but that binding is policy, not an inferred user identity.

## Alternatives and selected boundary

| Option | Advantages | Required evidence before implementation |
|---|---|---|
| Existing opaque, app-owned capability | No JWT/protocol library or remote dependency; existing minimal route guards | Real owner-selected service-account identity and resource policy; explicit rotation/expiry/revocation, transport and no-default-cookie fallback |
| Opaque OAuth access token with introspection | Authority can supply active state and metadata | Trusted configured endpoint/client credentials, maintained async client, explicit audience/account/scope checks, bounded timeout/cache and fail-closed outage behavior |
| JWT access token with JWKS | Local verification can avoid a lookup per request | Maintained validator; fixed allowed algorithms and issuer/audience/type/time rules; trusted key source, unknown-key rejection, bounded refresh/skew and explicit revocation/freshness limit |

The [oauth2 client documentation](https://docs.rs/oauth2/latest/oauth2/)
(5.0.0 when consulted on 2026-10-22) includes introspection/revocation support.
[jsonwebtoken](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/) (11.1.0 when
consulted) is a possible maintained JWT validation building block, not a selected
dependency or complete resource-server policy. Its unsafe/header-only decoding
interfaces do not establish verified identity. No crypto/provider/backend or
version is pinned by this study, and no permissive decoding mode is accepted.

[RFC 6750](https://www.rfc-editor.org/rfc/rfc6750.html) describes bearer transport
and challenge/error semantics. For a future Yetlet resolver, accept only the
explicitly selected Authorization-header mechanism, reject ambiguous duplicates,
keep tokens out of query strings/form/hydration, use applicable 401 challenges,
separate verified-but-denied 403, and never fall back to Session/cookies.

[RFC 7662](https://www.rfc-editor.org/rfc/rfc7662.html) defines token
introspection; an active result alone does not select the application's audience
or business policy. [RFC 9068](https://www.rfc-editor.org/rfc/rfc9068.html) defines
a JWT access-token profile; an OIDC ID Token is not automatically an API access
token. These sources were checked on 2026-10-22; the bounded resolver/error and
local-account decisions here remain application/Yetlet choices.

## Required contract if later selected

Provider verification precedes `Principal::from_verified`; exact issuer/subject
and optional tenant map to authoritative local policy. Missing, malformed,
expired/revoked and unavailable outcomes retain the [YT-113 distinctions](YT-113-AUTHENTICATION-CONTRACT.md).
No unsigned role/email claim grants permissions. Authentication success alone
does not permit a resource/action. Unknown key, ambiguous audience, stale policy
or unavailable verifier cannot become an anonymous or permissive request.

Only explicitly installed route scopes run the resolver; it reads headers/Parts
without buffering bodies. Secrets and provider handles stay in injected native
State; browser/WASM/default graphs stay free of the provider. Redacted no-store
errors/logs must not reveal tokens, provider bodies or private configuration.

The two current apps demonstrate HTTP-header capabilities but no duplicated
identity-aware token-verification implementation to extract. The existing Auth
resolver seam already accepts an ordinary native provider function. Keep that
seam rather than inventing an all-schemes registry or JWT implementation. A later
real delegated API consumer can change the selection with bounded negative,
expiry/revocation/permission/outage tests and separate independent review.

## Evidence and closure

This is source-backed research, not token-provider implementation or a fresh
runtime/security qualification of the two consumers. No credentials were read
from private settings or used, no endpoint contacted, no dependencies changed
and no passing HTTP/IdP workloads rerun. Source locations above and primary
references support the comparison. The study may be Done after its own Reviewer
PASS; any provider implementation remains unselected. Documentation validation
and review result are recorded at closeout.

Validation: `cargo xtask check docs` PASS (75 fixed required documents, 123
backlog items and English corpus); direct changed/new-file links, EOF and
whitespace checks PASS. Independent Reviewer `/root/auth_studies_review_20261022` PASS (2026-10-22),
zero issues and required followups for this item; all eight reviewed document
hashes matched before closeout. No runtime rerun.
