//! Existing Linux filesystem guarantees, selected only at compile time.
#[path = "safe_dir.rs"]
mod safe_dir;
pub(crate) use safe_dir::create as create_project;

pub(crate) const NEW_SUPPORTED: bool = true;
pub(crate) const NEW_DIAGNOSTIC: &str =
    "Linux directory-handle/no-replace implementation available; filesystem support checked by new";

use crate::framework;
use safe_dir::Anchor;
use std::{
    fs::{File, Metadata},
    io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

pub(crate) struct App {
    pub(in crate::platform) directory: File,
    pub(in crate::platform) cwd: PathBuf,
    parent: Anchor,
    private_cwd: bool,
}

pub(super) struct AppFiles {
    app: App,
}

fn trusted(metadata: &Metadata) -> bool {
    metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0
}

fn read(parent: &Anchor, name: &str, private_cwd: bool) -> Result<File, String> {
    let file = parent.read_file(name.as_ref())
        .map_err(|_| format!("{name} must already exist as a regular, non-symlink app file; create the app lock separately with cargo generate-lockfile --offline"))?;
    let metadata = file.metadata().map_err(|_| "cannot inspect app file")?;
    if metadata.uid() != unsafe { libc::geteuid() }
        || (!private_cwd && metadata.mode() & 0o022 != 0)
    {
        return Err(format!(
            "{name} must be owned and protected by a private app cwd or non-group/world-writable permissions"
        ));
    }
    Ok(file)
}

impl AppFiles {
    pub(super) fn open(manifest: &Path) -> Result<Self, String> {
        let manifest = framework::absolute(manifest)
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
        })
    }

    pub(super) fn read(&self, name: &str) -> Result<File, String> {
        read(&self.app.parent, name, self.app.private_cwd)
    }

    pub(super) fn finish(self) -> Result<App, String> {
        self.app
            .parent
            .verify()
            .map_err(|_| "app cwd identity changed")?;
        Ok(self.app)
    }
}

pub(crate) fn app_root(app: &App) -> &Path {
    &app.cwd
}

pub(super) fn verify_app(app: &App) -> Result<(), String> {
    app.parent
        .verify()
        .map_err(|_| "app cwd identity changed".into())
}

pub(crate) fn open_directory(path: &Path) -> Result<App, String> {
    if !path.is_absolute() {
        return Err("protected directory path must be absolute".into());
    }
    let cwd = framework::absolute(path)
        .map_err(|_| "directory path must be nonempty and contain no '..'")?;
    let parent =
        Anchor::open(&cwd).map_err(|_| "app cwd is missing or unsafe (symlinks are forbidden)")?;
    let directory = parent
        .working_directory()
        .map_err(|_| "cannot pin app cwd")?;
    if !trusted(&directory.metadata().map_err(|_| "cannot inspect app cwd")?) {
        return Err(
            "app cwd must be owned by the current user and not group/world-writable".into(),
        );
    }
    // Cargo's explicit offline lock step honors the user's umask. A 0664
    // lock inside an owned 0700 app is inaccessible to other principals.
    let private_cwd = directory
        .metadata()
        .map_err(|_| "cannot inspect app cwd")?
        .mode()
        & 0o077
        == 0;
    Ok(App {
        directory,
        cwd,
        parent,
        private_cwd,
    })
}

fn selected_directory(app: &App, relative: &str) -> Result<Anchor, String> {
    app.parent
        .verify()
        .map_err(|_| "app cwd identity changed")?;
    super::app_directory(relative)?;
    let anchor = Anchor::open(&app.cwd.join(relative))
        .map_err(|_| "app directory is unavailable or unsafe")?;
    let dir = anchor
        .working_directory()
        .map_err(|_| "cannot inspect app directory")?;
    if !trusted(&dir.metadata().map_err(|_| "cannot inspect app directory")?) {
        return Err("app directory must be owned and protected".into());
    }
    Ok(anchor)
}

pub(crate) fn app_entries(app: &App, child: bool) -> Result<Vec<String>, String> {
    app_directory_entries(app, if child { "src" } else { "" })
}

