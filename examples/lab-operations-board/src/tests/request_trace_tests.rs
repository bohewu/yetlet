use super::*;
use std::{
    io::Write,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tracing::{Dispatch, Instrument, instrument::WithSubscriber};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut b = self.0.lock().unwrap();
        if b.len() + bytes.len() > 64 * 1024 {
            return Err(std::io::Error::other("capture bound"));
        }
        b.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}
impl Capture {
    fn dispatch(&self) -> Dispatch {
        tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_ansi(false)
            .with_current_span(true)
            .with_span_list(true)
            .with_max_level(tracing::Level::INFO)
            .with_writer(self.clone())
            .finish()
            .into()
    }
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
    fn rows(&self) -> Vec<serde_json::Value> {
        self.text()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
}
fn root_id(row: &serde_json::Value) -> Option<u64> {
    row["spans"]
        .as_array()?
        .iter()
        .find(|s| s["name"] == "board_request")?["request_id"]
        .as_u64()
}
fn event(row: &serde_json::Value) -> &str {
    row["fields"]["event"].as_str().unwrap()
}
async fn board(path: &std::path::Path, ids: DemoIdentities) -> Board {
    Board::open_with_options(
        path,
        HOST.parse().unwrap(),
        ids,
        BoardOptions {
            request_tracing: true,
            write_authorization: cfg!(feature = "commit-write-auth"),
            ..BoardOptions::default()
        },
    )
    .await
    .unwrap()
}
fn get(path: &str, cookie: Option<&str>) -> Request<Body> {
    let mut b = Request::builder().uri(path).header(header::HOST, HOST);
    if let Some(cookie) = cookie {
        b = b.header(header::COOKIE, cookie);
    }
    b.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn request_trace_authenticated_ssr_json_privacy_and_worker_link() {
    let dir = tempfile::tempdir().unwrap();
    let ids = DemoIdentities::generate().unwrap();
    let board = board(&dir.path().join("privacy.sqlite"), ids.clone()).await;
    let capture = Capture::default();
    let dispatch = capture.dispatch();
    let client = Client::login(&board, &ids, "alice")
        .with_subscriber(dispatch.clone())
        .await;
    let item = board
        .database
        .save(alice(), None, 0, fields("PRIVATE_BUSINESS_TITLE"))
        .await
        .unwrap();
    capture.clear();
    let mut request = get(
        "/board/projects/1/items?q=PRIVATE_QUERY",
        client.cookie.as_deref(),
    );
    for (name, value) in [
        ("authorization", "Bearer PRIVATE_AUTH"),
        ("x-yetlet-request-id", "PRIVATE_INCOMING_ID"),
        ("traceparent", "PRIVATE_TRACEPARENT"),
    ] {
        request.headers_mut().insert(
            header::HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    }
    *request.body_mut() = Body::from("PRIVATE_BODY");
    let foreign = tracing::dispatcher::with_default(
        &dispatch,
        || tracing::info_span!(parent:None,"foreign_parent",private="PRIVATE_AMBIENT"),
    );
    let response = board
        .router
        .clone()
        .oneshot(request)
        .instrument(foreign)
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        DEFAULT_CSP
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    let id = response.headers()[yetlet_request_id::HEADER]
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!(id > 0);
    let rows = capture.rows();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| root_id(r) == Some(id)));
    assert!(
        rows.iter()
            .any(|r| event(r) == "sql_worker_completed" && r["fields"]["result"] == "ok")
    );
    assert_eq!(
        rows.iter()
            .filter(|r| event(r) == "http_response_constructed")
            .count(),
        1
    );
    let ssr = capture.text();
    capture.clear();
    let mut triage = yetlet_board_triage::Item::from(item);
    triage.title = "PRIVATE_JSON_TITLE".into();
    let (status, _) = triage_json(
        &board,
        &client,
        1,
        serde_json::json!({"csrf":client.csrf,"item":triage}),
    )
    .with_subscriber(dispatch)
    .await;
    assert_eq!(status, StatusCode::OK);
    let rows = capture.rows();
    assert!(rows.iter().any(|r| event(r) == "sql_worker_started"));
    let id2 = root_id(rows.first().unwrap()).unwrap();
    assert_ne!(id, id2);
    assert!(rows.iter().all(|r| root_id(r) == Some(id2)));
    let text = ssr + &capture.text();
    for forbidden in [
        "PRIVATE_QUERY",
        "PRIVATE_AUTH",
        "PRIVATE_INCOMING_ID",
        "PRIVATE_TRACEPARENT",
        "PRIVATE_BODY",
        "PRIVATE_AMBIENT",
        "PRIVATE_BUSINESS_TITLE",
        "PRIVATE_JSON_TITLE",
        &client.csrf,
        client.cookie.as_deref().unwrap(),
    ] {
        assert!(!text.contains(forbidden));
    }
    assert_eq!(board.database.get(1, 1).await.unwrap().0.version, 2);
    board.drain().await.unwrap();
}

#[tokio::test]
async fn request_trace_concurrent_requests_keep_independent_roots() {
    let dir = tempfile::tempdir().unwrap();
    let ids = DemoIdentities::generate().unwrap();
    let board = board(&dir.path().join("concurrent.sqlite"), ids.clone()).await;
    let capture = Capture::default();
    let dispatch = capture.dispatch();
    let client = Client::login(&board, &ids, "alice")
        .with_subscriber(dispatch.clone())
        .await;
    capture.clear();
    let (a, b) = tokio::join!(
        board
            .router
            .clone()
            .oneshot(get("/board/projects/1/items", client.cookie.as_deref()))
            .with_subscriber(dispatch.clone()),
        board
            .router
            .clone()
            .oneshot(get("/board/projects/1/items", client.cookie.as_deref()))
            .with_subscriber(dispatch)
    );
    let a = a.unwrap();
    let b = b.unwrap();
    assert_eq!(a.status(), StatusCode::OK);
    assert_eq!(b.status(), StatusCode::OK);
    let id = |r: &Response| {
        r.headers()[yetlet_request_id::HEADER]
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    let (x, y) = (id(&a), id(&b));
    assert_ne!(x, y);
    let rows = capture.rows();
    let roots = rows
        .iter()
        .filter(|r| event(r) == "http_response_constructed")
        .map(|r| root_id(r).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(roots, [x, y].into_iter().collect());
    assert!(
        rows.iter()
            .filter(|r| event(r) == "sql_worker_completed")
            .all(|r| matches!(root_id(r),Some(z) if z==x||z==y))
    );
    board.drain().await.unwrap();
}

#[tokio::test]
async fn request_trace_disabled_and_unknown_context_do_not_emit() {
    let dir = tempfile::tempdir().unwrap();
    let ids = DemoIdentities::generate().unwrap();
    let normal = Board::open(
        &dir.path().join("off.sqlite"),
        HOST.parse().unwrap(),
        ids.clone(),
    )
    .await
    .unwrap();
    let capture = Capture::default();
    let dispatch = capture.dispatch();
    let response = normal
        .router
        .clone()
        .oneshot(get("/board/login", None))
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert!(!response.headers().contains_key(yetlet_request_id::HEADER));
    assert!(capture.text().is_empty());
    let selected = board(&dir.path().join("unknown.sqlite"), ids).await;
    let foreign = tracing::dispatcher::with_default(
        &dispatch,
        || tracing::info_span!(target:crate::REQUEST_TRACING_TARGET,"foreign_context",private_context="PRIVATE_FOREIGN"),
    );
    selected
        .database
        .run(|c| {
            c.query_row("SELECT COUNT(*) FROM work_items", [], |r| {
                r.get::<_, i64>(0)
            })
            .map_err(Failure::from)
        })
        .instrument(foreign)
        .with_subscriber(dispatch)
        .await
        .unwrap();
    assert!(capture.text().is_empty());
    normal.drain().await.unwrap();
    selected.drain().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_trace_drop_retains_started_worker_context_and_permit() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(&dir.path().join("drop.sqlite"))
        .await
        .unwrap()
        .with_request_tracing();
    let capture = Capture::default();
    let dispatch = capture.dispatch();
    let (entered, started) = tokio::sync::oneshot::channel();
    let (release, resume) = std::sync::mpsc::channel();
    let signals = Arc::new(Mutex::new(Some((entered, resume))));
    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handler_db = db.clone();
    let handler_done = done.clone();
    let app = crate::request_tracing::install(Router::new().route(
        "/work",
        axum::routing::get(move || {
            let db = handler_db.clone();
            let signals = signals.clone();
            let done = handler_done.clone();
            async move {
                let (entered, resume) = signals.lock().unwrap().take().unwrap();
                db.run(move |c| {
                    entered.send(()).unwrap();
                    resume
                        .recv_timeout(Duration::from_secs(3))
                        .map_err(|_| Failure::Unavailable)?;
                    let n = c.query_row("SELECT COUNT(*) FROM work_items", [], |r| {
                        r.get::<_, i64>(0)
                    })?;
                    done.store(true, Ordering::SeqCst);
                    Ok(n)
                })
                .await
                .map(|_| StatusCode::OK)
            }
        }),
    ));
    let task = tokio::spawn(app.oneshot(get("/work", None)).with_subscriber(dispatch));
    started.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    let drain_db = db.clone();
    let mut drain = tokio::spawn(async move { drain_db.drain().await });
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut drain)
            .await
            .is_err()
    );
    assert!(!done.load(Ordering::SeqCst));
    release.send(()).unwrap();
    drain.await.unwrap().unwrap();
    assert!(done.load(Ordering::SeqCst));
    let rows = capture.rows();
    assert_eq!(
        rows.iter()
            .filter(|r| event(r) == "http_future_dropped")
            .count(),
        1
    );
    assert!(!rows.iter().any(|r| event(r) == "http_response_constructed"));
    let id = root_id(&rows[0]).unwrap();
    assert!(rows.iter().all(|r| root_id(r) == Some(id)));
    assert!(rows.iter().any(|r| event(r) == "sql_worker_completed"));
}

#[derive(Clone)]
struct Failing(Arc<AtomicUsize>);
impl Write for Failing {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::other("selected writer unavailable"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Failing {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}
#[tokio::test]
async fn request_trace_writer_error_does_not_rewrite_business_outcome() {
    let dir = tempfile::tempdir().unwrap();
    let ids = DemoIdentities::generate().unwrap();
    let board = board(&dir.path().join("writer.sqlite"), ids.clone()).await;
    let failed = Failing(Arc::new(AtomicUsize::new(0)));
    let dispatch: Dispatch = tracing_subscriber::fmt()
        .json()
        .log_internal_errors(false)
        .with_writer(failed.clone())
        .finish()
        .into();
    let mut client = Client::login(&board, &ids, "alice")
        .with_subscriber(dispatch.clone())
        .await;
    let response = client
        .call(
            &board.router,
            Method::POST,
            "/board/projects/1/items",
            Some(client.form("Still committed", "open", 0, "")),
        )
        .with_subscriber(dispatch)
        .await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    assert_eq!(board.database.get(1, 1).await.unwrap().0.version, 1);
    assert!(failed.0.load(Ordering::SeqCst) > 0);
    board.drain().await.unwrap();
}
