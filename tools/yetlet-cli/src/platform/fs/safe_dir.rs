//! Linux-only, descriptor-relative creation and atomic non-replacing publication.
//! No path-based recursive deletion, replacing rename, or symlink resolution.
use std::{
    ffi::{CStr, CString, OsStr},
    fs::{File, Metadata},
    io::{self, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Component, Path, PathBuf},
};

fn cstr(value: &OsStr) -> io::Result<CString> {
    CString::new(value.as_bytes()).map_err(|_| io::Error::other("NUL is forbidden in paths"))
}

fn same(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev() && left.ino() == right.ino()
}

fn open_at(parent: &File, name: &CStr, flags: i32, mode: libc::mode_t) -> io::Result<File> {
    // SAFETY: live directory FD, terminated name; result transfers exactly once.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            mode,
        )
    };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

fn directory(parent: &File, name: &CStr) -> io::Result<File> {
    open_at(parent, name, libc::O_RDONLY | libc::O_DIRECTORY, 0)
}

fn mkdir(parent: &File, name: &CStr) -> io::Result<()> {
    // SAFETY: live directory FD and terminated single-component name.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn unlink(parent: &File, name: &CStr, directory: bool) -> io::Result<()> {
    // SAFETY: relative to a live owned directory FD; never follows a symlink.
    if unsafe {
        libc::unlinkat(
            parent.as_raw_fd(),
            name.as_ptr(),
            if directory { libc::AT_REMOVEDIR } else { 0 },
        )
    } == 0
    {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(crate) struct Anchor {
    path: PathBuf,
    root: File,
    chain: Vec<(CString, File)>,
}

impl Anchor {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        let path = crate::framework::absolute(path)?;
        let root = File::open("/")?;
        let mut anchor = Self {
            path: path.clone(),
            root,
            chain: Vec::new(),
        };
        for part in path.components() {
            match part {
                Component::RootDir => (),
                Component::Normal(name) => {
                    let name = cstr(name)?;
                    let child = directory(anchor.fd(), &name)?;
                    anchor.chain.push((name, child));
                }
                _ => return Err(io::Error::other("unsupported path component")),
            }
        }
        anchor.verify()?;
        Ok(anchor)
    }

    fn fd(&self) -> &File {
        self.chain.last().map_or(&self.root, |(_, fd)| fd)
    }

    /// Pin the checked app cwd for fchdir before Cargo exec; never resolve it again.
    pub(crate) fn working_directory(&self) -> io::Result<File> {
        self.verify()?;
        self.fd().try_clone()
    }

    /// The entry created by mkdirat cannot be trusted if another principal can
    /// rename it before openat. Private owner-writable parents and trusted sticky
    /// directories protect newly created entries from other unprivileged users.
    fn require_protected_create_parent(&self) -> io::Result<()> {
        let metadata = self.fd().metadata()?;
        let mode = metadata.mode();
        let owner = metadata.uid();
        let me = unsafe { libc::geteuid() };
        let private_owner = owner == me && mode & 0o022 == 0;
        let protected_sticky = mode & 0o1000 != 0 && (owner == me || owner == 0);
        if !private_owner && !protected_sticky {
            return Err(io::Error::other(
                "destination parent permits untrusted directory-entry replacement",
            ));
        }
        self.verify()
    }

    /// Verify every original parent/name link, not just the final directory.
    pub(crate) fn verify(&self) -> io::Result<()> {
        let mut parent = &self.root;
        for (name, expected) in &self.chain {
            let actual = directory(parent, name)?;
            if !same(&actual.metadata()?, &expected.metadata()?) {
                return Err(io::Error::other("selected parent identity changed"));
            }
            parent = expected;
        }
        Ok(())
    }

    pub(crate) fn read_file(&self, name: &OsStr) -> io::Result<File> {
        let file = open_at(
            self.fd(),
            &cstr(name)?,
            libc::O_RDONLY | libc::O_NONBLOCK,
            0,
        )?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::other("expected regular file"));
        }
        Ok(file)
    }
}

