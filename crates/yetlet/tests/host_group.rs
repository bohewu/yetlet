use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    future::Future,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};
use yetlet::composition::{
    AdmissionOutcome, ExclusiveClaim, HostActivity, HostBuildError, HostOperation, HostRunError,
    HostRunMode, JoinStatus, ManagedResource, Phase, ReleaseCompletion, ReleaseStatus,
    ResourcePlan, ShutdownHandle, StartStatus, WorkerCompletion,
};
use yetlet::{Host, HostGroup, ResourceGroup};

type Log = Rc<RefCell<Vec<(&'static str, &'static str)>>>;

#[derive(Debug)]
struct DomainError {
    message: &'static str,
    identity: Rc<()>,
}

fn error(message: &'static str) -> DomainError {
    DomainError {
        message,
        identity: Rc::new(()),
    }
}

#[derive(Clone)]
struct Gate(Rc<(Cell<bool>, RefCell<Option<Waker>>)>);

impl Gate {
    fn new(ready: bool) -> Self {
        Self(Rc::new((Cell::new(ready), RefCell::new(None))))
    }
    fn poll(&self, cx: &mut Context<'_>) -> bool {
        if self.0.0.get() {
            true
        } else {
            *self.0.1.borrow_mut() = Some(cx.waker().clone());
            false
        }
    }
    fn open(&self) {
        self.0.0.set(true);
        let waker = self.0.1.borrow_mut().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
    fn close(&self) {
        self.0.0.set(false);
    }
}

struct Lease {
    name: &'static str,
    log: Log,
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.log.borrow_mut().push((self.name, "drop"));
    }
}

#[derive(Clone)]
struct Controls {
    name: &'static str,
    log: Log,
    prepared: Rc<RefCell<Vec<HostRunMode>>>,
    start: Gate,
    ready: Gate,
    finished: Gate,
    admission: Gate,
    signal: Gate,
    join: Gate,
    release: Gate,
}

impl Controls {
    fn new(name: &'static str, log: &Log) -> Self {
        Self {
            name,
            log: log.clone(),
            prepared: Rc::default(),
            start: Gate::new(true),
            ready: Gate::new(true),
            finished: Gate::new(false),
            admission: Gate::new(true),
            signal: Gate::new(true),
            join: Gate::new(true),
            release: Gate::new(true),
        }
    }
    fn record(&self, operation: &'static str) {
        self.log.borrow_mut().push((self.name, operation));
    }
    fn count(&self, operation: &'static str) -> usize {
        self.log
            .borrow()
            .iter()
            .filter(|entry| **entry == (self.name, operation))
            .count()
    }
    fn activity(&self) -> Activity {
        Activity {
            controls: self.clone(),
            owner: None,
            prepare_error: None,
            running_waker: None,
            start_error: None,
            admission_outcome: AdmissionOutcome::Opened,
            join_error: None,
            release_error: None,
        }
    }
    fn resource(&self) -> Dependency {
        Dependency {
            controls: self.clone(),
            owner: None,
            prepare_error: None,
            acquire_error: None,
            release_error: None,
            retry_error: None,
        }
    }
}

struct Activity {
    controls: Controls,
    owner: Option<Lease>,
    prepare_error: Option<DomainError>,
    running_waker: Option<mpsc::Sender<Waker>>,
    start_error: Option<DomainError>,
    admission_outcome: AdmissionOutcome,
    join_error: Option<DomainError>,
    release_error: Option<DomainError>,
}

