# YT-181–YT-184 — Application follow-up evaluation and schedule

Date: 2026-10-25. Status: **Scheduling checkpoint: all four items were Todo. YT-181 now has an
authorized opt-in implementation accepted at bounded scope; YT-181 Done, YT-182 Done at bounded reference scope and YT-183 Done at bounded reference scope and YT-184 In progress. No cloud
allocation started.**

## Authority and decision

The maintainer asks to schedule sensitive-write consistency, reassess useful
opt-in capabilities without treating every unselected candidate as missing,
and evaluate Azure CLI use for deployment applicability. This document selects
bounded follow-up scopes, not a generic framework or production qualification.
The benchmark phase through [YT-180](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection)
is closed for now. No old benchmark, test, build, analysis, platform or provider
matrix is replayed. Previous research PASS/Done decisions remain historical;
new consumer work does not retroactively deliver a crate or expand the original
nine release candidates. No Commit/Push is authorized.

## Selected work and priority

| Item | Selected follow-up | Placement and decision gate |
| --- | --- | --- |
| YT-181 | Sensitive Admin commit-time authorization reference | First priority. Contract first, then a separately activated minimal opt-in application reference for one concrete write and revoke scope; independent review before Done. |
| YT-182 | Optional native tracing reference integration | First capability candidate. Extend real request/worker diagnostics using existing Rust composition; app-owned feature/settings, no new tracing crate selected. |
| YT-183 | Optional non-authoritative data-cache consumer | Second capability candidate, conditional on a useful real derived-data read. Establish staleness/invalidation first, then a bounded opt-in reference if justified; no generic cache trait/crate selected. |
| YT-184 | Azure cross-host authority and persistent-restart reference | Small functional integration/recovery experiment after transport and cleanup admission. No capacity ladder or automatic HA claim. |

Priority is a recommendation, not a fabricated dependency: tracing and the
cache evaluation do not depend on fixing unrelated writes. Every future item
needs its own concrete contract and independent Reviewer acceptance before
Done; implementation and new checks execute only when that stage is selected.

## YT-181: strengthen one concrete application write

The owner has now activated the [minimal contract stage](YT-181-SENSITIVE-WRITE-CONTRACT.md):
SQL-owned account/project write revocation and the item/history commit boundary.
Keep native middleware and domain composition; no generic AOP. The independent
contract Reviewer now returns PASS, zero issues/followups; YT-181 remains In
progress. The later [implementation/proof stage](YT-181-SENSITIVE-WRITE-IMPLEMENTATION.md)
is now Done at bounded opt-in reference scope after independent implementation
Reviewer PASS, zero issues/followups. Earlier contract acceptance retains its scope.

[YT-138](YT-138-AUTHORIZATION-COMMIT-ORDERING.md) deliberately retains the current
pre-SQL race. The new strategy is to demonstrate an opt-in reference that
actually orders a selected business commit with its selected revocation,
rather than repeating the retained-limit study or moving another reread closer
to SQL. Start with an existing Admin/Board item or triage write.

Select exactly which revocation matters: account/project permission removal
and one-Session logout are different contracts. Compare a business-SQL-owned
monotonic authorization generation with a shared writer/revoker serialization
boundary. The effect authority must enforce the chosen condition atomically
with the effect; a copied Valkey/file version, local mutex, lease alone or
unrelated item edit version is insufficient. All relevant writers and revoke
paths must participate, including stale binaries and recovery. The current
file/Valkey/SQLite authorities cannot gain atomicity from a document or extra
lookup.

The selected slice must specify the ordering point and acknowledgments, paused
and already-running writers, unavailable authority, unknown commit/revoke,
restore rollback/ABA and reconciliation without blind replay. Keep identity,
CSRF and resource authorization separate. Prove the chosen scope with one new
bounded paused-write-versus-revoke case when implementation is activated;
retain pre-change evidence and failures. This is application/domain work,
without adding a mandatory Auth API or generic distributed lock service.
Until new implementation/proof exists, the current race remains a known limit.

## Capability reassessment

