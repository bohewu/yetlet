#![cfg(windows)]
use std::{fs, path::Path, process::Command};

fn framework(path: &Path) {
    fs::create_dir_all(path).unwrap();
    fs::write(
        path.join("Cargo.toml"),
        "[workspace]\n[workspace.package]\nrust-version='1.99'\n",
    )
    .unwrap();
    for name in ["yetlet-core", "yetlet-server-axum"] {
        let dir = path.join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname='{name}'\n"),
        )
        .unwrap();
        fs::write(dir.join("src/lib.rs"), "// local fixture\n").unwrap();
    }
}

#[test]
fn new_uses_shared_standalone_templates_without_cargo_or_network() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("framework 雪 😀");
    let parent = root.path().join("apps with spaces");
    let empty_path = root.path().join("no-executables");
    framework(&source);
    fs::create_dir(&parent).unwrap();
    fs::create_dir(&empty_path).unwrap();
    let before = fs::read(source.join("Cargo.toml")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "demo", "--framework-path"])
        .arg(&source)
        .arg("--dir")
        .arg(&parent)
        .env("PATH", &empty_path)
        .env("CARGO_HOME", root.path().join("must-not-exist"))
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let app = parent.join("demo");
    let manifest: toml::Value = fs::read_to_string(app.join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(manifest["package"]["publish"].as_bool(), Some(false));
    assert_eq!(manifest["workspace"]["resolver"].as_str(), Some("3"));
    assert_eq!(
        manifest["dependencies"]["yetlet-core"]["path"].as_str(),
        source.join("crates/yetlet-core").to_str()
    );
    assert_eq!(
        manifest["dependencies"]["yetlet-server-axum"]["path"].as_str(),
        source.join("crates/yetlet-server-axum").to_str()
    );
    assert_eq!(
        fs::read(app.join("src/main.rs")).unwrap(),
        include_bytes!("../templates/main.rs")
    );
    assert_eq!(
        fs::read_to_string(app.join("README.md")).unwrap(),
        include_str!("../templates/README.md").replace("@NAME@", "demo")
    );
    assert!(!app.join("Cargo.lock").exists());
    assert!(!app.join("target").exists());
    assert!(!root.path().join("must-not-exist").exists());
    assert_eq!(fs::read(source.join("Cargo.toml")).unwrap(), before);
}

#[test]
fn single_letter_name_publishes_complete_project_without_stage_residue() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("framework");
    let parent = root.path().join("projects");
    framework(&source);
    fs::create_dir(&parent).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "a", "--framework-path"])
        .arg(&source)
        .arg("--dir")
        .arg(&parent)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let app = parent.join("a");
    assert_eq!(
        fs::read_to_string(app.join("src/main.rs")).unwrap(),
        include_str!("../templates/main.rs")
    );
    let manifest: toml::Value = fs::read_to_string(app.join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(manifest["package"]["name"].as_str(), Some("a"));
    assert_eq!(
        fs::read_to_string(app.join("README.md")).unwrap(),
        include_str!("../templates/README.md").replace("@NAME@", "a")
    );
    assert!(app.join(".gitignore").is_file());
    assert!(!app.join("Cargo.lock").exists());
    let remaining: Vec<_> = fs::read_dir(&parent)
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(
        remaining,
        [std::ffi::OsString::from("a")],
        "no staging residue or foreign entry"
    );
}

#[test]
fn doctor_reports_windows_new_availability_and_selected_framework() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("framework 雪");
    framework(&source);
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["doctor", "--json", "--framework-path"])
        .arg(source)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    for capability in ["new-platform", "framework"] {
        let diagnostic = diagnostics
            .iter()
            .find(|d| d["capability"] == capability)
            .unwrap();
        assert_eq!(diagnostic["status"], "pass");
    }
}
