//! YT-282-A: test-only, std-only source proof of the reviewed YT-281 typed root.
//!
//! This is NOT yetlet::composition and does not add a public Yetlet feature.
//! The reviewed candidate is in docs/M28-F2-PUBLIC-COMPOSITION-CONTRACT.md, but
//! ADR0013 remains Proposed; public promotion requires a separate decision.
//!
//! Ordinary Rust generic values form a static chain. The internal PartActions
//! and Chain traits are implementation machinery, NOT a generic Host trait,
//! dynamic registry, scheduler or boxed polymorphic object.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Prepared,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Complete,
    Pending,
}

#[derive(Debug, Default)]
pub struct Failures<E> {
    primary: Option<E>,
    cleanup: Vec<E>,
}

impl<E> Failures<E> {
    fn record(&mut self, error: E) {
        if self.primary.is_none() {
            self.primary = Some(error);
        } else {
            self.cleanup.push(error);
        }
    }

    pub fn primary(&self) -> Option<&E> {
        self.primary.as_ref()
    }

    pub fn cleanup(&self) -> &[E] {
        &self.cleanup
    }
}

/// Statically typed storage, not a runtime Host registry.
pub struct Empty;

pub struct Node<P, T> {
    pub previous: P,
    pub part: T,
}

/// Internal only: specialized lifecycle operations for *parts*, not Hosts.
#[doc(hidden)]
pub trait PartActions<E> {
    fn is_ready(&self) -> bool;
    fn close_admission(&mut self, _failures: &mut Failures<E>) -> bool {
        true
    }
    fn close_local(&mut self, _failures: &mut Failures<E>) -> bool {
        true
    }
    fn signal_worker(&mut self, _failures: &mut Failures<E>) -> bool {
        true
    }
    fn observe(&mut self, _remaining: Duration, _failures: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
}

/// Internal static-chain recursion. It neither spawns nor schedules Hosts.
#[doc(hidden)]
pub trait Chain<E> {
    fn all_ready(&self) -> bool;
    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool;
    fn close_locals_reverse(&mut self, failures: &mut Failures<E>) -> bool;
    fn signal_workers(&mut self, failures: &mut Failures<E>) -> bool;
    fn observe_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion;
}

impl<E> Chain<E> for Empty {
    fn all_ready(&self) -> bool {
        true
    }
    fn close_admission(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn close_locals_reverse(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn signal_workers(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn observe_owned(&mut self, _: Instant, _: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
}

impl<E, P: Chain<E>, T: PartActions<E>> Chain<E> for Node<P, T> {
    fn all_ready(&self) -> bool {
        self.previous.all_ready() && self.part.is_ready()
    }

    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool {
        // Attempt all admission closures without short-circuiting; any failed
        // gate prevents local/domain release until a later retry succeeds.
        let this_closed = self.part.close_admission(failures);
        let previous_closed = self.previous.close_admission(failures);
        this_closed && previous_closed
    }

    fn close_locals_reverse(&mut self, failures: &mut Failures<E>) -> bool {
        // Attempt every owned cleanup; retain failed resource owners for a
        // follow-up stop rather than report complete on a fallible callback.
        let this_closed = self.part.close_local(failures);
        let previous_closed = self.previous.close_locals_reverse(failures);
        this_closed && previous_closed
    }

    fn signal_workers(&mut self, failures: &mut Failures<E>) -> bool {
        let this_signalled = self.part.signal_worker(failures);
        let previous_signalled = self.previous.signal_workers(failures);
        this_signalled && previous_signalled
    }

    fn observe_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion {
        let mine = self
            .part
            .observe(deadline.saturating_duration_since(Instant::now()), failures);
        let earlier = self.previous.observe_owned(deadline, failures);
        if mine == Completion::Pending || earlier == Completion::Pending {
            Completion::Pending
        } else {
            Completion::Complete
        }
    }
}

pub struct Owned<T, Close> {
    pub value: T,
    close: Close,
    ready: bool,
    stopped: bool,
    label: &'static str,
}

impl<T, Close> Owned<T, Close> {
    pub fn label(&self) -> &'static str {
        self.label
    }
    pub fn mark_ready(&mut self) {
        if !self.stopped {
            self.ready = true;
        }
    }
}

impl<E, T, Close> PartActions<E> for Owned<T, Close>
where
    Close: FnMut(&mut T) -> Result<(), E>,
{
    fn is_ready(&self) -> bool {
        self.ready
    }

    fn close_local(&mut self, failures: &mut Failures<E>) -> bool {
        if self.stopped {
            return true;
        }
        match (self.close)(&mut self.value) {
            Ok(()) => {
                self.stopped = true;
                self.ready = false;
                true
            }
            Err(error) => {
                // A failed close cannot justify releasing this owner's slot.
                failures.record(error);
                false
            }
        }
    }
}

pub struct Borrowed<'a, T> {
    pub value: &'a mut T,
    label: &'static str,
    stopped: bool,
}

impl<T> Borrowed<'_, T> {
    pub fn label(&self) -> &'static str {
        self.label
    }
}

impl<E, T> PartActions<E> for Borrowed<'_, T> {
    fn is_ready(&self) -> bool {
        !self.stopped
    }
    fn close_local(&mut self, _: &mut Failures<E>) -> bool {
        // No Drop or cancellation of caller-owned resources.
        self.stopped = true;
        true
    }
}

