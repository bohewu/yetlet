use std::{
    any::Any,
    fmt,
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Condvar, Mutex},
    task::{Context, Poll},
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio::{sync::oneshot, time::Sleep};

use crate::composition::HostRunMode;

use super::{WebDriver, WebError};

type ServiceResult = Result<(), WebError>;
type OriginalJoin = thread::Result<ServiceResult>;
type ObserverResult = Option<OriginalJoin>;

pub(super) type ServiceFuture = Pin<Box<dyn Future<Output = ServiceResult> + Send + 'static>>;

pub(super) enum DriverOwner {
    Caller(ServiceFuture),
    Thread(OwnedThread),
}

impl DriverOwner {
    pub(super) fn new(
        driver: WebDriver,
        mode: HostRunMode,
        future: ServiceFuture,
    ) -> Result<Self, WebError> {
        match driver {
            WebDriver::CallerRuntime => Ok(Self::Caller(future)),
            WebDriver::OwnedThread => OwnedThread::start(mode, future).map(Self::Thread),
        }
    }

    pub(super) fn poll_terminal(&mut self, cx: &mut Context<'_>) -> Poll<ServiceResult> {
        match self {
            Self::Caller(future) => future.as_mut().poll(cx),
            Self::Thread(thread) => thread.poll_terminal(cx),
        }
    }
}

enum BootstrapDecision {
    Pending,
    Original(JoinHandle<ServiceResult>),
    NoOriginal,
}

struct Bootstrap {
    decision: Mutex<BootstrapDecision>,
    available: Condvar,
}

