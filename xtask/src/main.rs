use serde_json::{Value, json};
use std::{
    ffi::OsString,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use yetlet_xtask::{
    Cli, HELP, parse,
    process::{self, Spec},
    repository_root,
};

fn main() -> std::process::ExitCode {
    let cli = match parse(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("usage: {error}\n{HELP}");
            return 2.into();
        }
    };
    if cli == Cli::Help {
        print!("{HELP}");
        return 0.into();
    }
    if process::install_cancellation().is_err() {
        eprintln!("infrastructure: cancellation setup failed");
        return 125.into();
    }
    match cli {
        Cli::Measure { suite, args } => {
            return command_result(yetlet_xtask::measure::run(
                &repository_root(),
                &suite,
                &args,
            ));
        }
        Cli::Qualify { suite, args } => {
            return command_result(yetlet_xtask::qualify::run(
                &repository_root(),
                &suite,
                &args,
            ));
        }
        Cli::PrivateRebuild => {
            return command_result(yetlet_xtask::release::private::run(&repository_root()));
        }
        Cli::Checksums {
            operation,
            directory,
        } => {
            return command_result(yetlet_xtask::release::checksums::run(
                &operation, &directory,
            ));
        }
        Cli::Smoke { suite, args } => {
            return command_result((|| {
                let root = repository_root();
                let out = yetlet_xtask::smoke::evidence(&root, &suite)?;
                println!("evidence: {}", out.display());
                let result = yetlet_xtask::smoke::run(&root, &suite, &args, &out)
                    .and_then(|()| yetlet_xtask::tooling::check_cancel());
                std::fs::write(
                    out.join(if suite.starts_with("scene-") {
                        "command-result.json"
                    } else {
                        "result.json"
                    }),
                    yetlet_xtask::tooling::json_bytes(
                        &json!({"suite":suite,"status":if result.is_ok(){"pass"}else{"fail"},"exit_code":result.as_ref().err().map_or(0,|e|e.code)}),
                    ),
                )?;
                result?;
                yetlet_xtask::tooling::check_cancel()
            })());
        }
        Cli::Fixture { suite, base } => {
            return command_result((|| {
                let root = repository_root();
                let out = yetlet_xtask::smoke::evidence(&root, &suite)?;
                println!("evidence: {}", out.display());
                yetlet_xtask::smoke::fixtures::run(
                    &root,
                    &out,
                    &base,
                    suite == "redis-three-app-browser",
                )?;
                yetlet_xtask::tooling::check_cancel()
            })());
        }
        Cli::Build { kind, options } => {
            return command_result(yetlet_xtask::build::run(&repository_root(), kind, options));
        }
        Cli::Check { samples } => {
            let result = if samples {
                yetlet_xtask::samples::check(&repository_root())
            } else {
                yetlet_xtask::docs::check(&repository_root())
            };
            return command_result(result.and_then(|failures| {
                if failures.is_empty() {println!("PASS: {}",if samples {"sample catalog, repository-backed routes, dependencies, READMEs and evidence"} else {"docs, internal links, Markdown structure, whitespace and DAG"});yetlet_xtask::tooling::check_cancel()}
                else {for failure in &failures {eprintln!(" - {failure}");}Err(yetlet_xtask::tooling::fail(format!("FAIL ({})",failures.len())))}
            }));
        }
        _ => {}
    }
    let Cli::Doctor { json: machine } = cli else {
        unreachable!()
    };
    let (report, code) = doctor();
    if machine {
        println!("{report}");
    } else {
        println!("doctor foundation: {}", report["status"].as_str().unwrap());
        for check in report["checks"].as_array().unwrap() {
            println!(
                "{}: {}",
                check["name"].as_str().unwrap(),
                check["status"].as_str().unwrap()
            );
            if let Some(version) = check["version"].as_str() {
                println!("  {version}");
            }
        }
    }
    if code != 0 {
        eprintln!("doctor: required foundation capability unavailable; see report");
    }
    (code as u8).into()
}

fn command_result(result: yetlet_xtask::tooling::Result<()>) -> std::process::ExitCode {
    match result {
        Ok(()) => 0.into(),
        Err(error) => {
            eprintln!("{}", error.message);
            (error.code as u8).into()
        }
    }
}

fn doctor() -> (Value, i32) {
    doctor_with(&process::CANCELLED, process::run)
}

