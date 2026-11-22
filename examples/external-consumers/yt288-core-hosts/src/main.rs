//! A caller-driven custom Host consumer with only Yetlet as a normal dependency.
//!
//! Five distinct resource-plan types use one lifecycle owner. The application
//! supplies a wake context and an explicit event callback; Core creates no
//! executor, timer, thread, socket or process.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};
use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, BuiltHostGroup, HostActivity, HostBuildError, HostFailure, HostList,
        HostParts, JoinStatus, ManagedResource, Phase, ReleaseCompletion, ReleaseStatus,
        ResourceParts, ResourcePlan, StartStatus, WorkerCompletion,
    },
};

#[derive(Debug, Default)]
struct Calls {
    acquire_attempts: Cell<usize>,
    acquired: Cell<usize>,
    started: Cell<usize>,
    opened: Cell<usize>,
    closed: Cell<usize>,
    stopped: Cell<usize>,
    joined: Cell<usize>,
    resource_released: Cell<usize>,
}

fn count(value: &Cell<usize>) {
    value.set(value.get() + 1);
}

// This error deliberately implements neither std::error::Error nor Clone.
// Its borrowed message and Rc also prove that the API imposes no 'static/Send bound.
#[derive(Debug)]
struct AppError<'a> {
    message: &'a str,
    calls: Rc<Calls>,
}

#[derive(Default)]
struct AcquisitionGate {
    allowed: Cell<bool>,
    waiting: RefCell<Option<Waker>>,
}

impl AcquisitionGate {
    fn poll(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.allowed.get() {
            Poll::Ready(())
        } else {
            *self.waiting.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    // In an event-loop application this is called by the actual resource event.
    fn allow_acquisition(&self) {
        self.allowed.set(true);
        if let Some(waker) = self.waiting.borrow_mut().take() {
            waker.wake();
        }
    }
}

struct Journal<'a> {
    entries: &'a mut Vec<&'static str>,
    calls: Rc<Calls>,
    gate: Rc<AcquisitionGate>,
    attempted: bool,
}

impl<'a> ManagedResource for Journal<'a> {
    type Error = AppError<'a>;

    fn poll_acquire(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.attempted {
            self.attempted = true;
            self.entries.push("acquire requested");
            count(&self.calls.acquire_attempts);
        }
        // Partial progress lives in this retained descriptor across Pending.
        if self.gate.poll(cx).is_pending() {
            return Poll::Pending;
        }
        self.entries.push("acquired");
        count(&self.calls.acquired);
        Poll::Ready(Ok(()))
    }

    fn poll_release(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        self.entries.push("released");
        count(&self.calls.resource_released);
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

struct LocalActivity<'a> {
    diagnostic: &'a str,
    calls: Rc<Calls>,
    started: bool,
    stopped: bool,
}

impl<'a> LocalActivity<'a> {
    fn new(diagnostic: &'a str, calls: &Rc<Calls>) -> Self {
        Self {
            diagnostic,
            calls: Rc::clone(calls),
            started: false,
            stopped: false,
        }
    }
}

impl<'a> HostActivity for LocalActivity<'a> {
    type Error = AppError<'a>;

    fn poll_start(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.started = true;
        count(&self.calls.started);
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.started {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(AppError {
                message: self.diagnostic,
                calls: Rc::clone(&self.calls),
            }))
        }
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
        count(&self.calls.opened);
        Ok(AdmissionOutcome::Opened)
    }

    fn close_admission(&mut self, _cx: &mut Context<'_>) -> Result<(), Self::Error> {
        count(&self.calls.closed);
        Ok(())
    }

    fn request_shutdown(&mut self, _cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.stopped = true;
        count(&self.calls.stopped);
        Ok(())
    }

    fn poll_join(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        assert!(self.stopped);
        count(&self.calls.joined);
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}

// A custom Host returns the provided typed plan; it does not implement or copy
// the coordinator. Different A/D types remain statically visible to HostGroup.
struct NamedHost<A, D> {
    name: &'static str,
    resources: ResourceGroup<A, D>,
}

impl<A, D> NamedHost<A, D> {
    fn new(name: &'static str, resources: ResourceGroup<A, D>) -> Self {
        Self { name, resources }
    }
}

impl<A: HostActivity, D: ResourcePlan<A::Error>> Host for NamedHost<A, D> {
    type Error = A::Error;
    type Activity = A;
    type Resources = D;

