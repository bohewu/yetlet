//! YT-266: private, std-only composition proof. No public Yetlet API is introduced.
//!
//! The caller drives two concrete Hosts on one thread. Only StateOwner can mutate
//! the authoritative counter; the Hosts receive bounded CommandPort and immutable
//! SnapshotPort values. No executor, transport, render backend or Host trait exists.

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    thread::{self, ThreadId},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Credit(u32),
    Debit(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub generation: u64,
    pub revision: u64,
    pub balance: i64,
}

struct Mailbox {
    generation: u64,
    accepting: bool,
    capacity: usize,
    pending: VecDeque<Command>,
}

/// A capability to request a change, never to mutate domain state directly.
/// Successful admission is *not* successful application.
#[derive(Clone)]
pub struct CommandPort {
    mailbox: Rc<RefCell<Mailbox>>,
    generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    Closed,
    Full,
    StaleGeneration,
}

impl CommandPort {
    pub fn submit(&self, command: Command) -> Result<(), AdmissionError> {
        let mut mailbox = self.mailbox.borrow_mut();
        if !mailbox.accepting {
            return Err(AdmissionError::Closed);
        }
        if self.generation != mailbox.generation {
            return Err(AdmissionError::StaleGeneration);
        }
        if mailbox.pending.len() >= mailbox.capacity {
            return Err(AdmissionError::Full);
        }
        mailbox.pending.push_back(command);
        Ok(())
    }
}

/// Publishes an owned, immutable value; the slot contains no reference to
/// StateOwner's mutable state. Readers must not retain a RefCell borrow.
#[derive(Clone)]
pub struct SnapshotPort {
    slot: Rc<RefCell<Rc<Snapshot>>>,
}

