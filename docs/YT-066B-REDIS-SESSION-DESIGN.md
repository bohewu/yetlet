# YT-066b Redis Session Provider Design

**Date:** 2026-10-10
**Task:** YT-066 / M10g, bounded YT-066b design
**Status:** Revised design independently reviewed PASS on 2026-10-11 after four material corrections. The subsequent private YT-066b1 prototype and pinned dependency experiment are recorded in [execution evidence](YT-066B1-VERIFICATION.md); the public/full provider remains unimplemented and its cookie-ordering guarantee remains unresolved.
**Related:** [Provider operations](PROVIDER-OPERATIONS.md), [M10g verification](M10G-VERIFICATION.md), [YT-065 / YT-066 plan](YT-065-066-PLAN.md), [ADR 0005](adr/0005-opt-in-capability-modules.md), [M8c verification](M8C-VERIFICATION.md), [planning verification](YT-066B-PLANNING-VERIFICATION.md).

## 1. Goal and non-goals

YT-066b may add a separately opt-in, asynchronous Redis-backed Session provider without changing the current process-local Memory Session contract. The provider must make network I/O, serialization, contention and uncertain outcomes visible rather than hiding them behind a synchronous compatibility layer.

The current `yetlet-session` crate remains the bounded in-memory provider:

```rust
session.read()?;
session.try_update(|state| { /* one local mutation */ })?;
session.rotate()?;
session.destroy()?;
```

The Redis provider may use the same semantic method names on separate types, with `.await` exposing remote I/O:

```rust
session.read().await?;
session.try_update(|state| { /* one local mutation */ }).await?;
session.rotate().await?;
session.destroy().await?;
```

This design does **not** authorize:

- changing `Session<T>` or `SessionManager<T>` to async;
- adding `_async` suffixes merely to imitate another ecosystem;
- blocking a Redis future to preserve a synchronous facade;
- adding Redis to Yetlet Core, the default Axum adapter, browser/WASM crates or the Memory provider graph;
- a generic `SessionStore<T, Backend, Serializer, Transaction, ...>` framework;
- Redis Cluster, Sentinel, cross-host HA, production SLOs, SQL Session or public crate publication;
- automatic fallback to Memory when Redis is unavailable; or
- weakening existing rotation, revocation or stale-response security claims without an explicit maintainer decision.

YT-066 remains In progress after this design. YT-066a is accepted only at its single-host Docker/Memory limitation scope. YT-066c cross-VM work remains cancelled.

## 2. Existing contract that must be preserved

The Memory provider uses one store lock to supply these bounded guarantees:

- an application closure mutates a clone and commits only when the closure succeeds;
- successful dependent updates do not overwrite each other within one process;
- update and rotation share one ordering boundary;
- a retired identity is rejected;
- a stale request cannot clear a replacement cookie before its response headers are produced; and
- `destroy` revokes the current identity rather than silently creating a replacement.

A remote provider must not claim parity merely because Redis stores bytes or because a third-party middleware can load and save a Session snapshot. The acceptance contract is semantic: versioned atomic mutation, explicit contention, atomic identity lifecycle, fail-closed errors and tested response-cookie behavior.

## 3. Provider and crate boundary

The proposed implementation boundary is a new unpublished opt-in crate:

```text
yetlet-session          # existing synchronous in-memory provider
yetlet-session-redis    # future asynchronous Redis provider
```

Conceptual provider-owned types:

```rust
RedisSessionManager<T>
RedisSession<T>
RedisSessionConfig
RedisSessionError
RedisSessionUpdateError<E>
RedisOperationId
```

These names are directional, not a frozen public API. The first implementation must avoid broad backend traits until a real Redis consumer proves which operations converge. `T` will require an explicit serialization contract inside the Redis crate; the Memory crate must not gain that bound.

The Redis middleware remains native Axum/Tower composition. It may expose a typed extractor or request extension, but it must not add a framework-wide DI container, service locator, plugin registry or browser-visible handle.

## 4. Current ecosystem comparison

No dependency is selected solely from memory or popularity. The implementation spike must pin exact versions after checking MSRV, feature graph, cancellation/reconnect behavior and API fit.

