use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn app(root: &std::path::Path) -> App {
    crate::platform::fs::create_project(
        root,
        "app",
        &[
            (
                "Cargo.toml",
                "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n".into(),
            ),
            (
                "Cargo.lock",
                "version=4\n[[package]]\nname='fixture'\nversion='0.1.0'\n".into(),
            ),
            ("src/main.rs", "fn main() {}\n".into()),
        ],
    )
    .unwrap();
    crate::platform::fs::open_app(&root.join("app/Cargo.toml")).unwrap()
}
fn private_executable(app: &App) -> PathBuf {
    let executable = app.cwd.join("native fixture 雪 😀.exe");
    // Copy bytes into a new file inheriting the app's private DACL, rather than
    // retaining security metadata from the runner's group-writable test binary.
    let mut source = fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let mut destination = fs::File::create_new(&executable).unwrap();
    io::copy(&mut source, &mut destination).unwrap();
    executable
}
fn fixture(executable: &Path, mode: &str) -> Command {
    assert!(executable.is_absolute());
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "platform::process::windows::tests::native_fixture",
            "--nocapture",
        ])
        .env("YT088C_FIXTURE", mode);
    command
}
fn wait_file(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "fixture did not publish {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn frozen_build_and_dev_commands_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let app = app(root.path());
    for release in [false, true] {
        let command = crate::platform::process::command(&app, false, release).unwrap();
        let expected = if release {
            vec![
                "build",
                "--frozen",
                "--manifest-path",
                "Cargo.toml",
                "--release",
            ]
        } else {
            vec!["build", "--frozen", "--manifest-path", "Cargo.toml"]
        };
        assert_eq!(command.get_program(), "cargo");
        assert_eq!(command.get_args().collect::<Vec<_>>(), expected);
        assert_eq!(command.get_current_dir(), Some(app.cwd.as_path()));
        let env = command
            .get_envs()
            .collect::<std::collections::BTreeMap<_, _>>();
        for (key, value) in [
            ("RUSTUP_AUTO_INSTALL", "0"),
            ("RUSTUP_SKIP_UPDATE_CHECK", "1"),
            ("CARGO_NET_OFFLINE", "true"),
        ] {
            assert_eq!(env.get(OsStr::new(key)), Some(&Some(OsStr::new(value))));
        }
    }
    let dev = crate::platform::process::command(&app, true, false).unwrap();
    assert_eq!(
        dev.get_args().collect::<Vec<_>>(),
        ["run", "--frozen", "--manifest-path", "Cargo.toml"]
    );
    assert_eq!(dev.get_current_dir(), Some(app.cwd.as_path()));
    assert_eq!(
        dev.get_envs().collect::<std::collections::BTreeMap<_, _>>(),
        crate::app::command(true, false).get_envs().collect()
    );
}

#[test]
fn owned_native_child_preserves_exit_bits_unicode_argv_cwd_and_environment() {
    let root = tempfile::Builder::new()
        .prefix("yt088c 雪 😀 ")
        .tempdir()
        .unwrap();
    let app = app(root.path());
    let executable = private_executable(&app);
    for code in [23_u32, 0xc0000005] {
        let mut command = fixture(&executable, "exit");
        command
            .env("YT088C_EXIT", code.to_string())
            .env("YT088C_CWD", &app.cwd)
            .env("YT088C_ARG", "空 間;\"literal\\")
            .args(["--skip", "空 間;\"literal\\"])
            .env("CARGO_NET_OFFLINE", "true")
            .env("RUSTUP_AUTO_INSTALL", "0");
        let Completion::Exited(status) = run(&app, &mut command).unwrap() else {
            panic!("expected native status")
        };
        assert_eq!(status.code().unwrap() as u32, code);
    }
}

