use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use tower::ServiceExt;

fn fixture() -> Oidc {
    fixture_at("http://127.0.0.1:3014")
}

fn fixture_at(origin: &str) -> Oidc {
    let metadata: CoreProviderMetadata = serde_json::from_value(serde_json::json!({
        "issuer": "https://issuer.example/",
        "authorization_endpoint": "https://issuer.example/authorize",
        "token_endpoint": "https://issuer.example/token",
        "jwks_uri": "https://issuer.example/jwks",
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    }))
    .unwrap();
    Oidc {
        client: CoreClient::from_provider_metadata(
            metadata,
            ClientId::new("test-client".into()),
            Some(ClientSecret::new("synthetic-secret".into())),
        )
        .set_redirect_uri(RedirectUrl::new(format!("{origin}/board/oidc/callback")).unwrap()),
        http: reqwest::Client::new(),
        issuer: "https://issuer.example/".into(),
        subjects: BTreeMap::new(),
    }
}

#[tokio::test]
async fn configured_oidc_origin_is_distinct_from_listener() {
    let dir = tempfile::tempdir().unwrap();
    for port in [3014, 41234] {
        let origin = format!("http://127.0.0.1:{port}");
        let authority = format!("127.0.0.1:{port}");
        let board = crate::Board::open_with_oidc(
            &dir.path().join(format!("board-{port}.sqlite3")),
            "127.0.0.1:3014".parse().unwrap(),
            fixture_at(&origin),
            Vec::new(),
            None,
        )
        .await
        .unwrap();
        for (path, host, origins, status) in [
            ("/board/login", authority.as_str(), vec![], StatusCode::OK),
            (
                "/board/oidc/callback",
                authority.as_str(),
                vec![],
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/board/oidc/callback",
                authority.as_str(),
                vec![origin.as_str()],
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/board/oidc/callback",
                authority.as_str(),
                vec!["null"],
                StatusCode::FORBIDDEN,
            ),
            (
                "/board/oidc/callback",
                authority.as_str(),
                vec!["http://other.example"],
                StatusCode::FORBIDDEN,
            ),
            (
                "/board/oidc/callback",
                authority.as_str(),
                vec![origin.as_str(), origin.as_str()],
                StatusCode::FORBIDDEN,
            ),
            (
                "/board/login",
                "other.example",
                vec![],
                StatusCode::FORBIDDEN,
            ),
        ] {
            let mut request = Request::builder()
                .uri(path)
                .header(header::HOST, host)
                .header("forwarded", format!("host={authority};proto=http"))
                .header("x-forwarded-host", &authority)
                .body(Body::empty())
                .unwrap();
            for value in origins {
                request
                    .headers_mut()
                    .append(header::ORIGIN, value.parse().unwrap());
            }
            let response = board.router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), status);
        }
        if port != 3014 {
            assert_eq!(
                call(
                    &board.router,
                    Method::GET,
                    "/board/login",
                    "",
                    String::new()
                )
                .await
                .0,
                StatusCode::FORBIDDEN
            );
        }
        board.drain().await.unwrap();
    }
    let demo = crate::Board::open(
        &dir.path().join("demo.sqlite3"),
        "127.0.0.1:3014".parse().unwrap(),
        crate::DemoIdentities::generate().unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        call(&demo.router, Method::GET, "/board/login", "", String::new())
            .await
            .0,
        StatusCode::OK
    );
    demo.drain().await.unwrap();
    for address in ["0.0.0.0:3014", "127.0.0.1:0"] {
        assert!(
            crate::Board::open_with_oidc(
                &dir.path().join("invalid.sqlite3"),
                address.parse().unwrap(),
                fixture(),
                Vec::new(),
                None
            )
            .await
            .is_err()
        );
    }
}

