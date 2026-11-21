#![cfg(feature = "composition")]

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TrySendError},
    thread::{self, JoinHandle, ThreadId},
    time::{Duration, Instant},
};
use yetlet::composition::{Builder, Completion, JoinStatus, Phase, WorkerCompletion};

type Error = &'static str;
const WAIT: Duration = Duration::from_secs(5);

fn deadline() -> Instant {
    Instant::now() + WAIT
}

#[derive(Clone, Copy)]
enum Outcome {
    Success,
    WorkerError,
    Panic,
}

#[derive(Debug, PartialEq, Eq)]
enum SubmitError {
    Closed,
    Full,
    Disconnected,
}

/// A real owned thread, held independently at its shutdown and completion gates.
/// No sleeps or scheduler-dependent polling establish test ordering.
struct HeldWorker {
    join: Option<JoinHandle<Result<(), Error>>>,
    stop: Sender<()>,
    done: Receiver<()>,
    data: SyncSender<u64>,
    admission_closed: bool,
    admission_calls: usize,
    signal_calls: usize,
    observation_calls: usize,
    observed_ids: Vec<ThreadId>,
    fail_signal_once: bool,
    fail_observation_once: bool,
}

struct WorkerControl {
    original_id: ThreadId,
    stop_seen: Receiver<()>,
    release: Sender<()>,
}

impl WorkerControl {
    fn wait_for_stop(&self) {
        self.stop_seen
            .recv_timeout(WAIT)
            .expect("the distinct shutdown signal must reach the original worker");
    }

    fn release(&self) {
        self.release
            .send(())
            .expect("the original worker must still own its completion gate");
    }
}

