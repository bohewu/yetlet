//! Advanced explicit service-thread ownership for native Web serving.
//!
//! The original thread handle remains with the caller through Pending and Join.
//! Dropping the shutdown sender requests graceful stop; dropping the handle
//! detaches the thread and cannot prove drain or original completion.

use std::{
    io::{self, BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use axum::Router;
use tokio::sync::oneshot;

use crate::composition::WorkerCompletion;

/// Advanced owned-thread HTTP worker whose original completion stays with its caller.
///
/// This is an explicit low-level service-thread owner, separate from `App`'s
/// caller-runtime convenience. Its ephemeral default and explicit HTTP 200 probe
/// retain the existing research worker contract; it is not the managed WebHost
/// driver or a default-readiness guarantee.
pub struct HttpWorker {
    endpoint: SocketAddr,
    thread: Option<JoinHandle<io::Result<()>>>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl HttpWorker {
    /// Safe loopback ephemeral default; HTTP readiness is tested separately.
    pub fn start(router: Router) -> io::Result<Self> {
        Self::start_on(router, SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
    }

    /// Explicit interface/port: non-loopback exposure is caller-controlled.
    pub fn start_on(router: Router, bind: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(bind)?;
        listener.set_nonblocking(true)?;
        let endpoint = listener.local_addr()?;
        let (shutdown, stop_rx) = oneshot::channel();
        let thread = thread::Builder::new()
            .name("yetlet-web-axum-owned".into())
            .spawn(move || serve_owned(listener, router, stop_rx))?;
        Ok(Self {
            endpoint,
            thread: Some(thread),
            shutdown: Some(shutdown),
        })
    }

    /// Actual address selected by the operating system.
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    /// An exited worker still needs the exact original Join.
    pub fn is_finished(&self) -> bool {
        // A consumed JoinHandle is terminal, not fresh HTTP readiness.
        self.thread.as_ref().is_none_or(JoinHandle::is_finished)
    }
    /// Whether the original worker JoinHandle remains owned.
    pub fn has_pending_join(&self) -> bool {
        self.thread.is_some()
    }

    /// Probe a selected HTTP 200 readiness route within a cooperative budget.
    ///
    /// The path must be an ASCII relative path without request-line injection.
    /// The caller must not mark its required Composition worker ready until
    /// this returns Ok. An HTTP-only route probe does not imply game readiness.
    pub fn wait_http_ready(&self, path: &str, budget: Duration) -> io::Result<()> {
        if !path.starts_with('/')
            || path.len() > 128
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid readiness path",
            ));
        }
        let started = Instant::now();
        loop {
            // A worker whose stop signal was consumed may still be draining
            // existing requests after its listener closes. Another server
            // could bind the same port; its 200 must never qualify this worker.
            if self.shutdown.is_none() {
                return Err(io::Error::other(
                    "Web worker stopping before HTTP readiness",
                ));
            }
            if self.is_finished() {
                return Err(io::Error::other("Web worker exited before HTTP readiness"));
            }
            let remaining = budget.saturating_sub(started.elapsed());
            match self.probe_once(path, remaining.min(Duration::from_millis(200))) {
                Ok(()) => return Ok(()),
                Err(error) if started.elapsed() >= budget => return Err(error),
                Err(_) => {
                    thread::sleep(
                        Duration::from_millis(5).min(budget.saturating_sub(started.elapsed())),
                    );
                }
            }
        }
    }

    fn probe_once(&self, path: &str, timeout: Duration) -> io::Result<()> {
        let duration = timeout.max(Duration::from_millis(1));
        let stream = TcpStream::connect_timeout(&self.endpoint, duration)?;
        stream.set_read_timeout(Some(duration))?;
        stream.set_write_timeout(Some(duration))?;
        let mut writer = stream.try_clone()?;
        write!(
            writer,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )?;
        writer.flush()?;
        let mut reader = BufReader::new(stream);
        let mut status_line = String::new();
        reader.read_line(&mut status_line)?;
        if status_line.starts_with("HTTP/1.1 200 ") {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "HTTP readiness was not 200: {}",
                status_line.trim()
            )))
        }
    }

    /// Dedicated one-shot graceful shutdown, not queued in application data.
    ///
    /// A failed send may mean the worker already exited; the root must still
    /// observe its original Join even if the shutdown signal failed.
    pub fn request_stop(&mut self) -> io::Result<()> {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Web worker already closed shutdown receiver",
                )
            })?;
        }
        Ok(())
    }

    /// Observe original Join once. Pending retains it; callback must not run
    /// after the Composition Core has reported terminal Joined.
    pub fn observe_within(&mut self, budget: Duration) -> io::Result<WorkerCompletion<io::Error>> {
        let started = Instant::now();
        while self.thread.as_ref().is_some_and(|t| !t.is_finished()) {
            if started.elapsed() >= budget {
                return Ok(WorkerCompletion::Pending);
            }
            thread::sleep(Duration::from_millis(2).min(budget.saturating_sub(started.elapsed())));
        }
        let handle = self.thread.take().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "original Web worker already joined",
            )
        })?;
        let outcome = handle
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("Web worker thread panicked")));
        Ok(WorkerCompletion::Joined(outcome))
    }
}

fn serve_owned(
    listener: TcpListener,
    router: Router,
    stop_rx: oneshot::Receiver<()>,
) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        super::serving::serve_router(listener, router, Some(stop_rx)).await
    })
}
