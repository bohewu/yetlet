//! One Axum serving implementation for caller-runtime and owned-thread entry points.

use std::{
    future::{Future, poll_fn},
    io,
    net::SocketAddr,
};

use axum::{Router, serve::Listener};
use tokio::{net::TcpListener, sync::oneshot};

pub(super) async fn serve_router(
    listener: TcpListener,
    router: Router,
    shutdown: Option<oneshot::Receiver<()>>,
) -> io::Result<()> {
    serve_router_observed(listener, router, shutdown, None).await
}

pub(super) async fn serve_router_observed(
    listener: impl Listener<Addr = SocketAddr>,
    router: Router,
    shutdown: Option<oneshot::Receiver<()>>,
    initialized: Option<oneshot::Sender<()>>,
) -> io::Result<()> {
    let listener = ObservedListener {
        listener,
        initialized,
    };
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

struct ObservedListener<L> {
    listener: L,
    initialized: Option<oneshot::Sender<()>>,
}

impl<L: Listener> Listener for ObservedListener<L> {
    type Io = L::Io;
    type Addr = L::Addr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let initialized = &mut self.initialized;
        let accept = self.listener.accept();
        let mut accept = std::pin::pin!(accept);
        poll_fn(|cx| {
            // Delegate to the selected transport before reporting initialization.
            // The acknowledgment is sent only after real serving execution has
            // polled acceptance, never merely after constructing its future.
            let result = accept.as_mut().poll(cx);
            if let Some(initialized) = initialized.take() {
                let _ = initialized.send(());
            }
            result
        })
        .await
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.listener.local_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Poll;

    #[tokio::test]
    async fn initialization_requires_polling_the_real_serve_future() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (stop, stopping) = oneshot::channel();
        let (initialized, mut observed) = oneshot::channel();
        let mut server = Box::pin(serve_router_observed(
            listener,
            Router::new(),
            Some(stopping),
            Some(initialized),
        ));
        assert!(matches!(
            observed.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        poll_fn(|cx| {
            assert!(server.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert_eq!(observed.try_recv(), Ok(()));
        stop.send(()).unwrap();
        server.await.unwrap();
    }
}
