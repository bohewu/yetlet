# Provider Operations — Local Evidence and Open Qualification

**Task:** YT-066 / M10g. **Status:** In progress; distributed qualification remains
unmet. This document follows the approved [remaining plan](M10-REMAINING-PLAN.md)
and [ADR 0005](adr/0005-opt-in-capability-modules.md). It introduces no runtime
provider API, store implementation or default dependency.

## Current topology

`yetlet-session` owns an in-memory map within each `SessionManager`; there is no
external-store integration or pluggable distributed-store contract. The
`auth-notes` sample creates a separate manager in each process. Identical
synthetic Alice/Bob login codes do not make Session records shared. Restart
discards the map, and a second worker cannot authenticate the first worker's
cookie. Sticky routing alone cannot provide durable Session recovery.

The SQL-first `sqlite-crud` sample uses a local SQLite WAL database with explicit
transactions, optimistic versions and transactional audit rows. Two local
processes opening that same file exercise local filesystem locking, not nodes,
replicas, network storage or a cluster. SQLite documents that WAL requires
processes on the same host and does not work over a network filesystem because
of its shared-memory coordination. See [SQLite WAL](https://sqlite.org/wal.html).

Existing [M8c Session evidence](M8C-VERIFICATION.md) retains atomic update,
rotation, CSRF and revocation regressions, including stale response cookie
protection. Existing [M9d file evidence](M9D-VERIFICATION.md) and
[M9f SSE evidence](M9F-VERIFICATION.md) retain cancellation/backpressure tests.
They are earlier local evidence, not newly executed distributed tests.

## One bounded local experiment

From the current checkout, after the main agent verifies the two existing
prebuilt debug binaries:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_local_operations.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/qualify_local_operations.py
```

The first command runs direct stdlib SQLite and mocked process lifecycle tests;
it launches no sample server. The second command launches only
`target/debug/yetlet-auth-notes` and `target/debug/yetlet-sqlite-crud` on ephemeral
IPv4 loopback ports. It does not build Rust, use Docker/Redis, contact a remote
service or install dependencies. Run it once; no repeated soak or latency gate
is defined. [M10g verification](M10G-VERIFICATION.md) separates the ten source/lifecycle tests from the main-agent actual local run, which passed with seven processes reaped and temporary data removed.

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
3. With no mutations during backup, use Python `sqlite3.Connection.backup` into
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
the null device. Only after cleanup attempts does the script write
`target/m10-remaining/local-operations.json`; cleanup failure prevents local
PASS. Report values are fixed outcomes/counts/statuses, durations and version
identity, never raw cookies, CSRF tokens, codes, environment, response bodies,
logs or exception text. No arbitrary directory deletion is used.

The report labels success `pass-for-local` and explicitly labels distributed,
multinode, HA and performance SLO qualification `not_verified`. Durations are
observations without latency thresholds. Binary SHA-256 and current checkout
package versions identify the inputs; the samples expose no binary-version or
linked SQLite-version endpoint, so those runtime versions remain `not_exposed`.
Python's SQLite version is observed separately and must not be attributed to
the Rust app's bundled SQLite library. The main agent must retain build/source
identity if it needs to connect a prebuilt binary to a particular source freeze.

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
| Stale cookie responses | Deliver an old response after rotation/revocation; its `Set-Cookie` must not clear or restore the replacement identity |
| TTL and consistency | Expiry and revocation enforcement across workers, cache invalidation, clock/lease assumptions, explicit behavior during partitions and recovery |
| Outage and recovery | Store unavailable, timeout, partial failure and failover fail closed; no authorization through a stale or invented fallback Session; replay/retry cannot resurrect revoked identities |
| Bounds and shutdown | Bounded concurrency, pools, queues, timeouts/retries, admission/backpressure and drain/cancellation under real contention; no unbounded work or abandoned owned resources |

The [tower-sessions concurrency documentation](https://docs.rs/tower-sessions/latest/tower_sessions/#data-races-under-concurrent-conditions)
warns that concurrent dependent mutations can lose updates because requests
load separate Session snapshots. A pluggable Redis backend by itself therefore
does not prove the stronger atomicity/rotation/revocation contract above. This
is a reason to qualify the selected integration, not to write an in-house store.

## Production ownership and unverified gates

No production topology, recovery budget, retention policy or SLA is chosen by
this experiment. The application owner, storage operator and security owner
must be designated; these roles are pending, not assigned to fictional people.

| Pending control | Owner to designate and required test/decision |
|---|---|
| RPO/RTO | Application/storage owners select objectives from actual business requirements; measure recoverable checkpoint age and end-to-end restoration on the selected topology |
| Restore verification | Storage owner verifies integrity, schema, application state, authorization and excluded late writes; document isolated restore, acceptance, rollback and promotion procedures |
| Backup protection | Storage/security owners select encryption and key custody/rotation, access controls, retention/deletion and off-host durability; test restoration with the required keys and loss/failure cases |
| Secret redaction | Security owner defines cookie/token/code/credential/log exclusions and tests failure paths, exports, metrics and retained evidence; do not attach raw sensitive logs |
| Observability | Application/operator owners define safe correlation, conflict/outage/queue/drain/backup metrics, alerts and recovery procedures; verify them under the real failure scenarios |

Provider outage, Redis failover, multinode consistency, HA, load/performance SLO,
production backup durability/RPO/RTO, real OIDC and key rotation remain **not
qualified**. Retained Session, file and SSE tests cannot replace these tests.
YT-066 remains **In progress** pending selected external topology and actual
distributed qualification. Local success does not close its gate or unblock
the external production-security review prerequisite by itself. YT-020 remains
Blocked. Local execution and source/local review `wc_job_YclpZ0R5tS4m8Y1I` passed. Provider selection, ownership and actual distributed acceptance remain open gates.
