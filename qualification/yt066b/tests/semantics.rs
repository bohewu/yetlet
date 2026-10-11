use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use yt066b_redis_spike::{Error, Mutation, State, Store, Ticket, UpdateError, random_id};

fn url() -> String {
    std::env::var("YT_REDIS_URL").expect("set the disposable Redis fixture URL")
}
async fn store(namespace: &str) -> Store {
    Store::connect(&url(), namespace, 2)
        .await
        .expect("fixture connection")
}
async fn create(a: &Store, count: u64, ttl: u64) -> Ticket {
    let t = a
        .prepare("create", None, 0, &State::new(count), ttl)
        .await
        .expect("prepare create");
    assert!(a.execute(&t).await.expect("create").status == "ok");
    t
}
async fn ok(a: &Store, ticket: &Ticket) -> Mutation {
    let r = a.execute(ticket).await.expect("mutation acknowledgement");
    assert!(r.status == "ok");
    r
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn corrupt_integer_metadata_is_rejected_before_mutation() {
    let a = store(&random_id().unwrap()).await;
    for field in ["issuance", "revision", "generation", "expires"] {
        let initial = create(&a, 7, 60000).await;
        let ticket = a
            .prepare("rotate", Some(initial.identity()), 0, &State::new(0), 60000)
            .await
            .unwrap();
        let key = a.record_key(initial.identity()).unwrap();
        a.command(command("HSET", &[&key, field, "1.5"]))
            .await
            .unwrap();
        assert!(a.read(initial.identity()).await.is_err());
        assert!(a.execute(&ticket).await.unwrap().status == "invalid");
        let replacement = a.record_key(ticket.replacement().unwrap()).unwrap();
        let exists: u8 =
            redis::from_redis_value(a.command(command("EXISTS", &[&replacement])).await.unwrap())
                .unwrap();
        assert!(exists == 0);
        let status: String =
            redis::from_redis_value(a.command(command("HGET", &[&key, "status"])).await.unwrap())
                .unwrap();
        assert!(status == "active");
    }
    a.shutdown().await.unwrap();
}
fn command(name: &str, args: &[&str]) -> redis::Cmd {
    let mut c = redis::cmd(name);
    c.arg(args);
    c
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn atomic_lifecycle_and_independent_results() {
    let ns = random_id().unwrap();
    let a = store(&ns).await;
    let b = store(&ns).await;
    let c = store(&ns).await;
    let first = create(&a, 0, 60000).await;
    assert!(b.read(first.identity()).await.unwrap().state.count == 0);
    assert!(c.read(first.identity()).await.unwrap().state.count == 0);
    let before = a.read(first.identity()).await.unwrap();
    let x = a
        .prepare("update", Some(first.identity()), 0, &State::new(1), 2500)
        .await
        .unwrap();
    let y = b
        .prepare("update", Some(first.identity()), 0, &State::new(10), 2500)
        .await
        .unwrap();
    let (rx, ry) = tokio::join!(a.execute(&x), b.execute(&y));
    let (rx, ry) = (rx.unwrap(), ry.unwrap());
    assert!(
        (rx.status == "ok" && ry.status == "conflict")
            || (ry.status == "ok" && rx.status == "conflict")
    );
    let winner = if rx.status == "ok" { 1 } else { 10 };
    let latest = c.read(first.identity()).await.unwrap();
    assert!(
        latest.state.count == winner && latest.revision == 1 && latest.expires == before.expires
    );
    assert!(a.resolve(&x).await.unwrap() == rx && c.resolve(&y).await.unwrap() == ry);

    // Updating versus rotation has one ordering boundary, with no successful update lost.
    let update = a
        .prepare("update", Some(first.identity()), 1, &State::new(21), 2500)
        .await
        .unwrap();
    let rotate = b
        .prepare("rotate", Some(first.identity()), 1, &State::new(999), 60000)
        .await
        .unwrap();
    let (u, r) = tokio::join!(a.execute(&update), b.execute(&rotate));
    let (u, r) = (u.unwrap(), r.unwrap());
    let rotated = if u.status == "ok" {
        assert!(r.status == "conflict");
        let fresh = b
            .prepare("rotate", Some(first.identity()), 2, &State::new(999), 60000)
            .await
            .unwrap();
        ok(&b, &fresh).await;
        fresh
    } else {
        assert!(matches!(u.status.as_str(), "rotated" | "missing") && r.status == "ok");
        rotate
    };
    let id = rotated.replacement().unwrap();
    let replacement = c.read(id).await.unwrap();
    assert!(replacement.state.count == if u.status == "ok" { 21 } else { winner });
    assert!(replacement.csrf != before.csrf && replacement.generation == 1);
    for s in [&a, &b, &c] {
        assert!(s.read(first.identity()).await.is_err());
    }
    assert!(a.resolve(&x).await.unwrap() == rx); // Evidence survives both later update and rotation.
    let revoke = b
        .prepare("revoke", Some(id), 0, &State::new(0), 2500)
        .await
        .unwrap();
    ok(&b, &revoke).await;
    for s in [&a, &b, &c] {
        assert!(s.read(id).await.is_err());
    }
    assert!(
        a.resolve(&x).await.unwrap() == rx && c.resolve(&rotated).await.unwrap().status == "ok"
    );
    // An exact acknowledged create repeat returns historical status, never revives the old ID.
    assert!(a.execute(&first).await.unwrap().status == "ok");
    assert!(a.read(first.identity()).await.is_err());
    let tombstone: Vec<String> = redis::from_redis_value(
        a.command(command(
            "HKEYS",
            &[&a.record_key(first.identity()).unwrap()],
        ))
        .await
        .unwrap(),
    )
    .unwrap();
    assert!(
        !tombstone
            .iter()
            .any(|s| matches!(s.as_str(), "state" | "csrf" | "replacement"))
    );
    // Two concurrent rotations: exactly one winner, no second active replacement.
    let pair = create(&a, 7, 60000).await;
    let r1 = a
        .prepare("rotate", Some(pair.identity()), 0, &State::new(0), 60000)
        .await
        .unwrap();
    let r2 = b
        .prepare("rotate", Some(pair.identity()), 0, &State::new(0), 60000)
        .await
        .unwrap();
    let (r1v, r2v) = tokio::join!(a.execute(&r1), b.execute(&r2));
    assert!((r1v.unwrap().status == "ok") != (r2v.unwrap().status == "ok"));
    assert!(
        c.read(r1.replacement().unwrap()).await.is_ok()
            != c.read(r2.replacement().unwrap()).await.is_ok()
    );
    for s in [&a, &b, &c] {
        s.shutdown().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn lost_reply_after_commit_and_callback_values() {
    let ns = random_id().unwrap();
    let a = store(&ns).await;
    let b = store(&ns).await;
    let first = create(&a, 0, 60000).await;
    let t = a
        .prepare("update", Some(first.identity()), 0, &State::new(1), 2500)
        .await
        .unwrap();
    // This is a real server-side reply suppression, not a simulated Redis result.
    assert!(matches!(
        a.command(command("CLIENT", &["REPLY", "OFF"])).await,
        Err(Error::Unavailable)
    ));
    let unknown = a.execute(&t).await;
    assert!(matches!(unknown, Err(Error::OutcomeUnknown(_))));
    assert!(a.running() == 2); // Both reply-less workers still hold their admission slots.
    assert!(b.read(first.identity()).await.unwrap().state.count == 1);
    let next = b
        .prepare("update", Some(first.identity()), 1, &State::new(2), 2500)
        .await
        .unwrap();
    ok(&b, &next).await;
    let rotate = b
        .prepare("rotate", Some(first.identity()), 2, &State::new(0), 60000)
        .await
        .unwrap();
    ok(&b, &rotate).await;
    let revoke = b
        .prepare("revoke", rotate.replacement(), 0, &State::new(0), 2500)
        .await
        .unwrap();
    ok(&b, &revoke).await;
    assert!(b.resolve(&t).await.unwrap().status == "ok");
    assert!(b.read(first.identity()).await.is_err());
    assert!(a.shutdown().await.unwrap() == 2 && a.running() == 0);

    // R has no Clone/Serialize implementation and its drop is observable.
    struct Value(Arc<AtomicUsize>);
    impl Drop for Value {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let fresh = create(&b, 0, 60000).await;
    let calls = AtomicUsize::new(0);
    let drops = Arc::new(AtomicUsize::new(0));
    let result: Result<Value, UpdateError<&str>> = b
        .try_update(fresh.identity(), |state| {
            calls.fetch_add(1, Ordering::SeqCst);
            state.count += 1;
            Ok(Value(drops.clone()))
        })
        .await;
    assert!(
        result.is_ok() && calls.load(Ordering::SeqCst) == 1 && drops.load(Ordering::SeqCst) == 0
    );
    drop(result);
    let rejected: Result<(), UpdateError<&str>> = b
        .try_update(fresh.identity(), |state| {
            state.count = 999;
            Err("application rejection")
        })
        .await;
    assert!(matches!(rejected, Err(UpdateError::Application(_))));
    assert!(b.read(fresh.identity()).await.unwrap().state.count == 1);
    // Read-only Lua and TIME still execute during WRITE pause. The one mutation is ambiguous.
    b.command(command("CLIENT", &["PAUSE", "700", "WRITE"]))
        .await
        .unwrap();
    let unknown: Result<Value, UpdateError<&str>> = b
        .try_update(fresh.identity(), |state| {
            calls.fetch_add(1, Ordering::SeqCst);
            state.count += 1;
            Ok(Value(drops.clone()))
        })
        .await;
    let ticket = match unknown {
        Err(UpdateError::Store(Error::OutcomeUnknown(t))) => t,
        _ => panic!("expected unknown mutation"),
    };
    assert!(calls.load(Ordering::SeqCst) == 2 && drops.load(Ordering::SeqCst) == 2);
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(b.resolve(&ticket).await.unwrap().status == "ok");
    assert!(
        b.read(fresh.identity()).await.unwrap().state.count == 2
            && calls.load(Ordering::SeqCst) == 2
    );
    b.shutdown().await.unwrap();
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn expiry_corruption_payload_and_admission() {
    let ns = random_id().unwrap();
    let a = store(&ns).await;
    let expiring = create(&a, 1, 100).await;
    tokio::time::sleep(Duration::from_millis(160)).await;
    assert!(a.read(expiring.identity()).await.is_err());
    assert!(a.resolve(&expiring).await.unwrap().status == "ok");
    tokio::time::sleep(Duration::from_millis(4900)).await;
    assert!(a.resolve(&expiring).await.unwrap().status == "unresolved");
    assert!(a.execute(&expiring).await.unwrap().status == "deadline");
    assert!(a.read(expiring.identity()).await.is_err());

    for (field, value) in [
        ("schema", "99"),
        ("state", "invalid-json"),
        ("state", &"x".repeat(1025)),
    ] {
        let t = create(&a, 1, 60000).await;
        let key = a.record_key(t.identity()).unwrap();
        a.command(command("HSET", &[&key, field, value]))
            .await
            .unwrap();
        assert!(matches!(
            a.read(t.identity()).await,
            Err(Error::InvalidRecord | Error::Codec)
        ));
    }
    let large = State {
        count: 0,
        note: "x".repeat(1025),
    };
    assert!(matches!(
        a.prepare("create", None, 0, &large, 2500).await,
        Err(Error::PayloadTooLarge)
    ));
    assert!(
        a.prepare("create", Some(expiring.identity()), 0, &State::new(1), 2500)
            .await
            .is_err()
    );
    // New namespace: exactly 64 retained operation outcomes, independent of record TTL.
    let bounded = store(&random_id().unwrap()).await;
    for _ in 0..64 {
        create(&bounded, 0, 100).await;
    }
    let excess = bounded
        .prepare("create", None, 0, &State::new(0), 100)
        .await
        .unwrap();
    assert!(bounded.execute(&excess).await.unwrap().status == "capacity");
    assert!(bounded.resolve(&excess).await.unwrap().status == "unresolved");
    a.shutdown().await.unwrap();
    bounded.shutdown().await.unwrap();
}
