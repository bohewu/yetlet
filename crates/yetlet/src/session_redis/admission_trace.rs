//! Private compiler-cfg research linkage, not a normal consumer API.
use crate::session_redis::RedisSessionError;
use axum::response::Response;
use serde::Serialize;
use std::{
    future::Future,
    io,
    path::Path,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

const CAPACITY: usize = 65536;
tokio::task_local! { static REQUEST: u32; }
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Event {
    Http {
        request: u32,
        start_ns: u64,
        end_ns: u64,
        status: Option<u16>,
    },
    Call {
        request: u32,
        call: u64,
        start_ns: u64,
        admitted_ns: Option<u64>,
        acquired_ns: Option<u64>,
        outcome_ns: Option<u64>,
        returned_ns: u64,
        active_free_hint: Option<usize>,
        admission_free_hint: Option<usize>,
        outcome: &'static str,
    },
    Worker {
        request: u32,
        call: u64,
        entered_ns: u64,
        reply_ns: Option<u64>,
        released_after_ns: u64,
    },
}
struct Capture {
    origin: Instant,
    next_call: AtomicU64,
    overflow: AtomicU64,
    append_ns_sum: AtomicU64,
    append_ns_max: AtomicU64,
    events: Mutex<Vec<Event>>,
}
fn capture() -> &'static Capture {
    static CAPTURE: OnceLock<Capture> = OnceLock::new();
    CAPTURE.get_or_init(|| Capture {
        origin: Instant::now(),
        next_call: AtomicU64::new(1),
        overflow: AtomicU64::new(0),
        append_ns_sum: AtomicU64::new(0),
        append_ns_max: AtomicU64::new(0),
        events: Mutex::new(Vec::with_capacity(CAPACITY)),
    })
}
fn stamp() -> u64 {
    capture()
        .origin
        .elapsed()
        .as_nanos()
        .try_into()
        .unwrap_or(u64::MAX)
}
fn append(event: Event) {
    let c = capture();
    let begin = Instant::now();
    if let Ok(mut events) = c.events.lock() {
        if events.len() < CAPACITY {
            events.push(event);
        } else {
            c.overflow.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        c.overflow.fetch_add(1, Ordering::Relaxed);
    }
    let ns = u64::try_from(begin.elapsed().as_nanos()).unwrap_or(u64::MAX);
    c.append_ns_sum.fetch_add(ns, Ordering::Relaxed);
    c.append_ns_max.fetch_max(ns, Ordering::Relaxed);
}
struct Http {
    request: u32,
    start: u64,
    status: Option<u16>,
}
impl Drop for Http {
    fn drop(&mut self) {
        append(Event::Http {
            request: self.request,
            start_ns: self.start,
            end_ns: stamp(),
            status: self.status,
        });
    }
}
/// Scope only a sanitized synthetic tag; it never supplies authentication data.
pub async fn request(id: u32, future: impl Future<Output = Response>) -> Response {
    let mut span = Http {
        request: id,
        start: stamp(),
        status: None,
    };
    let response = REQUEST.scope(id, future).await;
    span.status = Some(response.status().as_u16());
    response
}
fn category(error: &RedisSessionError) -> &'static str {
    use RedisSessionError::*;
    match error {
        Busy => "busy",
        QueueTimeout => "queue_timeout",
        DispatchDeadline => "dispatch_deadline",
        Closed => "closed",
        Unavailable => "unavailable",
        _ => "other",
    }
}
struct CallData {
    request: u32,
    call: u64,
    start: u64,
    admitted: Option<u64>,
    acquired: Option<u64>,
    outcome_at: Option<u64>,
    active_hint: Option<usize>,
    admission_hint: Option<usize>,
    outcome: &'static str,
}
pub(crate) struct Call(Option<CallData>);
#[derive(Clone, Copy)]
pub(crate) struct WorkerId {
    request: u32,
    call: u64,
}
impl Call {
    pub(crate) fn new() -> Self {
        Self(
            REQUEST
                .try_with(|id| CallData {
                    request: *id,
                    call: capture().next_call.fetch_add(1, Ordering::Relaxed),
                    start: stamp(),
                    admitted: None,
                    acquired: None,
                    outcome_at: None,
                    active_hint: None,
                    admission_hint: None,
                    outcome: "incomplete",
                })
                .ok(),
        )
    }
    pub(crate) fn admitted(&mut self, active: usize, admission: usize) {
        if let Some(v) = &mut self.0 {
            v.admitted = Some(stamp());
            v.active_hint = Some(active);
            v.admission_hint = Some(admission);
        }
    }
    pub(crate) fn acquired(&mut self) {
        if let Some(v) = &mut self.0 {
            v.acquired = Some(stamp());
        }
    }
    pub(crate) fn failed(&mut self, error: &RedisSessionError) {
        if let Some(v) = &mut self.0 {
            v.outcome = category(error);
            v.outcome_at = Some(stamp());
        }
    }
    pub(crate) fn finished(&mut self, result: &Result<redis::Value, RedisSessionError>) {
        if let Some(v) = &mut self.0 {
            v.outcome = result.as_ref().map_or_else(|e| category(e), |_| "ok");
            v.outcome_at = Some(stamp());
        }
    }
    pub(crate) fn worker(&self) -> Option<WorkerId> {
        self.0.as_ref().map(|v| WorkerId {
            request: v.request,
            call: v.call,
        })
    }
}
impl Drop for Call {
    fn drop(&mut self) {
        if let Some(v) = &self.0 {
            append(Event::Call {
                request: v.request,
                call: v.call,
                start_ns: v.start,
                admitted_ns: v.admitted,
                acquired_ns: v.acquired,
                outcome_ns: v.outcome_at,
                returned_ns: stamp(),
                active_free_hint: v.active_hint,
                admission_free_hint: v.admission_hint,
                outcome: v.outcome,
            });
        }
    }
}
pub(crate) struct Worker {
    id: Option<WorkerId>,
    entered: u64,
    reply: Option<u64>,
}
impl Worker {
    pub(crate) fn enter(id: Option<WorkerId>) -> Self {
        Self {
            entered: if id.is_some() { stamp() } else { 0 },
            id,
            reply: None,
        }
    }
    pub(crate) fn reply(&mut self) {
        if self.id.is_some() {
            self.reply = Some(stamp());
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(v) = self.id {
            append(Event::Worker {
                request: v.request,
                call: v.call,
                entered_ns: self.entered,
                reply_ns: self.reply,
                released_after_ns: stamp(),
            });
        }
    }
}
/// Called by the research fixture only after HTTP/Store drain. Refuses overwrite.
pub fn flush(path: &Path) -> io::Result<()> {
    let c = capture();
    let events = c
        .events
        .lock()
        .map_err(|_| io::Error::other("trace lock"))?;
    let report = serde_json::json!({"pid":std::process::id(),"clock":"one process-local std::time::Instant","capacity":CAPACITY,"overflow":c.overflow.load(Ordering::Relaxed),"append_ns_sum":c.append_ns_sum.load(Ordering::Relaxed),"append_ns_max":c.append_ns_max.load(Ordering::Relaxed),"events":*events,"secret_or_input_values_recorded":false});
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    serde_json::to_writer(file, &report).map_err(|_| io::Error::other("trace serialization"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn request_scope_keeps_worker_identity_and_no_ambient_leak() {
        let mut tasks = vec![];
        for id in [65001, 65002] {
            tasks.push(tokio::spawn(request(id, async {
                let mut call = Call::new();
                call.admitted(4, 7);
                call.acquired();
                let identity = call.worker();
                tokio::spawn(async move {
                    assert!(REQUEST.try_with(|_| ()).is_err());
                    let mut worker = Worker::enter(identity);
                    worker.reply();
                })
                .await
                .unwrap();
                call.finished(&Ok(redis::Value::Nil));
                Response::new(axum::body::Body::empty())
            })));
        }
        for task in tasks {
            assert_eq!(task.await.unwrap().status(), 200);
        }
        assert!(REQUEST.try_with(|_| ()).is_err());
        let rows = capture().events.lock().unwrap();
        for id in [65001, 65002] {
            assert_eq!(
                rows.iter()
                    .filter(
                        |r| matches!(r,Event::Http {request,status:Some(200),..} if *request==id)
                    )
                    .count(),
                1
            );
            let call = rows
                .iter()
                .find_map(|r| {
                    if let Event::Call {
                        request,
                        call,
                        outcome: "ok",
                        ..
                    } = r
                    {
                        (*request == id).then_some(*call)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(rows.iter().filter(|r| matches!(r,Event::Worker {request,call:worker,..} if *request==id && *worker==call)).count(),1);
        }
        let json = serde_json::to_string(&*rows).unwrap();
        for secret in ["cookie", "csrf", "tenant", "account", "state"] {
            assert!(!json.contains(secret));
        }
    }
}
