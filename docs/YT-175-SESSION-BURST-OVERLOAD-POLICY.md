# YT-175 — Session admission burst and overload policy

Date: 2026-10-25 (Asia/Taipei). Status: **Done at document-only contract scope;
independent Reviewer PASS, zero issues/followups. Runtime and limits unchanged.**

## Scope and evidence

The user authorizes contract documents comparing Session burst/overload policy
options and recommendations, retaining current limits and requiring independent
Reviewer PASS before completion. No runtime, API, configuration, benchmark,
container/cloud allocation or new-change Commit/Push is selected.

The four [YT-174](YT-174-HOST-PASSIVE-SAMPLING.md) documents are committed as
`8ad4b01`, strict/builtin Commit Guard PASS, zero findings, tree
`18a123b0f176f41229d5b2c66cfad7c6bf6e2640`; no Push. Preserve all prior reviews,
failures and acceptance boundaries. Reuse evidence without rerunning it:

| Retained study | What it supports | What it does not establish |
| --- | --- | --- |
| [YT-171](YT-171-AUTH-CAPACITY-RECOVERY.md) | Twenty Session-busy failures at selected fixed loads; recovery succeeds | A universal capacity ceiling or exact internal cause of those historical failures |
| [YT-172](YT-172-SESSION-ADMISSION-BURSTS.md) | Provider barrier waves reject above bounded admission; nominally spaced waves differ | HTTP fairness or a zero-error burst guarantee |
| [YT-173](YT-173-HTTP-SESSION-TIMELINE.md) | Twelve new busy failures prove four active plus four pending owners | Exact release timestamps or unique historical causation |
| [YT-174](YT-174-HOST-PASSIVE-SAMPLING.md) | Nine new busy: eight of sixteen JSON burst requests, one of 10001 SSR observe requests; all prove four active/four pending | Causal latency/capacity improvement from lower observer cost |

YT-174's recorded sample wall cost falls 88.54%, but observe HTTP p95 does not
improve. Preserve raw failures, nearest-rank/native-percentile distinctions,
short-run/arrival-distribution confounds and advisory cross-clock limits.

## Current contract and responsibility boundaries

The [provider implementation](../crates/yetlet/src/session_redis/store.rs),
[configuration](../crates/yetlet/src/session_redis/mod.rs) and
[provider contract](../crates/yetlet/guides/session-redis.md) establish:

- The selected reference uses **four active operations and eight total admitted
  operations**, including active work. Eight total is not eight extra waiters.
  Four pending describes the observed full gate with four active; it is not a
  separate universally fixed queue length.
- Admission is immediate `try_acquire_owned`; a full gate returns `Busy` without
  waiting for a total-admission slot. Admitted operations await an active permit
  for at most 80ms, capped by the remaining caller deadline.
- The shared **400ms deadline is per Session API operation**, across its request
  lock, permit and backend phases. It is not an end-to-end HTTP budget; several
  Session operations and finalization can occur within one request. `QueueTimeout`
  can also arise at a request lock; an error category alone is not an exact phase
  trace. The configuration constructor accepts active limits 1–4; this reference
  remains at four, with the existing two-times total-admission relationship.
- Store clones share semaphore/worker ownership. Separately connected managers
  create separate local budgets, even in one process or with the same namespace.
  A shared Valkey namespace shares Session state, not a distributed admission
  quota. Replicas can multiply backend load; per-user/tenant isolation is not
  provided by the current gate.
- Once dispatched, owned work retains both permits until completion/disconnect
  or shutdown reaping. Caller cancellation/timeout does not reclaim live backend
  capacity or prove a write was cancelled. Retain the existing two-second drain
  and all result/reconciliation, network, TTL, payload and security bounds.

| Mechanism | Unit and scope | Existing behavior / boundary |
| --- | --- | --- |
| Session admission | Operations sharing one Store | Bounds backend work and short active-permit waiting; full gate can reject a brief cluster |
| HTTP admission | Requests sharing an application gate | [Starter/YT-127](YT-127-ADMISSION-OVERLOAD.md) has immediate admission, zero HTTP queue and an application deadline; it is not installed in the Auth workload merely by selecting Session |
| Rate limit | Requests per time/key sharing one budget | [YT-117](YT-117-RATE-LIMIT.md) is independently opt-in and per-process; reuse policy values does not share accounting |
| Auth / authorization | Verified identity, then application resource/action policy | Infrastructure pressure cannot become anonymous success, login expiry or permission denial |

