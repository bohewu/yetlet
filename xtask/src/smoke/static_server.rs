//! Owned static fixture server used by the original keyed suites.
use crate::tooling::{Result, fail};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
pub struct StaticServer {
    pub base: String,
    thread: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}
impl StaticServer {
    pub fn start(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let server = tiny_http::Server::http(("127.0.0.1", 0))
            .map_err(|_| fail("static fixture listener unavailable"))?;
        let address = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| fail("static fixture address"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                match server.recv_timeout(Duration::from_millis(100)) {
                    Ok(Some(request)) => {
                        let response = match fixture_path(&root, request.url())
                            .and_then(|p| std::fs::read(&p).map(|b| (p, b)).map_err(Into::into))
                        {
                            Ok((path, bytes)) => {
                                let mime = match path.extension().and_then(|x| x.to_str()) {
                                    Some("js" | "mjs") => "text/javascript",
                                    Some("wasm") => "application/wasm",
                                    Some("html") => "text/html",
                                    Some("json") => "application/json",
                                    Some("css") => "text/css",
                                    _ => "application/octet-stream",
                                };
                                tiny_http::Response::from_data(bytes).with_header(
                                    tiny_http::Header::from_bytes("Content-Type", mime).unwrap(),
                                )
                            }
                            Err(_) => tiny_http::Response::from_data(b"not found".to_vec())
                                .with_status_code(404),
                        };
                        let _ = request.respond(response);
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            base: format!("http://{address}"),
            thread: Some(thread),
            stop,
        })
    }
    pub fn close(&mut self) -> Result<()> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| fail("static fixture thread failed"))?;
        }
        Ok(())
    }
}
impl Drop for StaticServer {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
pub fn fixture_path(root: &Path, url: &str) -> Result<PathBuf> {
    let path = percent_encoding::percent_decode_str(url.split('?').next().unwrap_or(""))
        .decode_utf8()
        .map_err(|_| fail("static path encoding"))?;
    let relative = path.strip_prefix('/').ok_or_else(|| fail("static path"))?;
    let relative = if relative.is_empty() {
        "index.html"
    } else {
        relative
    };
    let result = crate::tooling::safe(root, relative, false)?;
    if result.is_file() {
        Ok(result)
    } else {
        Err(fail("static fixture file missing"))
    }
}
