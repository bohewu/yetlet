use super::*;
use axum::{
    body::to_bytes,
    http::{Method, Request},
};
use futures_util::StreamExt;
use tower::ServiceExt;

const HOST: &str = "127.0.0.1:3012";

#[tokio::test]
async fn independent_boards_share_sqlite_records_conflicts_busy_errors_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shared.sqlite");
    let ids = DemoIdentities::generate().unwrap();
    let a = Board::open(&path, HOST.parse().unwrap(), ids.clone())
        .await
        .unwrap();
    let b = Board::open(&path, HOST.parse().unwrap(), ids.clone())
        .await
        .unwrap();
    let mut ca = Client::login(&a, &ids, "alice").await;
    let mut cb = Client::login(&b, &ids, "alice").await;
    let created = ca
        .call(
            &a.router,
            Method::POST,
            "/board/projects/1/items",
            Some(ca.form("Created through A", "open", 0, "")),
        )
        .await;
    assert_eq!(created.status, StatusCode::SEE_OTHER);
    let detail = created.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        cb.call(&b.router, Method::GET, &detail, None)
            .await
            .body
            .contains("Created through A")
    );
    assert_eq!(
        cb.call(
            &b.router,
            Method::POST,
            &detail,
            Some(cb.form("Updated through B", "active", 1, ""))
        )
        .await
        .status,
        StatusCode::SEE_OTHER
    );
    assert!(
        ca.call(&a.router, Method::GET, &detail, None)
            .await
            .body
            .contains("Updated through B")
    );
    let stale = ca
        .call(
            &a.router,
            Method::POST,
            &detail,
            Some(ca.form("Stale draft", "open", 1, "")),
        )
        .await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    assert!(stale.body.contains("Stale draft"));
    let fa = ca.form("Concurrent A", "active", 2, "");
    let fb = cb.form("Concurrent B", "active", 2, "");
    let (ra, rb) = tokio::join!(
        ca.call(&a.router, Method::POST, &detail, Some(fa)),
        cb.call(&b.router, Method::POST, &detail, Some(fb)),
    );
    let mut statuses = [ra.status.as_u16(), rb.status.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [303, 409]);
    let (item, history) = b.database.get(1, 1).await.unwrap();
    assert_eq!(item.version, 3);
    assert_eq!(history.len(), 3);

    // Hold a real external writer only to check the application's existing
    // busy/error mapping. This does not qualify SQLite capacity or durability.
    let blocker = rusqlite::Connection::open(&path).unwrap();
    blocker.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let busy = cb
        .call(
            &b.router,
            Method::POST,
            &detail,
            Some(cb.form("Not committed", "active", 3, "")),
        )
        .await;
    assert_eq!(busy.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(busy.body.contains("Storage is busy"));
    blocker.execute_batch("ROLLBACK;").unwrap();
    drop(blocker);
    assert_eq!(b.database.get(1, 1).await.unwrap().0.version, 3);
    assert_eq!(
        cb.call(
            &b.router,
            Method::POST,
            &detail,
            Some(cb.form("Recovered through B", "active", 3, ""))
        )
        .await
        .status,
        StatusCode::SEE_OTHER
    );

    a.drain().await.unwrap();
    drop(a);
    let restarted = Board::open(&path, HOST.parse().unwrap(), ids.clone())
        .await
        .unwrap();
    let mut renewed = Client::login(&restarted, &ids, "alice").await;
    assert!(
        renewed
            .call(&restarted.router, Method::GET, &detail, None)
            .await
            .body
            .contains("Recovered through B")
    );
    let (item, history) = restarted.database.get(1, 1).await.unwrap();
    assert_eq!(item.version, 4);
    assert_eq!(history.len(), 4);
    restarted.drain().await.unwrap();
    b.drain().await.unwrap();
}

