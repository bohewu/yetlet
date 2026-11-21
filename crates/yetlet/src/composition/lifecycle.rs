use super::{
    Attached, Borrowed, Completion, Empty, Failures, JoinStatus, Local, Worker, WorkerCompletion,
};
use std::time::{Duration, Instant};

// Private static dispatch machinery. Neither trait is an extension surface.
pub(super) trait Part<E> {
    fn ready(&self) -> bool;
    fn close_admission(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn close_local(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn signal(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn observe(&mut self, _: Instant, _: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
    fn release(&mut self, _: Instant, _: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
}

pub(super) trait Chain<E> {
    fn all_ready(&self) -> bool;
    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool;
    fn close_locals(&mut self, failures: &mut Failures<E>) -> bool;
    fn signal_workers(&mut self, failures: &mut Failures<E>) -> bool;
    fn observe_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion;
    fn release_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion;
    // Ordinary RAII destruction for structural groups; no lifecycle callbacks.
    fn drop_parts_reverse(self);
}

impl<E> Chain<E> for Empty {
    fn all_ready(&self) -> bool {
        true
    }
    fn close_admission(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn close_locals(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn signal_workers(&mut self, _: &mut Failures<E>) -> bool {
        true
    }
    fn observe_owned(&mut self, _: Instant, _: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
    fn release_owned(&mut self, _: Instant, _: &mut Failures<E>) -> Completion {
        Completion::Complete
    }
    fn drop_parts_reverse(self) {}
}

impl<E, P: Chain<E>, T: Part<E>> Chain<E> for Attached<P, T> {
    fn all_ready(&self) -> bool {
        self.previous.all_ready() && self.last.ready()
    }

    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool {
        // Every gate gets attempted, even if another failed.
        let last = self.last.close_admission(failures);
        let previous = self.previous.close_admission(failures);
        last && previous
    }

    fn close_locals(&mut self, failures: &mut Failures<E>) -> bool {
        // Preserve earlier dependency owners if a later close has not succeeded.
        self.last.close_local(failures) && self.previous.close_locals(failures)
    }

    fn signal_workers(&mut self, failures: &mut Failures<E>) -> bool {
        let last = self.last.signal(failures);
        let previous = self.previous.signal_workers(failures);
        last && previous
    }

    fn observe_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion {
        let last = self.last.observe(deadline, failures);
        let previous = self.previous.observe_owned(deadline, failures);
        if last == Completion::Pending || previous == Completion::Pending {
            Completion::Pending
        } else {
            Completion::Complete
        }
    }

    fn release_owned(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion {
        if self.last.release(deadline, failures) == Completion::Pending {
            return Completion::Pending;
        }
        self.previous.release_owned(deadline, failures)
    }

    fn drop_parts_reverse(self) {
        drop(self.last);
        self.previous.drop_parts_reverse();
    }
}

impl<E, T, C: FnMut(&mut T) -> Result<(), E>> Part<E> for Local<T, C> {
    fn ready(&self) -> bool {
        self.is_ready()
    }

    fn close_local(&mut self, failures: &mut Failures<E>) -> bool {
        if self.stopped {
            return true;
        }
        match (self.close)(&mut self.value) {
            Ok(()) => {
                self.stopped = true;
                self.ready = false;
                true
            }
            Err(error) => {
                failures.record(error);
                false
            }
        }
    }
}

impl<E, T> Part<E> for Borrowed<'_, T> {
    fn ready(&self) -> bool {
        !self.stopped
    }
    fn close_local(&mut self, _: &mut Failures<E>) -> bool {
        self.stopped = true;
        true
    }
}

impl<E, T, A, S, O> Part<E> for Worker<T, A, S, O>
where
    A: FnMut(&mut T) -> Result<(), E>,
    S: FnMut(&mut T) -> Result<(), E>,
    O: FnMut(&mut T, Duration) -> Result<WorkerCompletion<E>, E>,
{
    fn ready(&self) -> bool {
        self.is_ready()
    }

    fn close_admission(&mut self, failures: &mut Failures<E>) -> bool {
        if self.admission_closed {
            return true;
        }
        match (self.admission)(&mut self.value) {
            Ok(()) => {
                self.admission_closed = true;
                self.ready = false;
                true
            }
            Err(error) => {
                failures.record(error);
                false
            }
        }
    }

    fn signal(&mut self, failures: &mut Failures<E>) -> bool {
        if self.stop_signalled || self.join_status != JoinStatus::Pending {
            return true;
        }
        match (self.signal)(&mut self.value) {
            Ok(()) => {
                self.stop_signalled = true;
                true
            }
            Err(error) => {
                failures.record(error);
                false
            }
        }
    }

    fn observe(&mut self, deadline: Instant, failures: &mut Failures<E>) -> Completion {
        if self.join_status != JoinStatus::Pending {
            return Completion::Complete;
        }
        match (self.observe)(
            &mut self.value,
            deadline.saturating_duration_since(Instant::now()),
        ) {
            Ok(WorkerCompletion::Pending) => Completion::Pending,
            Err(error) => {
                // Failure to observe is not proof that the original task ended.
                failures.record(error);
                Completion::Pending
            }
            Ok(WorkerCompletion::Joined(result)) => {
                self.join_status = match result {
                    Ok(()) => JoinStatus::Succeeded,
                    Err(error) => {
                        failures.record(error);
                        JoinStatus::Failed
                    }
                };
                Completion::Complete
            }
        }
    }
}
