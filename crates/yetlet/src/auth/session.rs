//! Explicit bridge to the existing synchronous Memory Session provider.
//!
//! Install Session outside authentication. The application maps its server-owned
//! identity reference using current account policy; this module stores no roles,
//! credentials or Principal, and does not verify login or CSRF itself.

use crate::session::{Session, SessionError};
use axum::{
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use crate::auth::{Principal, misconfigured, private_response};

/// Resolve the selected Memory Session through application-owned identity mapping.
///
/// `identify` must distinguish anonymous state from disabled/unmapped accounts
/// and enforce current mapping/validity before constructing a Principal.
/// `retired` supplies a native authentication-failure response if the record is
/// retired between opening and reading it. Session unavailability is always 503;
/// missing Session middleware is 500. Neither condition becomes anonymous.
///
/// Memory's own middleware can replace an unknown/expired cookie with a new
/// default anonymous session before this function runs. Required extraction
/// still denies access; this helper does not reinterpret that provider behavior.
#[expect(
    clippy::result_large_err,
    reason = "preserve the reviewed native Response error contract without another allocation"
)]
pub fn memory<T>(
    parts: &Parts,
    identify: impl FnOnce(T) -> Result<Option<Principal>, Response>,
    retired: impl FnOnce() -> Response,
) -> Result<Option<Principal>, Response>
where
    T: Default + Clone + Send + 'static,
{
    let session = parts
        .extensions
        .get::<Session<T>>()
        .ok_or_else(misconfigured)?;
    match session.read() {
        Ok(value) => identify(value),
        Err(SessionError::Expired | SessionError::Destroyed) => Err(private_response(retired())),
        Err(SessionError::Unavailable) => Err(private_response(
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Authentication service unavailable",
            )
                .into_response(),
        )),
    }
}
