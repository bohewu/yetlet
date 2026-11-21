//! Built-in, statically typed application-owned lifetime composition.
//!
//! [HostGroup] groups high-level [Host] descriptions using their default
//! [ResourceGroup] plans. [HostActivity] and [ManagedResource] are the custom
//! extension contracts. This module uses only std, creates no runtime or thread,
//! and imposes no blanket Send/Sync/'static bound on local or borrowed resources.
//! No Cargo feature opt-in is needed.
//!
//! [Builder] and [Composition] remain the advanced callback engine. Their legacy
//! local close-before-worker phase is distinct from post-Join resource release.
//!
//! # Local and borrowed resources
//!
//! ```
//! use std::{cell::Cell, rc::Rc, time::Instant};
//! use yetlet::composition::{Builder, Completion, Phase};
//!
//! let revision = Rc::new(Cell::new(0_u64));
//! let mut audit = Vec::new();
//! let mut app = Builder::<&'static str>::new()
//!     .required_local("domain", revision.clone(), |_| Ok(()))
//!     .borrowed("audit", &mut audit)
//!     .build();
//! assert!(app.all_required_ready());
//! app.parts_mut().previous().last().value_mut().unwrap().set(12);
//! app.parts_mut().last().value_mut().unwrap().push(revision.get());
//! assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
//! assert_eq!(app.phase(), Phase::Stopped);
//! drop(app); // ends the borrow; it does not dispose the caller's audit
//! assert_eq!(audit, [12]);
//! ```
//!
//! # Lifecycle contract
//!
//! A [Builder] accepts attachments, then [Builder::build] seals the topology.
//! Locals are ready on attachment; foreground and worker parts require an
//! explicit observed-ready report through [PartsMut]. Construction or thread
//! spawn alone is not readiness. The caller drives foreground execution.
//!
//! All admission callbacks must confirm closure before any local stop callback.
//! Local stops run in reverse attachment order; a failure retains earlier local
//! owners for retry. Worker stop signals are independent of data/admission queues.
//! Each successful step is latched. [Composition::complete_until] retains the
//! original storage while returning [Completion::Pending]. An observation
//! error is retriable; only [WorkerCompletion::Joined] is terminal. After every
//! admission gate closes, observation proceeds even if a stop signal fails: an
//! already exited worker must still be joined. Live workers remain Pending and
//! retry their signal. Aggregate completion also requires local cleanup and is
//! separate from success: inspect [Composition::failures].
//!
//! Opt-in [Resource] and [ManagedWorker] finalizers run only after that global
//! local/Join barrier. They release in reverse registration order, including
//! nested [Group] parts, using the same absolute deadline. Pending or a retryable
//! error retains earlier dependencies; terminal release errors are latched and
//! do not repeat a finished finalizer. Resource values are dropped on terminal
//! release, while legacy Local/Worker value containers retain their old behavior.
//!
//! Callbacks must preserve resources on errors/Pending and truthfully report
//! readiness, closure and Join. A factory that returns an error must clean up
//! resources it never returned. Panics are not caught. Shutdown/observation is
//! cooperative: the deadline supplies remaining time, not preemption or a hard
//! process-exit guarantee. Stop callbacks should be short and nonblocking.
//!
//! **Drop performs ordinary field destruction only**, without implicit stop,
//! wait or Join. Keep a failed-start root or Pending composition alive and drive
//! explicit completion. Dropping one early can detach a user-owned JoinHandle;
//! `must_use` is a warning, not a destructor guarantee. Completed compositions
//! also retain their value containers until dropped; close callbacks must
//! release the actual resources they claim to close.
//!
//! Topology cannot change after build:
//!
//! ```compile_fail
//! use yetlet::composition::Builder;
//! let app = Builder::<()>::new().build();
//! app.required_local("late", (), |_| Ok(()));
//! ```
//!
//! Worker storage cannot be replaced through an active access view:
//!
//! ```compile_fail
//! use yetlet::composition::{Builder, WorkerCompletion};
//! let mut app = Builder::<()>::new().required_worker(
//!     "worker", (), |_| Ok(()), |_| Ok(()),
//!     |_, _| Ok(WorkerCompletion::Joined(Ok(()))),
//! ).build();
//! app.parts_mut().last().value_mut();
//! ```

