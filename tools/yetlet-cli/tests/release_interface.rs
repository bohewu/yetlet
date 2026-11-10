use std::process::Command;

#[test]
fn real_binary_version_flags_match_the_cli_manifest() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .arg(flag)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            output.stdout,
            format!("yetlet {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn real_binary_help_describes_the_published_preview() {
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let help = String::from_utf8(output.stdout).unwrap();
    assert_eq!(help.lines().next(), Some("Yetlet CLI preview"));
    assert!(help.contains("Usage:"));
    assert!(output.stderr.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn real_binary_offline_preflight_publishes_no_project() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args([
            "new",
            "consumerapp",
            "--offline",
            "--sdk-version",
            "0.9.0-alpha.7",
            "--parent",
        ])
        .arg(root.path())
        .env("CARGO_HOME", root.path().join("empty-cargo-home"))
        .current_dir(root.path())
        .output()
        .unwrap();
    let doctor = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&doctor.stdout).unwrap();
    if report["ok"] == true {
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("SDK_OFFLINE_CACHE_MISS"));
    } else {
        // Existing doctor policy refuses unsafe/missing local tool installs.
        // Metadata hit/miss behavior is independently tested with an injected
        // tool preflight in sdk_version tests; never weaken executable policy.
        assert_eq!(output.status.code(), Some(125), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("CLI_INFRASTRUCTURE"));
    }
    assert!(!root.path().join("consumerapp").exists());
    assert!(!root.path().join("empty-cargo-home").exists());
}
