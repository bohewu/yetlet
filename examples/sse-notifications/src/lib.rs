//! Native HTTP Server-Sent Events with bounded replay and subscriber queues.
//!
//! This is application-owned state, not a Yetlet-wide event circuit. Clients
//! explicitly connect to one HTTP endpoint and reconnect with Last-Event-ID.

use std::{
    collections::{BTreeMap, VecDeque},
    convert::Infallible,
    fmt, io,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, Mutex, Weak},
    task::{Context, Poll},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures_util::{Stream, StreamExt, stream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use yetlet::server_axum::App;

const REPLAY_CAPACITY: usize = 16;
const SUBSCRIBER_QUEUE_CAPACITY: usize = 4;
const MAX_SUBSCRIBERS: usize = 8;
const MAX_JSON_BODY: usize = 2 * 1024;
const MAX_MESSAGE_CHARS: usize = 200;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Notification {
    pub id: u64,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublishError {
    IdExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubscribeError {
    Capacity,
    FutureCursor,
    ReplayGap,
}

#[derive(Clone)]
pub struct Hub {
    inner: Arc<Mutex<HubInner>>,
}

struct HubInner {
    next_event_id: u64,
    next_subscriber_id: u64,
    replay: VecDeque<Notification>,
    subscribers: BTreeMap<u64, mpsc::Sender<Notification>>,
    lagged_disconnects: u64,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HubInner {
                next_event_id: 1,
                next_subscriber_id: 1,
                replay: VecDeque::with_capacity(REPLAY_CAPACITY),
                subscribers: BTreeMap::new(),
                lagged_disconnects: 0,
            })),
        }
    }

    fn publish(&self, message: String) -> Result<Notification, PublishError> {
        let mut inner = self.inner.lock().expect("SSE hub lock");
        let following = inner
            .next_event_id
            .checked_add(1)
            .ok_or(PublishError::IdExhausted)?;
        let notification = Notification {
            id: inner.next_event_id,
            message,
        };
        inner.next_event_id = following;
        inner.replay.push_back(notification.clone());
        while inner.replay.len() > REPLAY_CAPACITY {
            inner.replay.pop_front();
        }

        let mut remove = Vec::new();
        let mut lagged = 0_u64;
        for (&subscriber_id, sender) in &inner.subscribers {
            match sender.try_send(notification.clone()) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    remove.push(subscriber_id);
                    lagged = lagged.saturating_add(1);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => remove.push(subscriber_id),
            }
        }
        for subscriber_id in remove {
            inner.subscribers.remove(&subscriber_id);
        }
        inner.lagged_disconnects = inner.lagged_disconnects.saturating_add(lagged);
        Ok(notification)
    }

    fn subscribe(
        &self,
        last_event_id: Option<u64>,
    ) -> Result<(Vec<Notification>, LiveStream), SubscribeError> {
        let mut inner = self.inner.lock().expect("SSE hub lock");
        if inner.subscribers.len() >= MAX_SUBSCRIBERS {
            return Err(SubscribeError::Capacity);
        }
        let latest = inner.next_event_id.saturating_sub(1);
        let replay = if let Some(cursor) = last_event_id {
            if cursor > latest {
                return Err(SubscribeError::FutureCursor);
            }
            if let Some(first) = inner.replay.front()
                && cursor < first.id.saturating_sub(1)
            {
                return Err(SubscribeError::ReplayGap);
            }
            inner
                .replay
                .iter()
                .filter(|notification| notification.id > cursor)
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let subscriber_id = inner.next_subscriber_id;
        inner.next_subscriber_id = inner.next_subscriber_id.saturating_add(1);
        let (sender, receiver) = mpsc::channel(SUBSCRIBER_QUEUE_CAPACITY);
        inner.subscribers.insert(subscriber_id, sender);
        Ok((
            replay,
            LiveStream {
                receiver,
                subscriber_id,
                hub: Arc::downgrade(&self.inner),
            },
        ))
    }

    fn metrics(&self) -> Metrics {
        let inner = self.inner.lock().expect("SSE hub lock");
        Metrics {
            active_subscribers: inner.subscribers.len(),
            retained_events: inner.replay.len(),
            latest_id: inner.next_event_id.saturating_sub(1),
            lagged_disconnects: inner.lagged_disconnects,
            subscriber_queue_capacity: SUBSCRIBER_QUEUE_CAPACITY,
            replay_capacity: REPLAY_CAPACITY,
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

struct LiveStream {
    receiver: mpsc::Receiver<Notification>,
    subscriber_id: u64,
    hub: Weak<Mutex<HubInner>>,
}

impl Stream for LiveStream {
    type Item = Notification;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

impl Drop for LiveStream {
    fn drop(&mut self) {
        if let Some(hub) = self.hub.upgrade()
            && let Ok(mut inner) = hub.lock()
        {
            inner.subscribers.remove(&self.subscriber_id);
        }
    }
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct Metrics {
    active_subscribers: usize,
    retained_events: usize,
    latest_id: u64,
    lagged_disconnects: u64,
    subscriber_queue_capacity: usize,
    replay_capacity: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateNotification {
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct ApiFailure {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiFailure {
    const fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
        }
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        json_response(
            self.status,
            ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                },
            },
        )
    }
}

fn apply_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}

fn json_response<T: Serialize>(status: StatusCode, value: T) -> Response {
    let mut response = (status, Json(value)).into_response();
    apply_headers(&mut response);
    response
}

fn parse_json(
    payload: Result<Json<CreateNotification>, JsonRejection>,
) -> Result<String, ApiFailure> {
    let Json(payload) = match payload {
        Ok(value) => value,
        Err(JsonRejection::BytesRejection(_)) => {
            return Err(ApiFailure::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "JSON body exceeds the 2 KiB limit",
            ));
        }
        Err(_) => {
            return Err(ApiFailure::new(
                StatusCode::BAD_REQUEST,
                "invalid_json",
                "Request body does not match the JSON schema",
            ));
        }
    };
    if payload.message.chars().any(char::is_control) {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_message",
            "Message must contain 1 to 200 non-control characters",
        ));
    }
    let message = payload.message.trim().to_owned();
    if message.is_empty() || message.chars().count() > MAX_MESSAGE_CHARS {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_message",
            "Message must contain 1 to 200 non-control characters",
        ));
    }
    Ok(message)
}

