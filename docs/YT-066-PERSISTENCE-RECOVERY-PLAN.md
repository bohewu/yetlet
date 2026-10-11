# YT-066 — Framework qualification and deployment recovery boundary

**Date:** 2026-10-11
**Status:** Framework/deployment responsibility boundary corrected; independent documentation review PASS. Persistence/recovery rehearsals below are optional proposals, not YT-066 completion prerequisites or approved implementation. YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope; its separate final closeout review gate is recorded in M10g. YT-020 remains Blocked.
**Related:** [Backlog](BACKLOG.md), [provider operations](PROVIDER-OPERATIONS.md), [Redis design](YT-066B-REDIS-SESSION-DESIGN.md), [b3 evidence](YT-066B3-VERIFICATION.md), [Cookie contract](YT-066B-COOKIE-ORDERING.md), [ADR 0005](adr/0005-opt-in-capability-modules.md).

## Authority and committed baseline

The maintainer authorized peon to commit the reviewed YT-066b3 first, then plan
production persistence/recovery without implementation, WebCodex or push, using
the original checkout. Peon committed exactly the 18 b3 paths as
`4da0f6c1879c9a56598caec7cad449cd9127c7de`, parent
`1f38d20b8ebaf9b679aa69ce461b495db1856db2`, tree
`e500a7baeed459e5a32ca7dedfe1da104f3fafd6`.

The saved peon binding and native runtime trace matched `gpt-6-luna` / `high`.
The deterministic staged guard exited 0 under the existing audit/builtin policy,
with first-screen coverage, status `warn` and two findings. Nothing was bypassed
or weakened; guard tree equalled the committed tree. All 286 working-input hashes
were preserved using Git symlink semantics. Index/worktree were clean immediately
afterward. `origin/master` remains `8e157b43244be38c9afc22ed7bcd7bf06351416e`.
Delivery evidence is retained in ignored `target/yt066-production-plan-20261011/`.

The [b3 PASS](YT-066B3-VERIFICATION.md) remains bound to its recorded execution
and review identities. It proves three applications against one Redis primary,
including controlled outage and explicit application restart recovery, with
`appendonly no`, `save ""`, tmpfs and no persistent volume. It does not prove
durable writes, backup restore, power-loss recovery or automatic client reconnect
recovery. Committing that evidence does not turn it into a new execution.

## Framework responsibility and completion scope

The maintainer challenged treating TLS/ACL deployment policy, backup protection,
RPO/RTO and operational ownership as framework work. Consistent with ADR 0005,
those application/deployment decisions must not block bounded Yetlet qualification.
The initial draft's production checklist was too broad as a framework task list.

| Boundary | Yetlet/provider responsibility | Application/deployment responsibility |
|---|---|---|
| Session semantics | Typed operations, CAS, rotation/revocation, expiry, fail-closed errors, no replay, bounded cancellation/drain, scoped Cookie behavior and opt-in dependency isolation | Decide authentication policy and availability needs; configure the application |
| TLS/ACL | Expose/document the mature client's supported endpoint/configuration surface; reject unsupported or failed connections without silent downgrade or fallback; redact credentials | Select TLS requirements, certificates/trust, ACL users/permissions, secret rotation and network isolation |
| Persistence/recovery | State the exact tested storage boundary, distinguish acknowledged commit from durable persistence, document rollback/no-resurrection assumptions and available namespace configuration | Select AOF/RDB/service/storage, backups and retention; enforce any recovery fence/invalidation procedure |
| Recovery objectives and operations | Document limits and error categories honestly | Choose and measure business RPO/RTO/SLA, alerts, on-call ownership and recovery acceptance |