#[cfg(all(feature = "auth-valkey", target_os = "linux"))]
#[tokio::test]
#[ignore = "requires explicit task-owned YT140_VALKEY_URL"]
async fn proxy_callback_consumes_shared_pending_across_board_instances() {
    use futures_util::FutureExt;
    use std::{
        panic::AssertUnwindSafe,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };
    use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle, time::timeout};

    // Only the three actual HTTP task lifetimes need this abort-on-drop guard.
    struct HttpTask {
        stop: Option<oneshot::Sender<()>>,
        task: Option<JoinHandle<()>>,
    }
    impl HttpTask {
        fn start(listener: TcpListener, router: axum::Router) -> Self {
            let (stop, stopped) = oneshot::channel();
            let task = tokio::spawn(async move {
                axum::serve(listener, router)
                    .with_graceful_shutdown(async {
                        let _ = stopped.await;
                    })
                    .await
                    .unwrap_or_else(|_| panic!("owned HTTP server failed"));
            });
            Self {
                stop: Some(stop),
                task: Some(task),
            }
        }
        async fn stop(&mut self) {
            let _ = self.stop.take().unwrap().send(());
            assert!(
                matches!(
                    timeout(Duration::from_secs(4), self.task.as_mut().unwrap()).await,
                    Ok(Ok(()))
                ),
                "owned HTTP task stopped and joined"
            );
            self.task.take();
        }
    }
    impl Drop for HttpTask {
        fn drop(&mut self) {
            if let Some(stop) = self.stop.take() {
                let _ = stop.send(());
            }
            if let Some(task) = self.task.take() {
                task.abort();
            }
        }
    }
    async fn send(request: reqwest::RequestBuilder) -> reqwest::Response {
        request
            .send()
            .await
            .unwrap_or_else(|_| panic!("bounded fixture HTTP request failed"))
    }
    async fn text(response: reqwest::Response) -> String {
        response
            .text()
            .await
            .unwrap_or_else(|_| panic!("fixture response read failed"))
    }
    fn private_cookie(response: &reqwest::Response) -> String {
        let value = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_else(|| panic!("expected session cookie"));
        assert!(
            value.starts_with("yetlet_lab_valkey=")
                && value.contains("HttpOnly")
                && value.contains("SameSite=Lax")
                && value.contains("Path=/")
                && !value.contains("Domain=")
                && !value.contains("Secure"),
            "local cookie scope"
        );
        value.split(';').next().unwrap().to_owned()
    }
    async fn initiate(
        client: &reqwest::Client,
        origin: &str,
    ) -> (String, BTreeMap<String, String>) {
        let page = send(client.get(format!("{origin}/board/login"))).await;
        assert_eq!(page.status(), StatusCode::OK);
        let anonymous = private_cookie(&page);
        let html = text(page).await;
        let csrf = html
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .and_then(|v| v.split('"').next())
            .unwrap_or_else(|| panic!("expected native CSRF field"));
        let response = send(
            client
                .post(format!("{origin}/board/oidc/login"))
                .header(header::COOKIE, &anonymous)
                .header(header::ORIGIN, origin)
                .form(&[("csrf", csrf)]),
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let rotated = private_cookie(&response);
        assert!(rotated != anonymous, "initiation rotates session");
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| reqwest::Url::parse(v).ok())
            .unwrap_or_else(|| panic!("authorization location"));
        let parameters: BTreeMap<String, String> = location
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert!(
            parameters["redirect_uri"] == format!("{origin}/board/oidc/callback"),
            "canonical redirect"
        );
        assert!(
            parameters["code_challenge_method"] == "S256",
            "S256 selection"
        );
        let stale = send(
            client
                .get(format!("{origin}/board/projects/1/items"))
                .header(header::COOKIE, anonymous),
        )
        .await;
        assert_eq!(stale.status(), StatusCode::SEE_OTHER);
        assert!(!stale.headers().contains_key(header::SET_COOKIE));
        (rotated, parameters)
    }
    async fn pending(board: &crate::Board, cookie: &str) -> Option<Pending> {
        let crate::sessions::Manager::Valkey(manager) = &board.sessions else {
            panic!("remote manager required")
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            cookie
                .parse()
                .unwrap_or_else(|_| panic!("fixture cookie syntax")),
        );
        let session = manager
            .open(&headers)
            .unwrap_or_else(|_| panic!("remote handle"));
        let record = session
            .read()
            .await
            .unwrap_or_else(|_| panic!("remote record"));
        assert!(record.user.is_none(), "pending state never authenticates");
        record.oidc
    }
    async fn callback_result(response: reqwest::Response, state: &str) -> u16 {
        let status = response.status().as_u16();
        assert!(
            !response.headers().contains_key(header::SET_COOKIE),
            "no success cookie"
        );
        assert_eq!(response.headers()[header::REFERRER_POLICY], "no-referrer");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            crate::DEFAULT_CSP
        );
        assert!(
            !text(response).await.contains(state),
            "callback errors redact state"
        );
        status
    }

    let endpoint = std::env::var("YT140_VALKEY_URL")
        .unwrap_or_else(|_| panic!("explicit task-owned Valkey required"));
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_owned();
    let policy = root.join("policy.json");
    std::fs::write(
        root.join("policy-next.json"),
        r#"{"accounts":{"alice":{"enabled":true,"read":true,"write":true}}}"#,
    )
    .unwrap();
    std::fs::rename(root.join("policy-next.json"), &policy).unwrap();
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).unwrap();
    let namespace = format!(
        "yt140_{}",
        random
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    );
    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let b_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addresses = [
        proxy_listener.local_addr().unwrap(),
        a_listener.local_addr().unwrap(),
        b_listener.local_addr().unwrap(),
    ];
    let origin = format!("http://{}", addresses[0]);
    let backend = || crate::SessionBackend::AuthValkey {
        endpoint: endpoint.clone(),
        namespace: namespace.clone(),
        ttl: Duration::from_secs(1800),
        policy_file: policy.clone(),
    };
    let a = crate::Board::open_with_oidc_and_sessions(
        &root.join("board.sqlite3"),
        addresses[1],
        fixture_at(&origin),
        Vec::new(),
        None,
        backend(),
    )
    .await
    .unwrap();
    let b = crate::Board::open_with_oidc_and_sessions(
        &root.join("board.sqlite3"),
        addresses[2],
        fixture_at(&origin),
        Vec::new(),
        None,
        backend(),
    )
    .await
    .unwrap();
    let mut a_http = HttpTask::start(a_listener, a.router.clone());
    let mut b_http = HttpTask::start(b_listener, b.router.clone());
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let next_callback = Arc::new(AtomicUsize::new(1));
    let stripped = Arc::new(AtomicUsize::new(0));
    let targets = [
        format!("http://{}", addresses[1]),
        format!("http://{}", addresses[2]),
    ];
    let proxy = axum::Router::new().fallback({
        let client = client.clone();
        let next = next_callback.clone();
        let stripped = stripped.clone();
        move |request: axum::extract::Request| {
            let client = client.clone();
            let next = next.clone();
            let stripped = stripped.clone();
            let targets = targets.clone();
            async move {
                let work = async {
                    let target = if request.uri().path() == "/board/oidc/callback" {
                        next.fetch_add(1, Ordering::SeqCst) % 2
                    } else {
                        0
                    };
                    let (parts, body) = request.into_parts();
                    let body = to_bytes(body, 65536).await.map_err(|_| ())?;
                    let mut headers = parts.headers;
                    let untrusted: Vec<_> = headers
                        .keys()
                        .filter(|key| {
                            key.as_str() == "forwarded" || key.as_str().starts_with("x-forwarded-")
                        })
                        .cloned()
                        .collect();
                    stripped.fetch_add(untrusted.len(), Ordering::SeqCst);
                    for key in untrusted {
                        headers.remove(key);
                    }
                    for key in [
                        header::CONNECTION,
                        header::TRANSFER_ENCODING,
                        header::CONTENT_LENGTH,
                    ] {
                        headers.remove(key);
                    }
                    let mut response = client
                        .request(parts.method, format!("{}{}", targets[target], parts.uri))
                        .headers(headers)
                        .body(body)
                        .send()
                        .await
                        .map_err(|_| ())?;
                    let mut output = axum::response::Response::builder().status(response.status());
                    *output.headers_mut().ok_or(())? = response.headers().clone();
                    let mut bytes = Vec::new();
                    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
                        if bytes.len() + chunk.len() > 65536 {
                            return Err(());
                        }
                        bytes.extend_from_slice(&chunk);
                    }
                    output.body(Body::from(bytes)).map_err(|_| ())
                };
                match timeout(Duration::from_secs(4), work).await {
                    Ok(Ok(response)) => response,
                    _ => (StatusCode::BAD_GATEWAY, "Fixture proxy unavailable").into_response(),
                }
            }
        }
    });
    let mut proxy_http = HttpTask::start(proxy_listener, proxy);
    let result = AssertUnwindSafe(async {
        let (cookie, parameters) = initiate(&client, &origin).await;
        let transaction = pending(&b, &cookie).await.unwrap_or_else(|| panic!("B must see A transaction"));
        assert!(transaction.state == parameters["state"], "shared state");
        assert!(transaction.nonce.secret() == &parameters["nonce"], "shared nonce");
        assert!(PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(transaction.verifier.clone())).as_str()
            == parameters["code_challenge"], "shared PKCE verifier");
        assert!(transaction.deadline > SystemTime::now() && transaction.deadline <= SystemTime::now() + Duration::from_secs(300), "absolute pending deadline");
        let state = &parameters["state"];
        let callback = format!("{origin}/board/oidc/callback?state={state}");
        for bad_origin in ["null", "http://other.example"] {
            let response = send(client.get(&callback).header(header::COOKIE, &cookie).header(header::ORIGIN, bad_origin)).await;
            assert_eq!(callback_result(response, state).await, 403);
            assert!(pending(&b, &cookie).await.is_some(), "origin denial precedes consume");
        }
        let response = send(client.get(&callback).header(header::COOKIE, &cookie)
            .header(header::ORIGIN, &origin).header(header::ORIGIN, &origin)).await;
        assert_eq!(callback_result(response, state).await, 403);
        assert!(pending(&b, &cookie).await.is_some(), "duplicate origin precedes consume");
        let response = send(client.get(&callback).header(header::COOKIE, &cookie).header(header::HOST, addresses[1].to_string())
            .header("forwarded", format!("host={};proto=http", addresses[0]))
            .header("x-forwarded-host", addresses[0].to_string()).header("x-forwarded-proto", "http")).await;
        assert_eq!(callback_result(response, state).await, 403);
        assert!(stripped.load(Ordering::SeqCst) >= 3, "proxy strips forwarding headers");
        assert!(pending(&b, &cookie).await.is_some(), "Host denial precedes consume");
        // Direct injection cannot use forwarding headers to rescue a bad Host.
        let response = send(client.get(format!("http://{}/board/oidc/callback?state={state}", addresses[2]))
            .header(header::COOKIE, &cookie).header("x-forwarded-host", addresses[0].to_string())
            .header("x-forwarded-proto", "http")).await;
        assert_eq!(callback_result(response, state).await, 403);
        assert!(pending(&b, &cookie).await.is_some(), "untrusted forwarding ignored");
        // Duplicate Host is checked with native Router headers: HTTP servers may reject it earlier.
        let response = b.router.clone().oneshot(Request::builder().uri(format!("/board/oidc/callback?state={state}"))
            .header(header::HOST, addresses[0].to_string()).header(header::HOST, addresses[0].to_string())
            .header(header::COOKIE, &cookie).body(Body::empty()).unwrap_or_else(|_| panic!("fixture request"))).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(pending(&b, &cookie).await.is_some(), "duplicate Host precedes consume");
        next_callback.store(1, Ordering::SeqCst);
        let response = send(client.get(&callback).header(header::COOKIE, &cookie).header(header::ORIGIN, &origin)
            .header("forwarded", "host=evil.example;proto=https").header("x-forwarded-host", "evil.example")
            .header("x-forwarded-proto", "https")).await;
        assert_eq!(callback_result(response, state).await, 400, "B consumes A state before missing-code rejection");
        assert!(pending(&a, &cookie).await.is_none(), "shared transaction retired");
        for target in [0, 1] {
            next_callback.store(target, Ordering::SeqCst);
            assert_eq!(callback_result(send(client.get(&callback).header(header::COOKIE, &cookie)).await, state).await, 401);
        }
        let (cookie, parameters) = initiate(&client, &origin).await;
        let state = &parameters["state"];
        let callback = format!("{origin}/board/oidc/callback?state={state}");
        next_callback.store(0, Ordering::SeqCst);
        let (first, second) = tokio::join!(
            send(client.get(&callback).header(header::COOKIE, &cookie)),
            send(client.get(&callback).header(header::COOKIE, &cookie)));
        let mut statuses = [callback_result(first, state).await, callback_result(second, state).await];
        statuses.sort();
        assert!(statuses == [400, 401] || statuses == [400, 409], "exactly one acknowledged callback consumer");
        assert!(pending(&b, &cookie).await.is_none(), "race retires shared transaction");
        assert_eq!(callback_result(send(client.get(&callback).header(header::COOKIE, &cookie)).await, state).await, 401);
        for (index, board) in [&a, &b].into_iter().enumerate() {
            let protected = send(client.get(format!("http://{}/board/projects/1/items", addresses[index+1]))
                .header(header::HOST, addresses[0].to_string()).header(header::COOKIE, &cookie)).await;
            assert_eq!(protected.status(), StatusCode::SEE_OTHER);
            assert!(!protected.headers().contains_key(header::SET_COOKIE));
            for project in [1, 2] {
                let list = board.database.list(project, crate::domain::ListQuery::default().normalize().unwrap()).await.unwrap();
                assert_eq!(list.matching, 0);
                assert!(list.items.is_empty(), "no business effects");
            }
        }
        println!("YT140 observations PASS: A initiation/B missing-code400; shared state/nonce/PKCE/deadline; Host/Origin preconsume and forwarding refusal; replay401 both; race single consumer; no authenticated access; empty shared business lists");
    }).catch_unwind().await;
    proxy_http.stop().await;
    a.stop();
    b.stop();
    a_http.stop().await;
    b_http.stop().await;
    a.drain().await.unwrap();
    b.drain().await.unwrap();
    for address in addresses {
        assert!(
            tokio::net::TcpStream::connect(address).await.is_err(),
            "owned listener closed"
        );
        println!("YT140 cleaned listener={address}");
    }
    drop(a);
    drop(b);
    dir.close().unwrap();
    assert!(!root.exists(), "temporary SQLite/policy root removed");
    println!(
        "YT140 cleaned root={}; three HTTP tasks stopped/joined; two Board/provider drains completed",
        root.display()
    );
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

