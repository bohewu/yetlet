//! Original five Board profiles, including explicitly opt-in long/traced runs.
use crate::{
    process::{Service, Spec},
    smoke::{
        self, Server,
        board::{BoardClient, codes, metrics, percentile, read_chunk},
        require,
    },
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::Path,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Debug)]
pub struct Profile {
    pub warmup: u64,
    pub rounds: Vec<u64>,
    pub soak: u64,
    pub rps: f64,
    pub stagger: bool,
    pub trace: bool,
}
pub fn profile(name: &str) -> Result<Profile> {
    let (warmup, rounds, soak, rps, stagger, trace) = match name {
        "bounded" => (30, vec![60, 60, 60], 300, 2., false, false),
        "full" => (120, vec![600, 600, 600], 1800, 0.5, true, false),
        "full-sync" => (120, vec![600, 600, 600], 1800, 0.5, false, true),
        "sync-12m" => (120, vec![600], 0, 0.5, false, true),
        "staggered-12m" => (120, vec![600], 0, 0.5, true, true),
        _ => return Err(fail("unknown Board profile")),
    };
    Ok(Profile {
        warmup,
        rounds,
        soak,
        rps,
        stagger,
        trace,
    })
}
fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| fail("Board database operation"))
}
pub fn seed(path: &Path) -> Result<()> {
    let mut c = db(rusqlite::Connection::open(path))?;
    let t = db(c.transaction())?;
    for i in 1..=10000 {
        let title = format!("Work record {i:05}");
        db(t.execute("INSERT INTO work_items(id,project,title,description,status,priority,assignee,version) VALUES(?1,1,?2,'Synthetic local qualification','open','normal','',9)",rusqlite::params![i,title]))?;
        for v in 1..10 {
            db(t.execute("INSERT INTO history(item_id,project,version,actor,action,title,status) VALUES(?1,1,?2,'alice',?3,?4,'open')",rusqlite::params![i,v,if v==1{"create"}else{"update"},title]))?;
        }
    }
    db(t.commit())
}
fn start(
    root: &Path,
    private: &Path,
    out: &Path,
    path: &Path,
    p: &Profile,
    n: usize,
    report: &mut Value,
) -> Result<Server> {
    let folder = private.join(format!("process-{n}"));
    fs::create_dir(&folder)?;
    if !p.trace {
        return Server::start(
            root,
            &folder,
            "yetlet-lab-operations-board",
            "release",
            "YETLET_BOARD_LISTEN",
            &[("YETLET_BOARD_DB", path.display().to_string())],
            "/readyz",
        );
    }
    require(
        cfg!(target_os = "linux") && Path::new("/usr/bin/strace").is_file(),
        "strace profile capability unavailable",
    )?;
    let port = smoke::free_port()?;
    let binary = root.join("target/release/yetlet-lab-operations-board");
    let trace = out.join(format!("fsync-start-{n}.trace"));
    let log = folder.join("server.log");
    let mut args: Vec<_> = [
        "-D",
        "-f",
        "--seccomp-bpf",
        "-qq",
        "-ttt",
        "-T",
        "-yy",
        "-e",
        "trace=fsync,fdatasync",
        "-e",
        "signal=none",
        "-o",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    args.push(trace.into_os_string());
    args.push(binary.clone().into_os_string());
    let proc = Service::spawn(
        &Spec {
            program: "/usr/bin/strace".into(),
            args,
            cwd: root.into(),
            env: vec![
                (
                    "YETLET_BOARD_LISTEN".into(),
                    Some(format!("127.0.0.1:{port}").into()),
                ),
                ("YETLET_BOARD_DB".into(), Some(path.as_os_str().into())),
            ],
            timeout: Duration::from_secs(4000),
            output_limit: 1024 * 1024,
        },
        &log,
    )
    .map_err(|_| fail("traced Board spawn"))?;
    let mut s = Server {
        process: proc,
        port,
        base: format!("http://127.0.0.1:{port}"),
        log,
    };
    s.wait_ready("/readyz")?;
    require(
        fs::read_link(format!("/proc/{}/exe", s.process.id()))? == binary.canonicalize()?,
        "traced process identity",
    )?;
    let status = fs::read_to_string(format!("/proc/{}/status", s.process.id()))?;
    let tracer = status
        .lines()
        .find_map(|l| l.strip_prefix("TracerPid:"))
        .and_then(|s| s.trim().parse::<u32>().ok());
    require(tracer.is_some_and(|n| n > 0), "strace attachment")?;
    record_invocation(report, s.process.id(), tracer.unwrap(), port, n);
    Ok(s)
}
fn record_invocation(report: &mut Value, application: u32, tracer: u32, port: u16, n: usize) {
    report["sync_tracing"]["invocations"].as_array_mut().unwrap().push(
        json!({"application_pid":application,"tracer_pid":tracer,"trace":format!("fsync-start-{n}.trace"),"port":port})
    );
}
fn unix_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn phase_timestamps(summary: &mut Value, started: f64) {
    summary["started_unix"] = json!(started);
    summary["finished_unix"] = json!(unix_seconds());
}
/// Retain the original selected workspace/application inputs and the Rust tool.
pub fn input_hashes(root: &Path) -> Result<Value> {
    let listed = tooling::execute(
        root,
        "git".as_ref(),
        &[
            OsString::from("ls-files"),
            "--cached".into(),
            "--others".into(),
            "--exclude-standard".into(),
            "-z".into(),
        ],
        true,
    )?;
    let mut hashes = smoke::rust_sources(root)?;
    for bytes in listed.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let p = std::str::from_utf8(bytes).map_err(|_| fail("Board source path"))?;
        if (p.ends_with(".rs")
            || p.ends_with("Cargo.toml")
            || [
                "Cargo.lock",
                "rust-toolchain.toml",
                "scripts/smoke_browser.py",
            ]
            .contains(&p)
            || (p.starts_with("examples/lab-operations-board/") && !p.ends_with(".md")))
            && root.join(p).is_file()
        {
            hashes[p] = json!(tooling::sha(&fs::read(root.join(p))?));
        }
    }
    Ok(hashes)
}
#[derive(Clone)]
struct Streams {
    connections: [usize; 10],
    logins: [usize; 10],
    active: usize,
    events: Vec<Value>,
    errors: Vec<String>,
}
fn stream(port: u16, cookie: &str, stop: &AtomicBool, mut ready: impl FnMut()) -> Result<()> {
    let mut s = TcpStream::connect(("127.0.0.1", port))?;
    s.set_read_timeout(Some(Duration::from_secs(12)))?;
    s.set_write_timeout(Some(Duration::from_secs(12)))?;
    s.write_all(format!("GET /board/projects/1/events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nCookie: {cookie}\r\nConnection: close\r\n\r\n").as_bytes())?;
    let mut r = BufReader::new(s);
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        require(r.read_line(&mut line)? > 0, "SSE header EOF")?;
        headers.push_str(&line);
        require(headers.len() < 65536, "SSE header bound")?;
        if line == "\r\n" {
            break;
        }
    }
    require(
        headers.starts_with("HTTP/1.1 200 ")
            && headers.to_ascii_lowercase().contains("text/event-stream"),
        "SSE status",
    )?;
    let mut pending = Vec::new();
    let mut observed = false;
    while !stop.load(Ordering::Relaxed) {
        let chunk = read_chunk(&mut r)?;
        if chunk.is_empty() {
            break;
        }
        pending.extend(chunk);
        require(pending.len() < 65536, "SSE pending bound")?;
        while let Some(n) = pending.iter().position(|b| *b == b'\n') {
            let line: Vec<_> = pending.drain(..=n).collect();
            require(line.len() < 8192, "SSE line bound")?;
            if line.starts_with(b"data:") {
                let v: Value = serde_json::from_slice(&line[5..]).map_err(|_| fail("SSE JSON"))?;
                require(v["total"] == 10000, "SSE item total")?;
                if !observed {
                    ready();
                    observed = true;
                }
            }
        }
    }
    Ok(())
}
pub fn summarize(rows: &[Value], seconds: u64, wall: f64) -> Result<Value> {
    let mut result = json!({"configured_seconds":seconds,"wall_seconds_including_login":wall,"requests":rows.len()});
    let mut unexpected = 0;
    for kind in ["read", "write"] {
        let mut v = vec![];
        for r in rows {
            if r[0] == kind {
                v.push(r[1].as_f64().ok_or_else(|| fail("raw latency"))?);
                if r[2] != if kind == "read" { 200 } else { 303 } {
                    unexpected += 1
                }
            }
        }
        result[kind] = if v.is_empty() {
            json!({"count":0,"p50_ms":null,"p95_ms":null,"p99_ms":null,"max_ms":null})
        } else {
            json!({"count":v.len(),"p50_ms":percentile(&v,0.5)?,"p95_ms":percentile(&v,0.95)?,"p99_ms":percentile(&v,0.99)?,"max_ms":v.iter().copied().fold(0.,f64::max)})
        };
    }
    let all: Vec<_> = rows.iter().filter_map(|v| v[1].as_f64()).collect();
    result["combined_p99_ms"] = if all.is_empty() {
        Value::Null
    } else {
        json!(percentile(&all, 0.99)?)
    };
    result["combined_max_ms"] = json!(all.iter().copied().reduce(f64::max));
    result["unexpected_responses"] = json!(unexpected);
    result["achieved_rps_including_login_wall"] = json!(rows.len() as f64 / wall);
    result["thresholds_pass"] = json!(
        !rows.is_empty()
            && result["read"]["count"].as_u64().unwrap_or(0) > 0
            && result["write"]["count"].as_u64().unwrap_or(0) > 0
            && result["read"]["p95_ms"].as_f64().unwrap_or(f64::INFINITY) < 250.
            && result["write"]["p95_ms"].as_f64().unwrap_or(f64::INFINITY) < 500.
            && result["combined_p99_ms"].as_f64().unwrap_or(f64::INFINITY) < 1000.
            && unexpected as f64 / (rows.len().max(1) as f64) < 0.001
    );
    Ok(result)
}
fn phase(
    base: &str,
    codes: &std::collections::BTreeMap<String, String>,
    p: &Profile,
    label: &str,
    seconds: u64,
    versions: &mut [u64; 25],
    stop: &AtomicBool,
) -> Result<(Value, Vec<Value>)> {
    let begin = Instant::now();
    let gate = Arc::new((Mutex::new(None), Condvar::new()));
    let (tx, rx) = std::sync::mpsc::channel();
    let results = std::thread::scope(|scope| {
        let mut handles = vec![];
        for (index, version) in versions.iter().copied().enumerate() {
            let gate = gate.clone();
            let tx = tx.clone();
            handles.push(scope.spawn(move || {
                let mut c = BoardClient::new(base);
                let login = c.login(codes, "alice");
                let _ = tx.send(login.is_ok());
                let (lock, cv) = &*gate;
                let mut ready = lock.lock().unwrap();
                while ready.is_none() {
                    ready = cv.wait(ready).unwrap()
                }
                let run = ready.unwrap_or(false);
                drop(ready);
                let mut rows = vec![];
                let mut version = version;
                let mut logins = 1;
                let result: Result<()> = (|| {
                    login?;
                    require(run, "Board phase login gate")?;
                    let end = Instant::now() + Duration::from_secs(seconds);
                    let mut due = Instant::now();
                    let mut login_time = Instant::now();
                    let mut op = if p.stagger { index % 5 } else { 0 };
                    while Instant::now() < end && !stop.load(Ordering::Relaxed) {
                        tooling::check_cancel()?;
                        while Instant::now() < due
                            && Instant::now() < end
                            && !stop.load(Ordering::Relaxed)
                        {
                            std::thread::sleep(
                                due.saturating_duration_since(Instant::now())
                                    .min(Duration::from_millis(50)),
                            );
                            tooling::check_cancel()?;
                        }
                        if Instant::now() >= end || stop.load(Ordering::Relaxed) {
                            break;
                        }
                        if login_time.elapsed() >= Duration::from_secs(1200) {
                            c.login(codes, "alice")?;
                            logins += 1;
                            login_time = Instant::now();
                        }
                        due = Instant::now() + Duration::from_secs_f64(1. / p.rps);
                        let write = op % 5 == 4;
                        let then = Instant::now();
                        let r = if write {
                            let fields = c.fields(&format!("Worker {index:02}"), version, "open");
                            c.call(
                                &format!("/board/projects/1/items/{}", index + 1),
                                Some(&fields),
                            )?
                        } else {
                            c.call("/board/projects/1/items", None)?
                        };
                        rows.push(json!([
                            if write { "write" } else { "read" },
                            then.elapsed().as_secs_f64() * 1000.,
                            r.status
                        ]));
                        require(
                            r.status == if write { 303 } else { 200 },
                            "unexpected response; no automatic mutation retry",
                        )?;
                        if write {
                            version += 1
                        }
                        op += 1;
                    }
                    Ok(())
                })();
                if result.is_err() {
                    stop.store(true, Ordering::Relaxed);
                }
                (rows, version, logins, result.is_err())
            }));
        }
        drop(tx);
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut ready = true;
        for _ in 0..25 {
            if rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .ok()
                != Some(true)
            {
                ready = false;
                break;
            }
        }
        let (lock, cv) = &*gate;
        *lock.lock().unwrap() = Some(ready);
        cv.notify_all();
        handles
            .into_iter()
            .map(|h| h.join().map_err(|_| fail("Board load worker panic")))
            .collect::<Result<Vec<_>>>()
    })?;
    let mut all = vec![];
    let mut errors = vec![];
    let mut logins = 0;
    for (i, (rows, v, n, error)) in results.into_iter().enumerate() {
        versions[i] = v;
        all.extend(rows);
        logins += n;
        if error {
            errors.push(json!({"client":i,"type":"operation_failed"}))
        }
    }
    let mut summary = summarize(&all, seconds, begin.elapsed().as_secs_f64())?;
    summary["name"] = json!(label);
    summary["errors"] = json!(errors);
    summary["logins"] = json!(logins);
    summary["authentication_requests_excluded_from_mix"] = json!(3 * logins);
    if !errors.is_empty() || stop.load(Ordering::Relaxed) {
        summary["thresholds_pass"] = json!(false)
    }
    Ok((summary, all))
}
pub fn run(root: &Path, name: &str) -> Result<()> {
    let p = profile(name)?;
    require(
        cfg!(target_os = "linux"),
        "Board /proc qualification requires Linux",
    )?;
    let baseout = root.join("target/m12/yt086");
    fs::create_dir_all(&baseout)?;
    let out = tempfile::Builder::new()
        .prefix(&format!("board-{name}-"))
        .tempdir_in(baseout)?
        .keep();
    println!("ARTIFACT_DIRECTORY {}", out.display());
    let temp = tempfile::Builder::new().prefix("owned-").tempdir_in(&out)?;
    let private = temp.path();
    let mut path = private.join("board.sqlite");
    let mut servers = Vec::<Server>::new();
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(Streams {
        connections: [0; 10],
        logins: [0; 10],
        active: 0,
        events: vec![],
        errors: vec![],
    }));
    let mut threads = vec![];
    let resources = Arc::new(Mutex::new(Vec::<Value>::new()));
    let started = Instant::now();
    let mut report = json!({"status":"fail","profile":name,"rounds":[],"checks":[],"cleanup":"pending","thresholds":{"read_p95_ms_below":250,"write_p95_ms_below":500,"combined_p99_ms_below":1000,"unexpected_response_fraction_below":0.001},"envelope":{"initial_items":10000,"initial_history":90000,"clients":25,"sse_clients":10,"request_pattern":"four reads then one update","max_requests_per_second_per_client":2,"selected_requests_per_second_per_client":p.rps,"warmup_seconds":p.warmup,"round_seconds":p.rounds,"soak_seconds":p.soak,"session_relogin_seconds":1200,"initial_cycle_positions":(0..25).map(|i|if p.stagger{i%5}else{0}).collect::<Vec<_>>()},"limits":["single local Linux process and SQLite","synthetic identity and data","no real OS IME or spoken AT","paced throughput, not maximum capacity","SQL queue depth: Not instrumented","bounded RSS observations do not prove leak freedom","no SLA, HA or production rollout claim"]});
    report["started_utc"] = json!(tooling::utc_now());
    report["source"] = super::docker::workspace_identity(root)?;
    let hashes = input_hashes(root)?;
    let binary_hash = tooling::sha(&fs::read(
        root.join("target/release/yetlet-lab-operations-board"),
    )?);
    report["source_sha256"] = hashes.clone();
    report["binary_sha256"] = json!(binary_hash);
    report["head"] = json!(smoke::head(root)?);
    let result: Result<()> = (|| {
        if ["full", "full-sync"].contains(&name) {
            report["retained_fault_report_sha256"] = json!(tooling::sha(&fs::read(
                root.join("target/m11-board/sustained/report.json")
            )?));
        }
        if p.trace {
            report["sync_tracing"] = json!({"tool":"strace -D -f --seccomp-bpf -qq -ttt -T -yy -e trace=fsync,fdatasync -e signal=none","invocations":[],"effect":"syscall tracing can affect timings; traced timings are not uninstrumented qualification"});
        }
        servers.push(start(root, private, &out, &path, &p, 1, &mut report)?);
        servers.last_mut().unwrap().stop(true)?;
        seed(&path)?;
        servers.push(start(root, private, &out, &path, &p, 2, &mut report)?);
        let mut current = 1;
        let mut credentials = codes(&servers[current].log)?;
        let mut versions = [9; 25];
        if name == "bounded" {
            let mut c = BoardClient::new(&servers[current].base);
            c.login(&credentials, "alice")?;
            let lock = db(rusqlite::Connection::open(&path))?;
            db(lock.execute_batch("BEGIN IMMEDIATE"))?;
            let fields = c.fields("Must not commit", 9, "open");
            require(
                c.call("/board/projects/1/items/1", Some(&fields))?.status == 503,
                "writer lock 503",
            )?;
            require(
                c.call("/healthz", None)?.status == 200,
                "health during contention",
            )?;
            db(lock.execute_batch("ROLLBACK"))?;
            let version: i64 = db(lock.query_row(
                "SELECT version FROM work_items WHERE id=1",
                [],
                |r| r.get(0),
            ))?;
            let history: i64 =
                db(lock.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)))?;
            require(version == 9 && history == 90000, "locked write no mutation")?;
            drop(lock);
            let fields = c.fields("Recovered after contention", 9, "open");
            require(
                c.call("/board/projects/1/items/1", Some(&fields))?.status == 303,
                "explicit later write",
            )?;
            let fields = c.fields("Committed after response loss", 10, "open");
            let body = fields
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        percent_encoding::utf8_percent_encode(
                            k,
                            percent_encoding::NON_ALPHANUMERIC
                        ),
                        percent_encoding::utf8_percent_encode(
                            v,
                            percent_encoding::NON_ALPHANUMERIC
                        )
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            let port = servers[current].port;
            let mut lost = TcpStream::connect(("127.0.0.1", port))?;
            lost.set_write_timeout(Some(Duration::from_secs(5)))?;
            lost.write_all(format!("POST /board/projects/1/items/1 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: {}\r\nCookie: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",c_base(&servers[current]),c.cookie,body.len()).as_bytes())?;
            smoke::wait(5, "authoritative write after response loss", || {
                let d = db(rusqlite::Connection::open(&path))?;
                let v: i64 = db(d.query_row(
                    "SELECT version FROM work_items WHERE id=1",
                    [],
                    |r| r.get(0),
                ))?;
                Ok((v == 11).then_some(()))
            })?;
            drop(lost);
            require(
                c.call("/board/projects/1/items/1", None)?
                    .text()?
                    .contains("Committed after response loss"),
                "authoritative post-loss read",
            )?;
            let fields = c.fields("Explicit stale draft", 10, "open");
            require(
                c.call("/board/projects/1/items/1", Some(&fields))?.status == 409,
                "no replay; stale draft rejected",
            )?;
            let old = c.cookie;
            servers[current].stop(true)?;
            let backup = private.join("known-good.sqlite");
            super::local::online_backup(&path, &backup, Instant::now() + Duration::from_secs(5))?;
            let incompatible = private.join("incompatible.sqlite");
            super::local::online_backup(
                &backup,
                &incompatible,
                Instant::now() + Duration::from_secs(5),
            )?;
            db(rusqlite::Connection::open(&incompatible))?
                .pragma_update(None, "user_version", 2)
                .map_err(|_| fail("future schema fixture"))?;
            let bad = start(root, private, &out, &incompatible, &p, 3, &mut report);
            require(
                bad.is_err() && super::local::expected_startup_failure(&private.join("process-3"))?,
                "incompatible schema observed nonzero startup",
            )?;
            path = backup;
            servers.push(start(root, private, &out, &path, &p, 4, &mut report)?);
            current = 2;
            credentials = codes(&servers[current].log)?;
            let mut retired = BoardClient::new(&servers[current].base);
            retired.cookie = old;
            require(
                retired.call("/board/projects/1/items/1", None)?.status == 401,
                "Memory session retired on restart",
            )?;
            let mut fresh = BoardClient::new(&servers[current].base);
            fresh.login(&credentials, "alice")?;
            require(
                fresh
                    .call("/board/projects/1/items/1", None)?
                    .text()?
                    .contains("Committed after response loss"),
                "known-good latest data",
            )?;
            versions[0] = 11;
            report["checks"] = json!([
                "actual writer-lock contention 503 without mutation, health 200, explicit later write",
                "real unread response; authoritative commit/read, stale version 409; no replay",
                "incompatible schema refuses startup; known-good rollback retains data; fresh Memory login"
            ]);
        }
        let base = servers[current].base.clone();
        let port = servers[current].port;
        let pid = servers[current].process.id();
        for index in 0..10 {
            let base = base.clone();
            let credentials = credentials.clone();
            let state = state.clone();
            let stop = stop.clone();
            threads.push(std::thread::spawn(move || {
                let mut c = BoardClient::new(&base);
                let mut login_time = None;
                while !stop.load(Ordering::Relaxed) {
                    let result: Result<()> = (|| {
                        if login_time
                            .is_none_or(|t: Instant| t.elapsed() >= Duration::from_secs(1200))
                        {
                            c.login(&credentials, "alice")?;
                            login_time = Some(Instant::now());
                            let mut s = state.lock().unwrap();
                            s.logins[index] += 1;
                            s.events.push(json!({"reader":index,"event":"login"}));
                        }
                        let mut connected=false;
                        let result=stream(port,&c.cookie,&stop,||{
                            connected=true;
                            let mut s=state.lock().unwrap();s.active+=1;s.connections[index]+=1;
                            s.events.push(json!({"reader":index,"event":"connected","elapsed_seconds":started.elapsed().as_secs_f64()}));
                        });
                        if connected {let mut s=state.lock().unwrap();s.active-=1;s.events.push(json!({"reader":index,"event":"closed","elapsed_seconds":started.elapsed().as_secs_f64()}));}
                        result
                    })();
                    if result.is_err() && !stop.load(Ordering::Relaxed) {
                        state
                            .lock()
                            .unwrap()
                            .errors
                            .push("stream_operation_failed".into());
                        stop.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }));
        }
        smoke::wait(12, "ten SSE readers", || {
            let s = state.lock().unwrap();
            require(s.errors.is_empty(), "SSE initial failure")?;
            Ok((s.active == 10).then_some(()))
        })?;
        let flag = stop.clone();
        let values = resources.clone();
        threads.push(std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                if let Ok(mut r) = metrics(pid) {
                    r["threads"] = json!(
                        fs::read_dir(format!("/proc/{pid}/task"))
                            .map(|r| r.count())
                            .ok()
                    );
                    r["fds"] = json!(
                        fs::read_dir(format!("/proc/{pid}/fd"))
                            .map(|r| r.count())
                            .ok()
                    );
                    values.lock().unwrap().push(r);
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }));
        let phases = std::iter::once(("warmup".to_string(), p.warmup, false))
            .chain(
                p.rounds
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (format!("round-{}", i + 1), *s, true)),
            )
            .chain((p.soak > 0).then_some(("soak".into(), p.soak, true)));
        let mut writes = 0;
        for (label, seconds, retain) in phases {
            println!("STAGE {label} seconds={seconds}");
            let before = metrics(pid)?;
            let started_unix = unix_seconds();
            let (mut summary, raw) = phase(
                &base,
                &credentials,
                &p,
                &label,
                seconds,
                &mut versions,
                &stop,
            )?;
            if p.trace {
                phase_timestamps(&mut summary, started_unix);
            }
            let after = metrics(pid)?;
            summary["cpu_seconds"] = json!(
                after["cpu_seconds"].as_f64().unwrap_or(0.)
                    - before["cpu_seconds"].as_f64().unwrap_or(0.)
            );
            summary["rss_before_kib"] = before["rss_kib"].clone();
            summary["rss_after_kib"] = after["rss_kib"].clone();
            writes += summary["write"]["count"].as_u64().unwrap_or(0);
            super::super::measure::output(&out.join(format!("{label}-samples.json")), &json!(raw))?;
            if retain {
                report["rounds"]
                    .as_array_mut()
                    .unwrap()
                    .push(summary.clone())
            } else {
                report["warmup"] = summary.clone()
            };
            require(
                summary["errors"].as_array().unwrap().is_empty()
                    && summary["unexpected_responses"] == 0
                    && !stop.load(Ordering::Relaxed),
                "phase errors",
            )?;
            if retain || name == "bounded" {
                require(
                    summary["thresholds_pass"] == true,
                    "strict Board latency gates",
                )?;
            }
        }
        let check = db(rusqlite::Connection::open(&path))?;
        let items: i64 = db(check.query_row("SELECT COUNT(*) FROM work_items", [], |r| r.get(0)))?;
        let history: i64 = db(check.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0)))?;
        let integrity: String = db(check.query_row("PRAGMA integrity_check", [], |r| r.get(0)))?;
        require(
            items == 10000
                && history < 100000
                && history == 90000 + if name == "bounded" { 2 } else { 0 } + writes as i64
                && integrity == "ok",
            "final data conservation/integrity",
        )?;
        report["final_data"] = json!({"items":items,"history":history});
        stop.store(true, Ordering::Relaxed);
        servers[current].stop(true)?;
        require(
            state.lock().unwrap().connections.iter().all(|n| *n >= 2),
            "ten SSE readers must reconnect after lifetime",
        )?;
        require(
            input_hashes(root)? == hashes
                && tooling::sha(&fs::read(
                    root.join("target/release/yetlet-lab-operations-board"),
                )?) == binary_hash,
            "source and binary unchanged",
        )?;
        report["source_and_binary_unchanged"] = json!(true);
        Ok(())
    })();
    stop.store(true, Ordering::Relaxed);
    let mut cleanup = true;
    for s in servers.iter_mut().rev() {
        if s.stop(false).is_err() {
            cleanup = false
        }
    }
    for t in threads {
        if t.join().is_err() {
            cleanup = false
        }
    }
    let snapshot = state.lock().unwrap().clone();
    report["stream_connections"] = json!(snapshot.connections);
    report["stream_logins"] = json!(snapshot.logins);
    report["stream_events"] = json!(snapshot.events);
    report["active_streams"] = json!(snapshot.active);
    report["stream_errors"] = json!(snapshot.errors);
    report["resources"] = json!(*resources.lock().unwrap());
    report["process_exits"] = super::local::exit_receipts(private)?;
    let removed = temp.close().is_ok();
    cleanup = cleanup && removed && snapshot.active == 0 && snapshot.errors.is_empty();
    report["cleanup"] = json!(if cleanup {
        "owned process reaped, threads joined, listener closed, private synthetic databases/logs removed"
    } else {
        "failed"
    });
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["finished_utc"] = json!(tooling::utc_now());
    let success = result.is_ok() && cleanup && tooling::check_cancel().is_ok();
    if success {
        report["status"] = json!("pass")
    } else {
        report["failure_type"] = json!(result.as_ref().err().map(|e| e.code));
    }
    super::super::measure::output(&out.join("report.json"), &report)?;
    let mut sums = String::new();
    let mut files = fs::read_dir(&out)?.collect::<std::io::Result<Vec<_>>>()?;
    files.sort_by_key(|e| e.path());
    for e in files {
        if e.path().extension().is_some_and(|s| s == "json") {
            sums.push_str(&format!(
                "{}  {}\n",
                tooling::sha(&fs::read(e.path())?),
                e.file_name().to_string_lossy()
            ));
        }
    }
    fs::write(out.join("SHA256SUMS"), sums)?;
    if success {
        Ok(())
    } else {
        Err(fail("Board qualification failed; retained raw evidence"))
    }
}
fn c_base(s: &Server) -> &str {
    &s.base
}

#[cfg(test)]
mod trace_tests {
    use super::*;
    #[test]
    fn trace_invocations_and_phase_times_are_correlated() {
        let mut report = json!({"sync_tracing":{"invocations":[]}});
        record_invocation(&mut report, 1201, 1202, 39001, 1);
        record_invocation(&mut report, 1203, 1204, 39001, 2);
        assert_eq!(
            report["sync_tracing"]["invocations"],
            json!([
                {"application_pid":1201,"tracer_pid":1202,"trace":"fsync-start-1.trace","port":39001},
                {"application_pid":1203,"tracer_pid":1204,"trace":"fsync-start-2.trace","port":39001}
            ])
        );
        let mut end = 0.;
        for name in ["warmup", "round-1", "round-2", "soak"] {
            let start = unix_seconds();
            let mut phase = json!({"name":name});
            phase_timestamps(&mut phase, start);
            assert!(phase["started_unix"].as_f64().unwrap() >= end);
            end = phase["finished_unix"].as_f64().unwrap();
            assert!(end >= start);
            assert!(start > 1_700_000_000.);
        }
    }
}
