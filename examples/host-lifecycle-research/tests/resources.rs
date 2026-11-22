use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    io,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use yetlet_host_lifecycle_research::{
    Activity, Application, Completion, Host, ManagedResource, ReleaseCompletion, ResourceGroup,
    ResourcePlan, ShutdownHandle, StartStatus, WorkerCompletion,
};

type Trace = Rc<RefCell<Vec<String>>>;

#[derive(Default)]
struct State {
    acquired: Cell<usize>,
    releases: Cell<usize>,
    live: Cell<bool>,
    allow_release: Cell<bool>,
    drops: Cell<usize>,
}

struct ResourceDescription<'a> {
    name: &'static str,
    state: Rc<State>,
    trace: Trace,
    caller_acquisitions: &'a Cell<usize>,
    fail_acquire: bool,
    retry_error: bool,
    terminal_error: bool,
    stop_on_acquire: Option<ShutdownHandle>,
}
impl ManagedResource for ResourceDescription<'_> {
    fn acquire(&mut self) -> io::Result<()> {
        self.state.acquired.set(self.state.acquired.get() + 1);
        self.caller_acquisitions
            .set(self.caller_acquisitions.get() + 1);
        // Represents the partial owner: it is retained before failure is possible.
        self.state.live.set(true);
        self.trace
            .borrow_mut()
            .push(format!("{} acquired", self.name));
        if let Some(stop) = &self.stop_on_acquire {
            stop.request_shutdown();
        }
        if self.fail_acquire {
            Err(io::Error::other("partial acquisition failed"))
        } else {
            Ok(())
        }
    }

    fn release_within(&mut self, _: Duration) -> io::Result<ReleaseCompletion<io::Error>> {
        assert!(
            self.state.live.get(),
            "released resource must never be retried"
        );
        self.state.releases.set(self.state.releases.get() + 1);
        self.trace
            .borrow_mut()
            .push(format!("{} release", self.name));
        if std::mem::take(&mut self.retry_error) {
            return Err(io::Error::other("retryable cleanup failure"));
        }
        if !self.state.allow_release.get() {
            return Ok(ReleaseCompletion::Pending);
        }
        self.state.live.set(false);
        let result = if self.terminal_error {
            Err(io::Error::other("flush failed but handle closed"))
        } else {
            Ok(())
        };
        Ok(ReleaseCompletion::Released(result))
    }
}
impl Drop for ResourceDescription<'_> {
    fn drop(&mut self) {
        self.state.drops.set(self.state.drops.get() + 1);
        self.trace
            .borrow_mut()
            .push(format!("{} descriptor dropped", self.name));
    }
}

fn resource<'a>(
    name: &'static str,
    state: &Rc<State>,
    trace: &Trace,
    caller: &'a Cell<usize>,
) -> ResourceDescription<'a> {
    ResourceDescription {
        name,
        state: state.clone(),
        trace: trace.clone(),
        caller_acquisitions: caller,
        fail_acquire: false,
        retry_error: false,
        terminal_error: false,
        stop_on_acquire: None,
    }
}

