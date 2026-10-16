use super::*;
use crate::docker_fixture as fixture;
use std::{ffi::OsString, fs as disk, path::PathBuf};

struct Fixture {
    // Retain the OS TempDir lifetime; actual output uses a separate protected parent.
    _root: tempfile::TempDir,
    output_parent: PathBuf,
    app: PathBuf,
    framework: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            disk::set_permissions(root.path(), disk::Permissions::from_mode(0o700)).unwrap();
        }
        let framework = root.path().join("框 架");
        fixture::create(&framework);
        scaffold::create_cloud("demo_app", &framework, root.path()).unwrap();
        // A separate CLI-created app is an owner-protected output parent on
        // Windows; never disable the Docker generator ACL gate for a fixture.
        scaffold::create_cloud("output_parent", &framework, root.path()).unwrap();
        let output_parent = root.path().join("output_parent");
        let app = root.path().join("demo_app");
        disk::write(
            app.join("Cargo.lock"),
            "version=4\n[[package]]\nname='demo_app'\nversion='0.1.0'\n",
        )
        .unwrap();
        Self {
            _root: root,
            output_parent,
            app,
            framework,
        }
    }
    fn manifest(&self) -> PathBuf {
        self.app.join("Cargo.toml")
    }
    fn generate(&self) -> Result<(), String> {
        generate(
            &self.manifest(),
            &self.output_parent.join("container 設 定"),
        )
    }
    fn refused(&self) {
        assert!(self.generate().is_err());
        assert!(!self.output_parent.join("container 設 定").exists());
    }
}

#[test]
fn literal_docker_command_is_distinct_strict_and_requires_both_paths() {
    let argv = |parts: &[&str]| parts.iter().map(OsString::from).collect::<Vec<_>>();
    assert_eq!(
        crate::parse(&argv(&[
            "generate",
            "docker",
            "--output",
            "設定 ;$(literal)",
            "--manifest-path",
            "應 用/Cargo.toml"
        ])),
        Ok(crate::Cli::GenerateDocker {
            manifest: "應 用/Cargo.toml".into(),
            output: "設定 ;$(literal)".into()
        })
    );
    for parts in [
        vec!["generate"],
        vec!["generate", "compose"],
        vec!["generate", "k8s"],
        vec!["generate", "docker"],
        vec!["generate", "docker", "--manifest-path", "Cargo.toml"],
        vec!["generate", "docker", "--output", "out"],
        vec![
            "generate",
            "docker",
            "--manifest-path",
            "Cargo.toml",
            "--output",
            "",
        ],
        vec![
            "generate",
            "docker",
            "--manifest-path",
            "Cargo.toml",
            "--output",
            "out",
            "--output",
            "out",
        ],
        vec![
            "generate",
            "docker",
            "--manifest-path",
            "Cargo.toml",
            "--manifest-path",
            "Cargo.toml",
            "--output",
            "out",
        ],
        vec![
            "generate",
            "docker",
            "--manifest-path=Cargo.toml",
            "--output",
            "out",
        ],
        vec![
            "generate",
            "docker",
            "--manifest-path",
            "Cargo.toml",
            "--output",
            "out",
            "--cloud",
        ],
        vec!["new", "demo", "--output", "out"],
    ] {
        assert!(crate::parse(&argv(&parts)).is_err(), "{parts:?}");
    }
}