#[cfg(feature = "valkey")]
#[test]
fn pending_oidc_state_round_trips_without_serializing_permissions() {
    let (_, pending) = fixture().start();
    let state = LoginSession {
        user: Some(principal("viewer").unwrap()),
        oidc: Some(pending.clone()),
    };
    let encoded = serde_json::to_string(&state).unwrap();
    assert!(!encoded.contains("write") && !encoded.contains("project"));
    let decoded: LoginSession = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.user, state.user);
    assert!(decoded.oidc.unwrap().accepts(&pending.state));
}

#[test]
fn each_attempt_has_fresh_state_nonce_and_s256_without_secrets_in_debug() {
    let oidc = fixture();
    let (first, pending) = oidc.start();
    let (second, _) = oidc.start();
    let params = |url: String| {
        reqwest::Url::parse(&url)
            .unwrap()
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<BTreeMap<_, _>>()
    };
    let (a, b) = (params(first), params(second));
    assert_eq!(a["scope"], "openid");
    assert_eq!(a["response_type"], "code");
    assert_eq!(a["code_challenge_method"], "S256");
    for key in ["state", "nonce", "code_challenge"] {
        assert_ne!(a[key], b[key]);
    }
    assert!(!a.contains_key("client_secret"));
    assert!(pending.accepts(&a["state"]));
    assert!(!pending.accepts("wrong"));
    assert_eq!(format!("{pending:?}"), "Pending { .. }");
    let expired = Pending {
        deadline: SystemTime::now() - Duration::from_secs(1),
        ..pending
    };
    assert!(!expired.accepts(&a["state"]));
}

