use super::*;
use crate::UserId;
use std::collections::BTreeMap;

fn fixture() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open_with_options(
        dir.path().join("cache.sqlite"),
        crate::DatabaseOptions {
            catalog_cache: true,
        },
    )
    .unwrap();
    (dir, db)
}
fn line(id: &str, quantity: u8) -> BTreeMap<String, u8> {
    [(id.into(), quantity)].into()
}

#[tokio::test]
async fn catalog_cache_source_age_expiry_and_store_isolation() {
    let (dir, db) = fixture();
    let initial = db.catalog().await.unwrap();
    let cache = db.test_cache();
    let old = cache.values.get(&0).await.unwrap();
    db.test_connection()
        .execute("UPDATE products SET stock=3 WHERE id='rust-book'", [])
        .unwrap();
    assert_eq!(db.catalog().await.unwrap(), initial);
    assert_eq!(cache.test_source_loads(), 1);
    assert_eq!(cache.values.get(&0).await.unwrap().started, old.started);
    // Native insertion is new, but source age is expired: insertion TTL is insufficient.
    cache
        .values
        .insert(
            0,
            Arc::new(Entry {
                started: Instant::now() - MAX_AGE,
                products: initial,
            }),
        )
        .await;
    assert_eq!(
        db.catalog()
            .await
            .unwrap()
            .iter()
            .find(|p| p.id == "rust-book")
            .unwrap()
            .stock,
        3
    );
    assert_eq!(cache.test_source_loads(), 2);
    let separate = Database::open_with_options(
        dir.path().join("other.sqlite"),
        crate::DatabaseOptions {
            catalog_cache: true,
        },
    )
    .unwrap();
    separate
        .test_connection()
        .execute(
            "UPDATE products SET name='Other store' WHERE id='rust-book'",
            [],
        )
        .unwrap();
    assert!(
        separate
            .catalog()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name == "Other store")
    );
    assert!(
        !db.catalog()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name == "Other store")
    );
    let reopened = Database::open_with_options(
        db.path(),
        crate::DatabaseOptions {
            catalog_cache: true,
        },
    )
    .unwrap();
    assert_eq!(reopened.test_cache().test_source_loads(), 0);
    assert_eq!(
        reopened.catalog().await.unwrap(),
        db.catalog().await.unwrap()
    );
    let uncached = Database::open(dir.path().join("off.sqlite")).unwrap();
    assert!(!uncached.catalog_cache_enabled());
}

#[tokio::test]
async fn catalog_cache_commit_rollback_late_fill_and_generation_saturation() {
    let (_dir, db) = fixture();
    let cache = db.test_cache();
    db.catalog().await.unwrap();
    let late = cache.values.get(&0).await.unwrap();
    db.place_order(
        UserId::Alice,
        line("rust-book", 1),
        "cache-order-one".into(),
    )
    .await
    .unwrap();
    assert_eq!(cache.generation.load(Ordering::Acquire), 1);
    // Deterministically model an old in-flight fill publishing after commit.
    cache.values.insert(0, late).await;
    assert_eq!(
        db.catalog()
            .await
            .unwrap()
            .iter()
            .find(|p| p.id == "rust-book")
            .unwrap()
            .stock,
        7
    );
    assert_eq!(cache.test_source_loads(), 2);
    let g = cache.generation.load(Ordering::Acquire);
    assert!(matches!(
        db.place_order(UserId::Alice, line("keyboard", 3), "cache-reject".into())
            .await,
        Err(DatabaseError::OutOfStock)
    ));
    db.test_connection().execute_batch("CREATE TRIGGER cache_line_failure BEFORE INSERT ON order_lines BEGIN SELECT RAISE(ABORT,'cache rollback fixture'); END;").unwrap();
    assert!(
        db.place_order(UserId::Alice, line("rust-book", 1), "cache-rollback".into())
            .await
            .is_err()
    );
    assert_eq!(cache.generation.load(Ordering::Acquire), g);
    assert_eq!(db.stock("rust-book").await, 7);
    assert_eq!(db.order_count().await, 1);
    db.test_connection()
        .execute_batch("DROP TRIGGER cache_line_failure;")
        .unwrap();
    cache.generation.store(u64::MAX - 1, Ordering::Release);
    cache.committed();
    cache.committed();
    assert_eq!(cache.generation.load(Ordering::Acquire), u64::MAX);
    assert!(matches!(
        db.catalog().await,
        Err(DatabaseError::WorkerUnavailable)
    ));
    db.place_order(
        UserId::Alice,
        line("rust-book", 1),
        "cache-order-two".into(),
    )
    .await
    .unwrap();
    assert_eq!(db.stock("rust-book").await, 6);
}

