//! YT-267: private B2 worker variation; not a framework Host API.
//! The B1 caller-thread path is unchanged. The composition root owns exactly
//! one std worker, while the two foreground hosts borrow a caller scheduler.

use crate::{AdmissionError, Command, Snapshot};
use std::{
    cell::Cell,
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Condvar, Mutex, mpsc},
    thread::{self, JoinHandle, ThreadId},
    time::Duration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Prepared,
    Running,
    Stopping,
    Stopped,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    SecondHostStart,
    HostRuntime,
    InjectedWorker,
    ArithmeticOverflow,
    RevisionOverflow,
    WorkerPanic,
    WorkerReadiness,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cleanup {
    Joined,
    TimedOut,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostError {
    Stopped,
    Admission(AdmissionError),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopResult {
    pub phase: Phase,
    pub failure: Option<Failure>,
    pub cleanup: Cleanup,
    pub discarded_queued: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartError {
    pub failure: Failure,
    pub first_host_stopped: bool,
    pub worker_joined: bool,
    pub ever_running: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub capacity: usize,
    pub generation: u64,
    pub fail_second_host: bool,
    pub fail_first_command: bool,
    pub pause_processing: bool,
    pub delay: Duration,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            capacity: 2,
            generation: 1,
            fail_second_host: false,
            fail_first_command: false,
            pause_processing: false,
            delay: Duration::ZERO,
        }
    }
}

struct Inbox {
    generation: u64,
    limit: usize,
    pending: VecDeque<Command>,
    discarded_on_fault: usize,
    accepting: bool,
    stopping: bool,
    paused: bool,
    fail_first: bool,
    delay: Duration,
}
struct Shared {
    inbox: Mutex<Inbox>,
    wake: Condvar,
    snapshot: Mutex<Arc<Snapshot>>,
    published: Condvar,
    in_flight: Mutex<bool>,
    work_started: Condvar,
}

#[derive(Clone)]
pub struct Ingress {
    shared: Arc<Shared>,
    generation: u64,
}
impl Ingress {
    pub fn submit(&self, command: Command) -> Result<(), AdmissionError> {
        let mut inbox = self.shared.inbox.lock().expect("queue lock");
        if self.generation != inbox.generation {
            return Err(AdmissionError::StaleGeneration);
        }
        if !inbox.accepting || inbox.stopping {
            return Err(AdmissionError::Closed);
        }
        if inbox.pending.len() >= inbox.limit {
            return Err(AdmissionError::Full);
        }
        inbox.pending.push_back(command);
        drop(inbox);
        self.shared.wake.notify_one();
        Ok(())
    }

    /// Only a negative fixture; B2 does not implement generation rotation.
    pub fn old_generation(&self) -> Self {
        assert!(self.generation > 0);
        Self {
            shared: Arc::clone(&self.shared),
            generation: self.generation - 1,
        }
    }
}

#[derive(Clone)]
pub struct Snapshots(Arc<Shared>);
impl Snapshots {
    pub fn current(&self) -> Arc<Snapshot> {
        Arc::clone(&self.0.snapshot.lock().expect("snapshot lock"))
    }
    pub fn wait_for(&self, revision: u64, timeout: Duration) -> Option<Arc<Snapshot>> {
        let guard = self.0.snapshot.lock().expect("snapshot lock");
        let (guard, _) = self
            .0
            .published
            .wait_timeout_while(guard, timeout, |snapshot| snapshot.revision < revision)
            .expect("publication wait");
        (guard.revision >= revision).then(|| Arc::clone(&guard))
    }
}

/// A caller-owned scheduler token. Host stop only clears its own callbacks.
#[derive(Clone, Default)]
pub struct CallerScheduler(Rc<Cell<usize>>);
impl CallerScheduler {
    pub fn pulse(&self) -> usize {
        let next = self.0.get() + 1;
        self.0.set(next);
        next
    }
}

struct Foreground {
    phase: Phase,
    thread: ThreadId,
    ingress: Ingress,
    snapshots: Snapshots,
    callbacks: Rc<Cell<usize>>,
    _borrowed_scheduler: CallerScheduler,
}
impl Foreground {
    fn new(ingress: Ingress, snapshots: Snapshots, scheduler: &CallerScheduler) -> Self {
        Self {
            phase: Phase::Prepared,
            thread: thread::current().id(),
            ingress,
            snapshots,
            callbacks: Rc::new(Cell::new(0)),
            _borrowed_scheduler: scheduler.clone(),
        }
    }
    fn start(&mut self) {
        assert_eq!(self.phase, Phase::Prepared);
        self.callbacks.set(1);
        self.phase = Phase::Running;
    }
    fn submit(&self, command: Command) -> Result<(), HostError> {
        if self.phase != Phase::Running {
            return Err(HostError::Stopped);
        }
        self.ingress.submit(command).map_err(HostError::Admission)
    }
    fn stop(&mut self) {
        if self.phase == Phase::Running {
            self.callbacks.set(0);
            self.phase = Phase::Stopped;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Outcome {
    Stopped,
    Failed(Failure),
}
fn worker_loop(shared: &Shared) -> Outcome {
    // The mutable authoritative state only exists on this worker thread.
    let mut balance = 0_i64;
    let mut revision = 0_u64;
    loop {
        let (command, fault, delay) = {
            let mut inbox = shared.inbox.lock().expect("queue lock");
            while !inbox.stopping && (inbox.paused || inbox.pending.is_empty()) {
                inbox = shared.wake.wait(inbox).expect("queue wait");
            }
            if inbox.stopping {
                return Outcome::Stopped;
            }
            let command = inbox.pending.pop_front().expect("nonempty queue");
            let fault = std::mem::take(&mut inbox.fail_first);
            (command, fault, inbox.delay)
        };
        if !delay.is_zero() {
            *shared.in_flight.lock().expect("inflight lock") = true;
            shared.work_started.notify_all();
            thread::sleep(delay);
        }
        if fault {
            close_on_fault(shared);
            return Outcome::Failed(Failure::InjectedWorker);
        }
        let next_balance = match command {
            Command::Credit(value) => balance.checked_add(i64::from(value)),
            Command::Debit(value) => balance.checked_sub(i64::from(value)),
        };
        let Some(next_balance) = next_balance else {
            close_on_fault(shared);
            return Outcome::Failed(Failure::ArithmeticOverflow);
        };
        let Some(next_revision) = revision.checked_add(1) else {
            close_on_fault(shared);
            return Outcome::Failed(Failure::RevisionOverflow);
        };

        // Publication and cancellation agree under this short control lock.
        // No user callback or IO occurs while it is held.
        let inbox = shared.inbox.lock().expect("queue lock");
        if inbox.stopping {
            return Outcome::Stopped;
        }
        balance = next_balance;
        revision = next_revision;
        *shared.snapshot.lock().expect("snapshot lock") = Arc::new(Snapshot {
            generation: inbox.generation,
            revision,
            balance,
        });
        drop(inbox);
        shared.published.notify_all();
        *shared.in_flight.lock().expect("inflight lock") = false;
    }
}
fn close_on_fault(shared: &Shared) {
    let mut inbox = shared.inbox.lock().expect("queue lock");
    inbox.accepting = false;
    inbox.stopping = true;
    inbox.discarded_on_fault += inbox.pending.len();
    inbox.pending.clear();
    drop(inbox);
    shared.wake.notify_all();
}

struct Worker {
    shared: Arc<Shared>,
    handle: Option<JoinHandle<()>>,
    outcome_rx: mpsc::Receiver<Outcome>,
    outcome: Option<Outcome>,
    thread: ThreadId,
}
impl Worker {
    fn spawn(options: Options) -> Result<Self, Failure> {
        let shared = Arc::new(Shared {
            inbox: Mutex::new(Inbox {
                generation: options.generation,
                limit: options.capacity,
                pending: VecDeque::with_capacity(options.capacity),
                discarded_on_fault: 0,
                accepting: false,
                stopping: false,
                paused: options.pause_processing,
                fail_first: options.fail_first_command,
                delay: options.delay,
            }),
            wake: Condvar::new(),
            snapshot: Mutex::new(Arc::new(Snapshot {
                generation: options.generation,
                revision: 0,
                balance: 0,
            })),
            published: Condvar::new(),
            in_flight: Mutex::new(false),
            work_started: Condvar::new(),
        });
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (out_tx, out_rx) = mpsc::channel();
        let captured = Arc::clone(&shared);
        let handle = thread::Builder::new()
            .name("yetlet-b2-owner".into())
            .spawn(move || {
                let _ = ready_tx.send(thread::current().id());
                let result = worker_loop(&captured);
                let _ = out_tx.send(result);
            })
            .map_err(|_| Failure::WorkerPanic)?;
        let id = match ready_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(id) => id,
            Err(_) => {
                let mut inbox = shared.inbox.lock().expect("queue lock");
                inbox.accepting = false;
                inbox.stopping = true;
                drop(inbox);
                shared.wake.notify_all();
                // A failed start may not leave an unobserved worker behind.
                let _ = handle.join();
                return Err(Failure::WorkerReadiness);
            }
        };
        Ok(Self {
            shared,
            handle: Some(handle),
            outcome_rx: out_rx,
            outcome: None,
            thread: id,
        })
    }

    fn ports(&self, generation: u64) -> (Ingress, Snapshots) {
        (
            Ingress {
                shared: Arc::clone(&self.shared),
                generation,
            },
            Snapshots(Arc::clone(&self.shared)),
        )
    }
    fn open(&self) {
        self.shared.inbox.lock().expect("queue lock").accepting = true;
    }
    fn stop(&self) -> usize {
        let mut inbox = self.shared.inbox.lock().expect("queue lock");
        inbox.stopping = true;
        inbox.accepting = false;
        let discarded = inbox.pending.len() + std::mem::take(&mut inbox.discarded_on_fault);
        inbox.pending.clear();
        drop(inbox);
        self.shared.wake.notify_all();
        discarded
    }
    fn finish(&mut self, timeout: Duration) -> Cleanup {
        if self.handle.is_none() {
            return Cleanup::Joined;
        }
        if self.outcome.is_none() {
            match self.outcome_rx.recv_timeout(timeout) {
                Ok(outcome) => self.outcome = Some(outcome),
                Err(mpsc::RecvTimeoutError::Timeout) => return Cleanup::TimedOut,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.outcome = Some(Outcome::Failed(Failure::WorkerPanic));
                }
            }
        }
        if self.handle.take().expect("join handle").join().is_err() {
            self.outcome = Some(Outcome::Failed(Failure::WorkerPanic));
        }
        Cleanup::Joined
    }
    fn await_failure(&mut self, timeout: Duration) -> Option<Failure> {
        let outcome = self.outcome_rx.recv_timeout(timeout).ok()?;
        self.outcome = Some(outcome);
        match outcome {
            Outcome::Failed(reason) => Some(reason),
            Outcome::Stopped => None,
        }
    }
    fn unpause(&self) {
        self.shared.inbox.lock().expect("queue lock").paused = false;
        self.shared.wake.notify_one();
    }
    fn wait_inflight(&self, timeout: Duration) -> bool {
        let busy = self.shared.in_flight.lock().expect("inflight lock");
        let (busy, _) = self
            .shared
            .work_started
            .wait_timeout_while(busy, timeout, |inflight| !*inflight)
            .expect("inflight wait");
        *busy
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // Preserve owned thread completion instead of silently detaching.
            // Explicit bounded stop is required for a timed/observed result.
            self.stop();
            let _ = handle.join();
        }
    }
}

/// Two caller-thread hosts and one explicitly owned worker.
pub struct Group {
    panel: Foreground,
    rule: Foreground,
    worker: Worker,
    phase: Phase,
    failure: Option<Failure>,
    discarded: usize,
}
impl Group {
    pub fn start(options: Options, scheduler: &CallerScheduler) -> Result<Self, StartError> {
        assert!(options.capacity > 0);
        let mut worker = Worker::spawn(options).map_err(|failure| StartError {
            failure,
            first_host_stopped: false,
            worker_joined: false, // no worker was spawned and joined here
            ever_running: false,
        })?;
        let (ingress, snapshots) = worker.ports(options.generation);
        let mut panel = Foreground::new(ingress.clone(), snapshots.clone(), scheduler);
        let mut rule = Foreground::new(ingress, snapshots, scheduler);
        panel.start();
        if options.fail_second_host {
            panel.stop();
            worker.stop();
            let joined = worker.finish(Duration::from_secs(1)) == Cleanup::Joined;
            return Err(StartError {
                failure: Failure::SecondHostStart,
                first_host_stopped: panel.phase == Phase::Stopped,
                worker_joined: joined,
                ever_running: false,
            });
        }
        rule.start();
        worker.open();
        Ok(Self {
            panel,
            rule,
            worker,
            phase: Phase::Running,
            failure: None,
            discarded: 0,
        })
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn caller_thread(&self) -> ThreadId {
        self.panel.thread
    }
    pub fn worker_thread(&self) -> ThreadId {
        self.worker.thread
    }
    pub fn callbacks(&self) -> (usize, usize) {
        (self.panel.callbacks.get(), self.rule.callbacks.get())
    }
    pub fn panel_credit(&self, value: u32) -> Result<(), HostError> {
        self.panel.submit(Command::Credit(value))
    }
    pub fn rule_debit(&self, value: u32) -> Result<(), HostError> {
        self.rule.submit(Command::Debit(value))
    }
    pub fn panel_snapshot(&self) -> Arc<Snapshot> {
        self.panel.snapshots.current()
    }
    pub fn rule_snapshot(&self) -> Arc<Snapshot> {
        self.rule.snapshots.current()
    }
    pub fn wait_revision(&self, revision: u64, timeout: Duration) -> Option<Arc<Snapshot>> {
        self.panel.snapshots.wait_for(revision, timeout)
    }
    pub fn ingress(&self) -> Ingress {
        self.panel.ingress.clone()
    }
    pub fn wait_inflight(&self, timeout: Duration) -> bool {
        self.worker.wait_inflight(timeout)
    }
    pub fn unpause(&self) {
        self.worker.unpause();
    }
    pub fn fail_required_host(&mut self, timeout: Duration) -> StopResult {
        self.failure.get_or_insert(Failure::HostRuntime);
        self.stop(timeout)
    }
    pub fn observe_worker_failure(&mut self, timeout: Duration) -> Option<StopResult> {
        let cause = self.worker.await_failure(timeout)?;
        self.failure.get_or_insert(cause);
        Some(self.stop(timeout))
    }
    pub fn stop(&mut self, timeout: Duration) -> StopResult {
        if self.phase != Phase::Stopped && self.phase != Phase::Failed {
            self.phase = Phase::Stopping;
            self.panel.stop();
            self.rule.stop();
            self.discarded += self.worker.stop();
        }
        let cleanup = self.worker.finish(timeout);
        if cleanup == Cleanup::Joined {
            if self.failure.is_none()
                && let Some(Outcome::Failed(cause)) = self.worker.outcome
            {
                self.failure = Some(cause);
            }
            self.phase = if self.failure.is_some() {
                Phase::Failed
            } else {
                Phase::Stopped
            };
        }
        StopResult {
            phase: self.phase,
            failure: self.failure,
            cleanup,
            discarded_queued: self.discarded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEADLINE: Duration = Duration::from_secs(1);

    #[test]
    fn two_hosts_share_worker_owned_authority_and_snapshot() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(Options::default(), &scheduler).unwrap();
        assert_eq!(group.caller_thread(), thread::current().id());
        assert_ne!(group.worker_thread(), group.caller_thread());
        group.panel_credit(8).unwrap();
        group.rule_debit(3).unwrap();
        let current = group.wait_revision(2, DEADLINE).unwrap();
        assert_eq!(current.balance, 5);
        assert_eq!(current.generation, 1);
        assert!(Arc::ptr_eq(&current, &group.panel_snapshot()));
        assert!(Arc::ptr_eq(&current, &group.rule_snapshot()));
        assert_eq!(group.stop(DEADLINE).cleanup, Cleanup::Joined);
    }

    #[test]
    fn second_host_start_failure_stops_first_and_joins_worker() {
        let scheduler = CallerScheduler::default();
        let err = Group::start(
            Options {
                fail_second_host: true,
                ..Options::default()
            },
            &scheduler,
        )
        .err()
        .unwrap();
        assert_eq!(err.failure, Failure::SecondHostStart);
        assert!(err.first_host_stopped);
        assert!(err.worker_joined);
        assert!(!err.ever_running);
        assert_eq!(scheduler.pulse(), 1);
    }

    #[test]
    fn required_runtime_failure_retains_primary_cause_and_stops() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(Options::default(), &scheduler).unwrap();
        group.panel_credit(9).unwrap();
        let result = group.fail_required_host(DEADLINE);
        assert_eq!(result.failure, Some(Failure::HostRuntime));
        assert_eq!(result.phase, Phase::Failed);
        assert_eq!(result.cleanup, Cleanup::Joined);
        assert_eq!(group.rule_debit(1), Err(HostError::Stopped));
        assert_eq!(scheduler.pulse(), 1);
    }

    #[test]
    fn full_stale_closed_admission_and_shutdown_outside_data_queue() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(
            Options {
                capacity: 1,
                pause_processing: true,
                ..Options::default()
            },
            &scheduler,
        )
        .unwrap();
        group.panel_credit(4).unwrap();
        assert_eq!(
            group.rule_debit(1),
            Err(HostError::Admission(AdmissionError::Full))
        );
        assert_eq!(
            group.ingress().old_generation().submit(Command::Credit(1)),
            Err(AdmissionError::StaleGeneration)
        );
        let port = group.ingress();
        let result = group.stop(DEADLINE);
        assert_eq!(result.cleanup, Cleanup::Joined);
        assert_eq!(result.discarded_queued, 1);
        assert_eq!(group.rule_snapshot().revision, 0);
        assert_eq!(port.submit(Command::Credit(2)), Err(AdmissionError::Closed));
    }

    #[test]
    fn worker_failure_stops_group_without_publishing_failing_input() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(
            Options {
                fail_first_command: true,
                ..Options::default()
            },
            &scheduler,
        )
        .unwrap();
        let old = group.panel_snapshot();
        group.panel_credit(5).unwrap();
        let result = group.observe_worker_failure(DEADLINE).unwrap();
        assert_eq!(result.phase, Phase::Failed);
        assert_eq!(result.failure, Some(Failure::InjectedWorker));
        assert_eq!(result.cleanup, Cleanup::Joined);
        assert!(Arc::ptr_eq(&old, &group.rule_snapshot()));
    }

    #[test]
    fn worker_fault_counts_discarded_pending_command_once() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(
            Options {
                capacity: 2,
                pause_processing: true,
                fail_first_command: true,
                ..Options::default()
            },
            &scheduler,
        )
        .unwrap();
        let before = group.panel_snapshot();
        group.panel_credit(7).unwrap();
        group.rule_debit(2).unwrap();
        group.unpause();
        let first = group.observe_worker_failure(DEADLINE).unwrap();
        assert_eq!(first.failure, Some(Failure::InjectedWorker));
        assert_eq!(first.phase, Phase::Failed);
        assert_eq!(first.cleanup, Cleanup::Joined);
        assert_eq!(first.discarded_queued, 1);
        assert!(Arc::ptr_eq(&before, &group.rule_snapshot()));

        // The already accounted cancelled command must not be counted twice.
        let repeated = group.stop(DEADLINE);
        assert_eq!(repeated.failure, first.failure);
        assert_eq!(repeated.cleanup, Cleanup::Joined);
        assert_eq!(repeated.discarded_queued, 1);
    }
    #[test]
    fn timeout_keeps_worker_owned_until_later_confirmed_join() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(
            Options {
                delay: Duration::from_millis(80),
                ..Options::default()
            },
            &scheduler,
        )
        .unwrap();
        group.panel_credit(9).unwrap();
        assert!(group.wait_inflight(DEADLINE));
        let pending = group.stop(Duration::ZERO);
        assert_eq!(pending.phase, Phase::Stopping);
        assert_eq!(pending.cleanup, Cleanup::TimedOut);
        assert_eq!(group.panel_snapshot().revision, 0);
        let completed = group.stop(DEADLINE);
        assert_eq!(completed.phase, Phase::Stopped);
        assert_eq!(completed.cleanup, Cleanup::Joined);
    }

    #[test]
    fn borrowed_scheduler_survives_host_disposal() {
        let scheduler = CallerScheduler::default();
        let mut group = Group::start(Options::default(), &scheduler).unwrap();
        assert_eq!(scheduler.pulse(), 1);
        assert_eq!(group.callbacks(), (1, 1));
        assert_eq!(group.stop(DEADLINE).cleanup, Cleanup::Joined);
        assert_eq!(group.callbacks(), (0, 0));
        assert_eq!(scheduler.pulse(), 2);
    }
}
