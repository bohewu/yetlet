//! Example-local API scope; request correlation is now an opt-in crate.
use axum::{http::HeaderValue, response::Response};
use yetlet::server_axum::{
    App, RequestContext,
    middleware::{Next, from_fn},
};

#[derive(Clone, Copy)]
pub(crate) struct ApiScope;

/// Plain setup function, composed explicitly with App::configure.
pub(crate) fn with_api_scope<S>(app: App<S>) -> App<S>
where
    S: Clone + Send + Sync + 'static,
{
    app.route_layer(from_fn(api_scope))
}

async fn api_scope(mut context: RequestContext, next: Next) -> Response {
    context.extensions_mut().insert(ApiScope);
    let mut response = next.run(context.into_request()).await;
    response
        .headers_mut()
        .insert("x-yetlet-scope", HeaderValue::from_static("api"));
    response
}
