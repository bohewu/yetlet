//! Managed native Web hosting over the shared Core lifetime coordinator.

use std::{
    error::Error,
    fmt,
    future::{Future, poll_fn},
    io,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::IntoResponse,
};
use tokio::sync::oneshot;

use crate::composition::{
    AdmissionOutcome, ExclusiveClaim, Host, HostActivity, HostRunMode, ReleaseCompletion,
    ResourceGroup, WorkerCompletion,
};

mod driver;
mod probe;

use driver::{DriverOwner, ServiceFuture};
pub use probe::HttpProbe;

/// An exact Web execution choice. Unsupported placements fail; none falls back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriver {
    /// Progress the retained service future on the caller's Tokio runtime.
    CallerRuntime,
    /// Run the same service future on an owned service thread, with a dedicated
    /// join observer that keeps service-thread destruction off the caller.
    OwnedThread,
}

/// A Web failure retaining the original I/O error behind a shared owner.
///
/// Sharing a cause allows a startup failure and its actual terminal completion
/// to remain inspectable without replacing the cause with a formatted string.
/// A thread panic's payload stays in the retained error source; observing
/// completion does not run its custom destructor. It follows ordinary error Drop.
#[derive(Debug, Clone)]
pub struct WebError(Arc<io::Error>);

impl WebError {
    pub fn kind(&self) -> io::ErrorKind {
        self.0.kind()
    }

    pub fn io_error(&self) -> &io::Error {
        self.0.as_ref()
    }

    fn message(kind: io::ErrorKind, message: impl Into<String>) -> Self {
        io::Error::new(kind, message.into()).into()
    }
}

impl From<io::Error> for WebError {
    fn from(error: io::Error) -> Self {
        Self(Arc::new(error))
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0.as_ref(), formatter)
    }
}

impl Error for WebError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

const STARTING: u8 = 0;
const ADMITTING: u8 = 1;
const CLOSED: u8 = 2;

struct Shared {
    address: Mutex<Option<SocketAddr>>,
    ready: AtomicBool,
    admission: AtomicU8,
}

impl Shared {
    fn new() -> Self {
        Self {
            address: Mutex::new(None),
            ready: AtomicBool::new(false),
            admission: AtomicU8::new(STARTING),
        }
    }

    fn close(&self) {
        self.admission.store(CLOSED, Ordering::SeqCst);
        self.ready.store(false, Ordering::Release);
    }

    fn is_closed(&self) -> bool {
        self.admission.load(Ordering::SeqCst) == CLOSED
    }
}

/// Read-only observations for one Web Host; it owns no listener or completion.
///
/// The address is retained after shutdown as bound-address metadata. It is not
/// proof that the endpoint remains live or that another service has not reused it.
#[derive(Clone)]
pub struct WebHandle(Arc<Shared>);

