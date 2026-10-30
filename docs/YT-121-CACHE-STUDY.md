# YT-121 — Application data cache boundaries and extraction decision

**Date:** 2026-10-22. **Source:** `be2b105b12a25e58abfe250c19df26464bb795a2`.
**Status:** Done at research scope after independent Reviewer PASS, zero issues
and required followups.
**Decision:** Keep current reads and snapshots application-owned. No
`yetlet-cache` crate, cache-store trait, middleware or runtime is selected.

The owner selected documentation-only research under the
[M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md#yt-121--application-data-cache-provider-study-research-only).
Compare actual application read paths before choosing a common cache seam.
The inspected consumers below do not implement application data caching;
their possible cache uses are candidates, not evidence of duplicated adapters
or measured performance need. Research may close with a decision against
extraction. It does not expand the original nine release candidates.

## Observed consumers and authority

| Inspected consumer | Implemented behavior | Cache relevance and limits |
|---|---|---|
| [Production Starter store](../examples/production-starter/src/store.rs), `Store::{list,get,create,set_status}`; [handlers](../examples/production-starter/src/web.rs), `board`, `detail`, `api_list`, `api_get` | Each read opens a bounded blocking SQLite operation; list returns at most 100 current issues. Create and optimistic version-checked status changes use Immediate transactions and reread the written row | Real issue-list/detail read paths, with no cross-request cache, TTL, eviction or invalidation. DB path/semaphore are retained state, not cached rows. Operator-only/guarded APIs do not establish public data. A row version does not invalidate cached lists |
| [Modular Shop database](../examples/modular-shop/src/database.rs), `Database::{catalog,product,place_order}`; [catalog](../examples/modular-shop/src/catalog.rs), `page`; [cart](../examples/modular-shop/src/cart.rs), `page`, `page_view` | Catalog and product lookup query SQLite. Checkout rereads price/stock in its Immediate transaction, checks/decrements stock and persists an owner-scoped idempotent order with historical line snapshots | Independent product-display read path, with no cache or product revision/invalidation mechanism. The cart renderer's temporary lookup map lives for one render. Session cart contents, durable idempotency and historical order prices are state, not disposable data caches |
| [SQLite CRUD](../examples/sqlite-crud/src/lib.rs), `Database::{list,get,create,update,delete}` | Reads current rows with a bounded list limit; version-checked mutations and business-history insertion share the transaction | Supporting independent list/detail/delete case, not another cache implementation. Query limit belongs in any future list key. Database persistence and history retention remain store-owned |
| Admin reuses Board [database](../examples/lab-operations-board/src/database.rs), `Database::{list,get,snapshot,save}` | Reads current items and aggregate SQL each call; writes optimistic revisions and history together | The name `snapshot` means a result of a current SQL read, not a retained cache. Admin and Board share one lineage. Project filters, list dimensions and transactional aggregate consistency must survive any later read optimization |
| Admin [Vite snapshot](../examples/admin-dashboard/src/vite.rs), `Snapshot::{load,asset,cache_control}`; Board [embedded assets](../examples/lab-operations-board/src/asset_data.rs) and [asset handler](../examples/lab-operations-board/src/lib.rs), `asset` | Admin loads a bounded trusted build graph once and serves retained bytes without request filesystem reads. Board embeds a finite build-time allowlist | Actual byte reuse, with no dynamic data-cache admission or TTL/eviction. Admin deliberately serves `no-store`; Board's hashed public assets use HTTP `immutable`. An immutable in-memory lifetime alone does not establish safe HTTP caching |

Starter and Shop are the two primary application comparisons. SQLite CRUD,
Board and asset snapshots test their boundaries; none supplies a second
implemented data-cache adapter. The inspected application manifests do not
select a generic cache dependency for these paths. This is bounded source
inspection, not a claim about SQLite internals or every repository path.
Existing persistence/concurrency test definitions were read, not rerun.

## Responsibility boundaries

| Concern | Owner and meaning |
|---|---|
| Reconstructible application data | An app may explicitly cache a typed projection derived from an authority, with chosen freshness and privacy. A miss/eviction must not erase the authoritative record |
| Business storage and transactions | Application SQL/schema, optimistic concurrency, stock, orders, history, durable idempotency and deletion are authoritative. A cache cannot replace these correctness boundaries or become a homegrown ORM |
| Session, authorization and admission | Session identity/CSRF/rotation/revocation, current permission policy and active rate-limit budgets are authoritative state. Dropping an active entry can change security or quotas; their TTL/capacity settings do not make them generic cache consumers |
| Startup/request snapshots | Explicitly loaded assets/settings/OIDC clients and request-local Auth resolution have defined lifetimes. They are not evidence of a cross-request business-data cache or an automatic reload/JWKS refresh contract |
| HTTP output caching | Reuses status, headers and body under response-policy rules. A safe data value does not make rendered SSR/API output, cookies or CSRF reusable. [YT-122](YT-113-123-OPT-IN-CAPABILITIES.md#yt-122--private-safe-output-cache-policy-study-research-only) owns that separate study |

Follow [YT-095 composition](YT-095-COMPOSITION-RESEARCH.md): ordinary
constructors, explicit lifetimes and native Axum typed State/FromRef suffice.
An `Arc` shares one selected instance; it is not a DI container or provider
registry. A future app-owned cache may be constructed at startup and passed
to its consumers. No fixed Cache/User/Session field is added to RequestContext;
no settings service, cache dependency, network connection, worker or request
lookup is installed for applications that do not select one. Server cache
dependencies and private values must not enter browser WASM or hydration state.

## Conditions for any later application cache

These are selection questions and constraints, not a public Rust API, storage
schema or new runtime contract. No universal `CacheStore`, method naming,
sync/async wrapper or serialization format is chosen by this research.

| Dimension | Minimum application decision before implementation |
|---|---|
| Keys and isolation | Use a bounded canonical namespace for application/database, representation/schema version and resource or complete query dimensions. Include trusted tenant/user scope when data differs; authenticate and authorize independently. Do not key on raw URLs, untrusted forwarded headers, credentials, cookies, Session IDs or CSRF/OIDC/bearer material |
| Typed values and privacy | Select a minimal reconstructible projection with bounded size and classified sensitivity. Shared catalog data is different from user-specific cart/order detail. Do not cache a whole RequestContext, Principal, Session, raw error or secret. Key partitioning alone is not authorization or safe logging |
| Freshness and expiration | Choose maximum tolerated staleness per use. TTL since insertion/update and idle expiration are different; repeated hits must not silently extend a promised maximum age. Cache expiry is not database retention, record deletion or permission expiry |
| Capacity and fills | Bound key/value size, admission, entries/weight, concurrent source loads, wait/deadline and negative-entry lifetime. Coalesce equivalent fills where useful, with cancellation/error semantics. A count/weight target need not be a hard process-memory ceiling; values held by callers and in-flight fills also consume memory |
| Invalidation | Specify which committed writes affect item, list, query and aggregate entries. Failed transactions must not be presented as committed updates. Invalidation after commit still races with an older in-flight load that inserts afterward; choose versioned keys/generations or equivalent ordering if the selected consistency requires it |
| Failure | Distinguish a miss from unavailable/corrupt cache data and unavailable authoritative storage. Explicitly choose bounded bypass/fallback, load shedding and any negative caching. Do not cache arbitrary 401/403/503 errors or turn a source failure into a fresh value; stale-on-error is not a default |
| Lifecycle and IO | Select process-local versus network scope, typed serialization/version rejection if remote, timeouts, connection ownership and shutdown. Keep blocking work bounded/off the async executor. Native sync and async cache APIs need not be forced into a common trait; do not hold an application lock across awaited IO |

For Starter, a future cached list/detail needs a database namespace and coherent
write/list invalidation; the current row version is not a list generation.
For Shop, display freshness may be chosen separately from checkout correctness.
Checkout must retain its transactional authority read for current price/stock,
ownership and idempotency; a display cache cannot authorize spending or reserve
inventory. For SQLite CRUD, delete must not leave an entry indefinitely visible,
and differing query limits must not collide. None of these applications has
selected new TTLs, cache budgets, fallback or invalidation machinery here.

## Authentication freshness and multiple instances

Authorize the current caller and resource even on a data-cache hit. A resource
moving between projects/tenants or changing visibility requires a current
authority/policy decision; a previously authorized fill cannot grant access to
the next request. Do not infer public status from a catalog route name or a
successful authentication result.

The selected Board [Auth resolver](../examples/lab-operations-board/src/auth.rs)
rereads shared Session and trusted policy. Protected handlers and SSE repeat
current access checks; unavailable/malformed authority fails closed. Caching
positive permissions would add a revocation window and is not authorized.
[YT-126's existing pre-SQL ordering limit](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#existing-write-consistency-limit)
remains: authorization can precede queued/running SQL and concurrent revocation.
No cache TTL fixes that race or supplies a transaction across policy, Valkey
and SQLite. This study adds no cross-store fencing guarantee.

The existing [Redis/Valkey Session provider](../crates/yetlet/src/session_redis/mod.rs)
is an authoritative Session store, with lifecycle scripts, expiry and bounded
payload/IO. It is not a general data-cache API. Keep any future cache namespace,
value lifecycle and failure policy separate; eviction or outage must not restore
a revoked Session or reuse stale authorization. Selected Valkey deployments use
the existing compatible provider name; this study does not rename that package.

A process-local cache has independent copies on each instance. A shared Valkey
store alone does not invalidate those copies or prove coherent fills after
writes. Any later local-plus-shared design must explicitly handle notification
loss/disconnect/reconnect by safe bypass, invalidation or refresh under its
chosen consistency. No distributed data-cache protocol, client tracking
adapter, deployment qualification or HA guarantee is selected here.

## Data values versus output policy

The [threat model](THREAT-MODEL.md#private-response-caching-contract) already requires
private/personalized/authentication/Set-Cookie lifecycle responses, including
redirects and errors, to remain `no-store` with shared-edge/CDN bypass. `Vary`
does not replace that rule. Data caching cannot replay another user's body,
CSRF token, cookie or authorization result. Even an identity-independent value
can be rendered into private HTML; it does not transfer its cacheability to
the whole response.

[RFC 9111](https://www.rfc-editor.org/rfc/rfc9111.html), section 7.3, explicitly
does not make Set-Cookie a prohibition on HTTP caching by itself. Preserve
Yetlet's explicit restrictive policy; data TTLs do not set response freshness,
and a cache key is not access control. YT-122 must separately examine response
keys/status/headers/body, resource/policy changes and private/default bypass.
This study neither adds a response store nor claims YT-122 completion.

## Native options and extraction threshold

| Candidate | Fit and decision |
|---|---|
| Existing app-owned SQL and startup snapshots | Retain now. They meet inspected semantics; no performance issue or repeated cache glue has been demonstrated |
| App-selected native in-memory cache | [Moka](https://docs.rs/moka/latest/moka/) 0.12.16 offers sync and async-aware variants, expiration and entry/weighted capacity. Its bounding is best effort, not a hard peak-memory guarantee. The [future cache](https://docs.rs/moka/latest/moka/future/struct.Cache.html) supports coalesced fills; fallible-fill errors are not inserted. These mechanisms do not choose business consistency or invalidation |
| Optional shared Valkey through a mature client | Only if an actual application selects shared data caching and its network/serialization/failure contract. [Valkey client-side caching](https://valkey.io/topics/client-side-caching/) distinguishes local cached replies from invalidation tracking. No current Yetlet data-cache adapter or local-copy coherence is inferred from the Session backend |
| New `yetlet-cache` abstraction | Deferred. Multiple direct-SQL readers, immutable asset bytes and Session/limiter maps do not demonstrate the same replaceable cache-store glue |

Primary library/protocol documentation was checked on 2026-10-22. No dependency,
feature/version pin, implementation, benchmark comparison or upstream security
qualification is selected. The extraction decision is this study's inference
from repository consumers and documented mechanisms, not a library mandate.

Reconsider only when at least two independent real application cache uses
demonstrate repeated useful namespace/fill/invalidation/failure glue beyond
domain SQL. Identify the actual repeated functions, the replaceable resource
need and evidence that caching helps the chosen consumer. Select typed privacy,
maximum staleness, bounded capacity/IO and commit/fill ordering first. Prove
isolation, eviction, error/recovery and real consumer behavior in a separately
authorized minimal implementation with independent review. Do not manufacture
a second cache or impose a generic trait to satisfy the candidate's name.

## Evidence and completion gate

Source comparison and primary documentation research only. No runtime, schema,
dependency, feature, crate or Core API change; no old tests, browser/CSP,
Windows, Valkey/IdP, SQL workload, benchmark or security scan reruns. Private
settings and credentials were not read. No services were started. Historical
passing and failing evidence retains its original source and limits.

Independent Reviewer `/root/yt121_cache_study_independent_review_20261022` PASSed
with zero issues and required followups. It inspected the actual eight-document
patch, critical consumer/provider source and cited primary references. All eight
reviewed hashes, HEAD and index were unchanged during review; role, model and
effective effort were verified before acceptance. Only status/evidence
bookkeeping follows that result. `cargo xtask check docs` PASSed (75 required
documents, 126 backlog items), as did changed/new-file links, anchors, English,
fences, EOF and whitespace checks. No runtime validation was performed or
transferred from earlier work.

The repo-external `yetlet-yt121-cache-study-20261022-xxdmt4hl` directory retains
two source-inspection memos, `documentation-checks.json`, `review-source.json`,
`independent-review.json` and final `closure-checks.json`. All eight changed
paths are documentation: this study, Backlog, Roadmap, M21 plan, documentation
index, capability map, Extensions and ADR 0005. No source, manifest, lock, test
or runtime input changes.

YT-121 is Done at research scope. The next independent M21 study is YT-122. YT-123 remains Todo;
no automatic implementation or continuation is selected. YT-096 stays
In progress and YT-020 stays Blocked. HEAD remains `be2b105`, staged files zero,
no commit or push.
