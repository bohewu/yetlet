use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use tower::ServiceExt;

fn experiment() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn vite_adapter_rejects_untrusted_schema_metadata_and_references() {
    use serde_json::{Value, json};
    let build = assets::tests::Build::new();
    std::fs::create_dir(build.0.join(".vite")).unwrap();
    let manifest = build.0.join(".vite/manifest.json");
    for bytes in [b"[]".as_slice(), b"null", b"{", br#"{"app":{"file":5}}"#] {
        std::fs::write(&manifest, bytes).unwrap();
        assert!(matches!(
            vite::load(&build.0, "app"),
            Err("invalid Vite manifest")
        ));
    }
    let actual: Value = serde_json::from_slice(
        &std::fs::read(experiment().join("../vite-integration/dist/plain/.vite/manifest.json"))
            .unwrap(),
    )
    .unwrap();
    for (field, bad, expected) in [
        ("src", json!("../private"), "unsafe Vite metadata"),
        ("names", json!(["<script>"]), "unsafe Vite metadata"),
        (
            "file",
            json!("assets/../../private.js"),
            "unsafe asset path",
        ),
        ("surprise", json!(true), "invalid Vite manifest"),
        ("imports", json!(["missing"]), "missing graph reference"),
        (
            "dynamicImports",
            json!(["missing"]),
            "missing graph reference",
        ),
    ] {
        let mut value = actual.clone();
        value["front/plain.ts"][field] = bad;
        std::fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            matches!(vite::load(&build.0, "front/plain.ts"), Err(error) if error == expected),
            "{field}"
        );
    }
}
#[test]
fn actual_producers_use_one_contract_and_keep_entries_isolated() {
    for name in ["plain", "tailwind"] {
        let root = experiment().join("../vite-integration/dist").join(name);
        let snapshot = vite::load(&root, &format!("front/{name}.ts")).unwrap();
        let html = document(name, &snapshot).unwrap();
        assert!(html.contains("&lt;safe &amp; useful&gt;"));
        assert!(html.contains("type=\"module\""));
        assert!(!snapshot.links(name).unwrap().contains("lazy-"));
        assert_eq!(snapshot.images().len(), 1);
        assert!(
            snapshot.links(name).unwrap().find("shared-").unwrap()
                < snapshot.links(name).unwrap().find("app-").unwrap()
        );
        let companion = vite::load(&root, "front/companion.ts").unwrap();
        let links = companion.links(name).unwrap();
        assert!(links.contains("companion-"));
        assert!(!links.contains("app-"));
        if let Ok(secret) = std::env::var("BACKEND_ONLY_SECRET") {
            assert!(!secret.is_empty());
            assert!(!html.contains(&secret));
            for entry in std::fs::read_dir(root.join("assets")).unwrap() {
                let bytes = std::fs::read(entry.unwrap().path()).unwrap();
                assert!(!bytes.windows(secret.len()).any(|v| v == secret.as_bytes()));
            }
        }
    }
    let snapshot = standalone::load(&experiment().join("dist/standalone")).unwrap();
    let css = &snapshot.asset("assets/standalone.css").unwrap().bytes;
    assert!(css.starts_with(b"/*! tailwindcss v4.3.3"));
    assert!(String::from_utf8_lossy(css).contains(".font-bold"));
    let html = document("standalone", &snapshot).unwrap();
    assert!(html.contains("&lt;safe &amp; useful&gt;"));
    assert!(!html.contains("<script"));
    assert_eq!(snapshot.cache_control(), "no-store");
}
#[tokio::test]
async fn actual_assets_http_csp_whitelist_and_missing_disk_regression_without_listener() {
    let snapshot = standalone::load(&experiment().join("dist/standalone")).unwrap();
    let html = document("standalone", &snapshot).unwrap();
    let app = Router::new()
        .fallback(request)
        .with_state(Arc::new(BTreeMap::from([(
            "standalone".into(),
            Profile { snapshot, html },
        )])));
    for (path, expected, mime) in [
        ("/standalone", 200, "text/html; charset=utf-8"),
        (
            "/standalone/assets/standalone.css",
            200,
            "text/css; charset=utf-8",
        ),
        (
            "/standalone/assets/standalone.css?x=1",
            404,
            "text/plain; charset=utf-8",
        ),
        (
            "/standalone/.vite/manifest.json",
            404,
            "text/plain; charset=utf-8",
        ),
        (
            "/standalone/assets/%2e%2e%2fCargo.toml",
            404,
            "text/plain; charset=utf-8",
        ),
        (
            "/standalone/assets/not-listed.svg",
            404,
            "text/plain; charset=utf-8",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected, "{path}");
        assert_eq!(response.headers()[header::CONTENT_TYPE], mime);
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
        assert!(
            !to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
