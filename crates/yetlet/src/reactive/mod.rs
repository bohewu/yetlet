//! Explicitly owned synchronous local update graph for Yetlet.
//!
//! The graph is an optional, renderer-neutral primitive. Messages are queued
//! FIFO, a batch is reduced against one state draft, and one commit plus one
//! observer notification occurs per successful pass. Dispatch during a
//! commit/observer is deferred to the next bounded pass rather than recursing.
//! This crate performs no I/O, browser access, server work, dependency tracking,
//! async scheduling, or transport.

use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    fmt,
    rc::{Rc, Weak},
};

type Reducer<State, Message, Error> = Box<dyn FnMut(&mut State, Message) -> Result<(), Error>>;
type Commit<State, Error> = Box<dyn FnMut(&State) -> Result<(), Error>>;
type Observer<State> = Rc<RefCell<Box<dyn FnMut(&State)>>>;

/// Work completed by one outer dispatch/batch call.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FlushStats {
    pub messages: usize,
    pub passes: usize,
    pub commits: usize,
}

/// Local graph failures. Reducer failures keep the previous state and remain
/// recoverable. Commit failures and re-entrant pass exhaustion fault the owner,
/// preventing later propagation until the owner is disposed and rebuilt.
#[derive(Debug, PartialEq, Eq)]
pub enum ReactiveError<Error> {
    Disposed,
    Faulted,
    ObserverIdExhausted,
    Reducer(Error),
    Commit(Error),
    ReentrantLimit { limit: usize },
}

impl<Error: fmt::Display> fmt::Display for ReactiveError<Error> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disposed => formatter.write_str("reactive graph is disposed"),
            Self::Faulted => formatter.write_str("reactive graph is faulted"),
            Self::ObserverIdExhausted => formatter.write_str("reactive observer ID exhausted"),
            Self::Reducer(error) => write!(formatter, "reactive reducer failed: {error}"),
            Self::Commit(error) => write!(formatter, "reactive commit failed: {error}"),
            Self::ReentrantLimit { limit } => {
                write!(
                    formatter,
                    "reactive re-entrant pass limit exceeded: {limit}"
                )
            }
        }
    }
}

impl<Error> std::error::Error for ReactiveError<Error> where Error: std::error::Error + 'static {}

struct Inner<State, Message, Error> {
    state: Option<Rc<State>>,
    reducer: Option<Reducer<State, Message, Error>>,
    commit: Option<Commit<State, Error>>,
    queue: VecDeque<Message>,
    observers: BTreeMap<u64, Observer<State>>,
    next_observer_id: u64,
    flushing: bool,
    disposed: bool,
    faulted: bool,
    max_passes: usize,
}

/// Values detached from `Inner` and deliberately dropped only after its
/// `RefCell` borrow has ended. Captured subscriptions may re-enter the graph
/// during drop, so dropping these fields while a borrow is active is unsafe.
struct Detached<State, Message, Error> {
    _state: Option<Rc<State>>,
    _queue: VecDeque<Message>,
    _observers: BTreeMap<u64, Observer<State>>,
    _reducer: Option<Reducer<State, Message, Error>>,
    _commit: Option<Commit<State, Error>>,
}

fn detach_owned<State, Message, Error>(
    inner: &mut Inner<State, Message, Error>,
) -> Detached<State, Message, Error> {
    Detached {
        _state: inner.state.take(),
        _queue: std::mem::take(&mut inner.queue),
        _observers: std::mem::take(&mut inner.observers),
        _reducer: inner.reducer.take(),
        _commit: inner.commit.take(),
    }
}

/// The one strong owner of a local reactive graph.
///
/// `GraphHandle` and `Subscription` use weak references, so dropping/disposing
/// the owner releases reducer, renderer, observer, and queued-message captures.
/// This type is intentionally not `Clone`.
pub struct LocalGraph<State, Message, Error> {
    inner: Rc<RefCell<Inner<State, Message, Error>>>,
}

/// Weak read/dispatch capability. It cannot keep a graph alive.
pub struct GraphHandle<State, Message, Error> {
    inner: Weak<RefCell<Inner<State, Message, Error>>>,
}

impl<State, Message, Error> Clone for GraphHandle<State, Message, Error> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Explicit observer ownership. Dropping or calling `unsubscribe` removes the
/// observer; the handle does not keep the graph alive.
pub struct Subscription<State, Message, Error> {
    inner: Weak<RefCell<Inner<State, Message, Error>>>,
    id: u64,
    active: bool,
}

