//! Real local Linux child trees; fixtures execute without Cargo/npm/network work.
#![cfg(target_os = "linux")]
use std::{
    fs,
    os::unix::{fs::PermissionsExt, process::ExitStatusExt},
    process::{Child, Command, Output, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

static PORT: Mutex<()> = Mutex::new(());
const SCRIPT: &str = r#"#!/usr/bin/python3
import json, os, signal, socket, subprocess, sys, time
root=os.environ['YT_FRONTEND_TEST_ROOT']
role=os.path.basename(sys.argv[0])
mode=os.environ['YT_FRONTEND_TEST_MODE']
with open(root+'/trace', 'a') as out:
 out.write(json.dumps({'role':role, 'args':sys.argv[1:], 'cwd':os.getcwd(), 'secret':any(k.startswith('APP_') or k in ['YETLET_DEV_SECRETS','YETLET_CONFIG_ROOT'] for k in os.environ), 'public':os.environ.get('VITE_PUBLIC'), 'projectRoot':os.environ.get('YETLET_PROJECT_ROOT'), 'tlsCert':os.environ.get('YETLET_DEV_TLS_CERT'), 'tlsKey':os.environ.get('YETLET_DEV_TLS_KEY'), 'backendTls':os.environ.get('APP_SERVER__TLS__ENABLED'), 'backendHost':os.environ.get('APP_SERVER__HOST'), 'backendPort':os.environ.get('APP_SERVER__PORT'), 'browserWatch':os.environ.get('YETLET_BROWSER_WATCH')})+'\n')
if sys.argv[1:]==['run','build']:
 sys.exit(23 if mode=='build-fail' else 0)
if role=='cargo' and sys.argv[1]=='build': sys.exit(0)
child=subprocess.Popen(['/usr/bin/python3','-c','import signal,time; signal.signal(signal.SIGINT,signal.SIG_IGN); signal.signal(signal.SIGTERM,signal.SIG_IGN); time.sleep(120)'])
with open(root+'/'+role+'-leaf.pid','w') as out: out.write(str(child.pid))
with open(root+'/'+role+'.pid','w') as out: out.write(str(os.getpid()))
if role=='npm':
 server=socket.socket(); server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); server.bind(('127.0.0.1',5173)); server.listen()
if (mode=='frontend-fail' and role=='npm') or (mode=='rust-fail' and role=='cargo'):
 if role=='npm':
  while not os.path.exists(root+'/cargo.pid'): time.sleep(.005)
 time.sleep(.05); sys.exit(23)
if mode=='rust-signal' and role=='cargo':
 os.kill(os.getpid(),signal.SIGUSR1)
while True: time.sleep(.02)
"#;

struct Fixture {
    root: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("bin")).unwrap();
        fs::create_dir(root.path().join("frontend")).unwrap();
        fs::create_dir_all(root.path().join("frontend/dist/.vite")).unwrap();
        fs::write(root.path().join("frontend/package.json"), "{}").unwrap();
        fs::write(root.path().join("frontend/dist/.vite/manifest.json"), "{}").unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            root.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(root.path().join("base.toml"), "[server]\nhost='127.0.0.1'\nport=3000\n[cli.vite]\nenabled=true\nroot='frontend/'\nauto_start=true\nauto_build=true\n[cli.generate.k8s]\nreplicas=1\n").unwrap();
        for name in ["development.toml", "staging.toml", "production.toml"] {
            fs::write(root.path().join(name), "").unwrap();
        }
        for name in ["npm", "cargo"] {
            let path = root.path().join("bin").join(name);
            fs::write(&path, SCRIPT).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        // Fixtures satisfy the same protected-root contract as real apps,
        // independently of the test runner's umask.
        for directory in [
            "",
            "bin",
            "frontend",
            "frontend/dist",
            "frontend/dist/.vite",
        ] {
            fs::set_permissions(
                root.path().join(directory),
                // npm/Vite may inherit umask 002. Descendants remain protected
                // by the private frontend root, as with files in private apps.
                fs::Permissions::from_mode(if directory.starts_with("frontend/dist") {
                    0o775
                } else {
                    0o700
                }),
            )
            .unwrap();
        }
        Self { root }
    }
    fn command(&self, action: &str, mode: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("APP_") || key == "YETLET_CONFIG_ROOT" {
                command.env_remove(key);
            }
        }
        command
            .args([action, "--manifest-path"])
            .arg(self.root.path().join("Cargo.toml"))
            .env("PATH", self.root.path().join("bin"))
            .env("YT_FRONTEND_TEST_ROOT", self.root.path())
            .env("YT_FRONTEND_TEST_MODE", mode)
            .env("APP_ENVIRONMENT", "Production")
            .env("APP_BACKEND_CREDENTIAL", "fixture-private-marker")
            .env("YETLET_DEV_SECRETS", "fixture-private-selector")
            .env("VITE_PUBLIC", "fixture-public")
            .env("YETLET_PROJECT_ROOT", "/ignored/user/context")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
    fn trace(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.root.path().join("trace"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn ready(&self, name: &str) {
        let deadline = Instant::now() + Duration::from_secs(6);
        while !self.root.path().join(name).exists() {
            assert!(Instant::now() < deadline, "fixture readiness: {name}");
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn absent(&self) {
        for name in ["npm.pid", "npm-leaf.pid", "cargo.pid", "cargo-leaf.pid"] {
            let pid: i32 = fs::read_to_string(self.root.path().join(name))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "owned {name} survived ({pid})"
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
}
struct Running(Option<Child>);
impl Running {
    fn start(command: &mut Command) -> Self {
        Self(Some(command.spawn().unwrap()))
    }
    fn wait(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(8);
        while self.0.as_mut().unwrap().try_wait().unwrap().is_none() {
            assert!(
                Instant::now() < deadline,
                "supervisor failed to exit within cleanup bound"
            );
            thread::sleep(Duration::from_millis(5));
        }
        self.0.take().unwrap().wait_with_output().unwrap()
    }
    fn signal(&self, signal: i32) {
        assert_eq!(
            unsafe { libc::kill(self.0.as_ref().unwrap().id() as i32, signal) },
            0
        );
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(5);
            while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

#[test]
fn frontend_build_precedes_rust_and_filters_backend_environment() {
    let fixture = Fixture::new();
    let output = Running::start(&mut fixture.command("build", "build-ok")).wait();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = fixture.trace();
    assert_eq!(trace.len(), 2);
    assert_eq!(trace[0]["role"], "npm");
    assert_eq!(trace[0]["args"], serde_json::json!(["run", "build"]));
    assert_eq!(
        trace[0]["cwd"],
        fixture.root.path().join("frontend").to_str().unwrap()
    );
    assert_eq!(trace[0]["secret"], false);
    assert_eq!(trace[0]["public"], "fixture-public");
    assert_eq!(
        trace[0]["projectRoot"],
        fixture.root.path().to_str().unwrap()
    );
    assert_eq!(trace[1]["role"], "cargo");
    assert_eq!(
        trace[1]["args"],
        serde_json::json!(["build", "--frozen", "--manifest-path", "Cargo.toml"])
    );
    assert_eq!(trace[1]["secret"], true);
}

fn browser_fixture() -> Fixture {
    let fixture = Fixture::new();
    let base = fixture.root.path().join("base.toml");
    fs::write(
        &base,
        format!(
            "[cli]\ntarget='browser'\n{}",
            fs::read_to_string(&base).unwrap()
        ),
    )
    .unwrap();
    fs::write(
        fixture.root.path().join("frontend/dist/index.html"),
        "<!doctype html><title>Tasks</title>",
    )
    .unwrap();
    fixture
}

#[test]
fn browser_build_has_no_native_child_preserves_failure_and_honors_skip() {
    for (mode, expected) in [("build-ok", 0), ("build-fail", 23)] {
        let fixture = browser_fixture();
        let output = Running::start(&mut fixture.command("build", mode)).wait();
        assert_eq!(output.status.code(), Some(expected));
        let trace = fixture.trace();
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0]["role"], "npm");
        assert_eq!(trace[0]["secret"], false);
    }
    for (action, flag) in [
        ("build", "APP_CLI__VITE__AUTO_BUILD"),
        ("run", "APP_CLI__VITE__AUTO_START"),
        ("watch", "APP_CLI__VITE__AUTO_START"),
        ("watch", "APP_CLI__VITE__AUTO_BUILD"),
    ] {
        let fixture = browser_fixture();
        let output = fixture
            .command(action, "build-ok")
            .env(flag, "false")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(fixture.trace().is_empty());
    }
}

#[test]
fn browser_run_owns_only_frontend_and_reaps_it_on_cancellation() {
    let _port = PORT.lock().unwrap();
    for action in ["run", "watch"] {
        let fixture = browser_fixture();
        if action == "watch" {
            fs::remove_dir_all(fixture.root.path().join("frontend/dist")).unwrap();
        }
        let running = Running::start(
            fixture
                .command(action, "build-ok")
                .env("YETLET_BROWSER_WATCH", "untrusted"),
        );
        fixture.ready("npm-leaf.pid");
        assert_eq!(fixture.trace().len(), 1);
        assert_eq!(
            fixture.trace()[0]["browserWatch"],
            if action == "watch" {
                serde_json::json!("1")
            } else {
                serde_json::Value::Null
            }
        );
        assert_eq!(fixture.trace()[0]["secret"], false);
        assert!(!fixture.root.path().join("cargo.pid").exists());
        running.signal(libc::SIGINT);
        let output = running.wait();
        assert_eq!(output.status.signal(), Some(libc::SIGINT));
        for name in ["npm.pid", "npm-leaf.pid"] {
            let pid = fs::read_to_string(fixture.root.path().join(name))
                .unwrap()
                .parse::<i32>()
                .unwrap();
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "owned child survived");
        }
    }
}
#[test]
fn browser_run_requires_static_output_and_respects_frontend_root() {
    let fixture = browser_fixture();
    fs::remove_file(fixture.root.path().join("frontend/dist/index.html")).unwrap();
    let output = fixture.command("run", "build-ok").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(fixture.trace().is_empty());
    let base = fixture.root.path().join("base.toml");
    fs::write(
        &base,
        fs::read_to_string(&base)
            .unwrap()
            .replace("frontend/", "web/"),
    )
    .unwrap();
    fs::rename(
        fixture.root.path().join("frontend"),
        fixture.root.path().join("web"),
    )
    .unwrap();
    let output = fixture.command("build", "build-ok").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        fixture.trace()[0]["cwd"],
        fixture.root.path().join("web").to_str().unwrap()
    );
}
#[test]
fn frontend_build_failure_preserves_exit_without_starting_rust_or_guessing_cause() {
    let fixture = Fixture::new();
    let output = Running::start(&mut fixture.command("build", "build-fail")).wait();
    assert_eq!(output.status.code(), Some(23));
    assert_eq!(fixture.trace().len(), 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("CLI_CHILD_EXIT") && stderr.contains("frontend"),
        "{stderr}"
    );
    assert!(!stderr.contains("cached") && !stderr.contains("lock generation"));
    assert!(!stderr.contains("fixture-private"));
}
#[test]
fn either_child_failure_and_native_signal_stop_and_reap_both_owned_trees() {
    let _port = PORT.lock().unwrap();
    for mode in ["frontend-fail", "rust-fail", "rust-signal"] {
        let fixture = Fixture::new();
        let output = Running::start(&mut fixture.command("run", mode)).wait();
        if mode == "rust-signal" {
            assert_eq!(output.status.signal(), Some(libc::SIGUSR1));
        } else {
            assert_eq!(
                output.status.code(),
                Some(23),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        fixture.absent();
        let trace = fixture.trace();
        assert_eq!(trace[0]["role"], "npm");
        assert_eq!(
            trace[0]["args"],
            serde_json::json!(["run", "dev", "--", "--strictPort"])
        );
        assert_eq!(trace[0]["secret"], false);
        assert_eq!(trace[1]["role"], "cargo");
        assert_eq!(
            trace[1]["args"],
            serde_json::json!(["run", "--frozen", "--manifest-path", "Cargo.toml"])
        );
    }
}
#[test]
fn cancellation_preserves_sigint_and_sigterm_and_cleans_both_trees() {
    let _port = PORT.lock().unwrap();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let fixture = Fixture::new();
        let running = Running::start(&mut fixture.command("run", "wait"));
        fixture.ready("cargo.pid");
        running.signal(signal);
        let output = running.wait();
        assert_eq!(
            output.status.signal(),
            Some(signal),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fixture.absent();
    }
}
#[test]
fn occupied_frontend_port_refuses_before_starting_either_tree() {
    let _port = PORT.lock().unwrap();
    let _listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 5173)).unwrap();
    let fixture = Fixture::new();
    let output = Running::start(&mut fixture.command("run", "wait")).wait();
    assert_eq!(
        output.status.code(),
        Some(125),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.trace().is_empty());
}

#[test]
fn tty_session_fixture() {
    if std::env::var_os("YT_FRONTEND_TTY_SESSION").is_none() {
        return;
    }
    let root = std::path::PathBuf::from(std::env::var_os("YT_FRONTEND_TEST_ROOT").unwrap());
    let original = unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) };
    assert_eq!(original, unsafe { libc::getpgrp() });
    let mut child = Command::new(env!("CARGO_BIN_EXE_yetlet"))
        .args(["run", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        let group = fs::read_to_string(root.join("cargo.pid"))
            .ok()
            .and_then(|value| value.parse::<i32>().ok());
        if group.is_some_and(|group| unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) } == group) {
            break;
        }
        assert!(Instant::now() < deadline, "no Rust foreground handoff");
        thread::sleep(Duration::from_millis(5));
    }
    fs::write(root.join("terminal-handed-off"), "verified").unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGINT));
    assert_eq!(unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) }, original);
    fs::write(root.join("terminal-restored"), "verified").unwrap();
}

