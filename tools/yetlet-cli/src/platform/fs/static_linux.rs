//! Static export's scoped regular-file and no-replace operations.
use super::*;
use std::{
    ffi::{CString, OsStr, OsString},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
};

fn name(value: &OsStr) -> io::Result<CString> {
    if value.is_empty()
        || value.as_bytes().iter().any(|b| matches!(b, b'/' | 0))
        || value == "."
        || value == ".."
    {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    CString::new(value.as_bytes()).map_err(io::Error::other)
}
fn checked(app: &App) -> io::Result<()> {
    verify_app(app).map_err(io::Error::other)
}
fn regular(file: &File) -> io::Result<()> {
    let m = file.metadata()?;
    if !m.is_file() || m.nlink() != 1 || m.uid() != unsafe { libc::geteuid() } {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    Ok(())
}
pub(crate) fn read(app: &App, key: &OsStr, limit: usize) -> io::Result<Vec<u8>> {
    checked(app)?;
    let mut f = app.parent.read_file(key)?;
    regular(&f)?;
    let before = f.metadata()?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut f)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    let after = f.metadata()?;
    let actual = app.parent.read_file(key)?;
    if bytes.len() > limit
        || (
            before.dev(),
            before.ino(),
            before.len(),
            before.mtime(),
            before.mtime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.len(),
            after.mtime(),
            after.mtime_nsec(),
        )
        || (after.dev(), after.ino()) != (actual.metadata()?.dev(), actual.metadata()?.ino())
    {
        return Err(io::Error::other(
            "static input changed or exceeded its limit",
        ));
    }
    checked(app)?;
    Ok(bytes)
}
pub(crate) fn write(app: &App, key: &OsStr, bytes: &[u8]) -> io::Result<()> {
    checked(app)?;
    let key = name(key)?;
    let fd = unsafe {
        libc::openat(
            app.directory.as_raw_fd(),
            key.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    regular(&file)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    checked(app)
}
pub(crate) fn list(app: &App) -> io::Result<Vec<(OsString, bool)>> {
    checked(app)?;
    let mut result = Vec::new();
    for e in std::fs::read_dir(format!("/proc/self/fd/{}", app.directory.as_raw_fd()))? {
        let e = e?;
        if result.len() >= 1536 {
            return Err(io::Error::other("static directory entry limit"));
        }
        result.push((e.file_name(), e.file_type()?.is_dir()));
    }
    checked(app)?;
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
pub(crate) fn absent(app: &App, key: &OsStr) -> io::Result<()> {
    checked(app)?;
    let key = name(key)?;
    let fd = unsafe {
        libc::openat(
            app.directory.as_raw_fd(),
            key.as_ptr(),
            libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd >= 0 {
        drop(unsafe { File::from_raw_fd(fd) });
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    let e = io::Error::last_os_error();
    if e.kind() == io::ErrorKind::NotFound {
        Ok(())
    } else {
        Err(e)
    }
}
pub(crate) fn publish(stage: App, parent: &App, key: &OsStr) -> io::Result<()> {
    checked(&stage)?;
    checked(parent)?;
    if stage.cwd.parent() != Some(parent.cwd.as_path()) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let from = name(stage.cwd.file_name().ok_or(io::ErrorKind::InvalidInput)?)?;
    let to = name(key)?;
    let id = stage.directory.metadata()?;
    stage.directory.sync_all()?;
    let r = unsafe {
        libc::renameat2(
            parent.directory.as_raw_fd(),
            from.as_ptr(),
            parent.directory.as_raw_fd(),
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if r != 0 {
        return Err(io::Error::last_os_error());
    }
    let target = open_directory(&parent.cwd.join(key)).map_err(io::Error::other)?;
    let actual = target.directory.metadata()?;
    if (id.dev(), id.ino()) != (actual.dev(), actual.ino()) {
        return Err(io::Error::other(
            "published static directory identity changed",
        ));
    }
    parent.directory.sync_all()?;
    checked(parent)
}
pub(crate) fn remove_file(app: &App, key: &OsStr) -> io::Result<()> {
    checked(app)?;
    let f = app.parent.read_file(key)?;
    regular(&f)?;
    let key = name(key)?;
    let r = unsafe { libc::unlinkat(app.directory.as_raw_fd(), key.as_ptr(), 0) };
    if r == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
pub(crate) fn remove_dir(app: App) -> io::Result<()> {
    checked(&app)?;
    let parent = open_directory(app.cwd.parent().ok_or(io::ErrorKind::InvalidInput)?)
        .map_err(io::Error::other)?;
    let key = name(app.cwd.file_name().ok_or(io::ErrorKind::InvalidInput)?)?;
    let r = unsafe {
        libc::unlinkat(
            parent.directory.as_raw_fd(),
            key.as_ptr(),
            libc::AT_REMOVEDIR,
        )
    };
    if r == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
