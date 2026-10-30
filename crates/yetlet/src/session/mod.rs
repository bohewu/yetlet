//! Explicitly installed, process-local server-side sessions.
//!
//! The provider owns opaque random session IDs, absolute expiry, a bounded
//! in-memory store and a per-session CSRF token. Session is not authentication.
//! No dependency or middleware is enabled by Yetlet core.

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::server_axum::{
    App,
    middleware::{Next, from_fn_with_state},
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use subtle::ConstantTimeEq;

const TOKEN_BYTES: usize = 32;
const TOKEN_HEX_LEN: usize = TOKEN_BYTES * 2;
const MAX_TOKEN_ATTEMPTS: usize = 8;
const MAX_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
const MAX_SESSION_CAPACITY: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionConfigError {
    InvalidCookieName,
    InvalidTtl,
    InvalidCapacity,
}

impl fmt::Display for SessionConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidCookieName => "session cookie name is invalid",
            Self::InvalidTtl => "session TTL must be between one second and 30 days",
            Self::InvalidCapacity => "session capacity must be between 1 and 1,000,000",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SessionConfigError {}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    cookie_name: String,
    ttl_seconds: u64,
    max_sessions: usize,
    secure: bool,
}

impl SessionConfig {
    pub fn new(
        cookie_name: impl Into<String>,
        ttl: Duration,
        max_sessions: usize,
    ) -> Result<Self, SessionConfigError> {
        let cookie_name = cookie_name.into();
        if !valid_cookie_name(&cookie_name) {
            return Err(SessionConfigError::InvalidCookieName);
        }
        let ttl_seconds = ttl.as_secs();
        if ttl_seconds == 0 || ttl_seconds > MAX_TTL_SECONDS {
            return Err(SessionConfigError::InvalidTtl);
        }
        if max_sessions == 0 || max_sessions > MAX_SESSION_CAPACITY {
            return Err(SessionConfigError::InvalidCapacity);
        }
        Ok(Self {
            cookie_name,
            ttl_seconds,
            max_sessions,
            secure: true,
        })
    }

    /// Explicitly permit an insecure cookie for a direct localhost HTTP demo.
    /// Production deployments should retain the secure default.
    pub fn insecure_for_local_http(mut self) -> Self {
        self.secure = false;
        self
    }

    pub fn cookie_name(&self) -> &str {
        &self.cookie_name
    }

    pub fn is_secure(&self) -> bool {
        self.secure
    }
}

fn valid_cookie_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionClockError;

pub trait SessionClock: Send + Sync + 'static {
    fn now_seconds(&self) -> Result<u64, SessionClockError>;
}

#[derive(Debug, Default)]
pub struct SystemClock;

impl SessionClock for SystemClock {
    fn now_seconds(&self) -> Result<u64, SessionClockError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|_| SessionClockError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    Unavailable,
    Expired,
    Destroyed,
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Unavailable => "session is unavailable",
            Self::Expired => "session has expired",
            Self::Destroyed => "session has been destroyed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SessionError {}

#[derive(Debug)]
pub enum SessionUpdateError<E> {
    Session(SessionError),
    Application(E),
}

struct Record<T> {
    value: T,
    csrf_token: String,
    expires_at: u64,
}

#[derive(Default)]
struct Store<T> {
    records: HashMap<String, Record<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CookieDirective {
    None,
    Set(String),
    Clear,
}

struct RequestSession {
    id: Option<String>,
    csrf_token: Option<String>,
    cookie_directive: CookieDirective,
}

struct ManagerInner<T> {
    config: SessionConfig,
    clock: Arc<dyn SessionClock>,
    store: Mutex<Store<T>>,
}

pub struct SessionManager<T> {
    inner: Arc<ManagerInner<T>>,
}

impl<T> Clone for SessionManager<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> fmt::Debug for SessionManager<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionManager")
            .field("cookie_name", &self.inner.config.cookie_name)
            .field("ttl_seconds", &self.inner.config.ttl_seconds)
            .field("max_sessions", &self.inner.config.max_sessions)
            .field("secure", &self.inner.config.secure)
            .finish_non_exhaustive()
    }
}