impl<State, Message, Error> Subscription<State, Message, Error> {
    pub fn unsubscribe(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.active = false;
        let removed = self.inner.upgrade().and_then(|inner| {
            // Return the callback from this scope so it is dropped after the
            // mutable graph borrow has ended.
            inner.borrow_mut().observers.remove(&self.id)
        });
        let existed = removed.is_some();
        drop(removed);
        existed
    }
}

impl<State, Message, Error> Drop for Subscription<State, Message, Error> {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

impl<State, Message, Error> LocalGraph<State, Message, Error>
where
    State: Clone + 'static,
    Message: 'static,
    Error: 'static,
{
    pub const DEFAULT_MAX_PASSES: usize = 64;

    pub fn new(
        state: State,
        reducer: impl FnMut(&mut State, Message) -> Result<(), Error> + 'static,
        commit: impl FnMut(&State) -> Result<(), Error> + 'static,
    ) -> Self {
        Self::with_pass_limit(state, reducer, commit, Self::DEFAULT_MAX_PASSES)
    }

    pub fn with_pass_limit(
        state: State,
        reducer: impl FnMut(&mut State, Message) -> Result<(), Error> + 'static,
        commit: impl FnMut(&State) -> Result<(), Error> + 'static,
        max_passes: usize,
    ) -> Self {
        assert!(max_passes > 0, "reactive pass limit must be positive");
        Self {
            inner: Rc::new(RefCell::new(Inner {
                state: Some(Rc::new(state)),
                reducer: Some(Box::new(reducer)),
                commit: Some(Box::new(commit)),
                queue: VecDeque::new(),
                observers: BTreeMap::new(),
                next_observer_id: 1,
                flushing: false,
                disposed: false,
                faulted: false,
                max_passes,
            })),
        }
    }

    pub fn handle(&self) -> GraphHandle<State, Message, Error> {
        GraphHandle {
            inner: Rc::downgrade(&self.inner),
        }
    }

    /// Clone the published state, with the same semantics as
    /// [`GraphHandle::snapshot`].
    pub fn snapshot(&self) -> Result<State, ReactiveError<Error>> {
        snapshot(&self.inner)
    }

    pub fn subscribe(
        &self,
        observer: impl FnMut(&State) + 'static,
    ) -> Result<Subscription<State, Message, Error>, ReactiveError<Error>> {
        let callback: Observer<State> = Rc::new(RefCell::new(Box::new(observer)));
        let mut inner = self.inner.borrow_mut();
        if inner.disposed {
            drop(inner);
            drop(callback);
            return Err(ReactiveError::Disposed);
        }
        if inner.faulted {
            drop(inner);
            drop(callback);
            return Err(ReactiveError::Faulted);
        }
        let id = inner.next_observer_id;
        let Some(next_id) = inner.next_observer_id.checked_add(1) else {
            drop(inner);
            drop(callback);
            return Err(ReactiveError::ObserverIdExhausted);
        };
        inner.next_observer_id = next_id;
        let replaced = inner.observers.insert(id, callback);
        drop(inner);
        debug_assert!(replaced.is_none());
        drop(replaced);
        Ok(Subscription {
            inner: Rc::downgrade(&self.inner),
            id,
            active: true,
        })
    }

    pub fn dispose(&mut self) -> bool {
        let detached = {
            let mut inner = self.inner.borrow_mut();
            if inner.disposed {
                return false;
            }
            inner.disposed = true;
            detach_owned(&mut inner)
        };
        drop(detached);
        true
    }
}

impl<State, Message, Error> Drop for LocalGraph<State, Message, Error> {
    fn drop(&mut self) {
        let detached = {
            let mut inner = self.inner.borrow_mut();
            if inner.disposed {
                None
            } else {
                inner.disposed = true;
                Some(detach_owned(&mut inner))
            }
        };
        drop(detached);
    }
}

impl<State, Message, Error> GraphHandle<State, Message, Error>
where
    State: Clone + 'static,
    Message: 'static,
    Error: 'static,
{
    /// Clone the graph's published state without flushing queued messages or
    /// invoking its reducer, commit callback or observers.
    ///
    /// Before the first dispatch this reads the initial state. During a reducer
    /// or commit callback it reads the previous published state; an observer
    /// reads the newly published state, before any queued later pass executes.
    /// Reads do not guarantee that a re-entrant dispatch has completed.
    ///
    /// Returns [`ReactiveError::Disposed`] after disposal or owner drop, and
    /// [`ReactiveError::Faulted`] for a live faulted graph. Disposal takes
    /// precedence. The graph borrow ends before calling `State::clone`, so
    /// user-defined clone code may re-enter it. This is a read when the state
    /// reference is acquired, not a freshness guarantee after cloning or await.
    /// The returned value follows `State`'s clone semantics, which may share
    /// interior-mutable data. User-defined clone code controls its own side
    /// effects; the handle remains weak and this method adds no graph ownership
    /// to the returned value.
    pub fn snapshot(&self) -> Result<State, ReactiveError<Error>> {
        let inner = self.inner.upgrade().ok_or(ReactiveError::Disposed)?;
        snapshot(&inner)
    }

    pub fn dispatch(&self, message: Message) -> Result<FlushStats, ReactiveError<Error>> {
        self.batch([message])
    }

    /// Queue all messages in order and, when called outside an active flush,
    /// reduce them into one draft and perform one commit/notification pass.
    pub fn batch(
        &self,
        messages: impl IntoIterator<Item = Message>,
    ) -> Result<FlushStats, ReactiveError<Error>> {
        // Consume and drop the caller's iterator before borrowing the graph.
        // Iterator closures may own Subscriptions whose Drop re-enters it.
        let incoming = messages.into_iter().collect::<VecDeque<_>>();
        let inner = self.inner.upgrade().ok_or(ReactiveError::Disposed)?;
        let mut core = inner.borrow_mut();
        if core.disposed {
            drop(core);
            drop(incoming);
            return Err(ReactiveError::Disposed);
        }
        if core.faulted {
            drop(core);
            drop(incoming);
            return Err(ReactiveError::Faulted);
        }
        if incoming.is_empty() {
            drop(core);
            return Ok(FlushStats::default());
        }
        core.queue.extend(incoming);
        if core.flushing {
            return Ok(FlushStats::default());
        }
        core.flushing = true;
        drop(core);

        let outcome = flush(&inner);
        inner.borrow_mut().flushing = false;
        outcome
    }
}

fn snapshot<State: Clone, Message, Error>(
    shared: &RefCell<Inner<State, Message, Error>>,
) -> Result<State, ReactiveError<Error>> {
    let state = {
        let inner = shared.borrow();
        if inner.disposed {
            return Err(ReactiveError::Disposed);
        }
        if inner.faulted {
            return Err(ReactiveError::Faulted);
        }
        Rc::clone(
            inner
                .state
                .as_ref()
                .expect("active reactive graph owns committed state"),
        )
    };
    Ok((*state).clone())
}

fn flush<State, Message, Error>(
    shared: &Rc<RefCell<Inner<State, Message, Error>>>,
) -> Result<FlushStats, ReactiveError<Error>>
where
    State: Clone + 'static,
    Message: 'static,
    Error: 'static,
{
    let mut stats = FlushStats::default();
    loop {
        let limit_cleanup = {
            let mut inner = shared.borrow_mut();
            if inner.disposed {
                return Err(ReactiveError::Disposed);
            }
            if inner.faulted {
                return Err(ReactiveError::Faulted);
            }
            if inner.queue.is_empty() {
                return Ok(stats);
            }
            if stats.passes >= inner.max_passes {
                let limit = inner.max_passes;
                inner.faulted = true;
                Some((limit, std::mem::take(&mut inner.queue)))
            } else {
                None
            }
        };
        if let Some((limit, discarded)) = limit_cleanup {
            drop(discarded);
            if shared.borrow().disposed {
                return Err(ReactiveError::Disposed);
            }
            return Err(ReactiveError::ReentrantLimit { limit });
        }

        let (messages, committed_state, mut reducer, mut commit, pass_limit) = {
            let mut inner = shared.borrow_mut();
            let messages = std::mem::take(&mut inner.queue);
            let state = Rc::clone(
                inner
                    .state
                    .as_ref()
                    .expect("active reactive graph owns committed state"),
            );
            let reducer = inner
                .reducer
                .take()
                .expect("active reactive graph owns one reducer");
            let commit = inner
                .commit
                .take()
                .expect("active reactive graph owns one commit callback");
            (messages, state, reducer, commit, inner.max_passes)
        };
        let mut draft = (*committed_state).clone();
        drop(committed_state);
        if shared.borrow().disposed {
            drop(messages);
            drop(reducer);
            drop(commit);
            return Err(ReactiveError::Disposed);
        }

        let mut messages = messages.into_iter();
        while let Some(message) = messages.next() {
            if let Err(error) = reducer(&mut draft, message) {
                // Temporarily reject re-entrant dispatch while every value
                // owned by the failed pass is destroyed. Their destructors may
                // call back into this graph and must not leave replayable work.
                let queued = {
                    let mut inner = shared.borrow_mut();
                    inner.faulted = true;
                    std::mem::take(&mut inner.queue)
                };
                drop(messages);
                drop(queued);
                drop(draft);
                if shared.borrow().disposed {
                    drop(reducer);
                    drop(commit);
                    return Err(ReactiveError::Disposed);
                }
                {
                    let mut inner = shared.borrow_mut();
                    debug_assert!(inner.queue.is_empty());
                    inner.reducer = Some(reducer);
                    inner.commit = Some(commit);
                    inner.faulted = false;
                }
                return Err(ReactiveError::Reducer(error));
            }
            stats.messages += 1;

            // A reducer may explicitly tear down its external owner. Stop the
            // active pass immediately: do not reduce another drained message
            // and never call the renderer after disposal.
            if shared.borrow().disposed {
                drop(messages);
                drop(reducer);
                drop(commit);
                return Err(ReactiveError::Disposed);
            }
        }

        if let Err(error) = commit(&draft) {
            // Fault before dropping queued work or the rejected draft. Any
            // destructor-triggered dispatch is rejected and cannot survive for
            // an unrelated later operation.
            let queued = {
                let mut inner = shared.borrow_mut();
                inner.faulted = true;
                std::mem::take(&mut inner.queue)
            };
            drop(queued);
            drop(draft);
            if shared.borrow().disposed {
                drop(reducer);
                drop(commit);
                return Err(ReactiveError::Disposed);
            }
            {
                let mut inner = shared.borrow_mut();
                debug_assert!(inner.queue.is_empty());
                inner.reducer = Some(reducer);
                inner.commit = Some(commit);
            }
            return Err(ReactiveError::Commit(error));
        }

        if shared.borrow().disposed {
            drop(reducer);
            drop(commit);
            return Err(ReactiveError::Disposed);
        }

        let next_state = Rc::new(draft);
        let (previous_state, observers) = {
            let mut inner = shared.borrow_mut();
            inner.reducer = Some(reducer);
            inner.commit = Some(commit);
            let previous_state = inner.state.replace(Rc::clone(&next_state));
            stats.passes += 1;
            stats.commits += 1;
            (
                previous_state,
                inner
                    .observers
                    .iter()
                    .map(|(id, callback)| (*id, Rc::clone(callback)))
                    .collect::<Vec<_>>(),
            )
        };
        // State destructors may own Subscriptions, so release the previous
        // commit only after the graph borrow ends.
        drop(previous_state);
        if shared.borrow().disposed {
            drop(next_state);
            drop(observers);
            return Err(ReactiveError::Disposed);
        }
        let snapshot = (*next_state).clone();
        drop(next_state);
        if shared.borrow().disposed {
            drop(snapshot);
            drop(observers);
            return Err(ReactiveError::Disposed);
        }

        for (id, observer) in observers {
            let should_run = {
                let inner = shared.borrow();
                !inner.disposed && inner.observers.contains_key(&id)
            };
            if !should_run {
                continue;
            }
            (observer.borrow_mut())(&snapshot);
        }

        debug_assert!(stats.passes <= pass_limit);
    }
}
#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use super::{FlushStats, GraphHandle, LocalGraph, ReactiveError, Subscription};

