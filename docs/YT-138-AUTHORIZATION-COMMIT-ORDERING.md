# YT-138 — Application authorization-to-commit ordering research

Date: 2026-10-23. Status: Done at documentation-only research scope after
independent Reviewer PASS, zero issues and required followups.
Source revision: `4bc8080f99388f2b6af31a90b06d2985847b68d1`.
The user activated this documentation-only research after YT-137. No runtime,
API, schema, settings or provider change is selected. The
[registered contract](PRODUCTION-DEPLOYMENT-PROFILES.md#conditional-future-work-contracts)
permits Done after an independently reviewed application-owned ordering decision
**or retention of the YT-126 race limit**; research does not require implementation.

## Selected decision and concrete consumer

Retain the **YT-126 pre-SQL authorization-to-commit race limit** for Admin/Board
native create/update and triage saves. A successful final Session/policy check
can precede revocation while SQL remains queued or running; that save can commit
after acknowledged logout or policy replacement. Neither request start nor a
successful pre-SQL check establishes atomic authorization at business commit.
No strict-write feature or mandatory framework mechanism is selected.

The concrete protected effect is creating or editing a project work item and
appending its history, including triage status/assignment edits. These are
application-owned project/write decisions. Current consumers require valid
Session identity, current account/read/write permission, correct project, CSRF,
validated fields and optimistic item-version concurrency. Source and accepted
[YT-126 evidence](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#existing-write-consistency-limit)
do not establish a requirement that logout cancel every admitted work-item save.
No newly selected payment, irreversible external effect or stricter business
operation justifies changing that contract. The retained limit is a deliberate
bounded research outcome, not proof that strict ordering is impossible or that
current saves meet it.

This follows [ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md): authorization and business
transactions belong to the application; Session/Auth remain optional native
capabilities. [YT-113](YT-113-AUTHENTICATION-CONTRACT.md#d5--oidc-mapping-and-authentication-freshness)
and [YT-124](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md#permission-changes-and-sensitive-commits)
require stronger atomic ordering before qualifying a stronger sensitive-write
guarantee. This decision retains that gate and the explicit YT-126 limitation.

## Actual paths and independent authorities

All source links below bind to the revision above; line ranges describe source
inspection, not new execution. SHA256 values are recorded below and in the
research evidence. Existing tests were inspected without running them.

| Boundary | Source and actual behavior |
| --- | --- |
| Admin composition | [Admin main](../examples/admin-dashboard/src/main.rs#L55), lines 55–78, opens Board with the selected Session backend. Admin is the same business consumer, not an independent second authorization implementation. |
| Application identity/resource authority | [Domain](../examples/lab-operations-board/src/domain.rs#L6), lines 6–46 and 91–100, binds Alice to project 1/write, Bob to project 2/write and Viewer to project 1/read only; remote state stores the local account name. [Policy](../examples/lab-operations-board/src/auth.rs#L57), lines 57–114 and 123–165, freshly reads a bounded single-host file, checks enablement, intersects read/write with compiled authority and binds the resolved identity. The policy cannot grant Viewer write or change compiled project membership. |
| Protected routes and native save | [Board](../examples/lab-operations-board/src/lib.rs#L320), lines 320–379, protects native item and triage routes under Session middleware. Lines 507–537 reread Session then policy and check project/write; lines 719–800 check again after parsing, CSRF and field validation, then pass that final principal to the shared database save. Native create is POST `/board/projects/{project}/items`; update is POST `/board/projects/{project}/items/{id}`. |
| Triage save | [Triage](../examples/lab-operations-board/src/triage.rs#L285), lines 285–336, handles POST `/board/projects/{project}/triage` for an existing positive item ID/version, validates CSRF/fields, obtains a final current principal and invokes the same database save. It returns JSON; native success prepares a redirect. |
| Business effect/commit authority | [Database](../examples/lab-operations-board/src/database.rs#L107), lines 107–125 and 172–210, admits bounded workers, locks its process-local connection and uses an immediate SQLite transaction. Update checks project/item/version and transition, increments the version; create requires expected version zero. Item and history commit together. SQL receives a copied principal and performs no Session/policy lookup. [Schema](../examples/lab-operations-board/migrations/0001.sql#L1), lines 1–27, has business version/history constraints, no authorization epoch or revocation table. |
| Logout and rotation | [Lifecycle handlers](../examples/lab-operations-board/src/lib.rs#L603), lines 603–624, await replace-and-rotate after login proof, or CSRF then destroy for logout, before preparing redirects. [Session delegation](../examples/lab-operations-board/src/sessions.rs#L138), lines 138–179 and 198–209, awaits the selected provider and maps unknown/unavailable results to failure. Logout stays outside protected Auth so a disabled account can still revoke its Session. |
| Provider scope | [Provider](../crates/yetlet/src/session_redis/mod.rs#L349), lines 349–379 and 382–458, reads authoritative Session state and applies Session CAS/rotation/revoke. [Script](../crates/yetlet/src/session_redis/session.lua#L25), lines 25–50 and 96–119, checks active record/revision and retires that record within its single-primary authority. Neither script nor CAS participates in SQLite commit. [Store](../crates/yetlet/src/session_redis/store.rs#L317), lines 317–343, explicitly distinguishes dispatched write uncertainty from cancellation. |
| Policy replacement | [Accepted file authority](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#identity-and-current-local-policy) uses trusted operator atomic replacement, without a public policy-edit endpoint or cache. [Existing fixture](../examples/admin-dashboard/tests/auth_valkey.rs#L115), lines 115–117, writes a separate file then renames it. Replacement and SQL do not share a transaction, drain or version check. |

The SQL admission wait has a one-second timeout and SQLite busy timeout is
250 ms ([database](../examples/lab-operations-board/src/database.rs#L43), lines
43–57 and 107–125). These bounds do not make the whole authorization-to-commit
interval atomic or guarantee cancellation: an admitted blocking worker owns its
permit even if the waiting request is cancelled. The local connection mutex
protects that connection; it is not a cross-process revocation barrier.

Reuse [YT-124](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md),
[YT-125](YT-125-VALKEY-AUTH-INTEGRATION.md#concrete-local-authority-and-topology)
and [YT-126](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#actual-focused-results) at their
recorded scope. YT-125 proves lifecycle/current resource-read decisions and has
no business mutations. YT-126 covers allowed saves, subsequent denied saves,
optimistic conflicts, cross-process logout and fail-closed provider/policy
behavior. None is paused-SQL-write-versus-revocation proof. No accepted workload
or historical qualification matrix is rerun or promoted here.

## Event ordering and acknowledgment

Use these distinct events when describing this consumer:

| Event | Meaning |
| --- | --- |
| `Q` | Request starts. No permission is reserved by arrival time. |
| `S`, `P`, `F` | Final authoritative Session read snapshot, policy-file snapshot and completion of the final application check. `S` and `P` are sequential independent observations; `F` is not one atomic cross-authority snapshot. |
| `B`, `C` | Business SQL begins its transaction, then commits item/history. Admission or connection waits can separate `F` from `B`. |
| `R`, `R_ack` | Relevant Session revoke takes effect in its authority, or new policy becomes visible by atomic replacement; its successful acknowledgment follows. Logout HTTP acknowledgment and operator replacement acknowledgment are different paths. No application policy-edit response or power-loss durability acknowledgment is invented. |
| `W_ack` | Caller receives the successful save response. Preparing/returning a redirect or JSON and actual delivery to the caller are different from `C`. A lost response may leave no observed acknowledgment. |

An allowed current interleaving is `Q → S → P → F → R → R_ack → B → C → W_ack`.
A started SQL worker can similarly cross `R_ack` before committing. This is the
named limit. Moreover, a policy file opened before replacement can finish
reading its old handle afterward; `F > R_ack` alone does not prove its policy
snapshot was taken after replacement. Session read and policy read also need
not describe one common instant. A protected decision that actually observes a
retired Session or denied policy stops before SQL; a new authoritative lookup
must not restore the retired identity or bypass denied/unavailable authority.

Logout revokes **one Session**. It is not account-wide disable, logout of every
Session, upstream IdP logout or cancellation of all outstanding work. A policy
update can disable an account or remove its read/write access across Sessions
using that authority; project membership currently comes from compiled local
bindings. A future project-binding change would need its own coherent policy
and writer-version decision. Neither current path rolls back a previously
committed work item. Already delivered documents and prior authorized output
are not erased by changing permission.

For any future serialized rule, a writer ordered before revocation may commit
and its response may be acknowledged after the revocation response. Response
arrival order cannot establish transaction order. If the selected promise is
specifically **no business commit after `R_ack` for the revoked scope**, then
`R_ack` must wait until earlier admitted writers have committed or definitively
aborted, and later writers must deny. Merely reserving an earlier writer order
and letting it commit after `R_ack` meets only a weaker ordering promise. The
application must choose which meaning it intends before implementation.

Failure, cancellation, timeout, crash or an unacknowledged revoke/commit can be
unknown outcomes. They are not proof of rollback, successful revocation or
permission to replay. An already committed effect remains committed even if
response preparation/delivery fails. Keep the
[YT-137 no-automatic-retry and reconciliation decision](YT-137-WRITE-OUTCOME-IDEMPOTENCY.md#reconciliation-under-the-selected-current-decision);
commit ordering does not supply operation identity, deduplication or exactly-once
execution. Current authorization still governs reconciliation/result reads.

## Conditional application choices if a strict need is selected later

These are conceptual alternatives for a separately authorized operation and
implementation, not new APIs, storage migrations or a lock service deliverable.
The writer **and every relevant revoker/policy updater** must share an
authoritative atomic ordering rule. Protecting only writers leaves the race.

| Candidate | Requirement and limit |
| --- | --- |
| Existing Valkey Session plus atomic policy file and SQLite | Retain current next-decision freshness with the named race. Atomicity within each authority does not create a joint transaction. Another reread, Session CAS, short TTL/deadline or moving the reread closer to SQL leaves a gap. |
| Application business transaction with authorization version/fence | Conceptually the effect authority must atomically validate a current, correctly scoped authorization generation while committing the business effect; relevant revocation must advance/deny that same authority in a conflicting atomic operation. A copied version from Valkey or the file compared only outside SQL is insufficient. Current item `version` protects edit concurrency, not authorization. SQLite transaction serialization alone cannot order a revocation that still occurs only in Valkey/file. |
| Application serialization shared by writers and revokers | A single authoritative order can encompass the final authorization decision, effect and relevant revoke, with acknowledgment tied to that order. All instances, workers and administrative update paths must participate. A local mutex, advisory lock or distributed lease alone is insufficient: paused/expired owners must be unable to commit stale work, and the business authority must enforce fencing or an equivalent exclusion guarantee. Independent Valkey lock acquisition and SQL commit do not supply it. |

Before selecting either stronger option, bind the following to the concrete
operation rather than to generic Principal or request middleware:

- Define actor, action, resource/project and revocation scope: one Session
  logout/rotation/expiry, account disable, read/write removal or project change
  have different affected sets. State which authorities and replicas participate;
  a per-account fence does not automatically implement per-Session logout.
- Define the atomic ordering point, `R_ack` and `W_ack`, including queued,
  admitted-but-not-started and running writers. Specify whether earlier ordered
  writers finish, abort or delay revoke acknowledgment. An open SQL transaction
  alone cannot preserve independent remote authority freshness.
- Deny before effect when the selected authority cannot establish current allow.
  Specify crash/disconnect/commit-unknown and revocation-unknown reconciliation;
  do not acknowledge success from a timeout or replay a possibly committed save.
- Exclude old binaries, mixed-version workers, background/direct SQL writers and
  operator replacements that bypass the rule. A fence accepted only by new
  writers cannot protect effects from stale writers.
- Prevent epoch ABA: disable then reenable must not make an old allow/version
  valid again. Define monotonic generation/history identity, exhaustion/reset,
  restore rollback and recovery/re-login behavior without resurrecting revoked
  state. Session revisions, cookie issuance and item versions are not an
  interchangeable authorization epoch. Concrete topology/persistence/recovery
  integration belongs to separately activated YT-139.
- Keep Session authentication, CSRF and current resource/action authorization,
  including result/reconciliation access. Treat identity/fence material as
  server-owned bounded state; do not expose Session IDs, credentials, policy
  contents or cross-account resource existence through logs, SSR or errors.

No production-owner, SLO or real topology choice blocks the retained-limit
research decision. No instant revocation, HA, safe shared-filesystem deployment
or cross-store durability follows from a conceptual alternative.

## Minimal later evidence and current review boundary

Only if implementation is separately authorized, exercise the changed consumer
with deterministic bounded interleavings: pause a writer before its authoritative
ordering point, acknowledge the relevant revoke, then release and prove denial
before item/history effect; order a writer first and prove the selected finish/
abort/revoke-ack behavior while permitting a later response acknowledgment;
exercise queued/started writers, authority failure and commit/revoke-unknown
outcomes; reject stale fences and any selected mixed-version/recovery path.
Inspect the actual business effect and history under current read authorization,
not response status alone. Scope the cases to the selected guarantee and its
participating authorities. No new validation infrastructure, historical matrix
rerun, browser/service launch or paused-write proof is authorized by this study.

Read-only findings and exact source hashes are retained outside Git at
`/tmp/yetlet-yt138-write-ordering-20261023-uk4u9qlz/source-findings.json` and
`research-summary.json`. Hashes are SHA256 over the bound revision's file bytes;
source revisions and line ranges are recorded per finding, including contracts.
The runtime-source manifest is:

| Bound source path | SHA256 |
| --- | --- |
| `examples/admin-dashboard/src/main.rs` | `58b2e1e5e7c479a99e78465fa8e74f70510f0ca76e66813c2d7cb7f4531a16d9` |
| `examples/lab-operations-board/src/auth.rs` | `a55b4d2ef2f8e18435958c038a010bcc8ccab1b98641f59497371db1977b3fde` |
| `examples/lab-operations-board/src/lib.rs` | `6d7da3ed61ffbcb8c5986c365178ccc6185bd0526973a8c1f8ef468242b90dad` |
| `examples/lab-operations-board/src/triage.rs` | `7251c8b607ae2457d32b1b9aa25315075dbeb135e32aaa00dc88526cc95533f0` |
| `examples/lab-operations-board/src/database.rs` | `a600bbf72ad6584b397d06c6c265e448a3ee1a895dfefc0d8f5d1df5d5787776` |
| `examples/lab-operations-board/src/sessions.rs` | `92c1790c33ea9c89227c50f7fb118cd1aef875b11db00bc1dc14a8dc563f3170` |
| `examples/lab-operations-board/src/domain.rs` | `ecfa95aa16b6bed5d96f32b0d70443c2b8b7416c26d774b41f7d7c88892cb2d4` |
| `examples/lab-operations-board/migrations/0001.sql` | `6125a8205d5f3cd94e4abd4470efe17994ceadd4d318986a9d70dc278c172a98` |
| `crates/yetlet-session-redis/src/lib.rs` | `a67e5fd434aebb58c39f8bf5e92cdeae7a27f7bf93d2aad8e7cf89333b5bcf75` |
| `crates/yetlet-session-redis/src/store.rs` | `877d96f3b5175264536f259422a664de67469cdba01d6f80cd6a1fefc89e8816` |
| `crates/yetlet-session-redis/src/session.lua` | `19f6c256c6eddbda51cb5c2dd96935b100e37c7dd7cf92b62730cfbbaf84cfc3` |
| `examples/admin-dashboard/tests/auth_valkey.rs` | `442b5396df4e9eb877a67ca1a3962aaf0fbec6a89bc94877f7ffe8a77e394e21` |

No tests, builds, services, browser sessions, workloads, cloud operations,
signing, release, commit or push were started for this research. Nothing needs
process/service teardown; bounded source/hash commands completed. Repository
documentation and whitespace checks passed: 75 required documents and 141
backlog items. Independent Reviewer
`/root/yt138_ordering_independent_review_20261023` returned PASS, zero issues,
required followups and optional notes; all six reviewed file hashes matched
before and after review. The integrating parent's acceptance changes only
completion status and evidence references. The retained `review-input.json`,
`review.patch` and `review-final.json` preserve the original review snapshot and
result; source decisions and runtime remain unchanged.

Open choices are conditional strict operation/scope/ordering/acknowledgment and
its separately authorized implementation, not prerequisites to accept retaining
the limit. The next registered conditional item is YT-139's optional multi-instance
authority/recovery reference, only after separate activation. YT-139–YT-141 remain
Todo and unactivated; YT-096 D1–D6 remain Pending, YT-020 Blocked and immutable M20
candidate `0ce56e08` with its original nine libraries unchanged.
