//! Internal compile-time filesystem boundary; no public adapter API.
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
#[cfg(windows)]
pub(in crate::platform) use windows::Executable;

use crate::app;
pub(crate) use imp::{App, NEW_DIAGNOSTIC, NEW_SUPPORTED, create_project, read_framework};
pub(crate) use imp::{app_entries, read_app_file};
use std::path::Path;

const UNSUPPORTED_CREATE: &str = "new is unsupported on this platform: no verified no-replace/reparse implementation; nothing written";

// Must precede framework inspection/template rendering, retaining refusal ordering.
pub(crate) fn ensure_create_supported() -> Result<(), String> {
    if NEW_SUPPORTED {
        Ok(())
    } else {
        Err(UNSUPPORTED_CREATE.into())
    }
}

pub(crate) fn open_app(manifest: &Path) -> Result<App, String> {
    let files = imp::AppFiles::open(manifest)?;
    let manifest = app::parse_file("Cargo.toml", files.read("Cargo.toml")?)?;
    app::validate_manifest(&manifest)?;
    let lock = app::parse_file("Cargo.lock", files.read("Cargo.lock")?)?;
    app::validate_lock(&lock)?;
    files.finish()
}

// The two built-in generators each need exactly one fixed child directory.
// Keep arbitrary relative paths out of the native write/rollback adapters.
fn generated_subdirectory(files: &[(&str, String)]) -> Result<&'static str, String> {
    let mut child = None;
    for (path, _) in files {
        let directory = match *path {
            "src/main.rs" | "src/config.rs" => Some("src"),
            "cargo-overlay/Cargo.toml" => Some("cargo-overlay"),
            name if !name.is_empty()
                && !matches!(name, "." | "..")
                && !name.contains(['/', '\\', '\0']) =>
            {
                None
            }
            _ => return Err("unsupported generated file path".into()),
        };
        if let Some(directory) = directory {
            if child.is_some_and(|existing| existing != directory) {
                return Err("mixed generated subdirectories are unsupported".into());
            }
            child = Some(directory);
        }
    }
    Ok(child.unwrap_or("src"))
}
