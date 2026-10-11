# Yetlet Production Edge and Authentication Threat Model

**Date:** 2026-10-10
**Task:** YT-063 / M10d
**Status:** Independently reviewed pre-production reference model. It defines required boundaries; it does not certify a deployment.
**Applies to:** Applications built with Yetlet's native Axum/Tower adapter, optional Session provider and explicit application dependencies.

## 1. Scope and non-claims

This document defines the smallest security topology in which a Yetlet application could be qualified for production. It separates framework behavior, application policy and platform controls.

Public-release task YT-020 remains **Blocked**. Completing or reviewing this requirements document cannot close it.

It does **not**:

- add or approve a Yetlet identity provider, password/passkey server, MFA/recovery system or reusable auth framework;
- add a reverse proxy, WAF, rate limiter, secret manager, distributed Session provider, audit backend or deployment platform;
- select a cloud, IdP, database, load balancer or key-management product;
- claim that the loopback Auth Notes, Session Cart, File Server, SSE or Modular Shop samples are production deployments;
- authorize publication, deployment or closure of public-release task YT-020;
- replace a deployment-specific review, penetration test or incident-response exercise.

Yetlet remains small: applications compose mature edge/identity/storage middleware and services explicitly. No capability becomes a mandatory core dependency because it appears in this threat model.

## 2. Security objectives and protected assets

| Asset / objective | Security requirement |
|---|---|
| User identity | A request principal derives only from a validated external authentication flow and server-side Session state |
| Session identity | Opaque, high-entropy, confidential, rotated on security-boundary changes and revocable across all serving nodes |
| Authorization decisions | Server-side, resource/action specific, default deny, independent of UI visibility or caller-supplied ownership fields |
| OAuth/OIDC tokens and codes | Never exposed through SSR, browser-readable state, URLs after callback handling, normal logs or metrics |
| CSRF tokens | Bound to Session state, compared before mutation and never treated as authentication |
| Application data | Confidentiality and integrity according to application classification; tenant/user boundaries enforced on every access |
| Secrets and provider credentials | Stored and delivered through an external secret/workload-identity system with least privilege and rotation |
| Uploaded files and streams | Bounded, authorized, path-isolated, content-handled safely and cleaned up on cancellation/failure |
| Audit evidence | Actor/action/target/result/correlation recorded without credentials or sensitive bodies, with protected retention |
| Service availability | Requests, connections, queues, uploads, streams, storage operations and shutdown are bounded and recoverable |
| Build/release integrity | Source, dependencies, CI actions and promoted artifacts are attributable and immutable under the release policy |

## 3. Actors

| Actor | Capability / assumption |
|---|---|
| Anonymous Internet client | Fully untrusted; may send malformed, duplicated, slow, oversized or replayed requests |
| Authenticated user | May be malicious or compromised; identity does not imply authorization to another resource/tenant |
| Browser script / extension / compromised client | All browser-originated data and WASM state remain untrusted input |
| Trusted TLS edge | Operated by the deployment; authenticates/limits the private hop and canonicalizes external request metadata |
| External IdP | Trusted only for claims validated under the configured issuer/client contract; availability and account policy remain external dependencies |
| Yetlet application | Trusted code boundary for request validation, Session lookup, CSRF and resource authorization |
| Session/data/storage providers | Trusted for configured durability, atomicity and access controls; outages and compromise remain modeled risks |
| Operator / administrator | Privileged actor requiring least privilege, strong authentication, separation of duties and audit |
| CI/release actor | May produce artifacts only through authorized immutable inputs and approval policy |
| Dependency or infrastructure attacker | May compromise an upstream crate, Action, container/base image, secret channel or deployment setting |

## 4. Reference production topology

