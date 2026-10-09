//! API-only Yetlet sample using ordinary handlers, extractors and middleware.
//!
//! No browser target, DOM renderer, WASM build or generated asset is involved.

use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use yetlet_request_id::RequestIds;
use yetlet_server_axum::{
    App, RequestContext,
    middleware::{Next, from_fn},
};

const MAX_JSON_BODY: usize = 8 * 1024;
const API_VERSION: HeaderValue = HeaderValue::from_static("1");

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub year: u16,
    pub version: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateBook {
    title: String,
    author: String,
    year: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateBook {
    title: String,
    author: String,
    year: u16,
    expected_version: u64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListQuery {
    q: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct BookList {
    items: Vec<Book>,
    count: usize,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
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
        api_error(self.status, self.code, self.message)
    }
}

struct Store {
    books: BTreeMap<u64, Book>,
    next_id: u64,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            books: BTreeMap::new(),
            next_id: 1,
        }
    }
}

#[derive(Clone, Default)]
pub struct ApiState {
    store: Arc<Mutex<Store>>,
}

fn apply_response_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}

fn json_response<T: Serialize>(status: StatusCode, body: T) -> Response {
    let mut response = (status, Json(body)).into_response();
    apply_response_headers(&mut response);
    response
}

fn empty_response(status: StatusCode) -> Response {
    let mut response = status.into_response();
    apply_response_headers(&mut response);
    response
}

fn api_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    json_response(
        status,
        ErrorEnvelope {
            error: ErrorBody { code, message },
        },
    )
}

fn internal_error() -> Response {
    api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "Unable to complete request",
    )
}

fn parse_json<T>(payload: Result<Json<T>, JsonRejection>) -> Result<T, ApiFailure> {
    match payload {
        Ok(Json(value)) => Ok(value),
        Err(JsonRejection::BytesRejection(_)) => Err(ApiFailure::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "JSON body exceeds the 8 KiB limit",
        )),
        Err(_) => Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "Request body does not match the JSON schema",
        )),
    }
}

fn parse_id(path: Result<Path<u64>, PathRejection>) -> Result<u64, ApiFailure> {
    path.map(|Path(id)| id).map_err(|_| {
        ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_path",
            "Book id must be an unsigned integer",
        )
    })
}

fn validate_fields(
    title: String,
    author: String,
    year: u16,
) -> Result<(String, String, u16), ApiFailure> {
    let title = title.trim().to_owned();
    let author = author.trim().to_owned();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_title",
            "Title must contain 1 to 120 characters",
        ));
    }
    if author.is_empty() || author.chars().count() > 80 {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_author",
            "Author must contain 1 to 80 characters",
        ));
    }
    if !(1450..=2100).contains(&year) {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_year",
            "Year must be between 1450 and 2100",
        ));
    }
    Ok((title, author, year))
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Ok(Some(value)) = single_header(headers, header::CONTENT_TYPE) else {
        return false;
    };
    value
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
}

fn single_header(headers: &HeaderMap, name: header::HeaderName) -> Result<Option<&str>, ()> {
    let mut values = headers.get_all(name).iter();
    let Some(first) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(());
    }
    first.to_str().map(Some).map_err(|_| ())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectAuthority {
    host: String,
    port: u16,
}

fn parse_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn normalize_dns_or_ipv4(value: &str) -> Option<String> {
    if let Ok(address) = value.parse::<Ipv4Addr>() {
        return Some(address.to_string());
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return None;
    }
    if value.len() > 253
        || value.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                || !label
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric())
                || !label
                    .as_bytes()
                    .last()
                    .is_some_and(|byte| byte.is_ascii_alphanumeric())
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

