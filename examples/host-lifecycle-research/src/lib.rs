//! Unpublished YT-286-A/B ownership proof, not a supported SDK facade.
//!
//! One to four Hosts, one activity per Host, and typed managed/RAII resources.
//! Every Host uses the existing Composition engine. No global Send, Sync or
//! 'static bound is imposed. See the package README for the deliberately bounded
//! driver, acquisition, error and Drop contracts.

use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt, io,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use yetlet::composition::{
    Attached, Builder, Composition, Empty, Group, JoinStatus, ManagedWorker,
};

pub use yetlet::composition::{Completion, ReleaseCompletion, WorkerCompletion};
mod resources;
pub use resources::{Managed, ManagedResource, ResourcePlan, Retained};
mod web;
pub use web::{WebActivity, WebHost, WebStatus};

/// A caller-driven descriptor that retains every acquired owner, including
/// partial startup. Methods must also tolerate a failed start. No callback may
/// panic. Readiness must reflect usable service, not merely a successful spawn.
pub trait Activity {
    fn start(&mut self) -> io::Result<()>;
    fn wait_ready(&mut self) -> io::Result<()>;
    fn is_finished(&self) -> bool;
    /// Called after every Host passed readiness. Passive activities may do nothing.
    fn open_admission(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn close_admission(&mut self) -> io::Result<()>;
    fn request_shutdown(&mut self) -> io::Result<()>;
    /// Err is a retryable observation failure. Joined(Err) is terminal failure.
    /// The original activity owner must remain here on Pending or Err.
    fn observe_within(&mut self, budget: Duration) -> io::Result<WorkerCompletion<io::Error>>;
}

/// Inert Host descriptions produce the same typed ResourceGroup used by WebHost.
/// Validation and conversion must not acquire external resources.
pub trait Host {
    type Activity: Activity;
    type Resources: ResourcePlan;

    fn validate(&self) -> io::Result<()> {
        Ok(())
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources>;
}

/// One activity and its ordered resource dependencies. After all application
/// activities join, this activity is destroyed before its resource finalizers.
/// Later Pending or retryable failure blocks earlier resource release.
pub struct ResourceGroup<A, D = ()> {
    pub(crate) activity: A,
    resources: D,
    label: &'static str,
}

impl<A: Activity> ResourceGroup<A> {
    pub fn new(label: &'static str, activity: A) -> Self {
        Self {
            activity,
            resources: (),
            label,
        }
    }
}

impl<A, D> ResourceGroup<A, D> {
    /// Add an inert descriptor. Acquisition is deferred until it is in the core.
    /// General adoption of an already-live asynchronous owner is not supported.
    pub fn with_managed_resource<R: ManagedResource>(
        self,
        label: &'static str,
        resource: R,
    ) -> ResourceGroup<A, Managed<R, D>> {
        ResourceGroup {
            activity: self.activity,
            resources: Managed::new(label, resource, self.resources),
            label: self.label,
        }
    }

    /// Add an already supplied RAII value; existing obligations are preserved.
    /// Use with_managed_resource for inert descriptors with explicit finalization.
    pub fn with_resource<R>(self, value: R) -> ResourceGroup<A, Retained<R, D>> {
        ResourceGroup {
            activity: self.activity,
            resources: Retained {
                _value: value,
                _previous: self.resources,
            },
            label: self.label,
        }
    }
}

/// Error evidence is shared, not stringified: the original io::Error survives
/// destruction of the terminal lifecycle root.
#[derive(Clone, Debug)]
pub struct Failure {
    pub host: &'static str,
    pub operation: &'static str,
    pub resource: Option<&'static str>,
    source: Rc<io::Error>,
}

impl Failure {
    fn new(host: &'static str, operation: &'static str, source: io::Error) -> Self {
        Self {
            host,
            operation,
            resource: None,
            source: Rc::new(source),
        }
    }

    fn resource(
        host: &'static str,
        resource: &'static str,
        operation: &'static str,
        source: io::Error,
    ) -> Self {
        Self {
            host,
            operation,
            resource: Some(resource),
            source: Rc::new(source),
        }
    }

