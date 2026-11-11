#![cfg(not(target_arch = "wasm32"))]
//! Integration tests for the native Axum application startup helpers.

use std::{
    io,
    net::{SocketAddr, TcpListener as StdTcpListener},
    time::Duration,
};

use axum::{Router, routing::get};
use reqwest::{Client, StatusCode};
use tokio::{
    net::TcpListener,
    task::JoinHandle,
    time::{Instant, sleep},
};
use yetlet::server_axum::App;

const DEFAULT_ADDRESS: &str = "127.0.0.1:7878";

fn unused_loopback_address() -> SocketAddr {
    let listener = StdTcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.local_addr().unwrap()
}

async fn assert_http_response(
    server: &JoinHandle<io::Result<()>>,
    address: SocketAddr,
    expected_body: &str,
) {
    let client = Client::builder().no_proxy().build().unwrap();
    let url = format!("http://{address}/");
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        assert!(
            !server.is_finished(),
            "server exited before accepting an HTTP request at {url}"
        );

        match client.get(&url).send().await {
            Ok(response) => {
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.text().await.unwrap(), expected_body);
                return;
            }
            Err(error) => {
                assert!(
                    Instant::now() < deadline,
                    "server did not become ready at {url}: {error}"
                );
                sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

async fn stop_server(server: JoinHandle<io::Result<()>>) {
    server.abort();
    let error = server.await.expect_err("the server task should be aborted");
    assert!(error.is_cancelled());
}

#[tokio::test]
async fn serve_uses_loopback_default_and_supports_new_and_existing_routers() {
    let default_address: SocketAddr = DEFAULT_ADDRESS.parse().unwrap();

    match TcpListener::bind(default_address).await {
        Ok(listener) => drop(listener),
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
            let bind_error = App::new().serve().await.unwrap_err();
            assert_eq!(bind_error.kind(), io::ErrorKind::AddrInUse);
            eprintln!(
                "skipping live default-address HTTP assertions because {DEFAULT_ADDRESS} is already occupied"
            );
            return;
        }
        Err(error) => panic!("could not probe default address {DEFAULT_ADDRESS}: {error}"),
    }

    let app = App::new().get("/", || async { "new-app" });
    let server = tokio::spawn(app.serve());
    assert_http_response(&server, default_address, "new-app").await;
    stop_server(server).await;

    let router = Router::new().route("/", get(|| async { "existing-router" }));
    let server = tokio::spawn(App::from_router(router).serve());
    assert_http_response(&server, default_address, "existing-router").await;
    stop_server(server).await;

    let occupied = TcpListener::bind(default_address).await.unwrap();
    let bind_error = App::new().serve().await.unwrap_err();
    assert_eq!(bind_error.kind(), io::ErrorKind::AddrInUse);
    drop(occupied);
}

#[tokio::test]
async fn serve_on_uses_explicit_address_and_supports_existing_routers() {
    let address = unused_loopback_address();
    let router = Router::new().route("/", get(|| async { "explicit-address" }));
    let server = tokio::spawn(App::from_router(router).serve_on(address));

    assert_http_response(&server, address, "explicit-address").await;
    stop_server(server).await;
}

#[tokio::test]
async fn serve_on_returns_the_original_bind_error() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = occupied.local_addr().unwrap();

    let error = App::new().serve_on(address).await.unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AddrInUse);

    drop(occupied);
}

#[test]
fn explicit_router_conversion_remains_available() {
    let router = Router::new().route("/", get(|| async { "native" }));
    let app = App::from_router(router);
    let _native_router: Router = app.into_router();
}
