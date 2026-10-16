//! Board's native-form/browser/restart/short diagnostic journeys, not YT-086 load qualification.
use super::{
    Server,
    browser::{Browser, Client},
    http, require, wait,
};
use crate::tooling::{self, Result, fail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};
#[derive(Clone)]
pub(crate) struct BoardClient {
    base: String,
    pub(crate) cookie: String,
    pub(crate) csrf: String,
    agent: ureq::Agent,
}
impl BoardClient {
    pub(crate) fn new(base: &str) -> Self {
        Self {
            base: base.into(),
            cookie: String::new(),
            csrf: String::new(),
            agent: http::agent(8),
        }
    }
    pub(crate) fn call(
        &mut self,
        path: &str,
        fields: Option<&[(&str, String)]>,
    ) -> Result<http::Response> {
        let response = if let Some(fields) = fields {
            let body = fields
                .iter()
                .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            http::call_agent(
                &self.agent,
                &self.base,
                "POST",
                path,
                Some(body.as_bytes()),
                &[
                    ("Cookie", &self.cookie),
                    ("Content-Type", "application/x-www-form-urlencoded"),
                    ("Origin", &self.base),
                ],
            )?
        } else {
            http::call_agent(
                &self.agent,
                &self.base,
                "GET",
                path,
                None,
                &[("Cookie", &self.cookie)],
            )?
        };
        require(
            response.body.len() <= 2 * 1024 * 1024,
            "Board HTTP body bound",
        )?;
        if let Ok(cookie) = response.cookie() {
            self.cookie = cookie;
        }
        if let Ok(csrf) = response.hidden("csrf") {
            self.csrf = csrf;
        }
        require(
            !response
                .headers
                .get("content-security-policy")
                .is_some_and(|v| v.contains("unsafe-inline")),
            "Board strict CSP",
        )?;
        Ok(response)
    }
    pub(crate) fn login(&mut self, codes: &BTreeMap<String, String>, user: &str) -> Result<()> {
        expect(&self.call("/board/login", None)?, 200)?;
        let code = codes
            .get(user)
            .ok_or_else(|| fail("missing local identity"))?
            .clone();
        let fields = vec![
            ("csrf", self.csrf.clone()),
            ("username", user.into()),
            ("code", code),
        ];
        expect(&self.call("/board/login", Some(&fields))?, 303)?;
        expect(
            &self.call(
                if user == "bob" {
                    "/board/projects/2/items"
                } else {
                    "/board/projects/1/items"
                },
                None,
            )?,
            200,
        )
    }
    pub(crate) fn fields(
        &self,
        title: &str,
        version: u64,
        status: &str,
    ) -> Vec<(&'static str, String)> {
        vec![
            ("csrf", self.csrf.clone()),
            ("expected_version", version.to_string()),
            ("title", title.into()),
            ("description", "Local qualification record.".into()),
            ("status", status.into()),
            ("priority", "normal".into()),
            ("assignee", String::new()),
        ]
    }
    fn save(&mut self, path: &str, title: &str, version: u64, status: &str) -> Result<()> {
        let fields = self.fields(title, version, status);
        expect(&self.call(path, Some(&fields))?, 303)
    }
}
fn encode(v: &str) -> String {
    percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC).to_string()
}
fn expect(r: &http::Response, status: u16) -> Result<()> {
    require(
        r.status == status,
        &format!("Board expected HTTP {status}, received {}", r.status),
    )
}
pub(crate) fn codes(log: &Path) -> Result<BTreeMap<String, String>> {
    let re = regex::Regex::new(r"(?m)^LOCAL_LOGIN (alice|bob|viewer) ([0-9a-f]{64})$").unwrap();
    wait(12, "private synthetic identities", || {
        let text = fs::read_to_string(log)?;
        let codes: BTreeMap<_, _> = re
            .captures_iter(&text)
            .map(|m| (m[1].into(), m[2].into()))
            .collect();
        Ok((codes.len() == 3).then_some(codes))
    })
}
// One directory per attempt prevents a reused port from overwriting restart evidence.
fn process_directory(
    temp: &Path,
    role: &'static str,
    receipts: &mut Vec<(&'static str, PathBuf)>,
) -> Result<PathBuf> {
    let directory = temp.join(format!("process-{}", receipts.len() + 1));
    fs::create_dir(&directory).map_err(|_| tooling::Error {
        code: 125,
        message: "Board private process directory creation failed".into(),
    })?;
    receipts.push((role, directory.clone()));
    Ok(directory)
}
fn start(root: &Path, temp: &Path, receipts: &mut Vec<(&'static str, PathBuf)>) -> Result<Server> {
    let directory = process_directory(temp, "server", receipts)?;
    Server::start(
        root,
        &directory,
        "yetlet-lab-operations-board",
        "release",
        "YETLET_BOARD_LISTEN",
        &[(
            "YETLET_BOARD_DB",
            temp.join("board.sqlite").to_string_lossy().into_owned(),
        )],
        "/readyz",
    )
}
fn check(c: &mut Client, r: &mut Value, js: &str, label: &str) -> Result<()> {
    c.check(js)?;
    record(r, label);
    Ok(())
}
fn record(r: &mut Value, label: &str) {
    r["checks"].as_array_mut().unwrap().push(json!(label));
}
fn navigate(c: &mut Client, base: &str, path: &str) -> Result<()> {
    c.navigate(&format!("{base}{path}"))?;
    c.until(
        &format!(
            "document.readyState==='complete' && location.pathname==={}",
            json!(path.split('?').next().unwrap())
        ),
        "Board navigation",
    )
}
fn click(c: &mut Client, selector: &str) -> Result<()> {
    c.check(&format!(
        "document.querySelectorAll({}).length===1",
        json!(selector)
    ))?;
    c.click(selector)?;
    c.evaluate("document.readyState")?;
    Ok(())
}
fn fill(c: &mut Client, selector: &str, value: &str) -> Result<()> {
    c.check(&format!(
        "document.querySelectorAll({}).length===1",
        json!(selector)
    ))?;
    match c {
        Client::Firefox(f) => {
            f.type_text(selector, value)?;
        }
        Client::Chrome(d) => {
            d.evaluate(&format!(
                "document.querySelector({}).value={}",
                json!(selector),
                json!(value)
            ))?;
        }
    }
    c.equal(
        &format!("document.querySelector({}).value", json!(selector)),
        json!(value),
    )
}
fn capture(c: &mut Client, r: &mut Value, out: &Path, name: &str) -> Result<()> {
    c.check("document.readyState==='complete' && [...document.querySelectorAll('input[type=password]')].every(e=>!e.value)")?;
    let data = c.chrome()?.call(
        "Page.captureScreenshot",
        json!({"format":"png","captureBeyondViewport":false}),
    )?["data"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    let bytes = STANDARD
        .decode(data)
        .map_err(|_| fail("Board PNG base64"))?;
    super::png::decode(&bytes)?;
    fs::write(out.join(format!("{name}.png")), bytes)?;
    r["screenshots"]
        .as_array_mut()
        .unwrap()
        .push(json!(format!("{name}.png")));
    Ok(())
}
pub fn run(root: &Path, out: &Path, firefox: bool) -> Result<()> {
    let temporary = tempfile::Builder::new()
        .prefix("private-board-")
        .tempdir_in(out)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700))?;
    }
    let temp = temporary.path().to_owned();
    let mut report = json!({"status":"fail","checks":[],"load":[],"screenshots":[],"cleanup":"pending","not_verified":if firefox{json!(["real OS IME","spoken screen reader","Safari/Edge/mobile device"])}else{json!(["real OS IME","spoken assistive technology","Firefox/Edge/Safari/mobile","visual screenshot review","10k items / 100k history operating envelope","approved duration soak","real IdP","production rollout/HA","external security review"])}});
    if !firefox {
        report["task_ids"] = json!(["YT-073", "YT-074"]);
    }
    let mut server = None;
    let mut browser: Option<Browser> = None;
    let mut streams = vec![];
    let mut receipts = vec![];
    let result: Result<()> = (|| {
        let binary = root.join("target/release").join(format!(
            "yetlet-lab-operations-board{}",
            std::env::consts::EXE_SUFFIX
        ));
        report["binary_sha256"] = json!(tooling::sha(&fs::read(binary)?));
        report["head"] = json!(super::head(root)?);
        let mut sources = super::source_hashes(root, &root.join("examples/lab-operations-board"))?;
        for name in ["Cargo.lock", "Cargo.toml"] {
            sources[name] = json!(tooling::sha(&tooling::read(root, name)?));
        }
        report["source_sha256"] = sources;
        report["rust_source_sha256"] = super::rust_sources(root)?;
        server = Some(start(root, &temp, &mut receipts)?);
        let app = server.as_ref().unwrap();
        let base = app.base.clone();
        let identities = codes(&app.log)?;
        // Private logs/codes/Cookies never enter the retained report directory.
        let directory = process_directory(
            &temp,
            if firefox { "geckodriver" } else { "chrome" },
            &mut receipts,
        )?;
        browser = Some(Browser::start(root, &directory, firefox)?);
        let b = browser.as_mut().unwrap();
        report["browser"] = json!(b.version);
        if firefox {
            firefox_journey(b.client()?, &base, &identities, &mut report)?;
        } else {
            let mut operator = BoardClient::new(&base);
            operator.login(&identities, "alice")?;
            let mut viewer = BoardClient::new(&base);
            viewer.login(&identities, "viewer")?;
            let fields = viewer.fields("Denied", 0, "open");
            expect(&viewer.call("/board/projects/1/items", Some(&fields))?, 403)?;
            let mut bob = BoardClient::new(&base);
            bob.login(&identities, "bob")?;
            expect(&bob.call("/board/projects/1/items", None)?, 403)?;
            // Required auth/CSRF/IDOR boundaries stay real HTTP and never retry writes.
            let mut bad = operator.fields("Denied CSRF", 0, "open");
            bad[0].1 = "invalid".into();
            expect(&operator.call("/board/projects/1/items", Some(&bad))?, 403)?;
            record(
                &mut report,
                "real HTTP viewer mutation, CSRF and cross-project denial",
            );
            chrome_journey(
                b.client()?,
                &base,
                &mut operator,
                &identities["alice"],
                &mut report,
                out,
            )?;
            b.stop()?;
            browser = None;
            let old_cookie = operator.cookie.clone();
            server.as_mut().unwrap().stop(true)?;
            server = None;
            record(
                &mut report,
                "owned server graceful exit and listener closed",
            );
            report["seed"] = seed(&temp.join("board.sqlite"))?;
            server = Some(start(root, &temp, &mut receipts)?);
            let app = server.as_ref().unwrap();
            let base = app.base.clone();
            let identities = codes(&app.log)?;
            let mut retired = BoardClient::new(&base);
            retired.cookie = old_cookie;
            expect(&retired.call("/board/projects/1/items/1", None)?, 401)?;
            let mut fresh = BoardClient::new(&base);
            fresh.login(&identities, "alice")?;
            require(
                fresh
                    .call("/board/projects/1/items/1", None)?
                    .text()?
                    .contains("Live update"),
                "Board restart retained record",
            )?;
            // Bob's item-level cross-project denial is independent of the listing denial.
            let mut bob = BoardClient::new(&base);
            bob.login(&identities, "bob")?;
            expect(&bob.call("/board/projects/1/items/1", None)?, 403)?;
            record(
                &mut report,
                "actual process restart preserves records/history and rejects old Memory login",
            );
            for _ in 0..10 {
                streams.push(Stream::open(app.port, &fresh.cookie)?);
            }
            let mut versions = vec![10; 25];
            for round in 1..=3 {
                let (summary, raw) =
                    load_round(&base, &identities, app.process.id(), &mut versions, round)?;
                report["load"].as_array_mut().unwrap().push(summary);
                fs::write(
                    out.join(format!("load-{round}.json")),
                    tooling::json_bytes(&raw),
                )?;
            }
            record(
                &mut report,
                "three diagnostic closed-loop 25-client 80/20 read/write rounds with ten SSE connections",
            );
            server.as_mut().unwrap().stop(true)?;
            server = None;
            record(
                &mut report,
                "owned server graceful exit and listener closed",
            );
            for stream in &mut streams {
                stream.eof()?;
            }
            streams.clear();
            record(
                &mut report,
                "graceful shutdown reaches EOF on all ten owned SSE clients",
            );
        }
        require(
            super::unchanged(root, &report["source_sha256"])
                && super::unchanged(root, &report["rust_source_sha256"]),
            "Board source identity changed",
        )?;
        report["status"] = json!("pass");
        Ok(())
    })();
    if let Err(e) = &result {
        report["failure_type"] = json!("RustToolError");
        report["exit_code"] = json!(e.code);
        if let Some(b) = browser.as_mut()
            && let Ok(c) = b.client()
        {
            // Bounded sanitized observation, never navigation or mutation replay.
            if let Ok(v)=c.evaluate("({pathname:location.pathname,ready:document.readyState,invalidControls:['username','code','title','description'].filter(id=>{const e=document.getElementById(id);return e&&e.validity&&!e.validity.valid;})})") {report["browser_failure"]=v;}
            if let Client::Chrome(c) = c {
                report["failed_http_statuses"] = json!(
                    c.failed_responses
                        .iter()
                        .rev()
                        .take(8)
                        .map(|s| s.split(' ').next().unwrap_or(""))
                        .collect::<Vec<_>>()
                );
                report["browser_exception_count"] = json!(c.exceptions.len());
            }
        }
    }
    streams.clear();
    let browser_cleanup = browser.as_mut().map_or(Ok(()), Browser::stop);
    drop(browser);
    let server_cleanup = server.as_mut().map_or(Ok(()), |s| s.stop(false));
    drop(server);
    finalize(
        out,
        firefox,
        &mut report,
        result,
        [browser_cleanup, server_cleanup],
        temporary,
        &receipts,
    )
}
// Receipt values are untrusted: copy typed facts, never free-form strings or paths.
fn sanitized_receipt(role: &str, index: usize, directory: &Path) -> Result<Value> {
    let capture = || -> Option<Value> {
        let mut paths = vec![];
        for entry in fs::read_dir(directory).ok()? {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_str()?;
            let selected = match role {
                "server" => name
                    .strip_prefix("server-")
                    .and_then(|s| s.strip_suffix(".exit.json"))
                    .is_some_and(|s| s.parse::<u16>().is_ok()),
                "chrome" => name == "chrome.exit.json",
                "geckodriver" => name == "geckodriver.exit.json",
                _ => return None,
            };
            if selected {
                if !entry.file_type().ok()?.is_file() {
                    return None;
                }
                paths.push(entry.path());
            }
        }
        if paths.len() != 1 {
            return None;
        }
        let source: Value = serde_json::from_slice(&fs::read(&paths[0]).ok()?).ok()?;
        let mut receipt = json!({"role":role,"index":index});
        for field in ["native_exit_code", "signal"] {
            let value = source.get(field)?;
            if !value.is_null() && value.as_i64().is_none() && value.as_u64().is_none() {
                return None;
            }
            receipt[field] = value.clone();
        }
        for field in ["listener_closed", "stream_closed", "owned_tree_reaped"] {
            receipt[field] = json!(source.get(field)?.as_bool()?);
        }
        // Preserve an error summary without trusting an error string to be secret-free.
        let fields: &[&str] = if role == "server" {
            &["cleanup_error"]
        } else {
            for field in ["profile_removed", "session_cleanup_failed"] {
                receipt[field] = json!(source.get(field)?.as_bool()?);
            }
            &["cleanup_error", "profile_cleanup_error"]
        };
        for field in fields {
            let value = source.get(*field)?;
            receipt[*field] = if value.is_null() {
                Value::Null
            } else if value.is_string() {
                json!("reported")
            } else {
                return None;
            };
        }
        Some(receipt)
    };
    capture().ok_or_else(|| tooling::Error {
        code: 125,
        message: "Board process exit receipt capture failed".into(),
    })
}

