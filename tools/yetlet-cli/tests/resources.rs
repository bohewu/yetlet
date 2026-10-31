//! Package-owned resources and their deliberately selected public SDK closure.
use std::{collections::BTreeSet, path::Path, process::Command};

#[path = "../src/framework_files.rs"]
mod framework_files;

#[test]
fn package_resources_have_a_pinned_sdk_and_closed_relative_names() {
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
    let files = &*framework_files::FILES;
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
fn repository_snapshot_matches_reviewed_sources_when_checkout_is_present() {
    let cli = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = cli.parent().unwrap().parent().unwrap();
    if !root.join(".git").is_dir() || !root.join("crates/yetlet/src/lib.rs").is_file() {
        return; // An installed archive intentionally has no framework checkout.
    }
    let output = Command::new("git")
        .args(["ls-files", "-z", "--", "crates/yetlet"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let tracked: BTreeSet<_> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .split('\0')
        .filter(|p| !p.is_empty() && *p != "crates/yetlet/Cargo.lock")
        .map(str::to_owned)
        .collect();
    assert_eq!(tracked, framework_files::FILES.keys().cloned().collect());
    for (name, expected) in framework_files::FILES.iter() {
        assert_eq!(
            std::fs::read_to_string(root.join(name))
                .unwrap()
                .replace("\r\n", "\n"),
            *expected,
            "{name}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("Cargo.toml"))
            .unwrap()
            .replace("\r\n", "\n"),
        include_str!("../resources/workspace.toml")
    );
    assert_eq!(
        std::fs::read(root.join("Cargo.lock")).unwrap(),
        include_bytes!("../resources/framework.lock")
    );
    for name in ["wasm-startup.mjs", "wasm-startup.css"] {
        assert_eq!(
            std::fs::read(root.join("assets").join(name)).unwrap(),
            std::fs::read(cli.join("templates/browser").join(name)).unwrap()
        );
    }
}
