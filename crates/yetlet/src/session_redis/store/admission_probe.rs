//! Ignored research only; no probe is compiled into the production provider.
use super::*;
use serde::Serialize;
use std::sync::atomic::AtomicU64;

const CAPACITY: usize = 1024;
#[derive(Serialize)]
struct Record {
    wave: u64,
    sequence: u64,
    start_ns: u64,
    admitted_ns: Option<u64>,
    acquired_ns: Option<u64>,
    returned_ns: u64,
    active_free_before_wait: Option<usize>,
    outcome: &'static str,
}
pub(super) struct Probe {
    origin: std::time::Instant,
    wave: AtomicU64,
    sequence: AtomicU64,
    overflow: AtomicU64,
    append_ns_sum: AtomicU64,
    append_ns_max: AtomicU64,
    records: Mutex<Vec<Record>>,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            origin: std::time::Instant::now(),
            wave: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
            overflow: AtomicU64::new(0),
            append_ns_sum: AtomicU64::new(0),
            append_ns_max: AtomicU64::new(0),
            records: Mutex::new(Vec::with_capacity(CAPACITY)),
        }
    }
}
fn category(error: &Error) -> &'static str {
    match error {
        Error::Busy => "busy",
        Error::QueueTimeout => "queue_timeout",
        Error::DispatchDeadline => "dispatch_deadline",
        Error::Closed => "closed",
        Error::Unavailable => "unavailable",
        _ => "other",
    }
}
impl Probe {
    fn now(&self) -> u64 {
        self.origin
            .elapsed()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX)
    }
    pub(super) fn start(&self) -> Guard<'_> {
        let wave = self.wave.load(Ordering::Relaxed);
        Guard {
            owner: self,
            record: Record {
                wave,
                sequence: if wave == 0 {
                    0
                } else {
                    self.sequence.fetch_add(1, Ordering::Relaxed)
                },
                start_ns: self.now(),
                admitted_ns: None,
                acquired_ns: None,
                returned_ns: 0,
                active_free_before_wait: None,
                outcome: "incomplete",
            },
        }
    }
}
pub(super) struct Guard<'a> {
    owner: &'a Probe,
    record: Record,
}
impl Guard<'_> {
    pub(super) fn admitted(&mut self, active_free: usize) {
        self.record.admitted_ns = Some(self.owner.now());
        self.record.active_free_before_wait = Some(active_free);
    }
    pub(super) fn acquired(&mut self) {
        self.record.acquired_ns = Some(self.owner.now());
    }
    pub(super) fn failed(&mut self, error: &Error) {
        self.record.outcome = category(error);
    }
    pub(super) fn finished(&mut self, result: &Result<redis::Value, Error>) {
        self.record.outcome = result.as_ref().map_or_else(|e| category(e), |_| "ok");
    }
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if self.record.wave == 0 {
            return;
        }
        self.record.returned_ns = self.owner.now();
        let append_start = std::time::Instant::now();
        let mut records = self.owner.records.lock().expect("private probe lock");
        if records.len() == CAPACITY {
            self.owner.overflow.fetch_add(1, Ordering::Relaxed);
        } else {
            records.push(Record { ..self.record });
        }
        drop(records);
        let append_ns = u64::try_from(append_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.owner
            .append_ns_sum
            .fetch_add(append_ns, Ordering::Relaxed);
        self.owner
            .append_ns_max
            .fetch_max(append_ns, Ordering::Relaxed);
    }
}
#[derive(Clone, Copy, Serialize)]
struct Wave {
    wave: u64,
    round: usize,
    size: usize,
    spacing_us: u64,
}
async fn measure(
    store: &Store,
    plan: &[Wave],
    outcomes: &mut Vec<serde_json::Value>,
) -> Result<(), &'static str> {
    let expected = serde_json::json!({"account":1});
    let mut fixtures = vec![];
    for _ in 0..8 {
        let ticket = store
            .prepare("create", None, 0, &expected, Instant::now() + CALL_TIMEOUT)
            .await
            .map_err(|_| "fixture_prepare")?;
        let mutation = store
            .execute(&ticket, Instant::now() + CALL_TIMEOUT)
            .await
            .map_err(|_| "fixture_create")?;
        if mutation.status != "ok" {
            return Err("fixture_status");
        }
        fixtures.push((ticket.identity().to_owned(), mutation.revision));
    }
    for wave in plan {
        store.inner.probe.wave.store(wave.wave, Ordering::Relaxed);
        let barrier = Arc::new(tokio::sync::Barrier::new(wave.size + 1));
        let anchor = Arc::new(std::sync::OnceLock::new());
        let mut handles = vec![];
        for slot in 0..wave.size {
            let store = store.clone();
            let barrier = barrier.clone();
            let anchor = anchor.clone();
            let (id, revision) = fixtures[slot % 8].clone();
            let expected = expected.clone();
            let spacing = wave.spacing_us;
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                if spacing != 0 && slot != 0 {
                    tokio::time::sleep_until(
                        *anchor.get().expect("wave anchor")
                            + Duration::from_micros(spacing * slot as u64),
                    )
                    .await;
                }
                match store.read(&id, Instant::now() + CALL_TIMEOUT).await {
                    Ok(value)
                        if value.state == expected
                            && value.revision == revision
                            && valid_id(&value.csrf) =>
                    {
                        "ok"
                    }
                    Ok(_) => "invalid_snapshot",
                    Err(error) => category(&error),
                }
            }));
        }
        anchor.set(Instant::now()).map_err(|_| "anchor_reuse")?;
        barrier.wait().await;
        let mut results = vec![];
        let mut failed = false;
        for handle in handles {
            match handle.await {
                Ok(outcome) => {
                    failed |= !matches!(outcome, "ok" | "busy" | "queue_timeout");
                    results.push(outcome);
                }
                Err(_) => {
                    failed = true;
                    results.push("task_failure");
                }
            }
        }
        timeout(Duration::from_secs(1), async {
            while store.inner.active.available_permits() != 4
                || store.inner.admission.available_permits() != 8
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .map_err(|_| "wave_not_quiescent")?;
        outcomes.push(serde_json::json!({"wave":wave.wave,"outcomes":results,"active_free_after":4,"admission_free_after":8}));
        if failed {
            return Err("unexpected_read_or_task_failure");
        }
    }
    // Final plan entry is eight size-one waves: read serially on the original Store.
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "YT-172: one explicitly selected, disposable Valkey research fixture"]
async fn session_admission_burst_research() {
    let endpoint = std::env::var("YT172_SESSION_URL").expect("private fixture endpoint");
    let output = std::env::var("YT172_OUTPUT").expect("excluded research output");
    let config = RedisSessionConfig::new(
        random_id().expect("random namespace"),
        "yt172",
        Duration::from_secs(300),
        4,
        256,
    )
    .expect("fixed config");
    let store = Store::connect(&endpoint, config)
        .await
        .expect("owned fixture connect");
    let mut plan = vec![];
    for round in 0..8 {
        for position in 0..8 {
            let case = (position + round) % 8;
            plan.push(Wave {
                wave: plan.len() as u64 + 1,
                round,
                size: [4, 8, 12, 16][case % 4],
                spacing_us: if case < 4 { 0 } else { 1000 },
            });
        }
    }
    for _ in 0..8 {
        plan.push(Wave {
            wave: plan.len() as u64 + 1,
            round: 8,
            size: 1,
            spacing_us: 0,
        });
    }
    let mut outcomes = vec![];
    let begin = std::time::Instant::now();
    let result = measure(&store, &plan, &mut outcomes).await;
    store.inner.probe.wave.store(0, Ordering::Relaxed);
    let drain = store.shutdown().await;
    let elapsed = begin.elapsed().as_nanos();
    let records = store
        .inner
        .probe
        .records
        .lock()
        .expect("private probe lock");
    let overflow = store.inner.probe.overflow.load(Ordering::Relaxed);
    let recovery_ok = outcomes
        .iter()
        .filter(|v| v["wave"].as_u64().unwrap_or(0) > 64)
        .all(|v| v["outcomes"] == serde_json::json!(["ok"]))
        && outcomes.len() == 72;
    let report = serde_json::json!({"task":"YT-172","test_build_only":true,"plan":plan,"outcomes":outcomes,"records":*records,"capture_overflow":overflow,"append_ns_sum":store.inner.probe.append_ns_sum.load(Ordering::Relaxed),"append_ns_max":store.inner.probe.append_ns_max.load(Ordering::Relaxed),"elapsed_ns_including_setup_and_drain":elapsed,"running_workers_peak":store.inner.peak.load(Ordering::SeqCst),"running_workers_after_drain":store.inner.running.load(Ordering::SeqCst),"drain_reaped":drain.as_ref().ok(),"result":result.as_ref().err(),"recovery_ok":recovery_ok,"state_or_credentials_retained":false});
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&report).expect("redacted report"),
    )
    .expect("write excluded report");
    assert!(
        result.is_ok(),
        "bounded research failed; partial redacted records retained"
    );
    assert!(
        drain.is_ok() && overflow == 0 && records.len() == 648 && recovery_ok,
        "drain/capture/recovery failed; records retained"
    );
}
