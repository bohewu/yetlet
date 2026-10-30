//! YT-021: a fixed application document composed from lazily generated Views.
//! The supported renderer remains unchanged; each fragment uses its serializer.

use std::time::Duration;

use axum::{
    Router,
    body::{Body, Bytes},
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use futures_util::stream;
use yetlet::html::{RenderError, render_to_string};
use yetlet::view::{Element, View};

const PREFIX: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Streaming SSR research</title></head><body><main>";
const SUFFIX: &str = "</main></body></html>";
pub const CSP: &str =
    "default-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
pub const ROW_TEXT: &str = "<report> & \"quoted\" 'item'";

#[derive(Clone, Copy)]
pub struct Workload {
    pub rows: usize,
    pub text_repetitions: usize,
    /// Synthetic late data readiness; zero in the CPU-only comparisons.
    pub tail_delay: Duration,
    /// Deliberately invalid row used only for the error-boundary proof.
    pub invalid_row: Option<usize>,
}

fn heading() -> View {
    Element::new("h1")
        .child(View::text("Streaming SSR research"))
        .into()
}

fn row(workload: Workload, index: usize) -> View {
    let element = Element::new("li")
        .attribute("title", ROW_TEXT)
        .child(View::text(format!(
            "{index}: {}",
            ROW_TEXT.repeat(workload.text_repetitions)
        )));
    if workload.invalid_row == Some(index) {
        element.attribute("onclick", "forbidden").into()
    } else {
        element.into()
    }
}

/// The baseline builds the entire View and serializes it before returning HTML.
pub async fn full_html(workload: Workload) -> Result<String, RenderError> {
    if !workload.tail_delay.is_zero() {
        tokio::time::sleep(workload.tail_delay).await;
    }
    let tree: View = Element::new("main")
        .child(heading())
        .child(Element::new("ul").children((0..workload.rows).map(|index| row(workload, index))))
        .into();
    // The fixed document envelope is identical to the streaming envelope.
    let main = render_to_string(&tree)?;
    Ok(format!(
        "{}{}{}",
        PREFIX.strip_suffix("<main>").unwrap(),
        main,
        SUFFIX.strip_prefix("</main>").unwrap()
    ))
}

struct Fragments {
    workload: Workload,
    next_row: usize,
    first: Option<Bytes>,
    finished: bool,
    #[cfg(test)]
    tail_gate: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl Fragments {
    fn new(workload: Workload) -> Result<Self, RenderError> {
        // Only the initial heading is constructed/rendered before body polling.
        let first = Bytes::from(format!("{PREFIX}{}<ul>", render_to_string(&heading())?));
        Ok(Self {
            workload,
            next_row: 0,
            first: Some(first),
            finished: false,
            #[cfg(test)]
            tail_gate: None,
        })
    }

    fn into_body(self) -> Body {
        Body::from_stream(stream::try_unfold(self, |mut state| async move {
            if let Some(first) = state.first.take() {
                return Ok(Some((first, state)));
            }
            if state.finished {
                return Ok(None);
            }
            if state.next_row == 0 {
                #[cfg(test)]
                if let Some(gate) = state.tail_gate.take() {
                    gate.await
                        .expect("test releases the tail generation barrier");
                }
                if !state.workload.tail_delay.is_zero() {
                    tokio::time::sleep(state.workload.tail_delay).await;
                }
            }
            if state.next_row < state.workload.rows {
                // Build and serialize ONE row here, never a pre-rendered page.
                let html = render_to_string(&row(state.workload, state.next_row))?;
                state.next_row += 1;
                Ok::<_, RenderError>(Some((Bytes::from(html), state)))
            } else {
                state.finished = true;
                Ok(Some((Bytes::from(format!("</ul>{SUFFIX}")), state)))
            }
        }))
    }
}

fn response(status: StatusCode, body: Body) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CONTENT_SECURITY_POLICY, CSP)
        .header(header::CACHE_CONTROL, "no-store")
        .body(body)
        .unwrap()
}

