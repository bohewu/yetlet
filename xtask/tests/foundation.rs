use std::{
    ffi::OsString,
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
use yetlet_xtask::{
    Cli, parse,
    process::{self, Origin, Spec},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "yetlet YT083 測試 {}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

// This same integration-test binary is the only fixture. It is not a shipped command.
fn spec(mode: &str, cwd: PathBuf) -> Spec {
    Spec {
        program: std::env::current_exe().unwrap().into_os_string(),
        args: vec![
            "--exact".into(),
            "fixture".into(),
            "--ignored".into(),
            "--nocapture".into(),
        ],
        cwd,
        env: vec![("YT083_FIXTURE".into(), Some(mode.into()))],
        timeout: Duration::from_secs(5),
        output_limit: 256 * 1024,
    }
}
fn run(spec: &Spec) -> process::Report {
    process::run(spec, &AtomicBool::new(false))
}

struct Unrelated(Child);
impl Drop for Unrelated {
    fn drop(&mut self) {
        self.0.kill().unwrap();
        self.0.wait().unwrap();
    }
}

#[test]
#[ignore = "test-only subprocess fixture"]
fn fixture() {
    match std::env::var("YT083_FIXTURE").unwrap().as_str() {
        "literal" => {
            let expected = std::env::var_os("YT083_EXPECTED_CWD").unwrap();
            assert_eq!(std::env::current_dir().unwrap(), PathBuf::from(expected));
            assert_eq!(
                std::env::var_os("YT083_LITERAL").unwrap(),
                OsString::from(" ; & | $() `cmd` * ? 中文 ")
            );
            assert!(std::env::var_os("YT083_REMOVED").is_none());
            if std::env::var_os("YT083_CHECK_ARGV").is_some() {
                assert_eq!(
                    std::env::args_os().last().unwrap(),
                    OsString::from("; touch SHOULD_NOT_EXIST & $(echo bad) `cmd` | * 中文")
                );
            }
            println!("literal-ok");
        }
        "flood" => {
            let stdout = thread::spawn(|| {
                std::io::stdout().write_all(&vec![b'O'; 300_000]).unwrap();
            });
            std::io::stderr().write_all(&vec![b'E'; 300_000]).unwrap();
            stdout.join().unwrap();
        }
        "failure" => std::process::exit(37),
        #[cfg(windows)]
        "wide-failure" => std::process::exit(0xc000_013a_u32 as i32),
        #[cfg(unix)]
        "signal" => unsafe {
            libc::raise(libc::SIGTERM);
        },
        "wait" => thread::sleep(Duration::from_secs(30)),
        "descendant" | "exit-with-descendant" | "fail-with-descendant" => {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "fixture", "--ignored", "--nocapture"])
                .env("YT083_FIXTURE", "heartbeat")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();
            let address_file = PathBuf::from(std::env::var_os("YT083_HEARTBEAT").unwrap());
            let deadline = Instant::now() + Duration::from_secs(3);
            while !address_file.exists() {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
            if std::env::var("YT083_FIXTURE").unwrap() == "exit-with-descendant" {
                // Intentionally orphan this owned descendant, retaining inherited pipes.
                std::process::exit(0);
            }
            if std::env::var("YT083_FIXTURE").unwrap() == "fail-with-descendant" {
                std::process::exit(37);
            }
            let _ = child.wait();
        }
        "heartbeat" => {
            let path = PathBuf::from(std::env::var_os("YT083_HEARTBEAT").unwrap());
            let mut file = std::fs::File::create(path).unwrap();
            for _ in 0..3000 {
                file.write_all(b".").unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        }
        _ => panic!("unknown fixture mode"),
    }
}

#[test]
fn literal_native_arguments_environment_and_unicode_cwd() {
    let temp = Temp::new();
    let mut task = spec("literal", temp.0.clone());
    let executable = temp.0.join(if cfg!(windows) {
        "fixture 工具.exe"
    } else {
        "fixture 工具"
    });
    std::fs::copy(&task.program, &executable).unwrap();
    task.program = executable.into_os_string();
    // libtest's --skip value is passed literally; shell metacharacters do not execute.
    task.args.extend([
        "--skip".into(),
        "; touch SHOULD_NOT_EXIST & $(echo bad) `cmd` | * 中文".into(),
    ]);
    task.env.extend([
        ("YT083_CHECK_ARGV".into(), Some("1".into())),
        (
            "YT083_EXPECTED_CWD".into(),
            Some(temp.0.clone().into_os_string()),
        ),
        (
            "YT083_LITERAL".into(),
            Some(" ; & | $() `cmd` * ? 中文 ".into()),
        ),
        ("YT083_REMOVED".into(), None),
    ]);
    let report = run(&task);
    assert!(report.success(), "{report:?}");
    assert!(String::from_utf8_lossy(&report.stdout.bytes).contains("literal-ok"));
    assert!(!temp.0.join("SHOULD_NOT_EXIST").exists());
}

#[cfg(unix)]
#[test]
fn non_utf8_native_environment_and_cwd() {
    use std::os::unix::ffi::OsStringExt;
    let temp = Temp::new();
    let path = temp.0.join(OsString::from_vec(vec![b'x', 0xff]));
    std::fs::create_dir(&path).unwrap();
    let mut task = spec("literal", path.clone());
    task.env.extend([
        ("YT083_EXPECTED_CWD".into(), Some(path.into_os_string())),
        (
            "YT083_LITERAL".into(),
            Some(" ; & | $() `cmd` * ? 中文 ".into()),
        ),
        ("YT083_REMOVED".into(), None),
    ]);
    assert!(run(&task).success());
}

#[test]
fn simultaneous_output_is_drained_and_bounded() {
    let temp = Temp::new();
    let mut task = spec("flood", temp.0.clone());
    task.output_limit = 1024;
    let report = run(&task);
    assert!(report.success(), "{report:?}");
    assert_eq!(report.stdout.bytes.len(), 1024);
    assert_eq!(report.stderr.bytes.len(), 1024);
    assert!(report.stdout.discarded >= 298_976);
    assert!(report.stderr.discarded >= 298_976);
}

#[test]
fn exact_child_failure_and_missing_executable() {
    let temp = Temp::new();
    let report = run(&spec("failure", temp.0.clone()));
    assert_eq!(report.origin, Origin::Child);
    assert_eq!(report.native_code(), Some(37));
    assert_eq!(report.exit_code(), 37);
    assert!(!report.success());
    let mut task = spec("wait", temp.0.clone());
    task.program = temp.0.join("no such executable").into_os_string();
    let report = run(&task);
    assert_eq!(report.origin, Origin::Spawn);
    assert_eq!(report.exit_code(), 125);
    assert!(report.cleanup_ok);
}

#[cfg(unix)]
#[test]
fn exact_child_signal() {
    let temp = Temp::new();
    let report = run(&spec("signal", temp.0.clone()));
    assert_eq!(report.origin, Origin::Child);
    assert_eq!(report.signal(), Some(libc::SIGTERM));
    assert_eq!(report.exit_code(), 128 + libc::SIGTERM);
}

#[cfg(windows)]
#[test]
fn full_windows_status_survives_portable_mapping() {
    let temp = Temp::new();
    let report = run(&spec("wide-failure", temp.0.clone()));
    assert_eq!(report.origin, Origin::Child);
    assert_eq!(report.native_code(), Some(0xc000_013a));
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn timeout_and_cancellation_are_nonzero_and_bounded() {
    let temp = Temp::new();
    let mut task = spec("wait", temp.0.clone());
    task.timeout = Duration::from_millis(100);
    let start = Instant::now();
    let report = run(&task);
    assert_eq!(report.origin, Origin::Timeout);
    assert_eq!(report.exit_code(), 124);
    assert!(report.cleanup_ok);
    assert!(start.elapsed() < Duration::from_secs(4));
    task.timeout = Duration::from_secs(5);
    let cancelled = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            thread::sleep(Duration::from_millis(50));
            cancelled.store(true, Ordering::Relaxed);
        });
        let report = process::run(&task, &cancelled);
        assert_eq!(report.origin, Origin::Cancelled);
        assert_eq!(report.exit_code(), 130);
        assert!(report.cleanup_ok);
    });
}

