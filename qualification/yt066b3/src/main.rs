//! Private isolated qualification fixture. Control routes are not product APIs.
use axum::{
    Extension, Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use yetlet_server_axum::App;
use yetlet_session_redis::{
    RedisOperationId, RedisOperationStatus, RedisSession, RedisSessionConfig,
    RedisSessionError as Error, RedisSessionManager as Manager,
    RedisSessionUpdateError as UpdateError,
};

#[derive(Clone, Serialize, Deserialize)]
struct Counter {
    count: u64,
}
#[derive(Default)]
struct Gate {
    entered: AtomicUsize,
    released: AtomicBool,
}
impl Gate {
    async fn wait(&self) -> bool {
        self.entered.fetch_add(1, Ordering::SeqCst);
        let end = Instant::now() + Duration::from_secs(3);
        while !self.released.load(Ordering::SeqCst) {
            if Instant::now() >= end {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        true
    }
    fn wait_sync(&self) -> bool {
        tokio::task::block_in_place(|| {
            self.entered.fetch_add(1, Ordering::SeqCst);
            let end = Instant::now() + Duration::from_secs(3);
            while !self.released.load(Ordering::SeqCst) {
                if Instant::now() >= end {
                    return false;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            true
        })
    }
}
#[derive(Clone)]
struct Lab {
    manager: Manager<Counter>,
    short: Manager<Counter>,
    gate: Arc<Gate>,
    callbacks: Arc<AtomicUsize>,
    ticket: Arc<Mutex<Option<RedisOperationId>>>,
    instance: String,
    prepared_cookie: Arc<Mutex<Option<axum::http::HeaderValue>>>,
}
#[derive(Serialize)]
struct Observation {
    instance: String,
    category: &'static str,
    count: Option<u64>,
    callbacks: usize,
}
fn response(lab: &Lab, status: StatusCode, category: &'static str, count: Option<u64>) -> Response {
    (
        status,
        Json(Observation {
            instance: lab.instance.clone(),
            category,
            count,
            callbacks: lab.callbacks.load(Ordering::SeqCst),
        }),
    )
        .into_response()
}
async fn error(lab: &Lab, e: Error) -> Response {
    let (status, category) = match e {
        Error::Missing | Error::Expired | Error::Rotated | Error::Revoked => {
            (StatusCode::UNAUTHORIZED, "retired_or_missing")
        }
        Error::Conflict { .. } => (StatusCode::CONFLICT, "conflict"),
        Error::Busy => (StatusCode::TOO_MANY_REQUESTS, "busy"),
        Error::QueueTimeout | Error::DispatchDeadline => (StatusCode::GATEWAY_TIMEOUT, "deadline"),
        Error::OutcomeUnknown(ticket) => {
            *lab.ticket.lock().await = Some(ticket);
            (StatusCode::ACCEPTED, "unknown")
        }
        _ => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
    };
    response(lab, status, category, None)
}
fn hold(headers: &HeaderMap, value: &str) -> bool {
    headers.get("x-lab-hold").and_then(|h| h.to_str().ok()) == Some(value)
}
async fn create(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
    headers: HeaderMap,
) -> Response {
    match s.create(Counter { count: 1 }).await {
        Ok(()) => {
            if hold(&headers, "before") && !lab.gate.wait().await {
                return response(&lab, StatusCode::GATEWAY_TIMEOUT, "gate_timeout", None);
            }
            response(&lab, StatusCode::CREATED, "created", Some(1))
        }
        Err(e) => error(&lab, e).await,
    }
}
async fn read(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
) -> Response {
    match s.read().await {
        Ok(v) => response(&lab, StatusCode::OK, "active", Some(v.count)),
        Err(e) => error(&lab, e).await,
    }
}
async fn perform_update(lab: &Lab, s: RedisSession<Counter>, gated: bool) -> Response {
    let result = s
        .try_update(|v| {
            lab.callbacks.fetch_add(1, Ordering::SeqCst);
            if gated && !lab.gate.wait_sync() {
                return Err(());
            }
            v.count += 1;
            // Nonserializable return: the original Box is consumed only on success.
            let count = v.count;
            Ok(Box::new(move || count))
        })
        .await;
    match result {
        Ok(value) => response(lab, StatusCode::OK, "updated", Some(value())),
        Err(UpdateError::Session(e)) => error(lab, e).await,
        Err(UpdateError::Application(())) => {
            response(lab, StatusCode::GATEWAY_TIMEOUT, "gate_timeout", None)
        }
    }
}
async fn update(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
    headers: HeaderMap,
) -> Response {
    perform_update(&lab, s, hold(&headers, "cas")).await
}
async fn cancel_update(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
) -> Response {
    let worker_lab = lab.clone();
    let task = tokio::spawn(async move { perform_update(&worker_lab, s, true).await });
    let end = Instant::now() + Duration::from_secs(2);
    while !lab.gate.released.load(Ordering::SeqCst) && Instant::now() < end {
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    task.abort();
    let cancelled = task.await.is_err();
    response(
        &lab,
        if cancelled {
            StatusCode::ACCEPTED
        } else {
            StatusCode::CONFLICT
        },
        if cancelled {
            "caller_cancelled"
        } else {
            "already_completed"
        },
        None,
    )
}
async fn rotate(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
) -> Response {
    match s.rotate().await {
        Ok(()) => response(&lab, StatusCode::OK, "rotated", None),
        Err(e) => error(&lab, e).await,
    }
}
async fn destroy(
    Extension(lab): Extension<Lab>,
    Extension(s): Extension<RedisSession<Counter>>,
    headers: HeaderMap,
) -> Response {
    match s.destroy().await {
        Ok(()) => {
            if hold(&headers, "before") && !lab.gate.wait().await {
                return response(&lab, StatusCode::GATEWAY_TIMEOUT, "gate_timeout", None);
            }
            response(&lab, StatusCode::OK, "destroyed", None)
        }
        Err(e) => error(&lab, e).await,
    }
}
async fn prepared(State(lab): State<Lab>, request: Request, next: Next) -> Response {
    let gated = hold(request.headers(), "after");
    let mut response = next.run(request).await;
    if let Some(cookie) = response.headers().get(header::SET_COOKIE) {
        *lab.prepared_cookie.lock().await = Some(cookie.clone());
    }
    if gated && !lab.gate.wait().await {
        response = (StatusCode::GATEWAY_TIMEOUT, "gate timeout").into_response();
    }
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; frame-ancestors 'none'"
            .parse()
            .unwrap(),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
async fn operation(Extension(lab): Extension<Lab>) -> Response {
    let ticket = lab.ticket.lock().await.clone();
    let Some(ticket) = ticket else {
        return response(&lab, StatusCode::NOT_FOUND, "no_ticket", None);
    };
    match lab.manager.resolve_operation(&ticket).await {
        Ok(RedisOperationStatus::Committed { .. }) => {
            response(&lab, StatusCode::OK, "committed", None)
        }
        Ok(RedisOperationStatus::Unresolved) => response(&lab, StatusCode::OK, "unresolved", None),
        Ok(RedisOperationStatus::Rejected(_)) => response(&lab, StatusCode::OK, "rejected", None),
        Err(e) => error(&lab, e).await,
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // No default listener; this binary is confined to the disposable Docker lab.
    assert!(std::env::var("YT_ISOLATED_LAB").as_deref() == Ok("1"));
    let namespace = std::env::var("YT_NAMESPACE").expect("isolated namespace");
    let endpoint = std::env::var("YT_REDIS_URL").expect("isolated Redis");
    let config = |ttl, scope| {
        RedisSessionConfig::new(
            namespace.clone(),
            scope,
            Duration::from_millis(ttl),
            1,
            1024,
        )
        .unwrap()
        .insecure_for_local_http()
    };
    let manager = Manager::connect(&endpoint, config(60000, "yt_b3"))
        .await
        .expect("Redis fixture ready");
    let short = Manager::connect(&endpoint, config(600, "yt_b3_short"))
        .await
        .expect("Redis fixture ready");
    let lab = Lab {
        manager: manager.clone(),
        short: short.clone(),
        gate: Arc::new(Gate::default()),
        callbacks: Arc::new(AtomicUsize::new(0)),
        ticket: Arc::new(Mutex::new(None)),
        instance: std::env::var("YT_INSTANCE").unwrap(),
        prepared_cookie: Arc::new(Mutex::new(None)),
    };
    let routes = || {
        Router::new()
            .route("/create", get(create).post(create))
            .route("/read", get(read))
            .route("/update", post(update))
            .route("/rotate", get(rotate).post(rotate))
            .route("/destroy", get(destroy).post(destroy))
            .route("/cancel-update", post(cancel_update))
    };
    let scoped = manager
        .clone()
        .scoped(App::from_router(routes()))
        .into_router();
    let short_scoped = short
        .clone()
        .scoped(App::from_router(routes()))
        .into_router();
    let router = scoped
        .nest("/short", short_scoped)
        .route(
            "/healthz",
            get({
                let lab = lab.clone();
                move || async move { lab.instance }
            }),
        )
        .route("/operation", get(operation))
        .route(
            "/control/replay",
            get({
                let lab = lab.clone();
                move || async move {
                    let mut response = StatusCode::OK.into_response();
                    if let Some(cookie) = lab.prepared_cookie.lock().await.clone() {
                        response.headers_mut().insert(header::SET_COOKIE, cookie);
                    }
                    response
                }
            }),
        )
        .route(
            "/control/reset",
            post({
                let lab = lab.clone();
                move || async move {
                    lab.gate.entered.store(0, Ordering::SeqCst);
                    lab.gate.released.store(false, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        )
        .route(
            "/control/entered",
            get({
                let lab = lab.clone();
                move || async move { lab.gate.entered.load(Ordering::SeqCst).to_string() }
            }),
        )
        .route(
            "/control/release",
            post({
                let lab = lab.clone();
                move || async move {
                    lab.gate.released.store(true, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        )
        .route(
            "/control/shutdown",
            post({
                let lab = lab.clone();
                move || async move {
                    let a = lab.manager.shutdown().await;
                    let b = lab.short.shutdown().await;
                    if a.is_ok() && b.is_ok() {
                        StatusCode::OK
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                }
            }),
        )
        .layer(Extension(lab.clone()))
        // Outermost delay is after the provider's final Cookie validation.
        .layer(from_fn_with_state(lab, prepared));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3010").await.unwrap();
    let shutdown = async move {
        tokio::signal::ctrl_c().await.unwrap();
        manager.shutdown().await.unwrap();
        short.shutdown().await.unwrap();
    };
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}
