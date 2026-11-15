# Provider Operations — Local Evidence and Open Qualification

**Task:** YT-066 / M10g. **Status:** Done at single-host Docker multi-instance and selected Redis-provider scope.
Application operations are not framework blockers; the [final documentation review gate](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11) is recorded separately. This document follows the approved [remaining plan](M10-REMAINING-PLAN.md)
and [ADR 0005](adr/0005-opt-in-capability-modules.md). It introduces no runtime
provider API, store implementation or default dependency.

## Memory and selected experimental Redis topology

`yetlet-session` owns an in-memory map within each `SessionManager`; that Memory
crate has no external-store integration or pluggable distributed-store contract. The
`auth-notes` sample creates a separate manager in each process. Identical
synthetic Alice/Bob login codes do not make Session records shared. Restart
discards the map, and a second worker cannot authenticate the first worker's
cookie. Sticky routing alone cannot provide durable Session recovery.

The experimental Redis Session work had **three distinct evidence stages**:
the YT-066b1 private `redis-rs` spike established bounded atomic/Lua CAS,
caller-cancellation and owned-worker drain semantics, but did not ship a
framework provider and originally left the Cookie gate unsatisfied.
YT-066b2 added a separate opt-in async provider; **two early real-Redis
test attempts stalled on a synchronous CAS callback barrier** and were
explicitly terminated, **not** counted as overall PASS. Only a separately
authorized blocking-aware test-barrier correction and focused validation
closed that failure. The retained [b2 provider evidence](PROVIDER-OPERATIONS.md#bounded-provider-bursts-and-http-attribution)
records **CSRF**, once-only CAS callbacks and scoped native SSR middleware
in its focused integration tests. The separate
[Cookie decision](PROVIDER-OPERATIONS.md#redis-session-cas-operation-outcome-and-cookie-ordering) preserves pure-SSR
ordering, possible late-response login loss and stale/retired ID rejection.

The later independently reviewed [b3 three-application run](QUALIFICATION-MATRIX.md#historical-session-auth-oidc-and-cross-host-evidence)
qualified a selected **single-host/single-primary** Redis topology with
`save ""`, `appendonly no`, tmpfs and **no persistent volume**. It exercised
expiry, rotation/revocation, CAS, outage and application restart at
its recorded source. The b3 three-app matrix did **not** retest CSRF;
that proof belongs to the retained b2 provider record. A dispatched write
may finish after a caller timeout,
so failure outcomes can be unknown until separately reconciled; permits and
server work must remain owned through cancellation/drain.

Proposed clean AOF restart or fenced fresh-generation recovery were
**not executed** and did not become YT-066 completion prerequisites.
Applications/deployments choose persistence, backup restore, TLS/ACL,
RPO/RTO and operational acceptance; this lab proves neither durable
writes, multi-host HA nor automatic recovery. The complete original
b1 design/failed attempts and the YT-066 responsibility-plan rationale
remain recoverable at fixed **local Git**
`4ad0978f8a70b36c307f5e538aa1a3aa496a7403` (public SHA
reachability not yet verified). Memory Session was not changed.
The SQL-first `sqlite-crud` sample uses a local SQLite WAL database with explicit
transactions, optimistic versions and transactional audit rows. Two local
processes opening that same file exercise local filesystem locking, not nodes,
replicas, network storage or a cluster. SQLite documents that WAL requires
processes on the same host and does not work over a network filesystem because
of its shared-memory coordination. See [SQLite WAL](https://sqlite.org/wal.html).

Existing [M8c Session evidence](M8C-VERIFICATION.md) retains atomic update,
rotation, CSRF and revocation regressions, including stale response cookie
protection. Existing [M9d file evidence](SAMPLES.md#file-server-acceptance) and
[M9f SSE evidence](SAMPLES.md#sse-notifications-acceptance) retain cancellation/backpressure tests.
They are earlier local evidence, not newly executed distributed tests.

## Bounded provider bursts and HTTP attribution

**The experimental Redis provider did not pass on its first two real
test runs:** both hung at a synchronous CAS callback barrier and
were stopped with owned process/Compose cleanup. The subsequent
owner-approved **test-only** blocking-aware barrier correction
preserved once-only CAS behavior; five real provider integration
tests then passed, covering typed lifecycle, Cookie/CSRF, rotation,
revocation, native SSR middleware and unknown-outcome reconciliation.
Earlier cancellation/drain and Cookie/browser proofs were **retained
but not rerun**. Later single-host three-app testing disabled
persistence; these outcomes do not qualify production HA, restore,
cross-host operations, durable RPO/RTO or safe mutation retries.

A separate provider-only burst experiment recorded **648 attempts
(552 valid reads, 96 Busy)**, 72 finite waves, four active/eight
total admission, 80ms queue-wait and 400ms caller budgets, plus
eight successful sequential recovery reads. Barrier waves of
12 or 16 rejected excess attempts, while nominally spaced 1ms
waves had zero Busy; actual scheduling was neither exactly 1ms
nor synchronized. These results establish burst-sensitive
fail-fast behavior but not fairness, expiry at the configured
80ms bound, or a production capacity limit.

A later same-process HTTP/Session trace observed **12 SSR Busy/503
rejections**, with exactly four active and four pending owners
covering each rejected interval; they stopped before SQL.
Different 16-VU HTTP JSON/SSR workloads without Busy had
different actual arrivals, so the studies do not contradict.
Client clock offsets, connection setup and instrumentation
costs limit attribution. The coincidence of eleven errors with
a Valkey resource-sample interval is only a possible observer
effect, not confirmed unique causation or permit-release timing.
No SQL pool enlargement or automatic capacity policy is selected.

These are finite source/host-specific observations. Complete old
failure attempts, wave populations, reviewer and cleanup records
remain at **local Git** `c3478b7f4e0c929a94ccbf21e05217535553a73d` (unverified remote reachability).
See [Qualification Matrix]
(QUALIFICATION-MATRIX.md#early-provider-and-http-attribution-evidence)
and [HTTP Pipeline]
(HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing).
## Outer HTTP admission and downstream provider pressure

Application-owned HTTP and Session admission bound **different units**.
The outer cap applies to scoped HTTP request futures through
Session finalization; the provider separately permits **four
active/eight total** Session operations, with 80ms pending and
400ms caller budgets. Neither queue nor count is distributed.
More HTTP slots do not grant extra Session/SQL workers, and
HTTP cancellation cannot release already dispatched backend
work or undo uncertain mutations.

In three nonrandomized isolated source-bound cap **4/8/16** cohorts,
attempts were **12,550/12,549/12,552** and valid 200s
**12,452/12,475/12,540**. Checked private **503** were
**98/74/12**, all failed requests despite k6 command exit zero.
Caps four/eight had only **98/74 HTTP-only outer rejections**
and zero observed provider Busy/QueueTimeout. Cap sixteen had
**7 outer-only + 5 provider Busy** (two JSON burst, three
SSR observe) and zero observed QueueTimeout. Both eight-read
recovery sequences per cohort and exact-owned cleanup passed.

Exact correlated events were HTTP/provider/worker
**12,550/12,452/12,452** at cap four,
**12,549/12,475/12,475** at cap eight, and
**12,552/12,545/12,540** at cap sixteen. Only the five
provider Busy failures have an attempted provider call but no
worker; outer-only 503 have **no provider call or worker**.
That conditional distinction requires selected valid GET routes,
known source/feature/startup cap, correct private 200/503 oracles,
zero-overflow trace and reconciled provider/SQL/failure counters.
The trace does not record actual HTTP gate reason/admit/release
instants; a 503 body alone cannot prove which gate refused it.

Cap sixteen numerically reduced total failures while exposing
Session pressure; cap four shed more load before the provider.
Offer boundaries differed (JSON observe 2,501/2,500/2,503),
as did arrival, host allocation, warming, connection and
sampling. Neither causal monotonic capacity nor optimal default
can be inferred. See the [actual cap-selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection),
[Qualification Matrix](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts)
and retained [Session policy](PROVIDER-OPERATIONS.md#provider-overload-and-quota-distinctions).
## One bounded local experiment

From the current checkout, after the main agent verifies the two existing
prebuilt debug binaries:

```sh
cargo test --locked --manifest-path xtask/Cargo.toml --test local_operations
cargo xtask qualify local-operations
```

The first command runs Rust SQLite and direct owned-child lifecycle tests;
it launches no sample server. The second command launches only
`target/debug/yetlet-auth-notes` and `target/debug/yetlet-sqlite-crud` on ephemeral
IPv4 loopback ports. It does not build Rust, use Docker/Redis, contact a remote
service or install dependencies. Run it once; no repeated soak or latency gate
is defined. These current YT-086 commands remain pending independent review.
The historical commands were `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_local_operations.py`
and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/qualify_local_operations.py`.
[M10g verification](M10G-VERIFICATION.md) separates those ten original Python
source/lifecycle tests from the actual historical local run, which passed with
seven processes reaped and temporary data removed.

1. Start two real Auth Notes processes with the same explicit synthetic fixture
   codes. Authenticate Alice on A; her cookie returns 200 on A and 401 on B.
   Authenticate Alice separately on B; that different cookie returns 200 on B
   and 401 on A, while A's original cookie remains usable on A. Restart A with
   the same codes; its old cookie returns 401. Check `no-store` on these private
   responses. Passing records the limitations: Session is neither shared across
   workers nor persistent across restart.
2. Start two real SQLite processes against one private local WAL file. Create a
   note on A and read the exact committed value from both. Synchronize two HTTP
   updates with a barrier and the same expected version: require exactly one
   200 and one 409, final version 2, and exactly the create/update audit rows.
3. With no mutations during backup, use the Rust SQLite online backup API into
   a new owned 0600 checkpoint. Check integrity, foreign keys, schema version
   and exact notes/audit rows. Add one late note, stop both workers, then restore
   via the same API into a fresh isolated 0600 candidate. Start the actual app
   and verify the exact checkpoint state and absence of the late write.
4. Damage the header of a separate candidate created from the checkpoint;
   require the real app to exit nonzero before becoming healthy. Verify the
   checkpoint's contents and digest remain unchanged. Never restore over an
   active database or replace a live database file.

The [official SQLite online backup API](https://sqlite.org/backup.html) produces
a database snapshot through SQLite; copying a live main file while ignoring its
WAL is not this experiment. The script uses small fixtures, short SQLite busy
timeouts, bounded page steps and a deadline in the backup progress callback,
including busy retries. Restore uses the same API. Checkpoint equality includes
all note/audit columns and `user_version`, not just row counts.

Every child is tracked before readiness; all owned children receive bounded
SIGINT/wait and kill/wait fallback as needed. An owned 0700 `TemporaryDirectory`
contains databases and their WAL/SHM files and is removed after process cleanup
on success, startup failure, exception or cancellation. Child output goes to
private temporary logs. Only after cleanup attempts does the Rust tool write
`target/m12/yt086/local-operations-evidence-*/local-operations.json`. The old
Python receipt remains at `target/m10-remaining/local-operations.json`; cleanup failure prevents local
PASS. Report values are fixed outcomes/counts/statuses, durations and version
identity, never raw cookies, CSRF tokens, codes, environment, response bodies,
logs or exception text. No arbitrary directory deletion is used.

The report labels success `pass-for-local` and explicitly labels distributed,
multinode, HA and performance SLO qualification `not_verified`. Durations are
observations without latency thresholds. Binary SHA-256 and current checkout
package versions identify the inputs; the samples expose no binary-version or
linked SQLite-version endpoint, so those runtime versions remain `not_exposed`.
The tool's SQLite version is observed separately and must not be attributed to
the Rust app's bundled SQLite library. The main agent must retain build/source
identity if it needs to connect a prebuilt binary to a particular source freeze.

## Historical automated qualification and Redis staging

The YT-065/066 planning baseline distinguished native and browser
automation from unqualified **real OS IME, assistive-technology speech,
Safari/WebKit, mobile touch and 200%/400% zoom**. The accepted YT-065
automated scope did not convert those optional manual lanes into PASS;
see the [Qualification Matrix](QUALIFICATION-MATRIX.md). YT-066a's
actual single-host Docker test verified stateless routing and the
**inability** of process-local Memory Session cookies to authenticate
across another instance or an old restarted instance.

The separate opt-in Redis provider was accepted through staged design,
b1 private spike, b2 implementation/testing and b3 actual single-host
three-app qualification. The b2 CAS callback barrier originally hung
twice; a later separately accepted correction and tests resolved the
stall. The final b3 primary topology intentionally disabled persistence
and did **not** prove cross-host HA, backup restore or production RPO/RTO.
Unknown outcomes after dispatched writes, CSRF/Cookie ordering and worker
drain require their separate retained source-specific evidence.
Production storage, TLS/ACL, backup, recovery and business durability
remain application/operator responsibilities, not blocked mandatory
framework functionality. The original plan's earlier Pending and
Blocked observations retain their original dates and are not current
readiness labels. Full planning and review text remains at **local Git**
`42d8856b9acf5ddddab9a4b998b5bdd4c5e59210` (not yet verified reachable through public GitHub).
## Single-host Docker/Memory qualification

YT-066a adds a bounded deployment-shape proof without adding a Session provider. Run the focused source tests, then the owned Docker fixture:

```sh
cargo test --locked --manifest-path xtask/Cargo.toml --test qualify_multi_instance
cargo xtask qualify multi-instance
```

The second command performs an offline build from locked vendored inputs with the pinned Rust 1.99 image, builds one local nonroot application image, pulls only the pinned Nginx digest, creates three Auth Notes containers and one reverse proxy, exercises the topology, writes a fresh `target/m12/yt086/docker-logs-*/multi-instance.json` only after cleanup, and removes its generated image. The retained YT-066a execution below used the earlier Rust 1.88 image; it is not relabeled or rerun for the compiler pin update. [Integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records the new image identity and focused configuration checks. The application containers publish no ports and attach only to an internal backend. The proxy attaches to that backend plus a front bridge and publishes only an ephemeral host-loopback port; Docker 29.9 does not publish ports for a container attached solely to an internal network.

The Auth Notes default remains loopback-only. `YETLET_AUTH_ISOLATED_CONTAINER_LAB=1` selects a separately named fixture entrypoint that permits a wildcard listener only inside this isolated arrangement. It is not a production deployment option and does not change Session security semantics.

The historical direct tests used `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_qualify_multi_instance.py`.
The historical command `PYTHONDONTWRITEBYTECODE=1 python3 scripts/qualify_multi_instance.py` wrote
`target/yt066a/multi-instance.json`; its retained final execution passed. The
current Rust YT-086 candidate has not completed an independently accepted Docker
run. The historical observations were:

- pooled stateless requests reached three instances, then two surviving instances while A was stopped, then all three after A restarted;
- a Session created and authenticated on A returned 200 on A, 401 on B and C, and 401 on restarted A using the old cookie;
- application containers had zero host bindings; the proxy host address was `127.0.0.1`; backend network `Internal=true`; and
- four service containers, the build container, two networks, zero volumes and the generated application image were removed, with no related process or Docker object left.

This is `pass-for-docker-memory`: it demonstrates stateless multi-instance serving and the current Memory provider's limitations. It does **not** demonstrate distributed Session, sticky-session correctness, Redis, cross-host HA, failover, load/SLOs, production recovery or a deployment image. Durations are observations without acceptance thresholds. Exact source/image/Job identities and the two classified operational fixture failures are recorded in [M10g verification](M10G-VERIFICATION.md).

The YT-066a reviewers found cleanup-proof, signal/finalization and report-I/O defects in the harness, followed by a duplicated-fragment syntax failure; these do not invalidate the retained successful Yetlet behavior observation. The corrected harness uses one phase-aware SIGTERM/SIGINT state, an outer transition recovery boundary, fail-closed Docker queries, exact image verification, required owned-work removal and report-I/O failure handling. A first cancellation signal at the covered teardown/finalization stages records nonpassing cancellation while cleanup continues; subsequent cancellation signals are ignored and both prior handlers are restored. In-memory compilation and the existing 18 focused mocked regressions pass after the 2026-10-11 syntax repair. The successful Docker qualification was not rerun. Final independent read-only reviewer thread `01a1286a-5829-7411-9ecc-dbc3f00fe001` returned PASS, completing YT-066a at single-host Docker/Memory proof scope. See [M10g verification](M10G-VERIFICATION.md) for the historical source identities and each review outcome.

## Application-selected mature store acceptance contract

Select a concrete application topology and mature external integration before
testing. Native Axum/Tower composition remains available; the following is an
application acceptance contract, not a generic Yetlet store trait or commitment
to implement a new provider. Record exact versions, nodes, consistency model,
failure domains, store/cookie settings and responsible owner first.

| Area | Required actual-topology evidence |
|---|---|
| Dependent mutation | CAS or equivalent atomic mutation; simultaneous dependent updates retain every successful change without read/modify/write loss |
| Rotation and revocation | Atomic mutate/rotate/revoke, retired identities cannot resurrect, successful concurrent updates are preserved, revoked identity is rejected by all workers |
| Stale cookie responses | Under the [explicitly selected pure-SSR contract](PROVIDER-OPERATIONS.md#redis-session-cas-operation-outcome-and-cookie-ordering), suppress observed-stale Set and emit no logout Clear. A late prepared Set may cause login loss, but a retired identity must never authenticate; no old-cookie fallback |
| TTL and consistency | Expiry and revocation enforcement across workers, cache invalidation, clock/lease assumptions, explicit behavior during partitions and recovery |
| Outage and recovery | Store unavailable, timeout, partial failure and failover fail closed; no authorization through a stale or invented fallback Session; replay/retry cannot resurrect revoked identities |
| Bounds and shutdown | Bounded concurrency, pools, queues, timeouts/retries, admission/backpressure and drain/cancellation under real contention; no unbounded work or abandoned owned resources |

The [tower-sessions concurrency documentation](https://docs.rs/tower-sessions/latest/tower_sessions/#data-races-under-concurrent-conditions)
warns that concurrent dependent mutations can lose updates because requests
load separate Session snapshots. A pluggable Redis backend by itself therefore
does not prove the stronger atomicity/rotation/revocation contract above. This
is a reason to qualify the selected integration, not to write an in-house store.

## Optional sensitive writes, tracing and catalog consistency

The following remain **application-owned, opt-in references** rather than
new Yetlet Core/Session middleware, global providers or stable public APIs.
The original YT-181/182/183 contracts, implementation verifications, failed
probes and review receipts are recoverable from fixed **local Git**
`99906e421c0510231291bf527e2cbd2e7ad0ffda`; source Git does not
contain all ignored native/raw evidence. Precise activation, defaults and
business behavior remain in the [Lab Board](../examples/lab-operations-board/README.md)
and [Modular Shop](../examples/modular-shop/README.md) guides.

### Commit-time authority and revocation

Opt-in Board `commit-write-auth` adds a **server-issued private ticket**
bound to a trusted Store-held authority row/generation; the real application
SQL write checks current permission while holding the same independent SQLite
`BEGIN IMMEDIATE` business-effect transaction. A prior HTTP/Session/IdP
check alone cannot guarantee revocation against an already-admitted worker.
SQL revoke advances the generation, and explicit re-enable advances again:
a stale ticket never becomes valid simply because authority is later
re-enabled. A writer holding an IMMEDIATE transaction *before* revocation
commits may finish before revoke is eligible; a later response does not
change commit ordering. Private trusted integration controls issuance and
re-enable, **not** an untrusted form/query/client generation. Missing,
empty, legacy or replaced authority is denied on reopen; no implicit
allow/reseed or automatic restore of revoked authority.

Only selected SQL-owned write permission gains this commit-order property.
Session logout, account disable, file policy, IdP refresh and other current
Auth/Session/OIDC checks retain their own bounded freshness races. There is
no generic two-phase commit, distributed lock, cross-replica authority,
exactly-once outcome, automatic rollback or retry. Existing four SQL
workers, 250ms SQLite Busy limit, one-second worker admission, item/history
storage and safe 409/503 remain unchanged. Started SQL can complete after
HTTP cancellation or 504: check committed state before any new action.
The historical second-connection Busy/revoke-before-explicit-restore and
reopen-deny tests are source-specific evidence, not new production
certification. Original first compiler visibility warning was corrected
without treating earlier test execution as a post-change rerun.

### Request tracing and owned worker spans

Opt-in Board `request-tracing` is default off, requires deliberate
`BoardOptions.request_tracing=true` through the embedding host, and
**does not initialize a global subscriber**. It is independent of
`write_authorization`; merely compiling the feature selects neither.
Only a trusted server-issued positive process-local request ID, a finite
method, fixed worker operation and `ok`/`error` may enter the safe span.
No caller-supplied trace parent, raw URI/query/body, SQL parameters,
identity, Cookie, Session ID, credential or secret crosses the boundary.
Propagate only the selected root/span/Dispatch into actual owned async
and blocking worker work; unrelated ambient spans are not copied.
Worker lifetime and its permits continue after a dropped HTTP future.
`http_response_constructed`, `http_future_dropped`, and
`sql_worker_completed` are **different** events; neither response
construction nor worker success alone proves response-body delivery or
durable business commit. Host/Origin pre-router refusals, static routers
and streamed body chunks are outside that selected instrumentation.

No new tracing server/exporter, global recorder, implicit diagnostics
route, universal audit identity or worker cancellation is implied.
Original selected regression and redacted output proofs are bounded;
the old stricter Commit Guard finding in a synthetic foreign-span test
field was corrected without weakening Guard or changing runtime behavior.

### Opt-in catalog-data cache boundary

Shop's `catalog-cache` native feature and
`YETLET_SHOP_CATALOG_CACHE_ENABLED` exact true/false startup selection
are both default off. Reject invalid/non-Unicode input or enabled
selection without the compiled feature before listener startup.
Only the public ordered product id/name/price/stock projection may be
cached in an application-local private native Moka store; never cache
Session, CSRF, cart ownership, orders, idempotency, checkout pricing,
authorization or SQL stock authority. Browser responses are still
rendered freshly with private/no-store semantics.

The selected source-age cap is **2 seconds from monotonic SQL load start**
at value selection plus separate native 2s insertion TTL; cache hits
do not extend either. Admit at most 64 products and approximately
64KiB typed retained payload; oversized success returns **uncached**
without truncation. Moka's one-entry target is best-effort rather than a
whole-process peak-memory ceiling. One actual SQL-origin fill permit,
native same-generation coalescing and cooperative **500ms** lookup/fill
wait apply. Started blocking SQL holds its fill/SQL permits after caller
timeout/drop. Miss or source error never returns a stale-on-error or
cached negative authorization result, and no app retry queue is added.

Successful checkout/idempotent committed return advances a local
generation **inside the blocking SQL transaction closure before returning**;
rollback does not publish. A late old-generation fill cannot populate the
new active key, although readers that selected a prior generation may
finish with an older **bounded-age** display. Generation saturates
without wrap; cached reads fail unavailable until cold reopen without
changing checkout authority. Separate Store opens/processes and
out-of-band SQL writers are **not** coherent or invalidated remotely;
the two-second bound is not a linearizable read or durable distributed
cache guarantee. A previously observed missing-offline-dependency
exit101 was a fixture/preparation failure, not a cache product defect.
See [Shop cache activation](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183)
for the current app's actual feature and tests.

## Historical Session, Auth and OIDC contracts and failures

The selected Session, Auth and OIDC behavior is **explicitly opt-in and
application-owned**. Authoritative current API wiring remains in
[Auth](../crates/yetlet/guides/auth.md),
[Session](../crates/yetlet/guides/session.md),
[Redis-protocol Session](../crates/yetlet/guides/session-redis.md),
[rate limiting](../crates/yetlet/guides/rate-limit.md) and the
[Admin](../examples/admin-dashboard/README.md) application. This section
retains security-critical source history without replacing module contracts.
The 18 original dated YT-066B/094/113–117/124–126/139/140/175/184 reports,
their **4,645** source lines, original FAILs, precise command/fixture scopes,
headers and section anchors remain retrievable at **local Git**
`0ade9c62b67fe51ecb8e9ee6edc666daed24b5c1`. Local SHA reachability does
**not** prove those unpublished sources are currently available on GitHub;
task-owned ignored logs/browser/cloud receipts are separate from Git history.

### Redis Session CAS, operation outcome and cookie ordering

The original Redis-protocol design selected a **single-primary** opt-in
Session store, never automatic Core Session replacement. An authenticated
Session record uses opaque identity and a bounded envelope/namespace, a
revision and absolute expiration. An atomic compare-and-set/rotation/revocation
operation must not overwrite a successful other-revision write; a reported
success remains a committed historical operation even if later mutations
advance that record. Each state change and its **operation-result evidence**
must be atomically recorded; an immutable dispatch deadline prevents late
dispatch from turning a caller timeout into unbounded surprise mutation.
A transient timeout, lost response or unknown operation ticket is
**unresolved**, never proof of rollback or permission to blindly replay
login/logout, rotation or a dependent mutation. A later result lookup can
establish the original *commit status* but cannot reconstruct arbitrary
callback return values or reverse side effects.

The provider's current selected local reference bounds one Store's
**four active and eight total** operations; a full total gate fails
immediately rather than adding an unbounded queue. An admitted waiter
has at most **80ms** to acquire an active slot, further capped by the
shared **400ms per-Session-operation** deadline (including lock/permit/
backend phases). A request may make multiple such operations and perform
a finalizer; 400ms is **not** an HTTP deadline. Store clones share local
permits; distinct client managers/replicas do not share a global admission
budget merely because their Session namespace is shared. A started worker
retains its permits after caller cancellation; the selected shutdown drain
remains **two seconds**. Busy, queue timeout, backend error, invalid
record/protocol, closed manager or missing authority is never anonymous
authentication, a successful cookie rotation or Memory fallback.

Session loading must distinguish a genuinely absent/expired/rotated identity
from infrastructure unavailability. Only an authoritative, positively
resolved create/rotate/replacement operation can issue `Set-Cookie`;
a finalizer that observes a retired or wrong identity suppresses stale
writes and cookie clears, with truthful private errors on failure. Existing
evidence includes a stale response that must not erase a newer login,
competing CAS rotations, logout/revocation and absolute TTL that normal
reads/updates do **not** silently extend.

**Historical unclosed Cookie candidate P1 (do not promote to PASS):**
the dedicated YT-066B Cookie ordering experiment initially passed its
small focused tests after an input-envelope correction (fractional Lua
metadata such as `1.5` must be rejected), but its subsequent independent
Reviewer **FAIL** exposed an unresolved replacement-retention invariant.
With **180 delayed lower-issuance responses** arriving after a newer
Session/logout marker, actual **Chrome 155** retained only **150 cookies**
and evicted the newer marker. A smaller 57-cookie test did **not** model
this storage quota. Parsed old-cookie rejection alone does not prove the
browser still retains the necessary replacement or logout fence.
Increasing parser limits, assuming fewer delayed responses or requiring
client JavaScript was **not** an accepted fix. This is a negative result
for that proposed candidate protocol, **not** evidence of a current
implementation guarantee or approval to install the rejected candidate.
The separately accepted later 3-app single-primary Redis qualification
and cleanup are bounded source-specific runs and must not be cited as an
independent correction of that exact browser quota counterexample.

The selected YT-094 Admin/Valkey follow-up starts with a short-lived
**60-second test fixture**, then separately qualifies application
`1800`-second default Session TTL and a **1-second–30-day** bounded
configuration range. That lifetime is not refreshed by every read.
Cross-process cookies must use a consistent path/name/scope and protected
response ordering; local loopback ports **do not** isolate host-scoped
cookies. A future public HTTPS edge needs a separately reviewed Secure
Cookie/Host/Forwarded contract.

### Local Auth, permissions and multi-instance authority

The minimal native Auth foundation keeps **trusted identity provenance**
separate from business roles. An explicitly authenticated principal is
resolved from the selected native layer/verified local binding; a request
header, stored JSON permissions, cookie contents, display name, email or
foreign roles do not become trusted local permission authority.
Optional/required identity extraction has different anonymous outcomes.
An authorization denial must use the application's existing 403/CSRF
response; a policy/store/read/decoder outage is a redacted private
**503**, not `None`, 401 or a new default-allow. The optional Memory
Session bridge leaves the ordinary anonymous cart Session and its
CSRF/Cookie lifecycle intact, and does not provision new accounts or
silently enable Auth across unrelated routes.

The owner-selected YT-124–126 Admin/Board application tests actually
exercised **two independent processes** sharing one isolated Valkey
namespace. Login on A resolved on B through a fresh local policy, B
logout/revocation rejected retired identity across both, and forged
permission/tenant/account state was denied. A real policy-file replacement
or disabled local account must be read and validated at each selected
protected decision: remote Session identity is **not** an authorization
decision cache. Missing, malformed or unreadable current policy cannot
fall back to a stale allow; provider and authority errors fail closed
without leaking raw state. These findings apply only to the selected
same-host topology and trusted policy-file lifecycle, **not** arbitrary
multi-host policy replication, stable Redis service persistence or a
generic Auth/Session middleware guarantee. The original YT-125 and
YT-126 setup/lifecycle errors, omitted default Production overlay and
focused corrective-review records remain actual historical failures,
not a clean first execution.

The original shared-file SQLite YT-094 case exercised **two opened Board
instances**, visible writes, conflict **303/409**, held independent
writer/Busy **503**, and restart with preserved history. This is
**shared local SQLite file evidence**, not a distributed database or
cross-host transaction. A write already in an independent SQL closure
can finish after a logout, Account disable, HTTP cancellation or 503/504
response. Session/CSRF/current-policy prechecks are **not atomic** with
a later SQL business commit, nor a substitute for the separate
[commit-time Board reference](#commit-time-authority-and-revocation).
Unknown mutation outcomes require source reconciliation, not automatic
retry.

For current restrictive authority and recovery, retire the old
Session namespace/epoch under a trusted operator-controlled cutover.
Old unexpired Session records may still exist physically; restoring or
reusing their old namespace can **resurrect stale authority**.
Stop and drain old writers, preserve durable SQL and current policy,
select a new never-reused namespace, then require fresh login. Do not
silently re-enable revoked accounts or treat a backend restart as a
new authorization epoch. There is no automatic distributed epoch
allocator or session-to-SQL commit fence in this application.

### OIDC cross-instance origin and callback

The accepted YT-140 opt-in OIDC proxy example selects an explicit
**canonical local HTTP origin**, only `http://127.0.0.1:<port>` with
a canonical decimal nonzero explicit port **other than 80**. A fixed
`/board/oidc/callback` URL is computed from that selection; if it is
unset, the retained direct-listener address remains authoritative.
Reject userinfo, alternative host/scheme, paths, query, fragment,
whitespace/control and noncanonical spelling. All selected replicas
must use identical issuer, client ID, credential source, callback,
subject binding, remote Session namespace/cookie and local policy.
Disabled optional OIDC settings remain inert rather than triggering
discovery or secret lookup.

The single trusted loopback proxy preserves the original Host/Origin
and strips untrusted `Forwarded` and `X-Forwarded-*`; Board ignores
these values even when sent directly. Require **one exact canonical
Host**; an OIDC top-level callback may omit Origin, but any supplied
Origin must be **single, non-`null`, exactly matching**. Do not derive
issuer, redirect, public scheme or peer trust from arbitrary headers.
Login has native CSRF and fresh state/nonce/S256 PKCE, and installs a
short-lived (five-minute) pending record. The callback **atomically
takes/consumes** its pending state before token exchange; replay, wrong
state/issuer, missing code, expired transaction or unknown consume
never becomes a second successful login. Concurrency conflict and
provider/store unavailability remain explicit error outcomes; a
timeout does not show that no callback was consumed.

Only a verified issuer/subject maps to an **existing explicit local
identity**; email, provider roles and display name cannot provision or
authorize. OIDC discovery/JWKS/client credentials and subject maps are
**startup snapshots** in this reference, without automatic JWKS refresh,
upstream account-disable propagation, token introspection, provider
logout or hot-reload promises. ID Token expiration differs from the
local Session's absolute lifetime. A security-critical map/key change
requires a coherent restart and often a new Session namespace; no
rolling mixed-trust shortcut or public HTTPS/HA certification follows.
Original real-IdP loopback proof (including prior WAF rejection/setup
limits) is separate from the later two-local-listener static-proxy
test; neither turns this into a public IdP platform service.

### Provider overload and quota distinctions

The accepted YT-175 policy was a **reviewed operating recommendation,
not an added runtime HTTP admission feature**. An outer application
HTTP gate, backend Session operation permits and an opt-in per-key
rate budget each bound a different population. A zero-queue gate can
shed more work before Session/SQL, but cannot promise to admit a burst
or keep the provider below its own pressure limit. A rate-limit **429**
with its computed `Retry-After` is a real quota decision. A Session
Busy/closed/backend/capacity failure remains redacted no-store **503**;
do not make it a login redirect, 401/403 or universally **504**.
A genuine expired/absent identity and an authenticated but denied
business action retain their separate application responses.
No numeric retry delay can be inferred from the 80ms permit wait; do
not invent `Retry-After` on 503 or a hot retry loop. Any safe read
retry must separately budget bounded attempts/backoff/jitter and renew
fresh authority checks. Never retry an uncertain write or
login/logout/rotation merely because its HTTP response was lost.

## Application/operator responsibilities and unverified deployments

No production topology, recovery budget, retention policy or SLA is chosen by
this experiment. An application deploying to production designates its own
application, storage and security owners. These responsibilities are deployment
guidance, not Yetlet deliverables or blockers for bounded YT-066 completion.
Provider correctness, supported configuration/error behavior and redaction remain
framework responsibilities; deployment policy, backup protection, business RPO/RTO
and on-call staffing belong to the application/operator.

| Pending control | Owner to designate and required test/decision |
|---|---|
| RPO/RTO | Application/storage owners select objectives from actual business requirements; measure recoverable checkpoint age and end-to-end restoration on the selected topology |
| Restore verification | Storage owner verifies integrity, schema, application state, authorization and excluded late writes; document isolated restore, acceptance, rollback and promotion procedures |
| Backup protection | Storage/security owners select encryption and key custody/rotation, access controls, retention/deletion and off-host durability; test restoration with the required keys and loss/failure cases |
| Secret redaction | Security owner defines cookie/token/code/credential/log exclusions and tests failure paths, exports, metrics and retained evidence; do not attach raw sensitive logs |
| Observability | Application/operator owners define safe correlation, conflict/outage/queue/drain/backup metrics, alerts and recovery procedures; verify them under the real failure scenarios |

The [YT-066b3 fixture](QUALIFICATION-MATRIX.md#historical-session-auth-oidc-and-cross-host-evidence) is Done at bounded three-application single-host/single-primary scope after actual Session/CAS/rotation/revocation/outage/restart execution, cleanup and final independent reviewer PASS under an explicit no-persistence configuration. Production provider outage/recovery, Redis failover, cross-host consistency, HA, load/performance SLO,
production backup durability/RPO/RTO, real OIDC and key rotation remain **not
qualified**. Retained Session, file, SSE and YT-066a Docker/Memory results cannot
replace application-specific deployment qualification. YT-066 is **Done at single-host Docker multi-instance and selected Redis-provider scope**,
with the separate final documentation review gate recorded in M10g. Production
ownership or recovery-target selection is not a prerequisite for this scope. Its scope is the tested single-host
Docker topology and experimental single-primary provider. No bounded PASS
unblocks YT-020 publication. The earlier local execution and reviewer
`wc_job_YclpZ0R5tS4m8Y1I` passed; the Docker/Memory behavior execution passed,
and its six earlier review outcomes, bounded repairs and final PASS are recorded in M10g verification.
The experimental single-primary provider is explicitly selected; broader
deployment guarantees remain unverified and belong to application qualification.