The Starter reference's 32 HTTP handlers/four SQL permits are its own application
choices, not Session recommendations. An HTTP request may make multiple Session
calls; background work and finalizers can share the Store, while backend work
can outlive its caller. An outer request cap therefore cannot guarantee that
Session never returns Busy. Neither an arrival-rate quota nor a larger HTTP
queue increases provider capacity.

Keep [ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md): explicit native Axum/Tower
composition, typed application state, no mandatory middleware/DI/Core fields or
browser dependency. This document creates no policy type, Settings key or API.

## Options and recommendation

These options are application choices, not automatically installed features.

| Option | Benefit | Cost / condition | Recommendation at this scope |
| --- | --- | --- | --- |
| Keep bounded Session admission and fail fast | Protects real outstanding work; bounded memory/waiting | Bursts above free total slots can receive 503 even at a modest average rate | **Retain now**, including the observed failures |
| Early bounded HTTP admission, zero added queue | Rejects excess protected-route work before expensive Auth/Session/SQL phases | Requires an application gate shared across all intended routes; account for finalizers, background work and cancellation | **Preferred first integration option if a consumer needs explicit overload handling**, separately authorized |
| A bounded outer HTTP queue | Can smooth a finite arrival cluster without changing Session permits | Adds waiting/request memory; stacked waits can exhaust the response budget; no throughput guarantee | Alternative only after an application selects queue size, timeout and total budget; not selected now |
| Opt-in endpoint/group rate limiting | Enforces time/key quotas and can reduce offered bursts | Does not bound concurrent slow work or guarantee Session admission; keys and budget sharing must be explicit | Complement to admission when quotas are wanted; remain disabled unless the application enables it |
| Bounded caller retry for genuinely safe reads | May let a later attempt see recovered capacity | Can amplify overload, consume deadlines or synchronize clients; does not recover uncertain mutations | No new automatic retry; optional future caller policy only with explicit attempt/time/backoff bounds |

The recommendation is to keep the provider guard as the final safety boundary,
use explicit temporary-unavailable responses, and avoid hidden retries. If a
consumer later needs different burst acceptance, first select its application
request/latency policy; evaluate a small outer gate before expanding queues or
provider capacity. A zero-queue outer gate sheds load rather than promising to
accept all sixteen burst requests. Accepting more requests may require bounded
waiting or pacing and trades latency/resources; it needs separate evidence.

For an optional early gate, reject before Session resolution where possible and
keep the gate around downstream response construction/finalization. Every layer
must preserve existing private-response/security headers even on early rejection;
a native middleware order is meaningful, not just a list of enabled components.
Do not authenticate an expensive per-user quota lookup before the protection it
is meant to provide. Pre-auth keys need their own trust/cardinality limits;
raw forwarded headers, cookies or claimed tenant IDs are not trusted identity.
Health exceptions remain application-owned and bounded; they do not bypass
business authorization or create a guaranteed priority lane inside the Store.

Do not add an unbounded buffer, reserve Session permits for a whole HTTP request
while that request needs another permit, release permits at caller timeout,
instantiate extra managers to evade the shared budget, or fall back to Memory/
anonymous/stale identity when the authoritative provider is unavailable.
Priority lanes, adaptive limits, circuit breakers and distributed quotas are
unselected designs, not capabilities implied by this contract.

## Response and retry semantics

| Actual condition | Contract boundary |
| --- | --- |
| Session Busy, queue/caller failure, closed/unavailable backend | Retain redacted private 503 handling; do not convert infrastructure pressure to 401/403 or redirect to login |
| A real rate quota rejects a key | Existing limiter returns no-store 429 and its computed Retry-After; capacity/internal limiter failure remains 503 |
| A verified missing/expired/rotated/revoked identity | Preserve the application's established unauthenticated response or local login redirect; unavailable lookup is a different condition |
| Authenticated but denied business action / valid CSRF check returns false | Preserve the existing authorization/CSRF denial; infrastructure failure is not a false validation result |
| A separately selected HTTP deadline expires | Preserve that application's deadline mapping; Starter uses 504 and warns of uncertain submitted writes, rather than relabeling every provider failure 504 |