fn direct_http_authority(value: &str) -> Option<DirectAuthority> {
    if value.is_empty()
        || value.chars().any(|character| {
            character.is_ascii_whitespace() || matches!(character, '/' | '?' | '#' | '@' | ',')
        })
    {
        return None;
    }

    if let Some(bracketed) = value.strip_prefix('[') {
        let closing = bracketed.find(']')?;
        let address = bracketed[..closing].parse::<Ipv6Addr>().ok()?;
        let suffix = &bracketed[closing + 1..];
        let port = if suffix.is_empty() {
            80
        } else {
            parse_port(suffix.strip_prefix(':')?)?
        };
        return Some(DirectAuthority {
            host: address.to_string(),
            port,
        });
    }
    if value.contains('[') || value.contains(']') {
        return None;
    }

    let (host, port) = match value.rsplit_once(':') {
        Some((host, port)) => {
            if host.contains(':') {
                return None;
            }
            (host, parse_port(port)?)
        }
        None => (value, 80),
    };
    Some(DirectAuthority {
        host: normalize_dns_or_ipv4(host)?,
        port,
    })
}

fn same_authority(left: &DirectAuthority, right: &DirectAuthority) -> bool {
    left == right
}
fn same_origin_or_no_origin(headers: &HeaderMap) -> bool {
    let host = match single_header(headers, header::HOST) {
        Ok(None) => None,
        Ok(Some(host)) => match direct_http_authority(host) {
            Some(host) => Some(host),
            None => return false,
        },
        Err(()) => return false,
    };
    let origin = match single_header(headers, header::ORIGIN) {
        Ok(None) => return true,
        Ok(Some(origin)) => origin,
        Err(()) => return false,
    };
    let Some(host) = host else {
        return false;
    };
    let Some((scheme, origin_authority)) = origin.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http") {
        return false;
    }
    direct_http_authority(origin_authority).is_some_and(|origin| same_authority(&origin, &host))
}
/// RequestContext middleware owns only HTTP policy and response metadata.
async fn api_policy(context: RequestContext, next: Next) -> Response {
    let method = context.method().clone();
    let is_mutation = matches!(method, Method::POST | Method::PUT | Method::DELETE);
    let requires_json = matches!(method, Method::POST | Method::PUT);

    let mut response = if is_mutation && !same_origin_or_no_origin(context.headers()) {
        api_error(
            StatusCode::FORBIDDEN,
            "forbidden_origin",
            "Cross-origin mutation is not permitted",
        )
    } else if requires_json && !is_json_content_type(context.headers()) {
        api_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Expected application/json",
        )
    } else {
        next.run(context.into_request()).await
    };
    apply_response_headers(&mut response);
    response
        .headers_mut()
        .insert("x-yetlet-api-version", API_VERSION);
    response
}

async fn health() -> Response {
    json_response(StatusCode::OK, Health { status: "ok" })
}

async fn list_books(
    State(state): State<ApiState>,
    query: Result<Query<ListQuery>, QueryRejection>,
) -> Response {
    let query = match query {
        Ok(Query(query)) => query,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_query",
                "Query parameters do not match the schema",
            );
        }
    };
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_limit",
            "Limit must be between 1 and 100",
        );
    }
    let needle = match query.q {
        Some(value) if value.chars().count() > 80 => {
            return api_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_query_text",
                "Query text must not exceed 80 characters",
            );
        }
        Some(value) => value.trim().to_lowercase(),
        None => String::new(),
    };
    let store = match state.store.lock() {
        Ok(store) => store,
        Err(_) => return internal_error(),
    };
    let items = store
        .books
        .values()
        .filter(|book| {
            needle.is_empty()
                || book.title.to_lowercase().contains(&needle)
                || book.author.to_lowercase().contains(&needle)
        })
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let count = items.len();
    json_response(StatusCode::OK, BookList { items, count })
}

