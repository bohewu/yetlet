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