/// An owned task carries real *separate* admission, shutdown and completion.
/// This one-file proof does not start a task or runtime by itself.
pub struct Worker<T, CloseAdmission, SignalStop, Observe> {
    pub value: T,
    close_admission: CloseAdmission,
    signal_stop: SignalStop,
    observe: Observe,
    admission_closed: bool,
    stop_signalled: bool,
    finished: bool,
    label: &'static str,
}

impl<T, A, S, O> Worker<T, A, S, O> {
    pub fn label(&self) -> &'static str {
        self.label
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

impl<E, T, A, S, O> PartActions<E> for Worker<T, A, S, O>
where
    A: FnMut(&mut T) -> Result<(), E>,
    S: FnMut(&mut T) -> Result<(), E>,
    O: FnMut(&mut T, Duration) -> Result<Completion, E>,
{
    fn is_ready(&self) -> bool {
        true
    }

    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool {
        if self.admission_closed {
            return true;
        }
        match (self.close_admission)(&mut self.value) {
            Ok(()) => {
                self.admission_closed = true;
                true
            }
            Err(error) => {
                // A fallible gate is NOT proof that new commands were fenced.
                // The owner must remain alive and the next stop may retry.
                failures.record(error);
                false
            }
        }
    }

    fn signal_worker(&mut self, failures: &mut Failures<E>) -> bool {
        if self.stop_signalled {
            return true;
        }
        match (self.signal_stop)(&mut self.value) {
            Ok(()) => {
                self.stop_signalled = true;
                true
            }
            Err(error) => {
                // The HTTP worker still owns its original task; retry the
                // independent shutdown signal rather than report completion.
                failures.record(error);
                false
            }
        }
    }

    fn observe(&mut self, remaining: Duration, failures: &mut Failures<E>) -> Completion {
        if self.finished {
            return Completion::Complete;
        }
        match (self.observe)(&mut self.value, remaining) {
            Ok(Completion::Complete) => {
                self.finished = true;
                Completion::Complete
            }
            Ok(Completion::Pending) => Completion::Pending,
            Err(error) => {
                // An actual failed Join is terminal, but not a successful task.
                self.finished = true;
                failures.record(error);
                Completion::Complete
            }
        }
    }
}

#[must_use = "The root owns its parts and must be retained across Pending completion"]
pub struct Composition<E, P = Empty> {
    parts: P,
    failures: Failures<E>,
    stop_requested: bool,
    stop_dispatched: bool,
    phase: Phase,
}

pub struct StartFailure<E, P> {
    /// Holds the original root, including any still-pending worker owner.
    pub root: Composition<E, P>,
}

/// Named static return types keep nested attachment types inspectable.
pub type LocalAttach<E, P, T, Close> =
    Result<Composition<E, Node<P, Owned<T, Close>>>, StartFailure<E, P>>;
pub type WorkerAttach<E, P, T, A, S, O> =
    Result<Composition<E, Node<P, Worker<T, A, S, O>>>, StartFailure<E, P>>;

impl<E> Composition<E, Empty> {
    pub fn new() -> Self {
        Self {
            parts: Empty,
            failures: Failures {
                primary: None,
                cleanup: Vec::new(),
            },
            stop_requested: false,
            stop_dispatched: false,
            phase: Phase::Prepared,
        }
    }
}

impl<E> Default for Composition<E, Empty> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, P: Chain<E>> Composition<E, P> {
    fn push<T: PartActions<E>>(self, part: T) -> Composition<E, Node<P, T>> {
        assert!(
            !self.stop_requested,
            "cannot attach after stop in the private static proof"
        );
        Composition {
            parts: Node {
                previous: self.parts,
                part,
            },
            failures: self.failures,
            stop_requested: self.stop_requested,
            stop_dispatched: self.stop_dispatched,
            // A later required part must never inherit a previously Running
            // phase until its own readiness has been observed.
            phase: Phase::Prepared,
        }
    }

