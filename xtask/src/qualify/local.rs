//! Two-process Memory isolation and real SQLite CAS/online-backup/restart qualification.
use crate::{
    smoke::{Server, http, native, require},
    tooling::{self, Result, fail},
};
use rusqlite::{
    Connection, OpenFlags,
    backup::{Backup, StepResult},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::{Arc, Barrier},
    time::{Duration, Instant},
};
fn sql<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| fail("local SQLite operation failed"))
}
pub fn online_backup(source: &Path, destination: &Path, deadline: Instant) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(destination)?;
    drop(file);
    let src = sql(Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ))?;
    sql(src.busy_timeout(Duration::from_millis(200)))?;
    let mut dst = sql(Connection::open(destination))?;
    sql(dst.busy_timeout(Duration::from_millis(200)))?;
    let backup = sql(Backup::new(&src, &mut dst))?;
    loop {
        if Instant::now() >= deadline {
            return Err(tooling::Error {
                code: 124,
                message: "backup deadline".into(),
            });
        }
        tooling::check_cancel()?;
        match sql(backup.step(16))? {
            StepResult::Done => break,
            StepResult::More => {}
            StepResult::Busy | StepResult::Locked => std::thread::sleep(Duration::from_millis(10)),
            _ => return Err(fail("backup status")),
        }
    }
    Ok(())
}
pub fn database(path: &Path) -> Result<Value> {
    let db = sql(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ))?;
    sql(db.busy_timeout(Duration::from_millis(200)))?;
    let integrity: String = sql(db.query_row("PRAGMA integrity_check", [], |r| r.get(0)))?;
    require(integrity == "ok", "database integrity")?;
    let mut fk = sql(db.prepare("PRAGMA foreign_key_check"))?;
    require(
        sql(fk.query([]))?
            .next()
            .map_err(|_| fail("foreign key check"))?
            .is_none(),
        "foreign key violations",
    )?;
    let version: i64 = sql(db.query_row("PRAGMA user_version", [], |r| r.get(0)))?;
    require(version == 1, "database schema version")?;
    let mut tables = json!({"user_version":version});
    for table in ["notes", "note_audit"] {
        let mut statement = sql(db.prepare(&format!("SELECT * FROM {table} ORDER BY id")))?;
        let columns = statement.column_count();
        let rows = sql(statement.query_map([], |r| {
            let mut a = vec![];
            for i in 0..columns {
                a.push(match r.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => json!(n),
                    rusqlite::types::ValueRef::Real(n) => json!(n),
                    rusqlite::types::ValueRef::Text(s) => json!(String::from_utf8_lossy(s)),
                    _ => Value::Null,
                })
            }
            Ok(json!(a))
        }))?;
        tables[table] = json!(
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|_| fail("database rows"))?
        );
    }
    Ok(tables)
}
fn private_response(base: &str, cookie: &str, status: u16) -> Result<()> {
    let r = http::authenticated_get(base, "/auth/notes", cookie)?;
    require(
        r.status == status && r.header("cache-control")? == "no-store",
        "private Memory scope",
    )
}
pub fn write_report(path: &Path, v: &Value) -> Result<()> {
    let text = tooling::json_bytes(v);
    let s = String::from_utf8_lossy(&text);
    require(
        !s.contains(&"a".repeat(64))
            && !s.contains(&"b".repeat(64))
            && !s.contains("yetlet-auth-notes="),
        "sensitive fixture value entered report",
    )?;
    fs::create_dir_all(path.parent().ok_or_else(|| fail("report parent"))?)?;
    let mut f = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    use std::io::Write;
    f.write_all(&text)?;
    f.flush()?;
    f.persist(path).map_err(|_| fail("report write failed"))?;
    Ok(())
}
#[derive(Default)]
pub struct Owned {
    servers: Vec<Server>,
    attempts: usize,
}
impl Owned {
    fn start(
        &mut self,
        root: &Path,
        out: &Path,
        binary: &str,
        env: &[(&str, String)],
    ) -> Result<usize> {
        let key = if binary == "yetlet-auth-notes" {
            "YETLET_AUTH_LISTEN"
        } else {
            "YETLET_SQLITE_LISTEN"
        };
        let executable = root
            .join("target/debug")
            .join(format!("{binary}{}", std::env::consts::EXE_SUFFIX));
        self.start_with(
            out,
            |folder| Server::spawn_binary(root, folder, &executable, key, env),
            |owned, i| owned.servers[i].wait_ready("/healthz"),
        )
    }
    pub fn start_with(
        &mut self,
        out: &Path,
        spawn: impl FnOnce(&Path) -> Result<Server>,
        ready: impl FnOnce(&mut Self, usize) -> Result<()>,
    ) -> Result<usize> {
        self.attempts += 1;
        let folder = out.join(format!("attempt-{:04}", self.attempts));
        fs::create_dir(&folder)?;
        let s = spawn(&folder)?;
        self.servers.push(s); // Includes failed readiness and cancellation after spawn.
        let i = self.servers.len() - 1;
        ready(self, i)?;
        Ok(i)
    }
    pub fn server(&mut self, i: usize) -> &mut Server {
        &mut self.servers[i]
    }
    pub fn count(&self) -> usize {
        self.servers.len()
    }
    pub fn stop(&mut self, i: usize) -> Result<()> {
        self.servers[i].stop(true)
    }
    fn base(&self, i: usize) -> String {
        self.servers[i].base.clone()
    }
    fn cleanup(&mut self) -> bool {
        let mut ok = true;
        for s in self.servers.iter_mut().rev() {
            if s.stop(false).is_err() {
                ok = false
            }
        }
        ok
    }
}
pub fn run(root: &Path) -> Result<()> {
    run_with(
        root,
        |owned, private, report| exercise(root, owned, private, report),
        || crate::process::CANCELLED.load(std::sync::atomic::Ordering::Relaxed),
        write_report,
    )
}