impl HostActivity for Activity {
    type Error = DomainError;
    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), DomainError> {
        self.controls.record("prepare");
        self.controls.prepared.borrow_mut().push(mode);
        self.prepare_error.take().map_or(Ok(()), Err)
    }
    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), DomainError>> {
        self.controls.record("start");
        self.owner.get_or_insert_with(|| Lease {
            name: self.controls.name,
            log: self.controls.log.clone(),
        });
        if !self.controls.start.poll(cx) {
            return Poll::Pending;
        }
        Poll::Ready(self.start_error.take().map_or(Ok(()), Err))
    }
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), DomainError>> {
        self.controls.record("ready");
        if self.controls.ready.poll(cx) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
    fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(sender) = self.running_waker.take() {
            sender.send(cx.waker().clone()).unwrap();
        }
        if self.controls.finished.poll(cx) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
    fn open_admission(&mut self) -> Result<AdmissionOutcome, DomainError> {
        self.controls.record("open");
        Ok(self.admission_outcome)
    }
    fn close_admission(&mut self, cx: &mut Context<'_>) -> Result<(), DomainError> {
        self.controls.record("close");
        if self.controls.admission.poll(cx) {
            Ok(())
        } else {
            Err(error("fence unavailable"))
        }
    }
    fn request_shutdown(&mut self, cx: &mut Context<'_>) -> Result<(), DomainError> {
        self.controls.record("signal");
        if self.controls.signal.poll(cx) {
            Ok(())
        } else {
            Err(error("signal unavailable"))
        }
    }
    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<DomainError>, DomainError> {
        self.controls.record("observe");
        if !self.controls.join.poll(cx) {
            return Ok(WorkerCompletion::Pending);
        }
        self.controls.record("joined");
        Ok(WorkerCompletion::Joined(
            self.join_error.take().map_or(Ok(()), Err),
        ))
    }
    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        _: Duration,
    ) -> Result<ReleaseCompletion<DomainError>, DomainError> {
        self.controls.record("release");
        if !self.controls.release.poll(cx) {
            return Ok(ReleaseCompletion::Pending);
        }
        Ok(ReleaseCompletion::Released(
            self.release_error.take().map_or(Ok(()), Err),
        ))
    }
}

struct Dependency {
    controls: Controls,
    owner: Option<Lease>,
    prepare_error: Option<DomainError>,
    acquire_error: Option<DomainError>,
    release_error: Option<DomainError>,
    retry_error: Option<DomainError>,
}

impl ManagedResource for Dependency {
    type Error = DomainError;
    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), DomainError> {
        self.controls.record("prepare");
        self.controls.prepared.borrow_mut().push(mode);
        self.prepare_error.take().map_or(Ok(()), Err)
    }
    fn poll_acquire(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), DomainError>> {
        self.controls.record("acquire");
        self.owner.get_or_insert_with(|| Lease {
            name: self.controls.name,
            log: self.controls.log.clone(),
        });
        if !self.controls.start.poll(cx) {
            return Poll::Pending;
        }
        Poll::Ready(self.acquire_error.take().map_or(Ok(()), Err))
    }
    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        _: Duration,
    ) -> Result<ReleaseCompletion<DomainError>, DomainError> {
        self.controls.record("release");
        if !self.controls.release.poll(cx) {
            return Ok(ReleaseCompletion::Pending);
        }
        if let Some(error) = self.retry_error.take() {
            // This one-shot operation can make progress on the very next poll.
            cx.waker().wake_by_ref();
            return Err(error);
        }
        Ok(ReleaseCompletion::Released(
            self.release_error.take().map_or(Ok(()), Err),
        ))
    }
}

struct TestHost<A, R> {
    name: &'static str,
    resources: ResourceGroup<A, R>,
    invalid: bool,
    claim: Option<ExclusiveClaim>,
    conversions: Rc<Cell<usize>>,
}

fn host<A, R>(name: &'static str, resources: ResourceGroup<A, R>) -> TestHost<A, R> {
    TestHost {
        name,
        resources,
        invalid: false,
        claim: None,
        conversions: Rc::default(),
    }
}

impl<A: HostActivity<Error = DomainError>, R: ResourcePlan<DomainError>> Host for TestHost<A, R> {
    type Error = DomainError;
    type Activity = A;
    type Resources = R;
    fn name(&self) -> &'static str {
        self.name
    }
    fn validate(&self) -> Result<(), DomainError> {
        if self.invalid {
            Err(error("invalid plan"))
        } else {
            Ok(())
        }
    }
    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        self.claim.iter().cloned().collect()
    }
    fn into_resources(self) -> ResourceGroup<A, R> {
        self.conversions.set(self.conversions.get() + 1);
        self.resources
    }
}

#[derive(Default)]
struct Notifications(AtomicUsize);
impl Wake for Notifications {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
fn wake_context() -> (Arc<Notifications>, Waker) {
    let notifications = Arc::new(Notifications::default());
    let waker = Waker::from(notifications.clone());
    (notifications, waker)
}

#[test]
fn complete_plan_validates_before_conversion_or_acquisition() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    let first = host("a", ResourceGroup::new(a.activity()));
    let conversions = first.conversions.clone();
    let mut last = host("b", ResourceGroup::new(b.activity()));
    last.invalid = true;
    let result = HostGroup::new().with_host(first).with_host(last).build();
    assert!(matches!(result, Err(HostBuildError::InvalidHost(_))));
    assert_eq!(conversions.get(), 0);
    assert!(log.borrow().is_empty());
}