use std::fmt;
use std::task::{Context, Waker};
use std::time::{Duration, Instant};

mod hosts;
mod lifecycle;
mod resources;

pub use hosts::{
    BuiltHostGroup, ExclusiveClaim, Host, HostActivity, HostBuildError, HostFailure, HostGroup,
    HostOperation, HostOutcome, Managed, ManagedResource, ResourceGroup, ResourcePlan, Retained,
    ShutdownHandle, StartStatus,
};

#[doc(hidden)]
pub use hosts::{HostList, HostParts};

pub use resources::{
    ActivityBuilder, Group, ManagedWorker, ReleaseCompletion, ReleaseStatus, Resource,
    ResourceParts,
};

/// Internal callback storage that receives the caller's wake context.
#[doc(hidden)]
pub struct ContextCallback<F> {
    callback: F,
}

pub(crate) trait RequestOp<T, E> {
    fn request(&mut self, value: &mut T, cx: &mut Context<'_>) -> Result<(), E>;
}

impl<T, E, F> RequestOp<T, E> for F
where
    F: FnMut(&mut T) -> Result<(), E>,
{
    fn request(&mut self, value: &mut T, _: &mut Context<'_>) -> Result<(), E> {
        self(value)
    }
}

impl<T, E, F> RequestOp<T, E> for ContextCallback<F>
where
    F: FnMut(&mut T, &mut Context<'_>) -> Result<(), E>,
{
    fn request(&mut self, value: &mut T, cx: &mut Context<'_>) -> Result<(), E> {
        (self.callback)(value, cx)
    }
}

pub(crate) trait ObserveOp<T, E> {
    fn observe(
        &mut self,
        value: &mut T,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<E>, E>;
}

impl<T, E, F> ObserveOp<T, E> for F
where
    F: FnMut(&mut T, Duration) -> Result<WorkerCompletion<E>, E>,
{
    fn observe(
        &mut self,
        value: &mut T,
        _: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<E>, E> {
        self(value, budget)
    }
}

impl<T, E, F> ObserveOp<T, E> for ContextCallback<F>
where
    F: FnMut(&mut T, &mut Context<'_>, Duration) -> Result<WorkerCompletion<E>, E>,
{
    fn observe(
        &mut self,
        value: &mut T,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<E>, E> {
        (self.callback)(value, cx, budget)
    }
}

pub(crate) trait ReleaseOp<T, E> {
    fn release(
        &mut self,
        value: &mut T,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<E>, E>;
}

impl<T, E, F> ReleaseOp<T, E> for F
where
    F: FnMut(&mut T, Duration) -> Result<ReleaseCompletion<E>, E>,
{
    fn release(
        &mut self,
        value: &mut T,
        _: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<E>, E> {
        self(value, budget)
    }
}

impl<T, E, F> ReleaseOp<T, E> for ContextCallback<F>
where
    F: FnMut(&mut T, &mut Context<'_>, Duration) -> Result<ReleaseCompletion<E>, E>,
{
    fn release(
        &mut self,
        value: &mut T,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<E>, E> {
        (self.callback)(value, cx, budget)
    }
}

/// Lifecycle progress; failure details are independently retained in [Failures].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Topology is sealed but required readiness has not been confirmed.
    Prepared,
    /// All required components have explicitly become ready.
    Running,
    /// Stop was requested; some cleanup or owned completion may remain.
    Stopping,
    /// Cleanup completed without a recorded failure.
    Stopped,
    /// Cleanup completed with at least one recorded failure.
    Failed,
}

/// Whether explicit cleanup and all owned completion have been confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    /// All locals closed, workers joined and managed resources released; inspect failures.
    Complete,
    /// Original owners remain stored and need further caller-driven completion.
    Pending,
}

/// A worker observation distinguishes unfinished work from an actual Join.
#[derive(Debug, PartialEq, Eq)]
pub enum WorkerCompletion<E> {
    /// The same original worker must remain available to observe again.
    Pending,
    /// The original worker has been joined, with its actual execution outcome.
    Joined(Result<(), E>),
}

/// Cached terminal status of an owned worker, separate from group completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStatus {
    /// No terminal Join has been reported.
    Pending,
    /// Actual Join and worker execution succeeded.
    Succeeded,
    /// Actual Join completed with a worker failure.
    Failed,
}

/// First failure is stable; later failures are retained in observation order.
#[derive(Debug)]
pub struct Failures<E> {
    primary: Option<E>,
    secondary: Vec<E>,
}

impl<E> Failures<E> {
    fn record(&mut self, error: E) {
        if self.primary.is_none() {
            self.primary = Some(error);
        } else {
            self.secondary.push(error);
        }
    }

    /// Original startup, required-component, shutdown or Join failure.
    pub fn primary(&self) -> Option<&E> {
        self.primary.as_ref()
    }

    /// Subsequent failures, including repeated unsuccessful cleanup attempts.
    pub fn secondary(&self) -> &[E] {
        &self.secondary
    }
}

/// Empty static storage. Construct a composition through [Builder].
pub struct Empty;

/// A concrete typed attachment, with read-only structural navigation.
pub struct Attached<P, T> {
    previous: P,
    last: T,
}

impl<P, T> Attached<P, T> {
    /// Earlier attachments, in their original concrete types.
    pub fn previous(&self) -> &P {
        &self.previous
    }

    /// Last attachment's read-only value and lifecycle status.
    pub fn last(&self) -> &T {
        &self.last
    }
}

/// An owned caller-thread resource and its explicit close callback.
pub struct Local<T, Close> {
    value: T,
    close: Close,
    label: &'static str,
    ready: bool,
    stopped: bool,
}

impl<T, Close> Local<T, Close> {
    /// Caller-assigned diagnostic name.
    pub fn label(&self) -> &'static str {
        self.label
    }
    /// Read-only resource access; interior-mutability policy remains the caller's.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Whether actual required readiness has been reported.
    pub fn is_ready(&self) -> bool {
        self.ready && !self.stopped
    }
    /// Whether its close callback has confirmed success.
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }
}

/// A mutable borrow whose resource is never closed or owned by this module.
pub struct Borrowed<'a, T> {
    value: &'a mut T,
    label: &'static str,
    stopped: bool,
}

impl<T> Borrowed<'_, T> {
    /// Caller-assigned diagnostic name.
    pub fn label(&self) -> &'static str {
        self.label
    }
    /// Read-only access; the borrow ends when the composition is dropped.
    pub fn value(&self) -> &T {
        self.value
    }
}

/// Original owned worker storage with separate admission, signal and Join steps.
pub struct Worker<T, Admission, Signal, Observe> {
    value: T,
    admission: Admission,
    signal: Signal,
    observe: Observe,
    label: &'static str,
    ready: bool,
    admission_closed: bool,
    stop_signalled: bool,
    join_status: JoinStatus,
}

impl<T, A, S, O> Worker<T, A, S, O> {
    /// Caller-assigned diagnostic name.
    pub fn label(&self) -> &'static str {
        self.label
    }
    /// Read-only original worker storage; mutation is reserved for callbacks.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Whether actual required readiness has been reported before stop.
    pub fn is_ready(&self) -> bool {
        self.ready && !self.admission_closed
    }
    /// Whether the admission fence confirmed closure.
    pub fn admission_closed(&self) -> bool {
        self.admission_closed
    }
    /// Whether the distinct shutdown signal confirmed delivery.
    pub fn stop_signalled(&self) -> bool {
        self.stop_signalled
    }
    /// Actual terminal Join outcome, independently of other failures.
    pub fn join_status(&self) -> JoinStatus {
        self.join_status
    }
}

/// Restricted mutable navigation; it never exposes replaceable lifecycle slots.
pub struct PartsMut<'a, P> {
    parts: &'a mut P,
    active: bool,
}