```text
                         ┌─────────────────────────────┐
                         │ External OIDC IdP           │
                         │ issuer/JWKS/token endpoint  │
                         └──────────────▲──────────────┘
                                        │ server-side exchange/validation
User agent ── public HTTPS ──► TLS edge / reverse proxy / load balancer
                                        │
                                        │ private authenticated or allowlisted hop
                                        ▼
                         ┌─────────────────────────────┐
                         │ Yetlet application          │
                         │ Axum/Tower + app policy     │
                         └───┬────────┬────────┬───────┘
                             │        │        │
                 opaque Session   application  audit/logs/metrics
                    store          DB/object   (redacted/protected)
                                  storage
                             │
                      secret manager /
                      workload identity
```

### Required topology constraints

1. **No direct public application access.** The application listener is private, loopback, Unix-domain or otherwise network-restricted. A public route bypassing the trusted edge invalidates forwarded-header, TLS and coarse abuse-control assumptions.
2. **TLS terminates at a managed edge.** The edge owns certificates, supported protocol/cipher policy, public request limits and denial-of-service protections. The private hop uses network isolation plus mTLS or an equivalent authenticated/allowlisted channel appropriate to the platform.
3. **External identity.** A mature IdP owns account authentication, MFA/recovery and account lifecycle. The application validates the protocol and maps the stable external subject to local identity/authorization state.
4. **Server-side Session.** Browser credentials are opaque Session cookies. OAuth tokens and provider credentials remain server side.
5. **External operational providers.** Distributed Session, database/object storage, secret delivery and telemetry are deployment-selected explicit capabilities, not hidden Yetlet defaults.

## 5. Trust boundaries and data flows

### Flow A — ordinary public request

1. The edge accepts HTTPS, applies coarse connection/body/time/rate limits and rejects invalid protocol input.
2. The edge removes any client-supplied forwarding/client-IP headers, then writes one canonical configured set.
3. The private hop authenticates or restricts the edge as the only caller allowed to supply those assertions.
4. The application validates host/origin/method/media/body limits, loads Session state if the route opts in and performs route/resource authorization.
5. The response receives security headers and an explicit cache policy appropriate to public versus personalized content, then returns through the same edge.

**Fail-closed rule:** if the peer is not a configured trusted edge, the application ignores forwarding headers and protected production routes reject the deployment path. Duplicate, comma-list, malformed or inconsistent security-sensitive values are rejected rather than taking the first value.

### Flow B — login start and callback

1. Login start creates a short-lived, single-use server-side authentication transaction bound to the anonymous Session: random `state`, `nonce`, PKCE verifier/challenge where applicable, intended return target and timestamp.
2. The browser is redirected to one configured issuer using an exact registered redirect URI.
3. The callback validates single-value parameters, exact state binding, one-time use, expiry and expected issuer/flow before exchanging the authorization code server to server.
4. Returned tokens are verified for signature/key, issuer, audience, authorized party where applicable, nonce and time claims. Unknown/refreshing signing keys fail closed; keys are fetched only from configured issuer metadata over authenticated transport.
5. A stable issuer+subject (and tenant, when required) is mapped to a local account. Browser-supplied profile/role/tenant fields are not authorization facts.
6. The application rotates the Session ID and CSRF token, stores only the minimum required principal context and redirects to an allowlisted local target.

OAuth authorization codes, access tokens, ID tokens and refresh tokens never enter SSR state, query logs, analytics, browser storage or ordinary application logs. If refresh tokens are required, they are encrypted/protected server-side with explicit revocation/rotation and narrow scopes.

### Flow C — authenticated mutation

1. Session cookie lookup succeeds in the distributed provider.
2. Route/handler obtains the server-side principal.
3. CSRF validation runs before parsing or applying the protected mutation as appropriate to the body contract.
4. Authorization checks action plus resource/tenant ownership. Caller-supplied IDs or hidden form fields locate a target; they never prove ownership.
5. Validation, transaction/idempotency and audit occur without partial state.

Authentication answers “who”; Session continuity answers “which browser request carries that identity”; CSRF answers “was this credentialed browser mutation intentionally initiated”; authorization answers “may this principal perform this action on this resource.” No single flag substitutes for all four.

### Flow D — logout, privilege change and account disable

