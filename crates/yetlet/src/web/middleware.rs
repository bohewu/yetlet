//! Native Axum/Tower middleware composition, not a second service ecosystem.

use axum::{Router, extract::Request, response::IntoResponse};
use std::convert::Infallible;
use tower::{Layer, Service};

pub use axum::middleware::{Next, from_fn, from_fn_with_state};

/// Wrap the completed application Router before its route matching.
///
/// Unlike Router::layer, this observes unmatched paths and can rewrite a URI
/// before the application matches it. The fallback-only outer Router preserves
/// the existing Router/axum::serve API; do not add routes to that outer wrapper.
/// Compose multiple middleware with tower::ServiceBuilder (first declared is
/// outermost). For groups use Router::route_layer after registering routes;
/// for individual path/method routers use MethodRouter::layer.
///
/// Service errors must already be converted into responses, e.g. by Axum's
/// HandleErrorLayer. There is no panic swallowing or stream buffering here.
pub fn with_global<L>(router: Router, layer: L) -> Router
where
    L: Layer<Router>,
    L::Service: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
{
    Router::new().fallback_service(layer.layer(router))
}
