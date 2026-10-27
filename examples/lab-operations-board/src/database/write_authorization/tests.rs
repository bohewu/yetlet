use super::*;
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
async fn pair(path: &std::path::Path) -> (Database, Database) {
    let a = Database::open_with_write_authorization(path).await.unwrap();
    let b = Database::open_with_write_authorization(path).await.unwrap();
    b.set_write_enabled(alice(), true).await.unwrap();
    (a, b)
}
async fn counts(db: &Database) -> (i64, i64) {
    db.run(|c| {
        Ok((
            c.query_row("SELECT COUNT(*) FROM work_items", [], |r| r.get(0))?,
            c.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))?,
        ))
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn commit_write_normal_atomicity_and_no_missing_ticket_bypass() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open_with_write_authorization(&dir.path().join("strict.sqlite"))
        .await
        .unwrap();
    let item = db.save(alice(), None, 0, fields("Initial")).await.unwrap();
    assert_eq!(item.version, 1);
    assert_eq!(counts(&db).await, (1, 1));
    assert!(matches!(
        db.save_checked(alice(), Some(item.id), 1, fields("Bypass"), None)
            .await,
        Err(Failure::Forbidden)
    ));
    db.run(|c| {c.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON history BEGIN SELECT RAISE(ABORT,'fixture'); END;")?;Ok(())}).await.unwrap();
    assert!(matches!(
        db.save(alice(), Some(item.id), 1, fields("Rollback")).await,
        Err(Failure::Unavailable)
    ));
    assert_eq!(db.get(1, item.id).await.unwrap().0.version, 1);
    assert_eq!(counts(&db).await, (1, 1));
    db.drain().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn commit_write_paused_ticket_revocation_and_reenable_aba() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = pair(&dir.path().join("shared.sqlite")).await;
    let before = a.save(alice(), None, 0, fields("Original")).await.unwrap();
    let (issued, ready) = tokio::sync::oneshot::channel();
    let (release, resume) = tokio::sync::oneshot::channel();
    let worker = a.clone();
    let paused = tokio::spawn(async move {
        let ticket = worker.authorize_write(alice()).await.unwrap();
        issued.send(()).unwrap();
        resume.await.unwrap();
        worker
            .save_checked(alice(), Some(before.id), 1, fields("Stale"), Some(ticket))
            .await
    });
    ready.await.unwrap();
    b.set_write_enabled(alice(), false).await.unwrap();
    assert!(matches!(
        b.authorize_write(alice()).await,
        Err(Failure::Forbidden)
    ));
    // A distinct explicit re-enable must not make the paused generation valid.
    b.set_write_enabled(alice(), true).await.unwrap();
    release.send(()).unwrap();
    assert!(matches!(paused.await.unwrap(), Err(Failure::Forbidden)));
    assert_eq!(a.get(1, 1).await.unwrap().0.version, 1);
    assert_eq!(counts(&a).await, (1, 1));
    assert_eq!(
        a.save(alice(), Some(1), 1, fields("Fresh"))
            .await
            .unwrap()
            .version,
        2
    );
    // A ticket is also bound to its Store lifetime, even at the same generation.
    let ticket = a.authorize_write(alice()).await.unwrap();
    assert!(matches!(
        b.save_checked(alice(), Some(1), 2, fields("Wrong Store"), Some(ticket))
            .await,
        Err(Failure::Forbidden)
    ));
    a.drain().await.unwrap();
    b.drain().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn commit_write_held_transaction_prevents_premature_revoke_ack() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = pair(&dir.path().join("held.sqlite")).await;
    let ticket = a.authorize_write(alice()).await.unwrap();
    let brand = a.write_authority.clone().unwrap();
    let (entered, ready) = tokio::sync::oneshot::channel();
    let (release, resume) = std::sync::mpsc::channel();
    let worker = a.clone();
    let saving = tokio::spawn(async move {
        worker
            .run(move |c| {
                let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
                check(&tx, Some(&brand), Some(ticket), alice())?;
                let value = apply_save(&tx, alice(), None, 0, fields("Earlier writer"))?;
                entered.send(()).unwrap();
                resume
                    .recv_timeout(Duration::from_secs(3))
                    .map_err(|_| Failure::Unavailable)?;
                tx.commit()?;
                Ok(value)
            })
            .await
    });
    ready.await.unwrap();
    assert!(matches!(
        b.set_write_enabled(alice(), false).await,
        Err(Failure::Busy)
    ));
    release.send(()).unwrap();
    assert_eq!(saving.await.unwrap().unwrap().version, 1);
    // This is a new explicit control action after the earlier effect completed.
    b.set_write_enabled(alice(), false).await.unwrap();
    assert_eq!(counts(&b).await, (1, 1));
    assert!(matches!(
        a.save(alice(), Some(1), 1, fields("After revoke")).await,
        Err(Failure::Forbidden)
    ));
    a.drain().await.unwrap();
    b.drain().await.unwrap();
}

#[tokio::test]
async fn commit_write_generation_scope_and_authority_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open_with_write_authorization(&dir.path().join("negative.sqlite"))
        .await
        .unwrap();
    for p in [
        Principal {
            name: "bob",
            project: 2,
            write: true,
        },
        Principal {
            name: "viewer",
            project: 1,
            write: false,
        },
        Principal {
            name: "alice",
            project: 2,
            write: true,
        },
    ] {
        assert!(matches!(
            db.save(p, None, 0, fields("Denied")).await,
            Err(Failure::Forbidden | Failure::Invalid)
        ));
    }
    db.run(|c| {
        c.execute("UPDATE write_authority SET generation=?1", [i64::MAX - 1])?;
        Ok(())
    })
    .await
    .unwrap();
    let ticket = db.authorize_write(alice()).await.unwrap();
    db.set_write_enabled(alice(), false).await.unwrap();
    db.set_write_enabled(alice(), false).await.unwrap();
    assert!(matches!(
        db.set_write_enabled(alice(), true).await,
        Err(Failure::Forbidden)
    ));
    assert!(matches!(
        db.save_checked(alice(), None, 0, fields("Exhausted"), Some(ticket))
            .await,
        Err(Failure::Forbidden)
    ));
    db.run(|c| {
        c.execute("DELETE FROM write_authority", [])?;
        Ok(())
    })
    .await
    .unwrap();
    assert!(matches!(
        db.authorize_write(alice()).await,
        Err(Failure::Forbidden)
    ));
    db.run(|c| {
        c.execute_batch("DROP TABLE write_authority")?;
        Ok(())
    })
    .await
    .unwrap();
    assert!(matches!(db.ready().await, Err(Failure::Schema)));
    assert!(matches!(
        db.save(alice(), None, 0, fields("Missing authority")).await,
        Err(Failure::Schema)
    ));
    assert_eq!(counts(&db).await, (0, 0));
    db.drain().await.unwrap();
}

#[tokio::test]
async fn commit_write_reopen_denies_without_reseeding_and_legacy_refuses() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reopen.sqlite");
    let a = Database::open_with_write_authorization(&path)
        .await
        .unwrap();
    let ticket = a.authorize_write(alice()).await.unwrap();
    a.save(alice(), None, 0, fields("Persisted")).await.unwrap();
    a.drain().await.unwrap();
    drop(a);
    assert!(matches!(Database::open(&path).await, Err(Failure::Schema)));
    let b = Database::open_with_write_authorization(&path)
        .await
        .unwrap();
    assert!(matches!(
        b.save(alice(), Some(1), 1, fields("Unconfirmed reopen"))
            .await,
        Err(Failure::Forbidden)
    ));
    b.set_write_enabled(alice(), true).await.unwrap();
    assert!(matches!(
        b.save_checked(alice(), Some(1), 1, fields("Old lifetime"), Some(ticket))
            .await,
        Err(Failure::Forbidden)
    ));
    assert_eq!(
        b.save(alice(), Some(1), 1, fields("Explicitly reallowed"))
            .await
            .unwrap()
            .version,
        2
    );
    b.drain().await.unwrap();
    drop(b);
    let legacy_path = dir.path().join("v1.sqlite");
    let legacy = Database::open(&legacy_path).await.unwrap();
    legacy.drain().await.unwrap();
    drop(legacy);
    assert!(matches!(
        Database::open_with_write_authorization(&legacy_path).await,
        Err(Failure::Schema)
    ));
    let empty = dir.path().join("existing-empty.sqlite");
    std::fs::write(&empty, []).unwrap();
    assert!(matches!(
        Database::open_with_write_authorization(&empty).await,
        Err(Failure::Schema)
    ));
}
