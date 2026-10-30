use axum::{Extension, extract::Path, http::StatusCode};
use std::{
    ffi::OsString,
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use yetlet::view::{Element, View};
use yetlet::server_axum::{App, Page};

async fn home() -> View {
    Element::new("main")
        .child(Element::new("h1").child(View::text("Hello <Yetlet> & Rust!")))
        .into()
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

fn router(readiness: Arc<AtomicBool>) -> axum::Router {
    App::new()
        .page("/", "Yetlet", home)
        .get("/hello/{name}", hello)
        .get("/health/live", live)
        .get("/health/ready", ready)
        .into_router()
        .layer(Extension(readiness))
}

fn bind_address(host: Option<OsString>, port: Option<OsString>) -> io::Result<SocketAddr> {
    let host = host.unwrap_or_else(|| "127.0.0.1".into());
    let host = match host.to_str() {
        Some("127.0.0.1") => Ipv4Addr::LOCALHOST,
        Some("0.0.0.0") => Ipv4Addr::UNSPECIFIED,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "YETLET_HOST must be 127.0.0.1 or explicitly 0.0.0.0",
            ));
        }
    };
    let port = port.unwrap_or_else(|| "3000".into());
    let port = port
        .to_str()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "YETLET_PORT must be decimal digits in 1..=65535",
            )
        })?;
    Ok(SocketAddr::from((host, port)))
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
    let address = bind_address(
        std::env::var_os("YETLET_HOST"),
        std::env::var_os("YETLET_PORT"),
    )?;
    let readiness = Arc::new(AtomicBool::new(true));
    let shutdown = shutdown_signal(readiness.clone())?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router(readiness))
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