#[test]
fn duplicate_exclusive_claims_reject_the_inert_plan() {
    let log = Log::default();
    let mut first = host("a", ResourceGroup::new(Controls::new("a", &log).activity()));
    let mut second = host("b", ResourceGroup::new(Controls::new("b", &log).activity()));
    first.claim = Some(ExclusiveClaim::new("event-loop", "process"));
    second.claim = first.claim.clone();
    match HostGroup::new().with_host(first).with_host(second).build() {
        Err(HostBuildError::ConflictingClaim {
            first,
            second,
            claim,
        }) => {
            assert_eq!((first, second), ("a", "b"));
            assert_eq!(claim.key, "process");
        }
        _ => panic!("expected an inert claim conflict"),
    }
    assert!(log.borrow().is_empty());
}

#[test]
fn pending_readiness_blocks_admission_without_restarting_earlier_hosts() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    b.ready.close();
    let mut hosts = HostGroup::new()
        .with_host(host("a", ResourceGroup::new(a.activity())))
        .with_host(host("b", ResourceGroup::new(b.activity())))
        .build()
        .unwrap();
    assert!(log.borrow().is_empty());
    let (_, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert!(hosts.poll_start(&mut cx).is_pending());
    assert_eq!(a.count("open") + b.count("open"), 0);
    b.ready.open();
    assert_eq!(hosts.poll_start(&mut cx), Poll::Ready(StartStatus::Running));
    assert_eq!(hosts.poll_start(&mut cx), Poll::Ready(StartStatus::Running));
    assert_eq!(
        (a.count("start"), a.count("ready"), a.count("open")),
        (1, 1, 1)
    );
    assert_eq!(
        (b.count("start"), b.count("ready"), b.count("open")),
        (1, 2, 1)
    );
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
}

#[test]
fn earlier_termination_wakes_and_rolls_back_while_later_readiness_is_pending() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    b.ready.close();
    let mut hosts = HostGroup::new()
        .with_host(host("a", ResourceGroup::new(a.activity())))
        .with_host(host("b", ResourceGroup::new(b.activity())))
        .build()
        .unwrap();
    let (notifications, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert!(hosts.poll_run(&mut cx).is_pending());
    a.finished.open();
    assert!(notifications.0.load(Ordering::SeqCst) > 0);
    assert!(hosts.poll_run(&mut cx).is_ready());
    assert_eq!(a.count("open") + b.count("open"), 0);
    assert_eq!((a.count("joined"), b.count("joined")), (1, 1));
    assert_eq!(hosts.phase(), Phase::Stopped);
}

#[test]
fn closed_admission_stops_later_opens_and_retains_original_join_and_dependencies() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    let c = Controls::new("c", &log);
    let ra = Controls::new("ra", &log);
    let rb = Controls::new("rb", &log);
    b.join.close();
    let failure = error("original execution");
    let identity = failure.identity.clone();
    let mut closed = b.activity();
    closed.admission_outcome = AdmissionOutcome::Closed;
    closed.join_error = Some(failure);
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(a.activity()).with_managed_resource("ra", ra.resource()),
        ))
        .with_host(host(
            "b",
            ResourceGroup::new(closed).with_managed_resource("rb", rb.resource()),
        ))
        .with_host(host("c", ResourceGroup::new(c.activity())))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);

    // The preceding termination poll was Pending. Closure is first observed
    // inside admission, before the original execution outcome is available.
    assert_eq!(
        hosts.poll_start(&mut cx),
        Poll::Ready(StartStatus::Stopping)
    );
    assert_eq!(hosts.phase(), Phase::Stopping);
    assert_eq!(
        (a.count("open"), b.count("open"), c.count("open")),
        (1, 1, 0)
    );
    assert!(hosts.failures().primary().is_none());
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_pending());
    assert_eq!(
        (a.count("joined"), b.count("joined"), c.count("joined")),
        (1, 0, 1)
    );
    assert_eq!(ra.count("release") + rb.count("release"), 0);
    assert_eq!(
        a.count("drop") + b.count("drop") + c.count("drop") + ra.count("drop") + rb.count("drop"),
        0
    );
    assert!(hosts.failures().primary().is_none());
    {
        let events = log.borrow();
        let last_close = events.iter().rposition(|(_, op)| *op == "close").unwrap();
        let first_signal = events.iter().position(|(_, op)| *op == "signal").unwrap();
        assert!(last_close < first_signal);
    }

    b.join.open();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(hosts.phase(), Phase::Failed);
    let primary = hosts.failures().primary().unwrap();
    assert_eq!(primary.operation, HostOperation::Join);
    assert!(Rc::ptr_eq(&identity, &primary.error().identity));
    assert!(hosts.failures().secondary().is_empty());
    assert_eq!(
        (a.count("open"), b.count("open"), c.count("open")),
        (1, 1, 0)
    );
    assert_eq!(
        (a.count("joined"), b.count("joined"), c.count("joined")),
        (1, 1, 1)
    );
    assert_eq!((ra.count("release"), rb.count("release")), (1, 1));
    let before = log.borrow().clone();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(*log.borrow(), before);
}

