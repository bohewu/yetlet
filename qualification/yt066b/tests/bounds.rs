use std::time::Duration;
use yt066b_redis_spike::{Error, State, Store, random_id};

fn url() -> String {
    std::env::var("YT_REDIS_URL").expect("set the disposable Redis fixture URL")
}
fn command(name: &str, args: &[&str]) -> redis::Cmd {
    let mut c = redis::cmd(name);
    c.arg(args);
    c
}
async fn wait_blocked(observer: &Store) {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let info: String = redis::from_redis_value(
                observer
                    .command(command("INFO", &["clients"]))
                    .await
                    .unwrap(),
            )
            .unwrap();
            if info.lines().any(|l| l.trim() == "blocked_clients:1") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the owned blocking command reached Redis");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the disposable Redis fixture"]
async fn cancelled_waiter_keeps_capacity_until_reply_and_shutdown_reaps() {
    let ns = random_id().unwrap();
    let a = Store::connect(&url(), &ns, 1).await.unwrap();
    let observer = Store::connect(&url(), &ns, 1).await.unwrap();
    let a2 = a.clone();
    let key = a.record_key(&random_id().unwrap()).unwrap();
    let blocked = tokio::spawn(async move { a2.command(command("BLPOP", &[&key, "1"])).await });
    wait_blocked(&observer).await;
    blocked.abort();
    assert!(blocked.await.unwrap_err().is_cancelled());
    assert!(a.running() == 1);
    let (one, two, three) = tokio::join!(
        a.command(redis::cmd("PING")),
        a.command(redis::cmd("PING")),
        a.command(redis::cmd("PING"))
    );
    let results = [one, two, three];
    assert!(
        results
            .iter()
            .filter(|x| matches!(x, Err(Error::Busy)))
            .count()
            == 2
    );
    assert!(
        results
            .iter()
            .filter(|x| matches!(x, Err(Error::QueueTimeout)))
            .count()
            == 1
    );
    tokio::time::timeout(Duration::from_secs(2), async {
        while a.running() != 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    a.command(redis::cmd("PING")).await.unwrap();
    assert!(a.peak() == 1);
    let key = a.record_key(&random_id().unwrap()).unwrap();
    // The call times out; its worker remains owned until the bounded shutdown reaps it.
    assert!(matches!(
        a.command(command("BLPOP", &[&key, "60"])).await,
        Err(Error::Unavailable)
    ));
    assert!(a.running() == 1);
    assert!(a.shutdown().await.unwrap() == 1 && a.running() == 0);
    assert!(a.command(redis::cmd("PING")).await.is_err());
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let info: String = redis::from_redis_value(
                observer
                    .command(command("INFO", &["clients"]))
                    .await
                    .unwrap(),
            )
            .unwrap();
            if info.lines().any(|l| l.trim() == "blocked_clients:0") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("reaped redis-rs driver closed the blocked connection");
    observer.shutdown().await.unwrap();
}

#[tokio::test]
#[ignore = "requires the disposable Redis fixture"]
async fn disconnect_reconnect_manager_restart_and_refused_connection() {
    let ns = random_id().unwrap();
    let a = Store::connect(&url(), &ns, 2).await.unwrap();
    let observer = Store::connect(&url(), &ns, 2).await.unwrap();
    let ticket = a
        .prepare("create", None, 0, &State::new(9), 60000)
        .await
        .unwrap();
    assert!(a.execute(&ticket).await.unwrap().status == "ok");
    let id: i64 =
        redis::from_redis_value(a.command(command("CLIENT", &["ID"])).await.unwrap()).unwrap();
    observer
        .command(command("CLIENT", &["KILL", "ID", &id.to_string()]))
        .await
        .unwrap();
    // redis-rs returns the disconnect and schedules reconnection; it does not replay the command.
    let lost = a.read(ticket.identity()).await;
    assert!(matches!(lost, Err(Error::Unavailable)));
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(snapshot) = a.read(ticket.identity()).await {
                assert!(snapshot.state.count == 9);
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    a.shutdown().await.unwrap();
    let restarted = Store::connect(&url(), &ns, 2).await.unwrap();
    assert!(restarted.read(ticket.identity()).await.unwrap().state.count == 9);
    restarted.shutdown().await.unwrap();
    observer.shutdown().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let unavailable = Store::connect(&format!("redis://127.0.0.1:{port}/"), &ns, 1).await;
    assert!(matches!(unavailable, Err(Error::Unavailable)));
}