impl SnapshotPort {
    pub fn current(&self) -> Rc<Snapshot> {
        Rc::clone(&self.slot.borrow())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyError {
    Closed,
    ValueOverflow,
    RevisionOverflow,
}

/// The only mutable domain owner. Ports expose neither its state nor a mutable
/// reference to it. The host count does not decide the owner/thread count.
pub struct StateOwner {
    balance: i64,
    generation: u64,
    revision: u64,
    mailbox: Rc<RefCell<Mailbox>>,
    snapshots: Rc<RefCell<Rc<Snapshot>>>,
    caller_thread: ThreadId,
}

impl StateOwner {
    pub fn new(generation: u64, capacity: usize) -> Self {
        assert!(capacity > 0, "B1 requires a nonempty bounded queue");
        let initial = Rc::new(Snapshot {
            generation,
            revision: 0,
            balance: 0,
        });
        Self {
            balance: 0,
            generation,
            revision: 0,
            mailbox: Rc::new(RefCell::new(Mailbox {
                generation,
                accepting: false,
                capacity,
                pending: VecDeque::with_capacity(capacity),
            })),
            snapshots: Rc::new(RefCell::new(initial)),
            caller_thread: thread::current().id(),
        }
    }

    pub fn ports(&self) -> (CommandPort, SnapshotPort) {
        (
            CommandPort {
                mailbox: Rc::clone(&self.mailbox),
                generation: self.generation,
            },
            SnapshotPort {
                slot: Rc::clone(&self.snapshots),
            },
        )
    }

    pub fn caller_thread(&self) -> ThreadId {
        self.caller_thread
    }

    /// The composition root calls this after both required Hosts start.
    pub fn open(&mut self) {
        self.mailbox.borrow_mut().accepting = true;
    }

    /// One bounded, atomic application batch. Overflow leaves the entire
    /// pending batch and last published Snapshot intact.
    pub fn apply_pending(&mut self) -> Result<usize, ApplyError> {
        let mut mailbox = self.mailbox.borrow_mut();
        if !mailbox.accepting {
            return Err(ApplyError::Closed);
        }
        if mailbox.pending.is_empty() {
            return Ok(0);
        }

        let mut next_balance = self.balance;
        for command in &mailbox.pending {
            next_balance = match command {
                Command::Credit(amount) => next_balance.checked_add(i64::from(*amount)),
                Command::Debit(amount) => next_balance.checked_sub(i64::from(*amount)),
            }
            .ok_or(ApplyError::ValueOverflow)?;
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(ApplyError::RevisionOverflow)?;
        let applied = mailbox.pending.len();
        mailbox.pending.clear();
        drop(mailbox);

        self.balance = next_balance;
        self.revision = next_revision;
        *self.snapshots.borrow_mut() = Rc::new(Snapshot {
            generation: self.generation,
            revision: next_revision,
            balance: next_balance,
        });
        Ok(applied)
    }

    /// Returns the number of admitted-but-unapplied commands discarded.
    /// A complete delivery/shutdown protocol belongs to YT-267, not B1.
    pub fn close(&mut self) -> usize {
        let mut mailbox = self.mailbox.borrow_mut();
        mailbox.accepting = false;
        let discarded = mailbox.pending.len();
        mailbox.pending.clear();
        discarded
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Prepared,
    Running,
    Stopped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostError {
    NotPrepared,
    NotRunning,
    Admission(AdmissionError),
}

struct LocalLifecycle {
    phase: Phase,
    caller_thread: ThreadId,
}

impl LocalLifecycle {
    fn new() -> Self {
        Self {
            phase: Phase::Prepared,
            caller_thread: thread::current().id(),
        }
    }

    fn start(&mut self) -> Result<(), HostError> {
        if self.phase != Phase::Prepared {
            return Err(HostError::NotPrepared);
        }
        self.phase = Phase::Running;
        Ok(())
    }

    fn running(&self) -> Result<(), HostError> {
        if self.phase != Phase::Running {
            return Err(HostError::NotRunning);
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), HostError> {
        self.running()?;
        self.phase = Phase::Stopped;
        Ok(())
    }
}

/// Concrete, caller-driven panel input. Its Rc/RefCell view model is local
/// and intentionally !Send; it does not own the shared domain state.
pub struct PanelHost {
    life: LocalLifecycle,
    commands: CommandPort,
    snapshots: SnapshotPort,
    local_last_credit: Rc<RefCell<Option<u32>>>,
}

impl PanelHost {
    pub fn new(commands: CommandPort, snapshots: SnapshotPort) -> Self {
        Self {
            life: LocalLifecycle::new(),
            commands,
            snapshots,
            local_last_credit: Rc::new(RefCell::new(None)),
        }
    }

    pub fn start(&mut self) -> Result<(), HostError> {
        self.life.start()
    }

    pub fn credit(&self, amount: u32) -> Result<(), HostError> {
        self.life.running()?;
        self.commands
            .submit(Command::Credit(amount))
            .map_err(HostError::Admission)?;
        *self.local_last_credit.borrow_mut() = Some(amount);
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Rc<Snapshot>, HostError> {
        self.life.running()?;
        Ok(self.snapshots.current())
    }

    pub fn last_credit_model(&self) -> Rc<RefCell<Option<u32>>> {
        Rc::clone(&self.local_last_credit)
    }

    pub fn phase(&self) -> Phase {
        self.life.phase
    }

    pub fn caller_thread(&self) -> ThreadId {
        self.life.caller_thread
    }

    pub fn stop(&mut self) -> Result<(), HostError> {
        self.life.stop()
    }
}

/// Another concrete, *cooperatively driven* Host. It does not create a timer,
/// process, task, dedicated thread or new scheduler.
pub struct RuleHost {
    life: LocalLifecycle,
    commands: CommandPort,
    snapshots: SnapshotPort,
    local_accepted: Rc<Cell<u32>>,
}

impl RuleHost {
    pub fn new(commands: CommandPort, snapshots: SnapshotPort) -> Self {
        Self {
            life: LocalLifecycle::new(),
            commands,
            snapshots,
            local_accepted: Rc::new(Cell::new(0)),
        }
    }

    pub fn start(&mut self) -> Result<(), HostError> {
        self.life.start()
    }

    pub fn debit(&self, amount: u32) -> Result<(), HostError> {
        self.life.running()?;
        self.commands
            .submit(Command::Debit(amount))
            .map_err(HostError::Admission)?;
        self.local_accepted
            .set(self.local_accepted.get().saturating_add(1));
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Rc<Snapshot>, HostError> {
        self.life.running()?;
        Ok(self.snapshots.current())
    }

    pub fn accepted_model(&self) -> Rc<Cell<u32>> {
        Rc::clone(&self.local_accepted)
    }

    pub fn phase(&self) -> Phase {
        self.life.phase
    }

    pub fn caller_thread(&self) -> ThreadId {
        self.life.caller_thread
    }

    pub fn stop(&mut self) -> Result<(), HostError> {
        self.life.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(capacity: usize) -> (StateOwner, PanelHost, RuleHost) {
        let owner = StateOwner::new(42, capacity);
        let (input, output) = owner.ports();
        let panel = PanelHost::new(input.clone(), output.clone());
        let rule = RuleHost::new(input, output);
        (owner, panel, rule)
    }

    fn start(owner: &mut StateOwner, panel: &mut PanelHost, rule: &mut RuleHost) {
        panel.start().unwrap();
        rule.start().unwrap();
        owner.open();
    }

    #[test]
    fn two_concrete_hosts_share_one_owner_and_exact_snapshot() {
        let (mut owner, mut panel, mut rule) = setup(2);
        start(&mut owner, &mut panel, &mut rule);
        panel.credit(8).unwrap();
        rule.debit(3).unwrap();
        assert_eq!(owner.apply_pending(), Ok(2));
        let left = panel.snapshot().unwrap();
        let right = rule.snapshot().unwrap();
        assert!(Rc::ptr_eq(&left, &right));
        assert_eq!(
            (*left).clone(),
            Snapshot {
                generation: 42,
                revision: 1,
                balance: 5
            }
        );
        panel.stop().unwrap();
        rule.stop().unwrap();
        assert_eq!(owner.close(), 0);
    }

    #[test]
    fn separate_host_lifecycles_do_not_stop_the_other() {
        let (mut owner, mut panel, mut rule) = setup(2);
        assert_eq!(panel.phase(), Phase::Prepared);
        assert_eq!(rule.phase(), Phase::Prepared);
        start(&mut owner, &mut panel, &mut rule);
        panel.stop().unwrap();
        assert_eq!(panel.credit(10), Err(HostError::NotRunning));
        assert_eq!(rule.phase(), Phase::Running);
        rule.debit(4).unwrap();
        owner.apply_pending().unwrap();
        assert_eq!(rule.snapshot().unwrap().balance, -4);
        rule.stop().unwrap();
        assert_eq!(panel.phase(), Phase::Stopped);
        assert_eq!(rule.phase(), Phase::Stopped);
        assert_eq!(owner.close(), 0);
    }

    #[test]
    fn queue_is_bounded_and_submission_is_not_execution() {
        let (mut owner, mut panel, mut rule) = setup(1);
        start(&mut owner, &mut panel, &mut rule);
        let original = panel.snapshot().unwrap();
        panel.credit(7).unwrap();
        assert_eq!(
            rule.debit(3),
            Err(HostError::Admission(AdmissionError::Full))
        );
        assert_eq!(panel.snapshot().unwrap().revision, 0);
        assert!(Rc::ptr_eq(&original, &rule.snapshot().unwrap()));
        assert_eq!(owner.apply_pending(), Ok(1));
        assert_eq!(rule.snapshot().unwrap().balance, 7);
        rule.debit(3).unwrap();
        assert_eq!(owner.apply_pending(), Ok(1));
        assert_eq!(rule.snapshot().unwrap().balance, 4);
        panel.stop().unwrap();
        rule.stop().unwrap();
        assert_eq!(owner.close(), 0);
    }

    #[test]
    fn snapshots_are_owned_immutable_versions() {
        let (mut owner, mut panel, mut rule) = setup(2);
        start(&mut owner, &mut panel, &mut rule);
        let old = panel.snapshot().unwrap();
        panel.credit(2).unwrap();
        owner.apply_pending().unwrap();
        let fresh = rule.snapshot().unwrap();
        assert_eq!(old.revision, 0);
        assert_eq!(old.balance, 0);
        assert_eq!(fresh.revision, 1);
        assert_eq!(fresh.balance, 2);
        assert!(!Rc::ptr_eq(&old, &fresh));
        assert_eq!(owner.apply_pending(), Ok(0));
        assert!(Rc::ptr_eq(&fresh, &panel.snapshot().unwrap()));
        panel.stop().unwrap();
        rule.stop().unwrap();
        assert_eq!(owner.close(), 0);
    }

    #[test]
    fn host_local_rc_models_and_owner_share_caller_thread() {
        let (mut owner, mut panel, mut rule) = setup(2);
        let current = thread::current().id();
        assert_eq!(current, owner.caller_thread());
        assert_eq!(current, panel.caller_thread());
        assert_eq!(current, rule.caller_thread());
        let panel_local: Rc<RefCell<Option<u32>>> = panel.last_credit_model();
        let rule_local: Rc<Cell<u32>> = rule.accepted_model();
        start(&mut owner, &mut panel, &mut rule);
        panel.credit(10).unwrap();
        rule.debit(4).unwrap();
        assert_eq!(*panel_local.borrow(), Some(10));
        assert_eq!(rule_local.get(), 1);
        owner.apply_pending().unwrap();
        assert_eq!(panel.snapshot().unwrap().balance, 6);
        panel.stop().unwrap();
        rule.stop().unwrap();
        assert_eq!(owner.close(), 0);
    }

    #[test]
    fn unopened_and_closed_ingress_never_mutate_state() {
        let (mut owner, mut panel, mut rule) = setup(2);
        panel.start().unwrap();
        assert_eq!(
            panel.credit(3),
            Err(HostError::Admission(AdmissionError::Closed))
        );
        assert_eq!(owner.apply_pending(), Err(ApplyError::Closed));
        rule.start().unwrap();
        owner.open();
        panel.credit(3).unwrap();
        assert_eq!(owner.close(), 1); // admission does not promise completion
        assert_eq!(
            panel.credit(2),
            Err(HostError::Admission(AdmissionError::Closed))
        );
        assert_eq!(owner.apply_pending(), Err(ApplyError::Closed));
        assert_eq!(panel.snapshot().unwrap().balance, 0);
        panel.stop().unwrap();
        rule.stop().unwrap();
    }

    #[test]
    fn invalid_lifecycle_transitions_are_explicit() {
        let (mut owner, mut panel, mut rule) = setup(2);
        assert_eq!(panel.stop(), Err(HostError::NotRunning));
        panel.start().unwrap();
        assert_eq!(panel.start(), Err(HostError::NotPrepared));
        rule.start().unwrap();
        owner.open();
        panel.stop().unwrap();
        assert_eq!(panel.start(), Err(HostError::NotPrepared));
        assert_eq!(panel.snapshot(), Err(HostError::NotRunning));
        rule.stop().unwrap();
        assert_eq!(owner.close(), 0);
    }
}