#[test]
fn original_nonclone_failure_and_global_join_release_barriers_are_preserved() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    let ra = Controls::new("ra", &log);
    let rb = Controls::new("rb", &log);
    a.join.close();
    rb.release.close();
    let failure = error("original startup");
    let identity = failure.identity.clone();
    let mut broken = b.activity();
    broken.start_error = Some(failure);
    let mut later_resource = rb.resource();
    later_resource.release_error = Some(error("terminal release"));
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(a.activity()).with_managed_resource("ra", ra.resource()),
        ))
        .with_host(host(
            "b",
            ResourceGroup::new(broken).with_managed_resource("rb", later_resource),
        ))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert!(hosts.poll_run(&mut cx).is_pending());
    let primary = hosts.failures().primary().unwrap();
    assert_eq!(primary.operation, HostOperation::Start);
    assert!(Rc::ptr_eq(&identity, &primary.error().identity));
    assert_eq!(ra.count("release") + rb.count("release"), 0);
    assert_eq!(a.count("drop") + b.count("drop"), 0);
    a.join.open();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_pending());
    assert_eq!(ra.count("release"), 0);
    assert_eq!(rb.count("release"), 1);
    rb.release.open();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(hosts.phase(), Phase::Failed);
    assert!(Rc::ptr_eq(
        &identity,
        &hosts.failures().primary().unwrap().error().identity
    ));
    assert_eq!(
        hosts.failures().secondary()[0].error().message,
        "terminal release"
    );
    let before = log.borrow().clone();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(*log.borrow(), before);
    assert!(
        hosts
            .outcomes()
            .iter()
            .all(|outcome| outcome.join == JoinStatus::Succeeded)
    );
    let events = log.borrow();
    let last_join = events.iter().rposition(|(_, op)| *op == "joined").unwrap();
    let first_release = events.iter().position(|(_, op)| *op == "release").unwrap();
    assert!(last_join < first_release);
    let b_drop = events
        .iter()
        .position(|event| *event == ("b", "drop"))
        .unwrap();
    let rb_release = events
        .iter()
        .position(|event| *event == ("rb", "release"))
        .unwrap();
    assert!(b_drop < rb_release);
}

#[test]
fn partial_resource_failure_releases_only_attempted_descriptors() {
    let log = Log::default();
    let activity = Controls::new("a", &log);
    let first = Controls::new("first", &log);
    let later = Controls::new("later", &log);
    first.release.close();
    let mut resource = first.resource();
    resource.acquire_error = Some(error("partial acquisition"));
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(activity.activity())
                .with_managed_resource("first", resource)
                .with_managed_resource("later", later.resource()),
        ))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert!(hosts.poll_run(&mut cx).is_pending());
    assert_eq!(activity.count("start"), 0);
    assert_eq!(later.count("acquire") + later.count("release"), 0);
    assert_eq!(first.count("drop"), 0);
    first.release.open();
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(first.count("drop"), 1);
    assert_eq!(
        hosts.failures().primary().unwrap().operation,
        HostOperation::Acquire
    );
}

