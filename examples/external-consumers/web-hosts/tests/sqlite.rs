//! Real SQLite ownership through the public Web/Host resource contracts.

#[path = "support/sqlite_resource.rs"]
mod sqlite_resource;

use rusqlite::Connection;
use sqlite_resource::{Gate, OpenOnDrop, SqliteResource, SqliteStatus};
use std::{cell::Cell, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Notify,
    time::{sleep, timeout},
};
use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{ExclusiveClaim, HostOperation, JoinStatus, Managed, Retained},
    web::{App, WebActivity, WebError, WebHandle, WebHost, http::StatusCode},
};

const LIMIT: Duration = Duration::from_secs(5);

struct Earlier<'a> {
    database: SqliteStatus,
    dropped: &'a Cell<Option<bool>>,
}
impl Drop for Earlier<'_> {
    fn drop(&mut self) {
        self.dropped.set(Some(self.database.joined()));
    }
}

struct DatabaseHost<'a> {
    web: WebHost,
    database: SqliteResource,
    earlier_dropped: &'a Cell<Option<bool>>,
}
impl<'a> Host for DatabaseHost<'a> {
    type Error = WebError;
    type Activity = WebActivity;
    type Resources = Managed<SqliteResource, Retained<Earlier<'a>, ()>>;

    fn name(&self) -> &'static str {
        self.web.name()
    }
    fn validate(&self) -> Result<(), Self::Error> {
        self.web.validate()
    }
    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        self.web.exclusive_claims()
    }
    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        let earlier = Earlier {
            database: self.database.status(),
            dropped: self.earlier_dropped,
        };
        self.web
            .into_resources()
            .with_resource(earlier)
            .with_managed_resource("sqlite", self.database)
    }
}

struct UnblockRequest(Arc<Notify>);
impl Drop for UnblockRequest {
    fn drop(&mut self) {
        self.0.notify_one();
    }
}