#[tokio::test]
async fn catalog_cache_oversize_bypass_source_failure_and_recovery() {
    let (_dir, db) = fixture();
    let cache = db.test_cache();
    db.test_connection()
        .execute(
            "UPDATE products SET name=?1 WHERE id='rust-book'",
            ["x".repeat(MAX_PAYLOAD + 1)],
        )
        .unwrap();
    assert!(
        db.catalog()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name.len() > MAX_PAYLOAD)
    );
    assert_eq!(db.catalog().await.unwrap().len(), 3);
    assert_eq!(cache.test_source_loads(), 2);
    assert!(cache.values.get(&0).await.is_none());
    db.test_connection()
        .execute(
            "UPDATE products SET name='Restored product' WHERE id='rust-book'",
            [],
        )
        .unwrap();
    db.test_connection()
        .execute_batch("ALTER TABLE products RENAME TO cache_unavailable_products;")
        .unwrap();
    assert!(db.catalog().await.is_err());
    assert!(cache.values.get(&0).await.is_none());
    db.test_connection()
        .execute_batch("ALTER TABLE cache_unavailable_products RENAME TO products;")
        .unwrap();
    assert!(
        db.catalog()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name == "Restored product")
    );
    assert_eq!(cache.test_source_loads(), 4);
}

#[tokio::test]
async fn catalog_cache_concurrent_fill_and_bounded_wait() {
    let (_dir, db) = fixture();
    let cache = db.test_cache();
    let blocked = db.test_sql_gate().acquire_many_owned(4).await.unwrap();
    let a_db = db.clone();
    let a = tokio::spawn(async move { a_db.catalog().await });
    let b_db = db.clone();
    let b = tokio::spawn(async move { b_db.catalog().await });
    tokio::time::timeout(Duration::from_secs(1), async {
        while cache.test_source_loads() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    tokio::task::yield_now().await;
    assert_eq!(cache.test_source_loads(), 1);
    drop(blocked);
    assert_eq!(a.await.unwrap().unwrap(), b.await.unwrap().unwrap());
    assert_eq!(cache.test_source_loads(), 1);
    cache.committed();
    let blocked = db.test_sql_gate().acquire_many_owned(4).await.unwrap();
    let start = Instant::now();
    assert!(matches!(
        db.catalog().await,
        Err(DatabaseError::WorkerUnavailable)
    ));
    assert!(start.elapsed() >= WAIT && start.elapsed() < Duration::from_secs(2));
    assert_eq!(cache.fills.available_permits(), 1);
    drop(blocked);
    assert_eq!(db.catalog().await.unwrap().len(), 3);
    assert_eq!(cache.test_source_loads(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn catalog_cache_cancelled_started_worker_retains_fill_admission() {
    let (_dir, db) = fixture();
    let cache = db.test_cache();
    let (entered, started) = tokio::sync::oneshot::channel();
    let (release, resume) = std::sync::mpsc::channel();
    *cache.source_barrier.lock().unwrap() = Some((entered, resume));
    let copy = db.clone();
    let task = tokio::spawn(async move { copy.catalog().await });
    tokio::time::timeout(Duration::from_secs(1), started)
        .await
        .unwrap()
        .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(cache.fills.available_permits(), 0);
    assert!(matches!(
        db.catalog().await,
        Err(DatabaseError::WorkerUnavailable)
    ));
    assert_eq!(cache.test_source_loads(), 1);
    release.send(()).unwrap();
    // Same existing SQL gate: all permits means the real worker returned.
    let drained = tokio::time::timeout(
        Duration::from_secs(2),
        db.test_sql_gate().acquire_many_owned(4),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(cache.fills.available_permits(), 1);
    drop(drained);
    assert_eq!(db.catalog().await.unwrap().len(), 3);
    assert_eq!(cache.test_source_loads(), 2);
}
