//! Internal compile-time process boundary; no runtime backend selection.
mod probe;
pub(crate) use probe::{ProbeError, probe};
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

fn frontend_command(app: &App, frontend: &App, dev: bool) -> io::Result<Command> {
    #[cfg(windows)]
    let mut command = imp::npm_command(dev)?;
    #[cfg(not(windows))]
    let mut command = {
        let mut command = Command::new("npm");
        command.args(["run", if dev { "dev" } else { "build" }]);
        command
    };
    if dev {
        command.args(["--", "--strictPort"]);
        command.env("YETLET_VITE_PORT", "5173");
    }
    // Vite's own dotenv loader is confined by the generated frontend root.
    // Backend settings and secret selectors never cross this process boundary.
    for (key, _) in std::env::vars_os() {
        let upper = key.to_string_lossy().to_ascii_uppercase();
        if upper.starts_with("APP_")
            || matches!(upper.as_str(), "YETLET_DEV_SECRETS" | "YETLET_CONFIG_ROOT")
        {
            command.env_remove(key);
        }
    }
    command
        .env("YETLET_PROJECT_ROOT", crate::platform::fs::app_root(app))
        .env("npm_config_offline", "true")
        .env("npm_config_audit", "false")
        .env("npm_config_fund", "false");
    imp::pin_cwd(frontend, &mut command)?;
    Ok(command)
}

pub(crate) fn run_frontend_build(app: &App, frontend: &App) -> io::Result<Completion> {
    let mut command = frontend_command(app, frontend, false)?;
    #[cfg(windows)]
    let completion = imp::run(frontend, &mut command)?;
    #[cfg(not(windows))]
    let completion = imp::run(&mut command)?;
    Ok(frontend_completion(completion))
}

fn frontend_completion(completion: Completion) -> Completion {
    match completion {
        Completion::Exited(status) => Completion::FrontendExited(status),
        other => other,
    }
}

pub(crate) fn run_with_frontend(app: &App, frontend: &App) -> io::Result<Completion> {
    let mut frontend_command = frontend_command(app, frontend, true)?;
    frontend_command.stdin(std::process::Stdio::null());
    let mut rust = command(app, true, false)?;
    rust.env("YETLET_VITE_PORT", "5173");
    #[cfg(windows)]
    return imp::run_pair(app, &mut rust, frontend, &mut frontend_command);
    #[cfg(not(windows))]
    imp::run_pair(&mut rust, &mut frontend_command)
}

pub(crate) fn finish(completion: Completion) -> ! {
    let signal = match completion {
        Completion::Cancelled(signal) => {
            eprintln!("CLI_CANCELLED stage=execution: owned children stopped");
            Some(signal)
        }
        Completion::Exited(status) | Completion::FrontendExited(status) => {
            let stage = if matches!(completion, Completion::FrontendExited(_)) {
                "frontend"
            } else {
                "rust"
            };
            if !status.success() {
                eprintln!("CLI_CHILD_EXIT stage={stage}: child ended with {status}");
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
