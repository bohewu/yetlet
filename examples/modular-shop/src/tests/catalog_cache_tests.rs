use super::*;
fn cached() -> (TempDir, Database, Router) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open_with_options(
        dir.path().join("cached.sqlite"),
        crate::DatabaseOptions {
            catalog_cache: true,
        },
    )
    .unwrap();
    let router = app(
        "127.0.0.1:0".parse().unwrap(),
        db.clone(),
        DemoIdentities::from_codes(ALICE_CODE, BOB_CODE).unwrap(),
        ShopModules::default(),
    )
    .unwrap();
    (dir, db, router)
}

#[tokio::test]
async fn catalog_cache_http_reuse_keeps_session_and_csrf_authoritative() {
    let (_dir, db, router) = cached();
    let public = send(&router, Method::GET, "/shop/catalog", None, None).await;
    assert_eq!(public.status, StatusCode::OK);
    assert!(public.body.contains("up to two seconds old"));
    let (alice, old) = login(&router, "alice", ALICE_CODE).await;
    let (bob, _) = login(&router, "bob", BOB_CODE).await;
    let a = cart_page(&router, &alice).await;
    let b = cart_page(&router, &bob).await;
    assert!(a.body.contains("Alice") && !a.body.contains("Bob"));
    assert!(b.body.contains("Bob") && !b.body.contains("Alice"));
    assert!(a.body.contains("Estimated total"));
    assert_eq!(a.headers[header::CACHE_CONTROL], "no-store");
    assert!(a.headers.contains_key(header::CONTENT_SECURITY_POLICY));
    assert!(!a.headers.contains_key(header::SET_COOKIE));
    assert_ne!(hidden(&a.body, "csrf"), hidden(&b.body, "csrf"));
    assert_eq!(db.test_cache().test_source_loads(), 1);
    let bad = send(
        &router,
        Method::POST,
        "/shop/cart/add",
        Some(&alice),
        Some(format!(
            "csrf={}&product_id=rust-book&expected_revision=0",
            hidden(&b.body, "csrf")
        )),
    )
    .await;
    assert_eq!(bad.status, StatusCode::FORBIDDEN);
    assert_eq!(
        cart_page(&router, &old).await.status,
        StatusCode::UNAUTHORIZED
    );
    let logout = send(
        &router,
        Method::POST,
        "/identity/logout",
        Some(&alice),
        Some(format!("csrf={}", hidden(&a.body, "csrf"))),
    )
    .await;
    assert_eq!(logout.status, StatusCode::SEE_OTHER);
    assert_eq!(
        cart_page(&router, &alice).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(cart_page(&router, &bob).await.status, StatusCode::OK);
    assert_eq!(db.test_cache().test_source_loads(), 1);
}

#[tokio::test]
async fn catalog_cache_checkout_uses_current_price_stock_and_owner() {
    let (_dir, db, router) = cached();
    let (alice, _) = login(&router, "alice", ALICE_CODE).await;
    let (bob, _) = login(&router, "bob", BOB_CODE).await;
    add_product(&router, &alice, "rust-book").await;
    db.test_connection()
        .execute(
            "UPDATE products SET stock=0,price_cents=5999 WHERE id='rust-book'",
            [],
        )
        .unwrap();
    let stale = cart_page(&router, &alice).await;
    assert!(stale.body.contains("$39.99"));
    assert_eq!(db.test_cache().test_source_loads(), 1);
    let rejected_add = send(
        &router,
        Method::POST,
        "/shop/cart/add",
        Some(&alice),
        Some(format!(
            "csrf={}&product_id=rust-book&expected_revision={}",
            hidden(&stale.body, "csrf"),
            hidden(&stale.body, "expected_revision")
        )),
    )
    .await;
    assert_eq!(rejected_add.status, StatusCode::CONFLICT);
    assert_eq!(checkout(&router, &alice).await.status, StatusCode::CONFLICT);
    assert_eq!(db.order_count().await, 0);
    db.test_connection()
        .execute("UPDATE products SET stock=3 WHERE id='rust-book'", [])
        .unwrap();
    let committed = checkout(&router, &alice).await;
    assert_eq!(committed.status, StatusCode::SEE_OTHER);
    let id = order_id(&committed);
    let url = format!("/shop/orders/{id}");
    let detail = send(&router, Method::GET, &url, Some(&alice), None).await;
    assert!(detail.body.contains("$59.99"));
    assert_eq!(
        send(&router, Method::GET, &url, Some(&bob), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let refreshed = cart_page(&router, &alice).await;
    assert!(refreshed.body.contains("$59.99"));
    assert_eq!(db.stock("rust-book").await, 2);
    assert_eq!(db.test_cache().test_source_loads(), 2);
}