impl HeldWorker {
    fn start(outcome: Outcome) -> (Self, WorkerControl) {
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let (stop_seen_tx, stop_seen_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let (data_tx, data_rx) = mpsc::sync_channel(1);
        let join = thread::spawn(move || {
            let _data_receiver = data_rx;
            ready_tx
                .send(thread::current().id())
                .expect("test must retain the readiness receiver");
            let result = (|| {
                stop_rx
                    .recv_timeout(WAIT)
                    .map_err(|_| "shutdown signal missing")?;
                stop_seen_tx.send(()).map_err(|_| "stop receipt lost")?;
                release_rx
                    .recv_timeout(WAIT)
                    .map_err(|_| "completion gate not released")?;
                match outcome {
                    Outcome::WorkerError => Err("worker operation failed"),
                    Outcome::Success | Outcome::Panic => Ok(()),
                }
            })();
            let _ = done_tx.send(());
            if matches!(outcome, Outcome::Panic) && result.is_ok() {
                panic!("intentional worker panic to exercise a real failed Join");
            }
            result
        });
        let original_id = ready_rx
            .recv_timeout(WAIT)
            .expect("the worker must report actual readiness");
        assert_eq!(join.thread().id(), original_id);
        (
            Self {
                join: Some(join),
                stop: stop_tx,
                done: done_rx,
                data: data_tx,
                admission_closed: false,
                admission_calls: 0,
                signal_calls: 0,
                observation_calls: 0,
                observed_ids: Vec::new(),
                fail_signal_once: false,
                fail_observation_once: false,
            },
            WorkerControl {
                original_id,
                stop_seen: stop_seen_rx,
                release: release_tx,
            },
        )
    }

    fn try_command(&self, revision: u64) -> Result<(), SubmitError> {
        if self.admission_closed {
            return Err(SubmitError::Closed);
        }
        self.data.try_send(revision).map_err(|error| match error {
            TrySendError::Full(_) => SubmitError::Full,
            TrySendError::Disconnected(_) => SubmitError::Disconnected,
        })
    }

    fn close_admission(&mut self) -> Result<(), Error> {
        self.admission_calls += 1;
        self.admission_closed = true;
        Ok(())
    }

    fn signal_stop(&mut self) -> Result<(), Error> {
        self.signal_calls += 1;
        if self.fail_signal_once {
            self.fail_signal_once = false;
            return Err("shutdown signal temporarily unavailable");
        }
        self.stop.send(()).map_err(|_| "shutdown receiver lost")
    }

    fn observe(&mut self, remaining: Duration) -> Result<WorkerCompletion<Error>, Error> {
        self.observation_calls += 1;
        self.observed_ids.push(
            self.join
                .as_ref()
                .expect("the callback must retain the original JoinHandle until terminal Join")
                .thread()
                .id(),
        );
        if self.fail_observation_once {
            self.fail_observation_once = false;
            return Err("completion observation temporarily unavailable");
        }
        match self.done.recv_timeout(remaining) {
            Ok(()) => {
                let result = self
                    .join
                    .take()
                    .expect("terminal observation must consume the original JoinHandle")
                    .join()
                    .unwrap_or(Err("worker join panicked"));
                Ok(WorkerCompletion::Joined(result))
            }
            Err(RecvTimeoutError::Timeout) => Ok(WorkerCompletion::Pending),
            Err(RecvTimeoutError::Disconnected) => Err("completion receipt disconnected"),
        }
    }
}

struct CallerLog {
    revisions: Vec<u64>,
    drops: Rc<Cell<usize>>,
}

impl Drop for CallerLog {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

struct DropProbe(Rc<Cell<usize>>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn local_rc_and_borrowed_audit_remain_caller_owned_and_reusable() {
    let revision = Rc::new(Cell::new(0_u64));
    let closes = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let mut audit = CallerLog {
        revisions: Vec::new(),
        drops: drops.clone(),
    };
    let close_count = closes.clone();
    let mut app = Builder::<Error>::new()
        .required_local("domain", revision.clone(), move |_| {
            close_count.set(close_count.get() + 1);
            Ok(())
        })
        .borrowed("audit", &mut audit)
        .build();

    assert!(app.all_required_ready());
    assert_eq!(app.phase(), Phase::Running);
    assert_eq!(app.parts().previous().last().label(), "domain");
    assert_eq!(app.parts().last().label(), "audit");
    app.parts_mut()
        .previous()
        .last()
        .value_mut()
        .unwrap()
        .set(12);
    app.parts_mut()
        .last()
        .value_mut()
        .unwrap()
        .revisions
        .push(revision.get());
    assert_eq!(app.parts().last().value().revisions, [12]);

    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Stopped);
    assert_eq!(closes.get(), 1);
    assert!(app.parts_mut().last().value_mut().is_none());
    assert!(app.parts_mut().previous().last().value_mut().is_none());
    assert!(app.failures().primary().is_none());
    drop(app);

    assert_eq!(Rc::strong_count(&revision), 1);
    assert_eq!(drops.get(), 0);
    audit.revisions.push(13);
    assert_eq!(audit.revisions, [12, 13]);
    drop(audit);
    assert_eq!(drops.get(), 1);
}

#[test]
fn foreground_and_real_worker_each_require_observed_readiness() {
    let (worker, control) = HeldWorker::start(Outcome::Success);
    let mut app = Builder::<Error>::new()
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .required_foreground("foreground", None::<u64>, |_| Ok(()))
        .build();

    assert!(!app.all_required_ready());
    assert_eq!(app.phase(), Phase::Prepared);
    assert!(!app.parts().previous().last().is_ready());
    *app.parts_mut().last().value_mut().unwrap() = Some(24);
    assert!(app.parts_mut().last().mark_ready());
    assert!(!app.all_required_ready());
    assert!(app.parts_mut().previous().last().mark_ready());
    assert!(app.all_required_ready());
    assert_eq!(app.phase(), Phase::Running);

    app.request_stop();
    control.wait_for_stop();
    assert!(!app.all_required_ready());
    assert!(!app.parts_mut().last().mark_ready());
    assert!(!app.parts_mut().previous().last().mark_ready());
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Stopped);
    assert_eq!(
        app.parts().previous().last().join_status(),
        JoinStatus::Succeeded
    );
}

#[test]
fn failed_later_start_rolls_back_in_reverse_and_retains_original_pending_thread() {
    let (worker, control) = HeldWorker::start(Outcome::Success);
    let closed = Rc::new(RefCell::new(Vec::new()));
    let domain_closed = closed.clone();
    let overlay_closed = closed.clone();
    let result = Builder::<Error>::new()
        .required_local("domain", (), move |_| {
            domain_closed.borrow_mut().push("domain");
            Ok(())
        })
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .required_local("overlay", (), move |_| {
            overlay_closed.borrow_mut().push("overlay");
            Ok(())
        })
        .try_required_local(
            "later-required",
            || Err::<(), _>("later startup failed"),
            |_| Ok(()),
        );
    let failure = match result {
        Err(failure) => failure,
        Ok(_) => panic!("the later factory must fail"),
    };
    assert_eq!(
        failure.root().failures().primary(),
        Some(&"later startup failed")
    );
    assert_eq!(&*closed.borrow(), &["overlay", "domain"]);
    assert_eq!(failure.root().phase(), Phase::Stopping);
    let mut app = failure.into_root();
    control.wait_for_stop();

    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    assert_eq!(
        app.parts()
            .previous()
            .last()
            .value()
            .join
            .as_ref()
            .unwrap()
            .thread()
            .id(),
        control.original_id
    );
    assert_eq!(
        app.parts().previous().last().join_status(),
        JoinStatus::Pending
    );
    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(app.failures().primary(), Some(&"later startup failed"));
    assert!(app.failures().secondary().is_empty());
    assert_eq!(
        app.parts().previous().last().join_status(),
        JoinStatus::Succeeded
    );
    assert!(
        app.parts()
            .previous()
            .last()
            .value()
            .observed_ids
            .iter()
            .all(|id| *id == control.original_id)
    );
    assert_eq!(&*closed.borrow(), &["overlay", "domain"]);
}

#[test]
fn full_data_queue_cannot_block_distinct_stop_signal_or_original_join() {
    let (worker, control) = HeldWorker::start(Outcome::Success);
    let local_closes = Rc::new(Cell::new(0));
    let closes = local_closes.clone();
    let mut app = Builder::<Error>::new()
        .required_local("authority", (), move |_| {
            closes.set(closes.get() + 1);
            Ok(())
        })
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .build();

    assert_eq!(app.parts().last().value().try_command(12), Ok(()));
    assert_eq!(
        app.parts().last().value().try_command(13),
        Err(SubmitError::Full)
    );
    app.request_stop();
    app.request_stop();
    control.wait_for_stop();
    assert_eq!(local_closes.get(), 1);
    assert!(app.parts().last().admission_closed());
    assert!(app.parts().last().stop_signalled());
    assert_eq!(
        app.parts().last().value().try_command(14),
        Err(SubmitError::Closed)
    );
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    assert!(app.parts().last().value().join.is_some());

    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    let worker = app.parts().last().value();
    assert_eq!(worker.admission_calls, 1);
    assert_eq!(worker.signal_calls, 1);
    assert_eq!(worker.observation_calls, 2);
    assert_eq!(
        worker.observed_ids,
        [control.original_id, control.original_id]
    );
    assert!(worker.join.is_none());
    assert_eq!(app.parts().last().join_status(), JoinStatus::Succeeded);
    assert_eq!(app.phase(), Phase::Stopped);
}

struct Gate {
    allowed: Rc<Cell<bool>>,
    admissions: usize,
    signals: usize,
    observations: usize,
}

impl Gate {
    fn new(allowed: Rc<Cell<bool>>) -> Self {
        Self {
            allowed,
            admissions: 0,
            signals: 0,
            observations: 0,
        }
    }

