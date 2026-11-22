//! Optional bounded command admission and immutable copied snapshots.
//!
//! Producers submit typed commands without depending on a transport or executor.
//! Application domain state remains caller-owned; admission never proves that
//! a command was applied. Snapshot readers cannot publish domain revisions.

use std::{
    collections::VecDeque,
    io,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    },
};

/// Typed queue admission is not application execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    /// Exact close fence rejected a submission.
    Closed,
    /// Data queue full; graceful shutdown does not use this queue.
    Full,
    /// Consumer no longer exists.
    Disconnected,
    /// Gate was poisoned, so safe admission/closure could not be confirmed.
    Poisoned,
}

struct Gate<C> {
    accepting: bool,
    tx: SyncSender<C>,
}

/// Cloneable producer-to-domain ingress with a bounded, fail-closed admission gate.
pub struct CommandIngress<C> {
    shared: Arc<Mutex<Gate<C>>>,
}

impl<C> Clone for CommandIngress<C> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<C> CommandIngress<C> {
    /// The same mutex serializes a valid try_send and closing admission.
    pub fn try_submit(&self, command: C) -> Result<(), AdmissionError> {
        // A rejected C may run arbitrary Drop code that reenters this gate.
        // Carry the rejected value out of the mutex scope before dropping it.
        let result = match self.shared.lock() {
            Ok(gate) if !gate.accepting => Err((AdmissionError::Closed, command)),
            Ok(gate) => match gate.tx.try_send(command) {
                Ok(()) => Ok(()),
                Err(TrySendError::Full(value)) => Err((AdmissionError::Full, value)),
                Err(TrySendError::Disconnected(value)) => {
                    Err((AdmissionError::Disconnected, value))
                }
            },
            Err(_) => Err((AdmissionError::Poisoned, command)),
        };
        match result {
            Ok(()) => Ok(()),
            Err((error, value)) => {
                drop(value); // only after the admission mutex is released
                Err(error)
            }
        }
    }

    /// Closes new commands before the authoritative domain is stopped.
    pub fn close(&self) -> io::Result<()> {
        let mut gate = self
            .shared
            .lock()
            .map_err(|_| io::Error::other("command gate poisoned"))?;
        gate.accepting = false;
        Ok(())
    }

    /// Observe the admission fence without authorizing a new command.
    pub fn is_closed(&self) -> io::Result<bool> {
        Ok(!self
            .shared
            .lock()
            .map_err(|_| io::Error::other("command gate poisoned"))?
            .accepting)
    }
}

/// Nonblocking command receiver, owned by the application's domain.
pub struct CommandDrain<C> {
    rx: Receiver<C>,
    shared: Arc<Mutex<Gate<C>>>,
}

impl<C> CommandDrain<C> {
    /// Receive one message; the SDK never applies it to a domain.
    pub fn try_recv(&self) -> Result<Option<C>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Explicitly discard unapplied commands only after the exact close fence.
    pub fn discard_after_close(&self) -> io::Result<usize> {
        // Validate this receiver's own permanently closed gate, never a
        // caller-supplied ingress from another bridge. Once closed it cannot
        // reopen, so release the guard before discarding user values.
        let gate = self
            .shared
            .lock()
            .map_err(|_| io::Error::other("command gate poisoned"))?;
        if gate.accepting {
            return Err(io::Error::other("admission remains open"));
        }
        drop(gate); // C::drop may reenter is_closed on this same bridge
        let mut discarded = 0;
        while self.rx.try_recv().is_ok() {
            discarded += 1;
        }
        Ok(discarded)
    }
}

/// Immutable copied state with caller-selected generation and revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Published<S> {
    /// Caller-owned authority epoch.
    pub generation: u64,
    /// Caller-owned, strictly increasing revision.
    pub revision: u64,
    /// Copied projection, never a second mutable domain.
    pub snapshot: S,
}

struct History<S> {
    // Arc permits retaining an immutable revision under the lock while
    // cloning caller-owned S only after the read guard is released.
    entries: VecDeque<Arc<Published<S>>>,
    capacity: usize,
}