impl<'a, P, T> PartsMut<'a, Attached<P, T>> {
    /// Select earlier attachments without exposing their storage for replacement.
    pub fn previous(self) -> PartsMut<'a, P> {
        PartsMut {
            parts: &mut self.parts.previous,
            active: self.active,
        }
    }

    /// Select the last attachment's restricted view.
    pub fn last(self) -> PartMut<'a, T> {
        PartMut {
            part: &mut self.parts.last,
            active: self.active,
        }
    }
}

/// Restricted operations on one attachment; fields and callbacks remain private.
pub struct PartMut<'a, T> {
    part: &'a mut T,
    active: bool,
}

impl<'a, T, C> PartMut<'a, Local<T, C>> {
    /// Mutate the local resource only before required failure or stop.
    pub fn value_mut(self) -> Option<&'a mut T> {
        self.active.then_some(&mut self.part.value)
    }

    /// Report actual foreground readiness; returns false once stopping begins.
    pub fn mark_ready(self) -> bool {
        if !self.active {
            return false;
        }
        self.part.ready = true;
        true
    }
}

impl<'a, 'resource, T> PartMut<'a, Borrowed<'resource, T>> {
    /// Mutate the borrowed resource only while the composition is active.
    pub fn value_mut(self) -> Option<&'a mut T> {
        self.active.then_some(&mut *self.part.value)
    }
}

