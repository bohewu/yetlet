#![cfg(any(target_os = "linux", windows))]
use std::{fs, path::Path, process::Command};

#[cfg(windows)]
fn fixture_framework(root: &Path) -> std::path::PathBuf {
    // A self-contained framework identity fixture: this test verifies safe
    // scaffolding, not a production image build or a framework compilation.
    let source = root.join("framework");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[workspace]\n[workspace.package]\nrust-version='1.99'\n",
    )
    .unwrap();
    for name in ["yetlet-core", "yetlet-server-axum"] {
        let dir = source.join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname='{name}'\n"),
        )
        .unwrap();
        fs::write(dir.join("src/lib.rs"), "// fixture only\n").unwrap();
    }
    source
}

// Historical local-source qualification fixtures use the retained internal API.
fn scaffold(parent: &Path, framework: &Path, name: &str, cloud: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).unwrap();
    }
    if cloud {
        yetlet_cli::scaffold::create_cloud(name, framework, parent)
    } else {
        yetlet_cli::scaffold::create(name, framework, parent)
    }
}

#[test]
fn both_profiles_use_safe_no_replace_without_subprocess_lock_or_extra_files() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("apps");
    fs::create_dir(&parent).unwrap();
    #[cfg(windows)]
    let framework = fixture_framework(root.path());
    #[cfg(target_os = "linux")]
    let framework = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for (name, cloud) in [("ordinary", false), ("cloudapp", true)] {
        let output = scaffold(&parent, &framework, name, cloud);
        assert!(output.is_ok(), "{output:?}");
        let app = parent.join(name);
        let main = fs::read(app.join("src/main.rs")).unwrap();
        let readme = fs::read_to_string(app.join("README.md")).unwrap();
        let (expected_main, expected_readme) = if cloud {
            (
                include_bytes!("../templates/cloud/main.rs").as_slice(),
                include_str!("../templates/cloud/README.md"),
            )
        } else {
            (
                include_bytes!("../templates/main.rs").as_slice(),
                include_str!("../templates/README.md"),
            )
        };
        assert_eq!(main, expected_main);
        assert_eq!(readme, expected_readme.replace("@NAME@", name));
        let mut entries: Vec<_> = fs::read_dir(&app)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        entries.sort();
        assert_eq!(
            entries,
            [".gitignore", "Cargo.toml", "README.md", "src"].map(std::ffi::OsString::from)
        );
        assert_eq!(fs::read_dir(app.join("src")).unwrap().count(), 1);
        let manifest = fs::read(app.join("Cargo.toml")).unwrap();
        // Frozen pre-YT-097 bytes, including dependency order and cloud marker.
        let core = toml::Value::String(
            framework
                .join("crates/yetlet-core")
                .to_str()
                .unwrap()
                .into(),
        );
        let server = toml::Value::String(
            framework
                .join("crates/yetlet-server-axum")
                .to_str()
                .unwrap()
                .into(),
        );
        let signal = if cloud { ", \"signal\"" } else { "" };
        let mut expected = format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.99\"\ndescription = \"Native Yetlet SSR application preview\"\nlicense = \"MIT OR Apache-2.0\"\npublish = false\n\n[workspace]\nresolver = \"3\"\n\n[dependencies]\naxum = \"0.8\"\ntokio = {{ version = \"1\", features = [\"macros\", \"net\", \"rt-multi-thread\"{signal}] }}\nyetlet-core = {{ path = {core} }}\nyetlet-server-axum = {{ path = {server} }}\n"
        );
        if cloud {
            expected.push_str("\n[package.metadata.yetlet]\ncloud-profile = \"native-ssr-v1\"\n");
        }
        assert_eq!(manifest, expected.as_bytes());
        assert_eq!(
            fs::read(app.join(".gitignore")).unwrap(),
            b"/target/\n.env\n.env.*\n"
        );

        assert!(scaffold(&parent, &framework, name, !cloud).is_err());
        assert_eq!(fs::read(app.join("Cargo.toml")).unwrap(), manifest);
        assert_eq!(fs::read(app.join("src/main.rs")).unwrap(), main);
    }
    assert!(!parent.join("no-cargo-home").exists());
    assert_eq!(
        fs::read_dir(&parent).unwrap().count(),
        2,
        "no staging residue"
    );
}

