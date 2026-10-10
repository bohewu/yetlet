//! Native file-service demonstration, not a reusable Yetlet storage runtime.
//!
//! Static files use Tower HTTP. Uploads use native Multipart, private temporary
//! files, explicit authorization/limits and commit only after complete parsing.
use std::{
    collections::BTreeMap,
    io::{self, Write},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Request, State, multipart::MultipartError},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use subtle::ConstantTimeEq;
use tempfile::{NamedTempFile, TempDir};
use tokio::{
    io::AsyncWriteExt,
    sync::{OwnedSemaphorePermit, Semaphore},
};
use tokio_util::io::ReaderStream;
use tower_http::services::ServeFile;
use yetlet_core::{Element, View};
use yetlet_server_axum::{
    App, Page,
    middleware::{Next, from_fn, from_fn_with_state, with_global},
};

pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_BODY_BYTES: usize = MAX_FILE_BYTES + 8 * 1024;
pub const MAX_FILES: usize = 16;
pub const MAX_UPLOADS: usize = 2;
pub const MAX_DOWNLOADS: usize = 4;
pub const UPLOAD_TIMEOUT: Duration = Duration::from_secs(10);
pub const WELCOME: &str = "Yetlet file server: a bounded native HTTP sample.\n";

/// Runtime state explicitly owned by this one sample application.
#[derive(Clone)]
pub struct FileStore(Arc<Inner>);
struct Inner {
    root: Arc<TempDir>,
    token: String,
    files: Mutex<BTreeMap<String, Arc<Uploaded>>>,
    slots: Arc<Semaphore>,
    uploads: Arc<Semaphore>,
    downloads: Arc<Semaphore>,
    timeout: Duration,
}
struct Uploaded {
    file: NamedTempFile,
    name: String,
    size: usize,
    _slot: OwnedSemaphorePermit,
    _root: Arc<TempDir>,
}

impl FileStore {
    /// A 256-bit hex development capability; never put this in an HTTP page.
    pub fn new(token: String) -> io::Result<Self> {
        if token.len() != 64 || !token.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "token must be 64 hexadecimal characters",
            ));
        }
        let mut builder = tempfile::Builder::new();
        builder.prefix("yetlet-files-");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(std::fs::Permissions::from_mode(0o700));
        }
        let root = Arc::new(builder.tempdir()?);
        std::fs::create_dir(root.path().join("uploads"))?;
        // Fixed bundled content, never an operator-selected or user-uploaded path.
        let mut welcome = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.path().join("welcome.txt"))?;
        welcome.write_all(WELCOME.as_bytes())?;
        Ok(Self(Arc::new(Inner {
            root,
            token,
            files: Mutex::new(BTreeMap::new()),
            slots: Arc::new(Semaphore::new(MAX_FILES)),
            uploads: Arc::new(Semaphore::new(MAX_UPLOADS)),
            downloads: Arc::new(Semaphore::new(MAX_DOWNLOADS)),
            timeout: UPLOAD_TIMEOUT,
        })))
    }
}

/// Generate a development token from the operating-system RNG, not a password.
pub fn new_token() -> io::Result<String> {
    random_hex::<32>()
}
fn random_hex<const N: usize>() -> io::Result<String> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// The anonymous development listener may never bind a public/wildcard address.
pub fn listen_address(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid listener"))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "file sample requires a loopback listener",
        ));
    }
    Ok(address)
}

#[derive(Debug)]
struct Failure(StatusCode, &'static str);
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}
fn internal(_: impl std::fmt::Display) -> Failure {
    Failure(StatusCode::INTERNAL_SERVER_ERROR, "File operation failed")
}
fn multipart_error(error: MultipartError) -> Failure {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        Failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Request body exceeds the limit",
        )
    } else {
        Failure(StatusCode::BAD_REQUEST, "Malformed multipart upload")
    }
}

