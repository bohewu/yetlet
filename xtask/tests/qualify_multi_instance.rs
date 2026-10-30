use serde_json::json;
use std::{ffi::OsString, fs};
use yetlet_xtask::{
    qualify::{
        docker::{NGINX_IMAGE, RUST_IMAGE, cleanup_with, last_upstream},
        local::write_report,
    },
    repository_root,
};
fn strings(a: &[OsString]) -> Vec<String> {
    a.iter().map(|s| s.to_string_lossy().into_owned()).collect()
}
#[cfg(unix)]
#[test]
fn docker_lab_preserves_unix_setup() {
    for redis in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let lab = yetlet_xtask::qualify::docker::Lab::new(root.path(), redis).unwrap();
        assert_eq!(lab.redis, redis);
        assert!(lab.work.is_dir() && lab.logs.is_dir());
        assert!(lab.work.starts_with(root.path().join("target/m12/yt086")));
        assert!(lab.logs.starts_with(root.path().join("target/m12/yt086")));
        assert!(!lab.started);
    }
}
#[cfg(not(unix))]
#[test]
fn docker_qualification_requires_posix_before_setup() {
    let root = tempfile::tempdir().unwrap();
    let absent = root.path().join("not-created");
    for suite in ["multi-instance", "redis-three-app"] {
        let error = yetlet_xtask::qualify::run(&absent, suite, &[]).unwrap_err();
        assert_eq!(error.code, 1);
        assert!(error.message.contains("unsupported on this platform"));
        assert!(error.message.contains("POSIX"));
        assert!(!absent.exists());

        let output = std::process::Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
            .args(["qualify", suite])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("POSIX"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported on this platform"));
    }
}
#[cfg(not(unix))]
#[test]
fn docker_build_requires_posix_before_writes() {
    for redis in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut lab = fixture_lab(root.path());
        lab.redis = redis;
        let error = lab.build().unwrap_err();
        assert!(error.message.contains("POSIX"));
        assert!(!lab.work.join("build").exists());
        assert!(!lab.work.join("image").exists());
        assert_eq!(fs::read_dir(&lab.logs).unwrap().count(), 0);
    }
}
#[test]
fn last_upstream_uses_successful_retry() {
    assert_eq!(
        last_upstream("10.0.0.1:3010, 10.0.0.3:3010").unwrap(),
        "10.0.0.3:3010"
    );
    assert!(last_upstream(" , ").is_err());
}
#[test]
fn report_rejects_fixture_codes_and_raw_cookie() {
    let t = tempfile::tempdir().unwrap();
    for secret in [
        "a".repeat(64),
        "b".repeat(64),
        "yetlet-auth-notes=raw".into(),
    ] {
        assert!(
            write_report(&t.path().join("report.json"), &json!({"unexpected":secret})).is_err()
        );
    }
    write_report(&t.path().join("report.json"),&json!({"memory_session":{"cross_instance_status":401,"shared_across_instances":false},"sha256":"f".repeat(64)})).unwrap();
}
#[test]
fn compose_is_loopback_only_and_apps_publish_no_ports() {
    let text =
        fs::read_to_string(repository_root().join("qualification/yt066a/compose.yaml")).unwrap();
    assert_eq!(text.matches("ports:").count(), 1);
    assert!(text.contains("127.0.0.1:${YT066A_HOST_PORT:?set YT066A_HOST_PORT}:8080"));
    for s in [
        "app-a:",
        "app-b:",
        "app-c:",
        "front: {}",
        "- front\n      - lab",
        NGINX_IMAGE,
    ] {
        assert!(text.contains(s));
    }
    assert_eq!(text.matches("internal: true").count(), 1);
    assert!(!text.contains("network_mode: host"));
    assert!(!text.contains("privileged: true"));
}
#[test]
fn dockerfile_uses_exact_runtime_and_nonroot_user() {
    let text =
        fs::read_to_string(repository_root().join("qualification/yt066a/Dockerfile")).unwrap();
    assert!(text.contains(&format!("FROM {RUST_IMAGE}")));
    assert!(text.contains("USER 10001:10001"));
    for s in ["ARG RUNTIME_IMAGE", "apt-get", "ENV "] {
        assert!(!text.contains(s));
    }
}
#[test]
fn write_report_is_atomic_and_bounded() {
    let t = tempfile::tempdir().unwrap();
    write_report(
        &t.path().join("report.json"),
        &json!({"status":"pass-for-docker-memory"}),
    )
    .unwrap();
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 1);
}
#[test]
fn nginx_places_every_temp_path_on_tmpfs() {
    let text =
        fs::read_to_string(repository_root().join("qualification/yt066a/nginx.conf")).unwrap();
    for name in ["client-body", "proxy", "fastcgi", "uwsgi", "scgi"] {
        assert!(text.contains(&format!("/tmp/{name}")));
    }
    assert!(!text.contains("/var/cache/nginx"));
}
#[test]
fn cleanup_attempts_forced_removal_after_down_failure() {
    let mut calls = vec![];
    let mut lists = 0;
    let r = cleanup_with(
        "owned-project",
        "owned-build",
        "owned-image",
        &["compose".into(), "down".into()],
        |a, _| {
            let a = strings(a);
            let list = a.iter().any(|s| s == "--quiet");
            calls.push(a.clone());
            if list {
                lists += 1;
                return Some((
                    true,
                    if lists <= 3 {
                        "owned-id".into()
                    } else {
                        String::new()
                    },
                ));
            }
            Some((false, String::new()))
        },
    );
    assert_eq!(r["status"], "pass");
    for command in [
        vec!["rm", "--force", "owned-id"],
        vec!["network", "rm", "owned-id"],
        vec!["volume", "rm", "--force", "owned-id"],
        vec!["image", "rm", "--force", "owned-image"],
    ] {
        assert!(
            calls
                .iter()
                .any(|c| c.iter().map(String::as_str).collect::<Vec<_>>() == command)
        );
    }
    assert!(
        calls
            .iter()
            .filter(|a| a.iter().any(|s| s == "--filter"))
            .all(|a| a
                .iter()
                .any(|s| s == "label=com.docker.compose.project=owned-project"
                    || s == "name=^owned-build$"))
    );
}
#[test]
fn cleanup_fails_when_final_queries_fail() {
    let r = cleanup_with("owned", "build", "image", &[], |_, _| None);
    assert_eq!(r["status"], "failed");
    assert_eq!(r["verification_complete"], false);
    assert!(r["remaining"]["containers"].is_null());
}
#[test]
fn cleanup_fails_when_owned_image_remains() {
    let r = cleanup_with("owned", "build", "owned-image", &[], |a, _| {
        let s = strings(a);
        Some((
            true,
            if s == ["image", "ls", "--quiet", "--no-trunc", "owned-image"] {
                "image-id".into()
            } else {
                String::new()
            },
        ))
    });
    assert_eq!(r["status"], "failed");
    assert_eq!(r["verification_complete"], true);
    assert_eq!(r["remaining"]["application_image"], 1);
}

