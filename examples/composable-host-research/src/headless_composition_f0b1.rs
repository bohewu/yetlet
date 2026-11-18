//! F0-B1: private caller-local composition around the unchanged C1 reducer.
//! Input and observation are concrete capabilities, not generic framework Hosts.
//! No worker, socket, HTTP, renderer, executor, or shared Host trait is created.
use crate::simulation::{self, HeadlessRunner, Input, MAX_COMMANDS_PER_TICK, Snapshot};
use std::{collections::VecDeque, rc::Rc};

pub const MAX_QUEUED_FRAMES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Prepared,
    Running,
    Failed,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupError {
    InvalidCapacity,
    Domain(simulation::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    RootNotPrepared,
    ObserverNotPrepared,
    Domain(simulation::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotReady,
    NoFrame,
    Full,
    TooManyInputs,
    Domain(simulation::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cleanup {
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    NotApplicable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopReport {
    pub phase: Phase,
    pub primary_failure: Option<Failure>,
    pub cleanup: Cleanup,
    pub join: Join,
    pub discarded_frames: usize,
}

/// A finite, caller-local source. Frame payloads are validated by the
/// unchanged C1 reducer, never by an alternate composition reducer.
struct InputSource {
    phase: Phase,
    frames: VecDeque<Vec<Input>>,
    capacity: usize,
    discarded_frames: usize,
}

impl InputSource {
    fn new(capacity: usize) -> Result<Self, SetupError> {
        if !(1..=MAX_QUEUED_FRAMES).contains(&capacity) {
            return Err(SetupError::InvalidCapacity);
        }
        Ok(Self {
            phase: Phase::Prepared,
            frames: VecDeque::with_capacity(capacity),
            capacity,
            discarded_frames: 0,
        })
    }

    fn start(&mut self) {
        debug_assert_eq!(self.phase, Phase::Prepared);
        self.phase = Phase::Running;
    }

    fn enqueue(&mut self, frame: Vec<Input>) -> Result<(), Error> {
        if self.phase != Phase::Running {
            return Err(Error::NotReady);
        }
        if frame.len() > MAX_COMMANDS_PER_TICK {
            return Err(Error::TooManyInputs);
        }
        if self.frames.len() >= self.capacity {
            return Err(Error::Full);
        }
        self.frames.push_back(frame);
        Ok(())
    }

    fn take(&mut self) -> Result<Vec<Input>, Error> {
        if self.phase != Phase::Running {
            return Err(Error::NotReady);
        }
        self.frames.pop_front().ok_or(Error::NoFrame)
    }

    fn stop(&mut self) {
        if self.phase == Phase::Stopped {
            return;
        }
        self.phase = Phase::Stopped;
        self.discarded_frames += self.frames.len();
        self.frames.clear();
    }
}

/// A concrete, read-only snapshot attachment. It never owns Simulation.
/// A stopped observer can still return its last immutable Rc publication.
pub struct SnapshotObserver {
    phase: Phase,
    latest: Option<Rc<Snapshot>>,
}

impl Default for SnapshotObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotObserver {
    pub fn new() -> Self {
        Self {
            phase: Phase::Prepared,
            latest: None,
        }
    }

    fn attach(&mut self, snapshot: Rc<Snapshot>) -> Result<(), Failure> {
        if self.phase != Phase::Prepared {
            return Err(Failure::ObserverNotPrepared);
        }
        self.latest = Some(snapshot);
        self.phase = Phase::Running;
        Ok(())
    }

    fn publish(&mut self, snapshot: Rc<Snapshot>) {
        debug_assert_eq!(self.phase, Phase::Running);
        self.latest = Some(snapshot);
    }

    pub fn latest(&self) -> Option<Rc<Snapshot>> {
        self.latest.clone()
    }

    pub fn stop(&mut self) {
        self.phase = Phase::Stopped;
    }
}

/// Application-owned local root: one authority, one finite input source and
/// one observer. Worker join is *not applicable*, not a simulated success.
pub struct HeadlessComposition {
    owner: HeadlessRunner,
    source: InputSource,
    observer: SnapshotObserver,
    phase: Phase,
    primary_failure: Option<Failure>,
}

impl HeadlessComposition {
    pub fn new(generation: u64, queued_frames: usize) -> Result<Self, SetupError> {
        Self::with_observer(generation, queued_frames, SnapshotObserver::new())
    }

    /// Explicit observer injection permits a real precondition failure
    /// (already-stopped observer) without spawning an artificial worker.
    pub fn with_observer(
        generation: u64,
        queued_frames: usize,
        observer: SnapshotObserver,
    ) -> Result<Self, SetupError> {
        let source = InputSource::new(queued_frames)?;
        let owner = HeadlessRunner::new(generation).map_err(SetupError::Domain)?;
        Ok(Self {
            owner,
            source,
            observer,
            phase: Phase::Prepared,
            primary_failure: None,
        })
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn ready(&self) -> bool {
        self.phase == Phase::Running
            && self.source.phase == Phase::Running
            && self.observer.phase == Phase::Running
    }

    /// Start in dependency order. A required second-attachment failure
    /// stops the first attachment before reporting the original failure.
    pub fn start(&mut self) -> Result<(), Failure> {
        if self.phase != Phase::Prepared {
            return Err(Failure::RootNotPrepared);
        }
        self.source.start();
        if let Err(reason) = self.observer.attach(self.owner.snapshot()) {
            self.primary_failure = Some(reason);
            self.source.stop();
            self.observer.stop();
            self.phase = Phase::Failed;
            return Err(reason);
        }
        self.phase = Phase::Running;
        Ok(())
    }

    /// Enqueue one *complete* logical tick. An empty frame is an explicit
    /// tick; a completely empty queue never advances the simulation.
    pub fn submit_frame(&mut self, frame: Vec<Input>) -> Result<(), Error> {
        if !self.ready() {
            return Err(Error::NotReady);
        }
        self.source.enqueue(frame)
    }

    pub fn drive_one(&mut self) -> Result<Rc<Snapshot>, Error> {
        if !self.ready() {
            return Err(Error::NotReady);
        }
        let frame = self.source.take()?;
        match self.owner.tick(&frame) {
            Ok(next) => {
                self.observer.publish(Rc::clone(&next));
                Ok(next)
            }
            Err(cause) => {
                self.primary_failure.get_or_insert(Failure::Domain(cause));
                self.phase = Phase::Failed;
                self.stop();
                Err(Error::Domain(cause))
            }
        }
    }

    /// Latest read-only publication, retained even after stop. No mutable
    /// domain state or observer-owned stepping capability escapes here.
    pub fn latest(&self) -> Option<Rc<Snapshot>> {
        self.observer.latest()
    }

    pub fn stop(&mut self) -> StopReport {
        self.source.stop();
        self.observer.stop();
        if self.phase != Phase::Failed {
            self.phase = Phase::Stopped;
        }
        StopReport {
            phase: self.phase,
            primary_failure: self.primary_failure,
            cleanup: Cleanup::Complete,
            join: Join::NotApplicable,
            discarded_frames: self.source.discarded_frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{Command, Direction, Player, Position};

    fn input(tick: u64, sequence: u64, command: Command) -> Input {
        Input {
            generation: 7,
            tick,
            sequence,
            command,
        }
    }

    #[test]
    fn f0b1_one_authority_two_capabilities_and_immutable_publication() {
        let mut root = HeadlessComposition::new(7, 2).unwrap();
        assert_eq!(root.phase(), Phase::Prepared);
        assert!(!root.ready());
        assert_eq!(root.latest(), None);
        assert_eq!(root.submit_frame(vec![]), Err(Error::NotReady));
        root.start().unwrap();
        assert!(root.ready());
        assert_eq!(root.start(), Err(Failure::RootNotPrepared));
        assert!(root.ready());
        let initial = root.latest().unwrap();
        assert_eq!(initial.tick(), 0);

        root.submit_frame(vec![
            input(1, 2, Command::Hold(Player::Two, Direction::North)),
            input(1, 1, Command::Hold(Player::One, Direction::East)),
        ])
        .unwrap();
        root.submit_frame(vec![]).unwrap();
        let first = root.drive_one().unwrap();
        assert!(Rc::ptr_eq(&first, &root.latest().unwrap()));
        assert_eq!(
            first.positions(),
            [Position { x: 3, y: 2 }, Position { x: 20, y: 11 }]
        );
        assert_eq!(
            (first.tick(), first.revision(), first.last_sequence()),
            (1, 1, 2)
        );
        let second = root.drive_one().unwrap();
        assert_eq!((second.tick(), second.revision()), (2, 2));
        assert_eq!(
            second.positions(),
            [Position { x: 4, y: 2 }, Position { x: 20, y: 10 }]
        );
        assert_eq!(initial.tick(), 0);
        assert_eq!(first.positions()[0], Position { x: 3, y: 2 });
        assert!(!Rc::ptr_eq(&first, &second));
        assert_eq!(root.drive_one(), Err(Error::NoFrame));
        assert!(root.ready());
        assert_eq!(root.stop().join, Join::NotApplicable);
        assert_eq!(root.latest().unwrap().tick(), 2);
    }

    #[test]
    fn f0b1_required_observer_setup_failure_unwinds_earlier_source() {
        let mut unavailable = SnapshotObserver::new();
        unavailable.stop();
        let mut root = HeadlessComposition::with_observer(7, 2, unavailable).unwrap();
        assert_eq!(root.start(), Err(Failure::ObserverNotPrepared));
        assert_eq!(root.phase(), Phase::Failed);
        assert!(!root.ready());
        assert_eq!(root.source.phase, Phase::Stopped);
        assert_eq!(root.observer.phase, Phase::Stopped);
        assert_eq!(root.submit_frame(vec![]), Err(Error::NotReady));
        assert_eq!(
            root.stop(),
            StopReport {
                phase: Phase::Failed,
                primary_failure: Some(Failure::ObserverNotPrepared),
                cleanup: Cleanup::Complete,
                join: Join::NotApplicable,
                discarded_frames: 0,
            }
        );
    }

    #[test]
    fn f0b1_bounded_source_stop_discards_pending_and_rejects_late_frames() {
        assert!(matches!(
            HeadlessComposition::new(7, 0),
            Err(SetupError::InvalidCapacity)
        ));
        assert!(matches!(
            HeadlessComposition::new(7, MAX_QUEUED_FRAMES + 1),
            Err(SetupError::InvalidCapacity)
        ));
        assert!(matches!(
            HeadlessComposition::new(0, 1),
            Err(SetupError::Domain(_))
        ));
        let mut root = HeadlessComposition::new(7, 1).unwrap();
        root.start().unwrap();
        assert_eq!(
            root.submit_frame(vec![input(1, 1, Command::Pause); MAX_COMMANDS_PER_TICK + 1]),
            Err(Error::TooManyInputs)
        );
        root.submit_frame(vec![]).unwrap();
        assert_eq!(root.submit_frame(vec![]), Err(Error::Full));
        let first = root.stop();
        assert_eq!(
            first,
            StopReport {
                phase: Phase::Stopped,
                primary_failure: None,
                cleanup: Cleanup::Complete,
                join: Join::NotApplicable,
                discarded_frames: 1,
            }
        );
        assert_eq!(root.stop(), first);
        assert_eq!(root.submit_frame(vec![]), Err(Error::NotReady));
        assert_eq!(root.drive_one(), Err(Error::NotReady));
        assert_eq!(root.latest().unwrap().tick(), 0);
    }

    #[test]
    fn f0b1_domain_rejection_preserves_publication_and_stops_input() {
        let mut root = HeadlessComposition::new(7, 2).unwrap();
        root.start().unwrap();
        let before = root.latest().unwrap();
        root.submit_frame(vec![input(2, 1, Command::Pause)])
            .unwrap();
        root.submit_frame(vec![]).unwrap();
        assert_eq!(
            root.drive_one(),
            Err(Error::Domain(simulation::Error::WrongTick))
        );
        assert_eq!(root.phase(), Phase::Failed);
        assert_eq!(root.source.discarded_frames, 1);
        let last = root.latest().unwrap();
        assert!(Rc::ptr_eq(&before, &last));
        assert_eq!(root.submit_frame(vec![]), Err(Error::NotReady));
        assert_eq!(
            root.stop(),
            StopReport {
                phase: Phase::Failed,
                primary_failure: Some(Failure::Domain(simulation::Error::WrongTick)),
                cleanup: Cleanup::Complete,
                join: Join::NotApplicable,
                discarded_frames: 1,
            }
        );
    }

    #[test]
    fn f0b1_stop_before_start_closes_both_attachments_without_join() {
        let mut root = HeadlessComposition::new(7, 1).unwrap();
        assert_eq!(root.stop().phase, Phase::Stopped);
        assert_eq!(root.start(), Err(Failure::RootNotPrepared));
        assert_eq!(root.stop().join, Join::NotApplicable);
        assert!(!root.ready());
    }
}
