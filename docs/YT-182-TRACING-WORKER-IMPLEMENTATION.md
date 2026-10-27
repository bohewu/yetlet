# YT-182 — Opt-in tracing/worker integration and focused evidence

Date: 2026-10-25 (Asia/Taipei). Status: **Independent implementation Reviewer PASS,
zero issues/required followups; YT-182 Done at bounded application-reference scope.**

## Authority, prior commit and accepted contract

The maintainer authorizes the 19-path YT-181 strict/builtin commit, then a
contract-first minimal opt-in tracing/safe-worker integration, native middleware
without generic AOP, no passing-proof replay, and separate independent review
before Done. New changes must remain uncommitted/unpushed.

YT-181 is committed as `a10db2b0c0f5ed181f417bc449e6ad6056c5daf3`, parent
`0bf46e7f039062efb7d03317f122f24f5ae80ec2`, tree
`2c2e6ea89a02dfcc5ba3d1bf6e345521312537fb`. Explicit staged strict/builtin Guard
and commit hook each PASS with zero findings; exactly 19 authorized paths and
clean post-commit worktree. No Push. Retained receipts:
`target/yt181-implementation-commit-20261025/`.

The [contract](YT-182-TRACING-WORKER-CONTRACT.md) passed independent Reviewer
`/root/yt182_safe_tracing_contract_review_20261025`, zero issues/required followups;
five document/thirteen evidence/eighteen baseline/source hashes matched. That
acceptance predates implementation and remains a separate historical result.
Board library/Admin host constitute **one** actual consumer; unchanged Starter
and YT-123/128 are retained comparison evidence, not another modified consumer.

## Selected integration

- Board application feature `request-tracing` adds optional normal `tracing`
  and existing `yetlet-request-id` dependencies. Existing constructors keep it
  off. New explicit `open_with_request_tracing` /
  `open_with_oidc_and_request_tracing` constructors select it, including an
  explicit commit-write authorization choice. This application API does not
  add a framework tracing provider, Core field, common trait or generic AOP.
- Admin forwards the feature and optionally selects native JSON
  `tracing-subscriber`. Its existing Settings loader reads
  `request_tracing.enabled`, default false, sole environment override
  `APP_REQUEST_TRACING__ENABLED`. Enabled without feature fails validation.
  Its private startup helpers preserve Demo/OIDC and normal/commit-write choices.
- Board never sets the global subscriber. Enabled Admin initializes once before
  serving, INFO-and-higher for `REQUEST_TRACING_TARGET` and descendants only,
  other targets OFF. Conflict returns a fixed redacted startup error. Output is
  synchronous stderr, without RUST_LOG/EnvFilter, exporter or background queue.
- Board's optional layers sit inside existing transport/security:
  security → generated RequestId → trace → Session/Auth/handler. Existing outer
  Host/Origin refusals and Admin's separate static router remain outside tracing.
  The generated request-ID exhaustion response still traverses outer security.
- HTTP roots explicitly have no parent. Fields are a positive generated local
  ID and finite method; events add only fixed event, integer milliseconds and
  numeric response status. Worker child spans add fixed `database` operation
  and started/completed `ok` or `error`. No URI/query/headers/cookies/identity,
  SQL parameters/body, credentials, arbitrary error or Debug projection.
- Native `Instrument` scopes the async request. The completion guard retains
  its own safe span **and Dispatch**, restoring both for Response-return or
  future-destruction events. No entered guard crosses `.await`.
- An enabled Store captures only the current known reference HTTP root by fixed
  target/name, plus its Dispatch, before existing blocking dispatch. The worker
  restores scoped Dispatch and explicitly parents its synchronous child span.
  Unknown/nested foreign/absent context is not propagated. Direct Store/control
  calls remain unattributed. Existing permit/connection/transaction/drain
  ownership survives HTTP future drop; no Request/Session clone for telemetry.

`http_response_constructed` means Response return, not body delivery.
`http_future_dropped` means destruction, not proven client disconnect or SQL
abort. `sql_worker_completed` means closure return, not a general durable-commit
or audit guarantee. Completion can follow HTTP drop with the original local ID.
Filtered telemetry, panic or process loss can omit terminal events; absence
never establishes rollback/revoke success or permission to retry. IDs are local
to one provider and reset, without global/durable/remote trace identity claims.

## Changed paths and dependencies

The delivery changes 21 paths: two lockfiles; Board manifest, library/database/
request-tracing module and test registration/new test module; Admin manifest,
base Settings, config/main/output module; both application READMEs; Backlog,
Roadmap, docs index, follow-up assessment, contract and this implementation
record. No Core/browser, Starter, Session provider, SQL schema/grant/transaction,
worker/admission limits or prior test source is changed.

