//! Consumer-owned scheduling preparation over the existing NativeInput/Ports.
//!
//! No timer, event loop, thread, Simulation or authoritative Snapshot lives here.
//! The caller supplies monotonic Instants and explicitly pumps. A pump admits at
//! most four owed Ticks; each successful original Ports publication consumes one.
//! An error reports earlier successes and retains the failed/unperformed work.
//! This is per-Tick atomicity, not rollback of an entire multi-Tick pump.

use crate::{
    invalid,
    native_input::{Key, KeyResult, NativeInput},
    three_way::Ports,
    tick_m1::TICKS_PER_SECOND,
};
use std::{
    fmt, io,
    time::{Duration, Instant},
};

/// Match the private F1 integer-nanosecond step (16,666,666ns), not a wall timer.
pub const FIXED_STEP: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);
pub const MAX_ELAPSED: Duration = Duration::from_millis(100);
pub const MAX_TICKS_PER_WAKE: u32 = 4;

/// Copied diagnostics only. Mutating this copy cannot change the scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleState {
    pub observed_at: Instant,
    /// Admitted but unpublished Ticks, including retries; always at most four.
    pub pending_ticks: u32,
    /// Unadmitted fractional time; always less than FIXED_STEP.
    pub remainder: Duration,
    /// Lifetime overload discard, including the fraction discarded at the cap.
    pub dropped_ns: u128,
    /// Lifetime debt discarded only after a successful Pause/effective Resume.
    pub reset_ns: u128,
    /// Unadmitted time excluded by Native suspension/resumption, never owed work.
    pub lifecycle_ns: u128,
}

