//! Runnable middleware lab using native Axum and Tower composition.
//!
//! Pre-routing global, group and method-route scopes are intentionally visible.
//! No second HTTP pipeline, hidden Session or global request state is installed.

pub mod deadline;

use std::{
    io,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::{Body, Bytes},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use futures_util::stream;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use yetlet_server_axum::{
    App, RequestContext,
    middleware::{Next, from_fn, with_global},
};

#[derive(Clone)]
struct AppState {
    service: &'static str,
}

#[derive(Clone)]
struct RequestLabel(String);

#[derive(Clone)]
struct Trace(Arc<Mutex<Vec<&'static str>>>);

impl Trace {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(vec!["global.before"])))
    }

    fn record(&self, event: &'static str) {
        self.0.lock().expect("trace lock").push(event);
    }

    fn summary(&self) -> String {
        self.0.lock().expect("trace lock").join(",")
    }
}

fn trace<S>(context: &RequestContext<S>, event: &'static str) -> Trace {
    let trace = context
        .extensions()
        .get::<Trace>()
        .expect("global trace")
        .clone();
    trace.record(event);
    trace
}

async fn global_first(mut context: RequestContext, next: Next) -> Response {
    // Global scope is entered before application routing: no MatchedPath yet.
    debug_assert!(context.matched_path().is_none());
    let trace = Trace::new();
    let requested = context
        .headers()
        .get("x-lab-user")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("guest");
    let label = if requested.len() <= 32
        && requested
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        requested.to_owned()
    } else {
        "guest".to_owned()
    };
    context.extensions_mut().insert(RequestLabel(label));
    context.extensions_mut().insert(trace.clone());
    let mut response = next.run(context.into_request()).await;
    trace.record("global.after");
    response.headers_mut().insert(
        "x-mw-order",
        HeaderValue::from_str(&trace.summary()).expect("static trace markers"),
    );
    response
}

async fn global_second(context: RequestContext, next: Next) -> Response {
    let trace = trace(&context, "second.before");
    let response = next.run(context.into_request()).await;
    trace.record("second.after");
    response
}

async fn group_scope(context: RequestContext, next: Next) -> Response {
    let trace = trace(&context, "group.before");
    let response = next.run(context.into_request()).await;
    trace.record("group.after");
    response
}

async fn route_guard(context: RequestContext, next: Next) -> Response {
    let trace = trace(&context, "route.before");
    if context.headers().contains_key("x-stop") {
        return StatusCode::FORBIDDEN.into_response();
    }
    let response = next.run(context.into_request()).await;
    trace.record("route.after");
    response
}

async fn outside(context: RequestContext<Arc<AppState>>) -> String {
    trace(&context, "handler");
    format!("Outside: {}", context.state().service)
}

async fn handled(context: RequestContext<Arc<AppState>>) -> String {
    trace(&context, "handler");
    let name = &context
        .extensions()
        .get::<RequestLabel>()
        .expect("typed request label")
        .0;
    // The state is shared application state; RequestLabel is request-local.
    format!("Service: {}; Request: {name}", context.state().service)
}

fn stream_response(fail: bool) -> Response {
    let chunks: Vec<Result<Bytes, io::Error>> = if fail {
        vec![
            Ok(Bytes::from_static(b"first|")),
            Err(io::Error::other("expected lab stream failure")),
        ]
    } else {
        vec![
            Ok(Bytes::from_static(b"first|")),
            Ok(Bytes::from_static(b"second")),
        ]
    };
    let mut response = Response::new(Body::from_stream(stream::iter(chunks)));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}

async fn stream_ok(context: RequestContext<Arc<AppState>>) -> Response {
    trace(&context, "handler");
    stream_response(false)
}

async fn stream_failure(context: RequestContext<Arc<AppState>>) -> Response {
    trace(&context, "handler");
    stream_response(true)
}

