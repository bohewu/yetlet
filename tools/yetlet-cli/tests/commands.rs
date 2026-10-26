#![cfg(target_os = "linux")]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

fn cli(command: &str, path: &Path) -> Command {
    let mut result = Command::new(env!("CARGO_BIN_EXE_yetlet"));
    result.arg(command).arg("--manifest-path").arg(path);
    result
}

fn code(output: Output, expected: i32) -> Output {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn frozen_cli_build_run_and_lock_cache_failures_use_native_cargo() {
    let root = tempfile::Builder::new()
        .prefix("yt087b 空 間;$literal-")
        .tempdir()
        .unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let manifest = root.path().join("Cargo.toml");
    let lock = root.path().join("Cargo.lock");
    let manifest_text =
        "[package]\nname='yt087b_fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n";
    fs::write(&manifest, manifest_text).unwrap();
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0o600)).unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/main.rs"),
        "fn main() { std::process::exit(23); }\n",
    )
    .unwrap();
    code(cli("build", &manifest).output().unwrap(), 1);
    code(cli("run", &manifest).output().unwrap(), 1);
    assert!(!lock.exists());
    // Explicit fixture lock, independent of all repository/CLI lockfiles.
    fs::write(
        &lock,
        "version=4\n[[package]]\nname='yt087b_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
    let initial_lock = fs::read(&lock).unwrap();
    code(cli("build", &manifest).output().unwrap(), 0);
    code(cli("run", &manifest).output().unwrap(), 23);
    assert_eq!(fs::read(&lock).unwrap(), initial_lock);
    fs::write(
        &manifest,
        format!("{manifest_text}\n[dependencies]\nserde_json='=1.0.149'\n"),
    )
    .unwrap();
    for command in ["build", "run"] {
        let output = code(cli(command, &manifest).output().unwrap(), 101);
        assert!(String::from_utf8_lossy(&output.stderr).contains("lock"));
        assert_eq!(fs::read(&lock).unwrap(), initial_lock);
    }
    // Separate, explicit offline setup for the cache-negative test. Never CLI bootstrap.
    code(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(&manifest)
            .current_dir(root.path())
            .env("RUSTUP_AUTO_INSTALL", "0")
            .output()
            .unwrap(),
        0,
    );
    let cached_lock = fs::read(&lock).unwrap();
    let cache = tempfile::tempdir().unwrap();
    for command in ["build", "run"] {
        let output = code(
            cli(command, &manifest)
                .env("CARGO_HOME", cache.path())
                .output()
                .unwrap(),
            101,
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("offline"));
        assert_eq!(fs::read(&lock).unwrap(), cached_lock);
    }
}

#[test]
fn natural_cli_exit_keeps_grace_for_adopted_children_and_reaps_them() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let manifest = root.path().join("Cargo.toml");
    fs::write(
        &manifest,
        "[package]\nname='grace_fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        root.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='grace_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/main.rs"),
        r#"
use std::{fs, path::PathBuf, process::Command, thread, time::{Duration, Instant}};
fn main() {
    let root = PathBuf::from(std::env::var_os("GRACE_ROOT").unwrap());
    let script = if std::env::var("GRACE_MODE").unwrap() == "graceful" {
        "trap 'sleep 0.1; printf done > \"$1/graceful\"; exit 0' TERM; echo $$ > \"$1/leaf.pid\"; while :; do sleep 0.02; done"
    } else {
        "trap '' TERM; echo $$ > \"$1/leaf.pid\"; while :; do sleep 0.02; done"
    };
    let _child = Command::new("/bin/sh").args(["-c", script, "sh"]).arg(&root).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read_to_string(root.join("leaf.pid")).ok()
        .and_then(|value| value.trim().parse::<u32>().ok()).is_none() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    std::process::exit(17);
}
"#,
    )
    .unwrap();
    code(cli("build", &manifest).output().unwrap(), 0);
    for mode in ["graceful", "resistant"] {
        code(
            cli("run", &manifest)
                .env("GRACE_ROOT", root.path())
                .env("GRACE_MODE", mode)
                .output()
                .unwrap(),
            17,
        );
        let pid = fs::read_to_string(root.path().join("leaf.pid"))
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        assert_eq!(root.path().join("graceful").exists(), mode == "graceful");
        fs::remove_file(root.path().join("leaf.pid")).unwrap();
        if mode == "graceful" {
            fs::remove_file(root.path().join("graceful")).unwrap();
        }
    }
}