Both offline lock resolutions only add registry packages `sharded-slab 0.1.7`,
`thread_local 1.1.10`, `tracing-serde 0.2.0`, `tracing-subscriber 0.3.23` and
`valuable 0.1.1`, plus local dependency metadata. Admin also newly locks existing
`yetlet-request-id 0.1.0`; no existing version is upgraded. `valuable` is lock
closure, not a claimed always-enabled runtime capability. Normal default Cargo
trees for Board and Admin contain neither new `tracing-subscriber` nor
`yetlet-request-id`; native Axum already has transitive tracing, so this is not
a zero-tracing-dependency claim. Test-only capture subscriber is a dev dependency.
All selected code stays server-side; no new WASM feature edge is introduced.

## New focused evidence and retained failures

Artifacts are in `target/yt182-tracing-20261025/evidence/`. Each Cargo command
retains stdout, stderr and native exit in its named receipt. Cargo runs offline;
after the two necessary lock resolutions, all check/test/lint/tree runs use
`--locked`. No historical passing test/provider/benchmark/browser/Windows run
was repeated. Existing source and proof bindings are reused read-only.

| Check | Actual result and scope |
| --- | --- |
| `resolve-check-board` | Initial exit 101: draft constructor-argument replacement also changed a non-constructor `page` bool; both affected page calls restored to `false`. Original output retained. |
| `resolve-check-board-corrected` | Exit 0, Board library with `request-tracing,commit-write-auth`. |
| `resolve-check-admin` | Exit 0, Admin binary with `request-tracing,commit-write-auth,auth-valkey`; native OIDC/Session composition compiles, not live provider qualification. |
| `request-trace-board-tests` | Exit 101: four new cases pass; only new future-drop case fails because its lifecycle event lacks the selected dispatcher during destruction. 26 old cases filtered. |
| `request-trace-drop-corrected` | Exit 0, only that failing case rerun after completion guard retains scoped Dispatch; one passes, 30 cases filtered. The four passing cases are not replayed; subsequent compilation/lint covers the corrected source. |
| `request-trace-admin-enabled` | Exit 0, one new Settings case with `request-tracing,commit-write-auth`; 20 other cases filtered. |
| `request-trace-admin-default` | Exit 0, that new Settings case under the distinct feature-absent condition; 20 other cases filtered. |
| `clippy-board` | Exit 0, changed library/tests, `request-tracing,commit-write-auth`, `-D warnings`; no test execution. |
| `clippy-admin` | Exit 0, binary with `request-tracing,commit-write-auth,auth-valkey`, `-D warnings`. |
| `normal-graph-board`, `normal-graph-admin` | Exit 0; new request-ID/subscriber packages absent from each default normal graph. |
| Final format/docs/whitespace | Required changed-source formatting, `cargo xtask check docs` and tracked/new-file whitespace checks retained separately. |

There are **seven successful new test executions** (five Board cases, one Admin
case in two feature conditions) and one retained failed execution, not an
all-tests-pass claim for the initial suite. The first compile failure and
future-drop failure are product failures repaired at their smallest surfaces;
no harness redesign, replacement run, capability recovery or favorable replay.

New Board cases exercise actual authenticated SSR and JSON triage, untrusted
header/query/body sentinels, an enabled foreign ambient parent, distinct
concurrent request roots, runtime-disabled absence, unknown-context rejection,
and a failing native writer without changed committed result. SSR response
checks retain CSP/no-store/Session header behavior. The multithreaded future-drop
case starts a real blocking SQL closure, aborts its HTTP future, proves drain
still waits while the worker owns its permit, then releases/completes/drains
with correlated worker completion. Its bounded barriers are test-only, not new
runtime hooks. Fixtures use bounded native JSON capture and task-private
TempDir SQLite stores; no live listeners/processes/cloud resources are started.
The isolated worker task is joined/drained before temporary files are removed.

## Limits and acceptance

Ordinary writer errors do not rewrite application outcomes; a slow synchronous
writer can delay requests/workers while they hold resources. The host owns
output access, retention and capacity. No latency/zero-overhead, malicious custom
subscriber safety, durable audit, remote trace, complete edge/stream coverage,
body instrumentation, detached-job identity, retry or HA claim is made.
Existing local-demo terminal login codes are separate from tracing and must
never be retained as telemetry. No new collector, crate or universal default.

Independent implementation Reviewer
`/root/yt182_safe_tracing_implementation_review_20261025` returns PASS, zero
issues/required followups. All 21 implementation, 81 evidence and 15 retained
bindings match before/after review, including HEAD/tree/branch/full status and
staged zero. Native registered-role/model/effort observation matches, central
resolver enforced without degradation/conflict. After acceptance only six
document status/acceptance records change; runtime/test/manifest/lock/README
bytes remain reviewed. Final docs/whitespace verification does not replay
runtime tests. YT-182 is Done at this reference scope; YT-181 remains Done and
YT-183/184 are Todo and unactivated. YT-096 remains
In progress with D1–D6 Pending, YT-020 Blocked. New paths remain unstaged,
uncommitted and unpushed; next candidate is YT-183 only after owner activation.
