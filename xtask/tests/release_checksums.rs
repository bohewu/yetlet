#![cfg(unix)]
use serde_json::{Value, json};
use std::{fs, os::unix::fs::symlink, path::Path, process::Command};
use yetlet_xtask::release::checksums::{self as c, MANIFEST};
fn fixture() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("app.wasm"), b"wasm fixture").unwrap();
    fs::write(t.path().join("inventory.cdx.json"), b"{}\n").unwrap();
    t
}
#[test]
fn unvalidated_u64_total_overflow_is_redacted_exit_one() {
    let t = fixture();
    let manifest = json!({"schema_version":1,"notice":c::NOTICE,"artifacts":[
        {"path":"first","size":1,"sha256":"0".repeat(64)},
        {"path":"second","size":u64::MAX,"sha256":"0".repeat(64)}
    ]});
    let bytes = serde_json::to_vec(&manifest).unwrap();
    assert!(c::parse_manifest(&bytes).is_err());
    fs::write(t.path().join(MANIFEST), bytes).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["release", "checksums", "verify"])
        .arg(t.path())
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&r.stderr);
    assert!(!stderr.contains("panicked"));
    assert!(!stderr.contains(t.path().to_str().unwrap()));
    assert!(stderr.contains("release checksum preflight failed"));
}
fn create(p: &Path) {
    c::run("create", p).unwrap()
}
fn change(p: &Path, f: impl FnOnce(&mut Value)) {
    let m = p.join(MANIFEST);
    let mut v = serde_json::from_slice(&fs::read(&m).unwrap()).unwrap();
    f(&mut v);
    fs::write(m, serde_json::to_vec(&v).unwrap()).unwrap()
}
#[test]
fn cli_success_and_manifest_exclusion() {
    let t = fixture();
    for op in ["create", "verify"] {
        let r = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
            .args(["release", "checksums", op])
            .arg(t.path())
            .output()
            .unwrap();
        assert!(r.status.success(), "{:?}", r);
        assert!(String::from_utf8_lossy(&r.stdout).contains(c::NOTICE));
    }
    let v: Value = serde_json::from_slice(&fs::read(t.path().join(MANIFEST)).unwrap()).unwrap();
    assert_eq!(v["artifacts"].as_array().unwrap().len(), 2);
}
#[test]
fn canonical_content_for_identical_inputs() {
    let a = fixture();
    let b = fixture();
    create(a.path());
    create(b.path());
    assert_eq!(
        fs::read(a.path().join(MANIFEST)).unwrap(),
        fs::read(b.path().join(MANIFEST)).unwrap()
    );
}
#[test]
fn no_overwrite() {
    let t = fixture();
    create(t.path());
    let before = fs::read(t.path().join(MANIFEST)).unwrap();
    assert!(c::run("create", t.path()).is_err());
    assert_eq!(before, fs::read(t.path().join(MANIFEST)).unwrap());
}
#[test]
fn tampered_artifact_and_hash() {
    let t = fixture();
    create(t.path());
    fs::write(t.path().join("app.wasm"), b"evil fixture").unwrap();
    assert!(c::run("verify", t.path()).is_err());
    fs::write(t.path().join("app.wasm"), b"wasm fixture").unwrap();
    change(t.path(), |v| {
        v["artifacts"][0]["sha256"] = json!("0".repeat(64))
    });
    assert!(c::run("verify", t.path()).is_err());
}
#[test]
fn missing_and_extra() {
    let t = fixture();
    create(t.path());
    fs::remove_file(t.path().join("app.wasm")).unwrap();
    assert!(c::run("verify", t.path()).is_err());
    fs::write(t.path().join("app.wasm"), b"wasm fixture").unwrap();
    fs::write(t.path().join("extra"), b"extra").unwrap();
    assert!(c::run("verify", t.path()).is_err());
}
#[test]
fn unsafe_manifest_paths_and_self_reference() {
    for name in [
        "../secret",
        "/etc/passwd",
        "nested/file",
        "./app.wasm",
        "..",
        "C:\\secret",
        "a\\b",
        "a\0b",
        MANIFEST,
    ] {
        let t = fixture();
        create(t.path());
        change(t.path(), |v| v["artifacts"][0]["path"] = json!(name));
        assert!(c::run("verify", t.path()).is_err(), "{name}");
    }
}
#[test]
fn duplicate_entries_and_json_keys() {
    let t = fixture();
    create(t.path());
    change(t.path(), |v| {
        let e = v["artifacts"][0].clone();
        v["artifacts"].as_array_mut().unwrap().push(e)
    });
    assert!(c::run("verify", t.path()).is_err());
    assert!(c::parse_manifest(br#"{"schema_version":1,"schema_version":1,"notice":"unsigned local verification; not trusted provenance","artifacts":[]}"#).is_err());
    assert!(c::parse_manifest(br#"{"a":{"nested":1,"nested":2}}"#).is_err());
}
#[test]
fn symlink_artifact_create_and_verify() {
    let t = fixture();
    create(t.path());
    fs::remove_file(t.path().join("app.wasm")).unwrap();
    symlink("outside", t.path().join("app.wasm")).unwrap();
    assert!(c::run("verify", t.path()).is_err());
    fs::remove_file(t.path().join(MANIFEST)).unwrap();
    assert!(c::run("create", t.path()).is_err());
}
#[test]
fn symlink_manifest_never_overwritten_or_read() {
    let t = fixture();
    let outside = t.path().join("outside");
    fs::write(&outside, b"private").unwrap();
    symlink(&outside, t.path().join(MANIFEST)).unwrap();
    assert!(c::run("create", t.path()).is_err());
    assert!(c::run("verify", t.path()).is_err());
    assert_eq!(fs::read(outside).unwrap(), b"private");
}
#[test]
fn symlink_directory_and_ancestor() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("bundle");
    fs::create_dir(&p).unwrap();
    fs::write(p.join("x"), b"x").unwrap();
    create(&p);
    let alias = t.path().join("alias");
    symlink(&p, &alias).unwrap();
    assert!(c::run("verify", &alias).is_err());
    let parent = t.path().join("parent");
    symlink(t.path(), &parent).unwrap();
    assert!(c::run("verify", &parent.join("bundle")).is_err());
    assert!(c::run("create", &parent.join("bundle")).is_err());
}
#[test]
fn directory_traversal_rejected() {
    let t = fixture();
    assert!(
        c::run(
            "create",
            &t.path().join("../").join(t.path().file_name().unwrap())
        )
        .is_err()
    );
}
#[test]
fn nonregular_files() {
    let t = fixture();
    fs::create_dir(t.path().join("nonregular")).unwrap();
    assert!(c::run("create", t.path()).is_err());
    fs::remove_dir(t.path().join("nonregular")).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let s = std::ffi::CString::new(t.path().join("fifo").as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(s.as_ptr(), 0o600) }, 0);
    assert!(c::run("create", t.path()).is_err());
}
#[test]
fn nonempty_input() {
    let t = tempfile::tempdir().unwrap();
    assert!(c::run("create", t.path()).is_err());
}
#[test]
fn count_file_total_and_manifest_limits() {
    let t = tempfile::tempdir().unwrap();
    for i in 0..129 {
        fs::write(t.path().join(format!("x{i}")), b"x").unwrap();
    }
    assert!(c::run("create", t.path()).is_err());
    let t = fixture();
    fs::File::create(t.path().join("large"))
        .unwrap()
        .set_len(c::MAX_FILE_BYTES + 1)
        .unwrap();
    assert!(c::run("create", t.path()).is_err());
    let entries: Vec<_> = (0..5)
        .map(|i| json!({"path":format!("x{i}"),"size":c::MAX_FILE_BYTES,"sha256":"0".repeat(64)}))
        .collect();
    assert!(
        c::parse_manifest(
            &serde_json::to_vec(
                &json!({"schema_version":1,"notice":c::NOTICE,"artifacts":entries})
            )
            .unwrap()
        )
        .is_err()
    );
    assert!(c::parse_manifest(&vec![b' '; c::MAX_MANIFEST_BYTES as usize + 1]).is_err());
}
#[test]
fn malformed_manifest_and_types() {
    for (field, value) in [
        ("schema_version", json!(true)),
        ("notice", json!("trusted")),
        ("artifacts", json!([])),
    ] {
        let t = fixture();
        create(t.path());
        change(t.path(), |v| v[field] = value);
        assert!(c::run("verify", t.path()).is_err());
    }
    for (field, value) in [
        ("size", json!(true)),
        ("size", json!(-1)),
        ("sha256", json!("invalid")),
    ] {
        let t = fixture();
        create(t.path());
        change(t.path(), |v| v["artifacts"][0][field] = value);
        assert!(c::run("verify", t.path()).is_err());
    }
    assert!(c::parse_manifest(b"not JSON").is_err());
}
#[test]
fn failure_output_does_not_disclose_paths() {
    let t = fixture();
    let r = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["release", "checksums", "verify"])
        .arg(t.path().join("private-name"))
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(1));
    assert_eq!(r.stderr, b"release checksum preflight failed\n");
}
