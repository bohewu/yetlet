# YT-120 — Cookie primitives gap review

**Date:** 2026-10-22. **Status:** Done at research scope after independent Reviewer PASS.
**Decision:** Reuse existing Session providers and the Rust cookie crate. No
`yetlet-cookies` crate, ambient cookie jar or duplicate CSRF/store subsystem is
selected by the observed consumers.

## Concrete consumers and current implementations

| Inspected source | Actual need and behavior |
|---|---|
| [Auth Notes](../examples/auth-notes/src/lib.rs) and [YT-115 HTTP consumer](../crates/yetlet-auth/tests/session_http.rs) | Authenticated Session continuity, native hidden-form CSRF, rotation/logout and private responses; no independent app cookie API requirement |
| [Production Starter](../examples/production-starter/src/web.rs) and [Cart](../examples/cart-server/src/lib.rs) | Anonymous Session/CSRF and ordinary app data, demonstrating Session is useful without authentication; existing Memory provider already owns cookie operations |
| [Memory Session](../crates/yetlet/src/session/mod.rs), `session_cookie/session_middleware/apply_cookie` | Existing cookie crate parser/builder, rejects duplicate/malformed designated identity, opaque random ID, Secure-by-default/HttpOnly/Lax/Path=/, explicit local insecure choice, expiry/rotation/destruction and appended Set-Cookie/private headers |
| [Redis Session](../crates/yetlet/src/session_redis/mod.rs), `open/finalize/destroy` | Narrow configured-ID parsing, duplicate rejection, default __Host- naming with Secure and no Domain, distinct local HTTP name; authoritative finalize checks replacement state before emitting a cookie and preserves uncertain-operation semantics |

The second application use is concrete, but it repeats use of the same Session
provider rather than duplicating independent cookie-management glue. Neither
it nor the new Memory Auth bridge needs a second cookie store, jar, serializer,
signing layer or implicit identity extractor.

## Contracts to preserve

| Boundary | Required ownership |
|---|---|
| Parsing | Cookie values remain untrusted; duplicate designated Session cookies across headers fail closed. Cookie names/identities are compared exactly; malformed designated IDs cannot become trusted principal data |
| Provider differences | Memory parses cookie pairs using the mature cookie crate; Redis currently parses its narrow opaque-cookie contract. Their unrelated malformed-cookie handling is not declared identical, and this study does not silently rewrite either parser |
| Emission | Separate Set-Cookie headers are appended, not comma-combined or used to overwrite unrelated app cookies. Preserve Path/Domain/Secure/HttpOnly/SameSite and deletion identity scope |
| Rotation and expiry | Session store owns identity/CSRF replacement and revocation. A generic cookie helper cannot promise authoritative store expiry or cluster logout |
| Stale responses | Preserve existing provider protections against a stale request deleting or overwriting a replacement; unresolved Redis operations must not be replayed blindly |
| Browser security | SameSite is additional protection, not authentication/authorization/CSRF proof. Existing Lax/default-secure behavior is not a selected cross-site None policy; production TLS/domain/prefix selection is deployment-owned |
| Serialization and caching | No Session ID, credential or trusted Principal in SSR/hydration/logs; deliberate Session-bound CSRF form fields are protected no-store output. Personalized/Set-Cookie responses bypass shared caches |

The [cookie 0.18.1 documentation](https://docs.rs/cookie/0.18.1/cookie/), checked
on 2026-10-22, already supplies parsing/building and an explicit CookieJar with
optional signed/private facilities. Those options do not by themselves choose
application key rotation, revocation or multi-node storage. This study enables
no signing/encryption features, adds no key provider and writes no cryptography.
Applications with a genuine non-Session preference/consent cookie can use mature
native cookie primitives explicitly without making them Yetlet core defaults.

## Extraction threshold and decision

A separate module would be justified only by independently observed repeated
HTTP cookie glue that the existing provider/native primitives cannot express,
with a precisely selected duplicate-cookie, Set-Cookie, key lifecycle and
scope/deletion contract. Such evidence is absent here. Do not manufacture a
second consumer, broad provider registry or migration merely to fill a crate
name already listed as a candidate.

Keep Session-specific behavior where its authoritative state and response race
handling live. Keep optional Auth identity mapping separate, as demonstrated by
[YT-115](YT-115-SESSION-AUTH.md). No change to current provider API, Memory sync
semantics, Redis async semantics, production cookie support or browser WASM is
approved by this study. Any later cross-site/signed/private cookie capability
requires separately selected consumer and lifecycle/security evidence.

## Evidence and closure

Existing source and accepted Session qualification are used as evidence of
current behavior, without repeating them as fresh results. The new YT-115 tests
add scoped real-HTTP CSRF/login/logout/expiry/race and cookie-attribute proof;
the header attribute test is not a browser TLS/SameSite-enforcement test. No old
Windows/Redis/browser workload was rerun, no application runtime or manifest
changes occur for this item, and no new cookie module is implemented.

Independent Reviewer PASS is required before Done at research scope. Closeout
records documentation validation and the review result; future implementation
would need a separately selected gap and its own review.

Validation: `cargo xtask check docs` PASS (75 fixed required documents, 123
backlog items and English corpus); direct changed/new-file links, EOF and
whitespace checks PASS. Independent Reviewer `/root/auth_studies_review_20261022` PASS (2026-10-22),
zero issues and required followups for this item; all eight reviewed document
hashes matched before closeout. No runtime rerun.
