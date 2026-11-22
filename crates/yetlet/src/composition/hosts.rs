//! Public Host descriptions and a caller-driven lifetime owner.
//!
//! Built-in adapters and custom Hosts return the same typed ResourceGroup.
//! Scheduling belongs to the caller; shutdown uses the Composition coordinator.

use super::{
    Attached, Builder, Completion, ContextCallback, Empty, Failures, Group, JoinStatus,
    ManagedWorker, ReleaseCompletion, ReleaseStatus, ResourceParts, WorkerCompletion,
    lifecycle::Part,
};
use std::{
    error::Error,
    fmt,
    marker::PhantomData,
    task::{Context, Poll, ready},
    time::{Duration, Instant},
};

mod driver;
mod resources;
pub use driver::{BuiltHostGroup, HostRunError, HostRunMode, ShutdownHandle, StartStatus};
pub use resources::{Managed, ManagedResource, ResourcePlan, Retained};

/// A custom or built-in Host is an inert description of one activity and its
/// dependencies. Conversion and validation must not acquire external resources.
///
/// A group uses one application-selected error type. Custom adapters can map
/// their concrete errors into that type; no boxing, Clone, Send, Sync or 'static
/// bound is imposed. Hosts never implement the private lifecycle traversal.
pub trait Host {
    type Error;
    type Activity: HostActivity<Error = Self::Error>;
    type Resources: ResourcePlan<Self::Error>;

    /// Stable diagnostic name; this is not a service-registry lookup key.
    fn name(&self) -> &'static str;

    /// Validate configuration only. All Hosts validate before any conversion.
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Known exclusive plan claims, such as a particular event-loop driver.
    /// Actual acquisition must still detect resources owned outside this plan.
    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        Vec::new()
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources>;
}

/// The result of synchronously opening an activity's ordinary admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionOutcome {
    /// Ordinary work may be admitted.
    Opened,
    /// Admission is permanently closed. Stop the group and observe the retained
    /// original completion through [HostActivity::poll_join] before release.
    Closed,
}

/// Nonblocking activity progress in an application-owned execution environment.
///
/// Before returning Pending or an error, retain partial acquisition, in-flight
/// operations, original completion handles and terminal outcomes in self.
/// Pinned futures may be stored internally; no blanket Unpin bound is required.
/// A Pending poll must arrange a wake when it can progress, or the caller must
/// explicitly supply event-driven polls. Core provides neither a timer nor an
/// executor. Panics are not caught.
///
/// Admission fences and stop signals are short synchronous requests. Draining
/// admitted work and observing original completion belong in poll_join. An
/// asynchronous fence that cannot satisfy this contract needs a different
/// adapter contract; returning Ok before the fence exists is incorrect. A
/// retryable request error must register a wake through the supplied Context
/// when progress becomes possible, or require an explicitly caller-driven retry.
pub trait HostActivity {
    type Error;

    /// Validate/configure placement without acquiring resources. The complete
    /// group prepares once before any dependency or activity starts. Retain no
    /// obligation that requires asynchronous cleanup from this inert hook.
    fn prepare_driver(&mut self, _mode: HostRunMode) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Acquire/start while retaining even partial owners before returning.
    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Observe actual usability, not just allocation/spawn/unpolled work.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Termination notification only. Retain the actual outcome for poll_join.
    /// A passive service may wait for the group's explicit shutdown request.
    fn poll_finished(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    /// Enable ordinary admission only after every Host has observed readiness.
    ///
    /// If termination has already closed the gate, return [AdmissionOutcome::Closed]
    /// without replacing the retained original execution outcome with an admission
    /// error. Core stops the group and observes that outcome through poll_join.
    fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
        Ok(AdmissionOutcome::Opened)
    }

