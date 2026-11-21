use super::{HostFailure, HostOutcome, HostParts};
use crate::composition::{Completion, Composition, Failures, Phase, ResourceParts};
use std::{
    future::poll_fn,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Waker},
    time::Instant,
};

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
pub struct BuiltHostGroup<E, P> {
    core: Composition<HostFailure<E>, P>,
    shutdown: ShutdownHandle,
}

impl<E, P> Drop for BuiltHostGroup<E, P> {
    fn drop(&mut self) {
        // A caller's waker can own a ShutdownHandle back to this registration.
        // Break that cycle before ordinary field destruction; clearing the
        // registration invokes no lifecycle callback and drops outside its lock.
        self.shutdown.clear_waker();
    }
}

impl<E, P> BuiltHostGroup<E, P> {
    pub(super) fn new(core: Composition<HostFailure<E>, P>, shutdown: ShutdownHandle) -> Self {
        Self { core, shutdown }
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

impl<E, P: HostParts<E> + ResourceParts<HostFailure<E>>> BuiltHostGroup<E, P> {
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
        if let Err(error) = self.core.parts.open(&self.shutdown) {
            self.core.fail_with_context(error, cx);
            return Poll::Ready(StartStatus::Stopping);
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

    /// One caller-driven shutdown pass. Event-loop users arrange later progress;
    /// async users use poll_shutdown so adapters receive a real wake context.
    pub fn shutdown_until(&mut self, deadline: Instant) -> Completion {
        match self.poll_shutdown(&mut Context::from_waker(Waker::noop()), deadline) {
            Poll::Ready(()) => Completion::Complete,
            Poll::Pending => Completion::Pending,
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

    /// Borrowing, executor-neutral drive. Only terminal cleanup returns a
    /// result. The first original failure is borrowed; all later failures stay
    /// available through failures(). No error cloning/string conversion occurs.
    ///
    /// This Core extension proof is not qualification of any Web/Tokio/native
    /// adapter. Those adapters must obey the nonblocking/wake/retained-owner
    /// contract through acquisition, readiness, running, Join and release.
    pub async fn run_async(&mut self) -> Result<(), &HostFailure<E>> {
        poll_fn(|cx| self.poll_run(cx)).await;
        match self.core.failures().primary() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub fn outcomes(&self) -> Vec<HostOutcome> {
        let mut result = Vec::with_capacity(P::COUNT);
        self.core.parts.outcomes(&mut result);
        result
    }
}