impl WebHandle {
    pub fn local_addr(&self) -> Option<SocketAddr> {
        *self
            .0
            .address
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// This Host initialized and passed its optional probe. Other Hosts may
    /// still be starting, so this does not imply ordinary admission is open.
    pub fn is_ready(&self) -> bool {
        self.0.ready.load(Ordering::Acquire) && !self.0.is_closed()
    }

    /// Ordinary requests are admitted only after the whole group is ready.
    pub fn is_admitting(&self) -> bool {
        self.0.admission.load(Ordering::SeqCst) == ADMITTING
    }
}

/// An inert description of one HTTP service with default resource management.
///
/// Async group driving defaults to the caller's Tokio runtime, including its
/// current-thread runtime. Synchronous group driving defaults to an owned
/// service thread plus a dedicated join observer. An explicit driver never
/// silently changes placement.
///
/// Async driving, including an explicitly owned thread observed asynchronously,
/// requires a caller Tokio runtime with I/O and time enabled. The adapter never
/// creates or shuts down that caller runtime. Construction and build bind
/// nothing; actual acquisition begins only when the retained group is driven.
pub struct WebHost {
    name: &'static str,
    address: String,
    router: Router,
    driver: Option<WebDriver>,
    probe: Option<HttpProbe>,
    #[cfg(feature = "web-tls")]
    tls: Option<super::WebTls>,
    shared: Arc<Shared>,
}

impl WebHost {
    pub fn new(router: Router) -> Self {
        Self {
            name: "web",
            address: super::DEFAULT_LISTEN_ADDRESS.into(),
            router,
            driver: None,
            probe: None,
            #[cfg(feature = "web-tls")]
            tls: None,
            shared: Arc::new(Shared::new()),
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Configure a numeric address or hostname and port. DNS resolution belongs
    /// to asynchronous startup, not construction or inert plan validation.
    pub fn with_address(mut self, address: impl ToString) -> Self {
        self.address = address.to_string();
        self
    }

    /// The configured address, before any resolution or listener acquisition.
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn with_driver(mut self, driver: WebDriver) -> Self {
        self.driver = Some(driver);
        self
    }

    /// Serve HTTPS on this Host using its fixed or reloadable certificate.
    #[cfg(feature = "web-tls")]
    pub fn with_tls(mut self, tls: super::WebTls) -> Self {
        self.tls = Some(tls);
        self
    }

    pub fn with_http_probe(mut self, probe: HttpProbe) -> Self {
        self.probe = Some(probe);
        self
    }

    pub fn handle(&self) -> WebHandle {
        WebHandle(self.shared.clone())
    }
}

impl Host for WebHost {
    type Error = WebError;
    type Activity = WebActivity;
    type Resources = ();

    fn name(&self) -> &'static str {
        self.name
    }

    fn validate(&self) -> Result<(), Self::Error> {
        validate_address(&self.address)?;
        if let Some(probe) = &self.probe {
            #[cfg(feature = "web-tls")]
            if self.tls.is_some() {
                return Err(WebError::message(
                    io::ErrorKind::InvalidInput,
                    "HttpProbe requires a plain HTTP WebHost",
                ));
            }
            probe.validate()?;
        }
        Ok(())
    }

    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        self.address
            .parse::<SocketAddr>()
            .ok()
            .filter(|address| address.port() != 0)
            .map(|address| ExclusiveClaim::new("tcp-listener", address.to_string()))
            .into_iter()
            .collect()
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        ResourceGroup::new(WebActivity::new(self))
    }
}

fn validate_address(address: &str) -> Result<(), WebError> {
    if address.parse::<SocketAddr>().is_ok() {
        return Ok(());
    }
    let valid = address.rsplit_once(':').is_some_and(|(host, port)| {
        !host.is_empty()
            && !host.contains(':')
            && !host.chars().any(|character| {
                character.is_whitespace() || character.is_control() || "/?#".contains(character)
            })
            && port.parse::<u16>().is_ok()
    });
    if valid {
        Ok(())
    } else {
        Err(WebError::message(
            io::ErrorKind::InvalidInput,
            "Web address must be a socket address or hostname:port",
        ))
    }
}

type ProbeFuture = Pin<Box<dyn Future<Output = Result<(), WebError>> + Send>>;

/// The retained activity supplied by WebHost's ResourceGroup.
///
/// Its original service future or thread completion stays here through cancelled
/// borrowing drives, Pending and deadlines. Use Host/ResourceGroup adapters for
/// custom management; there is no mutable replacement access to live ownership.
pub struct WebActivity {
    host: Option<WebHost>,
    configured_driver: Option<WebDriver>,
    selected_driver: Option<WebDriver>,
    run_mode: Option<HostRunMode>,
    shared: Arc<Shared>,
    owner: Option<DriverOwner>,
    stop: Option<oneshot::Sender<()>>,
    bound: Option<oneshot::Receiver<SocketAddr>>,
    initialized: Option<oneshot::Receiver<Result<(), WebError>>>,
    terminal: Option<Result<(), WebError>>,
    readiness_error: Option<WebError>,
    launched: bool,
    bound_observed: bool,
    initialized_observed: bool,
    stopping: bool,
    joined: bool,
}

impl WebActivity {
    fn new(host: WebHost) -> Self {
        Self {
            configured_driver: host.driver,
            selected_driver: None,
            run_mode: None,
            shared: host.shared.clone(),
            host: Some(host),
            owner: None,
            stop: None,
            bound: None,
            initialized: None,
            terminal: None,
            readiness_error: None,
            launched: false,
            bound_observed: false,
            initialized_observed: false,
            stopping: false,
            joined: false,
        }
    }

    fn launch(&mut self) -> Result<(), WebError> {
        if self.run_mode.is_none() {
            self.prepare_driver(HostRunMode::Async)?;
        }
        self.launched = true;
        let (stop, stopping) = oneshot::channel();
        let (bound, bound_receiver) = oneshot::channel();
        let (initialized, initialized_receiver) = oneshot::channel();
        self.stop = Some(stop);
        self.bound = Some(bound_receiver);
        self.initialized = Some(initialized_receiver);
        let future: ServiceFuture = Box::pin(run_service(
            self.host.take().expect("unstarted Web host"),
            stopping,
            bound,
            initialized,
        ));
        match DriverOwner::new(
            self.selected_driver.expect("prepared Web driver"),
            self.run_mode.expect("prepared drive kind"),
            future,
        ) {
            Ok(owner) => {
                self.owner = Some(owner);
                Ok(())
            }
            Err(error) => {
                self.terminal = Some(Err(error.clone()));
                self.shared.close();
                Err(error)
            }
        }
    }