fn single_header<'a>(
    headers: &'a HeaderMap,
    name: &header::HeaderName,
) -> Result<Option<&'a str>, ()> {
    let mut values = headers.get_all(name).iter();
    let Some(first) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(());
    }
    first.to_str().map(Some).map_err(|_| ())
}

fn parse_last_event_id(headers: &HeaderMap) -> Result<Option<u64>, ApiFailure> {
    let name = header::HeaderName::from_static("last-event-id");
    let value = single_header(headers, &name).map_err(|_| {
        ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_last_event_id",
            "Last-Event-ID must appear at most once as an unsigned integer",
        )
    })?;
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_last_event_id",
            "Last-Event-ID must appear at most once as an unsigned integer",
        ));
    }
    value.parse().map(Some).map_err(|_| {
        ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_last_event_id",
            "Last-Event-ID must appear at most once as an unsigned integer",
        )
    })
}

fn reject_browser_origin(headers: &HeaderMap) -> Result<(), ApiFailure> {
    if headers.get_all(header::ORIGIN).iter().next().is_some() {
        return Err(ApiFailure::new(
            StatusCode::FORBIDDEN,
            "browser_origin_forbidden",
            "This local command-oriented mutation endpoint rejects Origin",
        ));
    }
    Ok(())
}

fn require_json_content_type(headers: &HeaderMap) -> Result<(), ApiFailure> {
    let value = single_header(headers, &header::CONTENT_TYPE).map_err(|_| {
        ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Exactly one application/json Content-Type is required",
        )
    })?;
    let valid = value
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"));
    if !valid {
        return Err(ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Exactly one application/json Content-Type is required",
        ));
    }
    Ok(())
}

fn notification_event(notification: Notification) -> Event {
    Event::default()
        .event("notification")
        .id(notification.id.to_string())
        .json_data(notification)
        .expect("Notification JSON serialization is infallible")
}

async fn health() -> Response {
    json_response(StatusCode::OK, Health { status: "ok" })
}

async fn metrics(State(hub): State<Hub>) -> Response {
    json_response(StatusCode::OK, hub.metrics())
}