    /// Fence new work before any group activity is stopped.
    fn close_admission(&mut self, _cx: &mut Context<'_>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Deliver a bounded stop request, independent of ordinary work queues.
    fn request_shutdown(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error>;

    /// Pending/Err retain the original owner; Err is a retryable observation
    /// failure, while Joined(Err) is an actual terminal execution failure.
    /// Register wake interest for unfinished progress. Budget is cooperative;
    /// even a zero budget may perform a nonblocking observation.
    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error>;

    /// Final activity cleanup after every original group completion. Retire all
    /// dependency-using payloads before Released. Core then drops this activity
    /// before its earlier resources. Err is retryable; Released(Err) is terminal.
    fn poll_release(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

/// An exact, caller-described exclusive claim within an inert plan.
///
/// Equal kind and key values conflict. This does not reserve OS resources,
/// canonicalize addresses, detect aliases or prove process-wide availability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusiveClaim {
    pub kind: &'static str,
    pub key: String,
}

impl ExclusiveClaim {
    pub fn new(kind: &'static str, key: impl Into<String>) -> Self {
        Self {
            kind,
            key: key.into(),
        }
    }
}

/// The operation that supplied an original failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOperation {
    Validate,
    PrepareDriver,
    Acquire,
    Start,
    Readiness,
    OpenAdmission,
    CloseAdmission,
    Shutdown,
    Observe,
    Join,
    Release,
    ReleaseProgress,
}

/// An owned, unmodified application error with its lifecycle context.
#[derive(Debug)]
pub struct HostFailure<E> {
    pub host: &'static str,
    pub resource: Option<&'static str>,
    pub operation: HostOperation,
    error: E,
}

impl<E> HostFailure<E> {
    fn new(host: &'static str, operation: HostOperation, error: E) -> Self {
        Self {
            host,
            resource: None,
            operation,
            error,
        }
    }

    fn resource(
        host: &'static str,
        resource: &'static str,
        operation: HostOperation,
        error: E,
    ) -> Self {
        Self {
            host,
            resource: Some(resource),
            operation,
            error,
        }
    }

    pub fn error(&self) -> &E {
        &self.error
    }

    pub fn into_error(self) -> E {
        self.error
    }
}

impl<E: fmt::Display> fmt::Display for HostFailure<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}: {}", self.host, self.operation, self.error)
    }
}

impl<E: Error + 'static> Error for HostFailure<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// A rejected inert plan. No Host acquisition has run.
#[derive(Debug)]
pub enum HostBuildError<E> {
    InvalidHost(HostFailure<E>),
    ConflictingClaim {
        first: &'static str,
        second: &'static str,
        claim: ExclusiveClaim,
    },
}

impl<E: fmt::Display> fmt::Display for HostBuildError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHost(error) => fmt::Display::fmt(error, f),
            Self::ConflictingClaim {
                first,
                second,
                claim,
            } => {
                write!(
                    f,
                    "{first} and {second} both claim {} {}",
                    claim.kind, claim.key
                )
            }
        }
    }
}

impl<E: Error + 'static> Error for HostBuildError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidHost(error) => Some(error),
            Self::ConflictingClaim { .. } => None,
        }
    }
}

/// One activity and ordered typed dependencies, with no resource acquisition.
///
/// Later dependencies may use earlier ones. Every activity is joined before
/// final release; release then proceeds in reverse registration order. A live
/// asynchronous owner requiring cleanup must not be adopted as a passive value.
#[must_use = "Install the resource plan in a HostGroup before acquiring resources"]
pub struct ResourceGroup<A, D = ()> {
    activity: A,
    resources: D,
}

impl<A: HostActivity> ResourceGroup<A> {
    pub fn new(activity: A) -> Self {
        Self {
            activity,
            resources: (),
        }
    }
}

impl<A, D> ResourceGroup<A, D> {
    /// Transform the inert activity while preserving all dependency storage.
    ///
    /// Custom Hosts can wrap an adapter to map its errors into their common
    /// application error. Existing managed descriptors must still use the new
    /// activity's error type; this method does not transform resource errors.
    pub fn map_activity<B: HostActivity>(self, map: impl FnOnce(A) -> B) -> ResourceGroup<B, D> {
        ResourceGroup {
            activity: map(self.activity),
            resources: self.resources,
        }
    }

