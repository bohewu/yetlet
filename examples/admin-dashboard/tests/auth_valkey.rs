//! Opt-in real Admin journeys. Caller owns a disposable single-host Valkey.
//! CLIENT PAUSE affects that whole instance. Credentials stay only in memory.
#![cfg(all(feature = "auth-valkey", target_os = "linux"))]
use reqwest::{Client, Response, header};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::{Instant, sleep, timeout},
};

struct Admin {
    child: Child,
    output: mpsc::Receiver<String>,
    reader: Option<std::thread::JoinHandle<()>>,
    base: String,
    codes: BTreeMap<String, String>,
}
impl Admin {
    async fn start(root: &Path, endpoint: &str, namespace: &str) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let mut child = Command::new(env!("CARGO_BIN_EXE_yetlet-admin-dashboard"))
            .env_clear()
            .env("YETLET_CONFIG_ROOT", root)
            .env("APP_SERVER__PORT", port.to_string())
            .env("APP_SESSION__BACKEND", "valkey")
            .env("APP_SESSION__VALKEY__ENDPOINT", endpoint)
            .env("APP_SESSION__VALKEY__NAMESPACE", namespace)
            .env("APP_AUTH__ENABLED", "true")
            .env("APP_AUTH__POLICY_FILE", root.join("policy.json"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start actual Admin");
        let stdout = child.stdout.take().unwrap();
        let (tx, output) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let mut app = Self {
            child,
            output,
            reader: Some(reader),
            base: format!("http://127.0.0.1:{port}"),
            codes: BTreeMap::new(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                app.child.try_wait().unwrap().is_none(),
                "Admin startup failed"
            );
            while let Ok(line) = app.output.try_recv() {
                if let Some(credentials) = line.strip_prefix("LOCAL_LOGIN ") {
                    let (name, code) = credentials.split_once(' ').expect("local login shape");
                    app.codes.insert(name.to_owned(), code.to_owned());
                } else if line.starts_with("READY ") {
                    assert_eq!(app.codes.len(), 3);
                    return app;
                }
            }
            assert!(Instant::now() < deadline, "Admin readiness timeout");
            sleep(Duration::from_millis(20)).await;
        }
    }
    async fn stop(&mut self) {
        // SAFETY: only this owned, unreaped child is signalled.
        assert_eq!(
            unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) },
            0
        );
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success(), "Admin drain failed");
                self.reader.take().unwrap().join().unwrap();
                assert!(
                    self.output
                        .try_iter()
                        .any(|line| line.starts_with("STOPPED "))
                );
                self.codes.clear();
                return;
            }
            assert!(Instant::now() < deadline, "Admin drain timeout");
            sleep(Duration::from_millis(20)).await;
        }
    }
}
impl Drop for Admin {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
fn policy(root: &Path, value: &Value) {
    std::fs::write(root.join("next.json"), serde_json::to_vec(value).unwrap()).unwrap();
    std::fs::rename(root.join("next.json"), root.join("policy.json")).unwrap();
}
fn allow() -> Value {
    json!({"accounts":{
        "alice":{"enabled":true,"read":true,"write":true},
        "bob":{"enabled":true,"read":true,"write":true},
        "viewer":{"enabled":true,"read":true,"write":true}
    }})
}
fn check(response: &Response, status: u16, cookie: bool) {
    assert_eq!(response.status().as_u16(), status);
    assert_eq!(response.headers().contains_key(header::SET_COOKIE), cookie);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(
        response
            .headers()
            .contains_key(header::CONTENT_SECURITY_POLICY)
    );
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    if status == 303 {
        assert!(
            response.headers()[header::LOCATION]
                .to_str()
                .unwrap()
                .starts_with("/board")
        );
    }
}
fn cookie(response: &Response) -> String {
    let header = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(header.contains("HttpOnly") && header.contains("SameSite=Lax"));
    header.split(';').next().unwrap().to_owned()
}
fn hidden(body: &str, name: &str) -> String {
    body.split(&format!("name=\"{name}\" value=\""))
        .nth(1)
        .expect("hidden field")
        .split('"')
        .next()
        .unwrap()
        .to_owned()
}
async fn get(client: &Client, app: &Admin, path: &str, cookie: &str) -> Response {
    client
        .get(format!("{}{path}", app.base))
        .header(header::COOKIE, cookie)
        .send()
        .await
        .unwrap()
}
async fn post(
    client: &Client,
    app: &Admin,
    path: &str,
    cookie: &str,
    fields: &[(&str, &str)],
) -> Response {
    client
        .post(format!("{}{path}", app.base))
        .header(header::COOKIE, cookie)
        .header(header::ORIGIN, &app.base)
        .form(fields)
        .send()
        .await
        .unwrap()
}
async fn login_rotation(client: &Client, app: &Admin, name: &str) -> (String, String, String) {
    let page = get(client, app, "/board/login", "").await;
    check(&page, 200, true);
    let anonymous = cookie(&page);
    let old_csrf = hidden(&page.text().await.unwrap(), "csrf");
    let response = post(
        client,
        app,
        "/board/login",
        &anonymous,
        &[
            ("csrf", &old_csrf),
            ("username", name),
            ("code", &app.codes[name]),
        ],
    )
    .await;
    check(&response, 303, true);
    let signed = cookie(&response);
    assert!(signed != anonymous, "login rotates cookie");
    (signed, anonymous, old_csrf)
}
async fn login(client: &Client, app: &Admin, name: &str) -> (String, String) {
    let (signed, anonymous, old_csrf) = login_rotation(client, app, name).await;
    let page = get(client, app, "/board", &signed).await;
    check(&page, 303, false);
    let path = page.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let page = get(client, app, &path, &signed).await;
    check(&page, 200, false);
    let csrf = hidden(&page.text().await.unwrap(), "csrf");
    assert!(csrf != old_csrf, "login rotates CSRF");
    check(&get(client, app, "/board", &anonymous).await, 303, false);
    check(
        &post(
            client,
            app,
            "/board/logout",
            &signed,
            &[("csrf", &old_csrf)],
        )
        .await,
        403,
        false,
    );
    (signed, csrf)
}
async fn pause(port: u16, mode: &str, millis: u32) {
    let operation = async {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap();
        stream
            .write_all(format!("CLIENT PAUSE {millis} {mode}\r\n").as_bytes())
            .await
            .unwrap();
        let mut reply = [0u8; 5];
        stream.read_exact(&mut reply).await.unwrap();
        assert_eq!(&reply, b"+OK\r\n");
    };
    timeout(Duration::from_secs(2), operation).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires caller-owned disposable Valkey at YT126_VALKEY_URL; pauses the whole instance"]
async fn two_admin_processes_use_fresh_auth_policy() {
    let endpoint = std::env::var("YT126_VALKEY_URL").expect("dedicated Valkey endpoint required");
    let port = endpoint
        .strip_prefix("redis://127.0.0.1:")
        .and_then(|s| s.strip_suffix('/'))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|p| *p != 0)
        .expect("loopback-only Valkey endpoint");
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("data")).unwrap();
    std::fs::write(root.path().join("base.toml"), include_str!("../base.toml")).unwrap();
    std::fs::write(root.path().join("production.toml"), "").unwrap();
    let namespace = format!(
        "yt126_{}_{}",
        std::process::id(),
        root.path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace('.', "_")
    );
    policy(root.path(), &allow());
    let mut a = Admin::start(root.path(), &endpoint, &namespace).await;
    let mut b = Admin::start(root.path(), &endpoint, &namespace).await;
    assert!(
        a.codes["alice"] != b.codes["alice"],
        "per-process credential proofs differ"
    );
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    for app in [&a, &b] {
        assert_eq!(get(&client, app, "/healthz", "").await.status(), 200);
        assert_eq!(
            get(&client, app, "/board/not-a-route", "").await.status(),
            404
        );
        assert_eq!(
            client
                .delete(format!("{}/board/projects/1/items", app.base))
                .send()
                .await
                .unwrap()
                .status(),
            405
        );
        check(&get(&client, app, "/board", "").await, 303, false);
        check(
            &get(&client, app, "/board", "yetlet_lab_valkey=bad").await,
            400,
            false,
        );
    }
    let (signed, csrf) = login(&client, &a, "alice").await;
    check(
        &get(&client, &b, "/board/projects/1/items", &signed).await,
        200,
        false,
    );
    check(
        &get(&client, &b, "/board/projects/2/items", &signed).await,
        403,
        false,
    );
    check(
        &get(&client, &a, "/board", &format!("{signed}; {signed}")).await,
        400,
        false,
    );
    let fields = [
        ("csrf", csrf.as_str()),
        ("expected_version", "0"),
        ("title", "YT126 created"),
        ("description", "Private bounded test"),
        ("status", "open"),
        ("priority", "normal"),
        ("assignee", "alice"),
    ];
    let created = post(&client, &a, "/board/projects/1/items", &signed, &fields).await;
    check(&created, 303, false);
    let detail = created.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let id: i64 = detail.rsplit('/').next().unwrap().parse().unwrap();
    let mut update = fields;
    update[1].1 = "1";
    update[2].1 = "YT126 native update";
    check(
        &post(&client, &b, &detail, &signed, &update).await,
        303,
        false,
    );
    check(
        &post(&client, &a, &detail, &signed, &update).await,
        409,
        false,
    );
    let payload = json!({"csrf":csrf,"item":{"id":id,"version":2,"title":"YT126 triage update","description":"Saved JSON draft","status":"active","priority":"normal","assignee":"alice"}});
    let triage = "/board/projects/1/triage";
    let response = client
        .post(format!("{}{triage}", b.base))
        .header(header::COOKIE, &signed)
        .json(&payload)
        .send()
        .await
        .unwrap();
    check(&response, 200, false);
    assert_eq!(response.json::<Value>().await.unwrap()["version"], 3);
    let response = client
        .post(format!("{}{triage}", a.base))
        .header(header::COOKIE, &signed)
        .json(&payload)
        .send()
        .await
        .unwrap();
    check(&response, 409, false);
    assert_eq!(
        response.json::<Value>().await.unwrap()["error"]["code"],
        "conflict"
    );
    let (viewer, viewer_csrf) = login(&client, &b, "viewer").await;
    check(
        &get(&client, &a, "/board/projects/1/items", &viewer).await,
        200,
        false,
    );
    let mut viewer_write = fields;
    viewer_write[0].1 = &viewer_csrf;
    check(
        &post(
            &client,
            &a,
            "/board/projects/1/items",
            &viewer,
            &viewer_write,
        )
        .await,
        403,
        false,
    );
    let mut restricted = allow();
    restricted["accounts"]["alice"]["write"] = false.into();
    policy(root.path(), &restricted);
    let before = get(&client, &a, &detail, &signed)
        .await
        .text()
        .await
        .unwrap();
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &signed).await, 200, false);
        check(
            &post(&client, app, &detail, &signed, &update).await,
            403,
            false,
        );
        let response = client
            .post(format!("{}{triage}", app.base))
            .header(header::COOKIE, &signed)
            .json(&payload)
            .send()
            .await
            .unwrap();
        check(&response, 403, false);
        assert_eq!(
            response.json::<Value>().await.unwrap()["error"]["code"],
            "forbidden"
        );
        assert_eq!(
            get(&client, app, &detail, &signed)
                .await
                .text()
                .await
                .unwrap(),
            before,
            "denied writes leave item and history unchanged"
        );
    }
    // Stream checks account/resource policy each iteration, closing after revocation.
    let mut stream = get(&client, &b, "/board/projects/1/events", &signed).await;
    check(&stream, 200, false);
    assert!(stream.chunk().await.unwrap().is_some());
    restricted["accounts"]["alice"]["read"] = false.into();
    policy(root.path(), &restricted);
    timeout(Duration::from_secs(3), async {
        while stream.chunk().await.unwrap().is_some() {}
    })
    .await
    .unwrap();
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &signed).await, 403, false);
    }
    for mode in ["disabled", "removed"] {
        let mut denied = allow();
        if mode == "disabled" {
            denied["accounts"]["alice"]["enabled"] = false.into();
        } else {
            denied["accounts"].as_object_mut().unwrap().remove("alice");
        }
        policy(root.path(), &denied);
        for app in [&a, &b] {
            check(&get(&client, app, &detail, &signed).await, 403, false);
            check(
                &post(&client, app, &detail, &signed, &update).await,
                403,
                false,
            );
        }
    }
    policy(root.path(), &allow());
    let anonymous = get(&client, &a, "/board/login", "").await;
    let anonymous_cookie = cookie(&anonymous);
    check(
        &get(&client, &b, "/board", &anonymous_cookie).await,
        303,
        false,
    );
    for missing in [false, true] {
        if missing {
            std::fs::remove_file(root.path().join("policy.json")).unwrap();
        } else {
            std::fs::write(root.path().join("policy.json"), "invalid").unwrap();
        }
        for app in [&a, &b] {
            check(&get(&client, app, &detail, &signed).await, 503, false);
            check(
                &get(&client, app, "/board", &anonymous_cookie).await,
                503,
                false,
            );
            check(
                &post(&client, app, &detail, &signed, &update).await,
                503,
                false,
            );
            assert_eq!(get(&client, app, "/healthz", "").await.status(), 200);
        }
    }
    // Logout does not depend on available policy authority.
    check(
        &post(
            &client,
            &a,
            "/board/logout",
            &viewer,
            &[("csrf", &viewer_csrf)],
        )
        .await,
        303,
        false,
    );
    policy(root.path(), &allow());
    b.stop().await;
    b = Admin::start(root.path(), &endpoint, &namespace).await;
    check(&get(&client, &b, &detail, &signed).await, 200, false);
    pause(port, "ALL", 1300).await;
    let began = Instant::now();
    let (ra, rb) = tokio::join!(
        get(&client, &a, &detail, &signed),
        get(&client, &b, &detail, &signed)
    );
    check(&ra, 503, false);
    check(&rb, 503, false);
    assert!(began.elapsed() < Duration::from_secs(2));
    sleep(Duration::from_millis(1400)).await;
    check(&get(&client, &a, &detail, &signed).await, 200, false);
    // A dispatched rotation can become uncertain: no success/cookie, no retry.
    let page = get(&client, &a, "/board/login", "").await;
    let anon = cookie(&page);
    let token = hidden(&page.text().await.unwrap(), "csrf");
    pause(port, "WRITE", 750).await;
    check(
        &post(
            &client,
            &a,
            "/board/login",
            &anon,
            &[
                ("csrf", &token),
                ("username", "alice"),
                ("code", &a.codes["alice"]),
            ],
        )
        .await,
        503,
        false,
    );
    sleep(Duration::from_millis(900)).await;
    // Disabled accounts retain the explicit CSRF logout lifecycle route.
    let mut disabled = allow();
    disabled["accounts"]["alice"]["enabled"] = false.into();
    policy(root.path(), &disabled);
    check(
        &post(&client, &b, "/board/logout", &signed, &[("csrf", &csrf)]).await,
        303,
        false,
    );
    policy(root.path(), &allow());
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &signed).await, 303, false);
    }
    a.stop().await;
    b.stop().await;
    println!(
        "YT126 actual Admin A/B: auth, policy freshness, native/JSON compatibility, stream close, restart, fail-closed pause, uncertain rotation and graceful drains passed; demo credentials not retained"
    );
}

