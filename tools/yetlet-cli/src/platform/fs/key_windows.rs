//! Key-only native NTFS transaction boundary. Existing general app readers stay unchanged.
use super::*;
fn fail() -> io::Error {
    io::Error::other("unsafe or changed key/settings file")
}
fn checked(file: &File, private: bool, writable: bool) -> io::Result<()> {
    inspect(file, false)?;
    protected(file, true)?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: valid handle and SDK output structure.
    win(unsafe { GetFileInformationByHandle(handle(file), &mut info) })?;
    if info.nNumberOfLinks != 1
        || (writable && info.dwFileAttributes & FILE_ATTRIBUTE_READONLY != 0)
    {
        return Err(fail());
    }
    if !private {
        return Ok(());
    }
    let current = private_security()?;
    let mut sid = null_mut();
    let mut defaulted = 0;
    win(unsafe { GetSecurityDescriptorOwner(current.0, &mut sid, &mut defaulted) })?;
    let mut dacl = null_mut();
    let mut sd = null_mut();
    let code = unsafe {
        GetSecurityInfo(
            handle(file),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut dacl,
            null_mut(),
            &mut sd,
        )
    };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    let _sd = Local(sd);
    if dacl.is_null() {
        return Err(fail());
    }
    for i in 0..unsafe { (*dacl).AceCount } {
        let mut ace = null_mut();
        win(unsafe { GetAce(dacl, i as u32, &mut ace) })?;
        let h = unsafe { &*ace.cast::<ACE_HEADER>() };
        if h.AceFlags as u32 & INHERIT_ONLY_ACE != 0 || h.AceType as u32 == ACCESS_DENIED_ACE_TYPE {
            continue;
        }
        if h.AceType as u32 != ACCESS_ALLOWED_ACE_TYPE {
            return Err(fail());
        }
        let offset = offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        if (h.AceSize as usize) < offset + 8 {
            return Err(fail());
        }
        let other = unsafe { ace.cast::<u8>().add(offset) };
        if offset + 8 + 4 * unsafe { *other.add(1) } as usize > h.AceSize as usize
            || unsafe { IsValidSid(other.cast()) } == 0
        {
            return Err(fail());
        }
        let mask = unsafe { (*ace.cast::<ACCESS_ALLOWED_ACE>()).Mask };
        if mask & (windows_sys::Win32::Foundation::GENERIC_READ | GENERIC_ALL | FILE_READ_DATA) != 0
            && !trusted_sid(other.cast(), sid, true)
        {
            return Err(fail());
        }
    }
    Ok(())
}
fn bytes(file: &File) -> io::Result<Vec<u8>> {
    let mut b = Vec::new();
    file.take(65537).read_to_end(&mut b)?;
    if b.len() > 65536 {
        return Err(fail());
    }
    Ok(b)
}
pub(crate) struct Directory {
    anchor: Anchor,
    path: PathBuf,
}
impl Directory {
    pub(crate) fn open(path: &Path, _private: bool) -> io::Result<Self> {
        let anchor = Anchor::open(path)?;
        protected(&anchor.security_directory()?, true)?;
        Ok(Self {
            anchor,
            path: path.into(),
        })
    }
    pub(crate) fn writable(&self) -> io::Result<()> {
        self.anchor.verify()?;
        let (name, _, id) = self.anchor.chain.last().ok_or_else(fail)?;
        let parent = if self.anchor.chain.len() == 1 {
            &self.anchor.root
        } else {
            &self.anchor.chain[self.anchor.chain.len() - 2].1
        };
        let f = relative_access(
            parent,
            name.as_ref(),
            true,
            false,
            None,
            READ_CONTROL | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | FILE_DELETE_CHILD,
        )?;
        if inspect(&f, true)? != *id {
            return Err(fail());
        }
        Ok(())
    }
    pub(crate) fn read(&self, n: &str, private: bool) -> io::Result<Vec<u8>> {
        self.anchor.verify()?;
        let f = relative_access(
            self.anchor.directory(),
            n.as_ref(),
            false,
            false,
            None,
            READ_CONTROL,
        )?;
        checked(&f, private, false)?;
        let b = bytes(&f)?;
        self.anchor.verify()?;
        Ok(b)
    }
    pub(crate) fn absent(&self, n: &str) -> io::Result<()> {
        self.anchor.verify()?;
        match relative(self.anchor.directory(), n.as_ref(), true, false, None) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            _ => Err(fail()),
        }
    }
    pub(crate) fn lock(&self) -> io::Result<File> {
        self.anchor.verify()?;
        let sd = private_security()?;
        let f = match relative_access(
            self.anchor.directory(),
            ".yetlet-key.lock".as_ref(),
            false,
            true,
            Some(&sd),
            READ_CONTROL,
        ) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => relative_access(
                self.anchor.directory(),
                ".yetlet-key.lock".as_ref(),
                false,
                false,
                None,
                READ_CONTROL | FILE_GENERIC_WRITE,
            )?,
            Err(e) => return Err(e),
        };
        checked(&f, true, true)?;
        f.try_lock()
            .map_err(|_| io::Error::other("another key operation holds settings lock"))?;
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
    id: Identity,
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
        let f = relative_access(
            dir.anchor.directory(),
            n.as_ref(),
            false,
            false,
            None,
            READ_CONTROL
                | if writable {
                    FILE_GENERIC_WRITE | DELETE
                } else {
                    0
                },
        )?;
        checked(&f, false, writable)?;
        let id = inspect(&f, false)?;
        let b = bytes(&f)?;
        Ok(Self {
            dir,
            name: n.into(),
            id,
            bytes: b,
            writable,
        })
    }
    pub(crate) fn ensure(&self) -> io::Result<()> {
        self.dir.anchor.verify()?;
        let f = relative_access(
            self.dir.anchor.directory(),
            self.name.as_ref(),
            false,
            false,
            None,
            READ_CONTROL,
        )?;
        checked(&f, false, self.writable)?;
        if inspect(&f, false)? != self.id || bytes(&f)? != self.bytes {
            return Err(fail());
        }
        Ok(())
    }
    pub(crate) fn replace(&self, data: &[u8]) -> io::Result<()> {
        self.ensure()?;
        if !self.writable || data.len() > 65536 {
            return Err(fail());
        }
        let sd = private_security()?;
        let tmp = format!(".yetlet-key-settings-{}", std::process::id());
        let mut f = relative(
            self.dir.anchor.directory(),
            tmp.as_ref(),
            false,
            true,
            Some(&sd),
        )?;
        let result = (|| {
            f.write_all(data)?;
            f.sync_all()?;
            self.ensure()?;
            let name = component(self.name.as_ref())?;
            let length = (offset_of!(FILE_RENAME_INFORMATION, FileName) + name.len() * 2)
                .max(size_of::<FILE_RENAME_INFORMATION>());
            let mut storage = vec![0_usize; length.div_ceil(size_of::<usize>())];
            let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
            // SAFETY: aligned zeroed SDK record and allocated UTF16 tail. Replace only selected settings.
            unsafe {
                (*info).Anonymous.ReplaceIfExists = true;
                (*info).RootDirectory = handle(self.dir.anchor.directory());
                (*info).FileNameLength = (name.len() * 2) as u32;
                std::ptr::copy_nonoverlapping(
                    name.as_ptr(),
                    (*info).FileName.as_mut_ptr(),
                    name.len(),
                );
            }
            let mut status = IO_STATUS_BLOCK::default();
            nt(unsafe {
                NtSetInformationFile(
                    handle(&f),
                    &mut status,
                    info.cast(),
                    length as u32,
                    FileRenameInformation,
                )
            })?;
            self.dir.anchor.verify()
        })();
        // Deletion is by exact created handle only; never remove a possibly published settings file.
        if result.is_err() {
            if let Ok(current) = relative(
                self.dir.anchor.directory(),
                tmp.as_ref(),
                false,
                false,
                None,
            ) {
                if inspect(&current, false)? == inspect(&f, false)? {
                    let _ = remove(&f);
                }
            }
        }
        result
    }
}
