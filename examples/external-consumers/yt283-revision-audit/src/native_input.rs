//! Consumer-owned Native key preparation for explicit logical Tick publication.
//!
//! This module owns physical key bits and at most32 pending commands, not a
//! Simulation, Snapshot, scheduler or Window. Winit KeyCode mapping is pure.
//! Only the original Ports advances C1 and publishes its immutable Audit proof.

use crate::{
    AuditStatus, C1Domain, Player, invalid,
    three_way::Ports,
    tick_m1::{
        HeldDirection, MAX_INPUTS_PER_TICK, TickClock, TickCommand, TickInput, TickObservation,
    },
};
use std::{
    io,
    rc::{Rc, Weak},
};
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    W,
    A,
    S,
    D,
    Up,
    Left,
    Down,
    Right,
    Pause,
    Resume,
    Exit,
}

/// F1 physical-key mapping; it neither receives events nor opens a Window.
pub fn map_key(code: KeyCode) -> Option<Key> {
    match code {
        KeyCode::KeyW => Some(Key::W),
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyS => Some(Key::S),
        KeyCode::KeyD => Some(Key::D),
        KeyCode::ArrowUp => Some(Key::Up),
        KeyCode::ArrowLeft => Some(Key::Left),
        KeyCode::ArrowDown => Some(Key::Down),
        KeyCode::ArrowRight => Some(Key::Right),
        KeyCode::Space => Some(Key::Pause),
        KeyCode::Enter => Some(Key::Resume),
        KeyCode::Escape => Some(Key::Exit),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    /// Physical state/control admission changed. A shadowed direction key
    /// may change bookkeeping without adding a command.
    Queued,
    Unchanged,
    /// A caller decision only; it does not close Ports or enqueue a command.
    ExitRequested,
}

fn directions(keys: &[bool; 8]) -> [HeldDirection; 2] {
    fn choose(n: bool, s: bool, w: bool, e: bool) -> HeldDirection {
        if n {
            HeldDirection::North
        } else if s {
            HeldDirection::South
        } else if w {
            HeldDirection::West
        } else if e {
            HeldDirection::East
        } else {
            HeldDirection::Still
        }
    }
    [
        choose(keys[0], keys[2], keys[1], keys[3]),
        choose(keys[4], keys[6], keys[5], keys[7]),
    ]
}

fn key_slot(key: Key) -> Option<usize> {
    match key {
        Key::W => Some(0),
        Key::A => Some(1),
        Key::S => Some(2),
        Key::D => Some(3),
        Key::Up => Some(4),
        Key::Left => Some(5),
        Key::Down => Some(6),
        Key::Right => Some(7),
        Key::Pause | Key::Resume | Key::Exit => None,
    }
}

/// Pending input bound to one existing Ports authority before its first
/// accepted RA1 action/Tick. The Weak identity and revision fence are validation
/// tokens, not another state owner; no Snapshot, clock or positions are stored.
///
/// Another caller advancing the bound Ports makes this adapter stale. Every
/// subsequent operation rejects without changing its keys/pending commands.
/// There is no implicit rebase, rebind or reconstruction of physical key state.
pub struct NativeInput {
    owner: Weak<C1Domain>,
    accepted_revision: u64,
    pressed: [bool; 8],
    pending: Vec<TickCommand>,
}

pub(crate) struct PendingControls {
    pub(crate) paused: bool,
    pub(crate) reset_clock: bool,
}

impl NativeInput {
    /// Attach only to a live, nonzero-generation, initial Ports with open
    /// admission/Audit. Construction does not select RA1/Tick mode.
    pub fn new(ports: &Ports) -> io::Result<Self> {
        let input = Self {
            owner: Rc::downgrade(ports.c1()),
            accepted_revision: 0,
            pressed: [false; 8],
            pending: Vec::new(),
        };
        let before = input.observe(ports)?;
        if before.clock != TickClock::default() {
            return Err(invalid(
                "Native input must attach before the first accepted Tick",
            ));
        }
        Ok(input)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn observe(&self, ports: &Ports) -> io::Result<TickObservation> {
        if !Weak::ptr_eq(&self.owner, &Rc::downgrade(ports.c1())) {
            return Err(invalid("Native input belongs to another Ports authority"));
        }
        if let Some(error) = ports.publication_failure() {
            return Err(io::Error::other(error));
        }
        if ports.ingress().is_closed()?
            || ports.c1().is_stopped()
            || ports.audit().status() != AuditStatus::Open
        {
            return Err(io::Error::other(
                "Native input admission or authority closed",
            ));
        }
        let before = ports.c1().tick_observation();
        if before.snapshot.generation == 0
            || before.snapshot.revision != self.accepted_revision
            || before.clock.tick != self.accepted_revision
        {
            return Err(invalid(
                "Native input authority is stale or outside Tick mode",
            ));
        }
        Ok(before)
    }

    // Share the exact pending control projection with scheduling. The reset
    // request is derived before publication and never consumes pending commands.
    pub(crate) fn pending_controls(&self, published_paused: bool) -> PendingControls {
        let mut controls = PendingControls {
            paused: published_paused,
            reset_clock: false,
        };
        for command in &self.pending {
            match command {
                TickCommand::Pause => {
                    controls.paused = true;
                    controls.reset_clock = true;
                }
                TickCommand::Resume => {
                    controls.reset_clock |= controls.paused;
                    controls.paused = false;
                }
                TickCommand::Hold(_, _) => {}
            }
        }
        controls
    }

    fn reserve(&mut self, additional: usize) -> io::Result<()> {
        if self.pending.len() + additional > MAX_INPUTS_PER_TICK {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Native input queue is full",
            ));
        }
        self.pending
            .try_reserve(additional)
            .map_err(io::Error::other)
    }

    /// Preflight the complete direction change before touching either player.
    fn apply_keys(&mut self, future: [bool; 8]) -> io::Result<()> {
        let prior = directions(&self.pressed);
        let next = directions(&future);
        let additional = prior
            .iter()
            .zip(next)
            .filter(|(old, new)| **old != *new)
            .count();
        self.reserve(additional)?;
        for (index, (old, new)) in prior.into_iter().zip(next).enumerate() {
            if old != new {
                let player = if index == 0 { Player::One } else { Player::Two };
                self.pending.push(TickCommand::Hold(player, new));
            }
        }
        self.pressed = future;
        Ok(())
    }

    /// Prepare input only. Repeat and duplicate movement state are ignored.
    /// N > S > W > E selects each player's direction. Pause clears physical
    /// keys after admission; Resume clears them only for an effective pending
    /// Paused -> Running transition. Releases of control keys are ignored.
    ///
    /// A full32-command queue returns WouldBlock without partial mutation.
    /// A physical change requiring no additional command can still succeed.
    pub fn key(
        &mut self,
        ports: &Ports,
        key: Key,
        pressed: bool,
        repeat: bool,
    ) -> io::Result<KeyResult> {
        let before = self.observe(ports)?;
        if repeat {
            return Ok(KeyResult::Unchanged);
        }
        match key {
            Key::Exit if pressed => Ok(KeyResult::ExitRequested),
            Key::Pause if pressed => {
                self.reserve(1)?;
                self.pending.push(TickCommand::Pause);
                self.pressed = [false; 8];
                Ok(KeyResult::Queued)
            }
            Key::Resume if pressed => {
                let was_paused = self.pending_controls(before.snapshot.paused).paused;
                self.reserve(1)?;
                self.pending.push(TickCommand::Resume);
                if was_paused {
                    self.pressed = [false; 8];
                }
                Ok(KeyResult::Queued)
            }
            Key::Pause | Key::Resume | Key::Exit => Ok(KeyResult::Unchanged),
            _ => {
                let Some(slot) = key_slot(key) else {
                    return Ok(KeyResult::Unchanged);
                };
                let mut future = self.pressed;
                future[slot] = pressed;
                if future == self.pressed {
                    return Ok(KeyResult::Unchanged);
                }
                self.apply_keys(future)?;
                Ok(KeyResult::Queued)
            }
        }
    }

    /// Admit both players' required Still commands atomically, then clear keys.
    /// A rejected focus loss must be retried after space becomes available.
    pub fn lost_focus(&mut self, ports: &Ports) -> io::Result<()> {
        self.observe(ports)?;
        self.apply_keys([false; 8])
    }

    /// Explicitly publish the next logical Tick, including an empty input batch.
    /// Stamp from the original C1's current generation/Tick/lifetime sequence.
    /// Clear pending input and advance the fence only after the original Ports
    /// atomically publishes and commits. Any error preserves all input state.
    pub fn step_tick(&mut self, ports: &Ports) -> io::Result<TickObservation> {
        let before = self.observe(ports)?;
        let tick = before
            .clock
            .tick
            .checked_add(1)
            .ok_or_else(|| invalid("Native input Tick exhausted"))?;
        let mut sequence = before.clock.last_sequence;
        let mut stamped = Vec::new();
        stamped
            .try_reserve_exact(self.pending.len())
            .map_err(io::Error::other)?;
        for &command in &self.pending {
            sequence = sequence
                .checked_add(1)
                .ok_or_else(|| invalid("Native input sequence exhausted"))?;
            stamped.push(TickInput {
                generation: before.snapshot.generation,
                tick,
                sequence,
                command,
            });
        }
        let applied = ports.step_tick(tick, &stamped)?;
        // No fallible work or caller callback after successful publication.
        self.accepted_revision = applied.snapshot.revision;
        self.pending.clear();
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Command, Position, TICK_EVIDENCE_CAPACITY};
    use std::{sync::Arc, time::Instant};
    use yetlet::composition::{Builder, Completion};

    fn input_state(input: &NativeInput) -> ([bool; 8], Vec<TickCommand>, u64) {
        (
            input.pressed,
            input.pending.clone(),
            input.accepted_revision,
        )
    }

    fn key(input: &mut NativeInput, ports: &Ports, key: Key, pressed: bool) {
        assert_eq!(
            input.key(ports, key, pressed, false).unwrap(),
            KeyResult::Queued
        );
    }

    fn accept(input: &mut NativeInput, ports: &Ports) -> TickObservation {
        let before = ports.c1().tick_observation();
        let pending = input.pending.clone();
        let applied = input.step_tick(ports).unwrap();
        let proof = ports
            .pinned_tick(applied.snapshot.generation, applied.snapshot.revision)
            .unwrap();
        assert_eq!(proof.before, before);
        assert_eq!(proof.applied, applied);
        assert_eq!(ports.c1().tick_observation(), applied);
        assert_eq!(ports.audit().pinned(), applied.snapshot);
        assert_eq!(ports.audit().pinned_tick(), Some(applied));
        assert_eq!(
            ports
                .pinned(applied.snapshot.generation, applied.snapshot.revision)
                .unwrap(),
            applied.snapshot
        );
        assert_eq!(applied.clock.tick, applied.snapshot.revision);
        assert_eq!(proof.inputs.len(), pending.len());
        for (index, (stamped, command)) in proof.inputs.iter().zip(pending).enumerate() {
            assert_eq!(
                *stamped,
                TickInput {
                    generation: before.snapshot.generation,
                    tick: before.clock.tick + 1,
                    sequence: before.clock.last_sequence + index as u64 + 1,
                    command,
                }
            );
        }
        assert!(Arc::ptr_eq(
            ports.audit().tick_records().last().unwrap(),
            &proof
        ));
        assert_eq!(input.pending_count(), 0);
        assert_eq!(input.accepted_revision, applied.snapshot.revision);
        applied
    }

    #[test]
    fn maps_only_the_f1_physical_keys() {
        for (code, key) in [
            (KeyCode::KeyW, Key::W),
            (KeyCode::KeyA, Key::A),
            (KeyCode::KeyS, Key::S),
            (KeyCode::KeyD, Key::D),
            (KeyCode::ArrowUp, Key::Up),
            (KeyCode::ArrowLeft, Key::Left),
            (KeyCode::ArrowDown, Key::Down),
            (KeyCode::ArrowRight, Key::Right),
            (KeyCode::Space, Key::Pause),
            (KeyCode::Enter, Key::Resume),
            (KeyCode::Escape, Key::Exit),
        ] {
            assert_eq!(map_key(code), Some(key));
        }
        assert_eq!(map_key(KeyCode::F1), None);
        assert_eq!(map_key(KeyCode::NumpadEnter), None);
    }

    #[test]
    fn both_players_follow_priority_release_repeat_and_exit_semantics() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        for k in [
            Key::D,
            Key::A,
            Key::S,
            Key::W,
            Key::Right,
            Key::Left,
            Key::Down,
            Key::Up,
        ] {
            key(&mut input, &ports, k, true);
        }
        assert_eq!(input.pending_count(), 8);
        let prepared = input_state(&input);
        for k in [Key::W, Key::Up, Key::Pause, Key::Resume, Key::Exit] {
            for pressed in [true, false] {
                assert_eq!(
                    input.key(&ports, k, pressed, true).unwrap(),
                    KeyResult::Unchanged
                );
            }
        }
        for k in [Key::W, Key::Up] {
            assert_eq!(
                input.key(&ports, k, true, false).unwrap(),
                KeyResult::Unchanged
            );
        }
        for k in [Key::Pause, Key::Resume, Key::Exit] {
            assert_eq!(
                input.key(&ports, k, false, false).unwrap(),
                KeyResult::Unchanged
            );
        }
        assert_eq!(
            input.key(&ports, Key::Exit, true, false).unwrap(),
            KeyResult::ExitRequested
        );
        assert_eq!(input_state(&input), prepared);
        assert_eq!(ports.c1().snapshot().revision, 0);
        let first = accept(&mut input, &ports);
        assert_eq!(first.clock.held, [HeldDirection::North; 2]);
        assert_eq!(first.clock.last_sequence, 8);
        assert_eq!(
            first.snapshot.positions,
            [Position { x: 2, y: 1 }, Position { x: 20, y: 11 }]
        );
        for (released, held, sequence, positions) in [
            (
                [Key::W, Key::Up],
                HeldDirection::South,
                10,
                [Position { x: 2, y: 2 }, Position { x: 20, y: 12 }],
            ),
            (
                [Key::S, Key::Down],
                HeldDirection::West,
                12,
                [Position { x: 1, y: 2 }, Position { x: 19, y: 12 }],
            ),
            (
                [Key::A, Key::Left],
                HeldDirection::East,
                14,
                [Position { x: 2, y: 2 }, Position { x: 20, y: 12 }],
            ),
            (
                [Key::D, Key::Right],
                HeldDirection::Still,
                16,
                [Position { x: 2, y: 2 }, Position { x: 20, y: 12 }],
            ),
        ] {
            for k in released {
                key(&mut input, &ports, k, false);
            }
            let applied = accept(&mut input, &ports);
            assert_eq!(applied.clock.held, [held; 2]);
            assert_eq!(applied.clock.last_sequence, sequence);
            assert_eq!(applied.snapshot.positions, positions);
        }
    }

    #[test]
    fn full_queue_preserves_rejected_state_and_admits_shadowed_keys() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        for k in [Key::W, Key::Up] {
            key(&mut input, &ports, k, true);
        }
        for _ in 0..30 {
            key(&mut input, &ports, Key::Resume, true);
        }
        assert_eq!(input.pending_count(), MAX_INPUTS_PER_TICK);
        let before = input_state(&input);
        for (k, pressed) in [(Key::Pause, true), (Key::Resume, true), (Key::W, false)] {
            assert_eq!(
                input.key(&ports, k, pressed, false).unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
            assert_eq!(input_state(&input), before);
        }
        assert_eq!(
            input.lost_focus(&ports).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(input_state(&input), before);
        for k in [Key::D, Key::Right] {
            key(&mut input, &ports, k, true);
        }
        assert_eq!(input.pending_count(), MAX_INPUTS_PER_TICK);
        let first = accept(&mut input, &ports);
        assert_eq!(first.clock.held, [HeldDirection::North; 2]);
        assert_eq!(first.clock.last_sequence, 32);
        let empty = accept(&mut input, &ports);
        assert_eq!(empty.clock.last_sequence, 32);
        assert_eq!(
            empty.snapshot.positions,
            [Position { x: 2, y: 0 }, Position { x: 20, y: 10 }]
        );
        for k in [Key::W, Key::Up] {
            key(&mut input, &ports, k, false);
        }
        let released = accept(&mut input, &ports);
        assert_eq!(released.clock.held, [HeldDirection::East; 2]);
        assert_eq!(released.clock.last_sequence, 34);
        assert_eq!(
            released.snapshot.positions,
            [Position { x: 3, y: 0 }, Position { x: 21, y: 10 }]
        );
    }

    #[test]
    fn resume_projects_pending_controls_before_clearing_keys() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        key(&mut input, &ports, Key::Pause, true);
        let paused = accept(&mut input, &ports);
        assert!(paused.snapshot.paused);
        for k in [Key::Resume, Key::W, Key::Resume] {
            key(&mut input, &ports, k, true);
        }
        key(&mut input, &ports, Key::W, false);
        let resumed = accept(&mut input, &ports);
        assert!(!resumed.snapshot.paused);
        assert_eq!(resumed.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(resumed.clock.last_sequence, 5);
        assert_eq!(resumed.snapshot.positions, paused.snapshot.positions);

        for k in [Key::Pause, Key::D, Key::Resume, Key::D] {
            key(&mut input, &ports, k, true);
        }
        let fresh = accept(&mut input, &ports);
        assert_eq!(
            fresh.clock.held,
            [HeldDirection::East, HeldDirection::Still]
        );
        assert_eq!(fresh.clock.last_sequence, 9);
        assert_eq!(fresh.snapshot.positions[0], Position { x: 3, y: 2 });
    }

    #[test]
    fn effective_resume_rejection_retains_keys_and_paused_holds_are_forgotten() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        key(&mut input, &ports, Key::Pause, true);
        let first = accept(&mut input, &ports);
        for _ in 0..31 {
            key(&mut input, &ports, Key::Pause, true);
        }
        key(&mut input, &ports, Key::W, true);
        let before = input_state(&input);
        for k in [Key::Resume, Key::Pause] {
            assert_eq!(
                input.key(&ports, k, true, false).unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
            assert_eq!(input_state(&input), before);
        }
        let still_paused = accept(&mut input, &ports);
        assert!(still_paused.snapshot.paused);
        assert_eq!(still_paused.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(still_paused.clock.last_sequence, 33);
        assert_eq!(still_paused.clock.active_ticks, 0);
        assert_eq!(still_paused.snapshot.positions, first.snapshot.positions);
        key(&mut input, &ports, Key::Resume, true);
        let resumed = accept(&mut input, &ports);
        assert_eq!(resumed.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(input.pressed, [false; 8]);
        key(&mut input, &ports, Key::W, true);
        let fresh = accept(&mut input, &ports);
        assert_eq!(fresh.clock.held[0], HeldDirection::North);
        assert_eq!(fresh.snapshot.positions[0].y, 1);
    }

    #[test]
    fn focus_loss_preflights_both_players_then_retries_after_publication() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        for k in [Key::W, Key::Up] {
            key(&mut input, &ports, k, true);
        }
        for _ in 0..29 {
            key(&mut input, &ports, Key::Resume, true);
        }
        assert_eq!(input.pending_count(), 31);
        let before = input_state(&input);
        assert_eq!(
            input.lost_focus(&ports).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(input_state(&input), before);
        let held = accept(&mut input, &ports);
        input.lost_focus(&ports).unwrap();
        assert_eq!(input.pressed, [false; 8]);
        assert_eq!(input.pending_count(), 2);
        input.lost_focus(&ports).unwrap();
        assert_eq!(input.pending_count(), 2);
        let released = accept(&mut input, &ports);
        assert_eq!(released.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(released.clock.last_sequence, 33);
        assert_eq!(released.snapshot.positions, held.snapshot.positions);
    }

    #[test]
    fn exact_owner_and_revision_fences_reject_without_implicit_rebase() {
        let ports = Ports::new(91).unwrap();
        let other = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        key(&mut input, &ports, Key::W, true);
        let before = input_state(&input);
        assert!(input.key(&other, Key::D, true, false).is_err());
        assert!(input.lost_focus(&other).is_err());
        assert!(input.step_tick(&other).is_err());
        assert_eq!(input_state(&input), before);
        assert_eq!(other.c1().snapshot().revision, 0);
        assert_eq!(ports.c1().snapshot().revision, 0);
        accept(&mut input, &ports);
        assert!(NativeInput::new(&ports).is_err());
        key(&mut input, &ports, Key::D, true);
        key(&mut input, &ports, Key::W, false);
        let stale = input_state(&input);
        let external = ports.step_tick(2, &[]).unwrap();
        assert!(input.key(&ports, Key::D, false, false).is_err());
        assert!(input.lost_focus(&ports).is_err());
        assert!(input.step_tick(&ports).is_err());
        assert_eq!(input_state(&input), stale);
        assert_eq!(ports.c1().tick_observation(), external);
        assert_eq!(ports.audit().pinned_tick(), Some(external));
        assert!(NativeInput::new(&Ports::new(0).unwrap()).is_err());
    }

    #[test]
    fn preparation_does_not_preempt_ra1_mode_or_mutate_after_mode_selection() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        key(&mut input, &ports, Key::W, true);
        let before = input_state(&input);
        let ra1 = ports
            .apply(Command {
                generation: 91,
                expected_revision: 0,
                action: Action::Pause,
            })
            .unwrap();
        assert!(input.step_tick(&ports).is_err());
        assert!(input.key(&ports, Key::Resume, true, false).is_err());
        assert!(input.lost_focus(&ports).is_err());
        assert_eq!(input_state(&input), before);
        assert_eq!(ports.c1().snapshot(), ra1);
        assert_eq!(ports.audit().records().len(), 1);
        assert!(ports.audit().tick_records().is_empty());
        assert!(NativeInput::new(&ports).is_err());
    }

    #[test]
    fn original_audit_borrow_rejection_retains_batch_and_sequence_for_retry() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        for k in [Key::D, Key::Up] {
            key(&mut input, &ports, k, true);
        }
        let before = input_state(&input);
        let observation = ports.c1().tick_observation();
        let guard = ports.audit().records.borrow();
        assert!(input.step_tick(&ports).is_err());
        assert_eq!(input_state(&input), before);
        assert_eq!(ports.c1().tick_observation(), observation);
        assert_eq!(ports.pinned(91, 0).unwrap(), observation.snapshot);
        assert_eq!(ports.audit().pinned(), observation.snapshot);
        assert!(ports.audit().pinned_tick().is_none());
        assert!(ports.pinned_tick(91, 1).is_err());
        drop(guard);
        let applied = accept(&mut input, &ports);
        assert_eq!(applied.clock.last_sequence, 2);
        assert_eq!(
            applied.clock.held,
            [HeldDirection::East, HeldDirection::North]
        );
    }

    #[test]
    fn checked_stamp_overflow_preserves_pending_input_and_identity() {
        for (tick, sequence) in [(u64::MAX, 0), (0, u64::MAX - 1)] {
            let ports = Ports::new(91).unwrap();
            let mut input = NativeInput::new(&ports).unwrap();
            for k in [Key::W, Key::Up] {
                key(&mut input, &ports, k, true);
            }
            // Private counter-boundary fixture reaches the new stamping checks;
            // no public owner mutation or production fault-injection seam.
            let mut observation = ports.c1().tick_observation();
            observation.snapshot.revision = tick;
            observation.clock.tick = tick;
            observation.clock.last_sequence = sequence;
            ports.c1().state.set(observation.snapshot);
            ports.c1().tick_clock.set(observation.clock);
            ports.audit().pinned.set(observation.snapshot);
            ports
                .audit()
                .tick_pinned
                .set((tick != 0).then_some(observation.clock));
            input.accepted_revision = tick;
            let before = input_state(&input);
            assert_eq!(
                input.step_tick(&ports).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(input_state(&input), before);
            assert_eq!(ports.c1().tick_observation(), observation);
            assert_eq!(ports.audit().pinned(), observation.snapshot);
            assert!(ports.audit().tick_records().is_empty());
            assert_eq!(ports.pinned(91, 0).unwrap().revision, 0);
        }
    }

    #[test]
    fn explicit_key_ticks_cross_retention_and_close_original_composition() {
        let ports = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&ports).unwrap();
        let mut first = None;
        let mut last = ports.c1().tick_observation();
        for tick in 1..=33 {
            if tick > 1 {
                input.lost_focus(&ports).unwrap();
            }
            for code in [KeyCode::KeyD, KeyCode::ArrowLeft] {
                key(&mut input, &ports, map_key(code).unwrap(), true);
            }
            last = accept(&mut input, &ports);
            assert_eq!(last.clock.tick, tick);
            assert_eq!(last.clock.active_ticks, tick);
            assert_eq!(last.clock.last_sequence, tick * 4 - 2);
            if tick == 1 {
                let proof = ports.pinned_tick(91, 1).unwrap();
                first = Some((Arc::clone(&proof), (*proof).clone()));
            }
        }
        let (first, saved) = first.unwrap();
        assert_eq!(ports.audit().tick_records().len(), TICK_EVIDENCE_CAPACITY);
        assert_eq!(ports.audit().tick_records()[0].applied.clock.tick, 18);
        assert_eq!(
            ports.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(
            ports.pinned(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(*first, saved);
        assert_eq!(first.applied.snapshot.text(), "91 1 false 3 2 19 12");
        key(&mut input, &ports, Key::W, true);
        let pending = input_state(&input);
        ports.ingress().close().unwrap();
        assert!(input.key(&ports, Key::W, false, false).is_err());
        assert!(input.lost_focus(&ports).is_err());
        assert!(input.step_tick(&ports).is_err());
        assert!(NativeInput::new(&ports).is_err());
        assert_eq!(input_state(&input), pending);

        let c1 = Rc::clone(ports.c1());
        let audit = Rc::clone(ports.audit());
        let attachment = ports.audit_attachment();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", ports, Ports::close)
            .required_local("audit", attachment, crate::three_way::close_audit)
            .build();
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(root.failures().primary().is_none());
        assert!(c1.is_stopped());
        assert_eq!(audit.status(), AuditStatus::Closed);
        assert_eq!(c1.tick_observation(), last);
        assert_eq!(audit.pinned_tick(), Some(last));
        assert_eq!(audit.tick_records().len(), TICK_EVIDENCE_CAPACITY);
        assert_eq!(input_state(&input), pending);
    }
}
