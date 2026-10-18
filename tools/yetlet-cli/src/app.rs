//! Explicit standalone application preflight and frozen, literal Cargo execution.
use std::{io::Read, path::Path, process::ExitStatus};

#[derive(Debug)]
pub struct Failure {
    pub code: i32,
    pub message: String,
}

#[derive(Debug)]
pub enum Completion {
    Exited(ExitStatus),
    FrontendExited(ExitStatus),
    WatcherExited(ExitStatus),
    Cancelled(i32),
}

impl Completion {
    /// CLI-only finalization, after verified cleanup. Preserve native signal identity.
    pub fn finish(self) -> ! {
        crate::platform::process::finish(self)
    }
}

pub fn run(manifest: &Path, dev: bool, release: bool) -> Result<Completion, Failure> {
    execute(manifest, dev, release, false)
}

/// Run the selected app with Rust watch enabled for this invocation only.
pub fn watch(manifest: &Path) -> Result<Completion, Failure> {
    execute(manifest, true, false, true)
}

fn execute(
    manifest: &Path,
    dev: bool,
    release: bool,
    force_watch: bool,
) -> Result<Completion, Failure> {
    let app =
        crate::platform::fs::open_app(manifest).map_err(|message| Failure { code: 1, message })?;
    let mut settings =
        crate::settings::load(&app).map_err(|message| Failure { code: 1, message })?;
    if force_watch {
        settings.rust.watch = true;
    }
    let selected = settings.vite.enabled
        && if dev {
            settings.vite.auto_start
        } else {
            settings.vite.auto_build
        };
    if settings.tls.enabled {
        let manifest = crate::platform::fs::read_app_file(&app, "Cargo.toml")
            .map_err(|message| Failure { code: 1, message })?;
        let manifest = parse_file("Cargo.toml", manifest.as_slice())
            .map_err(|message| Failure { code: 1, message })?;
        if manifest
            .get("features")
            .and_then(|features| features.get("yetlet-dev-tls"))
            .and_then(toml::Value::as_array)
            .is_none()
        {
            return Err(Failure {
                code: 1,
                message:
                    "CLI_PROJECT: Development TLS requires the app's yetlet-dev-tls Cargo feature"
                        .into(),
            });
        }
    }
    // Build selects the capability but never inspects development key files.
    if dev {
        settings
            .tls
            .paths(&app, &settings.vite.root)
            .map_err(|message| Failure { code: 1, message })?;
    }
    if selected {
        let frontend =
            crate::platform::fs::frontend_directory(&app, Path::new(&settings.vite.root))
                .map_err(|message| Failure { code: 1, message })?;
        crate::platform::fs::validate_frontend_dotenv(&frontend)
            .map_err(|message| Failure { code: 1, message })?;
        crate::platform::fs::read_app_file(&frontend, "package.json").map_err(|_| Failure {
            code: 1,
            message: "frontend package.json is missing or unsafe; prepare dependencies separately"
                .into(),
        })?;
        if dev {
            crate::platform::fs::read_frontend_manifest(&frontend)
                .map_err(|_| Failure { code: 1, message: "Vite startup requires an existing protected frontend dist; build it separately first".into() })?;
            return if settings.rust.watch {
                crate::platform::process::run_watch(
                    &app,
                    Some(&frontend),
                    &settings.vite.root,
                    &settings.tls,
                )
            } else {
                crate::platform::process::run_with_frontend(
                    &app,
                    &frontend,
                    &settings.tls,
                    &settings.vite.root,
                )
            }
            .map_err(process_failure);
        }
        println!("Frontend build selected by cli.vite.auto_build");
        match crate::platform::process::run_frontend_build(&app, &frontend)
            .map_err(process_failure)?
        {
            Completion::Exited(status) | Completion::FrontendExited(status) if status.success() => {
                println!("Frontend build completed")
            }
            completion => return Ok(completion),
        }
    } else if !dev {
        println!("Frontend build skipped by public CLI settings");
    }
    if dev && settings.rust.watch {
        crate::platform::process::run_watch(&app, None, &settings.vite.root, &settings.tls)
            .map_err(process_failure)
    } else {
        crate::platform::process::run_cargo(&app, dev, release, &settings.tls)
            .map_err(process_failure)
    }
}

fn process_failure(error: std::io::Error) -> Failure {
    Failure {
        code: 125,
        message: format!(
            "owned child spawn/supervision/cleanup failed ({:?}); success is not claimed",
            error.kind()
        ),
    }
}

pub(crate) fn parse_file(name: &str, file: impl std::io::Read) -> Result<toml::Value, String> {
    let mut data = String::new();
    file.take(1024 * 1024 + 1)
        .read_to_string(&mut data)
        .map_err(|_| format!("cannot read UTF-8 {name}"))?;
    if data.len() > 1024 * 1024 {
        return Err(format!("{name} exceeds the preview size limit"));
    }
    data.parse().map_err(|_| format!("{name} is invalid TOML"))
}

pub(crate) fn validate_manifest(manifest: &toml::Value) -> Result<(), String> {
    let package = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or("manifest must declare [package]")?;
    let workspace = manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .ok_or("manifest must declare its own standalone [workspace]")?;
    if package.contains_key("workspace")
        || ["members", "default-members"].iter().any(|key| {
            workspace
                .get(*key)
                .is_some_and(|v| v.as_array().is_none_or(|a| !a.is_empty()))
        })
    {
        return Err(
            "workspace inheritance/members are unsupported; select a standalone app".into(),
        );
    }
    Ok(())
}

pub(crate) fn validate_lock(lock: &toml::Value) -> Result<(), String> {
    if lock
        .get("version")
        .and_then(toml::Value::as_integer)
        .is_none()
        || lock
            .get("package")
            .and_then(toml::Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Err(
            "Cargo.lock must be this app's explicit existing lock; refresh it separately offline"
                .into(),
        );
    }
    Ok(())
}

pub(crate) fn command(dev: bool, release: bool) -> std::process::Command {
    use std::process::Command;
    let mut command = Command::new("cargo");
    command.args([
        if dev { "run" } else { "build" },
        "--frozen",
        "--manifest-path",
        "Cargo.toml",
    ]);
    if release {
        command.arg("--release");
    }
    command
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
        .env("CARGO_NET_OFFLINE", "true");
    command
}
