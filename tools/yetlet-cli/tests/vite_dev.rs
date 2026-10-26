#![cfg(any(target_os = "linux", windows))]
use std::{collections::BTreeMap, ffi::OsString, fs, path::Path};
use yetlet_cli::{Cli, parse, scaffold};

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

fn inventory(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            assert_eq!(path.file_name().unwrap(), "src");
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                files.insert(
                    format!("src/{}", path.file_name().unwrap().to_str().unwrap()),
                    fs::read(path).unwrap(),
                );
            }
        } else {
            files.insert(
                path.file_name().unwrap().to_str().unwrap().into(),
                fs::read(path).unwrap(),
            );
        }
    }
    files
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
fn removed_development_switches_refuse_before_creation() {
    let argv = |args: &[&str]| args.iter().map(OsString::from).collect::<Vec<_>>();
    for args in [
        vec!["new", "web", "--with-vite-dev"],
        vec!["new", "web", "--assets", "vite"],
        vec!["new", "web", "--vite", "dev"],
        vec!["run", "--with-vite-dev"],
    ] {
        assert!(parse(&argv(&args)).is_err());
    }
    assert!(matches!(
        parse(&argv(&["new", "web", "--vite"])),
        Ok(Cli::New {
            vite: true,
            runtime_settings: true,
            ..
        })
    ));
}

#[test]
fn eight_dev_profiles_only_add_two_root_files_and_script_readme_without_tools_or_overwrite() {
    let root = private_root();
    let legacy = root.path().join("legacy");
    let dev = root.path().join("dev");
    fs::create_dir(&legacy).unwrap();
    fs::create_dir(&dev).unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&legacy, &dev] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
    for service in [false, true] {
        for config in [false, true] {
            for tailwind in [false, true] {
                let name = format!("web-{service}-{config}-{tailwind}");
                scaffold::create_vite(&name, &framework(), &legacy, service, config, tailwind)
                    .unwrap();
                scaffold::create_vite_dev(&name, &framework(), &dev, service, config, tailwind)
                    .unwrap();
                let original = inventory(&legacy.join(&name));
                let generated = inventory(&dev.join(&name));
                assert_eq!(generated.len(), original.len() + 2);
                for (path, bytes) in &original {
                    if path != "README.md" && path != "package.json" {
                        assert_eq!(&generated[path], bytes, "{name}: {path}");
                    }
                }
                let mut package: serde_json::Value =
                    serde_json::from_slice(&generated["package.json"]).unwrap();
                assert_eq!(
                    package["scripts"]
                        .as_object_mut()
                        .unwrap()
                        .remove("dev")
                        .unwrap(),
                    "vite --config vite.dev.config.mjs"
                );
                assert_eq!(
                    package,
                    serde_json::from_slice::<serde_json::Value>(&original["package.json"]).unwrap()
                );
                assert!(generated.contains_key("vite.dev.config.mjs"));
                assert!(generated.contains_key("vite.dev.bridge.mjs"));
                assert!(
                    scaffold::create_vite_dev(&name, &framework(), &dev, service, config, tailwind)
                        .is_err()
                );
                assert_eq!(generated, inventory(&dev.join(&name)));
                assert!(!root.path().join("no-cache").exists());
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn dev_generation_retains_protected_parent_and_symlink_refusals() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = private_root();
    let parent = root.path().join("parent");
    fs::create_dir(&parent).unwrap();
    symlink(&parent, root.path().join("link")).unwrap();
    assert!(
        scaffold::create_vite_dev(
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
    assert!(scaffold::create_vite_dev("web", &framework(), &parent, true, true, true).is_err());
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(parent.join("keep"), "user resource").unwrap();
    symlink("keep", parent.join("web")).unwrap();
    assert!(scaffold::create_vite_dev("web", &framework(), &parent, false, false, true).is_err());
    assert_eq!(fs::read(parent.join("keep")).unwrap(), b"user resource");
    assert_eq!(
        fs::read_link(parent.join("web")).unwrap(),
        Path::new("keep")
    );
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 2);
}
