use super::{AdmissionOutcome, HostFailure, HostOperation, HostOutcome, HostParts};
use crate::composition::{Completion, Composition, Failures, Phase, ResourceParts};
use std::{
    error::Error,
    fmt,
    future::poll_fn,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};

/// A caller-thread driver for the group's existing lifecycle polling.
///
/// Construction is inert. Core prepares the whole plan before calling run.
/// Keep partial driver ownership in self, and pass the supplied progress callback
/// a valid wake context from the selected event environment. The shutdown handle
/// can interrupt pending acquisition without waiting for activity readiness.
///
/// Normal completion calls progress until it returns Ready. Returning earlier
/// retains the driver and all unfinished obligations in BuiltHostGroup; Core
/// requests stop without running affine cleanup outside this driver. A later
/// run borrows this same instance and never repeats preparation or replaces it.
/// Drivers that cannot resume after backend exit must report that limitation.
///
/// E is the application's original error type; no Clone, Send, Sync or 'static
/// bound is imposed. Panics are not caught.
pub trait BlockingDriver<E> {
    fn run(
        &mut self,
        shutdown: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), E>;
}

/// Default typed metadata for plans without a platform event-loop driver.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoDriver;

mod sealed {
    pub trait Driver {}
}

/// Sealed storage for selected drivers, independent of lifecycle ownership.
#[doc(hidden)]
pub trait DriverPlan<E>: sealed::Driver {
    fn names(&self, names: &mut Vec<&'static str>);
    fn has_driver(&self) -> bool;
    fn run(
        &mut self,
        shutdown: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), HostFailure<E>>;
}

impl sealed::Driver for NoDriver {}
impl<E> DriverPlan<E> for NoDriver {
    fn names(&self, _: &mut Vec<&'static str>) {}
    fn has_driver(&self) -> bool {
        false
    }
    fn run(
        &mut self,
        _: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), HostFailure<E>> {
        let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
        let mut cx = Context::from_waker(&waker);
        while progress(&mut cx).is_pending() {
            // unpark stores a token, including a wake between poll and park.
            thread::park_timeout(Duration::from_millis(10));
        }
        Ok(())
    }
}

#[doc(hidden)]
pub struct SelectedDriver<D> {
    pub(super) driver: D,
    pub(super) host: &'static str,
}

impl<D> sealed::Driver for SelectedDriver<D> {}
impl<E, D: BlockingDriver<E>> DriverPlan<E> for SelectedDriver<D> {
    fn names(&self, names: &mut Vec<&'static str>) {
        names.push(self.host);
    }
    fn has_driver(&self) -> bool {
        true
    }
    fn run(
        &mut self,
        shutdown: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), HostFailure<E>> {
        self.driver
            .run(shutdown, progress)
            .map_err(|error| HostFailure::new(self.host, HostOperation::Drive, error))
    }
}

#[doc(hidden)]
pub struct DriverList<P, D> {
    pub(super) previous: P,
    pub(super) last: D,
}

impl<P, D> sealed::Driver for DriverList<P, D> {}
impl<E, P: DriverPlan<E>, D: DriverPlan<E>> DriverPlan<E> for DriverList<P, D> {
    fn names(&self, names: &mut Vec<&'static str>) {
        self.previous.names(names);
        self.last.names(names);
    }
    fn has_driver(&self) -> bool {
        self.previous.has_driver() || self.last.has_driver()
    }
    fn run(
        &mut self,
        shutdown: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), HostFailure<E>> {
        if self.last.has_driver() {
            self.last.run(shutdown, progress)
        } else {
            self.previous.run(shutdown, progress)
        }
    }
}

/// The selected progress environment for an entire retained Host group.
///
/// Selection is latched before any acquisition. Core creates no executor or
/// service thread; each adapter validates and prepares its supported placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostRunMode {
    /// Nonblocking caller-driven progress; the default for polling/run_async.
    Async,
    /// The calling thread drives run; adapters may own explicit service threads.
    Blocking,
}

/// A synchronous drive rejection or the group's retained original failure.
///
/// A mode mismatch leaves the owner and admission state unchanged. Resume its
/// existing nonblocking drive instead of abandoning the retained obligations.
#[derive(Debug)]
pub enum HostRunError<'a, E> {
    ModeMismatch {
        selected: HostRunMode,
        requested: HostRunMode,
    },
    /// The selected driver returned while lifecycle obligations remain owned.
    /// Any original driver error is retained in failures(). Only resuming the
    /// same retained driver may advance its unfinished lifecycle callbacks.
    DriverReturnedEarly {
        host: &'static str,
    },
    HostFailure(&'a HostFailure<E>),
}

impl<E: fmt::Display> fmt::Display for HostRunError<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModeMismatch {
                selected,
                requested,
            } => write!(
                f,
                "cannot drive a {selected:?} Host group with {requested:?}; resume its selected driver"
            ),
            Self::DriverReturnedEarly { host } => write!(
                f,
                "{host} driver returned before terminal cleanup; unfinished owners are retained"
            ),
            Self::HostFailure(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl<E: Error + 'static> Error for HostRunError<'_, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostFailure(error) => Some(*error),
            Self::ModeMismatch { .. } | Self::DriverReturnedEarly { .. } => None,
        }
    }
}

