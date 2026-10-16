use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};
use yetlet_xtask::qualify::local::{exit_receipts, online_backup, write_report};
#[test]
fn online_backup_actual_checkpoint_and_no_overwrite() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("source.db");
    let checkpoint = t.path().join("backup.db");
    let restored = t.path().join("restore.db");
    let db = rusqlite::Connection::open(&src).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL;CREATE TABLE fixture(value TEXT);INSERT INTO fixture VALUES ('checkpoint');").unwrap();
    online_backup(&src, &checkpoint, Instant::now() + Duration::from_secs(5)).unwrap();
    db.execute("INSERT INTO fixture VALUES ('late')", [])
        .unwrap();
    online_backup(
        &checkpoint,
        &restored,
        Instant::now() + Duration::from_secs(5),
    )
    .unwrap();
    let copy = rusqlite::Connection::open(restored).unwrap();
    assert_eq!(
        copy.query_row::<i64, _, _>("SELECT COUNT(*) FROM fixture", [], |r| r.get(0))
            .unwrap(),
        1
    );
    assert_eq!(
        copy.query_row::<String, _, _>("SELECT value FROM fixture", [], |r| r.get(0))
            .unwrap(),
        "checkpoint"
    );
    assert!(online_backup(&src, &checkpoint, Instant::now() + Duration::from_secs(5)).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(checkpoint).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
#[test]
fn backup_progress_deadline_interrupts_busy_retry() {
    let t = tempfile::tempdir().unwrap();
    let src = t.path().join("source.db");
    let dst = t.path().join("deadline.db");
    let db = rusqlite::Connection::open(&src).unwrap();
    db.execute_batch("CREATE TABLE fixture(x);BEGIN EXCLUSIVE;INSERT INTO fixture VALUES(1);")
        .unwrap();
    let then = Instant::now();
    let e = online_backup(&src, &dst, then + Duration::from_millis(40)).unwrap_err();
    assert_eq!(e.code, 124);
    assert!(then.elapsed() < Duration::from_secs(2));
}
#[test]
fn spawn_failure_redacted_and_report_atomic() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("report.json");
    write_report(
        &path,
        &json!({"status":"failed-for-local","outcome":"expectation_or_operation_failed"}),
    )
    .unwrap();
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
    for secret in [
        "a".repeat(64),
        "b".repeat(64),
        "yetlet-auth-notes=raw".into(),
    ] {
        assert!(write_report(&path, &json!({"private":secret})).is_err());
    }
    fs::write(t.path().join("regular-parent"), b"x").unwrap();
    assert!(write_report(&t.path().join("regular-parent/report.json"), &json!({})).is_err());
}
#[test]
fn native_exit_receipts_allowlist_and_unknown_are_retained() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("process.exit.json"),serde_json::to_vec(&json!({"native_exit_code":null,"signal":2,"listener_closed":true,"stream_closed":true,"owned_tree_reaped":true,"cleanup_error":null,"private":"sensitive"})).unwrap()).unwrap();
    let v = exit_receipts(t.path()).unwrap();
    assert!(v[0]["native_exit_code"].is_null());
    assert_eq!(v[0]["signal"], 2);
    assert!(v[0].get("private").is_none());
}