#[test]
fn cancelled_borrowing_drives_resume_the_same_owners_at_every_pending_phase() {
    for phase in [
        "acquire",
        "start",
        "ready",
        "running",
        "join",
        "activity release",
        "resource release",
    ] {
        let log = Log::default();
        let a = Controls::new("a", &log);
        let r = Controls::new("resource", &log);
        match phase {
            "acquire" => r.start.close(),
            "start" => a.start.close(),
            "ready" => a.ready.close(),
            "running" => {}
            "join" => {
                a.finished.open();
                a.join.close();
            }
            "activity release" => {
                a.finished.open();
                a.release.close();
            }
            "resource release" => {
                a.finished.open();
                r.release.close();
            }
            _ => unreachable!(),
        }
        let mut hosts = HostGroup::new()
            .with_host(host(
                "a",
                ResourceGroup::new(a.activity()).with_managed_resource("resource", r.resource()),
            ))
            .build()
            .unwrap();
        let (_, waker) = wake_context();
        let mut cx = Context::from_waker(&waker);
        {
            let mut drive = Box::pin(hosts.run_async());
            assert!(drive.as_mut().poll(&mut cx).is_pending(), "{phase}");
        } // Only the borrowing drive is cancelled.
        assert_eq!(r.count("drop"), 0, "{phase}");
        hosts.shutdown_handle().request_shutdown();
        r.start.open();
        a.start.open();
        a.ready.open();
        a.join.open();
        a.release.open();
        r.release.open();
        {
            let mut resumed = Box::pin(hosts.run_async());
            assert!(
                matches!(resumed.as_mut().poll(&mut cx), Poll::Ready(Ok(()))),
                "{phase}"
            );
        }
        assert_eq!(r.count("drop"), 1, "{phase}");
        assert_eq!(r.count("acquire"), 1, "{phase}: acquisition was restarted");
        assert_eq!(a.count("start"), usize::from(phase != "acquire"), "{phase}");
        assert_eq!(hosts.phase(), Phase::Stopped);
    }
}

#[test]
fn retryable_fence_errors_receive_the_current_drive_waker() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    a.admission.close();
    let mut hosts = HostGroup::new()
        .with_host(host("a", ResourceGroup::new(a.activity())))
        .build()
        .unwrap();
    let (_, initial) = wake_context();
    assert_eq!(
        hosts.poll_start(&mut Context::from_waker(&initial)),
        Poll::Ready(StartStatus::Running)
    );
    hosts.shutdown_handle().request_shutdown();
    let (notifications, current) = wake_context();
    let mut cx = Context::from_waker(&current);
    {
        let mut drive = Box::pin(hosts.run_async());
        assert!(drive.as_mut().poll(&mut cx).is_pending());
        assert_eq!(a.count("signal") + a.count("observe"), 0);
        let before = notifications.0.load(Ordering::SeqCst);
        a.admission.open();
        assert!(notifications.0.load(Ordering::SeqCst) > before);
        assert!(matches!(drive.as_mut().poll(&mut cx), Poll::Ready(Err(_))));
    }
    assert_eq!(a.count("joined"), 1);
    assert_eq!(hosts.phase(), Phase::Failed);
}

#[test]
fn failed_signal_does_not_prevent_original_terminal_join() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    a.signal.close();
    let mut activity = a.activity();
    activity.join_error = Some(error("original execution"));
    let mut hosts = HostGroup::new()
        .with_host(host("a", ResourceGroup::new(activity)))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(hosts.poll_start(&mut cx), Poll::Ready(StartStatus::Running));
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(
        hosts.failures().primary().unwrap().operation,
        HostOperation::Shutdown
    );
    assert_eq!(
        hosts.failures().secondary()[0].operation,
        HostOperation::Join
    );
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Failed);
    assert_eq!(a.count("joined"), 1);
}

#[test]
fn retryable_resource_release_retains_dependencies_and_terminal_error_is_latched() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let first = Controls::new("first", &log);
    let later = Controls::new("later", &log);
    let mut resource = later.resource();
    resource.retry_error = Some(error("release progress"));
    resource.release_error = Some(error("release terminal"));
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(a.activity())
                .with_managed_resource("first", first.resource())
                .with_managed_resource("later", resource),
        ))
        .build()
        .unwrap();
    let (notifications, waker) = wake_context();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(hosts.poll_start(&mut cx), Poll::Ready(StartStatus::Running));
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_pending());
    assert!(notifications.0.load(Ordering::SeqCst) > 0);
    assert_eq!(first.count("release"), 0);
    assert_eq!(later.count("drop"), 0);
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
    assert_eq!(later.count("release"), 2);
    assert_eq!(first.count("release"), 1);
    assert_eq!(
        hosts.failures().primary().unwrap().operation,
        HostOperation::ReleaseProgress
    );
    assert_eq!(
        hosts.failures().secondary()[0].operation,
        HostOperation::Release
    );
    assert_eq!(hosts.outcomes()[0].release, ReleaseStatus::Succeeded);
    assert_eq!(a.count("joined"), 1);
}

