//! Key-only descriptor-relative files, confidentiality checks and atomic settings replacement.
use super::safe_dir::Anchor;
use std::{
    ffi::{CString, OsStr},
    fs::File,
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Path, PathBuf},
};

fn fail() -> io::Error {
    io::Error::other("unsafe or changed key/settings file")
}
fn name(value: &OsStr) -> io::Result<CString> {
    let b = value.as_bytes();
    if b.is_empty() || b == b"." || b == b".." || b.contains(&b'/') {
        return Err(fail());
    }
    CString::new(b).map_err(|_| fail())
}
fn open(parent: &File, n: &OsStr, flags: i32) -> io::Result<File> {
    let n = name(n)?;
    // SAFETY: live directory handle and validated single terminated component.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            n.as_ptr(),
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            0o600,
        )
    };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}
fn checked(file: &File, private: bool, write: bool) -> io::Result<()> {
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & (if private { 0o077 } else { 0o022 }) != 0
        || (write && m.mode() & 0o200 == 0)
    {
        return Err(fail());
    }
    Ok(())
}
fn bytes(file: &File) -> io::Result<Vec<u8>> {
    let mut result = Vec::new();
    // Independent open descriptions are supplied by callers; seek clones would share offsets.
    file.take(65537).read_to_end(&mut result)?;
    if result.len() > 65536 {
        return Err(fail());
    }
    Ok(result)
}
pub(crate) struct Directory {
    anchor: Anchor,
    fd: File,
    path: PathBuf,
}
impl Directory {
    pub(crate) fn open(path: &Path, private: bool) -> io::Result<Self> {
        let anchor = Anchor::open(path)?;
        let fd = anchor.working_directory()?;
        let m = fd.metadata()?;
        if m.uid() != unsafe { libc::geteuid() }
            || m.mode() & (if private { 0o077 } else { 0o022 }) != 0
        {
            return Err(fail());
        }
        Ok(Self {
            anchor,
            fd,
            path: path.into(),
        })
    }
    pub(crate) fn writable(&self) -> io::Result<()> {
        self.anchor.verify()?;
        if self.fd.metadata()?.mode() & 0o200 == 0 {
            return Err(fail());
        }
        Ok(())
    }
    pub(crate) fn read(&self, n: &str, private: bool) -> io::Result<Vec<u8>> {
        self.anchor.verify()?;
        let f = open(&self.fd, n.as_ref(), libc::O_RDONLY)?;
        checked(&f, private, false)?;
        let b = bytes(&f)?;
        self.anchor.verify()?;
        Ok(b)
    }
    pub(crate) fn absent(&self, n: &str) -> io::Result<()> {
        self.anchor.verify()?;
        let n = name(n.as_ref())?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: valid descriptor, component and output; AT_SYMLINK_NOFOLLOW observes links too.
        if unsafe {
            libc::fstatat(
                self.fd.as_raw_fd(),
                n.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
        {
            return Err(fail());
        }
        let e = io::Error::last_os_error();
        if e.kind() == io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(e)
        }
    }
    pub(crate) fn lock(&self) -> io::Result<File> {
        self.anchor.verify()?;
        let f = open(
            &self.fd,
            ".yetlet-key.lock".as_ref(),
            libc::O_RDWR | libc::O_CREAT,
        )?;
        checked(&f, true, true)?;
        f.try_lock()
            .map_err(|_| io::Error::other("another key operation holds the settings lock"))?;
        Ok(f)
    }
    pub(crate) fn publish(&self, n: &str, entries: &[(&str, String)]) -> io::Result<()> {
        self.anchor.verify()?;
        super::create_project(&self.path, n, entries).map_err(io::Error::other)?;
        self.anchor.verify()
    }
}
pub(crate) struct Snapshot {
    dir: Directory,
    name: String,
    dev: u64,
    ino: u64,
    pub(crate) bytes: Vec<u8>,
    writable: bool,
}
impl Snapshot {
    pub(crate) fn open(path: &Path, writable: bool) -> io::Result<Self> {
        let dir = Directory::open(path.parent().ok_or_else(fail)?, false)?;
        if writable {
            dir.writable()?;
        }
        let n = path.file_name().and_then(OsStr::to_str).ok_or_else(fail)?;
        let f = open(
            &dir.fd,
            n.as_ref(),
            if writable {
                libc::O_RDWR
            } else {
                libc::O_RDONLY
            },
        )?;
        checked(&f, false, writable)?;
        let m = f.metadata()?;
        Ok(Self {
            dir,
            name: n.into(),
            dev: m.dev(),
            ino: m.ino(),
            bytes: bytes(&f)?,
            writable,
        })
    }
    pub(crate) fn ensure(&self) -> io::Result<()> {
        self.dir.anchor.verify()?;
        let f = open(&self.dir.fd, self.name.as_ref(), libc::O_RDONLY)?;
        checked(&f, false, self.writable)?;
        let m = f.metadata()?;
        if m.dev() != self.dev || m.ino() != self.ino || bytes(&f)? != self.bytes {
            return Err(fail());
        }
        Ok(())
    }
    pub(crate) fn replace(&self, data: &[u8]) -> io::Result<()> {
        self.ensure()?;
        if !self.writable || data.len() > 65536 {
            return Err(fail());
        }
        let tmp = format!(".yetlet-key-settings-{}", std::process::id());
        let mut f = open(
            &self.dir.fd,
            tmp.as_ref(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
        )?;
        let owned = f.metadata()?;
        let result = (|| {
            f.write_all(data)?;
            f.sync_all()?;
            self.ensure()?;
            let src = name(tmp.as_ref())?;
            let dst = name(self.name.as_ref())?;
            // SAFETY: both names in the pinned protected directory; same-filesystem atomic rename.
            if unsafe {
                libc::renameat(
                    self.dir.fd.as_raw_fd(),
                    src.as_ptr(),
                    self.dir.fd.as_raw_fd(),
                    dst.as_ptr(),
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
            self.dir.fd.sync_all()?;
            self.dir.anchor.verify()
        })();
        if result.is_err()
            && let Ok(actual) = open(&self.dir.fd, tmp.as_ref(), libc::O_RDONLY)
        {
            let m = actual.metadata()?;
            if m.dev() == owned.dev() && m.ino() == owned.ino() {
                let n = name(tmp.as_ref())?;
                // SAFETY: verified exact owned temporary identity; no recursive deletion.
                unsafe { libc::unlinkat(self.dir.fd.as_raw_fd(), n.as_ptr(), 0) };
            }
        }
        result
    }
}
