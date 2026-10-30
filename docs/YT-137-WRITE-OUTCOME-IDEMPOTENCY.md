# YT-137 — Application write outcomes and idempotency

Date: 2026-10-23. Status: Done at documentation-only research scope after
independent Reviewer PASS, zero issues and required followups.
Documentation-only contract research against commit
`06868612db589424f9e9df530d9c0cf64782aa08`. The selected
[YT-137 contract](PRODUCTION-DEPLOYMENT-PROFILES.md#conditional-future-work-contracts)
permits an independently reviewed application-owned decision or a justified
no-automatic-retry decision. Implementation requires separate authorization.

## Selected decision and concrete need

Retain **no automatic mutation retry** for Production Starter and Board/Admin.
Keep their existing routes, payloads, settings and explicit save/reconciliation
behavior. Preserve Modular Shop's existing application-owned checkout
deduplication within its documented scope. Select no general idempotency crate,
middleware, public header, new response protocol or shared operation ledger.

The concrete need is an unacknowledged Starter create: its HTTP deadline can
expire after a blocking SQLite operation starts, and that operation can still
commit. The caller may never learn the new issue ID. Submitting the same title
and body again can create a second issue. This is a real reconciliation risk,
even though no automatic replay is selected. The bounded prototype can retain
deliberate human reconciliation and its unresolved outcomes; it does not claim
safe unattended retries or exactly-once effects. A consumer requiring those
guarantees must separately select and implement an application contract.

This follows [ADR 0002](adr/0002-events-and-actions.md),
[ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md): typed explicit commands,
ordinary Axum/Tower handlers, opt-in capabilities and application-owned
authorization. Neither research completion nor a future consumer decision
activates a runtime feature.

## Current consumer evidence

Line references below describe the bound source revision, not fresh execution
results. Tests were inspected as source; historical PASS records retain their
original scope and were not rerun.

| Consumer and actual need | Current behavior and authority | Storage/security boundary | Failure limit and selected decision |
| --- | --- | --- | --- |
| Production Starter: issue POST acknowledged late or not acknowledged; status PATCH response lost | Create inserts a new row on every accepted call. Status change compares `expected_version` and increments it. HTTP response creation has a deadline; started blocking SQL is independently owned. [Store](../examples/production-starter/src/store.rs#L159), lines 159–211 and 279–319; [HTTP](../examples/production-starter/src/web.rs#L166), lines 166–208, 557–652 and 723–780 | Schema 2 SQLite business authority has no operation ledger. SSR writes are anonymous Session/CSRF protected; the separate API requires the configured shared bearer secret and rejects Origin-bearing requests. It does not establish an individual user or tenant. [Schema](../examples/production-starter/migrations/0001_init.sql), [API authorization](../examples/production-starter/src/web.rs#L267), lines 267–286, and CSRF lines 338–351 | 504/disconnect is unknown, not rollback. A repeated PATCH may conflict after the first committed, without proving which caller committed. Keep manual inspection and no automatic mutation replay; ambiguous create may remain unresolved |
| Board/Admin: selected-item save, native create/update, uncertain response | Admin starts the actual Board application and loads the shared Rust triage client. A save issues one fetch; the UI claims a pending item before awaiting. Access/conflict/unconfirmed feedback requires a fresh page and prevents same-draft save. Native forms use explicit POST/redirect/get. [Admin composition](../examples/admin-dashboard/src/main.rs#L55), lines 55–78; [browser](../examples/board-triage/src/browser.rs#L349), lines 349–387 and 550–621; [draft policy](../examples/board-triage/src/lib.rs#L119), lines 119–172; [native save](../examples/lab-operations-board/src/lib.rs#L719), lines 719–800 | Application Session identity, project and write authorization, CSRF and current pre-SQL policy checks govern access. SQLite updates compare version/project and append history in the same transaction. [Save endpoint](../examples/lab-operations-board/src/triage.rs#L285), lines 285–336; [authorization](../examples/lab-operations-board/src/lib.rs#L507), lines 507–536; [business transaction](../examples/lab-operations-board/src/database.rs#L172), lines 172–209 | No durable operation identity or keyed create. Pending state lasts only in the tab, and fresh-page/manual reapplication cannot prove an earlier operation's identity. Keep the accepted [YT-107 recovery flow](YT-107-FORM-FEEDBACK.md#slice-2-contract) and no automatic mutation retry |
| Modular Shop: SQLite checkout committed before Session cart cleanup or response acknowledgment | Handler checks current Session user, CSRF and cart revision/nonempty snapshot; derives a checkout key from Session CSRF identity plus cart revision. The store returns the existing owner's order for that key, otherwise commits order, lines and inventory together. [Checkout](../examples/modular-shop/src/orders.rs#L150), lines 150–201; [store](../examples/modular-shop/src/database.rs#L196), lines 196–278 | SQLite `UNIQUE(owner, checkout_key)` plus `BEGIN IMMEDIATE` is the deduplication/business authority. This is persistent business storage, not a process-memory dedup map. Session/cart remain Memory and identity is a local synthetic user. [Schema](../examples/modular-shop/migrations/0001_init.sql#L8), lines 8–23; [Session setup](../examples/modular-shop/src/lib.rs#L246), lines 246–259 | Preserve this scoped application replay. After successful cleanup, the old revision is rejected; after Session loss/rotation, old checkout identity is not automatically recoverable. No general restart/replica or arbitrary client-key/payload replay contract is qualified |

The common [HttpExecutor](../crates/yetlet/src/actions/mod.rs#L123), lines
123–161, performs one fetch and returns its result/error; it contains no
automatic retry. The triage client owns its richer application fetch path
above. Board's [optional status stream](../examples/lab-operations-board/assets/enhance.js#L1),
lines 1–49, reconnects read-only EventSource status; it never submits mutations.
That reconnect behavior is not mutation retry evidence. This is a scoped source
inspection, not an assertion about every client, proxy or downstream application.

The existing Starter
[late-write test](../examples/production-starter/src/store.rs#L592), lines
592–644, explicitly distinguishes never-dispatched work from a started worker
that commits after the caller is canceled. The accepted
[YT-127 evidence](YT-127-ADMISSION-OVERLOAD.md) and
[A1 contract](PRODUCTION-READINESS-PHASE2.md#yt-127--a1-request-admission-and-overload-protection)
remain authoritative at their original scope. Generic 5xx mappings cannot tell
the caller whether SQL started. A 503 or `Retry-After` hint is not blanket
permission to replay a mutation.

## What the existing shop guarantee means

The order row is both business result and deduplication record. The lookup is
inside the same immediate transaction as the effect, with database uniqueness;
concurrent writers against that same SQLite authority serialize rather than
relying on a lookup cache or local mutex. Rollback of order/line failure also
rolls back inventory. Replays return the stored order, not a cached HTTP
response. The handler then conditionally clears only the unchanged user's cart;
a newer cart is preserved. Reads recheck owner through
[load_order](../examples/modular-shop/src/database.rs#L414), lines 414–430.

The key is server-derived from a Session snapshot. Cart mutations advance
revision, and checkout checks it before accessing the database. However,
`place_order` itself does **not** bind or compare a payload digest: a nonempty
different `lines` argument with the same owner/key returns the existing order
before resolving those lines. There is no generic caller-supplied operation-key
API or mismatch response. Its meaning depends on the handler's same-snapshot
discipline. A copied key does not authorize access; every handler attempt still
requires the current user and CSRF, and the database scopes lookup by owner.
The CSRF-derived key remains server-side; do not expose or log it as a public
operation identifier.

The inspected [tests](../examples/modular-shop/src/tests.rs#L233), lines
233–370, cover persistence/read ownership, line-failure rollback, sequential
same-key store replay, concurrent last-stock competition with different keys,
and invalid-CSRF/stale-revision refusal. They do not demonstrate concurrent
same-key HTTP replay, mismatched-payload rejection, crash/restore replay or
distributed Session durability. The SQLite transaction supports the scoped
serialization reasoning; those missing scenarios are not new execution claims.

No expiry, cleanup or tombstone mechanism for checkout keys is selected in this
schema; rows persist with orders. No finite replay window, storage-growth bound
or retention service is qualified. Memory Session loss and cart cleanup limit
key reachability even while the SQL record survives. A different Session/cart
revision expresses a new checkout intent and can place another order with the
same products. Payments/shipping are absent; the mock order transaction says
nothing about external side effects or exactly-once delivery.

## Reconciliation under the selected current decision

Treat a **new submission** as a separately reviewed intent. Treat a **retry** as
another attempt to obtain the outcome of the same intent; changing a version,
token or key to bypass uncertainty can instead cause another business effect.
Preserve draft/acknowledgment information available to the application and use
current authorized read paths. Do not turn a read or a page reload into an
implicit write.

| Evidence available | Permitted conclusion and next step |
| --- | --- |
| Known committed operation/result with a trustworthy acknowledgment and consistent current history | Inspect through current authorized reads; do not recreate the operation merely because a later display or response failed. Later edits or restore may change the visible state |
| Demonstrably not started, such as a proven pre-handler admission rejection or local future never dispatched | That attempt caused no business effect. A deliberate submission still needs current validation/security. An undifferentiated HTTP 503 cannot establish this category |
| Definitive validation, authorization or optimistic conflict rejection on the inspected pre-effect path | That attempt was rejected. Correct input/access or compare the latest version; a conflict on a repeat does not identify the outcome of an earlier attempt |
| Deadline, disconnect, worker uncertainty, invalid success payload or server/process reset after possible dispatch | Outcome remains unknown; do not blindly replay. A started worker may still run. Quiesce/drain relevant writers when needed for authoritative reconciliation, then inspect the correct business history; disconnect does not cancel SQL |
| Early empty list/not-found, matching title/body, or apparently unchanged row | These do not prove nonexecution or uniquely identify an operation. The first write may still be running; [Starter lists](../examples/production-starter/src/store.rs#L249), lines 249–257, are bounded; identical payloads may be legitimate separate issues. Escalate unresolved ambiguity for application/operator decision |

For a known-ID update, compare current version and relevant fields/history, then
make a deliberate fresh edit only after resolving the earlier write's possible
effect. Optimistic concurrency prevents an old version from overwriting a newer
one in the same history; it does not reproduce an original response or prove
that this caller performed the change. For an unacknowledged create without an
operation ID or a unique business identifier, payload matching alone may never
resolve attribution. There is no implemented query-by-operation endpoint.
Manual reconciliation accepts this limitation; it is not a durable deduplication
substitute.

The [YT-135 recovery rules](YT-135-CURRENT-SCHEMA-RECOVERY.md#reusable-operator-procedure)
still apply: stop/drain or establish the prior writer's fate, account for
acknowledged and unknown outcomes, preserve the prior history and approve any
data loss before restore. Restored snapshots can reuse record IDs and versions
in a different history. Refresh clients and reconcile against the selected
history rather than treating an old ID/version or a pre-restore absence as
proof. Telemetry/request correlation is not a durable audit acknowledgment.

## Identity and authority remain separate

| Existing value | Meaning and limit |
| --- | --- |
| Triage pending/submitted item ID | Local response-to-draft association; [Save](../examples/board-triage/src/lib.rs#L75), lines 75–83, carries CSRF and Item, not a durable operation ID |
| Request ID | Per-provider atomic correlation counter starting at 1; [provider](../crates/yetlet/src/request_id/mod.rs#L37), lines 37–74 and 113–134. Not persistent, globally unique, business identity or authority |
| `expected_version` / cart revision | Snapshot concurrency precondition. Cart revision contributes to this particular shop key; a version alone does not identify a durable operation |
| Session identity/CSRF, OIDC state/nonce | Session/security protocol values with their own lifecycle; not a general business replay identity. Do not reuse credentials or protocol nonces as public operation IDs |
| Rate-limit budget/admission permit | Work admission/accounting, not execution history or deduplication |
| Business row/order ID | Identifies a committed resource in a particular history; it may be learned only after acknowledgment and may be reused after restore |

Starter's shared API secret cannot be relabeled as a per-user principal.
Anonymous SSR Session identity is also not authenticated account ownership.
Board has a concrete project/write policy; Shop has owner-scoped orders. A
future consumer must choose its trusted actor, tenant/project and operation
namespace from its actual server policy, without adding fixed Principal fields
to RequestContext or treating a client key as identity or authorization.

Every future attempt **and replay** must apply current authentication and
resource/action authorization plus the route's current CSRF policy before
returning a result. Namespace lookup and mismatch handling must prevent
cross-actor/tenant result, payload or existence leakage. An unavailable security
authority must not be bypassed by a cached result. Preserve
[YT-124 freshness limits](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md) and
[YT-126 adoption limits](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md): a fresh pre-SQL
check is still a snapshot, not atomic revocation-to-commit ordering. Stronger
sensitive-write ordering remains separately gated by YT-138.

## Conditional application contract if replay is later selected

The following describes the decisions needed for a separately authorized
consumer implementation. It creates no current API, storage migration, HTTP
header/status convention, execution gate or production-owner prerequisite.

1. Define the intent and scope: stable opaque operation identity for the same
   intent, trusted actor/tenant and operation namespace, validity across selected
   restarts/replicas, and a new identity for a deliberately new submission.
   Bind key scope to a versioned canonical semantic payload, including relevant
   resource and concurrency precondition. Define bounded key/payload formats and
   reject a mismatched intent without returning another request's private result.
2. Keep effect and operation record under the **same business authority**: one
   transaction/unique constraint must select a concurrent winner, commit the
   effect and durable result/reference together, or roll them back together.
   A lookup-then-write cache, process mutex, Session CAS or separate Valkey record
   followed by SQLite commit cannot establish that atomicity. Shared Session
   storage alone does not make per-replica SQLite one authority; YT-139 owns any
   selected multi-instance placement/recovery qualification.
3. Define pending and terminal behavior. Concurrent same-intent attempts must
   observe one winner, a bounded pending outcome or a safe terminal replay;
   never execute twice because the first has not answered. Specify whether
   definitive pre-effect failures reserve identity. A rolled-back transaction
   can release identity only under the selected rollback proof; crash/commit
   ambiguity needs authoritative recovery. A timeout or TTL does not prove that
   an in-flight effect stopped. Define bounded waiting and overload behavior
   without inventing background retries.
4. Define the replay result: stable business outcome or current authorized
   result/reference, including later deletion/change and sensitive-field rules.
   Generate headers and security artifacts under current policy. Do not blindly
   replay cached `Set-Cookie`, tokens, CSRF values, session/OIDC nonces or private
   bodies. Idempotency is business deduplication, not permission to bypass
   authentication, CSRF, current response security or resource authorization.
5. Select replay/retention horizon, tombstone lifetime, key reuse/expiry rules,
   bounded storage and cleanup ownership. Do not delete in-flight identity on
   TTL and label a late retry new. If results expire before effects cease to
   matter, preserve enough durable rejection/tombstone history or require
   explicit reconciliation; state when replay is no longer supported. No
   retention number is selected for current consumers by this research.
6. Recover business data and the operation ledger as one consistent history.
   Records must survive the restarts/replicas required by the guarantee. Losing
   or restoring an older ledger while retaining effects destroys deduplication
   proof; independently restoring data/ledger is also unsafe. Snapshot forks,
   reused IDs and old client keys require a history/namespace decision and fresh
   client reconciliation. Do not claim safe replay merely because Session or
   Valkey recovered. Define how unknown operations are resolved after cutover.
7. External nontransactional effects need their own authority and failure
   decision, potentially an application outbox, downstream deduplication or
   compensation. None is selected here. A local SQL transaction cannot promise
   exactly-once payment, notification or other external execution.

If an owner separately authorizes implementation, focused acceptance evidence
should exercise the selected consumer's lost acknowledgment/late commit,
concurrent same-intent winner, payload mismatch and cross-actor/tenant refusal,
rollback/crash boundaries, current-auth/CSRF replay denial, bounded pending and
expiry behavior, and consistent restart/restore reconciliation. Include only
the selected topology and external effects. Reuse accepted YT-127/YT-107/shop
evidence for unchanged paths; do not rerun their full historical matrices or
build new validation infrastructure solely because research lists these cases.

## Research evidence and remaining boundaries

Read-only source inspection and ordinary source hashes are recorded outside
the repository at `/tmp/yetlet-yt137-write-outcome-20261023-dmze2tp9` in
`source-findings.json` and `research-summary.json`, alongside the parent's
pre-work `baseline.json`. They are research notes, not an executable validator
or a fresh runtime qualification. No tests, builds, browser sessions, services,
workloads, deployments or cloud actions were started for this research.
Repository documentation and whitespace checks passed: 75 required documents
and 141 backlog items. Independent Reviewer
`/root/yt137_write_outcome_independent_review_20261023` returned PASS, zero issues,
required followups and optional notes; all six reviewed file hashes matched
before and after review. The integrating parent's acceptance changes only
completion status and evidence references; reviewed source and contract
decisions remain unchanged. The retained `review-input.json`, `review.patch`
and `review-final.json` preserve the original review snapshot and result.

Actual production operators, retention periods and SLOs remain deployment
decisions; their absence does not block the bounded framework research decision.
No production safety, HA, instant revocation, exactly-once or universal retry
guarantee follows. YT-138–YT-141 remain Todo and require separate activation;
future performance work remains inactive. YT-096 D1–D6 remain Pending, YT-020
remains Blocked, and immutable M20 evidence at `0ce56e08` is unchanged.
YT-137 is Done only at the independently reviewed research scope. The next
unblockable registered item is conditional YT-138 research, only if separately
activated for a genuine sensitive-write ordering need.
