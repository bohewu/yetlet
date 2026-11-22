#![cfg(not(target_arch = "wasm32"))]
//! Public Web lifecycle proofs; the isolated consumer checks normal feature closure.

use std::{
    cell::{Cell, RefCell},
    fmt, io,
    net::{SocketAddr, TcpListener, TcpStream},
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    thread::{self, ThreadId},
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::State,
    http::{HeaderName, HeaderValue, Method, StatusCode},
    routing::{get, post},
};
use reqwest::Client;
use tokio::{
    sync::Notify,
    time::{sleep, timeout},
};
use tower_http::set_header::SetResponseHeaderLayer;
use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, ExclusiveClaim, HostActivity, HostBuildError, HostOperation, HostRunMode,
        JoinStatus, Managed, ManagedResource, Phase, ReleaseCompletion, Retained, ShutdownHandle,
        WorkerCompletion,
    },
    web::{App, HttpProbe, WebActivity, WebDriver, WebError, WebHandle, WebHost},
};

const LIMIT: Duration = Duration::from_secs(5);

struct StopOnDrop(ShutdownHandle);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

struct UnblockOnDrop(Arc<Notify>);
impl Drop for UnblockOnDrop {
    fn drop(&mut self) {
        self.0.notify_one();
    }
}

fn client() -> Client {
    Client::builder().no_proxy().timeout(LIMIT).build().unwrap()
}

async fn admitted(handle: &WebHandle) -> SocketAddr {
    timeout(LIMIT, async {
        while !handle.is_admitting() {
            sleep(Duration::from_millis(1)).await;
        }
        assert!(handle.is_ready());
        handle.local_addr().expect("admitted listener")
    })
    .await
    .expect("Web admission deadline")
}

