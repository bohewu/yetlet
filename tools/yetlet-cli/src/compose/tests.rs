use super::*;
use crate::cloud_fixture::Fixture;
use crate::{Cli, parse};
use std::{ffi::OsString, fs};

const IMAGE: &str = "localhost:5000/team/app:v1";

#[test]
fn literal_compose_arguments_are_required_unique_and_command_specific() {
    let args = |parts: &[&str]| parts.iter().map(OsString::from).collect::<Vec<_>>();
    let base = [
        "generate",
        "compose",
        "--manifest-path",
        "應 用/Cargo.toml",
        "--output",
        "設定 ;$(literal)",
        "--image",
        IMAGE,
    ];
    assert_eq!(
        parse(&args(&base)),
        Ok(Cli::GenerateCompose {
            manifest: "應 用/Cargo.toml".into(),
            output: "設定 ;$(literal)".into(),
            image: IMAGE.into()
        })
    );
    for tail in [
        vec!["--image", IMAGE],
        vec!["--output", "x"],
        vec!["--manifest-path", "x"],
        vec!["--cloud"],
        vec!["--build"],
        vec!["--"],
        vec!["--help"],
        vec!["positional"],
    ] {
        let mut invalid = base.to_vec();
        invalid.extend(tail);
        assert!(parse(&args(&invalid)).is_err(), "{invalid:?}");
    }
    for position in [4, 6] {
        let mut missing = base.to_vec();
        missing.drain(position..position + 2);
        assert!(parse(&args(&missing)).is_err());
        for blank in ["", " ", "\t", "--output"] {
            let mut invalid = base;
            invalid[position + 1] = blank;
            assert!(parse(&args(&invalid)).is_err());
        }
    }
    for invalid in [
        vec!["generate", "compose", "--image"],
        vec!["generate", "compose", "--image=ghcr.io/app:v1"],
        vec![
            "generate",
            "docker",
            "--manifest-path",
            "Cargo.toml",
            "--output",
            "out",
            "--image",
            IMAGE,
        ],
    ] {
        assert!(parse(&args(&invalid)).is_err());
    }
    for bad in [
        "app",
        "app:v1",
        "ghcr.io/app:latest",
        "ghcr.io/app:v1\nprivileged: true",
        "ghcr.io/app:${SECRET}",
    ] {
        let mut invalid = base;
        invalid[7] = bad;
        assert!(parse(&args(&invalid)).is_err());
    }
    let mut reordered = args(&[
        "generate",
        "compose",
        "--image",
        IMAGE,
        "--output",
        "out",
        "--manifest-path",
        "app/Cargo.toml",
    ]);
    assert!(parse(&reordered).is_ok());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        reordered[3] = OsString::from_vec(vec![0xff]);
        assert!(parse(&reordered).is_err());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        reordered[3] = OsString::from_wide(&[0xd800]);
        assert!(parse(&reordered).is_err());
    }
}

#[test]
fn two_exact_deterministic_files_without_sources_or_secrets() {
    let f = Fixture::new();
    let secret = concat!("SYNTHETIC_COMPOSE_", "SECRET");
    fs::write(f.app.join(".env"), secret).unwrap();
    fs::create_dir(f.app.join(".ssh")).unwrap();
    fs::write(f.app.join(".ssh/private"), secret).unwrap();
    let before =
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap());
    let expected = include_str!("../../templates/compose/compose.yaml").replace("@IMAGE@", IMAGE);
    for directory in ["first 設定", "second"] {
        let output = f.parent.join(directory);
        generate(&f.manifest(), &output, IMAGE).unwrap();
        let mut names = fs::read_dir(&output)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(names, ["README.md", "compose.yaml"]);
        assert_eq!(
            fs::read(output.join("compose.yaml")).unwrap(),
            expected.as_bytes()
        );
        assert_eq!(
            fs::read(output.join("README.md")).unwrap(),
            include_bytes!("../../templates/compose/README.md")
        );
        for file in names {
            let text = fs::read_to_string(output.join(file)).unwrap();
            for excluded in [
                secret,
                f.app.to_str().unwrap(),
                f.framework.to_str().unwrap(),
                f.root.path().to_str().unwrap(),
            ] {
                assert!(!text.contains(excluded));
            }
            assert!(!text.contains('\r'));
            assert!(text.ends_with('\n'));
        }
    }
    assert_eq!(
        ["Cargo.toml", "Cargo.lock", "src/main.rs"].map(|p| fs::read(f.app.join(p)).unwrap()),
        before
    );
    assert!(expected.contains("image: 'localhost:5000/team/app:v1'"));
    assert!(expected.contains("'127.0.0.1:8080:8080'"));
    assert!(expected.contains("stop_grace_period: '30s'"));
    for forbidden in [
        "build:",
        "healthcheck:",
        "volumes:",
        "privileged:",
        "network_mode:",
        "env_file:",
        "${",
        "command:",
    ] {
        assert!(!expected.contains(forbidden));
    }
    let digest = format!("example.com/app@sha256:{}", "ab".repeat(32));
    generate(&f.manifest(), &f.parent.join("digest"), &digest).unwrap();
    assert!(
        fs::read_to_string(f.parent.join("digest/compose.yaml"))
            .unwrap()
            .contains(&format!("image: '{digest}'"))
    );
}

