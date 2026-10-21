#![cfg(any(target_os = "linux", windows))]
use std::{fs, path::Path, process::Command};

fn framework() -> std::path::PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    #[cfg(windows)]
    {
        // Windows canonicalize() returns a verbatim \\?\ path which the
        // existing strict NTFS adapter deliberately rejects. Use the absolute
        // Cargo-provided directory spelling and its two lexical parents.
        crate_dir.parent().unwrap().parent().unwrap().to_path_buf()
    }
    #[cfg(target_os = "linux")]
    {
        crate_dir.join("../..").canonicalize().unwrap()
    }
}

// Historical local-source qualification fixtures use the retained internal API.
fn scaffold(parent: &Path, name: &str, cloud: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).unwrap();
    }
    yetlet_cli::scaffold::create_config(name, &framework(), parent, cloud)
}

#[test]
fn both_config_profiles_generate_only_safe_files_without_tools_or_overwrite() {
    let root = tempfile::tempdir().unwrap();
    for (name, cloud) in [("configured", false), ("configured-cloud", true)] {
        let result = scaffold(root.path(), name, cloud);
        assert!(result.is_ok(), "{result:?}");
        let app = root.path().join(name);
        let mut names = fs::read_dir(&app)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                ".env.example",
                ".gitignore",
                "Cargo.toml",
                "README.md",
                "base.toml",
                "development.toml",
                "production.toml",
                "src",
                "staging.toml"
            ]
            .map(std::ffi::OsString::from)
        );
        assert_eq!(fs::read_dir(app.join("src")).unwrap().count(), 2);
        for file in [
            "Cargo.toml",
            "README.md",
            ".env.example",
            "base.toml",
            "src/config.rs",
            "src/main.rs",
        ] {
            assert!(
                !fs::read_to_string(app.join(file))
                    .unwrap()
                    .contains("must-not-be-interpolated")
            );
        }
        let manifest = fs::read(app.join("Cargo.toml")).unwrap();
        assert!(scaffold(root.path(), name, !cloud).is_err());
        assert_eq!(fs::read(app.join("Cargo.toml")).unwrap(), manifest);
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
}

#[cfg(target_os = "linux")]
#[test]
fn config_profile_preserves_symlink_and_unsafe_parent_refusal() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let link = root.path().join("link");
    symlink(&real, &link).unwrap();
    // Call the API directly so the scaffold helper cannot change parent modes.
    assert!(yetlet_cli::scaffold::create_config("app", &framework(), &link, true).is_err());
    fs::set_permissions(&real, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(yetlet_cli::scaffold::create_config("app", &framework(), &real, false).is_err());
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(real.join("keep"), "existing").unwrap();
    symlink("keep", real.join("app")).unwrap();
    assert!(yetlet_cli::scaffold::create_config("app", &framework(), &real, true).is_err());
    assert_eq!(fs::read_to_string(real.join("keep")).unwrap(), "existing");
    assert_eq!(fs::read_link(real.join("app")).unwrap(), Path::new("keep"));
}

