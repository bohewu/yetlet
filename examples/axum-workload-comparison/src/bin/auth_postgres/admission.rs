use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::Response,
};
use std::{ffi::OsStr, sync::Arc};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(super) struct HttpAdmission {
    permits: Arc<Semaphore>,
}

fn parse_cap(value: Option<&OsStr>) -> Result<Option<usize>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.to_str().ok_or(())?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    let cap = value.parse::<usize>().map_err(|_| ())?;
    if !(1..=1024).contains(&cap) {
        return Err(());
    }
    Ok(Some(cap))
}

pub(super) fn from_environment() -> Result<Option<HttpAdmission>, ()> {
    let value = std::env::var_os("YT176_HTTP_MAX_IN_FLIGHT");
    Ok(parse_cap(value.as_deref())?.map(|cap| HttpAdmission {
        permits: Arc::new(Semaphore::new(cap)),
    }))
}

pub(super) fn install<S>(router: Router<S>, admission: Option<HttpAdmission>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    match admission {
        None => router,
        Some(admission) => router.route_layer(middleware::from_fn_with_state(admission, admit)),
    }
}

async fn admit(State(admission): State<HttpAdmission>, request: Request, next: Next) -> Response {
    let Ok(_permit) = admission.permits.try_acquire_owned() else {
        let mut response = super::fail(StatusCode::SERVICE_UNAVAILABLE);
        let headers = response.headers_mut();
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        headers.insert(header::VARY, HeaderValue::from_static("Cookie"));
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        return response;
    };
    // The inner Session middleware must finish before this HTTP permit drops.
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Method,
        response::IntoResponse,
        routing::{get, post},
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::{sync::Notify, time::Duration, time::timeout};
    use tower::ServiceExt;

    const BOUND: Duration = Duration::from_secs(5);

    #[test]
    fn cap_parser_is_optional_and_strict() {
        assert_eq!(parse_cap(None), Ok(None));
        for (input, cap) in [("1", 1), ("4", 4), ("1024", 1024), ("0004", 4)] {
            assert_eq!(parse_cap(Some(OsStr::new(input))), Ok(Some(cap)));
        }
        for input in [
            "",
            "0",
            "000",
            "1025",
            "999999999999999999999999999999999999",
            "-1",
            "+1",
            " 4",
            "4 ",
            "4\n",
            "1.0",
            "1_0",
            "４",
            "secret=value",
        ] {
            assert_eq!(parse_cap(Some(OsStr::new(input))), Err(()));
        }
    }

    #[cfg(unix)]
    #[test]
    fn cap_parser_rejects_non_unicode_without_environment_mutation() {
        use std::os::unix::ffi::OsStrExt;
        assert_eq!(parse_cap(Some(OsStr::from_bytes(b"4\xff"))), Err(()));
    }

    #[derive(Default)]
    struct Probe {
        sessions: AtomicUsize,
        auth: AtomicUsize,
        handlers: AtomicUsize,
        finalizer_entered: Notify,
        release_finalizer: Notify,
    }

    impl Probe {
        fn counts(&self) -> [usize; 3] {
            [
                self.sessions.load(Ordering::SeqCst),
                self.auth.load(Ordering::SeqCst),
                self.handlers.load(Ordering::SeqCst),
            ]
        }
    }

    async fn session(State(probe): State<Arc<Probe>>, request: Request, next: Next) -> Response {
        probe.sessions.fetch_add(1, Ordering::SeqCst);
        let held = request.uri().path() == "/api/documents/hold";
        let mut response = next.run(request).await;
        if held {
            // A synthetic finalizer after handler completion, not live Valkey/SQL proof.
            probe.finalizer_entered.notify_one();
            probe.release_finalizer.notified().await;
        }
        response
            .headers_mut()
            .insert("x-finalized", HeaderValue::from_static("yes"));
        response
    }

    async fn auth(State(probe): State<Arc<Probe>>, request: Request, next: Next) -> Response {
        probe.auth.fetch_add(1, Ordering::SeqCst);
        next.run(request).await
    }

    async fn handler(
        State(probe): State<Arc<Probe>>,
        request: Request,
    ) -> Result<Response, Response> {
        probe.handlers.fetch_add(1, Ordering::SeqCst);
        if request.uri().path() == "/api/documents/error" {
            return Err((StatusCode::BAD_GATEWAY, "downstream unavailable").into_response());
        }
        Ok((
            StatusCode::ACCEPTED,
            [
                (header::SET_COOKIE, "reference=1; HttpOnly"),
                (header::LOCATION, "/preserved"),
                (header::RETRY_AFTER, "9"),
            ],
            "original response",
        )
            .into_response())
    }

    fn scoped(probe: Arc<Probe>) -> Router {
        Router::new()
            .route("/api/documents/{id}", get(handler))
            .route("/private/documents/{id}", get(handler))
            .route("/auth/me", get(handler))
            .route("/auth/logout-form", get(handler))
            .route("/auth/logout", post(handler))
            .route("/auth/form", get(handler))
            .route("/auth/login", post(handler))
            .route_layer(middleware::from_fn_with_state(probe.clone(), auth))
            .route_layer(middleware::from_fn_with_state(probe.clone(), session))
            .with_state(probe)
    }

    fn fixture() -> (Router, HttpAdmission, Arc<Probe>) {
        let probe = Arc::new(Probe::default());
        let admission = HttpAdmission {
            permits: Arc::new(Semaphore::new(1)),
        };
        let router = Router::new()
            .merge(install(scoped(probe.clone()), Some(admission.clone())))
            .route("/health", get(|| async { "ready" }))
            .route("/fixture/observations", get(|| async { "observed" }))
            .layer(middleware::from_fn(super::super::security));
        (router, admission, probe)
    }

    async fn send(router: &Router, method: Method, path: &str) -> Response {
        timeout(
            BOUND,
            router.clone().oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            ),
        )
        .await
        .expect("request must finish within the test bound")
        .unwrap()
    }

    async fn body(response: Response) -> Vec<u8> {
        to_bytes(response.into_body(), 4096).await.unwrap().to_vec()
    }

    async fn assert_rejected(response: Response) {
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let headers = response.headers();
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert_eq!(headers[header::VARY], "Cookie");
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert_eq!(
            headers[header::CONTENT_SECURITY_POLICY],
            super::super::DEFAULT_CSP
        );
        assert_eq!(headers[header::CONTENT_TYPE], "text/plain; charset=utf-8");
        for name in [
            header::SET_COOKIE,
            header::LOCATION,
            header::RETRY_AFTER,
            header::WWW_AUTHENTICATE,
            header::PROXY_AUTHENTICATE,
        ] {
            assert!(!headers.contains_key(name));
        }
        assert!(!headers.contains_key("x-finalized"));
        assert_eq!(body(response).await, b"Request unavailable or denied");
    }

    async fn assert_original(response: Response) {
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers()[header::SET_COOKIE],
            "reference=1; HttpOnly"
        );
        assert_eq!(response.headers()[header::LOCATION], "/preserved");
        assert_eq!(response.headers()[header::RETRY_AFTER], "9");
        assert_eq!(response.headers()["x-finalized"], "yes");
        assert_eq!(body(response).await, b"original response");
    }

    #[tokio::test]
    async fn disabled_installation_preserves_inner_response() {
        let probe = Arc::new(Probe::default());
        let router = install(scoped(probe.clone()), None);
        let response = send(&router, Method::GET, "/auth/me").await;
        assert!(!response.headers().contains_key(header::CACHE_CONTROL));
        assert!(!response.headers().contains_key(header::VARY));
        assert_original(response).await;
        assert_eq!(probe.counts(), [1, 1, 1]);
    }

    #[tokio::test]
    async fn shared_gate_rejects_before_inner_work_through_finalization_and_recovers() {
        let (router, admission, probe) = fixture();
        let held_router = router.clone();
        let held =
            tokio::spawn(
                async move { send(&held_router, Method::GET, "/api/documents/hold").await },
            );
        timeout(BOUND, probe.finalizer_entered.notified())
            .await
            .expect("handler must reach the inner finalizer");
        assert_eq!(probe.counts(), [1, 1, 1]);
        assert_eq!(admission.permits.available_permits(), 0);
        for (method, path) in [
            (Method::GET, "/api/documents/1"),
            (Method::GET, "/private/documents/1"),
            (Method::GET, "/auth/me"),
            (Method::GET, "/auth/logout-form"),
            (Method::POST, "/auth/logout"),
            (Method::GET, "/auth/form"),
            (Method::POST, "/auth/login"),
        ] {
            assert_rejected(send(&router, method, path).await).await;
            assert_eq!(probe.counts(), [1, 1, 1]);
        }
        for (path, expected) in [("/health", "ready"), ("/fixture/observations", "observed")] {
            let response = send(&router, Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(body(response).await, expected.as_bytes());
        }
        assert_eq!(probe.counts(), [1, 1, 1]);
        probe.release_finalizer.notify_one();
        let response = timeout(BOUND, held)
            .await
            .expect("held task must complete")
            .expect("held task must be reaped successfully");
        assert_original(response).await;
        assert_eq!(admission.permits.available_permits(), 1);
        assert_original(send(&router, Method::GET, "/auth/me").await).await;
        assert_eq!(probe.counts(), [2, 2, 2]);
        assert_eq!(admission.permits.available_permits(), 1);
    }

    #[tokio::test]
    async fn downstream_error_and_cancel_release_http_capacity_without_retry() {
        let (router, admission, probe) = fixture();
        let response = send(&router, Method::GET, "/api/documents/error").await;
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(body(response).await, b"downstream unavailable");
        assert_eq!(probe.counts(), [1, 1, 1]);
        assert_eq!(admission.permits.available_permits(), 1);

        let held_router = router.clone();
        let held =
            tokio::spawn(
                async move { send(&held_router, Method::GET, "/api/documents/hold").await },
            );
        timeout(BOUND, probe.finalizer_entered.notified())
            .await
            .expect("handler must reach the inner finalizer");
        assert_eq!(probe.counts(), [2, 2, 2]);
        assert_rejected(send(&router, Method::GET, "/auth/me").await).await;
        held.abort();
        let error = timeout(BOUND, held)
            .await
            .expect("aborted task must be reaped")
            .expect_err("request must have been cancelled");
        assert!(error.is_cancelled());
        assert_eq!(probe.counts(), [2, 2, 2]);
        assert_eq!(admission.permits.available_permits(), 1);
        assert_original(send(&router, Method::GET, "/auth/me").await).await;
        assert_eq!(probe.counts(), [3, 3, 3]);
        assert_eq!(admission.permits.available_permits(), 1);
    }

    #[tokio::test]
    async fn closed_gate_uses_the_same_private_rejection() {
        let (router, admission, probe) = fixture();
        admission.permits.close();
        assert_rejected(send(&router, Method::GET, "/auth/me").await).await;
        assert_eq!(probe.counts(), [0, 0, 0]);
    }
}
