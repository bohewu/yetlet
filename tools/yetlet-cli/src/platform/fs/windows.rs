//! Local fixed NTFS only. Native relative opens never resolve an unpinned path.
//! Creating the private stage and acquiring its handle is ONE FILE_CREATE call;
//! no mkdir/open substitution window exists, even in a writable parent.
use std::{
    cell::RefCell,
    ffi::OsStr,
    fs::File,
    io::{self, Read, Write},
    mem::{offset_of, size_of},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, PathBuf, Prefix},
    ptr::{null, null_mut},
};
use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            FILE_CREATE, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN,
            FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT,
            FileRenameInformation, NtCreateFile, NtSetInformationFile,
        },
    },
    Win32::{
        Foundation::{
            GENERIC_ALL, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
            OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE, RtlNtStatusToDosError, UNICODE_STRING,
        },
        Security::{
            ACCESS_ALLOWED_ACE, ACE_HEADER,
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
                GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
            },
            Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
            DACL_SECURITY_INFORMATION, EqualSid, GetAce, GetSecurityDescriptorOwner,
            GetTokenInformation, INHERIT_ONLY_ACE, IsValidAcl, IsValidSid, IsWellKnownSid,
            OWNER_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER, TokenUser,
            WinBuiltinAdministratorsSid, WinLocalSystemSid,
        },
        Storage::FileSystem::*,
        System::{
            IO::IO_STATUS_BLOCK,
            SystemServices::{ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE},
            Threading::{GetCurrentProcess, OpenProcessToken},
            WindowsProgramming::DRIVE_FIXED,
        },
    },
};

pub(crate) const NEW_SUPPORTED: bool = true;
pub(crate) const NEW_DIAGNOSTIC: &str = "Windows native handle/no-replace implementation available; new requires a local fixed NTFS volume without reparse traversal";
// Keep the complete no-delete-share chain AND both no-write/delete-share files
// alive until native Cargo and its Job are finished. Cargo receives only a cwd
// spelling whose directory entries cannot be replaced while these handles live.
pub(crate) struct App {
    parent: Anchor,
    directory: File,
    pub(in crate::platform) cwd: PathBuf,
    files: Vec<(String, File, Identity)>,
}
pub(super) struct AppFiles {
    app: App,
    files: RefCell<Vec<(String, File, Identity)>>,
}
impl AppFiles {
    pub(super) fn open(manifest: &Path) -> Result<Self, String> {
        let manifest = crate::framework::absolute(manifest)
            .map_err(|_| "manifest path must be nonempty and contain no '..'")?;
        if manifest.file_name() != Some("Cargo.toml".as_ref()) {
            return Err("--manifest-path must select Cargo.toml".into());
        }
        let cwd = manifest
            .parent()
            .ok_or("manifest has no parent")?
            .to_owned();
        Ok(Self {
            app: open_directory(&cwd)?,
            files: RefCell::new(Vec::new()),
        })
    }
    pub(super) fn read(&self, name: &str) -> Result<File, String> {
        let file = relative_access(self.app.parent.directory(), name.as_ref(), false, false, None, READ_CONTROL)
            .map_err(|_| format!("{name} must already exist as a regular, non-reparse app file; create the app lock separately with cargo generate-lockfile --offline"))?;
        protected(&file, true).map_err(|_| {
            format!("{name} must be owned by the current user with no untrusted writable ACL")
        })?;
        let id = inspect(&file, false).map_err(|_| "cannot inspect app file")?;
        let reader = file
            .try_clone()
            .map_err(|_| "cannot read pinned app file")?;
        self.files.borrow_mut().push((name.into(), file, id));
        Ok(reader)
    }
    pub(super) fn finish(mut self) -> Result<App, String> {
        self.app.files = self.files.into_inner();
        self.app
            .verify()
            .map_err(|_| "app cwd/file identity or security changed")?;
        Ok(self.app)
    }
}

pub(crate) fn app_root(app: &App) -> &Path {
    &app.cwd
}

pub(super) fn verify_app(app: &App) -> Result<(), String> {
    app.verify()
        .map_err(|_| "app cwd/file identity or security changed".into())
}