fn fixture_lab(root: &std::path::Path) -> yetlet_xtask::qualify::docker::Lab {
    use yetlet_xtask::qualify::docker::Lab;
    let work = root.join("owned-work");
    let logs = root.join("logs");
    fs::create_dir(&work).unwrap();
    fs::write(work.join("private-build-input"), b"fixture").unwrap();
    fs::create_dir(&logs).unwrap();
    Lab {
        root: root.into(),
        redis: false,
        project: "owned-project".into(),
        build_container: "owned-build".into(),
        image: "owned-image".into(),
        base: "http://127.0.0.1:0".into(),
        port: 0,
        work,
        logs,
        namespace: "fixture".into(),
        env: vec![],
        started: false,
    }
}

#[test]
fn redis_persistence_configuration_is_checked_before_acceptance() {
    use yetlet_xtask::qualify::docker::redis_no_persistence_with;
    for (aof, save, pass) in [
        ("no", "", true),
        ("yes", "", false),
        ("no", "3600 1", false),
    ] {
        let mut calls = vec![];
        let result = redis_no_persistence_with(|key| {
            calls.push(key.to_owned());
            Ok(format!(
                "{key}\n{}\n",
                if key == "appendonly" { aof } else { save }
            ))
        });
        assert_eq!(result.is_ok(), pass);
        assert_eq!(calls[0], "appendonly");
        if aof == "no" {
            assert_eq!(calls, ["appendonly", "save"]);
        }
    }
}