fn finalize(
    out: &Path,
    firefox: bool,
    report: &mut Value,
    result: Result<()>,
    cleanup: [Result<()>; 2],
    temporary: tempfile::TempDir,
    receipts: &[(&str, PathBuf)],
) -> Result<()> {
    let [browser_cleanup, server_cleanup] = cleanup;
    // Capture only known process attempts, including ones whose startup returned Err.
    // Capture/report failures must never prevent destruction of private data.
    report["process_receipts"] = json!([]);
    let mut capture = Ok(());
    for (index, (role, directory)) in receipts.iter().enumerate() {
        match sanitized_receipt(role, index + 1, directory) {
            Ok(receipt) => report["process_receipts"]
                .as_array_mut()
                .unwrap()
                .push(receipt),
            Err(error) => {
                capture = Err(error);
            }
        }
    }
    let private_cleanup = temporary.close().map_err(|_| tooling::Error {
        code: 125,
        message: "Board private temporary directory cleanup failed".into(),
    });
    let cleaned = browser_cleanup.is_ok() && server_cleanup.is_ok() && private_cleanup.is_ok();
    report["cleanup"] = json!(if cleaned {
        "owned application/browser processes reaped; listeners closed; private temporary profiles, logs and synthetic database removed"
    } else {
        "failed: owned cleanup incomplete"
    });
    if !cleaned {
        report["status"] = json!("blocked_cleanup");
    }
    if capture.is_err() {
        report["status"] = json!("blocked_receipts");
    }
    let receipt = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join(if firefox {
            "firefox.json"
        } else {
            "qualification.json"
        }))
        .and_then(|mut file| file.write_all(&tooling::json_bytes(report)))
        .map_err(|_| tooling::Error {
            code: 125,
            message: "Board sanitized report write failed".into(),
        });
    if receipt.is_err() {
        report["status"] = json!("blocked_report");
    }
    println!(
        "{}",
        json!({"status":report["status"],"cleanup":report["cleanup"],"load":report["load"]})
    );
    browser_cleanup?;
    server_cleanup?;
    private_cleanup?;
    capture?;
    receipt?;
    result
}