struct LocalActivity<'a> {
    trace: Trace,
    dependencies: Vec<Rc<State>>,
    starts: &'a Cell<usize>,
    finish: &'a Cell<bool>,
    started: bool,
    joined: bool,
}
impl Activity for LocalActivity<'_> {
    fn start(&mut self) -> io::Result<()> {
        assert!(self.dependencies.iter().all(|s| s.live.get()));
        self.started = true;
        self.starts.set(self.starts.get() + 1);
        self.trace.borrow_mut().push("activity start".into());
        Ok(())
    }
    fn wait_ready(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.finish.get()
    }
    fn close_admission(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn request_shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn observe_within(&mut self, _: Duration) -> io::Result<WorkerCompletion<io::Error>> {
        if !self.finish.get() {
            assert!(self.dependencies.iter().all(|s| s.live.get()));
            return Ok(WorkerCompletion::Pending);
        }
        assert!(!self.joined);
        self.joined = true;
        self.trace.borrow_mut().push("activity joined".into());
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}
impl Drop for LocalActivity<'_> {
    fn drop(&mut self) {
        if self.started {
            assert!(self.joined, "activity owner retired before completion");
            assert!(
                self.dependencies.iter().all(|s| s.live.get()),
                "activity destructor lost a dependency"
            );
        }
        self.trace.borrow_mut().push("activity dropped".into());
    }
}
fn activity<'a>(
    trace: &Trace,
    dependencies: Vec<Rc<State>>,
    starts: &'a Cell<usize>,
    finish: &'a Cell<bool>,
) -> LocalActivity<'a> {
    LocalActivity {
        trace: trace.clone(),
        dependencies,
        starts,
        finish,
        started: false,
        joined: false,
    }
}

struct CustomHost<A, D> {
    group: ResourceGroup<A, D>,
    valid: bool,
}
impl<A: Activity, D: ResourcePlan> Host for CustomHost<A, D> {
    type Activity = A;
    type Resources = D;
    fn validate(&self) -> io::Result<()> {
        if self.valid {
            Ok(())
        } else {
            Err(io::Error::other("invalid inert plan"))
        }
    }
    fn into_resources(self) -> ResourceGroup<A, D> {
        self.group
    }
}
fn host<A, D>(group: ResourceGroup<A, D>) -> CustomHost<A, D> {
    CustomHost { group, valid: true }
}

#[test]
fn join_and_activity_destruction_precede_reverse_managed_release() {
    let trace = Trace::default();
    let (db, transaction) = (Rc::new(State::default()), Rc::new(State::default()));
    db.allow_release.set(true);
    let (starts, acquisitions) = (Cell::new(0), Cell::new(0));
    let finish = Cell::new(false);
    let group = ResourceGroup::new(
        "custom",
        activity(
            &trace,
            vec![db.clone(), transaction.clone()],
            &starts,
            &finish,
        ),
    )
    .with_managed_resource("database", resource("database", &db, &trace, &acquisitions))
    .with_managed_resource(
        "transaction",
        resource("transaction", &transaction, &trace, &acquisitions),
    );
    let mut app = Application::new().with_host(host(group)).build().unwrap();
    assert_eq!(acquisitions.get(), 0);
    assert_eq!(app.start(), StartStatus::Started);
    assert_eq!(acquisitions.get(), 2);
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert_eq!(transaction.releases.get(), 0);
    assert!(db.live.get() && transaction.live.get());
    finish.set(true);
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert_eq!(db.releases.get(), 0);
    assert!(db.live.get() && transaction.live.get());
    let log = trace.borrow().clone();
    assert_eq!(
        &log[..5],
        &[
            "database acquired",
            "transaction acquired",
            "activity start",
            "activity joined",
            "activity dropped"
        ]
    );
    assert_eq!(log[5], "transaction release");
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert_eq!(
        trace
            .borrow()
            .iter()
            .filter(|e| e.as_str() == "activity dropped")
            .count(),
        1
    );
    transaction.allow_release.set(true);
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert!(!db.live.get() && !transaction.live.get());
    let log = trace.borrow().clone();
    let transaction_drop = log
        .iter()
        .position(|e| e == "transaction descriptor dropped")
        .unwrap();
    let database_release = log.iter().position(|e| e == "database release").unwrap();
    assert!(transaction_drop < database_release);
    assert_eq!((db.drops.get(), transaction.drops.get()), (1, 1));
    assert!(app.report().unwrap().primary.is_none());
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!(*trace.borrow(), log);
}