/// The same ownership/cleanup/report boundary is used by runtime and direct fixtures.
pub fn run_with(
    root: &Path,
    scenario: impl FnOnce(&mut Owned, &Path, &mut Value) -> Result<()>,
    cancelled: impl Fn() -> bool,
    write: impl FnMut(&Path, &Value) -> Result<()>,
) -> Result<()> {
    let scratch = root.join("target/m12/yt086");
    fs::create_dir_all(&scratch)?;
    let evidence = tempfile::Builder::new()
        .prefix("local-operations-evidence-")
        .tempdir_in(&scratch)?
        .keep();
    let report_path = evidence.join("local-operations.json");
    println!("evidence: {}", evidence.display());
    let temp = tempfile::Builder::new()
        .prefix("local-operations-")
        .tempdir_in(&scratch)?;
    let mut owned = Owned::default();
    let mut report = json!({"task_id":"YT-066","port_task":"YT-086","status":"failed-for-local","scope":"prebuilt samples, loopback, synthetic data, local filesystem only","qualification":{},"rust_SQLite_version":rusqlite::version(),"scenarios":{}});
    for name in [
        "distributed_provider",
        "multinode",
        "high_availability",
        "performance_SLO",
        "provider_outage",
        "redis_failover",
        "load",
        "real_OIDC",
        "key_rotation",
        "production_RPO_RTO",
    ] {
        report["qualification"][name] = json!("not_verified");
    }
    let started = Instant::now();
    report["last_phase"] = json!("preflight");
    let result = scenario(&mut owned, temp.path(), &mut report);
    if let Err(error) = &result {
        report["failure_code"] = json!(error.code);
        report["failure_category"] = json!(if error
            .message
            .starts_with("loopback socket unavailable:")
        {
            error.message.as_str()
        } else {
            "expectation_or_operation_failed"
        });
    }
    let cleanup = owned.cleanup();
    let count = owned.count();
    let receipts = exit_receipts(temp.path());
    let cleanup = cleanup
        && receipts
            .as_ref()
            .is_ok_and(|r| r.as_array().is_some_and(|a| a.len() == count));
    report["process_exits"] = receipts.as_ref().cloned().unwrap_or_else(|_| json!([]));
    let removed = temp.close().is_ok();
    report["cleanup"] = json!({"status":if cleanup&&removed{"pass"}else{"failed"},"owned_processes":count,"processes_reaped":cleanup,"temporary_directory_removed":removed});
    report["duration_seconds"] = json!(started.elapsed().as_secs_f64());
    report["outcome"] = json!(if result.is_ok() {
        "success"
    } else if cancelled() {
        "cancelled"
    } else {
        "expectation_or_operation_failed"
    });
    let success = result.is_ok() && cleanup && removed && !cancelled();
    if success {
        report["status"] = json!("pass-for-local")
    };
    finalize_report(
        &report_path,
        &mut report,
        success,
        "pass-for-local",
        |_| cancelled(),
        write,
    )
}