impl<T> SessionManager<T>
where
    T: Default + Clone + Send + 'static,
{
    pub fn new(config: SessionConfig) -> Self {
        Self::with_clock(config, Arc::new(SystemClock))
    }

    pub fn with_clock(config: SessionConfig, clock: Arc<dyn SessionClock>) -> Self {
        Self {
            inner: Arc::new(ManagerInner {
                config,
                clock,
                store: Mutex::new(Store::default()),
            }),
        }
    }

    /// Install sessions only on routes already registered in this App/group.
    pub fn scoped<S>(self, app: App<S>) -> App<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        app.route_layer(from_fn_with_state(self, session_middleware::<T>))
    }

    fn now_seconds(&self) -> Result<u64, SessionError> {
        self.inner
            .clock
            .now_seconds()
            .map_err(|_| SessionError::Unavailable)
    }

    fn store(&self) -> Result<MutexGuard<'_, Store<T>>, SessionError> {
        self.inner
            .store
            .lock()
            .map_err(|_| SessionError::Unavailable)
    }

    fn open(&self, headers: &HeaderMap) -> Result<Session<T>, OpenError> {
        let now = self.now_seconds().map_err(|_| OpenError::Session)?;
        let supplied = session_cookie(headers, &self.inner.config.cookie_name)?;
        let mut store = self.store().map_err(|_| OpenError::Session)?;
        store.records.retain(|_, record| record.expires_at > now);

        if let Some(id) = supplied
            && let Some(record) = store.records.get(&id)
        {
            return Ok(Session::new(
                self.clone(),
                id,
                record.csrf_token.clone(),
                CookieDirective::None,
            ));
        }

        if store.records.len() >= self.inner.config.max_sessions {
            return Err(OpenError::Capacity);
        }
        let expires_at = now
            .checked_add(self.inner.config.ttl_seconds)
            .ok_or(OpenError::Session)?;
        let id = unique_token(&store.records).map_err(|_| OpenError::Entropy)?;
        let csrf_token = random_token().map_err(|_| OpenError::Entropy)?;
        store.records.insert(
            id.clone(),
            Record {
                value: T::default(),
                csrf_token: csrf_token.clone(),
                expires_at,
            },
        );
        Ok(Session::new(
            self.clone(),
            id.clone(),
            csrf_token,
            CookieDirective::Set(id),
        ))
    }

    fn session_cookie(&self, id: &str) -> Result<HeaderValue, SessionError> {
        let mut cookie = Cookie::new(self.inner.config.cookie_name.clone(), id.to_owned());
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_secure(self.inner.config.secure);
        cookie.set_max_age(CookieDuration::seconds(
            self.inner.config.ttl_seconds as i64,
        ));
        HeaderValue::from_str(&cookie.to_string()).map_err(|_| SessionError::Unavailable)
    }

    fn removal_cookie(&self) -> Result<HeaderValue, SessionError> {
        let mut cookie = Cookie::new(self.inner.config.cookie_name.clone(), String::new());
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_secure(self.inner.config.secure);
        cookie.make_removal();
        HeaderValue::from_str(&cookie.to_string()).map_err(|_| SessionError::Unavailable)
    }

    fn apply_cookie(&self, session: &Session<T>, response: &mut Response) {
        let directive = match session.request.lock() {
            Ok(request) => request.cookie_directive.clone(),
            Err(_) => CookieDirective::Clear,
        };
        let value = match directive {
            CookieDirective::None => None,
            CookieDirective::Set(id) => self.session_cookie(&id).ok(),
            CookieDirective::Clear => self.removal_cookie().ok(),
        };
        if let Some(value) = value {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
}

pub struct Session<T> {
    manager: SessionManager<T>,
    request: Arc<Mutex<RequestSession>>,
}

impl<T> Clone for Session<T> {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
            request: Arc::clone(&self.request),
        }
    }
}

impl<T> fmt::Debug for Session<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Session { .. }")
    }
}