pub(crate) fn open_directory(path: &Path) -> Result<App, String> {
    if !path.is_absolute() {
        return Err("protected directory path must be absolute".into());
    }
    let cwd = crate::framework::absolute(path)
        .map_err(|_| "directory path must be nonempty and contain no '..'")?;
    let parent = Anchor::open(&cwd)
        .map_err(|_| "app cwd is missing or unsafe (reparse paths are forbidden)")?;
    let directory = parent
        .security_directory()
        .map_err(|_| "cannot pin app cwd security")?;
    protected(&directory, true).map_err(
        |_| "app cwd must be owned by the current user and have no untrusted writable ACL",
    )?;
    Ok(App {
        parent,
        directory,
        cwd,
        files: Vec::new(),
    })
}

impl App {
    pub(in crate::platform) fn verify(&self) -> io::Result<()> {
        self.parent.verify()?;
        if inspect(&self.directory, true)? != inspect(self.parent.directory(), true)? {
            return Err(io::Error::other("app cwd identity changed"));
        }
        protected(&self.directory, true)?;
        for (name, file, id) in &self.files {
            let actual = relative(self.parent.directory(), name.as_ref(), false, false, None)?;
            if inspect(&actual, false)? != *id || inspect(file, false)? != *id {
                return Err(io::Error::other("app file identity changed"));
            }
            protected(file, true)?;
        }
        Ok(())
    }
}

pub(crate) fn app_entries(app: &App, child: bool) -> Result<Vec<String>, String> {
    app_directory_entries(app, if child { "src" } else { "" })
}

pub(crate) fn app_directory_entries(app: &App, relative: &str) -> Result<Vec<String>, String> {
    super::app_directory(relative)?;
    app.verify().map_err(|_| "app identity changed")?;
    let path = app.cwd.join(relative);
    let anchor = Anchor::open(&path).map_err(|_| "app directory is unavailable or unsafe")?;
    let security = anchor
        .security_directory()
        .map_err(|_| "cannot inspect app directory")?;
    protected(&security, true).map_err(|_| "app directory must be owned and protected")?;
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|_| "cannot enumerate pinned app directory")? {
        if names.len() >= 256 {
            return Err("app directory exceeds entry limit".into());
        }
        names.push(
            entry
                .map_err(|_| "cannot enumerate app")?
                .file_name()
                .into_string()
                .map_err(|_| "app entry names must be Unicode")?,
        );
    }
    anchor
        .verify()
        .map_err(|_| "app directory identity changed")?;
    app.verify().map_err(|_| "app identity changed")?;
    names.sort();
    Ok(names)
}

pub(crate) fn read_app_file(app: &App, name: &str) -> Result<Vec<u8>, String> {
    read_app_input(app, name).map(|(bytes, _)| bytes)
}

pub(crate) fn read_app_input(
    app: &App,
    name: &str,
) -> Result<(Vec<u8>, std::time::SystemTime), String> {
    app.verify().map_err(|_| "app identity changed")?;
    let (directory, name) = super::app_input(name)?;
    let path = app.cwd.join(directory);
    let anchor = Anchor::open(&path).map_err(|_| "app directory is unavailable or unsafe")?;
    let security = anchor
        .security_directory()
        .map_err(|_| "cannot inspect app directory")?;
    protected(&security, true).map_err(|_| "app directory must be owned and protected")?;
    let mut file = relative_access(
        anchor.directory(),
        name.as_ref(),
        false,
        false,
        None,
        READ_CONTROL,
    )
    .map_err(|_| "app input is unavailable or unsafe")?;
    protected(&file, true).map_err(|_| "app input must be owned and protected")?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read app input")?;
    if bytes.len() > 1024 * 1024 {
        return Err("app input exceeds size limit".into());
    }
    anchor
        .verify()
        .map_err(|_| "app directory identity changed")?;
    app.verify().map_err(|_| "app identity changed")?;
    let modified = file
        .metadata()
        .and_then(|m| m.modified())
        .map_err(|_| "cannot inspect app input time")?;
    Ok((bytes, modified))
}

