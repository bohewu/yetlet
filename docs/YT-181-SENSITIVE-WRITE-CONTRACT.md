# YT-181 — Minimal sensitive-write and revocation contract

Date: 2026-10-25 (Asia/Taipei). Status: **Contract accepted after independent Reviewer PASS, zero issues/
required followups. The later implementation stage is tracked separately below.**

## Authority, baseline and selected minimum

The original contract-stage execution/status statements below retain their
historical scope. The later authorized implementation is tracked in the final
section and its separate report; contract selection/invariants remain binding.

The maintainer selects documentation-only contract work after committing the
four [follow-up planning documents](YT-181-184-APPLICATION-FOLLOWUPS.md) as
`80d17a4`, strict/builtin Commit Guard PASS, zero findings, tree
`ec0c4f56603eb703cb15ab6de5787663d7d4245f`; no Push. Preserve native middleware,
typed State/Extensions and ordinary domain functions. No generic AOP, Auth
crate/API, DI container, runtime, migration, test or cloud allocation is added.
Reuse accepted evidence without replay. New changes remain uncommitted/unpushed.

Select **one additional SQL-owned write authority for a local account/project**,
initially the real Alice/project-one item-save flow. Its revoke removes this
account/project's permission to commit item writes in that authority. The
concrete effect is the existing item mutation plus its transactional history
append in Board/Admin `Database::save`; native create/update and triage all
reach that primitive and must participate when the optional mode is enabled.
This is one domain commit boundary, not three independent interception systems.

The chosen guarantee applies to revocation through this **new SQL write
authority**. It does not strengthen one-Session logout, rotation/expiry, IdP
logout, policy-file replacement or account disable performed only in the old
file/Valkey authority. Their documented [YT-138 race](YT-138-AUTHORIZATION-COMMIT-ORDERING.md)
remains. Those paths must not be described as atomic SQL revocation; applications
requiring that scope would have to integrate it into the same authority under
a separately selected contract. No currently running code gains this guarantee.

## What exists and what must change later

| Current source | Observed behavior | Contract consequence |
| --- | --- | --- |
| [Board user/save paths](../examples/lab-operations-board/src/lib.rs), [triage save](../examples/lab-operations-board/src/triage.rs) | Recheck Session/current file policy after input/CSRF checks, then pass a copied Principal to `Database::save` | Preserve all checks. Middleware/attribute wrapping or one more remote reread cannot make a later SQL commit atomic with a different authority. |
| [Database](../examples/lab-operations-board/src/database.rs), `run`, `save` | Four admitted workers, one-second admission timeout, 250ms SQLite busy timeout; worker owns admission after caller cancellation. Item/history commit in one `TransactionBehavior::Immediate` transaction | Keep those bounds and worker/drain ownership. Add the selected SQL grant check to the same future effect transaction, before any item/history mutation. |
| [Migration](../examples/lab-operations-board/migrations/0001.sql) and open path | Schema v1, item edit version and history, no authorization generation; current binaries reject `user_version > 1` | An item version is not an authorization version. A separate opt-in reference format is needed; a version marker alone does not exclude already-open legacy writers. |
| [Policy/identity](../examples/lab-operations-board/src/auth.rs), [local bindings](../examples/lab-operations-board/src/domain.rs) | Fresh file policy intersects compiled account/project capability; identity alone grants no business permission | SQL permission is an additional deny boundary. It cannot turn Viewer into a writer or broaden existing identity, project or file-policy permissions. |

The selection is SQL-owned authorization generations rather than a distributed
lease or application mutex: selected writers and revokers can share the actual
business transaction authority. Same-host multiple processes/connections can
participate in that order. No Valkey/SQLite distributed transaction, NFS/shared
SQL assumption, cross-host support or new lock service is selected.

## Authority row and server-owned authorization ticket

Conceptually each supported `(local account, project)` has a SQL-owned
`(write_enabled, generation)` row. Names here describe a private application
contract, not a new public Rust API or configuration schema. The fixture
bootstraps only explicitly approved Alice/project-one permission on a fresh
owned reference database; other/missing scopes deny. A request cannot create,
reset or enable a row. Reopening an existing database must not reseed permission.

After normal identity/resource/input/CSRF validation, acquire a current committed
SQL grant snapshot through the existing bounded Store path. If allowed, retain
a private authorization ticket bound to the exact Store lifetime, local account,
project and generation for **one server-side save attempt**. Never accept it
from HTTP input, cookie, hydration, untrusted headers or a client-provided version;
do not serialize, log, cache, persist or reuse it as a general capability.

