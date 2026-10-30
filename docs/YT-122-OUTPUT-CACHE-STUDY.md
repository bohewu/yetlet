# YT-122 — Output cache policy and extraction decision

**Date:** 2026-10-22. **Source:** `5d6bb14cbad8d0c1203e78fa0ef9c7c0e969889e`.
**Status:** Done at research scope after independent Reviewer PASS, zero issues
and required followups.
**Decision:** Retain explicit application HTTP policy and existing static-asset
delivery. Private content stays uncached by default. No `yetlet-output-cache`
crate, response-store API, middleware or runtime is selected.

The owner selected documentation-only research under the
[M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md#yt-122--private-safe-output-cache-policy-study-research-only).
[YT-121](YT-121-CACHE-STUDY.md) separates reconstructible data values from
authoritative storage and complete HTTP responses. This study compares actual
public and personalized response consumers before selecting output caching;
header policy alone is not an implemented server-side response cache. No
benchmark need, new provider or release qualification is inferred.

## Observed consumers

| Consumer and source | Current response boundary | Reuse evidence and limit |
|---|---|---|
| Board [embedded assets](../examples/lab-operations-board/src/asset_data.rs), [asset lookup](../examples/lab-operations-board/src/assets.rs) and [handlers](../examples/lab-operations-board/src/lib.rs), `asset`, `security` | Exact public build-time bytes get `public, max-age=31536000, immutable`; misses use the global `no-store` default. The global Host check can reject before asset lookup. Asset lookup uses the route path, ignoring query | Finite identity-independent allowlist with SHA-256-generated filenames, not rendered-page memoization. Public asset routes remain outside Session scopes. No bespoke conditional/range/encoding cache logic is demonstrated |
| Admin [Vite adapter](../examples/admin-dashboard/src/vite.rs), `Snapshot::{load,asset,cache_control}` | Retains validated startup bytes; all selected `/static/` asset responses use `no-store`. Original URI lookup rejects query-bearing requests | At most 16 MiB/256 files, with per-file 2 MiB and graph bounds. No automatic reload, digest-authenticity or old-build retention contract. Reusing memory bytes is not permission to change HTTP policy |
| [Auth Notes](../examples/auth-notes/src/lib.rs), protected routes, native forms, `login`, `logout`; existing [Memory Session middleware](../crates/yetlet/src/session/mod.rs) | Anonymous and authenticated Sessions can affect form tokens, note ownership, redirects and cookies. Scoped Session responses emit `no-store` and append `Vary: Cookie`; login/logout rotate identity and CSRF | Anonymous is not necessarily public. SSR may deliberately contain a Session-bound CSRF token; storing/replaying that HTML across Sessions would violate the existing form contract |
| Admin/Board [composition](../examples/admin-dashboard/src/main.rs), [Auth](../examples/lab-operations-board/src/auth.rs), protected handlers and SSE | Current Session/account/project/write checks govern data and actions. Login/OIDC/logout and private HTML/errors stay under the existing restrictive scopes. SSE revalidates access repeatedly | Admin reuses Board, not a second output-cache implementation. Project-to-Auth tenant metadata is application glue, not a generic tenant registry. An output-cache hit must not skip those authority/lifecycle checks |
| [Modular Shop](../examples/modular-shop/src/lib.rs), [cart](../examples/modular-shop/src/cart.rs), [orders](../examples/modular-shop/src/orders.rs) | Session cart/identity, native form tokens and owner-scoped orders are private state. A catalog read does not make cart/order HTML public | Product data and complete response identity differ. Current authoritative checkout and idempotency remain transactional; no response replay substitutes for a mutation |
| [REST API](../examples/rest-api/src/lib.rs), scoped API middleware and book handlers; [File Server](../examples/file-server/src/lib.rs), response middleware | Anonymous stateful books remain `no-store` inside the API scope. File delivery also uses explicit `no-store`, with native Tower file handling | GET, absence of login and a file extension are insufficient cacheability evidence. File Server uses native HEAD/ranges/date conditions; Board/Admin asset handlers do not implement that conditional/range logic. The REST `/api/missing` fallback has no Cache-Control in its existing source test; do not claim universal header coverage or infer public cache permission from absence |

These are actual response-policy comparisons, not two duplicated generic
output-cache adapters. The inspected flows have no cross-request rendered
response store. Static immutable bytes and repeated `no-store` headers do not
justify a new cache abstraction. Existing test definitions were inspected as
source; no accepted runtime checks were rerun. Admin Vite routes precede the
Board fallback and do not acquire Board's Host/Session guards by that
composition; their selected static bytes remain identity-independent.

## Policy ownership and private default

| Owner | Responsibility |
|---|---|
| Application/domain | Classify the complete response, current authorization, tenant/project/owner scope, safe query semantics and committed data changes |
| Session/Auth | Preserve current identity, CSRF, cookie lifecycle, failure and revocation checks. Authentication does not make an output reusable or grant business authorization |
| Origin response middleware | Apply final status/header/body policy at the actual native Tower/Axum scope; no implicit caching or body collection in Core |
| Browser and shared edge/CDN | Respect origin restrictions and explicit private-route bypass. Operator-owned cache keys, purge/retention and cross-user verification remain deployment concerns |

The [threat model](THREAT-MODEL.md#private-response-caching-contract) remains
authoritative: personalized bodies, identity/authorization results and identity
Set-Cookie responses use `no-store`; shared caches bypass those routes.
Personalized redirects and errors follow the same rule. `private` or a user key
does not authorize a weaker policy. Unknown classification stays bypassed.

Apply this to anonymous Sessions as well as signed-in users: CSRF, cart contents,
pending OIDC transactions and checkout keys are not public merely because a
login has not occurred. Never store/replay cookies, native-form CSRF, OIDC
code/state/nonce, bearer credentials or hydration containing private state.
Existing explicit SSR placement of Session-bound CSRF remains valid; accidental
logging, generic hydration or shared output remains forbidden.

Current ownership/project filtering is not general multi-tenant isolation.
A future tenant-aware consumer must resolve trusted application scope and
current resource visibility before reuse; raw tenant headers, URL labels or an
authenticated flag do not establish that authority. Adding a tenant/user/Session
key alone does not solve revocation, changing permissions or per-request CSRF.
No private-output cache exception is selected here. Board maps projects to
Auth tenant metadata, but a project key would still conflate operator/viewer
capabilities and Session CSRF. Shop user-only keys would conflate distinct
Sessions/carts belonging to the same account. Both are concrete reasons to
retain bypass, not proposed private key designs.

## Header meanings and final response

The following protocol distinctions are from
[RFC 9111](https://www.rfc-editor.org/rfc/rfc9111.html#section-5.2.2): `no-store`
prohibits storage; `no-cache` permits storage but requires validation before
reuse; unqualified `private` excludes shared caches but allows private caches.
`max-age`/`s-maxage` bound freshness, not revocation or deletion. Its
[authenticated-request rule](https://www.rfc-editor.org/rfc/rfc9111.html#section-3.5)
has explicit shared-cache exceptions; Yetlet selects no such relaxation for
protected responses. [Set-Cookie alone](https://www.rfc-editor.org/rfc/rfc9111.html#section-7.3)
does not stop HTTP caching.

A later HTTP cache must honor applicable request directives and age/freshness
semantics rather than reset age on each hit. No freshness duration or validator
algorithm is selected here.

[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.5) defines
`Vary` in terms of response-selection inputs. Preserve existing Vary fields;
include Cookie, Authorization or Origin only where the response actually
depends on them, and negotiated encoding/language/media type where applicable.
`Vary` partitions selection, not permissions; it never replaces the existing
private `no-store` rule. A candidate unable to honor variation, including `*`,
must bypass rather than reuse the wrong representation.

Policy must describe the final response after all applicable middleware.
Session finalization can add Set-Cookie or replace a handler result with 503;
Auth can reject before a handler. A cache inspecting only the handler's initial
200 is insufficient. Memory and Valkey finalize differently: Memory applies
pending cookies on returned statuses; Valkey finalizes only 2xx/3xx, and a
successful revoke does not currently emit a removal cookie. Do not infer
identical cookie lifecycle from a common header policy. The selected
[triage path](../examples/lab-operations-board/src/triage.rs) can also bind a
per-response style nonce to body/CSP. Do not remove Set-Cookie from a personalized response and
declare its remaining body public, or overwrite restrictive Cache-Control with
a permissive outer default. Preserve the actual scopes, including redirects,
errors, method fallback and global path fallback; one route layer need not
cover an unmatched path. No new middleware ordering is implemented here.

An already rendered private page remains a client snapshot; `no-store` does not
erase it. A newly restrictive origin header also cannot reach an existing
shared entry that is served without contacting origin. A later public-to-private
transition therefore needs operator-owned purge/bypass across every serving
path before private data is exposed, not a promise that logout or TTL alone
clears earlier copies.

## Conditions for a later public output cache

These are research selection constraints, not a new public Rust API or chosen
cache policy configuration. Keep native handlers/State/Extensions and existing
[HTTP pipeline semantics](HTTP-PIPELINE.md#scope-and-execution-order).

| Question | Minimum selection boundary |
|---|---|
| Eligible response | Start with explicitly public, identity-independent finite GET/HEAD success responses. Bypass protected/Session-dependent dynamic routes, credentialed dynamic requests, mutations, login/logout/OIDC, health/readiness, errors/redirects and uncertain cases. A deliberately public static route is a separate proven case even when a browser sends irrelevant cookies; it must not acquire Session effects |
| Key identity | Trusted configured public origin plus application/representation generation, exact route/resource, complete supported query and negotiated dimensions. Do not trust caller Host/forwarding or raw tenant metadata. Reject/bypass ambiguous or unsupported queries; never silently drop a query component that changes output. Bound cardinality and avoid secrets/raw request dumps |
| Response envelope | Preserve compatible status, content type/encoding, security policy and selected variation together with exact bytes. Request IDs, correlation, live admission/accounting and other per-request effects must run for the current request rather than replaying an earlier response's metadata |
| HTTP methods and validation | HEAD has no response content; validators, conditional 304 and ranges have distinct semantics under [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-13). Use a mature native implementation or bypass unsupported cases. Do not let a partial/HEAD/304 result replace a complete GET value or bypass current access checks |
| Body and resource limits | Select bounded complete bodies, size/entry/total-weight/fill-concurrency limits and deadlines. SSE, streaming SSR, unknown/incomplete bodies, trailers and failed streams stay outside the initial candidate. Do not globally buffer or clone streaming bodies to make them cacheable |
| Failure | A miss/outage must not replay private/stale authority or hide readiness/failure. Define bounded origin fallback/load admission; store failure cannot alter authorization or commit semantics. No stale-on-error/background reuse is selected for private content |

The [pipeline](HTTP-PIPELINE.md) after-processing point is response return, not
body completion; later body failure cannot be repaired by rewriting sent
headers. Merely observing a 200 or advertising a length does not prove a
complete reusable body. This is a reason to keep initial selection narrow,
not authorization to introduce a generic response collector.

## Invalidation, assets and multiple instances

For any future public dynamic output, the app must choose maximum freshness,
write/deletion/visibility dependencies and commit-aware invalidation of detail,
list and aggregate variants. Row version alone does not identify a rendered
list, template or permission policy. As [YT-121](YT-121-CACHE-STUDY.md) notes,
post-commit deletion races with older in-flight fills; a selected generation
or ordering strategy must address that before claiming consistency.

Process-local stores do not share invalidation. A shared Valkey store does not
automatically purge local copies, browser caches or CDN entries. Lost messages,
restart, mixed releases and backend failure require an explicit application and
operator strategy; no shared output provider, tracking adapter or deployment
protocol is selected. Existing Session Valkey storage remains authoritative
Session state, not response storage or an invalidation bus.

Session expiry/rotation/logout, local account disable and project/permission
changes cannot rely on an output TTL. Keep protected responses bypassed and
current authority checks intact. The OIDC callback is a side-effecting GET
that consumes pending state; its method does not authorize caching. The [YT-126 pre-SQL race](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#existing-write-consistency-limit)
remains unchanged: no cache policy adds fencing or atomicity across Session,
policy and SQL. Existing upstream OIDC client/configuration snapshots also
retain their documented freshness limits.

[RFC 8246](https://www.rfc-editor.org/rfc/rfc8246.html#section-2) describes
`immutable` as representation stability during freshness, not retention or
privacy. Board's finite hashed bytes and Admin's Vite-looking names are
different evidence. Preserve the latter's [asset contract](ASSETS.md#cache-and-retention)
and `no-store` policy. The accepted [YT-103 local proof](YT-103-ROLLING-ASSET-VERIFICATION.md)
retained complete old/new public asset graphs through rollout and rollback
using an independent static service, also with `no-store`; it did not qualify
CDN behavior or add old-build retention to default images. Cached HTML/offline
clients would extend the retention problem beyond that bounded proof. No prior
PASS, failure history or deployment scope is rewritten or rerun here.

## Extraction decision and completion gate

Retain app-owned header functions, native Tower/Axum scopes and existing static
delivery. There is no observed duplicated safe rendered-response store to
extract. Reconsider `yetlet-output-cache` only when independent real public
consumers demonstrate the same useful bounded response/key/invalidation glue
and a reason to reuse complete output. Select explicit body/header eligibility,
variation, freshness, failure and invalidation first; separately authorize a
minimal implementation and consumer adoption with negative isolation/lifecycle
evidence and independent review. A public asset handler and private Session
headers are not that pair of generic cache consumers.

No default provider, Core/context field, DI container, feature/dependency,
runtime, crate or browser/WASM coupling is added. Primary HTTP references were
checked on 2026-10-22; their mechanisms inform this study's application-owned
selection, not a new Yetlet HTTP cache implementation or production guarantee.
Runtime, browser/CSP, Windows, Valkey/IdP, edge/CDN, benchmark and security scans
were not rerun. No private settings/credentials were read or services started.

Independent Reviewer `/root/yt122_output_cache_study_independent_review_20261022`
PASSed with zero issues and required followups. It inspected the actual
eight-document patch, critical consumer/provider source and targeted primary
RFC text. All eight reviewed hashes, HEAD and index were unchanged during
review; role, model and effective effort were verified before acceptance. Only
status/evidence bookkeeping follows that result. `cargo xtask check docs`
PASSed (75 required documents, 126 backlog items), as did changed/new-file links,
anchors, English, fences, EOF and whitespace checks. No runtime validation was
performed or transferred from earlier work.

The repo-external `yetlet-yt122-output-cache-study-20261022-u8iqgley` directory
retains two source memos, `primary-references.json`, `documentation-checks.json`,
`review-source.json`, `independent-review.json` and final `closure-checks.json`.
All eight changed files are documentation: this study, Backlog, Roadmap, M21
plan, documentation index, capability map, Extensions and ADR 0005. No source,
manifest, lock, test or runtime input changes.

YT-122 is Done at research scope. The next independent M21 study is YT-123. YT-096 stays
In progress and YT-020 stays Blocked; original release scope is unchanged.
HEAD remains `5d6bb14`, staged files zero, no commit or push. No automatic
implementation or next-item continuation is selected.