    fn close(&mut self) -> Result<(), Error> {
        self.admissions += 1;
        if self.allowed.get() {
            Ok(())
        } else {
            Err("admission held")
        }
    }

    fn signal(&mut self) -> Result<(), Error> {
        self.signals += 1;
        Ok(())
    }

    fn observe(&mut self, _: Duration) -> Result<WorkerCompletion<Error>, Error> {
        self.observations += 1;
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}

#[test]
fn every_admission_fence_is_attempted_and_authority_survives_failed_retry() {
    let late_allowed = Rc::new(Cell::new(false));
    let closes = Rc::new(Cell::new(0));
    let domain_closes = closes.clone();
    let mut app = Builder::<Error>::new()
        .required_local("authority", (), move |_| {
            domain_closes.set(domain_closes.get() + 1);
            Ok(())
        })
        .required_worker(
            "early",
            Gate::new(Rc::new(Cell::new(true))),
            Gate::close,
            Gate::signal,
            Gate::observe,
        )
        .required_worker(
            "late",
            Gate::new(late_allowed.clone()),
            Gate::close,
            Gate::signal,
            Gate::observe,
        )
        .build();

    app.request_stop();
    assert_eq!(app.parts().last().value().admissions, 1);
    assert_eq!(app.parts().previous().last().value().admissions, 1);
    assert_eq!(closes.get(), 0);
    assert_eq!(app.parts().last().value().signals, 0);
    assert_eq!(app.parts().previous().last().value().signals, 0);
    assert!(!app.parts().previous().previous().last().is_stopped());
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    assert_eq!(app.parts().last().value().admissions, 2);
    assert_eq!(app.parts().previous().last().value().admissions, 1);
    assert_eq!(app.parts().last().value().observations, 0);
    assert_eq!(closes.get(), 0);
    assert_eq!(app.phase(), Phase::Stopping);

    late_allowed.set(true);
    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(closes.get(), 1);
    assert_eq!(app.parts().last().value().admissions, 3);
    assert_eq!(app.parts().last().value().signals, 1);
    assert_eq!(app.parts().previous().last().value().signals, 1);
    assert_eq!(app.parts().last().value().observations, 1);
    assert_eq!(app.parts().previous().last().value().observations, 1);
    assert_eq!(app.failures().primary(), Some(&"admission held"));
    assert_eq!(app.failures().secondary(), &["admission held"]);
    assert_eq!(app.phase(), Phase::Failed);
}

#[test]
fn failed_later_local_close_keeps_earlier_owner_but_does_not_block_signal() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let domain_events = events.clone();
    let guard_events = events.clone();
    let signals = Rc::new(Cell::new(0));
    let worker_signals = signals.clone();
    let mut guard_attempts = 0;
    let mut app = Builder::<Error>::new()
        .required_local("authority", (), move |_| {
            domain_events.borrow_mut().push("authority closed");
            Ok(())
        })
        .required_local("later guard", (), move |_| {
            guard_attempts += 1;
            if guard_attempts == 1 {
                guard_events.borrow_mut().push("guard close failed");
                Err("guard still active")
            } else {
                guard_events.borrow_mut().push("guard closed");
                Ok(())
            }
        })
        .required_worker(
            "worker",
            (),
            |_| Ok(()),
            move |_| {
                worker_signals.set(worker_signals.get() + 1);
                Ok(())
            },
            |_, _| Ok(WorkerCompletion::Joined(Ok(()))),
        )
        .build();

