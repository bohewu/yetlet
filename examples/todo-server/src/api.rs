//! Explicit HTTP transport adapter for typed Todo commands.
//!
//! The domain transition is shared with the browser-local executor. No DOM
//! event or persistent server circuit is ever serialized.

use std::sync::{Arc, Mutex};

use axum::{
    Json,
    body::to_bytes,
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use yetlet_actions::{ActionError, ActionExecutor, ApiErrorBody, ApiErrorEnvelope, LocalExecutor};
use yetlet_server_axum::{RequestContext, middleware::Next};
use yetlet_todo_ui::{TodoCommand, TodoSnapshot, apply_todo_command};

const MAX_ACTION_BODY: usize = 4 * 1024;

#[derive(Clone)]
pub struct TodoApiState {
    snapshot: Arc<Mutex<TodoSnapshot>>,
}

impl Default for TodoApiState {
    fn default() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(TodoSnapshot::initial())),
        }
    }
}

fn response_json<T: serde::Serialize>(status: StatusCode, body: T) -> Response {
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store".parse().expect("static header"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().expect("static header"),
    );
    response
}

fn api_error(status: StatusCode, code: &str, message: &str) -> Response {
    response_json(
        status,
        ApiErrorEnvelope {
            error: ApiErrorBody {
                code: code.to_owned(),
                message: message.to_owned(),
            },
        },
    )
}

fn command_error(error: ActionError) -> Response {
    let (status, message) = match error {
        ActionError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "Invalid command fields"),
        ActionError::NotFound => (StatusCode::NOT_FOUND, "Todo item does not exist"),
        ActionError::CapacityReached => (StatusCode::CONFLICT, "Todo capacity reached"),
        ActionError::RevisionOverflow => (StatusCode::CONFLICT, "Revision limit reached"),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to complete request",
        ),
    };
    api_error(status, error.code(), message)
}

pub async fn list_todos(context: RequestContext<TodoApiState>) -> Response {
    match context.state().snapshot.lock() {
        Ok(snapshot) => response_json(StatusCode::OK, snapshot.clone()),
        Err(_) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Unable to read Todo state",
        ),
    }
}

/// Route-level, header-only guard. It never consumes the streaming body.
/// Origin is not authentication; preserve the existing direct-HTTP demo policy.
pub async fn action_headers(context: RequestContext<TodoApiState>, next: Next) -> Response {
    if context.method() == Method::POST {
        if !same_origin_or_no_origin(context.headers()) {
            return api_error(
                StatusCode::FORBIDDEN,
                "forbidden_origin",
                "Cross-origin action is not permitted",
            );
        }
        if !is_json_content_type(context.headers()) {
            return api_error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "Expected application/json",
            );
        }
    }
    next.run(context.into_request()).await
}

pub async fn mutate_todos(context: RequestContext<TodoApiState>) -> Response {
    let state = context.state().clone();
    // The handler explicitly chooses the existing 4 KiB JSON limit. There is
    // no framework-wide body buffer and ownership is consumed exactly once.
    let bytes = match to_bytes(context.into_body(), MAX_ACTION_BODY).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return api_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "Command body exceeds size limit",
            );
        }
    };
    let command: TodoCommand = match serde_json::from_slice(&bytes) {
        Ok(command) => command,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                "Invalid typed Todo command",
            );
        }
    };

    // Inject a local state handler behind the same ActionExecutor interface
    // used by the browser's explicit HTTP executor.
    let executor = LocalExecutor::new(move |command: TodoCommand| {
        let mut snapshot = state.snapshot.lock().map_err(|_| ActionError::Internal)?;
        apply_todo_command(&mut snapshot, command)
    });
    match executor.execute(command).await {
        Ok(snapshot) => response_json(StatusCode::OK, snapshot),
        Err(error) => command_error(error),
    }
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|content| content.split(';').next())
        .is_some_and(|content| content.trim().eq_ignore_ascii_case("application/json"))
}