    /// Retain an ordinary owned value or borrow through final release. This is
    /// normal Rust ownership, not an automatically managed runtime/Join handle.
    pub fn with_resource<R>(self, value: R) -> ResourceGroup<A, Retained<R, D>> {
        ResourceGroup {
            activity: self.activity,
            resources: Retained {
                value,
                previous: self.resources,
            },
        }
    }

    /// Register an inert descriptor. Acquire runs only after the entire plan
    /// validates and every original descriptor is installed in the retained root.
    pub fn with_managed_resource<R>(
        self,
        name: &'static str,
        resource: R,
    ) -> ResourceGroup<A, Managed<R, D>>
    where
        A: HostActivity,
        R: ManagedResource<Error = A::Error>,
    {
        ResourceGroup {
            activity: self.activity,
            resources: Managed {
                resource,
                previous: self.resources,
                name,
            },
        }
    }
}

/// Build a statically typed group of Host lifetimes.
///
/// No runtime, thread, socket or event loop is created by new/with_host/build.
/// There is no fixed Host-count limit. The inferred recursive storage type keeps
/// local and borrowed values without type erasure.
///
/// See [BuiltHostGroup] for polling, explicit shutdown and cancellation.
#[must_use = "Build this inert plan and explicitly drive its lifetime"]
pub struct HostGroup<E, H = Empty> {
    hosts: H,
    shutdown: ShutdownHandle,
    error: PhantomData<fn() -> E>,
}

impl<E> HostGroup<E, Empty> {
    pub fn new() -> Self {
        Self {
            hosts: Empty,
            shutdown: ShutdownHandle::default(),
            error: PhantomData,
        }
    }
}

impl<E> Default for HostGroup<E, Empty> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, P> HostGroup<E, P> {
    pub fn with_host<H: Host<Error = E>>(self, host: H) -> HostGroup<E, HostListNode<P, H>> {
        HostGroup {
            hosts: HostListNode {
                host,
                previous: self.hosts,
            },
            shutdown: self.shutdown,
            error: PhantomData,
        }
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.shutdown.clone()
    }
}

impl<E, H: HostList<E>> HostGroup<E, H> {
    /// Validate the complete plan, then install descriptors in the shared engine.
    /// Build errors have no acquired obligations to dispose. Ordinary supplied
    /// RAII values still follow their normal destruction/lifetime rules.
    pub fn build(self) -> Result<BuiltHostGroup<E, H::Parts>, HostBuildError<E>> {
        self.hosts.validate(&mut Vec::new())?;
        Ok(BuiltHostGroup::new(
            self.hosts.into_builder().build(),
            self.shutdown,
        ))
    }
}

/// Read-only activity outcome; Complete does not imply successful execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostOutcome {
    pub host: &'static str,
    pub start_attempted: bool,
    pub join: JoinStatus,
    pub release: ReleaseStatus,
}

mod sealed {
    pub trait List {}
    pub trait Parts {}
}

/// Inferred recursive Host storage; use HostGroup::with_host.
#[doc(hidden)]
pub struct HostListNode<P, H> {
    host: H,
    previous: P,
}

/// Sealed type-level support for the public builder, not another extension trait.
#[doc(hidden)]
pub trait HostList<E>: sealed::List {
    type Parts: HostParts<E> + ResourceParts<HostFailure<E>>;
    fn validate(
        &self,
        claims: &mut Vec<(&'static str, ExclusiveClaim)>,
    ) -> Result<(), HostBuildError<E>>;
    fn into_builder(self) -> Builder<HostFailure<E>, Self::Parts>;
}

impl sealed::List for Empty {}
impl<E> HostList<E> for Empty {
    type Parts = Empty;

    fn validate(
        &self,
        _: &mut Vec<(&'static str, ExclusiveClaim)>,
    ) -> Result<(), HostBuildError<E>> {
        Ok(())
    }

    fn into_builder(self) -> Builder<HostFailure<E>> {
        Builder::new()
    }
}

impl<P, H> sealed::List for HostListNode<P, H> {}
impl<E, P: HostList<E>, H: Host<Error = E>> HostList<E> for HostListNode<P, H> {
    type Parts = Attached<P::Parts, Hosted<H::Activity, H::Resources>>;