| Capability | Existing consumer and useful follow-up | Current decision |
| --- | --- | --- |
| Tracing/correlation | Starter already has a JSON subscriber, request spans and `.instrument`; Board/Admin have real HTTP, Session and SQL worker boundaries. Improve safe propagation and distinguish response construction, cancellation and worker completion. | Schedule YT-182 as an opt-in native integration reference. Reuse `tracing` and Tower; extraction requires actual repeated useful glue across independent consumers. |
| Data cache | Current Board/Admin readers use authoritative SQL; no demonstrated cache hotspot or repeated adapter exists. A project summary/derived aggregate is a candidate only if its actual use justifies avoiding repeated work. | Schedule YT-183 as consumer selection and conditional reference. Consider direct Moka with typed State; do not promise a speedup or invent a second consumer. |
| Public output cache | Existing repeated bytes are primarily static assets; private Session/Auth responses remain no-store. No actual repeated public dynamic response store currently justifies extraction. | Keep conditional/deferred under YT-122. Select a real public consumer and eligibility/key/invalidation contract before creating an implementation item. |
| Audit | Board history is transaction-owned; Auth Notes has no comparable durable sink. Diagnostics and required transactional audit have different acknowledgment semantics. | Retain YT-118's no-extraction decision. No `yetlet-audit`, sink registry or invented audit consumer is scheduled. |

### YT-182 boundaries

The maintainer now selects the [concrete contract](YT-182-TRACING-WORKER-CONTRACT.md)
for a first Board/Admin consumer integration. Starter remains retained comparison
evidence; no second consumer or common tracing crate is manufactured. Contract
Reviewer `/root/yt182_safe_tracing_contract_review_20261025` PASS with zero
issues/followups precedes the [minimal integration/new proof](YT-182-TRACING-WORKER-IMPLEMENTATION.md);
final implementation Reviewer
`/root/yt182_safe_tracing_implementation_review_20261025` now returns PASS, zero
issues/followups, 21 implementation/81 evidence/15 retained bindings stable.
YT-182 is Done at bounded application-reference scope.

Preserve existing RequestIds, native Axum/Tower routes and app-owned subscribers.
Select bounded allowlisted fields and trusted local correlation before format;
exclude raw URI/query, credentials, cookies, tokens, bodies, SQL parameters and
unbounded user/tenant labels. Propagate only safe owned context across actual
async/blocking work. A request completion does not establish worker completion
or business commit. Avoid a default subscriber/exporter, OTLP service, mandatory
collector or invented durable-job consumer. The feature/settings disable path
must omit new instrumentation work; preserve existing explicitly selected
Starter diagnostics. No server-only dependencies enter browser builds.

### YT-183 boundaries

The maintainer now activates the [Shop display-cache contract](YT-183-CATALOG-CACHE-CONTRACT.md)
before minimal integration. Public catalog and authenticated cart share one
underlying projection; native Moka, bounded age/admission and commit generations
are selected. Existing mutation/checkout/Session authority remains unchanged.
Independent Reviewer `/root/yt183_catalog_cache_independent_review_20261025`
PASS, zero issues/followups; 20 file/41 evidence/16 retained bindings stable.
YT-183 is Done at application-reference scope; no old-proof replay or new-change
Commit/Push.

The first selection gate is a real useful derived-data consumer and acceptable
staleness; if absent, record that result rather than force an implementation.
Keep Session, permission/account state, revocation, CSRF, idempotency and active
rate budgets authoritative and uncached by this candidate. Run current Auth
and tenant/resource checks on each request. Bind any private derived value to
trusted scope and a version/generation; never reuse it across tenants.

For an admitted consumer, specify finite capacity/weight, TTL, fill concurrency,
error/fallback behavior and commit/invalidation-versus-in-flight-fill ordering.
Moka's capacity enforcement is best effort, not a hard RSS ceiling. Disabled
means no new cache construction/work; backend fallback remains bounded. New
focused behavior evidence may establish reuse/origin-read reduction without
reopening the old throughput benchmark phase. A measured benefit and repeated
useful glue are prerequisites for a later `yetlet-cache` extraction proposal.

## YT-184: minimum useful Azure experiment

The maintainer now activates the [concrete contract](YT-184-AZURE-AUTH-SESSION-RESTART.md),
including opt-in verified private transport, explicit epoch recovery and one
bounded allocation. Original proposal below remains the scheduling baseline;
new evidence/status is maintained in that contract before Done. The first
allocation now fails at a fixture Path expression before live Auth/restart;
minimal local correction and cleanup pass, but live acceptance is unexecuted
and YT-184 stays In progress. A second allocation needs explicit continuation
under the original one-generation bound.

Read-only CLI checks succeed: Azure CLI **2.91.0**, AzureCloud and a selected
Enabled default subscription. This is not proof of resource-creation rights,
quota, regional SKU capacity, storage transfer or deployment. No account IDs,
credentials, existing resources or subscription settings are changed/recorded.

Reuse [YT-164](YT-164-PRIVATE-HTTP-SSH-COMPARISON.md) and
[YT-169](YT-169-PRIVATE-HTTP-SUSTAINED-1800.md) isolation, private network,
Managed Run Command, protected Blob transfer and exact-owned cleanup patterns;
do not rerun their benchmarks or copy their large compute sizes as requirements.
Reuse [YT-139](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md) and
[YT-140](YT-140-OIDC-PROXY-REPLICAS.md) evidence without extending their scope.

