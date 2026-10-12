//! YT-029 application-owned experiment, not a supported Yetlet core API.

use std::time::Duration;

use axum::{BoxError, Router, error_handling::HandleErrorLayer, http::StatusCode};
use tower::{ServiceBuilder, timeout::TimeoutLayer};
use yetlet_server_axum::middleware::with_global;

/// Explicitly bound service execution until a response is returned.
///
/// Does not bound subsequent response-body streaming or cancel detached work.
/// Complete route registration before installing this outer global envelope.
pub fn with_deadline(router: Router, budget: Duration) -> Router {
    with_global(
        router,
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|error: BoxError| async move {
                if error.is::<tower::timeout::error::Elapsed>() {
                    StatusCode::GATEWAY_TIMEOUT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }))
            .layer(TimeoutLayer::new(budget)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, Bytes, to_bytes},
        http::Request,
        routing::get,
    };
    use futures_util::stream;
    use std::{
        convert::Infallible,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };
    use tokio::{
        sync::oneshot,
        time::{Instant, sleep},
    };
    use tower::ServiceExt;

    const BUDGET: Duration = Duration::from_millis(100);

    fn request() -> Request<Body> {
        Request::builder().uri("/").body(Body::empty()).unwrap()
    }

    #[tokio::test(start_paused = true)]
    async fn success_timeout_and_unlayered_default() {
        let fast = Router::new().route("/", get(|| async { "ok" }));
        let response = with_deadline(fast, BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response.into_body(), 16).await.unwrap(), "ok");

        let slow = Router::new().route(
            "/",
            get(|| async {
                sleep(BUDGET * 2).await;
                "completed"
            }),
        );
        let start = Instant::now();
        let response = with_deadline(slow.clone(), BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(Instant::now() - start, BUDGET);
        let start = Instant::now();
        let response = slow.oneshot(request()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), 16).await.unwrap(),
            "completed"
        );
        assert_eq!(Instant::now() - start, BUDGET * 2);
    }

    struct Dropped(Arc<AtomicUsize>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_drops_pending_future_without_rolling_back_completed_effect() {
        let effects = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let handler_effects = effects.clone();
        let handler_drops = drops.clone();
        let router = Router::new().route(
            "/",
            get(move || {
                let effects = handler_effects.clone();
                let drops = handler_drops.clone();
                async move {
                    let _guard = Dropped(drops);
                    effects.fetch_add(1, Ordering::SeqCst);
                    sleep(BUDGET * 2).await;
                    effects.fetch_add(1, Ordering::SeqCst);
                    "completed"
                }
            }),
        );
        let response = with_deadline(router, BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        sleep(BUDGET * 3).await;
        assert_eq!(effects.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn spawned_work_survives_timeout_and_is_joined() {
        let effects = Arc::new(AtomicUsize::new(0));
        let worker_effects = effects.clone();
        let (send_handle, recv_handle) = oneshot::channel();
        let fixture = Arc::new(Mutex::new(Some((worker_effects, send_handle))));
        let router = Router::new().route(
            "/",
            get(move || {
                let (worker_effects, send_handle) = fixture.lock().unwrap().take().unwrap();
                async move {
                    let handle = tokio::spawn(async move {
                        sleep(BUDGET * 2).await;
                        worker_effects.fetch_add(1, Ordering::SeqCst);
                    });
                    send_handle.send(handle).unwrap();
                    std::future::pending::<&'static str>().await
                }
            }),
        );
        let response = with_deadline(router, BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(effects.load(Ordering::SeqCst), 0);
        recv_handle.await.unwrap().await.unwrap();
        assert_eq!(effects.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn started_blocking_work_survives_timeout_and_abort_and_is_joined() {
        let effects = Arc::new(AtomicUsize::new(0));
        let worker_effects = effects.clone();
        let (release, wait_release) = std::sync::mpsc::channel();
        let (started, wait_started) = oneshot::channel();
        let handle = tokio::task::spawn_blocking(move || {
            started.send(()).unwrap();
            wait_release.recv().unwrap();
            worker_effects.fetch_add(1, Ordering::SeqCst);
        });
        // Establish that the work has started before dispatching the request.
        wait_started.await.unwrap();
        let (send_handle, recv_handle) = oneshot::channel();
        let fixture = Arc::new(Mutex::new(Some((handle, send_handle))));
        let router = Router::new().route(
            "/",
            get(move || {
                let (handle, send_handle) = fixture.lock().unwrap().take().unwrap();
                async move {
                    send_handle.send(handle).unwrap();
                    std::future::pending::<&'static str>().await
                }
            }),
        );
        let response = with_deadline(router, BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let handle = recv_handle.await.unwrap();
        assert_eq!(effects.load(Ordering::SeqCst), 0);
        handle.abort();
        release.send(()).unwrap();
        handle.await.unwrap();
        assert_eq!(effects.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn request_body_read_inside_handler_is_bounded() {
        let router = Router::new().route(
            "/",
            get(|request: Request<Body>| async {
                to_bytes(request.into_body(), 16).await.unwrap();
                "read"
            }),
        );
        let body = Body::from_stream(stream::once(async {
            sleep(BUDGET * 2).await;
            Ok::<_, Infallible>(Bytes::from_static(b"late"))
        }));
        let request = Request::builder().uri("/").body(body).unwrap();
        let response = with_deadline(router, BUDGET)
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[tokio::test(start_paused = true)]
    async fn response_body_stream_outlives_service_deadline() {
        let router = Router::new().route(
            "/",
            get(|| async {
                Body::from_stream(stream::once(async {
                    sleep(BUDGET * 2).await;
                    Ok::<_, Infallible>(Bytes::from_static(b"late"))
                }))
            }),
        );
        let response = with_deadline(router, BUDGET)
            .oneshot(request())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let start = Instant::now();
        assert_eq!(to_bytes(response.into_body(), 16).await.unwrap(), "late");
        assert_eq!(Instant::now() - start, BUDGET * 2);
    }
}
