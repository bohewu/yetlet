#![cfg(any(target_os = "linux", windows))]
use sha2::{Digest, Sha256};
use std::{ffi::OsString, fs, path::Path, process::Command};
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

// Historical local-source qualification fixtures use the retained internal API.
fn scaffold(parent: &Path, name: &str, service: bool, config: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).unwrap();
    }
    yetlet_cli::scaffold::create_tailwind(name, &framework(), parent, service, config)
}

#[test]
fn tailwind_parser_is_explicit_unique_new_only_and_composes() {
    for service in [false, true] {
        for config in [false, true] {
            let mut args = ["new", "--tailwind", "styled"].map(OsString::from).to_vec();
            if service {
                args.push("--service".into());
            }
            if config {
                args.push("--settings".into());
            }
            assert!(matches!(parse(&args), Ok(Cli::New {
                service: c, runtime_settings: s, tailwind: true, vite: false, ..
            }) if c == service && s == config));
            args.push("--tailwind".into());
            assert!(parse(&args).is_err());
        }
    }
    for args in [
        vec!["new", "styled", "--tailwind=true"],
        vec!["new", "styled", "--with-tailwind"],
        vec!["doctor", "--tailwind"],
        vec!["run", "--tailwind"],
        vec!["build", "--tailwind"],
        vec!["generate", "docker", "--tailwind"],
    ] {
        assert!(parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_err());
    }
}

#[test]
fn four_profiles_write_only_sources_without_tools_or_overwrite() {
    let root = tempfile::tempdir().unwrap();
    for service in [false, true] {
        for config in [false, true] {
            let name = format!("styled-{service}-{config}");
            let result = scaffold(root.path(), &name, service, config);
            assert!(result.is_ok(), "{result:?}");
            let app = root.path().join(&name);
            let mut names = fs::read_dir(&app)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect::<Vec<_>>();
            names.sort();
            let mut expected = vec![
                ".gitignore",
                "Cargo.toml",
                "README.md",
                "src",
                "tailwind.css",
            ];
            if config {
                expected.extend([
                    ".env.example",
                    "base.toml",
                    "development.toml",
                    "staging.toml",
                    "production.toml",
                ]);
            }
            expected.sort();
            assert_eq!(
                names,
                expected.iter().map(OsString::from).collect::<Vec<_>>()
            );
            let mut sources = fs::read_dir(app.join("src"))
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect::<Vec<_>>();
            sources.sort();
            let mut expected = vec!["main.rs", "tailwind.rs"];
            if config {
                expected.insert(0, "config.rs");
            }
            assert_eq!(
                sources,
                expected.iter().map(OsString::from).collect::<Vec<_>>()
            );
            assert_eq!(
                fs::read_to_string(app.join("tailwind.css")).unwrap(),
                "@import \"tailwindcss\" source(none);\n@source \"./src\";\n"
            );
            let main = fs::read(app.join("src/main.rs")).unwrap();
            assert!(scaffold(root.path(), &name, !service, !config).is_err());
            assert_eq!(fs::read(app.join("src/main.rs")).unwrap(), main);
        }
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 4);
}