// An explicit executable path, not CreateProcess's app-cwd-first search. Pin its
// ancestry and bytes against substitution for the whole child lifetime too.
pub(in crate::platform) struct Executable {
    pub(in crate::platform) path: PathBuf,
    parent: Anchor,
    file: File,
    id: Identity,
}
impl Executable {
    pub(in crate::platform) fn open(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(io::Error::other("executable path must be absolute"));
        }
        let parent = Anchor::open(
            path.parent()
                .ok_or_else(|| io::Error::other("missing executable parent"))?,
        )?;
        let file = relative_access(
            parent.directory(),
            path.file_name()
                .ok_or_else(|| io::Error::other("missing executable name"))?,
            false,
            false,
            None,
            READ_CONTROL,
        )?;
        protected(&file, false)?;
        let id = inspect(&file, false)?;
        Ok(Self {
            path: path.to_owned(),
            parent,
            file,
            id,
        })
    }
    pub(in crate::platform) fn verify(&self) -> io::Result<()> {
        self.parent.verify()?;
        let actual = relative(
            self.parent.directory(),
            self.path.file_name().unwrap(),
            false,
            false,
            None,
        )?;
        if inspect(&actual, false)? != self.id || inspect(&self.file, false)? != self.id {
            return Err(io::Error::other("executable identity changed"));
        }
        protected(&self.file, false)
    }
}

fn trusted_sid(
    sid: windows_sys::Win32::Security::PSID,
    current: windows_sys::Win32::Security::PSID,
    user_only: bool,
) -> bool {
    // SAFETY: callers pass validated SIDs owned by live SDK descriptors.
    unsafe {
        EqualSid(sid, current) != 0
            || (!user_only
                && (IsWellKnownSid(sid, WinLocalSystemSid) != 0
                    || IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0))
    }
}

fn protected(file: &File, require_user_owner: bool) -> io::Result<()> {
    let current = private_security()?;
    let mut current_sid = null_mut();
    let mut defaulted = 0;
    // SAFETY: initialized owned descriptor and valid outputs; SID borrows current.
    win(unsafe { GetSecurityDescriptorOwner(current.0, &mut current_sid, &mut defaulted) })?;
    let mut owner = null_mut();
    let mut dacl = null_mut();
    let mut sd = null_mut();
    // SAFETY: handle has READ_CONTROL; all returned pointers borrow LocalAlloc sd.
    let code = unsafe {
        GetSecurityInfo(
            handle(file),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
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
    // SAFETY: owner/DACL belong to the returned security descriptor.
    if owner.is_null()
        || dacl.is_null()
        || unsafe { IsValidSid(owner) == 0 || IsValidAcl(dacl) == 0 }
        || !trusted_sid(owner, current_sid, require_user_owner)
    {
        return Err(io::Error::other("untrusted owner or missing/invalid DACL"));
    }
    let mutating = GENERIC_ALL
        | GENERIC_WRITE
        | WRITE_DAC
        | WRITE_OWNER
        | DELETE
        | FILE_WRITE_DATA
        | FILE_APPEND_DATA
        | FILE_WRITE_EA
        | FILE_WRITE_ATTRIBUTES
        | FILE_DELETE_CHILD;
    for index in 0..unsafe { (*dacl).AceCount } {
        let mut ace = null_mut();
        // SAFETY: valid DACL, bounded index and output pointer.
        win(unsafe { GetAce(dacl, index as u32, &mut ace) })?;
        let header = unsafe { &*ace.cast::<ACE_HEADER>() };
        if header.AceFlags as u32 & INHERIT_ONLY_ACE != 0 {
            continue;
        }
        match header.AceType as u32 {
            ACCESS_DENIED_ACE_TYPE => continue,
            ACCESS_ALLOWED_ACE_TYPE => {}
            _ => return Err(io::Error::other("unsupported security ACE")),
        }
        let sid_offset = offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        if (header.AceSize as usize) < sid_offset + 8 {
            return Err(io::Error::other("invalid security ACE"));
        }
        let allowed = ace.cast::<ACCESS_ALLOWED_ACE>();
        let sid = unsafe { ace.cast::<u8>().add(sid_offset) };
        // Check the variable SID tail before asking the SDK to inspect it.
        if sid_offset + 8 + 4 * unsafe { *sid.add(1) } as usize > header.AceSize as usize
            || unsafe { IsValidSid(sid.cast()) == 0 }
            || (unsafe { (*allowed).Mask } & mutating != 0
                && !trusted_sid(sid.cast(), current_sid, false))
        {
            return Err(io::Error::other("untrusted writable ACL"));
        }
    }
    Ok(())
}

fn win(ok: i32) -> io::Result<()> {
    if ok != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
fn nt(status: i32) -> io::Result<()> {
    if status == 0 {
        Ok(())
    } else {
        // SAFETY: pure status conversion; no pointers or resources.
        Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(status) } as i32,
        ))
    }
}
fn handle(file: &File) -> HANDLE {
    file.as_raw_handle()
}

