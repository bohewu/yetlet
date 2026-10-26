#![allow(
    clippy::result_large_err,
    reason = "exercise native Axum rejection responses"
)]

use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{
    convert::Infallible,
    future::{Ready, ready},
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tower::{Layer, Service, ServiceExt, service_fn};
use yetlet_rate_limit::{Budget, Key, Policy, RateLimitLayer};

fn budget(burst: u32, cap: usize) -> Budget {
    Budget::new(Policy::new(Duration::from_secs(60), burst, cap).unwrap())
}
fn fixed(_: &Parts) -> Result<Key, Response> {
    Ok(Key::new("group").unwrap())
}
fn req(method: &str, path: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn shared_methods_routes_scopes_native_404_405_and_head() {
    let layer = RateLimitLayer::new(budget(2, 1), fixed);
    let app = Router::new()
        .route(
            "/a",
            get(|| async { "a" })
                .post(|| async { "posted" })
                .route_layer(layer.clone()),
        )
        .route("/b", get(|| async { "b" }).route_layer(layer))
        .route("/public", get(|| async { "public" }));
    for (method, path, expected) in [
        ("DELETE", "/a", StatusCode::METHOD_NOT_ALLOWED),
        ("GET", "/missing", StatusCode::NOT_FOUND),
        ("HEAD", "/a", StatusCode::OK),
        ("POST", "/a", StatusCode::OK),
        ("GET", "/b", StatusCode::TOO_MANY_REQUESTS),
        ("GET", "/a", StatusCode::TOO_MANY_REQUESTS),
        ("GET", "/public", StatusCode::OK),
        ("DELETE", "/a", StatusCode::METHOD_NOT_ALLOWED),
        ("GET", "/missing", StatusCode::NOT_FOUND),
    ] {
        let response = app.clone().oneshot(req(method, path)).await.unwrap();
        assert_eq!(response.status(), expected);
        if expected == StatusCode::METHOD_NOT_ALLOWED {
            let allow = response.headers()["allow"].to_str().unwrap();
            assert!(allow.contains("GET") && allow.contains("POST") && allow.contains("HEAD"));
        }
        if expected == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["retry-after"], "60");
            assert_eq!(
                to_bytes(response.into_body(), 1024).await.unwrap(),
                "Too many requests"
            );
        }
    }
}

#[derive(Clone)]
struct VerifiedAccount(&'static str);
fn account(parts: &Parts) -> Result<Key, Response> {
    parts
        .extensions
        .get::<VerifiedAccount>()
        .map(|account| Key::new(account.0).unwrap())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Verified identity required").into_response())
}
#[tokio::test]
async fn trusted_extension_missing_metadata_rejects_without_charge_and_headers_cannot_impersonate()
{
    let layer = RateLimitLayer::new(budget(1, 1), account);
    let app = Router::new().route("/", get(|| async { "ok" }).route_layer(layer));
    let mut forged = req("GET", "/");
    forged
        .headers_mut()
        .insert("x-forwarded-for", "trusted-account".parse().unwrap());
    forged
        .headers_mut()
        .insert("forwarded", "for=127.0.0.1".parse().unwrap());
    let response = app.clone().oneshot(forged).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
        to_bytes(response.into_body(), 1024).await.unwrap(),
        "Verified identity required"
    );
    for expected in [StatusCode::OK, StatusCode::TOO_MANY_REQUESTS] {
        let mut request = req("GET", "/");
        request
            .extensions_mut()
            .insert(VerifiedAccount("issuer/account/tenant"));
        request
            .headers_mut()
            .insert("x-forwarded-for", "different-each-request".parse().unwrap());
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            expected
        );
    }
}