impl ScheduleState {
    fn sample(self, now: Instant) -> io::Result<Self> {
        let elapsed = now
            .checked_duration_since(self.observed_at)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Native clock moved backwards")
            })?;
        let capped = elapsed.min(MAX_ELAPSED);
        // Both operands are bounded regardless of the caller's elapsed time.
        let total = self.remainder + capped;
        let due = (total.as_nanos() / FIXED_STEP.as_nanos()) as u32;
        let slots = MAX_TICKS_PER_WAKE - self.pending_ticks;
        let admitted = due.min(slots);
        let remainder = if due > slots {
            Duration::ZERO
        } else {
            total - FIXED_STEP * admitted
        };
        let discarded = elapsed.as_nanos() - capped.as_nanos()
            + (total - FIXED_STEP * admitted - remainder).as_nanos();
        let dropped_ns = self
            .dropped_ns
            .checked_add(discarded)
            .ok_or_else(|| invalid("Native clock discard counter exhausted"))?;
        Ok(Self {
            observed_at: now,
            pending_ticks: self.pending_ticks + admitted,
            remainder,
            dropped_ns,
            reset_ns: self.reset_ns,
            lifecycle_ns: self.lifecycle_ns,
        })
    }

    fn reanchor(self, now: Instant) -> io::Result<Self> {
        let elapsed = now
            .checked_duration_since(self.observed_at)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Native lifecycle time moved backwards",
                )
            })?;
        let lifecycle_ns = self
            .lifecycle_ns
            .checked_add(elapsed.as_nanos())
            .and_then(|n| n.checked_add(self.remainder.as_nanos()))
            .ok_or_else(|| invalid("Native lifecycle counter exhausted"))?;
        Ok(Self {
            observed_at: now,
            remainder: Duration::ZERO,
            lifecycle_ns,
            ..self
        })
    }

    /// Preflight all fallible bookkeeping before the first original publication.
    fn after_first(mut self, reset: bool) -> io::Result<Self> {
        self.pending_ticks -= 1;
        if reset {
            let discarded = (FIXED_STEP * self.pending_ticks + self.remainder).as_nanos();
            self.reset_ns = self
                .reset_ns
                .checked_add(discarded)
                .ok_or_else(|| invalid("Native clock reset counter exhausted"))?;
            self.pending_ticks = 0;
            self.remainder = Duration::ZERO;
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PumpReport {
    pub published_ticks: u32,
}

/// A later failed Tick does not undo earlier publications in the same pump.
#[derive(Debug)]
pub struct PumpError {
    pub published_ticks: u32,
    pub error: io::Error,
}
impl PumpError {
    fn new(published_ticks: u32, error: io::Error) -> Self {
        Self {
            published_ticks,
            error,
        }
    }
}
impl fmt::Display for PumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Native pump stopped after {} published Ticks: {}",
            self.published_ticks, self.error
        )
    }
}
impl std::error::Error for PumpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// One scheduling/input adapter bound to the original Ports before its first
/// accepted action/Tick. No mutable input/timing handle, rebase or reset bypass.
pub struct NativeScheduler {
    input: NativeInput,
    clock: ScheduleState,
    suspended: bool,
}
impl NativeScheduler {
    pub fn new(ports: &Ports, now: Instant) -> io::Result<Self> {
        Ok(Self {
            input: NativeInput::new(ports)?,
            clock: ScheduleState {
                observed_at: now,
                pending_ticks: 0,
                remainder: Duration::ZERO,
                dropped_ns: 0,
                reset_ns: 0,
                lifecycle_ns: 0,
            },
            suspended: false,
        })
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Suspend the existing owner, atomically admitting required Still inputs.
    /// Keep owed Ticks and all pending commands. Discard only unadmitted elapsed
    /// time/fraction into lifecycle_ns. Queue rejection leaves everything intact.
    pub fn suspend(&mut self, ports: &Ports, now: Instant) -> io::Result<()> {
        self.input.observe(ports)?;
        if now < self.clock.observed_at {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native lifecycle time moved backwards",
            ));
        }
        if self.suspended {
            return Ok(());
        }
        let next = self.clock.reanchor(now)?;
        self.input.lost_focus(ports)?;
        // All validation, arithmetic and release admission finished above.
        self.clock = next;
        self.suspended = true;
        Ok(())
    }

    /// Reanchor only a suspended scheduler after fresh Native readiness.
    /// Preserve owed work, pending release/input commands and exact ownership.
    /// Duplicate active resume does not observe time or reset any debt.
    pub fn resume(&mut self, ports: &Ports, now: Instant) -> io::Result<()> {
        self.input.observe(ports)?;
        if now < self.clock.observed_at {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native lifecycle time moved backwards",
            ));
        }
        if !self.suspended {
            return Ok(());
        }
        let next = self.clock.reanchor(now)?;
        self.clock = next;
        self.suspended = false;
        Ok(())
    }

    fn require_active(&self, ports: &Ports) -> io::Result<()> {
        self.input.observe(ports)?;
        if self.suspended {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Native scheduler suspended",
            ));
        }
        Ok(())
    }

    pub fn timing(&self) -> ScheduleState {
        self.clock
    }
    pub fn next_tick_in(&self) -> Duration {
        if self.clock.pending_ticks != 0 {
            Duration::ZERO
        } else {
            FIXED_STEP - self.clock.remainder
        }
    }
    pub fn pending_input_count(&self) -> usize {
        self.input.pending_count()
    }
    pub fn key(
        &mut self,
        ports: &Ports,
        key: Key,
        pressed: bool,
        repeat: bool,
    ) -> io::Result<KeyResult> {
        self.require_active(ports)?;
        self.input.key(ports, key, pressed, repeat)
    }
    pub fn lost_focus(&mut self, ports: &Ports) -> io::Result<()> {
        self.require_active(ports)?;
        self.input.lost_focus(ports)
    }

    /// Sample once and publish at most four Ticks through the original input.
    ///
    /// Older admitted work has priority over new elapsed time. Clamp each
    /// observation to100ms; on overflow discard all excess including its fraction.
    /// Count every discarded nanosecond. Same-Instant retries admit no new time;
    /// backwards time, foreign/stale/closed authority and accounting overflow
    /// reject without changing scheduling/input state, even when no Tick is due.
    ///
    /// Publication failure keeps sampled time and all unperformed admitted work,
    /// with the original pending input intact. Successfully published Ticks stay
    /// committed and are counted in PumpError. No fallible bookkeeping follows a
    /// publication. Pause (including an explicit repeated Pause), or an effective
    /// Resume in the pending batch, resets remaining debt only after that control
    /// Tick succeeds. Pause then Resume in one batch still resets. Repeated Resume
    /// while running does not reset. Paused logical Ticks can publish but do not
    /// move players/increment active_ticks. Reset time is counted separately.
    pub fn pump(&mut self, ports: &Ports, now: Instant) -> Result<PumpReport, PumpError> {
        let before = self
            .input
            .observe(ports)
            .map_err(|e| PumpError::new(0, e))?;
        if self.suspended {
            return Err(PumpError::new(
                0,
                io::Error::new(io::ErrorKind::WouldBlock, "Native scheduler suspended"),
            ));
        }
        let sampled = self.clock.sample(now).map_err(|e| PumpError::new(0, e))?;
        if sampled.pending_ticks == 0 {
            self.clock = sampled;
            return Ok(PumpReport { published_ticks: 0 });
        }
        let reset = self
            .input
            .pending_controls(before.snapshot.paused)
            .reset_clock;
        let mut completed = sampled
            .after_first(reset)
            .map_err(|e| PumpError::new(0, e))?;
        // Sampling admits work; it does not consume that work or pending input.
        self.clock = sampled;
        let mut published_ticks = 0;
        loop {
            self.input
                .step_tick(ports)
                .map_err(|e| PumpError::new(published_ticks, e))?;
            self.clock = completed;
            published_ticks += 1;
            if self.clock.pending_ticks == 0 {
                return Ok(PumpReport { published_ticks });
            }
            // The first Tick consumed the entire pending batch. Subsequent empty
            // Ticks need only an infallible debit, never another control reset.
            completed = self.clock;
            completed.pending_ticks -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Action, AuditStatus, Command, TICK_EVIDENCE_CAPACITY,
        tick_m1::{HeldDirection, TickError},
    };
    use std::{rc::Rc, sync::Arc};
    use yetlet::composition::{Builder, Completion};

    fn press(s: &mut NativeScheduler, p: &Ports, key: Key) {
        assert_eq!(s.key(p, key, true, false).unwrap(), KeyResult::Queued);
    }
    fn pump(s: &mut NativeScheduler, p: &Ports, now: Instant, count: u32) {
        assert_eq!(s.pump(p, now).unwrap().published_ticks, count);
    }
    fn identity(p: &Ports) {
        let observed = p.c1().tick_observation();
        assert_eq!(observed.snapshot.revision, observed.clock.tick);
        assert_eq!(p.audit().pinned_tick(), Some(observed));
        let proof = p.pinned_tick(91, observed.clock.tick).unwrap();
        assert_eq!(proof.applied, observed);
        assert_eq!(
            p.pinned(91, observed.clock.tick).unwrap(),
            observed.snapshot
        );
        assert!(Arc::ptr_eq(
            p.audit().tick_records().last().unwrap(),
            &proof
        ));
    }

    #[test]
    fn native_lifecycle_preserves_failed_work_and_releases_once_after_resume() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        let before = p.c1().tick_observation();
        let guard = p.audit().records.borrow();
        let now = start + FIXED_STEP * 3 + Duration::from_nanos(5);
        assert_eq!(s.pump(&p, now).unwrap_err().published_ticks, 0);
        s.suspend(&p, now).unwrap();
        assert!(s.is_suspended());
        assert_eq!(s.timing().pending_ticks, 3);
        assert_eq!(s.timing().remainder, Duration::ZERO);
        assert_eq!(s.timing().lifecycle_ns, 5);
        assert_eq!(s.timing().reset_ns, 0);
        assert_eq!(s.pending_input_count(), 4);
        assert_eq!(p.c1().tick_observation(), before);
        let suspended = s.timing();
        s.suspend(&p, now + Duration::from_secs(9)).unwrap();
        assert_eq!(s.timing(), suspended);
        assert_eq!(
            s.pump(&p, now + Duration::from_secs(9))
                .unwrap_err()
                .error
                .kind(),
            io::ErrorKind::WouldBlock
        );
        assert!(s.key(&p, Key::W, true, false).is_err());
        assert!(s.lost_focus(&p).is_err());
        assert_eq!(s.timing(), suspended);
        let resumed_at = now + Duration::from_secs(10);
        s.resume(&p, resumed_at).unwrap();
        assert!(!s.is_suspended());
        assert_eq!(s.timing().pending_ticks, 3);
        assert_eq!(s.timing().lifecycle_ns, 10_000_000_005);
        assert_eq!(s.pending_input_count(), 4);
        assert_eq!(s.pump(&p, resumed_at).unwrap_err().published_ticks, 0);
        assert_eq!(s.timing().pending_ticks, 3);
        drop(guard);
        pump(&mut s, &p, resumed_at, 3);
        assert_eq!(p.c1().snapshot().positions, before.snapshot.positions);
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 4);
        assert_eq!(
            p.c1().tick_observation().clock.held,
            [HeldDirection::Still; 2]
        );
        assert_eq!(p.pinned_tick(91, 1).unwrap().inputs.len(), 4);
        assert!(p.pinned_tick(91, 2).unwrap().inputs.is_empty());
        let timing = s.timing();
        s.resume(&p, resumed_at + FIXED_STEP).unwrap();
        assert_eq!(s.timing(), timing); // duplicate active notification is inert
        pump(&mut s, &p, resumed_at + FIXED_STEP * 2, 2);
        assert_eq!(p.c1().tick_observation().clock.tick, 5);
        identity(&p);
    }

    #[test]
    fn native_lifecycle_full_release_queue_rejects_before_timing_or_key_changes() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        for _ in 0..29 {
            press(&mut s, &p, Key::Resume);
        }
        assert_eq!(s.pending_input_count(), 31);
        let before = s.timing();
        assert_eq!(
            s.suspend(&p, start + FIXED_STEP).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert!(!s.is_suspended());
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 31);
        for k in [Key::D, Key::Up] {
            assert_eq!(s.key(&p, k, true, false).unwrap(), KeyResult::Unchanged);
        }
        pump(&mut s, &p, start + FIXED_STEP, 1);
        let moved = p.c1().snapshot();
        s.suspend(&p, start + FIXED_STEP).unwrap();
        assert_eq!(s.pending_input_count(), 2);
        s.resume(&p, start + Duration::from_secs(1)).unwrap();
        pump(&mut s, &p, start + Duration::from_secs(1) + FIXED_STEP, 1);
        assert_eq!(p.c1().snapshot().positions, moved.positions);
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 33);
        identity(&p);
    }

    #[test]
    fn native_lifecycle_backwards_and_overflow_preflight_before_release_admission() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        let before = s.timing();
        assert_eq!(
            s.suspend(&p, start - Duration::from_nanos(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 2);
        // Private accounting-boundary fixture, no product mutation seam.
        s.clock.lifecycle_ns = u128::MAX;
        let before = s.timing();
        assert!(s.suspend(&p, start + Duration::from_nanos(1)).is_err());
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 2);
        assert!(!s.is_suspended());
        assert_eq!(p.c1().snapshot().revision, 0);
        s.suspend(&p, start).unwrap();
        let inactive = s.timing();
        assert_eq!(s.pending_input_count(), 4);
        assert!(s.resume(&p, start + Duration::from_nanos(1)).is_err());
        assert!(s.resume(&p, start - Duration::from_nanos(1)).is_err());
        assert!(s.is_suspended());
        assert_eq!(s.timing(), inactive);
        assert_eq!(s.pending_input_count(), 4);
    }

    #[test]
    fn native_lifecycle_inactive_owner_stale_and_close_fences_remain_exact() {
        for stale in [false, true] {
            let p = Ports::new(91).unwrap();
            let foreign = Ports::new(91).unwrap();
            let now = Instant::now();
            let mut s = NativeScheduler::new(&p, now).unwrap();
            press(&mut s, &p, Key::D);
            s.suspend(&p, now).unwrap();
            let before = s.timing();
            assert!(s.resume(&foreign, now).is_err());
            assert!(s.suspend(&foreign, now).is_err());
            assert!(s.pump(&foreign, now).is_err());
            if stale {
                p.step_tick(1, &[]).unwrap();
            } else {
                p.ingress().close().unwrap();
            }
            let observed = p.c1().tick_observation();
            assert!(s.resume(&p, now).is_err());
            assert!(s.suspend(&p, now).is_err());
            assert!(s.pump(&p, now).is_err());
            assert!(s.is_suspended());
            assert_eq!(s.timing(), before);
            assert_eq!(s.pending_input_count(), 2);
            assert_eq!(p.c1().tick_observation(), observed);
        }
    }

    #[test]
    fn native_lifecycle_resume_does_not_resume_paused_game_or_clear_control_identity() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::Pause);
        pump(&mut s, &p, start + FIXED_STEP * 3, 1);
        let pause = p.c1().tick_observation();
        let reset = s.timing().reset_ns;
        s.suspend(&p, start + FIXED_STEP * 3 + Duration::from_nanos(10))
            .unwrap();
        let resumed = start + FIXED_STEP * 3 + Duration::from_secs(5);
        s.resume(&p, resumed).unwrap();
        pump(&mut s, &p, resumed, 0);
        pump(&mut s, &p, resumed + FIXED_STEP, 1);
        assert!(p.c1().snapshot().paused);
        assert_eq!(p.c1().snapshot().positions, pause.snapshot.positions);
        assert_eq!(p.c1().tick_observation().clock.active_ticks, 0);
        assert_eq!(s.timing().reset_ns, reset);
        assert_eq!(s.timing().lifecycle_ns, 5_000_000_000);
        press(&mut s, &p, Key::Resume);
        press(&mut s, &p, Key::D);
        pump(&mut s, &p, resumed + FIXED_STEP * 2, 1);
        assert_eq!(p.c1().tick_observation().clock.active_ticks, 1);
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 3);
        assert_eq!(p.c1().snapshot().positions[0].x, 3);
        identity(&p);
    }

    #[test]
    fn fixed_thresholds_keep_fraction_and_stamp_only_first_batch() {
        assert_eq!(FIXED_STEP, Duration::from_nanos(16_666_666));
        assert_eq!(MAX_ELAPSED, Duration::from_millis(100));
        assert_eq!(MAX_TICKS_PER_WAKE, 4);
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        pump(&mut s, &p, start + FIXED_STEP - Duration::from_nanos(1), 0);
        assert_eq!(s.pending_input_count(), 2);
        assert_eq!(s.next_tick_in(), Duration::from_nanos(1));
        assert_eq!(p.c1().snapshot().revision, 0);
        pump(&mut s, &p, start + FIXED_STEP, 1);
        pump(&mut s, &p, start + FIXED_STEP, 0);
        let first = p.pinned_tick(91, 1).unwrap();
        assert_eq!(first.inputs.len(), 2);
        assert_eq!((first.inputs[0].tick, first.inputs[0].sequence), (1, 1));
        assert_eq!((first.inputs[1].tick, first.inputs[1].sequence), (1, 2));
        pump(&mut s, &p, start + Duration::from_millis(37), 1);
        assert_eq!(s.timing().remainder, Duration::from_nanos(3_666_668));
        assert_eq!(s.next_tick_in(), Duration::from_nanos(12_999_998));
        assert_eq!(s.pending_input_count(), 0);
        assert!(p.pinned_tick(91, 2).unwrap().inputs.is_empty());
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 2);
        assert_eq!(p.c1().snapshot().text(), "91 2 false 4 2 20 10");
        identity(&p);
    }

    #[test]
    fn overload_discards_exact_excess_and_never_double_counts_same_time() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        let now = start + Duration::from_millis(260);
        pump(&mut s, &p, now, 4);
        let timing = s.timing();
        assert_eq!(timing.dropped_ns, 193_333_336);
        assert_eq!(timing.remainder, Duration::ZERO);
        assert_eq!(timing.pending_ticks, 0);
        assert_eq!(timing.reset_ns, 0);
        pump(&mut s, &p, now, 0);
        assert_eq!(s.timing(), timing);
        pump(&mut s, &p, now + FIXED_STEP, 1);
        assert_eq!(s.timing().dropped_ns, timing.dropped_ns);
        assert_eq!(p.c1().tick_observation().clock.tick, 5);
        identity(&p);
    }

    #[test]
    fn backwards_observation_rejects_without_moving_watermark_or_input() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::W);
        pump(&mut s, &p, start + Duration::from_millis(10), 0);
        let before = s.timing();
        let err = s.pump(&p, start + Duration::from_millis(9)).unwrap_err();
        assert_eq!(err.error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(err.published_ticks, 0);
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 1);
        assert_eq!(p.c1().snapshot().revision, 0);
        pump(&mut s, &p, start + FIXED_STEP, 1);
        assert_eq!(s.timing().remainder, Duration::ZERO);
        assert_eq!(s.timing().dropped_ns, 0);
        identity(&p);
    }

    #[test]
    fn audit_rejection_keeps_owed_ticks_fraction_and_input_for_same_time_retry() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        let before = p.c1().tick_observation();
        let guard = p.audit().records.borrow();
        let now = start + FIXED_STEP * 3 + Duration::from_nanos(5);
        let err = s.pump(&p, now).unwrap_err();
        assert_eq!(err.published_ticks, 0);
        assert_eq!(s.timing().pending_ticks, 3);
        assert_eq!(s.timing().remainder, Duration::from_nanos(5));
        assert_eq!(s.next_tick_in(), Duration::ZERO);
        assert_eq!(s.pending_input_count(), 2);
        assert_eq!(p.c1().tick_observation(), before);
        assert!(p.audit().pinned_tick().is_none());
        assert!(p.pinned_tick(91, 1).is_err());
        drop(guard);
        pump(&mut s, &p, now, 3);
        assert_eq!(s.timing().pending_ticks, 0);
        assert_eq!(s.timing().remainder, Duration::from_nanos(5));
        assert_eq!(s.pending_input_count(), 0);
        for tick in 1..=3 {
            let proof = p.pinned_tick(91, tick).unwrap();
            assert_eq!(proof.inputs.len(), if tick == 1 { 2 } else { 0 });
            assert_eq!(proof.applied.clock.last_sequence, 2);
        }
        identity(&p);
    }

    #[test]
    fn new_elapsed_overload_preserves_old_failed_work_with_bounded_debt() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::D);
        let guard = p.audit().records.borrow();
        let first = start + FIXED_STEP * 3 + Duration::from_nanos(5);
        assert_eq!(s.pump(&p, first).unwrap_err().published_ticks, 0);
        let later = first + Duration::from_millis(260);
        assert_eq!(s.pump(&p, later).unwrap_err().published_ticks, 0);
        let admitted = s.timing();
        assert_eq!(admitted.pending_ticks, 4);
        assert_eq!(admitted.remainder, Duration::ZERO);
        assert_eq!(admitted.dropped_ns, 243_333_339);
        assert_eq!(s.pump(&p, later).unwrap_err().published_ticks, 0);
        assert_eq!(s.timing(), admitted);
        assert_eq!(s.pending_input_count(), 1);
        drop(guard);
        pump(&mut s, &p, later, 4);
        assert_eq!(p.c1().tick_observation().clock.tick, 4);
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 1);
        assert_eq!(
            later.duration_since(start).as_nanos(),
            FIXED_STEP.as_nanos() * 4 + s.timing().dropped_ns
        );
        identity(&p);
    }

    #[test]
    fn successful_pause_resume_resets_debt_without_catchup_movement() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::D);
        pump(&mut s, &p, start + FIXED_STEP, 1);
        press(&mut s, &p, Key::Pause);
        let pause_at = start + FIXED_STEP * 5 + Duration::from_nanos(7);
        pump(&mut s, &p, pause_at, 1);
        let paused = p.c1().tick_observation();
        assert!(paused.snapshot.paused);
        assert_eq!((paused.clock.tick, paused.clock.active_ticks), (2, 1));
        assert_eq!(paused.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(s.timing().reset_ns, FIXED_STEP.as_nanos() * 3 + 7);
        assert_eq!(s.next_tick_in(), FIXED_STEP);
        let paused_at = pause_at + FIXED_STEP * 2;
        pump(&mut s, &p, paused_at, 2);
        assert_eq!(p.c1().tick_observation().clock.active_ticks, 1);
        assert_eq!(p.c1().snapshot().positions, paused.snapshot.positions);
        press(&mut s, &p, Key::Resume);
        press(&mut s, &p, Key::D);
        let resume_at = paused_at + FIXED_STEP * 4 + Duration::from_nanos(9);
        pump(&mut s, &p, resume_at, 1);
        let resumed = p.c1().tick_observation();
        assert_eq!((resumed.clock.tick, resumed.clock.active_ticks), (5, 2));
        assert!(!resumed.snapshot.paused);
        assert_eq!(
            resumed.snapshot.positions[0].x,
            paused.snapshot.positions[0].x + 1
        );
        assert_eq!(s.timing().reset_ns, FIXED_STEP.as_nanos() * 6 + 16);
        assert_eq!(s.timing().pending_ticks, 0);
        assert_eq!(s.timing().remainder, Duration::ZERO);
        assert_eq!(
            resume_at.duration_since(start).as_nanos(),
            FIXED_STEP.as_nanos() * 5 + s.timing().reset_ns
        );
        identity(&p);
    }

    #[test]
    fn pause_then_resume_in_one_batch_still_resets_after_one_publication() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::Pause, Key::Resume, Key::D] {
            press(&mut s, &p, k);
        }
        pump(&mut s, &p, start + FIXED_STEP * 4, 1);
        assert_eq!(s.timing().reset_ns, FIXED_STEP.as_nanos() * 3);
        assert_eq!(s.pending_input_count(), 0);
        assert!(!p.c1().snapshot().paused);
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 3);
        assert_eq!(p.pinned_tick(91, 1).unwrap().inputs.len(), 3);
        assert_eq!(p.c1().snapshot().positions[0].x, 3);
        identity(&p);
    }

    #[test]
    fn running_resume_repeat_release_and_focus_keep_existing_input_policy() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::D);
        press(&mut s, &p, Key::Resume);
        assert_eq!(
            s.key(&p, Key::Pause, true, true).unwrap(),
            KeyResult::Unchanged
        );
        assert_eq!(
            s.key(&p, Key::Pause, false, false).unwrap(),
            KeyResult::Unchanged
        );
        pump(&mut s, &p, start + FIXED_STEP * 3, 3);
        assert_eq!(s.timing().reset_ns, 0);
        assert_eq!(p.c1().snapshot().positions[0].x, 5);
        assert_eq!(s.key(&p, Key::D, false, false).unwrap(), KeyResult::Queued);
        press(&mut s, &p, Key::Up);
        s.lost_focus(&p).unwrap();
        pump(&mut s, &p, start + FIXED_STEP * 4, 1);
        assert_eq!(
            p.c1().tick_observation().clock.held,
            [HeldDirection::Still; 2]
        );
        assert_eq!(p.c1().snapshot().positions[0].x, 5);
        identity(&p);
    }

    #[test]
    fn failed_control_publication_defers_reset_until_successful_retry() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::Pause);
        let guard = p.audit().records.borrow();
        let now = start + FIXED_STEP * 4 + Duration::from_nanos(11);
        assert_eq!(s.pump(&p, now).unwrap_err().published_ticks, 0);
        assert_eq!(s.timing().pending_ticks, 4);
        assert_eq!(s.timing().remainder, Duration::from_nanos(11));
        assert_eq!(s.timing().reset_ns, 0);
        assert_eq!(s.pending_input_count(), 1);
        assert!(!p.c1().snapshot().paused);
        drop(guard);
        pump(&mut s, &p, now, 1);
        assert_eq!(s.timing().reset_ns, FIXED_STEP.as_nanos() * 3 + 11);
        assert_eq!(s.timing().pending_ticks, 0);
        assert_eq!(s.pending_input_count(), 0);
        assert!(p.c1().snapshot().paused);
        identity(&p);
    }

    #[test]
    fn foreign_stale_ra1_and_closed_gates_reject_even_without_due_time() {
        let start = Instant::now();
        let p = Ports::new(91).unwrap();
        let other = Ports::new(91).unwrap();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        press(&mut s, &p, Key::W);
        let before = s.timing();
        assert_eq!(s.pump(&other, start).unwrap_err().published_ticks, 0);
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 1);
        assert_eq!(other.c1().snapshot().revision, 0);
        p.step_tick(1, &[]).unwrap();
        assert!(s.pump(&p, start).is_err());
        assert_eq!(s.timing(), before);
        assert_eq!(s.pending_input_count(), 1);
        assert_eq!(p.c1().snapshot().revision, 1);
        for ra1 in [false, true] {
            let p = Ports::new(91).unwrap();
            let mut s = NativeScheduler::new(&p, start).unwrap();
            press(&mut s, &p, Key::W);
            if ra1 {
                p.apply(Command {
                    generation: 91,
                    expected_revision: 0,
                    action: Action::Pause,
                })
                .unwrap();
            } else {
                p.ingress().close().unwrap();
            }
            let observed = p.c1().tick_observation();
            assert!(s.pump(&p, start).is_err());
            assert!(s.key(&p, Key::W, false, false).is_err());
            assert!(s.lost_focus(&p).is_err());
            assert_eq!(s.timing(), before);
            assert_eq!(s.pending_input_count(), 1);
            assert_eq!(p.c1().tick_observation(), observed);
            assert_eq!(p.audit().records().len(), usize::from(ra1));
        }
    }

    #[test]
    fn discard_and_reset_counter_overflow_preflight_before_any_mutation() {
        for reset in [false, true] {
            let p = Ports::new(91).unwrap();
            let start = Instant::now();
            let mut s = NativeScheduler::new(&p, start).unwrap();
            press(&mut s, &p, if reset { Key::Pause } else { Key::D });
            // Private counter boundary only; no production setters/fault seam.
            if reset {
                s.clock.reset_ns = u128::MAX;
            } else {
                s.clock.dropped_ns = u128::MAX;
            }
            let before = s.timing();
            let observation = p.c1().tick_observation();
            let elapsed = if reset {
                FIXED_STEP * 2
            } else {
                Duration::from_millis(260)
            };
            let err = s.pump(&p, start + elapsed).unwrap_err();
            assert_eq!(err.published_ticks, 0);
            assert_eq!(err.error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(s.timing(), before);
            assert_eq!(s.pending_input_count(), 1);
            assert_eq!(p.c1().tick_observation(), observation);
            assert!(p.audit().tick_records().is_empty());
            assert!(p.pinned_tick(91, 1).is_err());
        }
    }

    #[test]
    fn thirty_three_scheduled_ticks_retain_original_proofs_and_close_normally() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        for k in [Key::D, Key::Left] {
            press(&mut s, &p, k);
        }
        pump(&mut s, &p, start + FIXED_STEP * 4, 4);
        let first = p.pinned_tick(91, 1).unwrap();
        let saved = (*first).clone();
        for wake in 2..=8 {
            pump(&mut s, &p, start + FIXED_STEP * (wake * 4), 4);
            identity(&p);
        }
        pump(&mut s, &p, start + FIXED_STEP * 33, 1);
        let last = p.c1().tick_observation();
        assert_eq!(
            (
                last.clock.tick,
                last.clock.active_ticks,
                last.clock.last_sequence
            ),
            (33, 33, 2)
        );
        assert_eq!(p.audit().tick_records().len(), TICK_EVIDENCE_CAPACITY);
        assert_eq!(p.audit().tick_records()[0].applied.clock.tick, 18);
        assert_eq!(*first, saved);
        assert_eq!(
            p.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(p.pinned(91, 1).unwrap_err().kind(), io::ErrorKind::NotFound);
        assert_eq!(s.timing().dropped_ns, 0);
        assert_eq!(s.timing().reset_ns, 0);
        identity(&p);
        let c1 = Rc::clone(p.c1());
        let audit = Rc::clone(p.audit());
        let attachment = p.audit_attachment();
        p.ingress().close().unwrap();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", p, Ports::close)
            .required_local("audit", attachment, crate::three_way::close_audit)
            .build();
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(root.failures().primary().is_none());
        assert!(c1.is_stopped());
        assert_eq!(audit.status(), AuditStatus::Closed);
        assert_eq!(c1.tick_observation(), last);
        assert_eq!(audit.pinned_tick(), Some(last));
        assert_eq!(*first, saved);
    }

    #[test]
    fn later_tick_failure_reports_progress_and_retries_only_unpublished_work() {
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut s = NativeScheduler::new(&p, start).unwrap();
        // Private initial counter-boundary fixture, not a duplicate Simulation
        // or injectable publisher. All following publications use original Ports.
        let mut clock = p.c1().tick_observation().clock;
        clock.active_ticks = u64::MAX - 1;
        p.c1().tick_clock.set(clock);
        p.audit().tick_pinned.set(Some(clock));
        for k in [Key::D, Key::Up] {
            press(&mut s, &p, k);
        }
        let now = start + FIXED_STEP * 3;
        let err = s.pump(&p, now).unwrap_err();
        assert_eq!(err.published_ticks, 1);
        assert_eq!(
            err.error.to_string(),
            TickError::ActiveTickOverflow.to_string()
        );
        assert_eq!(s.timing().pending_ticks, 2);
        assert_eq!(s.pending_input_count(), 0);
        let first = p.pinned_tick(91, 1).unwrap();
        let saved = (*first).clone();
        assert_eq!(first.applied.clock.active_ticks, u64::MAX);
        assert_eq!(first.applied.clock.last_sequence, 2);
        assert!(p.pinned_tick(91, 2).is_err());
        assert_eq!(s.pump(&p, now).unwrap_err().published_ticks, 0);
        assert_eq!(s.timing().pending_ticks, 2);
        assert_eq!(*first, saved);
        // A real Pause can publish at the boundary without advancing active time;
        // no synthetic repair, proof rewrite or accepted-revision rebase is used.
        press(&mut s, &p, Key::Pause);
        pump(&mut s, &p, now, 1);
        let last = p.c1().tick_observation();
        assert_eq!((last.clock.tick, last.clock.last_sequence), (2, 3));
        assert_eq!(last.clock.active_ticks, u64::MAX);
        assert!(last.snapshot.paused);
        assert_eq!(s.timing().pending_ticks, 0);
        assert_eq!(s.timing().reset_ns, FIXED_STEP.as_nanos());
        assert_eq!(p.pinned_tick(91, 2).unwrap().inputs.len(), 1);
        assert_eq!(*first, saved);
        identity(&p);
    }
}