fn chrome_journey(
    c: &mut Client,
    base: &str,
    operator: &mut BoardClient,
    code: &str,
    r: &mut Value,
    out: &Path,
) -> Result<()> {
    c.chrome()?.call(
        "Emulation.setDeviceMetricsOverride",
        json!({"width":1280,"height":960,"deviceScaleFactor":1,"mobile":false}),
    )?;
    navigate(c, base, "/board/login")?;
    check(
        c,
        r,
        "document.querySelector('.board-form').getBoundingClientRect().width>400 && document.querySelectorAll('script').length===0",
        "script-free full-width login layout",
    )?;
    require(!code.is_empty(), "private browser login code missing")?;
    fill(c, "#username", "alice")?;
    fill(c, "#code", code)?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "location.pathname==='/board/projects/1/items' && document.readyState==='complete'",
        "native login redirect",
    )?;
    check(
        c,
        r,
        "document.body.textContent.includes('No work items match')",
        "actual empty authorized list",
    )?;
    click(c, ".board-nav a[href$='/new']")?;
    c.until(
        "document.getElementById('title')!==null && document.readyState==='complete'",
        "new item form",
    )?;
    check(
        c,
        r,
        "!document.querySelector('[aria-invalid=true]') && document.querySelector('label[for=description]').control===document.getElementById('description')",
        "new form has no premature errors and has associated description label",
    )?;
    fill(c, "#title", "Browser <mark>literal</mark>")?;
    fill(
        c,
        "#description",
        "Leading plain text\n<unsafe> remains text.",
    )?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "location.pathname==='/board/projects/1/items/1' && document.readyState==='complete'",
        "native create PRG",
    )?;
    check(
        c,
        r,
        "document.body.textContent.includes('Browser <mark>literal</mark>') && !document.querySelector('mark') && document.body.textContent.includes('History')",
        "stored create/read/history use literal text",
    )?;
    click(c, ".board-main a[href$='/edit']")?;
    c.until(
        "document.querySelector('[name=expected_version]')?.value==='1'",
        "versioned edit",
    )?;
    fill(c, "#status", "closed")?;
    fill(c, "#priority", "high")?;
    click(c, ".board-form button[type=submit]")?;
    c.until("location.pathname==='/board/projects/1/items/1' && document.body.textContent.includes('version 2')","native edit PRG")?;
    click(c, ".board-main a[href$='/edit']")?;
    c.until(
        "document.querySelector('[name=expected_version]')?.value==='2'",
        "stale draft preflight",
    )?;
    operator.save("/board/projects/1/items/1", "Other client", 2, "open")?;
    fill(c, "#title", "Retained browser draft")?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "document.body.textContent.includes('Another change was saved first')",
        "conflict page",
    )?;
    check(
        c,
        r,
        "document.getElementById('title').value==='Retained browser draft' && document.querySelector('[name=expected_version]').value==='2'",
        "real conflict preserves draft without silently overwriting",
    )?;
    capture(c, r, out, "03-conflict-draft")?;
    let d = c.chrome()?;
    require(
        d.failed_responses == vec![format!("409 {base}/board/projects/1/items/1")],
        "exact intentional conflict response",
    )?;
    let expected = "Failed to load resource: the server responded with a status of 409 (Conflict)";
    r["expected_conflict_console_count"] = json!(
        d.exceptions
            .iter()
            .filter(|v| v.as_str() == expected)
            .count()
    );
    d.exceptions.retain(|v| v != expected);
    require(
        d.exceptions.is_empty(),
        "unexpected browser error at conflict boundary",
    )?;
    d.failed_responses.clear();
    navigate(c, base, "/board/projects/1/items?live=true")?;
    c.until(
        "document.querySelector('[data-board-stream]')?.dataset.streamState==='current'",
        "initial authoritative SSE snapshot",
    )?;
    operator.save("/board/projects/1/items/1", "Live update", 3, "active")?;
    c.until(
        "document.querySelector('[data-board-stream]')?.dataset.currentRevision==='4'",
        "real persisted stream update",
    )?;
    check(
        c,
        r,
        "document.querySelector('[data-stream-message]').textContent.includes('Refresh authoritative records')",
        "status invalidation keeps an explicit authoritative refresh path",
    )?;
    click(c, "[data-stream-stop]")?;
    check(
        c,
        r,
        "document.querySelector('[data-board-stream]').dataset.streamState==='paused'",
        "explicit live-stream pause",
    )?;
    click(c, "[data-stream-retry]")?;
    c.until(
        "document.querySelector('[data-board-stream]')?.dataset.streamState==='current'",
        "full snapshot after explicit reconnect",
    )?;
    click(c, "[data-stream-stop]")?;
    c.chrome()?.call(
        "Emulation.setDeviceMetricsOverride",
        json!({"width":390,"height":844,"deviceScaleFactor":1,"mobile":false}),
    )?;
    check(
        c,
        r,
        "document.documentElement.scrollWidth<=390",
        "narrow Board has no page-level horizontal overflow",
    )?;
    capture(c, r, out, "04-narrow-live-status")?;
    for (i, title) in [
        "Review backup restoration evidence",
        "Verify lab TLS certificate renewal",
        "Update onboarding documentation",
        "Inspect storage capacity trend",
        "Reconcile test environment access",
        "Record weekly maintenance outcome",
    ]
    .iter()
    .enumerate()
    {
        let mut fields = operator.fields(title, 0, ["open", "active", "blocked"][i % 3]);
        for (k, v) in &mut fields {
            match *k {"priority"=>*v=["high","normal","low"][i%3].into(),"assignee"=>*v="alice".into(),"description"=>*v="Synthetic demonstration record. Track the result and retain the review history.".into(),_=>{}}
        }
        expect(
            &operator.call("/board/projects/1/items", Some(&fields))?,
            303,
        )?;
    }
    c.chrome()?.call(
        "Emulation.setDeviceMetricsOverride",
        json!({"width":1440,"height":1000,"deviceScaleFactor":1,"mobile":false}),
    )?;
    for (path, name) in [
        ("/board/projects/1/items", "01-work-items"),
        ("/board/projects/1/items/2", "02-item-history"),
        ("/board/projects/1/items/2/edit", "05-edit-work-item"),
    ] {
        navigate(c, base, path)?;
        capture(c, r, out, name)?;
    }
    let d = c.chrome()?;
    require(
        d.exceptions.is_empty()
            && d.failed_responses.is_empty()
            && d.requests
                .iter()
                .all(|u| u.starts_with(base) || u == "about:blank"),
        "Board CSP/JS/network clean; no external requests",
    )?;
    record(
        r,
        "actual browser CSP/JS/network clean; no external requests",
    );
    Ok(())
}
fn firefox_journey(
    c: &mut Client,
    base: &str,
    identities: &BTreeMap<String, String>,
    r: &mut Value,
) -> Result<()> {
    c.firefox()?.command(
        "/window/rect",
        "POST",
        Some(&json!({"width":1280,"height":960})),
    )?;
    navigate(c, base, "/board/login")?;
    require(
        http::get(base, "/board/login")?.header("referrer-policy")? == "same-origin",
        "Board Referrer-Policy",
    )?;
    fill(c, "#username", "alice")?;
    c.firefox()?.keys("\u{e004}")?;
    c.equal("document.activeElement.id", json!("code"))?;
    fill(c, "#code", &identities["alice"])?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "location.pathname==='/board/projects/1/items' && document.readyState==='complete'",
        "native login",
    )?;
    check(
        c,
        r,
        "document.querySelectorAll('script').length===0",
        "native login, Tab between named fields, strict script-free SSR",
    )?;
    click(c, ".board-nav a[href$='/new']")?;
    c.check("!document.querySelector('[aria-invalid=true]')")?;
    let title = "本機工作項目 — Unicode";
    let description = "第一行\n<b>Literal markup</b>";
    fill(c, "#title", title)?;
    c.firefox()?.keys("\u{e004}")?;
    c.equal("document.activeElement.id", json!("description"))?;
    fill(c, "#description", description)?;
    c.check("document.querySelector('label[for=description]').control===document.getElementById('description')")?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "location.pathname==='/board/projects/1/items/1' && document.readyState==='complete'",
        "create PRG",
    )?;
    c.check(&format!(
        "document.body.textContent.includes({})",
        json!(title)
    ))?;
    check(
        c,
        r,
        "document.querySelectorAll('b').length===0 && document.body.textContent.includes('<b>Literal markup</b>')",
        "native Unicode input/create/read/history and plain-text escaping; not OS composition",
    )?;
    click(c, ".board-main a[href$='/edit']")?;
    c.firefox()?.clear("#title")?;
    c.equal("document.querySelector('#title').value", json!(""))?;
    click(c, ".board-form button[type=submit]")?;
    c.check("document.getElementById('title').getAttribute('aria-invalid')==='true' && document.getElementById('title').getAttribute('aria-describedby')==='title-error'")?;
    c.equal(
        "document.getElementById('description').value",
        json!(description),
    )?;
    fill(c, "#title", "Corrected native draft")?;
    click(c, ".board-form button[type=submit]")?;
    c.until(
        "location.pathname==='/board/projects/1/items/1' && document.readyState==='complete'",
        "corrected PRG",
    )?;
    record(
        r,
        "native validation retains Unicode draft and exposes error relationship; correction saves",
    );
    navigate(c, base, "/board/projects/1/items?live=true")?;
    c.until(
        "document.querySelector('[data-board-stream]')?.dataset.streamState==='current'",
        "SSE initial snapshot",
    )?;
    c.evaluate("(()=>{window.statusMutations=[];const observer=new MutationObserver(records=>{for(const record of records)window.statusMutations.push(record.target.hasAttribute('data-stream-message')?'message':'counts')});observer.observe(document.querySelector('[data-stream-message]'),{childList:true});observer.observe(document.getElementById('record-counts'),{childList:true});window.statusObserver=observer;})()")?;
    // Deliberate original unchanged-snapshot observation budget, no write replay.
    std::thread::sleep(Duration::from_millis(2200));
    c.equal("window.statusMutations.length", json!(0))?;
    let mut client = BoardClient::new(base);
    client.login(identities, "alice")?;
    client.save(
        "/board/projects/1/items/1",
        "Firefox live update",
        2,
        "active",
    )?;
    c.until(
        "document.querySelector('[data-board-stream]')?.dataset.currentRevision==='3'",
        "updated SSE",
    )?;
    c.check("document.querySelector('[data-stream-message]').textContent.includes('Refresh authoritative records')")?;
    c.equal("window.statusMutations", json!(["message", "counts"]))?;
    c.evaluate("window.statusObserver.disconnect()")?;
    record(
        r,
        "unchanged SSE snapshots do not mutate text; live message changes before counts to avoid Orca event coalescing",
    );
    click(c, "[data-stream-stop]")?;
    check(
        c,
        r,
        "document.querySelector('[data-board-stream]').dataset.streamState==='paused'",
        "Firefox actual persisted SSE update, authoritative refresh notice and pause",
    )?;
    r["viewport"] = c.evaluate("({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})")?;
    Ok(())
}
/// Mutates only the owned, stopped DB. Existing records/history remain intact.
pub fn seed(path: &Path) -> Result<Value> {
    let mut db = rusqlite::Connection::open(path).map_err(|_| fail("Board seed database open"))?;
    let tx = db
        .transaction()
        .map_err(|_| fail("Board seed transaction"))?;
    for id in 1..=1000 {
        let title = format!("Load record {id:04}");
        tx.execute("INSERT OR IGNORE INTO work_items(id,project,title,description,status,priority,assignee,version) VALUES(?1,1,?2,'Synthetic load record','open','normal','',1)",rusqlite::params![id,title]).map_err(|_|fail("Board seed item"))?;
        let previous: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(version),0) FROM history WHERE item_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| fail("Board seed history version"))?;
        for version in previous + 1..=10 {
            tx.execute("INSERT INTO history(item_id,project,version,actor,action,title,status) VALUES(?1,1,?2,'alice',?3,?4,'open')",rusqlite::params![id,version,if version==1{"create"}else{"update"},title]).map_err(|_|fail("Board seed history"))?;
        }
        tx.execute(
            "UPDATE work_items SET version=10,status='open' WHERE id=?1",
            [id],
        )
        .map_err(|_| fail("Board seed version"))?;
    }
    tx.commit().map_err(|_| fail("Board seed commit"))?;
    let items: i64 = db
        .query_row("SELECT COUNT(*) FROM work_items", [], |r| r.get(0))
        .map_err(|_| fail("Board seed count"))?;
    let history: i64 = db
        .query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))
        .map_err(|_| fail("Board seed history count"))?;
    Ok(json!({"items":items,"history":history}))
}
struct Stream {
    reader: BufReader<TcpStream>,
}
impl Stream {
    fn open(port: u16, cookie: &str) -> Result<Self> {
        let mut stream = TcpStream::connect(("127.0.0.1", port))?;
        stream.set_read_timeout(Some(Duration::from_secs(8)))?;
        stream.set_write_timeout(Some(Duration::from_secs(8)))?;
        stream.write_all(format!("GET /board/projects/1/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nCookie: {cookie}\r\nConnection: close\r\n\r\n").as_bytes())?;
        let mut reader = BufReader::new(stream);
        let mut header = String::new();
        loop {
            let mut line = String::new();
            require(reader.read_line(&mut line)? > 0, "Board SSE header EOF")?;
            header.push_str(&line);
            require(header.len() < 65536, "Board SSE header bound")?;
            if line == "\r\n" {
                break;
            }
        }
        let header = header.to_ascii_lowercase();
        require(
            header.starts_with("http/1.1 200 ")
                && header.contains("content-type: text/event-stream")
                && header.contains("transfer-encoding: chunked"),
            "Board ten actual SSE streams",
        )?;
        require(
            !read_chunk(&mut reader)?.is_empty(),
            "Board SSE initial data",
        )?;
        Ok(Self { reader })
    }
    fn eof(&mut self) -> Result<()> {
        let mut total = 0;
        loop {
            let chunk = read_chunk(&mut self.reader)?;
            if chunk.is_empty() {
                break;
            }
            total += chunk.len();
            require(total < 65536, "Board SSE graceful EOF bound")?;
        }
        Ok(())
    }
}
pub fn read_chunk(reader: &mut impl BufRead) -> Result<Vec<u8>> {
    let mut size = String::new();
    require(
        reader.read_line(&mut size)? > 0 && size.len() <= 64,
        "Board SSE chunk size",
    )?;
    let size = usize::from_str_radix(size.trim().split(';').next().unwrap_or(""), 16)
        .map_err(|_| fail("Board SSE chunk length"))?;
    require(size <= 65536, "Board SSE chunk bound")?;
    let mut data = vec![0; size];
    reader.read_exact(&mut data)?;
    let mut crlf = [0; 2];
    reader.read_exact(&mut crlf)?;
    require(crlf == *b"\r\n", "Board SSE chunk delimiter")?;
    Ok(data)
}
#[derive(Clone)]
struct Observation(&'static str, u128, u16);
pub fn percentile(values: &[f64], quantile: f64) -> Result<f64> {
    require(
        !values.is_empty()
            && quantile > 0.
            && quantile <= 1.
            && values.iter().all(|v| v.is_finite()),
        "Board percentile inputs",
    )?;
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    Ok(v[((v.len() as f64 * quantile).ceil() as usize).saturating_sub(1)])
}
pub(crate) fn metrics(pid: u32) -> Result<Value> {
    #[cfg(target_os = "linux")]
    {
        let text = fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let stat = text
            .rsplit_once(") ")
            .ok_or_else(|| fail("Board process stat"))?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        let ticks = stat[11]
            .parse::<f64>()
            .map_err(|_| fail("Board CPU stat"))?
            + stat[12]
                .parse::<f64>()
                .map_err(|_| fail("Board CPU stat"))?;
        // SAFETY: sysconf reads the process clock tick configuration.
        let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        require(hz > 0, "Board process clock ticks")?;
        let text = fs::read_to_string(format!("/proc/{pid}/status"))?;
        let rss = text.lines().find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<u64>().ok())
        });
        Ok(json!({"cpu_seconds":ticks/hz as f64,"rss_kib":rss}))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        Err(fail(
            "Original Board diagnostic /proc metrics unavailable on this platform",
        ))
    }
}
fn load_round(
    base: &str,
    codes: &BTreeMap<String, String>,
    pid: u32,
    versions: &mut [u64],
    round: usize,
) -> Result<(Value, Value)> {
    let before = metrics(pid)?;
    let begin = Instant::now();
    let gate = Arc::new((Mutex::new(None), Condvar::new()));
    let (sender, receiver) = mpsc::channel();
    let results = std::thread::scope(|scope| {
        let mut handles = vec![];
        for (index, version) in versions.iter().copied().enumerate() {
            let gate = Arc::clone(&gate);
            let sender = sender.clone();
            handles.push(scope.spawn(move || -> Result<(Vec<Observation>, u64)> {
                let mut client = BoardClient::new(base);
                let warmup: Result<()> = (|| {
                    client.login(codes, "alice")?;
                    for _ in 0..4 {
                        expect(&client.call("/board/projects/1/items", None)?, 200)?;
                    }
                    Ok(())
                })();
                let _ = sender.send(warmup.is_ok());
                let (lock, cv) = &*gate;
                let mut ready = lock.lock().map_err(|_| fail("Board diagnostic gate"))?;
                while ready.is_none() {
                    ready = cv.wait(ready).map_err(|_| fail("Board diagnostic gate"))?;
                }
                let run = ready.unwrap_or(false);
                drop(ready);
                warmup?;
                require(run, "Board diagnostic warmup aborted")?;
                let mut observations = vec![];
                let mut version = version;
                for operation in 0..50 {
                    tooling::check_cancel()?;
                    let write = operation % 5 == 4;
                    let start = Instant::now();
                    let response = if write {
                        let fields =
                            client.fields(&format!("Load worker {index:02}"), version, "open");
                        client.call(
                            &format!("/board/projects/1/items/{}", index + 1),
                            Some(&fields),
                        )?
                    } else {
                        client.call("/board/projects/1/items", None)?
                    };
                    let duration = start.elapsed().as_nanos();
                    let status = response.status;
                    observations.push(Observation(
                        if write { "write" } else { "read" },
                        duration,
                        status,
                    ));
                    expect(&response, if write { 303 } else { 200 })?;
                    if write {
                        version += 1;
                    }
                }
                Ok((observations, version))
            }));
        }
        drop(sender);
        let deadline = Instant::now() + Duration::from_secs(12);
        let mut ready = true;
        for _ in 0..25 {
            if receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .ok()
                != Some(true)
            {
                ready = false;
                break;
            }
        }
        let (lock, cv) = &*gate;
        *lock.lock().unwrap() = Some(ready);
        cv.notify_all();
        // Always join every admitted worker before returning an error.
        let mut output = vec![];
        for h in handles {
            output.push(
                h.join()
                    .map_err(|_| fail("Board diagnostic worker panicked"))
                    .and_then(|v| v),
            );
        }
        output
    });
    let wall = begin.elapsed().as_secs_f64();
    let after = metrics(pid)?;
    let mut rows = vec![];
    for (i, result) in results.into_iter().enumerate() {
        let (observations, version) = result?;
        versions[i] = version;
        rows.extend(observations);
    }
    let mut summary = json!({"run":round,"clients":25,"sse_clients":10,"requests":rows.len(),"unexpected_responses":0,"lifecycle_seconds_including_login_and_warmup":wall,"cpu_seconds":after["cpu_seconds"].as_f64().unwrap()-before["cpu_seconds"].as_f64().unwrap(),"rss_before_kib":before["rss_kib"],"rss_after_kib":after["rss_kib"]});
    for kind in ["read", "write"] {
        let values: Vec<_> = rows
            .iter()
            .filter(|o| o.0 == kind)
            .map(|o| o.1 as f64 / 1e6)
            .collect();
        summary[kind] = json!({"samples":values.len(),"p50_ms":percentile(&values,0.5)?,"p95_ms":percentile(&values,0.95)?,"p99_ms":percentile(&values,0.99)?,"max_ms":values.iter().copied().fold(0.,f64::max)});
    }
    Ok((
        summary,
        json!(
            rows.iter()
                .map(|o| json!([o.0, o.1, o.2]))
                .collect::<Vec<_>>()
        ),
    ))
}