// Fixed fixture commands return only integer markers. Session keys and binary DUMP
// values never leave server/test memory. This is not a general RESP client.
async fn recovery_integer(port: u16, script: &str, keys: &[&str]) -> i64 {
    assert!(script.len() <= 8192 && keys.len() <= 3);
    let operation = async {
        let count = keys.len().to_string();
        let mut parts = vec!["EVAL", script, count.as_str()];
        parts.extend_from_slice(keys);
        let mut request = format!("*{}\r\n", parts.len()).into_bytes();
        for part in parts {
            assert!(part.len() <= 8192);
            request.extend_from_slice(format!("${}\r\n", part.len()).as_bytes());
            request.extend_from_slice(part.as_bytes());
            request.extend_from_slice(b"\r\n");
        }
        assert!(request.len() <= 16384);
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect owned recovery fixture");
        stream.write_all(&request).await.expect("send fixed script");
        let mut reply = Vec::with_capacity(32);
        loop {
            assert!(reply.len() < 32, "bounded integer reply required");
            let byte = stream.read_u8().await.expect("read integer marker");
            reply.push(byte);
            if byte == b'\n' {
                break;
            }
        }
        assert!(reply.starts_with(b":") && reply.ends_with(b"\r\n"));
        let digits = &reply[1..reply.len() - 2];
        assert!(!digits.is_empty() && digits.iter().all(u8::is_ascii_digit));
        std::str::from_utf8(digits).unwrap().parse().unwrap()
    };
    timeout(Duration::from_secs(2), operation)
        .await
        .expect("fixed recovery script deadline")
}

