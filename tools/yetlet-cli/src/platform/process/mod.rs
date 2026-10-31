//! Internal compile-time process boundary; no runtime backend selection.
mod probe;
pub(crate) mod watch;
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

pub(crate) fn run_cargo(
    app: &App,
    dev: bool,
    release: bool,
    tls: &crate::settings::Tls,
) -> io::Result<Completion> {
    let mut command = command(app, dev, release)?;
    if tls.enabled {
        command.args(["--features", "yetlet-dev-tls"]);
    }
    #[cfg(windows)]
    return imp::run(app, &mut command);
    #[cfg(not(windows))]
    imp::run(&mut command)
}

/// Static export is finite trusted native execution, with the existing owned
/// group/Job and cancellation semantics plus a wall deadline.
pub(crate) fn run_static(
    app: &App,
    command: &mut Command,
    budget: std::time::Duration,
) -> io::Result<Completion> {
    imp::pin_cwd(app, command)?;
    #[cfg(windows)]
    return imp::run_bounded(app, command, budget);
    #[cfg(target_os = "linux")]
    return imp::run_bounded(command, budget);
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (command, budget);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "static export process ownership unavailable",
        ))
    }
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
            || matches!(
                upper.as_str(),
                "YETLET_DEV_SECRETS" | "YETLET_DEVELOPMENT_SECRET_FILE" | "YETLET_CONFIG_ROOT"
            )
        {
            command.env_remove(key);
        }
    }
    command
        .env_remove("YETLET_DEV_TLS_CERT")
        .env_remove("YETLET_DEV_TLS_KEY")
        .env_remove("YETLET_BROWSER_WATCH");
    command
        .env("YETLET_PROJECT_ROOT", crate::platform::fs::app_root(app))
        .env("npm_config_offline", "true")
        .env("npm_config_audit", "false")
        .env("npm_config_fund", "false");
    imp::pin_cwd(frontend, &mut command)?;
    Ok(command)
}

pub(crate) fn run_frontend_build(app: &App, frontend: &App) -> io::Result<Completion> {
    run_frontend(app, frontend, false)
}

/// Browser applications reuse single-child process ownership on both platforms.
pub(crate) fn run_frontend(app: &App, frontend: &App, dev: bool) -> io::Result<Completion> {
    let mut command = frontend_command(app, frontend, dev)?;
    #[cfg(windows)]
    let completion = imp::run(frontend, &mut command)?;
    #[cfg(not(windows))]
    let completion = imp::run(&mut command)?;
    Ok(frontend_completion(completion))
}

