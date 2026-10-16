//! Scenario-for-scenario ports of scripts/smoke_{sample}.py.
use super::{
    Server,
    http::{self, Cookies, Response},
    require, wait,
};
use crate::tooling::{Result, fail};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, Read, Write},
    net::TcpStream,
    path::Path,
    time::Duration,
};
macro_rules! check {
    ($e:expr) => {
        require($e, concat!(file!(), ":", line!(), ": ", stringify!($e)))?
    };
}
fn status(r: &Response, code: u16) -> Result<()> {
    require(
        r.status == code,
        &format!("HTTP status expected {code}, got {}", r.status),
    )
}
fn error(r: Response, code: u16, name: &str) -> Result<()> {
    status(&r, code)?;
    require(r.json()?["error"]["code"] == name, "typed error code")
}

pub const SUITES: &[&str] = &[
    "hello-world",
    "rest-api",
    "cart",
    "middleware-lab",
    "forms-validation",
    "file-server",
    "sqlite-crud",
    "sse-notifications",
    "auth-notes",
    "modular-shop",
];
pub fn run(root: &Path, suite: &str, out: &Path) -> Result<()> {
    // Persistent application state is disposable, receipts and logs are retained.
    let state = tempfile::Builder::new()
        .prefix("yetlet-yt085-state-")
        .tempdir()?;
    let state_path = state.path().to_owned();
    let result = match suite {
        "sqlite-crud" => sqlite(root, out, &state_path),
        "modular-shop" => shop(root, out, &state_path),
        _ => {
            let (binary, listen, ready) = match suite {
                "hello-world" => ("yetlet-hello-world", "YETLET_HELLO_LISTEN", "/"),
                "rest-api" => ("yetlet-rest-api", "YETLET_REST_LISTEN", "/healthz"),
                "cart" => ("yetlet-cart-server", "YETLET_CART_LISTEN", "/healthz"),
                "middleware-lab" => ("yetlet-middleware-lab", "YETLET_MW_LISTEN", "/outside"),
                "forms-validation" => ("yetlet-forms-validation", "YETLET_FORMS_LISTEN", "/form"),
                "file-server" => ("yetlet-file-server", "YETLET_FILES_LISTEN", "/healthz"),
                "sse-notifications" => {
                    ("yetlet-sse-notifications", "YETLET_SSE_LISTEN", "/healthz")
                }
                "auth-notes" => ("yetlet-auth-notes", "YETLET_AUTH_LISTEN", "/healthz"),
                _ => return Err(fail("unknown native smoke")),
            };
            let token = "c".repeat(64); // synthetic loopback-only bearer fixture, never logged
            let env = match suite {
                "file-server" => vec![
                    ("YETLET_FILES_TOKEN", token.clone()),
                    ("TMPDIR", state_path.to_string_lossy().into_owned()),
                    ("TMP", state_path.to_string_lossy().into_owned()),
                    ("TEMP", state_path.to_string_lossy().into_owned()),
                ],
                "auth-notes" => vec![
                    ("YETLET_AUTH_ALICE_CODE", "a".repeat(64)),
                    ("YETLET_AUTH_BOB_CODE", "b".repeat(64)),
                ],
                _ => vec![],
            };
            let mut server = Server::start(root, out, binary, "debug", listen, &env, ready)?;
            let checks = match suite {
                "hello-world" => hello(&server.base),
                "rest-api" => rest(&server.base),
                "cart" => cart(&server.base),
                "middleware-lab" => middleware(&server.base),
                "forms-validation" => forms(&server.base),
                "file-server" => files(&server, &state_path, &token),
                "sse-notifications" => sse(&server),
                "auth-notes" => auth(&server.base),
                _ => unreachable!(),
            };
            let cleanup = server.stop(matches!(
                suite,
                "file-server" | "sse-notifications" | "auth-notes"
            ));
            cleanup?;
            if suite == "file-server" {
                check!(fs::read_dir(&state_path)?.next().is_none());
            }
            if suite == "auth-notes" {
                let log = fs::read_to_string(&server.log)?;
                check!(!log.contains(&"a".repeat(64)) && !log.contains(&"b".repeat(64)));
            }
            checks
        }
    };
    state.close()?;
    check!(!state_path.exists());
    result?;
    println!(
        "PASS: {suite}: original native scenarios; owned processes reaped, listeners closed, disposable state removed"
    );
    Ok(())
}
pub fn hello(base: &str) -> Result<()> {
    let r = http::get(base, "/")?;
    status(&r, 200)?;
    let html = r.text()?;
    check!(html.contains("Hello, world!") && html.contains("<main>"));
    check!(
        r.header("content-security-policy")?
            .contains("default-src 'none'")
    );
    check!(r.header("content-type")?.starts_with("text/html"));
    check!(!html.contains("<script") && !html.contains("wasm"));
    check!(!r.headers.contains_key("set-cookie") && !r.headers.contains_key("x-yetlet-request-id"));
    let r = http::get(base, "/hello/%3Cimg%20src%3Dx%20onerror%3Dalert(1)%3E")?;
    status(&r, 200)?;
    check!(r.text()?.contains("&lt;img src=x onerror=alert(1)&gt;") && !r.text()?.contains("<img"));
    check!(!r.headers.contains_key("set-cookie"));
    status(&http::get(base, "/missing")?, 404)?;
    let r = http::call(base, "POST", "/hello/Yetlet", None, &[])?;
    status(&r, 405)?;
    r.header("allow")?;
    Ok(())
}
pub fn rest(base: &str) -> Result<()> {
    check!(http::get(base, "/healthz")?.json()? == json!({"status":"ok"}));
    let r = http::get(base, "/api/books")?;
    status(&r, 200)?;
    check!(r.json()? == json!({"items":[],"count":0}));
    check!(r.header("x-yetlet-api-version")? == "1" && r.header("cache-control")? == "no-store");
    r.header("x-yetlet-request-id")?;
    let r = http::json(
        base,
        "POST",
        "/api/books",
        &json!({"title":"API Design","author":"Yetlet","year":2026}),
        &[],
    )?;
    status(&r, 201)?;
    let v = r.json()?;
    check!(v["id"] == 1 && v["version"] == 1);
    check!(r.header("location")? == "/api/books/1");
    let r = http::get(base, "/api/books/1")?;
    status(&r, 200)?;
    check!(r.json()?["title"] == "API Design");
    let r = http::get(base, "/api/books?q=yetlet&limit=5")?;
    status(&r, 200)?;
    check!(r.json()?["count"] == 1);
    let mut update = json!({"title":"Changed","author":"Yetlet","year":2026,"expected_version":0});
    error(
        http::json(base, "PUT", "/api/books/1", &update, &[])?,
        409,
        "stale_version",
    )?;
    update["expected_version"] = json!(1);
    let r = http::json(base, "PUT", "/api/books/1", &update, &[])?;
    status(&r, 200)?;
    check!(r.json()?["version"] == 2);
    error(
        http::json(
            base,
            "POST",
            "/api/books",
            &json!({"title":"Attack","author":"Other","year":2020}),
            &[("Origin", "https://other.example")],
        )?,
        403,
        "forbidden_origin",
    )?;
    error(
        http::call(
            base,
            "POST",
            "/api/books",
            Some(b"{}"),
            &[("content-type", "text/plain")],
        )?,
        415,
        "unsupported_media_type",
    )?;
    let r = http::call(base, "DELETE", "/api/books/1", None, &[])?;
    status(&r, 204)?;
    check!(r.body.is_empty());
    error(http::get(base, "/api/books/1")?, 404, "not_found")?;
    status(&http::get(base, "/app")?, 404)?;
    status(&http::get(base, "/")?, 404)
}
pub fn cart(base: &str) -> Result<()> {
    let r = http::get(base, "/healthz")?;
    status(&r, 200)?;
    check!(r.body == b"ok" && !r.headers.contains_key("set-cookie"));
    let r = http::get(base, "/cart")?;
    status(&r, 200)?;
    check!(r.text()?.contains("Yetlet Session Cart") && !r.text()?.contains("<script"));
    let old = r.cookie()?;
    let token = r.hidden("csrf")?;
    check!(token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()));
    let policy = r.header("set-cookie")?;
    check!(
        policy.contains("HttpOnly")
            && policy.contains("SameSite=Lax")
            && policy.contains("Path=/")
            && !policy.contains("Secure")
    );
    check!(r.header("cache-control")? == "no-store");
    let r = http::form(
        base,
        "/cart/add",
        &old,
        &[("csrf", &token), ("product", "rust-book")],
    )?;
    status(&r, 303)?;
    check!(r.header("location")? == "/cart");
    let r = http::authenticated_get(base, "/cart", &old)?;
    status(&r, 200)?;
    check!(r.text()?.contains("Quantity: 1") && r.text()?.contains("Cart total: $39.99"));
    let token = r.hidden("csrf")?;
    let r = http::form(
        base,
        "/cart/add",
        &old,
        &[("csrf", "0"), ("product", "rust-book")],
    )?;
    status(&r, 403)?;
    check!(r.body == b"Invalid CSRF token");
    let r = http::authenticated_get(base, "/cart", &old)?;
    check!(r.text()?.contains("Quantity: 1") && !r.text()?.contains("Quantity: 2"));
    let r = http::form(base, "/cart/clear", &old, &[("csrf", &token)])?;
    status(&r, 303)?;
    let new = r.cookie()?;
    check!(new != old);
    let r = http::authenticated_get(base, "/cart", &new)?;
    status(&r, 200)?;
    check!(r.text()?.contains("Your cart is empty."));
    let token = r.hidden("csrf")?;
    let r = http::authenticated_get(base, "/cart", &old)?;
    status(&r, 200)?;
    check!(r.text()?.contains("Your cart is empty.") && r.cookie()? != old);
    let r = http::form(base, "/cart/end", &new, &[("csrf", &token)])?;
    status(&r, 303)?;
    check!(r.header("set-cookie")?.contains("Max-Age=0"));
    status(&http::get(base, "/")?, 303)?;
    status(&http::get(base, "/app")?, 404)
}
pub fn middleware(base: &str) -> Result<()> {
    let r = http::call(
        base,
        "GET",
        "/lab/guarded",
        None,
        &[("x-lab-user", "Alice")],
    )?;
    status(&r, 200)?;
    check!(r.body == b"Service: Yetlet; Request: Alice");
    check!(
        r.header("x-mw-order")?
            == "global.before,second.before,group.before,route.before,handler,route.after,group.after,second.after,global.after"
    );
    check!(r.header("x-native-tower")? == "tower-http");
    let r = http::call(base, "GET", "/lab/guarded", None, &[("x-stop", "1")])?;
    status(&r, 403)?;
    check!(!r.header("x-mw-order")?.contains("handler"));
    let r = http::get(base, "/outside")?;
    status(&r, 200)?;
    check!(r.body == b"Outside: Yetlet" && !r.header("x-mw-order")?.contains("group.before"));
    let r = http::get(base, "/lab/missing")?;
    status(&r, 404)?;
    check!(r.header("x-mw-order")? == "global.before,second.before,second.after,global.after");
    let r = http::call(base, "POST", "/lab/guarded", None, &[])?;
    status(&r, 405)?;
    check!(r.header("allow")?.contains("GET") && r.header("x-mw-order")?.contains("route.before"));
    let r = http::get(base, "/lab/stream")?;
    status(&r, 200)?;
    check!(r.body == b"first|second" && r.header("x-mw-order")?.contains("group.after"));
    Ok(())
}
pub fn forms(base: &str) -> Result<()> {
    let r = http::get(base, "/form")?;
    status(&r, 200)?;
    let html = r.text()?;
    check!(
        r.header("content-security-policy")?
            .contains("form-action 'self'")
    );
    check!(
        html.contains("for=\"name\"")
            && html.contains("for=\"email\"")
            && !html.contains("<script")
            && !r.headers.contains_key("set-cookie")
    );
    document(html)?;
    let r = http::form(
        base,
        "/form/submit",
        "",
        &[("name", "<script>"), ("email", "a\" onfocus=alert(1)")],
    )?;
    status(&r, 422)?;
    let html = r.text()?;
    document(html)?;
    check!(
        html.contains("&lt;script&gt;")
            && html.contains("&quot;")
            && html.contains("aria-describedby")
            && html.contains("email-error")
            && html.contains("autofocus")
            && !html.contains("<script>")
    );
    check!(r.header("cache-control")? == "no-store");
    let r = http::form(
        base,
        "/form/submit",
        "",
        &[("name", "x"), ("email", "not-an-email")],
    )?;
    status(&r, 422)?;
    let html = r.text()?;
    check!(
        html.contains("name-error")
            && html.contains("email-error")
            && html.contains("aria-describedby=\"name-error\"")
            && html.contains("autofocus")
    );
    let r = http::form(
        base,
        "/form/submit",
        "",
        &[("name", "Ada Lovelace"), ("email", "ada@example.com")],
    )?;
    status(&r, 303)?;
    check!(r.header("location")? == "/form/success" && !r.headers.contains_key("set-cookie"));
    let r = http::get(base, "/form/success")?;
    status(&r, 200)?;
    document(r.text()?)?;
    check!(r.text()?.contains("No data was saved.") && !r.text()?.contains("Ada Lovelace"));
    status(&http::get(base, "/form")?, 200)
}
fn document(html: &str) -> Result<()> {
    require(
        html.starts_with("<!doctype html><html lang=\"en\">") && !html.contains("<main lang="),
        "root English document semantics",
    )
}

