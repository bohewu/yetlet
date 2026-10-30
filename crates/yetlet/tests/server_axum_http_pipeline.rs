#![cfg(not(target_arch = "wasm32"))]
//! Real-router P0 contract tests. No replacement middleware test framework.
use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http_body::{Body as HttpBody, Frame};
use http_body_util::BodyExt;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    convert::Infallible,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};
use tower::{ServiceBuilder, ServiceExt};
use tower_http::set_header::SetResponseHeaderLayer;
use yetlet::server_axum::{
    HtmlPageError, RequestContext,
    middleware::{Next, from_fn, with_global},
    render_page,
};
use yetlet::view::{Element, View};

#[derive(Clone)]
struct Trace(Arc<Mutex<Vec<&'static str>>>);
#[derive(Clone)]
struct RequestValue(String);
fn record(ctx: &RequestContext, value: &'static str) {
    ctx.extensions()
        .get::<Trace>()
        .unwrap()
        .0
        .lock()
        .unwrap()
        .push(value);
}

async fn global(mut ctx: RequestContext, next: Next) -> Response {
    assert!(
        ctx.matched_path().is_none(),
        "application routing has not run"
    );
    let trace = Trace(Arc::new(Mutex::new(vec!["global.before"])));
    let value = ctx
        .headers()
        .get("x-test-value")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("request")
        .to_owned();
    ctx.extensions_mut().insert(trace.clone());
    ctx.extensions_mut().insert(RequestValue(value));
    ctx.headers_mut()
        .insert("x-injected", HeaderValue::from_static("present"));
    let mut response = next.run(ctx.into_request()).await;
    let mut events = trace.0.lock().unwrap();
    events.push("global.after");
    response
        .headers_mut()
        .insert("x-order", events.join(",").parse().unwrap());
    response
}
async fn global_two(ctx: RequestContext, next: Next) -> Response {
    assert!(ctx.matched_path().is_none());
    record(&ctx, "second.before");
    let trace = ctx.extensions().get::<Trace>().unwrap().clone();
    let response = next.run(ctx.into_request()).await;
    trace.0.lock().unwrap().push("second.after");
    response
}
async fn group(ctx: RequestContext, next: Next) -> Response {
    assert!(ctx.matched_path().is_some());
    record(&ctx, "group.before");
    let trace = ctx.extensions().get::<Trace>().unwrap().clone();
    let response = next.run(ctx.into_request()).await;
    trace.0.lock().unwrap().push("group.after");
    response
}
async fn nested(ctx: RequestContext, next: Next) -> Response {
    record(&ctx, "nested.before");
    let trace = ctx.extensions().get::<Trace>().unwrap().clone();
    let response = next.run(ctx.into_request()).await;
    trace.0.lock().unwrap().push("nested.after");
    response
}
async fn route(ctx: RequestContext, next: Next) -> Response {
    record(&ctx, "route.before");
    if ctx.headers().contains_key("x-stop") {
        return StatusCode::FORBIDDEN.into_response();
    }
    let trace = ctx.extensions().get::<Trace>().unwrap().clone();
    let mut response = next.run(ctx.into_request()).await;
    trace.0.lock().unwrap().push("route.after");
    response
        .headers_mut()
        .insert("x-route-after", HeaderValue::from_static("yes"));
    response
}
async fn endpoint(ctx: RequestContext) -> &'static str {
    record(&ctx, "handler");
    assert_eq!(ctx.headers()["x-injected"], "present");
    "ok"
}
fn fixture() -> Router {
    let inner = Router::new()
        .route("/{id}", get(endpoint).layer(from_fn(route)))
        .route_layer(from_fn(nested));
    let api = Router::new()
        .route("/plain", get(endpoint))
        .nest("/nested", inner)
        .route_layer(from_fn(group));
    let routes = Router::new()
        .route("/outside", get(endpoint))
        .nest("/api", api);
    with_global(
        routes,
        ServiceBuilder::new()
            .layer(from_fn(global))
            .layer(from_fn(global_two)),
    )
}
fn req(method: Method, path: &str) -> Request {
    Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap()
}
async fn text(response: Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 32 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn deterministic_order_and_reverse_unwind() {
    let response = fixture()
        .oneshot(req(Method::GET, "/api/nested/7"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["x-order"],
        "global.before,second.before,group.before,nested.before,route.before,handler,route.after,nested.after,group.after,second.after,global.after"
    );
    assert_eq!(response.headers()["x-route-after"], "yes");
}

#[tokio::test]
async fn group_scope_and_unmatched_paths_do_not_leak_layers() {
    for (path, expected) in [
        (
            "/outside",
            "global.before,second.before,handler,second.after,global.after",
        ),
        (
            "/api/plain",
            "global.before,second.before,group.before,handler,group.after,second.after,global.after",
        ),
        (
            "/api/missing",
            "global.before,second.before,second.after,global.after",
        ),
        (
            "/api/nested/missing/extra",
            "global.before,second.before,second.after,global.after",
        ),
    ] {
        let response = fixture().oneshot(req(Method::GET, path)).await.unwrap();
        assert_eq!(response.headers()["x-order"], expected);
        assert_eq!(
            response.status(),
            if path.contains("missing") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::OK
            }
        );
    }
}

#[tokio::test]
async fn method_mismatch_preserves_405_allow_and_scoped_unwind() {
    let response = fixture()
        .oneshot(req(Method::POST, "/api/nested/7"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        response.headers()["allow"]
            .to_str()
            .unwrap()
            .contains("GET")
    );
    assert_eq!(
        response.headers()["x-order"],
        "global.before,second.before,group.before,nested.before,route.before,route.after,nested.after,group.after,second.after,global.after"
    );
}

#[tokio::test]
async fn short_circuit_skips_handler_but_unwinds_entered_outer_layers() {
    let mut request = req(Method::GET, "/api/nested/7");
    request
        .headers_mut()
        .insert("x-stop", HeaderValue::from_static("yes"));
    let response = fixture().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.headers()["x-order"],
        "global.before,second.before,group.before,nested.before,route.before,nested.after,group.after,second.after,global.after"
    );
}

#[derive(Clone)]
struct Shared {
    name: &'static str,
}
#[derive(Deserialize)]
struct Params {
    id: u64,
}
#[derive(Deserialize)]
struct Search {
    q: String,
}
async fn inspect(mut ctx: RequestContext<Arc<Shared>>) -> Response {
    assert_eq!(ctx.method(), Method::GET);
    assert_eq!(ctx.path(), "/42");
    assert_eq!(ctx.original_uri().path(), "/inspect/42");
    assert_eq!(ctx.matched_path(), Some("/inspect/{id}"));
    let params: Params = match ctx.path_params().await {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let query: Search = match ctx.query() {
        Ok(q) => q,
        Err(e) => return e.into_response(),
    };
    let local = &ctx.extensions().get::<RequestValue>().unwrap().0;
    format!("{}:{local}:{}:{}", ctx.state().name, params.id, query.q).into_response()
}

#[tokio::test]
async fn typed_context_state_query_and_route_metadata() {
    let state = Arc::new(Shared {
        name: "application",
    });
    let routes = Router::new()
        .nest("/inspect", Router::new().route("/{id}", get(inspect)))
        .with_state(state);
    let response = with_global(routes, from_fn(global))
        .oneshot(req(Method::GET, "/inspect/42?q=a%20b"))
        .await
        .unwrap();
    assert_eq!(text(response).await, "application:request:42:a b");
}

#[tokio::test]
async fn invalid_query_and_path_errors_are_responses() {
    async fn decode(mut ctx: RequestContext) -> Response {
        let params: Params = match ctx.path_params().await {
            Ok(p) => p,
            Err(e) => return e.into_response(),
        };
        let query: Search = match ctx.query() {
            Ok(q) => q,
            Err(e) => return e.into_response(),
        };
        format!("{}:{}", params.id, query.q).into_response()
    }
    let routes = Router::new().route("/{id}", get(decode));
    for path in ["/not-a-number?q=x", "/1"] {
        let response = with_global(routes.clone(), from_fn(global))
            .oneshot(req(Method::GET, path))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()["x-order"], "global.before,global.after");
    }
}

#[tokio::test]
async fn global_can_rewrite_uri_before_application_routing() {
    async fn rewrite(mut ctx: RequestContext, next: Next) -> Response {
        assert!(ctx.matched_path().is_none());
        *ctx.uri_mut() = "/destination".parse().unwrap();
        next.run(ctx.into_request()).await
    }
    let app = with_global(
        Router::new().route("/destination", get(|| async { "rewritten" })),
        from_fn(rewrite),
    );
    assert_eq!(
        text(app.oneshot(req(Method::GET, "/original")).await.unwrap()).await,
        "rewritten"
    );
}

#[tokio::test]
async fn concurrent_requests_have_separate_extensions_and_shared_app_handle() {
    let barrier = Arc::new(tokio::sync::Barrier::new(12));
    async fn concurrent(ctx: RequestContext<Arc<tokio::sync::Barrier>>) -> String {
        let value = ctx.extensions().get::<RequestValue>().unwrap().0.clone();
        ctx.state().wait().await;
        tokio::task::yield_now().await;
        assert_eq!(value, ctx.extensions().get::<RequestValue>().unwrap().0);
        value
    }
    let routes = Router::new()
        .route("/", get(concurrent))
        .with_state(barrier);
    let app = with_global(routes, from_fn(global));
    let mut tasks = tokio::task::JoinSet::new();
    for id in 0..12 {
        let app = app.clone();
        tasks.spawn(async move {
            let mut request = req(Method::GET, "/");
            request
                .headers_mut()
                .insert("x-test-value", id.to_string().parse().unwrap());
            assert_eq!(
                text(app.oneshot(request).await.unwrap()).await,
                id.to_string()
            );
        });
    }
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
}

#[tokio::test]
async fn api_ssr_and_errors_use_identical_outer_pipeline() {
    async fn api(ctx: RequestContext) -> impl IntoResponse {
        record(&ctx, "handler");
        (StatusCode::UNPROCESSABLE_ENTITY, "typed-api-error")
    }
    async fn ssr(ctx: RequestContext) -> Result<Response, HtmlPageError> {
        record(&ctx, "handler");
        render_page("SSR", &View::text("content"))
    }
    async fn bad_ssr(ctx: RequestContext) -> Result<Response, HtmlPageError> {
        record(&ctx, "handler");
        render_page("SSR", &Element::new("script").into())
    }
    let routes = Router::new()
        .route("/api", get(api))
        .route("/ssr", get(ssr))
        .route("/bad-ssr", get(bad_ssr));
    let app = with_global(routes, from_fn(global));
    for (path, status) in [
        ("/api", StatusCode::UNPROCESSABLE_ENTITY),
        ("/ssr", StatusCode::OK),
        ("/bad-ssr", StatusCode::INTERNAL_SERVER_ERROR),
    ] {
        let response = app.clone().oneshot(req(Method::GET, path)).await.unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(
            response.headers()["x-order"],
            "global.before,handler,global.after"
        );
    }
}

#[tokio::test]
async fn existing_tower_http_layer_is_natively_composable() {
    let app = with_global(
        Router::new().route("/", get(|| async { "ok" })),
        ServiceBuilder::new()
            .layer(from_fn(global))
            .layer(SetResponseHeaderLayer::if_not_present(
                "x-third-party".parse().unwrap(),
                HeaderValue::from_static("tower-http"),
            )),
    );
    let response = app.oneshot(req(Method::GET, "/")).await.unwrap();
    assert_eq!(response.headers()["x-third-party"], "tower-http");
    assert_eq!(response.headers()["x-order"], "global.before,global.after");
}

struct CountedBody {
    polls: Arc<AtomicUsize>,
    frames: VecDeque<Result<Frame<Bytes>, std::io::Error>>,
}
impl HttpBody for CountedBody {
    type Data = Bytes;
    type Error = std::io::Error;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Poll::Ready(self.frames.pop_front())
    }
}
fn streaming_body(error: bool) -> (Body, Arc<AtomicUsize>) {
    let polls = Arc::new(AtomicUsize::new(0));
    let mut trailers = HeaderMap::new();
    trailers.insert("x-trailer", HeaderValue::from_static("preserved"));
    let mut frames = VecDeque::from([
        Ok(Frame::data(Bytes::from_static(b"first"))),
        Ok(Frame::data(Bytes::from_static(b"second"))),
    ]);
    frames.push_back(if error {
        Err(std::io::Error::other("source failed"))
    } else {
        Ok(Frame::trailers(trailers))
    });
    (
        Body::new(CountedBody {
            polls: polls.clone(),
            frames,
        }),
        polls,
    )
}

