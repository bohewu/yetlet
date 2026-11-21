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

#[cfg(test)]
mod final_release {
    use super::*;
    use yetlet::composition::{ReleaseCompletion, ReleaseStatus};

    #[test]
    fn resource_release_waits_for_the_original_join_across_group_boundary() {
        let (worker, control) = HeldWorker::start(Outcome::Success);
        let drops = Rc::new(Cell::new(0));
        let activity_releases = Rc::new(Cell::new(0));
        let releases = activity_releases.clone();
        let dependency_gate = activity_releases.clone();
        let mut app = Builder::<Error>::new()
            .with_resource("dependency", DropProbe(drops.clone()), move |_, _| {
                assert_eq!(dependency_gate.get(), 1);
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_group(
                "service",
                Builder::<Error>::new().with_activity(
                    "original worker",
                    worker,
                    HeldWorker::close_admission,
                    HeldWorker::signal_stop,
                    HeldWorker::observe,
                    move |worker, _| {
                        assert!(
                            worker.join.is_none(),
                            "final release requires the original Join"
                        );
                        releases.set(releases.get() + 1);
                        Ok(ReleaseCompletion::Released(Ok(())))
                    },
                ),
            )
            .build();
        assert!(app.parts_mut().last().parts_mut().last().mark_ready());
        assert!(app.all_required_ready());
        app.request_stop();
        control.wait_for_stop();

        assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(drops.get(), 0);
        assert_eq!(activity_releases.get(), 0);
        assert!(app.parts().previous().last().value().is_some());
        let held = app.parts().last().parts().last().value();
        assert_eq!(
            held.join.as_ref().unwrap().thread().id(),
            control.original_id
        );

        control.release();
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(drops.get(), 1);
        assert!(app.parts().previous().last().value().is_none());
        assert_eq!(
            app.parts().previous().last().release_status(),
            ReleaseStatus::Succeeded
        );
        let activity = app.parts().last().parts().last();
        assert_eq!(activity.join_status(), JoinStatus::Succeeded);
        assert_eq!(activity.release_status(), ReleaseStatus::Succeeded);
        assert!(activity.value().join.is_none());
        assert_eq!(
            activity.value().observed_ids,
            [control.original_id, control.original_id]
        );
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(activity_releases.get(), 1);
        assert_eq!(
            app.parts().last().parts().last().value().observation_calls,
            2
        );
    }

    #[test]
    fn pending_and_retryable_release_error_preserve_earlier_dependencies() {
        let earlier_drops = Rc::new(Cell::new(0));
        let held_drops = Rc::new(Cell::new(0));
        let latest_drops = Rc::new(Cell::new(0));
        let earlier_calls = Rc::new(Cell::new(0));
        let held_calls = Rc::new(Cell::new(0));
        let latest_calls = Rc::new(Cell::new(0));
        let before = earlier_calls.clone();
        let held_drop_observer = held_drops.clone();
        let held_attempts = held_calls.clone();
        let latest_attempts = latest_calls.clone();
        let mut app = Builder::<Error>::new()
            .with_resource("earlier", DropProbe(earlier_drops.clone()), move |_, _| {
                assert_eq!(
                    held_drop_observer.get(),
                    1,
                    "dependent value must already be dropped"
                );
                before.set(before.get() + 1);
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_resource("held", DropProbe(held_drops.clone()), move |_, _| {
                held_attempts.set(held_attempts.get() + 1);
                match held_attempts.get() {
                    1 => Ok(ReleaseCompletion::Pending),
                    2 => Err("release temporarily unavailable"),
                    _ => Ok(ReleaseCompletion::Released(Ok(()))),
                }
            })
            .with_resource("latest", DropProbe(latest_drops.clone()), move |_, _| {
                latest_attempts.set(latest_attempts.get() + 1);
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .build();

        assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(latest_calls.get(), 1);
        assert_eq!(latest_drops.get(), 1);
        assert_eq!(held_drops.get(), 0);
        assert_eq!(earlier_calls.get(), 0);
        assert!(app.parts().previous().last().value().is_some());
        assert!(app.parts().previous().previous().last().value().is_some());

        assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(
            app.failures().primary(),
            Some(&"release temporarily unavailable")
        );
        assert_eq!(
            app.parts().previous().last().release_status(),
            ReleaseStatus::Pending
        );
        assert_eq!(earlier_drops.get(), 0);
        assert_eq!(held_drops.get(), 0);
        assert_eq!(latest_calls.get(), 1);

        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.phase(), Phase::Failed);
        assert_eq!(held_calls.get(), 3);
        assert_eq!(earlier_calls.get(), 1);
        assert_eq!(held_drops.get(), 1);
        assert_eq!(earlier_drops.get(), 1);
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(held_calls.get(), 3);
        assert_eq!(earlier_calls.get(), 1);
        assert_eq!(latest_calls.get(), 1);
    }

    struct OrderedDrop {
        label: &'static str,
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Drop for OrderedDrop {
        fn drop(&mut self) {
            self.events.borrow_mut().push(self.label);
        }
    }

    #[test]
    fn terminal_release_error_drops_value_before_earlier_finalizer_and_latches_once() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let earlier_events = events.clone();
        let later_events = events.clone();
        let mut app = Builder::<Error>::new()
            .with_resource(
                "earlier",
                OrderedDrop {
                    label: "earlier dropped",
                    events: events.clone(),
                },
                move |_, _| {
                    assert_eq!(
                        &*earlier_events.borrow(),
                        &["later finalizer", "later dropped"]
                    );
                    earlier_events.borrow_mut().push("earlier finalizer");
                    Ok(ReleaseCompletion::Released(Ok(())))
                },
            )
            .with_resource(
                "later",
                OrderedDrop {
                    label: "later dropped",
                    events: events.clone(),
                },
                move |_, _| {
                    later_events.borrow_mut().push("later finalizer");
                    Ok(ReleaseCompletion::Released(Err("terminal release failed")))
                },
            )
            .build();

        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.phase(), Phase::Failed);
        assert_eq!(app.failures().primary(), Some(&"terminal release failed"));
        assert!(app.failures().secondary().is_empty());
        assert_eq!(app.parts().last().release_status(), ReleaseStatus::Failed);
        assert_eq!(
            app.parts().previous().last().release_status(),
            ReleaseStatus::Succeeded
        );
        assert!(app.parts().last().value().is_none());
        assert!(app.parts().previous().last().value().is_none());
        assert_eq!(
            &*events.borrow(),
            &[
                "later finalizer",
                "later dropped",
                "earlier finalizer",
                "earlier dropped"
            ]
        );
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        drop(app);
        assert_eq!(
            events.borrow().len(),
            4,
            "terminal release and Drop must not repeat"
        );
    }

    #[test]
    fn later_start_failure_keeps_primary_through_retry_and_terminal_release_errors() {
        let drops = Rc::new(Cell::new(0));
        let attempts = Rc::new(Cell::new(0));
        let observed_attempts = attempts.clone();
        let result = Builder::<Error>::new()
            .with_resource("dependency", DropProbe(drops.clone()), move |_, _| {
                observed_attempts.set(observed_attempts.get() + 1);
                if observed_attempts.get() == 1 {
                    Err("cleanup observation failed")
                } else {
                    Ok(ReleaseCompletion::Released(Err("cleanup terminal failure")))
                }
            })
            .try_required_local(
                "later start",
                || Err::<(), _>("original startup failure"),
                |_| Ok(()),
            );
        let failure = match result {
            Err(failure) => failure,
            Ok(_) => panic!("the later factory must fail"),
        };
        assert_eq!(
            failure.root().failures().primary(),
            Some(&"original startup failure")
        );
        assert_eq!(drops.get(), 0);
        assert_eq!(attempts.get(), 0, "request_stop must not run final release");
        let mut app = failure.into_root();
        assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(app.failures().primary(), Some(&"original startup failure"));
        assert_eq!(app.failures().secondary(), &["cleanup observation failed"]);
        assert!(app.parts().last().value().is_some());
        assert_eq!(drops.get(), 0);

        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.phase(), Phase::Failed);
        assert_eq!(app.failures().primary(), Some(&"original startup failure"));
        assert_eq!(
            app.failures().secondary(),
            &["cleanup observation failed", "cleanup terminal failure"]
        );
        assert_eq!(app.parts().last().release_status(), ReleaseStatus::Failed);
        assert_eq!(drops.get(), 1);
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(attempts.get(), 2);
    }

    #[test]
    fn failed_original_join_is_terminal_before_dependency_release() {
        let (worker, control) = HeldWorker::start(Outcome::WorkerError);
        let joined = Rc::new(Cell::new(false));
        let joined_for_release = joined.clone();
        let joined_for_observe = joined.clone();
        let drops = Rc::new(Cell::new(0));
        let mut app = Builder::<Error>::new()
            .with_resource("dependency", DropProbe(drops.clone()), move |_, _| {
                assert!(
                    joined_for_release.get(),
                    "failed Join still precedes final release"
                );
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .required_worker(
                "failing worker",
                worker,
                HeldWorker::close_admission,
                HeldWorker::signal_stop,
                move |worker, remaining| {
                    let result = worker.observe(remaining);
                    if matches!(&result, Ok(WorkerCompletion::Joined(_))) {
                        joined_for_observe.set(true);
                    }
                    result
                },
            )
            .build();
        app.request_stop();
        control.wait_for_stop();
        control.release();
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.phase(), Phase::Failed);
        assert_eq!(app.failures().primary(), Some(&"worker operation failed"));
        assert_eq!(app.parts().last().join_status(), JoinStatus::Failed);
        assert!(app.parts().last().value().join.is_none());
        assert_eq!(
            app.parts().last().value().observed_ids,
            [control.original_id]
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.parts().last().value().observation_calls, 1);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn nested_groups_keep_one_expired_deadline_for_observe_and_release() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let observed = events.clone();
        let activity_release = events.clone();
        let inner_release = events.clone();
        let outer_release = events.clone();
        let root_release = events.clone();
        let caller_owned = Cell::new(0);
        let inner = Builder::<Error>::new()
            .with_resource("inner dependency", (), move |_, remaining| {
                inner_release.borrow_mut().push(("inner", remaining));
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_activity(
                "local activity",
                (),
                |_| Ok(()),
                |_| Ok(()),
                move |_, remaining| {
                    observed.borrow_mut().push(("observe", remaining));
                    Ok(WorkerCompletion::Joined(Ok(())))
                },
                move |_, remaining| {
                    activity_release.borrow_mut().push(("activity", remaining));
                    Ok(ReleaseCompletion::Released(Ok(())))
                },
            );
        let outer = Builder::<Error>::new()
            .with_resource("outer dependency", (), move |_, remaining| {
                outer_release.borrow_mut().push(("outer", remaining));
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_group("inner group", inner);
        let mut app = Builder::<Error>::new()
            .with_resource("caller borrow", &caller_owned, move |counter, remaining| {
                counter.set(counter.get() + 1);
                root_release.borrow_mut().push(("root", remaining));
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_group("outer group", outer)
            .build();
        assert!(
            app.parts_mut()
                .last()
                .parts_mut()
                .last()
                .parts_mut()
                .last()
                .mark_ready()
        );
        assert!(app.all_required_ready());
        let expired = Instant::now();
        assert_eq!(app.complete_until(expired), Completion::Complete);
        assert!(app.failures().primary().is_none());
        let log = events.borrow();
        let order: Vec<_> = log.iter().map(|(label, _)| *label).collect();
        assert_eq!(order, ["observe", "activity", "inner", "outer", "root"]);
        assert!(log.iter().all(|(_, remaining)| remaining.is_zero()));
        drop(log);
        drop(app);
        assert_eq!(caller_owned.get(), 1);
        caller_owned.set(2);
    }

    #[test]
    fn failed_admission_and_unfinished_legacy_local_each_block_final_release() {
        let (worker, control) = HeldWorker::start(Outcome::Success);
        let admission_allowed = Rc::new(Cell::new(false));
        let local_allowed = Rc::new(Cell::new(false));
        let admission_gate = admission_allowed.clone();
        let local_gate = local_allowed.clone();
        let local_attempts = Rc::new(Cell::new(0));
        let local_calls = local_attempts.clone();
        let releases = Rc::new(Cell::new(0));
        let release_calls = releases.clone();
        let drops = Rc::new(Cell::new(0));
        let mut app = Builder::<Error>::new()
            .with_resource("final dependency", DropProbe(drops.clone()), move |_, _| {
                release_calls.set(release_calls.get() + 1);
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .required_local("legacy local", (), move |_| {
                local_calls.set(local_calls.get() + 1);
                if local_gate.get() {
                    Ok(())
                } else {
                    Err("legacy local not closed")
                }
            })
            .required_worker(
                "original worker",
                worker,
                move |worker| {
                    if admission_gate.get() {
                        worker.close_admission()
                    } else {
                        Err("admission not closed")
                    }
                },
                HeldWorker::signal_stop,
                HeldWorker::observe,
            )
            .build();
        assert_eq!(app.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(local_attempts.get(), 0);
        assert_eq!(app.parts().last().value().signal_calls, 0);
        assert_eq!(app.parts().last().value().observation_calls, 0);
        assert_eq!(releases.get(), 0);
        assert_eq!(drops.get(), 0);

        admission_allowed.set(true);
        app.request_stop();
        control.wait_for_stop();
        control.release();
        assert_eq!(app.complete_until(deadline()), Completion::Pending);
        assert_eq!(app.parts().last().join_status(), JoinStatus::Succeeded);
        assert_eq!(app.parts().last().value().observation_calls, 1);
        assert_eq!(releases.get(), 0);
        assert_eq!(drops.get(), 0);
        assert!(app.parts().previous().previous().last().value().is_some());

        local_allowed.set(true);
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(app.failures().primary(), Some(&"admission not closed"));
        assert!(
            app.failures()
                .secondary()
                .contains(&"legacy local not closed")
        );
        assert_eq!(app.parts().last().value().observation_calls, 1);
        assert_eq!(releases.get(), 1);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn managed_activity_release_pending_and_retry_do_not_repeat_original_join() {
        let (worker, control) = HeldWorker::start(Outcome::Success);
        let attempts = Rc::new(Cell::new(0));
        let release_attempts = attempts.clone();
        let drops = Rc::new(Cell::new(0));
        let mut app = Builder::<Error>::new()
            .with_resource("dependency", DropProbe(drops.clone()), |_, _| {
                Ok(ReleaseCompletion::Released(Ok(())))
            })
            .with_activity(
                "managed worker",
                worker,
                HeldWorker::close_admission,
                HeldWorker::signal_stop,
                HeldWorker::observe,
                move |worker, _| {
                    assert!(worker.join.is_none());
                    release_attempts.set(release_attempts.get() + 1);
                    match release_attempts.get() {
                        1 => Ok(ReleaseCompletion::Pending),
                        2 => Err("activity finalization retry"),
                        _ => Ok(ReleaseCompletion::Released(Ok(()))),
                    }
                },
            )
            .build();
        app.request_stop();
        control.wait_for_stop();
        control.release();
        assert_eq!(app.complete_until(deadline()), Completion::Pending);
        assert_eq!(app.parts().last().join_status(), JoinStatus::Succeeded);
        assert_eq!(app.parts().last().release_status(), ReleaseStatus::Pending);
        assert!(app.parts().last().value().join.is_none());
        assert_eq!(drops.get(), 0);
        assert_eq!(app.complete_until(deadline()), Completion::Pending);
        assert_eq!(
            app.failures().primary(),
            Some(&"activity finalization retry")
        );
        assert_eq!(app.parts().last().value().observation_calls, 1);
        assert_eq!(drops.get(), 0);

        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(
            app.parts().last().release_status(),
            ReleaseStatus::Succeeded
        );
        assert_eq!(app.parts().last().value().observation_calls, 1);
        assert_eq!(
            app.parts().last().value().observed_ids,
            [control.original_id]
        );
        assert_eq!(attempts.get(), 3);
        assert_eq!(drops.get(), 1);
        assert_eq!(app.complete_until(deadline()), Completion::Complete);
        assert_eq!(attempts.get(), 3);
    }
}
