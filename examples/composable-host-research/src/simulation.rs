//! C1 / YT-268: Twin Squares, a private integer-grid simulation.
//!
//! One logical step is 1/60 second. No wall clock, worker, renderer or IO is
//! needed. HeadlessRunner calls the same Simulation::step entry point that a
//! future foreground host can call. This does not implement that future host.

use std::{fmt, rc::Rc};

pub const TICKS_PER_SECOND: u32 = 60;
pub const ARENA_WIDTH: i32 = 24;
pub const ARENA_HEIGHT: i32 = 16;
pub const SQUARE_SIZE: i32 = 2;
pub const MAX_COMMANDS_PER_TICK: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}
impl Player {
    fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Still,
    North,
    East,
    South,
    West,
}
impl Direction {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Hold(Player, Direction),
    Pause,
    Resume,
}

/// Generation and tick bind an input to one lifetime and one step. Sequence
/// numbers are contiguous across that lifetime, starting at 1. They are not
/// player IDs, wall time or snapshot revisions. Transport parsing is out of scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub generation: u64,
    pub tick: u64,
    pub sequence: u64,
    pub command: Command,
}

/// Complete immutable publication. All fields are private; accessors return
/// copies, never mutable references or aliases to authoritative state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    generation: u64,
    tick: u64,
    active_ticks: u64,
    revision: u64,
    last_sequence: u64,
    paused: bool,
    positions: [Position; 2],
    held: [Direction; 2],
}
impl Snapshot {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn active_ticks(&self) -> u64 {
        self.active_ticks
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn positions(&self) -> [Position; 2] {
        self.positions
    }
    pub fn held(&self) -> [Direction; 2] {
        self.held
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidGeneration,
    WrongTick,
    WrongGeneration,
    TooManyCommands,
    SequenceMismatch,
    TickOverflow,
    ActiveTickOverflow,
    RevisionOverflow,
    SequenceOverflow,
}
impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "Twin Squares step rejected: {self:?}")
    }
}
impl std::error::Error for Error {}

/// The sole authoritative owner. No Clone or public state injection API.
/// A caller can hold any number of prior publications without affecting it.
pub struct Simulation {
    current: Rc<Snapshot>,
}
impl Simulation {
    pub fn new(generation: u64) -> Result<Self, Error> {
        if generation == 0 {
            return Err(Error::InvalidGeneration);
        }
        Ok(Self {
            current: Rc::new(Snapshot {
                generation,
                tick: 0,
                active_ticks: 0,
                revision: 0,
                last_sequence: 0,
                paused: false,
                positions: [Position { x: 2, y: 2 }, Position { x: 20, y: 12 }],
                held: [Direction::Still; 2],
            }),
        })
    }

    pub fn snapshot(&self) -> Rc<Snapshot> {
        Rc::clone(&self.current)
    }

    pub fn next_tick(&self) -> Result<u64, Error> {
        self.current.tick.checked_add(1).ok_or(Error::TickOverflow)
    }

    /// Validate and reduce an entire bounded tick transaction before publishing.
    /// Errors preserve the exact previous Rc and consume no command sequence.
    pub fn step(&mut self, tick: u64, inputs: &[Input]) -> Result<Rc<Snapshot>, Error> {
        let next = reduce(&self.current, tick, inputs)?;
        self.current = Rc::new(next);
        Ok(self.snapshot())
    }
}

