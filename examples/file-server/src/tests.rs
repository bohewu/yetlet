use super::*;
use axum::{
    body::{Bytes, to_bytes},
    http::Request,
};
use futures_util::stream;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
fn store() -> FileStore {
    FileStore::new(TOKEN.to_owned()).unwrap()
}
fn multipart(name: &str, data: &[u8], close: bool) -> Vec<u8> {
    let mut bytes = format!("--sample\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: text/plain\r\n\r\n").into_bytes();
    bytes.extend_from_slice(data);
    if close {
        bytes.extend_from_slice(b"\r\n--sample--\r\n");
    }
    bytes
}
fn post(bytes: impl Into<Body>) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/uploads")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "multipart/form-data; boundary=sample")
        .body(bytes.into())
        .unwrap()
}
fn get(uri: &str, method: Method) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap()
}
async fn call(store: &FileStore, request: Request<Body>) -> Response {
    app(store.clone()).oneshot(request).await.unwrap()
}
async fn bytes(response: Response) -> Bytes {
    to_bytes(response.into_body(), MAX_BODY_BYTES + 4096)
        .await
        .unwrap()
}
async fn upload_one(store: &FileStore, text: &[u8]) -> String {
    let response = call(store, post(multipart("example.txt", text, true))).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned()
}
fn pending_count(store: &FileStore) -> usize {
    std::fs::read_dir(store.0.root.path().join("uploads"))
        .unwrap()
        .count()
}
async fn wait_count(store: &FileStore, count: usize) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while pending_count(store) != count {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("pending file count changed");
}
fn unfinished() -> Request<Body> {
    let data = multipart("example.txt", b"hello, still sending", false);
    let stream = stream::once(async { Ok::<_, io::Error>(Bytes::from(data)) })
        .chain(stream::pending::<Result<Bytes, io::Error>>());
    post(Body::from_stream(stream))
}

#[test]
fn setup_is_private_bounded_and_loopback_only() {
    assert!(listen_address("127.0.0.1:0").is_ok());
    assert!(listen_address("[::1]:0").is_ok());
    for value in [
        "0.0.0.0:3006",
        "[::]:3006",
        "192.0.2.1:3006",
        "hostname:3006",
    ] {
        assert!(listen_address(value).is_err(), "{value}");
    }
    assert!(FileStore::new("short".to_owned()).is_err());
    let s = store();
    let root = s.0.root.path().to_owned();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&root).unwrap().permissions().mode() & 0o077,
            0
        );
    }
    assert_eq!(s.0.slots.available_permits(), MAX_FILES);
    drop(s);
    assert!(!root.exists());
}