#[test]
fn job_reaps_descendant_and_leaves_unrelated_native_child_running() {
    let root = tempfile::tempdir().unwrap();
    let app = app(root.path());
    let executable = private_executable(&app);
    let mut sentinel = fixture(&executable, "sleep").spawn().unwrap();
    let pid_file = app.cwd.join("descendant-pid");
    let mut command = fixture(&executable, "descendant");
    command.env("YT088C_PID", &pid_file);
    let result = run(&app, &mut command);
    let sentinel_alive = sentinel.try_wait().unwrap().is_none();
    sentinel.kill().unwrap();
    sentinel.wait().unwrap();
    let Completion::Exited(status) = result.unwrap() else {
        panic!("expected exit")
    };
    assert_eq!(status.code(), Some(23));
    assert!(sentinel_alive);
    let pid: u32 = fs::read_to_string(pid_file).unwrap().parse().unwrap();
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if raw.is_null() {
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(87)); // already gone
    } else {
        let process = unsafe { OwnedHandle::from_raw_handle(raw) };
        assert_eq!(
            unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
    }
}

#[test]
fn pinned_app_parent_and_files_cannot_be_replaced_during_native_execution() {
    let root = tempfile::tempdir().unwrap();
    let app = app(root.path());
    let executable = private_executable(&app);
    let marker = app.cwd.join("running");
    let release = app.cwd.join("release");
    let mut command = fixture(&executable, "held");
    command
        .env("YT088C_RUNNING", &marker)
        .env("YT088C_RELEASE", &release);
    std::thread::scope(|scope| {
        let child = scope.spawn(|| run(&app, &mut command));
        wait_file(&marker);
        let rename_parent = fs::rename(root.path(), root.path().with_extension("moved"));
        let rename_app = fs::rename(&app.cwd, root.path().join("substitute"));
        let replace_lock = fs::write(app.cwd.join("Cargo.lock"), "replacement");
        fs::write(&release, "done").unwrap();
        assert!(child.join().unwrap().is_ok());
        assert!(rename_parent.is_err());
        assert!(rename_app.is_err());
        assert!(replace_lock.is_err());
    });
}

