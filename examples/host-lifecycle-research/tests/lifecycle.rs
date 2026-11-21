use axum::{Router, routing::get};
use std::{
    cell::{Cell, RefCell},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Notify;
use yetlet::composition::JoinStatus;
use yetlet_host_lifecycle_research::{
    Activity, Application, Completion, Host, ResourceGroup, ShutdownHandle, StartStatus, WebHost,
    WebStatus, WorkerCompletion,
};

fn router() -> Router {
    Router::new().route("/", get(|| async { "hello" }))
}

fn http_get(address: SocketAddr, path: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn wait_address(status: &WebStatus) -> SocketAddr {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if status.admission_open() {
            return status.endpoint().expect("admitted listener has an address");
        }
        assert!(Instant::now() < deadline, "Web admission never opened");
        thread::sleep(Duration::from_millis(2));
    }
}

struct StopOnDrop(ShutdownHandle);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

#[test]
fn single_web_terminal_run_serves_and_joins_original_worker() {
    let host = WebHost::new(router());
    let status = host.status();
    let app = Application::new().with_host(host);
    assert!(status.endpoint().is_none());
    let stop = app.shutdown_handle();
    let mut bound = None;
    let report = thread::scope(|scope| {
        let observer = status.clone();
        let client = scope.spawn(move || {
            let _stop = StopOnDrop(stop);
            let address = wait_address(&observer);
            let response = http_get(address, "/").unwrap();
            assert!(response.starts_with("HTTP/1.1 200"));
            assert!(response.ends_with("hello"));
            address
        });
        let report = app.build().unwrap().run().unwrap();
        bound = Some(client.join().unwrap());
        report
    });
    assert_eq!(report.hosts.len(), 1);
    assert_eq!(report.hosts[0].join, JoinStatus::Succeeded);
    assert!(report.primary.is_none());
    assert!(status.joined());
    assert!(!status.admission_open());
    let rebound = TcpListener::bind(bound.unwrap()).unwrap();
    drop(rebound);
}

#[test]
fn two_web_hosts_are_independent_listeners_and_both_join() {
    let first = WebHost::new(router());
    let second = WebHost::new(router());
    let one = first.status();
    let two = second.status();
    let app = Application::new().with_host(first).with_host(second);
    let stop = app.shutdown_handle();
    let report = thread::scope(|scope| {
        let (one, two) = (one.clone(), two.clone());
        let client = scope.spawn(move || {
            let _stop = StopOnDrop(stop);
            let (a, b) = (wait_address(&one), wait_address(&two));
            assert_ne!(a, b);
            assert!(http_get(a, "/").unwrap().ends_with("hello"));
            assert!(http_get(b, "/").unwrap().ends_with("hello"));
        });
        let report = app.build().unwrap().run().unwrap();
        client.join().unwrap();
        report
    });
    assert_eq!(report.hosts.len(), 2);
    assert!(
        report
            .hosts
            .iter()
            .all(|host| host.join == JoinStatus::Succeeded)
    );
    assert!(one.joined() && two.joined());
    let _first = TcpListener::bind(one.endpoint().unwrap()).unwrap();
    let _second = TcpListener::bind(two.endpoint().unwrap()).unwrap();
}

#[test]
fn second_bind_failure_rolls_back_first_worker_and_preserves_external_listener() {
    let external = TcpListener::bind("127.0.0.1:0").unwrap();
    let occupied = external.local_addr().unwrap();
    let first = WebHost::new(router());
    let first_status = first.status();
    let second = WebHost::new(router()).named("second web").bind(occupied);
    let second_status = second.status();
    let error = Application::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap()
        .run()
        .unwrap_err();
    assert_eq!(
        error.report.primary.as_ref().unwrap().error().kind(),
        io::ErrorKind::AddrInUse
    );
    assert!(error.report.secondary.is_empty());
    assert!(first_status.joined());
    assert!(second_status.endpoint().is_none());
    assert!(
        !second_status.joined(),
        "no second worker was ever acquired"
    );
    assert_eq!(error.report.primary.as_ref().unwrap().host, "second web");
    let _rebound = TcpListener::bind(first_status.endpoint().unwrap()).unwrap();
    assert_eq!(external.local_addr().unwrap(), occupied);
    assert!(TcpListener::bind(occupied).is_err());
}

#[test]
fn readiness_failure_keeps_its_own_acquired_worker_until_terminal_cleanup() {
    let host = WebHost::new(Router::new().route(
        "/",
        get(|| async { axum::http::StatusCode::SERVICE_UNAVAILABLE }),
    ))
    .readiness_probe("/", Duration::from_millis(30));
    let status = host.status();
    let error = Application::new()
        .with_host(host)
        .build()
        .unwrap()
        .run()
        .unwrap_err();
    assert_eq!(
        error.report.primary.as_ref().unwrap().operation,
        "readiness"
    );
    assert!(status.joined());
    assert!(!status.admission_open());
    let _rebound = TcpListener::bind(status.endpoint().unwrap()).unwrap();
}

struct CustomHost<A, D>(ResourceGroup<A, D>);
impl<A: Activity, D: yetlet_host_lifecycle_research::ResourcePlan> Host for CustomHost<A, D> {
    type Activity = A;
    type Resources = D;
    fn into_resources(self) -> ResourceGroup<A, D> {
        self.0
    }
}

type Log = Rc<RefCell<Vec<&'static str>>>;

struct DropTrace {
    label: &'static str,
    log: Log,
}
impl Drop for DropTrace {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.label);
    }
}

