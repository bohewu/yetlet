//! Explicit, fixed private reference transport; not general deployment discovery.
use super::*;
use std::{fs::File, io::Read, os::unix::fs::PermissionsExt, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Transport {
    pub pg_host: String,
    pub ca_file: String,
    pub pg_password_file: String,
    pub valkey_url_file: String,
    pub namespace: String,
    pub port: u16,
    pub origin: String,
}
fn private_file(path: &Path, max: usize) -> Result<String, ()> {
    let metadata = path.symlink_metadata().map_err(|_| ())?;
    if !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > max as u64
    {
        return Err(());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| ())?
        .take((max + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() > max {
        return Err(());
    }
    String::from_utf8(bytes).map_err(|_| ())
}
impl Transport {
    pub(super) fn load() -> Result<Option<Self>, ()> {
        let Some(path) = std::env::var_os("YT184_TRANSPORT_CONFIG_FILE") else {
            return Ok(None);
        };
        let input = private_file(Path::new(&path), 4096)?;
        let value: Self = serde_json::from_str(&input).map_err(|_| ())?;
        value.validate()?;
        Ok(Some(value))
    }
    fn validate(&self) -> Result<(), ()> {
        if self.port < 1024
            || self.origin != "https://auth.yt184.test:8443"
            || !matches!(
                self.pg_host.as_str(),
                "sql.yt184.test" | "wrong.sql.yt184.test"
            )
            || !self.namespace.starts_with("yt184_")
            || self.namespace.len() > 64
            || !self
                .namespace
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(());
        }
        let ca = std::fs::metadata(&self.ca_file).map_err(|_| ())?;
        if !ca.is_file() || ca.len() > 64 * 1024 {
            return Err(());
        }
        self.password()?;
        self.valkey_url()?;
        Ok(())
    }
    pub(super) fn password(&self) -> Result<String, ()> {
        let value = private_file(Path::new(&self.pg_password_file), 256)?;
        let value = value.trim();
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(());
        }
        Ok(value.to_owned())
    }
    pub(super) fn valkey_url(&self) -> Result<String, ()> {
        let value = private_file(Path::new(&self.valkey_url_file), 1024)?;
        let value = value.trim();
        let (credential, host) = value
            .strip_prefix("rediss://yt184_app:")
            .and_then(|v| v.split_once('@'))
            .ok_or(())?;
        if credential.len() != 64
            || !credential.bytes().all(|b| b.is_ascii_hexdigit())
            || !matches!(
                host,
                "valkey.yt184.test:6379/" | "wrong.valkey.yt184.test:6379/"
            )
        {
            return Err(());
        }
        Ok(value.to_owned())
    }
}
pub(super) async fn canonical_origin(
    State(origin): State<String>,
    request: Request,
    next: Next,
) -> Response {
    let host_ok = request.headers().get_all(header::HOST).iter().count() == 1
        && request
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            == origin.strip_prefix("https://");
    let safe = matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    );
    let origins = request.headers().get_all(header::ORIGIN);
    let origin_ok = safe
        || (origins.iter().count() <= 1
            && origins
                .iter()
                .all(|v| v.to_str().ok() == Some(origin.as_str())));
    let mut response = if !host_ok || !origin_ok {
        fail(StatusCode::FORBIDDEN)
    } else {
        next.run(request).await
    };
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::VARY, "Cookie"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ] {
        response
            .headers_mut()
            .insert(name, axum::http::HeaderValue::from_static(value));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yt184_private_transport_inputs_reject_plaintext_and_public_files() {
        use std::{fs, os::unix::fs::OpenOptionsExt};
        struct Scratch(std::path::PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let dir = Scratch(std::env::temp_dir().join(format!(
                "yt184-input-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        fs::create_dir(&dir.0).unwrap();
        let write = |name: &str, value: &str| {
            let p = dir.0.join(name);
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&p)
                .unwrap();
            file.write_all(value.as_bytes()).unwrap();
            p.to_str().unwrap().to_owned()
        };
        let credential = "0".repeat(64);
        let ca = write("ca.pem", "test fixture CA metadata");
        let password_file = write("password", &credential);
        let endpoint_file = write(
            "endpoint",
            &format!("rediss://yt184_app:{credential}@valkey.yt184.test:6379/"),
        );
        let value = Transport {
            pg_host: "sql.yt184.test".into(),
            ca_file: ca,
            pg_password_file: password_file.clone(),
            valkey_url_file: endpoint_file.clone(),
            namespace: "yt184_test_e1".into(),
            port: 39270,
            origin: "https://auth.yt184.test:8443".into(),
        };
        assert!(value.validate().is_ok());
        fs::write(
            &endpoint_file,
            format!("redis://yt184_app:{credential}@valkey.yt184.test:6379/"),
        )
        .unwrap();
        assert!(value.validate().is_err());
        fs::write(
            &endpoint_file,
            format!("rediss://yt184_app:{credential}@valkey.yt184.test:6379/"),
        )
        .unwrap();
        fs::set_permissions(&password_file, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(value.validate().is_err());
        fs::set_permissions(&password_file, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&password_file, "x".repeat(257)).unwrap();
        assert!(value.validate().is_err());
    }
    #[tokio::test]
    async fn yt184_origin_refusals_keep_private_headers() {
        use tower::ServiceExt;
        let app =
            Router::new()
                .route("/", get(|| async { "ok" }))
                .layer(middleware::from_fn_with_state(
                    "https://auth.yt184.test:8443".to_owned(),
                    canonical_origin,
                ));
        let request = Request::builder()
            .uri("/")
            .header(header::HOST, "other.test:8443")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(response.headers()[header::VARY], "Cookie");
        assert_eq!(
            response.headers()[header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
    }
    #[tokio::test]
    async fn yt184_canonical_origin_rejects_spoofed_and_duplicate_headers() {
        use tower::ServiceExt;
        let router =
            Router::new()
                .route("/", get(|| async { "ok" }))
                .layer(middleware::from_fn_with_state(
                    "https://auth.yt184.test:8443".to_owned(),
                    canonical_origin,
                ));
        for (host, origin, expected) in [
            ("auth.yt184.test:8443", None, StatusCode::OK),
            ("other.test:8443", None, StatusCode::FORBIDDEN),
            ("auth.yt184.test:8443", Some("null"), StatusCode::FORBIDDEN),
            (
                "auth.yt184.test:8443",
                Some("https://auth.yt184.test:8443"),
                StatusCode::OK,
            ),
        ] {
            let mut request = Request::builder()
                .method("POST")
                .uri("/")
                .header(header::HOST, host);
            if let Some(value) = origin {
                request = request.header(header::ORIGIN, value);
            }
            let response = router
                .clone()
                .oneshot(request.body(axum::body::Body::empty()).unwrap())
                .await
                .unwrap();
            // Route method is immaterial: accepted POST reaches native405; rejected input gets403.
            assert_eq!(
                response.status(),
                if expected == StatusCode::OK {
                    StatusCode::METHOD_NOT_ALLOWED
                } else {
                    expected
                }
            );
        }
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header(header::HOST, "auth.yt184.test:8443")
            .header(header::ORIGIN, "https://auth.yt184.test:8443")
            .header(header::ORIGIN, "https://auth.yt184.test:8443")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            router.oneshot(request).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
    }
}
