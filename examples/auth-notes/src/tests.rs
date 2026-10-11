use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, header},
};
use tower::ServiceExt;

const ALICE_CODE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BOB_CODE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

struct Observed {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

fn identities() -> DemoIdentities {
    DemoIdentities::new(ALICE_CODE.to_owned(), BOB_CODE.to_owned()).unwrap()
}

fn router() -> Router {
    app("127.0.0.1:0".parse().unwrap(), identities()).unwrap()
}

async fn send(router: Router, request: Request<Body>) -> Observed {
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
    Observed {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

fn get(path: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(Method::GET).uri(path);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).unwrap()
}

fn encode(value: &str) -> String {
    let mut encoded = String::new();
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    encoded
}

fn post(path: &str, cookie: &str, fields: &[(&str, &str)]) -> Request<Body> {
    let body = fields
        .iter()
        .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap()
}

fn post_raw(path: &str, cookie: &str, body: String) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap()
}

fn cookie_pair(headers: &HeaderMap) -> String {
    headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn csrf(html: &str) -> String {
    let marker = "name=\"csrf\" value=\"";
    let start = html.find(marker).unwrap() + marker.len();
    let rest = &html[start..];
    rest[..rest.find('"').unwrap()].to_owned()
}

async fn login_as(application: Router, user: &str, code: &str) -> (String, String, String) {
    let login = send(application.clone(), get("/auth/login", None)).await;
    assert_eq!(login.status, StatusCode::OK);
    let anonymous_cookie = cookie_pair(&login.headers);
    let anonymous_csrf = csrf(&login.body);
    let authenticated = send(
        application.clone(),
        post(
            "/auth/login",
            &anonymous_cookie,
            &[
                ("csrf", &anonymous_csrf),
                ("username", user),
                ("code", code),
            ],
        ),
    )
    .await;
    assert_eq!(authenticated.status, StatusCode::SEE_OTHER);
    assert_eq!(authenticated.headers[header::LOCATION], "/auth/notes");
    let authenticated_cookie = cookie_pair(&authenticated.headers);
    assert_ne!(anonymous_cookie, authenticated_cookie);
    let page = send(application, get("/auth/notes", Some(&authenticated_cookie))).await;
    assert_eq!(page.status, StatusCode::OK);
    (authenticated_cookie, csrf(&page.body), anonymous_cookie)
}

#[test]
fn identity_codes_are_validated_redacted_and_constant_shape() {
    assert!(matches!(
        DemoIdentities::new("a".repeat(63), BOB_CODE.to_owned()),
        Err(IdentityConfigError::InvalidCode)
    ));
    assert!(matches!(
        DemoIdentities::new("A".repeat(64), BOB_CODE.to_owned()),
        Err(IdentityConfigError::InvalidCode)
    ));
    assert!(matches!(
        DemoIdentities::new(ALICE_CODE.to_owned(), ALICE_CODE.to_owned()),
        Err(IdentityConfigError::DuplicateCodes)
    ));
    let identities = identities();
    assert_eq!(
        format!("{identities:?}"),
        "DemoIdentities { synthetic_codes: [REDACTED] }"
    );
    assert_eq!(
        identities.authenticate("alice", ALICE_CODE),
        Some(UserId::Alice)
    );
    assert_eq!(identities.authenticate("bob", BOB_CODE), Some(UserId::Bob));
    assert_eq!(identities.authenticate("unknown", ALICE_CODE), None);
    assert_eq!(identities.authenticate("alice", "short"), None);
}

#[test]
fn local_http_exception_is_loopback_only_and_secure_is_the_provider_default() {
    let config = SessionConfig::new("secure-default", Duration::from_secs(60), 4).unwrap();
    assert!(config.is_secure());
    assert!(app("127.0.0.1:3010".parse().unwrap(), identities()).is_ok());
    assert!(app("[::1]:3010".parse().unwrap(), identities()).is_ok());
    assert!(matches!(
        app("0.0.0.0:3010".parse().unwrap(), identities()),
        Err(AuthAppError::NonLoopbackListener)
    ));
    assert!(app_for_isolated_container_lab("0.0.0.0:3010".parse().unwrap(), identities()).is_ok());
    assert!(app_for_isolated_container_lab("[::]:3010".parse().unwrap(), identities()).is_ok());
    assert!(matches!(
        app_for_isolated_container_lab("192.0.2.10:3010".parse().unwrap(), identities()),
        Err(AuthAppError::NonIsolatedContainerListener)
    ));
}

#[tokio::test]
async fn health_is_outside_session_and_login_page_leaks_no_codes() {
    let application = router();
    let health = send(application.clone(), get("/healthz", None)).await;
    assert_eq!(health.status, StatusCode::OK);
    assert!(!health.headers.contains_key(header::SET_COOKIE));

    let login = send(application, get("/auth/login", None)).await;
    assert_eq!(login.status, StatusCode::OK);
    assert!(login.body.contains("Yetlet Synthetic Auth Notes"));
    assert!(login.body.contains("<main lang=\"en\">"));
    assert!(!login.body.contains(ALICE_CODE));
    assert!(!login.body.contains(BOB_CODE));
    assert!(!login.body.contains("<script"));
    let set_cookie = login.headers[header::SET_COOKIE].to_str().unwrap();
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Path=/"));
    assert!(!set_cookie.contains("Secure"));
    assert_eq!(login.headers[header::CACHE_CONTROL], "no-store");
}

#[tokio::test]
async fn invalid_credentials_and_csrf_do_not_authenticate_or_rotate() {
    let application = router();
    let login = send(application.clone(), get("/auth/login", None)).await;
    let cookie = cookie_pair(&login.headers);
    let token = csrf(&login.body);

    let bad_csrf = send(
        application.clone(),
        post(
            "/auth/login",
            &cookie,
            &[("csrf", "bad"), ("username", "alice"), ("code", ALICE_CODE)],
        ),
    )
    .await;
    assert_eq!(bad_csrf.status, StatusCode::FORBIDDEN);
    assert!(!bad_csrf.headers.contains_key(header::SET_COOKIE));

    let bad_code = send(
        application.clone(),
        post(
            "/auth/login",
            &cookie,
            &[("csrf", &token), ("username", "alice"), ("code", BOB_CODE)],
        ),
    )
    .await;
    assert_eq!(bad_code.status, StatusCode::UNAUTHORIZED);
    assert!(
        bad_code
            .body
            .contains("Invalid synthetic identity or login code")
    );
    assert!(!bad_code.body.contains(BOB_CODE));
    assert!(!bad_code.headers.contains_key(header::SET_COOKIE));

    let notes = send(application, get("/auth/notes", Some(&cookie))).await;
    assert_eq!(notes.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn successful_login_rotates_identity_and_old_cookie_is_anonymous() {
    let application = router();
    let (cookie, token, old_cookie) = login_as(application.clone(), "alice", ALICE_CODE).await;
    let page = send(application.clone(), get("/auth/notes", Some(&cookie))).await;
    assert!(page.body.contains("Alice Example&#39;s private notes"));
    assert_eq!(csrf(&page.body), token);

    let old = send(application, get("/auth/notes", Some(&old_cookie))).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    assert_ne!(cookie_pair(&old.headers), old_cookie);
}

#[tokio::test]
async fn login_rotation_retires_the_anonymous_csrf_token() {
    let application = router();
    let login = send(application.clone(), get("/auth/login", None)).await;
    let old_cookie = cookie_pair(&login.headers);
    let old_csrf = csrf(&login.body);
    let authenticated = send(
        application.clone(),
        post(
            "/auth/login",
            &old_cookie,
            &[
                ("csrf", &old_csrf),
                ("username", "alice"),
                ("code", ALICE_CODE),
            ],
        ),
    )
    .await;
    let new_cookie = cookie_pair(&authenticated.headers);
    let page = send(application.clone(), get("/auth/notes", Some(&new_cookie))).await;
    let new_csrf = csrf(&page.body);
    assert_ne!(old_csrf, new_csrf);
    let rejected = send(
        application.clone(),
        post(
            "/auth/notes",
            &new_cookie,
            &[
                ("csrf", &old_csrf),
                ("title", "blocked"),
                ("body", "blocked"),
            ],
        ),
    )
    .await;
    assert_eq!(rejected.status, StatusCode::FORBIDDEN);
    let unchanged = send(application, get("/auth/notes", Some(&new_cookie))).await;
    assert!(unchanged.body.contains("No notes yet."));
}

#[tokio::test]
async fn per_user_notes_escape_content_and_prevent_idor_reads_and_writes() {
    let application = router();
    let (alice_cookie, alice_csrf, _) = login_as(application.clone(), "alice", ALICE_CODE).await;
    let created = send(
        application.clone(),
        post(
            "/auth/notes",
            &alice_cookie,
            &[
                ("csrf", &alice_csrf),
                ("title", "<script>alert(1)</script>"),
                ("body", "private & safe"),
            ],
        ),
    )
    .await;
    assert_eq!(created.status, StatusCode::SEE_OTHER);
    assert_eq!(created.headers[header::LOCATION], "/auth/notes/1");
    let alice_note = send(
        application.clone(),
        get("/auth/notes/1", Some(&alice_cookie)),
    )
    .await;
    assert_eq!(alice_note.status, StatusCode::OK);
    assert!(
        alice_note
            .body
            .contains("&lt;script&gt;alert(1)&lt;/script&gt;")
    );
    assert!(!alice_note.body.contains("<script>alert"));

    let (bob_cookie, bob_csrf, _) = login_as(application.clone(), "bob", BOB_CODE).await;
    let bob_list = send(application.clone(), get("/auth/notes", Some(&bob_cookie))).await;
    assert_eq!(bob_list.status, StatusCode::OK);
    assert!(!bob_list.body.contains("data-note-id=\"1\""));
    assert_eq!(
        send(application.clone(), get("/auth/notes/1", Some(&bob_cookie)))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let forbidden_update = send(
        application.clone(),
        post(
            "/auth/notes/1/update",
            &bob_cookie,
            &[("csrf", &bob_csrf), ("title", "stolen"), ("body", "stolen")],
        ),
    )
    .await;
    assert_eq!(forbidden_update.status, StatusCode::FORBIDDEN);
    let forbidden_delete = send(
        application.clone(),
        post("/auth/notes/1/delete", &bob_cookie, &[("csrf", &bob_csrf)]),
    )
    .await;
    assert_eq!(forbidden_delete.status, StatusCode::FORBIDDEN);
    let unchanged = send(application, get("/auth/notes/1", Some(&alice_cookie))).await;
    assert_eq!(unchanged.status, StatusCode::OK);
    assert!(unchanged.body.contains("private &amp; safe"));
}

#[tokio::test]
async fn every_mutation_requires_csrf_and_failed_attempts_leave_state_unchanged() {
    let application = router();
    let (cookie, token, _) = login_as(application.clone(), "alice", ALICE_CODE).await;
    let rejected = send(
        application.clone(),
        post(
            "/auth/notes",
            &cookie,
            &[("csrf", "wrong"), ("title", "blocked"), ("body", "blocked")],
        ),
    )
    .await;
    assert_eq!(rejected.status, StatusCode::FORBIDDEN);
    let page = send(application.clone(), get("/auth/notes", Some(&cookie))).await;
    assert!(page.body.contains("No notes yet."));

    let created = send(
        application.clone(),
        post(
            "/auth/notes",
            &cookie,
            &[("csrf", &token), ("title", "original"), ("body", "body")],
        ),
    )
    .await;
    assert_eq!(created.status, StatusCode::SEE_OTHER);
    let rejected = send(
        application.clone(),
        post(
            "/auth/notes/1/update",
            &cookie,
            &[("csrf", "wrong"), ("title", "changed"), ("body", "changed")],
        ),
    )
    .await;
    assert_eq!(rejected.status, StatusCode::FORBIDDEN);
    let rejected = send(
        application.clone(),
        post("/auth/notes/1/delete", &cookie, &[("csrf", "wrong")]),
    )
    .await;
    assert_eq!(rejected.status, StatusCode::FORBIDDEN);
    let note = send(application.clone(), get("/auth/notes/1", Some(&cookie))).await;
    assert!(note.body.contains("original"));

    let logout = send(
        application.clone(),
        post("/auth/logout", &cookie, &[("csrf", "wrong")]),
    )
    .await;
    assert_eq!(logout.status, StatusCode::FORBIDDEN);
    assert_eq!(
        send(application, get("/auth/notes", Some(&cookie)))
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn logout_rotates_to_anonymous_and_old_authenticated_cookie_is_retired() {
    let application = router();
    let (cookie, token, _) = login_as(application.clone(), "alice", ALICE_CODE).await;
    let logout = send(
        application.clone(),
        post("/auth/logout", &cookie, &[("csrf", &token)]),
    )
    .await;
    assert_eq!(logout.status, StatusCode::SEE_OTHER);
    assert_eq!(logout.headers[header::LOCATION], "/auth/login");
    let anonymous_cookie = cookie_pair(&logout.headers);
    assert_ne!(cookie, anonymous_cookie);
    assert_eq!(
        send(
            application.clone(),
            get("/auth/notes", Some(&anonymous_cookie))
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
    let old = send(application, get("/auth/notes", Some(&cookie))).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    assert_ne!(cookie_pair(&old.headers), cookie);
}

#[tokio::test]
async fn unknown_session_cookie_is_never_adopted_and_routing_statuses_are_clear() {
    let application = router();
    let supplied = format!("yetlet-auth-notes={}", "c".repeat(64));
    let login = send(application.clone(), get("/auth/login", Some(&supplied))).await;
    assert_eq!(login.status, StatusCode::OK);
    assert_ne!(cookie_pair(&login.headers), supplied);
    assert_eq!(
        send(
            application.clone(),
            get("/auth/notes/999", Some(&cookie_pair(&login.headers)))
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(application.clone(), get("/missing", None))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let method = send(application, get("/auth/logout", None)).await;
    assert_eq!(method.status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(method.headers.contains_key(header::ALLOW));
}

#[tokio::test]
async fn malformed_duplicate_unknown_and_oversized_forms_fail_closed() {
    let application = router();
    let login = send(application.clone(), get("/auth/login", None)).await;
    let cookie = cookie_pair(&login.headers);
    let token = csrf(&login.body);

    let duplicate = send(
        application.clone(),
        post_raw(
            "/auth/login",
            &cookie,
            format!(
                "csrf={}&username=alice&username=bob&code={}",
                encode(&token),
                ALICE_CODE
            ),
        ),
    )
    .await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);
    let unknown = send(
        application.clone(),
        post_raw(
            "/auth/login",
            &cookie,
            format!(
                "csrf={}&username=alice&code={}&admin=true",
                encode(&token),
                ALICE_CODE
            ),
        ),
    )
    .await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    let oversized = send(
        application.clone(),
        post_raw(
            "/auth/login",
            &cookie,
            format!(
                "csrf={}&username=alice&code={}&padding={}",
                encode(&token),
                ALICE_CODE,
                "x".repeat(MAX_FORM_BODY)
            ),
        ),
    )
    .await;
    assert_eq!(oversized.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        send(application, get("/auth/notes", Some(&cookie)))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn per_user_capacity_is_bounded_without_partial_creation() {
    let mut store = NotesStore::default();
    for index in 0..MAX_NOTES_PER_USER {
        store
            .create(UserId::Alice, format!("note-{index}"), "body".to_owned())
            .unwrap();
    }
    let before = store.list(UserId::Alice);
    assert_eq!(
        store.create(UserId::Alice, "overflow".to_owned(), "body".to_owned()),
        Err(NoteError::Capacity)
    );
    assert_eq!(store.list(UserId::Alice), before);
}