pub fn sqlite(root: &Path, out: &Path, state: &Path) -> Result<()> {
    let database = state.join("notes.sqlite3");
    let env = [(
        "YETLET_SQLITE_PATH",
        database.to_string_lossy().into_owned(),
    )];
    for phase in 0..3 {
        let mut s = Server::start(
            root,
            out,
            "yetlet-sqlite-crud",
            "debug",
            "YETLET_SQLITE_LISTEN",
            &env,
            "/healthz",
        )?;
        let b = &s.base;
        let result: Result<()> = (|| {
            check!(http::get(b, "/healthz")?.json()? == json!({"status":"ok","schema_version":1}));
            if phase == 0 {
                let r = http::json(
                    b,
                    "POST",
                    "/api/notes",
                    &json!({"title":"Persistent note","body":"survives process restart"}),
                    &[],
                )?;
                status(&r, 201)?;
                let v = r.json()?;
                check!(
                    v["id"] == 1
                        && v["version"] == 1
                        && r.header("location")? == "/api/notes/1"
                        && r.header("x-yetlet-sqlite-schema")? == "1"
                        && r.header("cache-control")? == "no-store"
                );
                error(
                    http::json(
                        b,
                        "POST",
                        "/api/notes",
                        &json!({"title":"Browser write","body":"rejected"}),
                        &[("Origin", "https://other.example")],
                    )?,
                    403,
                    "browser_origin_forbidden",
                )?;
                error(
                    http::json(
                        b,
                        "PUT",
                        "/api/notes/1",
                        &json!({"title":"stale","body":"no","expected_version":9}),
                        &[],
                    )?,
                    409,
                    "stale_version",
                )?;
                let r = http::json(
                    b,
                    "PUT",
                    "/api/notes/1",
                    &json!({"title":"Persisted update","body":"version two","expected_version":1}),
                    &[],
                )?;
                status(&r, 200)?;
                check!(r.json()?["version"] == 2 && database.is_file());
            } else if phase == 1 {
                let r = http::get(b, "/api/notes/1")?;
                status(&r, 200)?;
                let v = r.json()?;
                check!(v["title"] == "Persisted update" && v["version"] == 2);
                let results = std::thread::scope(|scope| {
                    let one=scope.spawn(||http::json(b,"PUT","/api/notes/1",&json!({"title":"Concurrent A","body":"Concurrent A","expected_version":2}),&[]));
                    let two=scope.spawn(||http::json(b,"PUT","/api/notes/1",&json!({"title":"Concurrent B","body":"Concurrent B","expected_version":2}),&[]));
                    (one.join().unwrap(), two.join().unwrap())
                });
                let mut codes = [results.0?.status, results.1?.status];
                codes.sort();
                check!(codes == [200, 409]);
                let r = http::get(b, "/api/notes/1")?;
                status(&r, 200)?;
                check!(r.json()?["version"] == 3);
                error(
                    http::call(b, "DELETE", "/api/notes/1?expected_version=2", None, &[])?,
                    409,
                    "stale_version",
                )?;
                let r = http::call(b, "DELETE", "/api/notes/1?expected_version=3", None, &[])?;
                status(&r, 204)?;
                check!(r.body.is_empty());
                status(&http::get(b, "/api/notes/1")?, 404)?;
                status(&http::get(b, "/app")?, 404)?;
            } else {
                let r = http::get(b, "/api/notes")?;
                status(&r, 200)?;
                check!(r.json()? == json!({"items":[],"count":0}));
            }
            Ok(())
        })();
        s.stop(true)?;
        result?;
    }
    Ok(())
}
pub fn login(base: &str, user: &str, code: &str) -> Result<(String, String, String, String)> {
    let r = http::get(base, "/auth/login")?;
    status(&r, 200)?;
    let old = r.cookie()?;
    let old_csrf = r.hidden("csrf")?;
    let r = http::form(
        base,
        "/auth/login",
        &old,
        &[("csrf", &old_csrf), ("username", user), ("code", code)],
    )?;
    status(&r, 303)?;
    check!(r.header("location")? == "/auth/notes");
    let cookie = r.cookie()?;
    check!(cookie != old);
    let r = http::authenticated_get(base, "/auth/notes", &cookie)?;
    status(&r, 200)?;
    Ok((cookie, r.hidden("csrf")?, old, old_csrf))
}
pub fn auth(base: &str) -> Result<()> {
    let a = "a".repeat(64);
    let b = "b".repeat(64);
    let r = http::get(base, "/healthz")?;
    status(&r, 200)?;
    check!(!r.headers.contains_key("set-cookie"));
    let r = http::get(base, "/auth/login")?;
    status(&r, 200)?;
    let html = r.text()?;
    check!(!html.contains(&a) && !html.contains(&b) && !html.contains("<script"));
    let policy = r.header("set-cookie")?;
    check!(
        policy.contains("HttpOnly")
            && policy.contains("SameSite=Lax")
            && !policy.contains("Secure")
    );
    let r = http::form(
        base,
        "/auth/login",
        &r.cookie()?,
        &[
            ("csrf", &r.hidden("csrf")?),
            ("username", "alice"),
            ("code", &b),
        ],
    )?;
    status(&r, 401)?;
    check!(
        r.text()?.contains("Invalid synthetic identity") && !r.headers.contains_key("set-cookie")
    );
    let (alice, csrf, old, old_csrf) = login(base, "alice", &a)?;
    check!(csrf != old_csrf);
    status(&http::authenticated_get(base, "/auth/notes", &old)?, 401)?;
    status(
        &http::form(
            base,
            "/auth/notes",
            &alice,
            &[
                ("csrf", &old_csrf),
                ("title", "blocked"),
                ("body", "blocked"),
            ],
        )?,
        403,
    )?;
    let r = http::form(
        base,
        "/auth/notes",
        &alice,
        &[
            ("csrf", &csrf),
            ("title", "<script>alert(1)</script>"),
            ("body", "Alice private body"),
        ],
    )?;
    status(&r, 303)?;
    check!(r.header("location")? == "/auth/notes/1");
    let r = http::authenticated_get(base, "/auth/notes/1", &alice)?;
    status(&r, 200)?;
    check!(
        r.text()?.contains("&lt;script&gt;alert(1)&lt;/script&gt;")
            && !r.text()?.contains("<script>alert")
    );
    let note_csrf = r.hidden("csrf")?;
    let (bob, bob_csrf, _, _) = login(base, "bob", &b)?;
    let r = http::authenticated_get(base, "/auth/notes", &bob)?;
    status(&r, 200)?;
    check!(!r.text()?.contains("data-note-id=\"1\""));
    status(&http::authenticated_get(base, "/auth/notes/1", &bob)?, 403)?;
    status(
        &http::form(
            base,
            "/auth/notes/1/update",
            &bob,
            &[("csrf", &bob_csrf), ("title", "stolen"), ("body", "stolen")],
        )?,
        403,
    )?;
    status(
        &http::form(base, "/auth/notes/1/delete", &bob, &[("csrf", &bob_csrf)])?,
        403,
    )?;
    status(
        &http::form(
            base,
            "/auth/notes/1/update",
            &alice,
            &[("csrf", "bad"), ("title", "changed"), ("body", "changed")],
        )?,
        403,
    )?;
    let r = http::authenticated_get(base, "/auth/notes/1", &alice)?;
    status(&r, 200)?;
    check!(r.text()?.contains("Alice private body"));
    let r = http::form(base, "/auth/logout", &alice, &[("csrf", &note_csrf)])?;
    status(&r, 303)?;
    check!(r.header("location")? == "/auth/login");
    let anonymous = r.cookie()?;
    check!(anonymous != alice);
    status(
        &http::authenticated_get(base, "/auth/notes", &anonymous)?,
        401,
    )?;
    status(&http::authenticated_get(base, "/auth/notes", &alice)?, 401)
}
fn shop_login(c: &mut Cookies, b: &str, user: &str) -> Result<()> {
    let r = c.get(b, "/identity/login")?;
    status(&r, 200)?;
    let r = c.form(
        b,
        "/identity/login",
        &[
            ("csrf", &r.hidden("csrf")?),
            ("username", user),
            (
                "code",
                &if user == "alice" {
                    "a".repeat(64)
                } else {
                    "b".repeat(64)
                },
            ),
        ],
    )?;
    status(&r, 303)?;
    check!(r.header("location")? == "/shop/catalog");
    Ok(())
}
pub fn shop(root: &Path, out: &Path, state: &Path) -> Result<()> {
    let db = state.join("shop.sqlite3");
    let mut order = String::new();
    for phase in 0..3 {
        let mut env = vec![
            ("YETLET_SHOP_DB", db.to_string_lossy().into_owned()),
            ("YETLET_SHOP_ALICE_CODE", "a".repeat(64)),
            ("YETLET_SHOP_BOB_CODE", "b".repeat(64)),
        ];
        if phase == 2 {
            env.push(("YETLET_SHOP_DISABLE_ORDERS", "1".into()));
        }
        let mut s = Server::start(
            root,
            out,
            "yetlet-modular-shop",
            "debug",
            "YETLET_SHOP_LISTEN",
            &env,
            "/healthz",
        )?;
        let b = &s.base;
        let result: Result<()> = (|| {
            let mut c = Cookies::default();
            if phase == 0 {
                let r = c.get(b, "/shop/catalog")?;
                status(&r, 200)?;
                check!(
                    r.text()?.contains("Modular Shop Catalog")
                        && !r.text()?.contains("<script")
                        && r.header("content-security-policy")?
                            .contains("default-src 'none'")
                );
                shop_login(&mut c, b, "alice")?;
                let r = c.get(b, "/shop/cart")?;
                status(&r, 200)?;
                let r = c.form(
                    b,
                    "/shop/cart/add",
                    &[
                        ("csrf", &r.hidden("csrf")?),
                        ("product_id", "rust-book"),
                        ("expected_revision", &r.hidden("expected_revision")?),
                    ],
                )?;
                status(&r, 303)?;
                let r = c.get(b, "/shop/cart")?;
                status(&r, 200)?;
                check!(r.text()?.contains("data-cart-product=\"rust-book\""));
                let r = c.form(
                    b,
                    "/shop/orders/checkout",
                    &[
                        ("csrf", &r.hidden("csrf")?),
                        ("expected_revision", &r.hidden("expected_revision")?),
                    ],
                )?;
                status(&r, 303)?;
                order = r.header("location")?.into();
                check!(order.starts_with("/shop/orders/"));
                check!(order.rsplit('/').next().unwrap().parse::<u64>().is_ok());
                let r = c.get(b, &order)?;
                status(&r, 200)?;
                check!(r.text()?.contains("The Rust Programming Language"));
            } else if phase == 1 {
                shop_login(&mut c, b, "alice")?;
                let r = c.get(b, "/shop/orders")?;
                status(&r, 200)?;
                check!(
                    r.text()?
                        .contains(&format!("Order #{}", order.rsplit('/').next().unwrap()))
                );
                let mut bob = Cookies::default();
                shop_login(&mut bob, b, "bob")?;
                status(&bob.get(b, &order)?, 403)?;
            } else {
                let r = c.get(b, "/shop/catalog")?;
                status(&r, 200)?;
                check!(!r.text()?.contains("href=\"/shop/orders\""));
                status(&c.get(b, "/shop/orders")?, 404)?;
                shop_login(&mut c, b, "alice")?;
                let r = c.get(b, "/shop/cart")?;
                status(&r, 200)?;
                check!(
                    r.text()?.contains("data-orders-disabled=\"true\"")
                        && !r.text()?.contains("action=\"/shop/orders/checkout\"")
                );
            }
            Ok(())
        })();
        s.stop(true)?;
        result?;
    }
    check!(db.is_file());
    Ok(())
}
fn multipart(name: &str, bytes: &[u8], close: bool) -> Vec<u8> {
    let mut body=format!("--sample\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: text/plain\r\n\r\n").into_bytes();
    body.extend(bytes);
    if close {
        body.extend(b"\r\n--sample--\r\n");
    }
    body
}
fn directory(path: &Path) -> Result<std::collections::BTreeSet<std::path::PathBuf>> {
    Ok(fs::read_dir(path)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?)
}
pub fn files(s: &Server, state: &Path, token: &str) -> Result<()> {
    let b = &s.base;
    let auth = format!("Bearer {token}");
    let call = |method: &str,
                path: &str,
                body: Option<&[u8]>,
                authenticated: bool,
                extra: &[(&str, &str)]| {
        let mut h = extra.to_vec();
        if authenticated {
            h.push(("Authorization", auth.as_str()));
        }
        if body.is_some() {
            h.push(("Content-Type", "multipart/form-data; boundary=sample"));
        }
        http::call(b, method, path, body, &h)
    };
    let r = http::get(b, "/")?;
    status(&r, 200)?;
    check!(
        r.text()?.contains("<main lang=\"en\">")
            && !r.text()?.contains(token)
            && !r.text()?.contains("<script")
            && r.header("content-security-policy")?
                .contains("default-src 'none'")
    );
    let r = http::get(b, "/static/welcome.txt")?;
    status(&r, 200)?;
    check!(r.body.starts_with(b"Yetlet") && r.header("x-content-type-options")? == "nosniff");
    let length = r.body.len();
    let r = call("HEAD", "/static/welcome.txt", None, false, &[])?;
    status(&r, 200)?;
    check!(r.body.is_empty() && r.header("content-length")?.parse::<usize>().ok() == Some(length));
    let r = call(
        "GET",
        "/static/welcome.txt",
        None,
        false,
        &[("Range", "bytes=0-5")],
    )?;
    status(&r, 206)?;
    check!(r.body == b"Yetlet");
    let text = b"A streamed UTF-8 upload.\n";
    let good = multipart("example.txt", text, true);
    status(&call("POST", "/uploads", Some(&good), false, &[])?, 401)?;
    status(
        &call("POST", "/uploads", Some(&good), true, &[("Origin", b)])?,
        403,
    )?;
    let r = call("POST", "/uploads", Some(&good), true, &[])?;
    status(&r, 201)?;
    let resource = r.header("location")?.to_owned();
    check!(resource.starts_with("/files/"));
    status(&call("GET", &resource, None, false, &[])?, 401)?;
    for _ in 0..2 {
        let r = call("GET", &resource, None, true, &[])?;
        status(&r, 200)?;
        check!(
            r.body == text
                && r.header("content-type")? == "application/octet-stream"
                && r.header("content-disposition")? == "attachment; filename=\"example.txt\""
                && r.header("cache-control")? == "no-store"
        );
    }
    let r = call("HEAD", &resource, None, true, &[])?;
    status(&r, 200)?;
    check!(
        r.body.is_empty() && r.header("content-length")?.parse::<usize>().ok() == Some(text.len())
    );
    let r = call("POST", "/uploads", Some(&good), true, &[])?;
    status(&r, 201)?;
    check!(r.header("location")? != resource);
    for (name, data, code) in [
        ("../escape.txt", b"bad".to_vec(), 400),
        ("run.exe", b"bad".to_vec(), 400),
        ("binary.txt", b"binary\0".to_vec(), 415),
        ("large.txt", vec![b'x'; 1024 * 1024 + 1], 413),
    ] {
        status(
            &call(
                "POST",
                "/uploads",
                Some(&multipart(name, &data, true)),
                true,
                &[],
            )?,
            code,
        )?;
    }
    status(
        &call("GET", "/static/%2e%2e%2fCargo.toml", None, false, &[])?,
        404,
    )?;
    status(
        &call("GET", "/files/%2e%2e%2fetc%2fpasswd", None, true, &[])?,
        404,
    )?;
    status(&call("GET", "/static/", None, false, &[])?, 404)?;
    status(&call("GET", "/uploads", None, true, &[])?, 405)?;
    let roots = directory(state)?
        .into_iter()
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("yetlet-files-")
        })
        .collect::<Vec<_>>();
    check!(roots.len() == 1);
    let uploads = roots[0].join("uploads");
    let baseline = directory(&uploads)?;
    let mut raw = TcpStream::connect(("127.0.0.1", s.port))?;
    raw.set_write_timeout(Some(Duration::from_secs(3)))?;
    raw.write_all(format!("POST /uploads HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {token}\r\nContent-Type: multipart/form-data; boundary=sample\r\nContent-Length: 100000\r\nConnection: close\r\n\r\n",s.port).as_bytes())?;
    raw.write_all(&multipart("aborted.txt", b"unfinished", false))?;
    wait(3, "partial upload creation", || {
        let files = directory(&uploads)?;
        Ok((files != baseline).then_some(files))
    })
    .and_then(|files| {
        require(
            files.difference(&baseline).count() == 1,
            "one partial upload",
        )
    })?;
    drop(raw);
    wait(12, "disconnected upload cleanup", || {
        Ok((directory(&uploads)? == baseline).then_some(()))
    })?;
    status(&call("POST", "/uploads", Some(&good), true, &[])?, 201)
}
/// A bounded stream reader; ureq handles HTTP chunk framing, not SSE semantics.
struct Events {
    reader: std::io::BufReader<Box<dyn Read + Send + Sync>>,
}
impl Events {
    fn open(s: &Server, cursor: Option<u64>) -> Result<Self> {
        let mut req = http::agent(30).get(format!("{}/events", s.base));
        if let Some(cursor) = cursor {
            req = req.header("Last-Event-ID", cursor.to_string());
        }
        let r = req.call().map_err(|_| fail("SSE open failed"))?;
        check!(
            r.status() == 200
                && r.headers()
                    .get("content-type")
                    .and_then(|h| h.to_str().ok())
                    .is_some_and(|v| v.starts_with("text/event-stream"))
        );
        check!(
            r.headers()["cache-control"] == "no-store" && r.headers()["x-accel-buffering"] == "no"
        );
        Ok(Self {
            reader: std::io::BufReader::new(Box::new(r.into_body().into_reader())),
        })
    }
    fn item(&mut self) -> Result<Value> {
        let mut fields = serde_json::Map::new();
        let mut data = Vec::new();
        loop {
            crate::tooling::check_cancel()?;
            let mut line = String::new();
            check!(self.reader.read_line(&mut line)? > 0);
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                if !data.is_empty() {
                    fields.insert("data".into(), json!(data.join("\n")));
                }
                if !fields.is_empty() {
                    return Ok(Value::Object(fields));
                }
                continue;
            }
            if let Some(comment) = line.strip_prefix(':') {
                return Ok(json!({"comment":comment.trim()}));
            }
            if let Some((key, value)) = line.split_once(':') {
                let value = value.strip_prefix(' ').unwrap_or(value);
                if key == "data" {
                    data.push(value.to_owned());
                } else if ["id", "event", "retry"].contains(&key) {
                    fields.insert(key.into(), json!(value));
                }
            }
        }
    }
}
pub fn sse(s: &Server) -> Result<()> {
    let b = &s.base;
    check!(http::get(b, "/healthz")?.json()? == json!({"status":"ok"}));
    let subscribers = |count: u64| {
        wait(3, "SSE subscriber count", || {
            let r = http::get(b, "/metrics")?;
            Ok((r.status == 200 && r.json()?["active_subscribers"] == count).then_some(()))
        })
    };
    let mut stream = Events::open(s, None)?;
    check!(stream.item()? == json!({"comment":"heartbeat"}));
    subscribers(1)?;
    for message in ["one", "two", "three"] {
        let r = http::json(
            b,
            "POST",
            "/notifications",
            &json!({"message":message}),
            &[],
        )?;
        status(&r, 201)?;
        check!(r.json()?["message"] == message);
    }
    for (i, message) in ["one", "two", "three"].iter().enumerate() {
        let item = stream.item()?;
        check!(
            item["id"].as_str().and_then(|s| s.parse::<usize>().ok()) == Some(i + 1)
                && item["event"] == "notification"
        );
        let data: Value = serde_json::from_str(
            item["data"]
                .as_str()
                .ok_or_else(|| fail("SSE data missing"))?,
        )
        .map_err(|_| fail("SSE JSON malformed"))?;
        check!(data["message"] == *message);
    }
    drop(stream);
    subscribers(0)?;
    for message in ["four", "five"] {
        status(
            &http::json(
                b,
                "POST",
                "/notifications",
                &json!({"message":message}),
                &[],
            )?,
            201,
        )?;
    }
    let mut stream = Events::open(s, Some(3))?;
    for (i, message) in ["four", "five"].iter().enumerate() {
        let item = stream.item()?;
        check!(item["id"].as_str().and_then(|s| s.parse::<usize>().ok()) == Some(i + 4));
        let data: Value = serde_json::from_str(
            item["data"]
                .as_str()
                .ok_or_else(|| fail("SSE data missing"))?,
        )
        .map_err(|_| fail("SSE JSON malformed"))?;
        check!(data["message"] == *message);
    }
    drop(stream);
    subscribers(0)?;
    error(
        http::call(b, "GET", "/events", None, &[("Last-Event-ID", "bad")])?,
        400,
        "invalid_last_event_id",
    )?;
    error(
        http::call(b, "GET", "/events", None, &[("Last-Event-ID", "99")])?,
        409,
        "future_cursor",
    )?;
    error(
        http::json(
            b,
            "POST",
            "/notifications",
            &json!({"message":"browser blocked"}),
            &[("Origin", "https://example.test")],
        )?,
        403,
        "browser_origin_forbidden",
    )
}
