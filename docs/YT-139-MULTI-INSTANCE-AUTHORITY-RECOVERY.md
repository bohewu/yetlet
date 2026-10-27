# YT-139 — Multi-instance authority and recovery reference

Date: 2026-10-23. Status: Done at bounded optional local reference scope after
independent contract and final Reviewer PASS, zero issues and required followups. Source baseline:
`b059c3269cc19c8f7db598edb925ef76dd5056cd`.
The owner selected this optional local integration after YT-138. Independent
contract Reviewer `/root/yt139_authority_contract_review_20261023` returned PASS,
zero issues and required followups; all six contract-review hashes matched. The
initial review snapshot and result are retained as `contract-review-input.json`,
`contract-review.patch` and `contract-review-final.json` in the evidence directory
below. The accepted implementation, isolated recovery proof and cleanup are
complete as recorded below. Final independent Reviewer
`/root/yt139_authority_recovery_final_review_20261023` returned PASS, zero issues
and required followups; all seven final review hashes matched. No commit or push
is selected for this delivery.

## Selected consumer and bounded delta

Use two actual Admin OS processes with the existing `auth-valkey` feature,
sharing one same-host SQLite business database, one trusted local account-policy
file and one optional Valkey single primary. The existing Admin/Board composition,
provider API, DTO, schema, cookie and settings remain unchanged. The implementation adds one focused
ignored integration test in the existing
[Auth/Valkey test](../examples/admin-dashboard/tests/auth_valkey.rs), not a new
application, crate, dependency, public API, storage layer or default capability.

The new acceptance delta is deliberately restoring an earlier, still-unexpired
Session record after its acknowledged logout, then recovering both Admins into
a new, never-reused application-owned Session namespace. Current business rows
and history and the current restrictive policy remain authoritative. Old browser
credentials must not regain protected access or mutate business data; deliberate
fresh login must work across the two recovered instances under current policy.

Reuse [YT-124](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md),
[YT-125](YT-125-VALKEY-AUTH-INTEGRATION.md),
[YT-126](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md),
[YT-094 shared SQLite](YT-094-SHARED-SQLITE.md) and
[YT-094 Valkey/TTL](YT-094-VALKEY-SESSIONS.md) at their original scopes.
Their passing full journeys, browser coverage and outage cases are historical
evidence, not new YT-139 executions. Follow [ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md): application-owned authority,
explicit optional native capabilities and ordinary Axum/Tower composition.

## Authority, trust and compatibility

| Authority | Placement and persistence | Trust and freshness | Recovery selection |
| --- | --- | --- | --- |
| Business effects | One local SQLite file and its engine-managed WAL/shared-memory files; Board schema 1 | Two independent connections to the same local authority; immediate transactions commit work item and history together; optimistic item versions are edit concurrency | Retain the current database, including post-Session-snapshot acknowledged changes. Do not replace it with an earlier Session-era database or an independent replica file |
| Accounts and policy | One trusted, atomically replaced local JSON file plus compiled account/project limits | Read afresh, bounded to 4096 bytes/250 ms; unknown/malformed/unavailable authority fails closed. Policy only restricts compiled accounts/projects/write permission | Retain the latest restrictive policy and exact account bindings. Never restore an older permissive file merely to match Session age |
| Session validity | Existing experimental single-primary Valkey namespace and provider schema | Authoritative remote read; no local fallback or permission object trusted from stored state; absolute expiry | A restored pre-logout record is potentially active again. Treat its entire old namespace as retired; restart every consumer into a new empty epoch namespace and require re-login |
| Recovery epoch | Application-owned deployment configuration outside the restored Valkey data | Trusted controller/operator must know which epoch is current and which were retired | Allocate a monotonically advancing generation with a deployment-unique prefix; never reuse a generation/name, including after application rollback. This prototype has no durable epoch allocator or enforcement service |

The existing setting `APP_SESSION__VALKEY__NAMESPACE` carries the recovery epoch;
there is no new provider epoch field. Both recovered processes must use exactly
the same newly allocated name, compatible TTL/settings, DTO and cookie rules.
Namespace syntax remains 1–64 ASCII alphanumeric, underscore or hyphen characters.
The fixture uses a fresh temporary deployment identifier and successive `_e1`
and `_e2` suffixes, checks `_e2` has no Session authority, and does not recycle
these names. A disposable fixture's fresh name is not evidence of a production
monotonic allocator. Unknown external epoch history means keep service stopped,
not guess a name or accept an old generation.