    #[test]
    fn handle_snapshots_match_owner_without_running_callbacks() {
        let reductions = Rc::new(Cell::new(0));
        let reduced = Rc::clone(&reductions);
        let commits = Rc::new(Cell::new(0));
        let committed = Rc::clone(&commits);
        let notifications = Rc::new(Cell::new(0));
        let notified = Rc::clone(&notifications);
        let graph = LocalGraph::new(
            vec![1],
            move |state, message| {
                reduced.set(reduced.get() + 1);
                state.push(message);
                Ok::<_, &'static str>(())
            },
            move |_state| {
                committed.set(committed.get() + 1);
                Ok(())
            },
        );
        let handle = graph.handle();
        let _subscription = graph
            .subscribe(move |_state| notified.set(notified.get() + 1))
            .unwrap();
        let initial = handle.snapshot().unwrap();
        assert_eq!(initial, vec![1]);
        assert_eq!(graph.snapshot(), Ok(initial.clone()));
        assert_eq!(
            (reductions.get(), commits.get(), notifications.get()),
            (0, 0, 0)
        );

        handle.batch([2, 3]).unwrap();
        assert_eq!(handle.snapshot(), Ok(vec![1, 2, 3]));
        assert_eq!(graph.snapshot(), handle.snapshot());
        assert_eq!(
            (reductions.get(), commits.get(), notifications.get()),
            (2, 1, 1)
        );
        drop(graph);
        assert_eq!(initial, vec![1]);
        assert_eq!(handle.snapshot(), Err(ReactiveError::Disposed));
    }

