#![cfg(any(target_os = "linux", windows))]
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const IMAGE: &str = "localhost:5000/team/selected:compat";
const SECRET: &str = "SYNTHETIC_GENERATOR_PRIVATE_VALUE";

fn cli(app: &Path, root: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("APP_") || key == "YETLET_CONFIG_ROOT" {
            command.env_remove(key);
        }
    }
    command
        .args(args)
        .current_dir(app)
        .env("APP_ENVIRONMENT", "Production")
        .env("PATH", root.join("no-tools"))
        .env("CARGO_HOME", root.join("no-cargo-home"))
        .env("DOCKER_HOST", "invalid://generation-must-not-connect")
        .env("APP_SERVICE__TOKEN", SECRET)
        .output()
        .unwrap()
}

fn success(result: Output) {
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

fn inventory(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, relative: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(root.join(relative)).unwrap() {
            let entry = entry.unwrap();
            let path = relative.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                walk(root, &path, files);
            } else {
                files.push((
                    path.to_str().unwrap().replace('\\', "/"),
                    fs::read(root.join(path)).unwrap(),
                ));
            }
        }
    }
    let mut files = vec![];
    walk(root, Path::new(""), &mut files);
    files.sort();
    files
}

#[test]
fn current_service_profiles_generate_all_outputs_with_explicit_offline_locks() {
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let framework = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    // Deployment output requires an owner-protected parent on Windows too.
    success(cli(root.path(), root.path(), &["new", "output-parent"]));
    let output_parent = root.path().join("output-parent");
    for (index, options) in [
        vec!["--service"],
        vec!["--service", "--settings"],
        vec!["--service", "--vite"],
        vec!["--service", "--vite", "--tailwind"],
    ]
    .iter()
    .enumerate()
    {
        let name = format!("selected-gen-{index}");
        let mut new = vec!["new", &name];
        new.extend(options.iter().copied());
        success(cli(root.path(), root.path(), &new));
        let app = root.path().join(&name);
        let path = app.join("Cargo.toml");
        let mut manifest: toml::Value = fs::read_to_string(&path).unwrap().parse().unwrap();
        for krate in ["yetlet-core", "yetlet-server-axum", "yetlet-config"] {
            if let Some(dependency) = manifest["dependencies"].get_mut(krate) {
                // Explicit Cargo-owned preparation retains the selected version.
                let dependency_path = framework.join("crates").join(krate);
                let dependency_path = dependency_path.to_str().unwrap();
                // Exercise native and Cargo-style separators across the profiles.
                let dependency_path = if index % 2 == 0 {
                    dependency_path.to_owned()
                } else {
                    dependency_path.replace('\\', "/")
                };
                dependency
                    .as_table_mut()
                    .unwrap()
                    .insert("path".into(), dependency_path.into());
            }
        }
        fs::write(&path, toml::to_string(&manifest).unwrap()).unwrap();
        let mut base: toml::Value = fs::read_to_string(app.join("base.toml"))
            .unwrap()
            .parse()
            .unwrap();
        base["cli"]["rust"]["watch"] = true.into();
        base["cli"]["vite"]["auto_start"] = false.into();
        base["cli"]["vite"]["auto_build"] = false.into();
        base["cli"]["generate"]["k8s"]["replicas"] = 3.into();
        fs::write(app.join("base.toml"), toml::to_string(&base).unwrap()).unwrap();
        let frontend = if index == 3 {
            fs::rename(app.join("frontend"), app.join("web-ui")).unwrap();
            let mut production: toml::Value = fs::read_to_string(app.join("production.toml"))
                .unwrap()
                .parse()
                .unwrap();
            production.as_table_mut().unwrap().insert(
                "cli".into(),
                toml::toml! { vite = { root = "web-ui/" } }.into(),
            );
            fs::write(
                app.join("production.toml"),
                toml::to_string(&production).unwrap(),
            )
            .unwrap();
            "web-ui"
        } else {
            "frontend"
        };
        // Explicit lock preparation belongs to the caller, never the generator.
        success(
            Command::new(env!("CARGO"))
                .args(["generate-lockfile", "--offline", "--manifest-path"])
                .arg(&path)
                .env("CARGO_NET_OFFLINE", "true")
                .env("RUSTUP_AUTO_INSTALL", "0")
                .output()
                .unwrap(),
        );
        if index >= 2 {
            let dist = app.join(frontend).join("dist");
            fs::create_dir_all(dist.join(".vite")).unwrap();
            fs::create_dir(dist.join("assets")).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // Explicit fixture preparation must not inherit a permissive host umask.
                for directory in [&dist, &dist.join(".vite"), &dist.join("assets")] {
                    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
                }
            }
            // Finite structural fixture only; actual npm/OCI proof is separate.
            fs::write(dist.join("assets/app.js"), "export {};\n").unwrap();
            fs::write(dist.join("assets/mark.svg"), "<svg/>\n").unwrap();
            fs::write(dist.join(".vite/manifest.json"), r#"{"app.ts":{"file":"assets/app.js","src":"app.ts","isEntry":true,"assets":["assets/mark.svg"]}}"#).unwrap();
        }
        fs::write(app.join(".env"), SECRET).unwrap();
        fs::create_dir(app.join(".ssh")).unwrap();
        fs::write(app.join(".ssh/private"), SECRET).unwrap();
        let before = inventory(&app);
        for kind in ["docker", "compose", "k8s"] {
            let destination = output_parent.join(format!("out-{index}-{kind}"));
            let mut args = vec!["generate", kind, "--output", destination.to_str().unwrap()];
            if kind != "docker" {
                args.extend(["--image", IMAGE]);
            }
            success(cli(&app, root.path(), &args));
            for (_, bytes) in inventory(&destination) {
                assert!(!String::from_utf8_lossy(&bytes).contains(SECRET));
            }
            if kind == "k8s" {
                let deployment: serde_json::Value =
                    serde_json::from_slice(&fs::read(destination.join("deployment.yaml")).unwrap())
                        .unwrap();
                assert_eq!(deployment["spec"]["replicas"], 3);
            }
            if kind == "docker" {
                let contexts: serde_json::Value = serde_json::from_slice(
                    &fs::read(destination.join("build-context-inventory.json")).unwrap(),
                )
                .unwrap();
                let mapping = contexts["app-src-source-paths"].as_object().unwrap();
                if index >= 2 {
                    assert_eq!(
                        mapping["dist/.vite/manifest.json"],
                        format!("{frontend}/dist/.vite/manifest.json")
                    );
                }
                let overlay: toml::Value =
                    fs::read_to_string(destination.join("cargo-overlay/Cargo.toml"))
                        .unwrap()
                        .parse()
                        .unwrap();
                assert_eq!(
                    overlay["dependencies"].get("yetlet-config").is_some(),
                    index > 0
                );
                assert!(
                    fs::read_to_string(destination.join("README.md"))
                        .unwrap()
                        .contains("app-src-source-paths")
                );
            }
        }
        assert_eq!(
            inventory(&app),
            before,
            "generation changed application inputs"
        );
        assert!(!root.path().join("no-cargo-home").exists());
        assert!(!root.path().join("no-tools").exists());
        if index == 3 {
            fs::write(app.join("src/main.rs"), "fn main() {}\n").unwrap();
            let destination = output_parent.join("edited-refused");
            assert!(
                !cli(
                    &app,
                    root.path(),
                    &[
                        "generate",
                        "docker",
                        "--output",
                        destination.to_str().unwrap()
                    ]
                )
                .status
                .success()
            );
            assert!(!destination.exists());
        }
    }
}
