//! Static export reuses the existing NTFS relative-handle/security boundary.
use super::*;
use std::ffi::OsString;
fn regular(file: &File) -> io::Result<()> {
    inspect(file, false)?;
    protected(file, true)?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    win(unsafe { GetFileInformationByHandle(handle(file), &mut info) })?;
    if info.nNumberOfLinks != 1 {
        return Err(io::Error::other("hard-linked static input"));
    }
    Ok(())
}
pub(crate) fn read(app: &App, key: &OsStr, limit: usize) -> io::Result<Vec<u8>> {
    app.verify()?;
    let mut file = relative_access(
        app.parent.directory(),
        key,
        false,
        false,
        None,
        READ_CONTROL,
    )?;
    regular(&file)?;
    let id = inspect(&file, false)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::other("static input exceeds limit"));
    }
    let actual = relative(app.parent.directory(), key, false, false, None)?;
    if inspect(&actual, false)? != id {
        return Err(io::Error::other("static file identity changed"));
    }
    app.verify()?;
    Ok(bytes)
}
pub(crate) fn write(app: &App, key: &OsStr, bytes: &[u8]) -> io::Result<()> {
    app.verify()?;
    let sd = private_security()?;
    let mut file = relative_access(
        app.parent.directory(),
        key,
        false,
        true,
        Some(&sd),
        READ_CONTROL,
    )?;
    regular(&file)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    app.verify()
}
pub(crate) fn list(app: &App) -> io::Result<Vec<(OsString, bool)>> {
    app.verify()?;
    let mut result = Vec::new();
    for e in std::fs::read_dir(&app.cwd)? {
        let e = e?;
        if result.len() >= 1536 {
            return Err(io::Error::other("static directory entry limit"));
        }
        result.push((e.file_name(), e.file_type()?.is_dir()));
    }
    app.verify()?;
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
pub(crate) fn absent(app: &App, key: &OsStr) -> io::Result<()> {
    app.verify()?;
    match relative(app.parent.directory(), key, false, false, None) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(io::ErrorKind::AlreadyExists.into()),
        Err(e) => Err(e),
    }
}
pub(crate) fn publish(stage: App, parent: &App, key: &OsStr) -> io::Result<()> {
    stage.verify()?;
    parent.verify()?;
    if stage.cwd.parent() != Some(parent.cwd.as_path()) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let from = stage
        .cwd
        .file_name()
        .ok_or(io::ErrorKind::InvalidInput)?
        .to_owned();
    let id = inspect(&stage.directory, true)?;
    // Protected parent remains pinned. Drop no-delete-sharing stage handles
    // before reopening the exact identity with DELETE for non-replacing rename.
    drop(stage);
    let file = open_delete(
        parent.parent.directory(),
        from.to_str().ok_or(io::ErrorKind::InvalidInput)?,
        true,
    )?;
    if inspect(&file, true)? != id {
        return Err(io::Error::other("static stage changed"));
    }
    rename(
        &file,
        parent.parent.directory(),
        key.to_str().ok_or(io::ErrorKind::InvalidInput)?,
    )?;
    drop(file);
    let target = open_directory(&parent.cwd.join(key)).map_err(io::Error::other)?;
    if inspect(&target.directory, true)? != id {
        return Err(io::Error::other("published static identity changed"));
    }
    parent.verify()
}
pub(crate) fn remove_file(app: &App, key: &OsStr) -> io::Result<()> {
    app.verify()?;
    let file = relative_access(
        app.parent.directory(),
        key,
        false,
        false,
        None,
        READ_CONTROL,
    )?;
    regular(&file)?;
    let id = inspect(&file, false)?;
    drop(file);
    let file = open_delete(
        app.parent.directory(),
        key.to_str().ok_or(io::ErrorKind::InvalidInput)?,
        false,
    )?;
    if inspect(&file, false)? != id {
        return Err(io::Error::other("static cleanup identity changed"));
    }
    remove(&file)
}
pub(crate) fn remove_dir(app: App) -> io::Result<()> {
    app.verify()?;
    let id = inspect(&app.directory, true)?;
    let parent = open_directory(app.cwd.parent().ok_or(io::ErrorKind::InvalidInput)?)
        .map_err(io::Error::other)?;
    let key = app
        .cwd
        .file_name()
        .ok_or(io::ErrorKind::InvalidInput)?
        .to_owned();
    drop(app);
    let file = open_delete(
        parent.parent.directory(),
        key.to_str().ok_or(io::ErrorKind::InvalidInput)?,
        true,
    )?;
    if inspect(&file, true)? != id {
        return Err(io::Error::other("static cleanup directory changed"));
    }
    remove(&file)
}
