//! Windows and other OSes remain fail-closed pending separately reviewed adapters.
use super::UNSUPPORTED_CREATE;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

pub(crate) const NEW_SUPPORTED: bool = false;
pub(crate) const NEW_DIAGNOSTIC: &str =
    "new is unsupported on this platform and fails closed; no Windows guarantee is claimed";
const UNSUPPORTED_APP: &str = "build/dev are unsupported on this platform: native cwd/process ownership Not verified; no child started";

pub(crate) struct App;
pub(super) struct AppFiles;
impl AppFiles {
    pub(super) fn open(_manifest: &Path) -> Result<Self, String> {
        Err(UNSUPPORTED_APP.into())
    }
    pub(super) fn read(&self, _name: &str) -> Result<File, String> {
        Err(UNSUPPORTED_APP.into())
    }
    pub(super) fn finish(self) -> Result<App, String> {
        Err(UNSUPPORTED_APP.into())
    }
}

pub(crate) fn create_project(
    _parent: &Path,
    _name: &str,
    _files: &[(&str, String)],
) -> Result<(), String> {
    Err(UNSUPPORTED_CREATE.into())
}

// Preserve the prior doctor-only probe; this is not secure Windows app preflight.
pub(crate) fn read_framework(path: &Path) -> io::Result<Vec<u8>> {
    let mut prefix = PathBuf::new();
    for part in path.components() {
        prefix.push(part);
        if fs::symlink_metadata(&prefix)?.file_type().is_symlink() {
            return Err(io::Error::other("framework symlinks are unsupported"));
        }
    }
    fs::read(path)
}
