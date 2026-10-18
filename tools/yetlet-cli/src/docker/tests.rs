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

// Structural fixtures exercise generation only; actual npm/Cargo builds are separate.
fn delivery_fixture(config: bool, assets: bool, plugin: bool) -> crate::cloud_fixture::Fixture {
    let f = crate::cloud_fixture::Fixture::new();
    if config {
        let dir = f.framework.join("crates/yetlet-config");
        disk::create_dir_all(dir.join("src")).unwrap();
        disk::write(
            dir.join("Cargo.toml"),
            include_bytes!("../../../../crates/yetlet-config/Cargo.toml"),
        )
        .unwrap();
        disk::write(
            dir.join("src/lib.rs"),
            include_bytes!("../../../../crates/yetlet-config/src/lib.rs"),
        )
        .unwrap();
    }
    let entries = if assets {
        scaffold::vite_files("composeapp", &f.framework, true, config, plugin).unwrap()
    } else {
        scaffold::config_files("composeapp", &f.framework, true).unwrap()
    };
    for (path, content) in entries {
        disk::write(f.app.join(path), content).unwrap();
    }
    let manifest = parse(&disk::read(f.manifest()).unwrap()).unwrap();
    let mut lock = parse(&disk::read(f.app.join("Cargo.lock")).unwrap()).unwrap();
    let packages = lock["package"].as_array_mut().unwrap();
    let app = packages
        .iter_mut()
        .find(|p| p["name"].as_str() == Some("composeapp"))
        .unwrap();
    app["dependencies"] = Value::Array(
        manifest["dependencies"]
            .as_table()
            .unwrap()
            .keys()
            .cloned()
            .map(Value::String)
            .collect(),
    );
    if assets {
        app["dependencies"].as_array_mut().unwrap().extend([
            Value::String("tempfile".into()),
            Value::String("tower".into()),
        ]);
        app["dependencies"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
    packages
        .iter_mut()
        .find(|p| p["name"].as_str() == Some("serde"))
        .unwrap()["version"] = Value::String("1.0.229".into());
    for (name, version, enabled) in [
        ("serde_json", "1.0.149", assets),
        ("tempfile", "3.27.0", assets),
        ("secrecy", "0.10.3", config),
        ("toml", "0.9.12+spec-1.1.0", config),
        ("dotenvy", "0.15.7", config),
        ("libc", "0.2.186", config),
    ] {
        if enabled {
            packages.push(format!("name='{name}'\nversion='{version}'\nsource='registry+https://github.com/rust-lang/crates.io-index'\nchecksum='{}'\n", "a".repeat(64)).parse().unwrap());
        }
    }
    if config {
        packages.push("name='yetlet-config'\nversion='0.0.0'\ndependencies=['dotenvy','libc','serde','toml']\n".parse().unwrap());
    }
    disk::write(f.app.join("Cargo.lock"), toml::to_string(&lock).unwrap()).unwrap();
    if assets {
        disk::create_dir_all(f.app.join("dist/.vite")).unwrap();
        disk::create_dir(f.app.join("dist/assets")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in ["dist", "dist/.vite", "dist/assets"] {
                disk::set_permissions(f.app.join(path), disk::Permissions::from_mode(0o700))
                    .unwrap();
            }
        }
        for file in [
            "app.js",
            "shared.js",
            "lazy.js",
            "companion.js",
            "app.css",
            "shared.css",
            "lazy.css",
            "mark.svg",
        ] {
            disk::write(f.app.join("dist/assets").join(file), file).unwrap();
        }
        disk::write(f.app.join("dist/.vite/manifest.json"), serde_json::json!({
            "app.ts": {"file":"assets/app.js","src":"app.ts","isEntry":true,"imports":["_shared.js"],"dynamicImports":["lazy.ts"],"css":["assets/app.css"],"assets":["assets/mark.svg"]},
            "_shared.js": {"file":"assets/shared.js","css":["assets/shared.css"]},
            "companion.ts": {"file":"assets/companion.js","isEntry":true,"imports":["_shared.js"]},
            "lazy.ts": {"file":"assets/lazy.js","isDynamicEntry":true,"css":["assets/lazy.css"]}
        }).to_string()).unwrap();
    }
    f
}

#[test]
fn delivery_profile_matrix_exact_inventory_env_and_no_secrets_or_developer_paths() {
    for (config, assets, plugin) in [
        (false, true, false),
        (false, true, true),
        (true, true, false),
        (true, true, true),
        (true, false, false),
    ] {
        let f = delivery_fixture(config, assets, plugin);
        let sentinel = "SYNTHETIC_BACKEND_ONLY_NEVER_EXPORT";
        for name in [".env", ".env.production", "private-key.pem"] {
            disk::write(f.app.join(name), sentinel).unwrap();
        }
        for name in ["node_modules", ".git"] {
            disk::create_dir(f.app.join(name)).unwrap();
            disk::write(f.app.join(name).join("private"), sentinel).unwrap();
        }
        let pinned = fs::open_app(&f.manifest()).unwrap();
        let snapshot = inspect(&pinned).unwrap();
        let files = render(&snapshot).unwrap();
        assert_eq!(files, render(&snapshot).unwrap());
        let files: BTreeMap<_, _> = files.into_iter().collect();
        let inv: serde_json::Value =
            serde_json::from_str(&files["build-context-inventory.json"]).unwrap();
        for (path, record) in inv["app-src"].as_object().unwrap() {
            let bytes = if path == ".dockerignore" {
                files["app-src.dockerignore"].as_bytes().to_vec()
            } else {
                disk::read(f.app.join(path)).unwrap()
            };
            assert_eq!(record["sha256"], digest(&bytes));
            assert_eq!(record["bytes"], bytes.len());
        }
        assert_eq!(
            inv["yetlet-src"].as_object().unwrap().len(),
            if config { 16 } else { 14 }
        );
        assert_eq!(inv["app-src"].get("src/config.rs").is_some(), config);
        assert_eq!(inv["app-src"].get("base.toml").is_some(), config);
        assert_eq!(
            inv["app-src"].get("dist/.vite/manifest.json").is_some(),
            assets
        );
        assert_eq!(
            files["Dockerfile"].contains("YETLET_ASSET_ROOT=/opt/yetlet/assets"),
            assets
        );
        assert_eq!(
            files["Dockerfile"].contains("YETLET_CONFIG_ROOT=/opt/yetlet/config"),
            config
        );
        // Private 0600/0700 BuildKit context exports must still work with
        // the selected non-root container UID, without altering legacy output.
        let permission_roots = match (assets, config) {
            (true, true) => Some("/opt/yetlet/assets /opt/yetlet/config"),
            (true, false) => Some("/opt/yetlet/assets"),
            (false, true) => Some("/opt/yetlet/config"),
            (false, false) => None,
        };
        if let Some(roots) = permission_roots {
            assert!(files["Dockerfile"].contains(&format!("RUN chmod -R a+rX {roots}\n")));
        } else {
            assert!(!files["Dockerfile"].contains("RUN chmod -R a+rX"));
        }
        assert!(!files["Dockerfile"].contains("APP_SERVICE__TOKEN="));
        let overlay: Value = files["cargo-overlay/Cargo.toml"].parse().unwrap();
        if config {
            assert_eq!(
                overlay["dependencies"]["yetlet-config"]["path"].as_str(),
                Some("/work/yetlet/crates/yetlet-config")
            );
        }
        for text in files.values() {
            assert!(!text.contains(sentinel));
            assert!(!text.contains(f.app.to_str().unwrap()));
            assert!(!text.contains(f.framework.to_str().unwrap()));
        }
        generate(&f.manifest(), &f.parent.join("docker")).unwrap();
        crate::compose::generate(
            &f.manifest(),
            &f.parent.join("compose"),
            "localhost:5000/yt101:local",
        )
        .unwrap();
        crate::k8s::generate(
            &f.manifest(),
            &f.parent.join("k8s"),
            "localhost:5000/yt101:local",
            1,
        )
        .unwrap();
        let yaml = disk::read_to_string(f.parent.join("compose/compose.yaml")).unwrap();
        assert_eq!(
            yaml.contains("APP_SERVICE__TOKEN: '${APP_SERVICE__TOKEN:?Set APP_SERVICE__TOKEN}'"),
            config
        );
        assert_eq!(
            yaml.contains("YETLET_ASSET_ROOT: '/opt/yetlet/assets'"),
            assets
        );
        let deploy: serde_json::Value =
            serde_json::from_slice(&disk::read(f.parent.join("k8s/deployment.yaml")).unwrap())
                .unwrap();
        let container = &deploy["spec"]["template"]["spec"]["containers"][0];
        assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
        assert_eq!(container["image"], "localhost:5000/yt101:local");
        let env = container["env"].as_array().unwrap();
        let token = env.iter().find(|v| v["name"] == "APP_SERVICE__TOKEN");
        assert_eq!(token.is_some(), config);
        if let Some(token) = token {
            assert!(token.get("value").is_none());
            assert_eq!(
                token["valueFrom"]["secretKeyRef"],
                serde_json::json!({"name":"composeapp-runtime","key":"service-token","optional":false})
            );
        }
    }
}

#[test]
fn delivery_refuses_attacker_inputs_stale_missing_and_unbounded_artifacts() {
    for mutation in 0..17 {
        let f = delivery_fixture(true, true, true);
        let manifest = f.app.join("dist/.vite/manifest.json");
        let mut graph: serde_json::Value =
            serde_json::from_slice(&disk::read(&manifest).unwrap()).unwrap();
        match mutation {
            0 => disk::write(
                f.app.join("base.toml"),
                include_str!("../../templates/config/base.toml")
                    .replace("token = \"\"", "token = \"PRIVATE_SENTINEL\""),
            )
            .unwrap(),
            1 => disk::write(
                f.app.join("production.toml"),
                "[service]\ntoken='PRIVATE_SENTINEL'\n",
            )
            .unwrap(),
            2 => disk::write(f.app.join("src/config.rs"), "// altered\n").unwrap(),
            3 => disk::write(f.app.join("vite.config.mjs"), "// unsafe plugin\n").unwrap(),
            4 => disk::write(f.app.join("dist/assets/app.js.map"), "PRIVATE_SENTINEL").unwrap(),
            5 => disk::write(f.app.join("dist/.env"), "PRIVATE_SENTINEL").unwrap(),
            6 => disk::remove_file(f.app.join("dist/assets/lazy.js")).unwrap(),
            7 => {
                graph["app.ts"]["file"] = "../private.js".into();
                disk::write(&manifest, graph.to_string()).unwrap();
            }
            8 => {
                graph["lazy.ts"]["dynamicImports"] = serde_json::json!(["app.ts"]);
                disk::write(&manifest, graph.to_string()).unwrap();
            }
            9 => {
                graph["app.ts"]["imports"] = serde_json::json!(["absent"]);
                disk::write(&manifest, graph.to_string()).unwrap();
            }
            10 => {
                graph["app.ts"]["css"] = serde_json::json!(["assets/app.js"]);
                disk::write(&manifest, graph.to_string()).unwrap();
            }
            11 => {
                graph["app.ts"]["unknown"] = true.into();
                disk::write(&manifest, graph.to_string()).unwrap();
            }
            12 => disk::File::options()
                .write(true)
                .open(&manifest)
                .unwrap()
                .set_times(disk::FileTimes::new().set_modified(std::time::SystemTime::UNIX_EPOCH))
                .unwrap(),
            13 => disk::File::options()
                .write(true)
                .open(f.app.join("dist/assets/app.js"))
                .unwrap()
                .set_len(1024 * 1024 + 1)
                .unwrap(),
            14 => disk::write(f.framework.join("crates/yetlet-config/src/extra.rs"), "").unwrap(),
            15 => disk::write(
                f.framework.join("crates/yetlet-config/src/lib.rs"),
                "// changed config closure\n",
            )
            .unwrap(),
            _ => disk::remove_file(&manifest).unwrap(),
        }
        let out = f.parent.join("refused");
        assert!(
            generate(&f.manifest(), &out).is_err(),
            "mutation {mutation}"
        );
        assert!(!out.exists());
    }
}

#[test]
fn delivery_revalidates_assets_public_config_and_frontend_inputs_before_publication() {
    for selected in ["dist/assets/app.js", "base.toml", "package-lock.json"] {
        let f = delivery_fixture(true, true, false);
        let out = f.parent.join("changed");
        assert!(
            generate_inner(&f.manifest(), &out, || {
                disk::write(f.app.join(selected), "changed").unwrap();
            })
            .is_err()
        );
        assert!(!out.exists());
    }
}

#[cfg(unix)]
#[test]
fn delivery_rejects_symlink_assets_config_and_manifest_paths() {
    use std::os::unix::fs::symlink;
    for selected in [
        "dist/assets/app.js",
        "dist/.vite/manifest.json",
        "base.toml",
        "src/config.rs",
        "dist/assets",
    ] {
        let f = delivery_fixture(true, true, false);
        let path = f.app.join(selected);
        let saved = f.app.join("saved");
        disk::rename(&path, &saved).unwrap();
        symlink(&saved, &path).unwrap();
        assert!(generate(&f.manifest(), &f.parent.join("refused")).is_err());
    }
}

#[test]
fn legacy_docker_template_outputs_retain_pre_yt101_bytes() {
    let f = Fixture::new();
    let app = fs::open_app(&f.manifest()).unwrap();
    let mut snapshot = inspect(&app).unwrap();
    snapshot.manifest["package"]["name"] = Value::String("legacy".into());
    let files: BTreeMap<_, _> = render(&snapshot).unwrap().into_iter().collect();
    for (path, hash) in [
        (
            "Dockerfile",
            "e345b1e83786ffe9a580cd64db02174967f89216092ff026b12d6cb830187d1e",
        ),
        (
            "README.md",
            "3a0df474b3c2ccf47707a987ec21490802977f4a372470088bccc0e208f0f4bb",
        ),
        (
            ".dockerignore",
            "4dbfc2de7ffa88f8b7a5895dba7cb86d7ce2370d311a9dd30ad71194e6174463",
        ),
        (
            "app-src.dockerignore",
            "303ea146ebcbf3f6caf512b12886effd25c9122b3658c55e64c57dbb3556c52d",
        ),
        (
            "yetlet-src.dockerignore",
            "01ab32213fc7cf4a5a2e0f8bcc07b47ef3f75f4b39e8a804f3f24c8a3c5bf9ff",
        ),
        (
            "framework-workspace.toml",
            "573adfb0e4bbedc9647fcf2767a901391a002971603046c00ec7d0bd8944726f",
        ),
        (
            "cargo-overlay/Cargo.toml",
            "16da14664d33cafc54df39569f50e16df238167ad4a835b7613277ae293b78d1",
        ),
    ] {
        assert_eq!(digest(files[path].as_bytes()), hash, "{path}");
    }
}

fn selected_fixture(config: bool, assets: bool, plugin: bool) -> crate::cloud_fixture::Fixture {
    let f = if config || assets {
        delivery_fixture(true, assets, plugin)
    } else {
        crate::cloud_fixture::Fixture::new()
    };
    let root = "web ui/前端";
    for (path, mut content) in
        scaffold::selected_files("composeapp", true, config, plugin, assets).unwrap()
    {
        if path == "base.toml" {
            content = content
                .replace("root = \"frontend/\"", &format!("root = \"{root}\""))
                .replace("replicas = 1", "replicas = 3")
                .replace("watch = false", "watch = true");
        }
        if path == "development.toml" {
            content.push_str("\n[cli.vite]\nroot = 'different-dev-root/'\n");
        }
        let path = path
            .strip_prefix("frontend/")
            .map(|tail| format!("{root}/{tail}"))
            .unwrap_or_else(|| path.into());
        let destination = f.app.join(path);
        disk::create_dir_all(destination.parent().unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut parent = destination.parent().unwrap();
            while parent != f.app {
                disk::set_permissions(parent, disk::Permissions::from_mode(0o700)).unwrap();
                parent = parent.parent().unwrap();
            }
        }
        disk::write(destination, content).unwrap();
    }
    let mut manifest = parse(&disk::read(f.manifest()).unwrap()).unwrap();
    for name in ["yetlet-core", "yetlet-server-axum", "yetlet-config"] {
        if manifest["dependencies"].get(name).is_some() {
            manifest["dependencies"][name]
                .as_table_mut()
                .unwrap()
                .insert(
                    "path".into(),
                    Value::String(
                        f.framework
                            .join(format!("crates/{name}"))
                            .to_str()
                            .unwrap()
                            .into(),
                    ),
                );
        }
    }
    disk::write(f.manifest(), toml::to_string(&manifest).unwrap()).unwrap();
    if assets {
        disk::rename(f.app.join("dist"), f.app.join(root).join("dist")).unwrap();
        // Structural graph only: refresh synthetic build timestamps after inputs.
        for dir in ["assets", ".vite"] {
            for entry in disk::read_dir(f.app.join(root).join("dist").join(dir)).unwrap() {
                let path = entry.unwrap().path();
                let bytes = disk::read(&path).unwrap();
                disk::write(path, bytes).unwrap();
            }
        }
    }
    f
}

#[test]
fn current_selected_profiles_keep_pinned_overrides_public_settings_and_asset_mapping() {
    for (config, assets, plugin) in [
        (false, false, false),
        (true, false, false),
        (true, true, false),
        (true, true, true),
    ] {
        let f = selected_fixture(config, assets, plugin);
        let app = fs::open_app(&f.manifest()).unwrap();
        let snapshot = inspect(&app).unwrap();
        let rendered: BTreeMap<_, _> = render(&snapshot).unwrap().into_iter().collect();
        let inventory: serde_json::Value =
            serde_json::from_str(&rendered["build-context-inventory.json"]).unwrap();
        assert!(inventory["app-src"].get("base.toml").is_some());
        assert_eq!(snapshot.with_config, config);
        assert_eq!(inventory["app-src"].get("src/config.rs").is_some(), config);
        assert_eq!(
            rendered["Dockerfile"].contains("YETLET_CONFIG_ROOT=/opt/yetlet/config"),
            config
        );
        assert!(rendered["Dockerfile"].contains("/opt/yetlet/config/base.toml"));
        assert_eq!(inventory["app-src-source-paths"]["base.toml"], "base.toml");
        assert!(
            inventory["app-src-source-paths"]
                .get(".dockerignore")
                .is_none()
        );
        if assets {
            assert_eq!(
                inventory["app-src-source-paths"]["dist/assets/app.js"],
                "web ui/前端/dist/assets/app.js"
            );
            assert!(inventory["app-src"].get("dist/assets/app.js").is_some());
            assert!(
                inventory["app-src"]
                    .get("web ui/前端/vite.dev.config.mjs")
                    .is_none()
            );
        }
    }
}

#[test]
fn current_selected_profiles_refuse_edited_sources_dependencies_and_runtime_public_tables() {
    for change in 0..7 {
        let f = selected_fixture(true, false, false);
        match change {
            0 => disk::write(f.app.join("src/config.rs"), "// edited\n").unwrap(),
            1 => {
                let mut manifest = parse(&disk::read(f.manifest()).unwrap()).unwrap();
                manifest["dependencies"]["yetlet-core"]["version"] = Value::String("*".into());
                disk::write(f.manifest(), toml::to_string(&manifest).unwrap()).unwrap();
            }
            _ => {
                let mut base = parse(&disk::read(f.app.join("base.toml")).unwrap()).unwrap();
                match change {
                    2 => base["service"]["token"] = Value::String("PRIVATE".into()),
                    3 => base["cli"]["vite"]["root"] = Value::String("../outside".into()),
                    4 => {
                        base["cli"]["vite"]
                            .as_table_mut()
                            .unwrap()
                            .insert("unknown".into(), Value::Boolean(true));
                    }
                    5 => base["cli"]["generate"]["k8s"]["replicas"] = Value::Integer(101),
                    _ => base["cli"]["rust"]["watch"] = Value::String("true".into()),
                }
                disk::write(f.app.join("base.toml"), toml::to_string(&base).unwrap()).unwrap();
            }
        }
        assert!(
            inspect(&fs::open_app(&f.manifest()).unwrap()).is_err(),
            "change {change}"
        );
    }
}
