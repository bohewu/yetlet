#![cfg(windows)]
use std::{fs, process::Command};

#[test]
fn new_uses_shared_standalone_templates_without_cargo_or_network() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("apps with spaces");
    let empty_path = root.path().join("no-executables");
    fs::create_dir(&parent).unwrap();
    fs::create_dir(&empty_path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "demo", "--parent"])
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
        manifest["dependencies"]["yetlet"]["version"].as_str(),
        Some("=0.9.0-alpha.5")
    );
    assert_eq!(
        manifest["dependencies"]["yetlet"]["version"].as_str(),
        Some("=0.9.0-alpha.5")
    );
    assert_eq!(
        fs::read(app.join("src/main.rs")).unwrap(),
        include_bytes!("../templates/main.rs")
    );
    assert_eq!(
        fs::read_to_string(app.join("README.md")).unwrap(),
        include_str!("../templates/SELECTED-README.md")
            .replace("@NAME@", "demo")
            .replace("@VERSION@", env!("CARGO_PKG_VERSION"))
    );
    assert!(!app.join("Cargo.lock").exists());
    assert!(!app.join("target").exists());
    assert!(!root.path().join("must-not-exist").exists());
    assert!(app.join("base.toml").is_file());
    assert!(!app.join("src/config.rs").exists());
}

#[test]
fn single_letter_name_publishes_complete_project_without_stage_residue() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("projects");
    fs::create_dir(&parent).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "a", "--parent"])
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
        include_str!("../templates/SELECTED-README.md")
            .replace("@NAME@", "a")
            .replace("@VERSION@", env!("CARGO_PKG_VERSION"))
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
fn doctor_reports_windows_new_availability_and_framework_not_checked() {
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["doctor", "--json"])
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
        assert_eq!(
            diagnostic["status"],
            if capability == "new-platform" {
                "pass"
            } else {
                "not-checked"
            }
        );
    }
}