#[test]
fn corrupt_start_production_decision_requires_nested_nonzero_exit_evidence() {
    use yetlet_xtask::{qualify::local::verify_corrupt_start, tooling::fail};
    let t = tempfile::tempdir().unwrap();
    let folder = t.path().join("corrupt-start");
    let attempt = folder.join("attempt-0007");
    fs::create_dir_all(&attempt).unwrap();
    let receipt = attempt.join("server-0.exit.json");
    let check = || verify_corrupt_start(Err(fail("startup exited")), &folder);
    assert!(check().is_err(), "missing exit evidence cannot pass");
    let valid = json!({"native_exit_code":23,"signal":null,"listener_closed":true,"stream_closed":true,"owned_tree_reaped":true,"cleanup_error":null});
    fs::write(&receipt, serde_json::to_vec(&valid).unwrap()).unwrap();
    assert!(
        check().is_ok(),
        "production accepts the nested nonzero exit"
    );
    assert!(verify_corrupt_start(Ok(0), &folder).is_err());
    for (key, value) in [
        ("native_exit_code", json!(0)),
        ("native_exit_code", json!(null)),
        ("native_exit_code", json!("23")),
        ("listener_closed", json!(false)),
        ("stream_closed", json!(false)),
        ("owned_tree_reaped", json!(false)),
        ("cleanup_error", json!("fixture failure")),
    ] {
        let mut invalid = valid.clone();
        invalid[key] = value;
        fs::write(&receipt, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(check().is_err(), "invalid {key} cannot pass");
    }
    fs::write(&receipt, b"invalid JSON").unwrap();
    assert!(check().is_err());
}

// Real owned subprocesses without a listener; port zero deliberately repeats.
#[test]
fn lifecycle_child() {
    match std::env::var("YT086_LOCAL_CHILD").as_deref() {
        Ok("exit") => std::process::exit(23),
        Ok("wait") => std::thread::sleep(Duration::from_secs(30)),
        _ => {}
    }
}
fn child(
    folder: &std::path::Path,
    mode: &str,
) -> yetlet_xtask::tooling::Result<yetlet_xtask::smoke::Server> {
    use yetlet_xtask::{
        process::{Service, Spec},
        smoke::Server,
        tooling::fail,
    };
    let log = folder.join("server-0.log");
    let mut process = Service::spawn(
        &Spec {
            program: std::env::current_exe().unwrap().into(),
            args: vec![
                "--exact".into(),
                "lifecycle_child".into(),
                "--nocapture".into(),
            ],
            cwd: folder.into(),
            env: vec![("YT086_LOCAL_CHILD".into(), Some(mode.into()))],
            timeout: Duration::from_secs(5),
            output_limit: 1024,
        },
        &log,
    )
    .map_err(|_| fail("fixture spawn"))?;
    if mode == "exit" {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !process.exited().unwrap() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    Ok(Server {
        process,
        port: 0,
        base: "http://127.0.0.1:0".into(),
        log,
    })
}

#[test]
fn registered_before_failed_readiness_and_reaped_before_report() {
    use yetlet_xtask::qualify::local::{expected_startup_failure, run_with};
    let root = tempfile::tempdir().unwrap();
    let private = std::cell::RefCell::new(None);
    let mut saved = None;
    let result = run_with(
        root.path(),
        |owned, folder, _| {
            *private.borrow_mut() = Some(folder.to_path_buf());
            let failed = owned.start_with(
                folder,
                |p| child(p, "exit"),
                |o, i| {
                    assert_eq!(o.count(), 1);
                    o.server(i).wait_ready("/healthz")
                },
            );
            assert_eq!(failed.as_ref().unwrap_err().code, 23);
            assert!(expected_startup_failure(folder)?);
            failed.map(|_| ())
        },
        || false,
        |p, v| {
            assert!(!private.borrow().as_ref().unwrap().exists());
            saved = Some(v.clone());
            write_report(p, v)
        },
    );
    assert!(result.is_err());
    let r = saved.unwrap();
    assert_eq!(r["cleanup"]["owned_processes"], 1);
    assert_eq!(r["cleanup"]["status"], "pass");
    assert_eq!(r["process_exits"][0]["native_exit_code"], 23);
}

#[test]
fn seven_launches_including_corrupt_start_have_distinct_receipts_on_port_reuse() {
    use yetlet_xtask::qualify::local::{run_with, verify_corrupt_start};
    let root = tempfile::tempdir().unwrap();
    run_with(
        root.path(),
        |owned, folder, _| {
            for _ in 0..6 {
                let i = owned.start_with(folder, |p| child(p, "exit"), |_, _| Ok(()))?;
                owned.stop(i).ok(); // Nonzero graceful exit still leaves its native receipt.
            }
            let corrupt_start = folder.join("corrupt-start");
            fs::create_dir(&corrupt_start)?;
            let failed = owned.start_with(
                &corrupt_start,
                |p| child(p, "exit"),
                |o, i| o.server(i).wait_ready("/healthz"),
            );
            assert_eq!(failed.as_ref().unwrap_err().code, 23);
            verify_corrupt_start(failed, &corrupt_start)
        },
        || false,
        |p, v| {
            assert_eq!(v["cleanup"]["owned_processes"], 7);
            let rows = v["process_exits"].as_array().unwrap();
            assert_eq!(rows.len(), 7);
            let attempts: std::collections::BTreeSet<_> = rows
                .iter()
                .map(|r| r["attempt"].as_str().unwrap())
                .collect();
            assert_eq!(attempts.len(), 7);
            assert!(attempts.contains("corrupt-start/attempt-0007"));
            assert!(
                rows.iter()
                    .all(|r| r["native_exit_code"] == 23 && r["owned_tree_reaped"] == true)
            );
            write_report(p, v)
        },
    )
    .unwrap();
}

#[test]
fn cancellation_and_deadline_after_spawn_still_cleanup_and_report() {
    use yetlet_xtask::{qualify::local::run_with, tooling::Error};
    for code in [130, 124] {
        let root = tempfile::tempdir().unwrap();
        let cancel = std::cell::Cell::new(false);
        let mut saved = None;
        assert!(
            run_with(
                root.path(),
                |owned, folder, _| {
                    owned.start_with(
                        folder,
                        |p| child(p, "wait"),
                        |o, _| {
                            assert_eq!(o.count(), 1);
                            cancel.set(code == 130);
                            if code == 124 {
                                yetlet_xtask::smoke::wait(0, "fixture deadline", || Ok(None::<()>))
                            } else {
                                Err(Error {
                                    code,
                                    message: "cancelled".into(),
                                })
                            }
                        },
                    )?;
                    Ok(())
                },
                || cancel.get(),
                |p, v| {
                    saved = Some(v.clone());
                    write_report(p, v)
                }
            )
            .is_err()
        );
        let v = saved.unwrap();
        assert_eq!(v["cleanup"]["status"], "pass");
        assert_eq!(v["cleanup"]["temporary_directory_removed"], true);
        assert_eq!(v["process_exits"][0]["owned_tree_reaped"], true);
        assert_eq!(v["failure_code"], code);
        assert_eq!(
            v["outcome"],
            if code == 130 {
                "cancelled"
            } else {
                "expectation_or_operation_failed"
            }
        );
    }
}

#[test]
fn spawn_failure_is_redacted_after_reaping_existing_child() {
    use yetlet_xtask::{
        process::{Service, Spec},
        qualify::local::run_with,
        tooling::fail,
    };
    let root = tempfile::tempdir().unwrap();
    let mut saved = None;
    assert!(
        run_with(
            root.path(),
            |owned, folder, _| {
                owned.start_with(folder, |p| child(p, "wait"), |_, _| Ok(()))?;
                owned.start_with(
                    folder,
                    |p| {
                        let result = Service::spawn(
                            &Spec {
                                program: p.join("private-cookie-secret-missing").into(),
                                args: vec![],
                                cwd: p.into(),
                                env: vec![],
                                timeout: Duration::from_secs(1),
                                output_limit: 1024,
                            },
                            &p.join("log"),
                        );
                        assert!(result.is_err());
                        Err(fail("private-cookie-secret-missing"))
                    },
                    |_, _| Ok(()),
                )?;
                Ok(())
            },
            || false,
            |p, v| {
                saved = Some(v.clone());
                write_report(p, v)
            }
        )
        .is_err()
    );
    let v = saved.unwrap();
    assert_eq!(v["cleanup"]["owned_processes"], 1);
    assert_eq!(v["cleanup"]["status"], "pass");
    assert!(!v.to_string().contains("private-cookie-secret"));
}

#[test]
fn all_cleanup_attempted_after_receipt_failure_and_success_cannot_be_reported() {
    use yetlet_xtask::qualify::local::run_with;
    let root = tempfile::tempdir().unwrap();
    let pids = std::cell::RefCell::new(vec![]);
    assert!(
        run_with(
            root.path(),
            |owned, folder, _| {
                for _ in 0..2 {
                    let i = owned.start_with(folder, |p| child(p, "wait"), |_, _| Ok(()))?;
                    pids.borrow_mut().push(owned.server(i).process.id());
                }
                owned.server(1).log = folder.join("missing/receipt.log");
                Ok(())
            },
            || false,
            |p, v| {
                assert_eq!(v["cleanup"]["status"], "failed");
                assert_eq!(v["process_exits"].as_array().unwrap().len(), 1);
                #[cfg(unix)]
                for pid in pids.borrow().iter() {
                    assert_eq!(unsafe { libc::kill(*pid as i32, 0) }, -1);
                }
                assert_ne!(v["status"], "pass-for-local");
                write_report(p, v)
            }
        )
        .is_err()
    );
}