| Candidate | Useful current capability | Yetlet cost or gap | Design disposition |
|---|---|---|---|
| `redis` / redis-rs | Officially documented async connections, reconnecting `ConnectionManager`, command pipeline and server-side `Script` support | A cloneable multiplexed connection does not by itself define bounded in-flight work, wait time, outcome ambiguity or Session semantics | **Preferred first API spike.** Add an explicit Yetlet semaphore, wait timeout and command timeout; reject it if bounds/reconnect behavior cannot be demonstrated cleanly |
| `deadpool-redis` | Mature Redis integration with explicit pool construction and managed object checkout | Adds a pool dependency and still does not supply Session CAS, rotation, idempotency or ambiguous-write reconciliation | **Fallback / comparison candidate** when an observable bounded physical pool is preferable to one managed multiplexed connection |
| `fred` | Broad async Redis client with pooling, reconnect, cluster/Sentinel and backpressure controls | Larger API/dependency surface than the first single-primary prototype needs | **Comparison only for the first slice.** Reconsider if direct redis-rs bounds are inadequate or later topology needs justify the surface |
| `tower-sessions` plus Redis store | Mature application-level middleware and a useful Axum/Tower interoperability example | Its load/mutate/save snapshot model and documented concurrent dependent-mutation caveat do not establish Yetlet's atomic `try_update`, rotate/revoke or stale-cookie guarantees | **Do not wrap as the Yetlet provider.** Retain as a DX and integration comparator |

Primary references:

- Redis transactions and optimistic locking: <https://redis.io/docs/latest/develop/using-commands/transactions/>
- Redis server-side Lua execution: <https://redis.io/docs/latest/develop/programmability/eval-intro/>
- redis-rs `ConnectionManager`: <https://docs.rs/redis/latest/redis/aio/struct.ConnectionManager.html>
- redis-rs `Script`: <https://docs.rs/redis/latest/redis/struct.Script.html>
- deadpool-redis: <https://docs.rs/deadpool-redis/latest/deadpool_redis/>
- fred: <https://docs.rs/fred/latest/fred/>
- tower-sessions concurrency caveat: <https://docs.rs/tower-sessions/latest/tower_sessions/#data-races-under-concurrent-conditions>

Current `cargo info` metadata for the five comparison crates is retained in ignored `target/yt066b-cargo-info.txt`. This design deliberately does not turn that transient snapshot into a committed version selection.

## 5. Preferred first spike

The first semantics spike should use redis-rs directly, not because it is already accepted for production, but because it is the smallest way to test Yetlet's real requirements:

- one async `ConnectionManager` or equivalent current redis-rs connection surface;
- a provider-owned `Semaphore` limiting total in-flight operations;
- a bounded semaphore wait;
- bounded connect/reconnect and command timeouts;
- server-side scripts for mutations that require one atomic decision;
- one local single-primary Redis topology; and
- no public stability or durability claim.

The spike must record the exact dependency tree, enabled features, MSRV result, reconnect behavior, cancellation behavior and maximum observed in-flight work. If the connection manager cannot make the required bounds observable and testable, the same semantics spike should be repeated with `deadpool-redis`; this is a dependency-selection experiment, not authorization for two shipped backends.

## 6. Record envelope and key scope

A Redis Session is a versioned server-side record, not a serialized browser Session. The cookie continues to contain only an opaque identity. Store provider metadata in a Redis Hash; the state field is an opaque serialized byte string which scripts do not decode or modify.

Conceptual private envelope:

```text
schema_version
record_revision
lifecycle_status          active | rotated | revoked
serialized_state
csrf_token
absolute_expires_at
generation               identity lifecycle generation
```

Use separate keys for records and operation results:

```text
<prefix>:session:<opaque_identity>    Hash: the fields above
<prefix>:operation:<operation_id>    Hash: operation kind, argument digest,
                                    terminal status and committed revision
```

Operation-result keys are independent of a Session record's latest mutation. A mutation script records its terminal result in the same atomic execution as the state transition; later updates, rotation and revocation must not overwrite or delete that result. Results contain neither application callback return values nor state, CSRF tokens or replacement bearer identities. A retired record is a rejecting tombstone, never a way to recover the replacement cookie.

Requirements:

- The envelope schema is provider-private and explicitly versioned.
- Unknown schema versions fail closed; they are never treated as an empty Session.
- State serialization failure is distinct from Redis unavailability.
- A configurable maximum encoded payload is enforced before dispatch and after read.
- Keys contain only a configured prefix and opaque random identifiers, never usernames, claims or state.
- Active records receive Redis TTL and retain an absolute expiry inside the envelope so clock/TTL inconsistencies fail closed.
- Redis primary time is authoritative for expiry checks; ordinary CAS updates preserve absolute expiry, while rotation intentionally starts a new lifetime.
- Rotation and revocation retain bounded tombstones; once a tombstone expires, a missing identity still fails closed. Update/rotate/revoke never upsert a missing record, and create never adopts an identity presented by a browser.
- Operation evidence has its own configured bounded TTL, independent of record and tombstone expiry. Its retention starts when the script records a result and must cover the maximum immutable operation dispatch window plus the chosen reconciliation window. Repeated resolution does not extend that TTL.
- A server-checked immutable dispatch deadline prevents an expired operation from executing later. Missing or expired operation evidence means **unresolved**, never proof of noncommit or permission to replay an old create. The spike must bound admission and retained operation count as well as payload size; TTL alone is not a global capacity limit.
- The first prototype targets one Redis primary. A future Cluster design must place every key touched by one script in the same hash slot or redesign the operation; no Cluster compatibility is inferred.

