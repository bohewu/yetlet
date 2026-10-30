//! Disposable pinned Docker labs; cleanup queries exact project labels and owned names.
use crate::{
    process::{self, Spec},
    smoke::{self, http, require},
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
pub const RUST_IMAGE: &str =
    "rust@sha256:b36c246742b4d323472588f789601be34eb4af053255e2af53969078ee8dcec7";
pub const NGINX_IMAGE: &str =
    "nginx@sha256:0985e772fb9f729e6fa0980da05fca5d9c468e870eed43071545afa9d2e27d94";
pub const REDIS_IMAGE: &str =
    "redis@sha256:47670742d7924adbcdb404d1288b6327815b23141969c0939b8e5d61f78c2634";
fn posix_user() -> Result<String> {
    #[cfg(unix)]
    {
        Ok(format!("{}:{}", unsafe { libc::getuid() }, unsafe {
            libc::getgid()
        }))
    }
    #[cfg(not(unix))]
    {
        Err(fail(
            "Docker qualification is unsupported on this platform: POSIX user identity is required",
        ))
    }
}
pub struct Lab {
    pub root: PathBuf,
    pub redis: bool,
    pub project: String,
    pub build_container: String,
    pub image: String,
    pub base: String,
    pub port: u16,
    pub work: PathBuf,
    pub logs: PathBuf,
    pub namespace: String,
    pub env: Vec<(OsString, Option<OsString>)>,
    pub started: bool,
}
impl Lab {
    pub fn new(root: &Path, redis: bool) -> Result<Self> {
        #[cfg(not(unix))]
        posix_user()?;
        let parent = root.join("target/m12/yt086");
        fs::create_dir_all(&parent)?;
        let work = tempfile::Builder::new()
            .prefix("docker-work-")
            .tempdir_in(&parent)?
            .keep();
        let token = work
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace("docker-work-", "")
            .to_ascii_lowercase();
        let project = format!(
            "yetlet-{}-{}-{token}",
            if redis { "yt066b3" } else { "yt066a" },
            std::process::id()
        );
        let image = format!("{project}:local");
        let port = smoke::free_port()?;
        let prefix = if redis { "YT066B3" } else { "YT066A" };
        let namespace = format!("b3_{token}");
        let mut env = vec![
            (
                format!("{prefix}_APP_IMAGE").into(),
                Some(image.clone().into()),
            ),
            (
                format!("{prefix}_HOST_PORT").into(),
                Some(port.to_string().into()),
            ),
        ];
        if redis {
            env.push(("YT066B3_NAMESPACE".into(), Some(namespace.clone().into())))
        } else {
            env.extend([
                ("YT066A_ALICE_CODE".into(), Some("a".repeat(64).into())),
                ("YT066A_BOB_CODE".into(), Some("b".repeat(64).into())),
            ]);
        }
        let logs = tempfile::Builder::new()
            .prefix("docker-logs-")
            .tempdir_in(parent)?
            .keep();
        Ok(Self {
            root: root.into(),
            redis,
            build_container: format!("{project}-build"),
            project,
            image,
            base: format!("http://127.0.0.1:{port}"),
            port,
            work,
            logs,
            namespace,
            env,
            started: false,
        })
    }
    pub fn compose_args(&self, args: &[&str]) -> Vec<OsString> {
        let mut a: Vec<OsString> = [
            "compose",
            "--ansi",
            "never",
            "--project-name",
            &self.project,
            "--file",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        a.push(
            self.root
                .join(if self.redis {
                    "qualification/yt066b3/compose.yaml"
                } else {
                    "qualification/yt066a/compose.yaml"
                })
                .into_os_string(),
        );
        a.extend(args.iter().map(OsString::from));
        a
    }
    pub fn command(&self, args: &[OsString], seconds: u64, cleanup: bool) -> process::Report {
        let never = AtomicBool::new(false);
        process::run(
            &Spec {
                program: "docker".into(),
                args: args.to_vec(),
                cwd: self.root.clone(),
                env: self.env.clone(),
                timeout: Duration::from_secs(seconds),
                output_limit: 16 * 1024 * 1024,
            },
            if cleanup { &never } else { &process::CANCELLED },
        )
    }
    pub fn checked(&self, args: &[OsString], seconds: u64) -> Result<String> {
        let r = self.command(args, seconds, false);
        require(
            r.success() && r.stdout.discarded == 0 && r.stderr.discarded == 0,
            "Docker operation failed; output retained privately",
        )?;
        String::from_utf8(r.stdout.bytes).map_err(|_| fail("Docker output encoding"))
    }
    pub fn compose(&self, args: &[&str], seconds: u64) -> Result<String> {
        self.checked(&self.compose_args(args), seconds)
    }
    fn logged(&self, args: &[OsString], label: &str, seconds: u64) -> Result<()> {
        let r = self.command(args, seconds, false);
        let mut log = r.stdout.bytes.clone();
        log.extend(&r.stderr.bytes);
        let path = self.logs.join(format!("{label}.log"));
        fs::write(&path, log)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        require(
            r.success() && r.stdout.discarded == 0 && r.stderr.discarded == 0,
            "Docker logged operation failed",
        )
    }
    pub fn build(&self) -> Result<Value> {
        let user = posix_user()?;
        let build = self.work.join("build");
        let context = self.work.join("image");
        fs::create_dir(&build)?;
        fs::create_dir(&context)?;
        let vendor = if self.redis {
            let path = self.work.join("vendor");
            tooling::execute(
                &self.root,
                &tooling::tool("cargo", ".cargo/bin")?,
                &[
                    "vendor".into(),
                    "--locked".into(),
                    "--offline".into(),
                    "--manifest-path".into(),
                    self.root
                        .join("qualification/yt066b3/Cargo.toml")
                        .into_os_string(),
                    path.clone().into_os_string(),
                ],
                true,
            )?;
            path
        } else {
            let p = self.root.join("target/m10-delivery/docker/vendor");
            require(p.is_dir(), "retained locked vendor directory missing")?;
            p
        };
        let command = if self.redis {
            "cargo build --release --locked --offline --manifest-path qualification/yt066b3/Cargo.toml"
        } else {
            "cargo build --release --locked --offline -p yetlet-auth-notes"
        };
        let script = format!(
            "set -euo pipefail\nexport CARGO_HOME=/tmp/cargo\nmkdir -p \"$CARGO_HOME\"\nprintf '[source.crates-io]\\nreplace-with=\"vendored-sources\"\\n[source.vendored-sources]\\ndirectory=\"/vendor\"\\n' > \"$CARGO_HOME/config.toml\"\nrustc -vV\ncargo --version\n{command}\n"
        );
        let mut args: Vec<OsString> = [
            "run",
            "--rm",
            "--name",
            &self.build_container,
            "--label",
            "yetlet.qualification=yt066-build",
            "--network",
            "none",
            "--read-only",
            "--user",
            &user,
            "--cpus",
            "4",
            "--memory",
            "6g",
            "--pids-limit",
            "384",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--tmpfs",
            "/tmp:rw,exec,nosuid,nodev,size=1073741824,mode=1777",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        for (src, dst) in [
            (&self.root, "/src"),
            (&build, "/build"),
            (&vendor, "/vendor"),
        ] {
            args.extend([
                "--mount".into(),
                format!(
                    "type=bind,src={},dst={dst}{}",
                    src.display(),
                    if dst == "/build" { "" } else { ",readonly" }
                )
                .into(),
            ]);
        }
        args.extend([
            "--env".into(),
            "CARGO_TARGET_DIR=/build".into(),
            "--workdir".into(),
            "/src".into(),
            RUST_IMAGE.into(),
            "bash".into(),
            "-ceu".into(),
            script.into(),
        ]);
        self.logged(&args, "app-build", 900)?;
        let name = if self.redis {
            "yetlet-redis-qualification"
        } else {
            "yetlet-auth-notes"
        };
        let binary = context.join(name);
        fs::copy(build.join("release").join(name), &binary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o555))?;
        }
        let args = vec![
            "build".into(),
            "--network=none".into(),
            "--pull=false".into(),
            "--label".into(),
            "yetlet.qualification=yt066-app".into(),
            "--tag".into(),
            self.image.clone().into(),
            "--file".into(),
            self.root
                .join(if self.redis {
                    "qualification/yt066b3/Dockerfile"
                } else {
                    "qualification/yt066a/Dockerfile"
                })
                .into_os_string(),
            context.into_os_string(),
        ];
        self.logged(&args, "image-build", 300)?;
        Ok(
            json!({"binary_sha256":tooling::sha(&fs::read(binary)?),"image_id":self.checked(&["image".into(),"inspect".into(),self.image.clone().into(),"--format".into(),"{{.Id}}".into()],120)?.trim()}),
        )
    }
    pub fn start(&mut self) -> Result<Value> {
        if !self.redis {
            self.logged(&["pull".into(), NGINX_IMAGE.into()], "proxy-pull", 300)?;
        }
        self.compose(&["config", "--quiet"], 120)?;
        self.started = true;
        self.compose(&["up", "--detach"], 180)?;
        for x in ["a", "b", "c"] {
            self.ready(x)?;
        }
        self.topology()
    }
    pub fn ready(&self, x: &str) -> Result<()> {
        smoke::wait(20, "owned application readiness", || {
            let r = self.request(x, "/healthz", "GET", None, None).ok();
            Ok(r.filter(|r| {
                r.status == 200
                    && if self.redis {
                        r.body == x.as_bytes()
                    } else {
                        r.body == b"ok"
                    }
            })
            .map(|_| ()))
        })
    }
    pub fn request(
        &self,
        x: &str,
        path: &str,
        method: &str,
        cookie: Option<&str>,
        hold: Option<&str>,
    ) -> Result<http::Response> {
        let prefix = if self.redis {
            "/__yt066b3"
        } else {
            "/__yt066a"
        };
        let path = if x.is_empty() {
            path.into()
        } else {
            format!("{prefix}/{x}{path}")
        };
        let mut headers = vec![];
        if let Some(c) = cookie {
            headers.push(("Cookie", c))
        }
        if let Some(h) = hold {
            headers.push(("X-Lab-Hold", h))
        }
        http::call_timeout(&self.base, method, &path, None, &headers, 5)
    }
    pub fn topology(&self) -> Result<Value> {
        let mut details = json!({});
        let mut services = vec!["app-a", "app-b", "app-c", "proxy"];
        if self.redis {
            services.push("redis")
        }
        for s in &services {
            let id = self.compose(&["ps", "--quiet", s], 120)?;
            require(!id.trim().is_empty(), "container ID missing")?;
            let r = self.checked(&["inspect".into(), id.trim().into()], 120)?;
            details[*s] = serde_json::from_str::<Value>(&r)
                .map_err(|_| fail("container inspect JSON"))?[0]
                .clone();
        }
        let backend = format!("{}_lab", self.project);
        for s in services.iter().filter(|s| **s != "proxy") {
            let n = &details[*s]["NetworkSettings"];
            require(
                n["Ports"].is_null()
                    || n["Ports"].as_object().is_some_and(|o| {
                        o.values()
                            .all(|v| v.is_null() || v.as_array().is_some_and(|a| a.is_empty()))
                    }),
                "backend host publication",
            )?;
            let networks = n["Networks"]
                .as_object()
                .ok_or_else(|| fail("backend network shape"))?;
            require(
                networks.len() == 1 && networks.contains_key(&backend),
                "backend network shape",
            )?;
            require(
                details[*s]["HostConfig"]["ReadonlyRootfs"] == true,
                "backend writable root",
            )?;
        }
        require(
            details["proxy"]["NetworkSettings"]["Ports"]["8080/tcp"]
                == json!([{"HostIp":"127.0.0.1","HostPort":self.port.to_string()}]),
            "proxy loopback binding",
        )?;
        let networks = details["proxy"]["NetworkSettings"]["Networks"]
            .as_object()
            .ok_or_else(|| fail("proxy network shape"))?;
        require(
            networks.len() == 2
                && networks.contains_key(&backend)
                && networks.contains_key(&format!("{}_front", self.project)),
            "proxy network shape",
        )?;
        for (name, internal) in [
            (&backend, true),
            (&format!("{}_front", self.project), false),
        ] {
            let v: Value = serde_json::from_str(
                &self.checked(&["network".into(), "inspect".into(), name.into()], 120)?,
            )
            .map_err(|_| fail("network JSON"))?;
            require(v[0]["Internal"] == internal, "network isolation")?;
        }
        if self.redis {
            redis_no_persistence_with(|key| {
                self.compose(
                    &[
                        "exec",
                        "-T",
                        "redis",
                        "redis-cli",
                        "--raw",
                        "CONFIG",
                        "GET",
                        key,
                    ],
                    120,
                )
            })?;
        }
        Ok(
            json!({"applications":3,"application_host_ports":0,"proxy_host_ip":"127.0.0.1","backend_network_internal":true,"front_network_internal":false,"redis_persistence":if self.redis{"none"}else{"not_applicable"}}),
        )
    }
    pub fn cleanup(&self) -> Value {
        cleanup_with(
            &self.project,
            &self.build_container,
            &self.image,
            &self.compose_args(&["down", "--volumes", "--remove-orphans", "--timeout", "5"]),
            |args, timeout| {
                let r = self.command(args, timeout, true);
                if r.stdout.discarded > 0 || r.stderr.discarded > 0 {
                    return None;
                }
                Some((
                    r.success(),
                    String::from_utf8_lossy(&r.stdout.bytes).into_owned(),
                ))
            },
        )
    }
}
pub fn redis_no_persistence_with(mut config: impl FnMut(&str) -> Result<String>) -> Result<()> {
    require(
        config("appendonly")?.trim_end() == "appendonly\nno",
        "Redis AOF must be disabled",
    )?;
    require(
        config("save")?.trim_end_matches(['\r', '\n']) == "save",
        "Redis RDB schedule must be empty",
    )
}
pub fn cleanup_with(
    project: &str,
    build: &str,
    image: &str,
    down: &[OsString],
    mut run: impl FnMut(&[OsString], u64) -> Option<(bool, String)>,
) -> Value {
    let mut actions = vec![];
    let mut call = |args: Vec<OsString>, label: &str, timeout| {
        let r = run(&args, timeout);
        actions.push(json!({"action":label,"success":r.as_ref().map(|r|r.0)}));
        r
    };
    let filter = format!("label=com.docker.compose.project={project}");
    call(down.to_vec(), "compose_down", 120);
    call(
        vec!["rm".into(), "--force".into(), build.into()],
        "build_container_remove",
        60,
    );
    let queries: Vec<(&str, Vec<OsString>)> = vec![
        (
            "containers",
            vec![
                "ps".into(),
                "--all".into(),
                "--quiet".into(),
                "--filter".into(),
                filter.clone().into(),
            ],
        ),
        (
            "networks",
            vec![
                "network".into(),
                "ls".into(),
                "--quiet".into(),
                "--filter".into(),
                filter.clone().into(),
            ],
        ),
        (
            "volumes",
            vec![
                "volume".into(),
                "ls".into(),
                "--quiet".into(),
                "--filter".into(),
                filter.into(),
            ],
        ),
    ];
    for (name, q) in &queries {
        if let Some((true, ids)) = call(q.clone(), &format!("{name}_list"), 60) {
            let ids: Vec<_> = ids.split_whitespace().collect();
            if !ids.is_empty() {
                let mut args: Vec<OsString> = match *name {
                    "containers" => vec!["rm".into(), "--force".into()],
                    "networks" => vec!["network".into(), "rm".into()],
                    _ => vec!["volume".into(), "rm".into(), "--force".into()],
                };
                args.extend(ids.into_iter().map(Into::into));
                call(args, &format!("forced_{name}_remove"), 60);
            }
        }
    }
    call(
        vec!["image".into(), "rm".into(), "--force".into(), image.into()],
        "app_image_remove",
        60,
    );
    let mut queries = queries;
    queries.extend([
        (
            "build_container",
            vec![
                "ps".into(),
                "--all".into(),
                "--quiet".into(),
                "--filter".into(),
                format!("name=^{build}$").into(),
            ],
        ),
        (
            "application_image",
            vec![
                "image".into(),
                "ls".into(),
                "--quiet".into(),
                "--no-trunc".into(),
                image.into(),
            ],
        ),
    ]);
    let mut remaining = json!({});
    let mut complete = true;
    let mut clean = true;
    for (name, q) in queries {
        match call(q, &format!("{name}_recheck"), 60) {
            Some((true, ids)) => {
                let n = ids.split_whitespace().count();
                remaining[name] = json!(n);
                clean &= n == 0
            }
            _ => {
                remaining[name] = Value::Null;
                complete = false;
                clean = false
            }
        }
    }
    json!({"status":if clean{"pass"}else{"failed"},"verification_complete":complete,"remaining":remaining,"actions":actions})
}
pub fn last_upstream(s: &str) -> Result<&str> {
    s.split(',')
        .map(str::trim)
        .rfind(|s| !s.is_empty())
        .ok_or_else(|| fail("empty upstream header"))
}
fn pool(l: &Lab, count: usize) -> Result<BTreeSet<String>> {
    let mut set = BTreeSet::new();
    for _ in 0..count {
        let r = l.request("", "/healthz", "GET", None, None)?;
        require(
            r.status == 200 && r.body == b"ok" && !r.headers.contains_key("set-cookie"),
            "stateless pool health",
        )?;
        set.insert(last_upstream(r.header("x-yetlet-upstream")?)?.into());
    }
    Ok(set)
}
fn rejected(l: &Lab, x: &str, c: &str) -> Result<()> {
    let r = l.request(x, "/auth/notes", "GET", Some(c), None)?;
    require(
        r.status == 401 && r.header("cache-control")? == "no-store",
        "Memory rejection/private cache",
    )
}
fn exercise(l: &Lab) -> Result<Value> {
    require(pool(l, 18)?.len() == 3, "three upstreams")?;
    let path = "/__yt066a/a/auth/login";
    let r = http::get(&l.base, path)?;
    require(
        r.status == 200 && r.header("cache-control")? == "no-store",
        "login page",
    )?;
    let csrf = r.hidden("csrf")?;
    let old = r.cookie()?;
    let r = http::form(
        &l.base,
        path,
        &old,
        &[
            ("csrf", &csrf),
            ("username", "alice"),
            ("code", &"a".repeat(64)),
        ],
    )?;
    require(r.status == 303, "login")?;
    let c = r.cookie()?;
    require(c != old, "Session rotation")?;
    let r = l.request("a", "/auth/notes", "GET", Some(&c), None)?;
    require(
        r.status == 200 && r.header("cache-control")? == "no-store",
        "issuer private read",
    )?;
    rejected(l, "b", &c)?;
    rejected(l, "c", &c)?;
    let stop = Instant::now();
    l.compose(&["stop", "--timeout", "5", "app-a"], 30)?;
    require(pool(l, 18)?.len() == 2, "two survivors")?;
    let degraded = stop.elapsed().as_secs_f64();
    let restart = Instant::now();
    l.compose(&["start", "app-a"], 30)?;
    l.ready("a")?;
    std::thread::sleep(Duration::from_millis(1100));
    let mut observed = BTreeSet::new();
    smoke::wait(10, "rejoined instance", || {
        observed.extend(pool(l, 12)?);
        Ok((observed.len() == 3).then_some(()))
    })?;
    rejected(l, "a", &c)?;
    Ok(
        json!({"stateless":{"initial_unique_upstreams":3,"degraded_unique_upstreams":2,"recovered_unique_upstreams":3,"requests_per_sample":18,"degraded_service_seconds":degraded,"recovery_seconds":restart.elapsed().as_secs_f64()},"memory_session":{"issuing_instance_status":200,"cross_instance_statuses":{"app_b":401,"app_c":401},"old_cookie_after_issuing_instance_restart":401,"shared_across_instances":false,"persistent_across_restart":false,"private_responses_no_store":true}}),
    )
}
pub fn run(root: &Path, redis: bool, report_path: Option<PathBuf>) -> Result<()> {
    let mut lab = Lab::new(root, redis)?;
    let path = report_path.unwrap_or_else(|| {
        lab.logs.join(if redis {
            "redis-three-app.json"
        } else {
            "multi-instance.json"
        })
    });
    let absolute = std::env::current_dir()?.join(&path);
    let rel = absolute
        .strip_prefix(root)
        .map_err(|_| fail("report must stay inside repository"))?;
    let path = tooling::safe(root, rel, true)?;
    require(
        !path.exists(),
        "existing qualification report; refusing overwrite",
    )?;
    let mut report = json!({"task_id":if redis{"YT-066b3"}else{"YT-066a"},"port_task":"YT-086","status":"failed","stage":"preflight","images":{"rust":RUST_IMAGE,"nginx":NGINX_IMAGE,"redis":if redis{Some(REDIS_IMAGE)}else{None}},"qualification":{"distributed_session":"not_verified","redis":if redis{"selected_single_host_only"}else{"not_started"},"cross_host_ha":"not_verified","load_or_slo":"not_verified","production_recovery":"not_verified"},"source":workspace_identity(root)?});
    run_with(
        &mut lab,
        &path,
        &mut report,
        |lab, report| {
            let result: Result<()> = (|| {
                lab.checked(
                    &[
                        "version".into(),
                        "--format".into(),
                        "{{.Server.Version}}".into(),
                    ],
                    120,
                )?;
                lab.checked(&["compose".into(), "version".into()], 120)?;
                report["stage"] = json!("build");
                report["application_image"] = lab.build()?;
                report["stage"] = json!("topology");
                report["topology"] = lab.start()?;
                report["stage"] = json!("exercise");
                report["observations"] = if lab.redis {
                    super::redis::exercise(lab)?
                } else {
                    exercise(lab)?
                };
                if lab.redis {
                    report["stage"] = json!("browser");
                    smoke::fixtures::run(root, &lab.logs, &lab.base, true)?;
                }
                Ok(())
            })();
            if result.is_err() && lab.started {
                let mut diagnostics = json!({});
                for (label, args) in [
                    ("compose_ps", vec!["ps", "--all", "--format", "json"]),
                    (
                        "compose_logs",
                        vec!["logs", "--no-color", "--no-log-prefix"],
                    ),
                ] {
                    let r = lab.command(&lab.compose_args(&args), 30, true);
                    let mut bytes = r.stdout.bytes.clone();
                    bytes.extend(&r.stderr.bytes);
                    diagnostics[label] = json!({"exit":r.exit_code(),"captured":fs::write(lab.logs.join(format!("failure-{label}.log")),bytes).is_ok()});
                }
                report["failure_diagnostics"] = diagnostics;
            }
            result
        },
        Lab::cleanup,
        process::cancellation_signal,
        super::local::write_report,
    )
}

/// Runtime orchestration with injected scenario/commands for direct lifecycle fixtures.
pub fn run_with(
    lab: &mut Lab,
    path: &Path,
    report: &mut Value,
    scenario: impl FnOnce(&mut Lab, &mut Value) -> Result<()>,
    cleanup: impl FnOnce(&Lab) -> Value,
    cancellation_signal: impl Fn() -> Option<i32>,
    write: impl FnMut(&Path, &Value) -> Result<()>,
) -> Result<()> {
    let started = Instant::now();
    let result = scenario(lab, report);
    report["cleanup"] = cleanup(lab);
    let work_cleanup = fs::remove_dir_all(&lab.work).is_ok() && !lab.work.exists();
    report["owned_work_removed"] = json!(work_cleanup);
    report["duration_seconds"] = json!(started.elapsed().as_secs_f64());
    let was_cancelled = cancellation_signal().is_some();
    let success = result.is_ok()
        && report["cleanup"]["status"] == "pass"
        && report["cleanup"]["verification_complete"] == true
        && work_cleanup
        && !was_cancelled;
    report["outcome"] = json!(if was_cancelled {
        "cancelled"
    } else if result.is_ok() {
        "success"
    } else {
        "operation_failed"
    });
    if success {
        report["status"] = json!(if lab.redis {
            "pass-at-single-host-no-persistence-scope"
        } else {
            "pass-for-docker-memory"
        });
        report["stage"] = json!("complete");
    } else {
        report["failure_code"] = json!(result.as_ref().err().map(|e| e.code));
    }
    super::local::finalize_report(
        path,
        report,
        success,
        if lab.redis {
            "pass-at-single-host-no-persistence-scope"
        } else {
            "pass-for-docker-memory"
        },
        |report| {
            if let Some(signal) = cancellation_signal() {
                report["signal"] = json!(signal);
                report["failure_type"] = json!(if signal == 2 {
                    "KeyboardInterrupt"
                } else {
                    "SIGTERM"
                });
                true
            } else {
                false
            }
        },
        write,
    )
}

/// Same filename/NUL/type/bytes digest used by the original Docker source record.
pub fn workspace_identity(root: &Path) -> Result<Value> {
    use sha2::{Digest, Sha256};
    let git = |args: &[&str]| {
        tooling::execute(
            root,
            "git".as_ref(),
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            true,
        )
    };
    let head = git(&["rev-parse", "HEAD"])?;
    let listed = git(&["ls-files", "-co", "--exclude-standard", "-z"])?;
    let paths: std::collections::BTreeSet<_> = listed
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .collect();
    let mut digest = Sha256::new();
    let mut count = 0;
    for relative in paths {
        let name = std::str::from_utf8(relative).map_err(|_| fail("source path encoding"))?;
        let path = root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        digest.update(relative);
        digest.update(b"\0");
        if metadata.file_type().is_symlink() {
            digest.update(b"symlink\0");
            digest.update(
                fs::read_link(&path)?
                    .to_str()
                    .ok_or_else(|| fail("symlink encoding"))?
                    .as_bytes(),
            );
        } else if metadata.is_file() {
            digest.update(b"file\0");
            digest.update(fs::read(path)?);
        }
        digest.update(b"\0");
        count += 1;
    }
    let status = git(&["status", "--porcelain", "--untracked-files=all"])?;
    Ok(
        json!({"head":String::from_utf8_lossy(&head).trim(),"working_source_sha256":format!("{:x}",digest.finalize()),"source_paths":count,"dirty":!status.is_empty()}),
    )
}