#[tokio::test]
async fn static_get_head_range_and_escaped_english_landing() {
    let s = store();
    let home = call(&s, get("/", Method::GET)).await;
    assert_eq!(home.status(), StatusCode::OK);
    assert_eq!(home.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    let html = String::from_utf8(bytes(home).await.to_vec()).unwrap();
    assert!(html.contains("<main lang=\"en\">"));
    assert!(!html.contains(TOKEN));
    assert!(!html.contains("<script"));
    let response = call(&s, get("/static/welcome.txt", Method::GET)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );
    assert_eq!(bytes(response).await, WELCOME);
    let response = call(&s, get("/static/welcome.txt", Method::HEAD)).await;
    assert_eq!(
        response.headers()[header::CONTENT_LENGTH],
        WELCOME.len().to_string()
    );
    assert!(bytes(response).await.is_empty());
    let mut request = get("/static/welcome.txt", Method::GET);
    request
        .headers_mut()
        .insert(header::RANGE, HeaderValue::from_static("bytes=0-5"));
    let response = call(&s, request).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(bytes(response).await, "Yetlet");
}

#[tokio::test]
async fn protected_routes_reject_missing_wrong_duplicate_and_browser_credentials() {
    let s = store();
    for kind in 0..4 {
        let mut request = post(multipart("example.txt", b"hello", true));
        match kind {
            0 => {
                request.headers_mut().remove(header::AUTHORIZATION);
            }
            1 => {
                request.headers_mut().insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer wrong"),
                );
            }
            2 => {
                request.headers_mut().append(
                    header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer another"),
                );
            }
            _ => {
                request.headers_mut().insert(
                    header::ORIGIN,
                    HeaderValue::from_static("http://127.0.0.1:3006"),
                );
            }
        }
        let response = call(&s, request).await;
        assert_eq!(
            response.status(),
            if kind == 3 {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-origin")
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
    assert!(s.0.files.lock().unwrap().is_empty());
    assert_eq!(pending_count(&s), 0);
}

#[tokio::test]
async fn upload_and_repeat_download_preserve_bytes_and_safe_headers() {
    let s = store();
    let text = "Hello UTF-8: 台灣\n".as_bytes();
    let path = upload_one(&s, text).await;
    for _ in 0..2 {
        let response = call(&s, get(&path, Method::GET)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/octet-stream"
        );
        assert_eq!(
            response.headers()[header::CONTENT_DISPOSITION],
            "attachment; filename=\"example.txt\""
        );
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            text.len().to_string()
        );
        assert_eq!(bytes(response).await, text);
    }
    let response = call(&s, get(&path, Method::HEAD)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_LENGTH],
        text.len().to_string()
    );
    assert!(bytes(response).await.is_empty());
    assert_eq!(s.0.downloads.available_permits(), MAX_DOWNLOADS);
}

#[tokio::test]
async fn paths_filenames_and_content_cannot_escape_or_execute() {
    let s = store();
    for name in [
        "../escape.txt",
        "/tmp/escape.txt",
        "a\\b.txt",
        "foo.html",
        "run.exe",
        "a%2fb.txt",
        ".hidden.txt",
        "a..txt",
        "a b.txt",
        "CON.txt",
    ] {
        assert!(!valid_filename(name), "{name}");
        let response = call(&s, post(multipart(name, b"hello", true))).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{name}");
    }
    for path in [
        "/static/",
        "/static/../Cargo.toml",
        "/static/%2e%2e%2fCargo.toml",
        "/files/%2e%2e%2fetc%2fpasswd",
        "/files/00000000000000000000000000000000",
    ] {
        assert_eq!(
            call(&s, get(path, Method::GET)).await.status(),
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    for data in [&b"binary\0value"[..], &b"\xff\xfe"[..], &b"\xe5\x8f"[..]] {
        assert_eq!(
            call(&s, post(multipart("binary.txt", data, true)))
                .await
                .status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
    }
    assert!(s.0.files.lock().unwrap().is_empty());
    assert_eq!(pending_count(&s), 0);
}

#[test]
fn utf8_validator_handles_split_codepoints_and_rejects_binary() {
    let mut check = TextCheck::default();
    for b in "文字\n".as_bytes() {
        check.chunk(&[*b]).unwrap();
    }
    check.finish().unwrap();
    let mut check = TextCheck::default();
    check.chunk(b"\xe5").unwrap();
    assert!(check.finish().is_err());
    assert!(TextCheck::default().chunk(b"nul\0").is_err());
}

#[tokio::test]
async fn size_multipart_and_media_failures_leave_no_partial_file() {
    let s = store();
    let over_file = vec![b'x'; MAX_FILE_BYTES + 1];
    let over_body = vec![b'x'; MAX_BODY_BYTES + 1];
    for data in [&over_file, &over_body] {
        assert_eq!(
            call(&s, post(multipart("large.txt", data, true)))
                .await
                .status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(pending_count(&s), 0);
    }
    let mut extra = multipart("one.txt", b"one", false);
    extra.extend_from_slice(b"\r\n--sample\r\nContent-Disposition: form-data; name=\"other\"\r\n\r\nextra\r\n--sample--\r\n");
    assert_eq!(
        call(&s, post(extra)).await.status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&s, post(multipart("empty.txt", b"", true)))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&s, post(multipart("partial.txt", b"hello", false)))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    let mut duplicate = post(multipart("one.txt", b"hello", true));
    duplicate
        .headers_mut()
        .append(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    assert_eq!(call(&s, duplicate).await.status(), StatusCode::BAD_REQUEST);
    let mut wrong = post("hello");
    wrong
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    assert_eq!(
        call(&s, wrong).await.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    assert_eq!(pending_count(&s), 0);
    assert_eq!(s.0.slots.available_permits(), MAX_FILES);
}

#[tokio::test]
async fn total_store_capacity_includes_reserved_slots() {
    let s = store();
    for _ in 0..MAX_FILES {
        upload_one(&s, b"one").await;
    }
    assert_eq!(
        call(&s, post(multipart("full.txt", b"one", true)))
            .await
            .status(),
        StatusCode::INSUFFICIENT_STORAGE
    );
    assert_eq!(s.0.files.lock().unwrap().len(), MAX_FILES);
    assert_eq!(pending_count(&s), MAX_FILES);
}

#[tokio::test]
async fn upload_deadline_drops_partial_and_releases_capacity() {
    let mut s = store();
    Arc::get_mut(&mut s.0).unwrap().timeout = Duration::from_millis(40);
    let response = call(&s, unfinished()).await;
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    wait_count(&s, 0).await;
    assert_eq!(s.0.slots.available_permits(), MAX_FILES);
    assert_eq!(s.0.uploads.available_permits(), MAX_UPLOADS);
    upload_one(&s, b"after timeout").await;
}

#[tokio::test]
async fn cancelled_concurrent_uploads_remove_partial_files_and_release_guards() {
    let s = store();
    let first = tokio::spawn(app(s.clone()).oneshot(unfinished()));
    let second = tokio::spawn(app(s.clone()).oneshot(unfinished()));
    wait_count(&s, 2).await;
    assert_eq!(
        call(&s, post(multipart("busy.txt", b"hello", true)))
            .await
            .status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    first.abort();
    second.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    assert!(second.await.unwrap_err().is_cancelled());
    wait_count(&s, 0).await;
    assert!(s.0.files.lock().unwrap().is_empty());
    assert_eq!(s.0.slots.available_permits(), MAX_FILES);
    assert_eq!(s.0.uploads.available_permits(), MAX_UPLOADS);
}

#[tokio::test]
async fn download_permit_lives_with_body_and_releases_on_disconnect() {
    let s = store();
    let path = upload_one(&s, b"hello").await;
    let mut responses = Vec::new();
    for _ in 0..MAX_DOWNLOADS {
        let response = call(&s, get(&path, Method::GET)).await;
        assert_eq!(response.status(), StatusCode::OK);
        responses.push(response);
    }
    assert_eq!(
        call(&s, get(&path, Method::GET)).await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    drop(responses.pop());
    assert_eq!(s.0.downloads.available_permits(), 1);
    assert_eq!(
        bytes(call(&s, get(&path, Method::GET)).await).await,
        "hello"
    );
    drop(responses);
    assert_eq!(s.0.downloads.available_permits(), MAX_DOWNLOADS);
}

#[cfg(unix)]
#[tokio::test]
async fn replaced_upload_symlink_cannot_read_a_different_file() {
    use std::os::unix::fs::symlink;
    let s = store();
    let path = upload_one(&s, b"original").await;
    let item = s.0.files.lock().unwrap().values().next().unwrap().clone();
    let original_path = item.file.path();
    let old = original_path.with_extension("old");
    std::fs::rename(original_path, &old).unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), "secret outside data").unwrap();
    symlink(outside.path(), original_path).unwrap();
    let response = call(&s, get(&path, Method::GET)).await;
    let status = response.status();
    let data = bytes(response).await;
    assert!(
        status == StatusCode::INTERNAL_SERVER_ERROR
            || (status == StatusCode::OK && data == "original")
    );
    assert!(!data.windows(6).any(|window| window == b"secret"));
}

#[tokio::test]
async fn response_ownership_keeps_storage_until_download_finishes() {
    let s = store();
    let root = s.0.root.path().to_owned();
    let path = upload_one(&s, b"survives").await;
    let response = call(&s, get(&path, Method::GET)).await;
    drop(s);
    assert!(root.exists());
    assert_eq!(bytes(response).await, "survives");
    assert!(!root.exists());
}

#[tokio::test]
async fn unsupported_methods_preserve_native_405() {
    let s = store();
    let response = call(&s, get("/uploads", Method::GET)).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        response.headers()[header::ALLOW]
            .to_str()
            .unwrap()
            .contains("POST")
    );
}