struct LocalActivity<'a> {
    log: Log,
    starts: &'a Cell<usize>,
    can_finish: &'a Cell<bool>,
    can_close: Option<&'a Cell<bool>>,
    fail_readiness: bool,
    check_closed_web: Option<WebStatus>,
    stop_on_readiness: Option<ShutdownHandle>,
    stop_on_open: Option<ShutdownHandle>,
    fail_signal_once: bool,
    fail_observe_once: bool,
    joined: bool,
}
impl<'a> LocalActivity<'a> {
    fn new(log: Log, starts: &'a Cell<usize>, can_finish: &'a Cell<bool>) -> Self {
        Self {
            log,
            starts,
            can_finish,
            can_close: None,
            fail_readiness: false,
            check_closed_web: None,
            stop_on_readiness: None,
            stop_on_open: None,
            fail_signal_once: false,
            fail_observe_once: false,
            joined: false,
        }
    }
}
impl Activity for LocalActivity<'_> {
    fn start(&mut self) -> io::Result<()> {
        self.starts.set(self.starts.get() + 1);
        self.log.borrow_mut().push("start");
        Ok(())
    }
    fn wait_ready(&mut self) -> io::Result<()> {
        self.log.borrow_mut().push("ready");
        if let Some(stop) = &self.stop_on_readiness {
            stop.request_shutdown();
        }
        if let Some(web) = &self.check_closed_web {
            assert!(!web.admission_open());
            assert!(http_get(web.endpoint().unwrap(), "/")?.starts_with("HTTP/1.1 503"));
        }
        if self.fail_readiness {
            Err(io::Error::other("original readiness failure"))
        } else {
            Ok(())
        }
    }
    fn open_admission(&mut self) -> io::Result<()> {
        self.log.borrow_mut().push("open admission");
        if let Some(stop) = &self.stop_on_open {
            stop.request_shutdown();
        }
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.can_finish.get()
    }
    fn close_admission(&mut self) -> io::Result<()> {
        if self.can_close.is_some_and(|allowed| !allowed.get()) {
            return Err(io::Error::other("admission fence failed"));
        }
        self.log.borrow_mut().push("close admission");
        Ok(())
    }
    fn request_shutdown(&mut self) -> io::Result<()> {
        self.log.borrow_mut().push("signal");
        if std::mem::take(&mut self.fail_signal_once) {
            Err(io::Error::other("secondary signal failure"))
        } else {
            Ok(())
        }
    }
    fn observe_within(&mut self, _: Duration) -> io::Result<WorkerCompletion<io::Error>> {
        if std::mem::take(&mut self.fail_observe_once) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "secondary observation failure",
            ));
        }
        if !self.can_finish.get() {
            return Ok(WorkerCompletion::Pending);
        }
        assert!(!self.joined, "original activity observed after Joined");
        self.joined = true;
        self.log.borrow_mut().push("joined");
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}
impl Drop for LocalActivity<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push("activity dropped");
    }
}

