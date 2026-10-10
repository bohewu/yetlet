# M10d Plan — Production Edge and Authentication Threat Model

**Date:** 2026-10-10
**Task:** YT-063 / M10d
**Status:** Planning accepted by independent security-focused reviewer `wc_job__wg1bHF6DG3ztjjg`; final threat model may proceed.
**Baseline:** YT-062 locally Done; YT-060 and YT-061 Pending by maintainer decision.

## 1. Purpose

M10d defines the **responsibility and trust boundaries** required to deploy an application built with Yetlet. It does not implement authentication, reverse proxying, rate limiting, secret storage or a production Session provider. Those remain explicit application/platform capabilities selected from mature external systems.

The document must answer:

- which components are inside and outside the Yetlet process;
- which network and identity assertions may be trusted;
- where authentication ends and authorization begins;
- which attacks are handled at the TLS edge, application middleware, handler/service or storage boundary;
- which bounded prototype controls are already evidenced;
- which controls remain mandatory before production use.

## 2. Reference deployment boundary

The threat model uses one intentionally small reference topology:

```text
User agent
    │ public HTTPS
    ▼
TLS edge / reverse proxy / load balancer
    │ private authenticated or allowlisted hop
    ▼
Yetlet application (Axum/Tower)
    ├── external OIDC identity provider
    ├── server-side distributed Session store
    ├── application database/object storage
    ├── secret manager / workload identity
    └── audit, logs and metrics sinks
```

Direct Internet access to the application listener is not part of the production model. Local loopback-only samples remain development fixtures.

The model deliberately does not prescribe one cloud, proxy, IdP, database or secret manager. It specifies observable contracts that any selected product must satisfy.

## 3. Trust boundaries to model

### 3.1 Public TLS edge

The edge owns TLS certificates, protocol policy, request-size ceilings, connection/time limits, coarse denial-of-service controls and canonical external origin information. It must strip caller-supplied forwarding headers before setting one canonical set. The application trusts those headers only from configured proxy identities/networks and rejects direct/untrusted use.

The model must state:

- no arbitrary `Host`, `Forwarded`, `X-Forwarded-*` or client-IP header is trusted by default;
- the externally visible scheme/host/base URL is configured or derived only from a trusted edge contract;
- duplicate, comma-list or malformed security-sensitive headers fail closed;
- the edge-to-application hop is private and protected by network policy, mTLS or equivalent deployment control;
- health/admin endpoints have separate exposure policy.

This does not authorize a built-in Yetlet proxy-header subsystem. A deployment may use narrow Tower middleware or platform integration chosen by the application.

### 3.2 Authentication and identity provider

A production application should use a mature external IdP through OIDC Authorization Code flow with PKCE where applicable. The server must own callback validation and exchange; browser-visible data must not become an authorization source.

The threat model must cover:

- exact redirect URI and configured issuer;
- `state` and `nonce` binding;
- PKCE where supported/required;
- signature, issuer, audience/authorized-party and time validation;
- JWKS cache/rotation and fail-closed key lookup;
- login CSRF, code replay and mix-up defenses;
- server-side subject/tenant mapping;
- logout and upstream-session limitations;
- MFA/account recovery remain IdP responsibilities unless an application explicitly owns them;
- access/refresh/ID tokens are never SSR-rendered, stored in browser-readable state or logged.

Yetlet must not grow a default IdP, password database, passkey server or authentication framework from this task.

### 3.3 Session and CSRF

The application converts validated identity into an opaque server-side Session. Production qualification requires a durable/distributed provider with atomic rotation/revocation semantics; the existing in-memory provider is evidence for the API shape only.

Required properties:

- high-entropy opaque IDs; Secure, HttpOnly and bounded cookie scope;
- login/logout/privilege-change rotation;
- absolute and, if selected, idle expiry;
- atomic update/rotate/destroy behavior across nodes;
- explicit revocation and account-disable behavior;
- Session records contain the minimum identity/authorization context;
- credentialed state-changing requests require CSRF protection independent of SameSite;
- stale responses cannot retire replacement identity;
- Session/store outage fails closed for protected routes.

The final document must distinguish authentication, Session continuity, CSRF and authorization instead of merging them into one middleware flag.

### 3.4 Authorization and application data

Authorization remains server-side and resource-specific. Every protected handler/service derives a principal from validated Session state, then checks action and resource ownership. Route visibility and guessed identifiers never grant access.

The model must include:

- default deny for protected routes;
- clear 401 versus 403 versus 404 disclosure policy;
- IDOR and mass-assignment defenses;
- tenant/account boundary checks on every read/write;
- privileged/admin actions use separate policy and stronger audit;
- optimistic concurrency/idempotency do not replace authorization;
- browser/WASM state is untrusted input.

No generic authorization DSL or policy engine is introduced by M10d.

### 3.5 Rate limiting and abuse controls

Rate limiting is layered and opt-in:

