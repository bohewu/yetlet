//! Experimental, single-site ACME HTTP-01 provisioning for managed Web Hosts.
//!
//! Add both Hosts returned by [AcmeHost::http01] to the same async HostGroup.
//! Plain HTTP can serve while the HTTPS identity is being obtained.

mod issuer;

use std::{
    future::{Future, poll_fn},
    io,
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, SystemTime},
};

use axum::{
    Router,
    extract::{Path, Request, State},
    http::{StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use instant_acme::ExternalAccountKey;
use tokio::sync::{oneshot, watch};

use crate::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, ExclusiveClaim, HostActivity, HostRunMode, StartStatus, WorkerCompletion,
    },
    web::{WebError, WebHandle, WebHost, WebTls},
};

const CHALLENGE_PATH: &str = "/.well-known/acme-challenge/{token}";
const DEFAULT_RETRY: Duration = Duration::from_secs(60);
const MAX_RETRY: Duration = Duration::from_secs(24 * 60 * 60);

/// Inert configuration for one DNS name and one ACME directory.
pub struct AcmeConfig {
    domain: String,
    directory: String,
    store: PathBuf,
    directory_root: Option<PathBuf>,
    issuance_root: Option<PathBuf>,
    eab: Option<ExternalAccountKey>,
    terms_agreed: bool,
    renew_before: Option<Duration>,
    retry_initial: Duration,
    retry_max: Duration,
}

impl AcmeConfig {
    /// Configure an HTTPS ACME directory and a private, dedicated state directory.
    ///
    /// Constructing the configuration does not access the network or filesystem.
    pub fn new(
        domain: impl Into<String>,
        directory: impl Into<String>,
        store: impl Into<PathBuf>,
    ) -> Result<Self, WebError> {
        let domain = domain.into().trim_end_matches('.').to_ascii_lowercase();
        let directory = directory.into();
        let store = store.into();
        if !matches!(
            rustls::pki_types::ServerName::try_from(domain.as_str()),
            Ok(rustls::pki_types::ServerName::DnsName(_))
        ) || domain.contains('*')
        {
            return Err(invalid("ACME HTTP-01 requires a single DNS name"));
        }
        let uri: Uri = directory
            .parse()
            .map_err(|_| invalid("invalid ACME directory URL"))?;
        if uri.scheme_str() != Some("https")
            || uri
                .authority()
                .is_none_or(|authority| authority.as_str().contains('@'))
            || directory.contains('#')
            || store.as_os_str().is_empty()
        {
            return Err(invalid(
                "ACME requires an HTTPS directory and a state directory",
            ));
        }
        Ok(Self {
            domain,
            directory: uri.to_string(),
            store,
            directory_root: None,
            issuance_root: None,
            eab: None,
            terms_agreed: false,
            renew_before: None,
            retry_initial: DEFAULT_RETRY,
            retry_max: MAX_RETRY,
        })
    }

    /// Supply a private CA root for the ACME API's HTTPS connection.
    pub fn with_directory_root(mut self, root_pem: impl Into<PathBuf>) -> Self {
        self.directory_root = Some(root_pem.into());
        self
    }

    /// Supply a private CA root for verification of issued identities.
    pub fn with_issuance_root(mut self, root_pem: impl Into<PathBuf>) -> Self {
        self.issuance_root = Some(root_pem.into());
        self
    }

    /// Bind the first ACME account registration with a CA-issued key.
    ///
    /// Supply the decoded MAC key bytes. The SDK persists the resulting account
    /// credential and reuses it on restart.
    pub fn with_eab(mut self, key_id: impl Into<String>, mac_key: &[u8]) -> Self {
        self.eab = Some(ExternalAccountKey::new(key_id.into(), mac_key));
        self
    }

    /// Indicate that the application operator has accepted the CA's terms.
    pub fn with_terms_of_service_agreed(mut self) -> Self {
        self.terms_agreed = true;
        self
    }

    /// Select how far before expiry to renew a certificate. A lead as long as
    /// the certificate's lifetime uses the lifetime-based default.
    pub fn with_renew_before(mut self, lead: Duration) -> Result<Self, WebError> {
        if lead.is_zero() {
            return Err(invalid("ACME renewal lead must be positive"));
        }
        self.renew_before = Some(lead);
        Ok(self)
    }