async fn events(State(hub): State<Hub>, headers: HeaderMap) -> Response {
    let last_event_id = match parse_last_event_id(&headers) {
        Ok(value) => value,
        Err(error) => return error.into_response(),
    };
    let (replay, live) = match hub.subscribe(last_event_id) {
        Ok(subscription) => subscription,
        Err(SubscribeError::Capacity) => {
            return ApiFailure::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "subscriber_capacity",
                "Subscriber capacity is currently full",
            )
            .into_response();
        }
        Err(SubscribeError::FutureCursor) => {
            return ApiFailure::new(
                StatusCode::CONFLICT,
                "future_cursor",
                "Last-Event-ID is newer than the current stream",
            )
            .into_response();
        }
        Err(SubscribeError::ReplayGap) => {
            return ApiFailure::new(
                StatusCode::CONFLICT,
                "replay_gap",
                "Last-Event-ID is older than the retained replay window",
            )
            .into_response();
        }
    };

    let stream = stream::iter(replay)
        .chain(live)
        .map(|notification| Ok::<Event, Infallible>(notification_event(notification)));
    let mut response = Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(HEARTBEAT_INTERVAL)
                .text("heartbeat"),
        )
        .into_response();
    apply_headers(&mut response);
    response.headers_mut().insert(
        header::HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    response
}

async fn publish(
    State(hub): State<Hub>,
    headers: HeaderMap,
    payload: Result<Json<CreateNotification>, JsonRejection>,
) -> Response {
    if let Err(error) = reject_browser_origin(&headers) {
        return error.into_response();
    }
    if let Err(error) = require_json_content_type(&headers) {
        return error.into_response();
    }
    let message = match parse_json(payload) {
        Ok(message) => message,
        Err(error) => return error.into_response(),
    };
    match hub.publish(message) {
        Ok(notification) => json_response(StatusCode::CREATED, notification),
        Err(PublishError::IdExhausted) => ApiFailure::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "event_id_exhausted",
            "The finite event ID space is exhausted",
        )
        .into_response(),
    }
}

/// The anonymous local demonstration may not bind publicly.
pub fn listen_address(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the anonymous SSE sample must bind a loopback address",
        ));
    }
    Ok(address)
}

pub fn app(hub: Hub) -> Router {
    App::<Hub>::new()
        .get("/healthz", health)
        .get("/metrics", metrics)
        .get("/events", events)
        .post("/notifications", publish)
        .route_layer(DefaultBodyLimit::max(MAX_JSON_BODY))
        .with_state(hub)
        .into_router()
}

