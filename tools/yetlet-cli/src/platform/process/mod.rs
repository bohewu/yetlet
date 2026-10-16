//! Internal compile-time process boundary; no runtime backend selection.
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "linux", windows)))]
mod unsupported;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(not(any(target_os = "linux", windows)))]
use unsupported as imp;
#[cfg(windows)]
use windows as imp;

use crate::{
    app::{self, Completion},
    platform::fs::App,
};
use std::{io, process::Command};

pub(crate) fn command(app: &App, dev: bool, release: bool) -> io::Result<Command> {
    let mut command = app::command(dev, release);
    imp::pin_cwd(app, &mut command)?;
    Ok(command)
}

pub(crate) fn run_cargo(app: &App, dev: bool, release: bool) -> io::Result<Completion> {
    let mut command = command(app, dev, release)?;
    #[cfg(windows)]
    return imp::run(app, &mut command);
    #[cfg(not(windows))]
    imp::run(&mut command)
}

pub(crate) fn finish(completion: Completion) -> ! {
    let signal = match completion {
        Completion::Cancelled(signal) => Some(signal),
        Completion::Exited(status) => {
            if !status.success() {
                eprintln!(
                    "Cargo ended with {status}; --frozen requires a current app lock and cached dependencies. No lock generation or network fallback was attempted."
                );
            }
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
            imp::signal(&status)
        }
    };
    if let Some(signal) = signal {
        imp::reraise(signal);
    }
    std::process::exit(signal.map_or(125, |value| 128 + value));
}