- Local logout validates CSRF, revokes/rotates the Session and removes privileged state.
- Upstream IdP logout is optional and must not be confused with local Session invalidation.
- Password/MFA/recovery changes at the IdP and local role/tenant changes require Session invalidation or re-evaluation under an explicit freshness policy.
- Account disable/revocation must propagate to all serving nodes within a defined bound. A cached Session must not grant indefinite access after disable.

### Flow E — files, SSE and expensive operations

- Upload/download routes require explicit authorization, content/size/time/concurrency limits, opaque server-owned paths and cancellation cleanup.
- SSE uses an authenticated/authorized subscription when data is private, bounded subscribers/queues/replay, heartbeat and reconnect policy. Cursor values are not authorization.
- Expensive queries/exports receive route-, account- and deployment-specific quotas independent of generic body limits.

## 6. Trusted edge and forwarding-header contract

The production deployment must choose exactly one canonical mechanism: platform-authenticated metadata, PROXY protocol from an authenticated hop, standardized `Forwarded`, or a documented narrow `X-Forwarded-*` set. Multiple ambiguous mechanisms must not be merged ad hoc.

Required edge behavior:

- strip inbound `Forwarded`, `X-Forwarded-For`, `X-Forwarded-Proto`, `X-Forwarded-Host`, `X-Forwarded-Port`, `X-Real-IP` and equivalent platform identity headers;
- set canonical values from the actual accepted connection and configured public origin;
- preserve a bounded request/correlation identifier separately from identity;
- reject invalid Host/SNI/public-origin combinations at the edge;
- prevent direct access to the private listener through firewall/network policy/security group or equivalent;
- protect health/admin/metrics endpoints separately from public application routes.

Required application behavior:

- configure an exact public base URL for OIDC redirects and security decisions; do not construct it from arbitrary request Host;
- trust canonical forwarding metadata only when the immediate peer is a configured trusted edge;
- require exactly one syntactically valid value for security-sensitive scheme/host/origin/client identity metadata;
- reject direct/untrusted use, duplicates, comma lists and inconsistencies;
- avoid using client IP as an authentication factor; it is an abuse-control/audit signal only;
- generate outbound redirects from allowlisted local paths or configured origins.

A future proxy-header adapter, if desired, is an explicit application/Tower integration with narrow tests. It is not required in Yetlet core.

### Private response caching contract

Authentication, Session and personalized responses cross the same edge, so cache ownership is part of the trust boundary:

- responses containing personalized bodies, authentication state, private authorization results or any `Set-Cookie` that creates, rotates, removes or refreshes identity use `Cache-Control: no-store`;
- shared edge/CDN caches are configured to bypass those routes and must never store or replay personalized bodies or `Set-Cookie` responses across users;
- `Vary: Cookie`, `Vary: Authorization` or `Vary: Origin` is added where the response actually varies, but `Vary` is a cache-selection guard and never substitutes for `no-store` on sensitive responses;
- personalized redirects and error pages follow the same `no-store` rule because they can disclose identity, tenant or workflow state;
- only explicitly public, identity-independent static assets or responses receive cacheable/immutable policy;
- cache keys and policy decisions use the configured public origin and trusted canonical metadata, never caller-supplied Host/forwarding values;
- the deployment proves cross-user isolation by requesting the same protected URL with distinct Sessions and verifying that the edge neither reuses a body nor replays a cookie/header from another response.

The bounded Session middleware already emits `Cache-Control: no-store` and `Vary: Cookie`; a production edge must preserve those headers. This requirement does not add a Yetlet cache subsystem.

## 7. Authentication protocol requirements

The reference flow is OIDC Authorization Code with a mature provider/client implementation.

