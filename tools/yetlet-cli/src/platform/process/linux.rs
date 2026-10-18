//! Linux cwd pinning and existing owned foreground supervision.
#[path = "foreground.rs"]
mod foreground;
use crate::{app::Completion, platform::fs::App};
use std::{
    io,
    os::{
        fd::AsRawFd,
        unix::process::{CommandExt, ExitStatusExt},
    },
    process::{Command, ExitStatus},
};

pub(crate) fn pin_cwd(app: &App, command: &mut Command) -> io::Result<()> {
    // std chdir is followed by fchdir to the held identity before exec.
    command.current_dir(&app.cwd);
    let fd = app.directory.as_raw_fd();
    // SAFETY: App outlives spawn; fchdir is async-signal-safe and allocates nothing.
    unsafe {
        command.pre_exec(move || {
            if libc::fchdir(fd) < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    Ok(())
}

pub(crate) fn run(command: &mut Command) -> io::Result<Completion> {
    foreground::run(command)
}

pub(crate) fn run_pair(rust: &mut Command, frontend: &mut Command) -> io::Result<Completion> {
    foreground::run_pair(rust, frontend)
}

pub(crate) fn signal(status: &ExitStatus) -> Option<i32> {
    status.signal()
}

pub(crate) fn reraise(signal: i32) {
    // SAFETY: reset/unblock only this final signal, after all owned resources
    // have been reaped and terminal ownership has been restored.
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        let mut mask = std::mem::zeroed();
        libc::sigemptyset(&mut mask);
        libc::sigaddset(&mut mask, signal);
        libc::sigprocmask(libc::SIG_UNBLOCK, &mask, std::ptr::null_mut());
        libc::raise(signal);
    }
}