Provider keys use `yetlet:redis:v1:<namespace>:` while the actual Session record
has schema `2`; the key-version token and record schema are different boundaries.
The application DTO requires an explicit `user` field (null is anonymous), stores
a local name and reconstructs compiled permissions. Unknown identities/fields
and incompatible records fail decoding. Existing optional OIDC pending data is
unchanged; no real IdP callback is selected here. Do not mix incompatible binary,
DTO, schema, account mapping or namespace configurations during this cutover.

The local HTTP cookie is `yetlet_lab_valkey`, HttpOnly, SameSite=Lax, Path=/.
It contains an opaque 64-character random identity, not a signed permission
assertion. Here “signed-in cookie” means the opaque cookie for a logged-in
Session. Cookie syntax does not bind it to an epoch; the server's configured
namespace selects authority. Existing secure-cookie production defaults and
explicit insecure local override are unchanged. No cookie/CSRF/Session payload,
credential proof or protected endpoint is printed into evidence. Generic Auth
principals and server capabilities never enter browser hydration.

## Recovery procedure and refusal rules

1. Remove all old-generation Admin instances from service, stop admission and
   gracefully terminate them. Wait for exit success and the existing drain marker
   for **every** writer, including provider-owned work. If an old process, direct
   writer or in-flight outcome remains unknown, do not perform or expose cutover.
   This fixture has only its two owned Admin writers and no gateway/background job.
2. Reconcile the current business database and policy deliberately. Record the
   latest acknowledged business changes and complete current row/history state;
   retain the restrictive policy separately from the stale Session snapshot.
   If current policy or business history cannot be established, refuse service.
   YT-139 does not authorize accepting unaccounted business loss.
3. Restore the selected stale Session record only while all old consumers are
   stopped. Treat the old namespace as unsafe, even on the same host and even
   when its earlier data passes provider schema/expiry validation. Logout
   tombstones, issuance counters and operation results are not durable external
   recovery epochs. Do not serve the old namespace as a rollback option.
4. Allocate a new never-reused externally controlled epoch namespace. Verify
   compatible exact binary/schema/DTO/account bindings and current restrictive
   policy; configure **both** restarted Admins with that namespace and the same
   retained business database. No old-generation instance may serve concurrently
   or rejoin later. This is a coordinated stop/restart procedure, not a rolling
   mixed-generation protocol or runtime fencing implementation.
5. Reachable-but-missing old Session identity requires explicit re-login (the
   current native protected scope uses 303 to `/board/login`, no Set-Cookie).
   Provider/policy unavailable, invalid or unknown state is a redacted 503, never
   stale allow, local Memory fallback or successful authentication. A dispatched
   mutation with unknown outcome is not a rollback or success acknowledgment;
   do not automatically replay login/logout or a business save. Keep recovery
   blocked until authority and relevant outcomes can be reconciled.
6. Only deliberate fresh credential verification and acknowledged Session/CSRF
   rotation can reestablish continuity. Current account/project/read/write policy
   still applies on both instances; disabled accounts remain denied even if the
   lifecycle login creates a Session. Policy-aware login admission is unchanged.

The guarantee is rejection of retired Session identities after a coordinated
namespace cutover under these prerequisites. A later configuration rollback to
an old namespace, reuse of an epoch, stale policy restore or surviving old writer
breaks the procedure. The application does not detect or prevent those operator
errors automatically. If they cannot be excluded, keep protected service closed.

This does not upgrade the [YT-138 pre-SQL race](YT-138-AUTHORIZATION-COMMIT-ORDERING.md):
Session/policy snapshots and SQL commit are separate authorities. Session
invalidation does not cancel or atomically order previously admitted business
work. Stopping and draining all old writers is a prerequisite here, not a new
strict revocation-to-commit implementation. Preserve
[YT-137 no automatic mutation retry](YT-137-WRITE-OUTCOME-IDEMPOTENCY.md);
no deduplication, exactly-once effect or universal operation ledger is selected.

## Minimum fixture and exact acceptance sequence

The parent owns creation/removal of a dedicated disposable loopback Valkey
container using the already selected image
`valkey/valkey@sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
The new test requires explicit `YT139_VALKEY_URL=redis://127.0.0.1:<port>/` and
will not select a default or shared service. Existing raw TCP fault-injection
code can send fixed bounded RESP commands/EVAL and consume only integer markers;
no Redis client dependency is added. Python 3 with standard-library `sqlite3`
is an explicit fixture prerequisite for a fixed read-only reconciliation query,
not a new product dependency or verification framework.

1. Start A/B using epoch `_e1`, one temporary `data/board.sqlite3` and permissive
   policy. Login Alice on A and use B to read that identity. Create one work item
   at version 1 with one matching history record. Capture the exact active Alice
   Session record with Valkey `DUMP`, together with its absolute key-expiry time,
   into one task-only scratch hash outside the provider namespace. Require the
   record to be active/unexpired; keep binary payload and key identity inside
   the server/test memory, never stdout or repository evidence.
