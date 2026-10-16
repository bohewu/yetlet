# YT-085 — Interrupted Reviewer and Native Exit Receipt Repair

**State:** repair validated, independent review continuation pending. YT-085
is not Done and YT-086 has not started.

The independent read-only Reviewer Job `wc_job_ddy0Xost8yg_9Mv4` was interrupted
when the Runner process was replaced. WebCodex recorded `lost_after_reconcile`
and `command_execution_state=outcome_unknown`, not a native exit or verdict.
The parent inspected recorded PID 1525566 and exact thread
`01a13e7c-7254-7c41-aa7a-94a83c879b11`; the process and all matching related
executions were absent. No final `result.json` or terminal telemetry existed.
The observed role model was `gpt-6-astra`, effective `high`; review inputs were
unchanged. The interruption is preserved in
`target/m12/yt085/reviewer/interruption-recovery.json`. No replacement started
while the original execution could still be active. No final PASS/FAIL is
inferred from the incomplete event stream.

## Concrete observation and minimal correction

The incomplete review independently verified the 26 indexed receipt hashes,
current source, Board's three 1,250-request diagnostics, and real Firefox pixel
and cleanup evidence. It also identified two reachable status-contract gaps:

1. `Server::start_binary` used a readiness helper that reaped the child but
   discarded its native status, returning a generic failure without an exit
   receipt for a child that had already exited.
2. `Server::stop` checked graceful-success before writing its exit receipt,
   so a nonzero graceful result returned without preserving the actual status.

The parent changed only `xtask/src/smoke/mod.rs` and a direct negative test:
`Server::wait_ready` now shares the real startup path and records teardown
before propagating its primary readiness error. An already-exited child keeps
its portable native exit/signal mapping; an early exit 0 is still a readiness
failure, with its actual native zero preserved in evidence. `finish_stop` writes
observed native code, signal, listener/stream/reap state and cleanup error before
checking a graceful failure. Missing native status remains unknown/null, never
invented as zero. Cleanup or receipt-write failure is still nonzero. The original
`Service::readiness` identity-preserving behavior and platform process ownership
were not changed.

## Focused validation without replaying successful browser journeys

`target/m12/yt085/parent-server-status-fix/` retains native build, strict Clippy,
focused test logs and two native application regressions. The actual tests cover
exit 23 before readiness, exit 17 on failed graceful shutdown, early zero, and
Unix signal-to-exit mapping, including listener closure/reaping and readable
receipts. The four original process foundation/cleanup test groups passed
21 test invocations (including three fixture entry functions), with one
pre-existing subprocess fixture ignored by the normal harness. There was no
new ignored acceptance case. All original cleanup/error-identity tests remain.

The actual hello-world and SQLite restart/mutation smoke commands passed at
this changed status-recording source. Strict Clippy and `git diff --check` passed.
No GUI/browser scenario changed or was replayed for these exit-report repairs;
previous source-bound browser evidence remains qualified by its exact input
hashes plus this focused lifecycle regression, not relabeled as a fresh run.

The migration ledger's stale per-row Not-run text was corrected to reflect
actual runtime results and separately unexecuted Redis fixture boundaries.
Windows persistent-service execution is still unavailable due to the recorded
artifact-transfer policy rejection; no alternative transfer bypass is allowed.
A distinct resumed Reviewer execution must inspect the repairs and complete the
remaining original audit before YT-085 acceptance.
## Subsequent independent review and closeout (2026-10-15)

The Server receipt repair was accepted, but full-scope Reviewer subsequently found Browser early-exit/native-status loss and generic infrastructure misclassification. `target/m12/yt085/parent-browser-status-fix/` repairs Browser; a later independent review found forms/scene/Board cleanup error-code propagation, fixed with focused tests at `target/m12/yt085/parent-suite-cleanup-classification-fix/`. A third independent review found Board's private teardown deleted `exit.json` receipts; `target/m12/yt085/parent-board-status-retention-fix/` now captures only typed, allowlisted native exit/signal and cleanup fields across restarted/failed-startup processes before deleting private logs/credentials/database. Final read-only independent Reviewer PASS: `target/m12/yt085/reviewer-board-retention-20261015/result.json`, no issues/followups. These changes retain existing source-bound runtime receipts without unconditionally rerunning accepted browser journeys. Windows persistent-Service and external Redis integration are still not verified.