| Control | Requirement |
|---|---|
| Issuer | Exact preconfigured issuer; no caller-controlled discovery URL |
| Redirect URI | Exact registered HTTPS URI generated from configured public base URL |
| State | At least cryptographically random, short-lived, single-use and bound to initiating Session/return path |
| Nonce | Random, stored server-side and verified in the ID token when used |
| PKCE | S256 verifier/challenge where supported/required; verifier kept server-side |
| Code exchange | Server-to-server, one-time, bounded timeout, no token/code logging |
| Token signature/key | Allowed algorithms only; configured issuer JWKS; bounded cache/refresh; unknown key fails closed |
| Claims | Verify issuer, audience, authorized party when applicable, expiration/not-before/issued-at policy and nonce |
| Subject mapping | Stable issuer+subject, plus explicit tenant mapping; email/display name are profile data, not immutable identity by default |
| Scopes | Minimum required; no broad offline access without a reviewed use case |
| Error handling | Generic browser error; detailed redacted correlation in protected operator logs |
| Return target | Local allowlist; no open redirect from arbitrary query/header input |
| Logout | Local Session revocation always; upstream logout/revocation capability documented separately |

Account passwords, passkeys, MFA enrollment and recovery remain IdP responsibilities unless a future application explicitly adopts and reviews those capabilities. This threat model does not create a Yetlet authentication crate.

## 8. Session and CSRF requirements

### Cookie and identifier

- opaque ID generated from a cryptographically secure source;
- production cookie is `Secure`, `HttpOnly`, `Path=/`, omits `Domain` where possible and preferably uses a `__Host-` name when deployment constraints permit;
- SameSite is selected from the actual cross-site/OIDC flow; `None` requires Secure and additional origin/CSRF review;
- bounded absolute lifetime, and idle lifetime if the product requires it;
- no Session ID in URL, HTML, browser-readable state or logs.

### Distributed store

Before multi-node production, the selected store must prove:

- atomic create/read/compare-update/rotate/destroy or an equivalent transaction/script contract;
- TTL/expiry consistency and clock assumptions;
- rotation that cannot lose a concurrent successful update;
- stale requests/responses cannot delete or overwrite a replacement identity;
- revocation/account-disable propagation bound;
- capacity, eviction and outage behavior;
- encryption/access controls, backup/restore/retention where Session state classification requires them;
- fail-closed behavior for protected routes on ambiguous/unavailable state.

The current in-memory provider demonstrates API semantics, rotation/CSRF and race tests only. Provider operations and distributed qualification remain YT-066.

### CSRF

Every credentialed state-changing browser route requires an explicit anti-CSRF strategy independent of SameSite:

- synchronizer token bound to Session, or an equivalently reviewed pattern;
- token checked before mutation and compared safely;
- unsafe methods only; no mutation by GET/HEAD;
- Origin/Fetch-Metadata checks may be additional signals, not sole protection unless separately justified;
- login and logout are included where credential/state impact exists;
- CORS configuration cannot silently make a cookie-authenticated API cross-origin writable.

## 9. Authorization requirements

- Protected route groups default deny when no valid principal exists.
- Each handler/service checks action and resource/tenant ownership server-side.
- User/tenant/role values from path, form, JSON, browser state or headers are untrusted selectors, not authorization assertions.
- Mass assignment is prevented through strict typed input and explicit writable-field allowlists.
- Admin/impersonation/support actions require stronger policy, reason/audit and preferably step-up authentication.
- 401 denotes absent/invalid authentication; 403 denotes known principal denied. Applications may use 404 to reduce resource-existence disclosure, but the policy must be consistent and tested.
- Authorization occurs again on every read/write/download/subscription; cached UI visibility or opaque IDs do not grant access.
- Database queries include the authorized tenant/owner boundary or verify it before returning/mutating data.
- Idempotency keys, optimistic versions and CSRF tokens do not grant authorization.

Applications may use ordinary functions, service methods or a selected external policy engine. Yetlet does not require a generic authorization DSL.

## 10. CORS, browser origin and redirect policy

Default production posture:

- one configured HTTPS public origin;
- no CORS headers;
- same-origin forms/API/islands;
- strict CSP and bounded external assets;
- cookies scoped to the required host/path;
- redirects restricted to local allowlisted destinations.

When a separate browser origin is required:

- exact scheme/host/port allowlist, never reflection of arbitrary Origin;
- no `*` with credentials;
- explicit methods, request headers and exposed headers;
- preflight and actual response policies match;
- `Vary: Origin` where dynamic allowlisting is used;
- credential mode, SameSite, CSRF and token storage reviewed together;
- SSE, WebSocket or future custom transport gets a distinct origin/reconnect/replay assessment.

## 11. Rate limiting and abuse resistance

Rate limiting is layered because the edge and application know different facts.

| Layer | Suggested keys / controls | Required behavior |
|---|---|---|
| TLS edge | Connection, source network/IP, request rate, body/time limit | Coarse protection before application allocation; bounded queues and timeouts |
| Login start/callback | Source signal, anonymous Session, issuer/client, account signal after safe identification | Limit state creation, callback failures and replay without disclosing account existence |
| Authenticated app | Account, Session, tenant, route/action | Shared counter for multi-node deployments or documented node-local limitation |
| Upload/export/search | Account/tenant plus bytes/concurrency/cost | Separate quotas; cancellation and cleanup; 413/429/503 semantics documented |
| SSE/streaming | Account/tenant plus active connections/subscriptions/replay | Bounded connections, queues and replay; disconnect slow consumers |
| Admin/security actions | Principal plus action/target | Stronger limits, step-up where required and audit |

Responses use bounded `Retry-After` where useful and do not reveal whether an account exists. Rate-limit state must itself be bounded and protected from cardinality attacks. No default limiter is added to Yetlet; applications compose mature platform/Tower capabilities and their own identity-aware keys.

## 12. Secrets and key lifecycle

### Secret inventory

At minimum, deployments classify:

- OIDC client credentials and refresh tokens if used;
- database, object-storage and Session-store credentials;
- Session encryption/signing material if a selected provider needs it;
- webhook/API credentials and telemetry exporters;
- TLS/private keys when not fully edge-managed;
- artifact signing/attestation keys (owned by release engineering, not the runtime app).

### Required controls

- external secret manager or workload identity; no committed secret, browser bundle, SSR state, command argument or ordinary log;
- least-privilege, environment-specific identity and network access;
- versioned rotation with defined overlap/cutover/rollback;
- startup and refresh failures fail closed for protected capabilities;
- memory/crash-dump/debug-endpoint exposure considered;
- emergency revocation runbook and ownership;
- developer fixtures are synthetic and never accepted by production configuration.

Yetlet provides no secret manager, vault client or key-rotation framework by default.

## 13. Logging, audit, metrics and privacy

### Never record in ordinary telemetry

- Cookie or Authorization header values;
- Session IDs, CSRF tokens, OAuth codes, access/refresh/ID tokens;
- passwords, passkeys, recovery secrets or client secrets;
- raw sensitive request bodies or uploaded contents;
- full database/storage credentials;
- unbounded user-controlled header/query/body values.

### Correlation and audit

Request IDs correlate events but are not identity. Security audit records should contain a stable local actor identifier, action, target identifier/type, result, timestamp, trusted request correlation, policy version and operator reason where applicable. They require access controls, retention/deletion policy and tamper resistance appropriate to risk.

Authentication failures use generic user-visible messages. Detailed reason categories are redacted and available only to protected operational/security telemetry. Metrics use bounded label cardinality; user IDs, paths with arbitrary IDs and error text are not metric labels.

## 14. Input, storage and rendering controls

Existing bounded evidence remains relevant but must be deployed with authorization and operations:

- HTML/title/attribute/state values remain escaped; no raw executable markup path is introduced;
- strict CSP remains a defense-in-depth policy, not a substitute for escaping;
- JSON/forms/multipart/path/query/header values use typed parsing, exact media types, unknown-field policy and body limits;
- SQL uses parameters, explicit transactions and authorization/tenant predicates;
- files use server-owned opaque paths, symlink/traversal defenses, safe disposition/type handling, size/time/concurrency limits and cancellation cleanup;
- browser/WASM state is schema-validated and untrusted;
- SSE IDs/cursors and idempotency/version fields control delivery/concurrency, not access.

Malware scanning, content moderation, encryption-at-rest, backup/restore and retention are application/provider decisions and remain production gates where data classification requires them.