fn doctor_with(
    cancel: &AtomicBool,
    mut execute: impl FnMut(&Spec, &AtomicBool) -> process::Report,
) -> (Value, i32) {
    let root = repository_root();
    let mut checks = Vec::new();
    let mut code = 0;
    // Root policy is deliberately read from the actual checkout, not inferred from PATH.
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"));
    let policy = std::fs::read_to_string(root.join("rust-toolchain.toml"));
    let files_ok = manifest
        .as_ref()
        .is_ok_and(|s| s.contains("rust-version = \"1.99\""))
        && policy
            .as_ref()
            .is_ok_and(|s| s.contains("channel = \"1.99.0\""))
        && root.join("Cargo.lock").is_file()
        && root.join("xtask/Cargo.lock").is_file();
    checks.push(json!({"name": "repository-policy-and-locks", "status": if files_ok {"ok"} else {"unavailable"}}));
    if !files_ok {
        code = 1;
    }
    for (tool, arg) in [("rustc", "-vV"), ("cargo", "--version")] {
        if cancel.load(Ordering::Relaxed) {
            if code != 125 {
                code = 130;
            }
            checks.push(json!({"name": tool, "status": "not_run", "reason": "cancelled"}));
            continue;
        }
        let spec = Spec {
            program: tool.into(),
            args: vec![arg.into()],
            cwd: root.clone(),
            // Prevent rustup proxies from provisioning an absent toolchain.
            env: vec![(OsString::from("RUSTUP_AUTO_INSTALL"), Some("0".into()))],
            timeout: Duration::from_secs(10),
            output_limit: 16 * 1024,
        };
        let result = execute(&spec, cancel);
        let output = std::str::from_utf8(&result.stdout.bytes).unwrap_or("");
        let version = output.lines().next().unwrap_or("");
        let compatible =
            result.success() && result.stdout.discarded == 0 && supported_version(tool, version);
        let mut check = json!({
            "name": tool, "status": if compatible {"ok"} else {"unavailable"},
            "origin": result.origin.as_str(), "native_exit_code": result.native_code(),
            "signal": result.signal(), "process_exit_code": result.exit_code(),
            "cleanup_ok": result.cleanup_ok,
            "error_kind": result.error.map(|e| format!("{e:?}")),
        });
        // Only validated version tokens enter reports; raw tool output may contain paths.
        if compatible {
            check["version"] = json!(
                version
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        } else {
            let failure = match result.origin {
                process::Origin::Spawn | process::Origin::Cleanup => 125,
                process::Origin::Timeout => 124,
                process::Origin::Cancelled => 130,
                process::Origin::Child => 1,
            };
            if code <= 1 || failure > 1 {
                code = failure;
            }
        }
        checks.push(check);
        if cancel.load(Ordering::Relaxed) && code != 125 {
            code = 130;
        }
    }
    // Cancellation during final probe cleanup must not become an incomplete PASS.
    if cancel.load(Ordering::Relaxed) && code != 125 {
        code = 130;
    }
    (
        json!({"schema_version": 1, "command": "doctor", "capability": "foundation",
        "status": if code == 0 {"ok"} else {"unavailable"}, "exit_code": code, "checks": checks}),
        code,
    )
}

fn supported_version(tool: &str, line: &str) -> bool {
    let mut words = line.split_whitespace();
    if words.next() != Some(tool) {
        return false;
    }
    let Some(version) = words.next() else {
        return false;
    };
    let mut parts = version.split('.');
    let major = parts.next().and_then(|v| v.parse::<u32>().ok());
    let minor = parts.next().and_then(|v| v.parse::<u32>().ok());
    matches!((major, minor), (Some(1), Some(99..)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_after_successful_probe_is_nonzero_and_remaining_check_is_explicit() {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        let cancel = AtomicBool::new(false);
        let mut called = Vec::new();
        let (report, code) = doctor_with(&cancel, |spec, token| {
            called.push(spec.program.clone());
            token.store(true, Ordering::Relaxed);
            process::Report {
                origin: process::Origin::Child,
                child_status: Some(std::process::ExitStatus::from_raw(0)),
                stdout: process::Capture {
                    bytes: b"rustc 1.99.0\n".to_vec(),
                    discarded: 0,
                },
                stderr: process::Capture::default(),
                error: None,
                cleanup_ok: true,
            }
        });
        assert_eq!(called, [OsString::from("rustc")]);
        assert_eq!(code, 130);
        assert_eq!(report["status"], "unavailable");
        assert_eq!(report["exit_code"], 130);
        assert_eq!(report["checks"][1]["native_exit_code"], 0);
        assert_eq!(report["checks"][2]["name"], "cargo");
        assert_eq!(report["checks"][2]["status"], "not_run");
        assert_eq!(report["checks"][2]["reason"], "cancelled");
    }

    #[test]
    fn cancellation_before_probes_launches_no_tool() {
        let cancel = AtomicBool::new(true);
        let (report, code) =
            doctor_with(&cancel, |_, _| panic!("cancelled doctor launched a tool"));
        assert_eq!(code, 130);
        assert_eq!(report["checks"][1]["status"], "not_run");
        assert_eq!(report["checks"][2]["status"], "not_run");
    }

    #[test]
    fn tool_identity_and_minimum_are_checked() {
        assert!(supported_version("rustc", "rustc 1.99.0 (abc)"));
        assert!(supported_version("cargo", "cargo 1.100.0"));
        assert!(!supported_version("rustc", "rustc 1.88.0"));
        assert!(!supported_version("rustc", "cargo 1.99.0"));
        assert!(!supported_version("rustc", "not installed"));
    }
}