async fn wait_until(mut predicate: impl FnMut() -> bool) {
    timeout(LIMIT, async {
        while !predicate() {
            sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("lifecycle observation deadline");
}

fn url(address: SocketAddr, path: &str) -> String {
    format!("http://{address}{path}")
}

fn api(name: &'static str) -> WebHost {
    App::new()
        .get("/api", move || async move { name })
        .into_host()
        .with_name(name)
        .with_address("127.0.0.1:0")
}

#[test]
fn configuration_is_inert_and_duplicate_default_claims_fail_before_acquisition() {
    let first = WebHost::new(Router::new()).with_name("first");
    let second = App::new().into_host().with_name("second");
    assert_eq!(first.address(), "127.0.0.1:7878");
    assert_eq!(second.address(), "127.0.0.1:7878");
    let one = first.handle();
    let two = second.handle();
    let result = HostGroup::new().with_host(first).with_host(second).build();
    assert!(matches!(
        result,
        Err(HostBuildError::ConflictingClaim { .. })
    ));
    for handle in [one, two] {
        assert!(handle.local_addr().is_none());
        assert!(!handle.is_ready() && !handle.is_admitting());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn caller_runtime_serves_two_api_only_hosts_and_keeps_local_timers_progressing() {
    let caller = thread::current().id();
    let threads = Arc::new(Mutex::new(Vec::<ThreadId>::new()));
    let observed = threads.clone();
    let first = App::new()
        .get("/api", move |State(body): State<&'static str>| {
            let observed = observed.clone();
            async move {
                sleep(Duration::from_millis(2)).await;
                observed.lock().unwrap().push(thread::current().id());
                body
            }
        })
        .with_state("first from state")
        .map_router(|router| {
            router.layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-managed-middleware"),
                HeaderValue::from_static("preserved"),
            ))
        })
        .into_host()
        .with_name("first")
        .with_address("127.0.0.1:0");
    let second = api("second");
    let one = first.handle();
    let two = second.handle();
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap();
    assert!(one.local_addr().is_none() && two.local_addr().is_none());
    let shutdown = hosts.shutdown_handle();
    let timer_progress = Cell::new(false);
    let inspect = async {
        let _stop = StopOnDrop(shutdown);
        let a = admitted(&one).await;
        let b = admitted(&two).await;
        assert_ne!(a, b);
        let client = client();
        let (response, ()) = tokio::join!(client.get(url(a, "/api")).send(), async {
            sleep(Duration::from_millis(1)).await;
            timer_progress.set(true);
        });
        let response = response.unwrap();
        assert_eq!(response.headers()["x-managed-middleware"], "preserved");
        assert_eq!(response.text().await.unwrap(), "first from state");
        assert_eq!(
            client
                .get(url(b, "/api"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
            "second"
        );
        assert_eq!(
            client.get(url(a, "/")).send().await.unwrap().status(),
            StatusCode::NOT_FOUND
        );
    };
    let (run, ()) = timeout(LIMIT, async { tokio::join!(hosts.run_async(), inspect) })
        .await
        .unwrap();
    run.unwrap();
    assert!(timer_progress.get());
    assert_eq!(&*threads.lock().unwrap(), &[caller]);
    assert!(
        hosts
            .outcomes()
            .iter()
            .all(|host| host.join == JoinStatus::Succeeded)
    );
    assert!(!one.is_ready() && !two.is_ready());
    let _first = TcpListener::bind(one.local_addr().unwrap()).unwrap();
    let _second = TcpListener::bind(two.local_addr().unwrap()).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_probe_resumes_the_same_server_and_preserves_its_exact_admission_exception() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_entered = entered.clone();
    let probe_release = release.clone();
    let observed_probes = probe_calls.clone();
    let work_calls = calls.clone();
    let router = Router::new()
        .route(
            "/ready",
            post(move || {
                let entered = probe_entered.clone();
                let release = probe_release.clone();
                let observed = observed_probes.clone();
                async move {
                    observed.fetch_add(1, Ordering::SeqCst);
                    entered.notify_one();
                    release.notified().await;
                    StatusCode::NO_CONTENT
                }
            }),
        )
        .route(
            "/work",
            get(move || {
                let calls = work_calls.clone();
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    "allowed"
                }
            }),
        );
    let host = WebHost::new(router)
        .with_address("127.0.0.1:0")
        .with_http_probe(
            HttpProbe::new(
                Method::POST,
                "/ready?token=expected",
                StatusCode::NO_CONTENT,
            )
            .with_timeout(LIMIT),
        );
    let handle = host.handle();
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    let _unblock = UnblockOnDrop(release.clone());
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("probe ended before release: {result:?}"),
            () = entered.notified() => {}
        }
    })
    .await
    .unwrap();
    // Only the borrowing drive was dropped; the original probe and server remain.
    let address = handle.local_addr().expect("probe's acquired listener");
    assert!(!handle.is_ready() && !handle.is_admitting());
    assert_eq!(probe_calls.load(Ordering::SeqCst), 1);
    let shutdown = hosts.shutdown_handle();
    let inspect = async {
        let _stop = StopOnDrop(shutdown);
        let client = client();
        for request in [
            client.get(url(address, "/work")),
            client.get(url(address, "/ready?token=expected")),
            client.post(url(address, "/ready?token=other")),
        ] {
            assert_eq!(
                request.send().await.unwrap().status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(probe_calls.load(Ordering::SeqCst), 1);
        assert_eq!(handle.local_addr(), Some(address));
        release.notify_one();
        assert_eq!(admitted(&handle).await, address);
        assert_eq!(
            client
                .get(url(address, "/work"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
            "allowed"
        );
    };
    let (run, ()) = timeout(LIMIT, async { tokio::join!(hosts.run_async(), inspect) })
        .await
        .unwrap();
    run.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(probe_calls.load(Ordering::SeqCst), 1);
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_probe_rejects_a_matching_response_delivered_after_its_original_deadline() {
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let returning_status = Arc::new(Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let handler_entered = entered.clone();
    let handler_release = release.clone();
    let handler_returning = returning_status.clone();
    let handler_calls = calls.clone();
    let router = Router::new().route(
        "/ready",
        get(move || {
            let entered = handler_entered.clone();
            let release = handler_release.clone();
            let returning = handler_returning.clone();
            let calls = handler_calls.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                entered.notify_one();
                release.notified().await;
                returning.notify_one();
                StatusCode::NO_CONTENT
            }
        }),
    );
    let probe_timeout = Duration::from_millis(100);
    let host = WebHost::new(router)
        .with_name("expired probe")
        .with_address("127.0.0.1:0")
        .with_driver(WebDriver::CallerRuntime)
        .with_http_probe(
            HttpProbe::new(Method::GET, "/ready", StatusCode::NO_CONTENT)
                .with_timeout(probe_timeout),
        );
    let handle = host.handle();
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    let _unblock = UnblockOnDrop(release.clone());
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("probe ended before its handler was held: {result:?}"),
            () = entered.notified() => {}
        }
    })
    .await
    .unwrap();
    let address = handle.local_addr().expect("original probe listener");
    assert!(!handle.is_ready() && !handle.is_admitting());

    // The original deadline started before the handler entered. Keep Tokio and
    // the admitted connection alive, but do not poll the retained group drive.
    sleep(probe_timeout + Duration::from_millis(20)).await;
    release.notify_one();
    timeout(LIMIT, returning_status.notified()).await.unwrap();
    // Let the existing connection flush its matching response before resuming
    // the suspended probe exchange; no replacement probe or server is created.
    sleep(Duration::from_millis(5)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(handle.local_addr(), Some(address));
    assert!(!handle.is_ready() && !handle.is_admitting());

    let incorrect_admission = Cell::new(false);
    let result = timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => result.map_err(|failure| failure.error().clone()),
            () = async {
                while !handle.is_admitting() {
                    sleep(Duration::from_millis(1)).await;
                }
                incorrect_admission.set(true);
            } => Ok(()),
        }
    })
    .await
    .unwrap();
    if incorrect_admission.get() {
        // A regression must still finish the original owner before asserting.
        hosts.request_shutdown();
        let _ = timeout(LIMIT, hosts.run_async()).await.unwrap();
    }
    assert!(
        !incorrect_admission.get(),
        "an expired probe response opened ordinary admission"
    );
    let original = result.expect_err("the retained absolute deadline must expire");
    assert_eq!(original.kind(), io::ErrorKind::TimedOut);
    let primary = hosts.failures().primary().unwrap();
    assert_eq!(primary.host, "expired probe");
    assert_eq!(primary.operation, HostOperation::Readiness);
    assert!(std::ptr::eq(
        primary.error().io_error(),
        original.io_error()
    ));
    let secondary = hosts.failures().secondary();
    assert_eq!(secondary.len(), 1);
    assert_eq!(secondary[0].operation, HostOperation::Join);
    assert!(std::ptr::eq(
        secondary[0].error().io_error(),
        original.io_error()
    ));
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Failed);
    assert_eq!(hosts.phase(), Phase::Failed);
    assert_eq!(handle.local_addr(), Some(address));
    assert!(!handle.is_ready() && !handle.is_admitting());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let _rebound = TcpListener::bind(address).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn failed_probe_rolls_back_its_acquired_server_without_opening_work() {
    let host = WebHost::new(
        Router::new().route("/ready", get(|| async { StatusCode::SERVICE_UNAVAILABLE })),
    )
    .with_name("probe")
    .with_address("127.0.0.1:0")
    .with_http_probe(
        HttpProbe::new(Method::GET, "/ready", StatusCode::NO_CONTENT)
            .with_timeout(Duration::from_millis(40)),
    );
    let handle = host.handle();
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    let failure = timeout(LIMIT, hosts.run_async())
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(failure.host, "probe");
    assert_eq!(failure.operation, HostOperation::Readiness);
    let original = failure.error().clone();
    let primary = hosts.failures().primary().unwrap();
    assert!(std::ptr::eq(
        primary.error().io_error(),
        original.io_error()
    ));
    let secondary = hosts.failures().secondary();
    assert_eq!(secondary.len(), 1);
    assert_eq!(secondary[0].host, "probe");
    assert_eq!(secondary[0].operation, HostOperation::Join);
    assert!(std::ptr::eq(
        secondary[0].error().io_error(),
        original.io_error()
    ));
    assert!(!handle.is_ready() && !handle.is_admitting());
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Failed);
    assert_eq!(hosts.phase(), Phase::Failed);
    let _rebound = TcpListener::bind(handle.local_addr().unwrap()).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn second_bind_failure_preserves_external_owner_and_joins_the_first_server() {
    let external = TcpListener::bind("127.0.0.1:0").unwrap();
    let occupied = external.local_addr().unwrap();
    let first = api("first");
    let first_handle = first.handle();
    let second = api("second").with_address(occupied);
    let second_handle = second.handle();
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap();
    let failure = timeout(LIMIT, hosts.run_async())
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(failure.host, "second");
    assert_eq!(failure.error().kind(), io::ErrorKind::AddrInUse);
    assert!(!first_handle.is_admitting() && !second_handle.is_admitting());
    assert!(second_handle.local_addr().is_none());
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
    let _rebound = TcpListener::bind(first_handle.local_addr().unwrap()).unwrap();
    assert_eq!(external.local_addr().unwrap(), occupied);
    assert!(TcpListener::bind(occupied).is_err());
}