    app.request_stop();
    assert_eq!(&*events.borrow(), &["guard close failed"]);
    assert!(!app.parts().previous().previous().last().is_stopped());
    assert!(!app.parts().previous().last().is_stopped());
    assert_eq!(signals.get(), 1);
    assert!(app.parts().last().stop_signalled());
    assert!(app.parts_mut().previous().last().value_mut().is_none());

    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(
        &*events.borrow(),
        &["guard close failed", "guard closed", "authority closed"]
    );
    assert_eq!(signals.get(), 1);
    assert_eq!(app.failures().primary(), Some(&"guard still active"));
    assert_eq!(app.phase(), Phase::Failed);
}

#[test]
fn failed_signal_and_transient_observation_retain_same_handle_until_retry() {
    let (mut worker, control) = HeldWorker::start(Outcome::Success);
    worker.fail_signal_once = true;
    worker.fail_observation_once = true;
    let closes = Rc::new(Cell::new(0));
    let domain_closes = closes.clone();
    let mut app = Builder::<Error>::new()
        .required_local("authority", (), move |_| {
            domain_closes.set(domain_closes.get() + 1);
            Ok(())
        })
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .build();

    app.request_stop();
    assert!(!app.parts().last().stop_signalled());
    assert_eq!(app.parts().last().value().observation_calls, 0);
    assert_eq!(closes.get(), 1);
    assert_eq!(
        app.failures().primary(),
        Some(&"shutdown signal temporarily unavailable")
    );
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    control.wait_for_stop();
    assert!(app.parts().last().stop_signalled());
    assert_eq!(app.parts().last().join_status(), JoinStatus::Pending);
    assert_eq!(
        app.parts()
            .last()
            .value()
            .join
            .as_ref()
            .unwrap()
            .thread()
            .id(),
        control.original_id
    );
    assert_eq!(
        app.failures().secondary(),
        &["completion observation temporarily unavailable"]
    );
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(closes.get(), 1);
    assert_eq!(app.parts().last().value().signal_calls, 2);
    assert_eq!(app.parts().last().value().observation_calls, 3);
    assert_eq!(
        app.parts().last().value().observed_ids,
        [
            control.original_id,
            control.original_id,
            control.original_id
        ]
    );
    assert_eq!(app.parts().last().join_status(), JoinStatus::Succeeded);
}

#[test]
fn first_required_failure_survives_a_real_failed_join() {
    let (worker, control) = HeldWorker::start(Outcome::Panic);
    let mut app = Builder::<Error>::new()
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .build();

    app.fail_required("foreground draw failed");
    control.wait_for_stop();
    assert_eq!(app.phase(), Phase::Stopping);
    assert_eq!(app.failures().primary(), Some(&"foreground draw failed"));
    assert!(!app.parts_mut().last().mark_ready());
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.parts().last().join_status(), JoinStatus::Failed);
    assert!(app.parts().last().value().join.is_none());
    assert_eq!(app.failures().primary(), Some(&"foreground draw failed"));
    assert_eq!(app.failures().secondary(), &["worker join panicked"]);
    assert_eq!(app.phase(), Phase::Failed);