    fn validate(
        &self,
        claims: &mut Vec<(&'static str, ExclusiveClaim)>,
    ) -> Result<(), HostBuildError<E>> {
        self.previous.validate(claims)?;
        let name = self.host.name();
        self.host.validate().map_err(|error| {
            HostBuildError::InvalidHost(HostFailure::new(name, HostOperation::Validate, error))
        })?;
        for claim in self.host.exclusive_claims() {
            if let Some((first, _)) = claims.iter().find(|(_, existing)| *existing == claim) {
                return Err(HostBuildError::ConflictingClaim {
                    first,
                    second: name,
                    claim,
                });
            }
            claims.push((name, claim));
        }
        Ok(())
    }

    fn into_builder(self) -> Builder<HostFailure<E>, Self::Parts> {
        let previous = self.previous.into_builder();
        let name = self.host.name();
        let ResourceGroup {
            activity,
            resources,
        } = self.host.into_resources();
        let parts = resources.into_builder(name).with_polled_activity(
            name,
            ActivitySlot {
                activity: Some(activity),
                host: name,
                attempted: false,
                started: false,
            },
            ActivitySlot::<H::Activity>::close
                as fn(&mut ActivitySlot<H::Activity>, &mut Context<'_>) -> _,
            ActivitySlot::<H::Activity>::signal
                as fn(&mut ActivitySlot<H::Activity>, &mut Context<'_>) -> _,
            ActivitySlot::<H::Activity>::join
                as fn(&mut ActivitySlot<H::Activity>, &mut Context<'_>, Duration) -> _,
            ActivitySlot::<H::Activity>::release
                as fn(&mut ActivitySlot<H::Activity>, &mut Context<'_>, Duration) -> _,
        );
        let group = Builder::<HostFailure<E>>::new()
            .with_group(name, parts)
            .build()
            .parts
            .last;
        previous.push(Hosted {
            group,
            resources: PhantomData,
        })
    }
}

/// Inferred original activity storage. No mutable public replacement operation.
#[doc(hidden)]
pub struct ActivitySlot<A> {
    activity: Option<A>,
    host: &'static str,
    attempted: bool,
    started: bool,
}

impl<A: HostActivity> ActivitySlot<A> {
    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), HostFailure<A::Error>> {
        self.activity
            .as_mut()
            .expect("activity retained before acquisition")
            .prepare_driver(mode)
            .map_err(|e| HostFailure::new(self.host, HostOperation::PrepareDriver, e))
    }

    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), HostFailure<A::Error>>> {
        self.attempted = true;
        let activity = self
            .activity
            .as_mut()
            .expect("activity retained before final release");
        if !self.started {
            ready!(activity.poll_start(cx))
                .map_err(|e| HostFailure::new(self.host, HostOperation::Start, e))?;
            self.started = true;
        }
        activity
            .poll_ready(cx)
            .map_err(|e| HostFailure::new(self.host, HostOperation::Readiness, e))
    }

    fn open(&mut self) -> Result<AdmissionOutcome, HostFailure<A::Error>> {
        self.activity
            .as_mut()
            .expect("ready activity")
            .open_admission()
            .map_err(|e| HostFailure::new(self.host, HostOperation::OpenAdmission, e))
    }

    fn close(&mut self, cx: &mut Context<'_>) -> Result<(), HostFailure<A::Error>> {
        if !self.attempted {
            return Ok(());
        }
        self.activity
            .as_mut()
            .expect("original activity")
            .close_admission(cx)
            .map_err(|e| HostFailure::new(self.host, HostOperation::CloseAdmission, e))
    }

    fn signal(&mut self, cx: &mut Context<'_>) -> Result<(), HostFailure<A::Error>> {
        if !self.attempted {
            return Ok(());
        }
        self.activity
            .as_mut()
            .expect("original activity")
            .request_shutdown(cx)
            .map_err(|e| HostFailure::new(self.host, HostOperation::Shutdown, e))
    }

    fn join(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<HostFailure<A::Error>>, HostFailure<A::Error>> {
        if !self.attempted {
            return Ok(WorkerCompletion::Joined(Ok(())));
        }
        match self
            .activity
            .as_mut()
            .expect("original activity")
            .poll_join(cx, budget)
        {
            Ok(WorkerCompletion::Pending) => Ok(WorkerCompletion::Pending),
            Ok(WorkerCompletion::Joined(result)) => {
                Ok(WorkerCompletion::Joined(result.map_err(|e| {
                    HostFailure::new(self.host, HostOperation::Join, e)
                })))
            }
            Err(e) => Err(HostFailure::new(self.host, HostOperation::Observe, e)),
        }
    }

    fn release(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<HostFailure<A::Error>>, HostFailure<A::Error>> {
        let completion = if self.attempted {
            self.activity
                .as_mut()
                .expect("activity retained until release")
                .poll_release(cx, budget)
        } else {
            Ok(ReleaseCompletion::Released(Ok(())))
        };
        match completion {
            Ok(ReleaseCompletion::Pending) => Ok(ReleaseCompletion::Pending),
            Ok(ReleaseCompletion::Released(result)) => {
                drop(self.activity.take());
                Ok(ReleaseCompletion::Released(result.map_err(|e| {
                    HostFailure::new(self.host, HostOperation::Release, e)
                })))
            }
            Err(e) => Err(HostFailure::new(
                self.host,
                HostOperation::ReleaseProgress,
                e,
            )),
        }
    }
}

#[doc(hidden)]
pub type HostActivityNode<A> = ManagedWorker<
    ActivitySlot<A>,
    ContextCallback<
        fn(
            &mut ActivitySlot<A>,
            &mut Context<'_>,
        ) -> Result<(), HostFailure<<A as HostActivity>::Error>>,
    >,
    ContextCallback<
        fn(
            &mut ActivitySlot<A>,
            &mut Context<'_>,
        ) -> Result<(), HostFailure<<A as HostActivity>::Error>>,
    >,
    ContextCallback<
        fn(
            &mut ActivitySlot<A>,
            &mut Context<'_>,
            Duration,
        ) -> Result<
            WorkerCompletion<HostFailure<<A as HostActivity>::Error>>,
            HostFailure<<A as HostActivity>::Error>,
        >,
    >,
    ContextCallback<
        fn(
            &mut ActivitySlot<A>,
            &mut Context<'_>,
            Duration,
        ) -> Result<
            ReleaseCompletion<HostFailure<<A as HostActivity>::Error>>,
            HostFailure<<A as HostActivity>::Error>,
        >,
    >,
>;

/// Structural wrapper only; every shutdown latch delegates to the same Group.
#[doc(hidden)]
pub struct Hosted<A: HostActivity, R: ResourcePlan<A::Error>> {
    group: Group<Attached<R::Parts, HostActivityNode<A>>>,
    resources: PhantomData<fn() -> R>,
}

impl<E, A: HostActivity<Error = E>, R: ResourcePlan<E>> Part<HostFailure<E>> for Hosted<A, R> {
    fn ready(&self) -> bool {
        self.group.ready()
    }
    fn close_admission(
        &mut self,
        cx: &mut Context<'_>,
        failures: &mut Failures<HostFailure<E>>,
    ) -> bool {
        self.group.close_admission(cx, failures)
    }
    fn close_local(&mut self, failures: &mut Failures<HostFailure<E>>) -> bool {
        self.group.close_local(failures)
    }
    fn signal(&mut self, cx: &mut Context<'_>, failures: &mut Failures<HostFailure<E>>) -> bool {
        self.group.signal(cx, failures)
    }
    fn observe(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<HostFailure<E>>,
    ) -> Completion {
        self.group.observe(cx, deadline, failures)
    }
    fn release(
        &mut self,
        cx: &mut Context<'_>,
        deadline: Instant,
        failures: &mut Failures<HostFailure<E>>,
    ) -> Completion {
        self.group.release(cx, deadline, failures)
    }
}

/// Sealed driver traversal for crate-built typed storage. It owns no shutdown
/// latches; activity and resource completion stay in Composition.
#[doc(hidden)]
pub trait HostParts<E>: sealed::Parts {
    const COUNT: usize;
    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), HostFailure<E>>;
    fn poll_start(
        &mut self,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>>;
    fn open(&mut self, shutdown: &ShutdownHandle) -> Result<AdmissionOutcome, HostFailure<E>>;
    fn poll_finished(&mut self, cx: &mut Context<'_>) -> bool;
    fn outcomes(&self, result: &mut Vec<HostOutcome>);
}

impl sealed::Parts for Empty {}
impl<E> HostParts<E> for Empty {
    const COUNT: usize = 0;
    fn prepare_driver(&mut self, _: HostRunMode) -> Result<(), HostFailure<E>> {
        Ok(())
    }
    fn poll_start(
        &mut self,
        _: &mut Context<'_>,
        _: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        Poll::Ready(Ok(()))
    }
    fn open(&mut self, _: &ShutdownHandle) -> Result<AdmissionOutcome, HostFailure<E>> {
        Ok(AdmissionOutcome::Opened)
    }
    fn poll_finished(&mut self, _: &mut Context<'_>) -> bool {
        false
    }
    fn outcomes(&self, _: &mut Vec<HostOutcome>) {}
}

impl<P, A: HostActivity, R: ResourcePlan<A::Error>> sealed::Parts for Attached<P, Hosted<A, R>> {}
impl<E, P: HostParts<E>, A: HostActivity<Error = E>, R: ResourcePlan<E>> HostParts<E>
    for Attached<P, Hosted<A, R>>
{
    const COUNT: usize = P::COUNT + 1;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), HostFailure<E>> {
        self.previous.prepare_driver(mode)?;
        let parts = self.last.group.parts_mut();
        R::prepare_driver(&mut parts.previous, mode)?;
        parts.last.driver_value_mut().prepare_driver(mode)
    }

    fn poll_start(
        &mut self,
        cx: &mut Context<'_>,
        shutdown: &ShutdownHandle,
    ) -> Poll<Result<(), HostFailure<E>>> {
        ready!(self.previous.poll_start(cx, shutdown))?;
        if shutdown.is_requested() {
            return Poll::Ready(Ok(()));
        }
        let parts = self.last.group.parts_mut();
        ready!(R::poll_acquire(&mut parts.previous, cx, shutdown))?;
        if shutdown.is_requested() {
            return Poll::Ready(Ok(()));
        }
        if !parts.last.is_ready() {
            ready!(parts.last.driver_value_mut().poll_start(cx))?;
            parts.last.mark_ready_for_driver();
        }
        Poll::Ready(Ok(()))
    }

    fn open(&mut self, shutdown: &ShutdownHandle) -> Result<AdmissionOutcome, HostFailure<E>> {
        if self.previous.open(shutdown)? == AdmissionOutcome::Closed || shutdown.is_requested() {
            return Ok(AdmissionOutcome::Closed);
        }
        self.last.group.parts_mut().last.driver_value_mut().open()
    }

    fn poll_finished(&mut self, cx: &mut Context<'_>) -> bool {
        let previous = self.previous.poll_finished(cx);
        let slot = self.last.group.parts_mut().last.driver_value_mut();
        let last = slot.started
            && slot
                .activity
                .as_mut()
                .expect("started activity")
                .poll_finished(cx)
                .is_ready();
        previous || last
    }

    fn outcomes(&self, result: &mut Vec<HostOutcome>) {
        self.previous.outcomes(result);
        let node = &self.last.group.parts().last;
        result.push(HostOutcome {
            host: self.last.group.label(),
            start_attempted: node.value().attempted,
            join: node.join_status(),
            release: node.release_status(),
        });
    }
}