#[test]
fn missing_stale_or_inconsistent_locks_are_refused_without_output() {
    for mutation in 0..13 {
        let f = Fixture::new();
        let path = f.app.join("Cargo.lock");
        let original = fs::read_to_string(&path).unwrap();
        let updated = match mutation {
            0 => {
                fs::remove_file(&path).unwrap();
                original
            }
            1 => original.replace("name='composeapp'", "name='oldapp'"),
            2 => original.replacen("version='0.1.0'", "version='0.0.9'", 1),
            3 => original.replace("'yetlet-core','yetlet-server-axum'", "'yetlet-core'"),
            4 => original.replace("name='yetlet-html'", "name='missing-html'"),
            5 => original.replace("version='0.8.9'", "version='0.7.9'"),
            6 => original.replace("dependencies=['signal-hook-registry']", "dependencies=[]"),
            7 => original.replace("'signal-hook-registry']", "'missing']"),
            8 => original.replace("version=4", "version=3"),
            9 => original.replace("version='1.0.228'", "version='0.9.0'"),
            10 => original.replace("version='0.5.3'", "version='0.4.0'"),
            11 => original.replace("version='1.53.2'", "version='1.invalid'"),
            _ => format!("{original}\n[[package]]\nname='yetlet-core'\nversion='0.1.0'\n"),
        };
        if mutation != 0 {
            fs::write(path, updated).unwrap();
        }
        let output = f.parent.join("out");
        assert!(
            generate(&f.manifest(), &output, IMAGE).is_err(),
            "mutation {mutation}"
        );
        assert!(!output.exists());
    }
}

#[test]
fn unchanged_cloud_source_manifest_and_framework_closure_are_required() {
    for mutation in 0..6 {
        let f = Fixture::new();
        match mutation {
            0 => fs::write(f.app.join("src/main.rs"), "fn main(){}\n").unwrap(),
            1 => fs::write(f.app.join("src/lib.rs"), "").unwrap(),
            2 => fs::write(f.app.join("build.rs"), "fn main(){}\n").unwrap(),
            3 => {
                let text = fs::read_to_string(f.manifest()).unwrap();
                fs::write(f.manifest(), text.replace("native-ssr-v1", "invalid")).unwrap();
            }
            4 => fs::write(
                f.framework.join("crates/yetlet-html/Cargo.toml"),
                "[package]\nname='other'\n",
            )
            .unwrap(),
            _ => fs::write(f.framework.join("crates/yetlet-core/src/extra.rs"), "").unwrap(),
        }
        let output = f.parent.join("out");
        assert!(generate(&f.manifest(), &output, IMAGE).is_err());
        assert!(!output.exists());
    }
}

