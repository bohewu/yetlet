#![cfg(target_os = "linux")]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

fn cli(command: &str, path: &Path) -> Command {
    let mut result = Command::new(env!("CARGO_BIN_EXE_yetlet"));
    result.arg(command).arg("--manifest-path").arg(path);
    result
}

fn code(output: Output, expected: i32) -> Output {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn frozen_cli_build_run_and_lock_cache_failures_use_native_cargo() {
    let root = tempfile::Builder::new()
        .prefix("yt087b 空 間;$literal-")
        .tempdir()
        .unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let manifest = root.path().join("Cargo.toml");
    let lock = root.path().join("Cargo.lock");
    let manifest_text =
        "[package]\nname='yt087b_fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n";
    fs::write(&manifest, manifest_text).unwrap();
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0o600)).unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/main.rs"),
        "fn main() { std::process::exit(23); }\n",
    )
    .unwrap();
    code(cli("build", &manifest).output().unwrap(), 1);
    code(cli("run", &manifest).output().unwrap(), 1);
    assert!(!lock.exists());
    // Explicit fixture lock, independent of all repository/CLI lockfiles.
    fs::write(
        &lock,
        "version=4\n[[package]]\nname='yt087b_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
    let initial_lock = fs::read(&lock).unwrap();
    code(cli("build", &manifest).output().unwrap(), 0);
    code(cli("run", &manifest).output().unwrap(), 23);
    assert_eq!(fs::read(&lock).unwrap(), initial_lock);
    fs::write(
        &manifest,
        format!("{manifest_text}\n[dependencies]\nserde_json='=1.0.149'\n"),
    )
    .unwrap();
    for command in ["build", "run"] {
        let output = code(cli(command, &manifest).output().unwrap(), 101);
        assert!(String::from_utf8_lossy(&output.stderr).contains("lock"));
        assert_eq!(fs::read(&lock).unwrap(), initial_lock);
    }
    // Separate, explicit offline setup for the cache-negative test. Never CLI bootstrap.
    code(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(&manifest)
            .current_dir(root.path())
            .env("RUSTUP_AUTO_INSTALL", "0")
            .output()
            .unwrap(),
        0,
    );
    let cached_lock = fs::read(&lock).unwrap();
    let cache = tempfile::tempdir().unwrap();
    for command in ["build", "run"] {
        let output = code(
            cli(command, &manifest)
                .env("CARGO_HOME", cache.path())
                .output()
                .unwrap(),
            101,
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("offline"));
        assert_eq!(fs::read(&lock).unwrap(), cached_lock);
    }
}

#[test]
fn discovery_is_exclusive_scoped_and_needs_no_project_or_tools() {
    let root = tempfile::tempdir().unwrap();
    for arguments in [
        vec![],
        vec!["--help"],
        vec!["-h"],
        vec!["--version"],
        vec!["-V"],
        vec!["new", "--help"],
        vec!["doctor", "-h"],
        vec!["run", "--help"],
        vec!["build", "-h"],
        vec!["generate", "--help"],
        vec!["generate", "docker", "-h"],
        vec!["generate", "compose", "--help"],
        vec!["generate", "k8s", "-h"],
    ] {
        let output = code(
            Command::new(env!("CARGO_BIN_EXE_yetlet"))
                .args(&arguments)
                .current_dir(root.path())
                .env("PATH", "")
                .env("YETLET_CONFIG_ROOT", "missing")
                .output()
                .unwrap(),
            0,
        );
        assert!(!output.stdout.is_empty(), "{arguments:?}");
        assert!(output.stderr.is_empty(), "{arguments:?}");
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    for arguments in [
        vec!["--version", "secret-value"],
        vec!["new", "demo", "--help"],
        vec!["doctor", "--help", "--json"],
        vec!["generate"],
        vec!["generate", "k8s", "--help", "--output", "secret-value"],
        vec!["run", "--version"],
        vec!["build", "--release", "-h"],
    ] {
        let output = code(
            Command::new(env!("CARGO_BIN_EXE_yetlet"))
                .args(&arguments)
                .current_dir(root.path())
                .output()
                .unwrap(),
            2,
        );
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("CLI_USAGE"));
        assert!(!stderr.contains("secret-value"));
        assert!(stderr.contains("Usage:"));
    }
}

#[test]
fn doctor_json_is_one_environment_report_even_when_tools_are_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let output = code(
        Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .args(["doctor", "--json"])
            .current_dir(root.path())
            .env("PATH", "")
            .env("YETLET_CONFIG_ROOT", "missing")
            .output()
            .unwrap(),
        1,
    );
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));
    assert_eq!(output.stdout.iter().filter(|&&v| v == b'\n').count(), 1);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["scope"], "environment");
    assert_eq!(value["completed"], true);
    assert_eq!(value["ok"], false);
    let diagnostics = value["diagnostics"].as_array().unwrap();
    for code in [
        "DOCTOR_RUSTC_VERSION",
        "DOCTOR_CARGO_VERSION",
        "DOCTOR_NEW_PLATFORM",
        "DOCTOR_FRAMEWORK",
        "DOCTOR_APP_READINESS",
        "DOCTOR_ASSETS",
        "DOCTOR_BUILD_DEV",
        "DOCTOR_CONSOLE",
        "DOCTOR_GENERATOR",
    ] {
        assert!(diagnostics.iter().any(|d| d["code"] == code), "{code}");
    }
    for d in diagnostics {
        for field in ["code", "origin", "capability", "status", "message"] {
            assert!(d[field].is_string());
        }
        if ![
            "DOCTOR_RUSTC_VERSION",
            "DOCTOR_CARGO_VERSION",
            "DOCTOR_NEW_PLATFORM",
        ]
        .iter()
        .any(|code| d["code"] == *code)
        {
            assert_eq!(d["status"], "not-checked");
        }
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}