/// Browser watch uses the same single owned frontend tree, including compilers.
pub(crate) fn run_browser_frontend(
    app: &App,
    frontend: &App,
    dev: bool,
    watch: bool,
) -> io::Result<Completion> {
    let mut command = frontend_command(app, frontend, dev)?;
    if watch {
        command.env("YETLET_BROWSER_WATCH", "1");
    }
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

pub(crate) fn run_with_frontend(
    app: &App,
    frontend: &App,
    tls: &crate::settings::Tls,
    frontend_root: &str,
) -> io::Result<Completion> {
    let mut frontend_command = frontend_command(app, frontend, true)?;
    frontend_command.stdin(std::process::Stdio::null());
    let mut rust = command(app, true, false)?;
    bridge_environment(app, &mut rust, &mut frontend_command, tls, frontend_root)?;
    #[cfg(windows)]
    return imp::run_pair(app, &mut rust, frontend, &mut frontend_command);
    #[cfg(not(windows))]
    imp::run_pair(&mut rust, &mut frontend_command)
}

pub(crate) fn run_watch(
    app: &App,
    frontend: Option<&App>,
    frontend_root: &str,
    tls: &crate::settings::Tls,
) -> io::Result<Completion> {
    let mut rust = command(app, true, false)?;
    if tls.enabled && frontend.is_none() {
        rust.args(["--features", "yetlet-dev-tls"]);
    }
    let mut watcher = watch::command(app, frontend_root);
    imp::pin_cwd(app, &mut watcher)?;
    let mut front = frontend
        .map(|front| frontend_command(app, front, true))
        .transpose()?;
    if let Some(front) = front.as_mut() {
        front.stdin(std::process::Stdio::null());
        bridge_environment(app, &mut rust, front, tls, frontend_root)?;
    }
    println!("Rust watch enabled by cli.rust.watch; Watchexec supplies file events");
    #[cfg(windows)]
    return imp::run_watch(app, &mut rust, &mut watcher, frontend.zip(front.as_mut()));
    #[cfg(not(windows))]
    imp::run_watch(&mut rust, &mut watcher, front.as_mut())
}

fn bridge_environment(
    app: &App,
    rust: &mut Command,
    frontend: &mut Command,
    tls: &crate::settings::Tls,
    root: &str,
) -> io::Result<()> {
    rust.env("YETLET_VITE_PORT", "5173")
        .env("APP_SERVER__HOST", "127.0.0.1")
        .env("APP_SERVER__PORT", "3000");
    if tls.enabled {
        rust.args(["--features", "yetlet-dev-tls"]);
        rust.env("APP_SERVER__TLS__ENABLED", "false");
    }
    if let Some((certificate, key)) = tls.paths(app, root).map_err(io::Error::other)? {
        frontend
            .env("YETLET_DEV_TLS_CERT", certificate)
            .env("YETLET_DEV_TLS_KEY", key);
    }
    Ok(())
}

pub(crate) fn finish(completion: Completion) -> ! {
    let signal = match completion {
        Completion::Skipped => std::process::exit(0),
        Completion::Cancelled(signal) => {
            eprintln!("CLI_CANCELLED stage=execution: owned children stopped");
            Some(signal)
        }
        Completion::Exited(status)
        | Completion::FrontendExited(status)
        | Completion::WatcherExited(status) => {
            let stage = match completion {
                Completion::FrontendExited(_) => "frontend",
                Completion::WatcherExited(_) => "watcher",
                _ => "rust",
            };
            if matches!(completion, Completion::WatcherExited(_)) && status.success() {
                eprintln!("CLI_CHILD_EXIT stage=watcher: file event source ended unexpectedly");
                std::process::exit(125);
            }
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

#[cfg(all(test, any(target_os = "linux", windows)))]
mod static_deadline_tests {
    use super::*;
    #[test]
    #[ignore = "spawned only by the bounded static-export deadline test"]
    fn yt223_sleep_helper() {
        std::fs::write("child.pid", std::process::id().to_string()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(20));
    }
    #[test]
    fn yt223_bounded_deadline_reaps_its_owned_child() {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        crate::platform::fs::create_project(temp.path(), "deadline", &[]).unwrap();
        let app = crate::platform::fs::open_directory(&temp.path().join("deadline")).unwrap();
        let executable = crate::platform::fs::app_root(&app).join(if cfg!(windows) {
            "helper.exe"
        } else {
            "helper"
        });
        std::fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let mut command = Command::new(executable);
        command.args([
            "--exact",
            "platform::process::static_deadline_tests::yt223_sleep_helper",
            "--ignored",
            "--nocapture",
        ]);
        let started = std::time::Instant::now();
        let result = run_static(&app, &mut command, std::time::Duration::from_secs(2));
        assert!(matches!(result,Err(ref e) if e.kind()==io::ErrorKind::TimedOut));
        assert!(started.elapsed() < std::time::Duration::from_secs(10));
        let pid: u32 =
            std::fs::read_to_string(crate::platform::fs::app_root(&app).join("child.pid"))
                .unwrap()
                .parse()
                .unwrap();
        #[cfg(target_os = "linux")]
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
        #[cfg(windows)]
        {
            use windows_sys::Win32::{
                Foundation::CloseHandle,
                System::Threading::{
                    GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
                },
            };
            let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
            if !handle.is_null() {
                let mut code = 0;
                assert_ne!(unsafe { GetExitCodeProcess(handle, &mut code) }, 0);
                unsafe { CloseHandle(handle) };
                assert_ne!(code, 259);
            }
        }
    }
}