    app.fail_required("later exit report");
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(app.failures().primary(), Some(&"foreground draw failed"));
    assert_eq!(
        app.failures().secondary(),
        &["worker join panicked", "later exit report"]
    );
    assert_eq!(app.parts().last().value().signal_calls, 1);
}

#[test]
fn joined_worker_error_can_be_primary_and_cleanup_complete_is_not_success() {
    let (worker, control) = HeldWorker::start(Outcome::WorkerError);
    let mut app = Builder::<Error>::new()
        .required_worker(
            "worker",
            worker,
            HeldWorker::close_admission,
            HeldWorker::signal_stop,
            HeldWorker::observe,
        )
        .build();

    app.request_stop();
    control.wait_for_stop();
    control.release();
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.parts().last().join_status(), JoinStatus::Failed);
    assert_eq!(app.failures().primary(), Some(&"worker operation failed"));
    assert!(app.failures().secondary().is_empty());
    assert_eq!(app.phase(), Phase::Failed);
}

#[test]
fn failed_worker_factory_rolls_back_without_invoking_unacquired_callbacks() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let first = events.clone();
    let second = events.clone();
    let callback_calls = Rc::new(Cell::new(0));
    let admission_calls = callback_calls.clone();
    let signal_calls = callback_calls.clone();
    let observation_calls = callback_calls.clone();
    let result = Builder::<Error>::new()
        .required_local("first", (), move |_| {
            first.borrow_mut().push("first");
            Ok(())
        })
        .required_local("second", (), move |_| {
            second.borrow_mut().push("second");
            Ok(())
        })
        .try_required_worker(
            "unacquired worker",
            || Err::<(), _>("worker startup failed"),
            move |_| {
                admission_calls.set(admission_calls.get() + 1);
                Ok(())
            },
            move |_| {
                signal_calls.set(signal_calls.get() + 1);
                Ok(())
            },
            move |_, _| {
                observation_calls.set(observation_calls.get() + 1);
                Ok(WorkerCompletion::Joined(Ok(())))
            },
        );
    let failure = match result {
        Err(failure) => failure,
        Ok(_) => panic!("the worker factory must fail"),
    };
    assert_eq!(&*events.borrow(), &["second", "first"]);
    assert_eq!(callback_calls.get(), 0);
    let mut app = failure.into_root();
    assert_eq!(app.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(app.failures().primary(), Some(&"worker startup failed"));
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(&*events.borrow(), &["second", "first"]);
    assert_eq!(callback_calls.get(), 0);
}

#[test]
fn ordinary_drop_destroys_owned_values_but_does_not_run_lifecycle_or_own_borrows() {
    let owned_drops = Rc::new(Cell::new(0));
    let borrowed_drops = Rc::new(Cell::new(0));
    let closes = Rc::new(Cell::new(0));
    let close_calls = closes.clone();
    let mut caller_log = CallerLog {
        revisions: Vec::new(),
        drops: borrowed_drops.clone(),
    };
    let app = Builder::<Error>::new()
        .required_local("owned", DropProbe(owned_drops.clone()), move |_| {
            close_calls.set(close_calls.get() + 1);
            Ok(())
        })
        .borrowed("caller log", &mut caller_log)
        .build();

    drop(app);
    assert_eq!(owned_drops.get(), 1);
    assert_eq!(closes.get(), 0);
    assert_eq!(borrowed_drops.get(), 0);
    caller_log.revisions.push(24);
    assert_eq!(caller_log.revisions, [24]);
    drop(caller_log);
    assert_eq!(borrowed_drops.get(), 1);
}