#[test]
fn fresh_destination_existing_parent_and_literal_safe_output_paths_only() {
    let f = Fixture::new();
    for output in [
        f.parent.join("child/../escape"),
        f.parent.join("missing/out"),
        f.parent.join("..\\escape"),
        f.app.clone(),
    ] {
        assert!(generate(&f.manifest(), &output, IMAGE).is_err());
    }
    let output = f.parent.join("out");
    generate(&f.manifest(), &output, IMAGE).unwrap();
    let before = fs::read(output.join("compose.yaml")).unwrap();
    assert!(generate(&f.manifest(), &output, "ghcr.io/other/app:v2").is_err());
    assert_eq!(fs::read(output.join("compose.yaml")).unwrap(), before);
    assert!(generate(&f.manifest(), &f.parent.join("bad-image"), "bad").is_err());
    assert!(!f.parent.join("bad-image").exists());
}

#[test]
fn changed_source_lock_framework_and_late_destination_fail_closed() {
    for mutation in 0..4 {
        let f = Fixture::new();
        let output = f.parent.join("out");
        assert!(
            generate_inner(&f.manifest(), &output, IMAGE, || {
                match mutation {
                    0 => fs::write(f.app.join("src/main.rs"), "fn main(){}\n").unwrap(),
                    1 => {
                        let path = f.app.join("Cargo.lock");
                        let text = fs::read_to_string(&path).unwrap();
                        if let Err(error) = fs::write(&path, format!("{text}\n# changed\n")) {
                            // Windows may pin the lock with a read-only HANDLE;
                            // sharing violation is an *extra* safety barrier,
                            // not an acceptable permission bypass. Mutate the
                            // already inspected main to keep the fail-closed
                            // publication assertion reachable on both OSes.
                            assert!(
                                cfg!(windows) && error.raw_os_error() == Some(32),
                                "unexpected lock mutation failure: {error}"
                            );
                            assert_eq!(fs::read_to_string(&path).unwrap(), text);
                            fs::write(f.app.join("src/main.rs"), "fn main(){}\n").unwrap();
                        }
                    }
                    2 => {
                        let path = f.framework.join("crates/yetlet-core/src/lib.rs");
                        if let Err(error) = fs::write(&path, "// changed\n") {
                            assert!(
                                cfg!(windows) && error.raw_os_error() == Some(32),
                                "unexpected protected framework mutation failure: {error}"
                            );
                            fs::write(f.app.join("src/main.rs"), "fn main(){}\n").unwrap();
                        }
                    }
                    _ => {
                        // The late foreign FILE must win a no-replace race.
                        // Windows native parent handles can reject an attempted
                        // CreateDirectory with ERROR_SHARING_VIOLATION; a file
                        // interloper exercises the same publication guarantee
                        // on both native platforms without weakening it.
                        fs::write(&output, "preserved").unwrap();
                    }
                }
            })
            .is_err()
        );
        if mutation == 3 {
            assert_eq!(fs::read(&output).unwrap(), b"preserved");
        } else {
            assert!(!output.exists());
        }
        assert_eq!(
            fs::read_dir(&f.parent).unwrap().count(),
            4 + usize::from(mutation == 3)
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn symlinks_unsafe_parent_and_app_rename_race_are_refused() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    for mutation in 0..5 {
        let f = Fixture::new();
        let output = f.parent.join("out");
        match mutation {
            0 => {
                let lock = f.app.join("Cargo.lock");
                fs::rename(&lock, f.app.join("saved-lock")).unwrap();
                symlink("saved-lock", lock).unwrap();
            }
            1 => {
                fs::rename(f.app.join("src"), f.app.join("saved-src")).unwrap();
                symlink("saved-src", f.app.join("src")).unwrap();
            }
            2 => symlink(&f.app, &output).unwrap(),
            3 => fs::set_permissions(&f.parent, fs::Permissions::from_mode(0o777)).unwrap(),
            _ => {
                fs::rename(&f.parent, f.root.path().join("saved-parent")).unwrap();
                symlink("saved-parent", &f.parent).unwrap();
            }
        }
        assert!(generate(&f.manifest(), &output, IMAGE).is_err());
    }
    let f = Fixture::new();
    let output = f.parent.join("out");
    assert!(
        generate_inner(&f.manifest(), &output, IMAGE, || {
            fs::rename(&f.app, f.root.path().join("moved")).unwrap();
            fs::create_dir(&f.app).unwrap();
        })
        .is_err()
    );
    assert!(!output.exists());
}
