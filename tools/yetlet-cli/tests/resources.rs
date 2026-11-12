//! Package-owned resources and their deliberately selected public SDK closure.
use std::{path::Path, process::Command};

fn framework_files() -> std::collections::BTreeMap<String, String> {
    serde_json::from_str(include_str!("../resources/framework.json")).unwrap()
}

#[test]
fn package_resources_have_a_pinned_sdk_and_closed_relative_names() {
    let policy: serde_json::Value =
        serde_json::from_str(include_str!("../resources/sdk-compatibility.json")).unwrap();
    assert_eq!(policy.as_object().unwrap().len(), 3);
    assert_eq!(policy["revision"], "sdk-latest-v2");
    assert_eq!(policy["selection"], "latest-semver");
    let capabilities = policy["scaffold_capabilities"].as_object().unwrap();
    assert_eq!(capabilities.len(), 1);
    let rows = capabilities["m26_startup"].as_array().unwrap();
    assert!(rows.iter().all(|row| row["sdk_version"] != "0.9.0-alpha.7"));
    let workspace: toml::Value = include_str!("../resources/workspace.toml").parse().unwrap();
    let cli: toml::Value = include_str!("../Cargo.toml").parse().unwrap();
    let version = workspace["workspace"]["package"]["version"]
        .as_str()
        .unwrap();
    assert_eq!(
        cli["dependencies"]["yetlet"]["version"].as_str(),
        Some(format!("={version}").as_str())
    );
    assert!(cli["dependencies"]["yetlet"].get("path").is_none());
    let files = framework_files();
    assert!(files.contains_key("crates/yetlet/src/lib.rs"));
    assert!(files.contains_key("crates/yetlet/LICENSE-MIT"));
    for name in files.keys() {
        assert!(name.starts_with("crates/yetlet/") && !name.contains('\\'));
        assert!(
            Path::new(name)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
        );
    }
}

#[test]
fn snapshot_digests_and_recorded_source_are_independent_of_current_sdk_checkout() {
    use sha2::{Digest, Sha256};
    let cli = Path::new(env!("CARGO_MANIFEST_DIR"));
    let identity: serde_json::Value =
        serde_json::from_str(include_str!("../resources/sdk-identity.json")).unwrap();
    let manifest: toml::Value = include_str!("../Cargo.toml").parse().unwrap();
    assert_eq!(
        manifest["dependencies"]["yetlet"]["version"].as_str(),
        Some(format!("={}", identity["build_sdk"].as_str().unwrap()).as_str())
    );
    for (path, sha) in identity["files"].as_object().unwrap() {
        let bytes = std::fs::read(cli.join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            sha.as_str().unwrap(),
            "{path}"
        );
    }
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../resources/sdk-compatibility.json"))
        ),
        identity["policy_sha256"].as_str().unwrap()
    );
    let source = identity["snapshot_source"].as_str().unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let root = cli.parent().unwrap().parent().unwrap();
    if !root.join(".git").is_dir() {
        return;
    }
    // An installed crate has no Git source. A source checkout verifies the exact
    // recorded SDK commit, even when today's workspace is on a different SDK.
    for (path, expected) in framework_files().iter() {
        let output = Command::new("git")
            .args(["show", &format!("{source}:{path}")])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "recorded SDK source must be present in source validation"
        );
        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .replace("\r\n", "\n"),
            *expected,
            "{path}"
        );
    }
    for (resource, path) in [
        ("resources/workspace.toml", "Cargo.toml"),
        ("resources/framework.lock", "Cargo.lock"),
        (
            "templates/browser/wasm-startup.mjs",
            "assets/wasm-startup.mjs",
        ),
        (
            "templates/browser/wasm-startup.css",
            "assets/wasm-startup.css",
        ),
    ] {
        let output = Command::new("git")
            .args(["show", &format!("{source}:{path}")])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            std::fs::read(cli.join(resource)).unwrap(),
            output.stdout,
            "{resource}"
        );
    }
}
