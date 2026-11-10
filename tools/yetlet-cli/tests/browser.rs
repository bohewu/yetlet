#![cfg(any(target_os = "linux", windows))]
#[path = "fixtures/source_scaffold.rs"]
mod source_scaffold;
use std::fs;

fn new(root: &std::path::Path, name: &str, template: &str, options: &[&str]) -> Result<(), String> {
    source_scaffold::create(root, name, template, options)
}

#[test]
fn browser_creation_is_explicit_finite_and_preserves_native_default() {
    let cli: toml::Value = include_str!("../Cargo.toml").parse().unwrap();
    let temp = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    for (name, options) in [
        ("plain-browser", vec![]),
        (
            "tailwind-browser",
            vec!["--vite", "--settings", "--tailwind"],
        ),
    ] {
        let output = new(temp.path(), name, "browser", &options);
        assert!(output.is_ok(), "{}", format!("{output:?}"));
        let app = temp.path().join(name);
        let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("[lib]") && manifest.contains("wasm32"));
        let parsed: toml::Value = manifest.parse().unwrap();
        let dependencies = &parsed["target"]["cfg(target_arch = \"wasm32\")"]["dependencies"];
        for name in ["yetlet", "yetlet-browser"] {
            assert_eq!(
                dependencies[name]["version"],
                cli["dependencies"]["yetlet"]["version"]
            );
            assert!(dependencies[name].get("path").is_none());
        }
        for native in ["tokio", "axum", "yetlet-config", "yetlet-secrets"] {
            assert!(!manifest.contains(native));
        }
        assert!(!app.join("src/main.rs").exists());
        assert!(!app.join("Cargo.lock").exists());
        assert!(!app.join("frontend/node_modules").exists());
        assert!(
            fs::read_to_string(app.join("base.toml"))
                .unwrap()
                .contains("target = \"browser\"")
        );
        for file in [
            "src/lib.rs",
            "src/model.rs",
            "frontend/index.html",
            "frontend/build-wasm.mjs",
            "frontend/serve.mjs",
            "frontend/package-lock.json",
        ] {
            assert!(app.join(file).is_file());
        }
        assert!(new(temp.path(), name, "browser", &options).is_err());
    }
    assert!(new(temp.path(), "native-app", "native", &[]).is_ok());
    assert!(temp.path().join("native-app/src/main.rs").is_file());
    source_scaffold::create(temp.path(), "default-app", "native", &[]).unwrap();
    assert!(temp.path().join("default-app/src/main.rs").is_file());
    for (name, template, options) in [
        ("bad-template", "unknown", vec![]),
        ("native-service", "browser", vec!["--service"]),
    ] {
        let output = new(temp.path(), name, template, &options);
        assert!(output.is_err());
        assert!(!temp.path().join(name).exists());
    }
}