    #[test]
    fn handle_snapshot_reads_published_state_in_each_flush_phase() {
        type Handle = GraphHandle<i32, i32, &'static str>;
        let slot: Rc<RefCell<Option<Handle>>> = Rc::new(RefCell::new(None));
        let reads = Rc::new(RefCell::new(Vec::new()));
        let reducer_slot = Rc::clone(&slot);
        let reducer_reads = Rc::clone(&reads);
        let commit_slot = Rc::clone(&slot);
        let commit_reads = Rc::clone(&reads);
        let graph = LocalGraph::new(
            0_i32,
            move |state, message| {
                *state += message;
                let handle = reducer_slot.borrow().as_ref().unwrap().clone();
                reducer_reads
                    .borrow_mut()
                    .push(("reducer", handle.snapshot().unwrap()));
                Ok::<_, &'static str>(())
            },
            move |_state| {
                let handle = commit_slot.borrow().as_ref().unwrap().clone();
                commit_reads
                    .borrow_mut()
                    .push(("commit", handle.snapshot().unwrap()));
                Ok(())
            },
        );
        let handle = graph.handle();
        *slot.borrow_mut() = Some(handle.clone());
        let observer_handle = handle.clone();
        let observer_reads = Rc::clone(&reads);
        let _subscription = graph
            .subscribe(move |state| {
                observer_reads
                    .borrow_mut()
                    .push(("observer", observer_handle.snapshot().unwrap()));
                if *state == 1 {
                    assert_eq!(observer_handle.dispatch(2), Ok(FlushStats::default()));
                    observer_reads
                        .borrow_mut()
                        .push(("queued", observer_handle.snapshot().unwrap()));
                }
            })
            .unwrap();

        let stats = handle.dispatch(1).unwrap();
        assert_eq!((stats.messages, stats.passes, stats.commits), (2, 2, 2));
        assert_eq!(handle.snapshot(), Ok(3));
        assert_eq!(graph.snapshot(), Ok(3));
        assert_eq!(
            *reads.borrow(),
            vec![
                ("reducer", 0),
                ("commit", 0),
                ("observer", 1),
                ("queued", 1),
                ("reducer", 1),
                ("commit", 1),
                ("observer", 3),
            ]
        );
    }