/// The only reducer. Sorting is bounded by MAX_COMMANDS_PER_TICK, with unique
/// contiguous sequence numbers enforced before any simulation mutation.
fn reduce(current: &Snapshot, tick: u64, inputs: &[Input]) -> Result<Snapshot, Error> {
    let expected_tick = current.tick.checked_add(1).ok_or(Error::TickOverflow)?;
    if tick != expected_tick {
        return Err(Error::WrongTick);
    }
    if inputs.len() > MAX_COMMANDS_PER_TICK {
        return Err(Error::TooManyCommands);
    }
    let mut ordered = inputs.to_vec();
    ordered.sort_unstable_by_key(|input| input.sequence);
    let mut sequence = current.last_sequence;
    for input in &ordered {
        if input.generation != current.generation {
            return Err(Error::WrongGeneration);
        }
        if input.tick != tick {
            return Err(Error::WrongTick);
        }
        sequence = sequence.checked_add(1).ok_or(Error::SequenceOverflow)?;
        if input.sequence != sequence {
            return Err(Error::SequenceMismatch);
        }
    }

    let mut next = current.clone();
    next.revision = current
        .revision
        .checked_add(1)
        .ok_or(Error::RevisionOverflow)?;
    next.tick = tick;
    next.last_sequence = sequence;
    for input in &ordered {
        match input.command {
            Command::Pause => {
                next.paused = true;
                next.held = [Direction::Still; 2];
            }
            Command::Resume => {
                if next.paused {
                    next.paused = false;
                    next.held = [Direction::Still; 2];
                }
            }
            Command::Hold(player, direction) => {
                // Input received while paused is consumed but never buffered
                // as movement to be replayed after Resume.
                if !next.paused {
                    next.held[player.index()] = direction;
                }
            }
        }
    }
    if !next.paused {
        next.active_ticks = current
            .active_ticks
            .checked_add(1)
            .ok_or(Error::ActiveTickOverflow)?;
        for (position, direction) in next.positions.iter_mut().zip(next.held) {
            let (dx, dy) = direction.delta();
            // Private valid positions plus a cardinal unit delta fit i32.
            position.x = (position.x + dx).clamp(0, ARENA_WIDTH - SQUARE_SIZE);
            position.y = (position.y + dy).clamp(0, ARENA_HEIGHT - SQUARE_SIZE);
        }
    }
    // Squares may overlap. This small arena has no collision/physics engine.
    Ok(next)
}