#[test]
fn custom_host_uses_local_borrows_and_releases_dependencies_after_join_in_reverse_order() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let first = DropTrace {
        label: "first dropped",
        log: log.clone(),
    };
    let last = DropTrace {
        label: "last dropped",
        log: log.clone(),
    };
    let host = CustomHost(
        ResourceGroup::new("custom", LocalActivity::new(log.clone(), &starts, &finish))
            .with_resource(first)
            .with_resource(last)
            .with_resource(&starts),
    );
    let report = Application::new()
        .with_host(host)
        .build()
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(starts.get(), 1);
    assert!(report.primary.is_none());
    assert_eq!(
        &*log.borrow(),
        &[
            "start",
            "ready",
            "open admission",
            "close admission",
            "signal",
            "joined",
            "activity dropped",
            "last dropped",
            "first dropped",
        ]
    );
    starts.set(2); // Borrowed caller state remains usable and caller-owned.
}

#[test]
fn failed_start_pending_cleanup_retains_original_primary_and_every_dependency() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(false);
    let mut activity = LocalActivity::new(log.clone(), &starts, &finish);
    activity.fail_readiness = true;
    activity.fail_signal_once = true;
    activity.fail_observe_once = true;
    let host = CustomHost(
        ResourceGroup::new("custom", activity).with_resource(DropTrace {
            label: "dependency dropped",
            log: log.clone(),
        }),
    );
    let mut app = Application::new().with_host(host).build().unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    let primary = app.primary_failure().unwrap();
    assert_eq!(primary.operation, "readiness");
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Pending);
    assert!(app.report().is_none());
    assert!(!log.borrow().contains(&"dependency dropped"));
    assert!(!log.borrow().contains(&"activity dropped"));
    assert_eq!(
        app.primary_failure().unwrap().error().to_string(),
        primary.error().to_string()
    );
    finish.set(true);
    assert_eq!(
        app.shutdown_until(Instant::now() + Duration::from_secs(1)),
        Completion::Complete
    );
    let report = app.report().unwrap();
    assert_eq!(report.primary.as_ref().unwrap().operation, "readiness");
    assert_eq!(report.secondary.len(), 2);
    assert_eq!(report.secondary[0].operation, "shutdown");
    assert_eq!(report.secondary[1].operation, "observe");
    assert_eq!(report.hosts[0].join, JoinStatus::Succeeded);
    let before = log.borrow().clone();
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!(*log.borrow(), before);
    assert_eq!(
        &before[before.len() - 3..],
        &["joined", "activity dropped", "dependency dropped"]
    );
}

struct SharedDrop(Arc<AtomicBool>);
impl Drop for SharedDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