2. After capture, update that same work item on B to version 2, with its second
   history record. Atomically restrict policy: Alice retains read but loses
   write; Bob is disabled. Confirm those current restrictions before cutover.
   Acknowledge Alice logout through B; both instances reject her old cookie.
3. Stop/drain A and B successfully before restoration. Record exact current
   schema version, sorted complete `work_items` and `history` rows (including
   IDs/timestamps), `sqlite_sequence` and index definitions through one fixed
   Python read-only SQLite query using one consistent read transaction. Expected business state is one item/version 2,
   exactly two history records (create/update), matching actor/project/version
   and retained current title/description/status/priority/assignee. Retain exact
   current policy bytes. No writes are allowed between this receipt and cutover.
4. Use fixed EVAL to `RESTORE ... REPLACE ABSTTL` that one record from its saved
   DUMP/expiry into the retired `_e1` namespace. Return an integer marker only
   after confirming byte-identical DUMP, active schema-2 status and still-future
   expiry. This must actually erase the later logout state for that record;
   merely deleting a namespace or starting empty Valkey is insufficient proof.
   Verify the new epoch has no Session record for that old identity before start.
5. Start both actual Admins under `_e2`. Keep `_e1` unserved. On each recovered
   instance, old signed-in Cookie GET and native/JSON saves using its old CSRF
   fail with the existing unauthenticated navigation and no success cookie;
   no business effect occurs. A finite existing `CLIENT PAUSE ALL` check during
   this new-generation stale-cookie lookup yields 503 on both, rather than
   interpreting provider uncertainty as an absent identity or accepting it.
   After the finite pause, the reachable empty epoch again requires re-login.
6. Deliberately login Alice on recovered A; B accepts her fresh identity and
   reads the retained version-2 detail/history. Fresh Alice native/JSON writes
   on both remain 403. Present the old CSRF with Alice's fresh Session to a
   CSRF-protected lifecycle action and require 403 without logout. Deliberately
   create a fresh Bob login Session using valid local proof; both protected
   instances return 403 under the still-disabled account. No policy promotion
   is needed for this test. Lifecycle login and protected allow remain distinct.
7. Stop/drain recovered A/B. Run the same fixed read-only query and require exact
   equality with the pre-restore business receipt and policy bytes: no lost
   acknowledged update, extra item/history row, sequence change or schema/index
   change. Clean the exact scratch key; task namespaces belong exclusively to
   this disposable server. The parent removes the owned container and checks
   its listener closes. TempDir removal and child kill/reap on failure reuse the
   existing helper; final evidence must include all four successful drains,
   no remaining owned children/listeners and removal of owned temporary data.

This is **selected-record stale restoration**, not a full namespace backup or
full server/RDB/AOF restore. It leaves unrelated old-namespace keys unchanged,
preserves the captured absolute expiry and does not claim crash/power-loss
consistency, retained operation results, persistent logout, replicated Valkey,
HA, Sentinel/Cluster, RPO/RTO or production durability. The old record is unsafe
regardless of whether the wider dataset was restored. SQLite and policy are
retained, not rolled back or restored from independent backups. A filesystem
copy while SQLite writers run is not part of this procedure. No safe NFS,
coherent per-replica SQLite or cross-host trust is assumed.

## Implementation and verification boundary

After independent contract PASS, the only runtime-test file selected is
`examples/admin-dashboard/tests/auth_valkey.rs`, adding
`two_admin_processes_recover_with_new_session_epoch` as one ignored Linux
`auth-valkey` integration test. Reuse `Admin::start`, `stop`, Drop, policy,
Cookie, native form and header helpers. Extract only the successful login
rotation part if needed to exercise Bob's disabled protected state; existing
YT-126 test semantics must remain unchanged. Extend the raw TCP seam only for
these fixed integer-returning scripts and add the one fixed SQLite receipt
function required by the exact row/history acceptance. No general RESP client,
snapshot library, orchestrator or validation generator is selected.

Executed focused command (one actual passing test, not an ignored-test claim):

```sh
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --test auth_valkey -- --ignored --exact two_admin_processes_recover_with_new_session_epoch --nocapture
```

Run focused Rust formatting, strict Admin all-target Clippy, documentation and
whitespace checks. The existing YT-126 runtime journey and unchanged provider,
Core/browser/IdP/platform matrices are not rerun. No frontend or default runtime
source changes are planned; if that changes, stop and reassess the bounded
contract before implementation. Report compile/lint, actual HTTP/restore, SQL
reconciliation and cleanup evidence separately. Browser/CSP enforcement is not
rerun; HTTP checks retain existing CSP/private-response headers only.

