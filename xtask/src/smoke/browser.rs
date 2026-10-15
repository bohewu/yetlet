//! Actual CDP and Firefox WebDriver/BiDi protocols used by migrated suites.
use super::{free_port, http, remove_owned_tree, require, wait, websocket::Socket};
use crate::{
    process::{Service, Spec},
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    net::TcpStream,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct Cdp {
    socket: Socket,
    next: u64,
    pub requests: Vec<String>,
    pub api_posts: Vec<Value>,
    pub websockets: Vec<String>,
    pub exceptions: Vec<String>,
    pub failed_responses: Vec<String>,
}
impl Cdp {
    pub fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            socket: Socket::connect(url)?,
            next: 0,
            requests: vec![],
            api_posts: vec![],
            websockets: vec![],
            exceptions: vec![],
            failed_responses: vec![],
        })
    }
    pub fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next += 1;
        let id = self.next;
        self.socket
            .send(&json!({"id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            let message = self.socket.receive(deadline)?;
            self.event(&message);
            if message["id"] == id {
                require(
                    message.get("error").is_none(),
                    &format!("CDP {method} failed: {}", message["error"]),
                )?;
                return Ok(message.get("result").cloned().unwrap_or(json!({})));
            }
        }
    }
    pub fn event(&mut self, message: &Value) {
        let p = &message["params"];
        match message["method"].as_str() {
            Some("Network.requestWillBeSent") => {
                let r = &p["request"];
                if let Some(url) = r["url"].as_str() {
                    self.requests.push(url.into());
                    if r["method"] == "POST" {
                        self.api_posts.push(json!({"url":url,"method":"POST","postData":r["postData"].as_str().unwrap_or("")}));
                    }
                }
            }
            Some("Network.responseReceived") => {
                let r = &p["response"];
                if r["status"].as_f64().is_some_and(|s| s >= 400.0) {
                    self.failed_responses.push(format!(
                        "{} {}",
                        r["status"],
                        r["url"].as_str().unwrap_or("")
                    ));
                }
            }
            Some("Network.webSocketCreated") => {
                self.websockets.push(p["url"].as_str().unwrap_or("").into())
            }
            Some("Runtime.exceptionThrown") => self.exceptions.push(
                p["exceptionDetails"]["text"]
                    .as_str()
                    .unwrap_or("JS exception")
                    .into(),
            ),
            Some("Log.entryAdded") if p["entry"]["level"] == "error" => self.exceptions.push(
                p["entry"]["text"]
                    .as_str()
                    .unwrap_or("Browser log error")
                    .into(),
            ),
            _ => {}
        }
    }
    pub fn evaluate(&mut self, expression: &str) -> Result<Value> {
        let data = self.call(
            "Runtime.evaluate",
            json!({"expression":expression,"returnByValue":true,"awaitPromise":true}),
        )?;
        require(
            data.get("exceptionDetails").is_none(),
            "CDP JS evaluation exception",
        )?;
        Ok(data["result"].get("value").cloned().unwrap_or(Value::Null))
    }
}
pub struct Firefox {
    pub base: String,
    pub session: Option<String>,
    pub capabilities: Value,
    pub deadline: Option<Instant>,
}
impl Firefox {
    pub fn call(&self, path: &str, method: &str, body: Option<&Value>) -> Result<Value> {
        let seconds = if let Some(deadline) = self.deadline {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(tooling::Error {
                    code: 124,
                    message: "Firefox suite deadline".into(),
                });
            }
            (left.as_secs_f64().ceil() as u64).min(30)
        } else {
            30
        };
        let bytes = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|_| fail("WebDriver JSON encode"))?;
        let response = http::call_timeout(
            &self.base,
            method,
            path,
            bytes.as_deref(),
            &[("Content-Type", "application/json")],
            seconds,
        );
        if self.deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(tooling::Error {
                code: 124,
                message: "Firefox suite deadline".into(),
            });
        }
        let r = response?;
        let value = r.json()?["value"].clone();
        require(
            r.status < 400 && value.get("error").is_none(),
            "Firefox WebDriver command failed",
        )?;
        Ok(value)
    }
    pub fn create_session(&mut self, options: Value, bidi: bool) -> Result<String> {
        let mut caps = json!({"browserName":"firefox","moz:firefoxOptions":options});
        if bidi {
            caps["webSocketUrl"] = json!(true);
        }
        let answer = self.call(
            "/session",
            "POST",
            Some(&json!({"capabilities":{"alwaysMatch":caps}})),
        )?;
        self.session = Some(
            answer["sessionId"]
                .as_str()
                .ok_or_else(|| fail("missing Firefox session"))?
                .into(),
        );
        self.capabilities = answer["capabilities"].clone();
        Ok(self.capabilities["browserVersion"]
            .as_str()
            .unwrap_or("unknown")
            .into())
    }
    pub fn command(&self, path: &str, method: &str, body: Option<&Value>) -> Result<Value> {
        self.call(
            &format!(
                "/session/{}{path}",
                self.session
                    .as_ref()
                    .ok_or_else(|| fail("Firefox no session"))?
            ),
            method,
            body,
        )
    }
    pub fn evaluate(&self, expression: &str) -> Result<Value> {
        self.command(
            "/execute/sync",
            "POST",
            Some(&json!({"script":format!("return ({expression});"),"args":[]})),
        )
    }
    pub fn async_script(&self, script: &str) -> Result<Value> {
        self.command(
            "/execute/async",
            "POST",
            Some(&json!({"script":script,"args":[]})),
        )
    }
    pub fn element(&self, selector: &str) -> Result<String> {
        let v = self.command(
            "/element",
            "POST",
            Some(&json!({"using":"css selector","value":selector})),
        )?;
        Ok(v["element-6066-11e4-a52e-4f735466cecf"]
            .as_str()
            .ok_or_else(|| fail("WebDriver element id"))?
            .into())
    }
    pub fn click(&self, selector: &str) -> Result<()> {
        self.command(
            &format!("/element/{}/click", self.element(selector)?),
            "POST",
            Some(&json!({})),
        )?;
        Ok(())
    }
    pub fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        self.command(
            &format!("/element/{}/value", self.element(selector)?),
            "POST",
            Some(&json!({"text":text})),
        )?;
        Ok(())
    }
    pub fn clear(&self, selector: &str) -> Result<()> {
        self.command(
            &format!("/element/{}/clear", self.element(selector)?),
            "POST",
            Some(&json!({})),
        )?;
        Ok(())
    }
    pub fn keys(&self, text: &str) -> Result<()> {
        let actions = text
            .chars()
            .flat_map(|key| {
                [
                    json!({"type":"keyDown","value":key.to_string()}),
                    json!({"type":"keyUp","value":key.to_string()}),
                ]
            })
            .collect::<Vec<_>>();
        self.command(
            "/actions",
            "POST",
            Some(&json!({"actions":[{"type":"key","id":"forms-keyboard","actions":actions}]})),
        )?;
        Ok(())
    }
    pub fn replace_text(&self, text: &str) -> Result<()> {
        self.command("/actions","POST",Some(&json!({"actions":[{"type":"key","id":"forms-keyboard","actions":[{"type":"keyDown","value":"\u{e009}"},{"type":"keyDown","value":"a"},{"type":"keyUp","value":"a"},{"type":"keyUp","value":"\u{e009}"}]}]})))?;
        self.keys(&format!("\u{e003}{text}"))
    }
    pub fn close(&mut self) -> Result<()> {
        if let Some(id) = self.session.take() {
            self.call(&format!("/session/{id}"), "DELETE", None)?;
        }
        Ok(())
    }
}
pub struct Bidi {
    socket: Socket,
    next: u64,
    pub events: Vec<Value>,
}
impl Bidi {
    pub fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            socket: Socket::connect_bidi(url)?,
            next: 0,
            events: vec![],
        })
    }
    pub fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next += 1;
        let id = self.next;
        self.socket
            .send(&json!({"id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let m = self.socket.receive(deadline)?;
            if m["id"] == id {
                require(m["type"] != "error", "BiDi command failed")?;
                return Ok(m["result"].clone());
            }
            self.events.push(m);
        }
    }
}
pub enum Client {
    Chrome(Cdp),
    Firefox(Firefox),
}
impl Client {
    pub fn evaluate(&mut self, js: &str) -> Result<Value> {
        match self {
            Self::Chrome(c) => c.evaluate(js),
            Self::Firefox(f) => f.evaluate(js),
        }
    }
    pub fn navigate(&mut self, url: &str) -> Result<()> {
        match self {
            Self::Chrome(c) => {
                c.call("Page.navigate", json!({"url":url}))?;
            }
            Self::Firefox(f) => {
                f.command("/url", "POST", Some(&json!({"url":url})))?;
            }
        }
        Ok(())
    }
    pub fn check(&mut self, js: &str) -> Result<()> {
        require(
            self.evaluate(js)? == true,
            &format!("browser assertion: {js}"),
        )
    }
    pub fn equal(&mut self, js: &str, expected: Value) -> Result<()> {
        require(
            self.evaluate(js)? == expected,
            &format!("browser value: {js}"),
        )
    }
    pub fn until(&mut self, js: &str, label: &str) -> Result<()> {
        wait(18, label, || {
            Ok(self.evaluate(js).ok().filter(|v| v == true).map(|_| ()))
        })
    }
    pub fn click(&mut self, selector: &str) -> Result<()> {
        match self {
            Self::Firefox(f) => f.click(selector),
            Self::Chrome(c) => {
                c.evaluate(&format!(
                    "document.querySelector({}).click()",
                    json!(selector)
                ))?;
                Ok(())
            }
        }
    }
    pub fn close(&mut self) -> Result<()> {
        if let Self::Firefox(f) = self {
            f.close()?;
        }
        Ok(())
    }
    pub fn chrome(&mut self) -> Result<&mut Cdp> {
        match self {
            Self::Chrome(c) => Ok(c),
            _ => Err(fail("Chrome-only operation")),
        }
    }
    pub fn firefox(&mut self) -> Result<&mut Firefox> {
        match self {
            Self::Firefox(f) => Ok(f),
            _ => Err(fail("Firefox-only operation")),
        }
    }
}
pub struct Browser {
    pub client: Option<Client>,
    process: Service,
    port: u16,
    profile: PathBuf,
    pub version: String,
    log: PathBuf,
    last_status: Option<std::process::ExitStatus>,
    stopped: bool,
}
/// Only suite-selected launch differences; ordinary browser ports keep defaults.
#[derive(Default)]
pub struct Launch {
    pub chrome_args: Option<Vec<std::ffi::OsString>>,
    pub env: Vec<(std::ffi::OsString, Option<std::ffi::OsString>)>,
    pub firefox_options: Option<Value>,
    pub bidi: bool,
    pub profile_parent: Option<PathBuf>,
}
impl Browser {
    pub fn start(root: &Path, out: &Path, firefox: bool) -> Result<Self> {
        Self::start_config(root, out, firefox, Launch::default())
    }
    pub fn start_config(root: &Path, out: &Path, firefox: bool, config: Launch) -> Result<Self> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("yt085-browser-");
        let profile = if let Some(parent) = &config.profile_parent {
            builder.tempdir_in(parent)
        } else {
            builder.tempdir()
        }
        .map_err(|_| tooling::Error {
            code: 125,
            message: "browser profile creation failed".into(),
        })?
        .keep();
        let result = Self::launch(root, out, firefox, profile.clone(), config);
        if result.is_err() {
            remove_owned_tree(&profile, 5).map_err(|_| tooling::Error {
                code: 125,
                message: "browser profile cleanup failed".into(),
            })?;
        }
        result
    }
    fn launch(
        root: &Path,
        out: &Path,
        firefox: bool,
        profile: PathBuf,
        config: Launch,
    ) -> Result<Self> {
        let port = free_port().map_err(|_| tooling::Error {
            code: 125,
            message: "browser port allocation failed".into(),
        })?;
        let program = if firefox {
            tooling::tool("geckodriver", "/snap/bin")?
        } else {
            ["google-chrome-stable", "google-chrome", "chromium"]
                .iter()
                .find_map(|n| tooling::tool(n, ".local/bin").ok())
                .ok_or_else(|| fail("Chrome unavailable; no installation attempted"))?
        };
        let mut args = if firefox {
            vec![
                "--host".into(),
                "127.0.0.1".into(),
                "--port".into(),
                port.to_string().into(),
            ]
        } else {
            vec![
                "--headless=new".into(),
                "--disable-gpu".into(),
                "--disable-background-networking".into(),
                "--no-first-run".into(),
                "--disable-extensions".into(),
                "--remote-debugging-address=127.0.0.1".into(),
                "--remote-allow-origins=http://127.0.0.1".into(),
                format!("--remote-debugging-port={port}").into(),
                format!("--user-data-dir={}", profile.display()).into(),
                "about:blank".into(),
            ]
        };
        if !firefox && let Some(extra) = &config.chrome_args {
            // Replace only the ordinary headless/disable-gpu prefix for GPU suites.
            args.drain(..2);
            args.splice(0..0, extra.iter().cloned());
        }
        let mut env = vec![("TMPDIR".into(), Some(profile.clone().into_os_string()))];
        env.extend(config.env.clone());
        let spec = Spec {
            program,
            args,
            cwd: root.into(),
            env,
            timeout: Duration::from_secs(120),
            output_limit: 1024 * 1024,
        };
        let mut browser = Self::spawn(
            &spec,
            out.join(if firefox {
                "geckodriver.log"
            } else {
                "chrome.log"
            }),
            profile,
            port,
        )?;
        browser.connect(firefox, config)?;
        Ok(browser)
    }
    fn spawn(spec: &Spec, log: PathBuf, profile: PathBuf, port: u16) -> Result<Self> {
        let process = Service::spawn(spec, &log).map_err(|_| tooling::Error {
            code: 125,
            message: "browser process spawn failed".into(),
        })?;
        Ok(Self {
            client: None,
            process,
            port,
            profile,
            version: String::new(),
            log,
            last_status: None,
            stopped: false,
        })
    }
    fn connect(&mut self, firefox: bool, config: Launch) -> Result<()> {
        let base = format!("http://127.0.0.1:{}", self.port);
        let mut exited_early = false;
        let startup = (|| {
            if firefox {
                let mut driver = Firefox {
                    base,
                    session: None,
                    capabilities: Value::Null,
                    deadline: None,
                };
                wait(15, "geckodriver readiness", || {
                    exited_early = self.exited_before_readiness()?;
                    require(!exited_early, "geckodriver exited before readiness")?;
                    Ok(driver
                        .call("/status", "GET", None)
                        .ok()
                        .filter(|v| v["ready"] == true))
                })?;
                // Store ownership before session creation can partially fail.
                self.client = Some(Client::Firefox(driver));
                driver = match self.client.take().unwrap() {
                    Client::Firefox(d) => d,
                    _ => unreachable!(),
                };
                let result = driver.create_session(
                    config
                        .firefox_options
                        .unwrap_or_else(|| json!({"args":["-headless"]})),
                    config.bidi,
                );
                self.client = Some(Client::Firefox(driver));
                self.version = result?;
            } else {
                let url = wait(15, "Chrome debugger", || {
                    exited_early = self.exited_before_readiness()?;
                    require(!exited_early, "Chrome exited before readiness")?;
                    Ok(http::get(&base, "/json/list")
                        .ok()
                        .and_then(|r| r.json().ok())
                        .and_then(|v| {
                            v.as_array()
                                .and_then(|a| a.iter().find(|p| p["type"] == "page"))
                                .and_then(|p| p["webSocketDebuggerUrl"].as_str().map(String::from))
                        }))
                })?;
                let mut c = Cdp::connect(&url)?;
                self.version = c.call("Browser.getVersion", json!({}))?["product"]
                    .as_str()
                    .unwrap_or("unknown")
                    .into();
                for method in [
                    "Page.enable",
                    "Runtime.enable",
                    "Network.enable",
                    "Log.enable",
                ] {
                    c.call(method, json!({}))?;
                }
                self.client = Some(Client::Chrome(c));
            }
            Ok(())
        })();
        self.finish_startup(startup, exited_early)
    }
    fn finish_startup(&mut self, startup: Result<()>, exited_early: bool) -> Result<()> {
        if let Err(mut error) = startup {
            let status = self.finish_stop()?;
            if exited_early {
                error.code = super::status_report(status).exit_code();
                if error.code == 0 {
                    error.code = 1;
                }
            }
            return Err(error);
        }
        Ok(())
    }
    fn exited_before_readiness(&mut self) -> Result<bool> {
        self.process.exited().map_err(|_| tooling::Error {
            code: 125,
            message: "browser status observation failed".into(),
        })
    }
    pub fn stop(&mut self) -> Result<()> {
        self.finish_stop().map(|_| ())
    }
    fn finish_stop(&mut self) -> Result<std::process::ExitStatus> {
        let session = self.client.as_mut().map_or(Ok(()), Client::close);
        self.client.take();
        let process = self.process.stop(false, Duration::from_secs(5));
        self.last_status = process.as_ref().ok().copied();
        let profile = remove_owned_tree(&self.profile, 5);
        let listener_closed = TcpStream::connect(("127.0.0.1", self.port)).is_err();
        let stream_closed = self.process.stream_closed();
        let report = self.last_status.map(super::status_report);
        // Drop must not overwrite this attempt's receipt (including a failure).
        self.stopped = true;
        std::fs::write(
            self.log.with_extension("exit.json"),
            tooling::json_bytes(&json!({
                "native_exit_code": report.as_ref().and_then(|r| r.native_code()),
                "signal": report.as_ref().and_then(|r| r.signal()),
                "status": self.last_status.map(|s| s.to_string()),
                "listener_closed": listener_closed,
                "stream_closed": stream_closed,
                "owned_tree_reaped": process.is_ok(),
                "profile_removed": profile.is_ok(),
                "cleanup_error": process.as_ref().err().map(|e| format!("{:?}", e.kind())),
                "profile_cleanup_error": profile.as_ref().err().map(|e| format!("{:?}", e.kind())),
                "session_cleanup_failed": session.is_err(),
            })),
        )
        .map_err(|_| tooling::Error {
            code: 125,
            message: "owned browser exit receipt failed".into(),
        })?;
        let status = process.map_err(|_| tooling::Error {
            code: 125,
            message: "owned browser cleanup failed".into(),
        })?;
        if profile.is_err() || !listener_closed || !stream_closed || session.is_err() {
            return Err(tooling::Error {
                code: 125,
                message: "browser process/session/profile cleanup failed".into(),
            });
        }
        // stop() owns termination; its signal/nonzero native status is evidence,
        // not a validation failure. Readiness maps an already exited child above.
        Ok(status)
    }
    pub fn client(&mut self) -> Result<&mut Client> {
        self.client
            .as_mut()
            .ok_or_else(|| fail("browser client unavailable"))
    }
    pub fn exit_code(&self) -> Option<i32> {
        self.last_status.and_then(|status| status.code())
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn pid(&self) -> u32 {
        self.process.id()
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        if !self.stopped {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn subprocess_fixture() {
        let Ok(mode) = std::env::var("YT085_BROWSER_STATUS_FIXTURE") else {
            return;
        };
        match mode.as_str() {
            "early" => std::process::exit(23),
            "graceful" => std::process::exit(17),
            "zero" => std::process::exit(0),
            #[cfg(unix)]
            "signal" => unsafe {
                libc::raise(libc::SIGTERM);
            },
            "running" => {
                fs::write(std::env::var_os("YT085_BROWSER_READY").unwrap(), b"ready").unwrap();
                std::thread::sleep(Duration::from_secs(60));
            }
            _ => panic!("unknown browser status fixture"),
        }
    }

    fn spec(temp: &Path, mode: &str) -> Spec {
        Spec {
            program: std::env::current_exe().unwrap().into(),
            args: vec![
                "--exact".into(),
                "smoke::browser::tests::subprocess_fixture".into(),
                "--nocapture".into(),
            ],
            cwd: temp.into(),
            env: vec![
                ("YT085_BROWSER_STATUS_FIXTURE".into(), Some(mode.into())),
                (
                    "YT085_BROWSER_READY".into(),
                    Some(temp.join("ready").into()),
                ),
            ],
            timeout: Duration::from_secs(5),
            output_limit: 1024,
        }
    }

    fn browser(temp: &Path, mode: &str) -> Browser {
        let profile = temp.join("profile");
        fs::create_dir(&profile).unwrap();
        let mut browser = Browser::spawn(
            &spec(temp, mode),
            temp.join("browser.log"),
            profile,
            // No listener is needed for an already exited fixture. Avoid binding
            // loopback sockets or discovering installed browsers in these tests.
            0,
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let ready = if mode == "running" {
                temp.join("ready").is_file()
            } else {
                browser.process.exited().unwrap()
            };
            if ready {
                return browser;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn receipt(browser: &Browser) -> Value {
        let value: Value =
            serde_json::from_slice(&fs::read(browser.log.with_extension("exit.json")).unwrap())
                .unwrap();
        assert_eq!(value["listener_closed"], true);
        assert_eq!(value["stream_closed"], true);
        assert_eq!(value["owned_tree_reaped"], true);
        assert!(value["cleanup_error"].is_null());
        assert!(browser.process.stream_closed());
        #[cfg(unix)]
        {
            assert_eq!(unsafe { libc::kill(-(browser.pid() as i32), 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
        value
    }

    #[test]
    fn both_readiness_paths_retain_early_native_exit_and_zero_failure() {
        for firefox in [false, true] {
            for (mode, native, code) in [("early", 23, 23), ("zero", 0, 1)] {
                let temp = tempfile::tempdir().unwrap();
                let mut browser = browser(temp.path(), mode);
                let error = browser.connect(firefox, Launch::default()).unwrap_err();
                assert_eq!(error.code, code);
                assert_eq!(
                    error.message,
                    if firefox {
                        "geckodriver exited before readiness"
                    } else {
                        "Chrome exited before readiness"
                    }
                );
                let value = receipt(&browser);
                assert_eq!(value["native_exit_code"], native);
                assert!(value["signal"].is_null());
                assert_eq!(value["profile_removed"], true);
                assert!(!browser.profile.exists());
                let path = browser.log.with_extension("exit.json");
                let bytes = fs::read(&path).unwrap();
                drop(browser);
                assert_eq!(fs::read(path).unwrap(), bytes);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn both_readiness_paths_retain_sigterm() {
        for firefox in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let mut browser = browser(temp.path(), "signal");
            assert_eq!(
                browser
                    .connect(firefox, Launch::default())
                    .unwrap_err()
                    .code,
                128 + libc::SIGTERM
            );
            let value = receipt(&browser);
            assert!(value["native_exit_code"].is_null());
            assert_eq!(value["signal"], libc::SIGTERM);
        }
    }

    #[test]
    fn stop_retains_status_without_rejecting_owned_termination() {
        for mode in ["graceful", "zero", "running"] {
            let temp = tempfile::tempdir().unwrap();
            let mut browser = browser(temp.path(), mode);
            browser.stop().unwrap();
            let value = receipt(&browser);
            assert_eq!(value["profile_removed"], true);
            match mode {
                "graceful" => {
                    assert_eq!(value["native_exit_code"], 17);
                    assert_eq!(browser.exit_code(), Some(17));
                }
                "zero" => assert_eq!(value["native_exit_code"], 0),
                _ => {
                    #[cfg(unix)]
                    {
                        assert!(value["native_exit_code"].is_null());
                        assert_eq!(value["signal"], libc::SIGTERM);
                    }
                }
            }
        }
    }

    #[test]
    fn successful_cleanup_preserves_original_validation_and_timeout_errors() {
        for code in [1, 124] {
            let temp = tempfile::tempdir().unwrap();
            let mut browser = browser(temp.path(), "graceful");
            let error = browser
                .finish_startup(
                    Err(tooling::Error {
                        code,
                        message: "original startup failure".into(),
                    }),
                    false,
                )
                .unwrap_err();
            assert_eq!(error.code, code);
            assert_eq!(error.message, "original startup failure");
            assert_eq!(receipt(&browser)["native_exit_code"], 17);
        }
    }

    #[test]
    fn spawn_and_profile_creation_failures_are_infrastructure_errors() {
        let temp = tempfile::tempdir().unwrap();
        let mut spec = spec(temp.path(), "early");
        spec.program = temp.path().join("missing-browser").into();
        let error = Browser::spawn(
            &spec,
            temp.path().join("browser.log"),
            temp.path().join("profile"),
            0,
        )
        .err()
        .unwrap();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "browser process spawn failed");
        let config = Launch {
            profile_parent: Some(temp.path().join("missing-parent")),
            ..Launch::default()
        };
        let error = Browser::start_config(temp.path(), temp.path(), false, config)
            .err()
            .unwrap();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "browser profile creation failed");
    }

    #[test]
    fn cleanup_and_receipt_failures_override_early_status_with_125() {
        for fail_profile in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let mut browser = browser(temp.path(), "early");
            let path = browser.log.with_extension("exit.json");
            if fail_profile {
                fs::remove_dir(&browser.profile).unwrap();
                fs::write(&browser.profile, b"not a directory").unwrap();
            } else {
                fs::create_dir(&path).unwrap();
            }
            assert_eq!(
                browser.connect(false, Launch::default()).unwrap_err().code,
                125
            );
            assert!(browser.process.stream_closed());
            if fail_profile {
                let value = receipt(&browser);
                assert_eq!(value["native_exit_code"], 23);
                assert_eq!(value["profile_removed"], false);
                assert!(value["profile_cleanup_error"].is_string());
                let bytes = fs::read(&path).unwrap();
                fs::remove_file(&browser.profile).unwrap();
                drop(browser);
                assert_eq!(fs::read(&path).unwrap(), bytes);
            } else {
                assert_eq!(browser.exit_code(), Some(23));
                assert!(!browser.profile.exists());
                assert!(path.is_dir());
            }
        }
    }
}
