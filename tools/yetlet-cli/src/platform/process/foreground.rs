//! CLI-only Linux foreground ownership. No executable-name or global waits/kills.
use crate::app::Completion;
use std::{
    fs::File,
    io,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::process::CommandExt,
    },
    process::{Child, Command},
    sync::atomic::{AtomicI32, Ordering},
    thread,
    time::{Duration, Instant},
};

static CANCEL: AtomicI32 = AtomicI32::new(0);
static NOTIFY_FD: AtomicI32 = AtomicI32::new(-1);
const POLL: Duration = Duration::from_millis(5);
const GRACE: Duration = Duration::from_millis(500);
const REAP: Duration = Duration::from_secs(3);

extern "C" fn cancel(signal: libc::c_int) {
    let _ = CANCEL.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
}

extern "C" fn notify(signal: libc::c_int) {
    // Only the forked notifier installs this handler. Never signal a parent PID
    // that could have been reused; the private pipe names its live reader.
    let byte = signal as u8;
    unsafe {
        libc::write(
            NOTIFY_FD.load(Ordering::Relaxed),
            (&byte as *const u8).cast(),
            1,
        );
        libc::_exit(0);
    }
}

fn prepare(command: &mut Command) -> io::Result<()> {
    // SAFETY: process-local adoption, before the sole Cargo spawn. The executable
    // never spawns unrelated children; adopted children are reaped only by owned PGID.
    unsafe {
        if libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = cancel as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        for signal in [libc::SIGINT, libc::SIGTERM] {
            if libc::sigaction(signal, &action, std::ptr::null_mut()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    command.process_group(0);
    Ok(())
}

fn terminal_group(group: libc::pid_t) -> io::Result<()> {
    // Block SIGTTOU during terminal handoff/restoration, including from background.
    unsafe {
        let mut mask = std::mem::zeroed();
        let mut old = std::mem::zeroed();
        libc::sigemptyset(&mut mask);
        libc::sigaddset(&mut mask, libc::SIGTTOU);
        if libc::sigprocmask(libc::SIG_BLOCK, &mask, &mut old) < 0 {
            return Err(io::Error::last_os_error());
        }
        let result = if libc::tcsetpgrp(libc::STDIN_FILENO, group) < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        };
        if libc::sigprocmask(libc::SIG_SETMASK, &old, std::ptr::null_mut()) < 0 {
            return Err(io::Error::last_os_error());
        }
        result
    }
}

struct Owned {
    child: Child,
    pgid: libc::pid_t,
    may_signal: bool,
    cleaned: bool,
    terminal: Option<libc::pid_t>,
    notifier: Option<libc::pid_t>,
    notices: Option<File>,
}

impl Owned {
    fn start_notifier(&mut self) -> io::Result<()> {
        let mut fds = [-1; 2];
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let input = unsafe { File::from_raw_fd(fds[0]) };
        let output = unsafe { File::from_raw_fd(fds[1]) };
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        unsafe { libc::sigemptyset(&mut action.sa_mask) };
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        if pid == 0 {
            // After fork use only async-signal-safe operations; no Rust runtime,
            // allocation, unwinding or external executable. Parent joins us to
            // its pinned Cargo PGID before handing that group the terminal.
            unsafe {
                libc::close(fds[0]);
                NOTIFY_FD.store(fds[1], Ordering::Relaxed);
                action.sa_sigaction = libc::SIG_IGN;
                if libc::sigaction(libc::SIGTERM, &action, std::ptr::null_mut()) < 0 {
                    libc::_exit(125);
                }
                action.sa_sigaction = notify as *const () as usize;
                if libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut()) < 0
                    || libc::write(fds[1], [0u8].as_ptr().cast(), 1) != 1
                {
                    libc::_exit(125);
                }
                loop {
                    libc::pause();
                }
            }
        }
        // Retain the unreaped direct child even if joining the group fails.
        self.notifier = Some(pid);
        self.notices = Some(input);
        drop(output);
        if unsafe { libc::setpgid(pid, self.pgid) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let deadline = Instant::now() + REAP;
        loop {
            if let Some(byte) = self.notice()? {
                return if byte == 0 {
                    Ok(())
                } else {
                    Err(io::Error::other("terminal notifier failed to start"))
                };
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        }
    }

    fn notice(&self) -> io::Result<Option<u8>> {
        let Some(input) = &self.notices else {
            return Ok(None);
        };
        let mut byte = 0u8;
        match unsafe { libc::read(input.as_raw_fd(), (&mut byte as *mut u8).cast(), 1) } {
            1 => Ok(Some(byte)),
            0 => Err(io::Error::other(
                "terminal notifier exited without cancellation",
            )),
            _ => {
                let error = io::Error::last_os_error();
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) {
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn cancellation(&self) -> io::Result<i32> {
        if CANCEL.load(Ordering::Relaxed) == 0
            && let Some(signal) = self.notice()?
        {
            if signal != libc::SIGINT as u8 {
                return Err(io::Error::other("invalid terminal cancellation"));
            }
            cancel(signal.into());
        }
        Ok(CANCEL.load(Ordering::Relaxed))
    }

    fn signal(&self, signal: i32) -> io::Result<()> {
        if self.may_signal && unsafe { libc::kill(-self.pgid, signal) } < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }

    fn exited(&self) -> io::Result<bool> {
        // WNOWAIT pins leader identity through the last signal, even after exit.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        loop {
            if unsafe {
                libc::waitid(
                    libc::P_PID,
                    self.pgid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            } == 0
            {
                break;
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        Ok(unsafe { info.si_pid() } != 0)
    }

    fn restore_terminal(&mut self) -> io::Result<()> {
        if let Some(group) = self.terminal {
            terminal_group(group)?;
            self.terminal = None;
        }
        Ok(())
    }

    fn grace(&self, forwarded: &mut bool) -> io::Result<()> {
        let deadline = Instant::now() + GRACE;
        while Instant::now() < deadline {
            let signal = self.cancellation()?;
            if signal != 0 && !*forwarded {
                self.signal(signal)?;
                *forwarded = true;
            }
            thread::sleep(POLL);
        }
        Ok(())
    }

    fn kill_reap(&mut self) -> io::Result<std::process::ExitStatus> {
        self.signal(libc::SIGKILL)?;
        if let Some(pid) = self.notifier
            && unsafe { libc::kill(pid, libc::SIGKILL) } < 0
            && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
        {
            return Err(io::Error::last_os_error());
        }
        // Never signal again after reaping, including in Drop on a later failure.
        self.may_signal = false;
        let deadline = Instant::now() + REAP;
        let status = loop {
            match self.child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => (),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        };
        while let Some(pid) = self.notifier {
            let result = unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) };
            if result == pid {
                self.notifier = None;
                break;
            }
            if result < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                return Err(io::Error::last_os_error());
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        }
        loop {
            let result = unsafe { libc::waitpid(-self.pgid, std::ptr::null_mut(), libc::WNOHANG) };
            if result > 0 {
                continue;
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.raw_os_error() != Some(libc::ECHILD) {
                    return Err(error);
                }
            }
            // Observation only; a reused PGID can never be signalled here.
            if unsafe { libc::kill(-self.pgid, 0) } < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error);
                }
                break;
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(POLL);
        }
        self.restore_terminal()?;
        self.cleaned = true;
        Ok(status)
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = self.kill_reap();
        }
        let _ = self.restore_terminal();
    }
}

/// Exactly one invocation per CLI process; stdio is inherited, never wrapped.
pub(crate) fn run(command: &mut Command) -> io::Result<Completion> {
    prepare(command)?;
    let terminal = if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
        let group = unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) };
        if group < 0 || group != unsafe { libc::getpgrp() } {
            return Err(io::Error::other("CLI does not own its foreground terminal"));
        }
        Some(group)
    } else {
        None
    };
    let signal = CANCEL.load(Ordering::Relaxed);
    if signal != 0 {
        return Ok(Completion::Cancelled(signal));
    }
    let child = command.spawn()?;
    let mut owned = Owned {
        pgid: child.id() as libc::pid_t,
        child,
        may_signal: true,
        cleaned: false,
        terminal,
        notifier: None,
        notices: None,
    };
    let result: io::Result<()> = (|| {
        let mut forwarded = false;
        if terminal.is_some() {
            owned.start_notifier()?;
            terminal_group(owned.pgid)?;
            owned.signal(libc::SIGCONT)?; // resume a possible pre-handoff SIGTTIN
        }
        loop {
            let signal = owned.cancellation()?;
            if signal != 0 {
                owned.signal(signal)?;
                forwarded = true;
                owned.grace(&mut forwarded)?;
                break;
            }
            if owned.exited()? {
                break;
            }
            thread::sleep(POLL);
        }
        // Natural Cargo exit also tears down any remaining application descendants.
        owned.signal(libc::SIGTERM)?;
        owned.grace(&mut forwarded)?;
        Ok(())
    })();
    let status = owned.kill_reap()?;
    result?;
    let signal = CANCEL.load(Ordering::Relaxed);
    Ok(if signal != 0 {
        Completion::Cancelled(signal)
    } else {
        Completion::Exited(status)
    })
}

#[cfg(test)]
#[path = "foreground_tests.rs"]
mod tests;
