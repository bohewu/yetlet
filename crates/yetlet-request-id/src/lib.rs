//! Optional, process-local request correlation. Never enabled by Yetlet core.
//!
//! Add this crate explicitly and select global or scoped registration. IDs are
//! per provider instance, not globally unique, persistent, secret or auth tokens.
//! Server-generated IDs replace any untrusted inbound value; no propagation
//! policy is inferred. Clones share one counter, independent instances do not.

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use yetlet_server_axum::{
    App,
    middleware::{Next, from_fn_with_state, with_global},
};

pub const HEADER: &str = "x-yetlet-request-id";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidStart;
impl std::fmt::Display for InvalidStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("request ID start must be between 1 and u64::MAX - 1")
    }
}
impl std::error::Error for InvalidStart {}

/// Constructing this value allocates a counter only when an app chooses it.
#[derive(Clone, Debug)]
pub struct RequestIds {
    next: Arc<AtomicU64>,
}

impl Default for RequestIds {
    fn default() -> Self {
        Self::new()
    }
}
impl RequestIds {
    pub fn new() -> Self {
        Self {
            next: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Typed startup validation; no routes or network effects on failure.
    pub fn try_new(first: u64) -> Result<Self, InvalidStart> {
        if first == 0 || first == u64::MAX {
            return Err(InvalidStart);
        }
        Ok(Self {
            next: Arc::new(AtomicU64::new(first)),
        })
    }

    // compare_exchange_weak preserves the established atomic allocation
    // semantics across the workspace's supported Rust baseline.
    fn next_id(&self) -> Option<RequestId> {
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            let following = current.checked_add(1)?;
            match self.next.compare_exchange_weak(
                current,
                following,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(RequestId(current)),
                Err(actual) => current = actual,
            }
        }
    }

    /// Install only on previously registered routes in this App/group.
    ///
    /// ```
    /// use yetlet_server_axum::{App, extract::Extension};
    /// use yetlet_request_id::{RequestId, RequestIds};
    /// async fn item(Extension(id): Extension<RequestId>) -> String { id.0.to_string() }
    /// let app: axum::Router = App::new().group("/api", |api| {
    ///     api.get("/item", item).configure(|api| RequestIds::new().scoped(api))
    /// }).into_router();
    /// ```
    pub fn scoped<S>(self, app: App<S>) -> App<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        app.route_layer(from_fn_with_state(self, assign_request_id))
    }

    /// Finalize a fully assembled, state-supplied app with a pre-routing layer.
    /// Do not add routes to the returned outer fallback router; that bypasses
    /// the global boundary. Scoped registration has no such outer wrapper.
    pub fn global(self, app: App) -> Router {
        with_global(
            app.into_router(),
            from_fn_with_state(self, assign_request_id),
        )
    }
}

/// Native Axum middleware, also usable directly with from_fn_with_state.
/// No request/response body reads, buffering, or lock held across await.
pub async fn assign_request_id(
    State(ids): State<RequestIds>,
    mut request: Request,
    next: Next,
) -> Response {
    let id = match ids.next_id() {
        Some(id) => id,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Request ID capacity exhausted",
            )
                .into_response();
        }
    };
    request.extensions_mut().insert(id);
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        HEADER,
        HeaderValue::from_str(&id.0.to_string()).expect("integer header"),
    );
    response
}
