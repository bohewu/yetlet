//! A candidate finalizer experiment, not shipped Session middleware.
//! Real Redis operations, an Axum HTTP socket and reqwest's actual cookie jar.
use axum::{
    Router,
    extract::State as AppState,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::cookie::{CookieStore, Jar};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{Notify, oneshot};
use yt066b_redis_spike::{State, Store, random_id};

#[derive(Clone, Copy)]
enum Pause {
    BeforeCheck,
    AfterCheck,
    AfterHeader,
}
struct Fixture {
    store: Store,
    old: String,
    fresh: Mutex<String>,
    phase: Pause,
    clear: bool,
    ready: Notify,
    release: Notify,
}
fn cookie(value: &str, clear: bool) -> String {
    // Test-only loopback HTTP exception. A real provider must default to Secure.
    if clear {
        "sid=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/".into()
    } else {
        format!("sid={value}; HttpOnly; SameSite=Lax; Path=/")
    }
}
fn response(directive: Option<String>) -> Response {
    let mut r = StatusCode::NO_CONTENT.into_response();
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    if let Some(value) = directive {
        r.headers_mut()
            .insert(header::SET_COOKIE, value.parse().unwrap());
    }
    r
}
async fn delayed(AppState(f): AppState<Arc<Fixture>>) -> Response {
    if matches!(f.phase, Pause::BeforeCheck) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    let valid = if f.clear {
        // A successful logout can have its clear delivered after a later new login.
        let snapshot = f.store.read(&f.old).await.unwrap();
        let t = f
            .store
            .prepare(
                "revoke",
                Some(&f.old),
                snapshot.revision,
                &State::new(0),
                2500,
            )
            .await
            .unwrap();
        f.store.execute(&t).await.unwrap().status == "ok"
    } else {
        f.store.read(&f.old).await.is_ok()
    };
    if matches!(f.phase, Pause::AfterCheck) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    let r = response(valid.then(|| cookie(&f.old, f.clear)));
    if matches!(f.phase, Pause::AfterHeader) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    r
}
async fn fresh(AppState(f): AppState<Arc<Fixture>>) -> Response {
    response(Some(cookie(&f.fresh.lock().unwrap(), false)))
}
async fn authenticated(AppState(f): AppState<Arc<Fixture>>, headers: HeaderMap) -> StatusCode {
    let id = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(';').find_map(|s| s.trim().strip_prefix("sid=")));
    match id {
        Some(id) if f.store.read(id).await.is_ok() => StatusCode::OK,
        _ => StatusCode::UNAUTHORIZED,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn real_cookie_jar_exposes_unresolved_ordering_gate() {
    let redis_url = std::env::var("YT_REDIS_URL").unwrap();
    for (phase, clear) in [
        (Pause::BeforeCheck, false),
        (Pause::AfterCheck, false),
        (Pause::AfterHeader, false),
        (Pause::AfterHeader, true),
    ] {
        let ns = random_id().unwrap();
        let a = Store::connect(&redis_url, &ns, 2).await.unwrap();
        let b = Store::connect(&redis_url, &ns, 2).await.unwrap();
        let initial = a
            .prepare("create", None, 0, &State::new(1), 60000)
            .await
            .unwrap();
        assert!(a.execute(&initial).await.unwrap().status == "ok");
        let fixture = Arc::new(Fixture {
            store: a.clone(),
            old: initial.identity().into(),
            fresh: Mutex::new(initial.identity().into()),
            phase,
            clear,
            ready: Notify::new(),
            release: Notify::new(),
        });
        let router = Router::new()
            .route("/delayed", get(delayed))
            .route("/fresh", get(fresh))
            .route("/auth", get(authenticated))
            .with_state(fixture.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = oneshot::channel();
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
        });
        let url: reqwest::Url = format!("http://{address}/").parse().unwrap();
        let jar = Arc::new(Jar::default());
        let client = reqwest::Client::builder()
            .no_proxy()
            .cookie_provider(jar.clone())
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        client.get(url.join("fresh").unwrap()).send().await.unwrap();
        let delayed_client = client.clone();
        let delayed_url = url.join("delayed").unwrap();
        let pending = tokio::spawn(async move { delayed_client.get(delayed_url).send().await });
        tokio::time::timeout(Duration::from_secs(2), fixture.ready.notified())
            .await
            .unwrap();
        let replacement = if clear {
            let t = b
                .prepare("create", None, 0, &State::new(2), 60000)
                .await
                .unwrap();
            assert!(b.execute(&t).await.unwrap().status == "ok");
            t.identity().to_owned()
        } else {
            let t = b
                .prepare("rotate", Some(initial.identity()), 0, &State::new(0), 60000)
                .await
                .unwrap();
            assert!(b.execute(&t).await.unwrap().status == "ok");
            t.replacement().unwrap().to_owned()
        };
        *fixture.fresh.lock().unwrap() = replacement.clone();
        let fresh_response = client.get(url.join("fresh").unwrap()).send().await.unwrap();
        assert!(
            fresh_response.headers().get(header::CACHE_CONTROL).unwrap() == "private, no-store"
        );
        assert!(jar.cookies(&url).unwrap().to_str().unwrap() == format!("sid={replacement}"));
        fixture.release.notify_one();
        let delayed_response = pending.await.unwrap().unwrap();
        assert!(a.read(initial.identity()).await.is_err() && b.read(&replacement).await.is_ok());
        if matches!(phase, Pause::BeforeCheck) {
            assert!(delayed_response.headers().get(header::SET_COOKIE).is_none());
            assert!(jar.cookies(&url).unwrap().to_str().unwrap() == format!("sid={replacement}"));
            assert!(
                client
                    .get(url.join("auth").unwrap())
                    .send()
                    .await
                    .unwrap()
                    .status()
                    == StatusCode::OK
            );
        } else {
            if clear {
                assert!(jar.cookies(&url).is_none());
            } else {
                assert!(
                    jar.cookies(&url).unwrap().to_str().unwrap()
                        == format!("sid={}", initial.identity())
                );
            }
            assert!(
                client
                    .get(url.join("auth").unwrap())
                    .send()
                    .await
                    .unwrap()
                    .status()
                    == StatusCode::UNAUTHORIZED
            );
            // Expected counterexample: the full-provider cookie-ordering gate remains UNSATISFIED.
        }
        stop.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        a.shutdown().await.unwrap();
        b.shutdown().await.unwrap();
    }
}
