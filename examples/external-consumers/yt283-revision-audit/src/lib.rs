//! Independent source consumer; the audit is an application Host, not an SDK trait.
use std::cell::{Cell, RefCell};
#[cfg(any(feature = "three-way", test))]
use std::io;
#[cfg(feature = "three-way")]
use std::{cell::RefMut, sync::Arc};

pub const ARENA_WIDTH: i32 = 24;
pub const ARENA_HEIGHT: i32 = 16;
pub const SQUARE_SIZE: i32 = 2;

/// Recent Tick proofs retained by the original audit and publication history.
/// Caller-held proof handles can outlive this window without preventing eviction.
#[cfg(feature = "three-way")]
pub const TICK_EVIDENCE_CAPACITY: usize = 16;

/// Consumer-owned logical Tick/Input reducer for bounded YT-283-M1.
/// It uses the existing C1Domain as the sole authoritative position owner.
pub mod tick_m1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub generation: u64,
    pub revision: u64,
    pub paused: bool,
    pub positions: [Position; 2],
}
impl Snapshot {
    pub fn text(self) -> String {
        let [one, two] = self.positions;
        format!(
            "{} {} {} {} {} {} {}",
            self.generation, self.revision, self.paused, one.x, one.y, two.x, two.y
        )
    }

    // Pure transition shared by C1 preflight and the audit's exact comparison.
    #[cfg(any(feature = "three-way", test))]
    fn next(mut self, action: Action) -> io::Result<Self> {
        match action {
            Action::Pause => self.paused = true,
            Action::Resume => self.paused = false,
            Action::Move { .. } if self.paused => return Err(invalid("movement paused")),
            Action::Move { player, direction } => {
                let position = &mut self.positions[match player {
                    Player::One => 0,
                    Player::Two => 1,
                }];
                let (dx, dy) = match direction {
                    Direction::North => (0, -1),
                    Direction::East => (1, 0),
                    Direction::South => (0, 1),
                    Direction::West => (-1, 0),
                };
                position.x = (position.x + dx).clamp(0, ARENA_WIDTH - SQUARE_SIZE);
                position.y = (position.y + dy).clamp(0, ARENA_HEIGHT - SQUARE_SIZE);
            }
        }
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("revision exhausted"))?;
        Ok(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Pause,
    Resume,
    Move {
        player: Player,
        direction: Direction,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub generation: u64,
    pub expected_revision: u64,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditRecord {
    pub applied: Snapshot,
    pub action: Action,
}
/// Exact immutable Tick evidence shared by the original bridge and audit ledger.
#[cfg(feature = "three-way")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickAuditRecord {
    pub before: tick_m1::TickObservation,
    pub applied: tick_m1::TickObservation,
    /// Validated inputs in canonical sequence order, including paused inputs.
    pub inputs: Vec<tick_m1::TickInput>,
}

#[cfg(not(feature = "three-way"))]
type AuditEntry = AuditRecord;

#[cfg(feature = "three-way")]
enum AuditEntry {
    Ra1(AuditRecord),
    Tick(Arc<TickAuditRecord>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditStatus {
    Open,
    Closed,
}

/// Bounded caller-local audit ledger: RA1 accepts at most 16 entries; Tick mode
/// retains the latest TICK_EVIDENCE_CAPACITY proofs in chronological order.
/// Every accepted entry is the exact next state of its pinned authority.
/// The latest identity and retained evidence stay available after close.
pub struct RevisionAuditHost {
    pinned: Cell<Snapshot>,
    records: RefCell<Vec<AuditEntry>>,
    tick_pinned: Cell<Option<tick_m1::TickClock>>,
    status: Cell<AuditStatus>,
}
#[cfg(any(feature = "three-way", test))]
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
impl RevisionAuditHost {
    pub fn new(initial: Snapshot) -> Self {
        Self {
            pinned: Cell::new(initial),
            records: RefCell::new(Vec::new()),
            tick_pinned: Cell::new(None),
            status: Cell::new(AuditStatus::Open),
        }
    }
    pub fn status(&self) -> AuditStatus {
        self.status.get()
    }
    pub fn pinned(&self) -> Snapshot {
        self.pinned.get()
    }
    /// Original RA1 projection of the single finite ledger.
    pub fn records(&self) -> Vec<AuditRecord> {
        #[cfg(not(feature = "three-way"))]
        {
            self.records.borrow().clone()
        }
        #[cfg(feature = "three-way")]
        {
            self.records
                .borrow()
                .iter()
                .filter_map(|entry| match entry {
                    AuditEntry::Ra1(record) => Some(*record),
                    AuditEntry::Tick(_) => None,
                })
                .collect()
        }
    }
    /// Latest accepted Tick identity, independent of how many older proofs remain.
    /// None before the first accepted Tick or in RA1 mode; unchanged by close.
    pub fn pinned_tick(&self) -> Option<tick_m1::TickObservation> {
        self.tick_pinned
            .get()
            .map(|clock| tick_m1::TickObservation {
                snapshot: self.pinned.get(),
                clock,
            })
    }
    /// Recent accepted proofs, oldest first, bounded by TICK_EVIDENCE_CAPACITY.
    /// The length is not a lifetime Tick count. Returned immutable handles remain
    /// valid after eviction but do not keep their revision in either history.
    #[cfg(feature = "three-way")]
    pub fn tick_records(&self) -> Vec<Arc<TickAuditRecord>> {
        self.records
            .borrow()
            .iter()
            .filter_map(|entry| match entry {
                AuditEntry::Ra1(_) => None,
                AuditEntry::Tick(record) => Some(Arc::clone(record)),
            })
            .collect()
    }
    #[cfg(any(feature = "three-way", test))]
    fn preflight(&self, before: Snapshot, applied: Snapshot, action: Action) -> io::Result<()> {
        if self.status.get() != AuditStatus::Open || self.tick_pinned.get().is_some() {
            return Err(invalid("audit closed or already in tick mode"));
        }
        if before != self.pinned.get() || before.next(action)? != applied {
            return Err(invalid("audit identity/order/state mismatch"));
        }
        if self.records.borrow().len() == 16 {
            return Err(invalid("finite audit ledger full"));
        }
        Ok(())
    }
    #[cfg(any(feature = "three-way", test))]
    fn append(&self, applied: Snapshot, action: Action) {
        let record = AuditRecord { applied, action };
        #[cfg(feature = "three-way")]
        let record = AuditEntry::Ra1(record);
        self.records.borrow_mut().push(record);
        self.pinned.set(applied);
        eprintln!(
            "AUDIT_APPLIED action={action:?} snapshot={}",
            applied.text()
        );
    }
    /// Validate an already applied publication against its exact pinned readback.
    /// Invalid input never changes the ledger or its last accepted identity.
    #[cfg(test)]
    fn audit_applied(
        &self,
        before: Snapshot,
        applied: Snapshot,
        action: Action,
        readback: Snapshot,
    ) -> io::Result<()> {
        self.preflight(before, applied, action)?;
        if readback != applied {
            return Err(invalid("audit pinned readback mismatch"));
        }
        self.append(applied, action);
        Ok(())
    }
    #[cfg(any(feature = "three-way", test))]
    pub(crate) fn close(&self) -> io::Result<()> {
        self.status.set(AuditStatus::Closed);
        eprintln!(
            "AUDIT_CLOSED entries={} pinned={}",
            self.records.borrow().len(),
            self.pinned.get().text()
        );
        Ok(())
    }
}

/// The only mutable domain authority. Cell/Rc usage deliberately precludes Send.
/// Live direct callers may inspect snapshots. A bound authority accepts
/// mutations only through its owning Ports' RA1 or atomic Tick publication.
///
/// ```compile_fail
/// use yt283_revision_audit::{Action, C1Domain, Command, RevisionAuditHost};
/// let domain = C1Domain::new(9);
/// let audit = RevisionAuditHost::new(domain.snapshot());
/// let proposed = domain.preflight(Command {
///     generation: 9, expected_revision: 0, action: Action::Pause,
/// }, &audit).unwrap();
/// domain.commit_published(proposed, &audit, domain.snapshot()).unwrap();
/// ```
pub struct C1Domain {
    state: Cell<Snapshot>,
    stopped: Cell<bool>,
    // Input/time metadata only; the Snapshot/positions have one owner in C1.
    tick_clock: Cell<tick_m1::TickClock>,
    // Only the owning Ports may publish/commit a bridge-bound authority.
    ports_bound: Cell<bool>,
}
/// Validated proposal, not an applied state or publication readback.
#[cfg(any(feature = "three-way", test))]
pub(crate) struct PreparedApply {
    before: Snapshot,
    candidate: Snapshot,
    action: Action,
}
#[cfg(any(feature = "three-way", test))]
impl PreparedApply {
    pub(crate) fn candidate(&self) -> Snapshot {
        self.candidate
    }
}
impl C1Domain {
    pub fn new(generation: u64) -> Self {
        Self {
            state: Cell::new(Snapshot {
                generation,
                revision: 0,
                paused: false,
                positions: [Position { x: 2, y: 2 }, Position { x: 20, y: 12 }],
            }),
            stopped: Cell::new(false),
            tick_clock: Cell::new(tick_m1::TickClock::default()),
            ports_bound: Cell::new(false),
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.get()
    }
    pub fn is_stopped(&self) -> bool {
        self.stopped.get()
    }
    /// Seal this exact C1 for its original CommandBridge/Audit owner.
    /// Direct local mutation and close must not bypass that owning Ports.
    #[cfg(feature = "three-way")]
    pub(crate) fn bind_ports(&self) -> io::Result<()> {
        if self.ports_bound.get()
            || self.stopped.get()
            || self.tick_clock.get().tick != 0
            || self.snapshot().revision != 0
        {
            return Err(invalid("C1 cannot be rebound to Ports"));
        }
        self.ports_bound.set(true);
        Ok(())
    }

    /// Copied Tick view of this same authoritative C1 Snapshot.
    /// Bound consumers obtain immutable publication proof from Ports::pinned_tick.
    pub fn tick_observation(&self) -> tick_m1::TickObservation {
        tick_m1::TickObservation {
            snapshot: self.state.get(),
            clock: self.tick_clock.get(),
        }
    }

    /// One explicit logical 1/60-second step, never a wall-clock scheduler.
    /// Atomic prevalidation/reduction and copy-only commit preserve the previous
    /// C1 state and all input counters on rejection. RA1/audit mode cannot
    /// execute concurrently with this intentionally local-only first slice.
    pub fn step_tick(
        &self,
        tick: u64,
        inputs: &[tick_m1::TickInput],
    ) -> Result<tick_m1::TickObservation, tick_m1::TickError> {
        if self.stopped.get() {
            return Err(tick_m1::TickError::Stopped);
        }
        let clock = self.tick_clock.get();
        let current = self.state.get();
        if self.ports_bound.get() || clock.tick != current.revision {
            return Err(tick_m1::TickError::ModeConflict);
        }
        let next = tick_m1::reduce(current, clock, tick, inputs)?;
        // Caller-local Rc/Cell: no callback, IO or fallible operation between
        // these two writes, and no second Simulation/position owner exists.
        self.state.set(next.snapshot);
        self.tick_clock.set(next.clock);
        Ok(next)
    }

    /// Honest Composition close callback for local tick-only consumers.
    /// A bound RA1 or Tick path must instead close its Audit/Ports in order.
    pub fn close_tick_m1(&self) -> Result<(), tick_m1::TickError> {
        let clock = self.tick_clock.get();
        if self.ports_bound.get() || clock.tick != self.state.get().revision {
            return Err(tick_m1::TickError::ModeConflict);
        }
        self.stopped.set(true);
        Ok(())
    }

    /// All validation and audit storage reservation precede bridge publication.
    /// The private prepared value retains the exact local ledger borrow until
    /// it is dropped on rejection or consumed by the infallible commit.
    #[cfg(feature = "three-way")]
    pub(crate) fn prepare_tick<'a>(
        &'a self,
        audit: &'a RevisionAuditHost,
        tick: u64,
        inputs: &[tick_m1::TickInput],
    ) -> io::Result<PreparedTick<'a>> {
        let before = self.tick_observation();
        if !self.ports_bound.get()
            || self.is_stopped()
            || before.clock.tick != before.snapshot.revision
        {
            return Err(invalid("tick authority is unbound, stopped or in RA1 mode"));
        }
        let applied = tick_m1::reduce(before.snapshot, before.clock, tick, inputs)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let mut entries = audit
            .records
            .try_borrow_mut()
            .map_err(|_| invalid("audit ledger is borrowed"))?;
        if audit.status.get() != AuditStatus::Open
            || audit.pinned.get() != before.snapshot
            || audit.tick_pinned.get().unwrap_or_default() != before.clock
            || entries.len() > TICK_EVIDENCE_CAPACITY
        {
            return Err(invalid(
                "tick audit identity, lifecycle or capacity mismatch",
            ));
        }
        // At capacity, commit reuses the oldest slot only after publication.
        // Preparation must neither evict evidence nor reserve a seventeenth slot.
        if entries.len() < TICK_EVIDENCE_CAPACITY {
            entries.try_reserve(1).map_err(io::Error::other)?;
        }
        let mut inputs = inputs.to_vec();
        inputs.sort_unstable_by_key(|input| input.sequence);
        let record = Arc::new(TickAuditRecord {
            before,
            applied,
            inputs,
        });
        Ok(PreparedTick {
            c1: self,
            audit,
            entries,
            record,
        })
    }

    #[cfg(any(feature = "three-way", test))]
    pub(crate) fn preflight(
        &self,
        command: Command,
        audit: &RevisionAuditHost,
    ) -> io::Result<PreparedApply> {
        let before = self.snapshot();
        if self.tick_clock.get().tick != 0 {
            return Err(invalid("local tick mode cannot use RA1/audit mutation"));
        }
        if self.is_stopped()
            || command.generation != before.generation
            || command.expected_revision != before.revision
        {
            return Err(invalid("domain closed or stale command"));
        }
        let applied = before.next(command.action)?;
        audit.preflight(before, applied, command.action)?;
        Ok(PreparedApply {
            before,
            candidate: applied,
            action: command.action,
        })
    }
    /// The owner supplies an observed publication. Validate before either local
    /// mutation; no callback or fallible operation separates the two commits.
    #[cfg(any(feature = "three-way", test))]
    pub(crate) fn commit_published(
        &self,
        prepared: PreparedApply,
        audit: &RevisionAuditHost,
        readback: Snapshot,
    ) -> io::Result<Snapshot> {
        if self.is_stopped() || self.snapshot() != prepared.before {
            return Err(invalid("domain changed after preflight"));
        }
        audit.preflight(prepared.before, prepared.candidate, prepared.action)?;
        if readback != prepared.candidate {
            return Err(invalid("published readback differs from candidate"));
        }
        self.state.set(readback);
        audit.append(readback, prepared.action);
        Ok(readback)
    }
    #[cfg(any(feature = "three-way", test))]
    pub(crate) fn close(&self, audit: &RevisionAuditHost) -> io::Result<()> {
        let clock = self.tick_clock.get();
        if clock.tick != 0 && !self.ports_bound.get() {
            return Err(invalid("local tick-only C1 has no bound audit close"));
        }
        let expected_clock = (clock.tick != 0).then_some(clock);
        if audit.status() != AuditStatus::Closed
            || audit.pinned() != self.snapshot()
            || audit.tick_pinned.get() != expected_clock
        {
            return Err(invalid(
                "audit must close at exact domain and tick identity",
            ));
        }
        self.stopped.set(true);
        eprintln!("C1_CLOSED snapshot={}", self.snapshot().text());
        Ok(())
    }
}

/// Private commit authority; it cannot outlive its exact C1/audit or be forged
/// through the public consumer API.
#[cfg(feature = "three-way")]
pub(crate) struct PreparedTick<'a> {
    c1: &'a C1Domain,
    audit: &'a RevisionAuditHost,
    entries: RefMut<'a, Vec<AuditEntry>>,
    record: Arc<TickAuditRecord>,
}

#[cfg(feature = "three-way")]
impl PreparedTick<'_> {
    pub(crate) fn record(&self) -> Arc<TickAuditRecord> {
        Arc::clone(&self.record)
    }

    /// Called only after successful publication through the owning Ports.
    /// No allocation, readback, callback, logging or fallible work follows
    /// publication. The reserved or evicted Vec slot and held RefMut make push
    /// infallible; only this successful commit can evict the oldest Tick proof.
    pub(crate) fn commit(mut self) -> tick_m1::TickObservation {
        let applied = self.record.applied;
        if self.entries.len() == TICK_EVIDENCE_CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push(AuditEntry::Tick(self.record));
        self.audit.pinned.set(applied.snapshot);
        self.audit.tick_pinned.set(Some(applied.clock));
        self.c1.state.set(applied.snapshot);
        self.c1.tick_clock.set(applied.clock);
        applied
    }
}

#[cfg(feature = "three-way")]
pub mod native_clock;
#[cfg(feature = "three-way")]
pub mod native_input;
#[cfg(feature = "three-way")]
pub mod three_way;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{rc::Rc, time::Instant};
    use yetlet::composition::{Builder, Completion};
    // These local-only tests use a copied local publication. Real CommandBridge
    // publication and SnapshotReader readback are exercised by three_way tests.
    fn apply_local(c: &C1Domain, command: Command, a: &RevisionAuditHost) -> io::Result<Snapshot> {
        let prepared = c.preflight(command, a)?;
        let local_publication = prepared.candidate();
        c.commit_published(prepared, a, local_publication)
    }
    #[test]
    fn moves_clamp_each_axis_and_pause_preserves_revision_and_positions() {
        let c = C1Domain::new(12);
        let mut s = c.snapshot();
        s.positions = [Position { x: 0, y: 0 }, Position { x: 22, y: 14 }];
        c.state.set(s);
        let a = RevisionAuditHost::new(s);
        let mut apply = |action| -> io::Result<Snapshot> {
            let next = apply_local(
                &c,
                Command {
                    generation: 12,
                    expected_revision: s.revision,
                    action,
                },
                &a,
            )?;
            s = next;
            Ok(next)
        };
        for (player, direction) in [
            (Player::One, Direction::North),
            (Player::One, Direction::West),
            (Player::Two, Direction::South),
            (Player::Two, Direction::East),
        ] {
            assert_eq!(
                apply(Action::Move { player, direction }).unwrap().positions,
                [Position { x: 0, y: 0 }, Position { x: 22, y: 14 }]
            );
        }
        let advance = Action::Move {
            player: Player::One,
            direction: Direction::East,
        };
        let paused = apply(Action::Pause).unwrap();
        assert_eq!(
            apply(advance).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(c.snapshot(), paused);
        assert_eq!(a.pinned(), paused);
        assert_eq!(a.records().len(), 5);
        assert_eq!(apply(Action::Resume).unwrap().positions, paused.positions);
        let moved = apply(advance).unwrap();
        assert_eq!(moved.revision, 7);
        assert_eq!(
            moved.positions,
            [Position { x: 1, y: 0 }, Position { x: 22, y: 14 }]
        );
        assert_eq!(a.pinned(), moved);
    }

    #[test]
    fn audit_rejects_wrong_player_position_and_paused_state() {
        let c = C1Domain::new(13);
        let initial = c.snapshot();
        let a = RevisionAuditHost::new(initial);
        let action = Action::Move {
            player: Player::Two,
            direction: Direction::North,
        };
        let expected = initial.next(action).unwrap();
        let mut bad = expected;
        bad.positions[0].y -= 1;
        assert!(a.audit_applied(initial, bad, action, bad).is_err());
        bad = expected;
        bad.positions[1].y += 1;
        assert!(a.audit_applied(initial, bad, action, bad).is_err());
        assert!(a.audit_applied(initial, expected, action, bad).is_err());
        bad = expected;
        bad.paused = true;
        assert!(a.audit_applied(initial, bad, action, bad).is_err());
        assert!(a.records().is_empty());
        assert_eq!(a.pinned(), initial);
        a.audit_applied(initial, expected, action, expected)
            .unwrap();
        assert_eq!(a.pinned(), expected);
    }

    #[test]
    fn local_rc_applies_audits_and_closes_in_dependency_order() {
        let c1 = Rc::new(C1Domain::new(7));
        let audit = Rc::new(RevisionAuditHost::new(c1.snapshot()));
        let closing = audit.clone();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", c1.clone(), move |c| c.close(&closing))
            .required_local("custom-audit", audit.clone(), |a| a.close())
            .build();
        assert!(root.all_required_ready());
        for action in [Action::Pause, Action::Resume] {
            let s = c1.snapshot();
            apply_local(
                &c1,
                Command {
                    generation: 7,
                    expected_revision: s.revision,
                    action,
                },
                &audit,
            )
            .unwrap();
        }
        assert_eq!(audit.records().len(), 2);
        assert_eq!(audit.pinned(), c1.snapshot());
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(c1.is_stopped());
        assert_eq!(audit.status(), AuditStatus::Closed);
    }
    #[test]
    fn invalid_publications_preserve_last_valid_entry_and_first_failure() {
        let c = C1Domain::new(8);
        let a = Rc::new(RevisionAuditHost::new(c.snapshot()));
        let before = c.snapshot();
        let next = apply_local(
            &c,
            Command {
                generation: 8,
                expected_revision: 0,
                action: Action::Pause,
            },
            &a,
        )
        .unwrap();
        let mut bad = next;
        bad.generation = 9;
        assert!(a.audit_applied(next, bad, Action::Pause, bad).is_err());
        assert!(a.audit_applied(before, next, Action::Pause, next).is_err());
        bad = next;
        bad.revision = 3;
        assert!(a.audit_applied(next, bad, Action::Pause, bad).is_err());
        bad = next;
        bad.revision = 2;
        assert!(a.audit_applied(next, bad, Action::Resume, bad).is_err());
        bad.paused = false;
        assert!(a.audit_applied(next, bad, Action::Resume, next).is_err());
        assert_eq!(a.records().len(), 1);
        assert_eq!(a.pinned(), next);
        let mut root = Builder::<io::Error>::new()
            .required_local("audit", a, |a| a.close())
            .build();
        root.fail_required(invalid("first invalid audit"));
        root.fail_required(invalid("later error"));
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert_eq!(
            root.failures().primary().unwrap().to_string(),
            "first invalid audit"
        );
        assert_eq!(root.failures().secondary().len(), 1);
    }
    #[test]
    fn pending_custom_close_retains_original_rc_and_domain() {
        let c = Rc::new(C1Domain::new(9));
        let a = Rc::new(RevisionAuditHost::new(c.snapshot()));
        let allow = Rc::new(Cell::new(false));
        let gate = allow.clone();
        let ac = a.clone();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", c.clone(), move |c| c.close(&ac))
            .required_local("audit", a.clone(), move |a| {
                if !gate.get() {
                    return Err(io::Error::other("synthetic audit storage busy"));
                }
                a.close()
            })
            .build();
        root.fail_required(invalid("deliberate primary"));
        assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
        assert!(Rc::ptr_eq(root.parts().last().value(), &a));
        assert!(!c.is_stopped());
        assert_eq!(a.status(), AuditStatus::Open);
        allow.set(true);
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(c.is_stopped());
        assert_eq!(
            root.failures().primary().unwrap().to_string(),
            "deliberate primary"
        );
    }
}