## 15. Availability and failure policy

- Define edge and application deadlines; propagate cancellation to DB/storage/provider work.
- Bound request bodies, concurrent requests, uploads, downloads, streams, subscribers, replay, work queues and per-identity state.
- Drain on shutdown with a maximum grace period; reject new work before terminating active work.
- IdP outage: existing Sessions follow explicit freshness/expiry policy; new login fails closed with generic error.
- Session-store outage: protected routes fail closed; public/health behavior is explicit; no silent anonymous downgrade for mutations.
- Database/storage outage: transactions roll back; responses avoid internal detail; retry/idempotency policy prevents duplicate writes.
- Secret-manager outage: no fallback to embedded/default secrets; startup/refresh behavior is explicit.
- Telemetry outage: bounded buffering/drop policy must not block critical request paths indefinitely; security audit loss policy is explicit.
- Backup/restore, retention, recovery objectives, encryption and multi-node behavior are YT-066 operational qualification.

## 16. Threat register

Qualitative severity expresses potential impact under the reference topology, not measured likelihood.

| ID | Severity | Threat / abuse path | Required owner/control | Existing evidence | Remaining production gate |
|---|---|---|---|---|---|
| T01 | Critical | Direct public access bypasses TLS edge and trusted-header policy | Platform network boundary; private listener; authenticated/allowlisted hop | Loopback-only samples reject non-loopback listeners | Deployment/network verification |
| T02 | High | Forged/duplicate forwarding or Host headers alter redirects/origin/client identity | Edge strips/sets; app trusts only configured peer and exact single values | REST sample rejects duplicate/malformed Host/Origin | Select/test proxy contract |
| T03 | Critical | OIDC callback injection, state/nonce/code replay or issuer mix-up | External IdP client integration with exact issuer/redirect, state/nonce/PKCE and one-time transaction | Synthetic Auth sample only | Implement and independently review real OIDC adapter |
| T04 | High | Token/code/secret leakage through SSR, browser state or logs | Server-side exchange/storage, telemetry redaction and secret manager | Tests prove synthetic codes not in HTTP/logs/browser | Production token/secret inspection and runbook |
| T05 | Critical | Session fixation/theft, stale cross-node response or shared-cache replay exposes/retires the wrong identity or personalized body | Secure cookie, rotation/revocation, atomic distributed store, `no-store`/`Vary` and shared-cache bypass | In-memory rotation/stale-request races plus Session `no-store`/`Vary: Cookie` | YT-066 distributed provider proof and cross-user cache-isolation test |
| T06 | High | CSRF on credentialed mutation/login/logout | Session-bound token plus optional Origin/Fetch-Metadata; unsafe methods only | Cart/Auth/Shop CSRF and rotation tests | Apply to every production mutation and OIDC flow |
| T07 | Critical | IDOR, tenant crossing or role/mass-assignment escalation | Per-handler/service default-deny resource authorization and strict writable fields | Auth Notes and Modular Shop cross-user tests | Production policy/tenant/admin matrix |
| T08 | High | CORS misconfiguration exposes cookie-authenticated API | Same-origin default; exact allowlist; no wildcard credentials | Existing samples have no default CORS | Review any cross-origin deployment separately |
| T09 | High | Login/upload/search/SSE abuse exhausts resources | Layered shared rate limits, body/deadline/concurrency/queue bounds | Body/upload/SSE bounds and cleanup tests | Choose distributed limiter and capacity budgets |
| T10 | High | Slow/oversized upload or stream retains files/permits/tasks | Edge/app deadlines, cancellation and ownership cleanup | File interrupted upload and streaming cleanup tests | Load/soak and provider cancellation evidence |
| T11 | High | SQL/path/HTML/state injection | Parameterization, opaque paths, escaping, typed schemas and strict CSP | M9 negative tests | Fuzz/property and production data-flow review |
| T12 | High | Session/DB/secret/IdP outage causes insecure fallback or partial mutation | Fail closed, transactions, explicit public/health behavior, idempotency | SQLite rollback and Session fail-closed semantics | YT-066 outage/recovery exercises |
| T13 | Medium | Account enumeration or excessive error disclosure | Generic responses, protected redacted detail and bounded timing | Auth sample dummy comparison/generic failures | Real IdP/application error review |
| T14 | High | Admin/operator misuse lacks step-up or audit | Least privilege, strong operator auth, reason/audit and separation of duties | No production admin surface | Production admin design and audit backend |
| T15 | High | Dependency/build/deployment artifact compromise | Locked dependencies, RustSec, SHA-pinned Actions, SBOM/provenance/signing and immutable promotion | CI pins/audit/commit guard | YT-064 release supply-chain evidence |
| T16 | Medium | Missing/tampered audit evidence prevents investigation | Protected append/central audit, retention and correlation | Request IDs and bounded sample logs only | Audit backend, retention and incident drills |
| T17 | High | Compromised browser/WASM sends privileged fields/actions | Treat all client state as input; server authz/validation | Typed actions, schema checks, server authorization | Maintain per-route server policy |
| T18 | Medium | Open redirect/phishing through return URL or Host | Configured public base URL and local return-target allowlist | Some sample origin/Host validators | Real OIDC redirect tests |
| T19 | High | Key/JWKS/secret rotation fails open or causes indefinite stale trust | Bounded cache/overlap, allowed algorithms, fail-closed unknown key and emergency revoke | No production provider | Provider integration and rotation exercise |
| T20 | High | Backup/restore or replica lag crosses tenant/session security state | Encryption/access controls, tested restore, consistency/revocation policy | SQLite restart only | YT-066 operational qualification |