    pub fn parts_mut(&mut self) -> &mut P {
        &mut self.parts
    }
    pub fn failures(&self) -> &Failures<E> {
        &self.failures
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Prepared components are not "running" until every required part is ready.
    pub fn all_required_ready(&mut self) -> bool {
        let ready =
            !self.stop_requested && self.failures.primary.is_none() && self.parts.all_ready();
        if ready {
            self.phase = Phase::Running;
        }
        ready
    }

    pub fn fail_required(&mut self, error: E) {
        self.failures.record(error);
        self.phase = Phase::Failed;
    }

    pub fn required_local<T, Close>(
        self,
        label: &'static str,
        value: T,
        close: Close,
    ) -> Composition<E, Node<P, Owned<T, Close>>>
    where
        Close: FnMut(&mut T) -> Result<(), E>,
    {
        self.push(Owned {
            value,
            close,
            ready: true,
            stopped: false,
            label,
        })
    }

    /// Foreground readiness is caller-observed, e.g. after winit+GPU init.
    pub fn required_foreground<T, Close>(
        self,
        label: &'static str,
        value: T,
        close: Close,
    ) -> Composition<E, Node<P, Owned<T, Close>>>
    where
        Close: FnMut(&mut T) -> Result<(), E>,
    {
        self.push(Owned {
            value,
            close,
            ready: false,
            stopped: false,
            label,
        })
    }

    pub fn borrowed<'a, T>(
        self,
        label: &'static str,
        value: &'a mut T,
    ) -> Composition<E, Node<P, Borrowed<'a, T>>> {
        self.push(Borrowed {
            value,
            label,
            stopped: false,
        })
    }

    pub fn try_required_local<T, Make, Close>(
        mut self,
        label: &'static str,
        make: Make,
        close: Close,
    ) -> LocalAttach<E, P, T, Close>
    where
        Make: FnOnce() -> Result<T, E>,
        Close: FnMut(&mut T) -> Result<(), E>,
    {
        match make() {
            Ok(value) => Ok(self.required_local(label, value, close)),
            Err(error) => {
                self.fail_required(error);
                self.request_stop();
                Err(StartFailure { root: self })
            }
        }
    }

    pub fn try_required_worker<T, Make, A, S, O>(
        mut self,
        label: &'static str,
        make: Make,
        close_admission: A,
        signal_stop: S,
        observe: O,
    ) -> WorkerAttach<E, P, T, A, S, O>
    where
        Make: FnOnce() -> Result<T, E>,
        A: FnMut(&mut T) -> Result<(), E>,
        S: FnMut(&mut T) -> Result<(), E>,
        O: FnMut(&mut T, Duration) -> Result<Completion, E>,
    {
        match make() {
            Ok(value) => Ok(self.push(Worker {
                value,
                close_admission,
                signal_stop,
                observe,
                admission_closed: false,
                stop_signalled: false,
                finished: false,
                label,
            })),
            Err(error) => {
                self.fail_required(error);
                self.request_stop();
                Err(StartFailure { root: self })
            }
        }
    }