#[test]
fn discovery_is_exclusive_scoped_and_needs_no_project_or_tools() {
    let root = tempfile::tempdir().unwrap();
    for arguments in [
        vec![],
        vec!["--help"],
        vec!["-h"],
        vec!["--version"],
        vec!["-V"],
        vec!["new", "--help"],
        vec!["doctor", "-h"],
        vec!["inspect", "--help"],
        vec!["run", "--help"],
        vec!["watch", "--help"],
        vec!["watch", "-h"],
        vec!["build", "-h"],
        vec!["generate", "--help"],
        vec!["generate", "docker", "-h"],
        vec!["generate", "compose", "--help"],
        vec!["generate", "k8s", "-h"],
    ] {
        let output = code(
            Command::new(env!("CARGO_BIN_EXE_yetlet"))
                .args(&arguments)
                .current_dir(root.path())
                .env("PATH", "")
                .env("YETLET_CONFIG_ROOT", "missing")
                .output()
                .unwrap(),
            0,
        );
        assert!(!output.stdout.is_empty(), "{arguments:?}");
        assert!(output.stderr.is_empty(), "{arguments:?}");
        let help = String::from_utf8(output.stdout).unwrap();
        if arguments == ["inspect", "--help"] {
            assert!(help.contains("project-execution-plan schema 1"));
            assert!(help.contains("browser watch plans select WASM rebuild/full-page reload"));
            assert!(help.contains("cli.vite.auto_start and cli.vite.auto_build are both true"));
            assert!(!help.contains("Browser watch remains unsupported"));
        }
        if arguments.first() == Some(&"watch") {
            assert!(help.contains("Native: forces cli.rust.watch"));
            assert!(help.contains("requires Watchexec on PATH"));
            assert!(
                help.contains("Browser: requires both cli.vite.auto_start and cli.vite.auto_build")
            );
            assert!(help.contains("WASM rebuild/full-page reload; no native watcher"));
            assert!(help.contains("Vite owns TS/CSS HMR"));
        }
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    for arguments in [
        vec!["--version", "secret-value"],
        vec!["new", "demo", "--help"],
        vec!["doctor", "--help", "--json"],
        vec!["inspect", "--help", "--json"],
        vec!["inspect", "--json", "--json"],
        vec!["inspect", "--json=secret-value"],
        vec!["inspect", "--release"],
        vec!["inspect", "secret-value"],
        vec!["inspect", "--manifest-path=x", "--manifest-path=y"],
        vec!["inspect", "--manifest-path="],
        vec!["generate"],
        vec!["generate", "k8s", "--help", "--output", "secret-value"],
        vec!["run", "--version"],
        vec!["watch", "--release"],
        vec!["watch", "secret-value"],
        vec!["watch", "--manifest-path=x", "--manifest-path=y"],
        vec!["watch", "--help", "--manifest-path=secret-value"],
        vec!["build", "--release", "-h"],
    ] {
        let output = code(
            Command::new(env!("CARGO_BIN_EXE_yetlet"))
                .args(&arguments)
                .current_dir(root.path())
                .output()
                .unwrap(),
            2,
        );
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("CLI_USAGE"));
        assert!(!stderr.contains("secret-value"));
        assert!(stderr.contains("Usage:"));
    }
}

