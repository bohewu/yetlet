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
fn real_binary_scaffold_keeps_the_published_sdk_dependency() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "consumerapp", "--parent"])
        .arg(root.path())
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let manifest: toml::Value = std::fs::read_to_string(root.path().join("consumerapp/Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let sdk = &manifest["dependencies"]["yetlet"];
    assert_eq!(sdk["version"].as_str(), Some("=0.9.0-alpha.5"));
    assert!(sdk.get("path").is_none());
    assert!(manifest.get("patch").is_none());
    assert!(manifest.get("replace").is_none());
}