impl<T, A, S, O> PartMut<'_, Worker<T, A, S, O>> {
    /// Report observed worker readiness, not merely successful thread creation.
    pub fn mark_ready(self) -> bool {
        if !self.active {
            return false;
        }
        self.part.ready = true;
        true
    }
}

/// Builds a static topology; build it or retain its failed-start root explicitly.
///
/// No startup callback or worker is automatically run in Drop.
#[must_use = "Build and complete owned resources explicitly; Drop does not stop or join"]
pub struct Builder<E, P = Empty> {
    root: Composition<E, P>,
}

/// Application-owned static resources with caller-driven lifecycle and completion.
///
/// There is no attach operation after build and no blanket Send/Sync requirement.
#[must_use = "Retain the composition across Pending and explicitly drive completion"]
pub struct Composition<E, P = Empty> {
    parts: P,
    failures: Failures<E>,
    phase: Phase,
    stop_requested: bool,
    admission_closed: bool,
    locals_stopped: bool,
}

/// Failed startup retains the exact previous composition, including pending work.
#[must_use = "Recover the original root with into_root and drive owned completion"]
pub struct StartFailure<E, P> {
    root: Composition<E, P>,
}

impl<E, P> StartFailure<E, P> {
    /// Inspect the original root and the retained first failure.
    pub fn root(&self) -> &Composition<E, P> {
        &self.root
    }

    /// Recover ownership to drive stop/completion; no worker is replaced.
    pub fn into_root(self) -> Composition<E, P> {
        self.root
    }
}

impl<E, P> fmt::Debug for StartFailure<E, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StartFailure")
            .field("phase", &self.root.phase)
            .field("has_failure", &self.root.failures.primary.is_some())
            .finish_non_exhaustive()
    }
}

/// Statically typed builder after one local attachment.
pub type LocalBuilder<E, P, T, C> = Builder<E, Attached<P, Local<T, C>>>;
/// Statically typed builder after one worker attachment.
pub type WorkerBuilder<E, P, T, A, S, O> = Builder<E, Attached<P, Worker<T, A, S, O>>>;
/// Fallible local construction preserves the original root on failure.
pub type LocalBuildResult<E, P, T, C> = Result<LocalBuilder<E, P, T, C>, StartFailure<E, P>>;
/// Fallible worker construction preserves the original root on failure.
pub type WorkerBuildResult<E, P, T, A, S, O> =
    Result<WorkerBuilder<E, P, T, A, S, O>, StartFailure<E, P>>;

impl<E> Builder<E, Empty> {
    /// Start an empty caller-owned topology.
    pub fn new() -> Self {
        Self {
            root: Composition {
                parts: Empty,
                failures: Failures {
                    primary: None,
                    secondary: Vec::new(),
                },
                phase: Phase::Prepared,
                stop_requested: false,
                admission_closed: false,
                locals_stopped: false,
            },
        }
    }
}

impl<E> Default for Builder<E, Empty> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, P> Builder<E, P> {
    fn push<T>(self, last: T) -> Builder<E, Attached<P, T>> {
        Builder {
            root: Composition {
                parts: Attached {
                    previous: self.root.parts,
                    last,
                },
                failures: self.root.failures,
                phase: Phase::Prepared,
                stop_requested: false,
                admission_closed: false,
                locals_stopped: false,
            },
        }
    }

