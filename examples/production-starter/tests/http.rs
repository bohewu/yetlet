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
use yetlet_production_starter::{
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
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let (status, headers, _) = call(
        &router,
        "GET",
        "/secret-path-marker?secret-query-marker",
        &[
            ("authorization", &bearer()),
            ("x-yetlet-request-id", "999999"),
            ("x-private", "secret-header-marker"),
        ],
        "secret-body-marker",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_ne!(headers["x-yetlet-request-id"], "999999");
    let logs = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
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
        "999999",
    ] {
        assert!(!logs.contains(marker));
    }
}