#[test]
fn terminal_ctrl_c_cleans_both_trees_and_restores_foreground_group() {
    use std::{
        io::Write,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::process::CommandExt,
        },
    };
    let _port = PORT.lock().unwrap();
    let fixture = Fixture::new();
    let (mut master, mut slave) = (-1, -1);
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
            )
        },
        0
    );
    let mut master = unsafe { fs::File::from_raw_fd(master) };
    let slave = unsafe { fs::File::from_raw_fd(slave) };
    let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
    assert_eq!(
        unsafe { libc::tcgetattr(slave.as_raw_fd(), &mut termios) },
        0
    );
    termios.c_lflag |= libc::ISIG | libc::ICANON;
    termios.c_cc[libc::VINTR] = 3;
    assert_eq!(
        unsafe { libc::tcsetattr(slave.as_raw_fd(), libc::TCSANOW, &termios) },
        0
    );
    let cli = fixture.command("run", "wait");
    let mut helper = Command::new(std::env::current_exe().unwrap());
    helper
        .args(["--exact", "tty_session_fixture", "--nocapture"])
        .envs(
            cli.get_envs()
                .filter_map(|(key, value)| value.map(|value| (key, value))),
        )
        .env("YT_FRONTEND_TTY_SESSION", "1")
        .stdin(slave.try_clone().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    unsafe {
        helper.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(libc::STDIN_FILENO, libc::TIOCSCTTY, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let running = Running::start(&mut helper);
    fixture.ready("terminal-handed-off");
    master.write_all(&[termios.c_cc[libc::VINTR]]).unwrap();
    let output = running.wait();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.root.path().join("terminal-restored").exists());
    fixture.absent();
}

#[test]
fn execution_switches_are_independent_and_disabled_frontend_stays_skipped() {
    for key in ["APP_CLI__VITE__AUTO_BUILD", "APP_CLI__VITE__ENABLED"] {
        let fixture = Fixture::new();
        let mut command = fixture.command("build", "build-ok");
        command.env(key, "false");
        let output = Running::start(&mut command).wait();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let trace = fixture.trace();
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0]["role"], "cargo");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Frontend build skipped"));
    }
    // auto_build remains true, but run consults auto_start alone.
    let fixture = Fixture::new();
    let mut command = fixture.command("run", "rust-fail");
    command.env("APP_CLI__VITE__AUTO_START", "false");
    let output = Running::start(&mut command).wait();
    assert_eq!(output.status.code(), Some(23));
    let trace = fixture.trace();
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0]["role"], "cargo");
    for file in ["cargo.pid", "cargo-leaf.pid"] {
        let pid = fs::read_to_string(fixture.root.path().join(file))
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
    // auto_start=false must not suppress the independently selected build.
    let fixture = Fixture::new();
    let mut command = fixture.command("build", "build-ok");
    command.env("APP_CLI__VITE__AUTO_START", "false");
    let output = Running::start(&mut command).wait();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture
            .trace()
            .iter()
            .map(|row| row["role"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["npm", "cargo"]
    );
}

#[test]
fn watch_forces_rust_watch_after_settings_validation_without_changing_run() {
    for rust in ["", "[cli.rust]\nwatch=false\n"] {
        let fixture = Fixture::new();
        let base_path = fixture.root.path().join("base.toml");
        let base = format!("{}{rust}", fs::read_to_string(&base_path).unwrap());
        fs::write(&base_path, &base).unwrap();
        let watcher = fixture.root.path().join("bin/watchexec");
        fs::write(
            &watcher,
            r#"#!/usr/bin/python3
import os, time
root=os.environ['YT_FRONTEND_TEST_ROOT']
with open(root+'/watcher-started', 'w') as out: out.write(str(os.getpid()))
while True: time.sleep(.02)
"#,
        )
        .unwrap();
        fs::set_permissions(&watcher, fs::Permissions::from_mode(0o700)).unwrap();
        for action in ["run", "watch"] {
            let mut command = fixture.command(action, "rust-fail");
            command.env("APP_CLI__VITE__AUTO_START", "false");
            if !rust.is_empty() {
                command.env("APP_CLI__RUST__WATCH", "false");
            }
            let running = Running::start(&mut command);
            let watcher_pid = if action == "watch" {
                fixture.ready("cargo.pid");
                let deadline = Instant::now() + Duration::from_secs(6);
                let pid = loop {
                    if let Some(pid) =
                        fs::read_to_string(fixture.root.path().join("watcher-started"))
                            .ok()
                            .and_then(|value| value.parse::<i32>().ok())
                            .filter(|pid| *pid > 0)
                    {
                        break pid;
                    }
                    assert!(Instant::now() < deadline, "watcher PID readiness");
                    thread::sleep(Duration::from_millis(5));
                };
                running.signal(libc::SIGINT);
                Some(pid)
            } else {
                None
            };
            let output = running.wait();
            if action == "run" {
                assert_eq!(output.status.code(), Some(23));
            } else {
                assert_eq!(
                    output.status.signal(),
                    Some(libc::SIGINT),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let pid = watcher_pid.unwrap();
                assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "owned watcher survived");
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
            }
            assert_eq!(
                fixture.root.path().join("watcher-started").exists(),
                action == "watch"
            );
            assert!(!fixture.root.path().join("npm.pid").exists());
            assert_eq!(fs::read_to_string(&base_path).unwrap(), base);
            for file in ["cargo.pid", "cargo-leaf.pid"] {
                let pid = fs::read_to_string(fixture.root.path().join(file))
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();
                assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "owned {file} survived");
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
                fs::remove_file(fixture.root.path().join(file)).unwrap();
            }
        }
        assert_eq!(fixture.trace().len(), 2);
        assert!(fixture.trace().iter().all(|row| row["role"] == "cargo"));
    }
    // A command override must not hide invalid declared settings or environment values.
    for (rust, override_value) in [
        ("[cli.rust]\nwatch='invalid'\n", None),
        ("[cli.rust]\nunknown=true\n", None),
        ("[cli.rust]\nwatch=false\n", Some("invalid")),
    ] {
        let fixture = Fixture::new();
        let base_path = fixture.root.path().join("base.toml");
        let base = format!("{}{rust}", fs::read_to_string(&base_path).unwrap());
        fs::write(&base_path, &base).unwrap();
        let mut command = fixture.command("watch", "rust-fail");
        command.env("APP_CLI__VITE__AUTO_START", "false");
        if let Some(value) = override_value {
            command.env("APP_CLI__RUST__WATCH", value);
        }
        let output = Running::start(&mut command).wait();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("CLI_PROJECT settings values:"));
        assert!(fixture.trace().is_empty());
        assert_eq!(fs::read_to_string(base_path).unwrap(), base);
    }
}

