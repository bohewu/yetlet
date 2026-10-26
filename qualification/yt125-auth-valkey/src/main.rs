//! Linux/local qualification fixture: synthetic identity, never credential verification.
#![allow(
    clippy::result_large_err,
    reason = "Preserve native Axum responses at the existing Auth resolver/handler boundary"
)]

use axum::{
    Extension, Form, Router,
    extract::{DefaultBodyLimit, Request, State, rejection::FormRejection},
    http::{StatusCode, header, request::Parts},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf, time::Duration};
use tokio::{io::AsyncReadExt, time::timeout};
use yetlet_auth::{Authenticated, Authentication, Principal, ResolveFuture, authenticate};
use yetlet_session_valkey::{
    RedisSession, RedisSessionConfig, RedisSessionError, RedisSessionManager, session_middleware,
};

// A small server-store DTO, not a serialized Principal or permission grant.
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
    // A decodable but unmapped reference must deny, never become anonymous.
    Unmapped,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    enabled: bool,
    can_view: bool,
}

#[derive(Clone)]
struct AppState {
    policy_file: PathBuf,
    instance: String,
}

fn fail(status: StatusCode) -> Response {
    (status, "Request unavailable or denied").into_response()
}

fn login_required() -> Response {
    Redirect::to("/auth/form").into_response()
}

fn remote_error(error: RedisSessionError) -> Response {
    use RedisSessionError::*;
    match error {
        Missing | Expired | Rotated | Revoked => login_required(),
        Configuration => fail(StatusCode::INTERNAL_SERVER_ERROR),
        InvalidCookie => fail(StatusCode::BAD_REQUEST),
        Conflict { .. } => fail(StatusCode::CONFLICT),
        // Includes OutcomeUnknown: do not retry or claim successful mutation.
        _ => fail(StatusCode::SERVICE_UNAVAILABLE),
    }
}

impl AppState {
    async fn policy(&self) -> Result<Policy, Response> {
        let read = async {
            // Trusted single-host local file. O_NONBLOCK prevents a substituted
            // FIFO from trapping a blocking-pool worker during open; reject all
            // nonregular files on the opened handle, not a racy path metadata check.
            let file = tokio::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
                .open(&self.policy_file)
                .await
                .map_err(|_| ())?;
            let metadata = file.metadata().await.map_err(|_| ())?;
            if !metadata.is_file() || metadata.len() > 1024 {
                return Err(());
            }
            let mut bytes = Vec::with_capacity(1025);
            file.take(1025)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| ())?;
            if bytes.len() > 1024 {
                return Err(());
            }
            serde_json::from_slice(&bytes).map_err(|_| ())
        };
        timeout(Duration::from_millis(250), read)
            .await
            .map_err(|_| fail(StatusCode::SERVICE_UNAVAILABLE))?
            .map_err(|_| fail(StatusCode::SERVICE_UNAVAILABLE))
    }
}

fn resolve<'a>(parts: &'a mut Parts, state: &'a AppState) -> ResolveFuture<'a> {
    Box::pin(async move {
        let session = parts
            .extensions
            .get::<RedisSession<IdentityRef>>()
            .cloned()
            .ok_or_else(|| fail(StatusCode::INTERNAL_SERVER_ERROR))?;
        let reference = session.read().await.map_err(remote_error)?;
        // Even optional anonymous access must not hide unavailable policy.
        let policy = state.policy().await?;
        match reference.account {
            Account::Anonymous => Ok(None),
            Account::One if policy.enabled => {
                Principal::from_verified("urn:yetlet:yt125", "account-one", "valkey", Some("one"))
                    .map(Some)
                    .map_err(|_| fail(StatusCode::INTERNAL_SERVER_ERROR))
            }
            _ => Err(fail(StatusCode::FORBIDDEN)),
        }
    })
}

fn html_form(action: &str, label: &str, csrf: &str) -> Result<Response, Response> {
    // Provider tokens are bounded lowercase hexadecimal. Validate before placing
    // the token in an HTML attribute; all other strings are application constants.
    if csrf.len() != 64
        || !csrf
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(fail(StatusCode::SERVICE_UNAVAILABLE));
    }
    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>YT-125 fixture</title></head><body><h1>Test-only synthetic identity</h1><form method=\"post\" action=\"{action}\"><input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><button type=\"submit\">{label}</button></form></body></html>"
    ))
    .into_response())
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
            .map_err(remote_error)?,
        Err(error) => return Err(remote_error(error)),
    }
    html_form(
        "/auth/login",
        "Use fixture identity",
        &session.csrf_token().await.map_err(remote_error)?,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationForm {
    csrf: String,
}

async fn check_csrf(
    session: &RedisSession<IdentityRef>,
    form: &MutationForm,
) -> Result<(), Response> {
    if !session
        .verify_csrf(&form.csrf)
        .await
        .map_err(remote_error)?
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
    check_csrf(&session, &form).await?;
    if !state.policy().await?.enabled {
        return Err(fail(StatusCode::FORBIDDEN));
    }
    // Deliberate fixture-only trust assertion. No submitted field selects an
    // account; this is NOT password, OIDC, or any real credential verification.
    session
        .replace_and_rotate(IdentityRef {
            account: Account::One,
        })
        .await
        .map_err(remote_error)?;
    Ok(Redirect::to("/auth/private").into_response())
}

async fn private(
    State(state): State<AppState>,
    user: Authenticated,
    Extension(session): Extension<RedisSession<IdentityRef>>,
) -> Result<Response, Response> {
    let principal = user.principal();
    let policy = state.policy().await?;
    // Resource authorization is separate from successful identity extraction.
    if !policy.enabled
        || !policy.can_view
        || principal.issuer() != "urn:yetlet:yt125"
        || principal.subject() != "account-one"
        || principal.authentication_name() != "valkey"
        || principal.tenant() != Some("one")
    {
        return Err(fail(StatusCode::FORBIDDEN));
    }
    html_form(
        "/auth/logout",
        "Sign out",
        &session.csrf_token().await.map_err(remote_error)?,
    )
}

async fn optional(user: Option<Authenticated>) -> &'static str {
    if user.is_some() {
        "signed-in"
    } else {
        "anonymous"
    }
}

