# YT-183 — Optional application catalog-data cache contract

Date: 2026-10-25 (Asia/Taipei). Status: **Contract selected before integration;
independent final Reviewer PASS, zero issues/followups; YT-183 Done at bounded
application-reference scope.**

## Authority and actual consumer

The maintainer selects actual-consumer/expiry/invalidation decisions before a
minimal opt-in integration, current authoritative authorization and Session,
no passing-proof replay, independent Reviewer before Done, and no new-change
Commit/Push. YT-182's 21 paths are committed as `074358d`, strict/builtin Guard
and hook PASS, zero findings, tree `04797ee6f1ed766e181ff5683d1101482dda6524`;
no Push. Initial Guard rejection of a synthetic foreign-span test field and its
single field-name correction remain in local commit receipts; no guard bypass
or passing runtime-test replay occurred.

[YT-121](YT-121-CACHE-STUDY.md) remains a historical no-extraction study. Current
source inspection selects **Modular Shop's existing product-display projection**:
[public catalog](../examples/modular-shop/src/catalog.rs) and authenticated
[cart display](../examples/modular-shop/src/cart.rs) both call
[Database::catalog](../examples/modular-shop/src/database.rs). Each direct read
opens a bounded-worker SQL connection and materializes the same ordered products.
The three seeded products make a small reference, not a throughput problem.
New focused origin-read-count evidence may establish reuse; no speed claim or
benchmark reopening is selected. These two handlers are one underlying consumer.

Do not cache Board/Admin: their list/aggregate transaction and authoritative SSE
snapshot/revision semantics should not be weakened to manufacture a cache use.
Starter list/detail remain current SQL. Shop cart mutations call `product` and
checkout rereads price/stock inside its IMMEDIATE transaction; these are excluded
from caching. Orders, ownership, durable idempotency, Session cart contents,
identity, CSRF, authorization and active rate budgets stay authoritative.

## Smallest opt-in and typed scope

Select only Shop application Cargo feature **`catalog-cache`**, default off,
optional native Moka future cache; no `yetlet-cache` crate, common trait,
registry, DI, AOP, cache middleware, output store or Core/browser API. Use the
existing typed Database/AppState lifetime. `Database::open` stays uncached;
an explicit `open_with_catalog_cache` constructs a selected private cache.
The standalone Shop uses **`YETLET_SHOP_CATALOG_CACHE_ENABLED`**, absent/`false`
by default, only exact `true`/`false`; true without feature and invalid/non-Unicode
selection return a fixed redacted startup error before listener startup. This
sample has native environment startup, not Admin's Settings loader; do not
invent a Settings-provider service or an undocumented second key.

Clones share one Store cache/generation; separate Database opens, even at the
same path, have independent cold caches. The implicit namespace is that Store,
this typed ordered product projection and its schema, with a local generation
as the only explicit key. No caller URL, cookie, user, tenant, Session, secret
or raw query enters keys/values. The projection is shared only because this
sample explicitly exposes all products publicly. A later private catalog or
visibility/tenant dimension requires a new contract and current scope checks;
this cache must not be reused unchanged.

## Freshness, admission and failure

| Choice | Selected reference contract |
| --- | --- |
| Data | Ordered id/name/price/stock projection only; no HTML/HTTP response, cart, order, identity or token |
| Maximum age | Two seconds since monotonic source-load start, checked at value selection; source wait is included, not a sliding idle lifetime or response-delivery bound |
| Cache TTL | Two-second native insertion TTL plus explicit source-age check; hits do not refresh either clock; expired hit invalidates and permits one fill attempt |
| Admission | At most 64 products and 64 KiB estimated typed payload per retained value; an oversized successful result is returned uncached, never truncated or inserted |
| Cache capacity | Native target one entry, best effort, not a hard peak-entry/RSS ceiling; caller clones, invalidated entries, in-flight data, source SQL and library metadata also use memory |
| Fills | Native fallible same-generation fill coalescing; one non-waiting fill permit per Store, owned by the real SQL closure once dispatched |
| Wait | 500 ms per enabled lookup/fill wait, no application retry loop or new fallback queue; timeout/capacity/source failure returns existing storage-unavailable result |
| Miss versus error | Miss may load current SQL once; errors/negative results are not retained. No stale-on-error fallback, TTL extension, cached authorization error or fabricated fresh value |
| Lifecycle | Process-local disposable state, cold after reopen. Existing four SQL workers and SQLite busy timeout remain; caller timeout/drop cannot release a started worker's cache/SQL permits early |

