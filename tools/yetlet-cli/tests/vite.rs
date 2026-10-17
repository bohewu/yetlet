#![cfg(any(target_os = "linux", windows))]
use std::{collections::BTreeMap, ffi::OsString, fs, path::Path, process::Command};
#[cfg(target_os = "linux")]
use yetlet_cli::scaffold;
use yetlet_cli::{Cli, parse};

fn framework() -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    #[cfg(windows)]
    {
        root.parent().unwrap().parent().unwrap().to_path_buf()
    }
    #[cfg(target_os = "linux")]
    {
        root.join("../..").canonicalize().unwrap()
    }
}

fn private_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

#[test]
fn vite_is_exact_unique_new_only_and_preserves_legacy_parse_results() {
    for cloud in [false, true] {
        for config in [false, true] {
            for tailwind in [false, true] {
                let mut args = ["new", "web", "--framework-path", "framework"]
                    .map(OsString::from)
                    .to_vec();
                if cloud {
                    args.push("--cloud".into());
                }
                if config {
                    args.push("--with-config".into());
                }
                if tailwind {
                    args.push("--with-tailwind".into());
                }
                assert_eq!(
                    parse(&args),
                    Ok(Cli::New {
                        name: "web".into(),
                        framework: "framework".into(),
                        dir: ".".into(),
                        cloud,
                        with_config: config,
                        with_tailwind: tailwind
                    })
                );
                args.extend(["--assets".into(), "vite".into()]);
                assert_eq!(
                    parse(&args),
                    Ok(Cli::NewVite {
                        name: "web".into(),
                        framework: "framework".into(),
                        dir: ".".into(),
                        cloud,
                        with_config: config,
                        with_tailwind: tailwind
                    })
                );
                args.extend(["--assets".into(), "vite".into()]);
                assert!(parse(&args).is_err());
            }
        }
    }
    for args in [
        vec!["--assets", "vite"],
        vec![
            "new",
            "--assets",
            "vite",
            "web",
            "--framework-path",
            "framework",
        ],
        vec!["new", "web", "--framework-path", "framework", "--assets"],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets",
            "",
        ],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets",
            "Vite",
        ],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets",
            "webpack",
        ],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets",
            "vite",
            "extra",
        ],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets=vite",
        ],
        vec!["new", "web", "--framework-path", "--assets", "vite"],
        vec![
            "new",
            "web",
            "--framework-path",
            "framework",
            "--assets",
            "--cloud",
        ],
        vec!["doctor", "--assets", "vite"],
        vec!["dev", "--manifest-path", "Cargo.toml", "--assets", "vite"],
        vec!["build", "--manifest-path", "Cargo.toml", "--assets", "vite"],
        vec!["generate", "docker", "--assets", "vite"],
        vec!["generate", "compose", "--assets", "vite"],
        vec!["generate", "k8s", "--assets", "vite"],
    ] {
        assert!(
            parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_err(),
            "{args:?}"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let mut args = ["new", "web", "--framework-path", "framework", "--assets"]
            .map(OsString::from)
            .to_vec();
        args.push(OsString::from_vec(vec![255]));
        assert!(parse(&args).is_err());
    }
}

fn inventory(app: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    for item in fs::read_dir(app).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() {
            assert_eq!(path.file_name().unwrap(), "src");
            for source in fs::read_dir(path).unwrap() {
                let source = source.unwrap().path();
                assert!(source.is_file());
                result.insert(
                    format!("src/{}", source.file_name().unwrap().to_str().unwrap()),
                    fs::read(source).unwrap(),
                );
            }
        } else {
            result.insert(
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                fs::read(path).unwrap(),
            );
        }
    }
    result
}

