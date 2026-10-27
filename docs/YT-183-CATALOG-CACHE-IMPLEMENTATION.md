# YT-183 — Optional Shop display-cache integration and new proof

Date: 2026-10-25 (Asia/Taipei). Status: **Independent Reviewer PASS, zero issues/required
followups; YT-183 Done at bounded application-reference scope.**

## Prior commit, authority and consumer decision

The maintainer authorizes YT-182's 21-path strict/builtin commit, then actual
consumer/expiry/invalidation decisions before conditional minimal opt-in data
caching, authoritative Auth/Session and independent PASS before Done. No old
passing-proof replay or new-change Commit/Push.

Prior commit `074358d583ad15cc5c1e8795ebf7d5215ac3578e`, parent `a10db2b`, tree
`04797ee6f1ed766e181ff5683d1101482dda6524`, contains exactly 21 paths; explicit
staged strict/builtin Guard and hook PASS, zero findings, post-commit tree clean,
no Push. Initial Guard exit 1 on a synthetic foreign-span test field is retained;
only that field name changed, preserving its sentinel/assertions. No scanner/hook
weakening or runtime-test replay. Commit receipts remain in
`target/yt182-tracing-commit-20261025/`; original YT-182 review/proof remains
historical, with the mechanical test-field delta explicitly recorded.

[The contract](YT-183-CATALOG-CACHE-CONTRACT.md) was written and docs-checked
before runtime source changes; all 23 inspected source/proof bindings still
matched at that point. Choose existing Shop catalog/cart product display as
**one** actual consumer, not two independent adapters. Repeated reads previously
opened SQL for the same ordered public projection. The three-product fixture
supports origin-read reduction proof, not a measured latency/throughput problem.
Existing Board list/SSE consistency and Starter SQL remain unchanged. YT-121's
historical no-extraction result is preserved; no new framework cache crate,
trait, registry, cache middleware, generic AOP or output cache is introduced.

## Minimal application integration and policy

Shop feature `catalog-cache` adds optional native `moka 0.12.16` future support
and Tokio time, entirely server-side. Ordinary `Database::open` keeps cache off
even when compiled; `open_with_catalog_cache` explicitly selects an instance.
Its clones share private typed cache/generation through existing State/Database.
No external caller API or existing constructor is removed. Standalone startup
reads exactly `YETLET_SHOP_CATALOG_CACHE_ENABLED`, absent/`false` off; only exact
`true`/`false`, and true requires the feature. Invalid/non-Unicode input returns
a fixed redacted startup error. Shop uses native startup environment; Admin's
Settings loader and framework configuration remain unchanged.

The selected public projection is ordered product id/name/price/stock only,
keyed by a local generation in a Store-specific typed cache. Public catalog
and authenticated cart displays share it; protected handlers still read current
Session/CSRF and render new private pages. Enabled UI describes advisory price/
stock and estimated cart totals. Product mutation checks, checkout SQL,
Session/CSRF/cart state, order ownership and idempotency are never cached.

| Policy | Implemented behavior |
| --- | --- |
| Source-age bound | Two seconds from monotonic load start, including waiting; checked on hit and after fill. Native insertion TTL is also two seconds, without idle extension. This is selection-time freshness, not complete response-delivery freshness |
| Admission | At most 64 products and estimated 64 KiB typed payload; over-budget successful value returned uncached, without truncation or a retained error/value |
| Capacity | Native target one entry, best effort. Old generations, caller clones, in-flight values, unbounded original source rows and library metadata prevent a hard entry/RSS claim |
| Fills/wait | Native fallible same-key coalescing, one `try_acquire_owned` fill permit across generations, cooperative 500 ms lookup/fill wait, no application retry/extra fallback queue |
| Worker ownership | Fill permit moves into the real existing blocking SQL closure, alongside its existing SQL permit; timeout/drop before dispatch releases it, after dispatch retains it until worker completion |
| Errors | Existing storage-unavailable result, no negative/authorization-error cache or stale-on-error retry. Healthy age-valid hits do not probe source availability; a miss cannot convert failure into fresh data |
| Invalidation | Successful checkout/idempotent-order transaction publishes a new local generation in that blocking closure before returning. A failed/rolled-back transaction does not publish. New generations cannot select a late old-key fill |
| Exhaustion | Generation saturates without wrap; cached display reads fail until cold reopen, checkout authority remains functional |
| Other instances | Separate opens/processes and external writers do not share generation publication; only the two-second source-age bound applies, without remote invalidation/coherence/HA |

A reader that selected a generation before publication may finish using that
bounded-age snapshot. SQL commit and generation publication are not one atomic
cross-store event. Started SQL can finish after caller cancellation; cache error
or missing response never establishes rollback or permits a blind checkout retry.
Existing four SQL workers/two-second SQLite busy timeout are unchanged. Selected
Moka internal maintenance follows native behavior, with no app refresher/service.

## Actual changed paths