struct OwnedFile {
    name: CString,
    file: File,
    child: Option<usize>,
}
struct Temp {
    name: CString,
    dir: File,
    children: Vec<(&'static str, File)>,
    files: Vec<OwnedFile>,
}

impl Temp {
    fn make(parent: &Anchor) -> io::Result<Self> {
        for _ in 0..8 {
            let mut random = [0_u8; 16];
            // SAFETY: valid writable buffer; flags=0 obtains kernel randomness.
            let read = unsafe { libc::getrandom(random.as_mut_ptr().cast(), random.len(), 0) };
            if read != random.len() as isize {
                return Err(io::Error::other("temporary name entropy unavailable"));
            }
            let name =
                CString::new(format!(".yetlet-new-{:032x}", u128::from_ne_bytes(random))).unwrap();
            match mkdir(parent.fd(), &name) {
                Ok(()) => {
                    let dir = directory(parent.fd(), &name)?;
                    let metadata = dir.metadata()?;
                    // A substituted directory is never trusted for writes.
                    if metadata.uid() != unsafe { libc::geteuid() }
                        || metadata.mode() & 0o777 != 0o700
                    {
                        return Err(io::Error::other(
                            "temporary directory identity/permissions uncertain; inspect parent",
                        ));
                    }
                    return Ok(Self {
                        name,
                        dir,
                        children: Vec::new(),
                        files: Vec::new(),
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::other(
            "temporary name collisions; nothing published",
        ))
    }

    fn verify(&self, parent: &Anchor) -> io::Result<()> {
        let actual = directory(parent.fd(), &self.name)?;
        if !same(&actual.metadata()?, &self.dir.metadata()?) {
            return Err(io::Error::other("temporary directory identity changed"));
        }
        Ok(())
    }

    fn write(&mut self, path: &str, data: &str) -> io::Result<()> {
        let (child, name) = if let Some((child, name)) = path.split_once('/') {
            let index =
                if let Some(index) = self.children.iter().position(|(name, _)| *name == child) {
                    index
                } else {
                    let child = match child {
                        "src" => "src",
                        "frontend" => "frontend",
                        "cargo-overlay" => "cargo-overlay",
                        _ => return Err(io::Error::other("unsupported generated child")),
                    };
                    let name = cstr(child.as_ref())?;
                    mkdir(&self.dir, &name)?;
                    self.children.push((child, directory(&self.dir, &name)?));
                    self.children.len() - 1
                };
            (Some(index), name)
        } else {
            (None, path)
        };
        let parent = child.map_or(&self.dir, |index| &self.children[index].1);
        let name = CString::new(name).unwrap();
        let file = open_at(
            parent,
            &name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        // Retain the FD before write, so partial writes also have an owned identity.
        self.files.push(OwnedFile { name, file, child });
        let file = &mut self.files.last_mut().unwrap().file;
        file.write_all(data.as_bytes())?;
        file.sync_all()
    }

    fn verify_contents(&self) -> io::Result<()> {
        for (name, child) in &self.children {
            let actual = directory(&self.dir, &cstr(name.as_ref())?)?;
            if !same(&actual.metadata()?, &child.metadata()?) {
                return Err(io::Error::other("temporary child identity changed"));
            }
        }
        for entry in &self.files {
            let parent = entry
                .child
                .map_or(&self.dir, |index| &self.children[index].1);
            let actual = open_at(parent, &entry.name, libc::O_PATH, 0)?;
            if !same(&actual.metadata()?, &entry.file.metadata()?) {
                return Err(io::Error::other("temporary file identity changed"));
            }
        }
        Ok(())
    }

    /// Remove only known entries after verifying their held identities. Unknown
    /// entries, substitutions or uncertain publication leave evidence for inspection.
    fn cleanup(&self, parent: &Anchor) -> io::Result<()> {
        self.verify(parent)?;
        self.verify_contents()?;
        for entry in &self.files {
            let parent = entry
                .child
                .map_or(&self.dir, |index| &self.children[index].1);
            let actual = open_at(parent, &entry.name, libc::O_PATH, 0)?;
            if !same(&actual.metadata()?, &entry.file.metadata()?) {
                return Err(io::Error::other("cleanup identity uncertain"));
            }
            unlink(parent, &entry.name, false)?;
        }
        for (name, _) in &self.children {
            unlink(&self.dir, &cstr(name.as_ref())?, true)?;
        }
        self.verify(parent)?;
        unlink(parent.fd(), &self.name, true)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    TempReady,
    FileWritten(usize),
    BeforePublish,
    ParentChecked,
    Published,
}

pub(crate) fn create(parent: &Path, name: &str, files: &[(&str, String)]) -> Result<(), String> {
    create_inner(parent, name, files, |_, _| Ok(()))
}

fn create_inner(
    parent: &Path,
    name: &str,
    files: &[(&str, String)],
    mut hook: impl FnMut(Step, &Path) -> io::Result<()>,
) -> Result<(), String> {
    super::super::generated_subdirectories(files)?;
    if name.is_empty() || matches!(name, "." | "..") || name.contains(['/', '\\', '\0']) {
        return Err("invalid destination component".into());
    }
    let parent = Anchor::open(parent).map_err(|_| {
        "destination parent is unavailable, unsafe or contains a symlink".to_owned()
    })?;
    parent.require_protected_create_parent().map_err(|_| {
        "destination parent allows untrusted replacement before temporary ownership is established".to_owned()
    })?;
    let destination = CString::new(name).map_err(|_| "invalid destination name".to_owned())?;
    // Friendly early refusal; the atomic syscall below remains the authority.
    match open_at(parent.fd(), &destination, libc::O_PATH, 0) {
        Ok(_) => return Err("destination already exists; nothing overwritten".into()),
        Err(e) if e.raw_os_error() == Some(libc::ENOENT) => (),
        Err(_) => return Err("destination cannot be safely inspected".into()),
    }
    let mut temp = Temp::make(&parent).map_err(|e| {
        format!(
            "cannot establish owned temporary directory (OS code {:?}); inspect parent",
            e.raw_os_error()
        )
    })?;
    let temp_path = parent.path.join(OsStr::from_bytes(temp.name.as_bytes()));
    let prepare = (|| -> io::Result<()> {
        hook(Step::TempReady, &temp_path)?;
        for (index, (path, contents)) in files.iter().enumerate() {
            temp.write(path, contents)?;
            hook(Step::FileWritten(index), &temp_path)?;
        }
        temp.dir.sync_all()?;
        hook(Step::BeforePublish, &temp_path)?;
        temp.verify(&parent)?;
        temp.verify_contents()?;
        parent.verify()?;
        hook(Step::ParentChecked, &temp_path)?;
        Ok(())
    })();
    if prepare.is_err() {
        return Err(if temp.cleanup(&parent).is_ok() {
            "pre-publication failure; owned temporary files cleaned; destination untouched".into()
        } else {
            "pre-publication failure; cleanup identity uncertain or foreign entries present; inspect temporary sibling; destination untouched".into()
        });
    }
    // SAFETY: held parent FD anchors both names, flags prohibit replacement.
    let result = unsafe {
        libc::renameat2(
            parent.fd().as_raw_fd(),
            temp.name.as_ptr(),
            parent.fd().as_raw_fd(),
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result != 0 {
        let error = io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(libc::EEXIST | libc::ENOTEMPTY)) {
            let cleaned = temp.cleanup(&parent).is_ok();
            return Err(format!(
                "destination appeared before publication; existing data intact; temporary cleanup {}",
                if cleaned {
                    "complete"
                } else {
                    "uncertain: inspect parent"
                }
            ));
        }
        // Unsupported syscall/filesystem and all unexpected errors fail closed.
        // Never infer mutation state from an unfamiliar error and blindly delete.
        return Err(format!(
            "publication unsupported or outcome uncertain (OS code {:?}); inspect destination and temporary sibling; no rollback attempted",
            error.raw_os_error()
        ));
    }
    let confirmed = (|| -> io::Result<()> {
        hook(Step::Published, &temp_path)?;
        parent.verify()?;
        let published = directory(parent.fd(), &destination)?;
        if !same(&published.metadata()?, &temp.dir.metadata()?) {
            return Err(io::Error::other("published identity changed"));
        }
        parent.fd().sync_all()
    })();
    if confirmed.is_err() {
        return Err("publication outcome uncertain after move; inspect destination; no rollback or deletion attempted".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };

    fn files() -> Vec<(&'static str, String)> {
        vec![
            ("Cargo.toml", "owned".into()),
            ("src/main.rs", "owned source".into()),
        ]
    }

    // tempfile respects the host umask; these tests require a protected
    // destination parent, independently of whether the developer uses 0002.
    fn private_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("create temporary test directory");
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        root
    }

    fn frontend_files() -> Vec<(&'static str, String)> {
        vec![
            ("Cargo.toml", "owned manifest".into()),
            ("src/main.rs", "owned source".into()),
            ("frontend/package.json", "owned frontend".into()),
            ("frontend/.env.example", "VITE_PUBLIC_VALUE=example".into()),
        ]
    }

    #[test]
    fn frontend_and_src_publish_together_without_replacement_or_partial_trees() {
        let root = private_root();
        let entries = frontend_files();
        create(root.path(), "app", &entries).unwrap();
        for (path, bytes) in &entries {
            assert_eq!(
                fs::read(root.path().join("app").join(path)).unwrap(),
                bytes.as_bytes()
            );
        }
        assert!(create(root.path(), "app", &entries).is_err());
        for failure in [
            Step::FileWritten(1),
            Step::FileWritten(2),
            Step::FileWritten(3),
            Step::BeforePublish,
        ] {
            assert!(
                create_inner(root.path(), "rollback", &entries, |step, _| {
                    if step == failure {
                        Err(io::Error::other("injected failure"))
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err()
                .contains("cleaned")
            );
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        }
        assert!(
            create_inner(root.path(), "race", &entries, |step, _| {
                if step == Step::ParentChecked {
                    symlink("private-target", root.path().join("race"))?;
                }
                Ok(())
            })
            .unwrap_err()
            .contains("destination appeared")
        );
        assert_eq!(
            fs::read_link(root.path().join("race")).unwrap(),
            Path::new("private-target")
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        for path in [
            "frontend/../escape",
            "frontend/nested/app.ts",
            "frontend/unlisted",
            "other/package.json",
        ] {
            assert!(create(root.path(), "bad", &[(path, "bad".into())]).is_err());
            assert!(!root.path().join("bad").exists());
        }
        for files in [
            vec![
                ("src/main.rs", "".into()),
                ("cargo-overlay/Cargo.toml", "".into()),
            ],
            vec![
                ("frontend/package.json", "".into()),
                ("cargo-overlay/Cargo.toml", "".into()),
            ],
        ] {
            assert!(create(root.path(), "mixed", &files).is_err());
        }
    }

    #[test]
    fn substituted_frontend_child_is_retained_without_following_links() {
        let root = private_root();
        let target = root.path().join("foreign");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("package.json"), "private").unwrap();
        let error = create_inner(root.path(), "app", &frontend_files(), |step, temp| {
            if step == Step::BeforePublish {
                fs::rename(temp.join("frontend"), temp.join("saved-frontend"))?;
                symlink(&target, temp.join("frontend"))?;
            }
            Ok(())
        })
        .unwrap_err();
        assert!(error.contains("cleanup identity uncertain"));
        assert!(!root.path().join("app").exists());
        assert_eq!(fs::read(target.join("package.json")).unwrap(), b"private");
        let temp = fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".yetlet-new-")
            })
            .unwrap();
        assert!(
            fs::symlink_metadata(temp.join("frontend"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read(temp.join("saved-frontend/package.json")).unwrap(),
            b"owned frontend"
        );
        assert_eq!(fs::read(temp.join("src/main.rs")).unwrap(), b"owned source");
    }

    #[test]
    fn k8s_root_files_keep_no_replace_race_and_partial_cleanup() {
        let entries = vec![
            ("deployment.yaml", "{}\n".into()),
            ("service.yaml", "{}\n".into()),
            ("README.md", "owned instructions\n".into()),
        ];
        let root = private_root();
        create(root.path(), "k8s", &entries).unwrap();
        assert_eq!(fs::read_dir(root.path().join("k8s")).unwrap().count(), 3);
        assert!(create(root.path(), "k8s", &entries).is_err());
        for failure in [
            Step::FileWritten(0),
            Step::FileWritten(1),
            Step::BeforePublish,
        ] {
            assert!(
                create_inner(root.path(), "rollback", &entries, |step, _| {
                    if step == failure {
                        Err(io::Error::other("injected failure"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
            );
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        }
        assert!(
            create_inner(root.path(), "race", &entries, |step, _| {
                if step == Step::ParentChecked {
                    fs::write(root.path().join("race"), "foreign")?;
                }
                Ok(())
            })
            .is_err()
        );
        assert_eq!(fs::read(root.path().join("race")).unwrap(), b"foreign");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    }

    #[test]
    fn compose_root_files_retain_no_replace_partial_cleanup_and_path_guards() {
        let entries = vec![
            ("compose.yaml", "owned YAML\n".into()),
            ("README.md", "owned README\n".into()),
        ];
        let root = private_root();
        create(root.path(), "compose", &entries).unwrap();
        assert_eq!(
            fs::read_dir(root.path().join("compose")).unwrap().count(),
            2
        );
        assert!(create(root.path(), "compose", &entries).is_err());
        for failure in [Step::FileWritten(0), Step::BeforePublish] {
            assert!(
                create_inner(root.path(), "rollback", &entries, |step, _| {
                    if step == failure {
                        Err(io::Error::other("injected failure"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
            );
            assert!(!root.path().join("rollback").exists());
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        }
        assert!(
            create_inner(root.path(), "race", &entries, |step, _| {
                if step == Step::ParentChecked {
                    fs::write(root.path().join("race"), "foreign")?;
                }
                Ok(())
            })
            .is_err()
        );
        assert_eq!(fs::read(root.path().join("race")).unwrap(), b"foreign");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        for path in [
            "../escape",
            "tree/compose.yaml",
            "tree/../escape",
            "tree\\compose.yaml",
        ] {
            assert!(create(root.path(), "bad", &[(path, "bad".into())]).is_err());
            assert!(!root.path().join("bad").exists());
        }
    }

    #[test]
    fn path_components_reject_symlink_and_traversal() {
        let root = private_root();
        let real = root.path().join("real");
        fs::create_dir(&real).unwrap();
        symlink(&real, root.path().join("link")).unwrap();
        for path in [
            root.path().join("link"),
            real.join(".."),
            root.path().join("missing"),
        ] {
            assert!(create(&path, "app", &files()).is_err());
        }
        assert!(fs::read_dir(&real).unwrap().next().is_none());
    }

    #[test]
    fn shared_nonsticky_parent_is_rejected_before_temporary_entry_can_be_substituted() {
        use std::os::unix::fs::PermissionsExt;
        let root = private_root();
        let shared = root.path().join("shared");
        fs::create_dir(&shared).unwrap();
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o777)).unwrap();
        let mut reached_temporary_stage = false;
        let result = create_inner(&shared, "app", &files(), |_, _| {
            reached_temporary_stage = true;
            Ok(())
        });
        assert!(result.unwrap_err().contains("untrusted replacement"));
        assert!(
            !reached_temporary_stage,
            "the create/open gap must never be entered in an unprotected parent"
        );
        assert!(fs::read_dir(&shared).unwrap().next().is_none());
        assert!(!shared.join("app").exists());
    }

    #[test]
    fn existing_file_directory_and_symlink_are_never_replaced() {
        let root = private_root();
        fs::write(root.path().join("file"), "private").unwrap();
        fs::create_dir(root.path().join("dir")).unwrap();
        symlink("file", root.path().join("link")).unwrap();
        for name in ["file", "dir", "link"] {
            assert!(create(root.path(), name, &files()).is_err());
        }
        assert_eq!(
            fs::read_to_string(root.path().join("file")).unwrap(),
            "private"
        );
        assert!(
            fs::symlink_metadata(root.path().join("link"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn late_destination_file_directory_and_symlink_win_publication_race() {
        for kind in 0..3 {
            let root = private_root();
            let dest = root.path().join("app");

            let result = create_inner(root.path(), "app", &files(), |step, _| {
                if step == Step::ParentChecked {
                    match kind {
                        0 => fs::write(&dest, "private")?,
                        1 => fs::create_dir(&dest)?,
                        _ => symlink("private-target", &dest)?,
                    }
                }
                Ok(())
            });
            let error = result.unwrap_err();
            assert!(
                error.contains("destination appeared"),
                "unexpected error: {error}"
            );
            match kind {
                0 => assert_eq!(fs::read_to_string(&dest).unwrap(), "private"),
                1 => assert!(dest.is_dir()),
                _ => assert_eq!(fs::read_link(&dest).unwrap(), Path::new("private-target")),
            }
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        }
    }

    #[test]
    fn parent_directory_or_symlink_substitution_fails_closed() {
        for symlink_swap in [false, true] {
            let root = private_root();
            let parent = root.path().join("parent");
            fs::create_dir(&parent).unwrap();
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
            let moved = root.path().join("moved");
            let replacement = root.path().join("replacement");
            fs::create_dir(&replacement).unwrap();
            let result = create_inner(&parent, "app", &files(), |step, _| {
                if step == Step::BeforePublish {
                    fs::rename(&parent, &moved)?;
                    if symlink_swap {
                        symlink(&replacement, &parent)?;
                    } else {
                        fs::create_dir(&parent)?;
                    }
                }
                Ok(())
            });
            assert!(result.is_err());
            assert!(!parent.join("app").exists());
            assert!(fs::read_dir(&replacement).unwrap().next().is_none());
            assert!(fs::read_dir(&moved).unwrap().next().is_none());
        }
    }

    #[test]
    fn ancestor_swap_is_detected_even_when_final_parent_is_held() {
        let root = private_root();
        let ancestor = root.path().join("ancestor");
        let parent = ancestor.join("parent");
        fs::create_dir_all(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let result = create_inner(&parent, "app", &files(), |step, _| {
            if step == Step::BeforePublish {
                fs::rename(&ancestor, root.path().join("old"))?;
                fs::create_dir_all(&parent)?;
            }
            Ok(())
        });
        assert!(result.is_err());
        assert!(!parent.join("app").exists());
        assert!(
            fs::read_dir(root.path().join("old/parent"))
                .unwrap()
                .next()
                .is_none()
        );
    }

    #[test]
    fn swap_after_parent_check_never_publishes_into_substituted_parent() {
        let root = private_root();
        let parent = root.path().join("parent");
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let moved = root.path().join("old");
        let result = create_inner(&parent, "app", &files(), |step, _| {
            if step == Step::ParentChecked {
                fs::rename(&parent, &moved)?;
                fs::create_dir(&parent)?;
            }
            Ok(())
        });
        assert!(result.unwrap_err().contains("outcome uncertain"));
        assert!(!parent.join("app").exists());
        assert_eq!(
            fs::read_to_string(moved.join("app/Cargo.toml")).unwrap(),
            "owned"
        );
    }

    #[test]
    fn partial_errors_clean_only_owned_files() {
        for fail_at in [Step::TempReady, Step::FileWritten(0), Step::FileWritten(1)] {
            let root = private_root();
            let result = create_inner(root.path(), "app", &files(), |step, _| {
                if step == fail_at {
                    return Err(io::Error::other("injected partial write failure"));
                }
                Ok(())
            });
            assert!(result.unwrap_err().contains("cleaned"));
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn substituted_temporary_directory_is_preserved_on_failure() {
        let root = private_root();
        let result = create_inner(root.path(), "app", &files(), |step, temp| {
            if step == Step::BeforePublish {
                fs::rename(temp, root.path().join("owned-old"))?;
                fs::create_dir(temp)?;
                fs::write(temp.join("private"), "keep")?;
            }
            Ok(())
        });
        assert!(result.unwrap_err().contains("cleanup identity uncertain"));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        for entry in fs::read_dir(root.path()).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() != "owned-old" {
                assert_eq!(fs::read_to_string(path.join("private")).unwrap(), "keep");
            }
        }
    }

    #[test]
    fn substituted_file_and_unknown_entries_are_not_blindly_deleted() {
        for replace in [false, true] {
            let root = private_root();
            let result = create_inner(root.path(), "app", &files(), |step, temp| {
                if step == Step::BeforePublish {
                    if replace {
                        fs::rename(temp.join("Cargo.toml"), temp.join("old"))?;
                        fs::write(temp.join("Cargo.toml"), "private")?;
                    } else {
                        fs::write(temp.join("foreign"), "private")?;
                    }
                    return Err(io::Error::other("injected"));
                }
                Ok(())
            });
            assert!(result.unwrap_err().contains("cleanup identity uncertain"));
            let temp = fs::read_dir(root.path())
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert_eq!(
                fs::read_to_string(temp.join(if replace { "Cargo.toml" } else { "foreign" }))
                    .unwrap(),
                "private"
            );
        }
    }

    #[test]
    fn uncertain_success_is_nonzero_without_rollback() {
        let root = private_root();
        let result = create_inner(root.path(), "app", &files(), |step, _| {
            if step == Step::Published {
                return Err(io::Error::other("injected post-move uncertainty"));
            }
            Ok(())
        });
        assert!(result.unwrap_err().contains("no rollback"));
        assert_eq!(
            fs::read_to_string(root.path().join("app/Cargo.toml")).unwrap(),
            "owned"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn docker_overlay_uses_the_same_no_replace_partial_cleanup_and_identity_checks() {
        let entries = vec![
            ("Dockerfile", "owned config".into()),
            ("cargo-overlay/Cargo.toml", "owned overlay".into()),
        ];
        for failure in [Step::FileWritten(1), Step::ParentChecked] {
            let root = private_root();
            let result = create_inner(
                root.path(),
                "container space 設定",
                &entries,
                |step, _| {
                    if step == failure {
                        if step == Step::ParentChecked {
                            symlink("private-target", root.path().join("container space 設定"))?;
                        } else {
                            return Err(io::Error::other("injected overlay write failure"));
                        }
                    }
                    Ok(())
                },
            );
            assert!(result.is_err());
            if failure == Step::ParentChecked {
                assert_eq!(
                    fs::read_link(root.path().join("container space 設定")).unwrap(),
                    Path::new("private-target")
                );
                assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
            } else {
                assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
            }
        }
        let root = private_root();
        assert!(
            create(
                root.path(),
                "out",
                &[("cargo-overlay/../escape", "bad".into())]
            )
            .is_err()
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
