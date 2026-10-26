//! YT-159 sample-only same-thread clocks; no async interval or product hook.
#[cfg(not(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")))]
compile_error!("ssr-thread-cpu-probe requires Linux x86_64 GNU");

use std::{
    ffi::{c_int, c_long, c_ulong},
    time::Instant,
};

// Private layouts match this target's installed time.h and pthreadtypes.h.
#[repr(C)]
#[derive(Default)]
struct Timespec {
    sec: c_long,
    nsec: c_long,
}
unsafe extern "C" {
    fn clock_gettime(clock: c_int, time: *mut Timespec) -> c_int;
    fn clock_getres(clock: c_int, time: *mut Timespec) -> c_int;
    fn pthread_self() -> c_ulong;
    fn pthread_equal(first: c_ulong, second: c_ulong) -> c_int;
    fn sched_getcpu() -> c_int;
}
fn clock(resolution: bool) -> Result<u64, i32> {
    let mut time = Timespec::default();
    // SAFETY: Correct writable/aligned GNU x86_64 timespec; Linux clock ID3 is
    // CLOCK_THREAD_CPUTIME_ID. No pointer is retained by either C function.
    let result = unsafe {
        if resolution {
            clock_getres(3, &mut time)
        } else {
            clock_gettime(3, &mut time)
        }
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(-1));
    }
    if time.sec < 0 || !(0..1_000_000_000).contains(&time.nsec) {
        return Err(-2);
    }
    (time.sec as u64)
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(time.nsec as u64))
        .ok_or(-2)
}
pub(super) fn resolution_ns() -> Result<u64, i32> {
    clock(true)
}

pub(super) struct Begin {
    wall: Instant,
    thread_id: c_ulong,
    cpu_id: c_int,
    thread_clock: Result<u64, i32>,
}
#[derive(Clone, Copy, serde::Serialize)]
pub(super) struct Measurement {
    pub(super) wall_ns: u128,
    pub(super) thread_cpu_ns: Option<u64>,
    pub(super) wall_minus_cpu_ns: Option<i128>,
    pub(super) thread_id_start: c_ulong,
    pub(super) thread_id_end: c_ulong,
    pub(super) cpu_id_start: c_int,
    pub(super) cpu_id_end: c_int,
    pub(super) same_thread: bool,
    pub(super) status: &'static str,
    pub(super) clock_error: Option<i32>,
}
impl Begin {
    pub(super) fn new() -> Self {
        let wall = Instant::now();
        // SAFETY: These read-only GNU functions take no pointers and allocate no
        // observer storage. pthread identity is opaque/local, not a kernel TID.
        let (thread_id, cpu_id) = unsafe { (pthread_self(), sched_getcpu()) };
        Self {
            wall,
            thread_id,
            cpu_id,
            thread_clock: clock(false),
        }
    }
    pub(super) fn finish(self) -> Measurement {
        let end = clock(false);
        // SAFETY: Both identities belong to the current live thread on this
        // synchronous path; pthread_equal provides the native identity check.
        let (thread_id_end, cpu_id_end, same_thread) = unsafe {
            let id = pthread_self();
            (id, sched_getcpu(), pthread_equal(self.thread_id, id) != 0)
        };
        let wall_ns = self.wall.elapsed().as_nanos();
        let mut record = Measurement {
            wall_ns,
            thread_cpu_ns: None,
            wall_minus_cpu_ns: None,
            thread_id_start: self.thread_id,
            thread_id_end,
            cpu_id_start: self.cpu_id,
            cpu_id_end,
            same_thread,
            status: "clock_error",
            clock_error: None,
        };
        match (self.thread_clock, end) {
            (Err(error), _) | (_, Err(error)) => record.clock_error = Some(error),
            (Ok(start), Ok(end)) => {
                if self.thread_id == 0 || thread_id_end == 0 || self.cpu_id < 0 || cpu_id_end < 0 {
                    record.status = "invalid_identity";
                } else if !same_thread {
                    record.status = "thread_changed";
                } else if let Some(cpu) = end.checked_sub(start) {
                    if cpu == 0 {
                        record.status = "clock_nonpositive";
                    } else {
                        record.status = "ok";
                        record.thread_cpu_ns = Some(cpu);
                        record.wall_minus_cpu_ns = Some(wall_ns as i128 - cpu as i128);
                    }
                } else {
                    record.status = "clock_reversed";
                }
            }
        }
        record
    }
}