/// Explicit-tick headless driver, not a real-time scheduler or a framework Host.
/// Caller owns this runner; neither construction nor step spawns anything.
pub struct HeadlessRunner {
    simulation: Simulation,
}
impl HeadlessRunner {
    pub fn new(generation: u64) -> Result<Self, Error> {
        Ok(Self {
            simulation: Simulation::new(generation)?,
        })
    }
    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.simulation.snapshot()
    }
    pub fn tick(&mut self, inputs: &[Input]) -> Result<Rc<Snapshot>, Error> {
        self.simulation.step(self.simulation.next_tick()?, inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(tick: u64, sequence: u64, command: Command) -> Input {
        Input {
            generation: 7,
            tick,
            sequence,
            command,
        }
    }
    fn hold(tick: u64, sequence: u64, player: Player, direction: Direction) -> Input {
        input(tick, sequence, Command::Hold(player, direction))
    }
    fn assert_rejected(sim: &mut Simulation, tick: u64, inputs: &[Input], error: Error) {
        let before = sim.snapshot();
        assert_eq!(sim.step(tick, inputs), Err(error));
        assert!(Rc::ptr_eq(&before, &sim.snapshot()));
    }
    fn replay_frames() -> Vec<Vec<Input>> {
        vec![
            vec![
                hold(1, 2, Player::Two, Direction::North),
                hold(1, 1, Player::One, Direction::East),
            ],
            vec![],
            vec![input(3, 3, Command::Pause)],
            vec![hold(4, 4, Player::One, Direction::West)],
            vec![],
            vec![input(6, 5, Command::Resume)],
            vec![
                hold(7, 7, Player::Two, Direction::West),
                hold(7, 6, Player::One, Direction::South),
            ],
            vec![],
        ]
    }

    #[test]
    fn initial_state_is_bounded_and_generation_is_explicit() {
        assert!(matches!(Simulation::new(0), Err(Error::InvalidGeneration)));
        let state = Simulation::new(7).unwrap().snapshot();
        assert_eq!(state.generation(), 7);
        assert_eq!(
            (
                state.tick(),
                state.active_ticks(),
                state.revision(),
                state.last_sequence()
            ),
            (0, 0, 0, 0)
        );
        assert_ne!(state.positions()[0], state.positions()[1]);
        assert_eq!(state.held(), [Direction::Still; 2]);
        assert_eq!(TICKS_PER_SECOND, 60);
    }

    #[test]
    fn headless_replay_and_direct_reducer_match_every_tick() {
        let mut a = HeadlessRunner::new(7).unwrap();
        let mut b = HeadlessRunner::new(7).unwrap();
        let mut direct = (*a.snapshot()).clone();
        for (index, frame) in replay_frames().iter().enumerate() {
            let tick = u64::try_from(index + 1).unwrap();
            direct = reduce(&direct, tick, frame).unwrap();
            assert_eq!(*a.tick(frame).unwrap(), direct);
            assert_eq!(*b.tick(frame).unwrap(), direct);
        }
        let result = a.snapshot();
        assert_eq!(
            result.positions(),
            [Position { x: 4, y: 4 }, Position { x: 18, y: 10 }]
        );
        assert_eq!(
            (
                result.tick(),
                result.active_ticks(),
                result.revision(),
                result.last_sequence()
            ),
            (8, 5, 8, 7)
        );
    }

    #[test]
    fn input_permutations_have_canonical_sequence_order() {
        let batch = [
            hold(1, 1, Player::One, Direction::East),
            input(1, 2, Command::Pause),
            input(1, 3, Command::Resume),
            hold(1, 4, Player::One, Direction::South),
        ];
        let mut reverse = batch;
        reverse.reverse();
        let mut rotated = batch;
        rotated.rotate_left(2);
        let expected = Simulation::new(7).unwrap().step(1, &batch).unwrap();
        for commands in [&reverse, &rotated] {
            assert_eq!(
                Simulation::new(7).unwrap().step(1, commands).unwrap(),
                expected
            );
        }
        assert_eq!(expected.positions()[0], Position { x: 2, y: 3 });
    }

    #[test]
    fn pause_discards_held_inputs_and_resume_has_no_catch_up() {
        let mut runner = HeadlessRunner::new(7).unwrap();
        runner
            .tick(&[hold(1, 1, Player::One, Direction::East)])
            .unwrap();
        let paused = runner.tick(&[input(2, 2, Command::Pause)]).unwrap();
        for tick in 3..=102 {
            let commands = if tick == 3 {
                vec![hold(tick, 3, Player::One, Direction::West)]
            } else {
                vec![]
            };
            let next = runner.tick(&commands).unwrap();
            assert_eq!(next.positions(), paused.positions());
            assert_eq!(next.active_ticks(), 1);
            assert_eq!(next.held(), [Direction::Still; 2]);
        }
        let resumed = runner.tick(&[input(103, 4, Command::Resume)]).unwrap();
        assert_eq!(resumed.positions(), paused.positions());
        assert_eq!(resumed.active_ticks(), 2);
        assert!(!resumed.paused());
        runner
            .tick(&[hold(104, 5, Player::One, Direction::South)])
            .unwrap();
        assert_eq!(runner.snapshot().positions()[0], Position { x: 3, y: 3 });
    }

    #[test]
    fn release_and_repeated_resume_have_defined_semantics() {
        let mut sim = Simulation::new(7).unwrap();
        sim.step(1, &[hold(1, 1, Player::One, Direction::East)])
            .unwrap();
        sim.step(2, &[input(2, 2, Command::Resume)]).unwrap();
        assert_eq!(sim.snapshot().positions()[0].x, 4);
        sim.step(3, &[hold(3, 3, Player::One, Direction::Still)])
            .unwrap();
        sim.step(4, &[]).unwrap();
        assert_eq!(sim.snapshot().positions()[0].x, 4);
    }

    #[test]
    fn both_full_squares_stay_within_all_four_arena_edges() {
        for (direction, axis, expected) in [
            (Direction::West, 0, 0),
            (Direction::East, 0, ARENA_WIDTH - SQUARE_SIZE),
            (Direction::North, 1, 0),
            (Direction::South, 1, ARENA_HEIGHT - SQUARE_SIZE),
        ] {
            let mut sim = Simulation::new(7).unwrap();
            sim.step(
                1,
                &[
                    hold(1, 1, Player::One, direction),
                    hold(1, 2, Player::Two, direction),
                ],
            )
            .unwrap();
            for tick in 2..=40 {
                sim.step(tick, &[]).unwrap();
            }
            for position in sim.snapshot().positions() {
                assert!((0..=ARENA_WIDTH - SQUARE_SIZE).contains(&position.x));
                assert!((0..=ARENA_HEIGHT - SQUARE_SIZE).contains(&position.y));
                assert_eq!(if axis == 0 { position.x } else { position.y }, expected);
            }
        }
    }

    #[test]
    fn rejected_envelope_never_partially_applies_an_earlier_command() {
        let mut sim = Simulation::new(7).unwrap();
        let valid = hold(1, 1, Player::One, Direction::East);
        let second = input(1, 2, Command::Pause);
        assert_rejected(
            &mut sim,
            1,
            &[
                valid,
                Input {
                    generation: 9,
                    ..second
                },
            ],
            Error::WrongGeneration,
        );
        assert_rejected(
            &mut sim,
            1,
            &[valid, Input { tick: 2, ..second }],
            Error::WrongTick,
        );
        assert_rejected(&mut sim, 1, &[valid, valid], Error::SequenceMismatch);
        assert_rejected(
            &mut sim,
            1,
            &[
                valid,
                Input {
                    sequence: 3,
                    ..second
                },
            ],
            Error::SequenceMismatch,
        );
        sim.step(1, &[valid]).unwrap();
        assert_rejected(
            &mut sim,
            2,
            &[Input { tick: 2, ..valid }],
            Error::SequenceMismatch,
        );
        sim.step(2, &[input(2, 2, Command::Pause)]).unwrap();
        assert_eq!(sim.snapshot().positions()[0].x, 3);
    }

    #[test]
    fn skipped_repeated_and_wrong_input_ticks_are_rejected() {
        let mut sim = Simulation::new(7).unwrap();
        assert_rejected(&mut sim, 0, &[], Error::WrongTick);
        assert_rejected(&mut sim, 2, &[], Error::WrongTick);
        assert_rejected(
            &mut sim,
            1,
            &[input(0, 1, Command::Pause)],
            Error::WrongTick,
        );
        sim.step(1, &[]).unwrap();
        assert_rejected(&mut sim, 1, &[], Error::WrongTick);
    }

    #[test]
    fn bounded_batch_accepts_limit_and_rejects_one_more_atomically() {
        let mut sim = Simulation::new(7).unwrap();
        let commands: Vec<_> = (1..=MAX_COMMANDS_PER_TICK + 1)
            .map(|sequence| input(1, u64::try_from(sequence).unwrap(), Command::Pause))
            .collect();
        assert_rejected(&mut sim, 1, &commands, Error::TooManyCommands);
        let result = sim.step(1, &commands[..MAX_COMMANDS_PER_TICK]).unwrap();
        assert_eq!(
            result.last_sequence(),
            u64::try_from(MAX_COMMANDS_PER_TICK).unwrap()
        );
        assert_eq!(result.active_ticks(), 0);
    }

    #[test]
    fn publications_are_immutable_and_read_cadence_has_no_effect() {
        let mut observed = HeadlessRunner::new(7).unwrap();
        let mut quiet = HeadlessRunner::new(7).unwrap();
        let initial = observed.snapshot();
        for frame in replay_frames() {
            let publication = observed.tick(&frame).unwrap();
            for _ in 0..19 {
                assert!(Rc::ptr_eq(&publication, &observed.snapshot()));
            }
            assert_eq!(publication, quiet.tick(&frame).unwrap());
        }
        assert_eq!(initial.tick(), 0);
        assert_eq!(initial.positions()[0], Position { x: 2, y: 2 });
        assert!(!Rc::ptr_eq(&initial, &observed.snapshot()));
    }

    // Near-exhaustion state is a private test fixture, never a public restore API.
    #[test]
    fn checked_control_tick_and_revision_overflow_preserve_publication() {
        let mut sim = Simulation::new(7).unwrap();
        Rc::get_mut(&mut sim.current).unwrap().tick = u64::MAX - 1;
        sim.step(u64::MAX, &[]).unwrap();
        assert_rejected(&mut sim, 0, &[], Error::TickOverflow);
        let mut sim = Simulation::new(7).unwrap();
        Rc::get_mut(&mut sim.current).unwrap().revision = u64::MAX - 1;
        sim.step(1, &[]).unwrap();
        assert_rejected(
            &mut sim,
            2,
            &[hold(2, 1, Player::One, Direction::East)],
            Error::RevisionOverflow,
        );
    }

    #[test]
    fn checked_active_tick_overflow_does_not_consume_resume() {
        let mut sim = Simulation::new(7).unwrap();
        Rc::get_mut(&mut sim.current).unwrap().active_ticks = u64::MAX - 1;
        sim.step(1, &[]).unwrap();
        sim.step(2, &[input(2, 1, Command::Pause)]).unwrap();
        assert_eq!(sim.snapshot().active_ticks(), u64::MAX);
        assert_rejected(
            &mut sim,
            3,
            &[input(3, 2, Command::Resume)],
            Error::ActiveTickOverflow,
        );
        assert!(sim.snapshot().paused());
        assert_eq!(sim.snapshot().last_sequence(), 1);
    }

    #[test]
    fn checked_sequence_exhaustion_never_wraps_but_empty_ticks_work() {
        let mut sim = Simulation::new(7).unwrap();
        Rc::get_mut(&mut sim.current).unwrap().last_sequence = u64::MAX - 1;
        sim.step(1, &[input(1, u64::MAX, Command::Pause)]).unwrap();
        sim.step(2, &[]).unwrap();
        assert_rejected(
            &mut sim,
            3,
            &[input(3, 0, Command::Resume)],
            Error::SequenceOverflow,
        );
        assert_eq!(sim.snapshot().last_sequence(), u64::MAX);
    }
}