#[test]
fn partial_acquisition_preserves_primary_retries_cleanup_and_skips_later_acquisition() {
    let trace = Trace::default();
    let (db, failed, later) = (
        Rc::new(State::default()),
        Rc::new(State::default()),
        Rc::new(State::default()),
    );
    for state in [&db, &failed, &later] {
        state.allow_release.set(true);
    }
    let (starts, acquisitions) = (Cell::new(0), Cell::new(0));
    let finish = Cell::new(true);
    let mut partial = resource("partial", &failed, &trace, &acquisitions);
    partial.fail_acquire = true;
    partial.retry_error = true;
    let group = ResourceGroup::new("custom", activity(&trace, vec![], &starts, &finish))
        .with_managed_resource("database", resource("database", &db, &trace, &acquisitions))
        .with_managed_resource("partial", partial)
        .with_managed_resource("later", resource("later", &later, &trace, &acquisitions));
    let mut app = Application::new().with_host(host(group)).build().unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    let primary = app.primary_failure().unwrap();
    assert_eq!(
        (primary.host, primary.resource, primary.operation),
        ("custom", Some("partial"), "acquire")
    );
    assert_eq!((starts.get(), acquisitions.get()), (0, 2));
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert!(db.live.get() && failed.live.get());
    assert_eq!(
        (
            db.releases.get(),
            later.acquired.get(),
            later.releases.get()
        ),
        (0, 0, 0)
    );
    assert_eq!(later.drops.get(), 1);
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    let report = app.report().unwrap();
    assert!(std::ptr::eq(
        primary.error(),
        report.primary.as_ref().unwrap().error()
    ));
    assert_eq!(report.secondary.len(), 1);
    assert_eq!(report.secondary[0].operation, "release progress");
    assert_eq!(report.secondary[0].resource, Some("partial"));
    assert!(!db.live.get() && !failed.live.get());
    assert_eq!((failed.releases.get(), db.releases.get()), (2, 1));
}

#[test]
fn terminal_release_failure_does_not_retry_and_run_returns_after_dependency_cleanup() {
    let trace = Trace::default();
    let (db, last) = (Rc::new(State::default()), Rc::new(State::default()));
    db.allow_release.set(true);
    last.allow_release.set(true);
    let (starts, acquisitions) = (Cell::new(0), Cell::new(0));
    let finish = Cell::new(true);
    let mut terminal = resource("last", &last, &trace, &acquisitions);
    terminal.terminal_error = true;
    let group = ResourceGroup::new(
        "custom",
        activity(&trace, vec![db.clone(), last.clone()], &starts, &finish),
    )
    .with_managed_resource("database", resource("database", &db, &trace, &acquisitions))
    .with_managed_resource("last", terminal);
    let error = Application::new()
        .with_host(host(group))
        .build()
        .unwrap()
        .run()
        .unwrap_err();
    assert_eq!(
        error.report.primary.as_ref().unwrap().resource,
        Some("last")
    );
    assert_eq!(error.report.primary.as_ref().unwrap().operation, "release");
    assert!(error.report.secondary.is_empty());
    assert_eq!((db.releases.get(), last.releases.get()), (1, 1));
    assert!(!db.live.get() && !last.live.get());
    assert_eq!((db.drops.get(), last.drops.get()), (1, 1));
}

#[test]
fn rejected_or_prestart_cancelled_inert_descriptors_acquire_nothing() {
    let trace = Trace::default();
    let state = Rc::new(State::default());
    let (starts, acquisitions) = (Cell::new(0), Cell::new(0));
    let finish = Cell::new(true);
    let group = || {
        ResourceGroup::new("custom", activity(&trace, vec![], &starts, &finish))
            .with_managed_resource(
                "database",
                resource("database", &state, &trace, &acquisitions),
            )
    };
    assert!(
        Application::new()
            .with_host(CustomHost {
                group: group(),
                valid: false
            })
            .build()
            .is_err()
    );
    let app = Application::new().with_host(host(group()));
    app.shutdown_handle().request_shutdown();
    app.build().unwrap().run().unwrap();
    assert_eq!(
        (
            starts.get(),
            acquisitions.get(),
            state.releases.get(),
            state.drops.get()
        ),
        (0, 0, 0, 2)
    );
}