impl<T> Session<T>
where
    T: Default + Clone + Send + 'static,
{
    fn new(
        manager: SessionManager<T>,
        id: String,
        csrf_token: String,
        cookie_directive: CookieDirective,
    ) -> Self {
        Self {
            manager,
            request: Arc::new(Mutex::new(RequestSession {
                id: Some(id),
                csrf_token: Some(csrf_token),
                cookie_directive,
            })),
        }
    }

    fn with_record<R>(&self, action: impl FnOnce(&mut Record<T>) -> R) -> Result<R, SessionError> {
        let now = self.manager.now_seconds()?;
        let mut request = self.request.lock().map_err(|_| SessionError::Unavailable)?;
        let id = request.id.clone().ok_or(SessionError::Destroyed)?;
        let mut store = self.manager.store()?;
        let expired = match store.records.get(&id) {
            Some(record) => record.expires_at <= now,
            None => {
                request.id = None;
                request.csrf_token = None;
                // A concurrent request may already have issued a replacement
                // cookie. A stale response must never delete that newer ID.
                request.cookie_directive = CookieDirective::None;
                return Err(SessionError::Destroyed);
            }
        };
        if expired {
            store.records.remove(&id);
            request.id = None;
            request.csrf_token = None;
            request.cookie_directive = CookieDirective::None;
            return Err(SessionError::Expired);
        }
        let record = store
            .records
            .get_mut(&id)
            .ok_or(SessionError::Unavailable)?;
        Ok(action(record))
    }
    pub fn read(&self) -> Result<T, SessionError> {
        self.with_record(|record| record.value.clone())
    }

    /// Apply an application mutation to a clone and commit only on success.
    pub fn try_update<R, E>(
        &self,
        action: impl FnOnce(&mut T) -> Result<R, E>,
    ) -> Result<R, SessionUpdateError<E>> {
        self.with_record(|record| {
            let mut candidate = record.value.clone();
            match action(&mut candidate) {
                Ok(value) => {
                    record.value = candidate;
                    Ok(value)
                }
                Err(error) => Err(error),
            }
        })
        .map_err(SessionUpdateError::Session)?
        .map_err(SessionUpdateError::Application)
    }

    pub fn csrf_token(&self) -> Result<String, SessionError> {
        self.with_record(|record| record.csrf_token.clone())
    }

    pub fn verify_csrf(&self, submitted: &str) -> Result<bool, SessionError> {
        let expected = self.csrf_token()?;
        Ok(expected.as_bytes().ct_eq(submitted.as_bytes()).into())
    }

    pub fn rotate(&self) -> Result<(), SessionError> {
        self.rotate_inner(None)
    }

    pub fn replace_and_rotate(&self, value: T) -> Result<(), SessionError> {
        self.rotate_inner(Some(value))
    }

    fn rotate_inner(&self, replacement: Option<T>) -> Result<(), SessionError> {
        let now = self.manager.now_seconds()?;
        let expires_at = now
            .checked_add(self.manager.inner.config.ttl_seconds)
            .ok_or(SessionError::Unavailable)?;
        let csrf_token = random_token().map_err(|_| SessionError::Unavailable)?;
        let mut request = self.request.lock().map_err(|_| SessionError::Unavailable)?;
        let old_id = request.id.clone().ok_or(SessionError::Destroyed)?;
        let mut store = self.manager.store()?;
        let record = match store.records.get(&old_id) {
            Some(record) => record,
            None => {
                request.id = None;
                request.csrf_token = None;
                request.cookie_directive = CookieDirective::None;
                return Err(SessionError::Destroyed);
            }
        };
        if record.expires_at <= now {
            store.records.remove(&old_id);
            request.id = None;
            request.csrf_token = None;
            request.cookie_directive = CookieDirective::None;
            return Err(SessionError::Expired);
        }
        // This snapshot and identity replacement share the same store lock as
        // try_update. A successful update either commits first and is retained,
        // or observes that the old identity has been retired.
        let value = replacement.unwrap_or_else(|| record.value.clone());
        let new_id = unique_token(&store.records).map_err(|_| SessionError::Unavailable)?;
        store.records.remove(&old_id);
        store.records.insert(
            new_id.clone(),
            Record {
                value,
                csrf_token: csrf_token.clone(),
                expires_at,
            },
        );
        request.id = Some(new_id.clone());
        request.csrf_token = Some(csrf_token);
        request.cookie_directive = CookieDirective::Set(new_id);
        Ok(())
    }
    pub fn destroy(&self) -> Result<(), SessionError> {
        let mut request = self.request.lock().map_err(|_| SessionError::Unavailable)?;
        let id = request.id.clone().ok_or(SessionError::Destroyed)?;
        let removed = self.manager.store()?.records.remove(&id).is_some();
        request.id = None;
        request.csrf_token = None;
        if removed {
            request.cookie_directive = CookieDirective::Clear;
            Ok(())
        } else {
            // A concurrent request may already have rotated this identity. A
            // stale destroy response must not delete the replacement cookie.
            request.cookie_directive = CookieDirective::None;
            Err(SessionError::Destroyed)
        }
    }
}
#[derive(Debug)]
enum OpenError {
    InvalidCookie,
    Capacity,
    Entropy,
    Session,
}