struct Passive;
impl HostActivity for Passive {
    type Error = Infallible;
    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }
    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), Infallible> {
        Ok(())
    }
    fn poll_join(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<Infallible>, Infallible> {
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}
struct PassiveHost;
impl Host for PassiveHost {
    type Error = Infallible;
    type Activity = Passive;
    type Resources = ();
    fn name(&self) -> &'static str {
        "passive"
    }
    fn into_resources(self) -> ResourceGroup<Passive> {
        ResourceGroup::new(Passive)
    }
}

#[test]
fn send_sync_owners_and_borrowing_futures_are_not_forced_local() {
    fn send_sync<T: Send + Sync>(_: &T) {}
    fn send<T: Send>(_: &T) {}
    let mut hosts = HostGroup::new().with_host(PassiveHost).build().unwrap();
    send_sync(&hosts);
    {
        let drive = hosts.run_async();
        send(&drive);
    }
    assert_eq!(
        hosts.shutdown_until(Instant::now()),
        yetlet::composition::Completion::Complete
    );
}

struct StopOnDrop(ShutdownHandle);
impl Wake for StopOnDrop {
    fn wake(self: Arc<Self>) {
        self.0.request_shutdown();
    }
}
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

#[test]
fn replacing_a_waker_can_reenter_shutdown_without_holding_its_mutex() {
    let mut hosts = HostGroup::new().with_host(PassiveHost).build().unwrap();
    let old = Waker::from(Arc::new(StopOnDrop(hosts.shutdown_handle())));
    assert_eq!(
        hosts.poll_start(&mut Context::from_waker(&old)),
        Poll::Ready(StartStatus::Running)
    );
    drop(old); // Registration now holds the final reference.
    let (_, replacement) = wake_context();
    let mut cx = Context::from_waker(&replacement);
    assert_eq!(
        hosts.poll_start(&mut cx),
        Poll::Ready(StartStatus::Stopping)
    );
    assert!(hosts.poll_shutdown(&mut cx, Instant::now()).is_ready());
}

#[test]
fn owner_drop_releases_a_waker_that_retains_its_shutdown_handle() {
    let mut hosts = HostGroup::new().with_host(PassiveHost).build().unwrap();
    let registration = Arc::new(StopOnDrop(hosts.shutdown_handle()));
    let weak = Arc::downgrade(&registration);
    let waker = Waker::from(registration);
    assert_eq!(
        hosts.poll_start(&mut Context::from_waker(&waker)),
        Poll::Ready(StartStatus::Running)
    );
    drop(waker); // Only the Core registration retains this shutdown handle.
    assert!(weak.upgrade().is_some());
    drop(hosts); // No replacement poll or explicit shutdown.
    assert!(weak.upgrade().is_none());
}

#[test]
fn early_drop_is_ordinary_destruction_not_shutdown() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let r = Controls::new("resource", &log);
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(a.activity()).with_managed_resource("resource", r.resource()),
        ))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    assert_eq!(
        hosts.poll_start(&mut Context::from_waker(&waker)),
        Poll::Ready(StartStatus::Running)
    );
    drop(hosts);
    assert_eq!(
        a.count("signal") + a.count("joined") + a.count("release") + r.count("release"),
        0
    );
    assert_eq!(a.count("drop") + r.count("drop"), 2);
}