#[test]
fn cancellation_during_acquisition_skips_the_dependent_activity_and_later_resource() {
    let trace = Trace::default();
    let (first, later) = (Rc::new(State::default()), Rc::new(State::default()));
    first.allow_release.set(true);
    let (starts, acquisitions) = (Cell::new(0), Cell::new(0));
    let finish = Cell::new(true);
    let app = Application::new();
    let mut cancelling = resource("first", &first, &trace, &acquisitions);
    cancelling.stop_on_acquire = Some(app.shutdown_handle());
    let group = ResourceGroup::new("custom", activity(&trace, vec![], &starts, &finish))
        .with_managed_resource("first", cancelling)
        .with_managed_resource("later", resource("later", &later, &trace, &acquisitions));
    let mut app = app.with_host(host(group)).build().unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    assert_eq!(
        (starts.get(), acquisitions.get(), later.acquired.get()),
        (0, 1, 0)
    );
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!((first.releases.get(), later.releases.get()), (1, 0));
    assert!(!first.live.get());
    assert!(app.report().unwrap().primary.is_none());
}

struct FutureResource<'a> {
    future: Option<Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>>,
    ready: &'a Cell<bool>,
    live: &'a Cell<bool>,
    creations: &'a Cell<usize>,
    polls: &'a Cell<usize>,
    completed: &'a Cell<usize>,
    wake: Rc<RefCell<Option<Waker>>>,
}
impl ManagedResource for FutureResource<'_> {
    fn acquire(&mut self) -> io::Result<()> {
        self.live.set(true);
        Ok(())
    }
    fn release_within(&mut self, _: Duration) -> io::Result<ReleaseCompletion<io::Error>> {
        if self.future.is_none() {
            self.creations.set(self.creations.get() + 1);
            let (ready, live, polls) = (self.ready, self.live, self.polls);
            let wake = self.wake.clone();
            self.future = Some(Box::pin(poll_fn(move |cx| {
                polls.set(polls.get() + 1);
                if ready.get() {
                    live.set(false);
                    Poll::Ready(Ok(()))
                } else {
                    *wake.borrow_mut() = Some(cx.waker().clone());
                    Poll::Pending
                }
            })));
        }
        // This test explicitly drives its owned future. Production adapters must
        // integrate their own executor/driver and wake strategy.
        match self
            .future
            .as_mut()
            .unwrap()
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Pending => Ok(ReleaseCompletion::Pending),
            Poll::Ready(result) => {
                drop(self.future.take());
                self.completed.set(self.completed.get() + 1);
                Ok(ReleaseCompletion::Released(result))
            }
        }
    }
}

#[test]
fn original_local_borrowing_cleanup_future_survives_pending_and_is_never_polled_after_ready() {
    let trace = Trace::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let (ready, live) = (Cell::new(false), Cell::new(false));
    let (creations, polls, completed) = (Cell::new(0), Cell::new(0), Cell::new(0));
    let wake = Rc::new(RefCell::new(None));
    let resource = FutureResource {
        future: None,
        ready: &ready,
        live: &live,
        creations: &creations,
        polls: &polls,
        completed: &completed,
        wake: wake.clone(),
    };
    let group = ResourceGroup::new("custom", activity(&trace, vec![], &starts, &finish))
        .with_managed_resource("future", resource);
    let mut app = Application::new().with_host(host(group)).build().unwrap();
    assert_eq!(app.start(), StartStatus::Started);
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert!(live.get());
    assert_eq!((creations.get(), polls.get(), completed.get()), (1, 1, 0));
    ready.set(true);
    wake.borrow_mut()
        .take()
        .expect("future registered wake")
        .wake();
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!((creations.get(), polls.get(), completed.get()), (1, 2, 1));
    assert!(!live.get());
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!(polls.get(), 2);
    assert!(app.report().unwrap().primary.is_none());
}