#[test]
fn doctor_json_is_one_environment_report_even_when_tools_are_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let output = code(
        Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .args(["doctor", "--json"])
            .current_dir(root.path())
            .env("PATH", "")
            .env("YETLET_CONFIG_ROOT", "missing")
            .output()
            .unwrap(),
        1,
    );
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));
    assert_eq!(output.stdout.iter().filter(|&&v| v == b'\n').count(), 1);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["scope"], "environment");
    assert_eq!(value["completed"], true);
    assert_eq!(value["ok"], false);
    let mut fields = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    fields.sort_unstable();
    assert_eq!(
        fields,
        [
            "cli_version",
            "completed",
            "diagnostics",
            "ok",
            "schema_version",
            "scope"
        ]
    );
    let diagnostics = value["diagnostics"].as_array().unwrap();
    for code in [
        "DOCTOR_RUSTC_VERSION",
        "DOCTOR_CARGO_VERSION",
        "DOCTOR_NEW_PLATFORM",
        "DOCTOR_FRAMEWORK",
        "DOCTOR_APP_READINESS",
        "DOCTOR_ASSETS",
        "DOCTOR_BUILD_DEV",
        "DOCTOR_CONSOLE",
        "DOCTOR_GENERATOR",
    ] {
        assert!(diagnostics.iter().any(|d| d["code"] == code), "{code}");
    }
    for d in diagnostics {
        assert_eq!(d.as_object().unwrap().len(), 5);
        for field in ["code", "origin", "capability", "status", "message"] {
            assert!(d[field].is_string());
        }
        if ![
            "DOCTOR_RUSTC_VERSION",
            "DOCTOR_CARGO_VERSION",
            "DOCTOR_NEW_PLATFORM",
        ]
        .iter()
        .any(|code| d["code"] == *code)
        {
            assert_eq!(d["status"], "not-checked");
        }
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

fn inspection_project() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname='inspect_fixture'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        root.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='inspect_fixture'\nversion='0.1.0'\n",
    )
    .unwrap();
    root
}

const INSPECT_BASE: &str = "[cli]\ntarget='native'\n[cli.rust]\nwatch=false\n[cli.vite]\nenabled=true\nroot='frontend/'\nauto_start=true\nauto_build=false\n[cli.generate.k8s]\nreplicas=1\n[service]\ntoken='PUBLIC_TOKEN_SENTINEL'\n";