#[test]
#[ignore = "explicit cached-dependency generated-app verification; native HTTP requires YETLET_TEST_NATIVE_HTTP=1"]
fn generated_cloud_app_offline_contract() {
    let root = tempfile::tempdir().unwrap();
    let framework = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let output = scaffold(root.path(), &framework, "cloudapp", true);
    assert!(output.is_ok(), "{output:?}");
    let app = root.path().join("cloudapp");
    let cargo = |args: &[&str]| {
        let output = Command::new("cargo")
            .args(args)
            .current_dir(&app)
            .env("CARGO_TARGET_DIR", root.path().join("target"))
            .env("CARGO_NET_OFFLINE", "true")
            .env("RUSTUP_AUTO_INSTALL", "0")
            .output()
            .unwrap();
        eprintln!(
            "cargo {}\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "cargo {args:?}: {}", output.status);
    };
    cargo(&["generate-lockfile", "--offline"]);
    let lock = fs::read(app.join("Cargo.lock")).unwrap();
    // First check the actual unmodified generated source.
    cargo(&["fmt", "--all", "--", "--check"]);
    cargo(&["check", "--offline", "--locked", "--all-targets"]);
    cargo(&[
        "clippy",
        "--offline",
        "--locked",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    let source = fs::read_to_string(app.join("src/main.rs")).unwrap();
    fs::write(
        app.join("src/main.rs"),
        format!("{source}\n{}", include_str!("fixtures/cloud_runtime.rs")),
    )
    .unwrap();
    cargo(&["test", "--offline", "--locked", "--", "--nocapture"]);
    cargo(&[
        "clippy",
        "--offline",
        "--locked",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    if std::env::var_os("YETLET_TEST_NATIVE_HTTP").as_deref() == Some(std::ffi::OsStr::new("1")) {
        cargo(&[
            "test",
            "--offline",
            "--locked",
            "--",
            "--ignored",
            "--nocapture",
        ]);
        #[cfg(target_os = "linux")]
        {
            fs::write(app.join("src/main.rs"), source).unwrap();
            cargo(&["build", "--offline", "--locked"]);
            native::check_binary(&root.path().join("target/debug/cloudapp"));
        }
    } else {
        eprintln!(
            "Native HTTP, real signals and drain: NOT VERIFIED (explicit host capability required)"
        );
    }
    assert_eq!(fs::read(app.join("Cargo.lock")).unwrap(), lock);
}

#[cfg(target_os = "linux")]
mod native {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        path::Path,
        process::{Child, Command},
        time::{Duration, Instant},
    };

    struct OwnedChild(Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    pub(super) fn check_binary(binary: &Path) {
        for signal in [libc::SIGTERM, libc::SIGINT] {
            let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = reservation.local_addr().unwrap();
            drop(reservation);
            let mut child = OwnedChild(
                Command::new(binary)
                    .env_remove("YETLET_HOST")
                    .env("YETLET_PORT", addr.port().to_string())
                    .spawn()
                    .unwrap(),
            );
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Ok(mut stream) =
                    TcpStream::connect_timeout(&addr, Duration::from_millis(100))
                {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    stream.write_all(b"GET /health/ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").unwrap();
                    let mut text = String::new();
                    stream.read_to_string(&mut text).unwrap();
                    assert!(text.starts_with("HTTP/1.1 200"), "{text}");
                    break;
                }
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "app exited before health"
                );
                assert!(Instant::now() < deadline, "app did not start");
                std::thread::sleep(Duration::from_millis(20));
            }
            assert_eq!(unsafe { libc::kill(child.0.id() as i32, signal) }, 0);
            loop {
                if let Some(status) = child.0.try_wait().unwrap() {
                    assert!(status.success(), "{status}");
                    break;
                }
                assert!(Instant::now() < deadline, "app failed to terminate");
                std::thread::sleep(Duration::from_millis(20));
            }
            assert!(TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_err());
            eprintln!("Actual generated main: signal={signal}, exit=0, listener closed");
        }
    }
}