struct ThreadWake(thread::Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

#[derive(Default)]
struct ShutdownState {
    requested: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

/// A clonable stop request. It wakes the current borrowing drive; it neither
/// transfers ownership nor creates a background shutdown task.
#[derive(Clone, Default)]
pub struct ShutdownHandle(Arc<ShutdownState>);

impl ShutdownHandle {
    pub fn request_shutdown(&self) {
        if !self.0.requested.swap(true, Ordering::AcqRel) {
            let waker = self
                .0
                .waker
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    pub fn is_requested(&self) -> bool {
        self.0.requested.load(Ordering::Acquire)
    }

    fn register(&self, waker: &Waker) {
        // RawWaker clone/drop callbacks are user code: never invoke them while
        // holding the lock they could re-enter through a shutdown request.
        let candidate = waker.clone();
        let previous = {
            let mut current = self.0.waker.lock().unwrap_or_else(|e| e.into_inner());
            if current
                .as_ref()
                .is_none_or(|registered| !registered.will_wake(&candidate))
            {
                current.replace(candidate)
            } else {
                None
            }
        };
        drop(previous);
        // The caller checks requested after registration. A concurrent request
        // either becomes visible there or takes this waker and wakes the drive.
    }

    fn clear_waker(&self) {
        let previous = self
            .0
            .waker
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        drop(previous);
    }
}

/// Readiness progress; cleanup success is separately recorded by the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartStatus {
    Running,
    Stopping,
}

/// The retained owner returned by HostGroup::build.
///
/// Poll methods are nonblocking extension points for a caller's event loop or
/// executor. run_async borrows this same owner and adds no runtime or thread.
/// Cancelling only that borrowing future preserves all stored progress. Keep
/// this owner outside the cancelled task/scope: dropping both still drops the
/// original owners and is NOT completed shutdown.
///
/// Ordinary Drop runs no stop, Join or finalization callbacks. Complete means
/// every original activity and finalizer is terminal, not that all succeeded.
/// Resources are retired on terminal finalization; the coordination shell and
/// original failures remain inspectable. Rust borrows may last until this owner
/// itself is dropped.
///
/// Topology and original completion storage are not publicly replaceable:
///
/// ```compile_fail
/// use yetlet::HostGroup;
/// let hosts = HostGroup::<()>::new().build().unwrap();
/// hosts.with_host(());
/// ```
#[must_use = "Keep this owner across Pending and drive explicit terminal shutdown"]
pub struct BuiltHostGroup<E, P, D = NoDriver> {
    state: HostState<E, P>,
    driver: D,
    driver_host: Option<&'static str>,
}

struct HostState<E, P> {
    core: Composition<HostFailure<E>, P>,
    shutdown: ShutdownHandle,
    run_mode: Option<HostRunMode>,
    driver_prepared: bool,
}

impl<E, P> Drop for HostState<E, P> {
    fn drop(&mut self) {
        // A caller's waker can own a ShutdownHandle back to this registration.
        // Break that cycle before ordinary field destruction; clearing the
        // registration invokes no lifecycle callback and drops outside its lock.
        self.shutdown.clear_waker();
    }
}

impl<E, P> HostState<E, P> {
    pub(super) fn new(core: Composition<HostFailure<E>, P>, shutdown: ShutdownHandle) -> Self {
        Self {
            core,
            shutdown,
            run_mode: None,
            driver_prepared: false,
        }
    }

    pub fn phase(&self) -> Phase {
        self.core.phase()
    }

    pub fn failures(&self) -> &Failures<HostFailure<E>> {
        self.core.failures()
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.shutdown.clone()
    }
}

impl<E, P: HostParts<E> + ResourceParts<HostFailure<E>>> HostState<E, P> {
    fn prepare_driver(&mut self, cx: &mut Context<'_>) -> bool {
        if !self.driver_prepared {
            let mode = *self.run_mode.get_or_insert(HostRunMode::Async);
            let prepared = self.core.parts.prepare_driver(mode);
            self.driver_prepared = true;
            if let Err(error) = prepared {
                self.core.fail_with_context(error, cx);
                return false;
            }
            if self.shutdown.is_requested() {
                self.core.request_stop_with_context(cx);
                return false;
            }
        }
        true
    }

    /// Sequential acquisition/readiness, then admission for all Hosts. A failure
    /// requests rollback in this same owner and returns Stopping, never an error
    /// that carries cleanup ownership and can accidentally be discarded with ?.
    pub fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<StartStatus> {
        self.shutdown.register(cx.waker());
        if self.shutdown.is_requested()
            || matches!(
                self.core.phase(),
                Phase::Stopping | Phase::Stopped | Phase::Failed
            )
        {
            self.core.request_stop_with_context(cx);
            return Poll::Ready(StartStatus::Stopping);
        }
        if self.core.phase() == Phase::Running {
            return Poll::Ready(StartStatus::Running);
        }
        if !self.prepare_driver(cx) {
            return Poll::Ready(StartStatus::Stopping);
        }
        let progress = self.core.parts.poll_start(cx, &self.shutdown);
        if let Poll::Ready(Err(error)) = progress {
            self.core.fail_with_context(error, cx);
            return Poll::Ready(StartStatus::Stopping);
        }
        if self.shutdown.is_requested() {
            self.core.request_stop_with_context(cx);
            return Poll::Ready(StartStatus::Stopping);
        }
        // Earlier started activities remain observable while a later Host is
        // still acquiring or waiting for readiness. Do not open admission after
        // an already observed termination.
        if self.core.parts.poll_finished(cx) {
            self.shutdown.request_shutdown();
            self.core.request_stop_with_context(cx);
            return Poll::Ready(StartStatus::Stopping);
        }
        if progress.is_pending() {
            return Poll::Pending;
        }
        assert!(self.core.all_required_ready());
        match self.core.parts.open(&self.shutdown) {
            Ok(AdmissionOutcome::Opened) => {}
            Ok(AdmissionOutcome::Closed) => {
                self.shutdown.request_shutdown();
                self.core.request_stop_with_context(cx);
                return Poll::Ready(StartStatus::Stopping);
            }
            Err(error) => {
                self.core.fail_with_context(error, cx);
                return Poll::Ready(StartStatus::Stopping);
            }
        }
        if self.shutdown.is_requested() {
            self.core.request_stop_with_context(cx);
            Poll::Ready(StartStatus::Stopping)
        } else {
            Poll::Ready(StartStatus::Running)
        }
    }

    pub fn request_shutdown(&mut self) {
        self.shutdown.request_shutdown();
        self.core.request_stop();
    }

    /// Advance the existing global admission/local/original-Join/release
    /// coordinator. Deadline is a cooperative budget, not preemption or owner
    /// disposal. Pending retains the exact same operations and dependencies.
    pub fn poll_shutdown(&mut self, cx: &mut Context<'_>, deadline: Instant) -> Poll<()> {
        self.shutdown.request_shutdown();
        match self.core.complete_with_context(cx, deadline) {
            Completion::Pending => Poll::Pending,
            Completion::Complete => {
                self.shutdown.clear_waker();
                Poll::Ready(())
            }
        }
    }

    /// Advance startup/running/shutdown once. Every running Host is polled per
    /// pass. The first finished activity requests coordinated group shutdown;
    /// its actual execution outcome is retained until original Join.
    pub fn poll_run(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if matches!(self.core.phase(), Phase::Stopped | Phase::Failed) {
            return Poll::Ready(());
        }
        match self.poll_start(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(StartStatus::Stopping) => {}
            Poll::Ready(StartStatus::Running) => {
                let finished = self.core.parts.poll_finished(cx);
                if P::COUNT != 0 && !finished && !self.shutdown.is_requested() {
                    return Poll::Pending;
                }
                self.shutdown.request_shutdown();
            }
        }
        self.poll_shutdown(cx, Instant::now())
    }

    pub fn outcomes(&self) -> Vec<HostOutcome> {
        let mut result = Vec::with_capacity(P::COUNT);
        self.core.parts.outcomes(&mut result);
        result
    }
}

impl<E, P, D> BuiltHostGroup<E, P, D> {
    pub(super) fn new(
        core: Composition<HostFailure<E>, P>,
        shutdown: ShutdownHandle,
        driver: D,
        driver_host: Option<&'static str>,
    ) -> Self {
        Self {
            state: HostState::new(core, shutdown),
            driver,
            driver_host,
        }
    }

    fn requires_blocking_driver(&self) -> bool {
        self.driver_host.is_some()
            && self.state.run_mode == Some(HostRunMode::Blocking)
            && !matches!(self.state.phase(), Phase::Stopped | Phase::Failed)
    }

    pub fn phase(&self) -> Phase {
        self.state.phase()
    }

    pub fn failures(&self) -> &Failures<HostFailure<E>> {
        self.state.failures()
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.state.shutdown_handle()
    }
}

impl<E, P: HostParts<E> + ResourceParts<HostFailure<E>>, D> BuiltHostGroup<E, P, D> {
    /// Prepare the complete plan, acquire sequentially and observe all readiness
    /// before opening admission. Startup failure retains rollback in this owner.
    /// Pending also retains progress that requires the selected blocking driver.
    pub fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<StartStatus> {
        if self.requires_blocking_driver() {
            return Poll::Pending;
        }
        self.state.poll_start(cx)
    }

    /// Request admission closure and coordinated shutdown on the calling thread.
    /// If a selected blocking driver is required, defer its callbacks until that
    /// same driver resumes; this request never bypasses its execution boundary.
    pub fn request_shutdown(&mut self) {
        if self.requires_blocking_driver() {
            self.state.shutdown.request_shutdown();
            self.state.core.request_stop_deferred();
        } else {
            self.state.request_shutdown();
        }
    }

    /// Advance the original coordinator without moving or discarding owners.
    /// The deadline is a cooperative budget, not preemption. If an unfinished
    /// selected blocking driver is required, request stop and return Pending
    /// without invoking lifecycle callbacks outside that driver.
    pub fn poll_shutdown(&mut self, cx: &mut Context<'_>, deadline: Instant) -> Poll<()> {
        if self.requires_blocking_driver() {
            self.request_shutdown();
            return Poll::Pending;
        }
        self.state.poll_shutdown(cx, deadline)
    }

    /// Perform one caller-driven shutdown pass. Pending selected-driver work
    /// must resume through run; another outside poll does not authorize cleanup.
    pub fn shutdown_until(&mut self, deadline: Instant) -> Completion {
        match self.poll_shutdown(&mut Context::from_waker(Waker::noop()), deadline) {
            Poll::Ready(()) => Completion::Complete,
            Poll::Pending => Completion::Pending,
        }
    }

    /// Advance the same startup/running/shutdown state used by both drive modes.
    /// The first finished activity requests coordinated group shutdown. Work
    /// requiring an unfinished selected blocking driver stays Pending here.
    pub fn poll_run(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.requires_blocking_driver() {
            return Poll::Pending;
        }
        self.state.poll_run(cx)
    }

    /// Borrow this retained owner without creating a runtime or thread.
    /// Cancelling this borrowing future preserves progress in the owner.
    /// The returned failure borrows its original, unmodified application error.
    /// After a selected blocking driver returns early, this future stays Pending
    /// without invoking callbacks. Cancel the borrowing future and resume the
    /// same retained driver through run; no outside executor can replace it.
    pub async fn run_async(&mut self) -> Result<(), &HostFailure<E>> {
        poll_fn(|cx| self.poll_run(cx)).await;
        match self.state.core.failures().primary() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub fn outcomes(&self) -> Vec<HostOutcome> {
        self.state.outcomes()
    }
}

impl<E, P: HostParts<E> + ResourceParts<HostFailure<E>>, D: DriverPlan<E>> BuiltHostGroup<E, P, D> {
    /// Drive the retained owner synchronously on the calling thread.
    ///
    /// All descriptors prepare in Blocking mode before a selected driver may
    /// create its event loop. A plan without a driver uses the ordinary parked
    /// polling loop. Core creates no runtime, worker thread or pool.
    ///
    /// An ongoing Async owner is rejected without altering its work. A selected
    /// driver that returns early remains stored with every unfinished owner.
    /// Core defers stop and records its original error without running affine
    /// cleanup outside that driver. A later run borrows the same driver again;
    /// resumption support is the driver's contract, not an automatic replacement.
    pub fn run(&mut self) -> Result<(), HostRunError<'_, E>> {
        if !matches!(self.state.core.phase(), Phase::Stopped | Phase::Failed) {
            match self.state.run_mode {
                None => self.state.run_mode = Some(HostRunMode::Blocking),
                Some(HostRunMode::Blocking) => {}
                Some(selected) => {
                    return Err(HostRunError::ModeMismatch {
                        selected,
                        requested: HostRunMode::Blocking,
                    });
                }
            }
            let mut cx = Context::from_waker(Waker::noop());
            if !self.state.driver_prepared
                && (self.state.shutdown.is_requested() || !self.state.prepare_driver(&mut cx))
            {
                // Preparation is inert: no driver or dependency was acquired.
                // Unattempted descriptors require only ordinary retirement.
                let _ = self.state.poll_shutdown(&mut cx, Instant::now());
            } else {
                let shutdown = self.state.shutdown.clone();
                let state = &mut self.state;
                let result = self.driver.run(&shutdown, &mut |cx| state.poll_run(cx));
                if let Err(error) = result {
                    self.state.core.fail_deferred(error);
                }
            }
            if !matches!(self.state.core.phase(), Phase::Stopped | Phase::Failed) {
                self.state.shutdown.request_shutdown();
                self.state.core.request_stop_deferred();
                return Err(HostRunError::DriverReturnedEarly {
                    host: self.driver_host.unwrap_or("HostGroup"),
                });
            }
        }
        match self.state.core.failures().primary() {
            Some(error) => Err(HostRunError::HostFailure(error)),
            None => Ok(()),
        }
    }
}