/// Cloneable **read-only** copied snapshot handle for selected consumers.
///
/// Snapshot readers cannot publish an invented revision:
///
/// ```compile_fail
/// use yetlet::command_bridge::CommandBridge;
/// let bridge = CommandBridge::<u64, u64>::new(1, 2).unwrap();
/// let reader = bridge.snapshots();
/// reader.publish(1, 1, 9);
/// ```
pub struct SnapshotReader<S> {
    shared: Arc<RwLock<History<S>>>,
}

impl<S> Clone for SnapshotReader<S> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<S: Clone> SnapshotReader<S> {
    /// Reject stale or cross-generation snapshots rather than corrupt history.
    fn publish(&self, generation: u64, revision: u64, snapshot: S) -> io::Result<()> {
        let mut history = self
            .shared
            .write()
            .map_err(|_| io::Error::other("snapshot history poisoned"))?;
        if let Some(last) = history.entries.back()
            && (last.generation != generation || revision <= last.revision)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stale or cross-generation publication",
            ));
        }
        // Dropping the evicted application snapshot can call back into the
        // same reader. Retain it until the write lock has been released.
        let evicted = if history.entries.len() == history.capacity {
            history.entries.pop_front()
        } else {
            None
        };
        history.entries.push_back(Arc::new(Published {
            generation,
            revision,
            snapshot,
        }));
        drop(history);
        drop(evicted);
        Ok(())
    }

    /// Read latest immutable copied state, with its exact identity.
    pub fn latest(&self) -> io::Result<Option<Published<S>>> {
        let selected = {
            let history = self
                .shared
                .read()
                .map_err(|_| io::Error::other("snapshot history poisoned"))?;
            history.entries.back().cloned() // Arc clone only, never S::clone
        };
        Ok(selected.map(|entry| (*entry).clone())) // caller Clone outside guard
    }

    /// Read an exact pinned revision while it remains in bounded history.
    pub fn at_revision(&self, revision: u64) -> io::Result<Option<Published<S>>> {
        let selected = {
            let history = self
                .shared
                .read()
                .map_err(|_| io::Error::other("snapshot history poisoned"))?;
            history
                .entries
                .iter()
                .find(|item| item.revision == revision)
                .cloned() // Arc clone only
        };
        Ok(selected.map(|entry| (*entry).clone())) // caller Clone outside guard
    }
}

/// Bounded typed admission, one domain receiver and copied read-only history.
///
/// `C: Send` and `S: Clone + Send + Sync` apply only at this crossing edge.
/// A new authority generation uses a new bridge.
pub struct CommandBridge<C, S> {
    ingress: CommandIngress<C>,
    commands: CommandDrain<C>,
    snapshots: SnapshotReader<S>,
}

