use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use tower::ServiceExt;

fn fixture() -> Oidc {
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
        .set_redirect_uri(
            RedirectUrl::new("http://127.0.0.1:3014/board/oidc/callback".into()).unwrap(),
        ),
        http: reqwest::Client::new(),
        issuer: "https://issuer.example/".into(),
        subjects: BTreeMap::new(),
    }
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
        deadline: Instant::now() - Duration::from_secs(1),
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
