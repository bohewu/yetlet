//! Original failed-readiness cleanup cases: real native signal/escalation/reap.
use std::{
    fs,
    time::{Duration, Instant},
};
use yetlet_xtask::{
    process::{Service, Spec},
    tooling::{Result, fail},
};
#[test]
fn subprocess_fixture() {
    let Some(mode) = std::env::var_os("YT085_SHOP_FIXTURE") else {
        return;
    };
    #[cfg(unix)]
    if mode == "ignore" {
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
    }
    fs::write(std::env::var_os("YT085_READY_FILE").unwrap(), "ready").unwrap();
    std::thread::sleep(Duration::from_secs(60));
}
fn exercise_failure(ignore: bool) {
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let spec = Spec {
        program: std::env::current_exe().unwrap().into(),
        args: vec![
            "--exact".into(),
            "subprocess_fixture".into(),
            "--nocapture".into(),
        ],
        cwd: temp.path().into(),
        env: vec![
            (
                "YT085_SHOP_FIXTURE".into(),
                Some(if ignore { "ignore" } else { "normal" }.into()),
            ),
            ("YT085_READY_FILE".into(), Some(ready.clone().into())),
        ],
        timeout: Duration::from_secs(5),
        output_limit: 1024,
    };
    let mut service = Service::spawn(&spec, &temp.path().join("shop.log")).unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    while !ready.exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(10));
    }
    let readiness_error = fail("simulated readiness failure");
    let identity = readiness_error.message.as_ptr();
    let result: Result<()> = Err(readiness_error);
    let start = Instant::now();
    let result = service.readiness(result);
    let status = service.stop(false, Duration::from_secs(4)).unwrap();
    let error = result.unwrap_err();
    assert_eq!(error.message, "simulated readiness failure");
    assert_eq!(error.message.as_ptr(), identity);
    assert!(service.stream_closed());
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(service.escalated, ignore);
        assert_eq!(
            status.signal(),
            Some(if ignore { libc::SIGKILL } else { libc::SIGTERM })
        );
        if ignore {
            assert!(start.elapsed() >= Duration::from_secs(4));
        }
    }
    assert!(start.elapsed() < Duration::from_secs(8));
}
#[test]
fn test_readiness_failure_terminates_reaps_and_closes_stream() {
    exercise_failure(false);
}
#[test]
fn test_readiness_failure_kills_after_terminate_timeout() {
    exercise_failure(true);
}