#[test]
fn scoped_descendants_die_on_timeout_and_leader_exit_with_inherited_pipes() {
    let temp = Temp::new();
    // Unrelated application remains alive; cleanup cannot use executable-name kills.
    let unrelated_path = temp.0.join("unrelated");
    let mut unrelated = Unrelated(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "fixture", "--ignored", "--nocapture"])
            .env("YT083_FIXTURE", "heartbeat")
            .env("YT083_HEARTBEAT", &unrelated_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    for mode in ["descendant", "exit-with-descendant", "fail-with-descendant"] {
        let address = temp.0.join(mode);
        let mut task = spec(mode, temp.0.clone());
        task.env.push((
            "YT083_HEARTBEAT".into(),
            Some(address.clone().into_os_string()),
        ));
        task.timeout = Duration::from_secs(2);
        let start = Instant::now();
        let report = run(&task);
        if mode == "descendant" {
            assert_eq!(report.origin, Origin::Timeout);
        } else if mode == "fail-with-descendant" {
            assert_eq!(report.origin, Origin::Child);
            assert_eq!(report.native_code(), Some(37));
        } else {
            assert!(report.success(), "{report:?}");
        }
        assert!(report.cleanup_ok, "{report:?}");
        assert!(start.elapsed() < Duration::from_secs(6));
        let stopped = std::fs::metadata(&address).unwrap().len();
        let live = std::fs::metadata(&unrelated_path).unwrap().len();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(std::fs::metadata(address).unwrap().len(), stopped);
        assert!(std::fs::metadata(&unrelated_path).unwrap().len() > live);
        assert!(unrelated.0.try_wait().unwrap().is_none());
    }
}

#[test]
fn unknown_commands_flags_and_capabilities_are_usage_errors() {
    assert_eq!(parse(Vec::new()).unwrap(), Cli::Help);
    for args in [
        vec!["build"],
        vec!["--json"],
        vec!["doctor", "--wat"],
        vec!["doctor", "--capability"],
        vec!["doctor", "--capability", "browser"],
        vec!["doctor", "--json", "--json"],
    ] {
        assert!(parse(args.into_iter().map(OsString::from)).is_err());
    }
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .arg("build")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

#[test]
fn doctor_negative_versions_and_absent_tools_are_not_success() {
    let temp = Temp::new();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["doctor", "--json"])
        .env("PATH", &temp.0)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(125));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "unavailable");
    assert_eq!(report["checks"][1]["origin"], "spawn");
    // Installed executables with the wrong identity/version protocol also fail closed.
    for name in ["rustc", "cargo"] {
        let name = if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_owned()
        };
        std::fs::copy(std::env::current_exe().unwrap(), temp.0.join(name)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["doctor", "--json"])
        .env("PATH", &temp.0)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "unavailable");
    assert_eq!(report["checks"][1]["origin"], "child");
}
