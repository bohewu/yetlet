# YT-124 — Multi-instance Auth and Valkey Session contract

**Date:** 2026-10-22. **Status:** Done at contract-research scope after independent Reviewer PASS.
**Source baseline:** `f2109514fba7397b6f3f264407d90fb78b100ede`.
**Authority:** Owner-selected documentation-only follow-up to YT-115. Runtime
implementation, new deployment execution and commit/push remain unselected.

## Decision and existing naming

The selected server for this study is **Valkey**, as already selected and
qualified for the Admin reference in [YT-094](YT-094-VALKEY-SESSIONS.md).
That record pins Valkey 9.1.2; this study makes no claim about the latest release.
The package `yetlet-session-redis`, its `RedisSession*` Rust types, key schema
and `redis://` endpoint scheme retain compatibility names. The Board aliases
the package as `yetlet-session-valkey`; its Cargo feature, backend setting and
typed configuration use `valkey`. A protocol-compatible name does not mean the
selected server is Redis. No package/API rename or server replacement is needed.

The new `yetlet-auth` foundation currently has only its synchronous
[Memory bridge](YT-115-SESSION-AUTH.md). The existing Board already composes an
async Valkey Session with its own identity model; it does not use that foundation.
These are different integrations. The missing work is a reviewed connection
between the native Auth resolver contract and the existing async provider,
not a new Session datastore or a first proof that Valkey can share sessions.

The recommended first implementation candidate is an **application-owned async
resolver** using the existing provider and `Resolver<S>`. A convenience helper
inside an explicitly enabled Auth feature can be considered only if that slice
demonstrates useful repeated glue. No new helper signature, feature name,
provider registry, crate or Core API is selected by this study. The owner decides
whether to implement the candidate after research review.

## Reused evidence and its exact limits