fn session_cookie(headers: &HeaderMap, name: &str) -> Result<Option<String>, OpenError> {
    let mut found = None;
    for header_value in headers.get_all(header::COOKIE).iter() {
        let raw = header_value
            .to_str()
            .map_err(|_| OpenError::InvalidCookie)?;
        for parsed in Cookie::split_parse(raw) {
            let cookie = parsed.map_err(|_| OpenError::InvalidCookie)?;
            if cookie.name() == name {
                if found.is_some() || !valid_token(cookie.value()) {
                    return Err(OpenError::InvalidCookie);
                }
                found = Some(cookie.value().to_owned());
            }
        }
    }
    Ok(found)
}

fn valid_token(value: &str) -> bool {
    value.len() == TOKEN_HEX_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn random_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::getrandom(&mut bytes)?;
    let mut token = String::with_capacity(TOKEN_HEX_LEN);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        token.push(HEX[(byte >> 4) as usize] as char);
        token.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(token)
}

fn unique_token<T>(records: &HashMap<String, Record<T>>) -> Result<String, ()> {
    for _ in 0..MAX_TOKEN_ATTEMPTS {
        let token = random_token().map_err(|_| ())?;
        if !records.contains_key(&token) {
            return Ok(token);
        }
    }
    Err(())
}

