//! Application-local selection over the existing Memory and Valkey-compatible providers.
use crate::{Failure, domain::LoginSession};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::time::Duration;
use yetlet_server_axum::App;
use yetlet_session::{Session as MemorySession, SessionConfig, SessionManager};
#[cfg(feature = "valkey")]
use yetlet_session_valkey::{
    RedisSession, RedisSessionConfig, RedisSessionError, RedisSessionManager,
    RedisSessionUpdateError,
};

/// Explicit selection. Endpoint values are never formatted or sent to a browser.
#[derive(Default)]
pub enum SessionBackend {
    #[default]
    Memory,
    #[cfg(feature = "valkey")]
    Valkey {
        endpoint: String,
        namespace: String,
        ttl: Duration,
    },
    /// Admin-owned opt-in Auth composition; uses the same Valkey Session wire format.
    #[cfg(feature = "auth-valkey")]
    AuthValkey {
        endpoint: String,
        namespace: String,
        ttl: Duration,
        policy_file: std::path::PathBuf,
    },
}

#[derive(Clone)]
pub(crate) enum Manager {
    Memory(SessionManager<LoginSession>),
    #[cfg(feature = "valkey")]
    Valkey(RedisSessionManager<LoginSession>),
}
impl Manager {
    pub async fn connect(backend: SessionBackend) -> Result<Self, Failure> {
        #[cfg(feature = "auth-valkey")]
        let backend = match backend {
            SessionBackend::AuthValkey {
                endpoint,
                namespace,
                ttl,
                ..
            } => SessionBackend::Valkey {
                endpoint,
                namespace,
                ttl,
            },
            backend => backend,
        };
        match backend {
            #[cfg(feature = "auth-valkey")]
            SessionBackend::AuthValkey { .. } => unreachable!("normalized above"),
            SessionBackend::Memory => Ok(Self::Memory(SessionManager::new(
                SessionConfig::new("yetlet_lab", Duration::from_secs(1800), 1024)
                    .map_err(|_| Failure::Invalid)?
                    .insecure_for_local_http(),
            ))),
            #[cfg(feature = "valkey")]
            SessionBackend::Valkey {
                endpoint,
                namespace,
                ttl,
            } => {
                let config = RedisSessionConfig::new(namespace, "yetlet_lab_valkey", ttl, 4, 4096)
                    .map_err(|_| Failure::Invalid)?
                    .insecure_for_local_http();
                Ok(Self::Valkey(
                    RedisSessionManager::connect(&endpoint, config)
                        .await
                        .map_err(remote_error)?,
                ))
            }
        }
    }
    pub fn scoped<S: Clone + Send + Sync + 'static>(&self, routes: App<S>) -> App<S> {
        match self {
            Self::Memory(manager) => manager.clone().scoped(routes),
            #[cfg(feature = "valkey")]
            Self::Valkey(manager) => manager.clone().scoped(routes),
        }
    }
    pub async fn shutdown(&self) -> Result<(), Failure> {
        match self {
            Self::Memory(_) => Ok(()),
            #[cfg(feature = "valkey")]
            Self::Valkey(manager) => manager.shutdown().await.map_err(remote_error),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Session {
    Memory(MemorySession<LoginSession>),
    #[cfg(feature = "valkey")]
    Valkey(RedisSession<LoginSession>),
}
impl<S: Send + Sync> FromRequestParts<S> for Session {
    type Rejection = Failure;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Failure> {
        if let Some(session) = parts.extensions.get::<MemorySession<LoginSession>>() {
            return Ok(Self::Memory(session.clone()));
        }
        #[cfg(feature = "valkey")]
        if let Some(session) = parts.extensions.get::<RedisSession<LoginSession>>() {
            return Ok(Self::Valkey(session.clone()));
        }
        Err(Failure::Unavailable)
    }
}
impl Session {
    // Only the login page may explicitly create anonymous shared state. Other
    // requests must never recreate a missing/revoked session or fall back locally.
    pub async fn ensure_anonymous(&self) -> Result<(), Failure> {
        match self {
            Self::Memory(_) => Ok(()),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => match session.read().await {
                Ok(_) => Ok(()),
                Err(
                    RedisSessionError::Missing
                    | RedisSessionError::Expired
                    | RedisSessionError::Rotated
                    | RedisSessionError::Revoked,
                ) => session
                    .create(LoginSession::default())
                    .await
                    .map_err(remote_error),
                Err(error) => Err(remote_error(error)),
            },
        }
    }
    pub async fn read(&self) -> Result<LoginSession, Failure> {
        match self {
            Self::Memory(session) => session.read().map_err(|_| Failure::Unauthorized),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session.read().await.map_err(remote_error),
        }
    }
    pub async fn csrf_token(&self) -> Result<String, Failure> {
        match self {
            Self::Memory(session) => session.csrf_token().map_err(|_| Failure::Unauthorized),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session.csrf_token().await.map_err(remote_error),
        }
    }
    pub async fn verify_csrf(&self, value: &str) -> Result<bool, Failure> {
        match self {
            Self::Memory(session) => session
                .verify_csrf(value)
                .map_err(|_| Failure::Unauthorized),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session.verify_csrf(value).await.map_err(remote_error),
        }
    }
    pub async fn replace_and_rotate(&self, value: LoginSession) -> Result<(), Failure> {
        match self {
            Self::Memory(session) => session
                .replace_and_rotate(value)
                .map_err(|_| Failure::Unavailable),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session
                .replace_and_rotate(value)
                .await
                .map_err(remote_error),
        }
    }
    pub async fn destroy(&self) -> Result<(), Failure> {
        match self {
            Self::Memory(session) => session.destroy().map_err(|_| Failure::Unauthorized),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session.destroy().await.map_err(remote_error),
        }
    }
    #[cfg(feature = "oidc")]
    pub async fn take_oidc(&self) -> Result<Option<crate::oidc::Pending>, Failure> {
        match self {
            Self::Memory(session) => session
                .try_update(|state| Ok::<_, std::convert::Infallible>(state.oidc.take()))
                .map_err(|_| Failure::Unauthorized),
            #[cfg(feature = "valkey")]
            Self::Valkey(session) => session
                .try_update(|state| Ok::<_, std::convert::Infallible>(state.oidc.take()))
                .await
                .map_err(|error| match error {
                    RedisSessionUpdateError::Session(error) => remote_error(error),
                    RedisSessionUpdateError::Application(never) => match never {},
                }),
        }
    }
}

#[cfg(feature = "valkey")]
pub(crate) fn remote_error(error: RedisSessionError) -> Failure {
    match error {
        RedisSessionError::Missing
        | RedisSessionError::Expired
        | RedisSessionError::Rotated
        | RedisSessionError::Revoked => Failure::Unauthorized,
        RedisSessionError::Configuration => Failure::Configuration,
        RedisSessionError::Conflict { .. } => Failure::Conflict,
        RedisSessionError::InvalidCookie => Failure::Invalid,
        _ => Failure::Unavailable,
    }
}

#[cfg(all(test, feature = "valkey"))]
mod tests {
    use super::*;
    #[test]
    fn stored_identity_selects_local_permissions_and_rejects_forged_state() {
        let value: LoginSession = serde_json::from_str(r#"{"user":"viewer"}"#).unwrap();
        let user = value.user.unwrap();
        assert_eq!(user.project, 1);
        assert!(!user.write);
        for value in [
            r#"{}"#,
            r#"{"user":"administrator"}"#,
            r#"{"user":{"name":"viewer","write":true,"project":2}}"#,
            r#"{"user":"alice","extra":"unrecognized"}"#,
        ] {
            assert!(serde_json::from_str::<LoginSession>(value).is_err());
        }
    }
    #[test]
    fn missing_identity_and_store_failure_have_distinct_fail_closed_responses() {
        assert_eq!(
            remote_error(RedisSessionError::Missing),
            Failure::Unauthorized
        );
        assert_eq!(
            remote_error(RedisSessionError::Revoked),
            Failure::Unauthorized
        );
        assert_eq!(
            remote_error(RedisSessionError::Unavailable),
            Failure::Unavailable
        );
        assert_eq!(remote_error(RedisSessionError::Codec), Failure::Unavailable);
    }
}

/// Required Auth extraction only when explicitly selected by this application.
#[derive(Clone)]
pub(crate) struct ProtectedSession {
    session: Session,
    #[cfg(feature = "auth-valkey")]
    pub(crate) auth: Option<(crate::auth::PolicyFile, yetlet_auth::Authenticated)>,
}
impl ProtectedSession {
    pub(crate) fn failure(&self, error: Failure) -> Failure {
        #[cfg(feature = "auth-valkey")]
        if self.auth.is_some() {
            return crate::auth::protected_failure(error);
        }
        error
    }
}
impl std::ops::Deref for ProtectedSession {
    type Target = Session;
    fn deref(&self) -> &Session {
        &self.session
    }
}
impl FromRequestParts<crate::AppState> for ProtectedSession {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        use axum::response::IntoResponse;
        #[cfg(feature = "auth-valkey")]
        let auth = if let Some(policy) = &state.policy {
            let identity = yetlet_auth::Authenticated::from_request_parts(parts, state).await?;
            Some((policy.clone(), identity))
        } else {
            None
        };
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        Ok(Self {
            session,
            #[cfg(feature = "auth-valkey")]
            auth,
        })
    }
}
