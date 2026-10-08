//! Small demonstration middleware, not an authentication or tracing subsystem.
use axum::{http::HeaderValue, response::Response};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use yetlet_server_axum::{RequestContext, middleware::Next};

#[derive(Clone, Copy)]
pub(crate) struct RequestId(pub u64);
#[derive(Clone, Copy)]
pub(crate) struct ApiScope;

/// Safe, explicitly selected SSR data. Neither RequestContext nor arbitrary
/// request extensions/headers are serializable through this type.
pub(crate) struct RenderRequest {
    pub request_id: Option<u64>,
}
impl RenderRequest {
    pub fn from_context<S>(context: &RequestContext<S>) -> Self {
        Self {
            request_id: context.extensions().get::<RequestId>().map(|id| id.0),
        }
    }
}

pub(crate) async fn assign_request_id(
    mut context: RequestContext<Arc<AtomicU64>>,
    next: Next,
) -> Response {
    let id = context.state().fetch_add(1, Ordering::Relaxed);
    context.extensions_mut().insert(RequestId(id));
    let mut response = next.run(context.into_request()).await;
    response.headers_mut().insert(
        "x-yetlet-request-id",
        HeaderValue::from_str(&id.to_string()).expect("integer header"),
    );
    response
}

pub(crate) async fn api_scope(mut context: RequestContext, next: Next) -> Response {
    context.extensions_mut().insert(ApiScope);
    let mut response = next.run(context.into_request()).await;
    response
        .headers_mut()
        .insert("x-yetlet-scope", HeaderValue::from_static("api"));
    response
}
