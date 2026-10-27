//! Opt-in, local-only PostgreSQL comparison. No default sample/provider changes.
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sqlx::{
    ConnectOptions, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::time::timeout;
use yetlet_server_axum::App;

const QUERY: &str = "SELECT id, random_number FROM public.world WHERE id = $1";
const ACQUIRE_MS: u64 = 250;
const QUERY_MS: u64 = 750;

#[derive(Serialize)]
struct World {
    id: i32,
    #[serde(rename = "randomNumber")]
    random_number: i32,
}

#[derive(Default)]
struct Counters {
    acquire: AtomicU64,
    query_timeout: AtomicU64,
    overall_timeout: AtomicU64,
    provider: AtomicU64,
}

#[derive(Clone)]
struct Database {
    pool: PgPool,
    counters: Arc<Counters>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Failure {
    Acquire,
    QueryTimeout,
    OverallTimeout,
    Provider,
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        match self {
            Self::Acquire | Self::Provider => {
                (StatusCode::SERVICE_UNAVAILABLE, "database unavailable").into_response()
            }
            Self::QueryTimeout | Self::OverallTimeout => {
                (StatusCode::GATEWAY_TIMEOUT, "database timeout").into_response()
            }
        }
    }
}

fn provider_failure(error: &sqlx::Error) -> Failure {
    if error.as_database_error().and_then(|e| e.code()).as_deref() == Some("57014") {
        Failure::QueryTimeout
    } else {
        Failure::Provider
    }
}

impl Database {
    async fn read(&self, id: i32) -> Result<World, Failure> {
        let operation = async {
            let mut connection = self.pool.acquire().await.map_err(|error| {
                if matches!(error, sqlx::Error::PoolTimedOut) {
                    Failure::Acquire
                } else {
                    Failure::Provider
                }
            })?;
            let row = timeout(
                Duration::from_millis(QUERY_MS),
                sqlx::query_as::<_, (i32, i32)>(QUERY)
                    .bind(id)
                    .fetch_one(&mut *connection),
            )
            .await
            .map_err(|_| Failure::QueryTimeout)?
            .map_err(|error| provider_failure(&error))?;
            Ok(World {
                id: row.0,
                random_number: row.1,
            })
        };
        let result = timeout(Duration::from_millis(1250), operation)
            .await
            .unwrap_or(Err(Failure::OverallTimeout));
        if let Err(failure) = result.as_ref() {
            match failure {
                Failure::Acquire => &self.counters.acquire,
                Failure::QueryTimeout => &self.counters.query_timeout,
                Failure::OverallTimeout => &self.counters.overall_timeout,
                Failure::Provider => &self.counters.provider,
            }
            .fetch_add(1, Ordering::Relaxed);
        }
        result
    }
}

async fn world(State(database): State<Database>, Path(id): Path<i32>) -> Response {
    if !(1..=10000).contains(&id) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match database.read(id).await {
        Ok(row) => Json(row).into_response(),
        Err(error) => error.into_response(),
    }
}

fn router(mode: &str, database: Database) -> Result<Router, &'static str> {
    let routes = match mode {
        "axum" => Router::new().route("/db/{id}", get(world)),
        "yetlet" => App::new().get("/db/{id}", world).into_router(),
        _ => return Err("invalid mode"),
    };
    Ok(routes.with_state(database))
}

fn options(
    socket: &str,
    password_file: &str,
    owner: bool,
) -> Result<PgConnectOptions, &'static str> {
    let password =
        std::fs::read_to_string(password_file).map_err(|_| "private credential unavailable")?;
    Ok(PgConnectOptions::new()
        .host(socket)
        .port(5432)
        .database("yt144")
        .username(if owner { "yt144_owner" } else { "yt144_app" })
        .password(password.trim())
        .ssl_mode(PgSslMode::Disable)
        .application_name(if owner {
            "yt144-owner-check"
        } else {
            "yt144-postgres-app"
        })
        .statement_cache_capacity(100)
        .disable_statement_logging())
}

