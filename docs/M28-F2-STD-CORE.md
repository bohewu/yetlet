# M28-F2 / YT-282-B — Public std-only Composition Core

- **Date:** 2026-11-19.
- **Decision:** [ADR0013 scoped acceptance](adr/0013-composable-application-and-hosts.md#accepted-scoped-f2-decision--2026-11-19), explicitly authorized by the maintainer.
- **Status:** Bounded public std-only core complete: current-source focused tests PASS and independent R2 actual PASS. Full YT-282 and F2 remain open.
- **Boundary:** Additive, default-off `composition` feature in the existing `yetlet` package. No runtime/native dependency, adapter, SDK/CLI version change or publication.

## Public entry points and ownership

The actual [module documentation](../crates/yetlet/src/composition/mod.rs) and
[compiling local example](../crates/yetlet/examples/composition_local.rs) define
this slice. YT-281's older signatures remain illustrative design evidence.

`Builder<E, Parts>` holds concrete typed attachments. `build()` seals them into
`Composition<E, Parts>`; an active/stopping composition cannot attach more
resources or invoke another startup factory. Fallible construction returns a
`StartFailure` retaining the exact previous root and original workers for
explicit completion. Factories own cleanup of partial resources not returned
to the builder.

| Part | Readiness | Mutation | Completion ownership |
| --- | --- | --- | --- |
| Required local | Ready when attached | Restricted active-only value access | Caller-supplied close callback |
| Required foreground | Explicit actual-ready report after build | Restricted active-only value access | Caller retains the loop and supplies close |
| Borrowed resource | Ready while active | Active-only access through the existing mutable borrow | Caller retains ownership; no runtime shutdown or resource close |
| Owned worker | Explicit actual-ready report after build | Storage mutable only inside lifecycle callbacks | Separate admission fence, stop signal and observation of the original worker |

Typed `previous()/last()` views navigate the static structure; no type-erased
registry, public lifecycle trait, unsafe downcast or blanket Send/Sync is
required. Private recursion traits cannot be implemented or imported as Host
extension points. Public views do not expose mutable lifecycle slots, so a
caller cannot replace a worker while retaining stale completion flags.
Interior-mutability policy and truthful callback implementation remain the
application's responsibility.

## Stop, failure and completion

1. Attempt **every** admission fence. Any failure prevents all local stops and
   worker signals for that pass, retaining the authoritative domain.
2. Close locals in reverse attachment order. If a close fails, retain that
   resource and earlier local dependency owners. Worker shutdown signals are
   attempted independently after admission closes, even while local close needs
   a retry. Successful operations are latched and never repeated.
3. After every admission fence confirms closure, observe the original workers
   with the remaining overall deadline budget even if local cleanup or signal
   delivery failed. A worker that already exited still needs its actual Join;
   a live unsignalled worker remains Pending and retries its signal. Joined
   workers need no later signal. An observation error remains **Pending**;
   only `WorkerCompletion::Joined(Result)` is terminal. Aggregate completion
   requires both successful local closure and actual Join of every worker.
4. Preserve the first failure through retries; retain later errors separately.
   `Completion::Complete` means cleanup finished, including failed Joins.
   Inspect aggregate failures and each worker's `JoinStatus` for success.

Required failure requests stop immediately. No queue, executor, thread or loop
is allocated by the core. A full data queue must not be the stop mechanism;
the caller supplies the separate signal callback.

## Explicit limits

Pending retains the same storage **while the caller retains the composition**.
Drop performs normal field destruction only: no blocking shutdown, hidden Join
or automatic error suppression. Dropping a pending JoinHandle can detach the
worker; keep the root, retry completion and then drop it. The same obligation
applies to failed-start roots and already populated builders.

Callbacks must be truthful, preserve original workers on Pending/errors and
release actual resources when reporting closure. Panics are not caught.
Deadlines provide a cooperative observation budget and cannot preempt user
callbacks or guarantee a process-exit bound. Completed roots retain value
containers until drop; borrowed resources remain caller-owned.

No real Web drain, UDP socket, Native window or GPU adapter is delivered by
this slice. Private F1 actual integration and private-A proof are reused only
at their recorded original source identities. External three-host/custom-Host
application qualification remains YT-283; package consumption/compatibility
remains YT-284; physical D4 is **Not verified**.

## Focused qualification

The public integration test target passes **13/13** tests after the R1 repair. Real standard-library
threads exercise original JoinHandle identity across Pending, separate shutdown
control with a saturated data queue, startup rollback, signal/observation retry,
worker-result failure and an actual panic/failed Join. Local Rc, borrowed lifetime,
foreground readiness and admission/local-close retry tests cover caller-thread
ownership. Synthetic gate fixtures are identified separately from real workers.

The module's **3/3** focused doctests pass, including compile-fail checks for
late attachment and mutable worker-slot access. The public caller-local example
prints revisions [1, 2, 3], a successful domain stop and retained caller audit.
Clippy with -D warnings and package Rustfmt pass. The first compile failure
(required explicit borrowed-value lifetime) and its exact correction are retained.
The intentional worker panic in the passing suite is a negative-path fixture,
not an unjoined worker or a failing test.

Normal/build dependency graphs for default, no-default and server-axum exactly
match the pre-edit baseline; adding composition contributes zero dependencies
to each. Existing features/defaults/dependency declarations and Cargo.lock are
unchanged apart from the additive empty composition feature. Bare/default core
checks and composition + server-axum compilation pass.

Global cargo xtask check docs remains **FAIL19**, with the same retained ignored
CJK evidence and no new official-document diagnostic. An additional full-crate
rustdoc -D warnings check fails on existing unclosed HTML tags generated by
view/tags.rs; that file is byte-identical to HEAD and there is no composition
source diagnostic. Preserve this failed command rather than claiming a global
rustdoc PASS or expanding this slice into unrelated documentation repairs.

Independent R1 wc_job_OLxZ-w1BynmPnv5A completed with actual FAIL1/P1: failed
signal delivery prevented observing a worker that had already exited. The
repair separates admission closure and local cleanup from Join observation.
A finished worker is now joined even after signal failure; a live unsignalled
worker stays Pending and retries its signal. Both cases use actual std threads
in focused regressions, preserve the original handle/failure, and prevent repeated
signal/Join after confirmed completion. R1 evidence is retained unchanged.
Independent R2 `wc_job_mqMeroZMeAIqAvMx` completed with exit0 and actual JSON
PASS, `issues=[]`, `required_followups=[]`. Observed saved-role model was
gpt-6.2-astra with effective high effort and unchanged operator Runner settings.
This was a role-directed Codex Job, not native agent_type dispatch or formal
assurance. The frozen reviewed implementation was unchanged through completion;
only status/receipt metadata was updated afterward. No unrelated old F1
GUI/HTTP/UDP tests were rerun.