/// Closed execution surface: this thread only waits for a private bootstrap,
/// joins the exact original service thread, and returns the untouched result.
/// It runs no runtime, application future, Waker, panic hook, error formatting,
/// or application-result destructor. The retained owner holds its JoinHandle
/// until the returned result can be moved onto the caller for interpretation.
fn observe_original(bootstrap: Arc<Bootstrap>) -> ObserverResult {
    let original = {
        let mut decision = bootstrap
            .decision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while matches!(&*decision, BootstrapDecision::Pending) {
            decision = bootstrap
                .available
                .wait(decision)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        match std::mem::replace(&mut *decision, BootstrapDecision::NoOriginal) {
            BootstrapDecision::Original(original) => Some(original),
            BootstrapDecision::Pending | BootstrapDecision::NoOriginal => None,
        }
    };
    // No bootstrap lock is held across the original Join, including arbitrary
    // service-thread TLS destruction. Never interpret its returned payload here.
    original.map(JoinHandle::join)
}

pub(super) struct OwnedThread {
    observer: Option<JoinHandle<ObserverResult>>,
    bootstrap: Arc<Bootstrap>,
    startup_error: Option<WebError>,
    exiting: oneshot::Receiver<()>,
    exit_observed: bool,
    mode: HostRunMode,
    final_exit_recheck: Option<Pin<Box<Sleep>>>,
}

struct NotifyExit(Option<oneshot::Sender<()>>);

impl Drop for NotifyExit {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

fn spawn_original(
    future: ServiceFuture,
    exiting: oneshot::Sender<()>,
) -> io::Result<JoinHandle<ServiceResult>> {
    thread::Builder::new()
        .name("yetlet-web-owned".into())
        .spawn(move || {
            // Notification, including any Waker callback, runs on the ORIGINAL
            // service thread. Its TLS is covered by observe_original's Join.
            let notify = NotifyExit(Some(exiting));
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            // Keep build-failure notification, then move the guard after the
            // runtime so normal return and unwind notify BEFORE runtime teardown
            // can wait for blocking work. The original Join still covers it all.
            let _notify = notify;
            runtime.block_on(future)
        })
}

impl OwnedThread {
    fn start(mode: HostRunMode, future: ServiceFuture) -> Result<Self, WebError> {
        Self::start_with_spawn(mode, future, spawn_original)
    }

    fn start_with_spawn(
        mode: HostRunMode,
        future: ServiceFuture,
        spawn: impl FnOnce(ServiceFuture, oneshot::Sender<()>) -> io::Result<JoinHandle<ServiceResult>>,
    ) -> Result<Self, WebError> {
        let bootstrap = Arc::new(Bootstrap {
            decision: Mutex::new(BootstrapDecision::Pending),
            available: Condvar::new(),
        });
        let (exiting, receiver) = oneshot::channel();
        let for_observer = bootstrap.clone();
        // Create the observer first: its spawn failure cannot detach a service
        // thread that already exists.
        let observer = thread::Builder::new()
            .name("yetlet-web-join".into())
            .spawn(move || observe_original(for_observer))?;
        let mut owner = Self {
            observer: Some(observer),
            bootstrap,
            startup_error: None,
            exiting: receiver,
            exit_observed: false,
            mode,
            final_exit_recheck: None,
        };
        let decision = match spawn(future, exiting) {
            Ok(original) => BootstrapDecision::Original(original),
            Err(error) => {
                // The failed spawn drops its captured sender on the caller,
                // waking observation without running a Waker on the observer.
                owner.startup_error = Some(error.into());
                BootstrapDecision::NoOriginal
            }
        };
        *owner
            .bootstrap
            .decision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = decision;
        owner.bootstrap.available.notify_one();
        // Even a service-spawn failure retains the precreated observer. Its
        // original error is returned only after that observer has been joined.
        Ok(owner)
    }

    fn observer_finished(&self) -> bool {
        self.observer.as_ref().is_some_and(JoinHandle::is_finished)
    }

    fn take_finished(&mut self) -> Poll<ServiceResult> {
        let observer = self.observer.take().expect("Web join observer retained");
        debug_assert!(observer.is_finished());
        self.final_exit_recheck = None;
        let observed = observer.join();
        if let Some(error) = self.startup_error.take() {
            return Poll::Ready(Err(error));
        }
        Poll::Ready(match observed {
            Ok(Some(Ok(result))) => result,
            Ok(Some(Err(payload))) => Err(panic_error("Web service thread", payload)),
            Err(payload) => Err(panic_error("Web join observer", payload)),
            Ok(None) => Err(WebError::message(
                io::ErrorKind::Other,
                "Web join observer completed without an original service",
            )),
        })
    }

    fn poll_terminal(&mut self, cx: &mut Context<'_>) -> Poll<ServiceResult> {
        if self.observer_finished() {
            // The application's original Join, including all its TLS, has
            // already returned. Only the closed observer's std housekeeping
            // remains after is_finished; this is not a hard OS-latency promise.
            return self.take_finished();
        }
        if !self.exit_observed && Pin::new(&mut self.exiting).poll(cx).is_ready() {
            self.exit_observed = true;
        }
        if self.observer_finished() {
            return self.take_finished();
        }
        if self.exit_observed && self.mode == HostRunMode::Async {
            let delay = self
                .final_exit_recheck
                .get_or_insert_with(|| Box::pin(tokio::time::sleep(Duration::from_millis(1))));
            if delay.as_mut().poll(cx).is_ready() {
                delay
                    .as_mut()
                    .reset(tokio::time::Instant::now() + Duration::from_millis(1));
                // The observer never invokes a registered Waker. This retained
                // timer yields while original runtime teardown, TLS destruction
                // and observer completion are pending, with no application Join
                // on the caller.
                let _ = delay.as_mut().poll(cx);
            }
        }
        // Blocking Core driving supplies bounded park_timeout re-polls; custom
        // event-loop driving likewise supplies progress.
        Poll::Pending
    }
}

impl Drop for OwnedThread {
    fn drop(&mut self) {
        // An interrupted bootstrap must not leave the observer waiting forever.
        // This never joins or signals an already-created application service.
        let mut decision = self
            .bootstrap
            .decision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(&*decision, BootstrapDecision::Pending) {
            *decision = BootstrapDecision::NoOriginal;
            self.bootstrap.available.notify_one();
        }
    }
}

struct ThreadPanic {
    label: &'static str,
    message: Option<String>,
    // Opaque payload destructors can run application code. Keep the original
    // payload alive until the last retained failure owner is explicitly dropped.
    _payload: Mutex<Box<dyn Any + Send + 'static>>,
}

impl fmt::Debug for ThreadPanic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ThreadPanic")
            .field("label", &self.label)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ThreadPanic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} panicked: {}",
            self.label,
            self.message
                .as_deref()
                .unwrap_or("non-string panic payload")
        )
    }
}

impl std::error::Error for ThreadPanic {}