#[test]
fn settings_hints_keep_exit_stderr_redaction_and_preflight_without_children() {
    for action in ["run", "watch", "build"] {
        for case in ["root", "missing", "profile", "toml", "key", "type"] {
            let fixture = Fixture::new();
            let mut command = fixture.command(action, "build-ok");
            let (group, hint) = match case {
                "root" | "missing" => (
                    "source",
                    "check the public settings directory and required base/profile files for safe ownership and readability",
                ),
                "profile" => (
                    "profile",
                    "select exactly Development, Staging or Production; the default is Production",
                ),
                _ => (
                    "values",
                    "check public TOML syntax, base-declared keys/types, ranges and target/TLS requirements",
                ),
            };
            match case {
                "root" => {
                    command.env("YETLET_CONFIG_ROOT", "../SYNTHETIC_SETTINGS_PATH");
                }
                "missing" => {
                    fs::remove_file(fixture.root.path().join("production.toml")).unwrap();
                }
                "profile" => {
                    command.env("APP_ENVIRONMENT", "production");
                }
                "toml" => {
                    fs::write(
                        fixture.root.path().join("base.toml"),
                        "SYNTHETIC_SETTINGS_TOML = [",
                    )
                    .unwrap();
                }
                "key" => {
                    command.env(
                        "APP_CLI__SYNTHETIC_SETTINGS_KEY",
                        "SYNTHETIC_SETTINGS_VALUE",
                    );
                }
                "type" => {
                    command.env("APP_CLI__VITE__AUTO_BUILD", "SYNTHETIC_SETTINGS_VALUE");
                }
                _ => unreachable!(),
            }
            let base = fs::read(fixture.root.path().join("base.toml")).unwrap();
            let output = command.output().unwrap();
            assert_eq!(output.status.code(), Some(1), "{action}/{case}");
            assert!(output.stdout.is_empty());
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!("CLI_PROJECT {action}: CLI_PROJECT settings {group}: {hint}\n"),
                "{action}/{case}"
            );
            assert!(fixture.trace().is_empty());
            assert!(!fixture.root.path().join("target").exists());
            assert_eq!(
                fs::read(fixture.root.path().join("base.toml")).unwrap(),
                base
            );
        }
    }
}