async fn create_book(
    State(state): State<ApiState>,
    payload: Result<Json<CreateBook>, JsonRejection>,
) -> Response {
    let payload = match parse_json(payload) {
        Ok(payload) => payload,
        Err(error) => return error.into_response(),
    };
    let (title, author, year) = match validate_fields(payload.title, payload.author, payload.year) {
        Ok(fields) => fields,
        Err(error) => return error.into_response(),
    };
    let mut store = match state.store.lock() {
        Ok(store) => store,
        Err(_) => return internal_error(),
    };
    let id = store.next_id;
    let Some(next_id) = id.checked_add(1) else {
        return api_error(
            StatusCode::CONFLICT,
            "id_exhausted",
            "No additional book ids are available",
        );
    };
    let book = Book {
        id,
        title,
        author,
        year,
        version: 1,
    };
    store.next_id = next_id;
    store.books.insert(id, book.clone());
    drop(store);

    let mut response = json_response(StatusCode::CREATED, book);
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/api/books/{id}")).expect("numeric location"),
    );
    response
}

async fn get_book(
    State(state): State<ApiState>,
    path: Result<Path<u64>, PathRejection>,
) -> Response {
    let id = match parse_id(path) {
        Ok(id) => id,
        Err(error) => return error.into_response(),
    };
    let store = match state.store.lock() {
        Ok(store) => store,
        Err(_) => return internal_error(),
    };
    match store.books.get(&id) {
        Some(book) => json_response(StatusCode::OK, book.clone()),
        None => api_error(StatusCode::NOT_FOUND, "not_found", "Book does not exist"),
    }
}

async fn update_book(
    State(state): State<ApiState>,
    path: Result<Path<u64>, PathRejection>,
    payload: Result<Json<UpdateBook>, JsonRejection>,
) -> Response {
    let id = match parse_id(path) {
        Ok(id) => id,
        Err(error) => return error.into_response(),
    };
    let payload = match parse_json(payload) {
        Ok(payload) => payload,
        Err(error) => return error.into_response(),
    };
    let (title, author, year) = match validate_fields(payload.title, payload.author, payload.year) {
        Ok(fields) => fields,
        Err(error) => return error.into_response(),
    };
    let mut store = match state.store.lock() {
        Ok(store) => store,
        Err(_) => return internal_error(),
    };
    let Some(book) = store.books.get_mut(&id) else {
        return api_error(StatusCode::NOT_FOUND, "not_found", "Book does not exist");
    };
    if book.version != payload.expected_version {
        return api_error(
            StatusCode::CONFLICT,
            "stale_version",
            "Book version does not match expected_version",
        );
    }
    let Some(version) = book.version.checked_add(1) else {
        return api_error(
            StatusCode::CONFLICT,
            "version_exhausted",
            "Book version cannot advance",
        );
    };
    book.title = title;
    book.author = author;
    book.year = year;
    book.version = version;
    json_response(StatusCode::OK, book.clone())
}

async fn delete_book(
    State(state): State<ApiState>,
    path: Result<Path<u64>, PathRejection>,
) -> Response {
    let id = match parse_id(path) {
        Ok(id) => id,
        Err(error) => return error.into_response(),
    };
    let mut store = match state.store.lock() {
        Ok(store) => store,
        Err(_) => return internal_error(),
    };
    if store.books.remove(&id).is_none() {
        return api_error(StatusCode::NOT_FOUND, "not_found", "Book does not exist");
    }
    empty_response(StatusCode::NO_CONTENT)
}