fn panic_error(label: &'static str, payload: Box<dyn Any + Send + 'static>) -> WebError {
    // Only known string payloads contribute to formatting. The original opaque
    // payload remains inside the returned error rather than being dropped here.
    let message = payload.downcast_ref::<String>().cloned().or_else(|| {
        payload
            .downcast_ref::<&str>()
            .map(|message| (*message).to_owned())
    });
    io::Error::other(ThreadPanic {
        label,
        message,
        _payload: Mutex::new(payload),
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        future::poll_fn,
        sync::atomic::{AtomicBool, Ordering},
        task::Waker,
        time::Instant,
    };

    use axum::{Router, http::StatusCode, routing::get};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{
        HostGroup,
        composition::{
            ExclusiveClaim, Host, Managed, ManagedResource, ReleaseCompletion, ResourceGroup,
            StartStatus,
        },
        web::{WebActivity, WebHost},
    };

    #[test]
    fn a_live_owned_thread_keeps_its_original_join() {
        let (release, held) = oneshot::channel();
        let future: ServiceFuture = Box::pin(async move {
            let _ = held.await;
            Ok(())
        });
        let mut owner = OwnedThread::start(HostRunMode::Blocking, future).unwrap();
        let mut cx = Context::from_waker(Waker::noop());
        assert!(owner.poll_terminal(&mut cx).is_pending());
        assert!(owner.observer.is_some());
        release.send(()).unwrap();
        // The test's synchronous wait joins the retained observer, which returns
        // the real original worker's outcome rather than a substitute completion.
        assert!(matches!(
            owner.observer.take().unwrap().join().unwrap(),
            Some(Ok(Ok(())))
        ));
    }

    #[tokio::test]
    async fn failed_worker_spawn_retains_observer_and_original_error() {
        let future: ServiceFuture = Box::pin(async { Ok(()) });
        let mut owner =
            OwnedThread::start_with_spawn(HostRunMode::Async, future, |future, exiting| {
                drop(future);
                drop(exiting);
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected spawn failure",
                ))
            })
            .unwrap();
        assert!(owner.observer.is_some());
        let original = owner.startup_error.as_ref().unwrap().clone();
        let error = tokio::time::timeout(
            Duration::from_secs(2),
            poll_fn(|cx| owner.poll_terminal(cx)),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(std::ptr::eq(original.io_error(), error.io_error()));
        assert!(owner.observer.is_none());
    }

    #[tokio::test]
    async fn panic_payload_remains_owned_until_failure_is_dropped() {
        struct PanicDropFlag(Arc<AtomicBool>);

        impl Drop for PanicDropFlag {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        let payload_dropped = dropped.clone();
        let future: ServiceFuture = Box::pin(async move {
            std::panic::panic_any(PanicDropFlag(payload_dropped));
        });
        let mut owner = OwnedThread::start(HostRunMode::Async, future).unwrap();
        let failure = tokio::time::timeout(
            Duration::from_secs(2),
            poll_fn(|cx| owner.poll_terminal(cx)),
        )
        .await
        .unwrap()
        .unwrap_err();

        assert!(!dropped.load(Ordering::SeqCst));
        assert!(
            failure
                .to_string()
                .contains("Web service thread panicked: non-string panic payload")
        );
        assert!(format!("{failure:?}").contains("ThreadPanic"));
        assert!(!dropped.load(Ordering::SeqCst));

        let retained_failure = failure.clone();
        drop(owner);
        assert!(!dropped.load(Ordering::SeqCst));
        drop(failure);
        assert!(!dropped.load(Ordering::SeqCst));
        drop(retained_failure);
        assert!(dropped.load(Ordering::SeqCst));
    }

    struct TlsGate {
        released: Mutex<bool>,
        changed: Condvar,
        completed: AtomicBool,
        timed_out: AtomicBool,
        resource_alive: AtomicBool,
    }

    struct HandlerTls {
        gate: Arc<TlsGate>,
        entered: Option<oneshot::Sender<()>>,
    }

    impl Drop for HandlerTls {
        fn drop(&mut self) {
            if let Some(entered) = self.entered.take() {
                let _ = entered.send(());
            }
            // Bounded only to make the regression fail instead of hanging if
            // a caller-thread Join prevents Tokio from releasing the gate.
            let deadline = Instant::now() + Duration::from_secs(2);
            let mut released = self.gate.released.lock().unwrap_or_else(|p| p.into_inner());
            while !*released {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    self.gate.timed_out.store(true, Ordering::SeqCst);
                    break;
                }
                let (next, _) = self
                    .gate
                    .changed
                    .wait_timeout(released, remaining)
                    .unwrap_or_else(|p| p.into_inner());
                released = next;
            }
            self.gate.completed.store(true, Ordering::SeqCst);
        }
    }

    thread_local! {
        static HANDLER_TLS: RefCell<Option<HandlerTls>> = const { RefCell::new(None) };
    }

    struct BorrowedResource<'a> {
        released: &'a Cell<bool>,
        released_after_tls: &'a Cell<bool>,
        gate: Arc<TlsGate>,
    }

    impl ManagedResource for BorrowedResource<'_> {
        type Error = WebError;

        fn poll_acquire(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.gate.resource_alive.store(true, Ordering::SeqCst);
            Poll::Ready(Ok(()))
        }

        fn poll_release(
            &mut self,
            _cx: &mut Context<'_>,
            _budget: Duration,
        ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
            self.released_after_tls
                .set(self.gate.completed.load(Ordering::SeqCst));
            self.gate.resource_alive.store(false, Ordering::SeqCst);
            self.released.set(true);
            Ok(ReleaseCompletion::Released(Ok(())))
        }
    }

    struct ResourceWeb<'a> {
        web: WebHost,
        resource: BorrowedResource<'a>,
    }

    impl<'a> Host for ResourceWeb<'a> {
        type Error = WebError;
        type Activity = WebActivity;
        type Resources = Managed<BorrowedResource<'a>, ()>;

        fn name(&self) -> &'static str {
            self.web.name()
        }

        fn validate(&self) -> Result<(), Self::Error> {
            self.web.validate()
        }

        fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
            self.web.exclusive_claims()
        }

        fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
            self.web
                .into_resources()
                .with_managed_resource("borrowed", self.resource)
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn owned_handler_tls_yields_to_caller_and_retains_borrowed_resource() {
        let released = Cell::new(false);
        let released_after_tls = Cell::new(false);
        let gate = Arc::new(TlsGate {
            released: Mutex::new(false),
            changed: Condvar::new(),
            completed: AtomicBool::new(false),
            timed_out: AtomicBool::new(false),
            resource_alive: AtomicBool::new(false),
        });
        let (entered, observed_tls) = oneshot::channel();
        let entered = Arc::new(Mutex::new(Some(entered)));
        let handler_gate = gate.clone();
        let router = Router::new().route(
            "/tls",
            get(move || {
                let gate = handler_gate.clone();
                let entered = entered.clone();
                async move {
                    let entered = entered.lock().unwrap().take();
                    HANDLER_TLS
                        .with(|slot| *slot.borrow_mut() = Some(HandlerTls { gate, entered }));
                    StatusCode::NO_CONTENT
                }
            }),
        );
        let web = WebHost::new(router)
            .with_name("tls-web")
            .with_address("127.0.0.1:0")
            .with_driver(WebDriver::OwnedThread);
        let handle = web.handle();
        let mut hosts = HostGroup::new()
            .with_host(ResourceWeb {
                web,
                resource: BorrowedResource {
                    released: &released,
                    released_after_tls: &released_after_tls,
                    gate: gate.clone(),
                },
            })
            .build()
            .unwrap();
        assert_eq!(
            poll_fn(|cx| hosts.poll_start(cx)).await,
            StartStatus::Running
        );
        let mut client = tokio::net::TcpStream::connect(handle.local_addr().unwrap())
            .await
            .unwrap();
        client
            .write_all(b"GET /tls HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        assert!(response.starts_with(b"HTTP/1.1 204"));
        hosts.shutdown_handle().request_shutdown();
        let mut drive = Box::pin(hosts.run_async());
        tokio::select! {
            result = &mut drive => panic!("completed before TLS gate release: {result:?}"),
            entered = observed_tls => entered.unwrap(),
        }
        drop(drive);
        assert!(!released.get());
        assert!(!gate.completed.load(Ordering::SeqCst));
        let release_gate = gate.clone();
        let release_task = tokio::spawn(async move {
            tokio::task::yield_now().await;
            let retained = release_gate.resource_alive.load(Ordering::SeqCst);
            *release_gate.released.lock().unwrap() = true;
            release_gate.changed.notify_one();
            retained
        });
        tokio::time::timeout(Duration::from_secs(3), hosts.run_async())
            .await
            .unwrap()
            .unwrap();
        assert!(release_task.await.unwrap());
        assert!(!gate.timed_out.load(Ordering::SeqCst));
        assert!(released.get());
        assert!(released_after_tls.get());
        assert!(!gate.resource_alive.load(Ordering::SeqCst));
        assert!(!handle.is_admitting());
    }
}