// LocalAlloc-owned strings and security descriptors returned by the SDK.
struct Local(*mut core::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: each successful SDK allocation is adopted exactly once.
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn private_security() -> io::Result<Local> {
    let mut token = null_mut();
    // SAFETY: live process pseudo-handle and valid output pointer.
    win(unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) })?;
    // SAFETY: successful OpenProcessToken transfers ownership to this RAII handle.
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut bytes = 0;
    // SAFETY: size query uses a null zero-length buffer.
    unsafe {
        GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut bytes);
    }
    if bytes < size_of::<TOKEN_USER>() as u32 {
        return Err(io::Error::last_os_error());
    }
    // usize storage supplies TOKEN_USER alignment and owns the embedded SID bytes.
    let mut storage = vec![0_usize; (bytes as usize).div_ceil(size_of::<usize>())];
    // SAFETY: aligned output buffer has at least the requested byte length.
    win(unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            bytes,
            &mut bytes,
        )
    })?;
    // SAFETY: successful TokenUser query initialized this aligned record.
    let sid = unsafe { (*(storage.as_ptr().cast::<TOKEN_USER>())).User.Sid };
    let mut text = null_mut();
    // SAFETY: SID lives in storage for this call; output is LocalAlloc-owned.
    win(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
    let text_owner = Local(text.cast());
    let mut len = 0;
    // SAFETY: SDK returns a terminated SID string. SID strings have bounded size.
    while unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) })
        .map_err(|_| io::Error::other("invalid token SID"))?;
    drop(text_owner);
    // Protected DACL: only this user; inheritable on owned src/files. Explicit
    // owner avoids relying on a token's default owner or parent inherited ACL.
    let sddl: Vec<_> = format!("O:{sid}D:P(A;OICI;FA;;;{sid})")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut descriptor = null_mut();
    // SAFETY: terminated SDDL and valid output pointer. Local frees the result.
    win(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    })?;
    Ok(Local(descriptor))
}

fn component(name: &OsStr) -> io::Result<Vec<u16>> {
    let text = name
        .to_str()
        .ok_or_else(|| io::Error::other("path must be Unicode"))?;
    let stem = text
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let device = matches!(
        stem.as_str(),
        "con" | "prn" | "aux" | "nul" | "conin$" | "conout$"
    ) || ["com", "lpt"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    });
    let wide: Vec<_> = name.encode_wide().collect();
    if text.is_empty()
        || text == "."
        || text == ".."
        || device
        || text.ends_with(['.', ' '])
        || wide.len() > 255
        || text.chars().any(|c| c < ' ' || "\\/:*?\"<>|".contains(c))
    {
        return Err(io::Error::other("ambiguous Windows path component"));
    }
    Ok(wide)
}

// dir/read/create are the only policies needed by this slice. No raw manual FFI.
fn relative(
    parent: &File,
    name: &OsStr,
    dir: bool,
    create: bool,
    security: Option<&Local>,
) -> io::Result<File> {
    relative_access(parent, name, dir, create, security, 0)
}

