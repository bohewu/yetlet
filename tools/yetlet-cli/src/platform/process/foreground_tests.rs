//! Native subprocess fixtures directly exercise the changed ownership boundary.
use super::*;
use std::{fs, io::Write, path::Path, process::Stdio};

static OBSERVED: AtomicI32 = AtomicI32::new(0);
extern "C" fn observe(signal: i32) {
    if signal == libc::SIGINT {
        OBSERVED.store(signal, Ordering::Relaxed);
    }
}

fn fixture(mode: &str, root: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "platform::process::linux::foreground::tests::owned_fixture",
            "--nocapture",
        ])
        .env("YT087B_FIXTURE", mode)
        .env("YT087B_ROOT", root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn ready(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "fixture did not become ready: {path:?}"
        );
        thread::sleep(POLL);
    }
}

#[test]
#[allow(clippy::zombie_processes)] // the product supervisor must adopt/reap this fixture
fn owned_fixture() {
    let Ok(mode) = std::env::var("YT087B_FIXTURE") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("YT087B_ROOT").unwrap());
    if mode == "tty-supervise-resistant" {
        let original = unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) };
        assert_eq!(original, unsafe { libc::getpgrp() });
        let tid = unsafe { libc::gettid() };
        fs::write(root.join("supervisor.tid"), tid.to_string()).unwrap();
        let mut command = fixture("resistant", &root);
        command.stdin(Stdio::inherit());
        let result = run(&mut command).unwrap();
        assert!(matches!(result, Completion::Cancelled(libc::SIGINT)));
        assert_eq!(unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) }, original);
        let children = fs::read_to_string(format!("/proc/self/task/{tid}/children")).unwrap();
        assert!(
            children.trim().is_empty(),
            "unreaped owned child: {children}"
        );
        fs::write(root.join("tty-restored"), original.to_string()).unwrap();
        result.finish();
    }
    if let Some(mode) = mode.strip_prefix("supervise-") {
        let mut command = fixture(mode, &root);
        match run(&mut command) {
            Ok(result) => result.finish(),
            Err(_) => std::process::exit(125),
        }
    }
    match mode.as_str() {
        "exit" => std::process::exit(23),
        "signal" => unsafe {
            libc::raise(libc::SIGUSR1);
            std::process::exit(99);
        },
        "leaf" | "tree" | "orphan" | "resistant" | "resistant-leaf" => {
            unsafe {
                let handler = if mode.starts_with("resistant") {
                    libc::SIG_IGN
                } else {
                    observe as *const () as usize
                };
                libc::signal(libc::SIGINT, handler);
                libc::signal(libc::SIGTERM, handler);
            }
            if mode == "leaf" || mode == "resistant-leaf" {
                fs::write(root.join("leaf.pid"), std::process::id().to_string()).unwrap();
                loop {
                    if OBSERVED.load(Ordering::Relaxed) != 0 {
                        fs::write(root.join("leaf-int"), "SIGINT").unwrap();
                    }
                    thread::sleep(POLL);
                }
            }
            let leaf_mode = if mode == "resistant" {
                "resistant-leaf"
            } else {
                "leaf"
            };
            let _leaf = fixture(leaf_mode, &root).spawn().unwrap();
            ready(&root.join("leaf.pid"));
            if mode == "resistant" {
                let deadline = Instant::now() + Duration::from_secs(5);
                while unsafe { libc::tcgetpgrp(libc::STDIN_FILENO) != libc::getpgrp() } {
                    assert!(Instant::now() < deadline, "no foreground terminal handoff");
                    thread::sleep(POLL);
                }
            }
            fs::write(root.join("group.pid"), std::process::id().to_string()).unwrap();
            if mode == "orphan" {
                std::process::exit(17);
            }
            loop {
                if OBSERVED.load(Ordering::Relaxed) != 0 {
                    fs::write(root.join("parent-int"), "SIGINT").unwrap();
                    std::process::exit(0);
                }
                thread::sleep(POLL);
            }
        }
        _ => panic!("unknown fixture mode"),
    }
}

struct Probe(Child);
impl Probe {
    fn start(mode: &str, root: &Path) -> Self {
        Self(fixture(mode, root).process_group(0).spawn().unwrap())
    }
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(6);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "owned supervisor did not terminate"
            );
            thread::sleep(POLL);
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            unsafe {
                libc::kill(self.0.id() as i32, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if self.0.try_wait().ok().flatten().is_some() {
                    return;
                }
                thread::sleep(POLL);
            }
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn absent(root: &Path) {
    for file in ["group.pid", "leaf.pid"] {
        let pid = fs::read_to_string(root.join(file))
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "owned PID {pid} survived"
        );
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
    }
    let group = fs::read_to_string(root.join("group.pid"))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert_eq!(unsafe { libc::kill(-group, 0) }, -1);
    assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
}

