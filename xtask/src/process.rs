//! Bounded literal subprocess execution with platform-scoped subtree ownership.
//! Captured bytes are never printed automatically: callers must redact before logging.
use std::{
    ffi::OsString,
    io::{self, Read},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
#[path = "process/unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "process/windows.rs"]
mod platform;
#[cfg(not(any(unix, windows)))]
compile_error!("xtask owned-process support requires Unix or Windows");

pub static CANCELLED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
static CANCELLATION_SIGNAL: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// First cancellation identity, retained through cleanup and report finalization.
pub fn cancellation_signal() -> Option<i32> {
    #[cfg(unix)]
    {
        let signal = CANCELLATION_SIGNAL.load(Ordering::Relaxed);
        (signal != 0).then_some(signal)
    }
    #[cfg(windows)]
    {
        // Console Ctrl-C/Break cancellation corresponds to Python's SIGINT.
        CANCELLED.load(Ordering::Relaxed).then_some(2)
    }
}

const CLEANUP_BUDGET: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(5);

pub use platform::install_cancellation;

#[derive(Debug)]
pub struct Spec {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    /// Native environment overrides; None removes a variable, others are inherited.
    pub env: Vec<(OsString, Option<OsString>)>,
    pub timeout: Duration,
    /// Per-stream retained prefix. Excess bytes are drained and counted, never stored.
    pub output_limit: usize,
}

#[derive(Debug, Default)]
pub struct Capture {
    pub bytes: Vec<u8>,
    pub discarded: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Child,
    Timeout,
    Cancelled,
    Spawn,
    Cleanup,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
            Self::Spawn => "spawn",
            Self::Cleanup => "cleanup",
        }
    }
}

#[derive(Debug)]
pub struct Report {
    pub origin: Origin,
    pub child_status: Option<ExitStatus>,
    pub stdout: Capture,
    pub stderr: Capture,
    /// Generic OS error kind only; no environment, argv, or private paths in diagnostics.
    pub error: Option<io::ErrorKind>,
    pub cleanup_ok: bool,
}

impl Report {
    /// Full Windows DWORD retained independently of the portable process exit mapping.
    pub fn native_code(&self) -> Option<u32> {
        self.child_status.and_then(|s| s.code()).map(|c| c as u32)
    }
    pub fn signal(&self) -> Option<i32> {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            self.child_status.and_then(|s| s.signal())
        }
        #[cfg(windows)]
        {
            None
        }
    }
    pub fn exit_code(&self) -> i32 {
        match self.origin {
            Origin::Spawn | Origin::Cleanup => 125,
            Origin::Timeout => 124,
            Origin::Cancelled => 130,
            Origin::Child => {
                if let Some(signal) = self.signal() {
                    return 128 + signal;
                }
                // Cargo and Unix shells cannot faithfully relay a Windows DWORD.
                self.native_code()
                    .map_or(125, |code| if code <= 255 { code as i32 } else { 1 })
            }
        }
    }
    pub fn success(&self) -> bool {
        self.origin == Origin::Child && self.exit_code() == 0 && self.cleanup_ok
    }
}

struct Owned {
    child: Child,
    scope: platform::Scope,
    cleaned: bool,
}

impl Owned {
    fn cleanup(&mut self, deadline: Instant) -> io::Result<ExitStatus> {
        self.scope.terminate()?;
        let status = loop {
            if let Some(status) = self.child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        };
        self.scope.finish(deadline)?;
        self.cleaned = true;
        Ok(status)
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.cleaned {
            // Same bounded kill/wait fallback for panics and partial startup.
            let _ = self.cleanup(Instant::now() + CLEANUP_BUDGET);
        }
    }
}