async fn database(socket: &str, password_file: &str) -> Result<Database, &'static str> {
    let pool = PgPoolOptions::new()
        .min_connections(16)
        .max_connections(16)
        .acquire_timeout(Duration::from_millis(ACQUIRE_MS))
        .idle_timeout(None)
        .max_lifetime(None)
        .test_before_acquire(false)
        .connect_with(options(socket, password_file, false)?)
        .await
        .map_err(|_| "database initialization failed")?;
    let mut held = Vec::new();
    let mut pids = BTreeSet::new();
    for _ in 0..16 {
        let mut connection = pool.acquire().await.map_err(|_| "preconnection failed")?;
        let pid: (i32,) = sqlx::query_as("SELECT pg_backend_pid()")
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| "backend check failed")?;
        pids.insert(pid.0);
        held.push(connection);
    }
    let sessions: (i64,) = sqlx::query_as("SELECT count(*) FROM pg_stat_activity WHERE usename = 'yt144_app' AND application_name = 'yt144-postgres-app'")
        .fetch_one(&mut *held[0]).await.map_err(|_| "session check failed")?;
    if pids.len() != 16 || sessions.0 != 16 || pool.size() != 16 {
        pool.close().await;
        return Err("preconnection count mismatch");
    }
    for connection in &mut held {
        let row: (i32, i32) = sqlx::query_as(QUERY)
            .bind(1_i32)
            .fetch_one(&mut **connection)
            .await
            .map_err(|_| "query preparation failed")?;
        if row != (1, 7920) {
            return Err("fixture mismatch");
        }
    }
    drop(held);
    println!(
        "{}",
        serde_json::json!({"event":"preconnected","backend_pids":pids,"sessions":sessions.0,"pool_size":pool.size(),"test_before_acquire":false,"statement_cache_capacity":100})
    );
    Ok(Database {
        pool,
        counters: Arc::new(Counters::default()),
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = (|| {
        if args.len() != 4 {
            return Err("expected mode port socket-directory private-password-file");
        }
        let port: u16 = args[1].parse().map_err(|_| "invalid port")?;
        if port == 0 {
            return Err("invalid port");
        }
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()
            .map_err(|_| "runtime initialization failed")?.block_on(async {
                let database = database(&args[2], &args[3]).await?;
                let app = router(&args[0], database.clone())?;
                let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await.map_err(|_| "listener unavailable")?;
                println!("listening 127.0.0.1:{port}");
                let serving = axum::serve(listener, app).with_graceful_shutdown(async {
                    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("termination signal");
                    tokio::select! { _ = signal.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
                }).await;
                database.pool.close().await;
                let c = &database.counters;
                println!("{}", serde_json::json!({"event":"stopped","pool_closed":database.pool.is_closed(),"acquire":c.acquire.load(Ordering::Relaxed),"query_timeout":c.query_timeout.load(Ordering::Relaxed),"overall_timeout":c.overall_timeout.load(Ordering::Relaxed),"provider":c.provider.load(Ordering::Relaxed)}));
                serving.map_err(|_| "HTTP serving failed")
            })
    })();
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn new_postgres_contract() {
        let socket = std::env::var("YT144_SOCKET").expect("isolated socket required");
        let password =
            std::env::var("YT144_APP_PASSWORD_FILE").expect("private app credential required");
        let owner_password =
            std::env::var("YT144_OWNER_PASSWORD_FILE").expect("private owner credential required");
        let db = database(&socket, &password).await.unwrap();
        for mode in ["axum", "yetlet"] {
            let app = router(mode, db.clone()).unwrap();
            for id in [1, 2, 9999, 10000] {
                let response = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(format!("/db/{id}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.headers()["content-type"], "application/json");
                for name in ["set-cookie", "content-encoding", "server"] {
                    assert!(!response.headers().contains_key(name));
                }
                let expected = format!(
                    "{{\"id\":{id},\"randomNumber\":{}}}",
                    1 + ((id * 7919) % 10000)
                );
                assert_eq!(
                    response.headers()["content-length"],
                    expected.len().to_string()
                );
                assert_eq!(
                    &to_bytes(response.into_body(), 128).await.unwrap()[..],
                    expected.as_bytes()
                );
            }
            for path in ["/db/0", "/db/-1", "/db/10001", "/db/nope"] {
                assert_eq!(
                    app.clone()
                        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::BAD_REQUEST
                );
            }
            assert_eq!(
                app.clone()
                    .oneshot(
                        Request::builder()
                            .uri("/missing")
                            .body(Body::empty())
                            .unwrap()
                    )
                    .await
                    .unwrap()
                    .status(),
                StatusCode::NOT_FOUND
            );
            assert_eq!(
                app.oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/db/1")
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
                StatusCode::METHOD_NOT_ALLOWED
            );
        }
        let read_only: (String,) = sqlx::query_as("SHOW transaction_read_only")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(read_only.0, "on");
        let privileges: (bool,) = sqlx::query_as("SELECT has_table_privilege(current_user, 'public.world', 'INSERT,UPDATE,DELETE,TRUNCATE')").fetch_one(&db.pool).await.unwrap();
        assert!(!privileges.0);
        let mut held = Vec::new();
        for _ in 0..16 {
            held.push(db.pool.acquire().await.unwrap());
        }
        let start = std::time::Instant::now();
        assert_eq!(db.read(1).await.err(), Some(Failure::Acquire));
        assert!(
            start.elapsed() >= Duration::from_millis(200)
                && start.elapsed() < Duration::from_millis(1250)
        );
        drop(held);
        let owner = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(options(&socket, &owner_password, true).unwrap())
            .await
            .unwrap();
        let mut transaction = owner.begin().await.unwrap();
        sqlx::query("LOCK TABLE public.world IN ACCESS EXCLUSIVE MODE")
            .execute(&mut *transaction)
            .await
            .unwrap();
        let start = std::time::Instant::now();
        assert_eq!(db.read(1).await.err(), Some(Failure::QueryTimeout));
        assert!(
            start.elapsed() >= Duration::from_millis(600)
                && start.elapsed() < Duration::from_millis(1250)
        );
        transaction.rollback().await.unwrap();
        assert_eq!(db.read(1).await.unwrap().random_number, 7920);
        for failure in [
            Failure::Acquire,
            Failure::Provider,
            Failure::QueryTimeout,
            Failure::OverallTimeout,
        ] {
            let response = failure.into_response();
            let expected = if matches!(failure, Failure::Acquire | Failure::Provider) {
                b"database unavailable".as_slice()
            } else {
                b"database timeout".as_slice()
            };
            assert_eq!(
                &to_bytes(response.into_body(), 128).await.unwrap()[..],
                expected
            );
        }
        db.pool.close().await;
        assert_eq!(db.read(1).await.err(), Some(Failure::Provider));
        assert!(db.pool.is_closed());
        owner.close().await;
        println!(
            "new PG exact route/read-only/preconnection/acquire/query-timeout/redaction/close checks PASS"
        );
    }
}