## 17. Abuse-case acceptance scenarios

A production candidate must automate or manually evidence at least these scenarios against its selected topology:

1. public connection to the private app listener is impossible;
2. caller-supplied forwarding headers are stripped or rejected; only the trusted edge changes canonical origin/client metadata;
3. duplicate/malformed Host, Origin, forwarding and callback parameters fail closed;
4. OIDC state/nonce/PKCE mismatch, expired/reused code, wrong issuer/audience/key and open return URL are rejected;
5. login/logout/privilege change rotates identity; old cookie/CSRF cannot authenticate or mutate; personalized and `Set-Cookie` responses are `no-store` and cannot be replayed across users by a shared cache;
6. Session-store race/outage cannot lose a successful update, revive a revoked identity or downgrade protected requests;
7. cross-user/cross-tenant read/write/download/SSE subscription is denied regardless of guessed IDs;
8. missing/invalid CSRF cannot partially mutate;
9. login, callback, upload, expensive query and SSE flood hit bounded rate/concurrency/queue policies without account enumeration;
10. slow/oversized/cancelled uploads and streams release permits/temp files/tasks;
11. hostile HTML/JSON/form/path/SQL/browser state is escaped/rejected/parameterized;
12. OAuth tokens, Session IDs/cookies and server secrets are absent from rendered HTML, browser state, normal logs and metrics; a Session-bound synchronizer CSRF token may appear only in the initiating user's protected `no-store` HTML/form and never in URLs, shared caches, logs, metrics or unrelated users' responses;
13. IdP, Session, DB/storage, secret manager and telemetry outages follow documented fail-closed/degraded behavior;
14. dependency/artifact compromise response can identify, revoke and replace affected builds;
15. audit evidence supports actor/action/target/result correlation without storing credentials.

## 18. Control ownership matrix

