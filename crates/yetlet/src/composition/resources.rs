//! Typed final-release attachments and structural groups.
//!
//! This module shares the parent lifecycle traversal and failure ledger. Groups
//! do not have independent schedulers or stop/join state machines.

use super::{
    Attached, Builder, Completion, ContextCallback, Failures, JoinStatus, ObserveOp, PartMut,
    PartsMut, ReleaseOp, RequestOp, Worker, WorkerCompletion,
    lifecycle::{Chain, Part},
};
use std::task::Context;
use std::time::{Duration, Instant};

/// Progress of final resource release, distinct from successful application exit.
#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseCompletion<E> {
    /// The original resource and any cleanup operation must remain owned.
    Pending,
    /// No outstanding operation may access the resource. A terminal cleanup
    /// error is retained, but must not cause an already released handle to retry.
    Released(Result<(), E>),
}

/// Latched outcome of final release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseStatus {
    Pending,
    Succeeded,
    Failed,
}

/// A sealed bound for generic code storing crate-built typed parts.
///
/// External Hosts use this bound to compose resource plans; they do not
/// implement it or the private lifecycle traversal. It adds no Send/Sync bound.
#[allow(private_bounds)]
pub trait ResourceParts<E>: Chain<E> {}
impl<E, P: Chain<E>> ResourceParts<E> for P {}

/// An owned dependency whose finalizer runs only after local shutdown and every
/// original worker Join. Complete finalization drops its value immediately.
pub struct Resource<T, R> {
    value: Option<T>,
    release: R,
    label: &'static str,
    status: ReleaseStatus,
}

impl<T, R> Resource<T, R> {
    pub fn label(&self) -> &'static str {
        self.label
    }
    /// None after terminal finalization, including a terminal release failure.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
    pub(super) fn driver_value_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
    pub fn release_status(&self) -> ReleaseStatus {
        self.status
    }
}

/// An existing owned Worker plus an opt-in final-release callback.
///
/// Worker readiness, admission, signal and original Join bookkeeping are
/// delegated unchanged. The coordination value remains stored for inspection.
/// The callback must retire any payload whose destructor needs earlier resources
/// before reporting Released; those dependencies can then be finalized.
pub struct ManagedWorker<T, A, S, O, R> {
    worker: Worker<T, A, S, O>,
    release: R,
    status: ReleaseStatus,
}

impl<T, A, S, O, R> ManagedWorker<T, A, S, O, R> {
    pub fn label(&self) -> &'static str {
        self.worker.label()
    }
    pub fn value(&self) -> &T {
        self.worker.value()
    }
    pub(super) fn driver_value_mut(&mut self) -> &mut T {
        &mut self.worker.value
    }
    pub(super) fn mark_ready_for_driver(&mut self) {
        self.worker.ready = true;
    }
    pub fn is_ready(&self) -> bool {
        self.worker.is_ready()
    }
    pub fn admission_closed(&self) -> bool {
        self.worker.admission_closed()
    }
    pub fn stop_signalled(&self) -> bool {
        self.worker.stop_signalled()
    }
    pub fn join_status(&self) -> JoinStatus {
        self.worker.join_status()
    }
    pub fn release_status(&self) -> ReleaseStatus {
        self.status
    }
}

/// A structural ownership scope inside the same root's traversal and error ledger.
///
/// Later resources/groups may depend on earlier ones. Reverse final release
/// retains earlier dependencies whenever a later release is unfinished.
/// Ordinary Drop destroys contained parts in reverse attachment order without
/// running admission, stop, join or release callbacks.
pub struct Group<P> {
    parts: Option<P>,
    drop_parts: fn(P),
    label: &'static str,
}

impl<P> Group<P> {
    pub fn label(&self) -> &'static str {
        self.label
    }
    pub fn parts(&self) -> &P {
        self.parts.as_ref().expect("owned group parts")
    }
    pub(super) fn parts_mut(&mut self) -> &mut P {
        self.parts.as_mut().expect("owned group parts")
    }
}

impl<P> Drop for Group<P> {
    fn drop(&mut self) {
        if let Some(parts) = self.parts.take() {
            (self.drop_parts)(parts);
        }
    }
}

impl<'a, P> PartMut<'a, Group<P>> {
    pub fn parts_mut(self) -> PartsMut<'a, P> {
        PartsMut {
            parts: self.part.parts_mut(),
            active: self.active,
        }
    }
}

