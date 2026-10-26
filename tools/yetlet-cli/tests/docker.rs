#![cfg(any(target_os = "linux", windows))]
use std::{fs, path::Path, process::Command};
#[path = "fixtures/docker_framework.rs"]
mod fixture;

fn regular_files(root: &Path, relative: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(root.join(relative)).unwrap() {
        let entry = entry.unwrap();
        let path = relative.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            regular_files(root, &path, files);
        } else {
            files.push(path.to_str().unwrap().replace('\\', "/"));
        }
    }
}

#[test]
fn binary_generates_only_selected_configuration_with_no_tools_cargo_cache_or_secret_access() {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let framework = root.path().join("framework space 框");
    fixture::create(&framework);
    yetlet_cli::scaffold::create_service("cloudapp", &framework, root.path()).unwrap();
    // Use a separate CLI-created owner-protected output parent; the selected
    // cloud app must not receive a generated deployment directory.
    yetlet_cli::scaffold::create_service("output_parent", &framework, root.path()).unwrap();
    let output_parent = root.path().join("output_parent");
    let app = root.path().join("cloudapp");
    fs::write(
        app.join("Cargo.lock"),
        "version=4\n[[package]]\nname='cloudapp'\nversion='0.1.0'\n",
    )
    .unwrap();
    let secret = concat!("SYNTHETIC_APP_ENV_SSH_", "GIT_TARGET_SECRET");
    for dir in [".ssh", ".git", "target"] {
        fs::create_dir(app.join(dir)).unwrap();
        fs::write(app.join(dir).join("private"), secret).unwrap();
    }
    fs::write(app.join(".env"), secret).unwrap();
    let source = fs::read(app.join("src/main.rs")).unwrap();
    let manifest = fs::read(app.join("Cargo.toml")).unwrap();
    let lock = fs::read(app.join("Cargo.lock")).unwrap();
    let destination = output_parent.join("container space 設定");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .args(["generate", "docker", "--manifest-path"])
            .arg(app.join("Cargo.toml"))
            .arg("--output")
            .arg(&destination)
            .env("PATH", root.path().join("no-tools"))
            .env("CARGO_HOME", root.path().join("no-cargo-home"))
            .env("DOCKER_HOST", "invalid://generation-must-not-connect")
            .env("SYNTHETIC_SECRET", secret)
            .output()
            .unwrap()
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(secret));
    let mut files = Vec::new();
    regular_files(&destination, Path::new(""), &mut files);
    files.sort();
    assert_eq!(
        files,
        [
            ".dockerignore",
            "Dockerfile",
            "README.md",
            "app-src.dockerignore",
            "build-context-inventory.json",
            "cargo-overlay/Cargo.toml",
            "framework-workspace.toml",
            "yetlet-src.dockerignore"
        ]
    );
    for file in &files {
        let bytes = fs::read_to_string(destination.join(file)).unwrap();
        assert!(!bytes.contains(secret));
        assert!(!bytes.contains(framework.to_str().unwrap()));
        assert!(!bytes.contains(app.to_str().unwrap()));
    }
    assert_eq!(run().status.code(), Some(1));
    assert_eq!(fs::read(app.join("src/main.rs")).unwrap(), source);
    assert_eq!(fs::read(app.join("Cargo.toml")).unwrap(), manifest);
    assert_eq!(fs::read(app.join("Cargo.lock")).unwrap(), lock);
    assert!(!root.path().join("no-cargo-home").exists());
    assert_eq!(
        fs::read_dir(root.path()).unwrap().count(),
        3,
        "no staging residue or tool side effects"
    );
    let bad = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args([
            "generate",
            "docker",
            "--output",
            "out",
            "--output",
            "duplicate",
            "--manifest-path",
        ])
        .arg(app.join("Cargo.toml"))
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
}
