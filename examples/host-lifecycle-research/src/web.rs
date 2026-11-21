use crate::{
    Activity, Host, Managed, ManagedResource, ResourceGroup, ResourcePlan, Retained,
    WorkerCompletion,
};
use axum::{
    Router,
    extract::Request,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use yetlet::web::HttpWorker;

#[derive(Default)]
struct WebState {
    endpoint: OnceLock<SocketAddr>,
    admission: AtomicBool,
    probing: AtomicBool,
    joined: AtomicBool,
}

/// Metadata only: cloning this handle never clones a worker or keeps it alive.
#[derive(Clone)]
pub struct WebStatus(Arc<WebState>);

impl WebStatus {
    /// Last acquired address; its presence is not a readiness or liveness claim.
    pub fn endpoint(&self) -> Option<SocketAddr> {
        self.0.endpoint.get().copied()
    }
    pub fn joined(&self) -> bool {
        self.0.joined.load(Ordering::Acquire)
    }
    pub fn admission_open(&self) -> bool {
        self.0.admission.load(Ordering::Acquire)
    }
}

/// Inert Web description. Default bind is loopback port 0; GET / must return 200.
/// The probe route must be read-only. During startup only its GET requests may
/// pass the closed admission gate. No hidden health route is added.
pub struct WebHost<D = ()> {
    group: ResourceGroup<WebActivity, D>,
}

impl WebHost {
    pub fn new(router: Router) -> Self {
        let state = Arc::new(WebState::default());
        Self {
            group: ResourceGroup::new(
                "web",
                WebActivity {
                    router: Some(router),
                    bind: (Ipv4Addr::LOCALHOST, 0).into(),
                    probe: "/".into(),
                    readiness_budget: Duration::from_secs(2),
                    worker: None,
                    state,
                },
            ),
        }
    }
}

impl<D> WebHost<D> {
    /// Add an inert managed resource while preserving every default Web obligation.
    pub fn with_managed_resource<R: ManagedResource>(
        self,
        label: &'static str,
        resource: R,
    ) -> WebHost<Managed<R, D>> {
        WebHost {
            group: self.group.with_managed_resource(label, resource),
        }
    }

    pub fn named(mut self, label: &'static str) -> Self {
        self.group.label = label;
        self
    }

    pub fn bind(mut self, address: SocketAddr) -> Self {
        self.group.activity.bind = address;
        self
    }

    pub fn readiness_probe(mut self, path: impl Into<String>, budget: Duration) -> Self {
        self.group.activity.probe = path.into();
        self.group.activity.readiness_budget = budget;
        self
    }

    pub fn status(&self) -> WebStatus {
        WebStatus(self.group.activity.state.clone())
    }

    /// Augment the default ResourceGroup without replacing the Web activity.
    pub fn with_resource<R>(self, value: R) -> WebHost<Retained<R, D>> {
        WebHost {
            group: self.group.with_resource(value),
        }
    }
}

impl<D: ResourcePlan> Host for WebHost<D> {
    type Activity = WebActivity;
    type Resources = D;

    fn validate(&self) -> io::Result<()> {
        let activity = &self.group.activity;
        let path = &activity.probe;
        if !path.starts_with('/')
            || path.len() > 128
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
            || activity.readiness_budget.is_zero()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid Web readiness probe",
            ));
        }
        Ok(())
    }

    fn into_resources(self) -> ResourceGroup<WebActivity, D> {
        self.group
    }
}

/// Concrete adapter storage, exposed only because it appears in Host's typed
/// associated resource group. It cannot be independently constructed.
pub struct WebActivity {
    router: Option<Router>,
    bind: SocketAddr,
    probe: String,
    readiness_budget: Duration,
    worker: Option<HttpWorker>,
    state: Arc<WebState>,
}

impl Activity for WebActivity {
    fn start(&mut self) -> io::Result<()> {
        let router = self
            .router
            .take()
            .ok_or_else(|| io::Error::other("Web start already attempted"))?;
        let gate = self.state.clone();
        let probe = self.probe.clone();
        self.state.probing.store(true, Ordering::Release);
        let router = router.layer(middleware::from_fn(move |request: Request, next: Next| {
            let gate = gate.clone();
            let probe = probe.clone();
            async move {
                let readiness_request = gate.probing.load(Ordering::Acquire)
                    && request.method() == Method::GET
                    && request.uri().path() == probe;
                if readiness_request || gate.admission.load(Ordering::Acquire) {
                    next.run(request).await
                } else {
                    StatusCode::SERVICE_UNAVAILABLE.into_response()
                }
            }
        }));
        // HttpWorker owns bind/spawn cleanup before return. Store its successful
        // owner before any later readiness operation can fail.
        self.worker = Some(HttpWorker::start_on(router, self.bind)?);
        let worker = self.worker.as_ref().expect("acquired worker");
        let _ = self.state.endpoint.set(worker.endpoint());
        Ok(())
    }

    fn wait_ready(&mut self) -> io::Result<()> {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| io::Error::other("Web worker missing"))?
            .wait_http_ready(&self.probe, self.readiness_budget);
        self.state.probing.store(false, Ordering::Release);
        result
    }

    fn open_admission(&mut self) -> io::Result<()> {
        self.state.admission.store(true, Ordering::Release);
        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.worker.as_ref().is_some_and(HttpWorker::is_finished)
    }

    fn close_admission(&mut self) -> io::Result<()> {
        // Requests that already passed this atomic gate are admitted work and
        // may finish using retained dependencies during graceful drain.
        self.state.admission.store(false, Ordering::Release);
        self.state.probing.store(false, Ordering::Release);
        Ok(())
    }

    fn request_shutdown(&mut self) -> io::Result<()> {
        match self.worker.as_mut() {
            Some(worker) if !worker.is_finished() => worker.request_stop(),
            Some(_) => Ok(()),
            None => Ok(()),
        }
    }

    fn observe_within(&mut self, budget: Duration) -> io::Result<WorkerCompletion<io::Error>> {
        let completion = match self.worker.as_mut() {
            Some(worker) => worker.observe_within(budget)?,
            None => WorkerCompletion::Joined(Ok(())),
        };
        if self.worker.is_some() && matches!(completion, WorkerCompletion::Joined(_)) {
            self.state.joined.store(true, Ordering::Release);
        }
        Ok(completion)
    }
}
