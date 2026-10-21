use std::{fs::File, io, path::Path};
fn unsupported<T>() -> io::Result<T> {
    Err(io::Error::other(
        "key administration unsupported on this platform",
    ))
}
pub(crate) struct Directory;
impl Directory {
    pub(crate) fn open(_: &Path, _: bool) -> io::Result<Self> {
        unsupported()
    }
    pub(crate) fn writable(&self) -> io::Result<()> {
        unsupported()
    }
    pub(crate) fn read(&self, _: &str, _: bool) -> io::Result<Vec<u8>> {
        unsupported()
    }
    pub(crate) fn absent(&self, _: &str) -> io::Result<()> {
        unsupported()
    }
    pub(crate) fn lock(&self) -> io::Result<File> {
        unsupported()
    }
    pub(crate) fn publish(&self, _: &str, _: &[(&str, String)]) -> io::Result<()> {
        unsupported()
    }
}
pub(crate) struct Snapshot {
    pub(crate) bytes: Vec<u8>,
}
impl Snapshot {
    pub(crate) fn open(_: &Path, _: bool) -> io::Result<Self> {
        unsupported()
    }
    pub(crate) fn ensure(&self) -> io::Result<()> {
        unsupported()
    }
    pub(crate) fn replace(&self, _: &[u8]) -> io::Result<()> {
        unsupported()
    }
}