fn drain<R: Read + platform::Pipe>(
    pipe: &mut R,
    capture: &mut Capture,
    limit: usize,
) -> io::Result<bool> {
    let mut buffer = [0; 8192];
    // Fairness: neither a noisy stream nor a pipe-holding descendant can hide a deadline.
    for _ in 0..8 {
        let available = platform::available(pipe)?;
        if available == 0 {
            return Ok(false);
        }
        let length = buffer.len().min(available);
        match pipe.read(&mut buffer[..length]) {
            Ok(0) => return Ok(true),
            Ok(n) => {
                let keep = n.min(limit.saturating_sub(capture.bytes.len()));
                capture.bytes.extend_from_slice(&buffer[..keep]);
                capture.discarded = capture.discarded.saturating_add((n - keep) as u64);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

pub fn run(spec: &Spec, cancel: &AtomicBool) -> Report {
    let mut report = Report {
        origin: Origin::Spawn,
        child_status: None,
        stdout: Capture::default(),
        stderr: Capture::default(),
        error: None,
        cleanup_ok: true,
    };
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(&spec.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in &spec.env {
        if let Some(value) = value {
            command.env(key, value);
        } else {
            command.env_remove(key);
        }
    }
    let started = Instant::now();
    let result = (|| -> io::Result<()> {
        let scope = platform::Scope::prepare(&mut command)?;
        let child = command.spawn()?;
        let mut owned = Owned {
            child,
            scope,
            cleaned: false,
        };
        // Registration happens before any child code runs on Windows. Unix setpgid
        // is performed by std in the child before exec, with the PID pinned until cleanup.
        if let Err(e) = owned.scope.attach(&owned.child) {
            // A failed assignment leaves a suspended Windows child outside the Job.
            let killed = owned.child.kill();
            let cleanup = owned.cleanup(Instant::now() + CLEANUP_BUDGET);
            report.cleanup_ok = killed.is_ok() && cleanup.is_ok();
            if !report.cleanup_ok {
                report.origin = Origin::Cleanup;
            }
            return Err(e);
        }
        let mut stdout = owned.child.stdout.take().ok_or(io::ErrorKind::BrokenPipe)?;
        let mut stderr = owned.child.stderr.take().ok_or(io::ErrorKind::BrokenPipe)?;
        platform::configure(&stdout)?;
        platform::configure(&stderr)?;
        report.origin = Origin::Child;
        let deadline = started + spec.timeout;
        let mut read_error = None;
        loop {
            if cancel.load(Ordering::Relaxed) {
                report.origin = Origin::Cancelled;
                break;
            }
            if Instant::now() >= deadline {
                report.origin = Origin::Timeout;
                break;
            }
            if let Err(e) = drain(&mut stdout, &mut report.stdout, spec.output_limit)
                .and_then(|_| drain(&mut stderr, &mut report.stderr, spec.output_limit))
            {
                read_error = Some(e);
                break;
            }
            match owned.scope.exited(&mut owned.child) {
                Ok(true) => break,
                Ok(false) => thread::sleep(POLL),
                Err(e) => {
                    read_error = Some(e);
                    break;
                }
            }
        }
        let cleanup_deadline = Instant::now() + CLEANUP_BUDGET;
        match owned.cleanup(cleanup_deadline) {
            Ok(status) => report.child_status = Some(status),
            Err(e) => {
                report.origin = Origin::Cleanup;
                report.cleanup_ok = false;
                return Err(e);
            }
        }
        // Pipes are nonblocking/peeked; no worker thread can hang in join().
        let (mut out_done, mut err_done) = (false, false);
        while !(out_done && err_done) {
            out_done |= drain(&mut stdout, &mut report.stdout, spec.output_limit)?;
            err_done |= drain(&mut stderr, &mut report.stderr, spec.output_limit)?;
            if out_done && err_done {
                break;
            }
            if Instant::now() >= cleanup_deadline {
                report.origin = Origin::Cleanup;
                report.cleanup_ok = false;
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        }
        if let Some(e) = read_error {
            return Err(e);
        }
        Ok(())
    })();
    if let Err(error) = result {
        report.error = Some(error.kind());
        if report.origin == Origin::Child {
            report.origin = Origin::Cleanup;
            report.cleanup_ok = false;
        }
    }
    final_report(report, cancel)
}

fn final_report(mut report: Report, cancel: &AtomicBool) -> Report {
    // Observe cancellation after cleanup and final pipe draining too. Keep the
    // original native status and never replace a cleanup/spawn/timeout failure.
    if report.origin == Origin::Child && report.cleanup_ok && cancel.load(Ordering::Relaxed) {
        report.origin = Origin::Cancelled;
    }
    report
}

/// Persistent suite-owned server/browser. File streams avoid pipe deadlock while
/// retaining the foundation's pinned process group / suspended Job registration.
pub struct Service {
    owned: Owned,
    stream: Option<std::fs::File>,
    pub escalated: bool,
}
impl Service {
    pub fn spawn(spec: &Spec, log: &std::path::Path) -> io::Result<Self> {
        let stream = std::fs::File::create(log)?;
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .stdin(Stdio::null())
            .stdout(stream.try_clone()?)
            .stderr(stream.try_clone()?);
        for (key, value) in &spec.env {
            if let Some(value) = value {
                command.env(key, value);
            } else {
                command.env_remove(key);
            }
        }
        let scope = platform::Scope::prepare(&mut command)?;
        let child = command.spawn()?;
        let mut service = Self {
            owned: Owned {
                child,
                scope,
                cleaned: false,
            },
            stream: Some(stream),
            escalated: false,
        };
        if let Err(error) = service.owned.scope.attach(&service.owned.child) {
            let _ = service.owned.child.kill();
            let _ = service.stop(false, CLEANUP_BUDGET);
            return Err(error);
        }
        Ok(service)
    }
    pub fn id(&self) -> u32 {
        self.owned.child.id()
    }
    /// Observe without reaping: the group's identity remains pinned until cleanup.
    pub fn exited(&mut self) -> io::Result<bool> {
        self.owned.scope.exited(&mut self.owned.child)
    }
    pub fn stop(&mut self, interrupt: bool, grace: Duration) -> io::Result<ExitStatus> {
        if self.owned.cleaned {
            return self.owned.child.wait();
        }
        let result = (|| {
            self.owned.scope.request_stop(interrupt)?;
            let deadline = Instant::now() + grace;
            while !self.exited()? && Instant::now() < deadline {
                thread::sleep(POLL);
            }
            self.escalated = !self.exited()?;
            // Even a gracefully exited parent can leave helpers. Kill the pinned
            // group before reaping, then reap adopted descendants and verify empty.
            self.owned.cleanup(Instant::now() + CLEANUP_BUDGET)
        })();
        self.stream.take();
        result
    }
    pub fn stream_closed(&self) -> bool {
        self.stream.is_none()
    }
    /// Failed startup must preserve the original readiness error after teardown.
    pub fn readiness(&mut self, result: crate::tooling::Result<()>) -> crate::tooling::Result<()> {
        if result.is_err() {
            self.stop(false, Duration::from_secs(4))
                .map_err(|_| crate::tooling::Error {
                    code: 125,
                    message: "failed-readiness cleanup failed".into(),
                })?;
        }
        result
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        if !self.owned.cleaned {
            let _ = self.stop(false, Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn finalization_observes_late_cancellation_without_losing_status_or_cleanup_failure() {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        let cancel = AtomicBool::new(true);
        let report = Report {
            origin: Origin::Child,
            child_status: Some(ExitStatus::from_raw(0)),
            stdout: Capture::default(),
            stderr: Capture::default(),
            error: None,
            cleanup_ok: true,
        };
        let mut report = final_report(report, &cancel);
        assert_eq!(report.origin, Origin::Cancelled);
        assert_eq!(report.native_code(), Some(0));
        assert_eq!(report.exit_code(), 130);
        assert!(!report.success());
        report.origin = Origin::Cleanup;
        report.cleanup_ok = false;
        report.error = Some(io::ErrorKind::TimedOut);
        let report = final_report(report, &cancel);
        assert_eq!(report.origin, Origin::Cleanup);
        assert_eq!(report.exit_code(), 125);
        assert_eq!(report.native_code(), Some(0));
        assert!(!report.success());
    }

    #[test]
    #[ignore = "test-only RAII child"]
    fn raii_fixture() {
        let mut file = std::fs::File::create(std::env::var_os("YT083_RAII_FILE").unwrap()).unwrap();
        for _ in 0..3000 {
            file.write_all(b".").unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn raii_fallback_kills_and_reaps_on_unwind() {
        let path = std::env::temp_dir().join(format!(
            "yt083-raii-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let unwind = std::panic::catch_unwind(|| {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "process::tests::raii_fixture",
                    "--ignored",
                    "--nocapture",
                ])
                .env("YT083_RAII_FILE", &path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let scope = platform::Scope::prepare(&mut command).unwrap();
            let child = command.spawn().unwrap();
            let mut owned = Owned {
                child,
                scope,
                cleaned: false,
            };
            owned.scope.attach(&owned.child).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            while !std::fs::metadata(&path).is_ok_and(|m| m.len() > 0) {
                assert!(Instant::now() < deadline);
                thread::sleep(POLL);
            }
            panic!("exercise RAII fallback after partial startup");
        });
        assert!(unwind.is_err());
        let stopped = std::fs::metadata(&path).unwrap().len();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(std::fs::metadata(&path).unwrap().len(), stopped);
        std::fs::remove_file(path).unwrap();
    }
}
