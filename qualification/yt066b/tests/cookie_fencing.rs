//! Selected pure-SSR contract: response reordering may lose a login, never
//! authenticate a retired bearer or select a different cookie as a fallback.
use axum::{
    Router,
    extract::{Path, State as AppState},
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
use yt066b_redis_spike::{
    Error, State, Store, Ticket,
    cookies::{CookieError, CookieScope},
    random_id,
};

fn scope() -> CookieScope {
    CookieScope::new("yt_ssr")
        .unwrap()
        .insecure_for_local_http()
}
async fn store() -> Store {
    Store::connect(
        &std::env::var("YT_REDIS_URL").unwrap(),
        &random_id().unwrap(),
        2,
    )
    .await
    .unwrap()
}
async fn create(s: &Store, count: u64, ttl: u64) -> Ticket {
    let t = s
        .prepare("create", None, 0, &State::new(count), ttl)
        .await
        .unwrap();
    assert!(s.execute(&t).await.unwrap().status == "ok");
    t
}
async fn change(s: &Store, kind: &str, id: &str) -> Ticket {
    let snap = s.read(id).await.unwrap();
    let t = s
        .prepare(kind, Some(id), snap.revision, &State::new(2), 60000)
        .await
        .unwrap();
    assert!(s.execute(&t).await.unwrap().status == "ok");
    t
}
fn response(status: StatusCode, marker: &str, directive: Option<String>) -> Response {
    let mut r = (
        status,
        format!("<html><body><p id=\"marker\">{marker}</p></body></html>"),
    )
        .into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/html; charset=utf-8".parse().unwrap(),
    );
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    r.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'".parse().unwrap(),
    );
    if let Some(value) = directive {
        r.headers_mut()
            .insert(header::SET_COOKIE, value.parse().unwrap());
    }
    r
}
fn presented(headers: &HeaderMap) -> Result<yt066b_redis_spike::cookies::Presented, CookieError> {
    scope().parse(
        headers
            .get_all(header::COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap_or("yt_ssr=")),
    )
}
async fn authorization(s: &Store, headers: HeaderMap) -> Response {
    let accepted = match presented(&headers) {
        Ok(p) => scope().load(s, &p).await.is_ok(),
        Err(_) => false,
    };
    response(
        if accepted {
            StatusCode::OK
        } else {
            StatusCode::UNAUTHORIZED
        },
        if accepted {
            "AUTHORIZED"
        } else {
            "UNAUTHORIZED"
        },
        None,
    )
}
async fn serve(
    router: Router,
) -> (
    reqwest::Url,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
) {
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", socket.local_addr().unwrap())
        .parse()
        .unwrap();
    let (stop, stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
        axum::serve(socket, router)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
            .unwrap();
    });
    (url, stop, task)
}
async fn stop(stop: oneshot::Sender<()>, task: tokio::task::JoinHandle<()>, s: &Store) {
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
    s.shutdown().await.unwrap();
}

#[test]
fn cookie_policy_has_one_exact_name_and_rejects_duplicates() {
    let secure = CookieScope::new("ssr").unwrap();
    assert_eq!(secure.name(), "__Host-ssr");
    assert_eq!(scope().name(), "yt_ssr");
    assert!(CookieScope::new("bad;name").is_err());
    let id = "a".repeat(64);
    assert!(
        scope()
            .parse([format!("yt_ssr={id}; yt_ssr={id}").as_str()])
            .is_err()
    );
    assert!(
        scope()
            .parse([
                format!("yt_ssr={id}").as_str(),
                format!("yt_ssr={id}").as_str()
            ])
            .is_err()
    );
    for invalid in ["yt_ssr", "yt_ssr=", "yt_ssr=!", "yt_ssr=short"] {
        assert!(scope().parse([invalid]).is_err());
    }
    assert!(
        scope()
            .parse([format!("yt_ssr_old={id}; unrelated=x").as_str()])
            .is_ok()
    );
}