#[test]
fn selected_cli_keeps_runtime_settings_opt_in_and_adds_public_tool_settings() {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    for service in [false, true] {
        for config in [false, true] {
            let name = format!("plain-{service}-{config}");
            let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
            command
                .args(["new", &name, "--parent"])
                .arg(root.path())
                .env("PATH", root.path().join("no-executables"));
            if service {
                command.arg("--service");
            }
            if config {
                command.arg("--settings");
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let app = root.path().join(&name);
            assert_eq!(app.join("src/config.rs").exists(), config);
            let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
            assert!(!manifest.contains("path ="));
            let manifest: toml::Value = manifest.parse().unwrap();
            assert_eq!(
                manifest["dependencies"]["yetlet"]["features"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|f| f.as_str() == Some("config")),
                config
            );
            let settings: toml::Value = fs::read_to_string(app.join("base.toml"))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(settings["cli"]["vite"]["enabled"].as_bool(), Some(false));
            assert_eq!(settings["cli"]["vite"]["auto_start"].as_bool(), Some(false));
            assert_eq!(settings["cli"]["vite"]["auto_build"].as_bool(), Some(false));
            assert!(!app.join("package.json").exists());
            assert!(!app.join("frontend").exists());
            assert!(!app.join("Cargo.lock").exists());
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn tailwind_retains_symlink_parent_and_destination_refusals() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    symlink(&real, root.path().join("link")).unwrap();
    assert!(
        yetlet_cli::scaffold::create_tailwind(
            "styled",
            &framework(),
            &root.path().join("link"),
            false,
            false
        )
        .is_err()
    );
    fs::set_permissions(&real, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(
        yetlet_cli::scaffold::create_tailwind("styled", &framework(), &real, true, true).is_err()
    );
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(real.join("keep"), "user resource").unwrap();
    symlink("keep", real.join("styled")).unwrap();
    assert!(
        yetlet_cli::scaffold::create_tailwind("styled", &framework(), &real, true, false).is_err()
    );
    assert_eq!(
        fs::read_to_string(real.join("keep")).unwrap(),
        "user resource"
    );
    assert_eq!(
        fs::read_link(real.join("styled")).unwrap(),
        Path::new("keep")
    );
}

#[test]
#[ignore = "explicit developer-selected verified standalone tool and cached Cargo dependencies"]
fn generated_tailwind_apps_offline_contract() {
    let binary =
        std::env::var_os("YETLET_TEST_TAILWIND_BIN").expect("select standalone tool explicitly");
    let bytes = fs::read(&binary).unwrap();
    #[cfg(target_os = "linux")]
    let digest = "dc61b3ac6b8c9ca874c0cc4c57b2409791a64c5540404ca5f5367360babc313a";
    #[cfg(windows)]
    let digest = "e0e260ce048014e9268f6237ff18f8ccf02cef521cbd0ae04e82c2cdf7aa3955";
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), digest);
    let root = tempfile::tempdir().unwrap();
    for service in [false, true] {
        for config in [false, true] {
            let name = format!("styled-{service}-{config}");
            let result = scaffold(root.path(), &name, service, config);
            assert!(result.is_ok(), "{result:?}");
            let app = root.path().join(&name);
            fs::create_dir(app.join("dist")).unwrap();
            let result = Command::new(&binary)
                .current_dir(&app)
                .args([
                    "-i",
                    "./tailwind.css",
                    "-o",
                    "./dist/tailwind.css",
                    "--minify",
                ])
                .output()
                .unwrap();
            eprintln!(
                "{name}: standalone build {} {}",
                result.status,
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(result.status.success());
            let css = fs::read_to_string(app.join("dist/tailwind.css")).unwrap();
            for class in [
                ".p-8{",
                ".font-sans{",
                ".text-3xl{",
                ".font-bold{",
                ".max-w-3xl{",
                ".text-slate-900{",
            ] {
                assert!(css.contains(class), "missing {class}");
            }
            assert!(!css.contains("@import"));
            let cargo = |args: &[&str]| {
                let result = Command::new("cargo")
                    .args(args)
                    .current_dir(&app)
                    .env("CARGO_TARGET_DIR", root.path().join("target"))
                    .env("CARGO_NET_OFFLINE", "true")
                    .env("RUSTUP_AUTO_INSTALL", "0")
                    .output()
                    .unwrap();
                eprintln!(
                    "{name}: cargo {}: {}\n{}{}",
                    args.join(" "),
                    result.status,
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                );
                assert!(result.status.success());
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
            cargo(&["build", "--offline", "--locked"]);
            #[cfg(target_os = "linux")]
            if std::env::var_os("YETLET_TEST_NATIVE_HTTP").as_deref()
                == Some(std::ffi::OsStr::new("1"))
            {
                let executable = root
                    .path()
                    .join("target/debug")
                    .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
                native::check(&executable, &app, service, config);
            }
            // Run real main with missing CSS. It must fail before any socket or
            // configuration load, independently of the runtime working directory.
            fs::remove_file(app.join("dist/tailwind.css")).unwrap();
            let executable = root
                .path()
                .join("target/debug")
                .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
            let output = Command::new(executable)
                .current_dir(root.path())
                .env_clear()
                .output()
                .unwrap();
            assert!(!output.status.success());
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("Build dist/tailwind.css explicitly")
            );
            assert!(!String::from_utf8_lossy(&output.stdout).contains("Listening"));
            for generator in ["docker", "compose", "k8s"] {
                let output = root.path().join(format!("{name}-{generator}"));
                let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
                command
                    .args(["generate", generator, "--manifest-path"])
                    .arg(app.join("Cargo.toml"))
                    .arg("--output")
                    .arg(&output);
                if generator != "docker" {
                    command.args(["--image", "localhost:5000/team/app:v1"]);
                }
                let result = command.output().unwrap();
                assert_eq!(result.status.code(), Some(1));
                assert!(!output.exists());
            }
            assert_eq!(fs::read(app.join("Cargo.lock")).unwrap(), lock);
            eprintln!(
                "{name}: literal utilities, socketless HTTP/CSP/MIME/escaping, startup failure and generator refusals PASS; processes reaped"
            );
        }
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

    fn response(address: SocketAddr, path: &str) -> std::io::Result<String> {
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(120))?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )?;
        let mut result = String::new();
        stream.read_to_string(&mut result)?;
        Ok(result)
    }

    pub(super) fn check(executable: &Path, app: &Path, service: bool, runtime_settings: bool) {
        let reserve = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reserve.local_addr().unwrap();
        drop(reserve);

        let mut command = Command::new(executable);
        command
            .env_clear()
            .current_dir(app.parent().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if runtime_settings {
            command
                .env("APP_SERVER__PORT", address.port().to_string())
                .env("APP_SERVICE__TOKEN", "000123-synthetic-private");
        } else {
            command.env("YETLET_PORT", address.port().to_string());
        }
        let mut child = Owned(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(12);
        let html = loop {
            if let Ok(html) = response(address, "/hello/%3Cscript%3E") {
                break html;
            }
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "app exited before HTTP"
            );
            assert!(Instant::now() < deadline, "app did not start HTTP");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert!(html.starts_with("HTTP/1.1 200"), "{html}");
        let lowered = html.to_ascii_lowercase();
        assert!(lowered.contains("content-security-policy:"));
        assert!(lowered.contains("style-src 'self'"));
        assert!(!lowered.contains("unsafe-inline"));
        assert!(html.contains("Hello, &lt;script&gt;!"));
        assert!(!html.contains("<script") && !html.contains("<style"));
        assert!(!html.contains("000123-synthetic-private"));
        let prefix = "href=\"/assets/tailwind.";
        let suffix = html.split_once(prefix).unwrap().1;
        let digest = suffix.split_once(".css\"").unwrap().0;
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|ch| ch.is_ascii_hexdigit()));
        let url = format!("/assets/tailwind.{digest}.css");
        let css = response(address, &url).unwrap();
        assert!(css.starts_with("HTTP/1.1 200"), "{css}");
        let css_lower = css.to_ascii_lowercase();
        assert!(css_lower.contains("content-type: text/css; charset=utf-8"));
        assert!(css_lower.contains("x-content-type-options: nosniff"));
        assert!(css_lower.contains("cache-control: public, max-age=31536000, immutable"));
        assert!(css.contains(".p-8{") && css.contains("tailwindcss v4.3.3"));
        assert!(
            response(address, "/assets/tailwind.css")
                .unwrap()
                .starts_with("HTTP/1.1 404")
        );

        if service {
            for route in ["/health/live", "/health/ready"] {
                assert!(
                    response(address, route)
                        .unwrap()
                        .starts_with("HTTP/1.1 200")
                );
            }
            let signal = if runtime_settings {
                libc::SIGINT
            } else {
                libc::SIGTERM
            };
            assert_eq!(unsafe { libc::kill(child.0.id() as i32, signal) }, 0);
            let deadline = Instant::now() + Duration::from_secs(12);
            let status = loop {
                if let Some(status) = child.0.try_wait().unwrap() {
                    break status;
                }
                assert!(Instant::now() < deadline, "signal shutdown did not finish");
                std::thread::sleep(Duration::from_millis(20));
            };
            assert!(status.success(), "signal termination status: {status}");
        } else {
            child.0.kill().unwrap();
            let _ = child.0.wait().unwrap();
        }
        assert!(TcpStream::connect_timeout(&address, Duration::from_millis(120)).is_err());
        eprintln!(
            "Actual Tailwind HTTP/CSS/CSP/escaping and cleanup PASS: cloud={service}, config={runtime_settings}"
        );
    }
}