#[test]
fn driver_preparation_precedes_all_acquisition_and_survives_drive_cancellation() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let b = Controls::new("b", &log);
    let ra = Controls::new("ra", &log);
    let rb = Controls::new("rb", &log);
    b.ready.close();
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(a.activity()).with_managed_resource("ra", ra.resource()),
        ))
        .with_host(host(
            "b",
            ResourceGroup::new(b.activity()).with_managed_resource("rb", rb.resource()),
        ))
        .build()
        .unwrap();
    assert!(log.borrow().is_empty());
    let (_, first) = wake_context();
    {
        let mut drive = Box::pin(hosts.run_async());
        assert!(
            drive
                .as_mut()
                .poll(&mut Context::from_waker(&first))
                .is_pending()
        );
    }
    assert_eq!(
        &log.borrow()[..4],
        &[
            ("ra", "prepare"),
            ("a", "prepare"),
            ("rb", "prepare"),
            ("b", "prepare")
        ]
    );
    assert_eq!(log.borrow()[4], ("ra", "acquire"));
    let (_, replacement) = wake_context();
    let mut cx = Context::from_waker(&replacement);
    {
        let mut resumed = Box::pin(hosts.run_async());
        assert!(resumed.as_mut().poll(&mut cx).is_pending());
    }
    for control in [&ra, &a, &rb, &b] {
        assert_eq!(*control.prepared.borrow(), [HostRunMode::Async]);
    }
    let before = log.borrow().clone();
    let phase = hosts.phase();
    assert!(matches!(
        hosts.run(),
        Err(HostRunError::ModeMismatch {
            selected: HostRunMode::Async,
            requested: HostRunMode::Blocking,
        })
    ));
    assert_eq!(*log.borrow(), before);
    assert_eq!(hosts.phase(), phase);
    assert!(!hosts.shutdown_handle().is_requested());
    assert_eq!(a.count("drop") + b.count("drop"), 0);
    hosts.shutdown_handle().request_shutdown();
    {
        let mut resumed = Box::pin(hosts.run_async());
        assert!(matches!(
            resumed.as_mut().poll(&mut cx),
            Poll::Ready(Ok(()))
        ));
    }
    assert_eq!(hosts.phase(), Phase::Stopped);
    assert_eq!(a.count("start") + b.count("start"), 2);
    let completed = log.borrow().clone();
    assert!(hosts.run().is_ok());
    assert_eq!(*log.borrow(), completed);
}

#[test]
fn preparation_failure_preserves_original_error_before_any_group_acquisition() {
    for resource_failure in [false, true] {
        let log = Log::default();
        let a = Controls::new("a", &log);
        let b = Controls::new("b", &log);
        let ra = Controls::new("ra", &log);
        let rb = Controls::new("rb", &log);
        let failure = error("driver incompatible");
        let identity = failure.identity.clone();
        let mut activity = b.activity();
        let mut resource = rb.resource();
        if resource_failure {
            resource.prepare_error = Some(failure);
        } else {
            activity.prepare_error = Some(failure);
        }
        let mut hosts = HostGroup::new()
            .with_host(host(
                "a",
                ResourceGroup::new(a.activity()).with_managed_resource("ra", ra.resource()),
            ))
            .with_host(host(
                "b",
                ResourceGroup::new(activity).with_managed_resource("rb", resource),
            ))
            .build()
            .unwrap();
        let (_, waker) = wake_context();
        let mut cx = Context::from_waker(&waker);
        {
            let mut drive = Box::pin(hosts.run_async());
            let Poll::Ready(Err(failure)) = drive.as_mut().poll(&mut cx) else {
                panic!("inert preparation failure must complete ordinary rollback");
            };
            assert_eq!(failure.operation, HostOperation::PrepareDriver);
            assert_eq!(failure.host, "b");
            assert_eq!(failure.resource, resource_failure.then_some("rb"));
            assert!(Rc::ptr_eq(&identity, &failure.error().identity));
        }
        for control in [&ra, &a, &rb, &b] {
            assert_eq!(control.count("acquire") + control.count("start"), 0);
            assert_eq!(control.count("release") + control.count("drop"), 0);
        }
        assert_eq!(hosts.phase(), Phase::Failed);
        assert!(
            hosts
                .outcomes()
                .iter()
                .all(|outcome| !outcome.start_attempted)
        );
    }
}