struct UnreadBody;
impl http_body::Body for UnreadBody {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Bytes>, Infallible>>> {
        panic!("limiter must not poll request body")
    }
}
#[tokio::test]
async fn rejection_never_polls_body_or_calls_downstream_and_extractor_runs_outside_lock() {
    let shared = budget(1, 1);
    let first = RateLimitLayer::new(shared.clone(), fixed).layer(service_fn(
        |_: Request<UnreadBody>| async { Ok::<_, Infallible>("ok".into_response()) },
    ));
    assert_eq!(
        first
            .oneshot(Request::new(UnreadBody))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let service =
        RateLimitLayer::new(shared, fixed).layer(service_fn(move |_: Request<UnreadBody>| {
            seen.fetch_add(1, Ordering::SeqCst);
            async { Ok::<_, Infallible>("never".into_response()) }
        }));
    assert_eq!(
        service
            .oneshot(Request::new(UnreadBody))
            .await
            .unwrap()
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let reject = RateLimitLayer::new(budget(1, 1), |_: &Parts| {
        Err((StatusCode::FORBIDDEN, "application rejection").into_response())
    })
    .layer(service_fn(|_: Request<UnreadBody>| async {
        panic!("no downstream");
        #[allow(unreachable_code)]
        Ok::<_, Infallible>("".into_response())
    }));
    assert_eq!(
        reject
            .oneshot(Request::new(UnreadBody))
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
}

struct ReadyInstance {
    ready: bool,
    ready_polls: Arc<AtomicUsize>,
    release: Arc<AtomicBool>,
}
impl Clone for ReadyInstance {
    fn clone(&self) -> Self {
        Self {
            ready: false,
            ready_polls: self.ready_polls.clone(),
            release: self.release.clone(),
        }
    }
}
impl Service<Request<Body>> for ReadyInstance {
    type Response = Response;
    type Error = &'static str;
    type Future = Ready<Result<Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.ready_polls.fetch_add(1, Ordering::SeqCst);
        if !self.release.load(Ordering::SeqCst) {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.ready = true;
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: Request<Body>) -> Self::Future {
        assert!(
            std::mem::take(&mut self.ready),
            "called unready cloned instance"
        );
        ready(Err("native downstream error"))
    }
}
#[tokio::test]
async fn native_readiness_backpressure_same_instance_and_downstream_error_charge() {
    let polls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(AtomicBool::new(false));
    let mut service = RateLimitLayer::new(budget(1, 1), fixed).layer(ReadyInstance {
        ready: false,
        ready_polls: polls.clone(),
        release: release.clone(),
    });
    let mut pending = Box::pin(service.ready());
    let waker = std::task::Waker::noop();
    assert!(
        pending
            .as_mut()
            .poll(&mut Context::from_waker(waker))
            .is_pending()
    );
    drop(pending);
    assert_eq!(polls.load(Ordering::SeqCst), 1);
    release.store(true, Ordering::SeqCst);
    assert_eq!(
        service
            .ready()
            .await
            .unwrap()
            .call(req("GET", "/"))
            .await
            .unwrap_err(),
        "native downstream error"
    );
    assert_eq!(
        service
            .ready()
            .await
            .unwrap()
            .call(req("GET", "/"))
            .await
            .unwrap()
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}
#[tokio::test]
async fn cancellation_and_stacked_denials_never_refund() {
    let shared = budget(1, 1);
    let mut pending =
        RateLimitLayer::new(shared.clone(), fixed).layer(service_fn(|_: Request<Body>| {
            std::future::pending::<Result<Response, Infallible>>()
        }));
    let future = pending.ready().await.unwrap().call(req("GET", "/"));
    drop(future); // Charge happened on call, even though downstream never finished.
    let check = RateLimitLayer::new(shared, fixed).layer(service_fn(|_: Request<Body>| async {
        Ok::<_, Infallible>("ok".into_response())
    }));
    assert_eq!(
        check.oneshot(req("GET", "/")).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );

    let inner_budget = budget(1, 1);
    let outer_budget = budget(2, 1);
    let inner = RateLimitLayer::new(inner_budget, fixed);
    let endpoint =
        || service_fn(|_: Request<Body>| async { Ok::<_, Infallible>("ok".into_response()) });
    let app = RateLimitLayer::new(outer_budget.clone(), fixed).layer(inner.layer(endpoint()));
    assert_eq!(
        app.clone().oneshot(req("GET", "/")).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(req("GET", "/")).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    let outer_only = RateLimitLayer::new(outer_budget, fixed).layer(endpoint());
    assert_eq!(
        outer_only.oneshot(req("GET", "/")).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

fn peer(parts: &Parts) -> Result<Key, Response> {
    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|peer| Key::new(peer.0.ip().to_string()).unwrap())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Transport identity required").into_response())
}
struct Server {
    address: SocketAddr,
    client: reqwest::Client,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}
impl Server {
    async fn start(app: Router) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, shutdown) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = shutdown.await;
            })
            .await
            .unwrap();
        });
        Self {
            address,
            client: reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            stop: Some(stop),
            task: Some(task),
        }
    }
    async fn finish(mut self) {
        let _ = self.stop.take().unwrap().send(());
        let mut task = self.task.take().unwrap();
        if tokio::time::timeout(Duration::from_secs(3), &mut task)
            .await
            .is_err()
        {
            task.abort();
            let _ = task.await;
            panic!("owned server shutdown timed out");
        }
        assert!(
            tokio::net::TcpStream::connect(self.address).await.is_err(),
            "listener closed"
        );
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
#[tokio::test]
async fn actual_http_peer_spoof_scope_and_headers_with_owned_shutdown() {
    let layer = RateLimitLayer::new(budget(2, 1), peer);
    let app = Router::new()
        .route(
            "/a",
            get(|| async { "a" })
                .post(|| async { "posted" })
                .route_layer(layer.clone()),
        )
        .route("/b", get(|| async { "b" }).route_layer(layer))
        .route("/public", get(|| async { "public" }));
    let server = Server::start(app).await;
    for (method, path, code, spoof) in [
        ("DELETE", "/a", 405, "1.1.1.1"),
        ("GET", "/absent", 404, "2.2.2.2"),
        ("GET", "/a", 200, "1.1.1.1"),
        ("POST", "/a", 200, "2.2.2.2"),
        ("GET", "/b", 429, "3.3.3.3"),
        ("GET", "/public", 200, "4.4.4.4"),
        ("DELETE", "/a", 405, "5.5.5.5"),
        ("GET", "/absent", 404, "6.6.6.6"),
    ] {
        let response = server
            .client
            .request(
                method.parse().unwrap(),
                format!("http://{}{path}", server.address),
            )
            .header("x-forwarded-for", spoof)
            .header("forwarded", format!("for={spoof}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), code);
        if code == 429 {
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["retry-after"], "60");
            assert!(!response.headers().contains_key("set-cookie"));
            assert_eq!(response.text().await.unwrap(), "Too many requests");
        }
    }
    server.finish().await;
}
#[tokio::test]
async fn actual_http_new_key_capacity_503_preserves_existing_key() {
    // Only this fixture assigns verified typed identities. The limiter itself
    // never parses header identities; the native router establishes each key.
    let shared = budget(1, 1);
    let a = RateLimitLayer::new(shared.clone(), |_: &Parts| Ok(Key::new("a").unwrap()));
    let b = RateLimitLayer::new(shared, |_: &Parts| Ok(Key::new("b").unwrap()));
    let app = Router::new()
        .route("/a", get(|| async { "a" }).route_layer(a))
        .route("/b", get(|| async { "b" }).route_layer(b));
    let server = Server::start(app).await;
    for (path, code) in [("/a", 200), ("/b", 503), ("/a", 429)] {
        let response = server
            .client
            .get(format!("http://{}{path}", server.address))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), code);
        if code == 503 {
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert!(!response.headers().contains_key("retry-after"));
            assert_eq!(response.text().await.unwrap(), "Service unavailable");
        }
    }
    server.finish().await;
}