#[test]
fn mappings_are_explicit_existing_permissions_and_unassigned_accounts_get_no_role() {
    assert!(subject_map(BTreeMap::new()).unwrap().is_empty());
    let map = subject_map(BTreeMap::from([("subject-one".into(), "viewer".into())])).unwrap();
    assert!(!map["subject-one"].write);
    assert_eq!(map["subject-one"].project, 1);
    assert!(!map.contains_key("another-subject"));
    for (subject, role) in [
        ("", "alice"),
        ("bad\0subject", "alice"),
        ("subject", "administrator"),
    ] {
        assert_eq!(
            subject_map(BTreeMap::from([(subject.into(), role.into())])).unwrap_err(),
            Failure::Invalid
        );
    }
}

#[tokio::test]
async fn invalid_issuer_is_rejected_before_network_access() {
    for issuer in [
        "http://issuer.example/",
        "https://user@issuer.example/",
        "https://issuer.example/?private",
    ] {
        assert!(matches!(
            Oidc::discover(
                issuer,
                "client",
                "http://127.0.0.1:3014/board/oidc/callback",
                BTreeMap::new(),
                ClientSecret::new("synthetic".into())
            )
            .await,
            Err(Failure::Invalid)
        ));
    }
}

async fn call(
    router: &axum::Router,
    method: Method,
    path: &str,
    cookie: &str,
    body: String,
) -> (StatusCode, HeaderMap, String) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::HOST, "127.0.0.1:3014")
                .header(header::ORIGIN, "http://127.0.0.1:3014")
                .header(header::COOKIE, cookie)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let html = String::from_utf8(
        to_bytes(response.into_body(), 65536)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    (status, headers, html)
}
fn cookie(headers: &HeaderMap) -> String {
    headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn start_requires_csrf_rotates_session_and_disables_demo_login() {
    let dir = tempfile::tempdir().unwrap();
    let board = crate::Board::open_with_oidc(
        &dir.path().join("board.sqlite3"),
        "127.0.0.1:3014".parse().unwrap(),
        fixture(),
        Vec::new(),
        None,
    )
    .await
    .unwrap();
    let (status, headers, html) = call(
        &board.router,
        Method::GET,
        "/board/login",
        "",
        String::new(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("form-action 'self' https://issuer.example")
    );
    assert!(html.contains("Continue with identity provider"));
    assert!(!html.contains("name=\"code\""));
    let old_cookie = cookie(&headers);
    let csrf = html
        .split("name=\"csrf\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert_eq!(
        call(
            &board.router,
            Method::POST,
            "/board/oidc/login",
            &old_cookie,
            "csrf=wrong".into()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &board.router,
            Method::POST,
            "/board/login",
            &old_cookie,
            format!("csrf={csrf}&username=alice&code=synthetic")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, headers, _) = call(
        &board.router,
        Method::POST,
        "/board/oidc/login",
        &old_cookie,
        format!("csrf={csrf}"),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_ne!(cookie(&headers), old_cookie);
    assert!(
        headers[header::LOCATION]
            .to_str()
            .unwrap()
            .starts_with("https://issuer.example/authorize?")
    );
    assert_eq!(
        call(
            &board.router,
            Method::GET,
            "/board/projects/1/items",
            &old_cookie,
            String::new()
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    board.drain().await.unwrap();
}

#[tokio::test]
async fn invalid_callbacks_are_retired_without_http_or_session_authentication() {
    for suffix in [
        "state=wrong&code=private-code",
        "state={state}&error=access_denied",
        "state={state}&iss=https%3A%2F%2Fother.example%2F&code=private-code",
        "state={state}",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let board = crate::Board::open_with_oidc(
            &dir.path().join("board.sqlite3"),
            "127.0.0.1:3014".parse().unwrap(),
            fixture(),
            Vec::new(),
            None,
        )
        .await
        .unwrap();
        let (_, h, html) = call(
            &board.router,
            Method::GET,
            "/board/login",
            "",
            String::new(),
        )
        .await;
        let csrf = html
            .split("name=\"csrf\" value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let (_, h, _) = call(
            &board.router,
            Method::POST,
            "/board/oidc/login",
            &cookie(&h),
            format!("csrf={csrf}"),
        )
        .await;
        let url = reqwest::Url::parse(h[header::LOCATION].to_str().unwrap()).unwrap();
        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .unwrap()
            .1
            .into_owned();
        let cookie = cookie(&h);
        let path = format!("/board/oidc/callback?{}", suffix.replace("{state}", &state));
        let (status, h, html) =
            call(&board.router, Method::GET, &path, &cookie, String::new()).await;
        assert!(matches!(
            status,
            StatusCode::UNAUTHORIZED | StatusCode::BAD_REQUEST
        ));
        assert_eq!(h[header::REFERRER_POLICY], "no-referrer");
        assert_eq!(h[header::CONTENT_SECURITY_POLICY], crate::DEFAULT_CSP);
        assert_eq!(h[header::CACHE_CONTROL], "no-store");
        assert!(!html.contains("private-code"));
        assert_eq!(
            call(
                &board.router,
                Method::GET,
                &format!("/board/oidc/callback?state={state}&code=unused"),
                &cookie,
                String::new()
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(
                &board.router,
                Method::GET,
                "/board/projects/1/items",
                &cookie,
                String::new()
            )
            .await
            .0,
            StatusCode::UNAUTHORIZED
        );
        board.drain().await.unwrap();
    }
}

// Public verification vectors adapted from openidconnect 4.0.1 (MIT),
// src/jwt/tests.rs and src/core/jwk/tests.rs. See vendor/openidconnect/LICENSE.
// The upstream primitive vector treats this signing input as opaque bytes;
// this does not claim that its RS256 header is a valid ES256/ES384 ID Token.
const EC_SIGNING_INPUT: &str = concat!(
    "eyJhbGciOiJSUzI1NiIsImtpZCI6ImJpbGJvLmJhZ2dpbnNAaG9iYml0b24uZX",
    "hhbXBsZSJ9.",
    "SXTigJlzIGEgZGFuZ2Vyb3VzIGJ1c2luZXNzLCBGcm9kbywgZ29pbmcgb3V0IH",
    "lvdXIgZG9vci4gWW91IHN0ZXAgb250byB0aGUgcm9hZCwgYW5kIGlmIHlvdSBk",
    "b24ndCBrZWVwIHlvdXIgZmVldCwgdGhlcmXigJlzIG5vIGtub3dpbmcgd2hlcm",
    "UgeW91IG1pZ2h0IGJlIHN3ZXB0IG9mZiB0by4",
);

fn ec_verification_vectors() -> [(
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreJwsSigningAlgorithm,
    Vec<u8>,
); 2] {
    use openidconnect::core::CoreJwsSigningAlgorithm;
    [
        (
            serde_json::from_str(
                r#"{
        "kty": "EC",
        "kid": "bilbo.baggins@hobbiton.example",
        "use": "sig",
        "crv": "P-256",
        "x": "t6PHivOTggpaX9lkMkis2p8kMhy-CktJAFTz6atReZw",
        "y": "ODobXupKlD0DeM1yRd7bX4XFNBO1HOgCT1UCu0KY3lc"
    }"#,
            )
            .unwrap(),
            CoreJwsSigningAlgorithm::EcdsaP256Sha256,
            vec![
                0x12, 0x72, 0x82, 0xb4, 0x01, 0xe1, 0xce, 0x1a, 0xb1, 0x57, 0x61, 0x93, 0xaf, 0x55,
                0x44, 0xba, 0xbb, 0x1e, 0xda, 0x44, 0x3b, 0xa0, 0x7a, 0x45, 0xa1, 0x5a, 0x8c, 0xeb,
                0xab, 0x18, 0x1a, 0x59, 0x9a, 0x84, 0x34, 0x46, 0x4a, 0x57, 0xeb, 0x54, 0x0b, 0x06,
                0x0b, 0x37, 0x4b, 0xbf, 0xae, 0x8e, 0x4b, 0x64, 0x1f, 0xd7, 0x3b, 0x7a, 0x38, 0xf1,
                0xd7, 0xfb, 0x96, 0x40, 0x90, 0xc8, 0x33, 0x94,
            ],
        ),
        (
            serde_json::from_str(
                r#"{
        "kty": "EC",
        "kid": "bilbo.baggins@hobbiton.example",
        "use": "sig",
        "crv" : "P-384",
        "x": "9ywsUbxX59kJXFRiWHcx97wRKNiF8Hc9F5wI08n8h2ek_qAl0veEc36k1Qz6KLiL",
        "y": "6PWlqjRbaV7V8ohDscM243IneuLZmxDGLiGNA1w69fQhEDsvZtKLUQ5KiHLgR3op"
    }"#,
            )
            .unwrap(),
            CoreJwsSigningAlgorithm::EcdsaP384Sha384,
            vec![
                0x07, 0xff, 0x68, 0x0c, 0x06, 0x9b, 0x31, 0xab, 0x19, 0xd9, 0x8b, 0x7f, 0x72, 0x70,
                0x12, 0xdb, 0x5a, 0x30, 0x68, 0xdd, 0x2e, 0x59, 0x22, 0x27, 0x40, 0x13, 0xf1, 0x4e,
                0xaa, 0xa2, 0x33, 0x73, 0x77, 0x5e, 0x39, 0x24, 0xb0, 0x14, 0x4c, 0x1a, 0xa8, 0x2e,
                0x24, 0xb5, 0x79, 0x5a, 0x84, 0x8c, 0x72, 0xfa, 0xa0, 0x5f, 0x01, 0xd3, 0x4d, 0x0b,
                0x5b, 0x1c, 0x6b, 0xa1, 0xd1, 0x47, 0xf8, 0x83, 0xa2, 0xc4, 0xaf, 0xb5, 0xed, 0x3b,
                0x54, 0xf6, 0x28, 0x2b, 0x6d, 0x41, 0x1d, 0xbb, 0x77, 0x96, 0x1b, 0xbd, 0x56, 0x5d,
                0x39, 0x1a, 0xf3, 0x0a, 0x6d, 0x0d, 0xb2, 0x6d, 0x73, 0x91, 0x5f, 0xa2,
            ],
        ),
    ]
}

#[test]
fn ec_dependency_accepts_known_public_signatures() {
    use openidconnect::JsonWebKey;
    for (key, algorithm, signature) in ec_verification_vectors() {
        key.verify_signature(&algorithm, EC_SIGNING_INPUT.as_bytes(), &signature)
            .unwrap();
    }
}

#[test]
fn ec_dependency_rejects_changed_messages_signatures_and_lengths() {
    use openidconnect::{JsonWebKey, SignatureVerificationError};
    for (key, algorithm, signature) in ec_verification_vectors() {
        let mut message = EC_SIGNING_INPUT.as_bytes().to_vec();
        message[0] ^= 1;
        assert!(matches!(
            key.verify_signature(&algorithm, &message, &signature),
            Err(SignatureVerificationError::CryptoError(_))
        ));
        let mut changed = signature.clone();
        changed[0] ^= 1;
        assert!(matches!(
            key.verify_signature(&algorithm, EC_SIGNING_INPUT.as_bytes(), &changed),
            Err(SignatureVerificationError::CryptoError(_))
        ));
        assert!(matches!(
            key.verify_signature(
                &algorithm,
                EC_SIGNING_INPUT.as_bytes(),
                &signature[..signature.len() - 1]
            ),
            Err(SignatureVerificationError::CryptoError(_))
        ));
        let mut extended = signature;
        extended.push(0);
        assert!(matches!(
            key.verify_signature(&algorithm, EC_SIGNING_INPUT.as_bytes(), &extended),
            Err(SignatureVerificationError::CryptoError(_))
        ));
    }
}

#[test]
fn ec_dependency_rejects_curve_mismatch_and_invalid_public_points() {
    use openidconnect::{
        JsonWebKey, SignatureVerificationError,
        core::{CoreJsonWebKey, CoreJwsSigningAlgorithm},
    };
    for (key, algorithm, signature) in ec_verification_vectors() {
        let (wrong_algorithm, coordinate_len) = match algorithm {
            CoreJwsSigningAlgorithm::EcdsaP256Sha256 => {
                (CoreJwsSigningAlgorithm::EcdsaP384Sha384, 43)
            }
            CoreJwsSigningAlgorithm::EcdsaP384Sha384 => {
                (CoreJwsSigningAlgorithm::EcdsaP256Sha256, 64)
            }
            _ => unreachable!(),
        };
        assert!(matches!(
            key.verify_signature(&wrong_algorithm, EC_SIGNING_INPUT.as_bytes(), &signature),
            Err(SignatureVerificationError::InvalidKey(_))
        ));
        let mut bad_point = serde_json::to_value(key).unwrap();
        bad_point["x"] = "A".repeat(coordinate_len).into();
        bad_point["y"] = "A".repeat(coordinate_len).into();
        let bad_key: CoreJsonWebKey = serde_json::from_value(bad_point).unwrap();
        assert!(matches!(
            bad_key.verify_signature(&algorithm, EC_SIGNING_INPUT.as_bytes(), &signature),
            Err(SignatureVerificationError::InvalidKey(_))
        ));
    }
}
