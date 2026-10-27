# YT-116 — OIDC adapter contract and extraction decision

**Date:** 2026-10-22. **Status:** Done at research scope after independent Reviewer PASS.
**Decision:** Retain the existing application-owned OIDC integration. No generic
Yetlet OIDC adapter, new dependency or consumer migration is selected.

The owner selected this study after [YT-115](YT-115-SESSION-AUTH.md). Its
completion is a research/direct-composition decision, not a claim that OIDC is
implemented by `yetlet-auth` or that all production authentication is qualified.
Any later extracted adapter requires a separately selected implementation and
real-provider evidence. The earlier [YT-094 accepted evidence](YT-094-OIDC.md)
retains its original scope, failures and owner waivers.

## Actual consumer and reuse assessment

| Source | Observed ownership |
|---|---|
| [Board OIDC](../examples/lab-operations-board/src/oidc.rs), `discover/start/finish/callback` | One configured issuer, maintained openidconnect client, callback transaction and local permission mapping |
| [Admin composition](../examples/admin-dashboard/src/oidc.rs), `configure` | Resolves SecretProvider at startup and builds that same Board client; not a second independent protocol implementation |
| [Board Session](../examples/lab-operations-board/src/sessions.rs) and [domain](../examples/lab-operations-board/src/domain.rs) | App-local Memory/Valkey selection and local-name permission reconstruction; existing store DTO is not the new generic Principal |
| [Auth foundation](../crates/yetlet-auth/src/lib.rs) and [Memory bridge](../crates/yetlet-auth/src/session.rs) | Verified-input Principal plus explicit resolver/extractors; no discovery, callback, token exchange, keys or account store |

Admin and Board already share the mature client implementation. Auth Notes is
a synthetic credential demonstration, not another OIDC consumer. Thus another
adapter crate would currently move configuration, callback and account policy
away from their one consumer without proving reduced duplicated protocol glue.
Keep ordinary Axum/Tower composition and injected State/SecretProvider; do not
create an all-provider registry, automatic provisioning, Controller or DI stack.

## Preserved protocol and trust contract

`openidconnect` 4.0.1 remains the selected client in the existing application.
The [primary client documentation](https://docs.rs/openidconnect/latest/openidconnect/)
(consulted 2026-10-22) supplies typed OIDC and async HTTP integration; its redirect
warning is consistent with the sample's HTTPS-only, no-redirect backchannel.
Protocol verification and cryptography stay in maintained libraries.

| Boundary | Existing behavior and requirement for any later reuse |
|---|---|
| Configuration/network | Explicit issuer/client/callback and startup-secret injection. Existing sample restricts callback to its loopback HTTP path; this is not a generic production proxy/TLS configuration API |
| Login initiation | Native CSRF-protected POST, Session rotation, random single-use state/nonce and S256 PKCE, five-minute pending transaction |
| Callback | Atomically consume transaction before exchange; reject expired/mismatched/error/replayed transaction; check returned issuer when supplied |
| ID Token | Client verifier checks signature, exact issuer/audience/time/nonce; verify access-token hash when present; never treat unverified decoded claims as identity |
| Account mapping | Exact verified issuer+subject selects an explicitly existing local identity. Email, display name or provider roles neither provision accounts nor grant permissions |
| Output | Protected native form CSRF is allowed; no code/state/token/nonce/verifier/credential in generic errors, normal logs or hydration. Callback stays no-referrer/no-store |
| Authorization/logout | Resource/project/write policy and CSRF remain app-owned. Local logout revokes local Session continuity; it does not terminate IdP SSO |

Existing CSP permits only the discovered authorization origin on the selected
login page's form-action. No blanket external-script/form origin or WASM
dependency is introduced. Configured issuer, callback and backchannel destinations
are trusted startup input, not arbitrary request URLs.

## Contract for a possible later Auth integration

The current Board stores a local Principal/name after callback. A future bridge
must preserve the verified issuer+subject+scheme reference, or a protected
server-owned binding to it; it cannot invent an issuer from a local username.
Convert only after verification and authoritative local mapping, keeping project
and write policy outside generic Principal. The [YT-113 contract](YT-113-AUTHENTICATION-CONTRACT.md)
defines separate tenant checks, 401/403/503 or explicit native login navigation,
fail-closed unknown state and request-local propagation.

Discovery/JWKS/client credentials and subject maps are startup snapshots in the
accepted consumer. Changes need restart; local Session lifetime is not ID Token
lifetime. No automatic key refresh, introspection, upstream logout, IdP account
disable propagation or cluster revocation is claimed. Before migration, select
the local reauthentication/authorization freshness bound, explicit provider-error
mapping and sensitive-write recheck; an old code-review PASS alone cannot prove
those new guarantees.

No dependency changes occur in this study. Preserve the existing
[advisory-applicability decision](YT-094-OIDC.md#dependency-audit-applicability--2026-10-21),
including the bounded RSA verification-only exception; revisit it before changing
libraries, algorithms, private-key use or consumer behavior. Do not substitute
homegrown OIDC, JWT cryptography, passwords, MFA or application WAF controls.

## Completion and evidence boundary

Source comparison and the decision above complete the selected study after
independent Reviewer PASS. No adapter is selected, so there is no adapter
implementation gate being silently waived or marked Done. If a distinct real
consumer later demonstrates duplicated configuration/callback glue, select one
bounded extraction, negative/security tests, real-provider verification and a
separate implementation review before claiming it implemented.

The earlier Linux real-provider/no-JS workflow and its accepted Windows waiver
remain in YT-094; neither is rerun or relabeled here. This item changes only
documentation, adds no manifest/runtime API, and does not reopen release gates.
Documentation validation and independent review are recorded at closeout.

Validation: `cargo xtask check docs` PASS (75 fixed required documents, 123
backlog items and English corpus); direct changed/new-file links, EOF and
whitespace checks PASS. Independent Reviewer `/root/auth_studies_review_20261022` PASS (2026-10-22),
zero issues and required followups for this item; all eight reviewed document
hashes matched before closeout. No runtime rerun.
