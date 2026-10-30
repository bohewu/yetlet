mod config;

use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use yetlet::view::{Element, View};
use yetlet::server_axum::{App, Page};

async fn home(State(settings): State<Arc<config::Settings>>) -> Page {
    Page::new("Yetlet", View::text(&settings.service.public_message))
}

async fn hello(Path(name): Path<String>) -> Page {
    Page::new(
        "Hello",
        Element::new("main").child(View::text(format!("Hello, {name}!"))),
    )
}

async fn live() -> StatusCode {
    StatusCode::OK
}

// App-owned local-process readiness; add dependency policy here when needed.
async fn ready(Extension(readiness): Extension<Arc<AtomicBool>>) -> StatusCode {
    if readiness.load(Ordering::SeqCst) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

fn router(settings: config::Settings, readiness: Arc<AtomicBool>) -> axum::Router {
    App::new()
        .get("/", home)
        .get("/hello/{name}", hello)
        .get("/health/live", live)
        .get("/health/ready", ready)
        .with_state(Arc::new(settings))
        .into_router()
        .layer(Extension(readiness))
}

fn shutdown_signal(readiness: Arc<AtomicBool>) -> io::Result<impl Future<Output = ()>> {
    // Register all OS handlers before binding; the portable ctrl_c() future
    // installs its handler only on first poll, leaving an early SIGINT race.
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
        tokio::select! {
            _ = terminate.recv() => {}
            _ = interrupt.recv() => {}
        }
        #[cfg(windows)]
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = ctrl_break.recv() => {}
        }
        #[cfg(not(any(unix, windows)))]
        std::future::pending::<()>().await;
        readiness.store(false, Ordering::SeqCst);
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::load()?;
    let address = settings.validate(true)?;
    let readiness = Arc::new(AtomicBool::new(true));
    let shutdown = shutdown_signal(readiness.clone())?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router(settings, readiness))
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