The prototype may use serde JSON for the opaque state field, but metadata remains separate Hash fields and the encoding is not a stable public wire contract. The implementation review must compare payload size, deterministic re-encoding and migration behavior before accepting it.

## 7. Atomic operation vocabulary

The provider should implement only the operations needed by its real Session consumer:

1. **Create** — create one active record only when the opaque identity is absent.
2. **Read** — return the active envelope and current revision, or a precise missing/expired/rotated/revoked result.
3. **Compare-and-set update** — commit a complete candidate envelope only when the expected revision and active lifecycle status still match.
4. **Rotate / replace-and-rotate** — atomically retire the old identity and create the replacement with a fresh identity and CSRF token.
5. **Destroy / revoke** — atomically retire the active identity and retain a rejecting tombstone.
6. **Resolve operation** — read the dedicated result key using a provider-owned operation ticket and verify the operation kind/argument digest. Return committed, explicitly rejected, or unresolved; do not infer a result from the Session's current revision or lifecycle.

Redis `WATCH`/`MULTI`/`EXEC` can provide optimistic locking, and a server-side Lua script can validate and mutate the required keys atomically. The spike should prefer small, deterministic scripts with explicit result codes. Only an explicit `NOSCRIPT` response, which proves that invocation did not execute, permits bounded loading and retry of the **same immutable operation arguments** within the original deadline. A transport timeout is not `NOSCRIPT`. No retry re-runs application code or generates a different identity. Scripts must validate record types, schema and arguments before writes; Redis script isolation is not rollback after a runtime error, which must remain an uncertain mutation outcome unless independent evidence proves a terminal result.

## 8. `try_update` and conflict semantics

The application closure executes exactly once against a cloned state value:

1. asynchronously read active state and revision;
2. clone state locally;
3. execute the synchronous application closure once;
4. serialize and size-check the candidate;
5. issue one CAS mutation with expected revision and a stable operation ID; and
6. return success, explicit conflict, a Session lifecycle error or an uncertain outcome.

The provider must not automatically re-run an arbitrary user closure after conflict or transport failure. A closure may perform side effects outside Session state. The caller may explicitly start a new operation after handling conflict, but that is a new closure invocation and not hidden provider retry.

A conflict is not an outage:

```text
Conflict { expected_revision, observed_revision }
```

The directional Rust return shape remains `Result<R, RedisSessionUpdateError<E>>`: acknowledged commit returns the closure's original `R`; application rejection returns `Application(E)` without dispatching a mutation; a definite conflict or Session failure returns a typed error. `OutcomeUnknown` carries a provider-owned operation ticket, **not `R`**. This deliberately drops the local `R` on an uncertain outcome rather than requiring it to be cloneable, serializable or recoverable from `T`.

`resolve_operation(ticket).await` returns commit status only. Even when resolution proves success, it cannot reconstruct an arbitrary `R`, undo external callback side effects or replay the callback. Callers needing a durable application result must arrange that in their own application contract. Cancellation/process loss may discard both `R` and a ticket not yet handed to the caller; if the caller retained a ticket, only status can later be resolved within its evidence window. No recovery of arbitrary callback values or discovery of lost tickets is promised.

A load test with simultaneous same-revision dependent updates must demonstrate that exactly one CAS succeeds per revision and that every reported success remains present. No successful change may be overwritten by a later stale snapshot.

## 9. Error and uncertain-outcome model

At minimum, callers must be able to distinguish:

- configuration or invalid key-prefix error;
- semaphore/pool wait timeout before Redis dispatch;
- Redis unavailable or command timeout;
- missing, expired, rotated, revoked or destroyed Session;
- CAS conflict;
- payload too large;
- serialization or unsupported schema version;
- script/protocol contract violation; and
- **outcome unknown** after a mutation may have reached Redis.

A network error after command dispatch cannot safely be described as “not committed.” Every mutation receives a provider-generated operation ID and immutable ticket binding its kind, arguments, namespace/generation and dispatch deadline. The dedicated operation-result key is written with the mutation and survives subsequent mutations for its own bounded retention window. Until resolution, the provider returns explicit `OutcomeUnknown` and does not invent a Memory fallback, new Session or success response.