    pub fn error(&self) -> &io::Error {
        &self.source
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(resource) = self.resource {
            write!(
                f,
                "{} resource {} {}: {}",
                self.host, resource, self.operation, self.source
            )
        } else {
            write!(f, "{} {}: {}", self.host, self.operation, self.source)
        }
    }
}

impl Error for Failure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub struct HostOutcome {
    pub host: &'static str,
    pub start_attempted: bool,
    pub join: JoinStatus,
}

/// Available only after the owned root and its passive resources were released.
#[derive(Debug)]
pub struct RunReport {
    pub hosts: Vec<HostOutcome>,
    pub primary: Option<Failure>,
    pub secondary: Vec<Failure>,
}

#[derive(Debug)]
pub struct RunFailure {
    pub report: RunReport,
}

impl fmt::Display for RunFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.report.primary.as_ref().expect("failure report"), f)
    }
}

impl Error for RunFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.report
            .primary
            .as_ref()
            .map(|error| error as &dyn Error)
    }
}

#[derive(Clone, Default)]
pub struct ShutdownHandle(Arc<AtomicBool>);

impl ShutdownHandle {
    pub fn request_shutdown(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Inert application plan. The private prototype supports one to four Hosts.
pub struct Application<H = ()> {
    hosts: H,
    shutdown: ShutdownHandle,
}

impl Application {
    pub fn new() -> Self {
        Self {
            hosts: (),
            shutdown: ShutdownHandle::default(),
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl<H> Application<H> {
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.shutdown.clone()
    }
}

macro_rules! append_host {
    ($($h:ident $v:ident),*) => {
        impl<$($h,)*> Application<($($h,)*)> {
            pub fn with_host<N: Host>(self, next: N) -> Application<($($h,)* N,)> {
                let ($($v,)*) = self.hosts;
                Application {
                    hosts: ($($v,)* next,),
                    shutdown: self.shutdown,
                }
            }
        }
    };
}
append_host!();
append_host!(H1 h1);
append_host!(H1 h1, H2 h2);
append_host!(H1 h1, H2 h2, H3 h3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartStatus {
    Started,
    Stopping,
}

// These public-but-hidden types only carry concrete inferred prototype storage.
// There is no erased resource registry and no replica of the core state machine.
#[doc(hidden)]
pub struct Slot<A> {
    activity: RefCell<Option<A>>,
    label: &'static str,
    attempted: Cell<bool>,
}

impl<A: Activity> Slot<A> {
    fn start(&self) -> Result<(), Failure> {
        self.attempted.set(true);
        let mut activity = self.activity.borrow_mut();
        let activity = activity
            .as_mut()
            .expect("activity retained until final release");
        activity
            .start()
            .map_err(|e| Failure::new(self.label, "start", e))?;
        activity
            .wait_ready()
            .map_err(|e| Failure::new(self.label, "readiness", e))
    }

    fn open(&self) -> Result<(), Failure> {
        self.activity
            .borrow_mut()
            .as_mut()
            .expect("active activity")
            .open_admission()
            .map_err(|e| Failure::new(self.label, "open admission", e))
    }

    fn finished(&self) -> bool {
        self.attempted.get()
            && self
                .activity
                .borrow()
                .as_ref()
                .is_some_and(Activity::is_finished)
    }

    fn admission(&mut self) -> Result<(), Failure> {
        if !self.attempted.get() {
            return Ok(());
        }
        self.activity
            .get_mut()
            .as_mut()
            .expect("activity before join")
            .close_admission()
            .map_err(|e| Failure::new(self.label, "admission", e))
    }

    fn signal(&mut self) -> Result<(), Failure> {
        if !self.attempted.get() {
            return Ok(());
        }
        self.activity
            .get_mut()
            .as_mut()
            .expect("activity before join")
            .request_shutdown()
            .map_err(|e| Failure::new(self.label, "shutdown", e))
    }

    fn observe(&mut self, budget: Duration) -> Result<WorkerCompletion<Failure>, Failure> {
        if !self.attempted.get() {
            return Ok(WorkerCompletion::Joined(Ok(())));
        }
        match self
            .activity
            .get_mut()
            .as_mut()
            .expect("activity before join")
            .observe_within(budget)
        {
            Ok(WorkerCompletion::Pending) => Ok(WorkerCompletion::Pending),
            Ok(WorkerCompletion::Joined(result)) => Ok(WorkerCompletion::Joined(
                result.map_err(|e| Failure::new(self.label, "join", e)),
            )),
            Err(error) => Err(Failure::new(self.label, "observe", error)),
        }
    }

    fn retire(&mut self, _: Duration) -> Result<ReleaseCompletion<Failure>, Failure> {
        // The shared root has joined every original activity. Drop this value
        // while all of its earlier dependencies are still intact.
        drop(self.activity.get_mut().take());
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

#[doc(hidden)]
pub type ActivityNode<A> = ManagedWorker<
    Slot<A>,
    fn(&mut Slot<A>) -> Result<(), Failure>,
    fn(&mut Slot<A>) -> Result<(), Failure>,
    fn(&mut Slot<A>, Duration) -> Result<WorkerCompletion<Failure>, Failure>,
    fn(&mut Slot<A>, Duration) -> Result<ReleaseCompletion<Failure>, Failure>,
>;

#[doc(hidden)]
pub type Node<H> = Group<
    Attached<<<H as Host>::Resources as ResourcePlan>::Parts, ActivityNode<<H as Host>::Activity>>,
>;

struct CoreOps<C> {
    start: fn(&mut C, &ShutdownHandle),
    finished: fn(&C) -> bool,
    primary: fn(&C) -> Option<Failure>,
    stop: fn(&mut C),
    complete: fn(&mut C, Instant) -> Completion,
    report: fn(&C) -> RunReport,
}

/// Caller-owned advanced path. Dropping a live or Pending owner does not join.
/// Keep it and continue shutdown_until, or use terminal-only run().
#[must_use = "retain this owner until shutdown is Complete"]
pub struct BuiltApplication<C> {
    core: Option<C>,
    ops: CoreOps<C>,
    shutdown: ShutdownHandle,
    start_attempted: bool,
    report: Option<RunReport>,
}

impl<C> BuiltApplication<C> {
    /// Startup failure stays in this owner and immediately requests rollback.
    /// There is no cleanup-bearing error to accidentally discard with '?'.
    pub fn start(&mut self) -> StartStatus {
        let Some(core) = self.core.as_mut() else {
            return StartStatus::Stopping;
        };
        if self.shutdown.is_requested() {
            (self.ops.stop)(core);
            return StartStatus::Stopping;
        }
        if !self.start_attempted {
            self.start_attempted = true;
            (self.ops.start)(core, &self.shutdown);
        }
        if self.shutdown.is_requested() {
            // The last admission callback can itself request cancellation.
            // Stopping must already have initiated the shared stop path.
            (self.ops.stop)(core);
            return StartStatus::Stopping;
        }
        if (self.ops.primary)(core).is_some() {
            StartStatus::Stopping
        } else {
            StartStatus::Started
        }
    }

    /// Retained cause is available while cleanup remains Pending.
    pub fn primary_failure(&self) -> Option<Failure> {
        match &self.core {
            Some(core) => (self.ops.primary)(core),
            None => self.report.as_ref().and_then(|r| r.primary.clone()),
        }
    }

    pub fn request_shutdown(&mut self) {
        self.shutdown.request_shutdown();
        if let Some(core) = self.core.as_mut() {
            (self.ops.stop)(core);
        }
    }

    pub fn shutdown_until(&mut self, deadline: Instant) -> Completion {
        self.shutdown.request_shutdown();
        let Some(core) = self.core.as_mut() else {
            return Completion::Complete;
        };
        if (self.ops.complete)(core, deadline) == Completion::Pending {
            return Completion::Pending;
        }
        let report = (self.ops.report)(core);
        // Original joins and ordered final release are complete. Drop the
        // remaining coordination shell before exposing the terminal report.
        drop(self.core.take());
        self.report = Some(report);
        Completion::Complete
    }

    pub fn report(&self) -> Option<&RunReport> {
        self.report.as_ref()
    }

    /// Blocks until terminal cleanup, including on startup failure. There is no
    /// timeout that discards an owner. Uncooperative callbacks may wait forever.
    /// Headless services stop through shutdown_handle or activity termination.
    pub fn run(mut self) -> Result<RunReport, RunFailure> {
        if self.start() == StartStatus::Started {
            while !self.shutdown.is_requested()
                && !(self.ops.finished)(self.core.as_ref().expect("active core"))
            {
                thread::sleep(Duration::from_millis(5));
            }
        }
        loop {
            if self.shutdown_until(Instant::now() + Duration::from_millis(20))
                == Completion::Complete
            {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        let report = self.report.take().expect("terminal report");
        if report.primary.is_some() {
            Err(RunFailure { report })
        } else {
            Ok(report)
        }
    }
}

// Composition owns the recursive traversal. ResourcePlan's sealed Parts bound
// lets these bounded Host tuples retain generic, statically typed resource lists.
// Function pointers erase operations, never resources or lifetimes.
// Every admission, retry, stop, join, release and failure latch stays in Composition.
macro_rules! parts {
    ($previous:ty;) => { $previous };
    ($previous:ty; $head:ident $($tail:ident)*) => {
        parts!(Attached<$previous, Node<$head>>; $($tail)*)
    };
}

macro_rules! build_hosts {
    ($($h:ident $v:ident [$($previous:ident)*]),+) => {
        impl<$($h: Host),+> Application<($($h,)+)> {
            pub fn build(self) -> io::Result<BuiltApplication<
                Composition<Failure, parts!(Empty; $($h)*)>
            >> {
                let ($($v,)+) = self.hosts;
                $($v.validate()?;)+
                $(let $v = {
                    let group = $v.into_resources();
                    let label = group.label;
                    let parts = group.resources.build(label).with_activity(
                        label,
                        Slot { activity: RefCell::new(Some(group.activity)), label,
                            attempted: Cell::new(false) },
                        Slot::<$h::Activity>::admission as fn(&mut Slot<$h::Activity>) -> _,
                        Slot::<$h::Activity>::signal as fn(&mut Slot<$h::Activity>) -> _,
                        Slot::<$h::Activity>::observe as fn(&mut Slot<$h::Activity>, Duration) -> _,
                        Slot::<$h::Activity>::retire as fn(&mut Slot<$h::Activity>, Duration) -> _,
                    );
                    (label, parts)
                };)+
                let core = Builder::<Failure>::new()
                    $(.with_group($v.0, $v.1))+
                    .build();
                Ok(BuiltApplication {
                    core: Some(core),
                    ops: CoreOps {
                        start: |core, shutdown| {
                            $(
                                if shutdown.is_requested() {
                                    core.request_stop();
                                    return;
                                }
                                let group = core.parts()$(.$previous())*.last();
                                match $h::Resources::acquire(group.parts().previous(), shutdown) {
                                    Err(error) => {
                                        core.fail_required(error);
                                        return;
                                    }
                                    Ok(false) => {
                                        core.request_stop();
                                        return;
                                    }
                                    Ok(true) => {}
                                }
                                if let Err(error) = group.parts().last().value().start() {
                                    core.fail_required(error);
                                    return;
                                }
                                assert!(core.parts_mut()$(.$previous())*.last().parts_mut().last().mark_ready());
                            )+
                            assert!(core.all_required_ready());
                            $(
                                if shutdown.is_requested() {
                                    core.request_stop();
                                    return;
                                }
                                if let Err(error) = core.parts()$(.$previous())*.last().parts().last().value().open() {
                                    core.fail_required(error);
                                    return;
                                }
                            )+
                        },
                        finished: |core| false $(|| core.parts()$(.$previous())*.last().parts().last().value().finished())+,
                        primary: |core| core.failures().primary().cloned(),
                        stop: |core| core.request_stop(),
                        complete: |core, deadline| core.complete_until(deadline),
                        report: |core| RunReport {
                            hosts: vec![$(
                                HostOutcome {
                                    host: core.parts()$(.$previous())*.last().label(),
                                    start_attempted: core.parts()$(.$previous())*.last().parts().last().value().attempted.get(),
                                    join: core.parts()$(.$previous())*.last().parts().last().join_status(),
                                },
                            )+],
                            primary: core.failures().primary().cloned(),
                            secondary: core.failures().secondary().to_vec(),
                        },
                    },
                    shutdown: self.shutdown,
                    start_attempted: false,
                    report: None,
                })
            }
        }
    };
}
build_hosts!(H1 h1 []);
build_hosts!(H1 h1 [previous], H2 h2 []);
build_hosts!(H1 h1 [previous previous], H2 h2 [previous], H3 h3 []);
build_hosts!(H1 h1 [previous previous previous], H2 h2 [previous previous], H3 h3 [previous], H4 h4 []);