fn recovery_business(root: &Path) -> Value {
    // One read-only transaction covers full rows, sequence and schema/index SQL.
    // Python sqlite3 is an explicit prerequisite of this optional Linux fixture.
    let output = Command::new("python3")
        .arg("-c")
        .arg(
            r#"import json, pathlib, sqlite3, sys
with sqlite3.connect(pathlib.Path(sys.argv[1]).resolve().as_uri() + '?mode=ro', uri=True, timeout=1) as db:
    db.execute('BEGIN')
    result = {'schema_version': db.execute('PRAGMA user_version').fetchone()[0]}
    for table in ('work_items', 'history', 'sqlite_sequence'):
        result[table] = db.execute('SELECT * FROM ' + table + ' ORDER BY 1').fetchall()
    result['schema'] = db.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').fetchall()
    print(json.dumps(result, separators=(',', ':')))
"#,
        )
        .arg(root.join("data/board.sqlite3"))
        .stdin(Stdio::null())
        .output()
        .expect("Python sqlite3 recovery prerequisite");
    assert!(output.status.success(), "read-only business receipt failed");
    assert!(output.stdout.len() < 16384, "bounded business receipt");
    serde_json::from_slice(&output.stdout).expect("synthetic business receipt JSON")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires caller-owned disposable Valkey at YT139_VALKEY_URL; restores one retired Session key and pauses the instance"]
async fn two_admin_processes_recover_with_new_session_epoch() {
    let endpoint = std::env::var("YT139_VALKEY_URL").expect("dedicated recovery endpoint required");
    let port = endpoint
        .strip_prefix("redis://127.0.0.1:")
        .and_then(|s| s.strip_suffix('/'))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|p| *p != 0)
        .expect("loopback-only recovery endpoint");
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("data")).unwrap();
    std::fs::write(root.path().join("base.toml"), include_str!("../base.toml")).unwrap();
    std::fs::write(root.path().join("production.toml"), "").unwrap();
    // This fresh deployment prefix plus increasing generations models externally
    // controlled never-reused configuration, not a durable epoch allocator.
    let deployment = format!(
        "yt139_{}_{}",
        std::process::id(),
        root.path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace('.', "_")
    );
    let old_epoch = format!("{deployment}_e1");
    let new_epoch = format!("{deployment}_e2");
    let scratch = format!("{deployment}_snapshot");
    policy(root.path(), &allow());
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let mut a = Admin::start(root.path(), &endpoint, &old_epoch).await;
    let mut b = Admin::start(root.path(), &endpoint, &old_epoch).await;
    let mut lifetimes = vec![
        (a.child.id(), a.base.clone()),
        (b.child.id(), b.base.clone()),
    ];
    let (old_cookie, old_csrf) = login(&client, &a, "alice").await;
    check(
        &get(&client, &b, "/board/projects/1/items", &old_cookie).await,
        200,
        false,
    );
    let fields = [
        ("csrf", old_csrf.as_str()),
        ("expected_version", "0"),
        ("title", "YT139 before Session snapshot"),
        ("description", "Retained canonical business row"),
        ("status", "open"),
        ("priority", "normal"),
        ("assignee", "alice"),
    ];
    let created = post(&client, &a, "/board/projects/1/items", &old_cookie, &fields).await;
    check(&created, 303, false);
    let detail = created.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let id: i64 = detail.rsplit('/').next().unwrap().parse().unwrap();
    let identity = old_cookie
        .strip_prefix("yetlet_lab_valkey=")
        .expect("opaque Session cookie");
    assert!(identity.len() == 64 && identity.bytes().all(|c| c.is_ascii_hexdigit()));
    let old_key = format!("yetlet:redis:v1:{old_epoch}:session:{identity}");
    let new_key = format!("yetlet:redis:v1:{new_epoch}:session:{identity}");
    let new_issuance = format!("yetlet:redis:v1:{new_epoch}:issuance");
    // Capture only one active record. Binary RDB payload remains in this scratch
    // hash; no full namespace snapshot, persistence or server crash is modeled.
    assert_eq!(
        recovery_integer(port, r#"
local t = redis.call('TIME')
local now = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)
if redis.call('EXISTS', KEYS[2]) ~= 0 then return 0 end
if redis.call('HGET', KEYS[1], 'status') ~= 'active' or redis.call('HGET', KEYS[1], 'schema') ~= '2' then return 0 end
local expiry = redis.call('PEXPIRETIME', KEYS[1])
if expiry <= now then return 0 end
local blob = redis.call('DUMP', KEYS[1])
if not blob or #blob > 8192 then return 0 end
redis.call('HSET', KEYS[2], 'blob', blob, 'expiry', tostring(expiry))
redis.call('PEXPIRE', KEYS[2], 300000)
return 1
"#, &[&old_key, &scratch]).await,
        1
    );
    let mut update = fields;
    update[1].1 = "1";
    update[2].1 = "YT139 retained current update";
    update[4].1 = "active";
    check(
        &post(&client, &b, &detail, &old_cookie, &update).await,
        303,
        false,
    );
    let mut restricted = allow();
    restricted["accounts"]["alice"]["write"] = false.into();
    restricted["accounts"]["bob"]["enabled"] = false.into();
    policy(root.path(), &restricted);
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &old_cookie).await, 200, false);
        check(
            &post(&client, app, &detail, &old_cookie, &update).await,
            403,
            false,
        );
    }
    check(
        &post(
            &client,
            &b,
            "/board/logout",
            &old_cookie,
            &[("csrf", &old_csrf)],
        )
        .await,
        303,
        false,
    );
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &old_cookie).await, 303, false);
    }
    a.stop().await;
    b.stop().await;
    for (_, base) in &lifetimes {
        let address = base.strip_prefix("http://").unwrap();
        assert!(
            tokio::net::TcpStream::connect(address).await.is_err(),
            "old listener closed before restore"
        );
    }
    let before = recovery_business(root.path());
    let policy_before = std::fs::read(root.path().join("policy.json")).unwrap();
    assert_eq!(before["schema_version"], 1);
    let rows = before["work_items"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], id);
    assert_eq!(rows[0][1], 1);
    assert_eq!(rows[0][2], "YT139 retained current update");
    assert_eq!(rows[0][3], "Retained canonical business row");
    assert_eq!(rows[0][4], "active");
    assert_eq!(rows[0][5], "normal");
    assert_eq!(rows[0][6], "alice");
    assert_eq!(rows[0][7], 2);
    let history = before["history"].as_array().unwrap();
    assert_eq!(history.len(), 2);
    for (index, action) in ["create", "update"].iter().enumerate() {
        assert_eq!(history[index][0], index + 1);
        assert_eq!(history[index][1], id);
        assert_eq!(history[index][2], 1);
        assert_eq!(history[index][3], index + 1);
        assert_eq!(history[index][4], "alice");
        assert_eq!(history[index][5], *action);
    }
    assert_eq!(before["sqlite_sequence"], json!([["history", 2]]));
    // All old consumers are stopped. Overwrite the logout tombstone with the
    // captured active record while preserving its original absolute key expiry.
    assert_eq!(
        recovery_integer(port, r#"
local blob = redis.call('HGET', KEYS[2], 'blob')
local expiry = redis.call('HGET', KEYS[2], 'expiry')
local t = redis.call('TIME')
local now = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)
if not blob or #blob > 8192 or not expiry or tonumber(expiry) <= now then return 0 end
if redis.call('EXISTS', KEYS[3]) ~= 0 then return 0 end
redis.call('RESTORE', KEYS[1], expiry, blob, 'REPLACE', 'ABSTTL')
if redis.call('DUMP', KEYS[1]) ~= blob or redis.call('HGET', KEYS[1], 'status') ~= 'active' or redis.call('HGET', KEYS[1], 'schema') ~= '2' or redis.call('PEXPIRETIME', KEYS[1]) ~= tonumber(expiry) then return 0 end
return 1
"#, &[&old_key, &scratch, &new_key]).await,
        1
    );
    assert_eq!(
        recovery_integer(
            port,
            "return redis.call('EXISTS', KEYS[1], KEYS[2])",
            &[&new_key, &new_issuance]
        )
        .await,
        0
    );
    a = Admin::start(root.path(), &endpoint, &new_epoch).await;
    b = Admin::start(root.path(), &endpoint, &new_epoch).await;
    lifetimes.extend([
        (a.child.id(), a.base.clone()),
        (b.child.id(), b.base.clone()),
    ]);
    let payload = json!({"csrf":old_csrf,"item":{"id":id,"version":2,"title":"Forbidden recovery mutation","description":"Retained canonical business row","status":"active","priority":"normal","assignee":"alice"}});
    let triage = "/board/projects/1/triage";
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &old_cookie).await, 303, false);
        check(
            &post(&client, app, &detail, &old_cookie, &update).await,
            303,
            false,
        );
        let response = client
            .post(format!("{}{triage}", app.base))
            .header(header::COOKIE, &old_cookie)
            .json(&payload)
            .send()
            .await
            .unwrap();
        check(&response, 303, false);
    }
    pause(port, "ALL", 1300).await;
    let (ra, rb) = tokio::join!(
        get(&client, &a, &detail, &old_cookie),
        get(&client, &b, &detail, &old_cookie)
    );
    check(&ra, 503, false);
    check(&rb, 503, false);
    sleep(Duration::from_millis(1400)).await;
    for app in [&a, &b] {
        check(&get(&client, app, &detail, &old_cookie).await, 303, false);
    }
    let (fresh, csrf) = login(&client, &a, "alice").await;
    assert!(
        fresh != old_cookie && csrf != old_csrf,
        "new epoch requires fresh identity/CSRF"
    );
    let mut fresh_update = update;
    fresh_update[0].1 = &csrf;
    let mut fresh_payload = payload.clone();
    fresh_payload["csrf"] = csrf.clone().into();
    for app in [&a, &b] {
        let page = get(&client, app, &detail, &fresh).await;
        check(&page, 200, false);
        let body = page.text().await.unwrap();
        assert!(
            body.contains("YT139 retained current update")
                && body.contains("version 2")
                && body.contains("YT139 before Session snapshot"),
            "retained current row and both history versions rendered"
        );
        check(
            &post(&client, app, &detail, &fresh, &fresh_update).await,
            403,
            false,
        );
        let response = client
            .post(format!("{}{triage}", app.base))
            .header(header::COOKIE, &fresh)
            .json(&fresh_payload)
            .send()
            .await
            .unwrap();
        check(&response, 403, false);
        assert_eq!(
            response.json::<Value>().await.unwrap()["error"]["code"],
            "forbidden"
        );
        check(
            &post(
                &client,
                app,
                "/board/logout",
                &fresh,
                &[("csrf", &old_csrf)],
            )
            .await,
            403,
            false,
        );
        check(&get(&client, app, &detail, &fresh).await, 200, false);
    }
    let (bob, _, _) = login_rotation(&client, &b, "bob").await;
    for app in [&a, &b] {
        check(
            &get(&client, app, "/board/projects/2/items", &bob).await,
            403,
            false,
        );
    }
    a.stop().await;
    b.stop().await;
    let after = recovery_business(root.path());
    assert_eq!(
        before, after,
        "exact current business rows/history/sequence/schema retained"
    );
    assert!(
        policy_before == std::fs::read(root.path().join("policy.json")).unwrap(),
        "current restrictive policy retained byte-for-byte"
    );
    assert_eq!(
        recovery_integer(port, "return redis.call('DEL', KEYS[1])", &[&scratch]).await,
        1
    );
    assert_eq!(
        recovery_integer(port, "return redis.call('EXISTS', KEYS[1])", &[&scratch]).await,
        0
    );
    for (pid, base) in &lifetimes {
        // SAFETY: signal zero observes only these already-reaped owned children.
        assert_eq!(
            unsafe { libc::kill(*pid as i32, 0) },
            -1,
            "owned child reaped"
        );
        assert!(
            tokio::net::TcpStream::connect(base.strip_prefix("http://").unwrap())
                .await
                .is_err(),
            "owned listener closed"
        );
    }
    let root_path = root.path().to_owned();
    root.close().unwrap();
    assert!(
        !root_path.exists(),
        "temporary business/policy data removed"
    );
    println!(
        "YT139 PASS: selected-record stale restore; new epoch; old Cookie/CSRF refusal; provider503; fresh cross-instance read/current policy denial; exact 1 item/version2, 2 history rows, sequence/schema/policy retained; scratch removed; four drains/reaped/closed listeners; temp data removed"
    );
    for (pid, base) in lifetimes {
        println!("YT139 cleaned Admin pid={pid} listener={base}");
    }
    println!("YT139 cleaned temporary root={}", root_path.display());
    println!("YT139 synthetic business receipt={before}");
}
