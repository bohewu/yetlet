//! Private allowlisted application diagnostics; no request/body Debug capture.
use crate::Failure;
use axum::{
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use std::time::Instant;
use tracing::{Dispatch, Instrument, Span};
use yetlet::request_id::{RequestId, RequestIds, assign_request_id};
use yetlet::server_axum::middleware::with_global;

pub const TARGET: &str = "yetlet_lab_operations_board::request_tracing";
const HTTP_SPAN: &str = "board_request";

pub(crate) fn install(app: Router) -> Router {
    let traced = with_global(app, middleware::from_fn(observe));
    with_global(
        traced,
        middleware::from_fn_with_state(RequestIds::new(), assign_request_id),
    )
}

async fn observe(request: Request, next: Next) -> Response {
    let Some(RequestId(request_id)) = request
        .extensions()
        .get::<RequestId>()
        .copied()
        .filter(|id| id.0 > 0)
    else {
        return next.run(request).await;
    };
    let method = match request.method().as_str() {
        "GET" => "GET",
        "HEAD" => "HEAD",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    };
    let span = tracing::info_span!(target:TARGET,parent:None,"board_request",request_id,method);
    let safe_span = span.clone();
    async move {
        let mut completion = HttpCompletion {
            span: safe_span,
            dispatch: tracing::dispatcher::get_default(Clone::clone),
            started: Instant::now(),
            status: None,
        };
        let response = next.run(request).await;
        completion.status = Some(response.status().as_u16());
        response
    }
    .instrument(span)
    .await
}

struct HttpCompletion {
    span: Span,
    dispatch: Dispatch,
    started: Instant,
    status: Option<u16>,
}
impl Drop for HttpCompletion {
    fn drop(&mut self) {
        let duration_ms = self.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        tracing::dispatcher::with_default(&self.dispatch, || {
            self.span.in_scope(|| {
            if let Some(status) = self.status {
                tracing::info!(target:TARGET,event="http_response_constructed",status,duration_ms);
            } else {
                tracing::info!(target:TARGET,event="http_future_dropped",duration_ms);
            }
        })
        });
    }
}

pub(crate) struct WorkerTrace {
    parent: Span,
    dispatch: Dispatch,
}
impl WorkerTrace {
    pub(crate) fn capture(enabled: bool) -> Option<Self> {
        if !enabled {
            return None;
        }
        let parent = Span::current();
        let metadata = parent.metadata()?;
        if metadata.target() != TARGET || metadata.name() != HTTP_SPAN {
            return None;
        }
        Some(Self {
            parent,
            dispatch: tracing::dispatcher::get_default(Clone::clone),
        })
    }
    pub(crate) fn run<T>(
        self,
        operation: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        tracing::dispatcher::with_default(&self.dispatch, || {
            let span = tracing::info_span!(target:TARGET,parent:&self.parent,"sql_worker",operation="database");
            span.in_scope(|| {
                tracing::info!(target:TARGET,event="sql_worker_started");
                let result=operation();
                tracing::info!(target:TARGET,event="sql_worker_completed",result=if result.is_ok(){"ok"}else{"error"});
                result
            })
        })
    }
}