impl fmt::Debug for Hub {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Hub").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    async fn request(
        hub: Hub,
        method: Method,
        uri: &str,
        body: Option<Value>,
        headers: &[(&str, &str)],
    ) -> Response {
        let mut builder = Request::builder().method(method).uri(uri);
        let payload = if let Some(body) = body {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            serde_json::to_vec(&body).unwrap()
        } else {
            Vec::new()
        };
        let mut request = builder.body(Body::from(payload)).unwrap();
        for (name, value) in headers {
            request.headers_mut().append(
                header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        app(hub).oneshot(request).await.unwrap()
    }

    async fn json_body(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn loopback_only_listener() {
        assert!(listen_address("127.0.0.1:3008").is_ok());
        assert!(listen_address("[::1]:3008").is_ok());
        assert!(listen_address("0.0.0.0:3008").is_err());
        assert!(listen_address("192.0.2.1:3008").is_err());
    }

    #[tokio::test]
    async fn replay_is_ordered_and_new_subscribers_do_not_receive_history() {
        let hub = Hub::new();
        hub.publish("one".to_owned()).unwrap();
        hub.publish("two".to_owned()).unwrap();
        let (replay, live) = hub.subscribe(Some(0)).unwrap();
        assert_eq!(
            replay.iter().map(|value| value.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
        drop(live);
        let (replay, live) = hub.subscribe(None).unwrap();
        assert!(replay.is_empty());
        drop(live);
        assert_eq!(hub.metrics().active_subscribers, 0);
    }

    #[tokio::test]
    async fn replay_gap_and_future_cursor_fail_closed() {
        let hub = Hub::new();
        for index in 0..(REPLAY_CAPACITY + 2) {
            hub.publish(format!("event-{index}")).unwrap();
        }
        assert!(matches!(
            hub.subscribe(Some(1)),
            Err(SubscribeError::ReplayGap)
        ));
        assert!(matches!(
            hub.subscribe(Some(99)),
            Err(SubscribeError::FutureCursor)
        ));
        let (replay, stream) = hub.subscribe(Some(2)).unwrap();
        assert_eq!(replay.first().unwrap().id, 3);
        assert_eq!(replay.last().unwrap().id, 18);
        drop(stream);
    }

    #[tokio::test]
    async fn lagging_subscriber_is_disconnected_and_can_replay() {
        let hub = Hub::new();
        let (_, mut live) = hub.subscribe(None).unwrap();
        for index in 1..=SUBSCRIBER_QUEUE_CAPACITY + 1 {
            hub.publish(format!("message-{index}")).unwrap();
        }
        assert_eq!(hub.metrics().active_subscribers, 0);
        assert_eq!(hub.metrics().lagged_disconnects, 1);
        let mut received = Vec::new();
        while let Some(notification) = live.next().await {
            received.push(notification.id);
        }
        assert_eq!(received, vec![1, 2, 3, 4]);
        let (replay, stream) = hub.subscribe(Some(4)).unwrap();
        assert_eq!(
            replay.iter().map(|value| value.id).collect::<Vec<_>>(),
            vec![5]
        );
        drop(stream);
    }

    #[tokio::test]
    async fn drop_and_capacity_release_subscribers() {
        let hub = Hub::new();
        let mut streams = Vec::new();
        for _ in 0..MAX_SUBSCRIBERS {
            streams.push(hub.subscribe(None).unwrap().1);
        }
        assert!(matches!(hub.subscribe(None), Err(SubscribeError::Capacity)));
        assert_eq!(hub.metrics().active_subscribers, MAX_SUBSCRIBERS);
        streams.clear();
        assert_eq!(hub.metrics().active_subscribers, 0);
    }

    #[test]
    fn finite_event_id_exhaustion_does_not_mutate_replay() {
        let hub = Hub::new();
        hub.inner.lock().unwrap().next_event_id = u64::MAX;
        assert_eq!(
            hub.publish("never".to_owned()),
            Err(PublishError::IdExhausted)
        );
        assert_eq!(hub.metrics().retained_events, 0);
    }

    #[tokio::test]
    async fn publish_endpoint_validates_policy_and_message() {
        let hub = Hub::new();
        let response = request(
            hub.clone(),
            Method::POST,
            "/notifications",
            Some(json!({"message":"  build complete  "})),
            &[],
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(json_body(response).await["message"], "build complete");

        let response = request(
            hub.clone(),
            Method::POST,
            "/notifications",
            Some(json!({"message":"blocked"})),
            &[("origin", "https://example.test")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        for invalid in ["line\nbreak", "\nhello", "hello\t"] {
            let response = request(
                hub.clone(),
                Method::POST,
                "/notifications",
                Some(json!({"message":invalid})),
                &[],
            )
            .await;
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                json_body(response).await["error"]["code"],
                "invalid_message"
            );
        }
        let response = request(hub.clone(), Method::GET, "/metrics", None, &[]).await;
        let metrics = json_body(response).await;
        assert_eq!(metrics["latest_id"], 1);
        assert_eq!(metrics["retained_events"], 1);

        let response = request(
            hub,
            Method::POST,
            "/notifications",
            None,
            &[("content-type", "text/plain")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn event_endpoint_rejects_bad_duplicate_gap_and_future_cursors() {
        let hub = Hub::new();
        for index in 0..(REPLAY_CAPACITY + 2) {
            hub.publish(format!("event-{index}")).unwrap();
        }
        let response = request(
            hub.clone(),
            Method::GET,
            "/events",
            None,
            &[("last-event-id", "bad")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = request(
            hub.clone(),
            Method::GET,
            "/events",
            None,
            &[("last-event-id", "2"), ("last-event-id", "3")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = request(
            hub.clone(),
            Method::GET,
            "/events",
            None,
            &[("last-event-id", "1")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["error"]["code"], "replay_gap");

        let response = request(
            hub,
            Method::GET,
            "/events",
            None,
            &[("last-event-id", "99")],
        )
        .await;
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(response).await["error"]["code"], "future_cursor");
    }

    #[tokio::test]
    async fn sse_response_has_explicit_headers_and_drop_unregisters() {
        let hub = Hub::new();
        let response = request(hub.clone(), Method::GET, "/events", None, &[]).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/event-stream"
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(response.headers()["x-accel-buffering"], "no");
        assert_eq!(hub.metrics().active_subscribers, 1);
        drop(response);
        assert_eq!(hub.metrics().active_subscribers, 0);
    }

    #[tokio::test]
    async fn metrics_and_native_routing_are_bounded() {
        let hub = Hub::new();
        let response = request(hub.clone(), Method::GET, "/metrics", None, &[]).await;
        assert_eq!(response.status(), StatusCode::OK);
        let metrics = json_body(response).await;
        assert_eq!(metrics["replay_capacity"], REPLAY_CAPACITY);
        assert_eq!(
            metrics["subscriber_queue_capacity"],
            SUBSCRIBER_QUEUE_CAPACITY
        );

        let response = request(hub.clone(), Method::GET, "/missing", None, &[]).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let response = request(hub, Method::POST, "/events", None, &[]).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(response.headers().contains_key(header::ALLOW));
    }
}