#[test]
fn inspect_explains_native_browser_and_profile_overrides_without_tools_or_writes() {
    let root = inspection_project();
    // Every tool would leave a marker if inspection accidentally invoked it.
    let tools = root.path().join("tools");
    fs::create_dir(&tools).unwrap();
    for tool in ["cargo", "rustc", "node", "npm", "watchexec"] {
        let path = tools.join(tool);
        fs::write(&path, "#!/bin/sh\nprintf child > child-marker\nexit 97\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let mut command = cli("inspect", &root.path().join("Cargo.toml"));
    command
        .current_dir(root.path())
        .env_clear()
        .env("PATH", &tools);
    let output = code(command.arg("--json").output().unwrap(), 0);
    let defaults: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(defaults["environment"], "Production");
    assert_eq!(defaults["vite"]["enabled"], false);
    assert_eq!(defaults["commands"]["run"]["rust_watch"]["selected"], false);
    assert_eq!(
        defaults["commands"]["watch"]["rust_watch"]["selected"],
        true
    );
    assert_eq!(
        defaults["commands"]["build"]["vite"]["reason"],
        "cli.vite.enabled is false"
    );

    fs::write(root.path().join("base.toml"), INSPECT_BASE).unwrap();
    fs::write(root.path().join("production.toml"), "").unwrap();
    fs::write(
        root.path().join("development.toml"),
        "[cli.rust]\nwatch=true\n[cli.vite]\nauto_start=false\n",
    )
    .unwrap();
    fs::write(
        root.path().join("staging.toml"),
        "[cli]\ntarget='browser'\n[cli.vite]\nauto_build=true\n",
    )
    .unwrap();
    fs::write(
        root.path().join("secret.toml"),
        "not valid TOML PRIVATE_FILE_SENTINEL",
    )
    .unwrap();
    let expected_files = fs::read_dir(root.path()).unwrap().count();
    let original_manifest = fs::read(root.path().join("Cargo.toml")).unwrap();
    let original_lock = fs::read(root.path().join("Cargo.lock")).unwrap();
    for (profile, target, watch, start, build) in [
        ("Production", "native", false, true, false),
        ("Development", "native", true, false, false),
        ("Staging", "browser", false, true, true),
    ] {
        for json in [false, true] {
            let mut command = cli("inspect", &root.path().join("Cargo.toml"));
            command
                .current_dir(root.path())
                .env_clear()
                .env("PATH", &tools)
                .env("APP_ENVIRONMENT", profile)
                .env("YETLET_DEV_SECRETS", root.path().join("secret.toml"))
                .env("APP_SERVICE__TOKEN", "ENV_TOKEN_SENTINEL")
                .env("APP_SIGNING__PASSWORD", "PASSWORD_SENTINEL");
            if json {
                command.arg("--json");
            }
            let output = code(command.output().unwrap(), 0);
            assert!(output.stderr.is_empty());
            let text = String::from_utf8(output.stdout).unwrap();
            for secret in [
                "PUBLIC_TOKEN_SENTINEL",
                "PRIVATE_FILE_SENTINEL",
                "ENV_TOKEN_SENTINEL",
                "PASSWORD_SENTINEL",
                root.path().to_str().unwrap(),
            ] {
                assert!(!text.contains(secret));
            }
            assert!(text.contains("not verified") && text.contains("full reload"));
            if json {
                assert_eq!(text.lines().count(), 1);
                let report: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(report["schema_version"], 1);
                assert_eq!(report["scope"], "project-execution-plan");
                assert_eq!(report["environment"], profile);
                assert_eq!(report["target"], target);
                assert_eq!(report["rust_watch"], watch);
                assert_eq!(
                    report["commands"]["build"]["rust"]["selected"],
                    target == "native"
                );
                assert_eq!(report["commands"]["build"]["rust_watch"]["selected"], false);
                assert_eq!(report["commands"]["build"]["vite"]["selected"], build);
                assert_eq!(report["commands"]["run"]["vite"]["selected"], start);
                assert_eq!(report["commands"]["run"]["rust_watch"]["selected"], watch);
                assert_eq!(report["commands"]["watch"]["supported"], true);
                assert_eq!(
                    report["commands"]["watch"]["vite"]["selected"],
                    start && (target == "native" || build)
                );
            } else {
                assert!(text.contains(&format!("Environment: {profile}\nTarget: {target}")));
                assert!(
                    text.contains("build:") && text.contains("run:") && text.contains("watch:")
                );
            }
        }
    }
    for (start, build) in [(true, true), (false, true), (true, false), (false, false)] {
        let output = code(
            cli("inspect", &root.path().join("Cargo.toml"))
                .arg("--json")
                .env_clear()
                .env("PATH", &tools)
                .env("APP_ENVIRONMENT", "Staging")
                .env("APP_CLI__VITE__AUTO_START", start.to_string())
                .env("APP_CLI__VITE__AUTO_BUILD", build.to_string())
                .output()
                .unwrap(),
            0,
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["commands"]["watch"]["vite"]["selected"],
            start && build
        );
        assert_eq!(
            report["commands"]["watch"]["rust_watch"]["selected"],
            start && build
        );
        assert_eq!(report["commands"]["watch"]["supported"], true);
    }
    // Explicit external public root, profile then ENV precedence, equals grammar.
    let public = inspection_project();
    fs::write(public.path().join("base.toml"), INSPECT_BASE).unwrap();
    fs::write(
        public.path().join("development.toml"),
        "[cli.vite]\nauto_build=true\n",
    )
    .unwrap();
    let output = code(
        Command::new(env!("CARGO_BIN_EXE_yetlet"))
            .args(["inspect", "--json", "--manifest-path=Cargo.toml"])
            .current_dir(root.path())
            .env_clear()
            .env("PATH", &tools)
            .env("YETLET_CONFIG_ROOT", public.path())
            .env("APP_ENVIRONMENT", "Development")
            .env("APP_CLI__VITE__AUTO_BUILD", "false")
            .env("APP_CLI__RUST__WATCH", "true")
            .output()
            .unwrap(),
        0,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["commands"]["build"]["vite"]["selected"], false);
    assert_eq!(report["commands"]["run"]["rust_watch"]["selected"], true);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), expected_files);
    assert_eq!(
        fs::read(root.path().join("Cargo.toml")).unwrap(),
        original_manifest
    );
    assert_eq!(
        fs::read(root.path().join("Cargo.lock")).unwrap(),
        original_lock
    );
    assert_eq!(
        fs::read_to_string(root.path().join("base.toml")).unwrap(),
        INSPECT_BASE
    );
    assert_eq!(fs::read_dir(&tools).unwrap().count(), 5);
    // No frontend, source, dist, target, node_modules or child marker exists.
    for name in [
        "frontend",
        "src",
        "dist",
        "target",
        "node_modules",
        "child-marker",
    ] {
        assert!(!root.path().join(name).exists());
    }
}