fn same_origin_or_no_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        // HTTP clients may omit Origin. Origin is not an authentication secret.
        return true;
    };
    let (Some(origin), Some(host)) = (
        origin.to_str().ok(),
        headers
            .get(header::HOST)
            .and_then(|value| value.to_str().ok()),
    ) else {
        return false;
    };
    // The example runs directly over HTTP. HTTPS reverse-proxy deployment is
    // outside this demo and requires a trusted proxy/scheme configuration.
    origin == format!("http://{host}")
}

#[cfg(test)]
mod tests {
    use crate::app;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use yetlet_todo_ui::TodoSnapshot;

    async fn fetch(router: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = to_bytes(response.into_body(), 32 * 1024).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    fn post(body: Value) -> Request<Body> {
        Request::builder()
            .uri("/api/todos")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn get() -> Request<Body> {
        Request::builder()
            .uri("/api/todos")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn get_create_toggle_and_read_share_one_store() {
        let router = app();
        let (status, initial) = fetch(router.clone(), get()).await;
        assert_eq!(status, StatusCode::OK);
        let initial: TodoSnapshot = serde_json::from_value(initial).unwrap();
        assert_eq!(initial.revision, 0);
        assert_eq!(initial.items.len(), 2);

        let (status, created) = fetch(
            router.clone(),
            post(json!({"type":"create","title":"Server saved task"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let created: TodoSnapshot = serde_json::from_value(created).unwrap();
        assert_eq!(created.revision, 1);
        assert_eq!(created.items[2].id, 3);
        assert_eq!(created.items[2].title, "Server saved task");

        let (status, toggled) = fetch(router.clone(), post(json!({"type":"toggle","id":3}))).await;
        assert_eq!(status, StatusCode::OK);
        let toggled: TodoSnapshot = serde_json::from_value(toggled).unwrap();
        assert_eq!(toggled.revision, 2);
        assert!(toggled.items[2].completed);
        let (_, after) = fetch(router, get()).await;
        assert_eq!(after["revision"], 2);
        assert_eq!(after["items"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn invalid_commands_have_typed_errors_and_preserve_state() {
        let router = app();
        for (request, expected_status, expected_code) in [
            (
                json!({"type":"create","title":" "}),
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_title",
            ),
            (
                json!({"type":"toggle","id":999}),
                StatusCode::NOT_FOUND,
                "not_found",
            ),
            (
                json!({"type":"create","title":"ok","event":"click"}),
                StatusCode::BAD_REQUEST,
                "invalid_json",
            ),
            (
                json!({"type":"unknown"}),
                StatusCode::BAD_REQUEST,
                "invalid_json",
            ),
        ] {
            let (status, body) = fetch(router.clone(), post(request)).await;
            assert_eq!(status, expected_status);
            assert_eq!(body["error"]["code"], expected_code);
            assert!(body["error"]["message"].as_str().unwrap().len() < 80);
        }
        let (_, snapshot) = fetch(router, get()).await;
        assert_eq!(snapshot["revision"], 0);
    }

    #[tokio::test]
    async fn rejects_wrong_origin_media_type_and_oversized_payload() {
        let router = app();
        let (status, error) = fetch(
            router.clone(),
            Request::builder()
                .uri("/api/todos")
                .method("POST")
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(error["error"]["code"], "unsupported_media_type");

        let (status, error) = fetch(
            router.clone(),
            Request::builder()
                .uri("/api/todos")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::HOST, "localhost:3000")
                .header(header::ORIGIN, "https://other.example")
                .body(Body::from(r#"{"type":"create","title":"attack"}"#))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error["error"]["code"], "forbidden_origin");

        let (status, error) = fetch(
            router,
            Request::builder()
                .uri("/api/todos")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("x".repeat(5000)))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(error["error"]["code"], "payload_too_large");
    }
}
