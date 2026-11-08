//! Same-directory replacement only for the styles publisher, under trusted ownership.
use super::*;
use std::{
    ffi::{CString, OsStr},
    io::Write,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
};

pub(crate) fn replace(
    app: &App,
    key: &OsStr,
    old: Option<&[u8]>,
    data: &[u8],
    replaced: &mut bool,
) -> io::Result<()> {
    let check = || match old {
        Some(bytes) if static_output::read(app, key, 2 * 1024 * 1024)? == bytes => Ok(()),
        Some(_) => Err(io::ErrorKind::InvalidData.into()),
        None => static_output::absent(app, key),
    };
    check()?;
    if old == Some(data) {
        return Ok(());
    }
    let temporary = format!(".yetlet-style-{}", std::process::id());
    let src = CString::new(temporary.as_bytes())?;
    // Exclusive creation proves ownership before any fallible write/sync/check.
    // A preexisting unknown temporary is never opened for writing or removed.
    let fd = unsafe {
        libc::openat(
            app.directory.as_raw_fd(),
            src.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let result = (|| {
        file.write_all(data)?;
        #[cfg(test)]
        fail_temporary(1)?;
        file.sync_all()?;
        #[cfg(test)]
        fail_temporary(2)?;
        verify_app(app).map_err(io::Error::other)?;
        check()?;
        let dst = CString::new(key.as_bytes())?;
        let flags = if old.is_none() {
            libc::RENAME_NOREPLACE
        } else {
            0
        };
        if unsafe {
            libc::renameat2(
                app.directory.as_raw_fd(),
                src.as_ptr(),
                app.directory.as_raw_fd(),
                dst.as_ptr(),
                flags,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        *replaced = true;
        #[cfg(test)]
        crate::platform::fs::styles_output::after_rename(key)?;
        app.directory.sync_all()?;
        verify_app(app).map_err(io::Error::other)
    })();
    if result.is_err() {
        // Only our exclusive temporary, never the destination.
        if let (Ok(current), Ok(owned)) =
            (app.parent.read_file(temporary.as_ref()), file.metadata())
        {
            if current
                .metadata()
                .is_ok_and(|m| (m.dev(), m.ino()) == (owned.dev(), owned.ino()))
            {
                let _ = static_output::remove_file(app, temporary.as_ref());
            }
        }
    }
    result
}

#[cfg(test)]
thread_local! { static FAIL_TEMPORARY: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
fn fail_temporary(stage: u8) -> io::Result<()> {
    if FAIL_TEMPORARY.with(|flag| {
        if flag.get() == stage {
            flag.set(0);
            true
        } else {
            false
        }
    }) {
        Err(io::Error::other("injected temporary write/sync failure"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_write_and_sync_failures_cleanup_and_same_process_recovers() {
        let temp = tempfile::tempdir().unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let app = open_directory(temp.path()).unwrap();
        for stage in [1, 2] {
            let mut replaced = false;
            FAIL_TEMPORARY.with(|flag| flag.set(stage));
            assert!(replace(&app, "app.css".as_ref(), None, b"a{}", &mut replaced).is_err());
            assert!(!replaced);
            assert!(
                !temp
                    .path()
                    .join(format!(".yetlet-style-{}", std::process::id()))
                    .exists()
            );
            replace(&app, "app.css".as_ref(), None, b"a{}", &mut replaced).unwrap();
            assert!(replaced);
            static_output::remove_file(&app, "app.css".as_ref()).unwrap();
        }
    }
    #[test]
    fn initially_unknown_temporary_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let app = open_directory(temp.path()).unwrap();
        let name = format!(".yetlet-style-{}", std::process::id());
        static_output::write(&app, name.as_ref(), b"unknown evidence").unwrap();
        let mut replaced = false;
        assert_eq!(
            replace(&app, "app.css".as_ref(), None, b"a{}", &mut replaced)
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(!replaced);
        assert_eq!(
            std::fs::read(temp.path().join(name)).unwrap(),
            b"unknown evidence"
        );
        assert!(!temp.path().join("app.css").exists());
    }
}
