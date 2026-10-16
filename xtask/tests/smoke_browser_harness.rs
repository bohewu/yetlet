//! Original test_smoke_browser_harness.py cases, using native owned children.
use std::{fs, process::Command, time::Duration};
use yetlet_xtask::{
    process::{Service, Spec},
    smoke::{profile, remove_tree_with},
};
#[test]
fn test_profile_argument_selects_debug_or_release() {
    assert_eq!(profile(&[], "debug").unwrap(), "debug");
    assert_eq!(
        profile(&["--profile".into(), "release".into()], "debug").unwrap(),
        "release"
    );
}
#[test]
fn test_owned_tree_retries_transient_not_empty_error() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("profile");
    fs::create_dir(&path).unwrap();
    fs::write(path.join("state"), "test").unwrap();
    let mut calls = 0;
    remove_tree_with(&path, Duration::from_secs(1), |p| {
        calls += 1;
        if calls == 1 {
            Err(std::io::ErrorKind::DirectoryNotEmpty.into())
        } else {
            fs::remove_dir_all(p)
        }
    })
    .unwrap();
    assert!(calls >= 2);
    assert!(!path.exists());
}
#[test]
fn subprocess_fixture() {
    let Ok(mode) = std::env::var("YT085_BROWSER_FIXTURE") else {
        return;
    };
    if mode == "tree" {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "subprocess_fixture", "--nocapture"])
            .env("YT085_BROWSER_FIXTURE", "leaf")
            .spawn()
            .unwrap();
        fs::write(
            std::env::var_os("YT085_PID_FILE").unwrap(),
            child.id().to_string(),
        )
        .unwrap();
        let _ = child.wait();
    } else {
        std::thread::sleep(Duration::from_secs(60));
    }
}
#[test]
fn test_owned_process_group_is_terminated_and_reaped() {
    let temp = tempfile::tempdir().unwrap();
    let pid_file = temp.path().join("pid");
    let spec = Spec {
        program: std::env::current_exe().unwrap().into(),
        args: vec![
            "--exact".into(),
            "subprocess_fixture".into(),
            "--nocapture".into(),
        ],
        cwd: temp.path().into(),
        env: vec![
            ("YT085_BROWSER_FIXTURE".into(), Some("tree".into())),
            ("YT085_PID_FILE".into(), Some(pid_file.clone().into())),
        ],
        timeout: Duration::from_secs(5),
        output_limit: 1024,
    };
    let mut service = Service::spawn(&spec, &temp.path().join("browser.log")).unwrap();
    let pid = service.id();
    let end = std::time::Instant::now() + Duration::from_secs(3);
    while !pid_file.exists() {
        assert!(std::time::Instant::now() < end);
        std::thread::sleep(Duration::from_millis(10));
    }
    let status = service.stop(false, Duration::from_secs(5)).unwrap();
    assert!(!status.success());
    assert!(service.stream_closed());
    assert!(service.exited().is_err() || service.exited().unwrap_or(true));
    #[cfg(unix)]
    {
        assert_eq!(unsafe { libc::kill(-(pid as i32), 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
    #[cfg(not(unix))]
    let _ = pid;
}