#[cfg(test)]
mod parent_suite_cleanup {
    use super::*;

    fn private_receipts(private: &Path, firefox: bool) -> Vec<(&'static str, PathBuf)> {
        let mut receipts = vec![];
        // Use the production attempt registration; reusing the same port is safe.
        for role in [
            "server",
            if firefox { "geckodriver" } else { "chrome" },
            "server",
        ] {
            let directory = process_directory(private, role, &mut receipts).unwrap();
            let mut receipt = json!({
                "native_exit_code": if receipts.len() == 3 { json!(23) } else { json!(0) },
                "signal": null,
                "listener_closed": true, "stream_closed": true, "owned_tree_reaped": true,
                "cleanup_error": null,
                "status": "Cookie=SECRET; /private/SECRET",
                "credentials": "SECRET", "unknown": {"token":"SECRET"}
            });
            let name = if role == "server" {
                "server-12345.exit.json".into()
            } else {
                receipt["native_exit_code"] = Value::Null;
                receipt["signal"] = json!(15);
                receipt["profile_removed"] = json!(true);
                receipt["session_cleanup_failed"] = json!(false);
                receipt["profile_cleanup_error"] = Value::Null;
                format!("{role}.exit.json")
            };
            fs::write(directory.join(name), tooling::json_bytes(&receipt)).unwrap();
            fs::write(
                directory.join("raw.log"),
                "SECRET credentials Cookie=SECRET",
            )
            .unwrap();
            fs::write(directory.join("arbitrary.exit.json"), "SECRET").unwrap();
        }
        fs::write(private.join("board.sqlite"), "SECRET database").unwrap();
        fs::write(private.join("Cookies"), "SECRET").unwrap();
        receipts
    }