    fn progress(&mut self, cx: &mut Context<'_>) {
        if let Some(owner) = self.owner.as_mut()
            && let Poll::Ready(result) = owner.poll_terminal(cx)
        {
            self.terminal = Some(result);
            self.owner = None;
            self.shared.close();
        }
    }

    fn early_completion(&self) -> Option<WebError> {
        self.terminal.as_ref().map(|result| {
            result.as_ref().err().cloned().unwrap_or_else(|| {
                WebError::message(
                    io::ErrorKind::UnexpectedEof,
                    "Web service completed before readiness",
                )
            })
        })
    }
}

impl HostActivity for WebActivity {
    type Error = WebError;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        if let Some(previous) = self.run_mode {
            return if previous == mode {
                Ok(())
            } else {
                Err(WebError::message(
                    io::ErrorKind::InvalidInput,
                    "a prepared Web driver cannot change drive kind",
                ))
            };
        }
        let selected = self.configured_driver.unwrap_or(match mode {
            HostRunMode::Async => WebDriver::CallerRuntime,
            HostRunMode::Blocking => WebDriver::OwnedThread,
        });
        if mode == HostRunMode::Blocking && selected == WebDriver::CallerRuntime {
            return Err(WebError::message(
                io::ErrorKind::InvalidInput,
                "caller-runtime Web cannot use the synchronous group driver",
            ));
        }
        if mode == HostRunMode::Async && tokio::runtime::Handle::try_current().is_err() {
            return Err(WebError::message(
                io::ErrorKind::NotConnected,
                "async Web driving requires a caller Tokio runtime with I/O and time enabled",
            ));
        }
        self.selected_driver = Some(selected);
        self.run_mode = Some(mode);
        Ok(())
    }

    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.launched
            && let Err(error) = self.launch()
        {
            return Poll::Ready(Err(error));
        }
        self.progress(cx);
        if let Some(error) = self.early_completion() {
            return Poll::Ready(Err(error));
        }
        if self.bound_observed {
            return Poll::Ready(Ok(()));
        }
        match self.bound.as_mut() {
            Some(bound) => match Pin::new(bound).poll(cx) {
                Poll::Ready(Ok(_)) => {
                    self.bound_observed = true;
                    self.bound = None;
                    Poll::Ready(Ok(()))
                }
                // Completion retains the actual bind/runtime/panic cause; a
                // closed notification alone is not the original Join outcome.
                Poll::Ready(Err(_)) => {
                    self.bound = None;
                    Poll::Pending
                }
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Pending,
        }
    }

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.progress(cx);
        if let Some(error) = self.early_completion() {
            return Poll::Ready(Err(error));
        }
        if let Some(error) = &self.readiness_error {
            return Poll::Ready(Err(error.clone()));
        }
        if self.stopping {
            return Poll::Ready(Err(WebError::message(
                io::ErrorKind::Interrupted,
                "Web readiness was stopped",
            )));
        }
        if self.shared.is_closed() {
            // The service can close status before its original thread exit is
            // observable. Wait for that cause instead of inventing a stop error.
            return Poll::Pending;
        }
        if !self.initialized_observed {
            match self.initialized.as_mut() {
                Some(initialized) => match Pin::new(initialized).poll(cx) {
                    Poll::Ready(Ok(Ok(()))) => {
                        self.initialized_observed = true;
                        self.initialized = None;
                    }
                    Poll::Ready(Ok(Err(error))) => {
                        self.initialized = None;
                        self.readiness_error = Some(error.clone());
                        return Poll::Ready(Err(error));
                    }
                    Poll::Ready(Err(_)) => {
                        self.initialized = None;
                        return Poll::Pending;
                    }
                    Poll::Pending => return Poll::Pending,
                },
                None => return Poll::Pending,
            }
        }
        self.shared.ready.store(true, Ordering::Release);
        if self.shared.is_closed() {
            self.shared.ready.store(false, Ordering::Release);
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }

    fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.progress(cx);
        if self.terminal.is_some() || self.shared.is_closed() {
            // Closure starts rollback even when original thread-local destruction
            // keeps Join pending. Only poll_join supplies the terminal outcome.
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
        if self.stopping || self.shared.is_closed() {
            return Ok(AdmissionOutcome::Closed);
        }
        if !self.shared.ready.load(Ordering::Acquire) {
            // close stores CLOSED before clearing readiness. Recheck after the
            // readiness load so that concurrent termination is not a new error.
            if self.shared.is_closed() {
                return Ok(AdmissionOutcome::Closed);
            }
            return Err(WebError::message(
                io::ErrorKind::NotConnected,
                "Web admission cannot open before readiness",
            ));
        }
        match self.shared.admission.compare_exchange(
            STARTING,
            ADMITTING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => Ok(AdmissionOutcome::Opened),
            // The service can close between poll_finished and this final gate.
            Err(CLOSED) => Ok(AdmissionOutcome::Closed),
            Err(_) => Err(WebError::message(
                io::ErrorKind::NotConnected,
                "Web admission was already opened",
            )),
        }
    }

