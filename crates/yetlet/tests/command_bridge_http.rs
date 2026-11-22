#![cfg(all(
    feature = "web",
    feature = "command-bridge",
    not(target_arch = "wasm32")
))]

use std::{
    cell::Cell,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use tokio::sync::Notify;
use yetlet::command_bridge::{AdmissionError, CommandBridge, CommandIngress, SnapshotReader};
use yetlet::composition::{Builder, Completion, JoinStatus, WorkerCompletion};
use yetlet::web::HttpWorker;

const BUDGET: Duration = Duration::from_secs(4);

fn deadline() -> Instant {
    Instant::now() + BUDGET
}

fn http(endpoint: SocketAddr, path: &str, body: Option<&str>) -> String {
    let mut stream = TcpStream::connect_timeout(&endpoint, BUDGET).unwrap();
    stream.set_read_timeout(Some(BUDGET)).unwrap();
    stream.set_write_timeout(Some(BUDGET)).unwrap();
    let request = match body {
        Some(body) => format!(
            "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len(),
        ),
        None => format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
    };
    stream.write_all(request.as_bytes()).unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    reply
}

fn status(reply: &str, code: u16) {
    assert!(
        reply.starts_with(&format!("HTTP/1.1 {code} ")),
        "unexpected HTTP result: {reply}"
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Intent {
    Pause,
    Resume,
}

#[derive(Clone)]
struct Ports {
    commands: CommandIngress<Intent>,
    published: SnapshotReader<u64>,
}

async fn post_intent(State(ports): State<Ports>, body: String) -> StatusCode {
    let intent = match body.as_str() {
        "pause" => Intent::Pause,
        "resume" => Intent::Resume,
        _ => return StatusCode::BAD_REQUEST,
    };
    match ports.commands.try_submit(intent) {
        Ok(()) => StatusCode::ACCEPTED,
        Err(AdmissionError::Full) => StatusCode::TOO_MANY_REQUESTS,
        Err(AdmissionError::Closed | AdmissionError::Disconnected | AdmissionError::Poisoned) => {
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

async fn snapshot(State(ports): State<Ports>) -> Result<String, StatusCode> {
    ports
        .published
        .latest()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .map(|s| format!("{}:{}:{}", s.generation, s.revision, s.snapshot))
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

#[test]
fn real_http_bridge_keeps_one_local_authority_and_joins_exact_worker() {
    let bridge = CommandBridge::<Intent, u64>::new(1, 3).unwrap();
    bridge.publish_snapshot(7, 0, 50).unwrap();
    let ingress = bridge.ingress();
    let ports = Ports {
        commands: ingress.clone(),
        published: bridge.snapshots(),
    };
    let router = Router::new()
        .route("/ready", get(|| async { "ready" }))
        .route("/snapshot", get(snapshot))
        .route("/intent", post(post_intent))
        .with_state(ports);
    let worker = HttpWorker::start(router).unwrap();
    let endpoint = worker.endpoint();
    assert_eq!(endpoint.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    worker.wait_http_ready("/ready", BUDGET).unwrap();
    status(&http(endpoint, "/ready", None), 200);
    status(&http(endpoint, "/snapshot", None), 200);

    let authority = Rc::new(Cell::new(0_u64));
    let root_authority = authority.clone();
    let close = ingress.clone();
    let mut root = Builder::<io::Error>::new()
        .required_local("C1-authority", root_authority, |_| Ok(()))
        .required_worker(
            "Web",
            worker,
            move |_| close.close(),
            HttpWorker::request_stop,
            HttpWorker::observe_within,
        )
        .build();
    assert!(!root.all_required_ready());
    assert!(root.parts_mut().last().mark_ready());
    assert!(root.all_required_ready());

    status(&http(endpoint, "/intent", Some("pause")), 202);
    status(&http(endpoint, "/intent", Some("resume")), 429);
    assert_eq!(
        authority.get(),
        0,
        "HTTP receipt was only admission, not execution"
    );
    assert_eq!(bridge.commands().try_recv().unwrap(), Some(Intent::Pause));
    authority.set(1); // the only domain owner explicitly applies the command
    bridge.publish_snapshot(7, 1, authority.get()).unwrap();
    let response = http(endpoint, "/snapshot", None);
    status(&response, 200);
    assert!(response.ends_with("7:1:1"));
    assert_eq!(
        bridge.snapshots().at_revision(0).unwrap().unwrap().snapshot,
        50
    );
    assert_eq!(
        bridge.snapshots().at_revision(1).unwrap().unwrap().revision,
        1
    );

    root.request_stop();
    assert!(ingress.is_closed().unwrap());
    assert_eq!(
        ingress.try_submit(Intent::Resume),
        Err(AdmissionError::Closed)
    );
    // The queued "resume" was rejected before any step; never claim it applied.
    assert_eq!(bridge.commands().try_recv().unwrap(), None);
    assert_eq!(root.complete_until(deadline()), Completion::Complete);
    assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
    assert!(!root.parts().last().value().has_pending_join());
    assert!(root.failures().primary().is_none());
    drop(root);
    let rebound = TcpListener::bind(endpoint).unwrap();
    drop(rebound);
}

#[derive(Clone)]
struct Held {
    entered: SyncSender<()>,
    release: Arc<Notify>,
}

async fn held(State(hold): State<Held>) -> &'static str {
    hold.entered.send(()).unwrap();
    hold.release.notified().await;
    "released"
}

#[test]
fn real_inflight_http_drains_and_original_join_stays_pending_until_release() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let release = Arc::new(Notify::new());
    let router = Router::new()
        .route("/ready", get(|| async { "ready" }))
        .route("/held", get(held))
        .with_state(Held {
            entered: entered_tx,
            release: release.clone(),
        });
    let mut worker = HttpWorker::start(router).unwrap();
    let endpoint = worker.endpoint();
    worker.wait_http_ready("/ready", BUDGET).unwrap();

    let client = thread::spawn(move || http(endpoint, "/held", None));
    entered_rx
        .recv_timeout(BUDGET)
        .expect("request must be in-flight");
    worker.request_stop().unwrap();
    assert!(worker.has_pending_join());
    // Listener shutdown can precede completion of this held request. A new
    // HTTP 200 (even after address reuse) cannot make this worker Ready.
    assert!(
        worker
            .wait_http_ready("/ready", Duration::from_millis(25))
            .is_err(),
        "stopping worker must reject readiness while its Join is Pending"
    );
    assert!(matches!(
        worker.observe_within(Duration::ZERO).unwrap(),
        WorkerCompletion::Pending
    ));
    assert!(
        worker.has_pending_join(),
        "never detach the original worker on timeout"
    );
    release.notify_one();
    assert!(matches!(
        worker.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
    assert!(!worker.has_pending_join());
    let response = client.join().unwrap();
    status(&response, 200);
    assert!(response.ends_with("released"));
    let rebound = TcpListener::bind(endpoint).expect("listener port released");
    drop(rebound);
}

#[test]
fn existing_direct_pure_web_router_is_still_native_axum() {
    let router = yetlet::web::App::new()
        .get("/ready", || async { "ready" })
        .into_router();
    let mut worker = HttpWorker::start(router).unwrap();
    worker.wait_http_ready("/ready", BUDGET).unwrap();
    let endpoint = worker.endpoint();
    status(&http(endpoint, "/ready", None), 200);
    worker.request_stop().unwrap();
    assert!(matches!(
        worker.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
}

#[test]
fn closed_ingress_rejects_real_http_post_before_worker_shutdown() {
    let bridge = CommandBridge::<Intent, u64>::new(2, 2).unwrap();
    let router = Router::new()
        .route("/ready", get(|| async { "ready" }))
        .route("/intent", post(post_intent))
        .with_state(Ports {
            commands: bridge.ingress(),
            published: bridge.snapshots(),
        });
    let mut worker = HttpWorker::start(router).unwrap();
    let endpoint = worker.endpoint();
    worker.wait_http_ready("/ready", BUDGET).unwrap();
    status(&http(endpoint, "/intent", Some("pause")), 202);
    bridge.close_admission().unwrap();
    status(&http(endpoint, "/intent", Some("resume")), 503);
    assert_eq!(bridge.commands().try_recv().unwrap(), Some(Intent::Pause));
    assert_eq!(bridge.commands().try_recv().unwrap(), None);
    worker.request_stop().unwrap();
    assert!(matches!(
        worker.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
}

#[test]
fn real_http_bind_failure_rolls_back_the_same_required_domain() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let occupied = listener.local_addr().unwrap();
    let stopped = Rc::new(Cell::new(false));
    let closed = Rc::clone(&stopped);
    let result = Builder::<io::Error>::new()
        .required_local("authority", (), move |_| {
            closed.set(true);
            Ok(())
        })
        .try_required_worker(
            "web",
            || HttpWorker::start_on(Router::new(), occupied),
            |_| Ok(()),
            HttpWorker::request_stop,
            HttpWorker::observe_within,
        );
    let failure = match result {
        Err(failure) => failure,
        Ok(_) => panic!("occupied listener must reject required Web startup"),
    };
    assert!(
        stopped.get(),
        "failure must close the previously acquired domain"
    );
    assert_eq!(
        failure.root().failures().primary().unwrap().kind(),
        io::ErrorKind::AddrInUse
    );
    let mut root = failure.into_root();
    assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
    assert!(root.parts().last().is_stopped());
    drop(listener);
}

#[test]
fn joined_worker_never_probes_a_different_server_reusing_its_port() {
    let router = Router::new().route("/ready", get(|| async { "ready" }));
    let mut original = HttpWorker::start(router.clone()).unwrap();
    let endpoint = original.endpoint();
    original.wait_http_ready("/ready", BUDGET).unwrap();
    original.request_stop().unwrap();
    assert!(matches!(
        original.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
    assert!(!original.has_pending_join());
    assert!(original.is_finished());
    // A different Axum worker deliberately reuses this exact socket address.
    let mut replacement = HttpWorker::start_on(router, endpoint).unwrap();
    replacement.wait_http_ready("/ready", BUDGET).unwrap();
    assert!(
        original
            .wait_http_ready("/ready", Duration::from_millis(100))
            .is_err(),
        "an already-joined worker cannot inherit the replacement's HTTP readiness"
    );
    replacement.request_stop().unwrap();
    assert!(matches!(
        replacement.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
}

#[test]
fn bind_collision_and_http_readiness_rejection_do_not_claim_success() {
    let bound = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let endpoint = bound.local_addr().unwrap();
    assert!(HttpWorker::start_on(Router::new(), endpoint).is_err());
    drop(bound);

    let mut worker =
        HttpWorker::start(Router::new().route("/ready", get(|| async { "ready" }))).unwrap();
    assert!(
        worker
            .wait_http_ready("/missing", Duration::from_millis(40))
            .is_err()
    );
    assert!(
        worker
            .wait_http_ready("/ready\r\nX:attack", Duration::from_millis(40))
            .is_err()
    );
    worker.wait_http_ready("/ready", BUDGET).unwrap();
    worker.request_stop().unwrap();
    assert!(matches!(
        worker.observe_within(BUDGET).unwrap(),
        WorkerCompletion::Joined(Ok(()))
    ));
}