    /// Set initial and maximum automatic retry delays, with exponential backoff.
    pub fn with_retry_policy(
        mut self,
        initial: Duration,
        maximum: Duration,
    ) -> Result<Self, WebError> {
        if initial.as_millis() == 0 || maximum < initial || maximum > MAX_RETRY {
            return Err(invalid(
                "ACME retry delays must be ordered, between 1 ms and 24 hours",
            ));
        }
        self.retry_initial = initial;
        self.retry_max = maximum;
        Ok(self)
    }
}

/// A safe category for the last unsuccessful automatic attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub enum AcmeFailure {
    Account,
    Order,
    Challenge,
    Certificate,
    Timeout,
}

/// Current certificate and retry observations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AcmeStatus {
    pub certificate_expires_at: Option<SystemTime>,
    pub last_failure: Option<AcmeFailure>,
    pub retry_at: Option<SystemTime>,
    pub https_address: Option<SocketAddr>,
}

/// Cloneable, non-secret status observer.
#[derive(Clone)]
pub struct AcmeHandle(watch::Receiver<AcmeStatus>);

impl AcmeHandle {
    pub fn status(&self) -> AcmeStatus {
        *self.0.borrow()
    }
}

type Service = Pin<Box<dyn Future<Output = Result<(), WebError>> + Send>>;
type ChallengeSlot = watch::Sender<Option<(String, String)>>;

/// The background Host and HTTPS listener for a single HTTP-01 site.
pub struct AcmeHost {
    config: AcmeConfig,
    http: WebHandle,
    https: Router,
    https_address: SocketAddr,
    challenge: ChallengeSlot,
    status: watch::Sender<AcmeStatus>,
}

impl AcmeHost {
    /// Mount HTTP-01 beside the supplied plaintext routes and manage HTTPS on
    /// the supplied address. The challenge path is reserved on `http_router`.
    ///
    /// Add both returned Hosts to one Tokio-driven HostGroup. The plaintext and
    /// HTTPS routers are explicit so applications can choose HTTPS-only routes.
    pub fn http01(
        config: AcmeConfig,
        http_router: Router,
        http_address: SocketAddr,
        https_router: Router,
        https_address: SocketAddr,
    ) -> (WebHost, Self) {
        let (challenge, _) = watch::channel(None);
        let challenge_route = Router::new()
            .route(CHALLENGE_PATH, get(http01_challenge))
            .with_state(challenge.clone());
        let http = WebHost::new(http_router.merge(challenge_route))
            .with_name("acme-http")
            .with_address(http_address);
        let (status, _) = watch::channel(AcmeStatus::default());
        let acme = Self {
            config,
            http: http.handle(),
            https: https_router,
            https_address,
            challenge,
            status,
        };
        (http, acme)
    }

    pub fn handle(&self) -> AcmeHandle {
        AcmeHandle(self.status.subscribe())
    }

    async fn run(
        self,
        gate: Arc<AtomicBool>,
        mut shutdown: watch::Receiver<()>,
    ) -> Result<(), WebError> {
        tokio::select! { biased;
            _ = shutdown.changed() => return Ok(()),
            _ = async {
                while !self.http.is_admitting() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            } => {}
        }
        let (identity, initial) = oneshot::channel();
        let (stop, stopping) = watch::channel(());
        let order = issuer::run(
            self.config,
            self.challenge,
            self.status.clone(),
            identity,
            stopping.clone(),
        );
        let serve = serve_https(
            initial,
            self.https,
            self.https_address,
            self.status.clone(),
            gate.clone(),
            stopping,
        );
        tokio::pin!(order, serve);
        let (result, order_done, serve_done) = tokio::select! { biased;
            _ = shutdown.changed() => (Ok(()), false, false),
            result = &mut order => (result, true, false),
            result = &mut serve => (result, false, true),
        };
        gate.store(false, Ordering::Release);
        drop(stop);
        let (ordered, served) = tokio::join!(
            async { if order_done { Ok(()) } else { order.await } },
            async { if serve_done { Ok(()) } else { serve.await } },
        );
        self.status
            .send_modify(|status| status.https_address = None);
        result.and(ordered).and(served)
    }
}