#[tokio::test]
async fn request_and_response_streams_are_not_eagerly_buffered() {
    async fn echo(ctx: RequestContext) -> Response {
        Response::new(ctx.into_body())
    }
    let app = with_global(Router::new().route("/echo", post(echo)), from_fn(global));
    let (body, polls) = streaming_body(false);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/echo")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        polls.load(Ordering::SeqCst),
        0,
        "middleware and handler must not pre-read a stream"
    );
    let collected = response.into_body().collect().await.unwrap();
    assert_eq!(collected.trailers().unwrap()["x-trailer"], "preserved");
    assert_eq!(collected.to_bytes(), "firstsecond");
    assert!(polls.load(Ordering::SeqCst) >= 3);
}

#[tokio::test]
async fn body_error_is_not_swallowed() {
    async fn echo(ctx: RequestContext) -> Response {
        Response::new(ctx.into_body())
    }
    let app = with_global(Router::new().route("/", post(echo)), from_fn(global));
    let (body, _) = streaming_body(true);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.into_body().collect().await.is_err());
}

#[tokio::test]
async fn short_circuit_does_not_consume_request_body() {
    async fn deny(_: RequestContext, _: Next) -> Response {
        StatusCode::FORBIDDEN.into_response()
    }
    let app = with_global(
        Router::new().route(
            "/",
            post(|| async {
                panic!("must not run");
                #[allow(unreachable_code)]
                StatusCode::OK
            }),
        ),
        from_fn(deny),
    );
    let (body, polls) = streaming_body(false);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(polls.load(Ordering::SeqCst), 0);
}

#[test]
fn context_is_request_only_and_preserves_original_parts() {
    fn is_send<T: Send>() {}
    is_send::<RequestContext<Arc<Shared>>>();
    let ctx = RequestContext::new(req(Method::POST, "/a?b=c"), ());
    assert!(ctx.matched_path().is_none());
    assert_eq!(ctx.uri().query(), Some("b=c"));
    let request = ctx.into_request();
    assert_eq!(request.method(), Method::POST);
    let _: Result<(), Infallible> = Ok(());
}
