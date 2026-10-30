//! Explicitly selected, experimental single-primary Redis sessions.
//!
//! Session is not authentication. Loading never creates state. The application
//! explicitly calls `create` or `replace_and_rotate` after its own policy checks.
//! No JavaScript, Memory dependency/fallback, implicit callback retry or DI layer.
mod store;

/// Unsupported research-build linkage; absent from ordinary consumer builds.
#[cfg(yetlet_session_admission_probe)]
#[doc(hidden)]
pub mod admission_trace;

use crate::server_axum::App;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt, marker::PhantomData, sync::Arc, time::Duration};
use store::{Snapshot, Store, Ticket};
use subtle::ConstantTimeEq;
use tokio::{
    sync::Mutex,
    time::{Instant, timeout_at},
};

/// Server-owned reconciliation ticket. Debug never includes IDs, state or CSRF.
#[derive(Clone)]
pub struct RedisOperationId(Ticket);
impl fmt::Debug for RedisOperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RedisOperationId([redacted])")
    }
}

#[derive(Debug)]
pub enum RedisSessionError {
    Configuration,
    InvalidCookie,
    AlreadyExists,
    Busy,
    QueueTimeout,
    DispatchDeadline,
    Closed,
    Unavailable,
    InvalidRecord,
    PayloadTooLarge,
    Codec,
    Protocol,
    Missing,
    Expired,
    Rotated,
    Revoked,
    Capacity,
    Conflict {
        expected_revision: u64,
        observed_revision: u64,
    },
    OutcomeUnknown(RedisOperationId),
}
impl fmt::Display for RedisSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // All variants contain only closed categories, revisions or a redacted ticket.
        write!(f, "Redis session: {self:?}")
    }
}
impl std::error::Error for RedisSessionError {}

#[derive(Debug)]
pub enum RedisSessionUpdateError<E> {
    Session(RedisSessionError),
    Application(E),
}
#[derive(Debug)]
pub enum RedisOperationStatus {
    Committed { revision: u64 },
    Rejected(RedisSessionError),
    Unresolved,
}

/// Bounded experimental settings. Fixed network/result bounds are documented in
/// the crate README; no unbounded Redis client defaults are inherited.
#[derive(Clone, Debug)]
pub struct RedisSessionConfig {
    namespace: String,
    cookie_name: String,
    ttl_ms: u64,
    max_in_flight: usize,
    payload_limit: usize,
    secure: bool,
}
impl RedisSessionConfig {
    pub fn new(
        namespace: impl Into<String>,
        cookie_scope: &str,
        ttl: Duration,
        max_in_flight: usize,
        payload_limit: usize,
    ) -> Result<Self, RedisSessionError> {
        let namespace = namespace.into();
        if namespace.is_empty()
            || namespace.len() > 64
            || !namespace
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
            || cookie_scope.is_empty()
            || cookie_scope.len() > 20
            || !cookie_scope
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            || ttl.as_millis() == 0
            || ttl > Duration::from_secs(30 * 24 * 60 * 60)
            || !(1..=4).contains(&max_in_flight)
            || !(1..=4096).contains(&payload_limit)
        {
            return Err(RedisSessionError::Configuration);
        }
        Ok(Self {
            namespace,
            cookie_name: format!("__Host-{cookie_scope}"),
            ttl_ms: ttl.as_millis() as u64,
            max_in_flight,
            payload_limit,
            secure: true,
        })
    }
    /// Only for explicitly selected local HTTP qualification. Uses a distinct name.
    pub fn insecure_for_local_http(mut self) -> Self {
        self.cookie_name = self.cookie_name.trim_start_matches("__Host-").to_owned();
        self.secure = false;
        self
    }
    pub fn cookie_name(&self) -> &str {
        &self.cookie_name
    }
}