impl<T, A, S, O, R> PartMut<'_, ManagedWorker<T, A, S, O, R>> {
    /// Report actual activity readiness, never just successful acquisition.
    pub fn mark_ready(self) -> bool {
        if !self.active {
            return false;
        }
        self.part.worker.ready = true;
        true
    }
}

/// Typed builder after adding a managed activity.
pub type ActivityBuilder<E, P, T, A, S, O, R> =
    Builder<E, Attached<P, ManagedWorker<T, A, S, O, R>>>;

pub(crate) type PolledActivityBuilder<E, P, T, A, S, O, R> = ActivityBuilder<
    E,
    P,
    T,
    ContextCallback<A>,
    ContextCallback<S>,
    ContextCallback<O>,
    ContextCallback<R>,
>;

impl<E, P> Builder<E, P> {
    /// Retain a supplied resource until final release after all original Joins.
    ///
    /// This does not acquire or start the value. A caller can supply a ready
    /// resource or acquire it later while the root retains its descriptor. Actual
    /// dependent activity readiness must follow successful acquisition.
    /// Err/Pending retain the value and original cleanup owner. Released latches
    /// the outcome and destroys the value before releasing earlier dependencies.
    /// Dropping an unfinished Builder/root does not run this callback.
    pub fn with_resource<T, R>(
        self,
        label: &'static str,
        value: T,
        release: R,
    ) -> Builder<E, Attached<P, Resource<T, R>>>
    where
        R: FnMut(&mut T, Duration) -> Result<ReleaseCompletion<E>, E>,
    {
        self.with_resource_op(label, value, release)
    }

