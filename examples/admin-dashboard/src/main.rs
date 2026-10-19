mod config;
mod oidc;
mod vite;

use std::io;
use std::sync::Arc;
use tokio::sync::watch;
use yetlet_lab_operations_board::{Board, DemoIdentities};
use yetlet_secrets::{SecretProvider, SettingsSecrets};

fn application(board: &Board, assets: Arc<vite::Snapshot>) -> axum::Router {
    // Board's global middleware is a fallback service. Keep it intact behind
    // the static route instead of replacing it with another Router fallback.
    vite::router(assets).fallback_service(board.router.clone())
}

fn shutdown_signal(stopping: watch::Sender<bool>) -> io::Result<impl Future<Output = ()>> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    #[cfg(unix)]
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    #[cfg(windows)]
    let mut interrupt = tokio::signal::windows::ctrl_c()?;
    #[cfg(windows)]
    let mut ctrl_break = tokio::signal::windows::ctrl_break()?;
    Ok(async move {
        #[cfg(unix)]
        tokio::select! { _ = terminate.recv() => {}, _ = interrupt.recv() => {} }
        #[cfg(windows)]
        tokio::select! { _ = interrupt.recv() => {}, _ = ctrl_break.recv() => {} }
        #[cfg(not(any(unix, windows)))]
        std::future::pending::<()>().await;
        let _ = stopping.send(true);
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::load()?;
    let address = settings.address()?;
    let assets = vite::Snapshot::load_selected(&settings.cli.vite.root)?;
    let oidc = if settings.oidc.enabled {
        let secrets: Arc<dyn SecretProvider> =
            Arc::new(SettingsSecrets::new(std::collections::BTreeMap::from([(
                "oidc-client-secret".into(),
                settings.oidc.client_secret.clone(),
            )])));
        Some(oidc::configure(&settings.oidc, address, secrets).await?)
    } else {
        None
    };
    let listener = tokio::net::TcpListener::bind(address).await?;
    let actual = listener.local_addr()?;
    let board = if let Some(oidc) = oidc {
        Board::open_with_oidc(
            &settings.database_path(),
            actual,
            oidc,
            assets.stylesheet_paths(),
            assets.entry_script_path(),
        )
        .await?
    } else {
        let identities = DemoIdentities::generate()?;
        for (name, code) in identities.terminal_codes() {
            println!("LOCAL_LOGIN {name} {code}");
        }
        Board::open_with_frontend(
            &settings.database_path(),
            actual,
            identities,
            assets.stylesheet_paths(),
            assets.entry_script_path(),
        )
        .await?
    };
    let shutdown = shutdown_signal(board.stop_signal())?;
    let app = application(&board, assets);
    println!("READY http://{actual}/board");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    board.drain().await?;
    println!("STOPPED streams signalled; admitted database work drained");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn static_routes_preserve_board_login_and_global_security() {
        let dir = tempfile::tempdir().unwrap();
        let assets = vite::Snapshot::load_selected("frontend/").unwrap();
        let stylesheet = assets.stylesheet_paths()[0].clone();
        let board = Board::open_with_frontend(
            &dir.path().join("board.sqlite3"),
            "127.0.0.1:3014".parse().unwrap(),
            DemoIdentities::generate().unwrap(),
            assets.stylesheet_paths(),
            assets.entry_script_path(),
        )
        .await
        .unwrap();
        let app = application(&board, assets);
        for (path, host, expected) in [
            ("/board/login", "127.0.0.1:3014", StatusCode::OK),
            ("/board/login", "evil.test", StatusCode::FORBIDDEN),
            ("/unknown", "127.0.0.1:3014", StatusCode::NOT_FOUND),
            (stylesheet.as_str(), "127.0.0.1:3014", StatusCode::OK),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .header(header::HOST, host)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{path}");
            assert_eq!(
                response.headers()[header::X_CONTENT_TYPE_OPTIONS],
                "nosniff"
            );
            if path == "/board/login" && expected == StatusCode::OK {
                assert!(response.headers().contains_key(header::SET_COOKIE));
                let html = String::from_utf8(
                    to_bytes(response.into_body(), 65536)
                        .await
                        .unwrap()
                        .to_vec(),
                )
                .unwrap();
                assert!(html.contains("name=\"username\""));
                assert!(html.contains(&stylesheet));
                assert!(!html.contains("<script"));
            }
        }
        board.drain().await.unwrap();
    }
}