#[test]
fn native_exit_and_signal_identity_survive_cleanup() {
    use std::os::unix::process::ExitStatusExt;
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        Probe::start("supervise-exit", root.path()).wait().code(),
        Some(23)
    );
    assert_eq!(
        Probe::start("supervise-signal", root.path())
            .wait()
            .signal(),
        Some(libc::SIGUSR1)
    );
}

#[test]
fn ctrl_c_reaches_owned_descendant_and_escalates_without_touching_other_group() {
    use std::os::unix::process::ExitStatusExt;
    let root = tempfile::tempdir().unwrap();
    let sentinel_root = tempfile::tempdir().unwrap();
    let mut sentinel = Probe::start("leaf", sentinel_root.path());
    ready(&sentinel_root.path().join("leaf.pid"));
    let mut supervisor = Probe::start("supervise-tree", root.path());
    ready(&root.path().join("group.pid"));
    assert_eq!(
        unsafe { libc::kill(supervisor.0.id() as i32, libc::SIGINT) },
        0
    );
    assert_eq!(supervisor.wait().signal(), Some(libc::SIGINT));
    assert_eq!(
        fs::read_to_string(root.path().join("parent-int")).unwrap(),
        "SIGINT"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("leaf-int")).unwrap(),
        "SIGINT"
    );
    absent(root.path());
    assert!(sentinel.0.try_wait().unwrap().is_none());
    assert!(!sentinel_root.path().join("leaf-int").exists());
    sentinel.0.kill().unwrap();
    sentinel.0.wait().unwrap();
}

#[test]
fn normal_leader_exit_still_kills_and_reaps_owned_orphan() {
    let root = tempfile::tempdir().unwrap();
    let mut supervisor = Probe::start("supervise-orphan", root.path());
    assert_eq!(supervisor.wait().code(), Some(17));
    absent(root.path());
}

#[test]
fn terminal_ctrl_c_cleans_resistant_group_and_restores_tty() {
    use std::os::unix::process::ExitStatusExt;
    let root = tempfile::tempdir().unwrap();
    let sentinel_root = tempfile::tempdir().unwrap();
    let mut sentinel = Probe::start("leaf", sentinel_root.path());
    ready(&sentinel_root.path().join("leaf.pid"));

    let (mut master_fd, mut slave_fd) = (-1, -1);
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
            )
        },
        0,
        "cannot allocate direct controlling-PTY fixture: {}",
        io::Error::last_os_error()
    );
    let mut master = unsafe { File::from_raw_fd(master_fd) };
    let slave = unsafe { File::from_raw_fd(slave_fd) };
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
    let mut command = fixture("tty-supervise-resistant", root.path());
    command.stdin(slave.try_clone().unwrap());
    // This fixture alone owns a fresh session and controlling terminal. Nothing
    // changes the runner's terminal, permissions, session or process group.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(libc::STDIN_FILENO, libc::TIOCSCTTY, 0) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut supervisor = Probe(command.spawn().unwrap());
    ready(&root.path().join("group.pid"));
    let group = fs::read_to_string(root.path().join("group.pid"))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let leaf = fs::read_to_string(root.path().join("leaf.pid"))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert_eq!(unsafe { libc::getpgid(leaf) }, group);
    assert_ne!(group, supervisor.0.id() as i32);
    let tid = fs::read_to_string(root.path().join("supervisor.tid")).unwrap();
    let children = fs::read_to_string(format!("/proc/{}/task/{tid}/children", supervisor.0.id()))
        .unwrap()
        .split_whitespace()
        .map(|pid| pid.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        children.len(),
        2,
        "expected Cargo fixture plus owned notifier"
    );
    for &pid in &children {
        assert_eq!(unsafe { libc::getpgid(pid) }, group);
    }
    let started = Instant::now();
    // The line discipline delivers SIGINT to the foreground group. No kill of
    // the CLI PID, test-only cancellation hook or raw-input polling is involved.
    master.write_all(&[termios.c_cc[libc::VINTR]]).unwrap();
    assert_eq!(supervisor.wait().signal(), Some(libc::SIGINT));
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(
        fs::read_to_string(root.path().join("tty-restored")).unwrap(),
        supervisor.0.id().to_string()
    );
    absent(root.path());
    for pid in children {
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "owned notifier/child {pid} survived"
        );
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
    }
    assert!(sentinel.0.try_wait().unwrap().is_none());
    assert!(!sentinel_root.path().join("leaf-int").exists());
    sentinel.0.kill().unwrap();
    sentinel.0.wait().unwrap();
}