fn relative_access(
    parent: &File,
    name: &OsStr,
    dir: bool,
    create: bool,
    security: Option<&Local>,
    extra_access: u32,
) -> io::Result<File> {
    let mut name = component(name)?;
    let bytes = (name.len() * 2) as u16;
    let name = UNICODE_STRING {
        Length: bytes,
        MaximumLength: bytes,
        Buffer: name.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: handle(parent),
        ObjectName: &name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        SecurityDescriptor: security.map_or(null(), |s| s.0.cast()),
        SecurityQualityOfService: null(),
    };
    let access = if dir {
        FILE_TRAVERSE | FILE_READ_ATTRIBUTES
    } else {
        FILE_GENERIC_READ
    };
    let access = access
        | extra_access
        | SYNCHRONIZE
        | if create {
            DELETE | if dir { 0 } else { FILE_GENERIC_WRITE }
        } else {
            0
        };
    let mut result = null_mut();
    let mut status = IO_STATUS_BLOCK::default();
    // SAFETY: single validated relative component, live pinned parent and optional
    // SD; synchronous call; output handle is adopted once, never inherited.
    let code = unsafe {
        NtCreateFile(
            &mut result,
            access,
            &attributes,
            &mut status,
            null(),
            FILE_ATTRIBUTE_NORMAL,
            if dir {
                FILE_SHARE_READ | FILE_SHARE_WRITE
            } else {
                FILE_SHARE_READ
            },
            if create { FILE_CREATE } else { FILE_OPEN },
            FILE_SYNCHRONOUS_IO_NONALERT
                | FILE_OPEN_REPARSE_POINT
                | if dir {
                    FILE_DIRECTORY_FILE
                } else {
                    FILE_NON_DIRECTORY_FILE
                },
            null(),
            0,
        )
    };
    nt(code)?;
    // SAFETY: successful synchronous NtCreateFile returned an owned file handle.
    let file = unsafe { File::from_raw_handle(result) };
    if let Err(error) = inspect(&file, dir) {
        if create {
            let _ = remove(&file);
        }
        return Err(error);
    }
    Ok(file)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    volume: u64,
    id: [u8; 16],
}
fn inspect(file: &File, dir: bool) -> io::Result<Identity> {
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: valid handle and correctly sized output record.
    win(unsafe { GetFileInformationByHandle(handle(file), &mut info) })?;
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != dir
    {
        return Err(io::Error::other(
            "reparse points or unexpected object types are forbidden",
        ));
    }
    let mut id = FILE_ID_INFO::default();
    // SAFETY: SDK class and output record/size match.
    win(unsafe {
        GetFileInformationByHandleEx(
            handle(file),
            FileIdInfo,
            (&mut id as *mut FILE_ID_INFO).cast(),
            size_of::<FILE_ID_INFO>() as u32,
        )
    })?;
    Ok(Identity {
        volume: id.VolumeSerialNumber,
        id: id.FileId.Identifier,
    })
}