## Actual focused results and cleanup

The actual selected test executed once against the parent's task-only Valkey
9.1.2 container, using the pinned image above and the explicit loopback
`YT139_VALKEY_URL`. The changed integration-test source was bound by SHA256
`8f0c842db56c5b768a718c670fad4826e110f15160bbc8f58dd43593bb38746c`.
Application runtime sources, Cargo manifests/lockfiles, provider and schema were
unchanged. The old YT-126 journey body remains byte-identical to the baseline;
its successful login rotation steps were extracted without removing its checks.

| Check | Actual result and scope |
| --- | --- |
| New exact ignored recovery test, explicitly executed | PASS: 1 passed, 0 failed/ignored; 1 historical test filtered out. Test 2.34 s; complete Cargo command 5.275 s. The old journey was not executed |
| Selected stale record restoration | Active schema-2 Session DUMP and original absolute expiry captured privately; logout acknowledged, old consumers stopped/drained, then exact active record restored into retired `_e1` with identical DUMP and original future expiry. Server-side integer markers passed; no old consumer served restored state |
| New epoch and provider uncertainty | `_e2` old-identity record and issuance authority absent before startup. Both actual recovered Admins rejected old-cookie GET/native/JSON saves with 303 and no success cookie. A finite 1300 ms pause yielded 503 on both; after recovery, old-cookie requests again required re-login |
| Fresh identity and current policy | Fresh Alice Session/CSRF rotated and read version-2 detail/history on both instances. Native/JSON writes remained 403; old CSRF against fresh logout was 403 without destroying the fresh Session. Fresh Bob login created a Session, but both protected decisions remained 403 under current disable policy |
| Complete business/policy reconciliation | Before/after full read-only SQLite receipts equal: schema 1, one item at version 2, two create/update history rows, history sequence 2, complete fields/timestamps and schema/index definitions. Current restrictive policy bytes equal; zero unknown business outcomes. The post-Session-snapshot acknowledged update survived |
| Targeted Rust formatting | PASS for the changed test source |
| Strict Admin Clippy | `cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --all-targets -- -D warnings` PASS, 0.718 s; compilation/lint evidence, not execution of every target |
| Owned cleanup | All four Admin lifetimes exited successfully with drain markers, were reaped and had closed listeners. Scratch deletion/absence passed and temporary SQLite/policy root was removed. Parent independently observed PID/port/root absence; owned Valkey stopped with exit 0, was removed and its loopback listener closed. No persistent volume was created |

The fixed Python query uses standard-library SQLite in read-only mode and one
read transaction; its complete synthetic business receipt is retained. Session
keys, binary DUMP, cookies, CSRF and credential proofs were not printed or saved
to reports. Only synthetic business data and bounded cleanup observations were
retained. HTTP assertions preserve private-response/CSP headers; they do not
rerun browser CSP enforcement or establish a production guarantee.

### Preserved fixture startup correction

The first task-only Valkey container exited 1 before any application test: its
root entrypoint attempted `chown` under dropped capabilities and failed. The
parent retained the failed container observation and log, removed that exited
owned container, and made one minimal fixture correction: use the same pinned
image's verified existing Valkey UID/GID `999:999`. Capabilities stayed dropped,
the root filesystem read-only, publication loopback-only and memory bounded to
64 MiB. The corrected fixture started successfully and later exited 0 during
cleanup. This was a fixture failure, not a product defect or recovery-stage
dispatch. The new test passed on its first execution; worker repair/retry counts
are zero. Both original and corrected owned containers are absent.

Contract-stage source findings and SHA256 bindings are retained in
`/tmp/yetlet-yt139-authority-recovery-20261023-ojzvhyhq/source-findings.json` and
`contract-summary.json`. `implementation-summary.json`, `recovery-test.json`,
raw test/lint/format outputs, `synthetic-business-receipt.json`,
`valkey-bootstrap-observation.json`, `valkey-fixture-correction.json` and
`cleanup.json` preserve actual execution, the initial failure, its correction and
independent cleanup. Documentation and whitespace checks PASS: 75 required
documents, 141 backlog items, internal links, Markdown structure and dependency
DAG. Final independent review PASS, zero issues, required followups and optional
notes; all seven reviewed hashes matched before and after review. The retained
`review-input.json`, `review.patch` and `review-final.json` preserve that snapshot
and result. Only completion status/evidence bookkeeping changed afterward; the
reviewed test source and proof remain unchanged. Parent-owned
backlog/roadmap/index/Admin usage updates mark YT-139 Done at the bounded
optional local reference scope. YT-140 real IdP
proxy/replica work and YT-141 performance work remain inactive; no actual operator,
consumer topology or SLO is a prerequisite for this local reference.