fn sync_request(address: SocketAddr) -> io::Result<String> {
    use std::io::{Read, Write};
    let mut stream = TcpStream::connect_timeout(&address, LIMIT)?;
    stream.set_read_timeout(Some(LIMIT))?;
    stream.set_write_timeout(Some(LIMIT))?;
    stream.write_all(b"GET /api HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn sync_admitted(handle: &WebHandle) -> io::Result<SocketAddr> {
    let deadline = Instant::now() + LIMIT;
    while !handle.is_admitting() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Web admission"));
        }
        thread::sleep(Duration::from_millis(1));
    }
    handle
        .local_addr()
        .ok_or_else(|| io::Error::other("missing address"))
}

#[test]
fn blocking_default_and_explicit_owned_driver_join_the_original_service_threads() {
    let caller = thread::current().id();
    let threads = Arc::new(Mutex::new(Vec::<ThreadId>::new()));
    let make = |name| {
        let threads = threads.clone();
        App::new()
            .get("/api", move || {
                let threads = threads.clone();
                async move {
                    threads.lock().unwrap().push(thread::current().id());
                    name
                }
            })
            .into_host()
            .with_name(name)
            .with_address("127.0.0.1:0")
    };
    let first = make("default owned");
    let second = make("explicit owned").with_driver(WebDriver::OwnedThread);
    let one = first.handle();
    let two = second.handle();
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap();
    let shutdown = hosts.shutdown_handle();
    let (run, requests) = thread::scope(|scope| {
        let requests = scope.spawn(|| -> io::Result<()> {
            let _stop = StopOnDrop(shutdown);
            for handle in [&one, &two] {
                let response = sync_request(sync_admitted(handle)?)?;
                if !response.starts_with("HTTP/1.1 200") {
                    return Err(io::Error::other("owned-thread HTTP response"));
                }
            }
            Ok(())
        });
        let run = hosts.run();
        (run, requests.join().unwrap())
    });
    run.unwrap();
    requests.unwrap();
    let observed = threads.lock().unwrap();
    assert_eq!(observed.len(), 2);
    assert!(observed.iter().all(|id| *id != caller));
    assert_ne!(observed[0], observed[1]);
    assert!(
        hosts
            .outcomes()
            .iter()
            .all(|host| host.join == JoinStatus::Succeeded)
    );
    assert!(!one.is_ready() && !two.is_ready());
    let _first = TcpListener::bind(one.local_addr().unwrap()).unwrap();
    let _second = TcpListener::bind(two.local_addr().unwrap()).unwrap();
}