pub async fn full_response(workload: Workload) -> Response {
    match full_html(workload).await {
        Ok(html) => response(StatusCode::OK, Body::from(html)),
        Err(_) => response(
            StatusCode::INTERNAL_SERVER_ERROR,
            Body::from("Render failed"),
        ),
    }
}

pub fn streaming_response(workload: Workload) -> Response {
    match Fragments::new(workload) {
        Ok(fragments) => response(StatusCode::OK, fragments.into_body()),
        Err(_) => response(
            StatusCode::INTERNAL_SERVER_ERROR,
            Body::from("Render failed"),
        ),
    }
}

/// Fixed research routes, with no user-controlled workload or general SSR API.
pub fn router(workload: Workload) -> Router {
    Router::new()
        .route("/full", get(move || full_response(workload)))
        .route(
            "/stream",
            get(move || async move { streaming_response(workload) }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use futures_util::FutureExt;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn workload(rows: usize) -> Workload {
        Workload {
            rows,
            text_repetitions: 2,
            tail_delay: Duration::ZERO,
            invalid_row: None,
        }
    }

    #[tokio::test]
    async fn streamed_bytes_equal_full_view_render_and_escape_untrusted_values() {
        for rows in [0, 1, 100] {
            let full = full_html(workload(rows)).await.unwrap();
            let streamed = to_bytes(streaming_response(workload(rows)).into_body(), usize::MAX)
                .await
                .unwrap();
            assert_eq!(streamed, full);
            if rows > 0 {
                assert!(full.contains("&lt;report&gt; &amp; &quot;quoted&quot; &#39;item&#39;"));
                assert!(!full.contains("<report>"));
            }
        }
    }

    #[tokio::test]
    async fn first_fragment_precedes_tail_generation_behind_deterministic_barrier() {
        let (release, gate) = tokio::sync::oneshot::channel();
        let mut fragments = Fragments::new(workload(1)).unwrap();
        assert_eq!(fragments.next_row, 0);
        fragments.tail_gate = Some(gate);
        let mut body = fragments.into_body();
        let first = tokio::time::timeout(Duration::from_secs(1), body.frame())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .into_data()
            .unwrap();
        assert!(first.ends_with(b"</h1><ul>"));
        assert!(!first.windows(3).any(|bytes| bytes == b"<li"));
        // Poll the tail while the barrier is closed: no row has been constructed.
        assert!(body.frame().now_or_never().is_none());
        release.send(()).unwrap();
        let rest = tokio::time::timeout(Duration::from_secs(1), body.collect())
            .await
            .unwrap()
            .unwrap()
            .to_bytes();
        assert_eq!(
            [first.as_ref(), rest.as_ref()].concat(),
            full_html(workload(1)).await.unwrap().as_bytes()
        );
    }

    #[tokio::test]
    async fn late_render_error_keeps_committed_status_and_headers_then_fails_body() {
        let mut input = workload(2);
        input.invalid_row = Some(1);
        let full = full_response(input).await;
        assert_eq!(full.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let mut streamed = streaming_response(input);
        assert_eq!(streamed.status(), StatusCode::OK);
        assert_eq!(streamed.headers()[header::CONTENT_SECURITY_POLICY], CSP);
        assert_eq!(streamed.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            streamed.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8"
        );
        let body = streamed.body_mut();
        assert!(body.frame().await.unwrap().unwrap().is_data());
        assert!(body.frame().await.unwrap().unwrap().is_data());
        let error = body.frame().await.unwrap().unwrap_err();
        assert!(error.to_string().contains("forbidden HTML attribute"));
        assert_eq!(streamed.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn axum_routes_keep_equivalent_html_and_strict_csp_without_browser_assets() {
        for path in ["/full", "/stream"] {
            let response = router(workload(3))
                .oneshot(
                    axum::http::Request::builder()
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[header::CONTENT_SECURITY_POLICY], CSP);
            let html = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(html, full_html(workload(3)).await.unwrap());
            assert!(!html.windows(7).any(|bytes| bytes == b"<script"));
        }
    }
}
