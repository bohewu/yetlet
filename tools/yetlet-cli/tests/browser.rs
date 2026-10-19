#![cfg(any(target_os = "linux", windows))]
use std::{fs, process::Command};

fn new(
    root: &std::path::Path,
    name: &str,
    template: &str,
    options: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", name, "--parent"])
        .arg(root)
        .args(["--template", template])
        .args(options)
        .env("APP_CLI__TARGET", "native")
        .output()
        .unwrap()
}

#[test]
fn browser_creation_is_explicit_finite_and_preserves_native_default() {
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
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let app = temp.path().join(name);
        let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("[lib]") && manifest.contains("wasm32"));
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
            "frontend/package-lock.json",
        ] {
            assert!(app.join(file).is_file());
        }
        assert!(!new(temp.path(), name, "browser", &options).status.success());
    }
    assert!(
        new(temp.path(), "native-app", "native", &[])
            .status
            .success()
    );
    assert!(temp.path().join("native-app/src/main.rs").is_file());
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["new", "default-app", "--parent"])
        .arg(temp.path())
        .env("APP_CLI__TARGET", "browser")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(temp.path().join("default-app/src/main.rs").is_file());
    for (name, template, options) in [
        ("bad-template", "unknown", vec![]),
        ("native-service", "browser", vec!["--service"]),
    ] {
        let output = new(temp.path(), name, template, &options);
        assert_eq!(output.status.code(), Some(2));
        assert!(!temp.path().join(name).exists());
    }
}