async fn logout(
    _user: Authenticated,
    Extension(session): Extension<RedisSession<IdentityRef>>,
    input: Result<Form<MutationForm>, FormRejection>,
) -> Result<Response, Response> {
    let Form(form) = input.map_err(form_error)?;
    check_csrf(&session, &form).await?;
    session.destroy().await.map_err(remote_error)?;
    Ok(Redirect::to("/auth/form").into_response())
}

fn form_error(error: FormRejection) -> Response {
    fail(if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    })
}

async fn security(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'"
            .parse()
            .unwrap(),
    );
    response
}

async fn run() -> Result<(), ()> {
    let env = |name| std::env::var(name).map_err(|_| ());
    let endpoint = env("YT125_VALKEY_URL")?;
    // Narrow local fixture destination, no credentials or inherited app settings.
    let port = endpoint
        .strip_prefix("redis://127.0.0.1:")
        .and_then(|s| s.strip_suffix('/'))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|p| *p != 0)
        .ok_or(())?;
    let namespace = env("YT125_NAMESPACE")?;
    if !namespace.starts_with("yt125_") {
        return Err(());
    }
    let instance = env("YT125_INSTANCE")?;
    if !matches!(instance.as_str(), "A" | "B") {
        return Err(());
    }
    let ttl: u64 = env("YT125_TTL_SECONDS")?.parse().map_err(|_| ())?;
    if !(1..=1800).contains(&ttl) {
        return Err(());
    }
    let state = AppState {
        policy_file: PathBuf::from(env("YT125_POLICY_FILE")?),
        instance,
    };
    let config =
        RedisSessionConfig::new(namespace, "yt125_session", Duration::from_secs(ttl), 4, 256)
            .map_err(|_| ())?
            .insecure_for_local_http();
    let manager =
        RedisSessionManager::<IdentityRef>::connect(&format!("redis://127.0.0.1:{port}/"), config)
            .await
            .map_err(|_| ())?;
    // Perform all fallible setup after connect inside this scope so shutdown is
    // always awaited even when listening or serving fails.
    let served = async {
        let authentication = Authentication::new("valkey", state.clone(), resolve, login_required)
            .map_err(|_| ())?;
        let protected = Router::new()
            .route("/auth/private", get(private))
            .route("/auth/optional", get(optional))
            .route("/auth/logout", post(logout))
            .route_layer(middleware::from_fn_with_state(authentication, authenticate));
        let session_routes = protected
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
        let router = Router::new()
            .route(
                "/health",
                get(|State(state): State<AppState>| async move { state.instance }),
            )
            .merge(session_routes)
            .layer(middleware::from_fn(security))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|_| ())?;
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .map_err(|_| ())?;
        println!(
            "READY {} {}",
            state.instance,
            listener.local_addr().map_err(|_| ())?
        );
        std::io::stdout().flush().map_err(|_| ())?;
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                terminate.recv().await;
            })
            .await
            .map_err(|_| ())
    }
    .await;
    let drained = manager.shutdown().await.map_err(|_| ());
    served.and(drained)
}

#[tokio::main(worker_threads = 2)]
async fn main() {
    if run().await.is_err() {
        eprintln!("Fixture startup, serving, or drain failed");
        std::process::exit(1);
    }
    println!("STOPPED drained");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_supported_identity_records_decode() {
        let anonymous: IdentityRef = serde_json::from_str(r#"{"account":"anonymous"}"#).unwrap();
        assert!(matches!(anonymous.account, Account::Anonymous));
        for invalid in [
            r#"{}"#,
            r#"{"account":null}"#,
            r#"{"account":"administrator"}"#,
            r#"{"account":"one","can_view":true}"#,
        ] {
            assert!(serde_json::from_str::<IdentityRef>(invalid).is_err());
        }
    }

    #[tokio::test]
    async fn missing_typed_provider_and_remote_failures_are_explicit() {
        let (mut parts, _) = Request::new(axum::body::Body::empty()).into_parts();
        let state = AppState {
            policy_file: PathBuf::from("unused"),
            instance: "A".into(),
        };
        let response = resolve(&mut parts, &state).await.err().unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        for error in [
            RedisSessionError::Codec,
            RedisSessionError::InvalidRecord,
            RedisSessionError::Closed,
            RedisSessionError::QueueTimeout,
        ] {
            assert_eq!(
                remote_error(error).status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
        }
        assert_eq!(
            remote_error(RedisSessionError::Conflict {
                expected_revision: 1,
                observed_revision: 2
            })
            .status(),
            StatusCode::CONFLICT
        );
    }
}
