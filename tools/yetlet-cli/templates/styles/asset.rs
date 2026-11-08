//! App-owned immutable CSS snapshot. Requests never read files or run generators.
use axum::{Extension, Router, body::Bytes, http::{header, HeaderValue, StatusCode}, response::{Html, IntoResponse, Response}, routing::get};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs::File, io::{self, Read}, path::{Path, Component}, sync::Arc};
use yetlet::{server_axum::{render_document, DEFAULT_CSP}, view::View};
const LIMIT: usize = 2 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record { logical: String, file: String, sha256: String }
pub struct Stylesheet { pub(crate) path: String, bytes: Bytes }
fn invalid() -> io::Error { io::Error::other("generated CSS unavailable/unaccepted; run yetlet build or run and inspect private style evidence") }
fn read(root: &Path, relative: &str, limit: usize) -> io::Result<Vec<u8>> {
    let mut path = root.to_owned();
    for component in Path::new(relative).components() {
        if !matches!(component,Component::Normal(_)) { return Err(invalid()); }
        path.push(component);
        let metadata = path.symlink_metadata()?;
        if metadata.file_type().is_symlink() { return Err(invalid()); }
        #[cfg(windows)] { use std::os::windows::fs::MetadataExt; if metadata.file_attributes() & 0x400 != 0 { return Err(invalid()); } }
        if path != root.join(relative) && !metadata.is_dir() { return Err(invalid()); }
    }
    if !path.symlink_metadata()?.is_file() { return Err(invalid()); }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() { return Err(invalid()); }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit { return Err(invalid()); }
    Ok(bytes)
}
impl Stylesheet {
    pub fn load() -> io::Result<Arc<Self>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        // Pending evidence is authoritative even if the new record and CSS match.
        // This adapter shares the coordinator's trusted-owner startup assumptions.
        for name in [".yetlet/styles/owner.pending.json", ".yetlet/styles/production.pending.json"] {
            match root.join(name).symlink_metadata() {
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                _ => return Err(invalid()),
            }
        }
        let development = std::env::var_os("YETLET_STYLE_DEV").as_deref() == Some(std::ffi::OsStr::new("1"));
        let record: Record = serde_json::from_slice(&read(root, if development {".yetlet/styles/owner.json"} else {".yetlet/styles/production.json"}, 65536)?).map_err(|_|invalid())?;
        if record.logical != "app-css" || record.sha256.len()!=64 || !record.sha256.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) { return Err(invalid()); }
        let expected = if development { "assets/generated/app.generated.css".into() } else {format!("assets/generated/app.{}.css",record.sha256)};
        if record.file != expected { return Err(invalid()); }
        let bytes = read(root,&record.file,LIMIT)?;
        if bytes.is_empty() || std::str::from_utf8(&bytes).is_err() || format!("{:x}",Sha256::digest(&bytes)) != record.sha256 { return Err(invalid()); }
        Ok(Arc::new(Self {path:format!("/{}",record.file),bytes:Bytes::from(bytes)}))
    }
    pub fn page(&self, title: &str, view: View) -> Response {
        let mut response = match render_document(title,&view) {
            Ok(document) => Html(document.replacen("</head>",&format!("<link rel=\"stylesheet\" href=\"{}\"></head>",self.path),1)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR,"Unable to render the page").into_response(),
        };
        response.headers_mut().insert(header::CONTENT_SECURITY_POLICY,HeaderValue::from_static(DEFAULT_CSP));
        response.headers_mut().insert(header::CACHE_CONTROL,HeaderValue::from_static("no-store"));
        response
    }
    fn response(&self) -> Response {
        ([(header::CONTENT_TYPE,"text/css; charset=utf-8"),(header::X_CONTENT_TYPE_OPTIONS,"nosniff"),(header::CACHE_CONTROL,"no-store"),(header::CONTENT_SECURITY_POLICY,DEFAULT_CSP)],self.bytes.clone()).into_response()
    }
}
pub fn install(router: Router, stylesheet: Arc<Stylesheet>) -> Router {
    let css = stylesheet.clone();
    router.route(&stylesheet.path,get(move || { let css=css.clone(); async move {css.response()} })).layer(Extension(stylesheet))
}