#[test]
fn blocking_incompatible_driver_is_rejected_before_any_web_acquisition() {
    let first = api("would bind");
    let second = api("caller only").with_driver(WebDriver::CallerRuntime);
    let one = first.handle();
    let two = second.handle();
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap();
    assert!(hosts.run().is_err());
    let failure = hosts.failures().primary().expect("driver incompatibility");
    assert_eq!(failure.host, "caller only");
    assert_eq!(failure.operation, HostOperation::PrepareDriver);
    assert!(one.local_addr().is_none() && two.local_addr().is_none());
    assert!(!one.is_ready() && !two.is_ready());
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_owned_thread_join_yields_and_resumes_its_original_completion() {
    let caller = thread::current().id();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let observed = Arc::new(Mutex::new(Vec::<ThreadId>::new()));
    let handler_entered = entered.clone();
    let handler_release = release.clone();
    let handler_threads = observed.clone();
    let host = App::new()
        .get("/slow", move || {
            let entered = handler_entered.clone();
            let release = handler_release.clone();
            let threads = handler_threads.clone();
            async move {
                threads.lock().unwrap().push(thread::current().id());
                entered.notify_one();
                release.notified().await;
                "owned drain"
            }
        })
        .into_host()
        .with_address("127.0.0.1:0")
        .with_driver(WebDriver::OwnedThread);
    let handle = host.handle();
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    let _unblock = UnblockOnDrop(release.clone());
    let request_handle = handle.clone();
    let mut request = Box::pin(async move {
        let address = admitted(&request_handle).await;
        client()
            .get(url(address, "/slow"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    });
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("unexpected owned completion: {result:?}"),
            () = async {
                tokio::select! {
                    response = &mut request => panic!("request finished early: {response}"),
                    () = entered.notified() => {}
                }
            } => {}
        }
    })
    .await
    .unwrap();
    let address = handle.local_addr().unwrap();
    let original_thread = observed.lock().unwrap()[0];
    assert_ne!(original_thread, caller);
    hosts.request_shutdown();
    assert!(!handle.is_admitting());
    let timer_progress = Cell::new(false);
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("live owned thread was not retained: {result:?}"),
            () = async {
                sleep(Duration::from_millis(3)).await;
                timer_progress.set(true);
            } => {}
        }
    })
    .await
    .unwrap();
    // The borrowing future was cancelled during original Join observation.
    assert!(timer_progress.get());
    assert_eq!(hosts.phase(), Phase::Stopping);
    assert_eq!(handle.local_addr(), Some(address));
    release.notify_one();
    let (run, response) = timeout(LIMIT, async {
        tokio::join!(hosts.run_async(), &mut request)
    })
    .await
    .unwrap();
    run.unwrap();
    assert_eq!(response, "owned drain");
    assert_eq!(&*observed.lock().unwrap(), &[original_thread]);
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
    assert!(!handle.is_ready());
    let _rebound = TcpListener::bind(address).unwrap();
}