The ticket is a stale-work detector, not a lock or a reservation. Before effect,
a fresh **IMMEDIATE write transaction** must read the authority row and require:
matching account/project/Store binding, enabled permission and exact expected
generation. Perform this check and the item-version/domain checks, item update
or insert and single history append in the same transaction. No network, policy
file read or await is placed inside the synchronous SQL transaction. A missing,
disabled or mismatched grant changes neither item nor history. The ticket cannot
replace the normal Principal/CSRF checks or the independent item edit version.

A trusted application operation revokes the selected account/project by setting
write-disabled and advancing its generation in an IMMEDIATE transaction on that
same database. No new public revoke endpoint is selected. Bootstrap, revoke and
re-enable remain explicit trusted application/control functions using the same
bounded SQL ownership, not raw caller-supplied roles or arbitrary policy edits.
A re-enable cannot broaden the existing compiled/file allow and must advance
again; an old ticket stays invalid across revoke/re-enable (ABA).

Generations are positive checked signed-64-bit values. Reserve the maximum for
a terminal disabled row; it must never issue an allowed ticket. A revoke from
the last active generation may enter that disabled terminal state; re-enable
there is refused. Already-disabled no-op revocation may confirm the committed
deny without resetting a generation. Never wrap, delete/recreate an active
scope to reuse a generation, or downgrade to legacy writes on exhaustion/error.

## Ordering promise and acknowledgments

Use `T` for issuing the SQL ticket, `B` for acquiring the effect write transaction,
`Cw` for item/history commit, `Cr` for selected SQL revoke commit, `R_ack` for
successful application acknowledgment of that revoke and `W_ack` for observing
a successful save response. Response arrival time is not transaction order.

**For a ticket issued from a generation invalidated by `Cr`, its protected item/
history effect cannot commit after `Cr`, including after `R_ack`.** Normal
re-enable creates a new generation; only a fresh allowed ticket can use it.
This is a scoped SQL-ordering promise, not cancellation of every HTTP request
that happened to start earlier or erasure of already delivered data.

| Interleaving | Required result |
| --- | --- |
| `T(old) → Cr → R_ack → B(old)` | Transaction sees deny/new generation and rejects before item/history changes. |
| Old writer acquires its IMMEDIATE transaction before revoker | Revoker cannot commit/ack while that conflicting writer remains unresolved. Writer may commit or roll back first; revoker then commits, or returns bounded busy/failure without a success acknowledgment. |
| `Cw → Cr → R_ack → W_ack` | Allowed: save response arrives late, but the effect committed before revoke. Do not claim response order proves a violation. |
| `T(old) → Cr → re-enable(new) → B(old)` | Old ticket rejects despite the currently enabled row; a fresh ticket may write under all current checks. |
| Authority read/transaction/schema unavailable | Fail closed for new effects; never fall back to a copied Principal or file-only allow. |

`R_ack` is allowed only after a known successful SQL commit of the scoped deny
(or confirmed already-committed terminal deny), not after enqueuing, beginning
or timing out the operation. `W_ack` requires known item/history commit. If a
caller disconnects or loses a result, the worker may still finish; timeout does
not prove rollback or revoke success. An unknown outcome is not permission to
retry automatically. Preserve [YT-137 reconciliation](YT-137-WRITE-OUTCOME-IDEMPOTENCY.md)
and current authorization of any follow-up read; no exactly-once, deduplication
or new idempotency API is selected.

## Coverage, optional adoption and recovery

Future implementation belongs to the Board/Admin application under an explicit
default-off feature/startup selection, with descriptive private naming following
[NAMING](NAMING.md). There is no new Core field, generic policy registry or
mandatory Settings provider. Feature-off keeps existing schema-v1 behavior and
its known race; feature-on never silently accepts a legacy/unfenced write path.
No exact external setting key or public interface is established in this study.

Use a dedicated fresh owned reference database in a separately identified format
(version 2 is the selected reference marker) for the first implementation proof.
Do not auto-migrate an operator's existing v1 database. All scoped native create,
update, triage and direct application Store calls must reach the checked commit
primitive. Other scopes without an explicitly provisioned allowed grant deny.
Trusted direct database/file owners are not an adversarial SQL sandbox; the
promise requires participating writers and controlled access to this authority.

Schema/version checks and actual authority-table validation must reject an
unsupported/corrupt/missing reference format before serving writes. Current old
binaries reject v2 at open, but a schema bump cannot fence an already-open old
connection. Drain/stop and exclude every legacy instance, worker and bypass
writer before preparing/adopting this reference file. Mixed legacy/strict writers
and online migration are unsupported. Startup/readiness must not label an
unvalidated write authority ready or downgrade after an error.