    #[test]
    fn finalize_retains_sanitized_restart_and_failed_startup_receipts() {
        for firefox in [false, true] {
            for failed_startup in [false, true] {
                let out = tempfile::tempdir().unwrap();
                let private = tempfile::tempdir_in(out.path()).unwrap();
                let path = private.path().to_owned();
                let receipts = private_receipts(&path, firefox);
                if failed_startup {
                    // Arbitrary error text may contain private paths or credentials.
                    let file = receipts[1].1.join(if firefox {
                        "geckodriver.exit.json"
                    } else {
                        "chrome.exit.json"
                    });
                    let mut value: Value =
                        serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
                    value["cleanup_error"] = json!("SECRET /private/SECRET");
                    value["profile_cleanup_error"] = json!("Cookie=SECRET");
                    fs::write(file, tooling::json_bytes(&value)).unwrap();
                }
                let mut report =
                    json!({"status":if failed_startup {"fail"} else {"pass"},"load":[]});
                let result = finalize(
                    out.path(),
                    firefox,
                    &mut report,
                    if failed_startup {
                        Err(tooling::Error {
                            code: 23,
                            message: "startup failed".into(),
                        })
                    } else {
                        Ok(())
                    },
                    [Ok(()), Ok(())],
                    private,
                    &receipts,
                );
                if failed_startup {
                    assert_eq!(result.unwrap_err().code, 23);
                } else {
                    result.unwrap();
                }
                assert!(!path.exists());
                let bytes = fs::read(out.path().join(if firefox {
                    "firefox.json"
                } else {
                    "qualification.json"
                }))
                .unwrap();
                let retained: Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(retained, report);
                let statuses = retained["process_receipts"].as_array().unwrap();
                assert_eq!(statuses.len(), 3);
                assert_eq!(
                    statuses[0],
                    json!({"role":"server","index":1,"native_exit_code":0,"signal":null,
                    "listener_closed":true,"stream_closed":true,"owned_tree_reaped":true,"cleanup_error":null})
                );
                assert_eq!(
                    statuses[1],
                    json!({"role":if firefox {"geckodriver"} else {"chrome"},"index":2,
                    "native_exit_code":null,"signal":15,"listener_closed":true,"stream_closed":true,"owned_tree_reaped":true,
                    "profile_removed":true,"session_cleanup_failed":false,
                    "cleanup_error":if failed_startup {json!("reported")} else {Value::Null},
                    "profile_cleanup_error":if failed_startup {json!("reported")} else {Value::Null}})
                );
                assert_eq!(
                    statuses[2],
                    json!({"role":"server","index":3,"native_exit_code":23,"signal":null,
                    "listener_closed":true,"stream_closed":true,"owned_tree_reaped":true,"cleanup_error":null})
                );
                let text = String::from_utf8(bytes).unwrap();
                for secret in [
                    "SECRET",
                    "Cookie",
                    "credentials",
                    "unknown",
                    path.to_str().unwrap(),
                    "12345",
                ] {
                    assert!(!text.contains(secret));
                }
                assert_eq!(fs::read_dir(out.path()).unwrap().count(), 1);
            }
        }
    }