#[test]
fn admitted_http_request_drains_before_dependency_release_and_pending_keeps_owner() {
    let released = Arc::new(AtomicBool::new(false));
    let release = Arc::new(Notify::new());
    let (entered_tx, entered_rx) = mpsc::channel();
    let host = WebHost::new(Router::new().route("/", get(|| async { "ready" })).route(
        "/slow",
        get({
            let (release, released) = (release.clone(), released.clone());
            move || {
                let (release, released, entered_tx) =
                    (release.clone(), released.clone(), entered_tx.clone());
                async move {
                    entered_tx.send(()).unwrap();
                    release.notified().await;
                    assert!(
                        !released.load(Ordering::Acquire),
                        "dependency released while handler was alive"
                    );
                    "drained"
                }
            }
        }),
    ))
    .with_resource(SharedDrop(released.clone()));
    let status = host.status();
    let mut app = Application::new().with_host(host).build().unwrap();
    assert_eq!(app.start(), StartStatus::Started);
    thread::scope(|scope| {
        let client = scope.spawn(|| http_get(status.endpoint().unwrap(), "/slow").unwrap());
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let pending = app.shutdown_until(Instant::now());
        let retained = !released.load(Ordering::Acquire) && !status.joined();
        // Always unblock the handler before assertions that might unwind a scope.
        release.notify_one();
        assert_eq!(pending, Completion::Pending);
        assert!(retained);
        assert_eq!(
            app.shutdown_until(Instant::now() + Duration::from_secs(3)),
            Completion::Complete
        );
        assert!(client.join().unwrap().ends_with("drained"));
    });
    assert!(released.load(Ordering::Acquire));
    assert!(status.joined());
    assert!(app.report().unwrap().primary.is_none());
}

#[test]
fn failed_other_admission_fence_still_closes_web_work_and_retains_all_owners() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let can_close = Cell::new(false);
    let mut local = LocalActivity::new(log.clone(), &starts, &finish);
    local.can_close = Some(&can_close);
    let web = WebHost::new(router());
    let status = web.status();
    let mut app = Application::new()
        .with_host(web)
        .with_host(CustomHost(ResourceGroup::new("custom", local)))
        .build()
        .unwrap();
    assert_eq!(app.start(), StartStatus::Started);
    app.request_shutdown();
    assert!(!status.admission_open());
    assert!(!status.joined());
    assert!(
        http_get(status.endpoint().unwrap(), "/")
            .unwrap()
            .starts_with("HTTP/1.1 503")
    );
    assert!(!log.borrow().contains(&"signal"));
    can_close.set(true);
    assert_eq!(
        app.shutdown_until(Instant::now() + Duration::from_secs(2)),
        Completion::Complete
    );
    assert_eq!(
        app.report().unwrap().primary.as_ref().unwrap().operation,
        "admission"
    );
    assert!(status.joined());
}

#[test]
fn invalid_plan_and_configuration_drop_never_start_activities_or_lose_owned_resources() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let invalid = WebHost::new(router()).readiness_probe("invalid", Duration::from_secs(1));
    let status = invalid.status();
    let custom = CustomHost(
        ResourceGroup::new("custom", LocalActivity::new(log.clone(), &starts, &finish))
            .with_resource(DropTrace {
                label: "invalid plan resource dropped",
                log: log.clone(),
            }),
    );
    assert!(
        Application::new()
            .with_host(custom)
            .with_host(invalid)
            .build()
            .is_err()
    );
    assert_eq!(starts.get(), 0);
    assert!(status.endpoint().is_none());
    assert_eq!(
        &*log.borrow(),
        &["activity dropped", "invalid plan resource dropped"]
    );
    let unstarted = WebHost::new(router()).with_resource(DropTrace {
        label: "unstarted resource dropped",
        log: log.clone(),
    });
    let status = unstarted.status();
    drop(Application::new().with_host(unstarted).build().unwrap());
    assert!(status.endpoint().is_none());
    assert_eq!(
        log.borrow()
            .iter()
            .filter(|&&e| e == "unstarted resource dropped")
            .count(),
        1
    );

    let drop_order = Log::default();
    let unstarted_custom = CustomHost(
        ResourceGroup::new(
            "unstarted custom",
            LocalActivity::new(drop_order.clone(), &starts, &finish),
        )
        .with_resource(DropTrace {
            label: "base dropped",
            log: drop_order.clone(),
        })
        .with_resource(DropTrace {
            label: "dependent dropped",
            log: drop_order.clone(),
        }),
    );
    let built = Application::new()
        .with_host(unstarted_custom)
        .build()
        .unwrap();
    assert_eq!(starts.get(), 0);
    assert!(drop_order.borrow().is_empty());
    drop(built);
    assert_eq!(starts.get(), 0);
    assert_eq!(
        &*drop_order.borrow(),
        &["activity dropped", "dependent dropped", "base dropped"]
    );
}

