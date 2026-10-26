//! Larger reference application composed from ordinary Rust feature modules.
//!
//! Identity, catalog, cart and orders own their routes and dependencies. There
//! is no plugin registry, controller base class, service locator or ORM.

mod cart;
mod catalog;
#[cfg(feature = "catalog-cache")]
mod catalog_cache;
mod database;
mod identity;
mod orders;

#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, fmt, io, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Form, Router,
    extract::rejection::FormRejection,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use yetlet_core::{Element, View};
use yetlet_server_axum::App;
use yetlet_session::{Session, SessionConfig, SessionConfigError, SessionError, SessionManager};

pub use database::{Database, DatabaseOptions};
pub use identity::DemoIdentities;

const MAX_FORM_BODY: usize = 4 * 1024;
const SESSION_TTL: Duration = Duration::from_secs(30 * 60);
const SESSION_CAPACITY: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UserId {
    Alice,
    Bob,
}

impl UserId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Alice => "alice",
            Self::Bob => "bob",
        }
    }

    pub(crate) const fn display(self) -> &'static str {
        match self {
            Self::Alice => "Alice",
            Self::Bob => "Bob",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CartState {
    pub(crate) lines: BTreeMap<String, u8>,
    pub(crate) revision: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShopSession {
    pub(crate) user: Option<UserId>,
    pub(crate) cart: CartState,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) database: Database,
    pub(crate) identities: Arc<DemoIdentities>,
    pub(crate) modules: ShopModules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopModules {
    pub identity: bool,
    pub catalog: bool,
    pub cart: bool,
    pub orders: bool,
}

impl Default for ShopModules {
    fn default() -> Self {
        Self {
            identity: true,
            catalog: true,
            cart: true,
            orders: true,
        }
    }
}

#[derive(Debug)]
pub enum ShopAppError {
    NonLoopbackListener,
    InvalidModules(&'static str),
    Session(SessionConfigError),
}

impl fmt::Display for ShopAppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonLoopbackListener => {
                formatter.write_str("the synthetic modular shop must bind a loopback address")
            }
            Self::InvalidModules(message) => formatter.write_str(message),
            Self::Session(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ShopAppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Session(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SessionConfigError> for ShopAppError {
    fn from(error: SessionConfigError) -> Self {
        Self::Session(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    InvalidForm,
    PayloadTooLarge,
    InvalidCsrf,
    InvalidCredentials,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    InvalidCart,
    OutOfStock,
    Session,
    Database,
}

impl Failure {
    fn status(self) -> StatusCode {
        match self {
            Self::InvalidForm | Self::InvalidCart => StatusCode::BAD_REQUEST,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::InvalidCsrf | Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidCredentials | Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict | Self::OutOfStock => StatusCode::CONFLICT,
            Self::Session | Self::Database => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::InvalidForm => "Invalid form",
            Self::PayloadTooLarge => "Form body exceeds the 4 KiB limit",
            Self::InvalidCsrf => "Invalid CSRF token",
            Self::InvalidCredentials => "Invalid synthetic identity or login code",
            Self::Unauthenticated => "Authentication required",
            Self::Forbidden => "You are not authorized to access this order",
            Self::NotFound => "Resource was not found",
            Self::Conflict => "The cart or order state changed; reload and retry",
            Self::InvalidCart => "Cart content is invalid",
            Self::OutOfStock => "One or more products are out of stock",
            Self::Session => "Session unavailable",
            Self::Database => "Shop storage unavailable",
        }
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.status(), self.message()).into_response()
    }
}

pub(crate) fn parse_form<T>(form: Result<Form<T>, FormRejection>) -> Result<T, Failure> {
    match form {
        Ok(Form(value)) => Ok(value),
        Err(FormRejection::BytesRejection(_)) => Err(Failure::PayloadTooLarge),
        Err(_) => Err(Failure::InvalidForm),
    }
}

pub(crate) fn map_session(error: SessionError) -> Failure {
    match error {
        SessionError::Unavailable | SessionError::Expired | SessionError::Destroyed => {
            Failure::Session
        }
    }
}

pub(crate) fn require_csrf(session: &Session<ShopSession>, submitted: &str) -> Result<(), Failure> {
    match session.verify_csrf(submitted).map_err(map_session)? {
        true => Ok(()),
        false => Err(Failure::InvalidCsrf),
    }
}

pub(crate) fn current_user(session: &Session<ShopSession>) -> Result<UserId, Failure> {
    session
        .read()
        .map_err(map_session)?
        .user
        .ok_or(Failure::Unauthenticated)
}

pub(crate) fn hidden(name: &'static str, value: impl Into<String>) -> View {
    Element::new("input")
        .attribute("type", "hidden")
        .attribute("name", name)
        .attribute("value", value)
        .into()
}

pub(crate) fn money(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn merge_module(app: App<AppState>, module: App<AppState>) -> App<AppState> {
    app.map_router(|router| router.merge(module.into_router()))
}

fn validate_modules(modules: ShopModules) -> Result<(), ShopAppError> {
    if modules.cart && (!modules.identity || !modules.catalog) {
        return Err(ShopAppError::InvalidModules(
            "cart requires identity and catalog modules",
        ));
    }
    if modules.orders && (!modules.identity || !modules.catalog || !modules.cart) {
        return Err(ShopAppError::InvalidModules(
            "orders require identity, catalog and cart modules",
        ));
    }
    Ok(())
}

/// Build the shop from explicit ordinary Rust module functions.
///
/// Disabling one module omits only its routes. No discovery or runtime plugin
/// registry is involved.
pub fn app(
    listener: SocketAddr,
    database: Database,
    identities: DemoIdentities,
    modules: ShopModules,
) -> Result<Router, ShopAppError> {
    if !listener.ip().is_loopback() {
        return Err(ShopAppError::NonLoopbackListener);
    }
    validate_modules(modules)?;
    let sessions = SessionManager::<ShopSession>::new(
        SessionConfig::new("yetlet-modular-shop", SESSION_TTL, SESSION_CAPACITY)?
            .insecure_for_local_http(),
    );
    let state = AppState {
        database,
        identities: Arc::new(identities),
        modules,
    };
    let root = if modules.catalog {
        "/shop/catalog"
    } else {
        "/healthz"
    };
    let mut application = App::<AppState>::new()
        .get("/", move || async move { Redirect::to(root) })
        .get("/healthz", || async { "ok" });
    if modules.identity {
        application = merge_module(application, identity::module(sessions.clone()));
    }
    if modules.catalog {
        application = merge_module(application, catalog::module());
    }
    if modules.cart {
        application = merge_module(application, cart::module(sessions.clone()));
    }
    if modules.orders {
        application = merge_module(application, orders::module(sessions));
    }
    Ok(application.with_state(state).into_router())
}

/// Validate the anonymous synthetic sample's listener before enabling insecure
/// localhost Session cookies.
pub fn listen_address(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the synthetic modular shop must bind a loopback address",
        ));
    }
    Ok(address)
}

pub(crate) const FORM_LIMIT: usize = MAX_FORM_BODY;
