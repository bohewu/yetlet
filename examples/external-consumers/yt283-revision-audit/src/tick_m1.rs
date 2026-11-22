//! YT-283-M1: consumer-owned deterministic logical ticks, not an SDK scheduler.
//!
//! A tick is one logical 1/60-second step. All movement still changes the
//! *existing* caller-local C1Domain Snapshot. The TickClock stored in that C1
//! owns only time, sequence and held-input metadata, never a second position
//! authority. Bound consumers publish through the original Ports/Bridge/Audit
//! in YT-283-M2-A; direct local mutation remains unavailable on a bound C1.
//! Accepted RA1 actions and accepted ticks still select exclusive modes.

use crate::{ARENA_HEIGHT, ARENA_WIDTH, Player, Position, SQUARE_SIZE, Snapshot};
use std::fmt;

pub const TICKS_PER_SECOND: u32 = 60;
pub const MAX_INPUTS_PER_TICK: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldDirection {
    Still,
    North,
    East,
    South,
    West,
}

impl HeldDirection {
    fn delta(self) -> (i32, i32) {
        match self {
            Self::Still => (0, 0),
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickCommand {
    Hold(Player, HeldDirection),
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickInput {
    pub generation: u64,
    pub tick: u64,
    pub sequence: u64,
    pub command: TickCommand,
}

/// Copy-only metadata in the sole consumer C1. No positions or second Snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickClock {
    pub tick: u64,
    pub active_ticks: u64,
    pub last_sequence: u64,
    pub held: [HeldDirection; 2],
}

impl Default for TickClock {
    fn default() -> Self {
        Self {
            tick: 0,
            active_ticks: 0,
            last_sequence: 0,
            held: [HeldDirection::Still; 2],
        }
    }
}

/// One copied observation of the *same* existing authoritative C1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickObservation {
    pub snapshot: Snapshot,
    pub clock: TickClock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickError {
    InvalidGeneration,
    Stopped,
    ModeConflict,
    WrongTick,
    WrongGeneration,
    TooManyInputs,
    SequenceMismatch,
    TickOverflow,
    RevisionOverflow,
    SequenceOverflow,
    ActiveTickOverflow,
}

impl fmt::Display for TickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "YT-283-M1 tick rejected: {self:?}")
    }
}
impl std::error::Error for TickError {}