#[test]
fn frontend_dotenv_preflight_refuses_unsafe_files_without_children_and_accepts_local_files() {
    for selected in ["frontend", "frontend 雪 with spaces"] {
        let fixture = Fixture::new();
        let frontend = fixture.root.path().join(selected);
        if selected != "frontend" {
            fs::rename(fixture.root.path().join("frontend"), &frontend).unwrap();
        }
        let outside = fixture.root.path().join(".env");
        fs::write(&outside, "VITE_PUBLIC=outside-marker\n").unwrap();
        let dotenv = frontend.join(".env");
        for action in ["run", "watch", "build"] {
            std::os::unix::fs::symlink(&outside, &dotenv).unwrap();
            let output = fixture
                .command(action, "build-ok")
                .env("APP_CLI__VITE__ROOT", selected)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("frontend dotenv must be a regular")
            );
            assert!(fixture.trace().is_empty());
            fs::remove_file(&dotenv).unwrap();
        }
        fs::create_dir(&dotenv).unwrap();
        let output = fixture
            .command("build", "build-ok")
            .env("APP_CLI__VITE__ROOT", selected)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(fixture.trace().is_empty());
        fs::remove_dir(&dotenv).unwrap();
        fs::write(&dotenv, "VITE_PUBLIC=frontend-local\n").unwrap();
        let output = fixture
            .command("build", "build-ok")
            .env("APP_CLI__VITE__ROOT", selected)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fixture.trace().len(), 2);
        assert!(!fixture.root.path().join("npm.pid").exists());
        assert!(!fixture.root.path().join("cargo.pid").exists());
    }
}