    /// Seal the topology without starting a scheduler or declaring readiness.
    pub fn build(self) -> Composition<E, P> {
        self.root
    }

    /// Attach a resource already ready for caller-thread use.
    pub fn required_local<T, C>(
        self,
        label: &'static str,
        value: T,
        close: C,
    ) -> LocalBuilder<E, P, T, C>
    where
        C: FnMut(&mut T) -> Result<(), E>,
    {
        self.push(Local {
            value,
            close,
            label,
            ready: true,
            stopped: false,
        })
    }

    /// Attach a foreground resource whose actual readiness will be reported later.
    pub fn required_foreground<T, C>(
        self,
        label: &'static str,
        value: T,
        close: C,
    ) -> LocalBuilder<E, P, T, C>
    where
        C: FnMut(&mut T) -> Result<(), E>,
    {
        self.push(Local {
            value,
            close,
            label,
            ready: false,
            stopped: false,
        })
    }

    /// Borrow a caller-owned resource without closing it or its runtime.
    pub fn borrowed<'a, T>(
        self,
        label: &'static str,
        value: &'a mut T,
    ) -> Builder<E, Attached<P, Borrowed<'a, T>>> {
        self.push(Borrowed {
            value,
            label,
            stopped: false,
        })
    }

    /// Attach an original worker; report its actual readiness after build.
    ///
    /// Admission and signal return Ok only after their respective effects succeed.
    /// Observation returns Err for a retriable observation failure; a terminal
    /// worker failure is Ok(WorkerCompletion::Joined(Err(error))).
    pub fn required_worker<T, A, S, O>(
        self,
        label: &'static str,
        value: T,
        admission: A,
        signal: S,
        observe: O,
    ) -> WorkerBuilder<E, P, T, A, S, O>
    where
        A: FnMut(&mut T) -> Result<(), E>,
        S: FnMut(&mut T) -> Result<(), E>,
        O: FnMut(&mut T, Duration) -> Result<WorkerCompletion<E>, E>,
    {
        self.push(Worker {
            value,
            admission,
            signal,
            observe,
            label,
            ready: false,
            admission_closed: false,
            stop_signalled: false,
            join_status: JoinStatus::Pending,
        })
    }
}

// These private bounds implement recursion over crate-created static storage.
// External Hosts supply ordinary values/callbacks, never implement this trait.
#[allow(private_bounds)]
impl<E, P: lifecycle::Chain<E>> Builder<E, P> {
    /// On startup error request rollback and return the original owned root.
    ///
    /// The factory must clean up any partial resources it does not return.
    pub fn try_required_local<T, Make, C>(
        mut self,
        label: &'static str,
        make: Make,
        close: C,
    ) -> LocalBuildResult<E, P, T, C>
    where
        Make: FnOnce() -> Result<T, E>,
        C: FnMut(&mut T) -> Result<(), E>,
    {
        match make() {
            Ok(value) => Ok(self.required_local(label, value, close)),
            Err(error) => {
                self.root.fail_required(error);
                Err(StartFailure { root: self.root })
            }
        }
    }

    /// On worker startup error retain all earlier workers through rollback.
    ///
    /// A successful factory alone does not mark the new worker ready.
    pub fn try_required_worker<T, Make, A, S, O>(
        mut self,
        label: &'static str,
        make: Make,
        admission: A,
        signal: S,
        observe: O,
    ) -> WorkerBuildResult<E, P, T, A, S, O>
    where
        Make: FnOnce() -> Result<T, E>,
        A: FnMut(&mut T) -> Result<(), E>,
        S: FnMut(&mut T) -> Result<(), E>,
        O: FnMut(&mut T, Duration) -> Result<WorkerCompletion<E>, E>,
    {
        match make() {
            Ok(value) => Ok(self.required_worker(label, value, admission, signal, observe)),
            Err(error) => {
                self.root.fail_required(error);
                Err(StartFailure { root: self.root })
            }
        }
    }
}