- edge-level connection/request and coarse IP/network limits;
- application-level account, Session and route limits where identity/context is known;
- stricter policies for login start/callback, code validation, uploads, mutations, expensive searches and SSE subscriptions;
- bounded request bodies, timeouts, queues and concurrency remain independent controls;
- responses expose bounded retry guidance without revealing account existence;
- distributed deployments use a shared limiter or accept/document node-local limitations.

M10d does not add a Yetlet rate-limit crate. Future implementation should compose mature Tower/platform middleware and application-specific keys.

### 3.6 CORS, origins and browser boundaries

The safe default remains same-origin with no CORS. If a separate browser origin is required:

- explicit exact origin allowlist;
- no wildcard with credentials;
- narrow methods/headers and correct preflight behavior;
- credentials and CSRF policy evaluated together;
- WebSocket/SSE/custom transports receive a separate origin/auth/reconnect review;
- CSP, cookie scope and redirect targets remain aligned with the public origin.

### 3.7 Secrets and key lifecycle

Secrets belong in an external secret manager or workload identity, not source, browser bundles, SSR state, command arguments or normal logs. The model must identify OIDC client credentials, database/storage credentials, Session/provider keys and telemetry credentials, with:

- least-privilege access;
- versioned rotation and overlap windows;
- startup/refresh failure behavior;
- redaction and crash-dump considerations;
- separation between environments;
- emergency revocation procedure.

Yetlet does not implement a secret manager or key vault.

### 3.8 Logging, audit and privacy

Request IDs are correlation values, never principals. Security telemetry must exclude cookies, Authorization headers, OAuth codes/tokens, CSRF tokens, secrets and sensitive request bodies. Audit records should capture actor, action, target, result and correlation with bounded retention and tamper controls.

The final model must separate operational logs, security audit and metrics, and address account enumeration/error disclosure.

### 3.9 Availability and operational safety

The model must carry forward bounded body sizes, upload cleanup, streaming/SSE cancellation, DB transactions and backpressure evidence, while defining production requirements for:

- end-to-end deadlines and cancellation;
- connection/concurrency limits;
- graceful shutdown/draining;
- distributed Session/storage outages;
- backup/restore and retention ownership;
- security incident isolation and rollback.

Provider implementation/operations remain YT-066; M10d identifies the security requirements without claiming they are qualified.

## 4. Threat-analysis method

The final document will use a compact asset/actor/data-flow inventory and a risk table mapped to familiar threat classes:

- spoofing/credential theft;
- tampering/request smuggling/header confusion;
- repudiation/audit gaps;
- information disclosure;
- denial of service/resource exhaustion;
- elevation of privilege/authorization bypass;
- software supply-chain and deployment misconfiguration.

Each risk records: affected flow, attack, required control owner, existing evidence, remaining gap and production gate. Risk labels are qualitative (`Critical`, `High`, `Medium`, `Low`) and must not imply measured probability.

## 5. Required abuse cases

At minimum, the final model covers:

1. direct application access bypassing the trusted edge;
2. forged/duplicate forwarding and Host headers;
3. OIDC state/nonce/code replay, mix-up and redirect abuse;
4. Session fixation, theft, stale response and cross-node race;
5. CSRF and malicious CORS configuration;
6. IDOR, tenant crossing and mass assignment;
7. login brute force/account enumeration;
8. oversized/slow upload and request-body exhaustion;
9. SSE subscriber/replay exhaustion;
10. SQL/path/HTML/browser-state injection;
11. secret/token leakage through logs or browser state;
12. compromised dependency/build/deployment artifact;
13. outage of IdP, Session store, database or secret manager;
14. insufficient audit/retention/incident response.

## 6. Deliverables

After planning reviewer PASS:

- `docs/THREAT-MODEL.md` — canonical reference topology, assets, actors, flows, trust boundaries, risk register, required controls and residual gaps;
- `docs/M10D-VERIFICATION.md` — evidence mapping, consistency checks, reviewer history and explicit non-claims;
- updates to Architecture, Release Readiness, CI/Security Review, README, Backlog and Roadmap;
- documentation registration in `scripts/check_docs.py`.

No Rust/Cargo/workflow/external-setting change is expected. If the analysis discovers a runtime defect, stop and create a separately reviewed implementation task rather than hiding code inside the threat-model slice.

## 7. Acceptance and reviewer gates

Planning gate:

```bash
python3 scripts/check_docs.py
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
git diff --check
```

The independent planning reviewer must verify completeness, responsibility ownership, tiny-framework boundaries and lack of production-security overclaim.

Final gate:

- every demonstrated control links to retained evidence;
- every production requirement is clearly `Required`, `Deferred to YT-066`, `Pending maintainer/external decision` or `Out of Yetlet scope`;
- the topology and header/OIDC/Session/CSRF/CORS/rate-limit/secrets/authorization contracts are internally consistent;
- YT-020 remains Blocked;
- YT-064–YT-067 status is unchanged;
- independent security-focused read-only reviewer PASS.

No commit or push is authorized by this task.