#[test]
#[ignore = "explicit real cached-dependency generated-app lock/check/test; HTTP requires YETLET_TEST_NATIVE_HTTP=1"]
fn generated_config_apps_offline_contract() {
    let root = tempfile::tempdir().unwrap();
    for (name, cloud) in [("configured", false), ("configured-cloud", true)] {
        let result = scaffold(root.path(), name, cloud);
        assert!(result.is_ok(), "{result:?}");
        let app = root.path().join(name);
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
                "{name}: cargo {}\n{}{}",
                args.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                output.status.success(),
                "{name}: cargo {args:?}: {}",
                output.status
            );
        };
        cargo(&["generate-lockfile", "--offline"]);
        let lock = fs::read(app.join("Cargo.lock")).unwrap();
        cargo(&["fmt", "--all", "--", "--check"]);
        cargo(&["check", "--offline", "--locked", "--all-targets"]);
        cargo(&["test", "--offline", "--locked"]);
        cargo(&[
            "clippy",
            "--offline",
            "--locked",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]);
        if cloud {
            // YT-101 now accepts exact cloud/config sources and public TOMLs.
            for generator in ["docker", "compose", "k8s"] {
                let output = root.path().join(generator);
                let mut cmd = Command::new(env!("CARGO_BIN_EXE_yetlet"));
                cmd.args(["generate", generator, "--manifest-path"])
                    .arg(app.join("Cargo.toml"))
                    .arg("--output")
                    .arg(&output);
                if generator != "docker" {
                    cmd.args(["--image", "localhost:5000/team/app:v1"]);
                }
                let result = cmd.output().unwrap();
                assert_eq!(result.status.code(), Some(0));
                assert!(output.exists());
                eprintln!(
                    "config cloud {generator}: generated optional delivery profile: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        #[cfg(target_os = "linux")]
        {
            cargo(&["build", "--offline", "--locked"]);
            let binary = root.path().join("target/debug").join(name);
            native::startup_failures(&binary, &app);
            if std::env::var_os("YETLET_TEST_NATIVE_HTTP").as_deref()
                == Some(std::ffi::OsStr::new("1"))
            {
                native::check(&binary, &app, cloud);
            } else {
                eprintln!("Native HTTP/signals: NOT VERIFIED (host socket permission required)");
            }
        }
        assert_eq!(fs::read(app.join("Cargo.lock")).unwrap(), lock);
    }
}

#[cfg(target_os = "linux")]
mod native {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{SocketAddr, TcpListener, TcpStream},
        process::{Child, Stdio},
        time::{Duration, Instant},
    };

    struct Owned(Child);
    impl Drop for Owned {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn command(binary: &Path, cwd: &Path, port: u16) -> Command {
        let mut cmd = Command::new(binary);
        cmd.env_clear()
            .current_dir(cwd)
            .env("APP_SERVER__PORT", port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }

    fn wait(child: &mut Owned) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "child did not exit");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn request(address: SocketAddr, path: &str) -> std::io::Result<String> {
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(100))?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )?;
        let mut text = String::new();
        stream.read_to_string(&mut text)?;
        Ok(text)
    }

    pub(super) fn startup_failures(binary: &Path, app: &Path) {
        let cwd = app.parent().unwrap();
        for (profile, key, value) in [
            ("Production", "APP_SERVICE__TOKEN", ""),
            ("Production", "APP_SERVICE__TOKEN", "   "),
            ("Production", "APP_UNKNOWN", "private-sentinel"),
            ("Invalid", "APP_SERVICE__TOKEN", "private-sentinel"),
            (
                "Development",
                "YETLET_DEV_SECRETS",
                "/nonexistent/yt097-private",
            ),
            ("Development", "YETLET_DEV_SECRETS", "relative-private.env"),
        ] {
            let mut child = Owned(
                command(binary, cwd, 3000)
                    .env("APP_ENVIRONMENT", profile)
                    .env(key, value)
                    .spawn()
                    .unwrap(),
            );
            assert!(!wait(&mut child).success());
            let mut error = String::new();
            child
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut error)
                .unwrap();
            assert!(
                error.contains("application configuration invalid"),
                "{error}"
            );
            assert!(
                !error.contains("private-sentinel") && error.len() < 256,
                "{error}"
            );
        }
        eprintln!("Actual generated main: six bounded redacted startup failures; children reaped");
    }

    pub(super) fn check(binary: &Path, app: &Path, cloud: bool) {
        let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reservation.local_addr().unwrap();
        drop(reservation);
        // A different runtime cwd proves all settings resolve from manifest root.
        let cwd = app.parent().unwrap();
        let dev = app.join("selected-development.env");
        fs::write(
            &dev,
            "APP_SERVICE__TOKEN=000456\nAPP_SERVICE__PUBLIC_MESSAGE=from-file\n",
        )
        .unwrap();
        // Production/Staging must ignore even an unusable development path;
        // Development's selected file supplies a token, then OS overrides win.
        for profile in ["Production", "Staging", "Development"] {
            let mut cmd = command(binary, cwd, address.port());
            cmd.env("APP_ENVIRONMENT", profile)
                .env("APP_SERVICE__PUBLIC_MESSAGE", "Visible <message> & safe")
                .env(
                    "YETLET_DEV_SECRETS",
                    if profile == "Development" {
                        dev.as_os_str()
                    } else {
                        std::ffi::OsStr::new("relative-invalid-private")
                    },
                );
            if profile != "Development" {
                cmd.env("APP_SERVICE__TOKEN", "000123");
            }
            let mut child = Owned(cmd.spawn().unwrap());
            let deadline = Instant::now() + Duration::from_secs(10);
            let response = loop {
                if let Ok(text) = request(address, "/") {
                    break text;
                }
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "app exited before request"
                );
                assert!(Instant::now() < deadline, "app did not start");
                std::thread::sleep(Duration::from_millis(20));
            };
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            assert!(
                response
                    .to_ascii_lowercase()
                    .contains("content-security-policy:")
            );
            assert!(
                response.contains("Visible &lt;message&gt; &amp; safe"),
                "{response}"
            );
            assert!(
                !response.contains("000123")
                    && !response.contains("000456")
                    && !response.contains("<script")
            );
            if cloud {
                for path in ["/health/live", "/health/ready"] {
                    assert!(request(address, path).unwrap().starts_with("HTTP/1.1 200"));
                }
                let signal = if profile == "Staging" {
                    libc::SIGINT
                } else {
                    libc::SIGTERM
                };
                assert_eq!(unsafe { libc::kill(child.0.id() as i32, signal) }, 0);
                assert!(wait(&mut child).success());
            } else {
                child.0.kill().unwrap();
                let _ = child.0.wait().unwrap();
            }
            assert!(TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err());
            eprintln!(
                "config cloud={cloud} {profile}: real SSR/CSP/escaping, startup rejection, cleanup PASS"
            );
        }
        // Parent tempfile owns the entire generated application and dev fixture.
    }
}