#[tokio::test]
async fn readonly_and_unavailable_storage_fail_closed_and_recover() {
    let (_dir, board, ids) = setup().await;
    let mut client = Client::login(&board, &ids, "alice").await;
    board
        .database
        .save(alice(), None, 0, fields("Original"))
        .await
        .unwrap();
    board
        .database
        .run(|c| {
            c.pragma_update(None, "query_only", true)?;
            Ok(())
        })
        .await
        .unwrap();
    let path = "/board/projects/1/items/1";
    let denied = client
        .call(
            &board.router,
            Method::POST,
            path,
            Some(client.form("Not committed", "open", 1, "")),
        )
        .await;
    assert_eq!(denied.status, StatusCode::SERVICE_UNAVAILABLE);
    let (item, history) = board.database.get(1, 1).await.unwrap();
    assert_eq!(item.version, 1);
    assert_eq!(history.len(), 1);
    board
        .database
        .run(|c| {
            c.pragma_update(None, "query_only", false)?;
            c.execute_batch("ALTER TABLE work_items RENAME TO temporarily_unavailable;")?;
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(
        client
            .call(&board.router, Method::GET, "/readyz", None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        client
            .call(&board.router, Method::GET, "/healthz", None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        client
            .call(&board.router, Method::GET, path, None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    board
        .database
        .run(|c| {
            c.execute_batch("ALTER TABLE temporarily_unavailable RENAME TO work_items;")?;
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(
        client
            .call(&board.router, Method::GET, "/readyz", None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        client
            .call(
                &board.router,
                Method::POST,
                path,
                Some(client.form("Recovered", "open", 1, ""))
            )
            .await
            .status,
        StatusCode::SEE_OTHER
    );
    assert_eq!(board.database.get(1, 1).await.unwrap().1.len(), 2);
    board.drain().await.unwrap();
}

#[tokio::test]
async fn exact_item_and_history_caps_reject_without_partial_writes() {
    let (_dir, board, _ids) = setup().await;
    board.database.run(|c| {
        let tx = c.transaction()?;
        tx.execute_batch("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<10000) INSERT INTO work_items(id,project,title,description,status,priority,assignee) SELECT x,1,'Capacity fixture','','open','normal','' FROM n;")?;
        tx.execute_batch("INSERT INTO history(item_id,project,version,actor,action,title,status) SELECT id,1,1,'alice','create',title,status FROM work_items;")?;
        tx.commit()?;
        Ok(())
    }).await.unwrap();
    assert_eq!(
        board
            .database
            .save(alice(), None, 0, fields("Overflow"))
            .await
            .unwrap_err(),
        Failure::Capacity
    );
    assert_eq!(board.database.snapshot(1).await.unwrap().total, 10000);
    board.database.run(|c| {
        let tx = c.transaction()?;
        tx.execute_batch("WITH RECURSIVE v(x) AS (SELECT 2 UNION ALL SELECT x+1 FROM v WHERE x<10) INSERT INTO history(item_id,project,version,actor,action,title,status) SELECT w.id,1,v.x,'alice','update',w.title,w.status FROM work_items w CROSS JOIN v; UPDATE work_items SET version=10;")?;
        tx.commit()?;
        Ok(())
    }).await.unwrap();
    let before = board.database.snapshot(1).await.unwrap().revision;
    assert_eq!(before, 100000);
    assert_eq!(
        board
            .database
            .save(alice(), Some(1), 10, fields("Overflow"))
            .await
            .unwrap_err(),
        Failure::Capacity
    );
    assert_eq!(board.database.get(1, 1).await.unwrap().0.version, 10);
    assert_eq!(board.database.snapshot(1).await.unwrap().revision, before);
    board.drain().await.unwrap();
}
#[derive(Default, Clone)]
struct Client {
    cookie: Option<String>,
    csrf: String,
}
struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}
impl Client {
    async fn call(
        &mut self,
        router: &Router,
        method: Method,
        path: &str,
        form: Option<String>,
    ) -> Reply {
        let mut request = Request::builder()
            .method(method.clone())
            .uri(path)
            .header(header::HOST, HOST);
        if method == Method::POST {
            request = request
                .header(header::ORIGIN, format!("http://{HOST}"))
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        }
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie);
        }
        let response = router
            .clone()
            .oneshot(request.body(Body::from(form.unwrap_or_default())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        if let Some(cookie) = headers.get(header::SET_COOKIE) {
            self.cookie = Some(
                cookie
                    .to_str()
                    .unwrap()
                    .split(';')
                    .next()
                    .unwrap()
                    .to_owned(),
            );
        }
        let body = String::from_utf8(
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        if let Some(value) = body.split("name=\"csrf\" value=\"").nth(1) {
            self.csrf = value.split('"').next().unwrap().to_owned();
        }
        assert_eq!(headers[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
        assert_eq!(headers[header::REFERRER_POLICY], "same-origin");
        Reply {
            status,
            headers,
            body,
        }
    }
    async fn login(board: &Board, identities: &DemoIdentities, name: &str) -> Self {
        let mut client = Self::default();
        let initial = client
            .call(&board.router, Method::GET, "/board/login", None)
            .await;
        assert_eq!(initial.status, StatusCode::OK);
        let before = client.cookie.clone();
        let code = identities
            .terminal_codes()
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        let form = serde_urlencoded::to_string([
            ("username", name),
            ("code", code),
            ("csrf", &client.csrf),
        ])
        .unwrap();
        let reply = client
            .call(&board.router, Method::POST, "/board/login", Some(form))
            .await;
        assert_eq!(reply.status, StatusCode::SEE_OTHER);
        assert!(client.cookie != before);
        let path = reply.headers[header::LOCATION].to_str().unwrap().to_owned();
        assert_eq!(
            client
                .call(&board.router, Method::GET, &path, None)
                .await
                .status,
            StatusCode::OK
        );
        client
    }
    fn form(&self, title: &str, status: &str, version: i64, description: &str) -> String {
        serde_urlencoded::to_string([
            ("csrf", self.csrf.clone()),
            ("expected_version", version.to_string()),
            ("title", title.to_owned()),
            ("description", description.to_owned()),
            ("status", status.to_owned()),
            ("priority", "normal".into()),
            ("assignee", String::new()),
        ])
        .unwrap()
    }
}
async fn setup() -> (tempfile::TempDir, Board, DemoIdentities) {
    let dir = tempfile::tempdir().unwrap();
    let identities = DemoIdentities::generate().unwrap();
    let board = Board::open(
        &dir.path().join("board.sqlite"),
        HOST.parse().unwrap(),
        identities.clone(),
    )
    .await
    .unwrap();
    (dir, board, identities)
}
fn alice() -> Principal {
    Principal {
        name: "alice",
        project: 1,
        write: true,
    }
}
fn fields(title: &str) -> Fields {
    Fields {
        title: title.into(),
        ..Fields::default()
    }
}

#[tokio::test]
async fn application_stylesheets_are_validated_before_storage_and_keep_ssr_security() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("themed.sqlite");
    let ids = DemoIdentities::generate().unwrap();
    for paths in [
        vec!["https://example.test/app.css".to_owned()],
        vec!["/static/assets/../app.css".to_owned()],
        vec!["/static/assets/app.css?x=1".to_owned()],
        vec!["/static/assets/app.css\" onload=\"x".to_owned()],
        vec!["/static/assets/app.css".to_owned(); 2],
        (0..9)
            .map(|n| format!("/static/assets/app-{n}.css"))
            .collect(),
    ] {
        assert!(matches!(
            Board::open_with_stylesheets(&path, HOST.parse().unwrap(), ids.clone(), paths).await,
            Err(Failure::Invalid)
        ));
        assert!(!path.exists());
    }
    let board = Board::open_with_stylesheets(
        &path,
        HOST.parse().unwrap(),
        ids.clone(),
        vec!["/static/assets/app-Test_123.css".to_owned()],
    )
    .await
    .unwrap();
    let mut client = Client::login(&board, &ids, "alice").await;
    let page = client
        .call(&board.router, Method::GET, "/board/projects/1/items", None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.body
            .contains("href=\"/static/assets/app-Test_123.css\"")
    );
    assert!(!page.body.contains("bootstrap"));
    assert!(!page.body.contains("<script"));
    assert_eq!(page.headers[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
    assert_eq!(page.headers[header::CACHE_CONTROL], "no-store");
    let denied = client
        .call(&board.router, Method::GET, "/board/projects/2/items", None)
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert!(
        denied
            .body
            .contains("href=\"/static/assets/app-Test_123.css\"")
    );
    assert!(denied.body.contains("Return to the board"));
    assert!(denied.body.contains("role=\"alert\""));
    assert!(!denied.body.contains("<script"));
    board.drain().await.unwrap();
}

#[tokio::test]
async fn native_pages_assets_host_and_private_cache_are_explicit() {
    let (_dir, board, ids) = setup().await;
    let mut guest = Client::default();
    let health = guest
        .call(&board.router, Method::GET, "/healthz", None)
        .await;
    assert_eq!(health.status, StatusCode::OK);
    assert!(!health.headers.contains_key(header::SET_COOKIE));
    assert_eq!(
        guest
            .call(&board.router, Method::GET, "/board", None)
            .await
            .status,
        StatusCode::SEE_OTHER
    );
    let mut client = Client::login(&board, &ids, "alice").await;
    let reply = client
        .call(&board.router, Method::GET, "/board/projects/1/items", None)
        .await;
    assert!(!reply.body.contains("<script"));
    assert!(reply.body.contains("No work items match"));
    assert_eq!(reply.headers[header::CACHE_CONTROL], "no-store");
    assert!(
        reply.headers[header::VARY]
            .to_str()
            .unwrap()
            .contains("Cookie")
    );
    for asset in asset_data::ASSETS {
        let reply = guest
            .call(
                &board.router,
                Method::GET,
                &format!("/board-assets/{}", asset.name),
                None,
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.headers[header::CONTENT_TYPE], asset.content_type);
        assert!(
            reply.headers[header::CACHE_CONTROL]
                .to_str()
                .unwrap()
                .contains("immutable")
        );
    }
    let response = board
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/board/login")
                .header(header::HOST, "attacker.test:3012")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    board.drain().await.unwrap();
}
#[tokio::test]
async fn roles_projects_csrf_and_logout_are_enforced_server_side() {
    let (_dir, board, ids) = setup().await;
    let mut operator = Client::login(&board, &ids, "alice").await;
    let mut viewer = Client::login(&board, &ids, "viewer").await;
    let mut bob = Client::login(&board, &ids, "bob").await;
    let reply = operator
        .call(
            &board.router,
            Method::POST,
            "/board/projects/1/items",
            Some(operator.form("First", "open", 0, "plain")),
        )
        .await;
    let detail = reply.headers[header::LOCATION].to_str().unwrap().to_owned();
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    assert_eq!(
        viewer
            .call(&board.router, Method::GET, &detail, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        viewer
            .call(
                &board.router,
                Method::POST,
                &detail,
                Some(viewer.form("Denied", "open", 1, ""))
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        bob.call(&board.router, Method::GET, &detail, None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        bob.call(
            &board.router,
            Method::POST,
            &detail,
            Some(bob.form("Denied", "open", 1, ""))
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        operator
            .call(&board.router, Method::GET, "/board/projects/2/events", None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let mut invalid = operator.clone();
    invalid.csrf = "invalid".into();
    assert_eq!(
        invalid
            .call(
                &board.router,
                Method::POST,
                &detail,
                Some(invalid.form("Denied", "open", 1, ""))
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let old = operator.clone();
    let form = serde_urlencoded::to_string([("csrf", operator.csrf.clone())]).unwrap();
    assert_eq!(
        operator
            .call(&board.router, Method::POST, "/board/logout", Some(form))
            .await
            .status,
        StatusCode::SEE_OTHER
    );
    let mut retired = old;
    assert_eq!(
        retired
            .call(&board.router, Method::GET, &detail, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    board.drain().await.unwrap();
}
#[tokio::test]
async fn persisted_text_history_and_conflict_keep_safe_drafts() {
    let (_dir, board, ids) = setup().await;
    let mut client = Client::login(&board, &ids, "alice").await;
    let title = "<img src=x onerror=alert(1)> & quotes";
    let notes = "\n</textarea><script>unsafe()</script>";
    let created = client
        .call(
            &board.router,
            Method::POST,
            "/board/projects/1/items",
            Some(client.form(title, "open", 0, notes)),
        )
        .await;
    assert_eq!(created.status, StatusCode::SEE_OTHER);
    let path = created.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let read = client.call(&board.router, Method::GET, &path, None).await;
    assert!(read.body.contains("&lt;img"));
    assert!(read.body.contains("&lt;script&gt;"));
    assert!(!read.body.contains("<script"));
    let edit = client
        .call(&board.router, Method::GET, &format!("{path}/edit"), None)
        .await;
    assert!(edit.body.contains("\n\n&lt;/textarea&gt;"));
    assert_eq!(
        client
            .call(
                &board.router,
                Method::POST,
                &path,
                Some(client.form("Changed", "active", 1, "saved"))
            )
            .await
            .status,
        StatusCode::SEE_OTHER
    );
    let conflict = client
        .call(
            &board.router,
            Method::POST,
            &path,
            Some(client.form("Retained draft", "closed", 1, "not committed")),
        )
        .await;
    assert_eq!(conflict.status, StatusCode::CONFLICT);
    assert!(conflict.body.contains("Retained draft"));
    assert!(conflict.body.contains("not committed"));
    let (item, history) = board.database.get(1, 1).await.unwrap();
    assert_eq!(item.version, 2);
    assert_eq!(item.fields.title, "Changed");
    assert_eq!(history.len(), 2);
    board.drain().await.unwrap();
}
#[tokio::test]
async fn fields_unknown_duplicate_origin_and_body_bounds_fail_without_writes() {
    let (_dir, board, ids) = setup().await;
    let mut client = Client::login(&board, &ids, "alice").await;
    let path = "/board/projects/1/items";
    assert_eq!(
        client
            .call(
                &board.router,
                Method::POST,
                path,
                Some(client.form(" ", "open", 0, ""))
            )
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for tail in ["&role=admin", "&title=duplicate"] {
        assert_eq!(
            client
                .call(
                    &board.router,
                    Method::POST,
                    path,
                    Some(client.form("Valid", "open", 0, "") + tail)
                )
                .await
                .status,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        client
            .call(&board.router, Method::POST, path, Some("x".repeat(70_000)))
            .await
            .status,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    for origin in ["https://other.test", "null"] {
        let request = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::HOST, HOST)
            .header(header::ORIGIN, origin)
            .header(header::COOKIE, client.cookie.as_ref().unwrap())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(client.form("Denied", "open", 0, "")))
            .unwrap();
        assert_eq!(
            board
                .router
                .clone()
                .oneshot(request)
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(board.database.snapshot(1).await.unwrap().total, 0);
    assert!(
        Fields {
            title: "測".repeat(160),
            ..Fields::default()
        }
        .errors(1)
        .is_empty()
    );
    assert!(
        !Fields {
            title: "測".repeat(161),
            ..Fields::default()
        }
        .errors(1)
        .is_empty()
    );
    assert!(
        !Fields {
            title: "Safe".into(),
            assignee: "bob".into(),
            ..Fields::default()
        }
        .errors(1)
        .is_empty()
    );
    board.drain().await.unwrap();
}
#[tokio::test]
async fn pagination_filter_sort_and_project_snapshot_use_real_records() {
    let (_dir, board, ids) = setup().await;
    for i in 0..51 {
        board
            .database
            .save(alice(), None, 0, fields(&format!("Record {i:03}")))
            .await
            .unwrap();
    }
    let mut client = Client::login(&board, &ids, "alice").await;
    let page = client
        .call(
            &board.router,
            Method::GET,
            "/board/projects/1/items?sort=title&page=2",
            None,
        )
        .await;
    assert!(page.body.contains("Record 050"));
    assert!(!page.body.contains("Record 000"));
    assert!(page.body.contains("Previous"));
    let page = client
        .call(
            &board.router,
            Method::GET,
            "/board/projects/1/items?q=050",
            None,
        )
        .await;
    assert!(page.body.contains("1 matching items"));
    assert_eq!(
        client
            .call(
                &board.router,
                Method::GET,
                "/board/projects/1/items?sort=DROP",
                None
            )
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .call(
                &board.router,
                Method::GET,
                "/board/projects/1/items?page=201",
                None
            )
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(board.database.snapshot(2).await.unwrap().total, 0);
    board.drain().await.unwrap();
}
#[tokio::test]
async fn history_failure_rolls_back_record_and_revision_atomically() {
    let (_dir, board, _ids) = setup().await;
    board
        .database
        .save(alice(), None, 0, fields("Before"))
        .await
        .unwrap();
    board.database.run(|c|{c.execute_batch("CREATE TRIGGER fail_history BEFORE INSERT ON history BEGIN SELECT RAISE(ABORT,'test'); END;")?;Ok(())}).await.unwrap();
    assert_eq!(
        board
            .database
            .save(alice(), Some(1), 1, fields("Must rollback"))
            .await
            .unwrap_err(),
        Failure::Unavailable
    );
    let (item, history) = board.database.get(1, 1).await.unwrap();
    assert_eq!(item.fields.title, "Before");
    assert_eq!(item.version, 1);
    assert_eq!(history.len(), 1);
    assert_eq!(board.database.snapshot(1).await.unwrap().revision, 1);
    board.drain().await.unwrap();
}
#[tokio::test]
async fn competing_updates_have_one_winner_and_one_history_row() {
    let (_dir, board, _ids) = setup().await;
    board
        .database
        .save(alice(), None, 0, fields("Initial"))
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        board.database.save(alice(), Some(1), 1, fields("A")),
        board.database.save(alice(), Some(1), 1, fields("B"))
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert!(matches!(a, Err(Failure::Conflict)) || matches!(b, Err(Failure::Conflict)));
    assert_eq!(board.database.get(1, 1).await.unwrap().1.len(), 2);
    board.drain().await.unwrap();
}
#[tokio::test]
async fn closed_items_need_reopen_and_unknown_objects_are_not_updated() {
    let (_dir, board, _ids) = setup().await;
    let mut first = fields("Closed");
    first.status = "closed".into();
    board.database.save(alice(), None, 0, first).await.unwrap();
    let mut changed = fields("Active");
    changed.status = "active".into();
    assert_eq!(
        board
            .database
            .save(alice(), Some(1), 1, changed)
            .await
            .unwrap_err(),
        Failure::Transition
    );
    board
        .database
        .save(alice(), Some(1), 1, fields("Reopened"))
        .await
        .unwrap();
    assert_eq!(
        board
            .database
            .save(alice(), Some(999), 1, fields("Missing"))
            .await
            .unwrap_err(),
        Failure::NotFound
    );
    board.drain().await.unwrap();
}
#[tokio::test]
async fn restart_preserves_business_data_but_retires_memory_login() {
    let (dir, board, ids) = setup().await;
    let mut client = Client::login(&board, &ids, "alice").await;
    client
        .call(
            &board.router,
            Method::POST,
            "/board/projects/1/items",
            Some(client.form("Durable", "open", 0, "")),
        )
        .await;
    board.drain().await.unwrap();
    drop(board);
    let replacement = Board::open(
        &dir.path().join("board.sqlite"),
        HOST.parse().unwrap(),
        ids.clone(),
    )
    .await
    .unwrap();
    assert_eq!(
        client
            .call(
                &replacement.router,
                Method::GET,
                "/board/projects/1/items/1",
                None
            )
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let mut fresh = Client::login(&replacement, &ids, "alice").await;
    assert!(
        fresh
            .call(
                &replacement.router,
                Method::GET,
                "/board/projects/1/items/1",
                None
            )
            .await
            .body
            .contains("Durable")
    );
    replacement.drain().await.unwrap();
}
#[tokio::test]
async fn future_and_malformed_schema_and_nonloopback_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let future = dir.path().join("future.sqlite");
    let c = rusqlite::Connection::open(&future).unwrap();
    c.pragma_update(None, "user_version", 2).unwrap();
    drop(c);
    assert!(matches!(
        Database::open(&future).await,
        Err(Failure::Schema)
    ));
    let bad = dir.path().join("bad.sqlite");
    let c = rusqlite::Connection::open(&bad).unwrap();
    c.execute_batch("CREATE TABLE work_items(id INTEGER); PRAGMA user_version=1;")
        .unwrap();
    drop(c);
    assert!(matches!(Database::open(&bad).await, Err(Failure::Schema)));
    assert!(matches!(
        Board::open(
            &dir.path().join("outside.sqlite"),
            "0.0.0.0:3012".parse().unwrap(),
            DemoIdentities::generate().unwrap()
        )
        .await,
        Err(Failure::Invalid)
    ));
}
#[tokio::test]
async fn cancellation_keeps_worker_admission_until_drain_completes() {
    let (_dir, board, _ids) = setup().await;
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let database = board.database.clone();
    let waiting = tokio::spawn(async move {
        database
            .run(move |_| {
                let _ = started_tx.send(());
                release_rx
                    .recv_timeout(Duration::from_secs(2))
                    .map_err(|_| Failure::Busy)?;
                Ok(())
            })
            .await
    });
    started_rx.await.unwrap();
    waiting.abort();
    let _ = waiting.await;
    assert!(
        tokio::time::timeout(Duration::from_millis(40), board.database.drain())
            .await
            .is_err()
    );
    release_tx.send(()).unwrap();
    board.drain().await.unwrap();
    assert_eq!(
        board.database.ready().await.unwrap_err(),
        Failure::Unavailable
    );
}
#[tokio::test]
async fn stream_sends_authorized_snapshot_and_stops_after_logout() {
    let (_dir, board, ids) = setup().await;
    let mut client = Client::login(&board, &ids, "alice").await;
    let request = Request::builder()
        .uri("/board/projects/1/events")
        .header(header::HOST, HOST)
        .header(header::COOKIE, client.cookie.as_ref().unwrap())
        .header("last-event-id", "9000")
        .body(Body::empty())
        .unwrap();
    let response = board.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let mut data = response.into_body().into_data_stream();
    let initial = data.next().await.unwrap().unwrap();
    let initial = String::from_utf8(initial.to_vec()).unwrap();
    assert!(initial.contains("event: snapshot"));
    assert!(initial.contains("\"total\":0"));
    board
        .database
        .save(alice(), None, 0, fields("Real update"))
        .await
        .unwrap();
    let updated = tokio::time::timeout(Duration::from_secs(3), data.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(
        String::from_utf8(updated.to_vec())
            .unwrap()
            .contains("\"total\":1")
    );
    let form = serde_urlencoded::to_string([("csrf", client.csrf.clone())]).unwrap();
    client
        .call(&board.router, Method::POST, "/board/logout", Some(form))
        .await;
    assert!(
        tokio::time::timeout(Duration::from_secs(3), data.next())
            .await
            .unwrap()
            .is_none()
    );
    board.drain().await.unwrap();
}

async fn triage_json(
    board: &Board,
    client: &Client,
    project: i64,
    payload: serde_json::Value,
) -> (StatusCode, String) {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(format!("/board/projects/{project}/triage"))
        .header(header::HOST, HOST)
        .header(header::ORIGIN, format!("http://{HOST}"))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = &client.cookie {
        request = request.header(header::COOKIE, cookie);
    }
    let response = board
        .router
        .clone()
        .oneshot(
            request
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        DEFAULT_CSP
    );
    let body = String::from_utf8(
        to_bytes(response.into_body(), 65536)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    (status, body)
}

#[tokio::test]
async fn triage_json_reuses_authorization_csrf_validation_and_atomic_conflicts() {
    let (_dir, board, ids) = setup().await;
    let item = board
        .database
        .save(alice(), None, 0, fields("Original"))
        .await
        .unwrap();
    let mut client = Client::login(&board, &ids, "alice").await;
    let mut item = yetlet_board_triage::Item::from(item);
    item.title = "Saved from Rust".into();
    let payload = serde_json::json!({"csrf":client.csrf,"item":item});
    assert_eq!(
        triage_json(&board, &Client::default(), 1, payload.clone())
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let mut forged = payload.clone();
    forged["csrf"] = "invalid".into();
    assert_eq!(
        triage_json(&board, &client, 1, forged).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        triage_json(&board, &client, 2, payload.clone()).await.0,
        StatusCode::FORBIDDEN
    );
    let viewer = Client::login(&board, &ids, "viewer").await;
    assert_eq!(
        triage_json(&board, &viewer, 1, payload.clone()).await.0,
        StatusCode::FORBIDDEN
    );
    let mut invalid = payload.clone();
    invalid["item"]["title"] = "".into();
    assert_eq!(
        triage_json(&board, &client, 1, invalid).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut malformed = payload.clone();
    malformed["unexpected"] = true.into();
    assert_eq!(
        triage_json(&board, &client, 1, malformed).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(board.database.get(1, item.id).await.unwrap().1.len(), 1);
    let (status, body) = triage_json(&board, &client, 1, payload.clone()).await;
    assert_eq!(status, StatusCode::OK);
    let saved: yetlet_board_triage::Item = serde_json::from_str(&body).unwrap();
    assert_eq!(saved.version, 2);
    assert_eq!(saved.title, item.title);
    let (status, body) = triage_json(&board, &client, 1, payload).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        serde_json::from_str::<yetlet_actions::ApiErrorEnvelope>(&body)
            .unwrap()
            .error
            .code,
        "conflict"
    );
    assert_eq!(board.database.get(1, item.id).await.unwrap().1.len(), 2);
    let reply = client
        .call(&board.router, Method::GET, "/board/projects/1/triage", None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(!reply.body.contains("<script"));
    board.drain().await.unwrap();
}

#[tokio::test]
async fn triage_only_enables_wasm_on_selected_page_and_escapes_bounded_bootstrap() {
    let dir = tempfile::tempdir().unwrap();
    let ids = DemoIdentities::generate().unwrap();
    let board = Board::open_with_frontend(
        &dir.path().join("triage.sqlite3"),
        HOST.parse().unwrap(),
        ids.clone(),
        Vec::new(),
        Some("/static/assets/app-123.js".into()),
    )
    .await
    .unwrap();
    for n in 0..14 {
        board
            .database
            .save(alice(), None, 0, fields(&format!("<script> & \"{n}\"")))
            .await
            .unwrap();
    }
    let client = Client::login(&board, &ids, "alice").await;
    let response = board
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/board/projects/1/triage")
                .header(header::HOST, HOST)
                .header(header::COOKIE, client.cookie.clone().unwrap())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let csp = response.headers()[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap()
        .to_owned();
    let nonce = csp
        .split("'nonce-")
        .nth(1)
        .unwrap()
        .split('\'')
        .next()
        .unwrap();
    assert_eq!(nonce.len(), 32);
    assert!(nonce.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(
        csp,
        triage::CSP.replace(
            "style-src 'self'",
            &format!("style-src 'self' 'nonce-{nonce}'; style-src-attr 'none'")
        )
    );
    assert!(!triage::CSP.contains("'unsafe-eval'"));
    assert!(!triage::CSP.contains("'unsafe-inline'"));
    let html = String::from_utf8(
        to_bytes(response.into_body(), 512 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(html.matches("data-id=").count(), 12);
    assert!(html.contains("Loaded item status"));
    assert!(html.contains("id=\"triage-status-0\">12</dd>"));
    for index in 1..4 {
        assert!(html.contains(&format!("id=\"triage-status-{index}\">0</dd>")));
    }
    assert_eq!(html.matches("<script").count(), 1);
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("type=\"module\" src=\"/static/assets/app-123.js\""));
    assert!(html.contains(&format!("<meta property=\"csp-nonce\" nonce=\"{nonce}\">")));
    let second = board
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/board/projects/1/triage")
                .header(header::HOST, HOST)
                .header(header::COOKIE, client.cookie.unwrap())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(second.headers()[header::CONTENT_SECURITY_POLICY], csp);
    board.drain().await.unwrap();
    assert!(
        Board::open_with_frontend(
            &dir.path().join("must-not-exist.sqlite3"),
            HOST.parse().unwrap(),
            ids,
            Vec::new(),
            Some("/static/assets/\"bad.js".into())
        )
        .await
        .is_err()
    );
    assert!(!dir.path().join("must-not-exist.sqlite3").exists());
}