#[test]
fn native_fixture() {
    let Ok(mode) = std::env::var("YT088C_FIXTURE") else {
        return;
    };
    match mode.as_str() {
        "dev-console" => {
            wait_file(&PathBuf::from(std::env::var_os("YT088D_GO").unwrap()));
            let app = crate::platform::fs::open_app(&PathBuf::from(
                std::env::var_os("YT088D_MANIFEST").unwrap(),
            ))
            .unwrap();
            assert!(
                unsafe { GetConsoleProcessList(&mut 0, 1) } > 0,
                "real console required"
            );
            assert_ne!(
                unsafe { SetConsoleCtrlHandler(Some(previous_control), 1) },
                0
            );
            for code in [23_u32, 0xc0000005] {
                let mut command = fixture(&std::env::current_exe().unwrap(), "exit");
                command
                    .env("YT088C_EXIT", code.to_string())
                    .env("YT088C_CWD", &app.cwd)
                    .env("YT088C_ARG", "literal")
                    .args(["--skip", "literal"])
                    .env("CARGO_NET_OFFLINE", "true")
                    .env("RUSTUP_AUTO_INSTALL", "0");
                let Completion::Exited(status) = run_owned(&app, &mut command, true).unwrap()
                else {
                    panic!("native status required")
                };
                assert_eq!(status.code().unwrap() as u32, code);
                assert_previous_restored();
            }
            // Real C and BREAK, including graceful exit 23, before escalation.
            for event in [CTRL_C_EVENT, CTRL_BREAK_EVENT] {
                PREVIOUS.store(0, AtomicOrdering::SeqCst);
                let marker = app.cwd.join(format!("grace-{event}"));
                let mut command = fixture(&std::env::current_exe().unwrap(), "grace");
                command.env("YT088D_READY", &marker);
                let sender = std::thread::spawn(move || {
                    wait_file(&marker);
                    // Only this fixture's new private console is in scope.
                    assert_ne!(unsafe { GenerateConsoleCtrlEvent(event, 0) }, 0);
                });
                let result = run_owned(&app, &mut command, true);
                sender.join().unwrap();
                let Completion::Exited(status) = result.unwrap() else {
                    panic!("native exit required")
                };
                assert_eq!(status.code(), Some(23));
                assert_eq!(PREVIOUS.load(AtomicOrdering::SeqCst), 0);
                assert_previous_restored();
            }
            // A resistant Cargo/app group must not make dev wait forever.
            // The separate sentinel was created outside the inner owned Job
            // and must survive its bounded termination.
            {
                let leaf_ready = app.cwd.join("resistant-leaf-ready");
                let leaf_pid = app.cwd.join("resistant-leaf-pid");
                let sentinel_ready = app.cwd.join("unrelated-sentinel-ready");
                let mut sentinel = fixture(&std::env::current_exe().unwrap(), "resistant-leaf")
                    .env("YT088D_LEAF_READY", &sentinel_ready)
                    .spawn()
                    .unwrap();
                wait_file(&sentinel_ready);
                let marker = app.cwd.join("resistant-ready");
                let mut command = fixture(&std::env::current_exe().unwrap(), "resistant");
                command
                    .env("YT088D_READY", &marker)
                    .env("YT088D_LEAF_READY", &leaf_ready)
                    .env("YT088D_LEAF_PID", &leaf_pid);
                let sender = std::thread::spawn(move || {
                    wait_file(&marker);
                    // Safe only in the helper's CREATE_NEW_CONSOLE.
                    assert_ne!(unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) }, 0);
                });
                let began = Instant::now();
                let result = run_owned(&app, &mut command, true);
                sender.join().unwrap();
                assert!(
                    began.elapsed() < CLEANUP + GRACE + Duration::from_secs(8),
                    "resistant child must not block bounded cancellation"
                );
                let Completion::Exited(status) = result.unwrap() else {
                    panic!("resistant child must return a native status")
                };
                assert_eq!(status.code().unwrap() as u32, CONTROL_EXIT);
                assert!(
                    sentinel.try_wait().unwrap().is_none(),
                    "unrelated process must survive owned Job cleanup"
                );
                let pid: u32 = fs::read_to_string(&leaf_pid).unwrap().parse().unwrap();
                let raw = unsafe {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                        0,
                        pid,
                    )
                };
                if raw.is_null() {
                    assert_eq!(io::Error::last_os_error().raw_os_error(), Some(87));
                } else {
                    let descendant = unsafe { OwnedHandle::from_raw_handle(raw) };
                    assert_eq!(
                        unsafe { WaitForSingleObject(descendant.as_raw_handle(), 0) },
                        WAIT_OBJECT_0,
                        "resistant owned descendant must be reaped"
                    );
                }
                sentinel.kill().unwrap();
                sentinel.wait().unwrap();
                assert_previous_restored();
            }
            let mut missing = Command::new(app.cwd.join("missing-native.exe"));
            assert!(run_owned(&app, &mut missing, true).is_err());
            assert_previous_restored();
            assert_ne!(
                unsafe { SetConsoleCtrlHandler(Some(previous_control), 0) },
                0
            );
        }
        "grace" => {
            assert_ne!(
                unsafe { SetConsoleCtrlHandler(Some(previous_control), 1) },
                0
            );
            fs::write(std::env::var_os("YT088D_READY").unwrap(), "ready").unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while PREVIOUS.load(AtomicOrdering::SeqCst) == 0 {
                assert!(Instant::now() < deadline, "no real graceful console event");
                std::thread::sleep(POLL);
            }
            std::process::exit(23);
        }
        "resistant" => {
            assert_ne!(unsafe { SetConsoleCtrlHandler(Some(ignore_control), 1) }, 0);
            let leaf_ready = std::env::var_os("YT088D_LEAF_READY").unwrap();
            let mut leaf = fixture(&std::env::current_exe().unwrap(), "resistant-leaf")
                .env("YT088D_LEAF_READY", &leaf_ready)
                .spawn()
                .unwrap();
            let pid = leaf.id();
            // Own the direct-child wait handle. The private Job also owns the
            // leaf if the resistant parent is terminated before it can reap.
            let _reaper = std::thread::spawn(move || {
                let _ = leaf.wait();
            });
            fs::write(
                std::env::var_os("YT088D_LEAF_PID").unwrap(),
                pid.to_string(),
            )
            .unwrap();
            wait_file(&PathBuf::from(leaf_ready));
            fs::write(std::env::var_os("YT088D_READY").unwrap(), "ready").unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        "resistant-leaf" => {
            assert_ne!(unsafe { SetConsoleCtrlHandler(Some(ignore_control), 1) }, 0);
            fs::write(std::env::var_os("YT088D_LEAF_READY").unwrap(), "ready").unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        "sleep" => std::thread::sleep(Duration::from_secs(20)),
        "descendant" => {
            let child = fixture(&std::env::current_exe().unwrap(), "sleep")
                .spawn()
                .unwrap();
            fs::write(
                std::env::var_os("YT088C_PID").unwrap(),
                child.id().to_string(),
            )
            .unwrap();
            // The private Job, rather than this leader, must own/clean the child.
            std::process::exit(23);
        }
        "held" => {
            fs::write(std::env::var_os("YT088C_RUNNING").unwrap(), "ready").unwrap();
            wait_file(&PathBuf::from(std::env::var_os("YT088C_RELEASE").unwrap()));
        }
        "exit" => {
            assert_eq!(
                std::env::current_dir().unwrap(),
                PathBuf::from(std::env::var_os("YT088C_CWD").unwrap())
            );
            let arg = std::env::var_os("YT088C_ARG").unwrap();
            assert!(std::env::args_os().any(|actual| actual == arg));
            assert_eq!(std::env::var("CARGO_NET_OFFLINE").unwrap(), "true");
            assert_eq!(std::env::var("RUSTUP_AUTO_INSTALL").unwrap(), "0");
            let code: u32 = std::env::var("YT088C_EXIT").unwrap().parse().unwrap();
            std::process::exit(code as i32);
        }
        _ => panic!("unknown private fixture"),
    }
}