pub(crate) fn app_directory_entries(app: &App, relative: &str) -> Result<Vec<String>, String> {
    use std::os::fd::AsRawFd;
    let anchor = selected_directory(app, relative)?;
    let dir = anchor
        .working_directory()
        .map_err(|_| "cannot pin app directory")?;
    let entries = std::fs::read_dir(format!("/proc/self/fd/{}", dir.as_raw_fd()))
        .map_err(|_| "cannot enumerate pinned app directory")?;
    let mut names = Vec::new();
    for entry in entries {
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
    app.parent
        .verify()
        .map_err(|_| "app cwd identity changed")?;
    names.sort();
    Ok(names)
}

pub(crate) fn read_app_file(app: &App, name: &str) -> Result<Vec<u8>, String> {
    read_app_input(app, name).map(|(bytes, _)| bytes)
}

pub(super) fn read_frontend_manifest(app: &App) -> Result<Vec<u8>, String> {
    use std::io::Read;
    verify_app(app)?;
    let mut selected = None;
    for relative in ["dist", "dist/.vite"] {
        let anchor =
            Anchor::open(&app.cwd.join(relative)).map_err(|_| "unsafe frontend dist directory")?;
        let metadata = anchor
            .working_directory()
            .and_then(|file| file.metadata())
            .map_err(|_| "cannot inspect frontend dist directory")?;
        if metadata.uid() != unsafe { libc::geteuid() }
            || !app.private_cwd && metadata.mode() & 0o022 != 0
        {
            return Err("frontend dist directory must be owned and protected".into());
        }
        selected = Some(anchor);
    }
    let anchor = selected.expect("fixed frontend directory selection");
    let mut bytes = Vec::new();
    read(&anchor, "manifest.json", app.private_cwd)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read frontend manifest")?;
    if bytes.len() > 1024 * 1024 {
        return Err("frontend manifest exceeds size limit".into());
    }
    anchor
        .verify()
        .map_err(|_| "frontend dist identity changed")?;
    verify_app(app)?;
    Ok(bytes)
}

pub(crate) fn read_app_input(
    app: &App,
    name: &str,
) -> Result<(Vec<u8>, std::time::SystemTime), String> {
    use std::io::Read;
    let (directory, name) = super::app_input(name)?;
    let anchor = selected_directory(app, directory)?;
    let mut file = read(&anchor, name, app.private_cwd)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read app input")?;
    if bytes.len() > 1024 * 1024 {
        return Err("app input exceeds size limit".into());
    }
    anchor
        .verify()
        .map_err(|_| "app directory identity changed")?;
    app.parent
        .verify()
        .map_err(|_| "app cwd identity changed")?;
    let modified = file
        .metadata()
        .and_then(|m| m.modified())
        .map_err(|_| "cannot inspect app input time")?;
    Ok((bytes, modified))
}

pub(crate) fn read_framework(path: &Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let parent = safe_dir::Anchor::open(path.parent().unwrap())?;
    let mut file = parent.read_file(path.file_name().unwrap())?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 1024 * 1024 {
        return Err(io::Error::other("framework file too large"));
    }
    parent.verify()?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::platform::fs::open_app;
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            root.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        for name in ["Cargo.toml", "Cargo.lock"] {
            fs::set_permissions(root.path().join(name), fs::Permissions::from_mode(0o600)).unwrap();
        }
        root
    }

    #[test]
    fn frontend_root_selection_is_app_relative_protected_and_separate_from_generator_inputs() {
        use crate::platform::fs::{app_root, frontend_directory, read_app_file};
        use std::path::Path;
        let root = fixture();
        for directory in ["frontend", "ui", "ui/client"] {
            fs::create_dir(root.path().join(directory)).unwrap();
            fs::set_permissions(
                root.path().join(directory),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        fs::write(
            root.path().join("frontend/package.json"),
            "frontend package",
        )
        .unwrap();
        let app = open_app(&root.path().join("Cargo.toml")).unwrap();
        let frontend = frontend_directory(&app, Path::new("frontend/")).unwrap();
        assert_eq!(app_root(&frontend), root.path().join("frontend"));
        assert_eq!(
            read_app_file(&frontend, "package.json").unwrap(),
            b"frontend package"
        );
        assert_eq!(
            app_root(&frontend_directory(&app, Path::new("ui/client")).unwrap()),
            root.path().join("ui/client")
        );
        assert!(read_app_file(&app, "frontend/package.json").is_err());
        assert!(crate::platform::fs::app_directory_entries(&app, "frontend").is_err());
        for directory in [
            "",
            ".",
            "..",
            "../frontend",
            "ui/../frontend",
            "/tmp",
            "C:/frontend",
            "ui\\client",
            "missing",
        ] {
            assert!(
                frontend_directory(&app, Path::new(directory)).is_err(),
                "{directory}"
            );
        }
        symlink("frontend", root.path().join("linked")).unwrap();
        assert!(frontend_directory(&app, Path::new("linked")).is_err());
        symlink("../ui", root.path().join("frontend/linked")).unwrap();
        assert!(frontend_directory(&app, Path::new("frontend/linked/client")).is_err());
        fs::set_permissions(root.path().join("ui"), fs::Permissions::from_mode(0o777)).unwrap();
        assert!(frontend_directory(&app, Path::new("ui/client")).is_err());
        fs::rename(
            root.path().join("frontend"),
            root.path().join("old-frontend"),
        )
        .unwrap();
        fs::create_dir(root.path().join("frontend")).unwrap();
        assert!(read_app_file(&frontend, "package.json").is_err());
        use std::os::unix::fs::MetadataExt;
        assert_eq!(
            frontend.directory.metadata().unwrap().ino(),
            fs::metadata(root.path().join("old-frontend"))
                .unwrap()
                .ino()
        );
    }

    #[test]
    fn explicit_public_settings_directory_needs_no_manifest_or_lock_and_retains_safe_reads() {
        use crate::platform::fs::{open_directory, read_app_file};
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(root.path().join("base.toml"), "[cli]\n").unwrap();
        let settings = open_directory(root.path()).unwrap();
        assert_eq!(read_app_file(&settings, "base.toml").unwrap(), b"[cli]\n");
        assert!(open_directory(std::path::Path::new("relative")).is_err());
        assert!(open_directory(&root.path().join("..")).is_err());
        fs::rename(
            root.path().join("base.toml"),
            root.path().join("private.toml"),
        )
        .unwrap();
        symlink("private.toml", root.path().join("base.toml")).unwrap();
        assert!(read_app_file(&settings, "base.toml").is_err());
        assert!(read_app_file(&settings, "private.toml").is_err());
        fs::remove_file(root.path().join("base.toml")).unwrap();
        fs::write(root.path().join("base.toml"), vec![0; 1024 * 1024 + 1]).unwrap();
        assert!(
            read_app_file(&settings, "base.toml")
                .unwrap_err()
                .contains("size limit")
        );
    }

    #[test]
    fn lock_and_manifest_preflight_reject_unsafe_or_implicit_selection() {
        let root = fixture();
        let path = root.path().join("Cargo.toml");
        assert!(open_app(&path).is_ok());
        assert!(open_app(&root.path().join("../Cargo.toml")).is_err());
        assert!(open_app(root.path()).is_err());
        fs::rename(
            root.path().join("Cargo.lock"),
            root.path().join("saved-lock"),
        )
        .unwrap();
        assert!(open_app(&path).is_err());
        assert!(!root.path().join("Cargo.lock").exists());
        symlink("saved-lock", root.path().join("Cargo.lock")).unwrap();
        assert!(open_app(&path).is_err());
        fs::remove_file(root.path().join("Cargo.lock")).unwrap();
        fs::rename(
            root.path().join("saved-lock"),
            root.path().join("Cargo.lock"),
        )
        .unwrap();
        fs::rename(&path, root.path().join("saved-manifest")).unwrap();
        symlink("saved-manifest", &path).unwrap();
        assert!(open_app(&path).is_err());
        fs::remove_file(&path).unwrap();
        fs::rename(root.path().join("saved-manifest"), &path).unwrap();
        symlink(root.path(), root.path().join("alias")).unwrap();
        assert!(open_app(&root.path().join("alias/Cargo.toml")).is_err());
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o777)).unwrap();
        assert!(open_app(&path).is_err());
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(
            root.path().join("Cargo.lock"),
            fs::Permissions::from_mode(0o664),
        )
        .unwrap();
        assert!(
            open_app(&path).is_ok(),
            "offline Cargo lock is protected by private cwd"
        );
        fs::set_permissions(
            root.path().join("Cargo.lock"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(open_app(&path).is_err());
    }

    #[test]
    fn preflight_rejects_workspace_roots_inheritance_and_invalid_locks() {
        let root = fixture();
        let path = root.path().join("Cargo.toml");
        for manifest in [
            "[workspace]\n",
            "[package]\nname='fixture'\nversion='0.1.0'\n",
            "[package]\nworkspace='..'\n[workspace]\n",
            "[package]\n[workspace]\nmembers=['other']\n",
        ] {
            fs::write(&path, manifest).unwrap();
            assert!(open_app(&path).is_err());
        }
        fs::write(
            &path,
            "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n",
        )
        .unwrap();
        for lock in ["", "broken TOML", "version=4\n", "version=4\npackage=[]\n"] {
            fs::write(root.path().join("Cargo.lock"), lock).unwrap();
            assert!(open_app(&path).is_err());
        }
    }

    #[test]
    fn frozen_command_has_literal_argv_pinned_cwd_and_offline_environment() {
        let root = fixture();
        let app = open_app(&root.path().join("Cargo.toml")).unwrap();
        for (dev, release, expected) in [
            (
                false,
                false,
                vec!["build", "--frozen", "--manifest-path", "Cargo.toml"],
            ),
            (
                false,
                true,
                vec![
                    "build",
                    "--frozen",
                    "--manifest-path",
                    "Cargo.toml",
                    "--release",
                ],
            ),
            (
                true,
                false,
                vec!["run", "--frozen", "--manifest-path", "Cargo.toml"],
            ),
        ] {
            let command = crate::platform::process::command(&app, dev, release).unwrap();
            assert_eq!(command.get_program(), "cargo");
            assert_eq!(command.get_args().collect::<Vec<_>>(), expected);
            assert_eq!(command.get_current_dir(), Some(root.path()));
            let env = command
                .get_envs()
                .collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(
                env.get(std::ffi::OsStr::new("RUSTUP_AUTO_INSTALL")),
                Some(&Some(std::ffi::OsStr::new("0")))
            );
            assert_eq!(
                env.get(std::ffi::OsStr::new("CARGO_NET_OFFLINE")),
                Some(&Some(std::ffi::OsStr::new("true")))
            );
        }
    }
}