async fn protected(State(store): State<FileStore>, request: Request, next: Next) -> Response {
    // This is a curl/client demonstration, not cookie authentication or a browser
    // upload form. Do not grant CORS or trust Origin/forwarded-host assertions.
    if request.headers().contains_key(header::ORIGIN) {
        return (
            StatusCode::FORBIDDEN,
            "Browser-origin requests are not accepted",
        )
            .into_response();
    }
    let auth = request.headers().get_all(header::AUTHORIZATION);
    let mut values = auth.iter();
    let supplied = values
        .next()
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    let valid = supplied.is_some_and(|value| {
        value.len() == store.0.token.len()
            && bool::from(value.as_bytes().ct_eq(store.0.token.as_bytes()))
    });
    if !valid || values.next().is_some() {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "A development bearer token is required",
        )
            .into_response();
    }
    next.run(request).await
}

async fn security(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'; sandbox"));
    response
}

async fn home() -> Page {
    Page::new("Yetlet Files", Element::new("main").attribute("lang", "en")
        .child(Element::new("h1").child(View::text("Yetlet File Server")))
        .child(Element::new("p").child(View::text("Use the sample README for authenticated curl upload/download commands. Tokens never appear in HTTP responses.")))
        .child(Element::new("a").attribute("href", "/static/welcome.txt").child(View::text("Read the static fixture"))))
}

fn valid_filename(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".txt") else {
        return false;
    };
    (1..=60).contains(&stem.len())
        && stem.as_bytes()[0].is_ascii_alphanumeric()
        && stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        && !matches!(
            stem.to_ascii_uppercase().as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "LPT1"
        )
}

/// Validate UTF-8 across chunk boundaries while retaining at most three bytes.
#[derive(Default)]
struct TextCheck(Vec<u8>);
impl TextCheck {
    fn chunk(&mut self, bytes: &[u8]) -> Result<(), Failure> {
        let mut combined = std::mem::take(&mut self.0);
        combined.extend_from_slice(bytes);
        let (prefix, tail) = match std::str::from_utf8(&combined) {
            Ok(text) => (text, &[][..]),
            Err(error) if error.error_len().is_none() => {
                let (prefix, tail) = combined.split_at(error.valid_up_to());
                (
                    std::str::from_utf8(prefix).map_err(|_| Self::invalid())?,
                    tail,
                )
            }
            Err(_) => return Err(Self::invalid()),
        };
        if prefix
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(Self::invalid());
        }
        self.0.extend_from_slice(tail);
        Ok(())
    }
    fn finish(self) -> Result<(), Failure> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Self::invalid())
        }
    }
    fn invalid() -> Failure {
        Failure(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Only UTF-8 text without binary controls is accepted",
        )
    }
}

async fn upload(
    State(store): State<FileStore>,
    headers: HeaderMap,
    multipart: Result<Multipart, axum::extract::multipart::MultipartRejection>,
) -> Result<Response, Failure> {
    if headers.get_all(header::CONTENT_TYPE).iter().count() != 1 {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "Exactly one multipart Content-Type is required",
        ));
    }
    let multipart = multipart.map_err(|_| {
        Failure(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Expected multipart/form-data",
        )
    })?;
    let _inflight = store.0.uploads.clone().try_acquire_owned().map_err(|_| {
        Failure(
            StatusCode::TOO_MANY_REQUESTS,
            "Upload concurrency limit reached",
        )
    })?;
    let slot = store.0.slots.clone().try_acquire_owned().map_err(|_| {
        Failure(
            StatusCode::INSUFFICIENT_STORAGE,
            "The bounded file store is full",
        )
    })?;
    tokio::time::timeout(store.0.timeout, receive(store.clone(), multipart, slot))
        .await
        .map_err(|_| Failure(StatusCode::REQUEST_TIMEOUT, "Upload deadline exceeded"))?
}

