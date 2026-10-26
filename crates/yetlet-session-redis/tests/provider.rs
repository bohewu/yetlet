use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Extension,
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use reqwest::cookie::{CookieStore, Jar};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tower::ServiceExt;
use yetlet_server_axum::App;
use yetlet_session_redis::{
    RedisOperationStatus, RedisSession, RedisSessionConfig, RedisSessionError as Error,
    RedisSessionManager as Manager, RedisSessionUpdateError as UpdateError,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cart {
    count: u64,
    note: String,
}
fn cart(count: u64) -> Cart {
    Cart {
        count,
        note: String::new(),
    }
}
fn endpoint() -> String {
    std::env::var("YT_REDIS_URL").expect("select disposable Redis fixture")
}
fn config(ttl: u64, capacity: usize) -> RedisSessionConfig {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    RedisSessionConfig::new(
        format!(
            "b2_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ),
        "b2",
        Duration::from_millis(ttl),
        capacity,
        1024,
    )
    .unwrap()
    .insecure_for_local_http()
}
async fn manager() -> Manager<Cart> {
    Manager::connect(&endpoint(), config(60000, 2))
        .await
        .unwrap()
}
fn headers(response: &Response) -> HeaderMap {
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, cookie.parse().unwrap());
    headers
}
async fn request(router: Router, path: &str, method: &str, headers: &HeaderMap) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    *request.headers_mut() = headers.clone();
    router.oneshot(request).await.unwrap()
}
async fn create(Extension(s): Extension<RedisSession<Cart>>) -> Response {
    match s.create(cart(1)).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
async fn create_update(Extension(s): Extension<RedisSession<Cart>>) -> Response {
    s.create(cart(1)).await.unwrap();
    s.try_update(|v| {
        v.count += 1;
        Ok::<_, ()>(())
    })
    .await
    .unwrap();
    StatusCode::CREATED.into_response()
}
async fn read(Extension(s): Extension<RedisSession<Cart>>) -> Response {
    match s.read().await {
        Ok(value) => value.count.to_string().into_response(),
        Err(Error::Missing | Error::Revoked | Error::Rotated | Error::Expired) => {
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
async fn rotate(Extension(s): Extension<RedisSession<Cart>>) -> Response {
    match s.rotate().await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::UNAUTHORIZED,
    }
    .into_response()
}
async fn destroy(Extension(s): Extension<RedisSession<Cart>>) -> Response {
    match s.destroy().await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::UNAUTHORIZED,
    }
    .into_response()
}
async fn rejected(Extension(s): Extension<RedisSession<Cart>>) -> StatusCode {
    s.create(cart(1)).await.unwrap();
    StatusCode::BAD_REQUEST
}
fn app(manager: Manager<Cart>) -> Router {
    let scoped = App::from_router(
        Router::new()
            .route("/create", post(create))
            .route("/create-update", post(create_update))
            .route("/read", get(read))
            .route("/rotate", post(rotate))
            .route("/destroy", post(destroy))
            .route("/rejected", post(rejected)),
    );
    manager
        .scoped(scoped)
        .into_router()
        .route("/health", get(|| async { "healthy" }))
}
async fn issued(manager: &Manager<Cart>) -> HeaderMap {
    let r = request(app(manager.clone()), "/create", "POST", &HeaderMap::new()).await;
    assert!(r.status() == StatusCode::CREATED);
    headers(&r)
}

#[test]
fn explicit_configuration_bounds_and_secure_name() {
    let c = RedisSessionConfig::new("app", "sid", Duration::from_secs(60), 2, 1024).unwrap();
    assert!(c.cookie_name() == "__Host-sid");
    assert!(c.insecure_for_local_http().cookie_name() == "sid");
    for millis in [1, 60_001, 1_800_000, 2_592_000_000] {
        assert!(
            RedisSessionConfig::new("app", "sid", Duration::from_millis(millis), 1, 10).is_ok()
        );
    }
    assert!(RedisSessionConfig::new("app", "sid", Duration::MAX, 1, 10).is_err());
    for (namespace, scope, ttl, capacity, payload) in [
        ("", "sid", 1000, 1, 10),
        ("app:*", "sid", 1000, 1, 10),
        ("app", "bad;", 1000, 1, 10),
        ("app", "sid", 0, 1, 10),
        ("app", "sid", 2_592_000_001, 1, 10),
        ("app", "sid", 1000, 0, 10),
        ("app", "sid", 1000, 5, 10),
        ("app", "sid", 1000, 1, 4097),
    ] {
        assert!(
            RedisSessionConfig::new(
                namespace,
                scope,
                Duration::from_millis(ttl),
                capacity,
                payload
            )
            .is_err()
        );
    }
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn configured_long_lifetime_reaches_store_and_cookie_without_sliding_expiry() {
    for ttl_ms in [1_800_000, 2_592_000_000] {
        let config = config(ttl_ms, 2);
        let a = Manager::<Cart>::connect(&endpoint(), config.clone())
            .await
            .unwrap();
        let b = Manager::<Cart>::connect(&endpoint(), config).await.unwrap();
        let response = request(app(a.clone()), "/create", "POST", &HeaderMap::new()).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert!(
            response.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains(&format!("Max-Age={};", ttl_ms / 1000))
        );
        let h = headers(&response);
        let id = h[header::COOKIE]
            .to_str()
            .unwrap()
            .split_once('=')
            .unwrap()
            .1;
        let mut observer = connection().await;
        // Each test namespace is isolated; discover its one active session by the
        // opaque identity rather than exposing provider configuration fields.
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("yetlet:redis:v1:*:session:{id}"))
            .query_async(&mut observer)
            .await
            .unwrap();
        assert_eq!(keys.len(), 1);
        let key = &keys[0];
        let remaining: i64 = redis::cmd("PTTL")
            .arg(key)
            .query_async(&mut observer)
            .await
            .unwrap();
        assert!(((ttl_ms as i64 - 10_000)..=ttl_ms as i64).contains(&remaining));
        let expires: String = redis::cmd("HGET")
            .arg(key)
            .arg("expires")
            .query_async(&mut observer)
            .await
            .unwrap();
        let s = b.open(&h).unwrap();
        assert_eq!(s.read().await.unwrap().count, 1);
        s.try_update(|cart| {
            cart.count += 1;
            Ok::<_, ()>(())
        })
        .await
        .unwrap();
        let after: String = redis::cmd("HGET")
            .arg(key)
            .arg("expires")
            .query_async(&mut observer)
            .await
            .unwrap();
        assert_eq!(after, expires);
        let rotated = request(app(b.clone()), "/rotate", "POST", &h).await;
        assert_eq!(rotated.status(), StatusCode::NO_CONTENT);
        assert!(
            rotated.headers()[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains(&format!("Max-Age={};", ttl_ms / 1000))
        );
        assert!(a.open(&h).unwrap().read().await.is_err());
        let newh = headers(&rotated);
        assert_eq!(a.open(&newh).unwrap().read().await.unwrap().count, 2);
        b.open(&newh).unwrap().destroy().await.unwrap();
        assert!(a.open(&newh).unwrap().read().await.is_err());
        a.shutdown().await.unwrap();
        b.shutdown().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn typed_lifecycle_csrf_callbacks_and_independent_managers() {
    let config = config(60000, 2);
    let a = Manager::<Cart>::connect(&endpoint(), config.clone())
        .await
        .unwrap();
    let b = Manager::<Cart>::connect(&endpoint(), config).await.unwrap();
    let h = issued(&a).await;
    let sa = a.open(&h).unwrap();
    let sb = b.open(&h).unwrap();
    assert!(sb.read().await.unwrap().count == 1);
    let token = sa.csrf_token().await.unwrap();
    assert!(sb.verify_csrf(&token).await.unwrap());
    assert!(!sb.verify_csrf("wrong").await.unwrap());
    let original = Box::new(9);
    let address = &*original as *const i32 as usize;
    let returned = sa
        .try_update(|s| {
            s.count += 1;
            Ok::<_, ()>(original)
        })
        .await
        .unwrap();
    assert!(&*returned as *const i32 as usize == address);
    let error = sa
        .try_update(|s| {
            s.count = 99;
            Err::<(), _>("application rejection")
        })
        .await;
    assert!(matches!(error, Err(UpdateError::Application(_))));
    assert!(sb.read().await.unwrap().count == 2);
    sa.rotate().await.unwrap();
    assert!(matches!(
        sb.read().await,
        Err(Error::Rotated | Error::Missing)
    ));
    assert!(sa.read().await.unwrap().count == 2);
    assert!(!sa.verify_csrf(&token).await.unwrap());
    sa.replace_and_rotate(cart(7)).await.unwrap();
    assert!(sa.read().await.unwrap().count == 7);
    sa.destroy().await.unwrap();
    assert!(matches!(sa.read().await, Err(Error::Missing)));
    assert!(matches!(
        sa.try_update(|_| Ok::<_, ()>(())).await,
        Err(UpdateError::Session(Error::Missing))
    ));
    assert!(a.open(&HeaderMap::new()).unwrap().read().await.is_err());
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn concurrent_cas_calls_run_once_and_rotation_has_one_winner() {
    let a = manager().await;
    let h = issued(&a).await;
    let barrier = Arc::new(Barrier::new(2));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    for _ in 0..2 {
        let s = a.open(&h).unwrap();
        let barrier = barrier.clone();
        let calls = calls.clone();
        tasks.push(tokio::spawn(async move {
            s.try_update(|v| {
                calls.fetch_add(1, Ordering::SeqCst);
                tokio::task::block_in_place(|| barrier.wait());
                v.count += 1;
                Ok::<_, ()>(())
            })
            .await
        }));
    }
    let mut successes = 0;
    let mut conflicts = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(()) => successes += 1,
            Err(UpdateError::Session(Error::Conflict { .. })) => conflicts += 1,
            _ => panic!("unexpected CAS category"),
        }
    }
    assert!(successes == 1 && conflicts == 1 && calls.load(Ordering::SeqCst) == 2);
    assert!(a.open(&h).unwrap().read().await.unwrap().count == 2);
    let x = a.open(&h).unwrap();
    let y = a.open(&h).unwrap();
    let (xr, yr) = tokio::join!(x.rotate(), y.rotate());
    assert!(xr.is_ok() != yr.is_ok());
    assert!(a.open(&h).unwrap().read().await.is_err());
    a.shutdown().await.unwrap();
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn scoped_ssr_middleware_and_reordered_fixed_cookie() {
    let a = manager().await;
    let router = app(a.clone());
    let old = request(router.clone(), "/create-update", "POST", &HeaderMap::new()).await;
    assert!(old.status() == StatusCode::CREATED);
    let oldh = headers(&old);
    assert!(a.open(&oldh).unwrap().read().await.unwrap().count == 2);
    let value = old
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(value.contains("Path=/; HttpOnly; SameSite=Lax") && !value.contains("Domain="));
    let fresh = request(router.clone(), "/rotate", "POST", &oldh).await;
    assert!(fresh.status() == StatusCode::NO_CONTENT);
    let newh = headers(&fresh);
    let jar = Jar::default();
    let url = "http://127.0.0.1/".parse().unwrap();
    jar.add_cookie_str(
        fresh
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap(),
        &url,
    );
    jar.add_cookie_str(value, &url); // already prepared old response delivered last
    let mut reordered = HeaderMap::new();
    reordered.insert(header::COOKIE, jar.cookies(&url).unwrap());
    let denied = request(router.clone(), "/read", "GET", &reordered).await;
    assert!(
        denied.status() == StatusCode::UNAUTHORIZED
            && !denied.headers().contains_key(header::SET_COOKIE)
    );
    let live = request(router.clone(), "/read", "GET", &newh).await;
    assert!(live.status() == StatusCode::OK && !live.headers().contains_key(header::SET_COOKIE));
    assert!(live.headers().get(header::CACHE_CONTROL).unwrap() == "no-store");
    assert!(live.headers().get(header::VARY).unwrap() == "Cookie");
    assert!(to_bytes(live.into_body(), 32).await.unwrap().as_ref() == b"2");
    let logout = request(router.clone(), "/destroy", "POST", &newh).await;
    assert!(
        logout.status() == StatusCode::NO_CONTENT
            && !logout.headers().contains_key(header::SET_COOKIE)
    );
    assert!(
        request(router.clone(), "/read", "GET", &newh)
            .await
            .status()
            == StatusCode::UNAUTHORIZED
    );
    let rejected = request(router.clone(), "/rejected", "POST", &HeaderMap::new()).await;
    assert!(!rejected.headers().contains_key(header::SET_COOKIE));
    let mut duplicate = newh.clone();
    duplicate.append(header::COOKIE, newh[header::COOKIE].clone());
    assert!(
        request(router.clone(), "/read", "GET", &duplicate)
            .await
            .status()
            == StatusCode::BAD_REQUEST
    );
    let mut legacy = HeaderMap::new();
    legacy.insert(header::COOKIE, "b2_old=unused".parse().unwrap());
    assert!(
        request(router.clone(), "/read", "GET", &legacy)
            .await
            .status()
            == StatusCode::UNAUTHORIZED
    );
    a.shutdown().await.unwrap();
    assert!(
        request(router.clone(), "/read", "GET", &oldh)
            .await
            .status()
            == StatusCode::SERVICE_UNAVAILABLE
    );
    let health = request(router, "/health", "GET", &HeaderMap::new()).await;
    assert!(
        health.status() == StatusCode::OK
            && !health.headers().contains_key(header::SET_COOKIE)
            && !health.headers().contains_key(header::CACHE_CONTROL)
    );
}

async fn connection() -> redis::aio::MultiplexedConnection {
    redis::Client::open(endpoint())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires isolated named-user Valkey with default disabled and TIME initially denied"]
async fn yt184_named_acl_time_grant_unblocks_session_lifecycle() {
    let app_url = std::env::var("YT184_LOCAL_APP_URL").expect("select named app fixture");
    let control_url =
        std::env::var("YT184_LOCAL_CONTROL_URL").expect("select isolated control fixture");
    let namespace = std::env::var("YT184_LOCAL_NAMESPACE").expect("select isolated namespace");
    let client = redis::Client::open(app_url.clone())
        .unwrap_or_else(|_| panic!("valid private app endpoint"));
    let mut app_connection = client
        .get_multiplexed_async_connection()
        .await
        .unwrap_or_else(|_| panic!("named app authentication"));
    let mut control = redis::Client::open(control_url)
        .unwrap_or_else(|_| panic!("valid private control endpoint"))
        .get_multiplexed_async_connection()
        .await
        .unwrap_or_else(|_| panic!("named control authentication"));
    let anonymous_info = client
        .get_connection_info()
        .clone()
        .set_redis_settings(redis::RedisConnectionInfo::default());
    let mut anonymous = redis::Client::open(anonymous_info)
        .unwrap_or_else(|_| panic!("valid anonymous endpoint"))
        .get_multiplexed_async_connection()
        .await;
    if let Ok(connection) = &mut anonymous {
        let ping: redis::RedisResult<String> = redis::cmd("PING").query_async(connection).await;
        assert!(ping.is_err(), "default user must not provide a fallback");
    }
    let clients: String = redis::cmd("CLIENT")
        .arg("LIST")
        .query_async(&mut control)
        .await
        .expect("control client metadata");
    assert!(clients.lines().any(|line| {
        line.split_ascii_whitespace()
            .any(|field| field == "user=yt184_app")
    }));
    let denied: redis::RedisResult<Vec<String>> =
        redis::cmd("TIME").query_async(&mut app_connection).await;
    assert!(denied.is_err_and(|error| error.code() == Some("NOPERM")));

    let c = RedisSessionConfig::new(namespace, "yt184_acl", Duration::from_secs(900), 4, 256)
        .expect("bounded reference config");
    let a = Manager::<Cart>::connect(&app_url, c.clone())
        .await
        .expect("app manager authentication");
    let denied = request(app(a.clone()), "/create", "POST", &HeaderMap::new()).await;
    assert_eq!(denied.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(!denied.headers().contains_key(header::SET_COOKIE));

    let granted: String = redis::cmd("ACL")
        .arg("SETUSER")
        .arg("yt184_app")
        .arg("+time")
        .query_async(&mut control)
        .await
        .expect("trusted operator grants only TIME");
    assert_eq!(granted, "OK");
    let clock: Vec<String> = redis::cmd("TIME")
        .query_async(&mut app_connection)
        .await
        .expect("app TIME after grant");
    assert!(clock.len() == 2 && clock.iter().all(|cell| cell.parse::<u64>().is_ok()));
    let outside: redis::RedisResult<Option<String>> = redis::cmd("GET")
        .arg("outside-yt184-namespace")
        .query_async(&mut app_connection)
        .await;
    assert!(outside.is_err_and(|error| error.code() == Some("NOPERM")));
    let admin: redis::RedisResult<Vec<String>> = redis::cmd("CONFIG")
        .arg("GET")
        .arg("maxmemory")
        .query_async(&mut app_connection)
        .await;
    assert!(admin.is_err_and(|error| error.code() == Some("NOPERM")));

    let response = request(app(a.clone()), "/create", "POST", &HeaderMap::new()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("__Host-yt184_acl="));
    assert!(cookie.contains("; Secure") && cookie.contains("; HttpOnly"));
    assert!(cookie.contains("; SameSite=Lax") && !cookie.contains("Domain="));
    let original = headers(&response);
    let b = Manager::<Cart>::connect(&app_url, c)
        .await
        .expect("second app manager authentication");
    let session = b.open(&original).unwrap();
    assert_eq!(session.read().await.unwrap().count, 1);
    let csrf = session.csrf_token().await.unwrap();
    assert!(session.verify_csrf(&csrf).await.unwrap());
    assert!(!session.verify_csrf("incorrect").await.unwrap());
    let rotated = request(app(a.clone()), "/rotate", "POST", &original).await;
    assert_eq!(rotated.status(), StatusCode::NO_CONTENT);
    let current = headers(&rotated);
    assert!(current[header::COOKIE] != original[header::COOKIE]);
    assert_eq!(
        request(app(b.clone()), "/read", "GET", &original)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(app(b.clone()), "/read", "GET", &current)
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(app(b.clone()), "/destroy", "POST", &current)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(app(a.clone()), "/read", "GET", &current)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn expiry_corruption_payload_and_unknown_outcome_fail_closed() {
    let c = config(120, 1);
    let short = Manager::<Cart>::connect(&endpoint(), c).await.unwrap();
    let h = issued(&short).await;
    tokio::time::sleep(Duration::from_millis(160)).await;
    assert!(short.open(&h).unwrap().read().await.is_err());
    short.shutdown().await.unwrap();
    let c = config(60000, 1);
    let a = Manager::<Cart>::connect(&endpoint(), c).await.unwrap();
    let h = issued(&a).await;
    let s = a.open(&h).unwrap();
    assert!(matches!(
        s.try_update(|v| {
            v.note = "x".repeat(1025);
            Ok::<_, ()>(())
        })
        .await,
        Err(UpdateError::Session(Error::PayloadTooLarge))
    ));
    let mut observer = connection().await;
    let _: () = redis::cmd("CLIENT")
        .arg("PAUSE")
        .arg(700)
        .arg("WRITE")
        .query_async(&mut observer)
        .await
        .unwrap();
    let calls = AtomicUsize::new(0);
    let result = s
        .try_update(|v| {
            calls.fetch_add(1, Ordering::SeqCst);
            v.count += 1;
            Ok::<_, ()>(Box::new(7))
        })
        .await;
    let ticket = match result {
        Err(UpdateError::Session(Error::OutcomeUnknown(t))) => t,
        _ => panic!("expected unknown mutation outcome"),
    };
    assert!(calls.load(Ordering::SeqCst) == 1);
    // The timed-out write retains its slot until its reply, rather than admitting unbounded work.
    assert!(matches!(
        a.open(&h).unwrap().read().await,
        Err(Error::QueueTimeout)
    ));
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert!(matches!(
        a.resolve_operation(&ticket).await.unwrap(),
        RedisOperationStatus::Committed { .. }
    ));
    assert!(s.read().await.unwrap().count == 2 && calls.load(Ordering::SeqCst) == 1);
    let rendered = format!("{ticket:?}");
    assert!(rendered == "RedisOperationId([redacted])");
    a.shutdown().await.unwrap();
    let namespace = format!("b2_corrupt_{}", std::process::id());
    let c = RedisSessionConfig::new(&namespace, "b2", Duration::from_secs(60), 1, 1024)
        .unwrap()
        .insecure_for_local_http();
    let corrupt = Manager::<Cart>::connect(&endpoint(), c).await.unwrap();
    let h = issued(&corrupt).await;
    let id = h[header::COOKIE]
        .to_str()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let key = format!("yetlet:redis:v1:{namespace}:session:{id}");
    let _: usize = redis::cmd("HSET")
        .arg(&key)
        .arg("state")
        .arg("invalid-json")
        .query_async(&mut observer)
        .await
        .unwrap();
    assert!(matches!(
        corrupt.open(&h).unwrap().read().await,
        Err(Error::Codec)
    ));
    assert!(corrupt.open(&h).unwrap().rotate().await.is_err());
    let _: usize = redis::cmd("HSET")
        .arg(&key)
        .arg("schema")
        .arg("unsupported")
        .query_async(&mut observer)
        .await
        .unwrap();
    assert!(matches!(
        corrupt.open(&h).unwrap().read().await,
        Err(Error::InvalidRecord)
    ));
    corrupt.shutdown().await.unwrap();
    drop(observer);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires an owned persistent named-user Valkey container"]
async fn yt184_service_restart_records_refusal_and_read_only_recovery() {
    let endpoint = std::env::var("YT184_LOCAL_APP_URL").expect("select private app endpoint");
    let namespace = std::env::var("YT184_LOCAL_NAMESPACE").expect("select private namespace");
    let container = std::env::var("YT184_RECOVERY_CONTAINER").expect("select owned fixture");
    assert!(container.starts_with("yt184-recovery-"));
    let c = RedisSessionConfig::new(
        namespace,
        "yt184_recovery",
        Duration::from_secs(900),
        4,
        256,
    )
    .unwrap();
    let a = Manager::<Cart>::connect(&endpoint, c.clone())
        .await
        .unwrap();
    let b = Manager::<Cart>::connect(&endpoint, c).await.unwrap();
    let credential = issued(&a).await;
    let stopped = std::process::Command::new("docker")
        .args(["stop", "--time", "5", &container])
        .output()
        .expect("stop owned fixture");
    assert!(stopped.status.success());
    for manager in [&a, &b] {
        let response = request(app(manager.clone()), "/read", "GET", &credential).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }
    tokio::time::sleep(Duration::from_millis(250)).await;
    let started = std::process::Command::new("docker")
        .args(["start", &container])
        .output()
        .expect("restart same owned fixture");
    assert!(started.status.success());
    tokio::time::sleep(Duration::from_secs(2)).await;
    let begin = tokio::time::Instant::now();
    let end = begin + Duration::from_secs(5);
    let mut ready = [false; 2];
    let mut observations = Vec::new();
    while !ready.iter().all(|value| *value) {
        for (index, manager) in [&a, &b].into_iter().enumerate() {
            if ready[index] {
                continue;
            }
            let response = request(app(manager.clone()), "/read", "GET", &credential).await;
            let status = response.status();
            observations.push(serde_json::json!({
                "manager": index,
                "elapsed_us_after_two_second_settle": begin.elapsed().as_micros(),
                "status": status.as_u16(),
            }));
            assert!(!response.headers().contains_key(header::SET_COOKIE));
            assert!(matches!(
                status,
                StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE
            ));
            if status == StatusCode::OK {
                let body = to_bytes(response.into_body(), 1024).await.unwrap();
                assert!(body.as_ref() == b"1");
                ready[index] = true;
            }
        }
        assert!(
            tokio::time::Instant::now() < end,
            "bounded read-only recovery deadline"
        );
        if !ready.iter().all(|value| *value) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    println!(
        "{}",
        serde_json::json!({"yt184_read_only_recovery": observations})
    );
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}