    #[test]
    fn finalize_capture_or_persistence_failure_is_125_and_still_removes_private_data() {
        for firefox in [false, true] {
            for failure in [
                "missing",
                "unreadable",
                "malformed",
                "invalid_fields",
                "report_write",
                "existing_report",
            ] {
                let out = tempfile::tempdir().unwrap();
                let private = tempfile::tempdir_in(out.path()).unwrap();
                let path = private.path().to_owned();
                let receipts = private_receipts(&path, firefox);
                let file = receipts[0].1.join("server-12345.exit.json");
                let report_path = out.path().join(if firefox {
                    "firefox.json"
                } else {
                    "qualification.json"
                });
                match failure {
                    "missing" => fs::remove_file(file).unwrap(),
                    "unreadable" => {
                        fs::remove_file(&file).unwrap();
                        fs::create_dir(file).unwrap();
                    }
                    "malformed" => fs::write(file, "SECRET {broken JSON").unwrap(),
                    "invalid_fields" => {
                        fs::write(file, br#"{"native_exit_code":"SECRET"}"#).unwrap()
                    }
                    "report_write" => fs::create_dir(&report_path).unwrap(),
                    "existing_report" => fs::write(&report_path, "historical receipt").unwrap(),
                    _ => unreachable!(),
                }
                let mut report = json!({"status":"pass","load":[]});
                let error = finalize(
                    out.path(),
                    firefox,
                    &mut report,
                    Ok(()),
                    [Ok(()), Ok(())],
                    private,
                    &receipts,
                )
                .unwrap_err();
                assert_eq!(error.code, 125);
                assert!(!path.exists());
                assert!(!error.message.contains(path.to_str().unwrap()));
                assert!(!report.to_string().contains("SECRET"));
                if matches!(failure, "report_write" | "existing_report") {
                    assert_eq!(report["status"], "blocked_report");
                    assert_eq!(report["process_receipts"].as_array().unwrap().len(), 3);
                    assert_eq!(error.message, "Board sanitized report write failed");
                    if failure == "existing_report" {
                        assert_eq!(
                            fs::read_to_string(report_path).unwrap(),
                            "historical receipt"
                        );
                    }
                } else {
                    assert_eq!(report["status"], "blocked_receipts");
                    // A bad first receipt must not discard subsequent valid evidence.
                    assert_eq!(report["process_receipts"].as_array().unwrap().len(), 2);
                    let retained: Value =
                        serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
                    assert_eq!(retained, report);
                    assert_eq!(error.message, "Board process exit receipt capture failed");
                }
            }
        }
    }

    #[test]
    fn private_directory_close_failure_returns_125_and_blocked_receipt() {
        for firefox in [false, true] {
            let out = tempfile::tempdir().unwrap();
            let private = tempfile::tempdir_in(out.path()).unwrap();
            let path = private.path().to_owned();
            // A regular file at an owned directory path deterministically fails close.
            fs::remove_dir(&path).unwrap();
            fs::write(&path, "private synthetic data").unwrap();
            let mut report = json!({"status":"pass","load":[]});
            let error = finalize(
                out.path(),
                firefox,
                &mut report,
                Ok(()),
                [Ok(()), Ok(())],
                private,
                &[],
            )
            .unwrap_err();
            assert_eq!(error.code, 125);
            assert_eq!(
                error.message,
                "Board private temporary directory cleanup failed"
            );
            assert_eq!(report["status"], "blocked_cleanup");
            let bytes = fs::read(out.path().join(if firefox {
                "firefox.json"
            } else {
                "qualification.json"
            }))
            .unwrap();
            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), report);
            assert!(
                !String::from_utf8(bytes)
                    .unwrap()
                    .contains("private synthetic data")
            );
            assert!(!error.message.contains(path.to_str().unwrap()));
        }
    }

    #[test]
    fn earlier_cleanup_error_survives_later_directory_and_report_failures() {
        for browser_fails in [true, false] {
            let out = tempfile::tempdir().unwrap();
            let private = tempfile::tempdir_in(out.path()).unwrap();
            fs::remove_dir(private.path()).unwrap();
            fs::write(private.path(), "fixture").unwrap();
            fs::create_dir(out.path().join("qualification.json")).unwrap();
            let mut report = json!({"status":"fail","load":[]});
            let error = finalize(
                out.path(),
                false,
                &mut report,
                Err(tooling::fail("primary validation failed")),
                [
                    if browser_fails {
                        Err(tooling::Error {
                            code: 125,
                            message: "first browser cleanup failed".into(),
                        })
                    } else {
                        Ok(())
                    },
                    Err(tooling::Error {
                        code: 125,
                        message: "server cleanup failed".into(),
                    }),
                ],
                private,
                &[],
            )
            .unwrap_err();
            assert_eq!(error.code, 125);
            assert_eq!(
                error.message,
                if browser_fails {
                    "first browser cleanup failed"
                } else {
                    "server cleanup failed"
                }
            );
            assert_eq!(report["cleanup"], "failed: owned cleanup incomplete");
            assert_ne!(report["status"], "pass");
        }
    }

    #[test]
    fn earlier_error_and_report_failure_do_not_skip_owned_directory_removal() {
        let out = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(out.path()).unwrap();
        let path = private.path().to_owned();
        fs::write(path.join("private.log"), "fixture").unwrap();
        fs::create_dir(out.path().join("firefox.json")).unwrap();
        let mut report = json!({"status":"pass","load":[]});
        let error = finalize(
            out.path(),
            true,
            &mut report,
            Ok(()),
            [
                Err(tooling::Error {
                    code: 125,
                    message: "first browser cleanup failed".into(),
                }),
                Ok(()),
            ],
            private,
            &[],
        )
        .unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "first browser cleanup failed");
        assert!(!path.exists());
        assert_ne!(report["status"], "pass");
    }

    #[test]
    fn successful_cleanup_preserves_primary_errors_but_required_receipt_cannot_pass() {
        for code in [0, 1, 23, 124, 125, 130] {
            for report_fails in [false, true] {
                let out = tempfile::tempdir().unwrap();
                let private = tempfile::tempdir_in(out.path()).unwrap();
                let path = private.path().to_owned();
                if report_fails {
                    fs::create_dir(out.path().join("qualification.json")).unwrap();
                }
                let mut report = json!({"status":if code==0 {"pass"}else{"fail"},"load":[]});
                let primary = if code == 0 {
                    Ok(())
                } else {
                    Err(tooling::Error {
                        code,
                        message: "primary diagnostic".into(),
                    })
                };
                let result = finalize(
                    out.path(),
                    false,
                    &mut report,
                    primary,
                    [Ok(()), Ok(())],
                    private,
                    &[],
                );
                assert!(!path.exists());
                if report_fails {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, 125);
                    assert_eq!(error.message, "Board sanitized report write failed");
                    assert_eq!(report["status"], "blocked_report");
                } else if code == 0 {
                    result.unwrap();
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, code);
                    assert_eq!(error.message, "primary diagnostic");
                }
            }
        }
    }
}