// An application-selected error may remain local and implement neither Error nor Clone.
struct LocalError {
    web: WebError,
    local: Rc<()>,
}
impl fmt::Debug for LocalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalError")
            .field("web", &self.web)
            .field("local_owners", &Rc::strong_count(&self.local))
            .finish()
    }
}

struct MappedWeb {
    web: WebActivity,
    local: Rc<()>,
}
impl MappedWeb {
    fn error(&self, web: WebError) -> LocalError {
        LocalError {
            web,
            local: self.local.clone(),
        }
    }
}
impl HostActivity for MappedWeb {
    type Error = LocalError;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        self.web
            .prepare_driver(mode)
            .map_err(|error| self.error(error))
    }
    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.web
            .poll_start(cx)
            .map(|result| result.map_err(|error| self.error(error)))
    }
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.web
            .poll_ready(cx)
            .map(|result| result.map_err(|error| self.error(error)))
    }
    fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.web.poll_finished(cx)
    }
    fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
        self.web.open_admission().map_err(|error| self.error(error))
    }
    fn close_admission(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.web
            .close_admission(cx)
            .map_err(|error| self.error(error))
    }
    fn request_shutdown(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.web
            .request_shutdown(cx)
            .map_err(|error| self.error(error))
    }
    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        self.web
            .poll_join(cx, budget)
            .map(|completion| match completion {
                WorkerCompletion::Pending => WorkerCompletion::Pending,
                WorkerCompletion::Joined(result) => {
                    WorkerCompletion::Joined(result.map_err(|error| self.error(error)))
                }
            })
            .map_err(|error| self.error(error))
    }
    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        self.web
            .poll_release(cx, budget)
            .map(|completion| match completion {
                ReleaseCompletion::Pending => ReleaseCompletion::Pending,
                ReleaseCompletion::Released(result) => {
                    ReleaseCompletion::Released(result.map_err(|error| self.error(error)))
                }
            })
            .map_err(|error| self.error(error))
    }
}

