//! One sequential opt-in test against a dedicated, disposable Valkey only.
//! CLIENT PAUSE affects the whole server: never point this at a shared service.
use reqwest::{Client, Response, header};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::time::{Instant, sleep, timeout};

struct AppProcess {
    child: Child,
    output: PathBuf,
    base: String,
}

impl AppProcess {
    async fn start(root: &Path, endpoint: &str, namespace: &str, instance: &str) -> Self {
        let output = root.join(format!("{instance}.out"));
        let child = Command::new(env!("CARGO_BIN_EXE_yt125-auth-valkey"))
            .env_clear()
            .env("YT125_VALKEY_URL", endpoint)
            .env("YT125_NAMESPACE", namespace)
            .env("YT125_POLICY_FILE", root.join("policy.json"))
            .env("YT125_INSTANCE", instance)
            .env("YT125_TTL_SECONDS", "60")
            .stdin(Stdio::null())
            .stdout(File::create(&output).unwrap())
            .stderr(Stdio::null())
            .spawn()
            .expect("start fixture process");
        // Construct the cleanup guard before any readiness assertion can fail.
        let mut app = Self {
            child,
            output,
            base: String::new(),
        };
        let end = Instant::now() + Duration::from_secs(8);
        loop {
            assert!(app.child.try_wait().unwrap().is_none(), "startup failed");
            let ready = std::fs::read_to_string(&app.output).unwrap();
            if let Some(address) = ready.trim().strip_prefix(&format!("READY {instance} ")) {
                let address: std::net::SocketAddr = address.parse().expect("safe readiness line");
                assert!(address.ip().is_loopback());
                app.base = format!("http://{address}");
                return app;
            }
            assert!(Instant::now() < end, "startup timeout");
            sleep(Duration::from_millis(20)).await;
        }
    }

    async fn stop(&mut self) {
        // SAFETY: signal only this still-owned unreaped child PID; no process
        // group or externally discovered process is targeted.
        assert_eq!(
            unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) },
            0
        );
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success(), "graceful drain failed");
                let output = std::fs::read_to_string(&self.output).unwrap();
                assert!(output.ends_with("STOPPED drained\n"));
                assert_eq!(output.lines().count(), 2, "unexpected retained output");
                return;
            }
            assert!(Instant::now() < end, "graceful shutdown timed out");
            sleep(Duration::from_millis(20)).await;
        }
    }
}