#[test]
fn exited_worker_is_joined_even_when_its_shutdown_receiver_is_already_closed() {
    struct ExitedWorker {
        join: Option<JoinHandle<Result<(), Error>>>,
        stop: Sender<()>,
        admission_calls: usize,
        signal_calls: usize,
        observation_calls: usize,
        observed_ids: Vec<ThreadId>,
    }

    let (stop, stop_receiver) = mpsc::channel::<()>();
    let join = thread::spawn(move || {
        drop(stop_receiver);
        Err::<(), Error>("worker exited before shutdown signal")
    });
    let original_id = join.thread().id();
    let finish_deadline = deadline();
    while !join.is_finished() {
        assert!(
            Instant::now() < finish_deadline,
            "the original worker must finish before shutdown is requested"
        );
        thread::yield_now();
    }
    let worker = ExitedWorker {
        join: Some(join),
        stop,
        admission_calls: 0,
        signal_calls: 0,
        observation_calls: 0,
        observed_ids: Vec::new(),
    };
    let mut app = Builder::<Error>::new()
        .required_worker(
            "already exited",
            worker,
            |worker| {
                worker.admission_calls += 1;
                Ok(())
            },
            |worker| {
                worker.signal_calls += 1;
                worker.stop.send(()).map_err(|_| "shutdown receiver closed")
            },
            |worker, _| {
                worker.observation_calls += 1;
                worker
                    .observed_ids
                    .push(worker.join.as_ref().unwrap().thread().id());
                let result = worker
                    .join
                    .take()
                    .expect("must join the original completed worker")
                    .join()
                    .expect("the worker returns an error without panicking");
                Ok(WorkerCompletion::Joined(result))
            },
        )
        .build();

    app.fail_required("initiating required failure");
    assert_eq!(
        app.failures().primary(),
        Some(&"initiating required failure")
    );
    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(app.parts().last().join_status(), JoinStatus::Failed);
    assert!(!app.parts().last().stop_signalled());
    assert!(app.parts().last().value().join.is_none());
    assert_eq!(app.parts().last().value().admission_calls, 1);
    assert_eq!(app.parts().last().value().observation_calls, 1);
    assert_eq!(app.parts().last().value().observed_ids, [original_id]);
    assert!(
        app.failures()
            .secondary()
            .contains(&"shutdown receiver closed")
    );
    assert_eq!(
        app.failures().secondary().last(),
        Some(&"worker exited before shutdown signal")
    );
    let signal_calls = app.parts().last().value().signal_calls;
    assert!(signal_calls >= 1);

    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    app.request_stop();
    assert_eq!(app.parts().last().value().signal_calls, signal_calls);
    assert_eq!(app.parts().last().value().observation_calls, 1);
    assert_eq!(
        app.failures().primary(),
        Some(&"initiating required failure")
    );
}

#[test]
fn live_worker_is_observed_pending_while_signal_fails_then_original_handle_joins() {
    let (worker, control) = HeldWorker::start(Outcome::Success);
    let signal_allowed = Rc::new(Cell::new(false));
    let gate = signal_allowed.clone();
    let signal_attempts = Rc::new(Cell::new(0));
    let attempts = signal_attempts.clone();
    let mut app = Builder::<Error>::new()
        .required_worker(
            "live worker",
            worker,
            HeldWorker::close_admission,
            move |worker| {
                attempts.set(attempts.get() + 1);
                if gate.get() {
                    worker.signal_stop()
                } else {
                    Err("shutdown signal retry gate held")
                }
            },
            HeldWorker::observe,
        )
        .build();

    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    assert_eq!(app.phase(), Phase::Stopping);
    assert!(!app.parts().last().stop_signalled());
    assert_eq!(app.parts().last().join_status(), JoinStatus::Pending);
    assert_eq!(app.parts().last().value().observation_calls, 1);
    assert_eq!(
        app.parts()
            .last()
            .value()
            .join
            .as_ref()
            .unwrap()
            .thread()
            .id(),
        control.original_id
    );
    assert_eq!(signal_attempts.get(), 1);
    assert_eq!(
        app.failures().primary(),
        Some(&"shutdown signal retry gate held")
    );

    signal_allowed.set(true);
    app.request_stop();
    control.wait_for_stop();
    assert!(app.parts().last().stop_signalled());
    assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
    assert!(app.parts().last().value().join.is_some());
    control.release();

    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(app.phase(), Phase::Failed);
    assert_eq!(app.parts().last().join_status(), JoinStatus::Succeeded);
    assert!(app.parts().last().value().join.is_none());
    assert_eq!(signal_attempts.get(), 2);
    assert_eq!(app.parts().last().value().signal_calls, 1);
    assert_eq!(app.parts().last().value().observation_calls, 3);
    assert_eq!(
        app.parts().last().value().observed_ids,
        [
            control.original_id,
            control.original_id,
            control.original_id
        ]
    );
    assert!(app.failures().secondary().is_empty());

    assert_eq!(app.complete_until(deadline()), Completion::Complete);
    assert_eq!(signal_attempts.get(), 2);
    assert_eq!(app.parts().last().value().observation_calls, 3);
}