Reconciliation is a read of that result key, not a retry of an uncertain mutation. A matching committed result proves that operation's historical commit even if another operation has since changed or retired the Session; it does not authorize a now-retired identity. An explicit rejection recorded by that script can prove noncommit. A missing, expired, inconsistent or unreadable result remains unresolved. Do not infer noncommit from a later state, extend a ticket deadline or recreate an expired identity to obtain an answer. An exact invocation repeated only after explicit `NOSCRIPT` must check any existing operation result before mutation and reject a mismatched argument digest.

## 10. Rotation, revocation and stale responses

Atomic Redis scripts can ensure that all application instances immediately reject a retired identity and observe one authoritative replacement. That is necessary but not sufficient for HTTP cookie ordering.

The design distinguishes two boundaries:

### 10.1 Staleness observed at final revalidation

Before adding `Set-Cookie` or a clearing cookie, the async response finalizer must revalidate the request identity, record revision and pending cookie directive. If that check observes rotation/revocation, the finalizer suppresses the stale directive. This is a mandatory prototype scenario, not a guarantee covering every instant before header emission or proof of parity with all Memory behavior.

### 10.2 After the final check, including before emission

There is still a reachable race: finalizer A validates identity X; instance B rotates X to Y; A then appends and emits `Set-Cookie: X`. No response bytes need have been emitted when B rotated. Another Redis read merely moves this gap. The same analysis applies to stale clear directives and competing rotations. A final store check alone does not close the pre-emission boundary.

### 10.3 A response already emitted or in flight

Redis cannot retract bytes that have already been committed to an HTTP response. Arbitrary cross-stream/network delivery order may still allow an older response to overwrite a newer browser cookie even though Redis correctly rejects the retired server identity.

The first prototype must reproduce both the check-to-emission race and reordered delivery with a real HTTP cookie jar. Assert retired-identity rejection separately from whether the jar retains the replacement identity. It may not claim complete stale-cookie ordering merely because scripts are atomic. An unresolved race blocks the full provider stage. Before that stage can be accepted, the maintainer must choose one of these evidence-backed directions:

- add and test a client/transport-aware fencing or response-serialization protocol that prevents an older emitted cookie directive from winning; or
- explicitly narrow the provider guarantee, keep the provider experimental/private and accept that a reordered stale response can cause fail-closed Session loss, never old-identity authorization.

Neither direction is selected by this design. Do not silently narrow the guarantee, add mandatory client JavaScript or use the design review PASS as acceptance of the unresolved cookie behavior.

Following a rotation tombstone to the replacement and granting access to a holder of the old cookie is forbidden: it would let a compromised retired identity recover the new Session.

## 11. Resource and retry bounds

`RedisSessionConfig` must validate explicit bounds rather than inherit unbounded client defaults:

- key prefix and maximum key length;
- absolute Session TTL and tombstone TTL;
- independent operation-result TTL, immutable dispatch deadline and bounded operation-result admission;
- maximum encoded payload;
- maximum in-flight provider operations;
- semaphore or pool wait timeout;
- connect/reconnect timeout and bounded backoff policy;
- command timeout;
- maximum script reload attempts for the same immutable operation;
- shutdown/drain deadline; and
- safe telemetry labels without Session IDs, cookie values, state or credentials.

A dropped Rust future may not prove that Redis did not execute the command. Cancellation after dispatch therefore uses the same uncertain-outcome model. Shutdown must stop admitting new work, drain within the configured deadline and leave no owned background task or connection manager process.

## 12. Security and configuration ownership

The application supplies Redis endpoint, TLS, ACL identity and secret material through its own configuration boundary. Yetlet documentation may show placeholders but must never commit credentials or print credential-bearing URLs.

Required controls:

- fail closed on Redis outage, schema error or uncertain authorization state;
- no Memory fallback;
- opaque random Session and operation IDs;
- constant-time CSRF verification after trusted record load;
- Secure/HttpOnly/SameSite cookie policy identical in intent to the Memory provider;
- redaction of endpoint credentials, Session IDs, CSRF values, serialized state and script arguments;
- no server capability or Redis dependency in browser/WASM builds; and
- explicit key-prefix isolation for tests and applications.

Redis durability is a topology decision, not a client-library property. The qualification topology must record whether AOF, RDB or no persistence is selected and test only the selected behavior. An application restart is different from a Redis restart; neither implies HA.

## 13. Bounded implementation stages

### YT-066b0 — Design and dependency/API spike plan

