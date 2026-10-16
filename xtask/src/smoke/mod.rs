//! Internal ports of the active native/browser suites, with suite-owned resources.
pub mod board;
pub mod browser;
pub mod dashboard;
pub mod fixtures;
pub mod forms;
pub mod http;
pub mod keyed;
pub mod native;
pub mod png;
pub mod proof;
pub mod scene;
mod scene_candidate;
mod scene_lifecycle;
mod scene_picking;
pub mod static_server;
pub mod todo;
pub mod websocket;
use crate::{
    process::{Service, Spec},
    tooling::{self, Result, fail},
};
use std::{
    fs,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub fn free_port() -> Result<u16> {
    Ok(TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| fail(format!("loopback socket unavailable: {:?}", e.kind())))?
        .local_addr()?
        .port())
}
pub fn wait<T>(
    seconds: u64,
    label: &str,
    mut check: impl FnMut() -> Result<Option<T>>,
) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        tooling::check_cancel()?;
        if let Some(value) = check()? {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            return Err(tooling::Error {
                code: 124,
                message: format!("timeout: {label}"),
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
pub fn require(condition: bool, label: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(fail(label)) }
}
pub fn profile(args: &[std::ffi::OsString], default: &str) -> Result<String> {
    if args.is_empty() {
        return Ok(default.into());
    }
    if args.len() == 2 && args[0] == "--profile" && (args[1] == "debug" || args[1] == "release") {
        return Ok(args[1].to_string_lossy().into_owned());
    }
    Err(tooling::Error {
        code: 2,
        message: "expected --profile debug|release".into(),
    })
}
/// Fresh evidence directory per invocation; never writes historical M11 paths.
pub fn evidence(root: &Path, suite: &str) -> Result<PathBuf> {
    let base = root.join("target/m12/yt085");
    fs::create_dir_all(&base)?;
    let dir = tempfile::Builder::new()
        .prefix(&format!("{suite}-"))
        .tempdir_in(base)?;
    Ok(dir.keep())
}
/// Recursive original report inventory, also reused for current Rust build inputs.
pub fn source_hashes(root: &Path, directory: &Path) -> Result<serde_json::Value> {
    fn visit(
        root: &Path,
        dir: &Path,
        hashes: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, hashes)?;
            } else if path.is_file() {
                let name = path
                    .strip_prefix(root)
                    .map_err(|_| fail("source outside root"))?
                    .to_str()
                    .ok_or_else(|| fail("source path UTF-8"))?
                    .replace('\\', "/");
                hashes.insert(name, serde_json::json!(tooling::sha(&fs::read(path)?)));
            }
        }
        Ok(())
    }
    let mut hashes = serde_json::Map::new();
    visit(root, directory, &mut hashes)?;
    Ok(serde_json::Value::Object(hashes))
}
pub fn head(root: &Path) -> Result<String> {
    let bytes = tooling::execute(
        root,
        std::ffi::OsStr::new("git"),
        &["rev-parse".into(), "HEAD".into()],
        true,
    )?;
    Ok(String::from_utf8(bytes)
        .map_err(|_| fail("git HEAD UTF-8"))?
        .trim()
        .into())
}
pub fn rust_sources(root: &Path) -> Result<serde_json::Value> {
    let mut values = source_hashes(root, &root.join("xtask/src"))?;
    for name in [
        "xtask/Cargo.toml",
        "xtask/Cargo.lock",
        ".cargo/config.toml",
        "rust-toolchain.toml",
    ] {
        values[name] = serde_json::json!(tooling::sha(&tooling::read(root, name)?));
    }
    Ok(values)
}
pub fn unchanged(root: &Path, hashes: &serde_json::Value) -> bool {
    hashes.as_object().is_some_and(|entries| {
        entries.iter().all(|(p, h)| {
            tooling::read(root, p)
                .is_ok_and(|bytes| h.as_str() == Some(tooling::sha(&bytes).as_str()))
        })
    })
}
pub struct Server {
    pub process: Service,
    pub port: u16,
    pub base: String,
    pub log: PathBuf,
}
impl Server {
    pub fn start(
        root: &Path,
        folder: &Path,
        binary: &str,
        profile: &str,
        listen: &str,
        env: &[(&str, String)],
        ready: &str,
    ) -> Result<Self> {
        let executable = root
            .join("target")
            .join(profile)
            .join(format!("{binary}{}", std::env::consts::EXE_SUFFIX));
        Self::start_binary(root, folder, &executable, listen, env, ready)
    }
    pub fn start_binary(
        root: &Path,
        folder: &Path,
        executable: &Path,
        listen: &str,
        env: &[(&str, String)],
        ready: &str,
    ) -> Result<Self> {
        let mut server = Self::spawn_binary(root, folder, executable, listen, env)?;
        server.wait_ready(ready)?;
        Ok(server)
    }
    /// Callers with an ownership registry register this handle before readiness.
    pub fn spawn_binary(
        root: &Path,
        folder: &Path,
        executable: &Path,
        listen: &str,
        env: &[(&str, String)],
    ) -> Result<Self> {
        let port = free_port()?;
        require(executable.is_file(), "build selected native fixture first")?;
        let log = folder.join(format!("server-{port}.log"));
        let mut variables = vec![(listen.into(), Some(format!("127.0.0.1:{port}").into()))];
        variables.extend(env.iter().map(|(k, v)| ((*k).into(), Some(v.into()))));
        let spec = Spec {
            program: executable.as_os_str().into(),
            args: vec![],
            cwd: root.into(),
            env: variables,
            timeout: Duration::from_secs(120),
            output_limit: 1024 * 1024,
        };
        let process = Service::spawn(&spec, &log).map_err(|_| tooling::Error {
            code: 125,
            message: "server spawn failed".into(),
        })?;
        Ok(Self {
            process,
            port,
            base: format!("http://127.0.0.1:{port}"),
            log,
        })
    }
    /// Failed readiness still records the observed child status after owned cleanup.
    pub fn wait_ready(&mut self, ready: &str) -> Result<()> {
        let mut exited_early = false;
        let result = wait(12, "server readiness", || {
            exited_early = self.process.exited()?;
            require(!exited_early, "server exited before readiness")?;
            Ok(http::get(&self.base, ready)
                .ok()
                .filter(|r| r.status == 200))
        });
        if let Err(mut error) = result {
            let status = self.finish_stop(false)?;
            if exited_early {
                let report = status_report(status);
                // A child exiting 0 before serving is still a readiness failure.
                error.code = if report.exit_code() == 0 {
                    1
                } else {
                    report.exit_code()
                };
            }
            return Err(error);
        }
        Ok(())
    }
    pub fn stop(&mut self, graceful: bool) -> Result<()> {
        self.finish_stop(graceful).map(|_| ())
    }
    fn finish_stop(&mut self, graceful: bool) -> Result<std::process::ExitStatus> {
        let observed = self
            .process
            .stop(graceful, Duration::from_secs(if graceful { 15 } else { 4 }));
        let listener_closed = TcpStream::connect(("127.0.0.1", self.port)).is_err();
        let stream_closed = self.process.stream_closed();
        let report = observed.as_ref().ok().map(|status| status_report(*status));
        // Persist facts before any failure return, including a nonzero graceful
        // exit. A missing native status is unknown, never invented as zero.
        fs::write(
            self.log.with_extension("exit.json"),
            tooling::json_bytes(&serde_json::json!({
                "native_exit_code": report.as_ref().and_then(|r| r.native_code()),
                "signal": report.as_ref().and_then(|r| r.signal()),
                "status": observed.as_ref().ok().map(ToString::to_string),
                "listener_closed": listener_closed,
                "stream_closed": stream_closed,
                "owned_tree_reaped": observed.is_ok(),
                "cleanup_error": observed.as_ref().err().map(|e| format!("{:?}", e.kind())),
            })),
        )
        .map_err(|_| tooling::Error {
            code: 125,
            message: "owned server exit receipt failed".into(),
        })?;
        let status = observed.map_err(|_| tooling::Error {
            code: 125,
            message: "owned server cleanup failed".into(),
        })?;
        if !listener_closed || !stream_closed {
            return Err(tooling::Error {
                code: 125,
                message: "owned server cleanup incomplete".into(),
            });
        }
        if graceful && !status.success() {
            return Err(tooling::Error {
                code: status_report(status).exit_code(),
                message: "server did not shut down gracefully".into(),
            });
        }
        Ok(status)
    }
}
fn status_report(status: std::process::ExitStatus) -> crate::process::Report {
    crate::process::Report {
        origin: crate::process::Origin::Child,
        child_status: Some(status),
        stdout: Default::default(),
        stderr: Default::default(),
        error: None,
        cleanup_ok: true,
    }
}
pub fn remove_owned_tree(path: &Path, seconds: u64) -> std::io::Result<()> {
    remove_tree_with(path, Duration::from_secs(seconds), |p| {
        fs::remove_dir_all(p)
    })
}
pub fn remove_tree_with(
    path: &Path,
    budget: Duration,
    mut remove: impl FnMut(&Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let end = Instant::now() + budget;
    loop {
        match remove(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                if Instant::now() >= end {
                    return Err(e);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

pub const BROWSER_SUITES: &[&str] = &[
    "browser-chrome",
    "browser-firefox",
    "keyed-bindings-chrome",
    "keyed-bindings-firefox",
    "multi-island-dashboard-chrome",
    "multi-island-dashboard-firefox",
    "forms-accessibility-firefox",
    "form-css-proof-chrome",
    "board-chrome",
    "board-firefox",
    "scene-chrome",
    "scene-firefox",
    "scene-lifecycle-firefox",
    "scene-picking-firefox",
    "scene-candidate-firefox",
    "scene-support-firefox",
];
pub fn default_profile(suite: &str) -> &'static str {
    if matches!(
        suite,
        "browser-chrome" | "multi-island-dashboard-chrome" | "multi-island-dashboard-firefox"
    ) {
        "debug"
    } else {
        "release"
    }
}
pub fn run(root: &Path, suite: &str, args: &[std::ffi::OsString], out: &Path) -> Result<()> {
    if native::SUITES.contains(&suite) {
        return native::run(root, suite, out);
    }
    if suite.starts_with("scene-") {
        return scene::run(root, out, suite);
    }
    if suite.starts_with("board-") {
        return board::run(root, out, suite.ends_with("firefox"));
    }
    let profile = profile(args, default_profile(suite))?;
    match suite {
        "browser-chrome" | "browser-firefox" => {
            todo::run(root, out, &profile, suite.ends_with("firefox"))
        }
        "keyed-bindings-chrome" | "keyed-bindings-firefox" => {
            keyed::run(root, out, &profile, suite.ends_with("firefox"))
        }
        "multi-island-dashboard-chrome" | "multi-island-dashboard-firefox" => {
            dashboard::run(root, out, &profile, suite.ends_with("firefox"))
        }
        "forms-accessibility-firefox" => forms::run(root, out, &profile),
        "form-css-proof-chrome" => proof::run(root, out),
        _ => Err(fail("unknown smoke suite")),
    }
}
