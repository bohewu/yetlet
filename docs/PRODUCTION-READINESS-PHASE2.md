# Production Readiness Phase 2 — bounded application capabilities

Date: 2026-10-23. Status: registration accepted; YT-127 is **Done** at bounded A1 scope; YT-128 is **Done** after independent Reviewer PASS; YT-129 is **Done** at [independently reviewed local load/recovery scope](YT-129-LOAD-SOAK-RECOVERY.md); YT-130 is **Done** at [independently reviewed local restore/runbook scope](YT-130-BACKUP-RESTORE.md); YT-131 is **Done** at independently reviewed local C1 scope.
Independent registration Reviewer PASS, zero issues and required followups. This document consolidates accepted application work and remaining gaps, without claiming production readiness.

## Baseline and evidence boundaries

T0 rechecked branch `docs/yt113-123-opt-in-capability-plan`, HEAD
`5db8b8c61b6d319a641814dff4af493ce3e5d965`, and a clean worktree before these
three documentation edits. Ignored pipeline observations are local execution
artifacts. A final repository search found no YT-127–YT-131 references before
registration. The T0 patch changed this document, [Backlog](BACKLOG.md)
and [Roadmap](ROADMAP.md); T0 inspected runtime source at the stated baseline.
Exact pre-edit observations and source SHA-256 identities are retained in
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T0/baseline.json`.
Historical PASS results below retain their original source and environment;
no historical runtime, browser/CSP or deployment qualification was rerun for T0.

The previously registered backlog has YT-020 **Blocked** and YT-096 **In progress**;
other existing task rows are Done at their selected scopes. YT-096 D1–D6 remain
**Pending**, and immutable release candidate `ce56e08` and its original nine
experimental libraries remain unchanged. M19 remains partial; M20 release gates
are separate. M21 implementation and research completion does not implement
all candidate providers. See [release readiness](RELEASE-READINESS.md),
[current capability map](CAPABILITIES.md) and the [M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md).

The handoff's generator defect is stale: [YT-102 source closure](YT-102-GENERATOR-SOURCE-CLOSURE.md)
records the bounded repair and independent acceptance. It is excluded here.
YT-126's documented authorization-check-to-SQL race remains an accepted limit;
sensitive-write transaction/fencing changes need separate tracking and authority.

## Baseline coverage and gap matrix

All source paths in this table are under `examples/production-starter/` unless
linked otherwise. This table preserves the T0 pre-implementation baseline; the final matrix below records accepted results. Inspected baseline tests identify existing contracts, not fresh execution.

| Capability | Existing source, tests and retained evidence | Missing requested capability / affected boundary |
|---|---|---|
| A1 / YT-127 | `src/store.rs`: four SQL permits held through `spawn_blocking`, two-second SQLite busy timeout; unit tests protect migration/reopen, conflicts and readiness damage. `src/main.rs` owns signals/readiness/Axum drain; `tests/http.rs` protects API, Session/CSRF and CSP. [Starter guide](../examples/production-starter/README.md), [deadline study](YT-029-HTTP-DEADLINE-RESEARCH.md) | `acquire_owned().await` has no explicit wait bound or overload result. Need application admission, deadline/cancellation outcomes and overload/shutdown cleanup across Store, HTTP, startup and typed settings; readiness shares Store capacity. SQL busy timeout and [YT-117 rate limiting](YT-117-RATE-LIMIT.md) do not prove capacity protection |
| A2 / YT-128 | `src/web.rs::observe` emits JSON method/status/duration/process-local request ID; excludes URI/query/headers/bodies/errors. [YT-123](YT-123-TRACING-CORRELATION-STUDY.md) is accepted research with no new tracing runtime/API selected | No collection endpoint/equivalent or SQL wait/saturation diagnostics. Observer ends at Response return, not body completion/commit. Need opt-in app/configuration/Store/lifecycle instrumentation and real fault diagnosis |
| B1 / YT-129 | Existing HTTP/Store tests and [YT-074](YT-074-QUALIFICATION.md) provide adjacent bounded evidence | No Starter workload, predefined thresholds, soak/recovery record or capacity range. YT-074 is a different application. Need isolated existing-tool workload and source/environment-bound measurements, without new general validation infrastructure |
| B2 / YT-130 | WAL/schema version 1 and startup rejection in `src/store.rs`; Starter guide describes supported backup or stopped-process copy and separate restore. [YT-066 responsibility boundary](YT-066-PERSISTENCE-RECOVERY-PLAN.md) and [YT-094 closeout](YT-094-CLOSEOUT.md) retain provider/application recovery evidence | No Starter fresh-instance business-data restore/cutover runbook or measured RPO/RTO. Need selected local configuration, consistent backup, schema/data/readiness checks and explicit Session/Valkey policy; do not recertify SQLite or transfer other-app recovery claims |
| C1 / YT-131 | `migrations/0001_init.sql`, fixed version checks and unsupported-schema tests. [Accepted YT-103](YT-103-ROLLING-ASSET-VERIFICATION.md) proves selected asset retention/update/rollback only | No real Starter v1-to-v2 API/schema/data scenario or mixed-version evidence. Need application migration/HTTP fixtures, stale-client policy and rollback/forward-fix decision; reuse unchanged YT-103 evidence without rerun or expansion |

## Operational and architectural boundaries

The [architecture](ARCHITECTURE.md), [ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md) remain authoritative.
Start with application-owned native Axum/Tower/Tokio composition in the existing
Starter. Extraction requires demonstrated cross-application need. Features stay
opt-in; Core/HTML/DOM/browser WASM acquire no server-only dependencies or serialized
secrets/capabilities. Preserve source compatibility and existing SQL/API,
Session/Auth/CSRF, CSP and escaping behavior. No mandatory DI, Controller, MVC,
registry, metrics, second ORM/tracing runtime/HTTP server/middleware engine,
generic timeout or migration framework is selected. Memory/Redis Session APIs
need not be unified.

Current Starter operation stays loopback-only with anonymous operator SSR pages,
a separate shared Bearer API credential, process-local Sessions, local SQLite
and no existing long-lived route. It is not a public service. Capacity admission
limits resource concurrency/waiting; rate limiting controls arrivals over time.
An HTTP timeout/cancelled caller cannot be presumed to stop started blocking SQL,
undo a commit or permit a safe write retry. Observation must distinguish caller
outcome, worker completion and durable business result. Existing startup/readiness
and graceful shutdown ownership remain explicit.

Apply [threat-model telemetry rules](THREAT-MODEL.md#13-logging-audit-metrics-and-privacy):
never record credentials, raw bodies, Session IDs or arbitrary request fields;
never label metrics by full URL, request ID or user ID. Operators own protected
collection, retention and external sinks. HTTP 200, Response return or span
completion does not prove durable business-write success.

Work is sequential in this checkout: registration review, then
**YT-127 → YT-128 → YT-129 → YT-130 → YT-131**. Fully validate and independently
review each predecessor before starting its successor. Each delivery records
actual changed files, decisions/tradeoffs, source hashes/branch/HEAD/dirty state,
commands/exits, focused compile/runtime/browser/CSP evidence separately, resource
cleanup, limitations and next unblockable item. Independent review inspects the
actual diff, affected consumers and evidence; unresolved material P0–P2 findings
block acceptance. An explicitly labeled self-review never satisfies the independent
gate. No state becomes Done from registration or implementation alone.

Use only bounded local or explicitly authorized isolated synthetic work.
No new generic validation infrastructure is authorized. Stop affected work on
new major architecture, external deployment, paid resources or security/data-policy
changes and retain a concrete decision point. No branch/worktree change, staging,
commit, push, signing, release, publication or deployment is authorized here.
Multi-host HA, multi-region, external certification, SLA and stable public API
claims remain excluded. Unchanged historical qualification is reused, not repeated.

## Registered work contracts

### YT-127 — A1 Request admission and overload protection

- **Goal / scope:** Small app-owned admission slice with bounded concurrency and
  permit waiting, consistent explicit overload rejection, request/operation deadline
  responsibility, cancellation and started blocking-work semantics. Cover shutdown
  during overload, readiness behavior and necessary long-lived-route exceptions;
  explicitly state if none apply. Keep overload distinct from rate limiting.
- **Dependencies:** YT-092 and YT-029; independently accepted T0 registration first.
- **Non-goals:** Generic core timeout API, distributed limiter, new HTTP engine,
  new long-lived route, release/deployment or transaction/fencing redesign.
- **Acceptance:** High concurrency cannot create unbounded waiting; normal load
  has no unjustified rejection. Overload responses are explicit and consistent.
  Demonstrate a blocking DB operation that can finish after caller timeout with
  correct result/retry semantics, overload recovery and shutdown cleanup; preserve
  SQL/API/Session/CSRF and security behavior.
- **Required evidence:** Focused unit/integration and actual localhost overload,
  timeout, recovery and shutdown observations tied to source/configuration;
  relevant fmt/Clippy/docs results and owned-process/resource teardown.
- **Known limitations:** Started blocking work may outlive its caller. Local
  admission proves no distributed capacity, rate quota or HA property.
- **Reviewer gate / state:** Independent diff/consumer/evidence PASS with no
  unresolved P0–P2 findings before acceptance or YT-128. **Done** at bounded A1 scope.
  [Implementation and local verification](YT-127-ADMISSION-OVERLOAD.md) and independent Reviewer PASS have zero issues and required followups.

### YT-128 — A2 Opt-in application observability

- **Goal / scope:** Usable metrics endpoint or equivalent collection mechanism
  using native ecosystem composition. Consider throughput, active requests,
  status/latency, admission rejection, SQL wait/saturation, readiness transitions
  and shutdown lifecycle; document precise observation lifetimes and omissions.
- **Dependencies:** YT-127 fully validated and independently accepted; YT-123 research.
- **Non-goals:** Mandatory metrics, framework subscriber/exporter, second tracing
  runtime, distributed tracing product, audit durability or Core/browser changes.
- **Acceptance:** At least one real fault is observable and diagnosable; bounded
  label policy, disable path and Core/browser isolation are demonstrated. Collection
  is usable with operating/limitations guidance and no forbidden sensitive fields
  or high-cardinality labels. Response/span success is separate from durable commit.
- **Required evidence:** Focused collection/disabled-mode/privacy tests, real fault
  observations and diagnosis, source/configuration-bound outputs, relevant static/docs
  checks, dependency isolation and owned-resource cleanup.
- **Known limitations:** App-owned Prometheus registry and finite labels are implemented and independently accepted at local A2 scope;
  app-local telemetry alone establishes no cross-service tracing or durable audit.
- **Reviewer gate / state:** Independent diff/consumer/evidence PASS with no
  unresolved P0–P2 findings before acceptance or YT-129. **Done**. [Local verification and preserved failure history](YT-128-OBSERVABILITY.md): concurrent tests and actual localhost fault/correlation/privacy/recovery/cleanup pass; independent Reviewer PASS, zero issues and required followups.

### YT-129 — B1 Load, soak and recovery qualification

- **Goal / scope:** Reproducible Starter synthetic workload in a declared isolated
  local production-like configuration using existing tools. Predeclare environment,
  workload/duration and numerical acceptance thresholds before execution. Cover
  normal baseline, step increases, sustained load and a full overload/recovery cycle.
- **Dependencies:** YT-128 independently accepted; transitively YT-127.
- **Non-goals:** General load harness/validator, production traffic, unrelated old
  qualification reruns, SQLite engine recertification, multi-host HA or SLA.
- **Acceptance:** Collect throughput, p50/p95/p99, errors/rejections, CPU/RSS, SQL
  admission wait and readiness/recovery; measure connections/tasks/resource release
  where needed. State a data-supported capacity range and uncovered deployment,
  OS/storage conditions against predefined thresholds.
- **Required evidence:** Exact source/configuration/environment/workload/thresholds,
  commands/exits and raw measurements/aggregates, complete recovery and cleanup;
  existing checks only as relevant to changed application artifacts.
- **Known limitations:** [Local declaration and measurements](YT-129-LOAD-SOAK-RECOVERY.md) cover a short debug/tmpfs run only: passing paced normal points 43.64–467.01 rps and sustained 153.08 rps for 45 seconds; eight-client rejection/CPU failures remain visible. Persistent storage, long soak and general deployment capacity remain uncovered.
- **Reviewer gate / state:** Independent workload/threshold/source/evidence review
  PASS with no unresolved P0–P2 findings before acceptance or YT-130. **Done**: one predeclared load/overload/recovery execution and cleanup complete; independent Reviewer PASS, zero issues and required followups.

### YT-130 — B2 Application backup, restore and recovery

- **Goal / scope:** Selected production-like local configuration, consistent backup,
  fresh isolated restore, schema/business-data integrity, startup/readiness and
  operator restore/cutover procedure. State Session/Valkey persistence policy and
  measure actual recovery duration and potential data loss.
- **Dependencies:** YT-129 independently accepted; reuse YT-066/YT-094 limits.
- **Non-goals:** SQLite engine recertification, mandatory storage dependency,
  cross-host DR/HA, external backup service activation or new security/data policy.
- **Acceptance:** Restore into a fresh instance, verify critical data and version,
  record measured RPO/RTO, and explicitly refuse service or stop cutover on failure.
  Complete an operator runbook with safe decision points and Session policy.
- **Required evidence:** Selected configuration, consistent-backup method, source-bound
  before/after schema/data/readiness observations, successful and refused cutover,
  RPO/RTO timings, exact commands/exits and isolated-instance teardown.
- **Known limitations:** Starter Sessions currently expire on restart; existing
  nonpersistent Valkey proof is not restore qualification. Destination/persistence
  choices remain open; protection/retention and production operations need owners.
- **Reviewer gate / state:** Independent runbook/data-integrity/failure/evidence
  PASS with no unresolved P0–P2 findings before acceptance or YT-131. **Done**: [local runbook and fresh restore/refusal evidence](YT-130-BACKUP-RESTORE.md), full schema/data/sequence checks and measured RPO/RTO complete; independent Reviewer PASS, zero issues and required followups.

### YT-131 — C1 Mixed-version schema, API and asset compatibility

- **Goal / scope:** Select one real app v1-to-v2 migration and reproduce old
  frontend/new API, old binary/new schema, new binary/existing data and mixed
  versions. Cover forward-only migration, stale clients/assets and explicit
  rollback versus forward-fix conditions beyond accepted YT-103 asset evidence.
- **Dependencies:** YT-130 independently accepted; YT-103 retained asset proof.
- **Non-goals:** Repeating or expanding YT-103 qualification, generic migration
  framework, production rolling deployment or stable public compatibility promise.
- **Acceptance:** Reproduce mixed-version behavior; incompatibility is explicitly
  refused or has a verified alternative. Demonstrate no silent data corruption,
  forward-only handling and concrete rollback/forward-fix applicability.
- **Required evidence:** Version/source/schema/API fixture identities, exact
  mixed-version commands/results and critical data assertions, stale-client
  behavior, refusal/recovery paths, decision record and owned-instance cleanup.
- **Known limitations:** V2 adds a status-filtered API and schema-2 index;
  retained v1 refuses schema 2 at startup. Asset retention cannot
  establish API/schema compatibility or multi-host deployment availability.
- **Reviewer gate / state:** Independent version/data/consumer/evidence PASS with
  no unresolved P0–P2 findings before acceptance. **Done** at [local C1 scope](YT-131-MIXED-VERSION-COMPATIBILITY.md), after independent Reviewer PASS, zero issues and required followups, and controller acceptance.

## Final Production Readiness Matrix

Implemented means the bounded capability exists. Verified identifies source-bound
execution and independent acceptance. Deployment-qualified records where those
observations apply; none of these local exercises qualifies a production deployment.
All five items are Done at their stated scopes after independent Reviewer PASS
with zero issues and zero required followups. Historical failures remain evidence.

| Capability | Implemented | Verified | Deployment-qualified | Known gap |
|---|---|---|---|---|
| Admission/overload / YT-127 (A1) | App-owned HTTP capacity 32, SQL capacity 4, zero permit queues; explicit 503/504 and drain. Cooperative request deadline 5 s, readiness 1 s, shutdown grace 10 s; SQLite busy timeout 2 s | [A1 delivery](YT-127-ADMISSION-OVERLOAD.md): 10 unit + 10 HTTP tests, normal writes, actual overload/recovery, stalled-upload deadlines and successful/failed shutdown. Independent T1 PASS. C1's later focused regression preserves A1 behavior; original fault proofs remain bound to T1 source/binary | No; original single-host Linux/debug/localhost fault configuration only | No TCP/header/body transport bounds, operation/filesystem watchdog, rate quota, distributed admission or deployment capacity. Started blocking writes can commit after timeout/disconnect; creates lack idempotency |
| Observability / YT-128 (A2) | Default-off app-local prometheus 0.14.0 registry; `APP_METRICS__ENABLED=true` or typed settings enables `/metrics`; native HTTP/SQL correlation, bounded labels, SQL late-completion accounting and lifecycle events | [A2 delivery and preserved failure/resume](YT-128-OBSERVABILITY.md): 11 unit tests plus corrected 11-thread/11-test HTTP suite; real SQLite contention, enabled/disabled collection, privacy, recovery and shutdown. Independent T2 PASS; no Core/HTML/DOM/browser dependency expansion | No; loopback Production settings/debug binary and synthetic fault only | Volatile process-local snapshots; response creation is not body completion or durable commit. Zero permit wait is immediate admission, not SQLite lock wait. Protected collection, retention, external sinks and cross-service propagation need deployment decisions |
| Load/soak/recovery / YT-129 (B1) | Predeclared closed-loop synthetic workload and [source-bound capacity/recovery record](YT-129-LOAD-SOAK-RECOVERY.md); no runtime change | One run, 42,043 requests: passing paced normal points 43.64–467.01 rps; two-client 45 s sustained phase 153.08 rps, p99 5.80 ms. Eight-client phase FAIL: 1.514% rejection and 341.73% CPU; excluded from passing range. Forced-lock overload/recovery and teardown complete; independent T3 PASS | No; single debug process, co-located clients/monitor, Linux host, `/tmp` tmpfs database, historical schema 1/A2 executable | No arrival-rate upper-capacity bound, release-build/persistent-disk capacity, long soak, HA or SLA. Short closed-loop results do not establish deployment limits or transfer to C1 v2 |
| Backup/restore / YT-130 (B2) | [Operator backup/fresh-restore/cutover runbook](YT-130-BACKUP-RESTORE.md); existing supported SQLite backup, no runtime change | Historical v1 debug/ext4: three complete rows, full schema/versions/sequence restored; readiness/API 200. RPO proxy age 41.30 ms, acknowledged-write exposure 39.26 ms, two excluded acknowledged mutations; RTO 52.36 ms through local cutover eligibility. Schema 999 refused before bind; old Session cookie 403; independent T4 PASS | No; serial synthetic local v1/schema-1 instances on ext4, source writer paused for backup | No v2 recovery requalification, external traffic switch, storage-loss/power-loss durability, backup retention/security or production RPO/RTO. Memory Sessions are lost on restart; earlier nonpersistent Valkey evidence does not establish recovery |
| Mixed-version compatibility / YT-131 (C1) | Schema-2 status index plus additive Bearer-protected `/api/v2/issues?status=...`; existing wire/source contracts retained; [upgrade/drain/rollback procedure](YT-131-MIXED-VERSION-COMPATIBILITY.md#upgrade-rollback-and-forward-fix-conditions) | 25 tests (13 unit/12 HTTP) and actual 26-response old/new matrix; exact rows/schema/sequence, refusal, stale Session 403 then fresh-session old form 303, stale version 409, restart/forward recovery. Retained v1 binary/source and v2 identities; independent T5 PASS and controller acceptance | No; synthetic local debug binaries, deliberate two-process overlap only for compatibility observation; unchanged YT-103 asset proof retained without rerun | Old live process becomes unready but still serves legacy writes: explicit removal/drain before cutover required. Old restart on schema 2 refuses service. Backup rollback loses newer writes; preserve schema 2 and forward-fix after new writes. No zero-downtime/rolling deployment or generic migration qualification |

## Delivery records and source bindings

All execution used branch `docs/yt113-123-opt-in-capability-plan`, HEAD
`5db8b8c61b6d319a641814dff4af493ce3e5d965`, with sequential accepted dirty
changes in the same checkout. HEAD alone does not identify tested input. Each
linked item record contains actual changed files, decisions/tradeoffs, exact
commands/results, limitations and teardown. Per-item source hashes and retained
executable identities establish each historical scope; later documentation
acceptance edits do not rewrite those records or extend their qualification.

Local run root: `.pipeline-output/prod-readiness-phase2-20261023T020406Z/`.
The following relative paths are under that root:

| Item delivery | Execution / source identity | Independent review |
|---|---|---|
| [YT-127 / A1](YT-127-ADMISSION-OVERLOAD.md) | `observations/T1/task-result.json`, `source-identity.json`, `commands.json`; executable `06624507302c6757c7ade2e69acd44e7de9e9433939b47f9df1eb1d7f3e44540` | `observations/T1/independent-review.json`: PASS, zero issues/followups |
| [YT-128 / A2](YT-128-OBSERVABILITY.md) | `observations/T2/task-result.json` (initial blocked result), `debugger-diagnosis-1.json`, `resume-task-result.json`, `resume-source.json`, `resume-local-command.txt`; preserved initial parallel tracing failure, user-authorized +5 same-run continuation and one in-place tracing-fixture correction | `observations/T2/independent-review.json`: PASS, zero issues/followups |
| [YT-129 / B1](YT-129-LOAD-SOAK-RECOVERY.md) | `observations/T3/task-result.json`; `observations/b1/source-environment-settings.json`, `predeclared-workload-thresholds.json`, `exact-workload-command.txt`, `threshold-analysis.json`, `after-source-cleanup.json` | `observations/T3/independent-review.json`: PASS, zero issues/followups |
| [YT-130 / B2](YT-130-BACKUP-RESTORE.md) | `observations/T4/task-result.json`; `observations/b2/source-environment-settings.json`, `exact-recovery-command.txt`, `derived-recovery-timings.json`, `after-source-cleanup.json` | `observations/T4/independent-review.json`: PASS, zero issues/followups |
| [YT-131 / C1](YT-131-MIXED-VERSION-COMPATIBILITY.md) | `observations/T5/task-result.json`; `observations/c1/v1-identity.json`, `v1-source-manifest.json`, `final-source.json`, `exact-matrix-command.txt`, `matrix-result.json`, `cleanup.json` | `observations/T5/independent-review.json`: PASS, zero issues/followups; `observations/T5/review-acceptance.json`: controller accepted |

A2/B1/B2 share retained v1 executable SHA-256
`4dc7dec913722dbcf86a8677790e69d20fa27a3676e855e31fd9f238a40fcb29`.
C1 retains that binary and its 988-entry source archive SHA-256
`bf384f6b96002016b5a3097cf7d428f8abd8926ce85eb09820f6bd0d166f1d38`;
v2 executable SHA-256 is
`fab124a026258d3a273b862ff01d9b4abb8b15e7022ecddb98dc5a08695cfde0`.
These separate identities prevent v1 capacity/recovery observations from being
promoted to v2 qualification. C1's test/build/Clippy/fmt logs establish its
current focused regression evidence; the older A1/A2 actual faults were not replayed.

All per-item cleanup completed: owned processes reaped, listeners/connections
closed and temporary runtime data removed. C1's two private synthetic SQLite
snapshots and version fixtures remain deliberately retained evidence, not active
resources. No browser/Windows/CSP campaign, YT-103 asset rerun or new deployment
qualification was performed during consolidation.

T6 changes documentation and acceptance bookkeeping only. Its source identity,
changed-file list and documentation check results are recorded in
`observations/T6/task-result.json`. The controller owns aggregate checks and the
final independent consolidation review record at
`observations/T6/independent-review.json`; per-item acceptance does not itself
attest that separate final review.

The next remaining decision is **YT-096 D1–D6**, all Pending. YT-096 remains
In progress, YT-020 Blocked, and candidate `ce56e08` immutable and unchanged.
No automatic signing, release, registry activation or deployment follows this
matrix. No general production, HA, SLA or release qualification is claimed.