/// Allowlist native process evidence before removing private credentials/logs/databases.
pub fn exit_receipts(private: &Path) -> Result<Value> {
    fn visit(root: &Path, p: &Path, rows: &mut Vec<Value>) -> Result<()> {
        for e in fs::read_dir(p)? {
            let p = e?.path();
            if p.is_dir() {
                visit(root, &p, rows)?;
            } else if p
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with("exit.json"))
            {
                let v: Value = serde_json::from_slice(&fs::read(&p)?)
                    .map_err(|_| fail("native exit receipt JSON"))?;
                let mut row = json!({"attempt":p.parent().unwrap().strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")});
                for k in [
                    "native_exit_code",
                    "signal",
                    "listener_closed",
                    "stream_closed",
                    "owned_tree_reaped",
                    "cleanup_error",
                ] {
                    row[k] = v[k].clone();
                }
                rows.push(row);
            }
        }
        Ok(())
    }
    let mut rows = vec![];
    visit(private, private, &mut rows)?;
    Ok(json!(rows))
}

pub fn expected_startup_failure(folder: &Path) -> Result<bool> {
    let receipts = exit_receipts(folder)?;
    Ok(receipts.as_array().is_some_and(|a| {
        !a.is_empty()
            && a.iter().all(|r| {
                r["native_exit_code"].as_u64().is_some_and(|n| n > 0)
                    && r["listener_closed"] == true
                    && r["stream_closed"] == true
                    && r["owned_tree_reaped"] == true
                    && r["cleanup_error"].is_null()
            })
    }))
}

pub fn verify_corrupt_start(start: Result<usize>, folder: &Path) -> Result<()> {
    require(start.is_err(), "corrupt candidate must exit before health")?;
    require(
        expected_startup_failure(folder)?,
        "corrupt candidate must have observed nonzero native exit and closed listener",
    )
}

/// Shared qualification report boundary: cleanup precedes this call; a late
/// cancellation or failed rewrite removes any provisional success report.
pub fn finalize_report(
    path: &Path,
    report: &mut Value,
    success: bool,
    pass_label: &str,
    cancelled: impl Fn(&mut Value) -> bool,
    mut write: impl FnMut(&Path, &Value) -> Result<()>,
) -> Result<()> {
    let mut pass = success && !cancelled(report);
    report["status"] = json!(if pass { pass_label } else { "failed" });
    if cancelled(report) {
        report["outcome"] = json!("cancelled");
        report["stage"] = json!("finalization");
        pass = false;
    }
    if let Err(e) = write(path, report) {
        let _ = fs::remove_file(path);
        return Err(e);
    }
    if cancelled(report) {
        report["status"] = json!("failed");
        report["outcome"] = json!("cancelled");
        report["stage"] = json!("finalization");
        if let Err(e) = write(path, report) {
            let _ = fs::remove_file(path);
            return Err(e);
        }
        pass = false;
    }
    if pass {
        Ok(())
    } else {
        Err(fail("qualification did not satisfy bounded gate"))
    }
}

fn exercise(root: &Path, owned: &mut Owned, private: &Path, report: &mut Value) -> Result<()> {
    let mut identities = json!({});
    for (name, binary) in [
        ("auth-notes", "yetlet-auth-notes"),
        ("sqlite-crud", "yetlet-sqlite-crud"),
    ] {
        identities[name] = json!({"binary_sha256":tooling::sha(&fs::read(root.join(format!("target/debug/{binary}{}",std::env::consts::EXE_SUFFIX)))?),"checkout_package_version":"0.1.0","observed_binary_version":"not_exposed","application_SQLite_version":if name=="sqlite-crud"{"not_exposed"}else{"not_applicable"}});
    }
    report["applications"] = identities;
    report["last_phase"] = json!("session_isolation_restart");
    let env = [
        ("YETLET_AUTH_ALICE_CODE", "a".repeat(64)),
        ("YETLET_AUTH_BOB_CODE", "b".repeat(64)),
    ];
    let a = owned.start(root, private, "yetlet-auth-notes", &env)?;
    let b = owned.start(root, private, "yetlet-auth-notes", &env)?;
    let ca = native::login(&owned.base(a), "alice", &"a".repeat(64))?.0;
    private_response(&owned.base(a), &ca, 200)?;
    private_response(&owned.base(b), &ca, 401)?;
    let cb = native::login(&owned.base(b), "alice", &"a".repeat(64))?.0;
    require(ca != cb, "distinct Memory sessions")?;
    private_response(&owned.base(b), &cb, 200)?;
    private_response(&owned.base(a), &cb, 401)?;
    private_response(&owned.base(a), &ca, 200)?;
    owned.stop(a)?;
    let c = owned.start(root, private, "yetlet-auth-notes", &env)?;
    private_response(&owned.base(c), &ca, 401)?;
    private_response(&owned.base(b), &cb, 200)?;
    owned.stop(b)?;
    report["scenarios"]["session_isolation_restart"] = json!({"status":"pass-for-local","same_cookie_worker_statuses":[200,401],"separate_worker_cookie_statuses":[200,401],"cookies_differ":true,"old_cookie_after_restart":401,"private_responses_no_store":true,"limitation":"Session is process-local, not shared, and lost on restart"});
    report["last_phase"] = json!("sqlite_concurrency_backup_restore");
    let path = private.join("live.sqlite3");
    let env = [("YETLET_SQLITE_PATH", path.display().to_string())];
    let a = owned.start(root, private, "yetlet-sqlite-crud", &env)?;
    let b = owned.start(root, private, "yetlet-sqlite-crud", &env)?;
    let bases = [owned.base(a), owned.base(b)];
    let r = http::json(
        &bases[0],
        "POST",
        "/api/notes",
        &json!({"title":"Checkpoint note","body":"initial"}),
        &[],
    )?;
    require(
        r.status == 201 && r.header("cache-control")? == "no-store",
        "create checkpoint note",
    )?;
    let created = r.json()?;
    require(
        created["id"] == 1 && created["version"] == 1,
        "initial note identity",
    )?;
    for base in &bases {
        let r = http::get(base, "/api/notes/1")?;
        require(
            r.status == 200 && r.json()? == created,
            "cross-process note visibility",
        )?;
    }
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let jobs:Vec<_>=bases.iter().enumerate().map(|(i,base)|{let barrier=barrier.clone();scope.spawn(move||{barrier.wait();http::json(base,"PUT","/api/notes/1",&json!({"title":format!("Writer {i}"),"body":"winner","expected_version":1}),&[])})}).collect();
        jobs.into_iter()
            .map(|j| j.join().map_err(|_| fail("writer thread"))?)
            .collect::<Result<Vec<_>>>()
    })?;
    let mut statuses: Vec<_> = results.iter().map(|r| r.status).collect();
    statuses.sort();
    require(statuses == [200, 409], "exactly one SQLite CAS winner")?;
    let winner = results.iter().find(|r| r.status == 200).unwrap().json()?;
    require(winner["version"] == 2, "CAS version")?;
    for base in &bases {
        let r = http::get(base, "/api/notes/1")?;
        require(r.status == 200 && r.json()? == winner, "winner convergence")?;
    }
    let state = database(&path)?;
    require(
        state["notes"].as_array().unwrap().len() == 1
            && state["note_audit"].as_array().unwrap().len() == 2
            && state["note_audit"][0][2] == "create"
            && state["note_audit"][0][3] == 1
            && state["note_audit"][1][2] == "update"
            && state["note_audit"][1][3] == 2,
        "transaction audit",
    )?;
    let db = sql(Connection::open(&path))?;
    require(
        sql(db.query_row::<String, _, _>("PRAGMA journal_mode", [], |r| r.get(0)))? == "wal",
        "SQLite WAL",
    )?;
    drop(db);
    let checkpoint = private.join("checkpoint.sqlite3");
    online_backup(&path, &checkpoint, Instant::now() + Duration::from_secs(5))?;
    require(database(&checkpoint)? == state, "online backup equality")?;
    let hash = tooling::sha(&fs::read(&checkpoint)?);
    let late = http::json(
        &bases[1],
        "POST",
        "/api/notes",
        &json!({"title":"Late note","body":"after backup"}),
        &[],
    )?;
    require(
        late.status == 201 && late.json()?["id"] == 2,
        "late mutation",
    )?;
    owned.stop(a)?;
    owned.stop(b)?;
    let restored = private.join("restored.sqlite3");
    online_backup(
        &checkpoint,
        &restored,
        Instant::now() + Duration::from_secs(5),
    )?;
    require(database(&restored)? == state, "exact restored checkpoint")?;
    let d = owned.start(
        root,
        private,
        "yetlet-sqlite-crud",
        &[("YETLET_SQLITE_PATH", restored.display().to_string())],
    )?;
    let r = http::get(&owned.base(d), "/api/notes")?;
    require(
        r.status == 200 && r.json()? == json!({"items":[winner],"count":1}),
        "restored API state",
    )?;
    require(
        http::get(&owned.base(d), "/api/notes/2")?.status == 404 && database(&restored)? == state,
        "late write absent",
    )?;
    owned.stop(d)?;
    let corrupt = private.join("corrupt-candidate.sqlite3");
    online_backup(
        &checkpoint,
        &corrupt,
        Instant::now() + Duration::from_secs(5),
    )?;
    use std::io::Write;
    fs::OpenOptions::new()
        .write(true)
        .open(&corrupt)?
        .write_all(b"invalid SQLite header")?;
    let faildir = private.join("corrupt-start");
    fs::create_dir(&faildir)?;
    let failed = owned.start(
        root,
        &faildir,
        "yetlet-sqlite-crud",
        &[("YETLET_SQLITE_PATH", corrupt.display().to_string())],
    );
    verify_corrupt_start(failed, &faildir)?;
    require(
        database(&checkpoint)? == state && tooling::sha(&fs::read(&checkpoint)?) == hash,
        "original checkpoint unchanged",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for p in [&checkpoint, &restored, &corrupt] {
            require(
                fs::metadata(p)?.permissions().mode() & 0o777 == 0o600,
                "candidate mode",
            )?;
        }
    }
    report["scenarios"]["sqlite_concurrency_backup_restore"] = json!({"status":"pass-for-local","scope":"two processes, one local WAL file","concurrent_update_statuses":[200,409],"final_version":2,"checkpoint_notes":1,"checkpoint_audit_rows":2,"user_version":1,"integrity_check":"ok","foreign_key_violations":0,"exact_checkpoint_restored":true,"late_write_absent":true,"corrupt_candidate_nonzero_before_health":true,"original_checkpoint_unchanged":true,"candidate_mode":"0600"});
    Ok(())
}
