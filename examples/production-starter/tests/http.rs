use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode},
};
use secrecy::SecretString;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;
use yetlet_production_starter::{
    observability::Metrics,
    store::{IssueStatus, Store},
    web::app,
};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn fixture() -> (tempfile::TempDir, Store, Arc<AtomicBool>, Router) {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open(directory.path().join("issues.sqlite3")).unwrap();
    let readiness = Arc::new(AtomicBool::new(true));
    let router = app(store.clone(), SecretString::from(TOKEN), readiness.clone()).unwrap();
    (directory, store, readiness, router)
}

async fn call(
    router: &Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: impl Into<Body>,
) -> (StatusCode, HeaderMap, String) {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = router
        .clone()
        .oneshot(request.body(body.into()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = String::from_utf8(
        to_bytes(response.into_body(), 256 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    (status, headers, body)
}

fn hidden(body: &str, name: &str) -> String {
    body.split(&format!("name=\"{name}\" value=\""))
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_owned()
}
fn encode(value: &str) -> String {
    value.bytes().map(|byte| format!("%{byte:02X}")).collect()
}
fn bearer() -> String {
    format!("Bearer {TOKEN}")
}
async fn api(
    router: &Router,
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, HeaderMap, String) {
    call(
        router,
        method,
        path,
        &[
            ("authorization", &bearer()),
            ("content-type", "application/json"),
        ],
        body.to_string(),
    )
    .await
}

#[tokio::test]
async fn native_forms_require_csrf_and_complete_create_status_prg() {
    let (_directory, store, _ready, router) = fixture();
    let (status, headers, page) = call(&router, "GET", "/", &[], "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("<html lang=\"en\">"));
    assert!(page.contains("No issues yet."));
    let cookie = headers["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert!(headers["set-cookie"].to_str().unwrap().contains("HttpOnly"));
    assert!(!headers["set-cookie"].to_str().unwrap().contains("Secure"));
    let csrf = hidden(&page, "csrf");
    let form_headers = [
        ("cookie", cookie),
        ("content-type", "application/x-www-form-urlencoded"),
    ];
    let (status, _, _) = call(
        &router,
        "POST",
        "/issues",
        &form_headers,
        "csrf=bad&title=Unwritten&body=No",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(store.list().await.unwrap().is_empty());
    let (status, _, page) = call(
        &router,
        "POST",
        "/issues",
        &form_headers,
        format!("csrf={csrf}&title=&body=Keep+this+description"),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(page.contains("<html lang=\"en\">"));
    assert!(page.contains("Keep this description"));
    assert!(page.contains("role=\"alert\""));
    let (status, headers, _) = call(
        &router,
        "POST",
        "/issues",
        &form_headers,
        format!("csrf={csrf}&title=First+issue&body=Details"),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = headers["location"].to_str().unwrap();
    let (status, _, page) = call(&router, "GET", location, &[("cookie", cookie)], "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("<html lang=\"en\">"));
    assert!(page.contains("First issue"));
    let version = hidden(&page, "expected_version");
    let csrf = hidden(&page, "csrf");
    let status_url = format!("{location}/status");
    let (status, _, _) = call(
        &router,
        "POST",
        &status_url,
        &form_headers,
        format!("csrf=wrong&status=closed&expected_version={version}"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(store.list().await.unwrap()[0].status, IssueStatus::Open);
    let (status, _, _) = call(
        &router,
        "POST",
        &status_url,
        &form_headers,
        format!("csrf={csrf}&status=in_progress&expected_version={version}"),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let issue = &store.list().await.unwrap()[0];
    assert_eq!(issue.status, IssueStatus::InProgress);
    assert_eq!(issue.version, 2);
    let (status, _, page) = call(
        &router,
        "POST",
        &status_url,
        &form_headers,
        format!("csrf={csrf}&status=closed&expected_version={version}"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(page.contains("Issue changed; reload before trying again"));
    assert!(page.contains("value=\"closed\" selected=\"selected\""));
    assert_eq!(
        store.list().await.unwrap()[0].status,
        IssueStatus::InProgress
    );
}

#[tokio::test]
async fn hostile_content_is_escaped_and_every_response_has_security_headers() {
    let (_directory, store, _ready, router) = fixture();
    let title = "<script>alert('hostile')</script> & \"title\"";
    let body = "</textarea><img src=x onerror=alert(1)>\nDescription";
    let issue = store.create(title.into(), body.into()).await.unwrap();
    for path in [
        "/".to_owned(),
        format!("/issues/{}", issue.id),
        "/missing".to_owned(),
    ] {
        let (_, headers, page) = call(&router, "GET", &path, &[], "").await;
        assert_eq!(headers["cache-control"], "no-store");
        assert_eq!(headers["x-content-type-options"], "nosniff");
        assert!(
            headers["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("default-src 'none'")
        );
        assert!(
            headers["x-yetlet-request-id"]
                .to_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > 0
        );
        assert!(!page.contains("<script>"));
        assert!(!page.contains("<img src=x"));
        assert!(!page.contains(TOKEN));
        if path != "/missing" {
            assert!(page.contains("&lt;script&gt;"));
        }
    }
    let (_, headers, page) = call(&router, "GET", "/", &[], "").await;
    let cookie = headers["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let csrf = hidden(&page, "csrf");
    let (status, _, page) = call(
        &router,
        "POST",
        "/issues",
        &[
            ("cookie", cookie),
            ("content-type", "application/x-www-form-urlencoded"),
        ],
        format!(
            "csrf={csrf}&title={}&body={}",
            encode(&"x".repeat(121)),
            encode(body)
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(page.contains("&lt;/textarea&gt;&lt;img"));
    assert!(page.contains(&"x".repeat(121)));
}

#[tokio::test]
async fn api_is_bearer_scoped_rejects_browser_origins_and_never_uses_ssr_sessions() {
    let (_directory, store, _ready, router) = fixture();
    for path in ["/api/issues", "/api/issues/1", "/api/missing"] {
        let (status, headers, body) = call(&router, "GET", path, &[], "").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(headers.get("set-cookie").is_none());
        assert_eq!(
            serde_json::from_str::<Value>(&body).unwrap()["error"],
            "Valid bearer token required"
        );
    }
    let (status, _, _) = call(
        &router,
        "GET",
        "/api/issues",
        &[("authorization", &bearer()), ("origin", "http://localhost")],
        "",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &router,
        "POST",
        "/api/issues",
        &[
            ("authorization", &bearer()),
            ("origin", "null"),
            ("content-type", "application/json"),
        ],
        "{\"title\":\"No write\"}",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(store.list().await.unwrap().is_empty());
    let (status, _, _) = call(&router, "GET", "/", &[("authorization", "invalid")], "").await;
    assert_eq!(status, StatusCode::OK);
    let (status, headers, body) = api(&router, "GET", "/api/issues", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "[]");
    assert!(headers.get("set-cookie").is_none());
}

#[tokio::test]
async fn api_create_read_update_and_stale_conflict_preserve_data() {
    let (_directory, store, _ready, router) = fixture();
    let (status, headers, body) = api(
        &router,
        "POST",
        "/api/issues",
        json!({"title":"API issue", "body":"Typed boundary"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let issue: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(issue["status"], "open");
    assert_eq!(issue["version"], 1);
    let path = headers["location"].to_str().unwrap();
    assert_eq!(
        api(&router, "GET", path, Value::Null).await.0,
        StatusCode::OK
    );
    let (status, _, body) = api(
        &router,
        "PATCH",
        path,
        json!({"status":"closed", "expected_version":1}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap()["version"], 2);
    assert_eq!(
        api(
            &router,
            "PATCH",
            path,
            json!({"status":"open", "expected_version":1})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(store.list().await.unwrap()[0].status, IssueStatus::Closed);
    assert_eq!(
        api(
            &router,
            "PATCH",
            path,
            json!({"status":"open", "expected_version":0})
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn v2_status_view_preserves_v1_wire_and_auth_boundaries() {
    let (_directory, store, _ready, router) = fixture();
    let (_, _, body) = api(
        &router,
        "POST",
        "/api/issues",
        json!({"title":"v1 create", "body":"unchanged"}),
    )
    .await;
    let original: Value = serde_json::from_str(&body).unwrap();
    store.create("second".into(), String::new()).await.unwrap();
    let (status, headers, body) = api(
        &router,
        "PATCH",
        "/api/issues/1",
        json!({"status":"closed", "expected_version":1}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["cache-control"], "no-store");
    let closed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(closed["title"], original["title"]);
    assert_eq!(closed["body"], original["body"]);
    let (status, headers, body) =
        api(&router, "GET", "/api/v2/issues?status=closed", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&body).unwrap(),
        json!([closed])
    );
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.get("set-cookie").is_none());
    assert_eq!(
        serde_json::from_str::<Vec<Value>>(
            &api(&router, "GET", "/api/issues", Value::Null).await.2
        )
        .unwrap()
        .len(),
        2
    );
    for query in [
        "",
        "?status=secret-marker",
        "?status=open&extra=secret-marker",
    ] {
        let (status, _, body) = api(
            &router,
            "GET",
            &format!("/api/v2/issues{query}"),
            Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(!body.contains("secret-marker"));
    }
    assert_eq!(
        call(&router, "GET", "/api/v2/issues?status=open", &[], "")
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &router,
            "GET",
            "/api/v2/issues?status=open",
            &[("authorization", &bearer()), ("origin", "http://localhost")],
            ""
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        api(
            &router,
            "PATCH",
            "/api/issues/1",
            json!({"status":"open", "expected_version":1})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(store.get(1).await.unwrap().status, IssueStatus::Closed);
}

#[tokio::test]
async fn bounded_validation_extractor_and_database_errors_do_not_leak_inputs() {
    let (directory, store, _ready, router) = fixture();
    let marker = "secret-input-marker";
    for (path, input, expected) in [
        (
            "/api/issues",
            json!({"title":"\u{0000}"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "/api/issues",
            json!({"title":"x".repeat(121)}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "/api/issues",
            json!({"title":"ok","body":"x".repeat(4001)}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "/api/issues",
            json!({"title":"ok","unexpected":marker}),
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let (status, _, body) = api(&router, "POST", path, input).await;
        assert_eq!(status, expected);
        assert!(!body.contains(marker));
        assert!(body.len() < 200);
    }
    for path in ["/api/issues/secret-input-marker", "/api/issues/-1"] {
        let (status, _, body) = api(&router, "GET", path, Value::Null).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(!body.contains(marker));
    }
    let (status, _, body) = call(
        &router,
        "POST",
        "/api/issues",
        &[
            ("authorization", &bearer()),
            ("content-type", "application/json"),
        ],
        format!("{{broken-{marker}"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!body.contains(marker));
    let (status, _, body) = api(
        &router,
        "POST",
        "/api/issues",
        json!({"title":"big", "body":"x".repeat(17*1024)}),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(body.len() < 200);
    assert!(store.list().await.unwrap().is_empty());
    std::fs::remove_file(directory.path().join("issues.sqlite3")).unwrap();
    let (status, _, body) = api(&router, "GET", "/api/issues", Value::Null).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!body.contains(directory.path().to_str().unwrap()));
    assert!(!body.contains("sqlite"));
    assert!(!body.contains(TOKEN));
}

#[tokio::test]
async fn liveness_is_independent_of_storage_and_readiness_tracks_drain_and_database() {
    let (directory, _store, readiness, router) = fixture();
    assert_eq!(
        call(&router, "GET", "/health/live", &[], "").await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::OK
    );
    readiness.store(false, Ordering::Release);
    let (status, _, body) = call(&router, "GET", "/health/ready", &[], "").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "not ready");
    readiness.store(true, Ordering::Release);
    std::fs::remove_file(directory.path().join("issues.sqlite3")).unwrap();
    assert_eq!(
        call(&router, "GET", "/health/live", &[], "").await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[derive(Clone)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[tokio::test(flavor = "current_thread")]
async fn tracing_observes_generated_request_id_without_urls_headers_or_bodies() {
    let (_directory, _store, _ready, router) = fixture();
    let capture = Capture(Arc::new(Mutex::new(Vec::new())));
    let writer = capture.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(move || writer.clone())
        .finish();
    // Keep two registered dispatchers so concurrent uninstrumented callsites
    // cannot cache single-dispatcher Interest::never for this scoped capture.
    let _other_dispatcher = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let dispatcher = tracing::Dispatch::new(subscriber);
    let (status, headers, _) = call(
        &router,
        "GET",
        "/secret-path-marker?secret-query-marker",
        &[
            ("authorization", &bearer()),
            ("x-yetlet-request-id", "999999"),
            ("x-private", "secret-header-marker"),
            ("cookie", "yetlet-starter=secret-session-marker"),
            ("traceparent", "secret-trace-marker"),
        ],
        "secret-body-marker",
    )
    .with_subscriber(dispatcher.clone())
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_ne!(headers["x-yetlet-request-id"], "999999");
    let (ssr, api_result) = async {
        tokio::join!(
            call(&router, "GET", "/", &[], ""),
            api(&router, "GET", "/api/issues", json!(null))
        )
    }
    .with_subscriber(dispatcher)
    .await;
    assert_eq!(ssr.0, StatusCode::OK);
    assert_eq!(api_result.0, StatusCode::OK);
    let logs = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    let events: Vec<Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for response in [&ssr, &api_result] {
        let id: u64 = response.1["x-yetlet-request-id"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| event["fields"]["event"] == "sql_finished"
                    && event["span"]["request_id"] == id)
                .count(),
            1,
            "captured events: {logs}"
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event["fields"]["event"] == "http_request"
                    && event["span"]["request_id"] == id)
                .count(),
            1
        );
    }
    assert!(logs.contains("http_request"));
    assert!(logs.contains("\"request_id\":1"));
    assert!(logs.contains("\"status\":404"));
    assert!(logs.contains("\"method\":\"GET\""));
    for marker in [
        TOKEN,
        "secret-path-marker",
        "secret-query-marker",
        "secret-header-marker",
        "secret-body-marker",
        "secret-session-marker",
        "secret-trace-marker",
        "999999",
    ] {
        assert!(!logs.contains(marker));
    }
}

// A request body that signals when the handler actually awaits its next frame.
// This makes saturation/cancellation tests independent of scheduler delays.
struct PendingBody(Option<tokio::sync::oneshot::Sender<()>>);
impl axum::body::HttpBody for PendingBody {
    type Data = axum::body::Bytes;
    type Error = std::convert::Infallible;
    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        if let Some(started) = self.0.take() {
            let _ = started.send(());
        }
        std::task::Poll::Pending
    }
}

#[tokio::test]
async fn http_admission_deadline_health_exceptions_and_recovery() {
    use std::time::Duration;
    use yetlet_production_starter::web::{Admission, app_with_admission};
    let (_directory, store, ready, _) = fixture();
    let metrics = Arc::new(Metrics::new().unwrap());
    let store = store.with_metrics(metrics);
    let router = app_with_admission(
        store.clone(),
        SecretString::from(TOKEN),
        ready,
        Admission {
            max_requests: 1,
            request_timeout: Duration::from_secs(1),
        },
    )
    .unwrap();
    let (started, received) = tokio::sync::oneshot::channel();
    let pending = Request::builder()
        .method("POST")
        .uri("/api/issues")
        .header("authorization", bearer())
        .header("content-type", "application/json")
        .body(Body::new(PendingBody(Some(started))))
        .unwrap();
    let request = tokio::spawn(router.clone().oneshot(pending));
    received.await.unwrap();
    let (status, headers, body) = api(&router, "GET", "/api/issues", json!(null)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        serde_json::from_str::<Value>(&body).unwrap()["error"],
        "Issue storage is unavailable"
    );
    assert_eq!(headers["cache-control"], "no-store");
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("x-yetlet-request-id"));
    assert_eq!(
        call(&router, "GET", "/", &[], "").await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&router, "GET", "/health/live", &[], "").await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::OK
    );
    let (status, _, collected) = call(&router, "GET", "/metrics", &[], "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(collected.contains("starter_http_requests_active 1\n"));
    assert!(collected.contains("starter_http_admission_rejections_total{reason=\"capacity\"} 2"));
    let response = request.await.unwrap().unwrap();
    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(response.headers().contains_key("content-security-policy"));
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert!(
        std::str::from_utf8(&body)
            .unwrap()
            .contains("may still complete")
    );
    let (_, _, collected) = call(&router, "GET", "/metrics", &[], "").await;
    assert!(collected.contains("starter_http_requests_active 0\n"));
    assert!(collected.contains("starter_http_admission_rejections_total{reason=\"deadline\"} 1"));
    assert!(store.list().await.unwrap().is_empty()); // upload never dispatched SQL
    assert_eq!(
        api(&router, "GET", "/api/issues", json!(null)).await.0,
        StatusCode::OK
    );
    store.drain().await;
    assert_eq!(
        api(&router, "GET", "/api/issues", json!(null)).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&router, "GET", "/health/live", &[], "").await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn declared_normal_load_four_simultaneous_operations_has_no_rejections() {
    let (_directory, store, _ready, router) = fixture();
    for _ in 0..20 {
        let calls = (0..4)
            .map(|_| {
                let router = router.clone();
                tokio::spawn(async move {
                    api(&router, "POST", "/api/issues", json!({"title":"normal"}))
                        .await
                        .0
                })
            })
            .collect::<Vec<_>>();
        for call in calls {
            assert_eq!(call.await.unwrap(), StatusCode::CREATED);
        }
    }
    assert_eq!(store.list().await.unwrap().len(), 80);
}

#[tokio::test]
async fn http_write_timeout_is_indeterminate_and_can_commit_after_response() {
    use std::time::Duration;
    use yetlet_production_starter::web::{Admission, app_with_admission};
    let (directory, store, ready, _) = fixture();
    let metrics = Arc::new(Metrics::new().unwrap());
    let store = store.with_metrics(metrics.clone());
    let connection = rusqlite::Connection::open(directory.path().join("issues.sqlite3")).unwrap();
    connection.execute_batch("BEGIN IMMEDIATE").unwrap();
    let router = app_with_admission(
        store.clone(),
        SecretString::from(TOKEN),
        ready,
        Admission {
            max_requests: 32,
            request_timeout: Duration::from_millis(100),
        },
    )
    .unwrap();
    let (status, _, body) = api(
        &router,
        "POST",
        "/api/issues",
        json!({"title":"late HTTP write"}),
    )
    .await;
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert!(body.contains("may still complete"));
    let collected = String::from_utf8(metrics.encode().unwrap()).unwrap();
    assert!(collected.contains("starter_http_requests_active 0\n"));
    assert!(collected.contains("starter_sql_operations_active 1\n"));
    assert!(!collected.contains("starter_sql_operations_total{outcome=\"ok\"}"));
    connection.execute_batch("ROLLBACK").unwrap();
    store.drain().await;
    let collected = String::from_utf8(metrics.encode().unwrap()).unwrap();
    assert!(collected.contains("starter_sql_operations_active 0\n"));
    assert!(collected.contains("starter_sql_operations_total{outcome=\"ok\"} 1"));
    let reopened = Store::open(directory.path().join("issues.sqlite3")).unwrap();
    let issues = reopened.list().await.unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "late HTTP write");
}

#[tokio::test]
async fn optional_metrics_are_isolated_bounded_and_report_storage_failure() {
    let (directory, store, ready, disabled) = fixture();
    assert_eq!(
        call(&disabled, "GET", "/metrics", &[], "").await.0,
        StatusCode::NOT_FOUND
    );
    let metrics = Arc::new(Metrics::new().unwrap());
    let separate = Metrics::new().unwrap();
    let store = store.with_metrics(metrics);
    let router = app(store, SecretString::from(TOKEN), ready).unwrap();
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::OK
    );
    for id in 1..=20 {
        let path = format!("/private-path-{id}?user_id=private-user-marker");
        call(
            &router,
            "PRIVATE_METHOD",
            &path,
            &[
                ("authorization", &bearer()),
                ("cookie", "private-session-marker"),
                ("x-yetlet-request-id", "private-id-marker"),
            ],
            "private-body-marker",
        )
        .await;
    }
    let connection = rusqlite::Connection::open(directory.path().join("issues.sqlite3")).unwrap();
    connection.execute_batch("DROP TABLE issues").unwrap();
    assert_eq!(
        call(&router, "GET", "/health/ready", &[], "").await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let (status, headers, collected) = call(&router, "GET", "/metrics", &[], "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers["content-type"],
        "text/plain; version=0.0.4; charset=utf-8"
    );
    assert_eq!(headers["cache-control"], "no-store");
    assert!(collected.contains("starter_http_requests_total{method=\"OTHER\",status=\"404\"} 20"));
    assert!(collected.contains("starter_http_request_duration_seconds_count{method=\"OTHER\"} 20"));
    assert!(collected.contains("starter_readiness_last_probe 0\n"));
    assert!(collected.contains("starter_readiness_checks_total{outcome=\"ready\"} 1"));
    assert!(collected.contains("starter_readiness_checks_total{outcome=\"not_ready\"} 1"));
    assert!(collected.contains("starter_sql_operations_total{outcome=\"invalid_schema\"} 1"));
    assert!(collected.contains("starter_sql_admission_wait_seconds_sum 0\n"));
    for marker in [
        TOKEN,
        "private-",
        "PRIVATE_METHOD",
        "user_id",
        "request_id",
        "url",
        "path=",
    ] {
        assert!(
            !collected.contains(marker),
            "unexpected telemetry: {marker}"
        );
    }
    // Scrapes exclude themselves and never call SQL or change readiness.
    assert_eq!(collected, call(&router, "GET", "/metrics", &[], "").await.2);
    let independent = String::from_utf8(separate.encode().unwrap()).unwrap();
    assert!(!independent.contains("starter_http_requests_total{"));
    assert!(independent.contains("starter_readiness_last_probe -1\n"));
}