This document and its independent reviewer. No dependency or code change. Acceptance: exact contract, candidate comparison, open cookie-ordering gate and test plan are internally consistent.

### YT-066b1 — Private semantics spike

After design PASS only:

- pin one redis-rs candidate and features;
- prove MSRV/build/dependency isolation;
- prototype the versioned envelope and small scripts;
- test create/read/CAS/rotate/revoke/resolve-operation against one disposable Redis;
- measure configured in-flight bounds and timeout behavior; and
- compare deadpool only if direct redis-rs bounds are inadequate.

This stage is internal and may live behind an unpublished crate/feature. It does not promise public API stability.

### YT-066b2 — Provider implementation

Only after spike and reviewer PASS:

- add the separate async provider types;
- integrate scoped Axum/Tower middleware;
- preserve Memory dependency/API isolation;
- add compile-level DX and lifecycle tests; and
- independently review implementation and evidence.

### YT-066b3 — Three-application Redis qualification

Use the accepted YT-066a application/proxy topology plus one pinned Redis service. Validate cross-instance read, CAS contention, rotate/revoke visibility, stale pre-header suppression, app restart, TTL, outage, uncertain outcome, resource bounds, cancellation/drain and full cleanup. Redis restart behavior is tested only under the explicitly selected persistence setting.

No phase marks YT-066 Done while the in-flight cookie-ordering decision, actual Redis semantics or cleanup evidence remains open.

## 14. Required test matrix

| Area | Required evidence |
|---|---|
| Dependency isolation | Core, default server, browser/WASM and Memory Session graphs contain no Redis dependency without explicit provider selection |
| API/DX | Memory examples remain synchronous; Redis examples use the same semantic names with `.await`; no `_async` facade or `block_on` |
| Create/read | Session created through A reads exact typed state through B and C |
| CAS | Simultaneous same-revision dependent updates produce explicit success/conflict and retain every reported success |
| Closure behavior | Application closure executes once per explicit call; application error writes nothing; an arbitrary non-serializable `R` returns only on acknowledged success and is deliberately lost after unknown outcome/cancellation; resolution returns status without replay |
| Rotation | One atomic replacement is visible across A/B/C; old identity is rejected everywhere |
| Revocation | Destroy/revoke is visible across A/B/C and cannot resurrect through retry or recovery |
| Staleness observed by finalizer | A final check observing rotate/revoke suppresses stale Set/Clear directives |
| Check-to-emission race | Pause A after final validation, rotate X to Y on B, then emit A's stale Set/Clear response; observe both server rejection and the real HTTP cookie jar, without claiming the final read prevents this race |
| In-flight response boundary | Force older prepared/emitted responses to arrive after replacement responses, including stale clear and competing rotations; unsupported guarantees remain explicit until a chosen strategy passes |
| TTL | Absolute expiry and Redis TTL agree within documented clock assumptions; expired state never becomes a new authenticated Session |
| Outage | Pool wait, connect and command failures fail closed; no Memory fallback |
| Uncertain mutation | Lose A's mutation reply, then update/rotate/revoke with B; resolve A from its dedicated result key without duplicate mutation or callback replay, independently of the current Session record |
| Expired reconciliation | Expire the result key and record/tombstone; resolution stays unresolved and cannot replay create or revive an identity; explicit rejection and unknown outcome remain distinct |
| App restart | Restarting one Yetlet instance does not lose Redis state |
| Redis restart | Tested only against the selected AOF/RDB/no-persistence contract; no inferred durability |
| Bounds | Maximum in-flight work, wait/command timeouts, payload limit and shutdown drain are exercised |
| Cleanup | All app/proxy/Redis containers, networks, volumes, generated images, temp state and owned processes are removed on pass/failure/cancellation |
| Redaction | Reports/logs contain no cookie, Session ID, CSRF, serialized state, credentials or exception text that may contain them |

## 15. Completion and open decisions

This design can be marked reviewed without selecting a production topology. YT-066b implementation cannot begin until an independent read-only reviewer accepts the design.

Still open after design review:

- redis-rs direct connection manager versus deadpool fallback after the API/bounds spike;
- concrete serialization format and migration policy;
- operation-reconciliation retention and tombstone TTL;
- the in-flight cookie-ordering guarantee and whether client/transport cooperation is acceptable;
- Redis persistence mode for qualification;
- production TLS/ACL/secret ownership; and
- any future Sentinel/Cluster/HA work.

These are explicit gates, not reasons to weaken the current Memory provider or to claim distributed qualification prematurely. YT-066 remains In progress and YT-020 remains Blocked.