Changes are limited to root `Cargo.lock`; Shop manifest/README, library module
registration, database/catalog/cart/main, existing test-module registration and
new private `catalog_cache.rs`, `catalog_cache/tests.rs`,
`tests/catalog_cache_tests.rs`; Backlog, Roadmap, docs index, current capability
map, Extensions, application-follow-up assessment, contract and this record.
Session/Auth/Core, existing Shop identity/orders handlers and migration, other
applications and all prior test bodies remain unchanged.

The root lock adds only `moka 0.12.16`, `crossbeam-channel 0.5.17`,
`crossbeam-epoch 0.9.21`, `crossbeam-utils 0.8.23`, `tagptr 0.2.0` and
`uuid 1.29.0`; no old package/version/source tuple is removed or upgraded.
Default normal Shop dependency tree has no Moka. No browser feature edge or
mandatory framework dependency is added. The single library cache instance
and public projection do not establish a replaceable cross-application adapter.

## New verification and preserved operational result

Receipts are in `target/yt183-catalog-cache-20261025/evidence/`: each command
retains native exit, stdout/stderr. Initial `resolve-check-shop` exit 101 is an
**operational missing-offline-dependency result**, before compilation/test
execution. One bounded online Cargo resolution fetched the selected packages;
`resolve-check-shop-online` exits 0. Subsequent commands are offline/locked.
That early draft compile warned about the atomic API and unused then-unwritten
test helpers. Saturating native compare-exchange plus completed focused cases
resolve those warnings; no global install, harness redesign or model recovery.

| New check | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-modular-shop --features catalog-cache --all-targets catalog_cache_ -- --test-threads=1` | 7 new library cases + 1 new startup case pass, zero failures. Nine old library cases filtered, no historical test replay |
| New startup case with feature absent, binary-only filter | 1 passes, proving distinct default-off/absent-feature rejection; nine total successful new test executions |
| `cargo clippy --offline --locked -p yetlet-modular-shop --features catalog-cache --all-targets -- -D warnings` | PASS, changed library/binary/tests compiled; tests not rerun |
| `cargo fmt -p yetlet-modular-shop -- --check` | PASS |
| `cargo tree --offline --locked -p yetlet-modular-shop --edges normal` | PASS; Moka absent from default normal graph |
| Lock tuple comparison | Six additions, zero removals/upgrades |
| Docs/whitespace | Contract-first docs PASS (75 required documents, 184 items); final integration/acceptance checks recorded separately |

The seven new library cases cover:

- Source-age expiry independently of native insertion age, non-sliding hits,
  separate stores and cold reopen; expiry uses aged private fixtures without
  waiting out or replaying historical qualification.
- Commit/rejected-checkout/true SQL-rollback distinctions, generation exhaustion
  without blocking checkout, and deterministic late insertion of an old-key
  value after commit. This models the cache-publication race; it is not a claim
  of reproducing every possible real threaded write/fill interleaving.
- Over-budget bypass without truncation, source failure without negative cache,
  and recovery on the next real load.
- Two overlapping same-generation requests sharing one source initializer,
  bounded SQL-admission wait, pre-dispatch timeout cleanup and later recovery.
- Cancellation after an actual catalog SQL worker starts; the test-only bounded
  channel barrier holds that worker, verifies the fill permit still blocks a
  second source worker, then releases/drains it. The barrier/counter/gate access
  compile only in tests; no production runtime hook or generic harness exists.
- Actual catalog plus two users' protected cart requests reuse one origin load
  while preserving current Session/CSRF, logout/old-cookie refusal, distinct
  user/form state, CSP and private no-store. These counts establish reduced
  source invocations, not speedup, saturation or benchmark qualification.
- With an age-valid display hit, external stock/price changes still drive
  authoritative add/checkout rejection/current pricing. A committed checkout
  invalidates local display and order ownership still rejects the other user.

No old Shop/Auth/Session/Valkey/IdP/browser/CSP/Windows/cloud/benchmark matrix or
security scan is rerun. In-process HTTP assertions are new focused evidence,
not browser acceptance. Native tests use private TempDir SQLite files and join/
drain started work before cleanup. No live listeners, child services, containers
or Azure resources are created. Production/HA and a performance win remain
unqualified; large source materialization retains the old source's memory limits,
regardless of the cache's separate admission budget.

## Independent acceptance and next item

Independent Reviewer `/root/yt183_catalog_cache_independent_review_20261025`
returns PASS, zero issues/required followups. All 20 changed-file, 41 evidence
and 16 retained-binding hashes match before/after review, with HEAD/tree/branch/
full status stable and index empty. All 21 tracked preintegration bindings match
the committed baseline. Native role/model/effective effort match the registered
profile; shared resolver enforced without degradation/conflict. YT-183 is Done
at bounded application-reference scope. After acceptance only eight docs status/
acceptance records change; runtime/test/README/manifest/lock bytes stay reviewed.
Final docs/whitespace checks add no runtime replay. YT-181/182 remain Done at their reviewed reference scopes, YT-184 stays
Todo/unactivated, YT-096 In progress with D1–D6 Pending and YT-020 Blocked. New
changes remain unstaged/uncommitted/unpushed. Framework extraction still requires
multiple independent useful consumers and separate evidence/authorization.
