//! One Axum serving implementation for caller-runtime and owned-thread entry points.

use std::io;

use axum::Router;
use tokio::{net::TcpListener, sync::oneshot};

pub(super) async fn serve_router(
    listener: TcpListener,
    router: Router,
    shutdown: Option<oneshot::Receiver<()>>,
) -> io::Result<()> {
    let server = axum::serve(listener, router);
    match shutdown {
        Some(stop_rx) => {
            server
                .with_graceful_shutdown(async move {
                    let _ = stop_rx.await;
                })
                .await
        }
        None => server.await,
    }
}
