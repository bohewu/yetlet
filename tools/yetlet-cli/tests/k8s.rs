#![cfg(any(target_os = "linux", windows))]
use std::{fs, path::Path, process::Command};
use yetlet_cli::scaffold;
#[path = "fixtures/docker_framework.rs"]
mod docker_fixture;
#[path = "fixtures/compose_app.rs"]
mod fixture;

const IMAGE: &str = "localhost:5000/team/app:v1";

fn generate(manifest: &Path, output: &Path, root: &Path, tail: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["generate", "k8s", "--manifest-path"])
        .arg(manifest)
        .arg("--output")
        .arg(output)
        .args(["--image", IMAGE])
        .args(tail)
        .env("PATH", root.join("no-tools"))
        .env("CARGO_HOME", root.join("no-cargo-home"))
        .env("DOCKER_HOST", "invalid://must-not-connect")
        .env("KUBECONFIG", root.join("no-kube-config"))
        .env("SYNTHETIC_SECRET", "SYNTHETIC_K8S_CLI_SECRET")
        .output()
        .unwrap()
}

fn files(output: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = fs::read_dir(output)
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            (
                path.file_name().unwrap().to_str().unwrap().into(),
                fs::read(path).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn settings_rejection_keeps_generator_exit_wrapper_and_no_output_tree() {
    let f = fixture::Fixture::new();
    fs::write(f.app.join("base.toml"), "[cli.vite]\nenabled=false\nroot='frontend/'\nauto_start=false\nauto_build=false\n[cli.generate.k8s]\nreplicas=1\n").unwrap();
    fs::write(f.app.join("production.toml"), "").unwrap();
    let output = f.parent.join("SYNTHETIC_SETTINGS_DESTINATION");
    let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("APP_") || key == "YETLET_CONFIG_ROOT" {
            command.env_remove(key);
        }
    }
    let result = command
        .args(["generate", "k8s", "--manifest-path"])
        .arg(f.manifest())
        .arg("--output")
        .arg(&output)
        .args(["--image", IMAGE])
        .env("APP_CLI__GENERATE__K8S__REPLICAS", "101")
        .env("PATH", f.root.path().join("no-tools"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        "CLI_SOURCE_UNSUPPORTED generate k8s: CLI_PROJECT settings values: check public TOML syntax, base-declared keys/types, ranges and target/TLS requirements\n"
    );
    assert!(!output.exists());
}

#[test]
fn binary_k8s_is_deterministic_tool_free_and_preserves_source_secrets_and_destinations() {
    let f = fixture::Fixture::new();
    let before =
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap());
    let secret = concat!("SYNTHETIC_K8S_CLI_", "SECRET");
    for name in [".env", ".env.production", "credentials"] {
        fs::write(f.app.join(name), secret).unwrap();
    }
    let first = f.parent.join("k8s space 雪");
    let second = f.parent.join("second");
    for output in [&first, &second] {
        let result = generate(&f.manifest(), output, f.root.path(), &[]);
        assert_eq!(
            result.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let generated = files(&first);
    assert_eq!(generated, files(&second));
    assert_eq!(
        generated
            .iter()
            .map(|(p, _)| p.as_str())
            .collect::<Vec<_>>(),
        ["README.md", "deployment.yaml", "service.yaml"]
    );
    for (_, bytes) in &generated {
        let text = std::str::from_utf8(bytes).unwrap();
        for forbidden in [
            secret,
            f.root.path().to_str().unwrap(),
            f.framework.to_str().unwrap(),
        ] {
            assert!(!text.contains(forbidden));
        }
        assert!(text.ends_with('\n'));
        assert!(!text.contains('\r'));
    }
    let d: serde_json::Value =
        serde_json::from_slice(&fs::read(first.join("deployment.yaml")).unwrap()).unwrap();
    assert_eq!(d["spec"]["replicas"], 1);
    assert_eq!(d["metadata"]["name"], "composeapp");
    assert_eq!(
        generate(&f.manifest(), &first, f.root.path(), &[])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(files(&first), generated);
    let foreign = f.parent.join("foreign-file");
    fs::write(&foreign, "untouched").unwrap();
    assert_eq!(
        generate(&f.manifest(), &foreign, f.root.path(), &[])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(fs::read(foreign).unwrap(), b"untouched");
    for tail in [
        vec!["--replicas", "0"],
        vec!["--replicas", "+2"],
        vec!["--replicas", "101"],
        vec!["--replicas", "2", "--replicas", "3"],
        vec!["--image", IMAGE],
        vec!["--apply"],
        vec!["--namespace", "secret"],
    ] {
        assert_eq!(
            generate(
                &f.manifest(),
                &f.parent.join("invalid"),
                f.root.path(),
                &tail
            )
            .status
            .code(),
            Some(2)
        );
        assert!(!f.parent.join("invalid").exists());
    }
    assert_eq!(
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap()),
        before
    );
    for name in [".env", ".env.production", "credentials"] {
        assert_eq!(fs::read(f.app.join(name)).unwrap(), secret.as_bytes());
    }
    for name in ["no-tools", "no-cargo-home", "no-kube-config"] {
        assert!(!f.root.path().join(name).exists());
    }
    assert_eq!(
        fs::read_dir(&f.parent).unwrap().count(),
        7,
        "no staging residue"
    );
}

#[test]
fn missing_stale_unsupported_lock_profile_source_and_output_are_refused() {
    for mutation in 0..9 {
        let f = fixture::Fixture::new();
        let output = f.parent.join("out");
        match mutation {
            0 => fs::remove_file(f.app.join("Cargo.lock")).unwrap(),
            1 | 2 => {
                let text = fs::read_to_string(f.app.join("Cargo.lock")).unwrap();
                let text = if mutation == 1 {
                    text.replace("name='composeapp'", "name='stale'")
                } else {
                    text.replace("version=4", "version=3")
                };
                fs::write(f.app.join("Cargo.lock"), text).unwrap();
            }
            3 => {
                let text = fs::read_to_string(f.manifest()).unwrap();
                fs::write(f.manifest(), text.replace("native-ssr-v1", "unsupported")).unwrap();
            }
            4 => fs::write(f.app.join("src/main.rs"), "fn main() {}\n").unwrap(),
            5 => fs::write(f.app.join("src/lib.rs"), "// arbitrary extra source\n").unwrap(),
            6 => {
                scaffold::create_service("unsafe_name", &f.framework, f.root.path()).unwrap();
                let lock = fs::read_to_string(f.app.join("Cargo.lock"))
                    .unwrap()
                    .replace("composeapp", "unsafe_name");
                fs::write(f.root.path().join("unsafe_name/Cargo.lock"), lock).unwrap();
                let result = generate(
                    &f.root.path().join("unsafe_name/Cargo.toml"),
                    &output,
                    f.root.path(),
                    &[],
                );
                assert_eq!(result.status.code(), Some(1));
                assert!(String::from_utf8_lossy(&result.stderr).contains("underscores"));
                assert!(!output.exists());
                continue;
            }
            7 => {
                assert_eq!(
                    generate(
                        &f.manifest(),
                        &f.parent.join("../escape"),
                        f.root.path(),
                        &[]
                    )
                    .status
                    .code(),
                    Some(1)
                );
                assert!(!f.root.path().join("escape").exists());
                continue;
            }
            _ => {
                assert_eq!(
                    generate(
                        &f.manifest(),
                        &f.parent.join("missing/out"),
                        f.root.path(),
                        &[]
                    )
                    .status
                    .code(),
                    Some(1)
                );
                assert!(!f.parent.join("missing").exists());
                continue;
            }
        }
        assert_eq!(
            generate(&f.manifest(), &output, f.root.path(), &[])
                .status
                .code(),
            Some(1),
            "mutation {mutation}"
        );
        assert!(!output.exists());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn real_generated_cloud_app_with_explicit_offline_cargo_lock_is_supported() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let framework = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    scaffold::create_service("k8sproof", &framework, root.path()).unwrap();
    let app = root.path().join("k8sproof");
    // Explicit test-fixture preparation; the generator must never run Cargo.
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline", "--manifest-path"])
        .arg(app.join("Cargo.toml"))
        .env("CARGO_NET_OFFLINE", "true")
        .env("RUSTUP_AUTO_INSTALL", "0")
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let before =
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(app.join(p)).unwrap());
    let output = root.path().join("real-source 設定");
    // Persistent replicas use the same public base/profile mechanism. This
    // separate settings root keeps the qualified legacy source bytes unchanged.
    let settings_root = root.path().join("public-settings");
    fs::create_dir(&settings_root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&settings_root, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::write(settings_root.join("base.toml"), "[cli.vite]\nenabled=false\nroot='frontend/'\nauto_start=false\nauto_build=false\n[cli.generate.k8s]\nreplicas=3\n").unwrap();
    fs::write(settings_root.join("production.toml"), "").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["generate", "k8s", "--manifest-path"])
        .arg(app.join("Cargo.toml"))
        .arg("--output")
        .arg(&output)
        .args(["--image", IMAGE])
        .env("YETLET_CONFIG_ROOT", &settings_root)
        .env("APP_ENVIRONMENT", "Production")
        .env("PATH", root.path().join("no-tools"))
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let d: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("deployment.yaml")).unwrap()).unwrap();
    assert_eq!(d["spec"]["replicas"], 3);
    assert_eq!(d["metadata"]["name"], "k8sproof");
    assert_eq!(
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(app.join(p)).unwrap()),
        before
    );
    symlink(&app, root.path().join("alias")).unwrap();
    assert_eq!(
        generate(
            &root.path().join("alias/Cargo.toml"),
            &root.path().join("bad"),
            root.path(),
            &[]
        )
        .status
        .code(),
        Some(1)
    );
    symlink(&output, root.path().join("linked-output")).unwrap();
    assert_eq!(
        generate(
            &app.join("Cargo.toml"),
            &root.path().join("linked-output"),
            root.path(),
            &[]
        )
        .status
        .code(),
        Some(1)
    );
    assert_eq!(files(&output).len(), 3);
    assert!(!root.path().join("no-cargo-home").exists());
}