    fn close_admission(&mut self, _cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.stopping = true;
        self.shared.close();
        Ok(())
    }

    fn request_shutdown(&mut self, _cx: &mut Context<'_>) -> Result<(), Self::Error> {
        self.stopping = true;
        self.shared.close();
        if let Some(stop) = self.stop.take() {
            // A closed receiver is not a substitute for actual completion.
            // The retained original owner still supplies the terminal outcome.
            let _ = stop.send(());
        }
        Ok(())
    }

    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        if self.joined {
            return Err(WebError::message(
                io::ErrorKind::AlreadyExists,
                "original Web completion already observed",
            ));
        }
        self.progress(cx);
        if self.owner.is_some() {
            return Ok(WorkerCompletion::Pending);
        }
        self.joined = true;
        Ok(WorkerCompletion::Joined(
            self.terminal.clone().unwrap_or(Ok(())),
        ))
    }

    fn poll_release(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        if !self.joined {
            return Ok(ReleaseCompletion::Pending);
        }
        self.shared.close();
        self.host = None;
        self.bound = None;
        self.initialized = None;
        self.stop = None;
        self.terminal = None;
        self.readiness_error = None;
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

fn gated_router(router: Router, shared: Arc<Shared>, probe: Option<HttpProbe>) -> Router {
    router.layer(from_fn(move |request: Request, next: Next| {
        let shared = shared.clone();
        let probe = probe.clone();
        async move {
            let phase = shared.admission.load(Ordering::SeqCst);
            let allowed = phase == ADMITTING
                || (phase == STARTING
                    && probe.as_ref().is_some_and(|probe| probe.matches(&request)));
            if allowed {
                next.run(request).await
            } else {
                axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response()
            }
        }
    }))
}

struct CloseOnExit(Arc<Shared>);

impl Drop for CloseOnExit {
    fn drop(&mut self) {
        self.0.close();
    }
}

async fn run_service(
    host: WebHost,
    stop: oneshot::Receiver<()>,
    bound: oneshot::Sender<SocketAddr>,
    ready: oneshot::Sender<Result<(), WebError>>,
) -> Result<(), WebError> {
    let WebHost {
        address,
        router,
        shared,
        probe,
        #[cfg(feature = "web-tls")]
        tls,
        ..
    } = host;
    let _close_on_exit = CloseOnExit(shared.clone());
    if shared.is_closed() {
        return Ok(());
    }
    let router = gated_router(router, shared.clone(), probe.clone());
    let listener = tokio::net::TcpListener::bind(address.as_str()).await?;
    let endpoint = listener.local_addr()?;
    *shared
        .address
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(endpoint);
    let _ = bound.send(endpoint);
    if shared.is_closed() {
        return Ok(());
    }
    let (initialized, mut initialization) = oneshot::channel();
    let mut server = Box::pin(async move {
        use super::serving::serve_router_observed;
        #[cfg(feature = "web-tls")]
        if let Some(tls) = tls {
            return serve_router_observed(
                super::tls::TlsListener::new(listener, tls),
                router,
                Some(stop),
                Some(initialized),
            )
            .await;
        }
        serve_router_observed(listener, router, Some(stop), Some(initialized)).await
    });
    let mut ready = Some(ready);
    let mut initialized = false;
    let mut probe_future: Option<ProbeFuture> = None;
    let mut probe_error: Option<WebError> = None;
    // Server, probe, deadline and notification all remain inside this one
    // retained operation. They are polled on the selected runtime in both modes.
    poll_fn(|cx| {
        if let Poll::Ready(result) = server.as_mut().poll(cx) {
            return Poll::Ready(match probe_error.take() {
                Some(error) => Err(error),
                None => result.map_err(Into::into),
            });
        }
        if shared.is_closed() {
            // Explicit stop closes the unspawned probe client before server
            // drain. Cancelling only the borrowing group drive never does this.
            probe_future = None;
            ready = None;
            return Poll::Pending;
        }
        if !initialized {
            match Pin::new(&mut initialization).poll(cx) {
                Poll::Ready(Ok(())) => initialized = true,
                Poll::Ready(Err(_)) => {
                    let error = WebError::message(
                        io::ErrorKind::UnexpectedEof,
                        "Web serving driver closed its initialization notification",
                    );
                    probe_error = Some(error.clone());
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(Err(error));
                    }
                    initialized = true;
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        if ready.is_some() {
            if let Some(probe) = &probe {
                if probe_future.is_none() {
                    probe_future = Some(Box::pin(probe::run(endpoint, probe.clone())));
                }
                match probe_future
                    .as_mut()
                    .expect("retained probe")
                    .as_mut()
                    .poll(cx)
                {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(result) => {
                        probe_future = None;
                        if let Err(error) = &result {
                            probe_error = Some(error.clone());
                        }
                        if let Some(ready) = ready.take() {
                            let _ = ready.send(result);
                        }
                    }
                }
            } else if let Some(ready) = ready.take() {
                let _ = ready.send(Ok(()));
            }
        }
        Poll::Pending
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    use crate::composition::{HostGroup, HostOperation, Retained};
    use std::{cell::RefCell, sync::mpsc};

    struct DependencyRelease(Arc<AtomicBool>);

    impl Drop for DependencyRelease {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }

    struct ControlledWebHost {
        name: &'static str,
        activity: ControlledWebActivity,
        released: Arc<AtomicBool>,
    }

    impl Host for ControlledWebHost {
        type Error = WebError;
        type Activity = ControlledWebActivity;
        type Resources = Retained<DependencyRelease, ()>;

        fn name(&self) -> &'static str {
            self.name
        }

        fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
            ResourceGroup::new(self.activity).with_resource(DependencyRelease(self.released))
        }
    }

    // Substitute only service preparation. The original driver, Web admission,
    // group lifecycle and final resource release all use their production paths.
    struct ControlledWebActivity {
        web: WebActivity,
        service: Option<ServiceFuture>,
        opened: Arc<AtomicBool>,
        admitted: Option<oneshot::Sender<()>>,
    }

    impl HostActivity for ControlledWebActivity {
        type Error = WebError;

        fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
            self.web.prepare_driver(mode)
        }

        fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            if let Some(service) = self.service.take() {
                self.web.launched = true;
                self.web.bound_observed = true;
                self.web.owner = Some(
                    DriverOwner::new(
                        self.web.selected_driver.unwrap(),
                        self.web.run_mode.unwrap(),
                        service,
                    )
                    .unwrap(),
                );
            }
            self.web.poll_start(cx)
        }

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.web.poll_ready(cx)
        }

        fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
            self.web.poll_finished(cx)
        }

        fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
            let outcome = self.web.open_admission()?;
            if outcome == AdmissionOutcome::Opened {
                self.opened.store(true, Ordering::Release);
                if let Some(admitted) = self.admitted.take() {
                    let _ = admitted.send(());
                }
            }
            Ok(outcome)
        }

        fn close_admission(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error> {
            self.web.close_admission(cx)
        }

        fn request_shutdown(&mut self, cx: &mut Context<'_>) -> Result<(), Self::Error> {
            self.web.request_shutdown(cx)
        }

        fn poll_join(
            &mut self,
            cx: &mut Context<'_>,
            budget: Duration,
        ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
            self.web.poll_join(cx, budget)
        }

        fn poll_release(
            &mut self,
            cx: &mut Context<'_>,
            budget: Duration,
        ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
            self.web.poll_release(cx, budget)
        }
    }

    #[tokio::test]
    async fn startup_probe_exception_matches_method_and_full_path_then_closes() {
        let shared = Arc::new(Shared::new());
        let router = gated_router(
            Router::new().route(
                "/health",
                get(|| async { StatusCode::NO_CONTENT }).post(|| async { StatusCode::NO_CONTENT }),
            ),
            shared.clone(),
            Some(HttpProbe::new(
                Method::GET,
                "/health?check=one",
                StatusCode::NO_CONTENT,
            )),
        );
        for (method, uri, expected) in [
            (Method::GET, "/health?check=one", StatusCode::NO_CONTENT),
            (
                Method::GET,
                "/health?check=two",
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (
                Method::POST,
                "/health?check=one",
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (Method::GET, "/missing", StatusCode::SERVICE_UNAVAILABLE),
        ] {
            let request = Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap();
            assert_eq!(
                router.clone().oneshot(request).await.unwrap().status(),
                expected
            );
        }
        shared.admission.store(ADMITTING, Ordering::SeqCst);
        let request = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            router.clone().oneshot(request).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        shared.close();
        let request = Request::builder()
            .uri("/health?check=one")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            router.oneshot(request).await.unwrap().status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn ready_web_failure_keeps_two_host_admission_closed_until_original_join() {
        struct HeldThreadExit {
            entered: Option<oneshot::Sender<()>>,
            release: mpsc::Receiver<()>,
            dependency_released: Arc<AtomicBool>,
            saw_retained_dependency: Arc<AtomicBool>,
        }

        impl Drop for HeldThreadExit {
            fn drop(&mut self) {
                let _ = self.entered.take().unwrap().send(());
                // Dropping the test's sender also unblocks this destructor when
                // an earlier assertion fails; no orphaned test thread is held.
                let _ = self.release.recv();
                self.saw_retained_dependency.store(
                    !self.dependency_released.load(Ordering::Acquire),
                    Ordering::Release,
                );
            }
        }

        thread_local! {
            static HELD_EXIT: RefCell<Option<HeldThreadExit>> = const { RefCell::new(None) };
        }

        let first_released = Arc::new(AtomicBool::new(false));
        let second_released = Arc::new(AtomicBool::new(false));
        let retained_at_exit = Arc::new(AtomicBool::new(false));
        let first_opened = Arc::new(AtomicBool::new(false));
        let second_opened = Arc::new(AtomicBool::new(false));
        let (exiting, observed_exit) = oneshot::channel();
        let (release_exit, held_exit) = mpsc::channel();
        let exit = HeldThreadExit {
            entered: Some(exiting),
            release: held_exit,
            dependency_released: first_released.clone(),
            saw_retained_dependency: retained_at_exit.clone(),
        };

        let mut first =
            WebActivity::new(WebHost::new(Router::new()).with_driver(WebDriver::OwnedThread));
        let first_shared = first.shared.clone();
        let first_service_shared = first.shared.clone();
        let (first_ready, initialized) = oneshot::channel();
        first.initialized = Some(initialized);
        let (fail, failure_requested) = oneshot::channel();
        let original = WebError::message(
            io::ErrorKind::ConnectionAborted,
            "original ready Web failure",
        );
        let failure = original.clone();
        let first_service: ServiceFuture = Box::pin(async move {
            let _close_on_exit = CloseOnExit(first_service_shared);
            HELD_EXIT.with(|slot| *slot.borrow_mut() = Some(exit));
            let _ = first_ready.send(Ok(()));
            let _ = failure_requested.await;
            Err(failure)
        });

        let mut second =
            WebActivity::new(WebHost::new(Router::new()).with_driver(WebDriver::CallerRuntime));
        let second_shared = second.shared.clone();
        let second_service_shared = second.shared.clone();
        let (second_ready, ready_requested) = oneshot::channel();
        let (initialized, initialization) = oneshot::channel();
        second.initialized = Some(initialization);
        let (stop, stopping) = oneshot::channel();
        second.stop = Some(stop);
        let second_service: ServiceFuture = Box::pin(async move {
            let _close_on_exit = CloseOnExit(second_service_shared);
            let _ = ready_requested.await;
            let _ = initialized.send(Ok(()));
            let _ = stopping.await;
            Ok(())
        });

        let mut group = HostGroup::new()
            .with_host(ControlledWebHost {
                name: "first",
                activity: ControlledWebActivity {
                    web: first,
                    service: Some(first_service),
                    opened: first_opened.clone(),
                    admitted: None,
                },
                released: first_released.clone(),
            })
            .with_host(ControlledWebHost {
                name: "second",
                activity: ControlledWebActivity {
                    web: second,
                    service: Some(second_service),
                    opened: second_opened.clone(),
                    admitted: None,
                },
                released: second_released.clone(),
            })
            .build()
            .unwrap();

        tokio::time::timeout(
            Duration::from_secs(2),
            poll_fn(|cx| {
                assert!(group.poll_start(cx).is_pending());
                if first_shared.ready.load(Ordering::Acquire) {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }),
        )
        .await
        .unwrap();
        assert!(!second_shared.ready.load(Ordering::Acquire));
        assert!(!first_opened.load(Ordering::Acquire));
        assert!(!second_opened.load(Ordering::Acquire));

        fail.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), observed_exit)
            .await
            .unwrap()
            .unwrap();
        assert!(first_shared.is_closed());
        second_ready.send(()).unwrap();

        // The second Host now becomes ready while the original first thread is
        // still in TLS destruction. Cancel only this borrowing drive after one
        // rollback poll; the group must keep the exact Join and dependencies.
        poll_fn(|cx| {
            let mut drive = Box::pin(group.run_async());
            assert!(drive.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert!(first_shared.is_closed());
        assert!(second_shared.is_closed());
        assert!(!first_opened.load(Ordering::Acquire));
        assert!(!second_opened.load(Ordering::Acquire));
        assert!(!first_released.load(Ordering::Acquire));
        assert!(!second_released.load(Ordering::Acquire));
        assert!(group.failures().primary().is_none());

        release_exit.send(()).unwrap();
        let failure = tokio::time::timeout(Duration::from_secs(2), group.run_async())
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(failure.host, "first");
        assert_eq!(failure.operation, HostOperation::Join);
        assert!(std::ptr::eq(
            failure.error().io_error(),
            original.io_error()
        ));
        assert!(retained_at_exit.load(Ordering::Acquire));
        assert!(first_released.load(Ordering::Acquire));
        assert!(second_released.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn owned_runtime_teardown_wakes_group_before_blocking_work_finishes() {
        tokio::task::LocalSet::new()
            .run_until(async {
                let first_released = Arc::new(AtomicBool::new(false));
                let second_released = Arc::new(AtomicBool::new(false));
                let retained_during_teardown = Arc::new(AtomicBool::new(false));
                let blocking_finished = Arc::new(AtomicBool::new(false));
                let (release_blocking, held_blocking) = mpsc::channel();

                let mut first = WebActivity::new(
                    WebHost::new(Router::new()).with_driver(WebDriver::OwnedThread),
                );
                let first_shared = first.shared.clone();
                let first_service_shared = first.shared.clone();
                let (first_ready, initialization) = oneshot::channel();
                first.initialized = Some(initialization);
                let (fail, failure_requested) = oneshot::channel();
                let original = WebError::message(
                    io::ErrorKind::ConnectionAborted,
                    "original failure before owned runtime teardown",
                );
                let failure = original.clone();
                let dependency_released = first_released.clone();
                let observed_retention = retained_during_teardown.clone();
                let observed_blocking_finish = blocking_finished.clone();
                let first_service: ServiceFuture = Box::pin(async move {
                    let _close_on_exit = CloseOnExit(first_service_shared);
                    let (entered, blocking_started) = oneshot::channel();
                    let _blocking = tokio::task::spawn_blocking(move || {
                        let _ = entered.send(());
                        // Sender drop also releases runtime teardown if setup or
                        // a prior assertion fails on the controlling test task.
                        let _ = held_blocking.recv();
                        observed_retention.store(
                            !dependency_released.load(Ordering::Acquire),
                            Ordering::Release,
                        );
                        observed_blocking_finish.store(true, Ordering::Release);
                    });
                    let _ = blocking_started.await;
                    let _ = first_ready.send(Ok(()));
                    let _ = failure_requested.await;
                    // Runtime Drop must await the held blocking task after this
                    // exact service outcome, without delaying group notification.
                    Err(failure)
                });

                let mut second = WebActivity::new(
                    WebHost::new(Router::new()).with_driver(WebDriver::CallerRuntime),
                );
                let second_shared = second.shared.clone();
                let second_service_shared = second.shared.clone();
                let (second_ready, initialization) = oneshot::channel();
                second.initialized = Some(initialization);
                let (stop, stopping) = oneshot::channel();
                second.stop = Some(stop);
                let (stopped, sibling_stopped) = oneshot::channel();
                let second_service: ServiceFuture = Box::pin(async move {
                    let _close_on_exit = CloseOnExit(second_service_shared.clone());
                    let _ = second_ready.send(Ok(()));
                    let _ = stopping.await;
                    let _ = stopped.send(second_service_shared.is_closed());
                    Ok(())
                });
                let (admitted, admission_observed) = oneshot::channel();
                let mut group = HostGroup::new()
                    .with_host(ControlledWebHost {
                        name: "first",
                        activity: ControlledWebActivity {
                            web: first,
                            service: Some(first_service),
                            opened: Arc::new(AtomicBool::new(false)),
                            admitted: None,
                        },
                        released: first_released.clone(),
                    })
                    .with_host(ControlledWebHost {
                        name: "second",
                        activity: ControlledWebActivity {
                            web: second,
                            service: Some(second_service),
                            opened: Arc::new(AtomicBool::new(false)),
                            admitted: Some(admitted),
                        },
                        released: second_released.clone(),
                    })
                    .build()
                    .unwrap();

                // Independently schedule the drive: notification timeouts on this
                // controlling task cannot poll the group or supply its missing wake.
                let drive = tokio::task::spawn_local(async move {
                    let failure = group.run_async().await.unwrap_err();
                    assert_eq!(failure.host, "first");
                    assert_eq!(failure.operation, HostOperation::Join);
                    failure.error().clone()
                });
                tokio::time::timeout(Duration::from_secs(2), admission_observed)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(first_shared.admission.load(Ordering::SeqCst), ADMITTING);
                assert_eq!(second_shared.admission.load(Ordering::SeqCst), ADMITTING);

                fail.send(()).unwrap();
                let stopped_before_release =
                    tokio::time::timeout(Duration::from_secs(2), sibling_stopped).await;
                let both_closed = first_shared.is_closed() && second_shared.is_closed();
                let drive_pending = !drive.is_finished();
                let work_pending = !blocking_finished.load(Ordering::Acquire);
                let dependencies_retained = !first_released.load(Ordering::Acquire)
                    && !second_released.load(Ordering::Acquire);

                // Unblock and join even when the wake/fence observation timed out,
                // so the regression's failure path does not strand runtime teardown.
                release_blocking.send(()).unwrap();
                let observed = tokio::time::timeout(Duration::from_secs(2), drive)
                    .await
                    .unwrap()
                    .unwrap();
                assert!(
                    matches!(&stopped_before_release, Ok(Ok(true))),
                    "sibling must stop behind its fence before runtime teardown is released: {stopped_before_release:?}"
                );
                assert!(both_closed);
                assert!(drive_pending);
                assert!(work_pending);
                assert!(dependencies_retained);
                assert!(std::ptr::eq(observed.io_error(), original.io_error()));
                assert!(retained_during_teardown.load(Ordering::Acquire));
                assert!(blocking_finished.load(Ordering::Acquire));
                assert!(first_released.load(Ordering::Acquire));
                assert!(second_released.load(Ordering::Acquire));
            })
            .await;
    }

    #[tokio::test]
    async fn closed_status_waits_for_the_original_owned_failure() {
        let mut activity =
            WebActivity::new(WebHost::new(Router::new()).with_driver(WebDriver::OwnedThread));
        activity.prepare_driver(HostRunMode::Async).unwrap();
        activity.launched = true;
        activity.bound_observed = true;
        let original =
            WebError::message(io::ErrorKind::ConnectionAborted, "original owned failure");
        let failure = original.clone();
        let shared = activity.shared.clone();
        let (close, closing) = oneshot::channel();
        let (closed, observed_closed) = oneshot::channel();
        let (release, held) = oneshot::channel();
        let future: ServiceFuture = Box::pin(async move {
            let _ = closing.await;
            shared.close();
            let _ = closed.send(());
            let _ = held.await;
            Err(failure)
        });
        activity.owner =
            Some(DriverOwner::new(WebDriver::OwnedThread, HostRunMode::Async, future).unwrap());
        activity.shared.ready.store(true, Ordering::Release);
        poll_fn(|cx| {
            assert!(activity.poll_finished(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        close.send(()).unwrap();
        observed_closed.await.unwrap();
        // Closure after a completed liveness check is a terminal gate outcome,
        // not an admission error that can supersede the original failed Join.
        assert_eq!(activity.open_admission().unwrap(), AdmissionOutcome::Closed);
        poll_fn(|cx| {
            assert!(activity.poll_finished(cx).is_ready());
            assert!(activity.poll_ready(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert!(activity.owner.is_some());
        release.send(()).unwrap();
        let observed = tokio::time::timeout(
            Duration::from_secs(2),
            poll_fn(|cx| activity.poll_ready(cx)),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(std::ptr::eq(observed.io_error(), original.io_error()));
        assert!(activity.owner.is_none());
        let mut cx = Context::from_waker(std::task::Waker::noop());
        let WorkerCompletion::Joined(Err(joined)) =
            activity.poll_join(&mut cx, Duration::ZERO).unwrap()
        else {
            panic!("original failed completion must be retained");
        };
        assert!(std::ptr::eq(joined.io_error(), original.io_error()));
        assert!(matches!(
            activity.poll_release(&mut cx, Duration::ZERO),
            Ok(ReleaseCompletion::Released(Ok(())))
        ));
    }

    #[test]
    fn inert_addresses_allow_dns_and_only_claim_canonical_nonzero_socket_addresses() {
        let host = WebHost::new(Router::new());
        assert_eq!(host.address(), "127.0.0.1:7878");
        assert!(host.handle().local_addr().is_none());
        assert_eq!(host.exclusive_claims().len(), 1);
        assert!(
            WebHost::new(Router::new())
                .with_address("localhost:0")
                .validate()
                .is_ok()
        );
        assert!(
            WebHost::new(Router::new())
                .with_address("localhost:7878")
                .exclusive_claims()
                .is_empty()
        );
        assert!(
            WebHost::new(Router::new())
                .with_address("127.0.0.1:0")
                .exclusive_claims()
                .is_empty()
        );
        assert!(
            WebHost::new(Router::new())
                .with_address("localhost:no-port")
                .validate()
                .is_err()
        );
    }
}
