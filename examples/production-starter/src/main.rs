use std::{
    io,
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use yetlet_production_starter::{config, observability::Metrics, store::Store, web};

fn shutdown_signal(
    readiness: Arc<AtomicBool>,
    store: Store,
) -> io::Result<impl Future<Output = ()>> {
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
        store.close();
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
    let mut store = Store::open(settings.database_path()).map_err(|_| "database_unavailable")?;
    if settings.metrics.enabled {
        let metrics = Metrics::new().map_err(|_| "metrics_unavailable")?;
        store = store.with_metrics(Arc::new(metrics));
    }
    let readiness = Arc::new(AtomicBool::new(false));
    let shutdown =
        shutdown_signal(readiness.clone(), store.clone()).map_err(|_| "signals_unavailable")?;
    let (shutdown_started, shutdown_received) = tokio::sync::oneshot::channel();
    let shutdown = async move {
        shutdown.await;
        let _ = shutdown_started.send(());
    };
    let router = web::app(store.clone(), settings.service.api_token, readiness.clone())?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|_| "listener_unavailable")?;
    readiness.store(true, Ordering::SeqCst);
    tracing::info!(event = "started", address = %address);
    let server = async {
        let result = axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await;
        readiness.store(false, Ordering::SeqCst);
        store.drain().await;
        result.map_err(|_| "server_unavailable")
    };
    tokio::pin!(server);
    tokio::select! {
        result = &mut server => result?,
        _ = shutdown_received => {
            if let Ok(result) = tokio::time::timeout(
                std::time::Duration::from_secs(10), &mut server,
            ).await {
                result?;
            } else {
                // Tokio cannot cancel started blocking SQL. Do not report a
                // clean stop or let Runtime::drop wait without a bound.
                tracing::error!(event = "shutdown_incomplete");
                std::process::exit(1);
            }
        }
    }
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