impl Drop for AppProcess {
    fn drop(&mut self) {
        // Failure/panic cleanup: kill and reap even when graceful stop was not reached.
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn policy(root: &Path, value: &str) {
    let pending = root.join("policy.next");
    std::fs::write(&pending, value).unwrap();
    std::fs::rename(pending, root.join("policy.json")).unwrap();
}

fn scoped(response: &Response, status: u16, cookie: bool) {
    assert_eq!(response.status().as_u16(), status);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(response.headers().get_all(header::VARY).iter().any(|h| {
        h.to_str()
            .unwrap()
            .split(',')
            .any(|v| v.trim().eq_ignore_ascii_case("cookie"))
    }));
    assert_eq!(response.headers().contains_key(header::SET_COOKIE), cookie);
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert!(
        response
            .headers()
            .contains_key(header::CONTENT_SECURITY_POLICY)
    );
    if status == 303 {
        assert!(matches!(
            response.headers()[header::LOCATION].to_str().unwrap(),
            "/auth/form" | "/auth/private"
        ));
    } else {
        assert!(!response.headers().contains_key(header::LOCATION));
    }
}

fn cookie(response: &Response) -> String {
    assert_eq!(
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .count(),
        1
    );
    let value = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(value.contains("HttpOnly") && value.contains("SameSite=Lax"));
    assert!(value.contains("Path=/") && value.contains("Max-Age=60"));
    assert!(!value.contains("Secure")); // Explicit loopback HTTP fixture override.
    value.split(';').next().unwrap().to_owned()
}

async fn csrf(response: Response) -> String {
    let body = response.text().await.unwrap();
    assert!(!body.contains("<script"));
    let value = body
        .split("name=\"csrf\" value=\"")
        .nth(1)
        .expect("native form CSRF field")
        .split('"')
        .next()
        .unwrap();
    assert_eq!(value.len(), 64);
    value.to_owned()
}

async fn get(client: &Client, app: &AppProcess, path: &str, cookie: Option<&str>) -> Response {
    let mut request = client.get(format!("{}{path}", app.base));
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    request.send().await.expect("loopback GET")
}

async fn post(
    client: &Client,
    app: &AppProcess,
    path: &str,
    cookie: &str,
    body: String,
) -> Response {
    client
        .post(format!("{}{path}", app.base))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("loopback POST")
}

async fn login(client: &Client, app: &AppProcess) -> (String, String) {
    let form = get(client, app, "/auth/form", None).await;
    scoped(&form, 200, true);
    let anonymous_cookie = cookie(&form);
    let anonymous_csrf = csrf(form).await;
    let response = post(
        client,
        app,
        "/auth/login",
        &anonymous_cookie,
        format!("csrf={anonymous_csrf}"),
    )
    .await;
    scoped(&response, 303, true);
    let signed_cookie = cookie(&response);
    assert!(signed_cookie != anonymous_cookie, "login must rotate ID");
    let private = get(client, app, "/auth/private", Some(&signed_cookie)).await;
    scoped(&private, 200, false);
    let signed_csrf = csrf(private).await;
    assert!(signed_csrf != anonymous_csrf, "login must rotate CSRF");
    (signed_cookie, signed_csrf)
}

async fn pause(connection: &mut redis::aio::MultiplexedConnection, mode: &str, ms: u64) {
    timeout(
        Duration::from_secs(2),
        redis::cmd("CLIENT")
            .arg("PAUSE")
            .arg(ms)
            .arg(mode)
            .query_async::<()>(connection),
    )
    .await
    .expect("bounded admin command")
    .expect("disposable Valkey pause");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires parent-owned disposable Valkey at YT125_VALKEY_URL; pauses the whole server"]
async fn two_process_auth_lifecycle_and_policy() {
    let endpoint = std::env::var("YT125_VALKEY_URL").expect("explicit disposable Valkey required");
    let port = endpoint
        .strip_prefix("redis://127.0.0.1:")
        .and_then(|s| s.strip_suffix('/'))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|p| *p != 0);
    assert!(port.is_some(), "dedicated loopback endpoint required");
    let root = tempfile::tempdir().unwrap();
    let namespace = format!(
        "yt125_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let enabled = r#"{"enabled":true,"can_view":true}"#;
    policy(root.path(), enabled);
    let mut a = AppProcess::start(root.path(), &endpoint, &namespace, "A").await;
    let mut b = AppProcess::start(root.path(), &endpoint, &namespace, "B").await;
    assert_ne!(a.child.id(), b.child.id());
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let mut admin = timeout(
        Duration::from_secs(2),
        redis::Client::open(endpoint.as_str())
            .unwrap()
            .get_multiplexed_async_connection(),
    )
    .await
    .unwrap()
    .unwrap();

    for app in [&a, &b] {
        let health = get(&client, app, "/health", Some("yt125_session=malformed")).await;
        assert_eq!(health.status(), 200);
        assert!(!health.headers().contains_key(header::SET_COOKIE));
        assert!(!health.headers().contains_key(header::CACHE_CONTROL));
        for path in ["/auth/private", "/auth/optional"] {
            scoped(&get(&client, app, path, None).await, 303, false);
        }
    }
    let form = get(&client, &a, "/auth/form", None).await;
    scoped(&form, 200, true);
    let anonymous = cookie(&form);
    let token = csrf(form).await;
    let optional = get(&client, &b, "/auth/optional", Some(&anonymous)).await;
    scoped(&optional, 200, false);
    assert_eq!(optional.text().await.unwrap(), "anonymous");
    scoped(
        &get(&client, &b, "/auth/private", Some(&anonymous)).await,
        303,
        false,
    );
    // Browser assertions cannot select identity, permission, or a redirect target.
    for body in [
        "csrf=wrong".to_owned(),
        format!("csrf={token}&account=admin&can_view=true&return_to=https://invalid.example"),
        "csrf=".to_owned() + &"x".repeat(4097),
    ] {
        let expected = if body.len() > 4096 {
            413
        } else if body.contains("account=") {
            400
        } else {
            403
        };
        scoped(
            &post(&client, &a, "/auth/login", &anonymous, body).await,
            expected,
            false,
        );
    }
    let response = post(
        &client,
        &a,
        "/auth/login",
        &anonymous,
        format!("csrf={token}"),
    )
    .await;
    scoped(&response, 303, true);
    let signed = cookie(&response);
    assert!(signed != anonymous);
    let private = get(&client, &b, "/auth/private", Some(&signed)).await;
    scoped(&private, 200, false);
    let signed_csrf = csrf(private).await;
    assert!(signed_csrf != token);
    scoped(
        &get(&client, &a, "/auth/optional", Some(&anonymous)).await,
        303,
        false,
    );
    scoped(
        &post(
            &client,
            &b,
            "/auth/logout",
            &signed,
            format!("csrf={token}"),
        )
        .await,
        403,
        false,
    );

    // The same live policy authority changes subsequent decisions in both OS processes.
    policy(root.path(), r#"{"enabled":true,"can_view":false}"#);
    for app in [&a, &b] {
        scoped(
            &get(&client, app, "/auth/private", Some(&signed)).await,
            403,
            false,
        );
        let optional = get(&client, app, "/auth/optional", Some(&signed)).await;
        scoped(&optional, 200, false);
        assert_eq!(optional.text().await.unwrap(), "signed-in");
    }
    policy(root.path(), r#"{"enabled":false,"can_view":true}"#);
    for app in [&a, &b] {
        for path in ["/auth/private", "/auth/optional"] {
            scoped(&get(&client, app, path, Some(&signed)).await, 403, false);
        }
    }
    for invalid in ["not JSON".to_owned(), " ".repeat(1025)] {
        policy(root.path(), &invalid);
        for app in [&a, &b] {
            scoped(
                &get(&client, app, "/auth/optional", Some(&signed)).await,
                503,
                false,
            );
        }
    }
    std::fs::remove_file(root.path().join("policy.json")).unwrap();
    for app in [&a, &b] {
        scoped(
            &get(&client, app, "/auth/optional", Some(&signed)).await,
            503,
            false,
        );
    }
    policy(root.path(), enabled);

    // Restart a process while Valkey and policy survive; the same cookie continues.
    b.stop().await;
    b = AppProcess::start(root.path(), &endpoint, &namespace, "B").await;
    scoped(
        &get(&client, &b, "/auth/private", Some(&signed)).await,
        200,
        false,
    );
    for app in [&a, &b] {
        for bad in [
            "yt125_session=malformed".to_owned(),
            format!("{signed}; {signed}"),
        ] {
            scoped(
                &get(&client, app, "/auth/optional", Some(&bad)).await,
                400,
                false,
            );
        }
    }
    scoped(
        &post(
            &client,
            &b,
            "/auth/logout",
            &signed,
            format!("csrf={signed_csrf}"),
        )
        .await,
        303,
        false,
    );
    for app in [&a, &b] {
        for path in ["/auth/private", "/auth/optional"] {
            scoped(&get(&client, app, path, Some(&signed)).await, 303, false);
        }
    }

    let (signed, signed_csrf) = login(&client, &a).await;
    pause(&mut admin, "ALL", 1300).await;
    // Concurrent reads ensure both requests occur inside the same finite pause.
    let (left, right) = tokio::join!(
        get(&client, &a, "/auth/private", Some(&signed)),
        get(&client, &b, "/auth/optional", Some(&signed)),
    );
    scoped(&left, 503, false);
    scoped(&right, 503, false);
    assert_eq!(get(&client, &a, "/health", None).await.status(), 200);
    sleep(Duration::from_millis(1400)).await;
    for app in [&a, &b] {
        scoped(
            &get(&client, app, "/auth/private", Some(&signed)).await,
            200,
            false,
        );
    }

    // Pause writes, while authoritative reads still work. The mutation is sent,
    // times out at the provider boundary, and commits after the response is 503.
    // Check retired server authority after recovery, never retry the mutation.
    pause(&mut admin, "WRITE", 750).await;
    scoped(
        &post(
            &client,
            &b,
            "/auth/logout",
            &signed,
            format!("csrf={signed_csrf}"),
        )
        .await,
        503,
        false,
    );
    sleep(Duration::from_millis(850)).await;
    for app in [&a, &b] {
        scoped(
            &get(&client, app, "/auth/optional", Some(&signed)).await,
            303,
            false,
        );
    }
    let form = get(&client, &a, "/auth/form", None).await;
    scoped(&form, 200, true);
    let anonymous = cookie(&form);
    let token = csrf(form).await;
    pause(&mut admin, "WRITE", 750).await;
    scoped(
        &post(
            &client,
            &a,
            "/auth/login",
            &anonymous,
            format!("csrf={token}"),
        )
        .await,
        503,
        false,
    );
    sleep(Duration::from_millis(850)).await;
    for app in [&a, &b] {
        // A delayed rotation did not publish the new cookie or restore the old one.
        scoped(
            &get(&client, app, "/auth/optional", Some(&anonymous)).await,
            303,
            false,
        );
    }
    let (recovered, _) = login(&client, &b).await;
    scoped(
        &get(&client, &a, "/auth/private", Some(&recovered)).await,
        200,
        false,
    );
    a.stop().await;
    b.stop().await;
    // The parent removes the disposable Valkey container; no global FLUSH or
    // deletion outside this fixture's temporary directory is issued by the test.
}