// Signals run only in this isolated child, never in the test runner process.
#[cfg(unix)]
#[test]
fn lifecycle_signal_child() {
    use std::{cell::Cell, sync::atomic::Ordering};
    use yetlet_xtask::{
        process,
        qualify::{docker, local::write_report},
        tooling::{check_cancel, fail},
    };
    let Ok(stage) = std::env::var("YT086_DOCKER_STAGE") else {
        return;
    };
    process::install_cancellation().unwrap();
    let t = tempfile::tempdir().unwrap();
    let mut lab = fixture_lab(t.path());
    let work = lab.work.clone();
    let path = lab.logs.join("report.json");
    let mut report = json!({"task_id":"YT-066a","status":"failed","stage":"preflight"});
    let calls = Cell::new(0);
    let mut writes = 0;
    let mut raise_count = 0;
    let raise = |signal| assert_eq!(unsafe { libc::raise(signal) }, 0);
    let result = docker::run_with(
        &mut lab,
        &path,
        &mut report,
        |lab, report| {
            lab.started = true; // Inject the compose-start boundary, as in the Python direct test.
            report["stage"] = json!("topology");
            if stage == "compose_start" || stage == "cleanup_transition" {
                raise(libc::SIGTERM);
                check_cancel()?;
            }
            Ok(())
        },
        |lab| {
            assert!(lab.started);
            let result = cleanup_with(
                &lab.project,
                &lab.build_container,
                &lab.image,
                &lab.compose_args(&["down"]),
                |_, _| {
                    let n = calls.get() + 1;
                    calls.set(n);
                    if n == 1
                        && [
                            "cleanup",
                            "cleanup_sigint",
                            "term_then_int",
                            "int_then_term",
                        ]
                        .contains(&stage.as_str())
                    {
                        raise(if stage == "cleanup_sigint" || stage == "int_then_term" {
                            libc::SIGINT
                        } else {
                            libc::SIGTERM
                        });
                        raise_count += 1;
                    }
                    if n == 2 && ["term_then_int", "int_then_term"].contains(&stage.as_str()) {
                        raise(if stage == "term_then_int" {
                            libc::SIGINT
                        } else {
                            libc::SIGTERM
                        });
                        raise_count += 1;
                    }
                    Some((true, String::new()))
                },
            );
            if stage == "removal_failure" {
                fs::remove_dir_all(&lab.work).unwrap();
                fs::write(&lab.work, b"blocks directory removal").unwrap();
            }
            result
        },
        process::cancellation_signal,
        |p, v| {
            writes += 1;
            assert!(calls.get() >= 10);
            assert_eq!(v["cleanup"]["verification_complete"], true);
            if stage == "removal_failure" {
                assert!(work.is_file());
                assert_eq!(v["owned_work_removed"], false);
            } else {
                assert!(!work.exists(), "cleanup must precede report writing");
                assert_eq!(v["owned_work_removed"], true);
            }
            if stage == "rewrite_failure" && writes == 2 {
                return Err(fail("report I/O fixture"));
            }
            write_report(p, v)?;
            if writes == 1
                && [
                    "report_write",
                    "report_write_sigint",
                    "report_term_then_int",
                    "report_int_then_term",
                    "rewrite_failure",
                ]
                .contains(&stage.as_str())
            {
                let first = if stage == "report_write_sigint" || stage == "report_int_then_term" {
                    libc::SIGINT
                } else {
                    libc::SIGTERM
                };
                raise(first);
                if stage == "report_term_then_int" || stage == "report_int_then_term" {
                    raise(if first == libc::SIGTERM {
                        libc::SIGINT
                    } else {
                        libc::SIGTERM
                    });
                }
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    assert!(
        calls.get() >= 10,
        "all cleanup/recheck commands finish even after cancellation"
    );
    if stage == "removal_failure" {
        assert!(!process::CANCELLED.load(Ordering::Relaxed));
        assert_eq!(report["owned_work_removed"], false);
    } else {
        assert!(!work.exists());
        assert_eq!(report["outcome"], "cancelled");
        let first = if [
            "cleanup_sigint",
            "int_then_term",
            "report_write_sigint",
            "report_int_then_term",
        ]
        .contains(&stage.as_str())
        {
            libc::SIGINT
        } else {
            libc::SIGTERM
        };
        assert_eq!(report["signal"], first);
        assert_eq!(
            report["failure_type"],
            if first == libc::SIGINT {
                "KeyboardInterrupt"
            } else {
                "SIGTERM"
            }
        );
        assert_eq!(process::cancellation_signal(), Some(first));
    }
    if stage == "rewrite_failure" {
        assert_eq!(writes, 2);
        assert!(!path.exists());
    } else {
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["status"], "failed");
        if stage != "removal_failure" {
            assert_eq!(saved["signal"], report["signal"]);
            assert_eq!(saved["failure_type"], report["failure_type"]);
            assert_eq!(saved["outcome"], "cancelled");
            assert_eq!(saved["cleanup"]["status"], "pass");
        }
    }
    if [
        "report_write",
        "report_write_sigint",
        "report_term_then_int",
        "report_int_then_term",
    ]
    .contains(&stage.as_str())
    {
        assert_eq!(writes, 2);
    }
    if stage == "term_then_int" || stage == "int_then_term" {
        assert_eq!(raise_count, 2);
    }
}
#[cfg(unix)]
fn cancellation_at(stage: &str) {
    let r = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lifecycle_signal_child", "--nocapture"])
        .env("YT086_DOCKER_STAGE", stage)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stdout));
}
#[cfg(unix)]
#[test]
fn sigterm_after_compose_start_cleans_and_cannot_pass() {
    cancellation_at("compose_start")
}
#[cfg(unix)]
#[test]
fn first_sigterm_during_cleanup_records_cancel_and_finishes_cleanup() {
    cancellation_at("cleanup")
}
#[cfg(unix)]
#[test]
fn first_sigterm_at_report_write_rewrites_nonpassing_result() {
    cancellation_at("report_write")
}
#[cfg(unix)]
#[test]
fn first_sigint_at_report_write_retains_identity_in_nonpassing_result() {
    cancellation_at("report_write_sigint")
}
#[cfg(unix)]
#[test]
fn mixed_signals_during_report_write_keep_first_identity() {
    cancellation_at("report_term_then_int");
    cancellation_at("report_int_then_term");
}
#[cfg(unix)]
#[test]
fn sigterm_at_cleanup_transition_still_runs_cleanup() {
    cancellation_at("cleanup_transition")
}
#[cfg(unix)]
#[test]
fn report_write_failure_still_removes_owned_work_and_cannot_pass() {
    cancellation_at("rewrite_failure")
}
#[cfg(unix)]
#[test]
fn owned_work_removal_failure_prevents_pass() {
    cancellation_at("removal_failure")
}
#[cfg(unix)]
#[test]
fn first_sigint_during_cleanup_records_cancel_and_finishes_cleanup() {
    cancellation_at("cleanup_sigint")
}
#[cfg(unix)]
#[test]
fn sigterm_then_sigint_is_ignored_during_cleanup() {
    cancellation_at("term_then_int")
}
#[cfg(unix)]
#[test]
fn sigint_then_sigterm_is_ignored_during_cleanup() {
    cancellation_at("int_then_term")
}

#[test]
fn successful_orchestration_removes_real_owned_work_before_report() {
    use yetlet_xtask::qualify::docker;
    let t = tempfile::tempdir().unwrap();
    let mut lab = fixture_lab(t.path());
    let work = lab.work.clone();
    let path = lab.logs.join("report.json");
    let mut report = json!({"task_id":"YT-066a"});
    let mut calls = 0;
    docker::run_with(
        &mut lab,
        &path,
        &mut report,
        |lab, _| {
            lab.started = true;
            Ok(())
        },
        |lab| {
            cleanup_with(
                &lab.project,
                &lab.build_container,
                &lab.image,
                &lab.compose_args(&["down"]),
                |_, _| {
                    calls += 1;
                    Some((true, String::new()))
                },
            )
        },
        || None,
        |p, v| {
            assert!(!work.exists());
            assert_eq!(v["cleanup"]["status"], "pass");
            write_report(p, v)
        },
    )
    .unwrap();
    assert!(calls >= 10);
    assert_eq!(report["status"], "pass-for-docker-memory");
}
