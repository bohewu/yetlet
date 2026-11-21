#[path = "../examples/support/sqlite_resource.rs"]
mod sqlite_resource;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use rusqlite::Connection;
use sqlite_resource::{SqliteClient, SqliteResource, SqliteStatus};
use std::{
    cell::Cell,
    fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    rc::Rc,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Notify;
use yetlet_host_lifecycle_research::{
    Application, BuiltApplication, Completion, ManagedResource, ReleaseCompletion, ShutdownHandle,
    StartStatus, WebHost, WebStatus,
};

type HttpError = (StatusCode, String);

fn http_error(error: io::Error) -> HttpError {
    (StatusCode::SERVICE_UNAVAILABLE, error.to_string())
}

async fn count(State(client): State<SqliteClient>) -> Result<String, HttpError> {
    client
        .count()
        .await
        .map(|value| value.to_string())
        .map_err(http_error)
}

async fn insert(
    State(client): State<SqliteClient>,
    value: String,
) -> Result<(StatusCode, String), HttpError> {
    client
        .insert(value)
        .await
        .map(|id| (StatusCode::CREATED, id.to_string()))
        .map_err(http_error)
}

fn router(client: SqliteClient) -> Router {
    Router::new()
        .route("/", get(count))
        .route("/records", post(insert).get(count))
        .with_state(client)
}

fn request(address: SocketAddr, method: &str, path: &str, body: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn body(response: &str) -> &str {
    response.split_once("\r\n\r\n").expect("HTTP headers").1
}

fn wait_ready(status: &WebStatus) -> io::Result<SocketAddr> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !status.admission_open() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Web readiness"));
        }
        thread::sleep(Duration::from_millis(1));
    }
    status
        .endpoint()
        .ok_or_else(|| io::Error::other("ready Web has no address"))
}