    #[test]
    fn snapshots_release_the_graph_borrow_before_cloning_state() {
        type CloneHook = Rc<RefCell<Option<Box<dyn Fn()>>>>;
        struct State {
            value: i32,
            on_clone: CloneHook,
        }
        impl Clone for State {
            fn clone(&self) -> Self {
                let callback = self.on_clone.borrow_mut().take();
                if let Some(callback) = callback {
                    callback();
                }
                Self {
                    value: self.value,
                    on_clone: Rc::clone(&self.on_clone),
                }
            }
        }
        let hook: CloneHook = Rc::new(RefCell::new(None));
        let graph = LocalGraph::new(
            State {
                value: 7,
                on_clone: Rc::clone(&hook),
            },
            |_state, _message: ()| Ok::<_, &'static str>(()),
            |_state| Ok(()),
        );
        for read_through_handle in [false, true] {
            let handle = graph.handle();
            *hook.borrow_mut() = Some(Box::new(move || {
                // An empty batch takes a mutable graph borrow without running
                // callbacks. This panics if snapshot still holds its borrow.
                assert_eq!(handle.batch([]), Ok(FlushStats::default()));
            }));
            let state = if read_through_handle {
                graph.handle().snapshot().unwrap()
            } else {
                graph.snapshot().unwrap()
            };
            assert_eq!(state.value, 7);
            assert!(hook.borrow().is_none());
        }
    }