async fn http01_challenge(
    State(challenge): State<ChallengeSlot>,
    Path(token): Path<String>,
) -> Response {
    match &*challenge.borrow() {
        Some((expected, key)) if *expected == token => {
            (StatusCode::OK, key.clone()).into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_https(
    initial: oneshot::Receiver<WebTls>,
    router: Router,
    address: SocketAddr,
    status: watch::Sender<AcmeStatus>,
    gate: Arc<AtomicBool>,
    mut stop: watch::Receiver<()>,
) -> Result<(), WebError> {
    let tls = tokio::select! { biased;
        _ = stop.changed() => return Ok(()),
        tls = initial => tls.map_err(|_| io::Error::other("ACME initial identity owner stopped"))?,
    };
    let router = router.layer(middleware::from_fn_with_state(
        gate,
        |State(gate): State<Arc<AtomicBool>>, req: Request, next: Next| async move {
            if gate.load(Ordering::Acquire) {
                next.run(req).await
            } else {
                StatusCode::SERVICE_UNAVAILABLE.into_response()
            }
        },
    ));
    let host = WebHost::new(router).with_address(address).with_tls(tls);
    let handle = host.handle();
    let mut group = HostGroup::new()
        .with_host(host)
        .build()
        .map_err(|error| io::Error::other(error.to_string()))?;
    let shutdown = group.shutdown_handle();
    let drive = async {
        if poll_fn(|cx| group.poll_start(cx)).await == StartStatus::Running {
            status.send_modify(|s| s.https_address = handle.local_addr());
        }
        group
            .run_async()
            .await
            .map_err(|error| io::Error::other(error.to_string()).into())
    };
    tokio::pin!(drive);
    tokio::select! { biased;
        _ = stop.changed() => {
            shutdown.request_shutdown();
            drive.await
        }
        result = &mut drive => result,
    }
}

impl Host for AcmeHost {
    type Error = WebError;
    type Activity = AcmeActivity;
    type Resources = ();

    fn name(&self) -> &'static str {
        "acme"
    }

    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        let mut claims = vec![ExclusiveClaim::new(
            "acme-store",
            self.config.store.to_string_lossy().into_owned(),
        )];
        if self.https_address.port() != 0 {
            claims.push(ExclusiveClaim::new(
                "tcp-listener",
                self.https_address.to_string(),
            ));
        }
        claims
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity> {
        let (stop, _) = watch::channel(());
        ResourceGroup::new(AcmeActivity {
            host: Some(self),
            service: None,
            result: None,
            gate: Arc::new(AtomicBool::new(false)),
            stop: Some(stop),
        })
    }
}

/// Retained lifetime of an [AcmeHost].
pub struct AcmeActivity {
    host: Option<AcmeHost>,
    service: Option<Service>,
    result: Option<Result<(), WebError>>,
    gate: Arc<AtomicBool>,
    stop: Option<watch::Sender<()>>,
}

impl AcmeActivity {
    fn stop(&mut self) {
        self.gate.store(false, Ordering::Release);
        self.stop.take();
    }
}

impl HostActivity for AcmeActivity {
    type Error = WebError;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        if mode != HostRunMode::Async || tokio::runtime::Handle::try_current().is_err() {
            return Err(invalid(
                "ACME requires an async HostGroup on a Tokio runtime",
            ));
        }
        Ok(())
    }

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, Self::Error> {
        let Some(stop) = &self.stop else {
            return Ok(AdmissionOutcome::Closed);
        };
        let host = self
            .host
            .take()
            .ok_or_else(|| invalid("ACME admission already opened"))?;
        self.gate.store(true, Ordering::Release);
        self.service = Some(Box::pin(host.run(self.gate.clone(), stop.subscribe())));
        Ok(AdmissionOutcome::Opened)
    }

    fn close_admission(&mut self, _: &mut Context<'_>) -> Result<(), Self::Error> {
        self.stop();
        Ok(())
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), Self::Error> {
        self.stop();
        Ok(())
    }

    fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(service) = &mut self.service {
            if let Poll::Ready(result) = service.as_mut().poll(cx) {
                self.service = None;
                self.result = Some(result);
                self.gate.store(false, Ordering::Release);
                return Poll::Ready(());
            }
            return Poll::Pending;
        }
        if self.result.is_some() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn poll_join(
        &mut self,
        cx: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<Self::Error>, Self::Error> {
        let _ = self.poll_finished(cx);
        if self.service.is_some() {
            return Ok(WorkerCompletion::Pending);
        }
        Ok(WorkerCompletion::Joined(
            self.result.take().unwrap_or(Ok(())),
        ))
    }
}

fn invalid(message: &'static str) -> WebError {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