| Record | Existing accepted evidence | What it does not establish |
|---|---|---|
| [YT-066b3](YT-066B3-VERIFICATION.md#actual-observations) | Three app processes, Redis 8.2.10 single primary, one host, no persistence; CAS, rotation/revocation, uncertain writes, delayed cookies, bounds and shutdown | A Valkey execution, new Auth integration, durable revocation or physical-host HA |
| [YT-094 Valkey](YT-094-VALKEY-SESSIONS.md#initial-verification-topology-and-scope-60-second-fixture) | Two real Admin containers with Valkey 9.1.2; A login accepted on B, B logout rejected on both, app restart, pause/outage/recovery, no-JS browser, cleanup and independent Reviewer PASS | New `yetlet-auth` wiring, live account-policy changes, real OIDC callbacks across replicas, Valkey failover or persistence |
| [YT-094 lifetime correction](YT-094-VALKEY-SESSIONS.md#configurable-lifetime-follow-up) | 1800-second app default, 1-second–30-day app range; measured PTTL/cookie values, non-sliding reads/CAS, rotation/logout and separate Reviewer PASS | A 30-day soak or a repeat of the initial full browser matrix |
| [YT-094 shared SQLite](YT-094-SHARED-SQLITE.md) | Later optional shared local business-data volume, cross-app edits/conflicts, busy/restart/logout evidence | A distributed database, live policy authority, or an atomic transaction spanning Valkey and business storage |
| [YT-103](YT-103-ROLLING-ASSET-VERIFICATION.md#rolling-availability-and-draining-results) | Three app replicas and two gateway/static replicas across three VMs on one host; update/rollback/draining, 2364 successful measured SSR requests and retained old assets | Auth/Session continuity during rollout, multi-host failure tolerance or schema compatibility for mutable authenticated state |
| [YT-113](YT-113-AUTHENTICATION-CONTRACT.md) and [YT-115](YT-115-SESSION-AUTH.md) | Reviewed identity/extractor/freshness contract and real local Memory/Auth journeys | An async Valkey bridge or a cross-node commit guarantee |

Historical Redis execution stays labeled Redis. The later Valkey records add
their own source identities, topology and review results. None is rerun,
relabelled or expanded here. Existing YT-066/094/103 statuses, waived coverage,
the original immutable release candidate and YT-096/020 gates remain unchanged.

## Source boundaries

| Inspected source | Boundary retained by this contract |
|---|---|
| [Auth resolver/extractors](../crates/yetlet/src/auth/mod.rs), `Resolver`, `authenticate`, `Authenticated` | Native async Parts/state resolution once per installed scope; private resolved state; explicit required/optional outcomes; body untouched |
| [Current Memory bridge](../crates/yetlet/src/auth/session.rs), `memory` | Synchronous typed Memory Session read; no Valkey selection or async rewrite |
| [Remote provider](../crates/yetlet/src/session_redis/mod.rs), `open`, `read`, `replace_and_rotate`, `destroy`, `finalize`, `shutdown` | Cookie parsing is not authentication; reads are authoritative; mutation acknowledgment, cookie finalization and owned-work drain are separate boundaries |
| [Provider guide](../crates/yetlet/guides/session-redis.md) | Existing bounded single-primary semantics, standalone workspace, private cookies, uncertain outcomes and pure-SSR ordering limits |
| [Board Session composition](../examples/lab-operations-board/src/sessions.rs), `ensure_anonymous`, `remote_error` | Explicit login-page anonymous creation, async remote handles and no local fallback |
| [Board identity DTO](../examples/lab-operations-board/src/domain.rs), `Principal`, `LoginSession` | Serialized local identity name is reconstructed through a fixed per-process table; permissions are not trusted from stored objects |
| [Board authorization](../examples/lab-operations-board/src/lib.rs), `user`, `save` | Resource checks and Session reread before SQL work; this does not make separate stores one atomic transaction |
| [Board OIDC](../examples/lab-operations-board/src/oidc.rs) and [Admin OIDC setup](../examples/admin-dashboard/src/oidc.rs) | Process-local verified-identity mapping/configuration, shared pending-state serialization and atomic consumption; no new provider verification |
| [Admin settings](../examples/admin-dashboard/src/config.rs) and [Valkey fixture](../qualification/yt094-valkey/compose.yaml) | Explicit feature/backend choice, protected endpoint, deployment namespace and absolute lifetime; missing capability fails startup |

## Async bridge and trusted identity

1. Install the selected provider's existing middleware outside authentication,
   on explicit native routes. Each process explicitly connects its own manager
   to the same selected Valkey service and deployment namespace. Keep Memory
   synchronous and separately selected. Never infer a provider from an arbitrary
   extension or silently fall back between Memory and Valkey.
2. In the native resolver, obtain the exact typed `RedisSession<T>` extension
   installed for that scope and clone its handle before awaiting authoritative
   `read()`. Missing/misordered middleware is configuration failure, not anonymous
   access. Do not read or buffer the HTTP body for resolution.
3. Keep `T` an application-owned server-store DTO. The remote provider's explicit
   serde bounds apply to this DTO, not to generic `yetlet_auth::Principal` or
   Memory state. Store a stable local account reference and, where required, its
   verified issuer/subject/tenant binding. Never store credentials, trust browser
   role fields, or deserialize a generic Principal as an authorization grant.
4. Map the reference through authoritative application identity/account policy,
   allowing async lookup. Construct the exact issuer/subject/scheme/optional
   tenant only after that mapping succeeds. Keep resource authorization separate;
   a mapped identity or matching tenant alone is not permission.
5. Return the existing `Result<Option<Principal>, Response>` semantics. Only a
   successfully read, explicitly anonymous record yields `Ok(None)`. In this
   protected remote scope, Missing/Expired/Rotated/Revoked remains an error even
   for an optional extractor; missing credentials do not implicitly create state.
   Anonymous Session creation belongs only to the explicit native login flow.
   Truly public routes stay outside that scope. A different public optional-auth
   policy would need explicit consumer selection, not inferred fallback.
6. Keep Auth and Valkey dependencies opt-in and native-only. Unselected SSR apps
   and browser WASM gain no dependency, state initialization, lookup, task or
   middleware cost. No RequestContext field, DI container or service locator is
   introduced. Server identities, operation tickets, endpoints and credentials
   must not enter logs, generic hydration or uncontrolled output.

## Cross-instance login, logout and cookie ordering

Replicas sharing login continuity must use one logical authoritative namespace,
compatible DTO/schema, cookie scope/attributes and identity/permission rules.
Different deployments use isolated namespaces. Serving the same cookie name
does not establish shared authority. Sticky routing is not a correctness control.

The login handler verifies its credential proof and CSRF before replacing and
rotating the Session. It establishes signed-in continuity only after acknowledged
rotation and successful provider finalization. B then resolves the new identity
from Valkey without obtaining A's synthetic code or replaying A's login proof.
Do not replace OIDC/password verification with a local account name.

Logout requires CSRF and acknowledged `destroy()`. A subsequent authoritative
read on A or B must reject the retired identity. This is logout of that Session,
not every Session for the account, upstream IdP logout or durable global account
revocation. Already-running reads can hold a request-local snapshot; no instant
cancellation or partition-tolerant stale allow is promised.

Retain the [selected pure-SSR cookie contract](YT-066B-COOKIE-ORDERING.md):
ordinary reads do not renew or mint cookies; observed-stale finalization suppresses
Set; logout emits no Set/Clear. A previously prepared response can still overwrite
a newer browser cookie and require login again. It cannot recreate retired
server authority. Never forward an old ID, choose an alternate cookie, clear a
newer login from a stale response, or infer unconditional replacement retention
from the narrower Memory fixture. Malformed/duplicate designated cookies are 400.
Secure/HttpOnly/SameSite and no-store/Vary remain provider-owned. The local-HTTP
cookie override is not production TLS or browser-enforcement qualification.

## Permission changes and sensitive commits

Valkey Session validity and current business authorization are distinct. The
existing Board reconstructs permissions from its fixed local table and OIDC
mapping is a process snapshot. That is not evidence of live permission changes
propagating between instances; sharing Session data does not repair stale policy.

The minimum future Auth consumer should use a shared authoritative account/policy
lookup on each protected request, with no authorization cache in the first
slice. That authority must expose committed changes to subsequent decisions,
rather than serve an unbounded stale replica or process snapshot. Account
disable, unmapped identity, removed permission or wrong resource
must deny on every replica's next policy decision after the change is committed
to that authority. Unknown authority state or an outage cannot become an allow.
The authority implementation is application-owned and not selected here.
If a later owner chooses caching instead, its maximum age, version/invalidation
rule, affected replicas and failure behavior must be explicit before acceptance.

Sensitive writes need a stronger application contract than a request snapshot.
Recheck Session validity and account/resource permission at the business commit
boundary, as required by YT-113. A process-local mutex does not serialize another
instance, and a Valkey reread followed by a separate SQL commit still has a race.
The provider's CAS protects its own Session record, not business data elsewhere.
The first implementation must not claim that a pre-logout request can never
commit after logout acknowledgment merely because it performed that reread.

Before qualifying such writes, the application must select an equivalent atomic
fencing/authorization-version or transactional serialization rule shared by the
write and the relevant revocation/policy-change operation, and define its ordering
point. This is an acceptance gate, not a new distributed-lock service or a
second Session store requirement. Without that selected rule, the first bridge
slice can qualify identity/lifecycle and policy-decision behavior only; protected
business mutations and a strict write-after-revocation guarantee remain outside
its qualified scope. Streams/long jobs likewise need their own revalidation rule.

## Failure and recovery semantics

| Condition | Selected Auth integration behavior |
|---|---|
| Missing Auth/provider layer or mismatched configured scheme | Generic no-store 500; never anonymous |
| Valid explicitly anonymous remote record | Optional None; required scope's explicit failure response |
| Missing/expired/rotated/revoked record on protected access | Explicit unauthenticated response, never a replacement Session or fallback; 401 needs an applicable challenge, while a native form scope may explicitly choose a fixed local 303 |
| Verified reference with no allowed local account, disabled account, wrong tenant/resource or denied permission | 403, or an explicitly selected existence-hiding 404; no login redirect |
| Invalid designated cookie or invalid mutation input/CSRF | 400 malformed input/cookie; 403 for failed CSRF; no protected work |
| Store/policy outage, queue/deadline/closed/busy/capacity failure, invalid record/codec/protocol | Redacted no-store 503 for the selected Auth resolver; never anonymous, stale allow, Memory fallback or login navigation |
| Mutation CAS conflict | Explicit 409 without replaying the callback or minting a success cookie |
| Dispatched mutation has OutcomeUnknown, or its result has expired | No successful login/logout acknowledgment or business allow; safe unavailable response. Keep any reconciliation ticket server-owned; resolve historical status only, never automatically retry the mutation or treat a historical commit as current identity |
| Finalization cannot establish current identity | Existing provider suppresses observed-stale Set; finalization failure is 503, not signed-in success |
| App restart while Valkey state survives | Only unexpired, currently authorized state can continue; reconnect and DTO/policy compatibility must succeed |
| Non-persistent Valkey restart or unreachable/partitioned store | Outage is 503; after reachable state loss, old identities are unauthenticated. No restoration from local state |

Provider admission limits remain the existing explicit experimental bounds; this
study neither installs a new rate limiter nor reclassifies an unavailable lookup
as a valid anonymous request. Cancellation/timeout is not proof a dispatched
write was rolled back. Applications must drain the existing manager at shutdown;
no request-owned detached reconciliation task or new unbounded retry is added.

Absolute Session TTL stays explicit. The Admin's 1800-second default and
1-second–30-day settings range are existing app policy, not a universal Auth
lifetime. Reads/CAS do not slide expiry; rotation starts a new configured lifetime.
Restoring an old Valkey dataset must never be assumed to preserve logout or
permission revocation. Persistence, restore, HA, Sentinel/Cluster, cross-host
TLS/ACL and operational recovery are separate unqualified choices.

## Rolling replicas and OIDC boundary

Reuse YT-103 for its already executed rollout, draining and asset-retention
observations. Mutable Session/identity DTO and Auth failure semantics must also
remain compatible across versions; unsupported data fails closed. Rollback must
not resurrect retired identity or stale policy. A policy/configuration snapshot
can require coordinated application restart; do not claim zero-delay permission
revocation across mixed revisions from the asset rollout alone.

The existing stored OIDC transaction can be consumed atomically and uses a
cross-process-interpretable deadline. A future selected cross-replica callback
must retain exact issuer/subject mapping, state/nonce/PKCE, redirect binding,
single-use transaction consumption and maintained-library token verification.
YT-094 did not execute real-provider callbacks across Valkey replicas; that is
a separate future gate, not a reason to rerun its accepted single-provider proof
or implement an OIDC protocol in this study.

## Implementation decision and narrowly new acceptance work

After independent research PASS, the owner can select an application-owned
resolver slice or defer it. A generic convenience helper requires a demonstrated
need; adopting one must not make the existing provider a default dependency.
Before execution, name the consumer, shared account-policy authority, DTO and
namespace/cookie/configuration contract. Select commit coordination only if
business mutations are in that slice. No implementation follows automatically.

| Future check, only after implementation selection | Scope delta beyond retained evidence |
|---|---|
| Two actual processes with selected Valkey and new Auth resolver | A login resolves through `yetlet-auth` on B without sticky affinity; optional/required distinctions and unrelated route isolation |
| B logout, A stale request/cookie and post-logout lookup | Exercise the new Auth/error path with retired state; preserve pure-SSR ordering limits without rerunning the whole provider matrix |
| Account disable/permission change through the chosen shared authority | Next decisions on both replicas deny; authority outage is 503; no serialized permission or cached Principal becomes an allow |
| Paused sensitive write, if included | Selected cross-instance fencing/ordering proof; Session CAS or a local mutex alone does not pass this gate |
| Read outage, uncertain login/logout result and recovery | New integration fails closed; no false success or cookie, blind retry or local fallback; reuse existing provider reconciliation/bounds evidence |
| App restart or mixed-version transition, if included | New DTO/mapping compatibility and unchanged revocation; reuse YT-103 asset/transport proof |
| Dependency/target isolation and owner cleanup | Selected integration only; unchanged baseline/browser graph and explicit provider drain, without replaying unrelated product suites |

These are bounded future acceptance deltas, not permission to run them now.
No new harness, deployment, service, credential, protocol algorithm or datastore
implementation is created. Neither generic rate limiting nor other M21 research
is pulled into this item.

## Validation and independent review

This item changes documentation only. Existing YT-066b3, YT-094 Valkey/TTL/shared
SQLite and YT-103 results are retained at their original versions/source scopes;
no previously passing Rust, HTTP, browser, OIDC, Docker or K3s suite is rerun.
The earlier Auth security scan remains evidence only for its recorded six-file
scope. Documentation/link/whitespace checks and an independent Reviewer result
are recorded here at closeout. Research may be marked Done only after PASS;
no runtime or production qualification is implied.

Documentation validation: `cargo xtask check docs` PASS (75 fixed required
documents, 124 backlog items, English corpus, links/structure/whitespace and
dependency DAG). Direct links, fences, EOF and tracked/new-file whitespace
checks PASS. Independent Reviewer `/root/yt124_valkey_auth_contract_review_20261022`
returned PASS with zero issues, required followups and optional notes. Native
trace verified the reviewer role, `gpt-6-astra` model and effective `high` effort;
all six frozen document hashes matched through review completion. Only
status/review bookkeeping changed afterward. No runtime validation was executed.

## Subsequent owner-selected implementation

After this research passed independent review, the owner requested continuation.
[YT-125](YT-125-VALKEY-AUTH-INTEGRATION.md) selects the minimal standalone
application-owned async resolver and focused local two-process qualification,
with a fresh shared local policy file and no business-data mutation. The reviewed
contract and research acceptance above remain unchanged. No generic helper or
production deployment is selected; YT-125 has its own evidence and review gate.
