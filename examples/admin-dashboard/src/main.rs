mod config;
mod oidc;
#[cfg(feature = "request-tracing")]
mod tracing_output;
mod vite;

use std::io;
use std::sync::Arc;
use tokio::sync::watch;
use yetlet_lab_operations_board::{Board, BoardOptions, DemoIdentities};
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
    #[cfg(feature = "request-tracing")]
    if settings.request_tracing.enabled {
        tracing_output::initialize()
            .map_err(|_| io::Error::other("request tracing initialization failed"))?;
    }
    let address = settings.address()?;
    let sessions = settings.session_backend()?;
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
        open_oidc_board(&settings, &assets, actual, oidc, sessions).await?
    } else {
        let identities = DemoIdentities::generate()?;
        for (name, code) in identities.terminal_codes() {
            println!("LOCAL_LOGIN {name} {code}");
        }
        open_demo_board(&settings, &assets, actual, identities, sessions).await?
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

fn board_options(
    settings: &config::Settings,
    assets: &vite::Snapshot,
    sessions: yetlet_lab_operations_board::SessionBackend,
) -> BoardOptions {
    BoardOptions {
        stylesheets: assets.stylesheet_paths(),
        triage_script: assets.entry_script_path(),
        sessions,
        write_authorization: settings.write_authorization.enabled,
        request_tracing: settings.request_tracing.enabled,
    }
}

async fn open_demo_board(
    settings: &config::Settings,
    assets: &vite::Snapshot,
    actual: std::net::SocketAddr,
    identities: DemoIdentities,
    sessions: yetlet_lab_operations_board::SessionBackend,
) -> Result<Board, yetlet_lab_operations_board::Failure> {
    Board::open_with_options(
        &settings.database_path(),
        actual,
        identities,
        board_options(settings, assets, sessions),
    )
    .await
}
async fn open_oidc_board(
    settings: &config::Settings,
    assets: &vite::Snapshot,
    actual: std::net::SocketAddr,
    oidc: yetlet_lab_operations_board::oidc::Oidc,
    sessions: yetlet_lab_operations_board::SessionBackend,
) -> Result<Board, yetlet_lab_operations_board::Failure> {
    Board::open_with_oidc_and_options(
        &settings.database_path(),
        actual,
        oidc,
        board_options(settings, assets, sessions),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[test]
    fn opt_in_options_map_settings_independently() {
        let assets = vite::Snapshot::load_selected("frontend/").unwrap();
        for (write, trace) in [(false, false), (true, false), (false, true), (true, true)] {
            let loaded = config::load_with_env(vec![
                (
                    "APP_WRITE_AUTHORIZATION__ENABLED".into(),
                    write.to_string().into(),
                ),
                (
                    "APP_REQUEST_TRACING__ENABLED".into(),
                    trace.to_string().into(),
                ),
            ]);
            let available = (!write || cfg!(feature = "commit-write-auth"))
                && (!trace || cfg!(feature = "request-tracing"));
            if !available {
                assert_eq!(
                    loaded.err().unwrap().to_string(),
                    "admin dashboard configuration invalid"
                );
                continue;
            }
            let settings = loaded.unwrap();
            let options = board_options(&settings, &assets, settings.session_backend().unwrap());
            assert_eq!(options.write_authorization, write);
            assert_eq!(options.request_tracing, trace);
            assert_eq!(options.stylesheets, assets.stylesheet_paths());
            assert_eq!(options.triage_script, assets.entry_script_path());
            assert!(matches!(
                options.sessions,
                yetlet_lab_operations_board::SessionBackend::Memory
            ));
        }
    }

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