    #[test]
    fn batch_coalesces_messages_into_one_commit_and_notification() {
        let commits = Rc::new(Cell::new(0));
        let observations = Rc::new(RefCell::new(Vec::new()));
        let commit_count = Rc::clone(&commits);
        let graph = LocalGraph::new(
            0_i32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            move |_state| {
                commit_count.set(commit_count.get() + 1);
                Ok(())
            },
        );
        let observed = Rc::clone(&observations);
        let mut subscription = graph
            .subscribe(move |state| observed.borrow_mut().push(*state))
            .unwrap();

        let stats = graph.handle().batch([1, 2, 3]).unwrap();
        assert_eq!(
            stats,
            FlushStats {
                messages: 3,
                passes: 1,
                commits: 1,
            }
        );
        assert_eq!(graph.snapshot(), Ok(6));
        assert_eq!(commits.get(), 1);
        assert_eq!(*observations.borrow(), vec![6]);

        assert!(subscription.unsubscribe());
        graph.handle().dispatch(1).unwrap();
        assert_eq!(*observations.borrow(), vec![6]);
    }

    #[test]
    fn reentrant_dispatch_is_fifo_and_runs_in_a_later_pass() {
        let commits = Rc::new(Cell::new(0));
        let commit_count = Rc::clone(&commits);
        let graph = LocalGraph::new(
            0_i32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            move |_state| {
                commit_count.set(commit_count.get() + 1);
                Ok(())
            },
        );
        let handle = graph.handle();
        let reentrant = handle.clone();
        let queued = Rc::new(Cell::new(false));
        let queued_once = Rc::clone(&queued);
        let _subscription = graph
            .subscribe(move |state| {
                if *state == 1 && !queued_once.replace(true) {
                    reentrant.dispatch(2).unwrap();
                }
            })
            .unwrap();

        let stats = handle.dispatch(1).unwrap();
        assert_eq!(
            stats,
            FlushStats {
                messages: 2,
                passes: 2,
                commits: 2,
            }
        );
        assert_eq!(graph.snapshot(), Ok(3));
        assert_eq!(commits.get(), 2);
    }

    #[test]
    fn reducer_failure_keeps_state_and_graph_recoverable() {
        let commits = Rc::new(Cell::new(0));
        let commit_count = Rc::clone(&commits);
        let graph = LocalGraph::new(
            5_i32,
            |state, message| {
                if message < 0 {
                    return Err("negative");
                }
                *state += message;
                Ok(())
            },
            move |_state| {
                commit_count.set(commit_count.get() + 1);
                Ok(())
            },
        );

        assert_eq!(
            graph.handle().dispatch(-1),
            Err(ReactiveError::Reducer("negative"))
        );
        assert_eq!(graph.snapshot(), Ok(5));
        assert_eq!(graph.handle().snapshot(), Ok(5));
        assert_eq!(commits.get(), 0);
        graph.handle().dispatch(2).unwrap();
        assert_eq!(graph.snapshot(), Ok(7));
        assert_eq!(graph.handle().snapshot(), Ok(7));
    }

    #[test]
    fn commit_failure_faults_graph_without_committing_state() {
        let mut graph = LocalGraph::new(
            0_i32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            |state| if *state == 1 { Err("render") } else { Ok(()) },
        );

        assert_eq!(
            graph.handle().dispatch(1),
            Err(ReactiveError::Commit("render"))
        );
        assert_eq!(graph.snapshot(), Err(ReactiveError::Faulted));
        let handle = graph.handle();
        assert_eq!(handle.snapshot(), Err(ReactiveError::Faulted));
        assert_eq!(graph.handle().dispatch(2), Err(ReactiveError::Faulted));
        assert!(graph.dispose());
        assert_eq!(graph.snapshot(), Err(ReactiveError::Disposed));
        assert_eq!(handle.snapshot(), Err(ReactiveError::Disposed));
    }

    type CleanupHandle = GraphHandle<i32, CleanupMessage, &'static str>;

    enum CleanupMessage {
        Add(i32),
        Reject,
        DispatchOnDrop { handle: CleanupHandle, value: i32 },
    }

    impl Drop for CleanupMessage {
        fn drop(&mut self) {
            if let Self::DispatchOnDrop { handle, value } = self {
                let _ = handle.dispatch(Self::Add(*value));
            }
        }
    }

