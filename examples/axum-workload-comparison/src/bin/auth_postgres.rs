//! Opt-in local functional fixture. Synthetic login is not credential verification.
#![allow(
    clippy::result_large_err,
    reason = "Keep native Auth/Axum failure responses"
)]
use axum::{
    Extension, Form, Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State, rejection::FormRejection},
    http::{StatusCode, header, request::Parts},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    ConnectOptions, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::{
    collections::BTreeSet,
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::time::timeout;
use yetlet::auth::{Authenticated, Authentication, Principal, ResolveFuture, authenticate};
use yetlet::view::{Element, View};
use yetlet::server_axum::{App, DEFAULT_CSP, DocumentMetadata, LanguageTag, Page};
use yetlet::session_redis::{
    RedisSession, RedisSessionConfig, RedisSessionError, RedisSessionManager, session_middleware,
};

#[cfg(feature = "auth-workload-observation")]
#[path = "auth_postgres/observation.rs"]
mod observation;

#[cfg(feature = "auth-workload-admission")]
#[path = "auth_postgres/admission.rs"]
mod admission;

#[cfg(feature = "auth-cross-host")]
#[path = "auth_postgres/cross_host.rs"]
mod cross_host;

const ACCOUNT_SQL: &str = "SELECT id, tenant, enabled, can_view FROM public.accounts WHERE id = $1";
const DOCUMENT_SQL: &str =
    "SELECT id, title, body FROM public.documents WHERE id = $1 AND tenant = $2";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityRef {
    account: Account,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Account {
    Anonymous,
    One,
}
#[derive(Clone)]
struct Policy {
    id: i32,
    tenant: String,
    enabled: bool,
    can_view: bool,
}
#[derive(Serialize)]
struct Document {
    id: i32,
    title: String,
    body: String,
}
#[derive(Default)]
struct Counters {
    account_queries: AtomicU64,
    document_queries: AtomicU64,
    database_errors: AtomicU64,
    session_errors: AtomicU64,
    denied: AtomicU64,
    #[cfg(feature = "auth-workload-observation")]
    observations: observation::Observations,
}
#[derive(Clone)]
struct AppState {
    pool: PgPool,
    counters: Arc<Counters>,
}
fn fail(status: StatusCode) -> Response {
    (status, "Request unavailable or denied").into_response()
}
fn login_required() -> Response {
    Redirect::to("/auth/form").into_response()
}
fn session_error(error: RedisSessionError) -> Response {
    match error {
        RedisSessionError::Missing
        | RedisSessionError::Expired
        | RedisSessionError::Rotated
        | RedisSessionError::Revoked => login_required(),
        RedisSessionError::InvalidCookie => fail(StatusCode::BAD_REQUEST),
        RedisSessionError::Conflict { .. } => fail(StatusCode::CONFLICT),
        RedisSessionError::Configuration => fail(StatusCode::INTERNAL_SERVER_ERROR),
        _ => fail(StatusCode::SERVICE_UNAVAILABLE),
    }
}
impl AppState {
    fn unavailable(&self) -> Response {
        self.counters
            .database_errors
            .fetch_add(1, Ordering::Relaxed);
        fail(StatusCode::SERVICE_UNAVAILABLE)
    }
    async fn account(&self) -> Result<Policy, Response> {
        self.counters
            .account_queries
            .fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "auth-workload-observation")]
        let acquire_start = std::time::Instant::now();
        let acquired = self.pool.acquire().await;
        #[cfg(feature = "auth-workload-observation")]
        {
            self.counters
                .observations
                .pool_acquire
                .record(acquire_start.elapsed());
            if let Err(error) = &acquired {
                self.counters.observations.acquire_error(error);
            }
        }
        let mut connection = acquired.map_err(|_| self.unavailable())?;
        #[cfg(feature = "auth-workload-observation")]
        let query_start = std::time::Instant::now();
        let queried = timeout(
            Duration::from_millis(750),
            sqlx::query_as::<_, (i32, String, bool, bool)>(ACCOUNT_SQL)
                .bind(1_i32)
                .fetch_optional(&mut *connection),
        )
        .await;
        #[cfg(feature = "auth-workload-observation")]
        {
            self.counters
                .observations
                .query
                .record(query_start.elapsed());
            match &queried {
                Err(_) => self.counters.observations.query_deadline(),
                Ok(Err(error)) => self.counters.observations.query_error(error),
                _ => (),
            }
        }
        let row = queried
            .map_err(|_| self.unavailable())?
            .map_err(|_| self.unavailable())?
            .ok_or_else(|| fail(StatusCode::FORBIDDEN))?;
        if row.0 != 1 || row.1 != "alpha" {
            return Err(fail(StatusCode::FORBIDDEN));
        }
        Ok(Policy {
            id: row.0,
            tenant: row.1,
            enabled: row.2,
            can_view: row.3,
        })
    }
    async fn document(&self, id: i32, tenant: &str) -> Result<Document, Response> {
        self.counters
            .document_queries
            .fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "auth-workload-observation")]
        let acquire_start = std::time::Instant::now();
        let acquired = self.pool.acquire().await;
        #[cfg(feature = "auth-workload-observation")]
        {
            self.counters
                .observations
                .pool_acquire
                .record(acquire_start.elapsed());
            if let Err(error) = &acquired {
                self.counters.observations.acquire_error(error);
            }
        }
        let mut connection = acquired.map_err(|_| self.unavailable())?;
        #[cfg(feature = "auth-workload-observation")]
        let query_start = std::time::Instant::now();
        let queried = timeout(
            Duration::from_millis(750),
            sqlx::query_as::<_, (i32, String, String)>(DOCUMENT_SQL)
                .bind(id)
                .bind(tenant)
                .fetch_optional(&mut *connection),
        )
        .await;
        #[cfg(feature = "auth-workload-observation")]
        {
            self.counters
                .observations
                .query
                .record(query_start.elapsed());
            match &queried {
                Err(_) => self.counters.observations.query_deadline(),
                Ok(Err(error)) => self.counters.observations.query_error(error),
                _ => (),
            }
        }
        let row = queried
            .map_err(|_| self.unavailable())?
            .map_err(|_| self.unavailable())?
            .ok_or_else(|| fail(StatusCode::NOT_FOUND))?;
        Ok(Document {
            id: row.0,
            title: row.1,
            body: row.2,
        })
    }
}
fn resolve<'a>(parts: &'a mut Parts, state: &'a AppState) -> ResolveFuture<'a> {
    Box::pin(async move {
        let session = parts
            .extensions
            .get::<RedisSession<IdentityRef>>()
            .cloned()
            .ok_or_else(|| fail(StatusCode::INTERNAL_SERVER_ERROR))?;
        #[cfg(feature = "auth-workload-observation")]
        let session_start = std::time::Instant::now();
        let read = session.read().await;
        #[cfg(feature = "auth-workload-observation")]
        {
            state
                .counters
                .observations
                .session_read
                .record(session_start.elapsed());
            if let Err(error) = &read {
                state.counters.observations.session_error(error);
            }
        }
        let reference = read.map_err(|e| {
            state
                .counters
                .session_errors
                .fetch_add(1, Ordering::Relaxed);
            session_error(e)
        })?;
        if matches!(reference.account, Account::Anonymous) {
            return Ok(None);
        }
        let policy = state.account().await?;
        if !policy.enabled {
            return Err(fail(StatusCode::FORBIDDEN));
        }
        let principal = Principal::from_verified(
            "urn:yetlet:yt170",
            "account-one",
            "valkey-postgres",
            Some(&policy.tenant),
        )
        .map_err(|_| fail(StatusCode::INTERNAL_SERVER_ERROR))?;
        parts.extensions.insert(policy);
        Ok(Some(principal))
    })
}
fn authorize(state: &AppState, principal: &Principal, policy: &Policy) -> Result<(), Response> {
    if !policy.enabled
        || !policy.can_view
        || policy.id != 1
        || principal.issuer() != "urn:yetlet:yt170"
        || principal.subject() != "account-one"
        || principal.authentication_name() != "valkey-postgres"
        || principal.tenant() != Some(policy.tenant.as_str())
    {
        state.counters.denied.fetch_add(1, Ordering::Relaxed);
        return Err(fail(StatusCode::FORBIDDEN));
    }
    Ok(())
}
async fn json_document(
    State(state): State<AppState>,
    user: Authenticated,
    Extension(policy): Extension<Policy>,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    authorize(&state, user.principal(), &policy)?;
    Ok(Json(state.document(id, &policy.tenant).await?).into_response())
}
fn page(document: Document) -> Page {
    let view = Element::new("main")
        .attribute("id", "private-document")
        .child(Element::new("h1").child(View::text(document.title.clone())))
        .child(
            Element::new("p")
                .attribute("data-document", document.id.to_string())
                .child(View::text(document.body)),
        );
    Page::with_metadata(
        DocumentMetadata::with_language(
            document.title,
            LanguageTag::new("en").expect("constant language"),
        ),
        view,
    )
}
async fn ssr_document(
    State(state): State<AppState>,
    user: Authenticated,
    Extension(policy): Extension<Policy>,
    Path(id): Path<i32>,
) -> Result<Response, Response> {
    authorize(&state, user.principal(), &policy)?;
    Ok(page(state.document(id, &policy.tenant).await?).into_response())
}
fn form_page(action: &str, csrf: &str) -> Result<Response, Response> {
    if csrf.len() != 64
        || !csrf
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(fail(StatusCode::SERVICE_UNAVAILABLE));
    }
    Ok(Html(format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Fixture sign-in</title></head><body><form method=\"post\" action=\"{action}\"><input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><button type=\"submit\">Continue</button></form></body></html>")).into_response())
}
async fn form(
    Extension(session): Extension<RedisSession<IdentityRef>>,
) -> Result<Response, Response> {
    match session.read().await {
        Ok(_) => (),
        Err(
            RedisSessionError::Missing
            | RedisSessionError::Expired
            | RedisSessionError::Rotated
            | RedisSessionError::Revoked,
        ) => session
            .create(IdentityRef {
                account: Account::Anonymous,
            })
            .await
            .map_err(session_error)?,
        Err(error) => return Err(session_error(error)),
    }
    form_page(
        "/auth/login",
        &session.csrf_token().await.map_err(session_error)?,
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationForm {
    csrf: String,
}
fn form_error(error: FormRejection) -> Response {
    fail(if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    })
}
async fn csrf(session: &RedisSession<IdentityRef>, form: &MutationForm) -> Result<(), Response> {
    if !session
        .verify_csrf(&form.csrf)
        .await
        .map_err(session_error)?
    {
        return Err(fail(StatusCode::FORBIDDEN));
    }
    Ok(())
}
async fn login(
    State(state): State<AppState>,
    Extension(session): Extension<RedisSession<IdentityRef>>,
    input: Result<Form<MutationForm>, FormRejection>,
) -> Result<Response, Response> {
    let Form(form) = input.map_err(form_error)?;
    csrf(&session, &form).await?;
    if !state.account().await?.enabled {
        return Err(fail(StatusCode::FORBIDDEN));
    }
    // Fixture-only trusted assertion, never a submitted credential/identity.
    session
        .replace_and_rotate(IdentityRef {
            account: Account::One,
        })
        .await
        .map_err(session_error)?;
    Ok(Redirect::to("/private/documents/1").into_response())
}
async fn logout_form(
    _user: Authenticated,
    Extension(session): Extension<RedisSession<IdentityRef>>,
) -> Result<Response, Response> {
    form_page(
        "/auth/logout",
        &session.csrf_token().await.map_err(session_error)?,
    )
}
async fn logout(
    _user: Authenticated,
    Extension(session): Extension<RedisSession<IdentityRef>>,
    input: Result<Form<MutationForm>, FormRejection>,
) -> Result<Response, Response> {
    let Form(form) = input.map_err(form_error)?;
    csrf(&session, &form).await?;
    session.destroy().await.map_err(session_error)?;
    Ok(Redirect::to("/auth/form").into_response())
}
async fn identity(_user: Authenticated) -> &'static str {
    "signed-in"
}
async fn security(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        DEFAULT_CSP.parse().expect("constant CSP"),
    );
    response
}
#[cfg(yetlet_session_admission_probe)]
async fn admission_timeline(request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-yetlet-probe")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| (1..=65535).contains(v));
    match id {
        Some(id) => yetlet::session_redis::admission_trace::request(id, next.run(request)).await,
        None => next.run(request).await,
    }
}
#[cfg(feature = "auth-workload-observation")]
async fn observations(State(state): State<AppState>) -> Response {
    let c = &state.counters;
    let mut response = Json(serde_json::json!({"observations":c.observations.snapshot(),"account_queries":c.account_queries.load(Ordering::Relaxed),"document_queries":c.document_queries.load(Ordering::Relaxed),"database_errors":c.database_errors.load(Ordering::Relaxed),"session_errors":c.session_errors.load(Ordering::Relaxed),"denied":c.denied.load(Ordering::Relaxed)})).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store".parse().expect("constant header"),
    );
    response
}
async fn database(
    socket: &str,
    password_file: &str,
    #[cfg(feature = "auth-cross-host")] cross: Option<&cross_host::Transport>,
) -> Result<AppState, ()> {
    #[cfg(feature = "auth-cross-host")]
    let password = if let Some(cross) = cross {
        cross.password()?
    } else {
        std::fs::read_to_string(password_file).map_err(|_| ())?
    };
    #[cfg(not(feature = "auth-cross-host"))]
    let password = std::fs::read_to_string(password_file).map_err(|_| ())?;
    let options = PgConnectOptions::new()
        .host(socket)
        .port(5432)
        .database("yt170")
        .username("yt170_app")
        .password(password.trim())
        .ssl_mode(PgSslMode::Disable)
        .application_name("yt170-auth-postgres")
        .statement_cache_capacity(100)
        .disable_statement_logging();
    #[cfg(feature = "auth-cross-host")]
    let options = if let Some(cross) = cross {
        options
            .host(&cross.pg_host)
            .ssl_mode(PgSslMode::VerifyFull)
            .ssl_root_cert(&cross.ca_file)
            .application_name("yt184-auth-postgres")
    } else {
        options
    };
    let pool = PgPoolOptions::new()
        .min_connections(4)
        .max_connections(4)
        .acquire_timeout(Duration::from_millis(250))
        .idle_timeout(None)
        .max_lifetime(None)
        .test_before_acquire({
            #[cfg(feature = "auth-cross-host")]
            {
                cross.is_some()
            }
            #[cfg(not(feature = "auth-cross-host"))]
            {
                false
            }
        })
        .connect_with(options)
        .await
        .map_err(|_| ())?;
    let state = AppState {
        pool,
        counters: Arc::new(Counters::default()),
    };
    let admitted = async {
        let mut held = Vec::new();let mut pids = BTreeSet::new();
        for _ in 0..4 {
            let mut c = state.pool.acquire().await.map_err(|_| ())?;
            let pid: (i32,) = sqlx::query_as("SELECT pg_backend_pid()").fetch_one(&mut *c).await.map_err(|_| ())?;
            pids.insert(pid.0);
            let account: (i32, String, bool, bool) = sqlx::query_as(ACCOUNT_SQL).bind(1_i32).fetch_one(&mut *c).await.map_err(|_| ())?;
            let doc: (i32, String, String) = sqlx::query_as(DOCUMENT_SQL).bind(1_i32).bind("alpha").fetch_one(&mut *c).await.map_err(|_| ())?;
            #[cfg(feature="auth-cross-host")]
            let remote=cross.is_some();
            #[cfg(not(feature="auth-cross-host"))]
            let remote=false;
            if account.0!=1 || account.1!="alpha" || (!remote && (!account.2||!account.3)) || doc.0!=1 {return Err(());}
            #[cfg(feature="auth-cross-host")]
            if remote {
                let ssl:(bool,)=sqlx::query_as("SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()").fetch_one(&mut *c).await.map_err(|_| ())?;
                if !ssl.0 {return Err(());}
            }
            held.push(c);
        }
        if pids.len() != 4 || state.pool.size() != 4 { return Err(()); }
        #[cfg(feature="auth-cross-host")]
        let remote=cross.is_some();
        #[cfg(not(feature="auth-cross-host"))]
        let remote=false;
        let count: (i64,) = sqlx::query_as("SELECT count(*) FROM pg_stat_activity WHERE usename='yt170_app' AND application_name=$1").bind(if remote {"yt184-auth-postgres"}else{"yt170-auth-postgres"}).fetch_one(&mut *held[0]).await.map_err(|_| ())?;
        if !remote && count.0 != 4 { return Err(()); }
        println!("{}", serde_json::json!({"event":"preconnected","backend_pids":pids,"sessions":count.0,"pool_size":4,"test_before_acquire":remote,"statement_cache_capacity":100}));
        Ok(())
    }.await;
    if admitted.is_err() {
        state.pool.close().await;
        return Err(());
    }
    Ok(state)
}
async fn run() -> Result<(), ()> {
    #[cfg(feature = "auth-workload-admission")]
    let http_admission = admission::from_environment()?;
    #[cfg(feature = "auth-cross-host")]
    let cross = cross_host::Transport::load()?;
    #[cfg(not(feature = "auth-cross-host"))]
    if std::env::var_os("YT184_TRANSPORT_CONFIG_FILE").is_some() {
        return Err(());
    }
    let env = |key| std::env::var(key).map_err(|_| ());
    #[cfg(feature = "auth-cross-host")]
    let selected = if let Some(cross) = &cross {
        Some((cross.port, cross.namespace.clone(), cross.valkey_url()?))
    } else {
        None
    };
    #[cfg(not(feature = "auth-cross-host"))]
    let selected: Option<(u16, String, String)> = None;
    let (port, namespace, endpoint) = if let Some(selected) = selected {
        selected
    } else {
        let port: u16 = env("YT170_PORT")?.parse().map_err(|_| ())?;
        let endpoint = env("YT170_VALKEY_URL")?;
        let remote_port = endpoint
            .strip_prefix("redis://127.0.0.1:")
            .and_then(|s| s.strip_suffix('/'))
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|p| *p >= 1024)
            .ok_or(())?;
        let namespace = env("YT170_NAMESPACE")?;
        if !namespace.starts_with("yt170_") {
            return Err(());
        }
        (port, namespace, format!("redis://127.0.0.1:{remote_port}/"))
    };
    if port < 1024 {
        return Err(());
    }
    #[cfg(feature = "auth-cross-host")]
    let remote = cross.is_some();
    #[cfg(not(feature = "auth-cross-host"))]
    let remote = false;
    let socket = if remote {
        String::new()
    } else {
        env("YT170_PG_SOCKET")?
    };
    let password_file = if remote {
        String::new()
    } else {
        env("YT170_PG_PASSWORD_FILE")?
    };
    let state = database(
        &socket,
        &password_file,
        #[cfg(feature = "auth-cross-host")]
        cross.as_ref(),
    )
    .await?;
    let config = if remote {
        RedisSessionConfig::new(namespace, "yt184_session", Duration::from_secs(900), 4, 256)
            .map_err(|_| ())?
    } else {
        RedisSessionConfig::new(namespace, "yt170_session", Duration::from_secs(300), 4, 256)
            .map_err(|_| ())?
            .insecure_for_local_http()
    };
    let manager = match RedisSessionManager::<IdentityRef>::connect(&endpoint, config).await {
        Ok(m) => m,
        Err(_) => {
            state.pool.close().await;
            return Err(());
        }
    };
    let served = async {
        let auth = Authentication::new("valkey-postgres", state.clone(), resolve, login_required)
            .map_err(|_| ())?;
        let protected = App::new()
            .get("/api/documents/{id}", json_document)
            .get("/private/documents/{id}", ssr_document)
            .get("/auth/me", identity)
            .get("/auth/logout-form", logout_form)
            .into_router()
            .route("/auth/logout", post(logout))
            .route_layer(middleware::from_fn_with_state(auth, authenticate));
        let scoped = protected
            .merge(
                Router::new()
                    .route("/auth/form", get(form))
                    .route("/auth/login", post(login)),
            )
            .layer(DefaultBodyLimit::max(4096))
            .route_layer(middleware::from_fn_with_state(
                manager.clone(),
                session_middleware::<IdentityRef>,
            ));
        #[cfg(feature = "auth-workload-admission")]
        let scoped = admission::install(scoped, http_admission);
        let router = Router::new()
            .route("/health", get(|| async { "ready" }))
            .merge(scoped)
            .with_state(state.clone());
        #[cfg(feature = "auth-cross-host")]
        let router = if let Some(cross) = &cross {
            router.layer(middleware::from_fn_with_state(
                cross.origin.clone(),
                cross_host::canonical_origin,
            ))
        } else {
            router
        };
        let router = router.layer(middleware::from_fn(security));
        #[cfg(feature = "auth-workload-observation")]
        let router = router.route(
            "/fixture/observations",
            get(observations).with_state(state.clone()),
        );
        #[cfg(yetlet_session_admission_probe)]
        let router = router.layer(middleware::from_fn(admission_timeline));
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
            .await
            .map_err(|_| ())?;
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .map_err(|_| ())?;
        println!("READY 127.0.0.1:{port}");
        std::io::stdout().flush().map_err(|_| ())?;
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                tokio::select! { _ = terminate.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
            })
            .await
            .map_err(|_| ())
    }
    .await;
    let drained = manager.shutdown().await.is_ok();
    state.pool.close().await;
    #[cfg(yetlet_session_admission_probe)]
    let trace_ok = std::env::var("YT173_TRACE_PATH").ok().is_some_and(|p| {
        yetlet::session_redis::admission_trace::flush(std::path::Path::new(&p)).is_ok()
    });
    let c = &state.counters;
    println!(
        "{}",
        serde_json::json!({"event":"stopped","session_drained":drained,"pool_closed":state.pool.is_closed(),"account_queries":c.account_queries.load(Ordering::Relaxed),"document_queries":c.document_queries.load(Ordering::Relaxed),"database_errors":c.database_errors.load(Ordering::Relaxed),"session_errors":c.session_errors.load(Ordering::Relaxed),"denied":c.denied.load(Ordering::Relaxed)})
    );
    if served.is_ok() && drained {
        #[cfg(yetlet_session_admission_probe)]
        if !trace_ok {
            return Err(());
        }
        Ok(())
    } else {
        Err(())
    }
}
#[tokio::main(worker_threads = 2)]
async fn main() {
    if run().await.is_err() {
        eprintln!("Functional fixture startup, serving or drain failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn private_document_json_and_ssr_oracles() {
        let document = || Document {
            id: 1,
            title: "Report 1 <&>".into(),
            body: "Private <&> \"quoted\" 'café' 1".into(),
        };
        let json = Json(document()).into_response();
        let bytes = axum::body::to_bytes(json.into_body(), 4096).await.unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            "{\"id\":1,\"title\":\"Report 1 <&>\",\"body\":\"Private <&> \\\"quoted\\\" 'café' 1\"}"
        );
        let response = page(document()).into_response();
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Report 1 &lt;&amp;&gt;</title></head><body><main id=\"private-document\"><h1>Report 1 &lt;&amp;&gt;</h1><p data-document=\"1\">Private &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39; 1</p></main></body></html>"
        );
    }
}
