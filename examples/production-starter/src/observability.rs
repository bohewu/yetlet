//! Optional, application-owned collection. No default registry or exporter tasks.
use prometheus::{
    Encoder, Histogram, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};

pub struct Metrics {
    registry: Registry,
    pub(crate) http_requests: IntCounterVec,
    pub(crate) http_duration: HistogramVec,
    pub(crate) http_active: IntGauge,
    pub(crate) http_rejections: IntCounterVec,
    pub(crate) sql_admission: IntCounterVec,
    pub(crate) sql_wait: Histogram,
    pub(crate) sql_active: IntGauge,
    pub(crate) sql_operations: IntCounterVec,
    pub(crate) readiness: IntGauge,
    pub(crate) readiness_checks: IntCounterVec,
}

impl Metrics {
    /// Each instance owns an isolated registry; construction is explicitly opt-in.
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        let http_requests = IntCounterVec::new(
            Opts::new(
                "starter_http_requests_total",
                "Responses created, excluding the metrics endpoint",
            ),
            &["method", "status"],
        )?;
        let http_duration = HistogramVec::new(
            HistogramOpts::new(
                "starter_http_request_duration_seconds",
                "Time to response creation, excluding the metrics endpoint",
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 60.0,
            ]),
            &["method"],
        )?;
        let http_active = IntGauge::new(
            "starter_http_requests_active",
            "Observed handler futures, including rejected requests; excludes metrics",
        )?;
        let http_rejections = IntCounterVec::new(
            Opts::new(
                "starter_http_admission_rejections_total",
                "HTTP admission and response deadline failures",
            ),
            &["reason"],
        )?;
        let sql_admission = IntCounterVec::new(
            Opts::new(
                "starter_sql_admission_total",
                "Immediate SQL permit acquisition attempts",
            ),
            &["outcome"],
        )?;
        let sql_wait = Histogram::with_opts(
            HistogramOpts::new("starter_sql_admission_wait_seconds", "Permit queue wait per attempt; always zero for try-acquire, not SQLite lock or blocking-pool wait")
                .buckets(vec![0.0]),
        )?;
        let sql_active = IntGauge::new(
            "starter_sql_operations_active",
            "Admitted SQL closures until connection close, including detached and blocking-pool queued work",
        )?;
        let sql_operations = IntCounterVec::new(
            Opts::new(
                "starter_sql_operations_total",
                "Finished SQL operations by bounded result; not a durable write or audit count",
            ),
            &["outcome"],
        )?;
        let readiness = IntGauge::new(
            "starter_readiness_last_probe",
            "Last completed readiness probe: -1 unknown, 0 not ready, 1 ready; not continuous health",
        )?;
        readiness.set(-1);
        let readiness_checks = IntCounterVec::new(
            Opts::new(
                "starter_readiness_checks_total",
                "Completed readiness probes",
            ),
            &["outcome"],
        )?;
        for collector in [
            Box::new(http_requests.clone()) as Box<dyn prometheus::core::Collector>,
            Box::new(http_duration.clone()),
            Box::new(http_active.clone()),
            Box::new(http_rejections.clone()),
            Box::new(sql_admission.clone()),
            Box::new(sql_wait.clone()),
            Box::new(sql_active.clone()),
            Box::new(sql_operations.clone()),
            Box::new(readiness.clone()),
            Box::new(readiness_checks.clone()),
        ] {
            registry.register(collector)?;
        }
        Ok(Self {
            registry,
            http_requests,
            http_duration,
            http_active,
            http_rejections,
            sql_admission,
            sql_wait,
            sql_active,
            sql_operations,
            readiness,
            readiness_checks,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, prometheus::Error> {
        let mut bytes = Vec::new();
        TextEncoder::new().encode(&self.registry.gather(), &mut bytes)?;
        Ok(bytes)
    }
}

// RAII ties active counts to the actual future/closure lifetime, including drop.
pub(crate) struct Active(pub(crate) IntGauge);
impl Active {
    pub(crate) fn new(gauge: &IntGauge) -> Self {
        gauge.inc();
        Self(gauge.clone())
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.0.dec();
    }
}