fn apply_private_headers(response: &mut Response) {
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

pub async fn session_middleware<T>(
    State(manager): State<SessionManager<T>>,
    mut request: Request,
    next: Next,
) -> Response
where
    T: Default + Clone + Send + 'static,
{
    let session = match manager.open(request.headers()) {
        Ok(session) => session,
        Err(error) => {
            let (status, message, clear_cookie) = match error {
                OpenError::InvalidCookie => {
                    (StatusCode::BAD_REQUEST, "Invalid session cookie", true)
                }
                OpenError::Capacity => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Session capacity reached",
                    false,
                ),
                OpenError::Entropy | OpenError::Session => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Session service unavailable",
                    false,
                ),
            };
            let mut response = (status, message).into_response();
            if clear_cookie && let Ok(cookie) = manager.removal_cookie() {
                response.headers_mut().append(header::SET_COOKIE, cookie);
            }
            apply_private_headers(&mut response);
            return response;
        }
    };
    request.extensions_mut().insert(session.clone());
    let mut response = next.run(request).await;
    manager.apply_cookie(&session, &mut response);
    apply_private_headers(&mut response);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::Extension,
        http::{Method, Request},
    };
    use std::{
        sync::{
            Barrier,
            atomic::{AtomicU64, Ordering},
        },
        thread,
    };
    use tower::ServiceExt;

    #[derive(Debug)]
    struct ManualClock(AtomicU64);

    impl ManualClock {
        fn new(now: u64) -> Self {
            Self(AtomicU64::new(now))
        }

        fn set(&self, now: u64) {
            self.0.store(now, Ordering::Relaxed);
        }
    }

    impl SessionClock for ManualClock {
        fn now_seconds(&self) -> Result<u64, SessionClockError> {
            Ok(self.0.load(Ordering::Relaxed))
        }
    }

    fn config(capacity: usize) -> SessionConfig {
        SessionConfig::new("yetlet-test", Duration::from_secs(60), capacity)
            .unwrap()
            .insecure_for_local_http()
    }

    async fn read(Extension(session): Extension<Session<u32>>) -> String {
        session.read().unwrap().to_string()
    }

    async fn increment(Extension(session): Extension<Session<u32>>) -> String {
        session
            .try_update(|value| {
                *value += 1;
                Ok::<_, ()>(*value)
            })
            .unwrap()
            .to_string()
    }

    async fn rotate(Extension(session): Extension<Session<u32>>) -> String {
        let token = session.csrf_token().unwrap();
        session.rotate().unwrap();
        format!("{token}:{}", session.csrf_token().unwrap())
    }

    fn app(manager: SessionManager<u32>) -> Router {
        App::new()
            .get("/health", || async { "ok" })
            .group("/session", |group| {
                group
                    .get("/read", read)
                    .post("/increment", increment)
                    .post("/rotate", rotate)
                    .configure(|group| manager.scoped(group))
            })
            .into_router()
    }

    async fn send(router: Router, request: Request<Body>) -> (StatusCode, HeaderMap, String) {
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        (status, headers, String::from_utf8(body.to_vec()).unwrap())
    }

    fn request(method: Method, path: &str, cookie: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn cookie_pair(headers: &HeaderMap) -> String {
        headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    fn cookie_for_session(manager: &SessionManager<u32>, session: &Session<u32>) -> String {
        let id = session
            .request
            .lock()
            .unwrap()
            .id
            .clone()
            .expect("active session id");
        format!("{}={id}", manager.inner.config.cookie_name)
    }

    fn headers_with_cookie(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        headers
    }

    #[test]
    fn configuration_is_bounded_and_secure_by_default() {
        assert!(SessionConfig::new("", Duration::from_secs(1), 1).is_err());
        assert!(SessionConfig::new("session", Duration::ZERO, 1).is_err());
        assert!(SessionConfig::new("session", Duration::from_secs(1), 0).is_err());
        let config = SessionConfig::new("session", Duration::from_secs(60), 1).unwrap();
        assert!(config.is_secure());
        assert!(!config.insecure_for_local_http().is_secure());
    }

    #[tokio::test]
    async fn scoped_session_is_opt_in_and_persists() {
        let router = app(SessionManager::new(config(4)));
        let (_, outside, _) = send(router.clone(), request(Method::GET, "/health", None)).await;
        assert!(!outside.contains_key(header::SET_COOKIE));

        let (status, headers, body) = send(
            router.clone(),
            request(Method::POST, "/session/increment", None),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "1");
        let cookie = cookie_pair(&headers);
        let set_cookie = headers[header::SET_COOKIE].to_str().unwrap();
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Lax"));
        assert!(!set_cookie.contains("Secure"));

        let (_, _, body) = send(router, request(Method::GET, "/session/read", Some(&cookie))).await;
        assert_eq!(body, "1");
    }

    #[tokio::test]
    async fn unknown_cookie_is_rotated_instead_of_adopted() {
        let router = app(SessionManager::new(config(4)));
        let supplied = format!("yetlet-test={}", "a".repeat(TOKEN_HEX_LEN));
        let (_, headers, body) = send(
            router,
            request(Method::GET, "/session/read", Some(&supplied)),
        )
        .await;
        assert_eq!(body, "0");
        assert_ne!(cookie_pair(&headers), supplied);
    }

    #[tokio::test]
    async fn expiry_replaces_the_session_and_old_cookie() {
        let clock = Arc::new(ManualClock::new(100));
        let manager = SessionManager::with_clock(config(4), clock.clone());
        let router = app(manager);
        let (_, headers, _) = send(
            router.clone(),
            request(Method::POST, "/session/increment", None),
        )
        .await;
        let old_cookie = cookie_pair(&headers);
        clock.set(161);
        let (_, headers, body) = send(
            router,
            request(Method::GET, "/session/read", Some(&old_cookie)),
        )
        .await;
        assert_eq!(body, "0");
        assert_ne!(cookie_pair(&headers), old_cookie);
    }

    #[tokio::test]
    async fn capacity_is_bounded_until_sessions_expire() {
        let router = app(SessionManager::new(config(1)));
        let (status, _, _) =
            send(router.clone(), request(Method::GET, "/session/read", None)).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _, body) = send(router, request(Method::GET, "/session/read", None)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, "Session capacity reached");
    }

    #[tokio::test]
    async fn duplicate_or_malformed_session_cookie_fails_closed() {
        let router = app(SessionManager::new(config(4)));
        for cookie in [
            "yetlet-test=bad",
            &format!(
                "yetlet-test={}; yetlet-test={}",
                "a".repeat(TOKEN_HEX_LEN),
                "b".repeat(TOKEN_HEX_LEN)
            ),
        ] {
            let (status, headers, _) = send(
                router.clone(),
                request(Method::GET, "/session/read", Some(cookie)),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(
                headers[header::SET_COOKIE]
                    .to_str()
                    .unwrap()
                    .contains("Max-Age=0")
            );
        }
    }

    #[test]
    fn rotation_never_loses_a_successful_concurrent_update() {
        for _ in 0..128 {
            let manager = SessionManager::<u32>::new(config(8));
            let initial = manager.open(&HeaderMap::new()).unwrap();
            let old_cookie = cookie_for_session(&manager, &initial);
            let headers = headers_with_cookie(&old_cookie);
            let updater = manager.open(&headers).unwrap();
            let rotator = manager.open(&headers).unwrap();
            let rotator_observer = rotator.clone();
            let barrier = Arc::new(Barrier::new(3));

            let update_barrier = Arc::clone(&barrier);
            let update = thread::spawn(move || {
                update_barrier.wait();
                updater.try_update(|value| {
                    *value += 1;
                    Ok::<_, ()>(())
                })
            });
            let rotate_barrier = Arc::clone(&barrier);
            let rotate = thread::spawn(move || {
                rotate_barrier.wait();
                rotator.rotate()
            });
            barrier.wait();

            let update_result = update.join().unwrap();
            assert_eq!(rotate.join().unwrap(), Ok(()));
            let new_cookie = cookie_for_session(&manager, &rotator_observer);
            let current = manager.open(&headers_with_cookie(&new_cookie)).unwrap();
            match update_result {
                Ok(()) => assert_eq!(current.read().unwrap(), 1),
                Err(SessionUpdateError::Session(SessionError::Destroyed)) => {
                    assert_eq!(current.read().unwrap(), 0)
                }
                other => panic!("unexpected concurrent update result: {other:?}"),
            }
        }
    }

    #[test]
    fn stale_request_after_rotation_does_not_clear_replacement_cookie() {
        let manager = SessionManager::<u32>::new(config(8));
        let initial = manager.open(&HeaderMap::new()).unwrap();
        let old_cookie = cookie_for_session(&manager, &initial);
        let headers = headers_with_cookie(&old_cookie);
        let stale = manager.open(&headers).unwrap();
        let rotator = manager.open(&headers).unwrap();

        rotator.replace_and_rotate(7).unwrap();
        let new_cookie = cookie_for_session(&manager, &rotator);
        assert_ne!(old_cookie, new_cookie);
        assert_eq!(stale.read(), Err(SessionError::Destroyed));

        let mut stale_response = StatusCode::SERVICE_UNAVAILABLE.into_response();
        manager.apply_cookie(&stale, &mut stale_response);
        assert!(!stale_response.headers().contains_key(header::SET_COOKIE));

        let current = manager.open(&headers_with_cookie(&new_cookie)).unwrap();
        assert_eq!(current.read().unwrap(), 7);
    }

    #[test]
    fn stale_destroy_after_rotation_does_not_clear_replacement_cookie() {
        let manager = SessionManager::<u32>::new(config(8));
        let initial = manager.open(&HeaderMap::new()).unwrap();
        let old_cookie = cookie_for_session(&manager, &initial);
        let headers = headers_with_cookie(&old_cookie);
        let stale = manager.open(&headers).unwrap();
        let rotator = manager.open(&headers).unwrap();

        let submitted_csrf = stale.csrf_token().unwrap();
        assert!(stale.verify_csrf(&submitted_csrf).unwrap());

        rotator.replace_and_rotate(7).unwrap();
        let new_cookie = cookie_for_session(&manager, &rotator);
        assert_ne!(old_cookie, new_cookie);

        assert_eq!(stale.destroy(), Err(SessionError::Destroyed));
        let mut stale_response = StatusCode::SERVICE_UNAVAILABLE.into_response();
        manager.apply_cookie(&stale, &mut stale_response);
        assert!(!stale_response.headers().contains_key(header::SET_COOKIE));

        let current = manager.open(&headers_with_cookie(&new_cookie)).unwrap();
        assert_eq!(current.read().unwrap(), 7);
    }

    #[tokio::test]
    async fn rotation_changes_identity_and_csrf_and_invalidates_old_cookie() {
        let router = app(SessionManager::new(config(4)));
        let (_, headers, _) = send(
            router.clone(),
            request(Method::POST, "/session/increment", None),
        )
        .await;
        let old_cookie = cookie_pair(&headers);
        let (_, headers, body) = send(
            router.clone(),
            request(Method::POST, "/session/rotate", Some(&old_cookie)),
        )
        .await;
        let new_cookie = cookie_pair(&headers);
        let (old_csrf, new_csrf) = body.split_once(':').unwrap();
        assert_ne!(old_cookie, new_cookie);
        assert_ne!(old_csrf, new_csrf);

        let (_, headers, body) = send(
            router,
            request(Method::GET, "/session/read", Some(&old_cookie)),
        )
        .await;
        assert_eq!(body, "0");
        assert_ne!(cookie_pair(&headers), old_cookie);
    }
}
