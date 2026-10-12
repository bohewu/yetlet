//! Bounded YT-029 in-process native comparison; no listener or external service.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::Response,
    routing::get,
};
use tower::{ServiceBuilder, ServiceExt};
use yetlet_middleware_lab::deadline::with_deadline;
use yetlet_server_axum::{
    RequestContext,
    middleware::{Next, from_fn, with_global},
};

async fn context(context: RequestContext, next: Next) -> Response {
    next.run(context.into_request()).await
}

async fn batch(router: &Router, iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        let request = Request::builder()
            .uri("/bench")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 16).await.unwrap();
        assert_eq!(body, "ok");
        black_box(body);
    }
    start.elapsed()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    const ITERATIONS: usize = 100_000;
    const WARMUP: usize = 10_000;
    const SAMPLES: usize = 10;
    let baseline = Router::new().route("/bench", get(|| async { "ok" }));
    let variants = [
        ("baseline", baseline.clone()),
        (
            "global_envelope",
            with_global(baseline.clone(), ServiceBuilder::new()),
        ),
        (
            "request_context",
            with_global(baseline.clone(), from_fn(context)),
        ),
        ("deadline", with_deadline(baseline, Duration::from_secs(1))),
    ];
    println!(
        "# iterations={ITERATIONS},warmup_per_variant={WARMUP},samples={SAMPLES},runtime=current_thread,deadline_ms=1000"
    );
    println!(
        "# GET /bench,empty_request_body,2_byte_response_body,request_construct_clone_oneshot_collect_assert_included"
    );
    for (_, router) in &variants {
        batch(router, WARMUP).await;
    }
    println!("sample,variant,iterations,total_ns,ns_per_request");
    for sample in 0..SAMPLES {
        // Rotate the first variant to limit fixed-order thermal/scheduler bias.
        for offset in 0..variants.len() {
            let (name, router) = &variants[(sample + offset) % variants.len()];
            let elapsed = batch(router, ITERATIONS).await.as_nanos();
            println!(
                "{},{},{},{},{:.3}",
                sample + 1,
                name,
                ITERATIONS,
                elapsed,
                elapsed as f64 / ITERATIONS as f64
            );
        }
    }
}
