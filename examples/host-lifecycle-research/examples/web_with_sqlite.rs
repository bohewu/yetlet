//! Finite Web + managed SQLite consumer; no SDK database adapter is exported.
#[path = "support/sqlite_resource.rs"]
mod sqlite_resource;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use rusqlite::Connection;
use sqlite_resource::{SqliteClient, SqliteResource};
use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};
use yetlet_host_lifecycle_research::{Application, ShutdownHandle, WebHost};

async fn count(State(client): State<SqliteClient>) -> Result<String, (StatusCode, String)> {
    client
        .count()
        .await
        .map(|count| count.to_string())
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))
}

async fn append(
    State(client): State<SqliteClient>,
    value: String,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    client
        .insert(value)
        .await
        .map(|id| (StatusCode::CREATED, id.to_string()))
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))
}

struct StopOnDrop(ShutdownHandle);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let database_path = directory.path().join("managed.sqlite");
    let database = SqliteResource::new(&database_path);
    let client = database.client();
    let database_status = database.status();
    let router = Router::new()
        .route("/", get(count))
        .route("/records", post(append))
        .with_state(client.clone());
    let host = WebHost::new(router)
        .named("records")
        .with_managed_resource("sqlite", database);
    let web_status = host.status();
    let app = Application::new().with_host(host);
    let shutdown = app.shutdown_handle();
    let app = app.build()?;
    assert!(!database_path.exists(), "build must remain inert");

    let (outcome, request) = thread::scope(|scope| {
        let request = scope.spawn(|| -> io::Result<()> {
            let _stop = StopOnDrop(shutdown);
            let deadline = Instant::now() + Duration::from_secs(5);
            while !web_status.admission_open() {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "Web readiness"));
                }
                thread::sleep(Duration::from_millis(1));
            }
            let address = web_status.endpoint().expect("ready Web address");
            let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
            stream.set_read_timeout(Some(Duration::from_secs(3)))?;
            stream.set_write_timeout(Some(Duration::from_secs(2)))?;
            let value = "written through the managed database";
            write!(stream, "POST /records HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{value}", value.len())?;
            let mut response = String::new();
            stream.read_to_string(&mut response)?;
            if !response.starts_with("HTTP/1.1 201") {
                return Err(io::Error::other(response));
            }
            println!("HTTP 201: record inserted through {address}");
            Ok(())
        });
        let outcome = app.run();
        (outcome, request.join().expect("client thread"))
    });
    let report = outcome?;
    request?;
    assert!(web_status.joined());
    assert!(database_status.release_started());
    assert!(database_status.joined());
    assert!(!database_status.is_open());
    assert!(!client.available());
    let persisted: String = Connection::open(&database_path)?.query_row(
        "SELECT value FROM records WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(persisted, "written through the managed database");
    println!(
        "SQLite original actor {:?} joined; connection closed",
        database_status.thread_id()
    );
    println!("Persisted readback: {persisted}");
    println!(
        "{} Host completed; managed dependency released",
        report.hosts.len()
    );
    Ok(())
}