struct Anchor {
    root: File,
    chain: Vec<(String, File, Identity)>,
}
impl Anchor {
    fn open(path: &Path) -> io::Result<Self> {
        let path = crate::framework::absolute(path)?;
        let mut parts = path.components();
        let drive = match parts.next() {
            Some(Component::Prefix(p)) => match p.kind() {
                Prefix::Disk(letter) => letter,
                _ => {
                    return Err(io::Error::other(
                        "UNC/device/verbatim paths are unsupported",
                    ));
                }
            },
            _ => return Err(io::Error::other("expected absolute drive path")),
        };
        if parts.next() != Some(Component::RootDir) {
            return Err(io::Error::other("drive-relative paths are unsupported"));
        }
        let root_path: Vec<_> = format!("{}:\\", drive as char)
            .encode_utf16()
            .chain([0])
            .collect();
        // SAFETY: terminated drive-root path.
        if unsafe { GetDriveTypeW(root_path.as_ptr()) } != DRIVE_FIXED {
            return Err(io::Error::other(
                "only local fixed NTFS volumes are supported",
            ));
        }
        // Deny delete sharing for EVERY ancestor. Share-write is required by the
        // OS's rename target-directory open (FILE_RENAME_INFORMATION remarks).
        // All traversal below is relative/no-reparse; it never re-resolves a DOS path.
        let extended: Vec<_> = format!("\\\\?\\{}:\\", drive as char)
            .encode_utf16()
            .chain([0])
            .collect();
        // SAFETY: root-only extended path; directory/non-following open flags.
        let raw = unsafe {
            CreateFileW(
                extended.as_ptr(),
                FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: CreateFileW success transfers one owned handle.
        let root = unsafe { File::from_raw_handle(raw) };
        inspect(&root, true)?;
        let mut fs_name = [0_u16; 32];
        // SAFETY: live root handle and bounded filesystem-name output buffer.
        win(unsafe {
            GetVolumeInformationByHandleW(
                handle(&root),
                null_mut(),
                0,
                null_mut(),
                null_mut(),
                null_mut(),
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            )
        })?;
        if fs_name[..5] != [b'N' as u16, b'T' as u16, b'F' as u16, b'S' as u16, 0] {
            return Err(io::Error::other("only NTFS is supported"));
        }
        let mut anchor = Self {
            root,
            chain: Vec::new(),
        };
        for part in parts {
            let Component::Normal(name) = part else {
                return Err(io::Error::other("unsupported path component"));
            };
            let child = relative(anchor.directory(), name, true, false, None)?;
            let id = inspect(&child, true)?;
            anchor
                .chain
                .push((name.to_str().unwrap().into(), child, id));
        }
        anchor.verify()?;
        Ok(anchor)
    }
    fn directory(&self) -> &File {
        self.chain.last().map_or(&self.root, |(_, file, _)| file)
    }
    fn security_directory(&self) -> io::Result<File> {
        let (name, _, id) = self
            .chain
            .last()
            .ok_or_else(|| io::Error::other("volume root is not an app cwd"))?;
        let parent = if self.chain.len() == 1 {
            &self.root
        } else {
            &self.chain[self.chain.len() - 2].1
        };
        let file = relative_access(parent, name.as_ref(), true, false, None, READ_CONTROL)?;
        if inspect(&file, true)? != *id {
            return Err(io::Error::other("directory identity changed"));
        }
        Ok(file)
    }
    fn verify(&self) -> io::Result<()> {
        let mut parent = &self.root;
        inspect(parent, true)?;
        for (name, expected, id) in &self.chain {
            let actual = relative(parent, name.as_ref(), true, false, None)?;
            if inspect(&actual, true)? != *id || inspect(expected, true)? != *id {
                return Err(io::Error::other("parent identity changed"));
            }
            parent = expected;
        }
        Ok(())
    }
}

pub(crate) fn read_framework(path: &Path) -> io::Result<Vec<u8>> {
    let parent = Anchor::open(
        path.parent()
            .ok_or_else(|| io::Error::other("missing framework parent"))?,
    )?;
    let mut file = relative(
        parent.directory(),
        path.file_name()
            .ok_or_else(|| io::Error::other("missing framework filename"))?,
        false,
        false,
        None,
    )?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1024 * 1024 {
        return Err(io::Error::other("framework file too large"));
    }
    parent.verify()?;
    Ok(bytes)
}

fn remove(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: deletes only the exact owned handle, never a path or directory tree.
    win(unsafe {
        SetFileInformationByHandle(
            handle(file),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    })
}

struct Entry {
    name: String,
    id: Identity,
    file: Option<File>,
    dir: bool,
}
impl Entry {
    fn create(parent: &File, name: &str, dir: bool, sd: &Local) -> io::Result<Self> {
        let file = relative(parent, name.as_ref(), dir, true, Some(sd))?;
        let id = inspect(&file, dir)?;
        Ok(Self {
            name: name.into(),
            id,
            file: Some(file),
            dir,
        })
    }
    fn delete(mut self, parent: &File) -> io::Result<()> {
        let file = match self.file.take() {
            Some(file) => file,
            None => {
                // Reopen for DELETE, not FILE_CREATE. No path-based fallback.
                open_delete(parent, &self.name, self.dir)?
            }
        };
        if inspect(&file, self.dir)? != self.id {
            return Err(io::Error::other("owned staging identity changed; retained"));
        }
        remove(&file)
    }
}
fn open_delete(parent: &File, name: &str, dir: bool) -> io::Result<File> {
    // The reopen must request DELETE. Reuse a separate native open policy by
    // opening through the original handle's relative component, never a DOS path.
    let mut wide = component(name.as_ref())?;
    let name = UNICODE_STRING {
        Length: (wide.len() * 2) as u16,
        MaximumLength: (wide.len() * 2) as u16,
        Buffer: wide.as_mut_ptr(),
    };
    let attrs = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: handle(parent),
        ObjectName: &name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        ..Default::default()
    };
    let mut raw = null_mut();
    let mut status = IO_STATUS_BLOCK::default();
    // SAFETY: live pinned parent, bounded single component and synchronous output.
    nt(unsafe {
        NtCreateFile(
            &mut raw,
            DELETE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            &attrs,
            &mut status,
            null(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN,
            FILE_SYNCHRONOUS_IO_NONALERT
                | FILE_OPEN_REPARSE_POINT
                | if dir {
                    FILE_DIRECTORY_FILE
                } else {
                    FILE_NON_DIRECTORY_FILE
                },
            null(),
            0,
        )
    })?;
    // SAFETY: successful NtCreateFile transfers one owned handle.
    Ok(unsafe { File::from_raw_handle(raw) })
}

struct Stage {
    root: Option<Entry>,
    children: Vec<Entry>,
    files: Vec<(Option<usize>, Entry)>,
}
impl Stage {
    fn make(parent: &Anchor, sd: &Local) -> io::Result<Self> {
        let mut random = [0_u8; 16];
        // SAFETY: output slice is writable; null algorithm selects system RNG.
        nt(unsafe {
            BCryptGenRandom(
                null_mut(),
                random.as_mut_ptr(),
                random.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        })?;
        let name = format!(".yetlet-{:032x}", u128::from_le_bytes(random));
        let root = Entry::create(parent.directory(), &name, true, sd)?;
        Ok(Self {
            root: Some(root),
            children: Vec::new(),
            files: Vec::new(),
        })
    }
    fn root(&self) -> &File {
        self.root.as_ref().unwrap().file.as_ref().unwrap()
    }
    fn rollback(&mut self) -> io::Result<()> {
        let mut error = None;
        for child in &mut self.children {
            if child.file.is_none() {
                let file = open_delete(
                    self.root.as_ref().unwrap().file.as_ref().unwrap(),
                    &child.name,
                    true,
                )?;
                if inspect(&file, true)? != child.id {
                    return Err(io::Error::other("owned child identity changed; retained"));
                }
                child.file = Some(file);
            }
        }
        while let Some((child, entry)) = self.files.pop() {
            let parent = child.map_or_else(
                || self.root(),
                |index| self.children[index].file.as_ref().unwrap(),
            );
            if let Err(e) = entry.delete(parent) {
                error = Some(e);
            }
        }
        while let Some(child) = self.children.pop() {
            if let Err(e) = child.delete(self.root()) {
                error = Some(e);
            }
        }
        if let Some(root) = self.root.take() {
            let file = root.file.unwrap();
            if inspect(&file, true)? != root.id {
                return Err(io::Error::other("staging root identity changed; retained"));
            }
            if let Err(e) = remove(&file) {
                error = Some(e);
            }
        }
        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    fn populate(&mut self, entries: &[(&str, String)], sd: &Local) -> io::Result<()> {
        for (path, bytes) in entries {
            let (child, name) = if let Some((child, name)) = path.split_once('/') {
                let index = if let Some(index) =
                    self.children.iter().position(|entry| entry.name == child)
                {
                    index
                } else {
                    self.children
                        .push(Entry::create(self.root(), child, true, sd)?);
                    self.children.len() - 1
                };
                (Some(index), name)
            } else {
                (None, *path)
            };
            let parent = child.map_or_else(
                || self.root(),
                |index| self.children[index].file.as_ref().unwrap(),
            );
            let entry = Entry::create(parent, name, false, sd)?;
            self.files.push((child, entry));
            let file = self.files.last_mut().unwrap().1.file.as_mut().unwrap();
            file.write_all(bytes.as_bytes())?;
            file.sync_all()?;
        }
        Ok(())
    }
    fn publish(&mut self, parent: &Anchor, name: &str) -> io::Result<()> {
        parent.verify()?;
        if inspect(self.root(), true)? != self.root.as_ref().unwrap().id {
            return Err(io::Error::other("stage identity changed"));
        }
        // NTFS directory rename requires its descendant handles to be closed.
        // The private protected DACL protects these names after close. On failure
        // rollback reopens each exact name and compares the original volume/ID.
        for (_, entry) in &mut self.files {
            entry.file.take();
        }
        for child in &mut self.children {
            child.file.take();
        }
        rename(self.root(), parent.directory(), name)?;
        // Successful native rename is the sole ownership-transfer point. Never
        // delete the published destination, even if a later close fails.
        self.root.take();
        self.files.clear();
        self.children.clear();
        Ok(())
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        if self.root.is_some() {
            let _ = self.rollback();
        }
    }
}

fn rename(source: &File, parent: &File, name: &str) -> io::Result<()> {
    let name = component(name.as_ref())?;
    let length = (offset_of!(FILE_RENAME_INFORMATION, FileName) + name.len() * 2)
        .max(size_of::<FILE_RENAME_INFORMATION>());
    // usize allocation is aligned for the SDK struct and includes its trailing
    // variable-length UTF-16 array; zeroed ReplaceIfExists means NO replacement.
    let mut storage = vec![
        0_usize;
        length
            .max(size_of::<FILE_RENAME_INFORMATION>())
            .div_ceil(size_of::<usize>())
    ];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    // SAFETY: aligned zeroed buffer, correct SDK offsets and allocated tail size.
    unsafe {
        (*info).RootDirectory = handle(parent);
        (*info).FileNameLength = (name.len() * 2) as u32;
        std::ptr::copy_nonoverlapping(name.as_ptr(), (*info).FileName.as_mut_ptr(), name.len());
    }
    let mut status = IO_STATUS_BLOCK::default();
    // SAFETY: synchronous source handle has DELETE; relative single-component
    // name and live parent refer to the same NTFS volume. No replacing flags.
    nt(unsafe {
        NtSetInformationFile(
            handle(source),
            &mut status,
            info.cast(),
            length as u32,
            FileRenameInformation,
        )
    })
}

pub(crate) fn create_project(
    parent: &Path,
    name: &str,
    entries: &[(&str, String)],
) -> Result<(), String> {
    create(parent, name, entries, || Ok(()))
        .map_err(|error| format!("cannot safely create project; destination preserved: {error}"))
}
fn create(
    parent: &Path,
    name: &str,
    entries: &[(&str, String)],
    before_publish: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    component(name.as_ref())?;
    let children = super::generated_subdirectories(entries).map_err(io::Error::other)?;
    let parent = Anchor::open(parent)?;
    // Deployment configuration additionally requires the explicitly selected
    // output parent to be owner-protected. Retain the historical new policy.
    let parent_security = if children.contains(&"cargo-overlay")
        || entries
            .iter()
            .any(|(path, _)| matches!(*path, "compose.yaml" | "deployment.yaml" | "service.yaml"))
    {
        let security = parent.security_directory()?;
        protected(&security, true)?;
        Some(security)
    } else {
        None
    };
    let sd = private_security()?;
    let mut stage = Stage::make(&parent, &sd)?;
    let result = stage
        .populate(entries, &sd)
        .and_then(|()| before_publish())
        .and_then(|()| stage.publish(&parent, name));
    if let Err(error) = result {
        return match stage.rollback() {
            Ok(()) => Err(error),
            Err(cleanup) => Err(io::Error::other(format!(
                "{error}; exact-owned staging cleanup failed (retained): {cleanup}"
            ))),
        };
    }
    drop(parent_security);
    Ok(())
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