#[test]
fn configured_frontend_root_with_spaces_and_unicode_is_literal_and_filters_backend_env() {
    let fixture = Fixture::new();
    let selected = "frontend 雪 with spaces";
    fs::rename(
        fixture.root.path().join("frontend"),
        fixture.root.path().join(selected),
    )
    .unwrap();
    let mut command = fixture.command("build", "build-ok");
    command.env("APP_CLI__VITE__ROOT", selected);
    let output = Running::start(&mut command).wait();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = fixture.trace();
    assert_eq!(
        trace[0]["cwd"],
        fixture.root.path().join(selected).to_str().unwrap()
    );
    assert_eq!(trace[0]["secret"], false);
    assert_eq!(trace[1]["cwd"], fixture.root.path().to_str().unwrap());
}

#[test]
fn development_tls_build_selects_feature_without_certificate_reads_and_filters_paths() {
    let fixture = Fixture::new();
    let root = fixture.root.path();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='fixture'\nversion='0.1.0'\n[workspace]\n[features]\nyetlet-dev-tls=[]\n",
    )
    .unwrap();
    let mut base = fs::read_to_string(root.join("base.toml")).unwrap();
    base.push_str("[server.tls]\nenabled=false\ncertificate='.dev/certs/absent.pem'\nprivate_key='.dev/certs/absent-key.pem'\n");
    fs::write(root.join("base.toml"), base).unwrap();
    let mut command = fixture.command("build", "build-ok");
    command
        .env("APP_ENVIRONMENT", "Development")
        .env("APP_SERVER__TLS__ENABLED", "true")
        .env("YETLET_DEV_TLS_CERT", "must-not-reach-frontend")
        .env("YETLET_DEV_TLS_KEY", "must-not-reach-frontend");
    let output = Running::start(&mut command).wait();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = fixture.trace();
    assert_eq!(trace.len(), 2);
    assert_eq!(trace[0]["tlsCert"], serde_json::Value::Null);
    assert_eq!(trace[0]["tlsKey"], serde_json::Value::Null);
    assert!(
        trace[1]["args"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "yetlet-dev-tls")
    );
    fs::remove_file(root.join("trace")).unwrap();
    let mut command = fixture.command("run", "build-ok");
    command
        .env("APP_ENVIRONMENT", "Development")
        .env("APP_SERVER__TLS__ENABLED", "true");
    let output = Running::start(&mut command).wait();
    assert!(!output.status.success());
    assert!(
        fixture.trace().is_empty(),
        "missing keys must refuse before any child"
    );
}