impl<C: Send + 'static, S: Clone + Send + Sync + 'static> CommandBridge<C, S> {
    /// No zero-sized unbounded/accidental capacity defaults.
    pub fn new(command_capacity: usize, snapshot_capacity: usize) -> io::Result<Self> {
        if command_capacity == 0 || snapshot_capacity == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "capacities must be nonzero",
            ));
        }
        let (tx, rx) = mpsc::sync_channel(command_capacity);
        let shared = Arc::new(Mutex::new(Gate {
            accepting: true,
            tx,
        }));
        Ok(Self {
            ingress: CommandIngress {
                shared: Arc::clone(&shared),
            },
            commands: CommandDrain { rx, shared },
            snapshots: SnapshotReader {
                shared: Arc::new(RwLock::new(History {
                    entries: VecDeque::new(),
                    capacity: snapshot_capacity,
                })),
            },
        })
    }

    /// Cloneable typed ingress for command producers.
    pub fn ingress(&self) -> CommandIngress<C> {
        self.ingress.clone()
    }
    /// Nonblocking receiver of the application's single domain owner.
    pub fn commands(&self) -> &CommandDrain<C> {
        &self.commands
    }
    /// Cloneable **read-only** projection for selected consumers.
    pub fn snapshots(&self) -> SnapshotReader<S> {
        self.snapshots.clone()
    }
    /// Only the domain-owned bridge may publish an applied revision.
    pub fn publish_snapshot(&self, generation: u64, revision: u64, snapshot: S) -> io::Result<()> {
        self.snapshots.publish(generation, revision, snapshot)
    }
    /// The exact fence used before stopping the domain owner.
    pub fn close_admission(&self) -> io::Result<()> {
        self.ingress.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            Mutex, Weak,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
        time::Duration,
    };

    #[derive(Clone)]
    struct ReentrantSnapshot {
        history: Weak<RwLock<History<Self>>>,
        observed: Arc<AtomicBool>,
        on_drop: bool,
    }

    impl Drop for ReentrantSnapshot {
        fn drop(&mut self) {
            if !self.on_drop {
                return;
            }
            let history = self.history.upgrade().expect("original reader still owned");
            // Deterministically fail, rather than deadlock, if eviction occurs
            // before the history write guard is released.
            assert!(
                history.try_read().is_ok(),
                "snapshot evicted under write lock"
            );
            let reader = SnapshotReader { shared: history };
            assert!(reader.at_revision(0).unwrap().is_none());
            self.observed.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn evicted_snapshot_destructor_can_read_same_revision_history() {
        let bridge = CommandBridge::<u64, ReentrantSnapshot>::new(1, 1).unwrap();
        let reader = bridge.snapshots();
        let observed = Arc::new(AtomicBool::new(false));
        let new_value = |on_drop| ReentrantSnapshot {
            history: Arc::downgrade(&reader.shared),
            observed: Arc::clone(&observed),
            on_drop,
        };
        bridge.publish_snapshot(7, 0, new_value(true)).unwrap();
        bridge.publish_snapshot(7, 1, new_value(false)).unwrap();
        assert!(observed.load(Ordering::SeqCst));
        assert_eq!(reader.latest().unwrap().unwrap().revision, 1);
    }

    struct CloneBlock {
        entered: mpsc::SyncSender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }

    struct PausingClone {
        value: u64,
        gate: Option<Arc<CloneBlock>>,
    }

    impl Clone for PausingClone {
        fn clone(&self) -> Self {
            if let Some(gate) = &self.gate {
                gate.entered.send(()).unwrap();
                gate.release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(3))
                    .expect("clone must be explicitly released");
            }
            Self {
                value: self.value,
                gate: None,
            }
        }
    }

    #[test]
    fn latest_and_exact_revision_clone_without_holding_history_read_guard() {
        for exact in [false, true] {
            let bridge = CommandBridge::<u64, PausingClone>::new(1, 1).unwrap();
            let (entered_tx, entered_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            let gate = Arc::new(CloneBlock {
                entered: entered_tx,
                release: Mutex::new(release_rx),
            });
            bridge
                .publish_snapshot(
                    7,
                    0,
                    PausingClone {
                        value: 42,
                        gate: Some(gate),
                    },
                )
                .unwrap();
            let reader = bridge.snapshots();
            let read_thread = thread::spawn(move || {
                if exact {
                    reader.at_revision(0).unwrap().unwrap()
                } else {
                    reader.latest().unwrap().unwrap()
                }
            });
            entered_rx
                .recv_timeout(Duration::from_secs(3))
                .expect("reader entered its caller-controlled Clone");

            let writer = bridge.snapshots();
            let (published_tx, published_rx) = mpsc::sync_channel(1);
            let write_thread = thread::spawn(move || {
                let result = writer.publish(
                    7,
                    1,
                    PausingClone {
                        value: 99,
                        gate: None,
                    },
                );
                published_tx.send(result).unwrap();
            });
            // The writer must publish while the first reader's Clone is
            // deliberately blocked. Release and join even if this check fails.
            let progressed = published_rx
                .recv_timeout(Duration::from_millis(400))
                .is_ok();
            release_tx.send(()).unwrap();
            write_thread.join().unwrap();
            let selected = read_thread.join().unwrap();
            assert!(
                progressed,
                "foreground publish must not wait for S::clone (exact={exact})"
            );
            assert_eq!(selected.generation, 7);
            assert_eq!(selected.revision, 0);
            assert_eq!(selected.snapshot.value, 42);
            assert_eq!(bridge.snapshots().latest().unwrap().unwrap().revision, 1);
        }
    }
}
