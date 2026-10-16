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
    Cancelled(i32),
}

impl Completion {
    /// CLI-only finalization, after verified cleanup. Preserve native signal identity.
    pub fn finish(self) -> ! {
        crate::platform::process::finish(self)
    }
}

pub fn run(manifest: &Path, dev: bool, release: bool) -> Result<Completion, Failure> {
    let app =
        crate::platform::fs::open_app(manifest).map_err(|message| Failure { code: 1, message })?;
    crate::platform::process::run_cargo(&app, dev, release).map_err(|error| Failure {
        code: 125,
        message: format!(
            "Cargo spawn/supervision/cleanup failed ({:?}); success is not claimed",
            error.kind()
        ),
    })
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
