# YT-118 — Audit events, ownership and extraction decision

**Date:** 2026-10-22. **Source:** `8c4ca0618b7c4f86a8400a32c0163bc45333d121`.
**Status:** Done at research scope after independent Reviewer PASS, zero issues
and required followups.
**Decision:** Keep business history and security-event policy application-owned.
No `yetlet-audit` crate, public event/sink trait, middleware or runtime is selected.

The owner selected documentation-only research under the
[M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md#yt-118--generic-audit-event-and-sink-feasibility-research-only).
Compare the required Auth Notes and Admin consumers before choosing an API;
additional persistence/observation examples test whether their apparent common
vocabulary actually justifies extraction. Research completion does not deliver
a security-audit backend or expand the original nine release candidates.

## Observed consumers and actual repetition

| Inspected consumer | Implemented behavior | Reuse and evidence boundary |
|---|---|---|
| [Auth Notes](../examples/auth-notes/src/lib.rs), `login`, `logout`, `NotesStore`, note mutation handlers | Checks synthetic credentials, server Session/ownership and CSRF; stores current notes in a bounded process-local map. Login rotates authenticated state; logout rotates to anonymous state. Errors become HTTP responses | No history/event append, durable sink, denial log or request-correlation field in these paths. Existing live-state limits are not audit retention. This is an observed authentication consumer, not a second audit implementation |
| [Admin composition](../examples/admin-dashboard/src/main.rs), `application`; Board [native save](../examples/lab-operations-board/src/lib.rs), [triage save](../examples/lab-operations-board/src/triage.rs) and [database](../examples/lab-operations-board/src/database.rs), `Database::save` | Admin reuses Board. Both save entry points perform current identity/project/write checks and call the same transactional business-history path | Admin/Board and native/triage are one implementation lineage, not independently duplicated audit sinks. They are the second required application comparison with Auth Notes |
| Board [history schema](../examples/lab-operations-board/migrations/0001.sql) and `Database::save` | Appends item/project/version, local actor name, fixed create/update action, title/status snapshot and SQLite insertion time in the same Immediate transaction as the write. Persisted rows imply successful commit; no explicit outcome/correlation column | History insertion/capacity failure prevents business commit. Global history cap is 100,000 rows; latest-100 display is a read limit, not pruning. No automatic retention or tamper protection is established. Title is app-owned business content, not a safe generic telemetry field |
| Board [auth](../examples/lab-operations-board/src/auth.rs), [OIDC](../examples/lab-operations-board/src/oidc.rs), login/logout/security handlers | Maps trusted identity to a server-local account. Inspected login/logout, denied policy/resource/CSRF, validation/conflict and storage-error paths do not append history. OIDC stderr reports bounded HTTP status/failure stages | No complete security-event trail. Token-response HTTP status precedes token verification and cannot mean authenticated success. History actor names do not retain external issuer/subject, login method or policy revision |
| [SQLite CRUD](../examples/sqlite-crud/src/lib.rs), `Database::{create,update,delete}`, `insert_audit`; [schema](../examples/sqlite-crud/migrations/0001_init.sql) | `note_audit` stores note ID, fixed action, version, title and Unix-seconds time within each successful mutation transaction; insertion failure rolls back the business change | Independent business-history example, with no actor, explicit outcome or request correlation. Domain SQL and title snapshots differ from Board. No replaceable delivery/retention implementation is shared or duplicated |
| [Production Starter observer](../examples/production-starter/src/web.rs), `observe`; [subscriber setup](../examples/production-starter/src/main.rs) | Explicit app-owned JSON tracing records request ID, allowlisted method, HTTP status and duration; excludes URI/query/headers/body/actor. Subscriber level is configured by the app | Diagnostic response observation, with no durable domain-event acknowledgment, transaction linkage or protected audit retention. It is not another business-audit sink |

Board SSE sends current snapshots; its history-derived revision is neither an
audit-event replay nor request correlation. Modular Shop's persisted orders
similarly do not demonstrate a separate reusable audit-delivery mechanism.
Existing rollback/history tests were inspected as source, not rerun.

The common concept is recording an operation, but the actual shared work is
small domain-specific SQL inside each transaction. Auth Notes has no audit
producer, and diagnostic HTTP logging has different semantics. These examples
inform a contract; they do not demonstrate repeated generic event validation,
redaction, sink dispatch or delivery that a new trait would remove.

## Responsibility and trust boundaries

| Owner | Responsibility |
|---|---|
| Application handler/domain | Select which events matter, trusted local actor mapping, resource/action vocabulary, permission decision, safe data projection and whether the event describes attempt, decision or committed effect |
| Auth/provider and Session | Establish verified identity and continuity with the existing freshness/revocation rules. Neither successful extraction nor a Session callback establishes business authorization, commit or audit delivery |
| Application transaction/store | Keep required business history atomic with the mutation; own SQL/schema, constraints, migrations, idempotency and any later outbox. PostgreSQL schema is a possible application concern, not an implemented backend or Yetlet requirement |
| Application/operator sink | Select explicit IO/deadline/capacity policy, access control, delivery acknowledgment, retries/deduplication, retention/deletion, backup and tamper protection |
| Native framework composition | Supply existing Axum/Tower scopes, ordinary functions, typed State/Extensions and opt-in correlation. Add no fixed Audit/User/Session field, automatic event capture, DI container or ambient sink |

The [YT-063 threat model](THREAT-MODEL.md#13-logging-audit-metrics-and-privacy)
is authoritative for protected audit evidence and redaction. Request IDs are
correlation, not identity. The [existing provider](../crates/yetlet-request-id/src/lib.rs)
is process/provider-local; replicas/restarts require an explicitly selected
application namespace or other trusted correlation strategy. Do not reuse
Session IDs, CSRF tokens, OIDC state/nonce or arbitrary inbound headers as audit
identity/correlation. A local history version is a business revision, not a
request ID or globally ordered event identifier.

The [YT-126 pre-SQL limit](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md#existing-write-consistency-limit)
still applies: final authorization precedes queued/running SQL; no atomic
transaction spans the policy file, Valkey and business database. History does
not prove that a concurrent revocation prevented a later commit. This study
neither changes that race nor adds cross-store fencing.

## Minimum event semantics if a later consumer selects emission

This is an application projection checklist, not a Rust struct, serialization
schema or selected public API:

| Fact | Required interpretation |
|---|---|
| Actor | Trusted stable local reference in its namespace, or explicit anonymous/system classification. An attempted username, browser field or unverified claim is not a verified actor. Include provider/policy provenance only when justified and safely mapped |
| Action and target | App-controlled finite operation/category and minimal resource identifier/type; validate untrusted locators and permissions separately. Business title/body snapshots require a separate storage/content decision |
| Outcome and phase | Distinguish rejected/failed/committed/unknown and attempt/decision/effect. Use bounded reason categories; do not label a handler start, token HTTP 200 or successful sink enqueue as business success |
| Time and correlation | App-selected timestamp/source and trusted bounded request/event linkage where available. Wall-clock timestamps alone do not establish cross-instance ordering, deduplication or exactly-once processing |
| Optional policy/reason | Safely selected policy revision or bounded operator reason where required by the application. Do not invent a policy revision the current authority does not expose |

Allowlist field names, types and bounded lengths before emission. Omit or safely
classify a malformed optional field without discarding the entire denial event;
encode for the chosen output format to prevent delimiter/control-character
injection. Do not dump `RequestContext`, State, Extensions, Principal Debug,
error chains, raw URLs/query strings, headers, bodies or serialized Session.
Never capture passwords, SecretProvider values, cookies, CSRF, bearer tokens,
OIDC codes/state/nonce/verifiers or token responses. Minimize actor/target data;
IP address, user agent and arbitrary business text are not default event fields.

Existing Board/SQLite title snapshots remain their deliberate domain history;
this research neither silently removes that content nor promotes it into a
generic redacted envelope. New projection would need explicit application
selection and content/retention review.

## Failure, timing and delivery

Retain current transactional history: an external best-effort call cannot replace
the same-transaction insert while preserving atomicity. A future asynchronous
outbox or durable sink requires a separately selected consumer and failure
contract; no queue, PostgreSQL table or worker is implemented here.

For optional diagnostics, the app may select bounded loss with observable drop
counts; for required audit, it must define bounded admission/durable failure
behavior before the protected mutation. No sink outage may hang a request
indefinitely or weaken authentication/authorization. Cap event size, queue,
wait, retries and shutdown drain; classify capacity, rejected and unknown
delivery distinctly. A successful in-memory enqueue is not durable storage.

Cancellation, timeout or a lost response may occur after a business commit.
An independent sink failure then cannot roll it back; returning a generic error
does not justify replaying a non-idempotent mutation. Persisted row IDs and
timestamps are not an exactly-once or total-order guarantee. Cross-store
atomicity/recovery must remain an explicit application decision.

In particular, [Redis Session](../crates/yetlet-session-redis/src/lib.rs)
`try_update` does not roll back external callback effects, and middleware
finalization can replace a handler's response with 503. HTTP status therefore
does not establish whether a business operation committed. An audit failure
after logout/revocation must not restore privileged Session state or imply that
the revocation did not occur. Native scope ordering can observe HTTP responses;
only the domain transaction can authoritatively describe its committed effect.

## Composition options and extraction threshold

| Option | Decision from observed consumers |
|---|---|
| App-owned transaction/function and explicit typed State | Selected direction. Preserve domain history in its transaction; use ordinary app functions for any later explicit event projection. There is no current need for a framework sink registry or new trait |
| Existing Rust diagnostic tracing/subscriber | Suitable existing composition for diagnostic events when explicitly selected by an app; not a replacement for required transactional audit. Filtering/writers do not by themselves prove retention, durable acknowledgment or security-event completeness |
| New `yetlet-audit` event/sink abstraction | Deferred. A candidate name alone and two domain history tables do not establish a common replaceable sink contract |

Reconsider extraction only when independent real applications demonstrate the
same useful bounded event-construction/redaction/delivery glue and a concrete
replaceable-sink need. Identify actual duplicated functions and remove that work
without absorbing domain actions, schemas or transaction ownership. Select
delivery/failure/cancellation/overflow and acknowledgment semantics first;
preserve mandatory business-history atomicity and default/browser isolation.
Then separately authorize one minimal implementation, real consumer adoption,
negative/lifecycle verification and an independent review. Auth Notes need not
gain a manufactured sink just to satisfy a candidate crate's consumer count.

The [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
supports purpose-specific event selection, safe field handling and protected
collection. The [tracing documentation](https://docs.rs/tracing/latest/tracing/)
(0.1.44 when consulted) describes structured diagnostic events/spans;
[tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)
(0.3.23) supplies subscriber/layer/filter composition. Checked 2026-10-22.
The ownership/extraction decision above is this study's inference from those
mechanisms and repository consumers, not a library-mandated audit protocol.
No dependency, version pin, exporter or external infrastructure is selected.

## Evidence and completion gate

Source comparison and primary documentation research only. No runtime, schema,
dependency, feature, crate or Core API changes; no tests, browser/CSP, Windows,
Valkey/IdP, database workload or prior security scan reruns. Private settings and
credentials were not read. No services/background resources were started.
Existing YT-063/YT-094/Auth/Session/Valkey verification retains its original
source, acceptance, failure history and production limits.

Independent Reviewer `/root/yt118_audit_study_independent_review_20261022` PASSed
with zero issues and required followups. It inspected the actual eight-document
patch and critical consumer/provider source; reviewed hashes, HEAD and index were
unchanged during review. Only acceptance/status/evidence bookkeeping follows
that result. `cargo xtask check docs` PASSed (75 required documents, 126 backlog
items), as did changed/new-file links, anchors, English, fences, EOF and whitespace
checks. No runtime validation was performed or transferred from earlier work.

The repo-external `yetlet-yt118-audit-study-20261022-cxv2o8y0` directory retains
two source-inspection memos, exact documentation-check commands/outputs/exits,
`review-source.json`, `independent-review.json` and final `closure-checks.json`.
All eight changed paths are documentation: this study, Backlog, Roadmap, M21 plan,
documentation index, capability map, Extensions and ADR 0005. No other source,
manifest, lock, test or runtime input changes.

YT-118 is Done at research scope. The next independent M21 study is YT-121; no
implementation or automatic continuation is selected. YT-096 stays In progress
and YT-020 stays Blocked. HEAD remains `8c4ca06`, staged files zero, no commit
or push.