    /// Admission first; only after every admission gate actually confirms
    /// closure may local/domain resources stop and workers receive shutdown.
    /// A failed gate retains ALL owners and can be retried; no false cleanup.
    pub fn request_stop(&mut self) {
        if self.stop_dispatched {
            return;
        }
        self.stop_requested = true;
        self.phase = Phase::Stopping;
        if !self.parts.close_admission(&mut self.failures) {
            return; // keep C1 alive while any worker could still admit commands
        }
        let local_closed = self.parts.close_locals_reverse(&mut self.failures);
        let workers_signalled = self.parts.signal_workers(&mut self.failures);
        self.stop_dispatched = local_closed && workers_signalled;
        // Failed cleanup/signal parts remain owned, and a repeat stop retries
        // them without reopening any successful admission or close operation.
    }

    /// Bounded observation only. The SAME root and worker survive Pending.
    pub fn complete_until(&mut self, deadline: Instant) -> Completion {
        self.request_stop();
        if !self.stop_dispatched {
            self.phase = Phase::Stopping;
            return Completion::Pending; // closing admission is not yet proven
        }
        let state = self.parts.observe_owned(deadline, &mut self.failures);
        self.phase = if state == Completion::Pending {
            Phase::Stopping
        } else if self.failures.primary.is_some() {
            Phase::Failed
        } else {
            Phase::Stopped
        };
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    fn now() -> Instant {
        Instant::now()
    }

    #[test]
    fn local_rc_and_borrowed_revision_log_remain_caller_local() {
        let state = Rc::new(Cell::new(0u64));
        let mut log = Vec::new();
        let mut root = Composition::<&'static str>::new()
            .required_local("domain", Rc::clone(&state), |_: &mut Rc<Cell<u64>>| Ok(()))
            .borrowed("revision-log", &mut log);
        assert!(root.all_required_ready());
        assert_eq!(root.parts_mut().previous.part.label(), "domain");
        root.parts_mut().previous.part.value.set(1);
        root.parts_mut().part.value.push(state.get());
        assert_eq!(root.parts_mut().part.label(), "revision-log");
        root.request_stop();
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(root.phase(), Phase::Stopped);
        drop(root); // ends the exclusive borrow, not the caller's log.
        assert_eq!(log, vec![1]);
        assert_eq!(state.get(), 1);
    }

    #[test]
    fn actual_foreground_observation_is_required_for_running() {
        let mut root =
            Composition::<&'static str>::new()
                .required_foreground("native", (), |_: &mut ()| Ok(()));
        assert!(!root.all_required_ready());
        assert_eq!(root.phase(), Phase::Prepared);
        root.parts_mut().part.mark_ready(); // a real caller callback must do this
        assert!(root.all_required_ready());
        assert_eq!(root.phase(), Phase::Running);
        root.request_stop();
        assert_eq!(root.complete_until(now()), Completion::Complete);
    }

    #[test]
    fn attaching_later_required_native_part_revokes_earlier_running_claim() {
        let mut root =
            Composition::<&'static str>::new().required_local("domain", (), |_: &mut ()| Ok(()));
        assert!(root.all_required_ready());
        assert_eq!(root.phase(), Phase::Running);
        let mut root = root.required_foreground("native", (), |_: &mut ()| Ok(()));
        assert_eq!(root.phase(), Phase::Prepared);
        assert!(!root.all_required_ready());
        root.parts_mut().part.mark_ready();
        assert!(root.all_required_ready());
        assert_eq!(root.complete_until(now()), Completion::Complete);
    }

    #[test]
    fn failed_web_start_rolls_back_owned_udp_and_domain_in_reverse_order() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let domain_log = Rc::clone(&log);
        let udp_log = Rc::clone(&log);
        let mut root = Composition::<&'static str>::new()
            .required_local("domain", (), move |_: &mut ()| {
                domain_log.borrow_mut().push("stop-domain");
                Ok(())
            })
            .try_required_local(
                "udp",
                || Ok(()),
                move |_: &mut ()| {
                    udp_log.borrow_mut().push("close-udp");
                    Ok(())
                },
            )
            .unwrap_or_else(|_| panic!("UDP test setup failed"));
        let failure = match root.try_required_worker(
            "web",
            || Err::<(), _>("http-bind"),
            |_: &mut ()| Ok(()),
            |_: &mut ()| Ok(()),
            |_: &mut (), _: Duration| Ok(Completion::Complete),
        ) {
            Ok(_) => panic!("Web must fail startup"),
            Err(failure) => failure,
        };
        root = failure.root;
        assert_eq!(root.failures().primary(), Some(&"http-bind"));
        assert_eq!(*log.borrow(), vec!["close-udp", "stop-domain"]);
        assert_eq!(root.complete_until(now()), Completion::Complete);
    }