pub struct RedisSessionManager<T> {
    store: Store,
    config: RedisSessionConfig,
    state: PhantomData<fn() -> T>,
}
impl<T> Clone for RedisSessionManager<T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            config: self.config.clone(),
            state: PhantomData,
        }
    }
}
impl<T> fmt::Debug for RedisSessionManager<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RedisSessionManager([redacted])")
    }
}

struct RequestSession {
    identity: Option<String>,
    pending: Option<Ticket>,
}
pub struct RedisSession<T> {
    manager: RedisSessionManager<T>,
    request: Arc<Mutex<RequestSession>>,
}
impl<T> Clone for RedisSession<T> {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
            request: self.request.clone(),
        }
    }
}
impl<T> fmt::Debug for RedisSession<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RedisSession([redacted])")
    }
}

fn deadline() -> Instant {
    Instant::now() + store::CALL_TIMEOUT
}
fn committed(result: store::Mutation, expected: u64) -> Result<u64, RedisSessionError> {
    match result.status.as_str() {
        "ok" => Ok(result.revision),
        "conflict" => Err(RedisSessionError::Conflict {
            expected_revision: expected,
            observed_revision: result.revision,
        }),
        _ => Err(store::lifecycle(&result.status)),
    }
}
fn typed<T: DeserializeOwned>(snapshot: &Snapshot) -> Result<T, RedisSessionError> {
    serde_json::from_value(snapshot.state.clone()).map_err(|_| RedisSessionError::Codec)
}
fn private_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("Cookie"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static> RedisSessionManager<T> {
    /// Endpoint is application-owned and never included in errors or Debug.
    /// Single-primary redis:// remains the default private mode. Optional `tls`
    /// admits checked rediss:// using native Rustls trust; insecure fragments reject.
    /// Deployment, ACL and persistence/recovery guarantees require separate evidence.
    pub async fn connect(
        endpoint: &str,
        config: RedisSessionConfig,
    ) -> Result<Self, RedisSessionError> {
        let store = Store::connect(endpoint, config.clone()).await?;
        Ok(Self {
            store,
            config,
            state: PhantomData,
        })
    }
    /// Install on the current route scope only. Native Axum Extension also works.
    pub fn scoped<S: Clone + Send + Sync + 'static>(self, app: App<S>) -> App<S> {
        app.route_layer(from_fn_with_state(self, session_middleware::<T>))
    }
    /// Parse one configured opaque cookie; no I/O or implicit Session creation.
    /// A subsequent read always checks the authoritative record.
    pub fn open(&self, headers: &HeaderMap) -> Result<RedisSession<T>, RedisSessionError> {
        let mut identity = None;
        for h in headers.get_all(header::COOKIE) {
            let h = h.to_str().map_err(|_| RedisSessionError::InvalidCookie)?;
            for pair in h.split(';') {
                let pair = pair.trim();
                let Some((name, value)) = pair.split_once('=') else {
                    if pair == self.config.cookie_name {
                        return Err(RedisSessionError::InvalidCookie);
                    }
                    continue;
                };
                if name != self.config.cookie_name {
                    continue;
                }
                if identity.is_some() || !store::valid_id(value) {
                    return Err(RedisSessionError::InvalidCookie);
                }
                identity = Some(value.to_owned());
            }
        }
        Ok(RedisSession {
            manager: self.clone(),
            request: Arc::new(Mutex::new(RequestSession {
                identity,
                pending: None,
            })),
        })
    }
    /// Status only: never replay an operation or recover the callback's return value.
    pub async fn resolve_operation(
        &self,
        ticket: &RedisOperationId,
    ) -> Result<RedisOperationStatus, RedisSessionError> {
        let result = self.store.resolve(&ticket.0, deadline()).await?;
        if result.status == "unresolved" {
            return Ok(RedisOperationStatus::Unresolved);
        }
        match committed(result, ticket.0.expected_revision()) {
            Ok(revision) => Ok(RedisOperationStatus::Committed { revision }),
            Err(error) => Ok(RedisOperationStatus::Rejected(error)),
        }
    }
    /// Stops admission across all clones and drains/reaps owned driver work.
    pub async fn shutdown(&self) -> Result<(), RedisSessionError> {
        self.store.shutdown().await.map(|_| ())
    }
    async fn finalize(
        &self,
        session: &RedisSession<T>,
    ) -> Result<Option<HeaderValue>, RedisSessionError> {
        let end = deadline();
        let request = timeout_at(end, session.request.lock())
            .await
            .map_err(|_| RedisSessionError::QueueTimeout)?;
        let Some(ticket) = &request.pending else {
            return Ok(None);
        };
        let result = self.store.resolve(ticket, end).await?;
        if result.status == "unresolved" {
            return Err(RedisSessionError::OutcomeUnknown(RedisOperationId(
                ticket.clone(),
            )));
        }
        if result.status != "ok" {
            return Ok(None);
        }
        let id = ticket.replacement().unwrap_or(ticket.identity());
        let snapshot = match self.store.read(id, end).await {
            Ok(snapshot) => snapshot,
            Err(
                RedisSessionError::Missing
                | RedisSessionError::Expired
                | RedisSessionError::Rotated
                | RedisSessionError::Revoked,
            ) => return Ok(None),
            Err(error) => return Err(error),
        };
        let _: T = typed(&snapshot)?;
        let value = format!(
            "{}={id}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax{}",
            self.config.cookie_name,
            self.config.ttl_ms.div_ceil(1000),
            if self.config.secure { "; Secure" } else { "" }
        );
        HeaderValue::from_str(&value)
            .map(Some)
            .map_err(|_| RedisSessionError::Protocol)
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static> RedisSession<T> {
    /// Explicit creation only. Existing active Sessions require replace_and_rotate.
    pub async fn create(&self, value: T) -> Result<(), RedisSessionError> {
        let end = deadline();
        let mut request = timeout_at(end, self.request.lock())
            .await
            .map_err(|_| RedisSessionError::QueueTimeout)?;
        request.pending = None;
        if let Some(id) = &request.identity {
            match self.manager.store.read(id, end).await {
                Ok(_) => return Err(RedisSessionError::AlreadyExists),
                Err(
                    RedisSessionError::Missing
                    | RedisSessionError::Expired
                    | RedisSessionError::Rotated
                    | RedisSessionError::Revoked,
                ) => (),
                Err(error) => return Err(error),
            }
        }
        let value = serde_json::to_value(value).map_err(|_| RedisSessionError::Codec)?;
        let ticket = self
            .manager
            .store
            .prepare("create", None, 0, &value, end)
            .await?;
        committed(self.manager.store.execute(&ticket, end).await?, 0)?;
        request.identity = Some(ticket.identity().to_owned());
        request.pending = Some(ticket);
        Ok(())
    }
    pub async fn read(&self) -> Result<T, RedisSessionError> {
        let end = deadline();
        let request = timeout_at(end, self.request.lock())
            .await
            .map_err(|_| RedisSessionError::QueueTimeout)?;
        let id = request
            .identity
            .as_deref()
            .ok_or(RedisSessionError::Missing)?;
        typed(&self.manager.store.read(id, end).await?)
    }
    pub async fn csrf_token(&self) -> Result<String, RedisSessionError> {
        let end = deadline();
        let request = timeout_at(end, self.request.lock())
            .await
            .map_err(|_| RedisSessionError::QueueTimeout)?;
        let id = request
            .identity
            .as_deref()
            .ok_or(RedisSessionError::Missing)?;
        let snapshot = self.manager.store.read(id, end).await?;
        let _: T = typed(&snapshot)?;
        Ok(snapshot.csrf)
    }
    /// Applications must invoke this for their protected unsafe actions.
    pub async fn verify_csrf(&self, supplied: &str) -> Result<bool, RedisSessionError> {
        Ok(self
            .csrf_token()
            .await?
            .as_bytes()
            .ct_eq(supplied.as_bytes())
            .into())
    }
    /// Runs the callback once, returns its original R only after acknowledged CAS.
    /// External callback side effects are not rolled back. Unknown outcomes drop R.
    pub async fn try_update<R, E>(
        &self,
        action: impl FnOnce(&mut T) -> Result<R, E>,
    ) -> Result<R, RedisSessionUpdateError<E>> {
        use RedisSessionUpdateError::Session;
        let end = deadline();
        let mut request = timeout_at(end, self.request.lock())
            .await
            .map_err(|_| Session(RedisSessionError::QueueTimeout))?;
        let pending = request.pending.take();
        let id = request
            .identity
            .as_deref()
            .ok_or(Session(RedisSessionError::Missing))?;
        let snapshot = self.manager.store.read(id, end).await.map_err(Session)?;
        let mut candidate: T = typed(&snapshot).map_err(Session)?;
        let value = action(&mut candidate).map_err(RedisSessionUpdateError::Application)?;
        let candidate =
            serde_json::to_value(candidate).map_err(|_| Session(RedisSessionError::Codec))?;
        let ticket = self
            .manager
            .store
            .prepare("update", Some(id), snapshot.revision, &candidate, end)
            .await
            .map_err(Session)?;
        committed(
            self.manager
                .store
                .execute(&ticket, end)
                .await
                .map_err(Session)?,
            snapshot.revision,
        )
        .map_err(Session)?;
        request.pending = pending;
        Ok(value)
    }
    pub async fn rotate(&self) -> Result<(), RedisSessionError> {
        self.change("rotate", None).await
    }
    pub async fn replace_and_rotate(&self, value: T) -> Result<(), RedisSessionError> {
        self.change("replace", Some(value)).await
    }
    /// Revokes in Redis; deliberately emits no clearing cookie.
    pub async fn destroy(&self) -> Result<(), RedisSessionError> {
        self.change("revoke", None).await
    }
    async fn change(&self, kind: &str, replacement: Option<T>) -> Result<(), RedisSessionError> {
        let end = deadline();
        let mut request = timeout_at(end, self.request.lock())
            .await
            .map_err(|_| RedisSessionError::QueueTimeout)?;
        request.pending = None;
        let id = request
            .identity
            .as_deref()
            .ok_or(RedisSessionError::Missing)?;
        let snapshot = self.manager.store.read(id, end).await?;
        let _: T = typed(&snapshot)?;
        let value = match replacement {
            Some(value) => serde_json::to_value(value).map_err(|_| RedisSessionError::Codec)?,
            None => snapshot.state,
        };
        let ticket = self
            .manager
            .store
            .prepare(kind, Some(id), snapshot.revision, &value, end)
            .await?;
        committed(
            self.manager.store.execute(&ticket, end).await?,
            snapshot.revision,
        )?;
        request.identity = ticket.replacement().map(str::to_owned);
        request.pending = if kind == "revoke" { None } else { Some(ticket) };
        Ok(())
    }
}

pub async fn session_middleware<T>(
    State(manager): State<RedisSessionManager<T>>,
    mut request: Request,
    next: Next,
) -> Response
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    let session = match manager.open(request.headers()) {
        Ok(session) => session,
        Err(_) => {
            let mut response = (StatusCode::BAD_REQUEST, "Invalid session cookie").into_response();
            private_headers(&mut response);
            return response;
        }
    };
    request.extensions_mut().insert(session.clone());
    let mut response = next.run(request).await;
    if response.status().is_success() || response.status().is_redirection() {
        match manager.finalize(&session).await {
            Ok(Some(cookie)) => {
                response.headers_mut().append(header::SET_COOKIE, cookie);
            }
            Ok(None) => (),
            Err(_) => {
                response = (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Session service unavailable",
                )
                    .into_response();
            }
        }
    }
    private_headers(&mut response);
    response
}
