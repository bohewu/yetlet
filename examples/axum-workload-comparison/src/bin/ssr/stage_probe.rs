//! Sample-only YT-157 observer; no product API or response header changes.
#[cfg(feature = "ssr-thread-cpu-probe")]
#[path = "thread_cpu.rs"]
mod thread_cpu;

use super::*;
use axum::{
    Extension,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

const CAP: usize = 1024;
#[derive(Default)]
struct Creation {
    ns: u128,
    #[cfg(feature = "ssr-thread-cpu-probe")]
    construction_cpu: Option<thread_cpu::Measurement>,
    #[cfg(feature = "ssr-thread-cpu-probe")]
    empty_cpu: Option<thread_cpu::Measurement>,
}
#[derive(Clone, Default)]
struct Sample(Option<Arc<Mutex<Creation>>>);
#[derive(serde::Serialize)]
struct Record {
    phase: &'static str,
    id: u64,
    entry_tick_ns: u128,
    pipeline_ns: u128,
    construction_ns: u128,
    #[cfg(feature = "ssr-thread-cpu-probe")]
    construction_cpu: Option<thread_cpu::Measurement>,
    #[cfg(feature = "ssr-thread-cpu-probe")]
    empty_cpu: Option<thread_cpu::Measurement>,
}
pub(super) struct Probe {
    epoch: Instant,
    records: Mutex<Vec<Record>>,
    overflow: AtomicUsize,
    invalid_ids: AtomicUsize,
    #[cfg(feature = "ssr-thread-cpu-probe")]
    thread_cpu_resolution_ns: u64,
}
impl Probe {
    pub(super) fn new() -> Self {
        Self {
            epoch: Instant::now(),
            records: Mutex::new(Vec::with_capacity(CAP)),
            overflow: AtomicUsize::new(0),
            invalid_ids: AtomicUsize::new(0),
            #[cfg(feature = "ssr-thread-cpu-probe")]
            thread_cpu_resolution_ns: thread_cpu::resolution_ns()
                .expect("thread CPU clock unavailable"),
        }
    }
    pub(super) fn dump(&self) {
        let records = self.records.lock().expect("probe record lock");
        for record in records.iter() {
            println!(
                "yt157_record {}",
                serde_json::to_string(record).expect("finite record")
            );
        }
        #[cfg(not(feature = "ssr-thread-cpu-probe"))]
        println!(
            "yt157_summary {}",
            serde_json::json!({"record_count": records.len(),
            "capacity": CAP, "overflow": self.overflow.load(Ordering::Relaxed),
            "invalid_ids": self.invalid_ids.load(Ordering::Relaxed)})
        );
        #[cfg(feature = "ssr-thread-cpu-probe")]
        println!(
            "yt157_summary {}",
            serde_json::json!({"record_count": records.len(),
            "capacity": CAP, "overflow": self.overflow.load(Ordering::Relaxed),
            "invalid_ids": self.invalid_ids.load(Ordering::Relaxed),
            "thread_cpu_resolution_ns": self.thread_cpu_resolution_ns,
            "invalid_thread_records": records.iter().filter(|r|
                !r.construction_cpu.is_some_and(|m| m.status == "ok") ||
                !r.empty_cpu.is_some_and(|m| m.status == "ok")).count()})
        );
    }
}
fn request_id(req: &Request, probe: &Probe) -> Option<(&'static str, u64)> {
    let header = req.headers().get("x-yt157-id")?;
    let parsed = header
        .to_str()
        .ok()
        .and_then(|v| v.split_once(':'))
        .and_then(|(phase, id)| {
            let phase = match phase {
                "w" => "warmup",
                "m" => "measured",
                _ => return None,
            };
            if id.is_empty() || id.len() > 5 || !id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let id: u64 = id.parse().ok()?;
            (id < 20_000).then_some((phase, id))
        });
    if parsed.is_none() {
        probe.invalid_ids.fetch_add(1, Ordering::Relaxed);
    }
    parsed
}
async fn observe(State(probe): State<Arc<Probe>>, mut req: Request, next: Next) -> Response {
    let selected = request_id(&req, &probe).filter(|(_, id)| id % 32 == 0);
    let sample = Sample(selected.map(|_| Arc::new(Mutex::new(Creation::default()))));
    req.extensions_mut().insert(sample.clone());
    if let Some((phase, id)) = selected {
        let tick = probe.epoch.elapsed().as_nanos();
        let started = Instant::now();
        let response = next.run(req).await;
        let pipeline_ns = started.elapsed().as_nanos();
        #[cfg(not(feature = "ssr-thread-cpu-probe"))]
        let construction_ns = sample.0.unwrap().lock().expect("creation lock").ns;
        #[cfg(feature = "ssr-thread-cpu-probe")]
        let (construction_ns, construction_cpu, empty_cpu) = {
            let sampled = sample.0.unwrap();
            let record = sampled.lock().expect("creation lock");
            (record.ns, record.construction_cpu, record.empty_cpu)
        };
        let mut records = probe.records.lock().expect("probe record lock");
        if records.len() < CAP {
            records.push(Record {
                phase,
                id,
                entry_tick_ns: tick,
                pipeline_ns,
                construction_ns,
                #[cfg(feature = "ssr-thread-cpu-probe")]
                construction_cpu,
                #[cfg(feature = "ssr-thread-cpu-probe")]
                empty_cpu,
            });
        } else {
            probe.overflow.fetch_add(1, Ordering::Relaxed);
        }
        response
    } else {
        next.run(req).await
    }
}
async fn handler(
    Path(size): Path<String>,
    Extension(sample): Extension<Sample>,
) -> Result<Page, StatusCode> {
    let rows = CASES
        .iter()
        .find(|(name, _)| *name == size)
        .map(|(_, rows)| *rows)
        .ok_or(StatusCode::BAD_REQUEST)?;
    #[cfg(not(feature = "ssr-thread-cpu-probe"))]
    let started = sample.0.as_ref().map(|_| Instant::now());
    #[cfg(feature = "ssr-thread-cpu-probe")]
    let started = sample.0.as_ref().map(|_| thread_cpu::Begin::new());
    let (metadata, view) = page(rows);
    let result = Page::with_metadata(metadata, view);
    #[cfg(not(feature = "ssr-thread-cpu-probe"))]
    if let (Some(started), Some(record)) = (started, sample.0) {
        let ns = started.elapsed().as_nanos();
        record.lock().expect("creation lock").ns = ns;
    }
    #[cfg(feature = "ssr-thread-cpu-probe")]
    if let (Some(started), Some(record)) = (started, sample.0) {
        let construction = started.finish();
        let empty = thread_cpu::Begin::new();
        std::hint::black_box(());
        let empty = empty.finish();
        let mut record = record.lock().expect("creation lock");
        record.ns = construction.wall_ns;
        record.construction_cpu = Some(construction);
        record.empty_cpu = Some(empty);
    }
    Ok(result)
}
pub(super) fn router(probe: Arc<Probe>) -> Router {
    App::<Arc<Probe>>::new()
        .get("/ssr/{size}", handler)
        .into_router()
        .layer(middleware::from_fn_with_state(probe.clone(), observe))
        .with_state(probe)
}
#[cfg(all(test, not(feature = "ssr-thread-cpu-probe")))]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request as HttpRequest, header},
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn yt157_sampled_pipeline_keeps_exact_medium_response() {
        let probe = Arc::new(Probe::new());
        let response = router(probe.clone())
            .oneshot(
                HttpRequest::builder()
                    .uri("/ssr/medium")
                    .header("X-Yt157-Id", "m:32")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let expected = expected(128).1;
        assert_eq!(expected.len(), 31063);
        assert_eq!(response.status(), StatusCode::OK);
        let h = response.headers();
        assert_eq!(h[header::CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(h[header::CONTENT_LENGTH], "31063");
        assert_eq!(
            h[header::CONTENT_SECURITY_POLICY],
            "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'"
        );
        for forbidden in [
            "content-encoding",
            "set-cookie",
            "server",
            "server-timing",
            "x-yt157-id",
        ] {
            assert!(!h.contains_key(forbidden));
        }
        assert_eq!(
            to_bytes(response.into_body(), 32768)
                .await
                .unwrap()
                .as_ref(),
            expected.as_bytes()
        );
        let records = probe.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!((records[0].phase, records[0].id), ("measured", 32));
        assert!(
            records[0].construction_ns > 0 && records[0].construction_ns <= records[0].pipeline_ns
        );
        assert_eq!(probe.overflow.load(Ordering::Relaxed), 0);
        assert_eq!(probe.invalid_ids.load(Ordering::Relaxed), 0);
    }
}

#[cfg(all(test, feature = "ssr-thread-cpu-probe"))]
mod tests_cpu {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request as HttpRequest, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn yt159_cpu_observer_preserves_selected_and_unselected_medium_responses() {
        let probe = Arc::new(Probe::new());
        let expected = expected(128).1;
        assert_eq!(expected.len(), 31063);
        for id in [32, 33] {
            let response = router(probe.clone())
                .oneshot(
                    HttpRequest::builder()
                        .uri("/ssr/medium")
                        .header("X-Yt157-Id", format!("m:{id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let h = response.headers();
            assert_eq!(h[header::CONTENT_TYPE], "text/html; charset=utf-8");
            assert_eq!(h[header::CONTENT_LENGTH], "31063");
            assert_eq!(
                h[header::CONTENT_SECURITY_POLICY],
                "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'"
            );
            for forbidden in [
                "content-encoding",
                "set-cookie",
                "server",
                "server-timing",
                "x-yt157-id",
            ] {
                assert!(!h.contains_key(forbidden));
            }
            assert_eq!(
                to_bytes(response.into_body(), 32768)
                    .await
                    .unwrap()
                    .as_ref(),
                expected.as_bytes()
            );
        }
        let records = probe.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!((record.phase, record.id), ("measured", 32));
        assert!(0 < record.construction_ns && record.construction_ns <= record.pipeline_ns);
        for measurement in [record.construction_cpu.unwrap(), record.empty_cpu.unwrap()] {
            assert_eq!(measurement.status, "ok");
            assert!(measurement.same_thread);
            assert_eq!(measurement.thread_id_start, measurement.thread_id_end);
            assert!(
                measurement.thread_id_start != 0
                    && measurement.cpu_id_start >= 0
                    && measurement.cpu_id_end >= 0
            );
            assert!(measurement.thread_cpu_ns.unwrap() > 0);
            assert_eq!(
                measurement.wall_minus_cpu_ns,
                Some(measurement.wall_ns as i128 - measurement.thread_cpu_ns.unwrap() as i128)
            );
        }
        assert_eq!(
            record.construction_cpu.unwrap().wall_ns,
            record.construction_ns
        );
        assert!(probe.thread_cpu_resolution_ns > 0);
        assert_eq!(probe.overflow.load(Ordering::Relaxed), 0);
        assert_eq!(probe.invalid_ids.load(Ordering::Relaxed), 0);
    }
}