unsafe extern "system" fn ignore_control(event: u32) -> i32 {
    if matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        1
    } else {
        0
    }
}

static PREVIOUS: AtomicU32 = AtomicU32::new(0);
unsafe extern "system" fn previous_control(event: u32) -> i32 {
    PREVIOUS.store(event + 1, AtomicOrdering::SeqCst);
    1
}

fn assert_previous_restored() {
    assert!(!CONSOLE_OWNED.load(AtomicOrdering::SeqCst));
    CONTROL.store(0, AtomicOrdering::SeqCst);
    PREVIOUS.store(0, AtomicOrdering::SeqCst);
    // No product child remains in this otherwise empty task-owned console.
    assert_ne!(unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, 0) }, 0);
    let deadline = Instant::now() + CLEANUP;
    while PREVIOUS.load(AtomicOrdering::SeqCst) == 0 {
        assert!(
            Instant::now() < deadline,
            "previous console handler was not restored"
        );
        std::thread::sleep(POLL);
    }
    assert_eq!(CONTROL.load(AtomicOrdering::SeqCst), 0);
}

#[test]
fn dev_native_console_exit_identity_and_handler_restoration() {
    use std::os::windows::process::CommandExt;
    let root = tempfile::tempdir().unwrap();
    let app = app(root.path());
    let executable = private_executable(&app);
    let go = app.cwd.join("console-go");
    let job = Job::new().unwrap();
    let mut helper = fixture(&executable, "dev-console")
        .env("YT088D_GO", &go)
        .env("YT088D_MANIFEST", app.cwd.join("Cargo.toml"))
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .unwrap();
    // Helper waits for go before creating any descendant. RAII Job also owns
    // it on assertion failure; never leave a failed console fixture running.
    if unsafe { AssignProcessToJobObject(job.0.as_raw_handle(), helper.as_raw_handle()) } == 0 {
        let error = io::Error::last_os_error();
        helper.kill().unwrap();
        helper.wait().unwrap();
        panic!("cannot own real console helper: {error}");
    }
    fs::write(go, "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = helper.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "console helper timed out; real console proof incomplete"
        );
        std::thread::sleep(POLL);
    };
    job.cleanup(125).unwrap();
    assert!(
        status.success(),
        "real Windows console test failed: {status}"
    );
}