#[test]
fn synchronous_drive_uses_current_thread_wakes_and_preserves_local_error_identity() {
    let log = Log::default();
    let a = Controls::new("a", &log);
    let r = Controls::new("resource", &log);
    let failure = error("original synchronous completion");
    let identity = failure.identity.clone();
    let (sender, receiver) = mpsc::channel();
    let mut activity = a.activity();
    activity.join_error = Some(failure);
    activity.running_waker = Some(sender);
    let mut hosts = HostGroup::new()
        .with_host(host(
            "a",
            ResourceGroup::new(activity).with_managed_resource("resource", r.resource()),
        ))
        .build()
        .unwrap();
    let stop = hosts.shutdown_handle();
    let controller = thread::spawn(move || {
        let waker = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        stop.request_shutdown();
        waker.wake();
    });
    match hosts.run() {
        Err(HostRunError::HostFailure(failure)) => {
            assert_eq!(failure.operation, HostOperation::Join);
            assert!(Rc::ptr_eq(&identity, &failure.error().identity));
        }
        result => panic!("expected retained completion failure, got {result:?}"),
    }
    controller.join().unwrap();
    assert_eq!(*a.prepared.borrow(), [HostRunMode::Blocking]);
    assert_eq!(*r.prepared.borrow(), [HostRunMode::Blocking]);
    assert_eq!(a.count("joined"), 1);
    assert_eq!(r.count("release"), 1);
    assert_eq!(hosts.phase(), Phase::Failed);
    let before = log.borrow().clone();
    let (_, waker) = wake_context();
    {
        let mut terminal = Box::pin(hosts.run_async());
        assert!(matches!(
            terminal.as_mut().poll(&mut Context::from_waker(&waker)),
            Poll::Ready(Err(_))
        ));
    }
    assert_eq!(*log.borrow(), before);
}

struct TextActivity {
    prepared: Rc<Cell<Option<HostRunMode>>>,
}

impl HostActivity for TextActivity {
    type Error = &'static str;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        self.prepared.set(Some(mode));
        Ok(())
    }

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Err("transport startup"))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_join(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}

struct MappedTextActivity {
    inner: TextActivity,
    identity: Rc<()>,
}

impl MappedTextActivity {
    fn map_error(&self, message: &'static str) -> DomainError {
        DomainError {
            message,
            identity: self.identity.clone(),
        }
    }
}

impl HostActivity for MappedTextActivity {
    type Error = DomainError;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        self.inner
            .prepare_driver(mode)
            .map_err(|e| self.map_error(e))
    }

    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_start(cx).map_err(|e| self.map_error(e))
    }

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| self.map_error(e))
    }

    fn request_shutdown(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.inner
            .request_shutdown(cx)
            .map_err(|e| self.map_error(e))
    }

    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        Ok(
            match self
                .inner
                .poll_join(cx, budget)
                .map_err(|e| self.map_error(e))?
            {
                WorkerCompletion::Pending => WorkerCompletion::Pending,
                WorkerCompletion::Joined(result) => {
                    WorkerCompletion::Joined(result.map_err(|e| self.map_error(e)))
                }
            },
        )
    }
}

#[test]
fn map_activity_preserves_resources_and_accepts_a_local_common_error() {
    let log = Log::default();
    let r = Controls::new("managed", &log);
    let prepared = Rc::new(Cell::new(None));
    let identity = Rc::new(());
    let mapped_identity = identity.clone();
    let resources = ResourceGroup::new(TextActivity {
        prepared: prepared.clone(),
    })
    .with_resource(Lease {
        name: "passive",
        log: log.clone(),
    })
    .map_activity(move |inner| MappedTextActivity {
        inner,
        identity: mapped_identity,
    })
    .with_managed_resource("managed", r.resource());
    let mut hosts = HostGroup::new()
        .with_host(host("mapped", resources))
        .build()
        .unwrap();
    let (_, waker) = wake_context();
    {
        let mut drive = Box::pin(hosts.run_async());
        let Poll::Ready(Err(failure)) = drive.as_mut().poll(&mut Context::from_waker(&waker))
        else {
            panic!("mapped startup failure must retain its original error");
        };
        assert_eq!(failure.operation, HostOperation::Start);
        assert_eq!(failure.error().message, "transport startup");
        assert!(Rc::ptr_eq(&identity, &failure.error().identity));
    }
    assert_eq!(prepared.get(), Some(HostRunMode::Async));
    assert_eq!(r.count("acquire"), 1);
    assert_eq!(r.count("release"), 1);
    let events = log.borrow();
    let release = events
        .iter()
        .position(|event| *event == ("managed", "release"))
        .unwrap();
    let passive_drop = events
        .iter()
        .position(|event| *event == ("passive", "drop"))
        .unwrap();
    assert!(release < passive_drop);
}