| Control | Yetlet framework | Application | Platform / external service | Status |
|---|---|---|---|---|
| HTML escaping / strict CSP defaults | Shared renderer/server adapter | Keep untrusted data typed | Edge may add stronger headers | Demonstrated |
| Request parsing/body limits | Native Axum/Tower composition | Select per route | Edge adds coarse limits | Demonstrated in samples; production values required |
| TLS and forwarding metadata | No default proxy subsystem | Validate configured origin/peer contract | Edge/network owns termination and canonical headers | Required, not implemented |
| OIDC authentication | No built-in IdP/client | Select mature client and map subject | External IdP owns account/MFA/recovery | Required, not implemented |
| Opaque Session/CSRF API | Optional in-memory semantics | Install only on selected routes | Distributed store required for multi-node | Prototype demonstrated; YT-066 required |
| Private response caching | Session middleware emits `no-store` and `Vary: Cookie` | Mark all personalized/authentication/`Set-Cookie` responses sensitive; test distinct Sessions | Edge/CDN bypasses sensitive routes and preserves cache headers | Prototype header evidence; production cross-user cache isolation required |
| Resource authorization | No generic policy DSL | Every protected handler/service | External policy engine optional | Demonstrated in bounded samples |
| Rate limiting | No default limiter | Identity/route keys and responses | Edge/shared limiter | Required, not implemented |
| CORS | No default CORS | Exact allowlist only when needed | Edge must not broaden silently | Same-origin demonstrated |
| Secrets/keys | No vault client | Reference only injected handles | Secret manager/workload identity | Required, not implemented |
| DB/files/SSE | No default provider | Explicit selected capability and authz | Storage/network operations | Bounded sample evidence; operations pending |
| Audit/logs/metrics | Request ID is correlation only | Redaction and event semantics | Protected sinks/retention | Partial, operations pending |
| Release integrity | No runtime concern | Version metadata | CI/release system, signing/provenance | CI partial; YT-064 pending |

## 19. Production readiness checklist

A specific application deployment is not production-qualified until all applicable items are evidenced:

- [ ] public base URL, TLS edge, private hop and direct-access prevention documented/tested;
- [ ] exact forwarding-header mechanism and trusted proxy identities configured/tested;
- [ ] real OIDC issuer/client/redirect/state/nonce/PKCE/token/key/subject mapping tests pass;
- [ ] Session store atomicity, expiry, rotation, revocation, outage and cross-node tests pass;
- [ ] personalized/authentication and every `Set-Cookie` response is `no-store`; shared caches bypass those routes and a two-Session cross-user replay test passes;
- [ ] every protected route has authentication, CSRF and resource authorization coverage as applicable;
- [ ] edge/application rate and resource budgets are defined, shared where required and tested;
- [ ] CORS remains absent or exact credential-aware allowlist tests pass;
- [ ] secret inventory, least privilege, rotation and emergency revocation are tested;
- [ ] logs/metrics/audit are redacted, bounded, protected and retained under policy;
- [ ] storage encryption, backup/restore, retention/deletion and recovery objectives are qualified;
- [ ] cancellation, draining, outage/degraded behavior, load and incident exercises pass;
- [ ] release artifacts have approved source/license policy, SBOM, checksums, provenance/signing and immutable promotion;
- [ ] the maintainer reviews complete internal security-assessment results and residual risk against the selected deployment; no external certification is claimed;
- [ ] YT-020 publication/release gates are explicitly approved.

## 20. Residual risk and follow-up ownership

- **YT-060 / YT-061:** dual licensing/security policy and private-state repository governance are complete; public-transition scanner/reporting activation remains separate.
- **YT-064:** release design and unsigned local preflight are complete; signing, immutable release inputs and independent reproducibility remain activation gates.
- **YT-065:** bounded automated desktop/native/browser qualification is complete; optional real-device/IME/assistive-technology/WebKit/mobile/zoom evidence and any support/conformance declaration remain independent of this threat model.
- **YT-066:** distributed Session/storage, backup/restore, load, cancellation, observability and recovery must prove the operational controls identified here.
- **YT-067:** the maintainer requires an internal Codex Security Standard source scan rather than a paid external assessment. The updated-source scan completed with a canonical report (227/227 regular files, one Low finding); its local corrective patch passed regressions and independent reviewer `wc_job_sTA8TdP3N3Veu9uC` (0 material findings). The earlier scan remains incomplete. YT-066 is independently necessary for any distributed-deployment claims.

This document is a requirements boundary. Passing its documentation review does not prove any external IdP, proxy, limiter, store, secret manager or production deployment meets it.
