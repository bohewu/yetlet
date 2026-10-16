//! Failed server lifecycle must retain native status and cleanup receipts.
use std::{
    fs,
    time::{Duration, Instant},
};
use yetlet_xtask::{
    process::{Service, Spec},
    smoke::{Server, free_port},
};

#[test]
fn subprocess_fixture() {
    let Ok(mode) = std::env::var("YT085_SERVER_STATUS_FIXTURE") else {
        return;
    };
    match mode.as_str() {
        "early" => std::process::exit(23),
        "graceful" => std::process::exit(17),
        "zero" => std::process::exit(0),
        #[cfg(unix)]
        "signal" => unsafe {
            libc::raise(libc::SIGTERM);
        },
        _ => panic!("unknown test fixture mode"),
    }
}
fn exited_server(temp: &std::path::Path, mode: &str) -> Server {
    let log = temp.join("server.log");
    let port = free_port().unwrap();
    let spec = Spec {
        program: std::env::current_exe().unwrap().into(),
        args: vec![
            "--exact".into(),
            "subprocess_fixture".into(),
            "--nocapture".into(),
        ],
        cwd: temp.into(),
        env: vec![("YT085_SERVER_STATUS_FIXTURE".into(), Some(mode.into()))],
        timeout: Duration::from_secs(5),
        output_limit: 1024,
    };
    let mut process = Service::spawn(&spec, &log).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !process.exited().unwrap() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    Server {
        process,
        port,
        base: format!("http://127.0.0.1:{port}"),
        log,
    }
}
fn receipt(server: &Server) -> serde_json::Value {
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(server.log.with_extension("exit.json")).unwrap()).unwrap();
    assert_eq!(value["listener_closed"], true);
    assert_eq!(value["stream_closed"], true);
    assert_eq!(value["owned_tree_reaped"], true);
    assert!(value["cleanup_error"].is_null());
    assert!(server.process.stream_closed());
    value
}
#[test]
fn early_exit_keeps_native_code_and_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let mut server = exited_server(temp.path(), "early");
    let err = server.wait_ready("/healthz").unwrap_err();
    assert_eq!(err.code, 23);
    assert_eq!(err.message, "server exited before readiness");
    assert_eq!(receipt(&server)["native_exit_code"], 23);
}
#[test]
fn failed_graceful_exit_writes_receipt_before_returning_error() {
    let temp = tempfile::tempdir().unwrap();
    let mut server = exited_server(temp.path(), "graceful");
    assert_eq!(server.stop(true).unwrap_err().code, 17);
    assert_eq!(receipt(&server)["native_exit_code"], 17);
}
#[test]
fn early_zero_is_not_a_successful_readiness() {
    let temp = tempfile::tempdir().unwrap();
    let mut server = exited_server(temp.path(), "zero");
    assert_eq!(server.wait_ready("/healthz").unwrap_err().code, 1);
    assert_eq!(receipt(&server)["native_exit_code"], 0);
}
#[cfg(unix)]
#[test]
fn signalled_exit_retains_signal_and_portable_status() {
    let temp = tempfile::tempdir().unwrap();
    let mut server = exited_server(temp.path(), "signal");
    assert_eq!(
        server.wait_ready("/healthz").unwrap_err().code,
        128 + libc::SIGTERM
    );
    let value = receipt(&server);
    assert!(value["native_exit_code"].is_null());
    assert_eq!(value["signal"], libc::SIGTERM);
}
