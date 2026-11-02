# YT-231 — Lab SQLite concurrency-test stability

Date: 2026-11-02. Status: Done at local test-correction scope.

## Scope and preserved authority

The owner authorizes cause analysis and the smallest correction to the Lab's
flaky concurrent-write test. Preserve Busy/conflict and security semantics,
reuse accepted evidence, retain failures and require independent Reviewer PASS
before Done. No Commit/Push, hosted retry, deployment or release is authorized.
The implementation changes only native test assertions/coverage and documentation.

## Observed failure and confirmed contract mismatch

PR52's required CI41368715313/attempt1 passes. Its identical merged tree at
master2a4c01c fails in41369277335/attempt1, Stable job125036053021:
independent_boards_share_sqlite_records_conflicts_busy_errors_and_restart expects
303/409 but receives303/503. The explicitly authorized Stable-only second attempt
passes; the first log, run/job metadata and unchanged runtime-source identities
remain in the repository-external YT-230 evidence run. Earlier CLI0.1.1 recorded
the same status pattern; none of these historical results is overwritten.

The old CI assertion records statuses, not the failed response's typed error,
body or lock timing. It cannot prove the exact origin of that historical503 or
whether CPU scheduling, filesystem latency or another delay caused it. The
confirmed defect is an assertion stronger than the existing application contract:

- Database connections have a250ms SQLite busy budget. Each save uses
  BEGIN IMMEDIATE; only after acquiring the transaction does it compare versions
  and atomically modify the item/history. SQLite Busy/Locked maps to Failure::Busy
  and503; a mismatched item version maps to409. Four workers, one-second worker
  admission, FULL synchronization and no application retry are unchanged.
- [SQLite transaction control](https://www.sqlite.org/lang_transaction.html)
  permits only one simultaneous writer; BEGIN IMMEDIATE can fail with SQLITE_BUSY
  while another write transaction is active.
- [SQLite busy_timeout](https://www.sqlite.org/c3ref/busy_timeout.html) bounds the
  handler's waiting; expiry returns SQLITE_BUSY instead of guaranteeing eventual
  write-lock acquisition. A concurrent request therefore need not reach the
  version comparison. A faster observed303/409 run is not a timing guarantee.

Source authority is the [existing database implementation](../examples/lab-operations-board/src/database.rs),
[failure/HTTP mapping](../examples/lab-operations-board/src/lib.rs),
[documented application bounds](../examples/lab-operations-board/README.md) and
[YT-181's preserved limits](YT-181-SENSITIVE-WRITE-CONTRACT.md).
The [YT-094 shared-file record](YT-094-SHARED-SQLITE.md) retains its actual
303/409 observation and separate held-writer503 check; it is not rewritten as a
promise that every concurrent edit completes within the busy budget.

## Minimal correction and deterministic diagnosis

One new test holds a real external IMMEDIATE write transaction over the existing
private SQLite fixture. The authenticated HTTP save must return503 with the
existing exact safe Busy message, while item/version/history remain unchanged.
After rollback, an independent application Database commits a change; a new
explicit submission of the old version must return409 with an escaped draft and
no additional effect. No sleep, larger timeout, runtime hook or automatic retry
is introduced. This controlled case establishes the reachable SQL Busy path;
it does not retroactively add missing telemetry to the historical CI response.

The original two-Board race remains concurrent and uses independent connections.
It requires exactly one303. The loser may be409 with its unsaved draft, or503
only with the exact safe Failure::Busy message. An unrelated503, other
status or second success fails the test. Item version advances once, history
contains one matching winning effect, and the persisted title identifies the
winning request. Once both requests finish, a new old-version submission must
strictly return409 and leave that state unchanged. The original held-writer,
recovery and restart assertions remain.

HTTP assertions use the actual status and existing Busy message; they change no
middleware or application response. A first test-only probe incorrectly assumed
that the final response retained the internal Failure extension: it observed
None rather than Busy; separate controlled checks verify the503/Busy body.
Preserve that failed probe
and remove the unsupported test-only observation. No runtime metadata change is
needed. Client checks retain CSP/referrer headers; the controlled stale draft
checks escaping. Logs do not dump bodies, Session IDs, Cookies or CSRF values.

## Bounded verification

Only the new controlled case and changed race run; old full application,
Auth/Valkey, browser, Windows, deployment and benchmark matrices are reused at
their original scope. Each test drains its databases and owns a disposable
temporary directory. No listener, container or cloud resource is started.

- Initial controlled-lock proof, before the old race assertion changes:
  Rust1.99 offline/locked exact-name test PASS,1 passed/21 filtered,0.27s.
  This establishes the HTTP Busy/no-effect then post-commit conflict sequence.
- Corrected original race: Rust1.99 offline/locked exact-name test PASS,
  1 passed/21 filtered,0.28s; full workspace format check PASS.
- Final HTTP-Busy controlled case after removing the unsupported extension
  probe: PASS,1 passed/21 filtered,0.26s. This is the same real-lock scenario;
  its original failed probe remains retained.
- Final original race after the probe removal: PASS,1 passed/21 filtered,0.28s.
  The controlled case's code is unchanged by the later race-comment correction.
- Focused package tests Clippy with -D warnings and final workspace format PASS;
  documentation check PASS,75 required documents/231 backlog items. Whitespace
  PASS. Independent Reviewer PASS, zero issues/required followups; no
  full-suite/stress-repeat claim.

Executed focused commands use Rust1.99 and the locked, already prepared cache:

```sh
cargo +1.99.0 test --offline --locked -p yetlet-lab-operations-board --lib tests::sqlite_writer_contention_is_busy_then_stale_version_conflicts_after_commit -- --exact
cargo +1.99.0 test --offline --locked -p yetlet-lab-operations-board --lib tests::independent_boards_share_sqlite_records_conflicts_busy_errors_and_restart -- --exact
cargo +1.99.0 clippy --offline --locked -p yetlet-lab-operations-board --tests -- -D warnings
cargo +1.99.0 fmt --all -- --check
cargo xtask check docs
```

Raw logs, original-failure references and source hashes are retained outside the
repository under yt231-lab-sqlite-concurrency-20261102. Independent read-only
Reviewer PASS, zero issues/required followups, verifies all five frozen source
hashes, unchanged runtime blobs and the historical failure log digest. It accepts
Done at this local scope, followed only by status updates and their
documentation/whitespace checks. These native tests do not qualify SQLite
capacity, durability, HA or a new public/platform release. No Commit/Push or
hosted action occurs.
