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
}

pub(super) struct AppFiles {
    app: App,
    parent: Anchor,
    private_cwd: bool,
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
        let parent = Anchor::open(&cwd)
            .map_err(|_| "app cwd is missing or unsafe (symlinks are forbidden)")?;
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
        Ok(Self {
            app: App { directory, cwd },
            parent,
            private_cwd,
        })
    }

    pub(super) fn read(&self, name: &str) -> Result<File, String> {
        read(&self.parent, name, self.private_cwd)
    }

    pub(super) fn finish(self) -> Result<App, String> {
        self.parent
            .verify()
            .map_err(|_| "app cwd identity changed")?;
        Ok(self.app)
    }
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