async fn wait_until(mut predicate: impl FnMut() -> bool) {
    timeout(LIMIT, async {
        while !predicate() {
            sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("lifecycle progress deadline");
}

async fn admitted(handle: &WebHandle) -> SocketAddr {
    wait_until(|| handle.is_admitting()).await;
    handle.local_addr().expect("admitted address")
}

async fn request(address: SocketAddr, path: &str) -> io::Result<String> {
    timeout(LIMIT, async {
        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .await?;
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "HTTP request"))?
}

fn rows(path: &std::path::Path) -> Vec<String> {
    let connection = Connection::open(path).unwrap();
    let mut statement = connection
        .prepare("SELECT value FROM records ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_drives_keep_the_original_database_through_acquisition_drain_and_release() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("drain.sqlite");
    let start = Gate::new(false);
    let close = Gate::new(false);
    let _open_start = OpenOnDrop(start.clone());
    let _open_close = OpenOnDrop(close.clone());
    let resource = SqliteResource::new(&path, start.clone(), close.clone());
    let database = resource.status();
    let client = resource.client();
    let handler_client = client.clone();
    let entered = Arc::new(Notify::new());
    let release_request = Arc::new(Notify::new());
    let handler_entered = entered.clone();
    let handler_release = release_request.clone();
    let _unblock = UnblockRequest(release_request.clone());
    let web = App::new()
        .get("/slow", move || {
            let client = handler_client.clone();
            let entered = handler_entered.clone();
            let release = handler_release.clone();
            async move {
                client
                    .insert("before stop")
                    .await
                    .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))?;
                entered.notify_one();
                release.notified().await;
                client
                    .insert("after stop")
                    .await
                    .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))?;
                Ok::<_, (StatusCode, String)>("drained")
            }
        })
        .into_host()
        .with_name("database API")
        .with_address("127.0.0.1:0");
    let web_handle = web.handle();
    let earlier_dropped = Cell::new(None);
    let mut hosts = HostGroup::new()
        .with_host(DatabaseHost {
            web,
            database: resource,
            earlier_dropped: &earlier_dropped,
        })
        .build()
        .unwrap();
    assert!(!path.exists());
    assert!(database.thread_id().is_none() && web_handle.local_addr().is_none());
    let timer_progress = Cell::new(false);
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("acquisition unexpectedly completed: {result:?}"),
            () = async {
                wait_until(|| database.thread_id().is_some()).await;
                sleep(Duration::from_millis(2)).await;
                timer_progress.set(true);
            } => {}
        }
    })
    .await
    .unwrap();
    // The actor and readiness receiver stay in the resource after drive cancellation.
    let original = database.thread_id().unwrap();
    assert!(timer_progress.get());
    assert!(!path.exists() && !client.available());
    assert_eq!(earlier_dropped.get(), None);
    start.open();

    let request_handle = web_handle.clone();
    let mut request = Box::pin(async move {
        request(admitted(&request_handle).await, "/slow")
            .await
            .unwrap()
    });
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("running lifetime ended early: {result:?}"),
            () = async {
                tokio::select! {
                    response = &mut request => panic!("request drained before release: {response}"),
                    () = entered.notified() => {}
                }
            } => {}
        }
    })
    .await
    .unwrap();
    assert_eq!(database.thread_id(), Some(original));
    assert!(database.is_open() && client.available());
    hosts.request_shutdown();
    assert!(!web_handle.is_admitting());

    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("admitted database work was lost: {result:?}"),
            () = async { sleep(Duration::from_millis(2)).await; } => {}
        }
    })
    .await
    .unwrap();
    assert!(!database.release_started() && !database.joined());
    assert!(client.available());
    assert_eq!(earlier_dropped.get(), None);

    release_request.notify_one();
    timeout(LIMIT, async {
        tokio::select! {
            result = hosts.run_async() => panic!("database finalizer gate was bypassed: {result:?}"),
            () = async {
                let response = (&mut request).await;
                assert!(response.starts_with("HTTP/1.1 200") && response.ends_with("drained"));
                wait_until(|| database.release_started()).await;
            } => {}
        }
    }).await.unwrap();
    // Web has drained, but the original database thread still owns its connection.
    assert_eq!(hosts.outcomes()[0].join, JoinStatus::Succeeded);
    assert!(!client.available() && database.is_open() && !database.joined());
    assert_eq!(database.thread_id(), Some(original));
    assert_eq!(earlier_dropped.get(), None);

    close.open();
    timeout(LIMIT, hosts.run_async()).await.unwrap().unwrap();
    assert!(database.joined() && !database.is_open());
    assert_eq!(database.thread_id(), Some(original));
    assert_eq!(earlier_dropped.get(), Some(true));
    assert_eq!(rows(&path), ["before stop", "after stop"]);
}

#[tokio::test(flavor = "current_thread")]
async fn failed_database_readiness_retains_its_original_cause_and_actor_until_join() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invalid.sqlite");
    std::fs::write(&path, b"not a SQLite database").unwrap();
    let resource = SqliteResource::new(&path, Gate::new(true), Gate::new(true));
    let database = resource.status();
    let web = App::new()
        .get("/api", || async { "unused" })
        .into_host()
        .with_name("database API")
        .with_address("127.0.0.1:0");
    let web_handle = web.handle();
    let earlier_dropped = Cell::new(None);
    let mut hosts = HostGroup::new()
        .with_host(DatabaseHost {
            web,
            database: resource,
            earlier_dropped: &earlier_dropped,
        })
        .build()
        .unwrap();
    let failure = timeout(LIMIT, hosts.run_async())
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(failure.host, "database API");
    assert_eq!(failure.resource, Some("sqlite"));
    assert_eq!(failure.operation, HostOperation::Acquire);
    let original = database.startup_error().expect("original SQLite cause");
    assert!(std::ptr::eq(
        failure.error().io_error(),
        original.io_error()
    ));
    assert!(
        original
            .io_error()
            .get_ref()
            .unwrap()
            .downcast_ref::<rusqlite::Error>()
            .is_some()
    );
    assert!(database.thread_id().is_some() && database.joined());
    assert!(!database.is_open() && web_handle.local_addr().is_none());
    assert_eq!(earlier_dropped.get(), Some(true));
}