fn finish<C>(app: &mut BuiltApplication<C>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.shutdown_until(Instant::now() + Duration::from_millis(5)) == Completion::Pending {
        assert!(
            Instant::now() < deadline,
            "retained application did not finish"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn advance_to_release<C>(app: &mut BuiltApplication<C>, database: &SqliteStatus) -> Completion {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let completion = app.shutdown_until(Instant::now() + Duration::from_millis(5));
        if database.release_started() {
            return completion;
        }
        assert_eq!(completion, Completion::Pending);
        assert!(
            Instant::now() < deadline,
            "database release was never reached"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn persisted(path: &Path) -> Vec<String> {
    let connection = Connection::open(path).unwrap();
    let mut statement = connection
        .prepare("SELECT value FROM records ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<String>>>()
        .unwrap()
}

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

struct EarlierDependency {
    database: SqliteStatus,
    observed: Rc<Cell<(bool, bool)>>,
}
impl Drop for EarlierDependency {
    fn drop(&mut self) {
        self.observed
            .set((true, self.database.joined() && !self.database.is_open()));
    }
}

struct ObserveRelease {
    resource: SqliteResource,
    web: WebStatus,
    calls: Rc<Cell<usize>>,
    before_web_join: Rc<Cell<bool>>,
    fail_first_progress: bool,
}
impl ManagedResource for ObserveRelease {
    fn acquire(&mut self) -> io::Result<()> {
        self.resource.acquire()
    }
    fn release_within(&mut self, budget: Duration) -> io::Result<ReleaseCompletion<io::Error>> {
        let calls = self.calls.get() + 1;
        self.calls.set(calls);
        if !self.web.joined() {
            self.before_web_join.set(true);
        }
        if self.fail_first_progress && calls == 1 {
            // Deliberate retryable lifecycle fault, not a claimed SQLite close error.
            return Err(io::Error::other("controlled release progress failure"));
        }
        self.resource.release_within(budget)
    }
}

#[test]
fn terminal_run_uses_real_sqlite_and_preserves_rows_after_both_original_joins() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("records.sqlite");
    let resource = SqliteResource::new(&path);
    let client = resource.client();
    let database = resource.status();
    let host = WebHost::new(router(client.clone())).with_managed_resource("sqlite", resource);
    let web = host.status();
    let app = Application::new().with_host(host);
    let shutdown = app.shutdown_handle();
    let app = app.build().unwrap();
    assert!(!path.exists());
    assert!(!client.available());
    assert!(web.endpoint().is_none());
    assert!(database.thread_id().is_none());

    let (result, requests) = thread::scope(|scope| {
        let worker = scope.spawn(|| -> io::Result<()> {
            let _stop = StopOnDrop(shutdown);
            let address = wait_ready(&web)?;
            for value in ["alpha", "beta's record"] {
                let response = request(address, "POST", "/records", value)?;
                if !response.starts_with("HTTP/1.1 201") {
                    return Err(io::Error::other(response));
                }
            }
            let response = request(address, "GET", "/records", "")?;
            if !response.starts_with("HTTP/1.1 200") || body(&response) != "2" {
                return Err(io::Error::other(response));
            }
            Ok(())
        });
        let result = app.run();
        (result, worker.join().unwrap())
    });
    requests.unwrap();
    let report = result.unwrap();
    assert!(report.primary.is_none());
    assert!(web.joined());
    assert!(database.release_started());
    assert!(database.joined());
    assert!(!database.is_open());
    assert!(
        !client.available(),
        "retained client cannot keep the sender alive"
    );
    assert_eq!(persisted(&path), ["alpha", "beta's record"]);
}

#[test]
fn admitted_request_uses_sqlite_during_drain_then_pending_cleanup_keeps_earlier_owner() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("drain.sqlite");
    let resource = SqliteResource::new(&path);
    let client = resource.client();
    let database = resource.status();
    let gate = Arc::new(Notify::new());
    let (entered, received) = mpsc::channel();
    let service = router(client.clone()).route(
        "/slow",
        get({
            let client = client.clone();
            let gate = gate.clone();
            move || {
                let client = client.clone();
                let gate = gate.clone();
                let entered = entered.clone();
                async move {
                    client
                        .insert("before stop".into())
                        .await
                        .map_err(http_error)?;
                    let _ = entered.send(());
                    gate.notified().await;
                    client
                        .insert("after stop".into())
                        .await
                        .map_err(http_error)?;
                    Ok::<_, HttpError>("drained")
                }
            }
        }),
    );
    let host = WebHost::new(service);
    let web = host.status();
    let ancestor = Rc::new(Cell::new((false, false)));
    let calls = Rc::new(Cell::new(0));
    let premature = Rc::new(Cell::new(false));
    let host = host
        .with_resource(EarlierDependency {
            database: database.clone(),
            observed: ancestor.clone(),
        })
        .with_managed_resource(
            "sqlite",
            ObserveRelease {
                resource,
                web: web.clone(),
                calls: calls.clone(),
                before_web_join: premature.clone(),
                fail_first_progress: false,
            },
        );
    let mut app = Application::new().with_host(host).build().unwrap();
    assert_eq!(app.start(), StartStatus::Started);
    let original = database.thread_id().unwrap();
    let address = web.endpoint().unwrap();

    thread::scope(|scope| {
        let _unblock = UnblockOnDrop(gate.clone());
        let request_worker = scope.spawn(|| request(address, "GET", "/slow", ""));
        received.recv_timeout(Duration::from_secs(3)).unwrap();
        let completion = app.shutdown_until(Instant::now());
        let still_owned = database.is_open()
            && client.available()
            && !database.release_started()
            && !database.joined()
            && !web.joined()
            && !ancestor.get().0;
        let admission_closed = !web.admission_open();
        // A late request must not enter the handler or append another record.
        let late = request(address, "POST", "/records", "must not be inserted");
        // Always unblock real work before any assertion can unwind this scope.
        gate.notify_one();
        assert_eq!(completion, Completion::Pending);
        assert!(still_owned);
        assert!(admission_closed);
        if let Ok(response) = late {
            assert!(response.starts_with("HTTP/1.1 503"));
        }
        let response = request_worker.join().unwrap().unwrap();
        assert!(response.starts_with("HTTP/1.1 200"));
        assert_eq!(body(&response), "drained");
    });

    assert_eq!(advance_to_release(&mut app, &database), Completion::Pending);
    assert!(web.joined());
    assert!(
        !database.joined(),
        "original database completion is still unobserved"
    );
    assert_eq!(database.thread_id(), Some(original));
    assert!(!client.available());
    assert!(!ancestor.get().0);
    assert!(app.report().is_none());
    assert!(!premature.get());
    finish(&mut app);
    assert!(database.joined());
    assert!(!database.is_open());
    assert_eq!(database.thread_id(), Some(original));
    assert_eq!(ancestor.get(), (true, true));
    assert!(app.report().unwrap().primary.is_none());
    let terminal_calls = calls.get();
    assert_eq!(app.shutdown_until(Instant::now()), Completion::Complete);
    assert_eq!(calls.get(), terminal_calls);
    assert_eq!(persisted(&path), ["before stop", "after stop"]);
}

#[test]
fn later_bind_failure_preserves_primary_and_database_through_controlled_cleanup_retry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rollback.sqlite");
    let resource = SqliteResource::new(&path);
    let client = resource.client();
    let database = resource.status();
    let host = WebHost::new(router(client.clone())).named("first");
    let web = host.status();
    let calls = Rc::new(Cell::new(0));
    let premature = Rc::new(Cell::new(false));
    let ancestor = Rc::new(Cell::new((false, false)));
    let host = host
        .with_resource(EarlierDependency {
            database: database.clone(),
            observed: ancestor.clone(),
        })
        .with_managed_resource(
            "sqlite",
            ObserveRelease {
                resource,
                web: web.clone(),
                calls: calls.clone(),
                before_web_join: premature.clone(),
                fail_first_progress: true,
            },
        );
    let external = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let occupied = external.local_addr().unwrap();
    let failing = WebHost::new(Router::new().route("/", get(|| async { "unused" })))
        .named("occupied")
        .bind(occupied);
    let failed_web = failing.status();
    let mut app = Application::new()
        .with_host(host)
        .with_host(failing)
        .build()
        .unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    let primary = app.primary_failure().unwrap();
    assert_eq!(primary.host, "occupied");
    assert_eq!(primary.error().kind(), io::ErrorKind::AddrInUse);
    let original = database.thread_id().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while calls.get() == 0 {
        assert_eq!(
            app.shutdown_until(Instant::now() + Duration::from_millis(5)),
            Completion::Pending
        );
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    assert!(web.joined());
    assert!(
        !failed_web.joined(),
        "failed bind never acquired a Web worker"
    );
    assert!(database.is_open());
    assert!(!database.release_started());
    assert!(!database.joined());
    assert!(!ancestor.get().0);
    assert!(!premature.get());
    assert!(std::ptr::eq(
        primary.error(),
        app.primary_failure().unwrap().error()
    ));
    assert_eq!(advance_to_release(&mut app, &database), Completion::Pending);
    assert_eq!(database.thread_id(), Some(original));
    assert!(!database.joined());
    finish(&mut app);
    let report = app.report().unwrap();
    assert!(std::ptr::eq(
        primary.error(),
        report.primary.as_ref().unwrap().error()
    ));
    assert_eq!(report.secondary.len(), 1);
    assert_eq!(report.secondary[0].resource, Some("sqlite"));
    assert_eq!(report.secondary[0].operation, "release progress");
    assert!(database.joined());
    assert!(!database.is_open());
    assert_eq!(ancestor.get(), (true, true));
    assert_eq!(persisted(&path), Vec::<String>::new());
    assert_eq!(
        TcpListener::bind(occupied).unwrap_err().kind(),
        io::ErrorKind::AddrInUse
    );
    drop(external);
}

#[test]
fn real_sqlite_schema_failure_keeps_original_actor_for_join_before_any_web_start() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("corrupt.sqlite");
    let original_bytes = b"This is an existing non-SQLite file.";
    fs::write(&path, original_bytes).unwrap();
    let resource = SqliteResource::new(&path);
    let client = resource.client();
    let database = resource.status();
    let host = WebHost::new(router(client.clone())).with_managed_resource("sqlite", resource);
    let web = host.status();
    let mut app = Application::new().with_host(host).build().unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    let primary = app.primary_failure().unwrap();
    assert_eq!(primary.operation, "acquire");
    assert_eq!(primary.resource, Some("sqlite"));
    assert!(
        primary
            .error()
            .get_ref()
            .unwrap()
            .downcast_ref::<rusqlite::Error>()
            .is_some()
    );
    assert!(database.thread_id().is_some());
    assert!(!database.joined());
    assert!(!client.available());
    assert!(web.endpoint().is_none());
    assert_eq!(advance_to_release(&mut app, &database), Completion::Pending);
    finish(&mut app);
    assert!(database.joined());
    assert!(!database.is_open());
    assert!(!web.joined());
    assert!(!app.report().unwrap().hosts[0].start_attempted);
    assert!(std::ptr::eq(
        primary.error(),
        app.report().unwrap().primary.as_ref().unwrap().error()
    ));
    assert_eq!(fs::read(&path).unwrap(), original_bytes);
}

#[test]
fn real_sqlite_open_failure_retains_original_thread_and_skips_web_acquisition() {
    let directory = tempfile::tempdir().unwrap();
    let resource = SqliteResource::new(directory.path());
    let client = resource.client();
    let database = resource.status();
    let host = WebHost::new(router(client.clone())).with_managed_resource("sqlite", resource);
    let web = host.status();
    let mut app = Application::new().with_host(host).build().unwrap();
    assert_eq!(app.start(), StartStatus::Stopping);
    let primary = app.primary_failure().unwrap();
    assert_eq!(primary.operation, "acquire");
    assert_eq!(primary.resource, Some("sqlite"));
    assert!(
        primary
            .error()
            .get_ref()
            .unwrap()
            .downcast_ref::<rusqlite::Error>()
            .is_some()
    );
    assert!(database.thread_id().is_some());
    assert!(!database.is_open());
    assert!(!database.joined());
    assert!(web.endpoint().is_none());
    assert_eq!(advance_to_release(&mut app, &database), Completion::Pending);
    finish(&mut app);
    assert!(database.joined());
    assert!(!client.available());
    assert!(!web.joined());
    assert!(std::ptr::eq(
        primary.error(),
        app.report().unwrap().primary.as_ref().unwrap().error()
    ));
}