The smallest proposed topology is two private VMs: app A on one host, app B
plus a single PostgreSQL/Valkey authority on the other, with dedicated owned
persistent storage. It demonstrates cross-host clients, a shared authority
and failure/restart recovery. Both authorities still share one failure domain;
this topology cannot demonstrate automatic backend failover or HA.

The current Auth/PostgreSQL fixture uses a Unix socket, explicitly disables
SQL TLS and restricts Valkey URLs to `redis://127.0.0.1:<port>/`; its Session
cookie configuration is explicitly for local HTTP. These are source-observed
local-fixture boundaries, not cross-host secure defaults. Before allocation,
select application transport/cookie trust, authenticated private PostgreSQL/
Valkey connections, TLS endpoint verification and bounded secret-file inputs
using supported native clients. Private IP alone is not authentication or
encryption. Preserve the existing default/local topology and no public data
listeners. Define the smallest opt-in reference transport change before cloud
startup; no opportunistic protocol implementation, tunnel workaround or trust
downgrade.

The future contract should admit one fresh task-tagged Resource Group, at most
two VMs and required private network/storage, one allocation generation and a
**90-minute total lifetime with 20 minutes reserved for cleanup**, not a resettable
clock. Confirm exact region/SKU, rights/quota, current cost ceiling, transfer,
TLS inputs and deletion access before creation. Numbers are experiment bounds,
not user hardware or SLO requirements. Failed admission stops without alternate
regions/SKUs or replacement allocations; retain failures and partial evidence.

Use a small finite functional journey: read the same tenant-scoped data from
both apps, validate cross-instance Session/logout and current account decisions,
restart one component at a time, then reconcile SQL data and Session outcomes.
Select PostgreSQL durability and Valkey persistence/re-login semantics explicitly;
AOF/RDB configuration alone is not a no-revocation-resurrection guarantee.
Lost/rolled-back Session authority must require a defined fail-closed/re-login
or equivalent safe recovery. Include bounded authority outage/refusal and
same-service recovery; do not add throughput targets or arbitrary fault matrices.

Stop owned processes and delete the exact-owned group and all VM/disks/NICs/
network/storage/Blob/private credentials. Poll deletion to actual absence and
check for owned leftovers; a submitted asynchronous delete is not cleanup proof.
Never prune shared caches or modify/delete pre-existing resources. A separate
HA/failover item requires a real selected need, mature PostgreSQL/Valkey HA
mechanisms and its own cost/topology contract; no custom consensus or mandatory
cloud product is introduced.

## Sources and evidence limits

Repository decisions above are an evaluation from current consumers and the
accepted YT-118/121/122/123/138 studies, not proof of new runtime behavior.
Primary documentation consulted on 2026-10-25:

- [Rust tracing](https://docs.rs/tracing/0.1.44/tracing/) explains spans/events and native instrumentation.
- [Tower HTTP trace](https://docs.rs/tower-http/0.7.1/tower_http/trace/) exposes optional middleware and distinct response/body/failure callbacks; consulted for comparison only, without upgrading current dependencies.
- [Moka async cache](https://docs.rs/moka/0.12.16/moka/future/struct.Cache.html) provides opt-in concurrent caching with best-effort capacity; it does not select business consistency.
- [PostgreSQL TLS](https://www.postgresql.org/docs/18/ssl-tcp.html) describes native encrypted/authenticated connection configuration, not this fixture's qualification.
- [Valkey persistence](https://valkey.io/topics/persistence/) distinguishes snapshots and append-only logging; protocol configuration does not prove application revocation recovery.
- [Azure resource groups](https://learn.microsoft.com/en-us/cli/azure/group?view=azure-cli-latest) supplies group create/delete/exists/wait operations; actual scoped cleanup is still required.

The original scheduling delivery changed planning documents only and ran only
document checks. The later separately authorized YT-181 implementation and new
focused proof are tracked in its implementation record; do not replay previous
qualification or allocate Azure resources through this planning record.
YT-181 is Done at its separately reviewed reference scope; YT-182 is Done at
its separately reviewed application-reference scope; YT-183 is Done at its
reviewed application-reference scope; YT-184 is now Done at its separately
accepted bounded cross-host/orderly-persistent-restart reference scope after final
independent Reviewer PASS. Actual proof composes generation4 Auth with generation5
remaining recovery/VM/SQL-policy/Session-epoch evidence; all owned resources are
deleted and original failures retained.
YT-096 stays In progress with D1–D6 Pending; YT-020 remains Blocked. Actual stage
activation and independent review remain separate from this assessment.