#[test]
fn work_stays_closed_during_startup_and_failure_skips_later_hosts() {
    let log = Log::default();
    let starts = Cell::new(0);
    let later_starts = Cell::new(0);
    let finish = Cell::new(true);
    let first = WebHost::new(router());
    let status = first.status();
    let mut failing = LocalActivity::new(log.clone(), &starts, &finish);
    failing.check_closed_web = Some(status.clone());
    failing.fail_readiness = true;
    let later = LocalActivity::new(log, &later_starts, &finish);
    let error = Application::new()
        .with_host(first)
        .with_host(CustomHost(ResourceGroup::new("failing", failing)))
        .with_host(CustomHost(ResourceGroup::new("later", later)))
        .build()
        .unwrap()
        .run()
        .unwrap_err();
    assert_eq!(starts.get(), 1);
    assert_eq!(later_starts.get(), 0);
    assert_eq!(error.report.primary.as_ref().unwrap().host, "failing");
    assert!(!error.report.hosts[2].start_attempted);
    assert!(!status.admission_open());
    assert!(status.joined());
}

#[test]
fn shutdown_during_last_readiness_does_not_open_work_admission() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let web = WebHost::new(router());
    let status = web.status();
    let app = Application::new().with_host(web);
    let mut activity = LocalActivity::new(log.clone(), &starts, &finish);
    activity.stop_on_readiness = Some(app.shutdown_handle());
    let mut app = app
        .with_host(CustomHost(ResourceGroup::new("custom", activity)))
        .build()
        .unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    assert!(!status.admission_open());
    assert!(!log.borrow().contains(&"open admission"));
    assert_eq!(
        app.shutdown_until(Instant::now() + Duration::from_secs(2)),
        Completion::Complete
    );
    assert!(status.joined());
    assert!(app.report().unwrap().primary.is_none());
}

#[test]
fn shutdown_from_final_admission_closes_already_open_web_gate_before_stopping_returns() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let web = WebHost::new(router());
    let status = web.status();
    let app = Application::new().with_host(web);
    let mut activity = LocalActivity::new(log.clone(), &starts, &finish);
    activity.stop_on_open = Some(app.shutdown_handle());
    let mut app = app
        .with_host(CustomHost(ResourceGroup::new("custom", activity)))
        .build()
        .unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    assert!(log.borrow().contains(&"open admission"));
    assert!(log.borrow().contains(&"close admission"));
    assert!(
        status.endpoint().is_some(),
        "Web acquired its original worker"
    );
    assert!(
        !status.admission_open(),
        "Stopping must already close earlier gates"
    );
    assert!(
        !status.joined(),
        "the original join remains with this advanced owner"
    );
    assert_eq!(
        app.shutdown_until(Instant::now() + Duration::from_secs(2)),
        Completion::Complete
    );
    assert!(status.joined());
    assert!(app.report().unwrap().primary.is_none());
}

#[test]
fn four_host_shape_and_shutdown_before_start_preserve_local_lifetimes() {
    let log = Log::default();
    let starts = Cell::new(0);
    let finish = Cell::new(true);
    let host = || {
        CustomHost(ResourceGroup::new(
            "custom",
            LocalActivity::new(log.clone(), &starts, &finish),
        ))
    };
    let app = Application::new()
        .with_host(host())
        .with_host(host())
        .with_host(host())
        .with_host(host());
    app.shutdown_handle().request_shutdown();
    let report = app.build().unwrap().run().unwrap();
    assert_eq!(starts.get(), 0);
    assert_eq!(report.hosts.len(), 4);
    assert!(
        report
            .hosts
            .iter()
            .all(|host| !host.start_attempted && host.join == JoinStatus::Succeeded)
    );
    assert_eq!(
        log.borrow()
            .iter()
            .filter(|&&e| e == "activity dropped")
            .count(),
        4
    );
}