#[test]
fn deterministic_selected_files_exact_overlay_and_minimal_workspace_without_leaks() {
    let f = Fixture::new();
    let secret = concat!("SYNTHETIC_SECRET_", "MUST_NEVER_EXPORT");
    for name in [".env", ".env.production", "id_rsa"] {
        disk::write(f.app.join(name), secret).unwrap();
    }
    for name in ["target", ".git", ".ssh"] {
        disk::create_dir(f.app.join(name)).unwrap();
        disk::write(f.app.join(name).join("secret"), secret).unwrap();
    }
    let before = disk::read(f.manifest()).unwrap();
    let pin = fs::open_app(&f.manifest()).unwrap();
    let snapshot = inspect(&pin).unwrap();
    let first = render(&snapshot).unwrap();
    assert_eq!(first, render(&snapshot).unwrap());
    assert_eq!(
        first.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [
            "Dockerfile",
            ".dockerignore",
            "cargo-overlay/Cargo.toml",
            "framework-workspace.toml",
            "app-src.dockerignore",
            "yetlet-src.dockerignore",
            "README.md",
            "build-context-inventory.json"
        ]
    );
    let values: BTreeMap<_, _> = first.iter().map(|(p, v)| (*p, v)).collect();
    let original: Value = parse(&before).unwrap();
    let mut overlay: Value = values["cargo-overlay/Cargo.toml"].parse().unwrap();
    for name in ["yetlet-core", "yetlet-server-axum"] {
        assert_eq!(
            overlay["dependencies"][name]["path"].as_str(),
            Some(format!("/work/yetlet/crates/{name}").as_str())
        );
        overlay["dependencies"][name]["path"] = original["dependencies"][name]["path"].clone();
    }
    assert_eq!(overlay, original);
    let minimal: Value = values["framework-workspace.toml"].parse().unwrap();
    let root = parse(&disk::read(f.framework.join("Cargo.toml")).unwrap()).unwrap();
    assert_eq!(
        minimal["workspace"]["package"],
        root["workspace"]["package"]
    );
    assert_eq!(
        minimal["workspace"]["dependencies"],
        root["workspace"]["dependencies"]
    );
    assert_eq!(
        minimal["workspace"]
            .as_table()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["dependencies", "members", "package", "resolver"]
    );
    assert_eq!(minimal["workspace"]["members"].as_array().unwrap().len(), 3);
    let inventory: serde_json::Value =
        serde_json::from_str(values["build-context-inventory.json"]).unwrap();
    assert_eq!(
        inventory["app-src"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [".dockerignore", "Cargo.lock", "src/main.rs"]
    );
    assert_eq!(inventory["yetlet-src"].as_object().unwrap().len(), 14);
    assert_eq!(
        inventory["app-src"]["Cargo.lock"]["sha256"],
        digest(&snapshot.lock)
    );
    for (_, text) in &first {
        assert!(!text.contains(secret));
        assert!(!text.contains(f.framework.to_str().unwrap()));
        assert!(!text.contains(f.app.to_str().unwrap()));
    }
    assert!(values["Dockerfile"].contains("COPY --from=app-src"));
    assert!(values["Dockerfile"].contains("COPY --from=yetlet-src"));
    assert!(values["Dockerfile"].contains("cargo metadata --locked"));
    assert!(values["Dockerfile"].contains("cargo build --locked --release"));
    assert!(!values["Dockerfile"].contains("COPY ."));
    f.generate().unwrap();
    assert_eq!(disk::read(f.manifest()).unwrap(), before);
    for (path, content) in first {
        assert_eq!(
            disk::read(f.output_parent.join("container 設 定").join(path)).unwrap(),
            content.as_bytes()
        );
    }
}

#[test]
fn source_marker_alone_default_manual_edited_and_extra_source_are_rejected() {
    for mutation in 0..7 {
        let f = Fixture::new();
        let main = f.app.join("src/main.rs");
        match mutation {
            0 => disk::write(main, include_str!("../../templates/main.rs")).unwrap(),
            1 => disk::write(main, "fn main() {}\n").unwrap(),
            2 => {
                let source = disk::read_to_string(&main).unwrap();
                disk::write(main, source.replace("0.0.0.0", "127.0.0.2")).unwrap();
            }
            3 => disk::write(main, "this is not Rust").unwrap(),
            4 => disk::write(f.app.join("src/lib.rs"), "").unwrap(),
            5 => disk::write(f.app.join("build.rs"), "fn main(){}\n").unwrap(),
            _ => {
                disk::create_dir(f.app.join(".cargo")).unwrap();
            }
        }
        f.refused();
    }
}

#[test]
fn strict_generated_manifest_lock_and_framework_closure_gating() {
    for mutation in 0..11 {
        let f = Fixture::new();
        let original = disk::read_to_string(f.manifest()).unwrap();
        match mutation {
            0 => disk::write(
                f.manifest(),
                original.replace("native-ssr-v1", "unverified"),
            )
            .unwrap(),
            1 => disk::write(f.manifest(), original.replace("signal", "process")).unwrap(),
            2 => disk::write(f.manifest(), format!("{original}\n[features]\nextra=[]\n")).unwrap(),
            3 => disk::write(
                f.manifest(),
                format!("{original}\n[build-dependencies]\nextra='1'\n"),
            )
            .unwrap(),
            4 => disk::remove_file(f.app.join("Cargo.lock")).unwrap(),
            5 => disk::write(f.app.join("Cargo.lock"), "invalid").unwrap(),
            6 => {
                let path = f.framework.join("crates/yetlet-html/Cargo.toml");
                let data = disk::read_to_string(&path).unwrap();
                disk::write(path, format!("{data}\nunknown={{path='../unknown'}}\n")).unwrap();
            }
            7 => {
                let path = f.framework.join("Cargo.toml");
                let data = disk::read_to_string(&path).unwrap();
                disk::write(
                    path,
                    data.replace("rust-version='1.99'", "rust-version='1.98'"),
                )
                .unwrap();
            }
            8 => disk::write(
                f.framework.join("crates/yetlet-core/build.rs"),
                "fn main(){}\n",
            )
            .unwrap(),
            9 => disk::write(f.framework.join("crates/yetlet-core/src/unknown.rs"), "").unwrap(),
            _ => {
                let path = f.framework.join("crates/yetlet-core/Cargo.toml");
                let data = disk::read_to_string(&path).unwrap();
                disk::write(
                    path,
                    data.replace("license.workspace=true", "unknown.workspace=true"),
                )
                .unwrap();
            }
        }
        f.refused();
    }
}

#[test]
fn output_traversal_existing_outputs_and_missing_parents_fail_without_writes() {
    let f = Fixture::new();
    for output in [
        f.output_parent.join("child/../escape"),
        f.output_parent.join("../escape"),
        f.output_parent.join("missing/out"),
        f.app.clone(),
    ] {
        assert!(generate(&f.manifest(), &output).is_err());
    }
    f.generate().unwrap();
    let marker = f.output_parent.join("container 設 定/README.md");
    let bytes = disk::read(&marker).unwrap();
    assert!(f.generate().is_err());
    assert_eq!(disk::read(marker).unwrap(), bytes);
    assert!(!f.output_parent.join("escape").exists());
}

#[test]
fn source_change_between_inspection_and_publication_is_refused() {
    let f = Fixture::new();
    let output = f.output_parent.join("out");
    assert!(
        generate_inner(&f.manifest(), &output, || {
            disk::write(f.app.join("src/main.rs"), "fn main(){}\n").unwrap();
        })
        .is_err()
    );
    assert!(!output.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn no_follow_app_or_output_symlinks_and_pinned_app_rename_race() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    for mutation in 0..6 {
        let f = Fixture::new();
        match mutation {
            0 => {
                disk::rename(f.app.join("src"), f.app.join("saved-src")).unwrap();
                symlink("saved-src", f.app.join("src")).unwrap();
            }
            1 => {
                disk::rename(f.app.join("src/main.rs"), f.app.join("saved-main")).unwrap();
                symlink("../saved-main", f.app.join("src/main.rs")).unwrap();
            }
            2 => {
                disk::rename(f.app.join("Cargo.lock"), f.app.join("saved-lock")).unwrap();
                symlink("saved-lock", f.app.join("Cargo.lock")).unwrap();
            }
            3 => {
                symlink(&f.app, f.output_parent.join("container 設 定")).unwrap();
            }
            4 => {
                disk::set_permissions(&f.output_parent, disk::Permissions::from_mode(0o777))
                    .unwrap();
            }
            _ => {
                disk::rename(
                    f.framework.join("crates/yetlet-core/src/lib.rs"),
                    f.framework.join("saved.rs"),
                )
                .unwrap();
                symlink(
                    f.framework.join("saved.rs"),
                    f.framework.join("crates/yetlet-core/src/lib.rs"),
                )
                .unwrap();
            }
        }
        assert!(f.generate().is_err());
    }
    let f = Fixture::new();
    let output = f.output_parent.join("out");
    let result = generate_inner(&f.manifest(), &output, || {
        disk::rename(&f.app, f._root.path().join("moved")).unwrap();
        disk::create_dir(&f.app).unwrap();
    });
    assert!(result.is_err());
    assert!(!output.exists());
}