    fn name(&self) -> &'static str {
        self.name
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        self.resources
    }
}

#[derive(Default)]
struct WakeCount(AtomicUsize);

impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

// Generic caller helpers can name the sealed framework bounds without
// implementing another coordinator or exposing mutable completion storage.
fn build<E, H: HostList<E>>(
    plan: HostGroup<E, H>,
) -> Result<BuiltHostGroup<E, H::Parts>, HostBuildError<E>> {
    plan.build()
}

fn poll_once<E, P>(group: &mut BuiltHostGroup<E, P>, cx: &mut Context<'_>) -> Poll<()>
where
    P: HostParts<E> + ResourceParts<HostFailure<E>>,
{
    group.poll_run(cx)
}

fn main() {
    let diagnostic = String::from("local activity has not started");
    let configuration = String::from("application-owned configuration");
    let calls = Rc::new(Calls::default());
    let local_state = Rc::new(Cell::new(7_u32));
    let gate = Rc::new(AcquisitionGate::default());
    let mut journal = Vec::new();

    let plan = HostGroup::new()
        .with_host(NamedHost::new(
            "journal",
            ResourceGroup::new(LocalActivity::new(&diagnostic, &calls)).with_managed_resource(
                "borrowed journal",
                Journal {
                    entries: &mut journal,
                    calls: Rc::clone(&calls),
                    gate: Rc::clone(&gate),
                    attempted: false,
                },
            ),
        ))
        .with_host(NamedHost::new(
            "local Rc",
            ResourceGroup::new(LocalActivity::new(&diagnostic, &calls))
                .with_resource(Rc::clone(&local_state)),
        ))
        .with_host(NamedHost::new(
            "borrowed configuration",
            ResourceGroup::new(LocalActivity::new(&diagnostic, &calls))
                .with_resource(configuration.as_str()),
        ))
        .with_host(NamedHost::new(
            "owned values",
            ResourceGroup::new(LocalActivity::new(&diagnostic, &calls))
                .with_resource(vec![10_u32, 20]),
        ))
        .with_host(NamedHost::new(
            "local activity",
            ResourceGroup::new(LocalActivity::new(&diagnostic, &calls)),
        ));

    assert_eq!(calls.acquire_attempts.get(), 0);
    assert_eq!(calls.started.get(), 0);
    let mut hosts = build(plan).expect("valid inert configuration");
    assert_eq!(calls.acquire_attempts.get(), 0);
    assert_eq!(calls.started.get(), 0);
    assert_eq!(hosts.outcomes().len(), 5);
    assert_eq!(Rc::strong_count(&local_state), 2);

    let notifications = Arc::new(WakeCount::default());
    let waker = Waker::from(Arc::clone(&notifications));
    let mut cx = Context::from_waker(&waker);

    assert_eq!(hosts.poll_start(&mut cx), Poll::Pending);
    assert_eq!(calls.acquire_attempts.get(), 1);
    assert_eq!(calls.acquired.get(), 0);
    assert_eq!(calls.started.get(), 0);

    gate.allow_acquisition();
    assert_eq!(notifications.0.load(Ordering::SeqCst), 1);
    assert_eq!(hosts.poll_start(&mut cx), Poll::Ready(StartStatus::Running));
    assert_eq!(calls.acquire_attempts.get(), 1);
    assert_eq!(calls.acquired.get(), 1);
    assert_eq!(calls.started.get(), 5);
    assert_eq!(calls.opened.get(), 5);
    assert_eq!(hosts.phase(), Phase::Running);
    assert!(poll_once(&mut hosts, &mut cx).is_pending());

    // A passive service remains running until the caller explicitly requests stop.
    let stop = hosts.shutdown_handle();
    stop.request_shutdown();
    assert!(stop.is_requested());
    assert_eq!(notifications.0.load(Ordering::SeqCst), 2);
    assert_eq!(
        hosts.poll_shutdown(&mut cx, Instant::now()),
        Poll::Ready(())
    );

    if let Some(failure) = hosts.failures().primary() {
        let error = failure.error();
        panic!(
            "{} after {} starts",
            error.message,
            error.calls.started.get()
        );
    }
    assert_eq!(hosts.phase(), Phase::Stopped);
    assert_eq!(calls.closed.get(), 5);
    assert_eq!(calls.stopped.get(), 5);
    assert_eq!(calls.joined.get(), 5);
    assert_eq!(calls.resource_released.get(), 1);
    assert_eq!(Rc::strong_count(&local_state), 1);
    assert!(hosts.outcomes().iter().all(|outcome| {
        outcome.start_attempted
            && outcome.join == JoinStatus::Succeeded
            && outcome.release == ReleaseStatus::Succeeded
    }));

    drop(hosts); // Ends the mutable borrow; shutdown was already observed above.
    assert_eq!(journal, ["acquire requested", "acquired", "released"]);
    assert_eq!(local_state.get(), 7);
    println!("Five custom Hosts completed with a caller-driven, std-only lifecycle.");
}