#[derive(Clone, Copy)]
enum Pause {
    BeforeCheck,
    AfterCheck,
    AfterHeader,
}
struct Race {
    store: Store,
    operation: Ticket,
    fresh: Mutex<Option<Ticket>>,
    phase: Pause,
    ready: Notify,
    release: Notify,
}
async fn delayed(AppState(f): AppState<Arc<Race>>) -> Response {
    if matches!(f.phase, Pause::BeforeCheck) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    let directive = scope().finalize(&f.store, &f.operation).await.unwrap();
    if matches!(f.phase, Pause::AfterCheck) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    let r = response(StatusCode::OK, "DELAYED", directive);
    if matches!(f.phase, Pause::AfterHeader) {
        f.ready.notify_one();
        f.release.notified().await;
    }
    r
}
async fn fresh(AppState(f): AppState<Arc<Race>>) -> Response {
    let op = f.fresh.lock().unwrap().take().unwrap();
    response(
        StatusCode::OK,
        "FRESH",
        scope().finalize(&f.store, &op).await.unwrap(),
    )
}
async fn auth(AppState(f): AppState<Arc<Race>>, headers: HeaderMap) -> Response {
    authorization(&f.store, headers).await
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn delayed_responses_preserve_safety_with_explicit_login_loss() {
    for (phase, logout) in [
        (Pause::BeforeCheck, false),
        (Pause::AfterCheck, false),
        (Pause::AfterHeader, false),
        (Pause::AfterHeader, true),
    ] {
        let s = store().await;
        let first = create(&s, 1, 60000).await;
        let old = first.identity().to_owned();
        let op = if logout {
            change(&s, "revoke", &old).await
        } else {
            first
        };
        let f = Arc::new(Race {
            store: s.clone(),
            operation: op,
            fresh: Mutex::new(None),
            phase,
            ready: Notify::new(),
            release: Notify::new(),
        });
        let router = Router::new()
            .route("/delayed", get(delayed))
            .route("/fresh", get(fresh))
            .route("/auth", get(auth))
            .with_state(f.clone());
        let (url, shutdown, task) = serve(router).await;
        let jar = Arc::new(Jar::default());
        let client = reqwest::Client::builder()
            .no_proxy()
            .cookie_provider(jar.clone())
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        let delayed_client = client.clone();
        let delayed_url = url.join("delayed").unwrap();
        let pending =
            tokio::spawn(async move { delayed_client.get(delayed_url).send().await.unwrap() });
        tokio::time::timeout(Duration::from_secs(2), f.ready.notified())
            .await
            .unwrap();
        let replacement = if logout {
            create(&s, 2, 60000).await
        } else {
            change(&s, "rotate", &old).await
        };
        let new = replacement
            .replacement()
            .unwrap_or(replacement.identity())
            .to_owned();
        *f.fresh.lock().unwrap() = Some(replacement);
        client.get(url.join("fresh").unwrap()).send().await.unwrap();
        assert!(jar.cookies(&url).unwrap().to_str().unwrap() == format!("yt_ssr={new}"));
        f.release.notify_one();
        let older = pending.await.unwrap();
        let keep_new = logout || matches!(phase, Pause::BeforeCheck);
        assert_eq!(older.headers().get(header::SET_COOKIE).is_none(), keep_new);
        let selected = jar.cookies(&url).unwrap();
        assert!(
            selected.to_str().unwrap() == format!("yt_ssr={}", if keep_new { &new } else { &old })
        );
        let authorized = client.get(url.join("auth").unwrap()).send().await.unwrap();
        assert_eq!(
            authorized.status(),
            if keep_new {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
        assert!(authorized.headers().get(header::SET_COOKIE).is_none());
        assert!(s.read(&old).await.is_err());
        assert!(s.read(&new).await.is_ok());
        // Retiring the replacement must not reveal any independent live cookie.
        let other = create(&s, 99, 60000).await;
        jar.add_cookie_str(&format!("yt_ssr_legacy={}; Path=/", other.identity()), &url);
        change(&s, "revoke", &new).await;
        assert_eq!(
            client
                .get(url.join("auth").unwrap())
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        stop(shutdown, task, &s).await;
    }
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn cookie_emission_requires_proven_success_and_current_identity() {
    let s = store().await;
    let t = create(&s, 1, 60000).await;
    let secure = CookieScope::new("ssr")
        .unwrap()
        .finalize(&s, &t)
        .await
        .unwrap()
        .unwrap();
    assert!(secure.starts_with("__Host-ssr="));
    assert!(secure.contains("; HttpOnly; SameSite=Lax; Secure"));
    assert!(secure.contains("; Path=/") && !secure.contains("Domain="));
    let update = change(&s, "update", t.identity()).await;
    assert!(scope().finalize(&s, &update).await.unwrap().is_none());
    let snap = s.read(t.identity()).await.unwrap();
    let a = s
        .prepare(
            "rotate",
            Some(t.identity()),
            snap.revision,
            &State::new(0),
            60000,
        )
        .await
        .unwrap();
    let b = s
        .prepare(
            "rotate",
            Some(t.identity()),
            snap.revision,
            &State::new(0),
            60000,
        )
        .await
        .unwrap();
    let (ar, br) = tokio::join!(s.execute(&a), s.execute(&b));
    let (winner, loser) = if ar.unwrap().status == "ok" {
        assert!(br.unwrap().status != "ok");
        (a, b)
    } else {
        assert!(br.unwrap().status == "ok");
        (b, a)
    };
    assert!(scope().finalize(&s, &winner).await.unwrap().is_some());
    assert!(scope().finalize(&s, &loser).await.unwrap().is_none());
    assert!(scope().finalize(&s, &t).await.unwrap().is_none());
    let replacement = change(&s, "replace", winner.replacement().unwrap()).await;
    assert!(scope().finalize(&s, &replacement).await.unwrap().is_some());
    let revoked = change(&s, "revoke", replacement.replacement().unwrap()).await;
    assert!(scope().finalize(&s, &revoked).await.unwrap().is_none());
    let pending = s
        .prepare("create", None, 0, &State::new(0), 60000)
        .await
        .unwrap();
    assert!(matches!(
        scope().finalize(&s, &pending).await,
        Err(Error::OutcomeUnknown(_))
    ));
    let expired = create(&s, 0, 1).await;
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert!(scope().finalize(&s, &expired).await.unwrap().is_none());
    let corrupt = create(&s, 0, 60000).await;
    let mut cmd = redis::cmd("HSET");
    cmd.arg(s.record_key(corrupt.identity()).unwrap())
        .arg("state")
        .arg("invalid-json");
    s.command(cmd).await.unwrap();
    assert!(scope().finalize(&s, &corrupt).await.is_err());
    s.shutdown().await.unwrap();
    assert!(scope().finalize(&s, &winner).await.is_err());
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn loading_never_creates_or_falls_back() {
    let s = store().await;
    let active = create(&s, 1, 60000).await;
    let unknown = random_id().unwrap();
    for headers in [
        String::new(),
        format!("yt_ssr={unknown}"),
        format!("yt_ssr_legacy={}", active.identity()),
    ] {
        assert!(
            scope()
                .load(&s, &scope().parse([headers.as_str()]).unwrap())
                .await
                .is_err()
        );
    }
    let valid = format!("yt_ssr={}", active.identity());
    let p = scope().parse([valid.as_str()]).unwrap();
    assert!(scope().load(&s, &p).await.unwrap().snapshot.state.count == 1);
    change(&s, "revoke", active.identity()).await;
    assert!(scope().load(&s, &p).await.is_err());
    tokio::time::sleep(Duration::from_millis(450)).await;
    assert!(scope().load(&s, &p).await.is_err());
    let mut cmd = redis::cmd("EXISTS");
    cmd.arg(s.record_key(&unknown).unwrap());
    let exists: u8 = redis::from_redis_value(s.command(cmd).await.unwrap()).unwrap();
    assert_eq!(exists, 0);
    let latest = create(&s, 2, 60000).await;
    let newest = scope()
        .parse([format!("yt_ssr={}", latest.identity()).as_str()])
        .unwrap();
    s.shutdown().await.unwrap();
    assert!(scope().load(&s, &newest).await.is_err());
}

struct BrowserFixture {
    store: Store,
    current: String,
    old_header: String,
    fresh_header: Mutex<Option<String>>,
}
async fn browser_fresh(AppState(f): AppState<Arc<BrowserFixture>>) -> Response {
    response(
        StatusCode::OK,
        "FRESH",
        f.fresh_header.lock().unwrap().take(),
    )
}
async fn browser_old(
    AppState(f): AppState<Arc<BrowserFixture>>,
    Path(step): Path<usize>,
) -> Response {
    response(
        StatusCode::OK,
        &format!("OLD-{step}"),
        Some(f.old_header.clone()),
    )
}
async fn browser_auth(AppState(f): AppState<Arc<BrowserFixture>>, headers: HeaderMap) -> Response {
    authorization(&f.store, headers).await
}
async fn browser_login(AppState(f): AppState<Arc<BrowserFixture>>) -> Response {
    // Explicit synthetic login fixture, not anonymous load or a public auth API.
    let t = create(&f.store, 3, 60000).await;
    response(
        StatusCode::OK,
        "LOGIN",
        scope().finalize(&f.store, &t).await.unwrap(),
    )
}
async fn browser_logout(
    AppState(f): AppState<Arc<BrowserFixture>>,
    headers: HeaderMap,
) -> Response {
    let p = presented(&headers).unwrap();
    let loaded = scope().load(&f.store, &p).await.unwrap();
    let op = change(&f.store, "revoke", &loaded.identity).await;
    response(
        StatusCode::OK,
        "LOGOUT",
        scope().finalize(&f.store, &op).await.unwrap(),
    )
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires disposable Redis and local Chrome"]
async fn real_chrome_without_js_keeps_one_cookie_and_rejects_retired_ids() {
    let s = store().await;
    let first = create(&s, 1, 60000).await;
    let old_header = scope().finalize(&s, &first).await.unwrap().unwrap();
    let replacement = change(&s, "rotate", first.identity()).await;
    let f = Arc::new(BrowserFixture {
        store: s.clone(),
        current: replacement.replacement().unwrap().to_owned(),
        old_header,
        fresh_header: Mutex::new(scope().finalize(&s, &replacement).await.unwrap()),
    });
    let router = Router::new()
        .route("/fresh", get(browser_fresh))
        .route("/old/{step}", get(browser_old))
        .route("/auth", get(browser_auth))
        .route("/login", get(browser_login))
        .route("/logout", get(browser_logout))
        .with_state(f.clone());
    let (url, shutdown, task) = serve(router).await;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cookie_browser.py");
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("python3")
            .arg(root)
            .arg(url.as_str())
            .output()
    })
    .await
    .unwrap()
    .unwrap();
    let replacement_still_live = s.read(&f.current).await.is_ok();
    stop(shutdown, task, &s).await;
    assert!(
        output.status.success(),
        "focused Chrome fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(replacement_still_live);
    println!("{}", String::from_utf8(output.stdout).unwrap());
}
