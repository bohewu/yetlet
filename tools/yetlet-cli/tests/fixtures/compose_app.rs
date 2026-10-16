//! Structural lock fixture only; real Cargo buildability is a separate gate.
use std::{fs, path::PathBuf};

pub struct Fixture {
    pub root: tempfile::TempDir,
    pub framework: PathBuf,
    pub app: PathBuf,
    pub parent: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        }
        let framework = root.path().join("framework space 框");
        crate::docker_fixture::create(&framework);
        crate::scaffold::create_cloud("composeapp", &framework, root.path()).unwrap();
        crate::scaffold::create_cloud("output_parent", &framework, root.path()).unwrap();
        let app = root.path().join("composeapp");
        fs::write(app.join("Cargo.lock"), lock()).unwrap();
        let parent = root.path().join("output_parent");
        Self {
            root,
            framework,
            app,
            parent,
        }
    }
    pub fn manifest(&self) -> PathBuf {
        self.app.join("Cargo.toml")
    }
}

fn lock() -> String {
    let mut lock = "version=4\n".to_owned();
    for (name, dependencies) in [
        (
            "composeapp",
            "['axum','tokio','yetlet-core','yetlet-server-axum']",
        ),
        ("yetlet-core", "[]"),
        ("yetlet-html", "['yetlet-core']"),
        (
            "yetlet-server-axum",
            "['axum','serde','tower','yetlet-core','yetlet-html']",
        ),
    ] {
        lock.push_str(&format!(
            "[[package]]\nname='{name}'\nversion='0.1.0'\ndependencies={dependencies}\n"
        ));
    }
    for (name, version, dependencies) in [
        ("axum", "0.8.9", "['tokio','tower']"),
        ("tokio", "1.53.2", "['signal-hook-registry']"),
        ("signal-hook-registry", "1.4.8", "[]"),
        ("serde", "1.0.228", "[]"),
        ("tower", "0.5.3", "[]"),
    ] {
        lock.push_str(&format!("[[package]]\nname='{name}'\nversion='{version}'\nsource='registry+https://github.com/rust-lang/crates.io-index'\nchecksum='{}'\ndependencies={dependencies}\n", "a".repeat(64)));
    }
    lock
}
