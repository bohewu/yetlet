use std::{
    io,
    os::{fd::AsRawFd, unix::process::CommandExt},
    process::{Child, Command},
    sync::atomic::Ordering,
    time::Instant,
};

pub trait Pipe: AsRawFd {}
impl<T: AsRawFd> Pipe for T {}

pub fn configure(pipe: &impl Pipe) -> io::Result<()> {
    // SAFETY: live borrowed pipe fd; preserve all pre-existing flags.
    unsafe {
        let flags = libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL);
        if flags < 0 || libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn available(_: &impl Pipe) -> io::Result<usize> {
    Ok(8192)
}

pub struct Scope {
    pgid: Option<libc::pid_t>,
    signalled: bool,
}
impl Scope {
    pub fn prepare(command: &mut Command) -> io::Result<Self> {
        // Process-local adoption lets Linux reap owned orphaned grandchildren.
        // Never wait for other groups or change any host-wide setting.
        #[cfg(target_os = "linux")]
        if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } < 0 {
            return Err(io::Error::last_os_error());
        }
        command.process_group(0);
        Ok(Self {
            pgid: None,
            signalled: false,
        })
    }
    pub fn attach(&mut self, child: &Child) -> io::Result<()> {
        self.pgid = Some(child.id() as libc::pid_t);
        Ok(())
    }
    pub fn exited(&self, _: &mut Child) -> io::Result<bool> {
        // WNOWAIT keeps the group leader's PID reserved even after it exits.
        // Never use Child::try_wait before signalling the pinned process group.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                self.pgid.unwrap() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { info.si_pid() } != 0)
    }
    pub fn terminate(&mut self) -> io::Result<()> {
        if let Some(pgid) = self.pgid.filter(|_| !self.signalled) {
            // No untrusted PID lookup: group leader has not been reaped.
            if unsafe { libc::kill(-pgid, libc::SIGKILL) } < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error);
                }
            }
            // Never signal again after reaping, including during fallback Drop.
            self.signalled = true;
        }
        Ok(())
    }
    pub fn request_stop(&self, interrupt: bool) -> io::Result<()> {
        if let Some(pgid) = self.pgid.filter(|_| !self.signalled)
            && unsafe {
                libc::kill(
                    -pgid,
                    if interrupt {
                        libc::SIGINT
                    } else {
                        libc::SIGTERM
                    },
                )
            } < 0
        {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }
    pub fn finish(&self, deadline: Instant) -> io::Result<()> {
        let Some(pgid) = self.pgid else {
            return Ok(());
        };
        loop {
            #[cfg(target_os = "linux")]
            loop {
                // The direct child was already reaped by Child; only this owned
                // group's adopted descendants are eligible here.
                let result = unsafe { libc::waitpid(-pgid, std::ptr::null_mut(), libc::WNOHANG) };
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
                break;
            }
            // Observation only after reaping: never signal a reusable group ID.
            if unsafe { libc::kill(-pgid, 0) } < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::ESRCH) {
                    return Ok(());
                }
                return Err(error);
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(super::POLL);
        }
    }
}

extern "C" fn cancel(signal: libc::c_int) {
    // Only lock-free atomics; no allocation, I/O, locks, or cleanup in handler.
    let _ = super::CANCELLATION_SIGNAL.compare_exchange(
        0,
        signal,
        Ordering::Relaxed,
        Ordering::Relaxed,
    );
    super::CANCELLED.store(true, Ordering::Relaxed);
}

pub fn install_cancellation() -> io::Result<()> {
    // CLI-only installation, before spawning; library callers supply their own token.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = cancel as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        for signal in [libc::SIGINT, libc::SIGTERM] {
            if libc::sigaction(signal, &action, std::ptr::null_mut()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