/// Compose a completed application and then wrap it in pre-routing middleware.
///
/// No route may be appended after with_global: that outer fallback-only router
/// is the boundary that observes 404s, 405s and every registered endpoint.
pub fn app() -> Router {
    let state = Arc::new(AppState { service: "Yetlet" });
    let routes = App::<Arc<AppState>>::new()
        .get("/outside", outside)
        .group("/lab", |group| {
            group
                .get("/plain", handled)
                .route("/guarded", get(handled).layer(from_fn(route_guard)))
                .get("/stream", stream_ok)
                .get("/broken-stream", stream_failure)
                .route_layer(from_fn(group_scope))
        })
        .with_state(state)
        .into_router();

    with_global(
        routes,
        ServiceBuilder::new()
            .layer(from_fn(global_first))
            .layer(from_fn(global_second))
            .layer(SetResponseHeaderLayer::if_not_present(
                "x-native-tower".parse().expect("static header name"),
                HeaderValue::from_static("tower-http"),
            )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::{HeaderMap, Method, Request},
    };
    use http_body::{Body as HttpBody, Frame};
    use std::{
        pin::Pin,
        sync::atomic::{AtomicUsize, Ordering},
        task::{Context, Poll},
    };
    use tower::ServiceExt;

    fn request(method: Method, path: &str, user: Option<&str>, stop: bool) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(user) = user {
            builder = builder.header("x-lab-user", user);
        }
        if stop {
            builder = builder.header("x-stop", "1");
        }
        builder.body(Body::empty()).unwrap()
    }

    async fn send(req: Request<Body>) -> (StatusCode, HeaderMap, Body) {
        let response = app().oneshot(req).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        (status, headers, response.into_body())
    }

    fn order(headers: &HeaderMap) -> &str {
        headers["x-mw-order"].to_str().unwrap()
    }

    #[tokio::test]
    async fn global_group_and_route_unwind_in_reverse_order() {
        let (status, headers, body) =
            send(request(Method::GET, "/lab/guarded", Some("alpha"), false)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            order(&headers),
            "global.before,second.before,group.before,route.before,handler,route.after,group.after,second.after,global.after"
        );
        assert_eq!(headers["x-native-tower"], "tower-http");
        assert_eq!(
            to_bytes(body, 1024).await.unwrap(),
            "Service: Yetlet; Request: alpha"
        );
    }

    #[tokio::test]
    async fn short_circuit_skips_handler_and_unwinds_entered_outer_scopes() {
        let (status, headers, body) =
            send(request(Method::GET, "/lab/guarded", Some("alpha"), true)).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(
            order(&headers),
            "global.before,second.before,group.before,route.before,group.after,second.after,global.after"
        );
        assert_eq!(to_bytes(body, 1024).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn unmatched_404_and_matched_405_keep_native_semantics() {
        let (status, headers, _) = send(request(Method::GET, "/lab/unknown", None, false)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            order(&headers),
            "global.before,second.before,second.after,global.after"
        );

        let (status, headers, _) = send(request(Method::POST, "/lab/guarded", None, false)).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(headers[header::ALLOW].to_str().unwrap().contains("GET"));
        assert_eq!(
            order(&headers),
            "global.before,second.before,group.before,route.before,route.after,group.after,second.after,global.after"
        );
    }

    #[tokio::test]
    async fn scopes_do_not_leak_and_request_extensions_are_isolated() {
        let (status, headers, _) = send(request(Method::GET, "/outside", Some("one"), false)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            order(&headers),
            "global.before,second.before,handler,second.after,global.after"
        );
        let (_, headers, body) = send(request(Method::GET, "/lab/plain", Some("two"), false)).await;
        assert_eq!(
            order(&headers),
            "global.before,second.before,group.before,handler,group.after,second.after,global.after"
        );
        assert_eq!(
            to_bytes(body, 1024).await.unwrap(),
            "Service: Yetlet; Request: two"
        );
        let (_, _, body) = send(request(Method::GET, "/lab/plain", None, false)).await;
        assert_eq!(
            to_bytes(body, 1024).await.unwrap(),
            "Service: Yetlet; Request: guest"
        );
    }

    #[tokio::test]
    async fn native_tower_headers_and_body_streaming_are_preserved() {
        let (status, headers, body) = send(request(Method::GET, "/lab/stream", None, false)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers["x-native-tower"], "tower-http");
        assert_eq!(to_bytes(body, 2048).await.unwrap(), "first|second");

        let (status, headers, body) =
            send(request(Method::GET, "/lab/broken-stream", None, false)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(order(&headers).ends_with("group.after,second.after,global.after"));
        // The 200 response headers were returned before this body error. The
        // middleware must not swallow, convert or prebuffer the failed stream.
        assert!(to_bytes(body, 2048).await.is_err());
    }

    struct CountedBody(Arc<AtomicUsize>);

    impl HttpBody for CountedBody {
        type Data = Bytes;
        type Error = io::Error;
        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(None)
        }
    }

    #[tokio::test]
    async fn short_circuit_does_not_poll_request_body() {
        let polls = Arc::new(AtomicUsize::new(0));
        let req = Request::builder()
            .method(Method::GET)
            .uri("/lab/guarded")
            .header("x-stop", "1")
            .body(Body::new(CountedBody(Arc::clone(&polls))))
            .unwrap();
        let (status, _, _) = send(req).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(polls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn bounded_request_label_rejects_untrusted_header_text() {
        let (_, _, body) = send(request(Method::GET, "/lab/plain", Some("a b"), false)).await;
        assert_eq!(
            to_bytes(body, 1024).await.unwrap(),
            "Service: Yetlet; Request: guest"
        );
    }
}