The active framework completion target is **single-host Docker multi-instance
and selected experimental Redis-provider scope**, as already defined by the
[approved plan](YT-065-066-PLAN.md#8-completion-and-evidence-accounting).
Retained b1/b2/b3 and Cookie evidence qualify only their recorded boundaries.
They do not qualify every deployment or guarantee safe authorization after an
arbitrary external restore. Such a deployment must meet the documented recovery
preconditions or invalidate its previous Sessions before accepting traffic.
This preserves the authorization contract rather than pretending the provider
can recover lifecycle history that its external store has lost.

No production operator, certificate/ACL policy, backup system or business RPO/RTO
is needed to close the bounded framework task. The [bounded closeout](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11)
records the selected completion scope and its separate final documentation review.
No new runtime defect or uncovered mandatory path is established here.
Application operations do not keep that bounded framework task open.

## Conditional persistence/recovery proposals

This plan derives its blocking safety requirements from the existing Redis
design, its persistence/recovery boundary and provider-operations contract:
retired identities cannot regain authorization; failures stay closed; uncertain
mutations are not replayed; expiry, bounds, cleanup and redaction remain intact.
The maintainer's current request authorizes the plan only. The concrete storage
configuration and maintenance policy below are recommendations, not new approved
requirements or an implementation instruction.

Keep Memory synchronous and unchanged; Redis remains application opt-in and
outside the Core/default-server/browser dependency graphs. Keep native
Axum/Tower composition, pure SSR and the selected Cookie policy. No mandatory
JS, fallback authentication, generic store/recovery framework, ORM, DI, registry
or new HTTP stack is proposed. The fixture's GET mutations and control routes
remain private synthetic qualification tools, not production authentication APIs.

If a later application or explicit framework persistence claim needs additional
evidence, the smallest proposed target is the same three-app, single-host,
single-primary arrangement with one isolated persistent Redis volume. Use the
existing pinned Redis 8.2.10 image and redis-rs 1.7.1 dependency unless a concrete
compatibility defect requires a separately reviewed change. Keep the old b3
no-persistence behavior and evidence intact. No production host, storage service,
business recovery objective or deployment is selected by this document.

| Persistence choice | Scope and tradeoff | Proposed disposition |
|---|---|---|
| No persistence | Already qualified in b3; Redis restart loses Sessions | Preserve this result; it cannot close durability qualification |
| AOF `everysec` | Periodic fsync can lose acknowledged recent writes on an unclean failure, including a rotation/revocation | First optional fixture setting if persistence qualification is separately selected, with the recovery assumptions below |
| AOF `always` | More frequent fsync; different latency/storage tradeoff, still not a backup or proof of hardware durability | Evaluate only if an application requires a stronger durability boundary; do not add a second automatic test matrix |
| RDB snapshots | Point-in-time backup; may omit later writes | An isolated restore source when explicitly selected, never proof that a later revocation survived |

Redis documents these [persistence options](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/).
`everysec` is not a measured one-second RPO, zero-loss claim or SLA here. Business
data must not rely on this Session-only qualification as its durable system of
record. A production application selects acceptable Session loss, maintenance
downtime and any required recovery objective for its deployment. Appointing
production owners or choosing business objectives is not a prerequisite for a
synthetic local fixture or bounded framework completion.

Proposed lab settings are `appendonly yes`, `appendfsync everysec`, scheduled RDB
snapshots disabled, `aof-load-truncated no`, and existing `noeviction` memory
bounds. Record all effective settings, rewrite policy, filesystem/mount identity,
permissions and image version in the eventual evidence. A Redis PING, mounted
volume or startup success alone is not acceptance.

## Optional deployment recovery procedure

The following is a conditional runbook proposal for an application that selects
this storage arrangement. It is not a Yetlet recovery service, production policy
default, or a new mandatory framework implementation/test stage.

### Planned, verified clean restart

1. Gate Session-dependent traffic using the application's deployment boundary.
   Stop mutations, drain the existing managers and terminate all three workers;
   reconcile any still-retained uncertain result without replay. An unresolved
   mutation, forced termination or storage error disqualifies the clean path.
2. Stop the exact Redis process through its normal shutdown sequence. Redis
   [SHUTDOWN](https://redis.io/docs/latest/commands/shutdown/) flushes AOF; a
   connection closing is not by itself proof that shutdown succeeded. Observe
   process termination and persistence status. A timeout/kill fallback is an
   unclean recovery, not a successful graceful restart.
3. Restart the same Redis version and original volume privately. Start fresh
   application managers with the same namespace only when the clean boundary
   and unchanged storage are established. This deliberately does not require
   transparent reconnect by the old connection managers.
4. Before opening traffic, require unexpired active state/revision/CSRF continuity
   on A/B/C, continued rotation/revocation rejection and unchanged absolute expiry.
   Storage/load inconsistency leaves traffic closed and moves to invalidation.

A clean restart can retain Sessions; a later successful operation still does
not establish per-operation fsync acknowledgment. Do not conflate a graceful
process restart with host power loss or device failure.

### Unclean restart, missing storage or older backup restore

An older store can contain an active identity whose later revocation was lost.
This is an application authorization consequence of rollback, not a Redis bug
or something record/tombstone TTL alone can prevent. The proposed policy is to
invalidate all pre-recovery Sessions whenever the latest lifecycle history cannot
be established, accepting explicit login again rather than old-ID forwarding.

1. Keep Session traffic closed. Disable automatic Redis restart/promotion and
   application auto-start against recovering storage in the selected deployment
   policy. Terminate all old workers and outstanding connections before exposing
   any recovered primary to application clients. Otherwise automatic reconnect
   could serve rolled-back identities before an operator intervenes.
2. Preserve the source backup/storage unchanged and inspect only an owned,
   isolated candidate. Never repair or restore over the running store. Missing,
   corrupt, truncated or incompatible input cannot silently become a trusted
   empty authentication store.
3. Select a fresh, never-reused namespace/generation through application
   configuration controlled outside the recovered Redis data. Record that choice
   in the deployment's existing protected configuration; do not derive it from
   an older backup or introduce a new registry/service. All A/B/C workers must
   use the same new namespace. Do not migrate restored Session records into it.
4. Start fresh managers, verify every pre-recovery Cookie is rejected on A/B/C
   without authenticated auto-create, and explicitly create a fresh Session.
   Preserve the fixed-name Cookie contract and its permitted login-loss boundary.
5. Old operation tickets must never execute against or authorize the new
   generation. Missing evidence remains Unresolved; a namespace mismatch may be
   explicitly rejected. Historical commit status is not durable acknowledgment
   or permission to recreate a Session. There is no callback/command replay.
6. Reopen Session traffic only after the storage/app/security owners accept the
   new generation and tests. If that generation is unsuitable, remain closed;
   rollback must not restart an older generation and revive its credentials.

The existing configuration already accepts a namespace; this is a proposed
operational procedure, not an implemented recovery controller. It requires a
demonstrably enforced deployment fence and coordinated worker replacement.
If the selected environment cannot enforce them, stop and identify the smallest
missing seam before proposing code. Mixed generations may fail availability;
they must not be used as a rolling recovery strategy.

## Proposed bounded qualification slices

These labels retain the initial draft's possible experiments; they are not new
backlog tickets, required YT-066 gates or authorized executions. Run a slice only
for a separately selected application need, explicit persistence claim or concrete
uncovered provider behavior. Each selected slice needs explicit scope approval
and independent review; all new cells remain **Not verified / optional**.

| Proposed slice | Minimal actual observation and acceptance | Boundary |
|---|---|---|
| YT-066p1: persistent clean restart | Explicit persistent volume/config; create/update/rotate/revoke before the stop; verified graceful Redis stop, fresh apps, exact unexpired active state on A/B/C, retired IDs still rejected; elapsed absolute TTL not refreshed | One persistence setting and one planned restart; no hardware/crash durability claim |
| YT-066p2: isolated recovery and invalidation | One abrupt Redis process stop and one older-checkpoint restore scenario, traffic fenced throughout; fresh external generation, every old Cookie rejected, explicit fresh Session works; no replay of a retained old ticket | Process SIGKILL is not host power loss; restored history never authorizes old identities |
| YT-066p2: invalid candidate and cleanup | Reject a deliberately damaged copy or missing required persistence input; source/checkpoint hashes unchanged; failed/cancelled execution still removes all exact-owned resources | No automatic `--fix`, partial-load success, broad filesystem deletion or generalized proof framework |

For the restore scenario, create an immutable checkpoint while the app workers
and Redis are stopped, then restart the original store and rotate/revoke an
identity that was active at the checkpoint. Restore only to a new isolated volume.
Prove that the old image contains earlier history using private synthetic
observations, but never open that old generation for authorization. This directly
tests rollback safety without waiting for a random crash to lose a particular write.

For Redis 8.2, retain the complete multi-part AOF artifact set and manifest, not
just a guessed single filename. Use the mature Redis integrity tools read-only
and ordinary copy/hash commands on the stopped source. The [current backup
documentation](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/)
also describes newer Redis 8.10 BACKUP commands; those are not assumed available
on this pinned 8.2.10 target. No online backup/rewrite coordinator is needed for
this bounded offline rehearsal. Raw artifacts contain Session/CSRF material:
keep them private, outside Git/reports, and remove all lab copies after cleanup.

Reuse the accepted b3 application and exact-owner cleanup helpers with only the
small persistence/recovery seam needed by the selected slice. Keep request,
startup, shutdown, drain and copy/restore deadlines explicit before execution.
Limits are fixture budgets, not production SLOs. Cleanup includes persistent and
candidate volumes/checkpoints as well as containers, networks, images, processes
and temp work, on success/failure/cancellation. Never remove unrelated objects.
Retain existing CAS/Cookie/MSRV/browser evidence; rerun only a concretely changed
path or existing required check. Record exact execution inputs separately from
historical b3 identities. A focused failure gets only the material repair and
affected revalidation, not a new harness, broad rerun or certification program.

## Application/operator responsibilities, not framework blockers

The following are reference questions for an actual deployment. They are not
Yetlet deliverables, required owners to appoint in this repository, prerequisites
for YT-066 completion, or a mandate to build a production operations platform.

| Decision | Owner and evidence still needed |
|---|---|
| Actual deployment/failure domain | Application and storage owners select host, filesystem/service and availability policy; local Compose is not a production deployment or off-host durability proof |
| Session retention and recovery | Application owner accepts which failures invalidate all Sessions and whether maintenance downtime is acceptable; storage owner selects and measures any RPO/RTO target |
| Admission and restart fence | Deployment owner specifies the actual gate, auto-start policy, worker inventory, generation storage and promotion/rollback steps; enforce closed traffic until recovery acceptance |
| Endpoint security | Security/storage owners select ACL identity, minimum commands/key scope, rotation and transport protection; verify denied access and redaction on the chosen topology |
| Backups, if required | Storage/security owners select protected destination, encryption/key custody, retention/deletion and off-host copies; restore with required keys and reject integrity/version failures |
| Operations | Designate real on-call owners; define safe outage, persistence-error/disk-capacity and failed-backup alerts plus maintenance/recovery instructions; no invented owner, SLA or unmeasured alert threshold |

Current provider code accepts only `redis://` and its selected features do not
provide a qualified `rediss://` path. The b3 unauthenticated internal network is
a lab boundary, not production ACL/TLS evidence. A deployment requiring TLS needs
a separately scoped mature-client integration and actual verification, not an
in-house proxy/transport or a claimed capability that does not exist. Consult
Redis's [ACL](https://redis.io/docs/latest/operate/oss_and_stack/management/security/acl/)
and [TLS](https://redis.io/docs/latest/operate/oss_and_stack/management/security/encryption/)
documentation when selecting that environment. This planning round changes no
endpoint, secret, client feature or dependency.

Sentinel/Cluster/replica promotion, cross-host partitions, host power-cut testing,
business-database recovery, load/soak, SQL Session, a device lab and public-release
work are excluded from these proposed local slices. YT-066c remains cancelled.
Requiring any of them needs an application-driven separate scope; they are not
silently made prerequisites for YT-066p1/p2.

## Planning closeout and next decision

This round delivers documentation only after the authorized b3 commit. No Rust,
Cargo, Compose, workflow, Python/browser harness, sample or generated asset is
changed; no container, volume, backup, server or browser is started. The initial planning
diff was unstaged/uncommitted. The later maintainer instruction authorizes a
local commit after the separate final closeout reviewer PASS, with no push. Existing doc/link/language,
backlog and whitespace checks validate the proposal's consistency, not recovery.

Independent review of this scope reconciliation passed with no material gap
reported. The separately requested [bounded framework closeout](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11)
uses retained evidence and its own final independent reviewer gate. Choosing AOF,
business recovery objectives or production owners is not mandatory framework
work. No production acceptance is claimed. YT-065 and YT-067 retain their prior
bounded Done states; YT-066 is Done only at single-host Docker multi-instance
and selected Redis-provider scope, and YT-020 remains Blocked. Even a separately selected future p1/p2 PASS
would qualify only the selected lab persistence/recovery procedure, not all
production controls, HA, supported devices, accessibility, SLA or publication.

## Independent responsibility-scope review — 2026-10-11

The saved repository reviewer contract at
`/home/bohe/.codex/agents-pipeline/agents/reviewer.md` was supplied to an ordinary
local `codex exec` process with a read-only sandbox, not `codex exec review`,
WebCodex, current-session self-review or native managed-role dispatch. Fresh
workspace profile checks reported `health=ok`, `catalog_state=current` and
`profile_eligibility=eligible`; the shared resolver selected the saved
`reviewer` binding, `gpt-6-astra`, strong tier, adaptive ordinary `review` in
`ad-hoc-review` context, with requested effort `high`.

Reviewer thread `01a12a04-d58f-7210-a995-c6f35860437c` reached terminal exit 0.
Runtime metadata matched `gpt-6-astra` / `high`, read-only sandbox and approval
policy `never`. There was no WebCodex Job ID or native child trace. Its canonical
output was:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewed baseline was HEAD `4da0f6c1879c9a56598caec7cad449cd9127c7de`, with
287 tracked/nonignored working-input hashes, using symlink target text for Git
symlinks. The sorted compact input-map SHA-256 was
`ffb186eb9ee73a300942cb027af9efd6e3160777ec40de9c67f8e4fdd6d5e19b`.
All inputs, HEAD, working-tree status and empty index matched before/after review.
The process was gone after terminal completion. Evidence remains in ignored
`target/yt066-scope-review-20261011/`.

The exact responsibility-reconciliation change set at that review was:

- `README.md`
- `docs/BACKLOG.md`
- `docs/M10-REMAINING-PLAN.md`
- `docs/M10G-VERIFICATION.md`
- `docs/PROVIDER-OPERATIONS.md`
- `docs/RELEASE-READINESS.md`
- `docs/ROADMAP.md`
- `docs/YT-065-066-PLAN.md`
- `docs/YT-066-PERSISTENCE-RECOVERY-PLAN.md` (new)
- `docs/YT-066B-REDIS-SESSION-DESIGN.md`
- `docs/YT-066B3-VERIFICATION.md`

Documentation validation passed for the existing 75 required documents and 67
backlog items. Selecting all 100 tracked/nonignored Markdown documents in memory
through the same checker also passed language, links, structure and backlog
checks, without modifying the checker. Tracked and new-file whitespace checks
passed; all non-documentation inputs remained unchanged. No Rust, Docker,
Redis or browser suite was repeated, and no runtime resources were started.

After review, only this receipt/status and the M10g receipt were appended to
record the verified outcome; the reviewed responsibility decisions are unchanged.
Those receipt edits receive the same bounded documentation checks, not another
qualification run. All 11 files were unstaged/uncommitted at that reconciliation checkpoint, with
no push. The later bounded closeout and newly authorized local delivery are
separate; their gate is recorded in M10g, not claimed by this prior review.