    #[test]
    fn first_draw_error_survives_cleanup_error_and_later_window_close() {
        let may_close = Rc::new(Cell::new(false));
        let gate = Rc::clone(&may_close);
        let mut root = Composition::<&'static str>::new().required_foreground(
            "window",
            (),
            move |_: &mut ()| {
                if gate.get() { Ok(()) } else { Err("cleanup") }
            },
        );
        root.parts_mut().part.mark_ready();
        root.fail_required("draw-error"); // logged before caller requests exit
        root.request_stop(); // failed cleanup cannot claim completion
        assert_eq!(root.phase(), Phase::Stopping);
        assert_eq!(root.failures().primary(), Some(&"draw-error"));
        assert_eq!(root.failures().cleanup(), ["cleanup"]);
        may_close.set(true);
        root.request_stop(); // the same original owned window is retried
        root.request_stop(); // repeated window-close cannot change cause
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(root.failures().primary(), Some(&"draw-error"));
        assert_eq!(root.failures().cleanup(), ["cleanup"]);
        assert_eq!(root.phase(), Phase::Failed);
    }

    #[derive(Debug)]
    struct FakeWorker {
        identity: u64,
    }

    #[test]
    fn admission_closed_before_domain_and_independent_worker_shutdown() {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let ready = Rc::new(Cell::new(false));
        let same_handle = Rc::new(Cell::new(0u64));
        let log_domain = Rc::clone(&log);
        let log_gate = Rc::clone(&log);
        let log_signal = Rc::clone(&log);
        let join_id = Rc::clone(&same_handle);
        let ready_for_join = Rc::clone(&ready);
        let mut root = Composition::<&'static str>::new()
            .required_local("domain", (), move |_: &mut ()| {
                log_domain.borrow_mut().push("stop-domain");
                Ok(())
            })
            .try_required_worker(
                "web",
                || Ok(FakeWorker { identity: 71 }),
                move |_: &mut FakeWorker| {
                    log_gate.borrow_mut().push("close-admission");
                    Ok(())
                },
                move |_: &mut FakeWorker| {
                    log_signal.borrow_mut().push("signal-http");
                    Ok(())
                },
                move |worker: &mut FakeWorker, _: Duration| {
                    join_id.set(worker.identity);
                    Ok(if ready_for_join.get() {
                        Completion::Complete
                    } else {
                        Completion::Pending
                    })
                },
            )
            .unwrap_or_else(|_| panic!("fake worker startup failed"));
        root.request_stop();
        root.request_stop();
        assert_eq!(root.parts_mut().part.label(), "web");
        assert_eq!(
            &*log.borrow(),
            &["close-admission", "stop-domain", "signal-http"]
        );
        assert_eq!(root.complete_until(now()), Completion::Pending);
        assert_eq!(same_handle.get(), 71);
        assert_eq!(root.phase(), Phase::Stopping);
        assert!(!root.parts_mut().part.is_finished());
        ready.set(true);
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(same_handle.get(), 71);
        assert!(root.parts_mut().part.is_finished());
        assert_eq!(root.phase(), Phase::Stopped);
    }

    #[test]
    fn failed_admission_close_retains_c1_until_a_verified_retry() {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let gate_recovered = Rc::new(Cell::new(false));
        let domain_log = Rc::clone(&log);
        let gate_log = Rc::clone(&log);
        let worker_log = Rc::clone(&log);
        let may_close = Rc::clone(&gate_recovered);
        let mut root = Composition::<&'static str>::new()
            .required_local("domain", (), move |_: &mut ()| {
                domain_log.borrow_mut().push("stop-domain");
                Ok(())
            })
            .try_required_worker(
                "web",
                || Ok(FakeWorker { identity: 81 }),
                move |_: &mut FakeWorker| {
                    if !may_close.get() {
                        gate_log.borrow_mut().push("gate-failed");
                        Err("admission-still-open")
                    } else {
                        gate_log.borrow_mut().push("close-admission");
                        Ok(())
                    }
                },
                move |_: &mut FakeWorker| {
                    worker_log.borrow_mut().push("signal-http");
                    Ok(())
                },
                |_: &mut FakeWorker, _: Duration| Ok(Completion::Complete),
            )
            .unwrap_or_else(|_| panic!("fake worker startup failed"));
        root.request_stop();
        assert_eq!(&*log.borrow(), &["gate-failed"]);
        assert_eq!(root.phase(), Phase::Stopping);
        assert_eq!(root.complete_until(now()), Completion::Pending);
        assert_eq!(&*log.borrow(), &["gate-failed", "gate-failed"]);
        assert_eq!(root.failures().primary(), Some(&"admission-still-open"));
        assert_eq!(root.failures().cleanup(), ["admission-still-open"]);
        gate_recovered.set(true);
        root.request_stop();
        assert_eq!(
            &*log.borrow(),
            &[
                "gate-failed",
                "gate-failed",
                "close-admission",
                "stop-domain",
                "signal-http"
            ],
        );
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(root.phase(), Phase::Failed);
    }

    #[test]
    fn failed_later_native_start_retains_original_pending_worker_in_start_failure() {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let ready = Rc::new(Cell::new(false));
        let observed_handle = Rc::new(Cell::new(0u64));
        let domain_log = Rc::clone(&log);
        let gate_log = Rc::clone(&log);
        let signal_log = Rc::clone(&log);
        let can_complete = Rc::clone(&ready);
        let handle_id = Rc::clone(&observed_handle);
        let root = Composition::<&'static str>::new()
            .required_local("domain", (), move |_: &mut ()| {
                domain_log.borrow_mut().push("stop-domain");
                Ok(())
            })
            .try_required_worker(
                "web",
                || Ok(FakeWorker { identity: 92 }),
                move |_: &mut FakeWorker| {
                    gate_log.borrow_mut().push("close-admission");
                    Ok(())
                },
                move |_: &mut FakeWorker| {
                    signal_log.borrow_mut().push("signal-http");
                    Ok(())
                },
                move |worker: &mut FakeWorker, _: Duration| {
                    handle_id.set(worker.identity);
                    Ok(if can_complete.get() {
                        Completion::Complete
                    } else {
                        Completion::Pending
                    })
                },
            )
            .unwrap_or_else(|_| panic!("fake worker startup failed"));
        let failure = match root.try_required_local(
            "native",
            || Err::<(), _>("native-start-failure"),
            |_: &mut ()| Ok(()),
        ) {
            Err(failure) => failure,
            Ok(_) => panic!("native startup must fail"),
        };
        let mut owner = failure.root; // identical worker, NOT silently dropped
        assert_eq!(owner.failures().primary(), Some(&"native-start-failure"));
        assert_eq!(
            &*log.borrow(),
            &["close-admission", "stop-domain", "signal-http"]
        );
        assert_eq!(owner.complete_until(now()), Completion::Pending);
        assert_eq!(observed_handle.get(), 92);
        ready.set(true);
        assert_eq!(owner.complete_until(now()), Completion::Complete);
        assert_eq!(observed_handle.get(), 92);
        assert_eq!(owner.phase(), Phase::Failed);
    }

    #[test]
    fn failed_local_close_never_reports_complete_until_actual_retry() {
        let may_close = Rc::new(Cell::new(false));
        let attempts = Rc::new(Cell::new(0u32));
        let gate = Rc::clone(&may_close);
        let count = Rc::clone(&attempts);
        let mut root =
            Composition::<&'static str>::new().required_local("domain", (), move |_: &mut ()| {
                count.set(count.get() + 1);
                if gate.get() {
                    Ok(())
                } else {
                    Err("close-not-complete")
                }
            });
        root.request_stop();
        assert_eq!(attempts.get(), 1);
        assert_eq!(root.complete_until(now()), Completion::Pending);
        assert_eq!(attempts.get(), 2);
        assert_eq!(root.failures().primary(), Some(&"close-not-complete"));
        assert_eq!(root.phase(), Phase::Stopping);
        may_close.set(true);
        root.request_stop();
        assert_eq!(attempts.get(), 3);
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(attempts.get(), 3);
        assert_eq!(root.phase(), Phase::Failed); // original error retained
    }

    #[test]
    fn failed_worker_stop_signal_retains_original_worker_until_retry() {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let may_signal = Rc::new(Cell::new(false));
        let gate_log = Rc::clone(&log);
        let domain_log = Rc::clone(&log);
        let signal_log = Rc::clone(&log);
        let signallable = Rc::clone(&may_signal);
        let mut root = Composition::<&'static str>::new()
            .required_local("domain", (), move |_: &mut ()| {
                domain_log.borrow_mut().push("stop-domain");
                Ok(())
            })
            .try_required_worker(
                "web",
                || Ok(FakeWorker { identity: 121 }),
                move |_: &mut FakeWorker| {
                    gate_log.borrow_mut().push("close-admission");
                    Ok(())
                },
                move |_: &mut FakeWorker| {
                    if signallable.get() {
                        signal_log.borrow_mut().push("signal-http");
                        Ok(())
                    } else {
                        signal_log.borrow_mut().push("signal-failed");
                        Err("signal-unconfirmed")
                    }
                },
                |worker: &mut FakeWorker, _: Duration| {
                    assert_eq!(worker.identity, 121);
                    Ok(Completion::Complete)
                },
            )
            .unwrap_or_else(|_| panic!("worker setup must succeed"));
        root.request_stop();
        assert_eq!(
            &*log.borrow(),
            &["close-admission", "stop-domain", "signal-failed"]
        );
        assert_eq!(root.complete_until(now()), Completion::Pending);
        assert_eq!(
            &*log.borrow(),
            &[
                "close-admission",
                "stop-domain",
                "signal-failed",
                "signal-failed"
            ]
        );
        assert!(!root.parts_mut().part.is_finished()); // was not even observed
        may_signal.set(true);
        root.request_stop();
        assert_eq!(
            &*log.borrow(),
            &[
                "close-admission",
                "stop-domain",
                "signal-failed",
                "signal-failed",
                "signal-http"
            ]
        );
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert!(root.parts_mut().part.is_finished());
        assert_eq!(root.failures().primary(), Some(&"signal-unconfirmed"));
        assert_eq!(root.phase(), Phase::Failed);
    }

    #[test]
    fn worker_join_failure_is_terminal_but_not_successful_and_preserves_prior_error() {
        let mut root = Composition::<&'static str>::new()
            .try_required_worker(
                "web",
                || Ok(()),
                |_: &mut ()| Ok(()),
                |_: &mut ()| Ok(()),
                |_: &mut (), _: Duration| Err("join-failed"),
            )
            .unwrap_or_else(|_| panic!("start should succeed"));
        root.fail_required("native-early-error");
        root.request_stop();
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(root.failures().primary(), Some(&"native-early-error"));
        assert_eq!(root.failures().cleanup(), ["join-failed"]);
        assert!(root.parts_mut().part.is_finished());
        assert_eq!(root.phase(), Phase::Failed);
    }

    #[test]
    fn worker_first_failure_retained_when_no_other_failure() {
        let mut root = Composition::<&'static str>::new()
            .try_required_worker(
                "web",
                || Ok(()),
                |_: &mut ()| Ok(()),
                |_: &mut ()| Ok(()),
                |_: &mut (), _: Duration| Err("failed-join"),
            )
            .unwrap_or_else(|_| panic!("start should succeed"));
        root.request_stop();
        assert_eq!(root.complete_until(now()), Completion::Complete);
        assert_eq!(root.failures().primary(), Some(&"failed-join"));
    }
}