async fn receive(
    store: FileStore,
    mut multipart: Multipart,
    slot: OwnedSemaphorePermit,
) -> Result<Response, Failure> {
    let mut field = multipart
        .next_field()
        .await
        .map_err(multipart_error)?
        .ok_or(Failure(
            StatusCode::BAD_REQUEST,
            "Exactly one file field is required",
        ))?;
    if field.name() != Some("file") {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "Expected a field named file",
        ));
    }
    let name = field
        .file_name()
        .filter(|n| valid_filename(n))
        .ok_or(Failure(
            StatusCode::BAD_REQUEST,
            "Use a simple ASCII .txt filename",
        ))?
        .to_owned();
    if field
        .content_type()
        .is_some_and(|t| t != "text/plain" && t != "text/plain; charset=utf-8")
    {
        return Err(Failure(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Expected a plain text file",
        ));
    }
    // A random private path, NEVER the untrusted original filename. On any
    // parse/write/timeout/cancellation failure this owner removes the partial.
    let pending = tempfile::Builder::new()
        .prefix("pending-")
        .tempfile_in(store.0.root.path().join("uploads"))
        .map_err(internal)?;
    let mut writer = tokio::fs::File::from_std(pending.reopen().map_err(internal)?);
    let mut size = 0usize;
    let mut text = TextCheck::default();
    while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
        size = size.checked_add(chunk.len()).ok_or(Failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "File exceeds the limit",
        ))?;
        if size > MAX_FILE_BYTES {
            return Err(Failure(
                StatusCode::PAYLOAD_TOO_LARGE,
                "File exceeds the limit",
            ));
        }
        text.chunk(&chunk)?;
        writer.write_all(&chunk).await.map_err(internal)?;
    }
    text.finish()?;
    if size == 0 {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "Empty files are not accepted",
        ));
    }
    drop(field);
    if multipart
        .next_field()
        .await
        .map_err(multipart_error)?
        .is_some()
    {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "Only one file field is accepted",
        ));
    }
    writer.flush().await.map_err(internal)?;
    drop(writer);
    let mut files = store.0.files.lock().map_err(internal)?;
    let id = loop {
        let candidate = random_hex::<16>().map_err(internal)?;
        if !files.contains_key(&candidate) {
            break candidate;
        }
    };
    files.insert(
        id.clone(),
        Arc::new(Uploaded {
            file: pending,
            name,
            size,
            _slot: slot,
            _root: store.0.root.clone(),
        }),
    );
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/files/{id}"))],
        format!("/files/{id}\n"),
    )
        .into_response())
}

async fn download(
    State(store): State<FileStore>,
    Path(id): Path<String>,
    method: Method,
) -> Result<Response, Failure> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Failure(StatusCode::NOT_FOUND, "File not found"));
    }
    let entry = store
        .0
        .files
        .lock()
        .map_err(internal)?
        .get(&id)
        .cloned()
        .ok_or(Failure(StatusCode::NOT_FOUND, "File not found"))?;
    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        let permit = store.0.downloads.clone().try_acquire_owned().map_err(|_| {
            Failure(
                StatusCode::TOO_MANY_REQUESTS,
                "Download concurrency limit reached",
            )
        })?;
        // Reopen guarantees the same file identity even if the name was swapped;
        // File::open(entry.file.path()) would not provide that contract.
        let file = tokio::fs::File::from_std(entry.file.reopen().map_err(internal)?);
        let retained = entry.clone();
        Body::from_stream(
            ReaderStream::with_capacity(file, 16 * 1024).map(move |chunk| {
                let _owners = (&retained, &permit);
                chunk
            }),
        )
    };
    let mut response = Response::new(body);
    let h = response.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    h.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&entry.size.to_string()).map_err(internal)?,
    );
    h.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", entry.name))
            .map_err(internal)?,
    );
    Ok(response)
}

/// Assemble with ordinary native Router APIs; no new framework-level provider.
pub fn app(store: FileStore) -> Router {
    let static_file = store.0.root.path().join("welcome.txt");
    let guarded = App::new()
        .post("/uploads", upload)
        .get("/files/{id}", download)
        .route_layer(from_fn_with_state(store.clone(), protected))
        .route_layer(DefaultBodyLimit::max(MAX_BODY_BYTES));
    let routes = App::new()
        .get("/", home)
        .get("/healthz", || async { "ok" })
        .map_router(|router| {
            router.route_service("/static/welcome.txt", ServeFile::new(static_file))
        })
        .map_router(|router| router.merge(guarded.into_router()))
        .with_state(store)
        .into_router();
    with_global(routes, from_fn(security))
}

#[cfg(test)]
mod tests;
