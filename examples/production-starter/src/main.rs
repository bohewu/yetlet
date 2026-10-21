use std::{
    io,
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use yetlet_production_starter::{config, store::Store, web};

fn shutdown_signal(readiness: Arc<AtomicBool>) -> io::Result<impl Future<Output = ()>> {
    // Register before bind so an early terminal interrupt is handled too.
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
        readiness.store(false, Ordering::SeqCst);
        tracing::info!(event = "shutdown_requested");
    })
}

async fn run() -> Result<(), &'static str> {
    let settings = config::load().map_err(|_| "configuration_invalid")?;
    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_max_level(settings.logging.level.filter())
        .try_init()
        .map_err(|_| "logging_unavailable")?;
    let address = settings.address().map_err(|_| "configuration_invalid")?;
    let store = Store::open(settings.database_path()).map_err(|_| "database_unavailable")?;
    let readiness = Arc::new(AtomicBool::new(false));
    let shutdown = shutdown_signal(readiness.clone()).map_err(|_| "signals_unavailable")?;
    let router = web::app(store.clone(), settings.service.api_token, readiness.clone())?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|_| "listener_unavailable")?;
    readiness.store(true, Ordering::SeqCst);
    tracing::info!(event = "started", address = %address);
    let result = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await;
    readiness.store(false, Ordering::SeqCst);
    // Axum has drained handlers: each completed store operation closed its connection.
    drop(store);
    result.map_err(|_| "server_unavailable")?;
    tracing::info!(event = "stopped");
    Ok(())
}
#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(category) => {
            eprintln!("starter failed: {category}");
            ExitCode::FAILURE
        }
    }
}
