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
fn frozen_cli_build_dev_and_lock_cache_failures_use_native_cargo() {
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
    code(cli("dev", &manifest).output().unwrap(), 1);
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
    code(cli("dev", &manifest).output().unwrap(), 23);
    assert_eq!(fs::read(&lock).unwrap(), initial_lock);
    fs::write(
        &manifest,
        format!("{manifest_text}\n[dependencies]\nserde_json='=1.0.149'\n"),
    )
    .unwrap();
    for command in ["build", "dev"] {
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
    for command in ["build", "dev"] {
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
