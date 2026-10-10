use std::{collections::BTreeMap, net::SocketAddr};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use tempfile::TempDir;
use tower::ServiceExt;

use super::*;

const ALICE_CODE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BOB_CODE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

fn fixture(modules: ShopModules) -> (TempDir, Database, Router) {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(directory.path().join("shop.sqlite3")).unwrap();
    let router = app(
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        database.clone(),
        DemoIdentities::from_codes(ALICE_CODE, BOB_CODE).unwrap(),
        modules,
    )
    .unwrap();
    (directory, database, router)
}

async fn send(
    router: &Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    form: Option<String>,
) -> Reply {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let body = if let Some(form) = form {
        builder = builder.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        Body::from(form)
    } else {
        Body::empty()
    };
    let response = router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    Reply {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

fn cookie(reply: &Reply) -> String {
    reply.headers[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn hidden(html: &str, name: &str) -> String {
    let marker = format!("name=\"{name}\" value=\"");
    let start = html.find(&marker).unwrap() + marker.len();
    let rest = &html[start..];
    rest[..rest.find('"').unwrap()].to_owned()
}

async fn login(router: &Router, username: &str, code: &str) -> (String, String) {
    let page = send(router, Method::GET, "/identity/login", None, None).await;
    assert_eq!(page.status, StatusCode::OK);
    let anonymous_cookie = cookie(&page);
    let csrf = hidden(&page.body, "csrf");
    let response = send(
        router,
        Method::POST,
        "/identity/login",
        Some(&anonymous_cookie),
        Some(format!("csrf={csrf}&username={username}&code={code}")),
    )
    .await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    let authenticated_cookie = cookie(&response);
    assert_ne!(authenticated_cookie, anonymous_cookie);
    (authenticated_cookie, anonymous_cookie)
}

async fn cart_page(router: &Router, cookie: &str) -> Reply {
    send(router, Method::GET, "/shop/cart", Some(cookie), None).await
}

async fn add_product(router: &Router, cookie_value: &str, product: &str) {
    let page = cart_page(router, cookie_value).await;
    assert_eq!(page.status, StatusCode::OK);
    let csrf = hidden(&page.body, "csrf");
    let revision = hidden(&page.body, "expected_revision");
    let response = send(
        router,
        Method::POST,
        "/shop/cart/add",
        Some(cookie_value),
        Some(format!(
            "csrf={csrf}&product_id={product}&expected_revision={revision}"
        )),
    )
    .await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
}

async fn checkout(router: &Router, cookie_value: &str) -> Reply {
    let page = cart_page(router, cookie_value).await;
    let csrf = hidden(&page.body, "csrf");
    let revision = hidden(&page.body, "expected_revision");
    send(
        router,
        Method::POST,
        "/shop/orders/checkout",
        Some(cookie_value),
        Some(format!("csrf={csrf}&expected_revision={revision}")),
    )
    .await
}

fn order_id(reply: &Reply) -> i64 {
    reply.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn identities_are_distinct_valid_and_redacted() {
    assert!(DemoIdentities::from_codes(ALICE_CODE, BOB_CODE).is_ok());
    assert!(DemoIdentities::from_codes(ALICE_CODE, ALICE_CODE).is_err());
    let identities = DemoIdentities::from_codes(ALICE_CODE, BOB_CODE).unwrap();
    assert_eq!(
        identities.authenticate("alice", ALICE_CODE),
        Some(UserId::Alice)
    );
    assert_eq!(identities.authenticate("bob", BOB_CODE), Some(UserId::Bob));
    assert_eq!(identities.authenticate("mallory", ALICE_CODE), None);
    let debug = format!("{identities:?}");
    assert!(!debug.contains(ALICE_CODE));
    assert!(!debug.contains(BOB_CODE));
}

#[tokio::test]
async fn modules_are_explicit_and_orders_can_be_disabled() {
    let modules = ShopModules {
        orders: false,
        ..ShopModules::default()
    };
    let (_directory, _database, router) = fixture(modules);
    assert_eq!(
        send(&router, Method::GET, "/shop/catalog", None, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        send(&router, Method::GET, "/shop/orders", None, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let catalog = send(&router, Method::GET, "/shop/catalog", None, None).await;
    assert!(!catalog.body.contains(r#"href="/shop/orders""#));
    let (alice, _) = login(&router, "alice", ALICE_CODE).await;
    let cart = cart_page(&router, &alice).await;
    assert!(cart.body.contains(r#"data-orders-disabled="true""#));
    assert!(!cart.body.contains(r#"action="/shop/orders/checkout""#));
    let invalid = ShopModules {
        identity: false,
        ..ShopModules::default()
    };
    let directory = tempfile::tempdir().unwrap();
    let database = Database::open(directory.path().join("invalid.sqlite3")).unwrap();
    assert!(matches!(
        app(
            "127.0.0.1:0".parse().unwrap(),
            database,
            DemoIdentities::from_codes(ALICE_CODE, BOB_CODE).unwrap(),
            invalid
        ),
        Err(ShopAppError::InvalidModules(_))
    ));
}

#[tokio::test]
async fn login_rotates_session_and_old_identity_cannot_access_cart() {
    let (_directory, _database, router) = fixture(ShopModules::default());
    let (authenticated, anonymous) = login(&router, "alice", ALICE_CODE).await;
    assert_eq!(
        cart_page(&router, &authenticated).await.status,
        StatusCode::OK
    );
    assert_eq!(
        cart_page(&router, &anonymous).await.status,
        StatusCode::UNAUTHORIZED
    );
    let catalog = send(&router, Method::GET, "/shop/catalog", None, None).await;
    assert_eq!(catalog.status, StatusCode::OK);
    assert!(catalog.body.contains("<main lang=\"en\">"));
    assert!(!catalog.body.contains("<script"));
    assert!(
        catalog.headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .contains("default-src 'none'")
    );
}

#[tokio::test]
async fn checkout_commits_inventory_persists_and_enforces_cross_user_ownership() {
    let (directory, database, router) = fixture(ShopModules::default());
    let (alice, _) = login(&router, "alice", ALICE_CODE).await;
    add_product(&router, &alice, "rust-book").await;
    let response = checkout(&router, &alice).await;
    assert_eq!(response.status, StatusCode::SEE_OTHER);
    let id = order_id(&response);
    assert_eq!(database.stock("rust-book").await, 7);
    assert_eq!(database.order_count().await, 1);

    let detail = send(
        &router,
        Method::GET,
        &format!("/shop/orders/{id}"),
        Some(&alice),
        None,
    )
    .await;
    assert_eq!(detail.status, StatusCode::OK);
    assert!(detail.body.contains("The Rust Programming Language"));

    let (bob, _) = login(&router, "bob", BOB_CODE).await;
    assert_eq!(
        send(
            &router,
            Method::GET,
            &format!("/shop/orders/{id}"),
            Some(&bob),
            None,
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    let bob_list = send(&router, Method::GET, "/shop/orders", Some(&bob), None).await;
    assert_eq!(bob_list.status, StatusCode::OK);
    assert!(!bob_list.body.contains(&format!("Order #{id}")));

    let reopened = Database::open(directory.path().join("shop.sqlite3")).unwrap();
    assert_eq!(reopened.orders_for(UserId::Alice).await.unwrap().len(), 1);
}

#[tokio::test]
async fn failed_order_rolls_back_inventory_and_keeps_session_cart() {
    let (_directory, database, router) = fixture(ShopModules::default());
    let (alice, _) = login(&router, "alice", ALICE_CODE).await;
    add_product(&router, &alice, "travel-mug").await;
    let connection = database.test_connection();
    connection
        .execute_batch(
            "CREATE TRIGGER fail_order_line BEFORE INSERT ON order_lines BEGIN \
             SELECT RAISE(ABORT, 'forced line failure'); END;",
        )
        .unwrap();
    drop(connection);
    let response = checkout(&router, &alice).await;
    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(database.order_count().await, 0);
    assert_eq!(database.stock("travel-mug").await, 5);
    let page = cart_page(&router, &alice).await;
    assert!(page.body.contains("data-cart-product=\"travel-mug\""));
}

#[tokio::test]
async fn checkout_key_is_idempotent_and_does_not_double_decrement() {
    let (_directory, database, _router) = fixture(ShopModules::default());
    let mut lines = BTreeMap::new();
    lines.insert("rust-book".to_owned(), 1);
    let first = database
        .place_order(UserId::Alice, lines.clone(), "same-key".to_owned())
        .await
        .unwrap();
    let second = database
        .place_order(UserId::Alice, lines, "same-key".to_owned())
        .await
        .unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(database.order_count().await, 1);
    assert_eq!(database.stock("rust-book").await, 7);
}

#[tokio::test]
async fn concurrent_last_stock_has_one_transactional_winner() {
    let (_directory, database, _router) = fixture(ShopModules::default());
    database
        .test_connection()
        .execute("UPDATE products SET stock=1 WHERE id='keyboard'", [])
        .unwrap();
    let mut lines = BTreeMap::new();
    lines.insert("keyboard".to_owned(), 1);
    let first = database.clone();
    let second = database.clone();
    let (alice, bob) = tokio::join!(
        first.place_order(UserId::Alice, lines.clone(), "alice-last".to_owned()),
        second.place_order(UserId::Bob, lines, "bob-last".to_owned())
    );
    assert_eq!(usize::from(alice.is_ok()) + usize::from(bob.is_ok()), 1);
    let loser = if alice.is_err() { alice } else { bob };
    assert!(matches!(loser, Err(database::DatabaseError::OutOfStock)));
    assert_eq!(database.order_count().await, 1);
    assert_eq!(database.stock("keyboard").await, 0);
}

#[tokio::test]
async fn invalid_csrf_and_stale_revision_do_not_place_orders() {
    let (_directory, database, router) = fixture(ShopModules::default());
    let (alice, _) = login(&router, "alice", ALICE_CODE).await;
    add_product(&router, &alice, "rust-book").await;
    let page = cart_page(&router, &alice).await;
    let revision = hidden(&page.body, "expected_revision");
    let invalid = send(
        &router,
        Method::POST,
        "/shop/orders/checkout",
        Some(&alice),
        Some(format!("csrf=bad&expected_revision={revision}")),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::FORBIDDEN);
    assert_eq!(database.order_count().await, 0);

    let csrf = hidden(&page.body, "csrf");
    let stale = send(
        &router,
        Method::POST,
        "/shop/orders/checkout",
        Some(&alice),
        Some(format!("csrf={csrf}&expected_revision=0")),
    )
    .await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    assert_eq!(database.order_count().await, 0);
    assert!(
        cart_page(&router, &alice)
            .await
            .body
            .contains("data-cart-product=\"rust-book\"")
    );
}

#[test]
fn schema_mismatch_and_public_listener_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("shop.sqlite3");
    let database = Database::open(&path).unwrap();
    database
        .test_connection()
        .execute_batch("ALTER TABLE products RENAME COLUMN stock TO inventory")
        .unwrap();
    drop(database);
    assert!(matches!(
        Database::open(&path),
        Err(database::DatabaseError::SchemaMismatch)
    ));
    assert!(listen_address("0.0.0.0:3008").is_err());
    assert!(listen_address("127.0.0.1:3008").is_ok());
}