SQL work already started can outlive a 500 ms caller wait; the wait is not a
worker-abort or rollback promise. Native coalescing may elect a surviving waiter
if an initializer is cancelled; the actual one-fill permit still prevents a
second concurrent source worker. There is no new background refresher/exporter
or remote cache protocol. Library maintenance follows native semantics.

## Commit/invalidation and old fills

On successful checkout transaction commit, advance the Store's monotonic local
cache generation **inside the same blocking closure**, before its successful
return, so dropping the awaiting HTTP future cannot skip publication. The
idempotent committed-order return may conservatively advance it too. Rollback,
failed transaction or rejected checkout does not advertise a successful write.
Saturate generation without wrap; at saturation cache reads fail unavailable
until a cold reopen, never reuse an old generation. Checkout authority is not
changed by cache-generation exhaustion.

A request that selected its generation before publication may finish with that
bounded-age snapshot; no linearizable display-read or SQL-to-memory atomicity
claim. After publication new reads select the new generation. A late old fill
can only populate the old key and cannot become a hit for the new generation;
bare invalidate-after-commit alone would not establish this ordering. Logical
invalidation need not eagerly free every old allocation.

Separate opens/processes or trusted out-of-band SQL writers do not publish this
Store's generation. Their display changes become visible through the two-second
source-age bound; no cross-process notification, remote invalidation, HA or
immediate external read-after-write promise. Mutations and checkout always
reread authority regardless of cache state. Unknown checkout outcomes retain
existing idempotency/reconciliation rules; never blindly retry for cache errors.

## User-visible and security boundaries

When selected, catalog/cart explicitly describe displayed price/stock as
advisory and potentially two seconds old; cart total is an estimate. Checkout
uses current transactional price/stock. Render fresh pages/forms with each
request's current Session/CSRF; preserve existing private `no-store`, cookie,
CSP and output escaping. Public product data in memory does not make cart HTML
publicly cacheable. Auth/Session/CSRF failure must reject before a cache hit can
grant access. No tokens or cached UI state establish identity/ownership.

## Minimum new proof and completion

Use only new named focused tests, changed-source compile/lint/format/docs and
optional normal dependency inspection. Prove actual public/cart reuse and fewer
source reads, default-off absence, authoritative cart/checkout failure despite
a display hit, current cross-user Session/CSRF/order isolation, source-age
expiration without sleeps, rollback/commit generation distinction, late old-key
publication, independent Store isolation, oversize bypass, failure/no negative
cache, and cancellation retaining actual fill admission. Test-only native
capture/barriers are ordinary product fixtures, not new runtime hooks or a
verification framework. Join/drain owned work before TempDir cleanup. No old
Shop/Auth/provider/benchmark/browser/cloud/security matrix is rerun.

Final independent Reviewer covers this contract, minimum integration and new
evidence before Done. Keep YT-181/182 accepted scopes and release gates unchanged;
YT-184 stays Todo. A later framework extraction still needs measured benefit
and repeated useful glue across independent real consumers.

Mechanisms were checked against primary [Moka future Cache](https://docs.rs/moka/0.12.16/moka/future/struct.Cache.html)
(fallible coalescing, non-insertion of errors, native maintenance) and
[CacheBuilder](https://docs.rs/moka/0.12.16/moka/future/struct.CacheBuilder.html)
(TTL and best-effort capacity). The selected application freshness, generation,
privacy and authority rules are this reference's decisions, not library guarantees.


## Subsequent integration checkpoint

The [minimal Shop integration/new proof](YT-183-CATALOG-CACHE-IMPLEMENTATION.md)
now has nine successful new test executions, strict Clippy/format/default-graph
checks and retained offline-dependency failure handling. Existing nine Shop
cases were filtered; no prior qualification replay. Independent final Reviewer
`/root/yt183_catalog_cache_independent_review_20261025` now returns PASS, zero
issues/required followups; 20 file/41 evidence/16 retained bindings stable.
YT-183 is Done at bounded application-reference scope, with the implementation
record retaining exact limitations and separate current acceptance metadata.