    pub(crate) fn with_polled_resource<T, R>(
        self,
        label: &'static str,
        value: T,
        release: R,
    ) -> Builder<E, Attached<P, Resource<T, ContextCallback<R>>>>
    where
        R: FnMut(&mut T, &mut Context<'_>, Duration) -> Result<ReleaseCompletion<E>, E>,
    {
        self.with_resource_op(label, value, ContextCallback { callback: release })
    }

    fn with_resource_op<T, R>(
        self,
        label: &'static str,
        value: T,
        release: R,
    ) -> Builder<E, Attached<P, Resource<T, R>>>
    where
        R: ReleaseOp<T, E>,
    {
        self.push(Resource {
            value: Some(value),
            release,
            label,
            status: ReleaseStatus::Pending,
        })
    }

    /// Attach an original activity and its post-Join final-release operation.
    ///
    /// No callback runs here. As with required_worker, actual readiness must be
    /// reported after build. The final callback cannot run before every original
    /// worker in the entire root has joined and legacy local cleanup completed.
    pub fn with_activity<T, A, S, O, R>(
        self,
        label: &'static str,
        value: T,
        admission: A,
        signal: S,
        observe: O,
        release: R,
    ) -> ActivityBuilder<E, P, T, A, S, O, R>
    where
        A: FnMut(&mut T) -> Result<(), E>,
        S: FnMut(&mut T) -> Result<(), E>,
        O: FnMut(&mut T, Duration) -> Result<WorkerCompletion<E>, E>,
        R: FnMut(&mut T, Duration) -> Result<ReleaseCompletion<E>, E>,
    {
        self.with_activity_ops(label, value, admission, signal, observe, release)
    }

    pub(crate) fn with_polled_activity<T, A, S, O, R>(
        self,
        label: &'static str,
        value: T,
        admission: A,
        signal: S,
        observe: O,
        release: R,
    ) -> PolledActivityBuilder<E, P, T, A, S, O, R>
    where
        A: FnMut(&mut T, &mut Context<'_>) -> Result<(), E>,
        S: FnMut(&mut T, &mut Context<'_>) -> Result<(), E>,
        O: FnMut(&mut T, &mut Context<'_>, Duration) -> Result<WorkerCompletion<E>, E>,
        R: FnMut(&mut T, &mut Context<'_>, Duration) -> Result<ReleaseCompletion<E>, E>,
    {
        self.with_activity_ops(
            label,
            value,
            ContextCallback {
                callback: admission,
            },
            ContextCallback { callback: signal },
            ContextCallback { callback: observe },
            ContextCallback { callback: release },
        )
    }

    fn with_activity_ops<T, A, S, O, R>(
        self,
        label: &'static str,
        value: T,
        admission: A,
        signal: S,
        observe: O,
        release: R,
    ) -> ActivityBuilder<E, P, T, A, S, O, R>
    where
        A: RequestOp<T, E>,
        S: RequestOp<T, E>,
        O: ObserveOp<T, E>,
        R: ReleaseOp<T, E>,
    {
        self.push(ManagedWorker {
            worker: Worker {
                value,
                admission,
                signal,
                observe,
                label,
                ready: false,
                admission_closed: false,
                stop_signalled: false,
                join_status: JoinStatus::Pending,
            },
            release,
            status: ReleaseStatus::Pending,
        })
    }

    /// Nest typed parts without creating another coordinator or failure ledger.
    pub fn with_group<Q: ResourceParts<E>>(
        self,
        label: &'static str,
        group: Builder<E, Q>,
    ) -> Builder<E, Attached<P, Group<Q>>> {
        self.push(Group {
            parts: Some(group.root.parts),
            drop_parts: <Q as Chain<E>>::drop_parts_reverse,
            label,
        })
    }
}

fn advance_release<T, E, R>(
    value: &mut T,
    release: &mut R,
    status: &mut ReleaseStatus,
    cx: &mut Context<'_>,
    deadline: Instant,
    failures: &mut Failures<E>,
) -> Completion
where
    R: ReleaseOp<T, E>,
{
    if *status != ReleaseStatus::Pending {
        return Completion::Complete;
    }
    match release.release(
        value,
        cx,
        deadline.saturating_duration_since(Instant::now()),
    ) {
        Ok(ReleaseCompletion::Pending) => Completion::Pending,
        Err(error) => {
            failures.record(error);
            Completion::Pending
        }
        Ok(ReleaseCompletion::Released(result)) => {
            *status = match result {
                Ok(()) => ReleaseStatus::Succeeded,
                Err(error) => {
                    failures.record(error);
                    ReleaseStatus::Failed
                }
            };
            Completion::Complete
        }
    }
}

impl<E, T, R> Part<E> for Resource<T, R>
where
    R: ReleaseOp<T, E>,
{
    fn ready(&self) -> bool {
        self.status == ReleaseStatus::Pending
    }

    fn release(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<E>,
    ) -> Completion {
        let Some(value) = self.value.as_mut() else {
            return Completion::Complete;
        };
        let result = advance_release(
            value,
            &mut self.release,
            &mut self.status,
            cx,
            deadline,
            failures,
        );
        if result == Completion::Complete {
            drop(self.value.take());
        }
        result
    }
}

impl<E, T, A, S, O, R> Part<E> for ManagedWorker<T, A, S, O, R>
where
    A: RequestOp<T, E>,
    S: RequestOp<T, E>,
    O: ObserveOp<T, E>,
    R: ReleaseOp<T, E>,
{
    fn ready(&self) -> bool {
        Part::ready(&self.worker)
    }
    fn close_admission(&mut self, cx: &mut Context<'_>, failures: &mut Failures<E>) -> bool {
        Part::close_admission(&mut self.worker, cx, failures)
    }
    fn close_local(&mut self, failures: &mut Failures<E>) -> bool {
        Part::close_local(&mut self.worker, failures)
    }
    fn signal(&mut self, cx: &mut Context<'_>, failures: &mut Failures<E>) -> bool {
        Part::signal(&mut self.worker, cx, failures)
    }
    fn observe(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<E>,
    ) -> Completion {
        Part::observe(&mut self.worker, cx, deadline, failures)
    }
    fn release(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<E>,
    ) -> Completion {
        advance_release(
            &mut self.worker.value,
            &mut self.release,
            &mut self.status,
            cx,
            deadline,
            failures,
        )
    }
}

impl<E, P: Chain<E>> Part<E> for Group<P> {
    fn ready(&self) -> bool {
        self.parts().all_ready()
    }
    fn close_admission(&mut self, cx: &mut Context<'_>, failures: &mut Failures<E>) -> bool {
        self.parts_mut().close_admission(cx, failures)
    }
    fn close_local(&mut self, failures: &mut Failures<E>) -> bool {
        self.parts_mut().close_locals(failures)
    }
    fn signal(&mut self, cx: &mut Context<'_>, failures: &mut Failures<E>) -> bool {
        self.parts_mut().signal_workers(cx, failures)
    }
    fn observe(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<E>,
    ) -> Completion {
        self.parts_mut().observe_owned(cx, deadline, failures)
    }
    fn release(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<E>,
    ) -> Completion {
        self.parts_mut().release_owned(cx, deadline, failures)
    }
}