/// Pure candidate reduction. The C1 caller publishes both copied values only
/// after this returns Ok. No failure or caller callback follows that commit.
pub(crate) fn reduce(
    current: Snapshot,
    clock: TickClock,
    target_tick: u64,
    inputs: &[TickInput],
) -> Result<TickObservation, TickError> {
    if current.generation == 0 {
        return Err(TickError::InvalidGeneration);
    }
    if target_tick != clock.tick.checked_add(1).ok_or(TickError::TickOverflow)? {
        return Err(TickError::WrongTick);
    }
    if inputs.len() > MAX_INPUTS_PER_TICK {
        return Err(TickError::TooManyInputs);
    }

    let mut ordered = inputs.to_vec();
    ordered.sort_unstable_by_key(|input| input.sequence);
    let mut next_sequence = clock.last_sequence;
    for input in &ordered {
        if input.generation != current.generation {
            return Err(TickError::WrongGeneration);
        }
        if input.tick != target_tick {
            return Err(TickError::WrongTick);
        }
        next_sequence = next_sequence
            .checked_add(1)
            .ok_or(TickError::SequenceOverflow)?;
        if input.sequence != next_sequence {
            return Err(TickError::SequenceMismatch);
        }
    }

    let mut next = current;
    let mut meta = clock;
    next.revision = current
        .revision
        .checked_add(1)
        .ok_or(TickError::RevisionOverflow)?;
    meta.tick = target_tick;
    meta.last_sequence = next_sequence;
    for input in ordered {
        match input.command {
            TickCommand::Pause => {
                next.paused = true;
                meta.held = [HeldDirection::Still; 2];
            }
            TickCommand::Resume => {
                if next.paused {
                    next.paused = false;
                    meta.held = [HeldDirection::Still; 2];
                }
            }
            TickCommand::Hold(player, direction) if !next.paused => {
                meta.held[match player {
                    Player::One => 0,
                    Player::Two => 1,
                }] = direction;
            }
            TickCommand::Hold(_, _) => {} // paused input is consumed, not buffered
        }
    }

    if !next.paused {
        meta.active_ticks = clock
            .active_ticks
            .checked_add(1)
            .ok_or(TickError::ActiveTickOverflow)?;
        for (position, held) in next.positions.iter_mut().zip(meta.held) {
            let Position { x, y } = *position;
            let (dx, dy) = held.delta();
            position.x = (x + dx).clamp(0, ARENA_WIDTH - SQUARE_SIZE);
            position.y = (y + dy).clamp(0, ARENA_HEIGHT - SQUARE_SIZE);
        }
    }
    Ok(TickObservation {
        snapshot: next,
        clock: meta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, C1Domain, Command, Direction, RevisionAuditHost};
    use std::{io, rc::Rc, time::Instant};
    use yetlet::composition::{Builder, Completion};

    fn input(tick: u64, sequence: u64, command: TickCommand) -> TickInput {
        TickInput {
            generation: 7,
            tick,
            sequence,
            command,
        }
    }

    fn hold(tick: u64, seq: u64, player: Player, dir: HeldDirection) -> TickInput {
        input(tick, seq, TickCommand::Hold(player, dir))
    }

    #[test]
    fn replay_is_deterministic_permutation_independent_and_uses_one_c1() {
        let a = Rc::new(C1Domain::new(7));
        let b = Rc::new(C1Domain::new(7));
        let frames: [Vec<TickInput>; 8] = [
            vec![
                hold(1, 2, Player::Two, HeldDirection::North),
                hold(1, 1, Player::One, HeldDirection::East),
            ],
            vec![],
            vec![input(3, 3, TickCommand::Pause)],
            vec![hold(4, 4, Player::One, HeldDirection::West)],
            vec![],
            vec![input(6, 5, TickCommand::Resume)],
            vec![
                hold(7, 7, Player::Two, HeldDirection::West),
                hold(7, 6, Player::One, HeldDirection::South),
            ],
            vec![],
        ];

        let pinned_initial = a.snapshot();
        let alias = Rc::clone(&a);
        let mut root = Builder::<io::Error>::new()
            .required_local("sole C1", alias, |c| {
                c.close_tick_m1().map_err(io::Error::other)
            })
            .build();
        assert!(root.all_required_ready());
        assert_eq!(TICKS_PER_SECOND, 60);
        for (i, frame) in frames.iter().enumerate() {
            let tick = i as u64 + 1;
            let mut permuted = frame.clone();
            permuted.reverse();
            let current = a.step_tick(tick, frame).unwrap();
            assert_eq!(b.step_tick(tick, &permuted).unwrap(), current);
            assert_eq!(a.snapshot(), current.snapshot);
            assert_eq!(a.tick_observation(), current);
        }
        assert_eq!(pinned_initial.revision, 0);
        let final_state = a.tick_observation();
        assert_eq!(
            final_state.snapshot.positions,
            [Position { x: 4, y: 4 }, Position { x: 18, y: 10 }]
        );
        assert_eq!(
            (
                final_state.clock.tick,
                final_state.clock.active_ticks,
                final_state.clock.last_sequence
            ),
            (8, 5, 7)
        );
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(a.is_stopped());
        assert_eq!(a.step_tick(9, &[]), Err(TickError::Stopped));
        assert!(!b.is_stopped());
    }

    #[test]
    fn paused_ticks_discard_held_and_do_not_catch_up_on_resume() {
        let c = C1Domain::new(7);
        c.step_tick(1, &[hold(1, 1, Player::One, HeldDirection::East)])
            .unwrap();
        let paused = c.step_tick(2, &[input(2, 2, TickCommand::Pause)]).unwrap();
        for tick in 3..=102 {
            let inputs = if tick == 3 {
                vec![hold(3, 3, Player::One, HeldDirection::West)]
            } else {
                vec![]
            };
            let current = c.step_tick(tick, &inputs).unwrap();
            assert_eq!(current.snapshot.positions, paused.snapshot.positions);
            assert_eq!(current.clock.active_ticks, 1);
            assert_eq!(current.clock.held, [HeldDirection::Still; 2]);
        }
        let resumed = c
            .step_tick(103, &[input(103, 4, TickCommand::Resume)])
            .unwrap();
        assert!(!resumed.snapshot.paused);
        assert_eq!(resumed.clock.active_ticks, 2);
        assert_eq!(resumed.snapshot.positions, paused.snapshot.positions);
        assert_eq!(
            c.step_tick(104, &[hold(104, 5, Player::One, HeldDirection::South)])
                .unwrap()
                .snapshot
                .positions[0],
            Position { x: 3, y: 3 }
        );
    }

    #[test]
    fn input_validation_is_transactional_and_batch_is_bounded() {
        let c = C1Domain::new(7);
        let before = c.tick_observation();
        let valid = hold(1, 1, Player::One, HeldDirection::East);
        let wrong = TickInput {
            generation: 9,
            ..hold(1, 2, Player::Two, HeldDirection::North)
        };
        for (tick, inputs, error) in [
            (2, vec![], TickError::WrongTick),
            (1, vec![valid, wrong], TickError::WrongGeneration),
            (1, vec![valid, valid], TickError::SequenceMismatch),
            (
                1,
                vec![valid, hold(1, 3, Player::Two, HeldDirection::North)],
                TickError::SequenceMismatch,
            ),
            (
                1,
                vec![valid, TickInput { tick: 2, ..valid }],
                TickError::WrongTick,
            ),
        ] {
            assert_eq!(c.step_tick(tick, &inputs), Err(error));
            assert_eq!(c.tick_observation(), before);
        }
        let inputs: Vec<_> = (1..=MAX_INPUTS_PER_TICK as u64 + 1)
            .map(|seq| input(1, seq, TickCommand::Pause))
            .collect();
        assert_eq!(c.step_tick(1, &inputs), Err(TickError::TooManyInputs));
        assert_eq!(c.tick_observation(), before);
        assert_eq!(
            c.step_tick(1, &inputs[..MAX_INPUTS_PER_TICK])
                .unwrap()
                .clock
                .last_sequence,
            32
        );
        assert_eq!(c.step_tick(1, &[]), Err(TickError::WrongTick));
    }

    #[test]
    fn both_full_squares_clamp_and_released_input_stops_further_motion() {
        for (direction, axis, target) in [
            (HeldDirection::West, 0, 0),
            (HeldDirection::East, 0, ARENA_WIDTH - SQUARE_SIZE),
            (HeldDirection::North, 1, 0),
            (HeldDirection::South, 1, ARENA_HEIGHT - SQUARE_SIZE),
        ] {
            let c = C1Domain::new(7);
            for tick in 1..=40 {
                let inputs = if tick == 1 {
                    vec![
                        hold(tick, 1, Player::One, direction),
                        hold(tick, 2, Player::Two, direction),
                    ]
                } else {
                    vec![]
                };
                c.step_tick(tick, &inputs).unwrap();
            }
            for pos in c.snapshot().positions {
                assert!((0..=ARENA_WIDTH - SQUARE_SIZE).contains(&pos.x));
                assert!((0..=ARENA_HEIGHT - SQUARE_SIZE).contains(&pos.y));
                assert_eq!(if axis == 0 { pos.x } else { pos.y }, target);
            }
        }
        let c = C1Domain::new(7);
        c.step_tick(1, &[hold(1, 1, Player::One, HeldDirection::East)])
            .unwrap();
        let stop = c
            .step_tick(2, &[hold(2, 2, Player::One, HeldDirection::Still)])
            .unwrap();
        c.step_tick(3, &[]).unwrap();
        assert_eq!(c.snapshot().positions, stop.snapshot.positions);
    }

    #[test]
    fn headless_tick_and_ra1_are_exclusive_without_second_authority() {
        let c = C1Domain::new(7);
        assert_eq!(c.step_tick(1, &[]).unwrap().clock.tick, 1);
        let audit = RevisionAuditHost::new(c.snapshot());
        assert!(
            c.preflight(
                Command {
                    generation: 7,
                    expected_revision: 1,
                    action: Action::Move {
                        player: Player::One,
                        direction: Direction::East,
                    },
                },
                &audit,
            )
            .is_err()
        );
        assert!(c.close(&audit).is_err()); // tick mode never forges Audit
        let other = C1Domain::new(7);
        let other_audit = RevisionAuditHost::new(other.snapshot());
        let input = Command {
            generation: 7,
            expected_revision: 0,
            action: Action::Pause,
        };
        let proposed = other.preflight(input, &other_audit).unwrap();
        let candidate = proposed.candidate();
        other
            .commit_published(proposed, &other_audit, candidate)
            .unwrap();
        assert_eq!(other.step_tick(1, &[]), Err(TickError::ModeConflict));
        assert_eq!(other.tick_observation().clock.tick, 0);
    }

    #[cfg(feature = "three-way")]
    #[test]
    fn ports_bound_ra1_owner_cannot_accept_local_ticks_at_revision_zero() {
        let mut ports = crate::three_way::Ports::new(81).unwrap();
        let original = ports.c1().snapshot();
        assert_eq!(ports.c1().close_tick_m1(), Err(TickError::ModeConflict));
        assert!(!ports.c1().is_stopped());
        assert_eq!(
            ports.c1().step_tick(1, &[]),
            Err(TickError::ModeConflict),
            "the public C1 handle must not bypass its bound Audit/bridge"
        );
        assert_eq!(ports.c1().snapshot(), original);
        assert_eq!(ports.pinned(81, 0).unwrap(), original);
        assert_eq!(ports.audit().pinned(), original);
        assert!(ports.audit().records().is_empty());
        ports.ingress().close().unwrap();
        let mut attached = ports.audit_attachment();
        attached.close().unwrap();
        ports.close().unwrap();
        assert!(ports.c1().is_stopped());
    }

    #[test]
    fn local_tick_root_can_close_before_first_tick_or_after_rejected_input() {
        for rejected_first_input in [false, true] {
            let owner = Rc::new(C1Domain::new(7));
            let root_owner = Rc::clone(&owner);
            let mut root = Builder::<io::Error>::new()
                .required_local("sole pre-tick C1", root_owner, |c| {
                    c.close_tick_m1().map_err(io::Error::other)
                })
                .build();
            assert!(root.all_required_ready());
            if rejected_first_input {
                assert_eq!(
                    owner.step_tick(
                        1,
                        &[TickInput {
                            generation: 9,
                            tick: 1,
                            sequence: 1,
                            command: TickCommand::Pause
                        }]
                    ),
                    Err(TickError::WrongGeneration)
                );
            }
            assert_eq!(owner.tick_observation().clock.tick, 0);
            assert_eq!(owner.snapshot().revision, 0);
            assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
            assert!(owner.is_stopped());
            assert_eq!(owner.step_tick(1, &[]), Err(TickError::Stopped));
            assert!(root.failures().primary().is_none());
        }
    }

    #[test]
    fn overflow_and_invalid_generation_reject_without_commit() {
        let c = C1Domain::new(0);
        assert_eq!(c.step_tick(1, &[]), Err(TickError::InvalidGeneration));
        assert_eq!(c.tick_observation().snapshot.revision, 0);
        let state = C1Domain::new(7).snapshot();
        let clock = TickClock {
            tick: u64::MAX,
            ..TickClock::default()
        };
        assert_eq!(
            reduce(state, clock, u64::MAX, &[]),
            Err(TickError::TickOverflow)
        );
        let clock = TickClock {
            last_sequence: u64::MAX,
            ..TickClock::default()
        };
        assert_eq!(
            reduce(
                state,
                clock,
                1,
                &[hold(1, 1, Player::One, HeldDirection::East)]
            ),
            Err(TickError::SequenceOverflow)
        );
        let clock = TickClock {
            active_ticks: u64::MAX,
            ..TickClock::default()
        };
        assert_eq!(
            reduce(state, clock, 1, &[]),
            Err(TickError::ActiveTickOverflow)
        );
        let state = Snapshot {
            revision: u64::MAX,
            ..state
        };
        assert_eq!(
            reduce(state, TickClock::default(), 1, &[]),
            Err(TickError::RevisionOverflow)
        );
    }
}