Tickets are process/Store-private and cannot survive restart/reopen. Revoke rows
and generations persist with the business authority; do not reset them at normal
startup. No online restore/file replacement or automatic old-backup promotion is
supported. A restored/rolled-back authority must remain write-disabled until
all old writers are excluded and an explicit fresh authority incarnation/current
permission reconciliation is established. Never blindly reseed an older allow.
[YT-139](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md) retains its original scope;
concrete persistent/cloud restore proof belongs to separately activated YT-184.
This contract does not qualify power-loss durability, cross-host SQLite or HA.

The existing worker admission, busy/deadline, drain, item/history limits and
transaction ownership remain. A denied/mismatched ticket is a redacted 403 for
an otherwise valid caller; normal item conflicts remain 409. Busy/unavailable
or uncertain authority stays redacted 503 without an automatic retry or success
cookie. Unsupported schema refuses startup. Preserve current privacy, no-store,
CSP and generic error behavior; credentials, tickets, policy/generation contents
and unauthorized resource existence must not leak into logs or browser state.

## Minimum future evidence and this stage's acceptance

Only after separately selecting implementation, use the existing focused test
pattern with one owned SQLite file and independent connections/Store instances.
Prove normal item/history atomicity, a writer paused **after old ticket issuance
but before its transaction** rejecting after acknowledged revoke, old-ticket
rejection after revoke/re-enable, and an already-held write transaction preventing
premature revoke success. Cover missing/mismatched authority, terminal generation
and unsupported/legacy format through focused cases. Reconcile exact item/version/
history/authority rows and worker drain; a same-mutex-only result is insufficient
for the shared-database ordering claim. Do not build a generic validator, new
benchmark, cloud fixture or broad old-proof replay to establish these cases.

This delivery performs only source/evidence inspection, primary-documentation
research and new document checks. Independent Reviewer must accept the selected
scope, transaction/ticket/acknowledgment semantics and retained limitations before
the contract stage is accepted. **YT-181 remains In progress** until the future
opt-in reference, focused new evidence and independent implementation review are
complete. Contract PASS never means the race has been fixed in runtime.

[SQLite transaction control](https://www.sqlite.org/lang_transaction.html) and
[isolation](https://www.sqlite.org/isolation.html) document one write transaction
at a time and IMMEDIATE write acquisition; [user_version](https://www.sqlite.org/pragma.html#pragma_user_version)
is application-owned metadata, not an enforcement mechanism. The concrete
write/revoke design above is this contract's application-level inference from
those mechanisms, not a guarantee SQLite makes about external Session/policy.

No runtime tests/builds/load/analysis programs, SQL execution, Azure resources,
new crate/API, migration implementation, default change, Commit or Push occurs.
YT-182–184 remain Todo, YT-096 D1–D6 Pending and YT-020 Blocked. Retain historical
reviews, known limits and every failed result without replay or reinterpretation.


## Independent contract acceptance

Reviewer `/root/yt181_sensitive_write_contract_review_20261025`: **PASS**, zero
issues and required followups. All five document, 13 evidence and 17 accepted
source/document hashes match before/after review; native reviewer role/model/
high effort match the saved configuration/resolver. Review is read-only and
performs no tests, builds, services or old-proof replay. New document checks
pass (75 documents/184 backlog items), with clean tracked and new-contract
whitespace diagnostics. Only stage-status/review-reference metadata changes
after acceptance; runtime and the retained evidence remain immutable.

Contract stage accepted; **YT-181 remains In progress**. A future separately
selected opt-in implementation, focused ordering/negative proof and independent
implementation review are still required before Done. Current runtime retains
the old Session/file-policy race and has no SQL grant/ticket enforcement.
Five new document changes remain unstaged/uncommitted/unpushed; local
`closeout.json` binds final hashes, Git state, prior commit and reused evidence.


## Later authorized implementation stage

The maintainer subsequently selected the [minimal opt-in implementation](YT-181-SENSITIVE-WRITE-IMPLEMENTATION.md).
The contract-stage record above is historical and retains its original PASS.
YT-181 is now Done at the bounded opt-in reference scope after independent
implementation Reviewer PASS, zero issues/followups. The
implementation chooses conservative SQL deny/generation advancement on every
existing v2 Store open, requiring trusted current-authority confirmation before
re-enable. This never resets a generation or reseeds an existing allow, and
also invalidates other instances' outstanding tickets. No automatic restart
write availability, restore qualification or stronger Session/file-policy
promise is added. Refer to the implementation record for actual new cases,
feature/settings spelling, changed files and separate acceptance.
