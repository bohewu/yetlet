//! No Windows Cargo child is started before native cwd/Job Object support exists.
use crate::{app::Completion, platform::fs::App};
use std::{
    io,
    process::{Command, ExitStatus},
};

pub(crate) fn pin_cwd(_app: &App, _command: &mut Command) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

pub(crate) fn run(_command: &mut Command) -> io::Result<Completion> {
    Err(io::ErrorKind::Unsupported.into())
}

pub(crate) fn signal(_status: &ExitStatus) -> Option<i32> {
    None
}

// Preserve the old non-Linux Cancelled fallback; no native cancellation is claimed.
pub(crate) fn reraise(_signal: i32) {}