#[test]
fn inspect_escapes_allowlisted_relative_root_in_human_and_json_output() {
    let root = inspection_project();
    let relative = "ui assets/\u{1b}[31m\u{9b}\u{202e}前端😀\u{7f}";
    fs::write(
        root.path().join("base.toml"),
        INSPECT_BASE.replace(
            "root='frontend/'",
            "root=\"ui assets/\\u001b[31m\\u009b\\u202e前端😀\\u007f\"",
        ),
    )
    .unwrap();
    fs::write(root.path().join("production.toml"), "").unwrap();
    for json in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
        command
            .arg("inspect")
            .current_dir(root.path())
            .env_clear()
            .env("PATH", "");
        if json {
            command.arg("--json");
        }
        let output = code(command.output().unwrap(), 0);
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.is_ascii());
        assert!(text.chars().all(|c| !c.is_control() || c == '\n'));
        if json {
            let report: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(report["vite"]["root"], relative);
        } else {
            assert!(text.contains("\\u{1b}[31m\\u{9b}\\u{202e}"));
        }
    }
}

#[test]
fn inspect_retains_safe_source_profile_values_errors_and_ordering() {
    let root = inspection_project();
    fs::write(root.path().join("base.toml"), INSPECT_BASE).unwrap();
    fs::write(root.path().join("production.toml"), "").unwrap();
    for json in [false, true] {
        for (key, value, hint) in [
            ("YETLET_CONFIG_ROOT", "SOURCE_SENTINEL", "source"),
            ("APP_ENVIRONMENT", "PROFILE_SENTINEL", "profile"),
            ("APP_CLI__VITE__AUTO_START", "VALUES_SENTINEL", "values"),
            ("APP_CLI__VITE__ROOT", "/ABSOLUTE_SENTINEL", "values"),
            (
                "APP_CLI__VITE__UNKNOWN_SENTINEL",
                "TOKEN_SENTINEL",
                "values",
            ),
        ] {
            let mut command = cli("inspect", &root.path().join("Cargo.toml"));
            command.env_clear().env("PATH", "").env(key, value);
            if key == "YETLET_CONFIG_ROOT" {
                command.env("APP_ENVIRONMENT", "PROFILE_SENTINEL");
            }
            if json {
                command.arg("--json");
            }
            let output = code(command.output().unwrap(), 1);
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.starts_with("CLI_PROJECT inspect: CLI_PROJECT settings"));
            assert!(text.contains(&format!("settings {hint}:")), "{text}");
            assert!(!text.contains("SENTINEL") && !text.contains(root.path().to_str().unwrap()));
        }
    }
    fs::write(root.path().join("base.toml"), "INVALID_TOML_SENTINEL").unwrap();
    let output = code(
        cli("inspect", &root.path().join("Cargo.toml"))
            .env_clear()
            .output()
            .unwrap(),
        1,
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("settings values:"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("SENTINEL"));
    // Required source checks precede parsing values, and manifest/lock precedes settings.
    fs::remove_file(root.path().join("production.toml")).unwrap();
    let output = code(
        cli("inspect", &root.path().join("Cargo.toml"))
            .env_clear()
            .output()
            .unwrap(),
        1,
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("settings source:"));
    fs::remove_file(root.path().join("Cargo.lock")).unwrap();
    let output = code(
        cli("inspect", &root.path().join("Cargo.toml"))
            .env_clear()
            .env("APP_ENVIRONMENT", "PROFILE_SENTINEL")
            .output()
            .unwrap(),
        1,
    );
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("Cargo.lock") && !text.contains("settings") && !text.contains("SENTINEL")
    );
}