#[derive(Default)]
struct ResourceLog {
    events: RefCell<Vec<&'static str>>,
    release_started: Cell<bool>,
    release_allowed: Cell<bool>,
    waiter: RefCell<Option<Waker>>,
}
impl ResourceLog {
    fn allow_release(&self) {
        self.release_allowed.set(true);
        let wake = self.waiter.borrow_mut().take();
        if let Some(wake) = wake {
            wake.wake();
        }
    }
}
struct Passive<'a>(&'a ResourceLog);
impl Drop for Passive<'_> {
    fn drop(&mut self) {
        self.0.events.borrow_mut().push("passive dropped");
    }
}
struct Journal<'a>(&'a ResourceLog);
impl ManagedResource for Journal<'_> {
    type Error = LocalError;

    fn poll_acquire(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.events.borrow_mut().push("acquired");
        Poll::Ready(Ok(()))
    }
    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        if !self.0.release_started.replace(true) {
            self.0.events.borrow_mut().push("release started");
        }
        if !self.0.release_allowed.get() {
            *self.0.waiter.borrow_mut() = Some(cx.waker().clone());
            return Ok(ReleaseCompletion::Pending);
        }
        self.0.events.borrow_mut().push("released");
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}
struct ResourceHost<'a> {
    web: WebHost,
    local: Rc<()>,
    log: &'a ResourceLog,
}
impl<'a> Host for ResourceHost<'a> {
    type Error = LocalError;
    type Activity = MappedWeb;
    type Resources = Managed<Journal<'a>, Retained<Passive<'a>, ()>>;

    fn name(&self) -> &'static str {
        self.web.name()
    }
    fn validate(&self) -> Result<(), Self::Error> {
        self.web.validate().map_err(|web| LocalError {
            web,
            local: self.local.clone(),
        })
    }
    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        self.web.exclusive_claims()
    }
    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        self.web
            .into_resources()
            .map_activity(|web| MappedWeb {
                web,
                local: self.local,
            })
            .with_resource(Passive(self.log))
            .with_managed_resource("journal", Journal(self.log))
    }
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_drive_retains_admitted_work_and_local_resources_through_final_release() {
    let entered = Arc::new(Notify::new());
    let release_request = Arc::new(Notify::new());
    let handler_entered = entered.clone();
    let handler_release = release_request.clone();
    let log = ResourceLog::default();
    let web = App::new()
        .get("/slow", move || {
            let entered = handler_entered.clone();
            let release = handler_release.clone();
            async move {
                entered.notify_one();
                release.notified().await;
                "drained"
            }
        })
        .into_host()
        .with_address("127.0.0.1:0");
    let handle = web.handle();
    let host = ResourceHost {
        web,
        local: Rc::new(()),
        log: &log,
    };
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    assert!(log.events.borrow().is_empty());
    let _unblock = UnblockOnDrop(release_request.clone());
    let request_handle = handle.clone();
    let mut request = Box::pin(async move {
        let address = admitted(&request_handle).await;
        client()
            .get(url(address, "/slow"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    });
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("unexpected terminal run: {result:?}"),
            () = async {
                tokio::select! {
                    response = &mut request => panic!("request finished before release: {response}"),
                    () = entered.notified() => {}
                }
            } => {}
        }
    }).await.unwrap();
    // The first borrowing drive was cancelled while a real request was admitted.
    assert_eq!(&*log.events.borrow(), &["acquired"]);
    assert!(handle.is_admitting());
    hosts.request_shutdown();
    assert!(!handle.is_admitting());

    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("admitted work was not retained: {result:?}"),
            () = async {
                sleep(Duration::from_millis(2)).await;
                let late = client().get(url(handle.local_addr().unwrap(), "/slow")).send().await;
                if let Ok(response) = late {
                    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
                }
            } => {}
        }
    })
    .await
    .unwrap();
    // Cancellation during original completion still cannot release dependencies.
    assert!(!log.release_started.get());
    assert_eq!(&*log.events.borrow(), &["acquired"]);

    release_request.notify_one();
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("finalizer gate was bypassed: {result:?}"),
            () = async {
                assert_eq!((&mut request).await, "drained");
                wait_until(|| log.release_started.get()).await;
            } => {}
        }
    })
    .await
    .unwrap();
    // Cleanup state and its wake registration are retained in the original resource.
    assert_eq!(&*log.events.borrow(), &["acquired", "release started"]);
    log.allow_release();
    timeout(LIMIT, hosts.run_async()).await.unwrap().unwrap();
    assert_eq!(hosts.phase(), Phase::Stopped);
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
    assert_eq!(
        &*log.events.borrow(),
        &["acquired", "release started", "released", "passive dropped"]
    );
    let terminal = log.events.borrow().clone();
    timeout(LIMIT, hosts.run_async()).await.unwrap().unwrap();
    assert_eq!(&*log.events.borrow(), &terminal);
}
