#![cfg(feature = "session")]
#![expect(
    clippy::result_large_err,
    reason = "this HTTP consumer deliberately returns native Axum Response errors, as required by the contract"
)]
//! Real loopback HTTP exercises the bridge with a synthetic, server-selected
//! verified identity. This fixture is not a deployable login provider.
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, Extension, State},
    http::{StatusCode, request::Parts},
    middleware::from_fn_with_state,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{Notify, oneshot},
    task::JoinHandle,
};
use tower::{ServiceBuilder, ServiceExt};
use yetlet_auth::{Authenticated, Authentication, Principal, ResolveFuture, authenticate, session};
use yetlet_core::{Element, View};
use yetlet_server_axum::Page;
use yetlet_session::{
    Session, SessionClock, SessionClockError, SessionConfig, SessionError, SessionManager,
};

#[derive(Default, Clone)]
struct IdentityRef {
    account: Option<&'static str>,
}
struct Account {
    enabled: bool,
    write: bool,
    writes: u64,
}
#[derive(Clone)]
struct Accounts {
    account: Arc<Mutex<Account>>,
    entered: Arc<Notify>,
    resume: Arc<Notify>,
}
impl Accounts {
    fn new() -> Self {
        Self {
            account: Arc::new(Mutex::new(Account {
                enabled: true,
                write: true,
                writes: 0,
            })),
            entered: Arc::new(Notify::new()),
            resume: Arc::new(Notify::new()),
        }
    }
}
struct Clock {
    now: AtomicU64,
    unavailable: AtomicBool,
}
impl SessionClock for Clock {
    fn now_seconds(&self) -> Result<u64, SessionClockError> {
        if self.unavailable.load(Ordering::SeqCst) {
            Err(SessionClockError)
        } else {
            Ok(self.now.load(Ordering::SeqCst))
        }
    }
}
fn login_required() -> Response {
    Redirect::to("/auth/form").into_response()
}
fn unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "Account service unavailable",
    )
        .into_response()
}
fn resolve<'a>(parts: &'a mut Parts, accounts: &'a Accounts) -> ResolveFuture<'a> {
    Box::pin(async move {
        session::memory::<IdentityRef>(
            parts,
            |identity| {
                let Some(subject) = identity.account else {
                    return Ok(None);
                };
                let account = accounts.account.lock().map_err(|_| unavailable())?;
                if subject != "account-one" || !account.enabled {
                    return Err(StatusCode::FORBIDDEN.into_response());
                }
                Ok(Some(
                    Principal::from_verified(
                        "urn:yetlet:http-fixture",
                        subject,
                        "memory",
                        Some("one"),
                    )
                    .unwrap(),
                ))
            },
            login_required,
        )
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Csrf {
    csrf: String,
}
fn session_failure(error: SessionError) -> Response {
    match error {
        SessionError::Unavailable => unavailable(),
        SessionError::Expired | SessionError::Destroyed => login_required(),
    }
}
fn verify(session: &Session<IdentityRef>, csrf: &str) -> Result<(), Response> {
    match session.verify_csrf(csrf) {
        Ok(true) => Ok(()),
        Ok(false) => Err(StatusCode::FORBIDDEN.into_response()),
        Err(error) => Err(session_failure(error)),
    }
}
async fn form(Extension(session): Extension<Session<IdentityRef>>) -> Result<Page, Response> {
    let csrf = session.csrf_token().map_err(|_| unavailable())?;
    Ok(Page::new(
        "Session authentication",
        Element::new("form")
            .attribute("method", "post")
            .attribute("action", "/auth/login")
            .child(
                Element::new("input")
                    .attribute("type", "hidden")
                    .attribute("name", "csrf")
                    .attribute("value", csrf),
            )
            .child(
                Element::new("button")
                    .attribute("type", "submit")
                    .child(View::text("Sign in")),
            ),
    ))
}
async fn login(
    Extension(session): Extension<Session<IdentityRef>>,
    State(accounts): State<Accounts>,
    Form(form): Form<Csrf>,
) -> Result<Response, Response> {
    verify(&session, &form.csrf)?;
    let account = accounts.account.lock().map_err(|_| unavailable())?;
    if !account.enabled {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    // The test fixture already owns this verified upstream decision; no user
    // field, header, email or role can choose the identity.
    session
        .replace_and_rotate(IdentityRef {
            account: Some("account-one"),
        })
        .map_err(|_| unavailable())?;
    Ok(Redirect::to("/auth/private").into_response())
}
async fn private(_: Authenticated) -> &'static str {
    "private account"
}
async fn optional(identity: Option<Authenticated>) -> &'static str {
    if identity.is_some() {
        "signed in"
    } else {
        "anonymous"
    }
}
fn commit(
    accounts: &Accounts,
    session: &Session<IdentityRef>,
    identity: &Authenticated,
    csrf: &str,
) -> Result<Response, Response> {
    // Local logout, permission edits and writes serialize through this same
    // application lock. Re-read Session at the business commit boundary.
    let mut account = accounts.account.lock().map_err(|_| unavailable())?;
    verify(session, csrf)?;
    let reference = session.read().map_err(session_failure)?;
    if reference.account != Some(identity.principal().subject())
        || !account.enabled
        || !account.write
    {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    account.writes += 1;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn write(
    identity: Authenticated,
    Extension(session): Extension<Session<IdentityRef>>,
    State(accounts): State<Accounts>,
    Form(form): Form<Csrf>,
) -> Result<Response, Response> {
    commit(&accounts, &session, &identity, &form.csrf)
}
async fn slow(
    identity: Authenticated,
    Extension(session): Extension<Session<IdentityRef>>,
    State(accounts): State<Accounts>,
    Form(form): Form<Csrf>,
) -> Result<Response, Response> {
    accounts.entered.notify_one();
    accounts.resume.notified().await;
    commit(&accounts, &session, &identity, &form.csrf)
}
async fn logout(
    _: Authenticated,
    Extension(session): Extension<Session<IdentityRef>>,
    State(accounts): State<Accounts>,
    Form(form): Form<Csrf>,
) -> Result<Response, Response> {
    let _account = accounts.account.lock().map_err(|_| unavailable())?;
    verify(&session, &form.csrf)?;
    session.destroy().map_err(session_failure)?;
    Ok(login_required())
}
fn routes(accounts: Accounts, clock: Arc<Clock>, secure: bool) -> Router {
    let config = SessionConfig::new("yetlet_auth_test", Duration::from_secs(60), 32).unwrap();
    let config = if secure {
        config
    } else {
        config.insecure_for_local_http()
    };
    let manager = SessionManager::<IdentityRef>::with_clock(config, clock);
    let auth = Authentication::new("memory", accounts.clone(), resolve, login_required).unwrap();
    let protected = Router::new()
        .route("/form", get(form))
        .route("/login", post(login))
        .route("/private", get(private))
        .route("/optional", get(optional))
        .route("/write", post(write))
        .route("/slow", post(slow))
        .route("/logout", post(logout))
        .layer(DefaultBodyLimit::max(4096))
        .route_layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(
                    manager,
                    yetlet_session::session_middleware::<IdentityRef>,
                ))
                .layer(from_fn_with_state(auth, authenticate::<Accounts>)),
        );
    Router::new()
        .route("/health", get(|| async { "ready" }))
        .nest("/auth", protected)
        .with_state(accounts)
}
struct Server {
    base: String,
    client: reqwest::Client,
    accounts: Accounts,
    clock: Arc<Clock>,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}
impl Server {
    async fn start(secure: bool) -> Self {
        let accounts = Accounts::new();
        let clock = Arc::new(Clock {
            now: AtomicU64::new(1000),
            unavailable: AtomicBool::new(false),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let app = routes(accounts.clone(), clock.clone(), secure);
        let (stop, shutdown) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown.await;
                })
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self {
            base,
            client,
            accounts,
            clock,
            stop: Some(stop),
            task: Some(task),
        }
    }
    async fn get(&self, path: &str, cookie: Option<&str>) -> reqwest::Response {
        let mut req = self.client.get(format!("{}{path}", self.base));
        if let Some(cookie) = cookie {
            req = req.header("Cookie", cookie);
        }
        req.send().await.unwrap()
    }
    async fn post(&self, path: &str, cookie: &str, csrf: &str) -> reqwest::Response {
        self.client
            .post(format!("{}{path}", self.base))
            .header("Cookie", cookie)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("csrf={csrf}"))
            .send()
            .await
            .unwrap()
    }
    async fn initial(&self) -> (String, String) {
        let response = self.get("/auth/form", None).await;
        privacy(&response);
        let cookie = cookie(&response);
        let html = response.text().await.unwrap();
        assert!(!html.contains("<script"));
        assert!(!html.contains("Principal"));
        let csrf = html
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned();
        (cookie, csrf)
    }
    async fn signed_in(&self) -> (String, String) {
        let (anonymous, csrf) = self.initial().await;
        let response = self.post("/auth/login", &anonymous, &csrf).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        privacy(&response);
        let signed = cookie(&response);
        assert!(signed != anonymous);
        let response = self.get("/auth/form", Some(&signed)).await;
        let html = response.text().await.unwrap();
        let current = html
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned();
        assert!(current != csrf);
        (signed, current)
    }
    async fn finish(mut self) {
        let _ = self.stop.take().unwrap().send(());
        tokio::time::timeout(Duration::from_secs(3), self.task.take().unwrap())
            .await
            .unwrap()
            .unwrap();
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
fn cookie(response: &reqwest::Response) -> String {
    response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}
fn privacy(response: &reqwest::Response) {
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(
        response.headers()["vary"]
            .to_str()
            .unwrap()
            .contains("Cookie")
    );
}

#[tokio::test]
async fn native_forms_rotate_identity_csrf_and_revoke_logout_over_http() {
    let server = Server::start(false).await;
    let health = server.get("/health", None).await;
    assert!(!health.headers().contains_key("set-cookie"));
    assert!(!health.headers().contains_key("cache-control"));
    let (anonymous, csrf) = server.initial().await;
    assert_eq!(
        server
            .post("/auth/login", &anonymous, "wrong")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let login = server.post("/auth/login", &anonymous, &csrf).await;
    let signed = cookie(&login);
    assert!(signed != anonymous);
    privacy(&login);
    assert_eq!(
        server.get("/auth/private", Some(&anonymous)).await.status(),
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        server.get("/auth/private", Some(&signed)).await.status(),
        StatusCode::OK
    );
    let response = server.get("/auth/form", Some(&signed)).await;
    let html = response.text().await.unwrap();
    let fresh = html
        .split("name=\"csrf\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(fresh != csrf);
    assert_eq!(
        server.post("/auth/write", &signed, &csrf).await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        server.post("/auth/write", &signed, fresh).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        server.post("/auth/logout", &signed, "wrong").await.status(),
        StatusCode::FORBIDDEN
    );
    let logout = server.post("/auth/logout", &signed, fresh).await;
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    privacy(&logout);
    assert!(
        logout.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    assert_eq!(
        server.get("/auth/private", Some(&signed)).await.status(),
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        server
            .get("/auth/optional", Some(&signed))
            .await
            .text()
            .await
            .unwrap(),
        "anonymous"
    );
    assert_eq!(server.accounts.account.lock().unwrap().writes, 1);
    server.finish().await;
}

#[tokio::test]
async fn expiry_forged_duplicate_cookie_and_outage_fail_closed() {
    let server = Server::start(false).await;
    let (signed, _) = server.signed_in().await;
    server.clock.now.fetch_add(61, Ordering::SeqCst);
    let expired = server.get("/auth/private", Some(&signed)).await;
    assert_eq!(expired.status(), StatusCode::SEE_OTHER);
    privacy(&expired);
    assert!(cookie(&expired) != signed);
    let forged = format!("yetlet_auth_test={}", "a".repeat(64));
    assert_eq!(
        server.get("/auth/private", Some(&forged)).await.status(),
        StatusCode::SEE_OTHER
    );
    for invalid in [
        "yetlet_auth_test=bad".to_owned(),
        format!("{signed}; {signed}"),
    ] {
        let response = server.get("/auth/optional", Some(&invalid)).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        privacy(&response);
    }
    server.clock.unavailable.store(true, Ordering::SeqCst);
    let outage = server.get("/auth/optional", Some(&signed)).await;
    assert_eq!(outage.status(), StatusCode::SERVICE_UNAVAILABLE);
    privacy(&outage);
    assert!(!outage.headers().contains_key("location"));
    server.finish().await;
}

#[tokio::test]
async fn permissions_and_disabled_accounts_are_checked_on_each_request() {
    let server = Server::start(false).await;
    let (signed, csrf) = server.signed_in().await;
    server.accounts.account.lock().unwrap().write = false;
    assert_eq!(
        server.get("/auth/private", Some(&signed)).await.status(),
        StatusCode::OK
    );
    assert_eq!(
        server.post("/auth/write", &signed, &csrf).await.status(),
        StatusCode::FORBIDDEN
    );
    server.accounts.account.lock().unwrap().enabled = false;
    assert_eq!(
        server.get("/auth/optional", Some(&signed)).await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(server.accounts.account.lock().unwrap().writes, 0);
    server.finish().await;
}

#[tokio::test]
async fn in_flight_write_rechecks_revocation_and_preserves_replacement_cookie() {
    let server = Server::start(false).await;
    let (signed, csrf) = server.signed_in().await;
    let client = server.client.clone();
    let base = server.base.clone();
    let old = signed.clone();
    let old_csrf = csrf.clone();
    let slow = tokio::spawn(async move {
        client
            .post(format!("{base}/auth/slow"))
            .header("Cookie", old)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("csrf={old_csrf}"))
            .send()
            .await
            .unwrap()
    });
    tokio::time::timeout(Duration::from_secs(3), server.accounts.entered.notified())
        .await
        .unwrap();
    let logout = server.post("/auth/logout", &signed, &csrf).await;
    assert_eq!(logout.status(), StatusCode::SEE_OTHER);
    let (replacement, _) = server.initial().await;
    server.accounts.resume.notify_one();
    let stale = slow.await.unwrap();
    assert_eq!(stale.status(), StatusCode::SEE_OTHER);
    assert!(!stale.headers().contains_key("set-cookie"));
    assert_eq!(
        server
            .get("/auth/optional", Some(&replacement))
            .await
            .text()
            .await
            .unwrap(),
        "anonymous"
    );
    assert_eq!(server.accounts.account.lock().unwrap().writes, 0);
    server.finish().await;
}

#[tokio::test]
async fn in_flight_provider_outage_does_not_redirect_or_mutate() {
    let server = Server::start(false).await;
    let (signed, csrf) = server.signed_in().await;
    let client = server.client.clone();
    let base = server.base.clone();
    let slow = tokio::spawn(async move {
        client
            .post(format!("{base}/auth/slow"))
            .header("Cookie", signed)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("csrf={csrf}"))
            .send()
            .await
            .unwrap()
    });
    tokio::time::timeout(Duration::from_secs(3), server.accounts.entered.notified())
        .await
        .unwrap();
    server.clock.unavailable.store(true, Ordering::SeqCst);
    server.accounts.resume.notify_one();
    let response = slow.await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(!response.headers().contains_key("location"));
    assert_eq!(server.accounts.account.lock().unwrap().writes, 0);
    server.finish().await;
}

#[tokio::test]
async fn secure_cookie_defaults_survive_bridge_composition() {
    let server = Server::start(true).await;
    let response = server.get("/auth/form", None).await;
    let value = response.headers()["set-cookie"].to_str().unwrap();
    for attribute in ["Secure", "HttpOnly", "SameSite=Lax", "Path=/"] {
        assert!(value.contains(attribute));
    }
    assert!(!value.contains("Domain="));
    privacy(&response);
    server.finish().await;
}

#[tokio::test]
async fn missing_or_misordered_session_layer_is_configuration_error() {
    let accounts = Accounts::new();
    let auth = Authentication::new("memory", accounts, resolve, login_required).unwrap();
    let app = Router::new()
        .route("/", get(optional))
        .route_layer(from_fn_with_state(auth, authenticate::<Accounts>));
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.headers()["cache-control"], "no-store");
}