#[test]
fn eight_vite_profiles_have_exact_finite_sources_locks_and_no_tool_or_overwrite() {
    let root = private_root();
    for cloud in [false, true] {
        for config in [false, true] {
            for tailwind in [false, true] {
                let name = format!("web-{cloud}-{config}-{tailwind}");
                let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
                command
                    .args(["new", &name, "--assets", "vite", "--framework-path"])
                    .arg(framework())
                    .arg("--dir")
                    .arg(root.path())
                    .env("PATH", root.path().join("no-tools"))
                    .env("CARGO_HOME", root.path().join("no-cache"));
                if cloud {
                    command.arg("--cloud");
                }
                if config {
                    command.arg("--with-config");
                }
                if tailwind {
                    command.arg("--with-tailwind");
                }
                let output = command.output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let app = root.path().join(&name);
                let before = inventory(&app);
                let mut expected = vec![
                    ".gitignore",
                    "Cargo.toml",
                    "README.md",
                    "src/main.rs",
                    "src/vite.rs",
                    "package.json",
                    "package-lock.json",
                    "tsconfig.json",
                    "vite.config.mjs",
                    "app.ts",
                    "app.css",
                    "shared.ts",
                    "shared.css",
                    "companion.ts",
                    "lazy.ts",
                    "lazy.css",
                    "mark.svg",
                ];
                if config {
                    expected.extend([
                        "src/config.rs",
                        ".env.example",
                        "base.toml",
                        "development.toml",
                        "staging.toml",
                        "production.toml",
                    ]);
                }
                expected.sort();
                assert_eq!(
                    before.keys().map(String::as_str).collect::<Vec<_>>(),
                    expected
                );
                assert!(!root.path().join("no-cache").exists());
                assert!(!command.output().unwrap().status.success());
                assert_eq!(inventory(&app), before);
                let package: serde_json::Value =
                    serde_json::from_slice(&before["package.json"]).unwrap();
                let lock: serde_json::Value =
                    serde_json::from_slice(&before["package-lock.json"]).unwrap();
                assert_eq!(
                    package["devDependencies"],
                    lock["packages"][""]["devDependencies"]
                );
                assert_eq!(package["devDependencies"]["vite"], "8.3.4");
                assert_eq!(package["devDependencies"]["typescript"], "7.0.2");
                for key in lock["packages"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .filter(|s| !s.is_empty())
                {
                    assert!(key.starts_with("node_modules/"));
                    assert_eq!(
                        key.contains("tailwind"),
                        tailwind && key.contains("tailwind")
                    );
                    let entry = &lock["packages"][key];
                    assert!(
                        entry["resolved"]
                            .as_str()
                            .unwrap()
                            .starts_with("https://registry.npmjs.org/")
                    );
                    assert!(entry["integrity"].as_str().unwrap().starts_with("sha512-"));
                }
                assert_eq!(
                    package["devDependencies"]["@tailwindcss/vite"].as_str(),
                    tailwind.then_some("4.3.3")
                );
                let vite = std::str::from_utf8(&before["vite.config.mjs"]).unwrap();
                assert_eq!(vite.contains("tailwindcss()"), tailwind);
                for setting in [
                    "envDir: false",
                    "publicDir: false",
                    "assetsInlineLimit: 0",
                    "sourcemap: false",
                    "rolldownOptions:",
                ] {
                    assert!(vite.contains(setting));
                }
                let main = std::str::from_utf8(&before["src/main.rs"]).unwrap();
                assert!(
                    main.find("Snapshot::load_from_env").unwrap()
                        < main.find("TcpListener::bind").unwrap()
                );
                assert!(!main.contains("@MESSAGE@"));
                assert!(!before.contains_key("src/tailwind.rs"));
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn vite_retains_protected_parent_symlink_and_destination_refusals() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = private_root();
    let parent = root.path().join("real");
    fs::create_dir(&parent).unwrap();
    symlink(&parent, root.path().join("link")).unwrap();
    assert!(
        scaffold::create_vite(
            "web",
            &framework(),
            &root.path().join("link"),
            false,
            false,
            false
        )
        .is_err()
    );
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(scaffold::create_vite("web", &framework(), &parent, true, true, true).is_err());
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(parent.join("keep"), "user resource").unwrap();
    symlink("keep", parent.join("web")).unwrap();
    assert!(scaffold::create_vite("web", &framework(), &parent, false, false, true).is_err());
    assert_eq!(fs::read(parent.join("keep")).unwrap(), b"user resource");
    assert_eq!(
        fs::read_link(parent.join("web")).unwrap(),
        Path::new("keep")
    );
    fs::write(parent.join("file"), "keep").unwrap();
    assert!(scaffold::create_vite("file", &framework(), &parent, false, false, false).is_err());
    assert_eq!(fs::read(parent.join("file")).unwrap(), b"keep");
}