    fn cleanup_reducer(state: &mut i32, message: CleanupMessage) -> Result<(), &'static str> {
        match &message {
            CleanupMessage::Add(value) => *state += *value,
            CleanupMessage::Reject => return Err("reject"),
            CleanupMessage::DispatchOnDrop { .. } => {}
        }
        Ok(())
    }

    #[test]
    fn reducer_failure_discards_destructor_generated_messages() {
        let graph = LocalGraph::new(0_i32, cleanup_reducer, |_state| Ok(()));
        let handle = graph.handle();
        let result = handle.batch([
            CleanupMessage::Reject,
            CleanupMessage::DispatchOnDrop {
                handle: handle.clone(),
                value: 100,
            },
        ]);

        assert_eq!(result, Err(ReactiveError::Reducer("reject")));
        let inner = handle.inner.upgrade().unwrap();
        assert!(inner.borrow().queue.is_empty());
        handle.dispatch(CleanupMessage::Add(1)).unwrap();
        assert_eq!(graph.snapshot(), Ok(1));
    }

    #[test]
    fn commit_failure_discards_cleanup_generated_messages() {
        let handle_slot: Rc<RefCell<Option<CleanupHandle>>> = Rc::new(RefCell::new(None));
        let commit_handle = Rc::clone(&handle_slot);
        let graph = LocalGraph::new(0_i32, cleanup_reducer, move |_state| {
            let handle = commit_handle
                .borrow()
                .as_ref()
                .expect("test installs handle before dispatch")
                .clone();
            handle
                .dispatch(CleanupMessage::DispatchOnDrop {
                    handle: handle.clone(),
                    value: 100,
                })
                .expect("re-entrant dispatch is queued during commit");
            Err("render")
        });
        let handle = graph.handle();
        *handle_slot.borrow_mut() = Some(handle.clone());

        assert_eq!(
            handle.dispatch(CleanupMessage::Add(1)),
            Err(ReactiveError::Commit("render"))
        );
        let inner = handle.inner.upgrade().unwrap();
        let inner = inner.borrow();
        assert!(inner.faulted);
        assert!(inner.queue.is_empty());
    }

    #[test]
    fn pass_limit_stops_unbounded_reentrant_observers() {
        let graph = LocalGraph::with_pass_limit(
            0_u32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            |_state| Ok(()),
            3,
        );
        let handle = graph.handle();
        let reentrant = handle.clone();
        let _subscription = graph
            .subscribe(move |_state| {
                reentrant.dispatch(1).unwrap();
            })
            .unwrap();

        assert_eq!(
            handle.dispatch(1),
            Err(ReactiveError::ReentrantLimit { limit: 3 })
        );
        assert_eq!(handle.dispatch(1), Err(ReactiveError::Faulted));
        assert_eq!(graph.snapshot(), Err(ReactiveError::Faulted));
        assert_eq!(handle.snapshot(), Err(ReactiveError::Faulted));
    }

    #[test]
    fn batch_consumes_iterator_captures_before_graph_borrow() {
        let notifications = Rc::new(Cell::new(0_u32));
        let observed = Rc::clone(&notifications);
        let graph = LocalGraph::new(
            0_i32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            |_state| Ok(()),
        );
        let subscription = graph
            .subscribe(move |_state| observed.set(observed.get() + 1))
            .unwrap();
        let iterator = std::iter::empty::<i32>().inspect(move |_| {
            let _ = &subscription;
        });

        assert_eq!(graph.handle().batch(iterator), Ok(FlushStats::default()));
        graph.handle().dispatch(1).unwrap();
        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn state_replacement_drops_owned_subscription_outside_graph_borrow() {
        type Stored = Rc<RefCell<Option<Subscription<State, Message, &'static str>>>>;

        #[derive(Clone)]
        struct State {
            value: i32,
            stored: Option<Stored>,
        }

        enum Message {
            Store(Stored),
            Clear,
            Increment,
        }

        let graph = LocalGraph::new(
            State {
                value: 0,
                stored: None,
            },
            |state, message| {
                match message {
                    Message::Store(subscription) => state.stored = Some(subscription),
                    Message::Clear => state.stored = None,
                    Message::Increment => state.value += 1,
                }
                Ok::<_, &'static str>(())
            },
            |_state| Ok(()),
        );
        let subscription = graph.subscribe(|_state| {}).unwrap();
        let stored = Rc::new(RefCell::new(Some(subscription)));
        graph
            .handle()
            .dispatch(Message::Store(Rc::clone(&stored)))
            .unwrap();
        drop(stored);

        // Replacing committed state drops its final Stored Rc, which drops the
        // Subscription and re-enters unsubscribe. The graph borrow must already
        // be released, and later updates must remain usable.
        graph.handle().dispatch(Message::Clear).unwrap();
        graph.handle().dispatch(Message::Increment).unwrap();
        let snapshot = graph.snapshot().unwrap();
        assert_eq!(snapshot.value, 1);
        assert!(snapshot.stored.is_none());
    }

    #[test]
    fn cleanup_drops_nested_subscription_captures_outside_graph_borrow() {
        let mut graph = LocalGraph::new(
            0_i32,
            |_state, _message: ()| Ok::<_, &'static str>(()),
            |_state| Ok(()),
        );
        let first = graph.subscribe(|_state| {}).unwrap();
        let dropped = Rc::new(Cell::new(false));
        let dropped_capture = Rc::clone(&dropped);
        let second = graph
            .subscribe(move |_state| {
                let _ = &first;
                dropped_capture.set(true);
            })
            .unwrap();

        assert!(graph.dispose());
        // Disposing drops the second observer, which drops its captured first
        // subscription and re-enters unsubscribe. The test would panic if any
        // graph RefCell borrow were still held.
        assert!(!dropped.get());
        drop(second);
    }

    #[test]
    fn reducer_disposal_aborts_remaining_messages_and_skips_commit() {
        type Graph = LocalGraph<i32, i32, &'static str>;

        let owner: Rc<RefCell<Option<Graph>>> = Rc::new(RefCell::new(None));
        let reducer_owner = Rc::clone(&owner);
        let reducer_calls = Rc::new(Cell::new(0_u32));
        let calls = Rc::clone(&reducer_calls);
        let commits = Rc::new(Cell::new(0_u32));
        let commit_count = Rc::clone(&commits);
        let graph = LocalGraph::new(
            0_i32,
            move |state, message| {
                calls.set(calls.get() + 1);
                *state += message;
                if message == 1 {
                    let retired = reducer_owner.borrow_mut().take();
                    drop(retired);
                }
                Ok(())
            },
            move |_state| {
                commit_count.set(commit_count.get() + 1);
                Ok(())
            },
        );
        let handle = graph.handle();
        *owner.borrow_mut() = Some(graph);

        assert_eq!(handle.batch([1, 2]), Err(ReactiveError::Disposed));
        assert_eq!(reducer_calls.get(), 1);
        assert_eq!(commits.get(), 0);
        assert_eq!(handle.dispatch(1), Err(ReactiveError::Disposed));
    }

    #[test]
    fn weak_handles_and_subscriptions_do_not_keep_owned_captures_alive() {
        #[derive(Clone)]
        struct DropProbe(Rc<Cell<u32>>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let dropped = Rc::new(Cell::new(0_u32));
        let handle;
        let subscription;
        {
            let state = DropProbe(Rc::clone(&dropped));
            let reducer_probe = DropProbe(Rc::clone(&dropped));
            let commit_probe = DropProbe(Rc::clone(&dropped));
            let observer_probe = DropProbe(Rc::clone(&dropped));
            let graph = LocalGraph::new(
                state,
                move |_state, _message: ()| {
                    let _ = &reducer_probe;
                    Ok::<_, &'static str>(())
                },
                move |_state| {
                    let _ = &commit_probe;
                    Ok(())
                },
            );
            handle = graph.handle();
            subscription = graph
                .subscribe(move |_state| {
                    let _ = &observer_probe;
                })
                .unwrap();
        }

        assert_eq!(dropped.get(), 4);
        assert_eq!(handle.dispatch(()), Err(ReactiveError::Disposed));
        assert!(matches!(handle.snapshot(), Err(ReactiveError::Disposed)));
        drop(subscription);
        assert_eq!(dropped.get(), 4);
    }

    #[test]
    fn disposal_releases_observers_and_rejects_weak_handles() {
        let notifications = Rc::new(Cell::new(0));
        let observed = Rc::clone(&notifications);
        let mut graph = LocalGraph::new(
            0_i32,
            |state, message| {
                *state += message;
                Ok::<_, &'static str>(())
            },
            |_state| Ok(()),
        );
        let handle = graph.handle();
        let _subscription = graph
            .subscribe(move |_state| observed.set(observed.get() + 1))
            .unwrap();
        assert!(graph.dispose());
        assert!(!graph.dispose());
        assert_eq!(handle.dispatch(1), Err(ReactiveError::Disposed));
        assert_eq!(graph.snapshot(), Err(ReactiveError::Disposed));
        assert_eq!(handle.snapshot(), Err(ReactiveError::Disposed));
        assert_eq!(notifications.get(), 0);
    }
}
