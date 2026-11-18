//! C2 / YT-269: private native GameHost input, clock and scene projection.
//! No Yetlet SDK API, universal Renderer trait, wall-clock reducer or transport.

use std::{
    rc::Rc,
    time::{Duration, Instant},
};
use winit::keyboard::KeyCode;
use yetlet_composable_host_research::simulation::{
    ARENA_HEIGHT, ARENA_WIDTH, Command, Direction, Error as SimulationError, Input,
    MAX_COMMANDS_PER_TICK, Player, SQUARE_SIZE, Simulation, Snapshot, TICKS_PER_SECOND,
};

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
pub enum InputError {
    QueueFull,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    Queued,
    Unchanged,
    ExitRequested,
}

fn directions(keys: &[bool; 8]) -> [Direction; 2] {
    // Fixed priority for conflicting physical keys. No wall clock ordering.
    fn choose(n: bool, s: bool, w: bool, e: bool) -> Direction {
        if n {
            Direction::North
        } else if s {
            Direction::South
        } else if w {
            Direction::West
        } else if e {
            Direction::East
        } else {
            Direction::Still
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

pub struct GameDriver {
    owner: Simulation,
    pressed: [bool; 8],
    pending: Vec<Command>,
}
impl GameDriver {
    pub fn new(generation: u64) -> Result<Self, SimulationError> {
        Ok(Self {
            owner: Simulation::new(generation)?,
            pressed: [false; 8],
            pending: Vec::new(),
        })
    }

    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.owner.snapshot()
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Project only control-state transitions already admitted for this tick.
    /// The published C1 Snapshot remains the sole authoritative state.
    fn paused_after_pending(&self) -> bool {
        self.pending.iter().fold(
            self.owner.snapshot().paused(),
            |paused, command| match command {
                Command::Pause => true,
                Command::Resume => false,
                Command::Hold(_, _) => paused,
            },
        )
    }

    fn admit(&mut self, command: Command) -> Result<(), InputError> {
        if self.pending.len() == MAX_COMMANDS_PER_TICK {
            return Err(InputError::QueueFull);
        }
        self.pending.push(command);
        Ok(())
    }

    pub fn key(&mut self, key: Key, pressed: bool, repeat: bool) -> Result<KeyResult, InputError> {
        if repeat {
            return Ok(KeyResult::Unchanged);
        }
        match key {
            Key::Exit if pressed => Ok(KeyResult::ExitRequested),
            Key::Pause if pressed => {
                self.admit(Command::Pause)?;
                self.pressed = [false; 8]; // held input must not survive pause
                Ok(KeyResult::Queued)
            }
            Key::Resume if pressed => {
                let was_paused = self.paused_after_pending();
                self.admit(Command::Resume)?;
                if was_paused {
                    // Only an effective Paused -> Running transition clears keys.
                    // A queued second Resume is a no-op even when the last
                    // *published* Snapshot is still paused.
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
                if self.pressed == future {
                    return Ok(KeyResult::Unchanged);
                }
                let prior = directions(&self.pressed);
                let next = directions(&future);
                let changed: Vec<_> = (0..2).filter(|&i| prior[i] != next[i]).collect();
                if self.pending.len() + changed.len() > MAX_COMMANDS_PER_TICK {
                    return Err(InputError::QueueFull);
                }
                for index in changed {
                    self.pending.push(Command::Hold(
                        if index == 0 { Player::One } else { Player::Two },
                        next[index],
                    ));
                }
                self.pressed = future;
                Ok(KeyResult::Queued)
            }
        }
    }

    pub fn lost_focus(&mut self) -> Result<(), InputError> {
        let old = directions(&self.pressed);
        let mut commands = Vec::new();
        for (i, dir) in old.into_iter().enumerate() {
            if dir != Direction::Still {
                commands.push(Command::Hold(
                    if i == 0 { Player::One } else { Player::Two },
                    Direction::Still,
                ));
            }
        }
        if self.pending.len() + commands.len() > MAX_COMMANDS_PER_TICK {
            return Err(InputError::QueueFull);
        }
        self.pending.extend(commands);
        self.pressed = [false; 8];
        Ok(())
    }

    /// The only C1 state-advance invocation. Failed steps leave all commands
    /// pending and do not publish another Snapshot.
    pub fn step(&mut self) -> Result<Rc<Snapshot>, SimulationError> {
        let before = self.owner.snapshot();
        let tick = self.owner.next_tick()?;
        let mut sequence = before.last_sequence();
        let mut stamped = Vec::with_capacity(self.pending.len());
        for &command in &self.pending {
            sequence = sequence
                .checked_add(1)
                .ok_or(SimulationError::SequenceOverflow)?;
            stamped.push(Input {
                generation: before.generation(),
                tick,
                sequence,
                command,
            });
        }
        let fresh = self.owner.step(tick, &stamped)?;
        self.pending.clear();
        Ok(fresh)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub center_extent: [f32; 4],
    pub color: [f32; 4],
}
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub generation: u64,
    pub tick: u64,
    pub revision: u64,
    pub rects: [Rect; 2],
}
/// Immutable projection: no Simulation handle or mutable domain alias.
pub fn project(snapshot: &Snapshot) -> Scene {
    let positions = snapshot.positions();
    let colors = [[0.24, 0.65, 1.0, 1.0], [1.0, 0.57, 0.26, 1.0]];
    let rects = std::array::from_fn(|i| {
        let x = positions[i].x as f32;
        let y = positions[i].y as f32;
        let width = ARENA_WIDTH as f32;
        let height = ARENA_HEIGHT as f32;
        let size = SQUARE_SIZE as f32;
        Rect {
            center_extent: [
                (2.0 * x + size) / width - 1.0,
                1.0 - (2.0 * y + size) / height,
                size / width,
                size / height,
            ],
            color: colors[i],
        }
    });
    Scene {
        generation: snapshot.generation(),
        tick: snapshot.tick(),
        revision: snapshot.revision(),
        rects,
    }
}

/// Encode two 8-float instance records without adding another dependency.
pub fn scene_bytes(scene: &Scene) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    for (i, rect) in scene.rects.iter().enumerate() {
        for (j, value) in rect.center_extent.into_iter().chain(rect.color).enumerate() {
            bytes[(i * 8 + j) * 4..(i * 8 + j + 1) * 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

pub const FIXED_STEP: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);
pub const RENDER_STEP: Duration = Duration::from_nanos(1_000_000_000 / 30);
pub const MAX_ELAPSED: Duration = Duration::from_millis(100);
pub const MAX_TICKS_PER_WAKE: u32 = 4;

/// Draw cadence does not wake a fully obscured native window. Keeping an
/// expired draw deadline while occluded would otherwise busy-spin the loop.
/// The simulation tick deadline is still honored while rendering is hidden.
pub fn c3_foreground_wake_delay(
    occluded: bool,
    next_tick: Duration,
    next_draw: Duration,
) -> Duration {
    if occluded {
        next_tick
    } else {
        next_tick.min(next_draw)
    }
}

/// Monotonic foreground scheduler. Neither rendering nor window events
/// mutate the game clock. C3 owns stress/performance qualification.
pub struct TickClock {
    last: Instant,
    debt: Duration,
    dropped_ns: u128,
}
impl TickClock {
    pub fn new(now: Instant) -> Self {
        Self {
            last: now,
            debt: Duration::ZERO,
            dropped_ns: 0,
        }
    }
    pub fn advance(&mut self, now: Instant) -> u32 {
        let elapsed = now.saturating_duration_since(self.last);
        self.last = now;
        if elapsed > MAX_ELAPSED {
            self.dropped_ns += (elapsed - MAX_ELAPSED).as_nanos();
        }
        self.debt += elapsed.min(MAX_ELAPSED);
        let due = (self.debt.as_nanos() / FIXED_STEP.as_nanos()) as u32;
        let ticks = due.min(MAX_TICKS_PER_WAKE);
        if due > ticks {
            self.dropped_ns += ((due - ticks) as u128) * FIXED_STEP.as_nanos();
            self.debt = Duration::ZERO;
        } else {
            self.debt -= FIXED_STEP * ticks;
        }
        ticks
    }
    pub fn next_tick_in(&self) -> Duration {
        FIXED_STEP.saturating_sub(self.debt)
    }
    pub fn dropped_ns(&self) -> u128 {
        self.dropped_ns
    }
    pub fn reset(&mut self, now: Instant) {
        self.last = now;
        self.debt = Duration::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn advance(driver: &mut GameDriver, n: usize) {
        for _ in 0..n {
            driver.step().unwrap();
        }
    }
    #[test]
    fn physical_keyboard_map_has_disjoint_player_and_control_keys() {
        assert_eq!(map_key(KeyCode::KeyW), Some(Key::W));
        assert_eq!(map_key(KeyCode::ArrowRight), Some(Key::Right));
        assert_eq!(map_key(KeyCode::Space), Some(Key::Pause));
        assert_eq!(map_key(KeyCode::Enter), Some(Key::Resume));
        assert_eq!(map_key(KeyCode::Escape), Some(Key::Exit));
        assert_eq!(map_key(KeyCode::KeyZ), None);
    }
    #[test]
    fn both_players_movement_and_key_release_use_only_single_owner() {
        let mut driver = GameDriver::new(5).unwrap();
        driver.key(Key::D, true, false).unwrap();
        driver.key(Key::Up, true, false).unwrap();
        let before = driver.snapshot();
        let after = driver.step().unwrap();
        assert_eq!(after.positions()[0].x, before.positions()[0].x + 1);
        assert_eq!(after.positions()[1].y, before.positions()[1].y - 1);
        advance(&mut driver, 2);
        driver.key(Key::D, false, false).unwrap();
        driver.key(Key::Up, false, false).unwrap();
        let released = driver.step().unwrap();
        advance(&mut driver, 3);
        assert_eq!(driver.snapshot().positions(), released.positions());
    }
    #[test]
    fn pause_resume_releases_held_and_prevents_catchup() {
        let mut d = GameDriver::new(7).unwrap();
        d.key(Key::W, true, false).unwrap();
        advance(&mut d, 2);
        d.key(Key::Pause, true, false).unwrap();
        advance(&mut d, 100);
        let stopped = d.snapshot();
        assert!(stopped.paused());
        assert_eq!(stopped.active_ticks(), 2);
        d.key(Key::Resume, true, false).unwrap();
        advance(&mut d, 1);
        let resumed = d.snapshot();
        assert!(!resumed.paused());
        advance(&mut d, 3);
        assert_eq!(resumed.positions(), d.snapshot().positions());
        assert_eq!(d.snapshot().active_ticks(), 6);
    }

    #[test]
    fn resume_while_running_preserves_movement_release_and_focus_loss() {
        // C1 treats Resume while already Running as a no-op, so the native
        // pressed-key record must keep tracking held movement until release.
        let mut d = GameDriver::new(7).unwrap();
        d.key(Key::D, true, false).unwrap();
        assert_eq!(d.step().unwrap().held()[0], Direction::East);
        d.key(Key::Resume, true, false).unwrap();
        assert_eq!(d.step().unwrap().held()[0], Direction::East);
        d.key(Key::D, false, false).unwrap();
        let released = d.step().unwrap();
        assert_eq!(released.held()[0], Direction::Still);
        let position = released.positions();
        advance(&mut d, 4);
        assert_eq!(d.snapshot().positions(), position);

        // Losing focus after an otherwise no-op Resume must also release
        // tracked movement rather than allowing it to continue forever.
        let mut lost_focus = GameDriver::new(8).unwrap();
        lost_focus.key(Key::D, true, false).unwrap();
        lost_focus.step().unwrap();
        lost_focus.key(Key::Resume, true, false).unwrap();
        lost_focus.step().unwrap();
        lost_focus.lost_focus().unwrap();
        let stopped = lost_focus.step().unwrap();
        assert_eq!(stopped.held()[0], Direction::Still);
        let stopped_at = stopped.positions();
        advance(&mut lost_focus, 5);
        assert_eq!(lost_focus.snapshot().positions(), stopped_at);
    }

    #[test]
    fn queued_resume_pause_projection_keeps_key_release_and_focus_loss_effective() {
        // All three commands are queued for one future tick while the last
        // published Snapshot still says paused. The second Resume is a
        // reducer no-op and must not erase the D key that precedes it.
        for focus_loss in [false, true] {
            let mut driver = GameDriver::new(if focus_loss { 12 } else { 11 }).unwrap();
            driver.key(Key::Pause, true, false).unwrap();
            let paused = driver.step().unwrap();
            assert!(paused.paused());

            driver.key(Key::Resume, true, false).unwrap();
            driver.key(Key::Resume, false, false).unwrap();
            driver.key(Key::D, true, false).unwrap();
            driver.key(Key::Resume, true, false).unwrap();
            assert_eq!(driver.pending_count(), 3);
            assert!(driver.snapshot().paused()); // pending != published

            let resumed = driver.step().unwrap();
            assert!(!resumed.paused());
            assert_eq!(resumed.held()[0], Direction::East);
            assert_eq!(resumed.positions()[0].x, paused.positions()[0].x + 1);

            if focus_loss {
                driver.lost_focus().unwrap();
            } else {
                driver.key(Key::D, false, false).unwrap();
            }
            assert_eq!(driver.pending_count(), 1);
            let stopped = driver.step().unwrap();
            assert_eq!(stopped.held()[0], Direction::Still);
            let stopped_at = stopped.positions();
            advance(&mut driver, 6);
            assert_eq!(driver.snapshot().positions(), stopped_at);
            assert_eq!(driver.snapshot().held()[0], Direction::Still);
        }
    }
    #[test]
    fn conflicting_keys_have_stable_priority_and_repeat_is_ignored() {
        let mut d = GameDriver::new(1).unwrap();
        assert_eq!(d.key(Key::W, true, false).unwrap(), KeyResult::Queued);
        assert_eq!(d.key(Key::W, true, true).unwrap(), KeyResult::Unchanged);
        assert_eq!(d.key(Key::S, true, false).unwrap(), KeyResult::Queued);
        assert_eq!(d.pending_count(), 1);
        let first = d.step().unwrap();
        assert_eq!(first.held()[0], Direction::North);
        d.key(Key::W, false, false).unwrap();
        assert_eq!(d.step().unwrap().held()[0], Direction::South);
    }
    #[test]
    fn lost_focus_releases_both_players_and_does_not_clone_authority() {
        let mut d = GameDriver::new(1).unwrap();
        d.key(Key::D, true, false).unwrap();
        d.key(Key::Right, true, false).unwrap();
        d.step().unwrap();
        d.lost_focus().unwrap();
        let s = d.step().unwrap();
        assert_eq!(s.held(), [Direction::Still; 2]);
        assert_eq!(s.positions(), d.snapshot().positions());
    }
    #[test]
    fn render_projection_never_steps_or_mutates_original_snapshot() {
        let mut d = GameDriver::new(2).unwrap();
        let old = d.snapshot();
        let scene = project(&old);
        let bytes = scene_bytes(&scene);
        assert_eq!(bytes.len(), 64);
        for _ in 0..100 {
            assert_eq!(scene, project(&d.snapshot()));
        }
        d.key(Key::A, true, false).unwrap();
        let next = d.step().unwrap();
        assert_eq!(old.tick(), 0);
        assert_eq!(scene.tick, 0);
        assert_eq!(project(&next).tick, 1);
    }
    #[test]
    fn rendering_every_tick_vs_every_fourth_tick_is_identical() {
        let mut a = GameDriver::new(1).unwrap();
        let mut b = GameDriver::new(1).unwrap();
        a.key(Key::D, true, false).unwrap();
        b.key(Key::D, true, false).unwrap();
        for i in 0..36 {
            let sa = a.step().unwrap();
            let sb = b.step().unwrap();
            let _ = project(&sa);
            if i % 4 == 0 {
                let _ = project(&sb);
            }
        }
        assert_eq!(a.snapshot(), b.snapshot());
    }

    #[test]
    fn c3_occluded_expired_draw_deadline_waits_for_tick_and_restores_redraw() {
        let start = Instant::now();
        let mut clock = TickClock::new(start);
        // After actual due work, the next simulation tick lies in the future.
        assert_eq!(clock.advance(start + Duration::from_millis(37)), 2);
        let future_tick = clock.next_tick_in();
        assert!(future_tick > Duration::ZERO);
        assert!(future_tick <= FIXED_STEP);

        // The draw deadline is already expired, but hiding the surface must
        // NOT turn WaitUntil into a continuous immediate wakeup.
        assert_eq!(
            c3_foreground_wake_delay(true, future_tick, Duration::ZERO),
            future_tick,
        );
        // Once visible, a requested redraw may happen immediately.
        assert_eq!(
            c3_foreground_wake_delay(false, future_tick, Duration::ZERO),
            Duration::ZERO,
        );
        // After requesting redraw on restoration, resetting that deadline
        // permits bounded future waits instead of duplicate immediate wakes.
        assert_eq!(
            c3_foreground_wake_delay(false, future_tick, RENDER_STEP),
            future_tick,
        );
    }
    #[test]
    fn c3_delayed_foreground_callback_caps_catchup_and_records_time_debt() {
        let start = Instant::now();
        let mut clock = TickClock::new(start);
        let simulation = GameDriver::new(41).unwrap();
        let original = simulation.snapshot();
        let delay = Duration::from_millis(260);
        let advanced = clock.advance(start + delay);
        assert_eq!(advanced, MAX_TICKS_PER_WAKE);
        assert!(clock.dropped_ns() >= 190_000_000);
        assert_eq!(clock.next_tick_in(), FIXED_STEP);
        assert_eq!(simulation.snapshot(), original); // no implicit step
        assert_eq!(clock.advance(start + delay), 0);
        // Discarded debt does not trigger an unbounded replay on next wake.
        assert_eq!(clock.advance(start + delay + FIXED_STEP), 1);
        assert_eq!(simulation.snapshot(), original);
    }
    #[test]
    fn c3_zero_time_and_backwards_clock_observation_do_not_create_ticks() {
        let start = Instant::now();
        let mut clock = TickClock::new(start);
        assert_eq!(clock.advance(start), 0);
        assert_eq!(clock.advance(start - Duration::from_millis(1)), 0);
        assert_eq!(clock.dropped_ns(), 0);
        clock.reset(start);
        assert_eq!(clock.advance(start + Duration::from_millis(2)), 0);
        assert!(clock.next_tick_in() > Duration::ZERO);
    }
    #[test]
    fn clock_is_bounded_but_never_changes_simulation_without_step() {
        let start = Instant::now();
        let mut clock = TickClock::new(start);
        assert_eq!(clock.advance(start + Duration::from_millis(16)), 0);
        assert_eq!(clock.advance(start + Duration::from_millis(17)), 1);
        let mut d = GameDriver::new(1).unwrap();
        assert_eq!(d.snapshot().tick(), 0);
        let due = clock.advance(start + Duration::from_secs(1));
        assert_eq!(due, 4);
        assert!(clock.dropped_ns() > 0);
        assert_eq!(d.snapshot().tick(), 0);
        advance(&mut d, due as usize);
        assert_eq!(d.snapshot().tick(), 4);
        clock.reset(start + Duration::from_secs(2));
        assert_eq!(clock.advance(start + Duration::from_secs(2)), 0);
    }
    #[test]
    fn queue_saturation_returns_error_without_overwriting_pending_commands() {
        let mut d = GameDriver::new(1).unwrap();
        for _ in 0..MAX_COMMANDS_PER_TICK / 2 {
            d.key(Key::Pause, true, false).unwrap();
            d.key(Key::Resume, true, false).unwrap();
        }
        assert_eq!(d.pending_count(), MAX_COMMANDS_PER_TICK);
        assert_eq!(d.key(Key::Pause, true, false), Err(InputError::QueueFull));
        assert_eq!(d.pending_count(), MAX_COMMANDS_PER_TICK);
        let s = d.step().unwrap();
        assert_eq!(s.last_sequence(), MAX_COMMANDS_PER_TICK as u64);
    }
}
