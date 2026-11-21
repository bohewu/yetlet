//! An owning, request-only adapter. It never reads the body implicitly.

use std::convert::Infallible;

use axum::{
    body::Body,
    extract::rejection::{PathRejection, QueryRejection},
    extract::{FromRequest, FromRequestParts, MatchedPath, OriginalUri, Path, Query, Request},
    http::{Extensions, HeaderMap, Method, Uri, request::Parts},
};
use serde::de::DeserializeOwned;

/// The original request head/body plus an explicit typed application-state handle.
///
/// `S` is cloned by extraction; prefer Arc-backed state. Request-local values
/// stay in the request's own Extensions, not in S. This type deliberately has
/// no Clone/Serialize implementation and belongs only to the server adapter.
/// In a handler it is a body-owning extractor and must be the last extractor.
pub struct RequestContext<S = ()> {
    parts: Parts,
    body: Body,
    state: S,
}

impl<S> RequestContext<S> {
    pub fn new(request: Request, state: S) -> Self {
        let (parts, body) = request.into_parts();
        Self { parts, body, state }
    }

    pub fn method(&self) -> &Method {
        &self.parts.method
    }
    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.parts.uri
    }
    pub fn path(&self) -> &str {
        self.parts.uri.path()
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.parts.headers
    }
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.parts.headers
    }
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }
    pub fn state(&self) -> &S {
        &self.state
    }
    pub fn body(&self) -> &Body {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// None before application routing or on an unmatched fallback.
    pub fn matched_path(&self) -> Option<&str> {
        self.parts
            .extensions
            .get::<MatchedPath>()
            .map(MatchedPath::as_str)
    }

    /// Full external URI when Axum nesting has stripped a route prefix.
    pub fn original_uri(&self) -> &Uri {
        self.parts
            .extensions
            .get::<OriginalUri>()
            .map(|original| &original.0)
            .unwrap_or(&self.parts.uri)
    }

    /// On-demand Axum query decoding; does not touch the body.
    pub fn query<T: DeserializeOwned>(&self) -> Result<T, QueryRejection> {
        Query::<T>::try_from_uri(&self.parts.uri).map(|query| query.0)
    }

    /// Move the original body once, retaining its streaming behavior.
    pub fn into_body(self) -> Body {
        self.body
    }

    /// Hand ownership back to native Axum Next or another HTTP service.
    pub fn into_request(self) -> Request {
        Request::from_parts(self.parts, self.body)
    }
}

impl<S: Send + Sync> RequestContext<S> {
    /// Available only after matching. Native PathRejection describes malformed
    /// parameters or missing routing metadata; there is no eager path cache.
    pub async fn path_params<T: DeserializeOwned + Send>(&mut self) -> Result<T, PathRejection> {
        Path::<T>::from_request_parts(&mut self.parts, &self.state)
            .await
            .map(|path| path.0)
    }
}

impl<S: Clone + Send + Sync> FromRequest<S> for RequestContext<S> {
    type Rejection = Infallible;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::new(request, state.clone()))
    }
}
