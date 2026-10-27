//! Opt-in native identity resolution, independent of Session and business policy.
//!
//! Install [`authenticate`] with Axum's `from_fn_with_state` on selected routes.
//! The application supplies verification and failure responses explicitly. An
//! [`Authenticated`] principal identifies a caller; it grants no permissions.
//! See the crate README for a complete composition example and trust boundary.

#![doc = include_str!("../README.md")]

#[cfg(target_arch = "wasm32")]
compile_error!("yetlet-auth is native-only; keep it out of browser dependencies");

#[cfg(feature = "session")]
pub mod session;

use std::{fmt, future::Future, pin::Pin, sync::Arc};

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request, State},
    http::{StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Maximum UTF-8 byte length of an issuer or local identity namespace.
pub const MAX_ISSUER_BYTES: usize = 2048;
/// Maximum UTF-8 byte length of a subject. Protocol-specific rules are separate.
pub const MAX_SUBJECT_BYTES: usize = 255;
/// Maximum UTF-8 byte length of an application-configured authentication name.
pub const MAX_AUTHENTICATION_NAME_BYTES: usize = 64;
/// Maximum UTF-8 byte length of a validated optional tenant binding.
pub const MAX_TENANT_BYTES: usize = 255;

/// Invalid principal shape/size or authentication registration configuration.
/// Never contains the rejected input value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidAuthenticationInput;

impl fmt::Display for InvalidAuthenticationInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid principal or authentication scheme")
    }
}
impl std::error::Error for InvalidAuthenticationInput {}

fn valid(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

/// Immutable verified identity. No roles, credentials, serde or View conversion.
#[derive(Clone)]
pub struct Principal {
    issuer: String,
    subject: String,
    authentication_name: String,
    tenant: Option<String>,
}

impl Principal {
    /// Assert prior verification by trusted application/provider code.
    ///
    /// This checks shape only: it does not authenticate a token, cookie, header
    /// or submitted identity. Values are preserved exactly, never normalized.
    pub fn from_verified(
        issuer: &str,
        subject: &str,
        authentication_name: &str,
        tenant: Option<&str>,
    ) -> Result<Self, InvalidAuthenticationInput> {
        if !valid(issuer, MAX_ISSUER_BYTES)
            || !valid(subject, MAX_SUBJECT_BYTES)
            || !valid(authentication_name, MAX_AUTHENTICATION_NAME_BYTES)
            || tenant.is_some_and(|value| !valid(value, MAX_TENANT_BYTES))
        {
            return Err(InvalidAuthenticationInput);
        }
        Ok(Self {
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
            authentication_name: authentication_name.to_owned(),
            tenant: tenant.map(str::to_owned),
        })
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    /// Application-configured authentication registration/resolver name.
    /// This is separate from an HTTP WWW-Authenticate protocol scheme.
    pub fn authentication_name(&self) -> &str {
        &self.authentication_name
    }
    pub fn tenant(&self) -> Option<&str> {
        self.tenant.as_deref()
    }

    /// Compare issuer+subject only.
    /// Check authentication name, tenant and permissions separately.
    pub fn same_identity(&self, other: &Self) -> bool {
        self.issuer == other.issuer && self.subject == other.subject
    }
}

impl fmt::Debug for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Principal { identity: [REDACTED] }")
    }
}

/// Borrowed request-parts resolution; boxing is paid only on installed routes.
pub type ResolveFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<Principal>, Response>> + Send + 'a>>;
/// Ordinary function pointer with explicitly supplied typed application state.
pub type Resolver<S> = for<'a> fn(&'a mut Parts, &'a S) -> ResolveFuture<'a>;

struct Configuration<S> {
    authentication_name: String,
    state: S,
    resolve: Resolver<S>,
    unauthenticated: Arc<dyn Fn() -> Response + Send + Sync>,
}

/// Explicit per-scope configuration; clones share the supplied state.
pub struct Authentication<S>(Arc<Configuration<S>>);

impl<S> Clone for Authentication<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<S> Authentication<S> {
    /// Select one authentication name, resolver and anonymous-required failure response.
    ///
    /// The resolver returns None only for a definite anonymous outcome. Invalid
    /// credentials and outages return a redacted native error response. Required
    /// extraction calls `unauthenticated`; optional extraction returns None.
    /// A 401 response must include an applicable WWW-Authenticate challenge.
    /// A form consumer may explicitly return a local 303 login redirect instead.
    pub fn new(
        authentication_name: &str,
        state: S,
        resolve: Resolver<S>,
        unauthenticated: impl Fn() -> Response + Send + Sync + 'static,
    ) -> Result<Self, InvalidAuthenticationInput> {
        if !valid(authentication_name, MAX_AUTHENTICATION_NAME_BYTES) {
            return Err(InvalidAuthenticationInput);
        }
        Ok(Self(Arc::new(Configuration {
            authentication_name: authentication_name.to_owned(),
            state,
            resolve,
            unauthenticated: Arc::new(unauthenticated),
        })))
    }
}

#[derive(Clone)]
struct Resolved {
    principal: Option<Principal>,
    unauthenticated: Arc<dyn Fn() -> Response + Send + Sync>,
}

/// Required verified identity extractor; still requires application authorization.
#[derive(Clone, Debug)]
pub struct Authenticated(Principal);

impl Authenticated {
    pub fn principal(&self) -> &Principal {
        &self.0
    }
    pub fn into_principal(self) -> Principal {
        self.0
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for Authenticated {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Response> {
        let resolved = parts
            .extensions
            .get::<Resolved>()
            .ok_or_else(misconfigured)?;
        Ok(resolved.principal.clone().map(Self))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Authenticated {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Response> {
        let resolved = parts
            .extensions
            .get::<Resolved>()
            .ok_or_else(misconfigured)?;
        resolved
            .principal
            .clone()
            .map(Self)
            .ok_or_else(|| private_response((resolved.unauthenticated)()))
    }
}

fn misconfigured() -> Response {
    let mut response = (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Authentication configuration error",
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}

fn private_response(mut response: Response) -> Response {
    if response.status() == StatusCode::UNAUTHORIZED
        && !response
            .headers()
            .get_all(header::WWW_AUTHENTICATE)
            .iter()
            .any(|v| v.to_str().is_ok_and(|value| !value.trim().is_empty()))
    {
        return misconfigured();
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}

/// Native Axum middleware. Apply with from_fn_with_state after registering routes.
///
/// Resolves once from Parts without reading the body. Duplicate resolvers and
/// results with a different authentication name fail closed. Native sibling
/// scopes and 404/405 behavior are unchanged. All scoped responses are no-store;
/// existing headers survive.
pub async fn authenticate<S: Send + Sync + 'static>(
    State(authentication): State<Authentication<S>>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();
    if parts.extensions.get::<Resolved>().is_some() {
        return misconfigured();
    }
    // Only the private resolved state is accepted by Authenticated. A public
    // Principal extension is never evidence of successful resolution.
    parts.extensions.remove::<Principal>();
    let config = &authentication.0;
    let principal = match (config.resolve)(&mut parts, &config.state).await {
        Ok(principal) => principal,
        Err(response) => return private_response(response),
    };
    if principal
        .as_ref()
        .is_some_and(|p| p.authentication_name() != config.authentication_name)
    {
        return misconfigured();
    }
    parts.extensions.insert(Resolved {
        principal,
        unauthenticated: Arc::clone(&config.unauthenticated),
    });
    private_response(next.run(Request::from_parts(parts, body)).await)
}