impl<E, P> Composition<E, P> {
    /// Read-only typed storage and individual lifecycle receipts.
    pub fn parts(&self) -> &P {
        &self.parts
    }

    /// Restricted access; local mutation/readiness reporting ends at stop/failure.
    pub fn parts_mut(&mut self) -> PartsMut<'_, P> {
        PartsMut {
            parts: &mut self.parts,
            active: !self.stop_requested && self.failures.primary.is_none(),
        }
    }

    /// Current aggregate progress, separate from retained failure evidence.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// First and subsequent failures, retained across retries and completion.
    pub fn failures(&self) -> &Failures<E> {
        &self.failures
    }
}

#[allow(private_bounds)]
impl<E, P: lifecycle::Chain<E>> Composition<E, P> {
    /// Confirm all required readiness without running or blocking any host loop.
    pub fn all_required_ready(&mut self) -> bool {
        let ready =
            !self.stop_requested && self.failures.primary.is_none() && self.parts.all_ready();
        if ready {
            self.phase = Phase::Running;
        }
        ready
    }

    /// Retain the first required failure and immediately request coordinated stop.
    pub fn fail_required(&mut self, error: E) {
        self.fail_with_context(error, &mut Context::from_waker(Waker::noop()));
    }

    pub(crate) fn fail_with_context(&mut self, error: E, cx: &mut Context<'_>) {
        self.failures.record(error);
        if matches!(self.phase, Phase::Stopped | Phase::Failed) {
            self.phase = Phase::Failed;
        } else {
            self.request_stop_with_context(cx);
        }
    }

    /// Fence every admission gate, then stop locals and signal owned workers.
    ///
    /// A failed fence prevents every local stop and worker signal in this pass.
    /// A failed local close retains that resource and all earlier local owners.
    /// Signals remain independent of local-close retries. Successes are latched;
    /// workers already confirmed Joined need no further shutdown signal.
    /// Repeating this method retries only unfinished operations.
    pub fn request_stop(&mut self) {
        self.request_stop_with_context(&mut Context::from_waker(Waker::noop()));
    }

    pub(crate) fn request_stop_with_context(&mut self, cx: &mut Context<'_>) {
        if matches!(self.phase, Phase::Stopped | Phase::Failed) {
            return;
        }
        self.stop_requested = true;
        self.phase = Phase::Stopping;
        if !self.admission_closed {
            self.admission_closed = self.parts.close_admission(cx, &mut self.failures);
            if !self.admission_closed {
                return;
            }
        }
        self.locals_stopped = self.parts.close_locals(&mut self.failures);
        self.parts.signal_workers(cx, &mut self.failures);
    }

    /// Request stop and observe original workers within a cooperative time budget.
    ///
    /// All callbacks receive the same overall deadline as remaining time.
    /// Callbacks may exceed it; no preemption, automatic retry loop or hard timeout
    /// is promised. Pending retains the same root and worker storage. Complete
    /// means locals closed, workers joined (including failed Joins), and opt-in
    /// finalizers terminal. Final release uses the same remaining deadline;
    /// Pending/retryable failure retains earlier dependencies. Inspect failures
    /// and individual Join/release status. After admission closes, a failed signal does not
    /// prevent observation of an already finished worker.
    pub fn complete_until(&mut self, deadline: Instant) -> Completion {
        self.complete_with_context(&mut Context::from_waker(Waker::noop()), deadline)
    }

    pub(crate) fn complete_with_context(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
    ) -> Completion {
        self.request_stop_with_context(cx);
        if !self.admission_closed {
            return Completion::Pending;
        }
        // A failed signal may mean the original worker already exited. Observe
        // after admission closes even if local cleanup or signal delivery failed.
        let workers = self.parts.observe_owned(cx, deadline, &mut self.failures);
        let completion = if self.locals_stopped && workers == Completion::Complete {
            self.parts.release_owned(cx, deadline, &mut self.failures)
        } else {
            Completion::Pending
        };
        self.phase = match completion {
            Completion::Pending => Phase::Stopping,
            Completion::Complete if self.failures.primary.is_some() => Phase::Failed,
            Completion::Complete => Phase::Stopped,
        };
        completion
    }
}