pub fn app() -> Router {
    let app = App::new()
        .get("/healthz", health)
        .group("/api", |api| {
            api.route("/books", get(list_books).post(create_book))
                .route(
                    "/books/{id}",
                    get(get_book).put(update_book).delete(delete_book),
                )
                .route_layer(DefaultBodyLimit::max(MAX_JSON_BODY))
                .route_layer(from_fn(api_policy))
        })
        .with_state(ApiState::default());
    RequestIds::new().global(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    struct Observed {
        status: StatusCode,
        headers: HeaderMap,
        body: Vec<u8>,
    }

    impl Observed {
        fn json(&self) -> Value {
            serde_json::from_slice(&self.body).expect("JSON response")
        }
    }

    async fn send(router: Router, request: Request<Body>) -> Observed {
        let response = router.oneshot(request).await.expect("router response");
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("response body")
            .to_vec();
        Observed {
            status,
            headers,
            body,
        }
    }

    fn request(method: Method, uri: &str, body: impl Into<Body>) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(body.into())
            .expect("request")
    }

    fn json_request(method: Method, uri: &str, value: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string()))
            .expect("JSON request")
    }

    async fn create(router: Router, title: &str) -> Value {
        let response = send(
            router,
            json_request(
                Method::POST,
                "/api/books",
                json!({"title": title, "author": "Ada Lovelace", "year": 1843}),
            ),
        )
        .await;
        assert_eq!(response.status, StatusCode::CREATED);
        response.json()
    }

    #[tokio::test]
    async fn health_api_headers_and_method_errors_are_explicit() {
        let router = app();
        let health = send(
            router.clone(),
            request(Method::GET, "/healthz", Body::empty()),
        )
        .await;
        assert_eq!(health.status, StatusCode::OK);
        assert_eq!(health.json(), json!({"status":"ok"}));
        assert!(health.headers.contains_key("x-yetlet-request-id"));
        assert!(!health.headers.contains_key("x-yetlet-api-version"));

        let list = send(
            router.clone(),
            request(Method::GET, "/api/books", Body::empty()),
        )
        .await;
        assert_eq!(list.status, StatusCode::OK);
        assert_eq!(list.headers["x-yetlet-api-version"], "1");
        assert_eq!(list.headers[header::CACHE_CONTROL], "no-store");
        assert!(list.headers.contains_key("x-yetlet-request-id"));

        let method = send(
            router.clone(),
            request(Method::PATCH, "/api/books", Body::empty()),
        )
        .await;
        assert_eq!(method.status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(method.headers["x-yetlet-api-version"], "1");
        assert!(method.body.is_empty());
        assert_eq!(method.headers[header::CACHE_CONTROL], "no-store");
        assert_eq!(method.headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        assert!(method.headers.contains_key("x-yetlet-request-id"));

        let missing_api = send(
            router.clone(),
            request(Method::GET, "/api/missing", Body::empty()),
        )
        .await;
        assert_eq!(missing_api.status, StatusCode::NOT_FOUND);
        assert!(missing_api.body.is_empty());
        assert!(missing_api.headers.contains_key("x-yetlet-request-id"));
        assert!(!missing_api.headers.contains_key("x-yetlet-api-version"));
        assert!(!missing_api.headers.contains_key(header::CACHE_CONTROL));
        assert!(
            !missing_api
                .headers
                .contains_key(header::X_CONTENT_TYPE_OPTIONS)
        );

        assert_eq!(
            send(router.clone(), request(Method::GET, "/", Body::empty()))
                .await
                .status,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            send(router, request(Method::GET, "/app", Body::empty()))
                .await
                .status,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn create_read_and_filtered_list_share_one_store() {
        let router = app();
        let created = create(router.clone(), "Analytical Engine Notes").await;
        assert_eq!(created["id"], 1);
        assert_eq!(created["version"], 1);

        let read = send(
            router.clone(),
            request(Method::GET, "/api/books/1", Body::empty()),
        )
        .await;
        assert_eq!(read.status, StatusCode::OK);
        assert_eq!(read.json()["title"], "Analytical Engine Notes");

        let list = send(
            router,
            request(Method::GET, "/api/books?q=lovelace&limit=10", Body::empty()),
        )
        .await;
        assert_eq!(list.status, StatusCode::OK);
        assert_eq!(list.json()["count"], 1);
    }

    #[tokio::test]
    async fn optimistic_update_conflict_preserves_then_advances_state() {
        let router = app();
        create(router.clone(), "Original").await;
        let stale = send(
            router.clone(),
            json_request(
                Method::PUT,
                "/api/books/1",
                json!({
                    "title":"Changed",
                    "author":"Grace Hopper",
                    "year":1952,
                    "expected_version":0
                }),
            ),
        )
        .await;
        assert_eq!(stale.status, StatusCode::CONFLICT);
        assert_eq!(stale.json()["error"]["code"], "stale_version");

        let unchanged = send(
            router.clone(),
            request(Method::GET, "/api/books/1", Body::empty()),
        )
        .await;
        assert_eq!(unchanged.json()["title"], "Original");

        let updated = send(
            router,
            json_request(
                Method::PUT,
                "/api/books/1",
                json!({
                    "title":"Compiler Design",
                    "author":"Grace Hopper",
                    "year":1952,
                    "expected_version":1
                }),
            ),
        )
        .await;
        assert_eq!(updated.status, StatusCode::OK);
        assert_eq!(updated.json()["version"], 2);
    }

    #[tokio::test]
    async fn delete_is_idempotency_visible_and_missing_is_typed() {
        let router = app();
        create(router.clone(), "Delete Me").await;
        let deleted = send(
            router.clone(),
            request(Method::DELETE, "/api/books/1", Body::empty()),
        )
        .await;
        assert_eq!(deleted.status, StatusCode::NO_CONTENT);
        assert!(deleted.body.is_empty());

        let missing = send(router, request(Method::GET, "/api/books/1", Body::empty())).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND);
        assert_eq!(missing.json()["error"]["code"], "not_found");
    }

    #[test]
    fn direct_authority_parser_is_bounded_and_canonical() {
        assert_eq!(
            direct_http_authority("Example.COM"),
            Some(DirectAuthority {
                host: "example.com".to_owned(),
                port: 80,
            })
        );
        assert_eq!(
            direct_http_authority("127.0.0.1:8080"),
            Some(DirectAuthority {
                host: "127.0.0.1".to_owned(),
                port: 8080,
            })
        );
        assert_eq!(
            direct_http_authority("[2001:0db8::1]:8080"),
            Some(DirectAuthority {
                host: "2001:db8::1".to_owned(),
                port: 8080,
            })
        );
        for invalid in [
            "2001:db8::1",
            "999.1.1.1",
            "-example.com",
            "example-.com",
            "example..com",
            "localhost:abc",
            "localhost:99999",
            "localhost:80,evil",
            "user@localhost",
            "localhost/path",
        ] {
            assert_eq!(direct_http_authority(invalid), None, "accepted {invalid:?}");
        }
    }

    #[tokio::test]
    async fn origin_policy_accepts_one_direct_origin_and_rejects_ambiguous_values() {
        let router = app();
        let payload = json!({"title":"Allowed","author":"Browser","year":2020});
        let mut same_origin = json_request(Method::POST, "/api/books", payload.clone());
        same_origin
            .headers_mut()
            .insert(header::HOST, HeaderValue::from_static("localhost:3001"));
        same_origin.headers_mut().insert(
            header::ORIGIN,
            HeaderValue::from_static("http://localhost:3001"),
        );
        assert_eq!(
            send(router.clone(), same_origin).await.status,
            StatusCode::CREATED
        );

        let mut default_port = json_request(
            Method::POST,
            "/api/books",
            json!({"title":"Default Port","author":"Browser","year":2020}),
        );
        default_port
            .headers_mut()
            .insert(header::HOST, HeaderValue::from_static("LOCALHOST"));
        default_port.headers_mut().insert(
            header::ORIGIN,
            HeaderValue::from_static("HTTP://localhost:80"),
        );
        assert_eq!(
            send(router.clone(), default_port).await.status,
            StatusCode::CREATED
        );

        let mut no_origin = json_request(
            Method::POST,
            "/api/books",
            json!({"title":"No Origin","author":"CLI","year":2020}),
        );
        no_origin
            .headers_mut()
            .insert(header::HOST, HeaderValue::from_static("localhost:3001"));
        assert_eq!(
            send(router.clone(), no_origin).await.status,
            StatusCode::CREATED
        );

        let cases: &[(&[&str], &[&str])] = &[
            (
                &["localhost:3001"],
                &["http://localhost:3001", "https://other.example"],
            ),
            (
                &["localhost:3001", "localhost:3001"],
                &["http://localhost:3001"],
            ),
            (&["localhost:3001/path"], &["http://localhost:3001/path"]),
            (&["localhost:3001"], &["http://localhost:3001/path"]),
            (
                &["localhost:3001"],
                &["http://localhost:3001, https://other.example"],
            ),
            (&["localhost"], &["http://localhost:abc"]),
            (&["localhost"], &["http://localhost:99999"]),
            (&["localhost:abc"], &[]),
            (&["localhost:3001", "localhost:3001"], &[]),
            (&["localhost:80,evil"], &[]),
            (&["localhost:80,evil"], &["http://localhost"]),
            (&["localhost,evil"], &["http://localhost"]),
            (&["localhost"], &["http://localhost,evil"]),
        ];
        for (hosts, origins) in cases {
            let mut request = json_request(Method::POST, "/api/books", payload.clone());
            for host in *hosts {
                request.headers_mut().append(
                    header::HOST,
                    HeaderValue::from_str(host).expect("test Host header"),
                );
            }
            for origin in *origins {
                request.headers_mut().append(
                    header::ORIGIN,
                    HeaderValue::from_str(origin).expect("test Origin header"),
                );
            }
            let response = send(router.clone(), request).await;
            assert_eq!(response.status, StatusCode::FORBIDDEN);
            assert_eq!(response.json()["error"]["code"], "forbidden_origin");
        }

        let list = send(router, request(Method::GET, "/api/books", Body::empty())).await;
        assert_eq!(list.json()["count"], 3);
    }

    #[tokio::test]
    async fn invalid_inputs_origins_and_limits_fail_closed() {
        let router = app();
        let blank = send(
            router.clone(),
            json_request(
                Method::POST,
                "/api/books",
                json!({"title":" ","author":"Nobody","year":2020}),
            ),
        )
        .await;
        assert_eq!(blank.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(blank.json()["error"]["code"], "invalid_title");

        let unknown = send(
            router.clone(),
            json_request(
                Method::POST,
                "/api/books",
                json!({"title":"Book","author":"A","year":2020,"event":"click"}),
            ),
        )
        .await;
        assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
        assert_eq!(unknown.json()["error"]["code"], "invalid_json");

        let wrong_type = send(
            router.clone(),
            request(Method::POST, "/api/books", Body::from("{}")),
        )
        .await;
        assert_eq!(wrong_type.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let mut duplicate_type = json_request(
            Method::POST,
            "/api/books",
            json!({"title":"Ambiguous","author":"Nobody","year":2020}),
        );
        duplicate_type
            .headers_mut()
            .append(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        let duplicate_type = send(router.clone(), duplicate_type).await;
        assert_eq!(duplicate_type.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            duplicate_type.json()["error"]["code"],
            "unsupported_media_type"
        );

        let cross_origin = send(
            router.clone(),
            Request::builder()
                .method(Method::POST)
                .uri("/api/books")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::HOST, "localhost:3001")
                .header(header::ORIGIN, "https://other.example")
                .body(Body::from(
                    json!({"title":"Attack","author":"A","year":2020}).to_string(),
                ))
                .expect("request"),
        )
        .await;
        assert_eq!(cross_origin.status, StatusCode::FORBIDDEN);

        let oversized = send(
            router.clone(),
            Request::builder()
                .method(Method::POST)
                .uri("/api/books")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("x".repeat(MAX_JSON_BODY + 1)))
                .expect("request"),
        )
        .await;
        assert_eq!(oversized.status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(oversized.json()["error"]["code"], "payload_too_large");

        let bad_path = send(
            router.clone(),
            request(Method::GET, "/api/books/not-a-number", Body::empty()),
        )
        .await;
        assert_eq!(bad_path.status, StatusCode::BAD_REQUEST);
        assert_eq!(bad_path.json()["error"]["code"], "invalid_path");

        let bad_query = send(
            router.clone(),
            request(Method::GET, "/api/books?limit=0", Body::empty()),
        )
        .await;
        assert_eq!(bad_query.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(bad_query.json()["error"]["code"], "invalid_limit");

        let list = send(router, request(Method::GET, "/api/books", Body::empty())).await;
        assert_eq!(list.json()["count"], 0);
    }
}