This mapping recommendation matches the
[Auth workload](../examples/axum-workload-comparison/src/bin/auth_postgres.rs),
[Auth contract](../crates/yetlet/guides/auth.md) and
[rate limiter](../crates/yetlet/guides/rate-limit.md); the Session provider does
not itself define every handler's HTTP policy. Keep actual malformed-input,
configuration and conflict semantics separate. Do not clear/rotate cookies,
replay login/logout or emit successful authentication because a lookup is busy.
Preserve no-store/private headers, CSRF, authoritative revocation/permission
freshness and tenant isolation. A 401 still needs the existing applicable
WWW-Authenticate contract; form flows can use their explicit redirect instead.

HTTP distinguishes temporary service overload from request-rate quotas;
Retry-After on 503 is optional, not a required recovery promise.
See [RFC 9110 §15.6.4](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.6.4)
and [RFC 6585 §4](https://www.rfc-editor.org/rfc/rfc6585.html#section-4).
Do not derive a recovery estimate from the 80ms queue bound or return Retry-After
solely because a semaphore is full. Keep the current Session response without
an invented hint; a future application may supply a meaningful conservative
hint, not reserved capacity or retry authorization. Header syntax/timing is in
[RFC 9110 §10.2.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.3).

There is no new server, proxy or client retry loop. A future read retry must
have an application-confirmed safe operation, bounded attempts/backoff with
jitter, one total time budget including waiting, and normal fresh authentication/
authorization on every attempt. An absent Retry-After is not permission for a
hot loop. No numeric retry defaults are chosen by this document.

A Busy result proves only that the particular Store call was not admitted;
it does not prove the whole HTTP action had no earlier effects. Timeout/lost
response or finalizer 503 can follow committed business or Session mutations.
Retain `OutcomeUnknown`/server-owned reconciliation semantics; resolving a ticket
does not replay a mutation or authenticate a retired identity, and cancellation
may lose a ticket. Do not blindly retry unsafe actions, CAS callbacks or an
unknown login/logout outcome. Reconcile application state first; no new generic
idempotency mechanism is claimed. Automatic non-idempotent retry restrictions
are described by [RFC 9110 §9.2.2](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.2.2).

## Application choices left open

These are advisory inputs for a later consumer integration, **not operational
ownership/SLO gates for completing this framework document**:

| Choice | What the consumer would select |
| --- | --- |
| Burst objective | Whether prompt explicit rejection is acceptable, or how much bounded waiting is worth accepting more work |
| Request budget and scope | Protected routes, shared gate instance, Session calls/fan-out/background producers and finalization/cancellation behavior |
| Quota, if wanted | Enablement through existing Settings, trusted key, endpoint/group policy, reused values versus shared budget and per-instance versus global expectations |
| Retry, if wanted | Truly safe operation, attempt/time/jitter bounds, reconciliation for uncertain writes and any meaningful response hint |

No owner, hardware allocation, deployment topology or production SLO must be
chosen to accept YT-175. No application policy, numeric cap, Settings value or
provider limit changes until separately selected. Research-build timings are
not production tuning targets, and local admission is not cross-instance fairness.

## Acceptance and next boundary

Only this report, Backlog, Roadmap and documentation index change. Verify
current source/evidence bindings and new documentation structure/links/DAG;
no build, benchmark, provider/HTTP/security test or prior analysis is replayed.
No test resource is created, and no production qualification gate is expanded.

Independent Reviewer PASS must precede Done. Review options against concrete
source semantics and retained YT-171–174 evidence, distinguish recommendations
from implemented/current behavior, and preserve all initial failures/results.
The task evidence lives under
`target/yt175-session-burst-overload-policy-20261025/evidence/`.
YT-096 remains In progress with D1–D6 Pending; YT-020 remains Blocked.

Independent Reviewer `/root/yt175_session_burst_overload_contract_review_20261025`
returns **PASS**, zero issues/required followups; native reviewer/model/high effort
are verified. Four source, seventeen original task-evidence, forty-one accepted
and sixteen research-input bindings remain stable through review. Documentation
checks pass: 75 documents/175 backlog items, links/structure/whitespace/DAG and
`git diff --check`. Only status/review-reference updates follow PASS.
No prior validation or analysis is replayed and no test resource is created.
The initial auxiliary-inspection preflight context conflict is retained in the
evidence, corrected before dispatch with the same factual scope; no product
repair, model escalation or workflow restart occurs. `closeout.json` records
final source hashes, unchanged accepted inputs and the exact Git state.

The next unblockable action is consumer selection of an application overload
policy if an integration is desired. No implementation/measurement item is
automatically activated, and the present four-active/eight-total reference
remains the baseline. New documentation remains uncommitted/unpushed.
