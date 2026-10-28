//! Manual CI preparation only. No signing, publication, overlays or shared package caches.
use crate::{
    process::{self, Spec},
    smoke::require,
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
pub(super) const PACKAGE_VERSION: &str = "0.9.0-alpha.1";
pub const NAMES: [&str; 9] = [
    "yetlet-core",
    "yetlet-actions",
    "yetlet-reactive",
    "yetlet-html",
    "yetlet-dom",
    "yetlet-dom-keyed",
    "yetlet-server-axum",
    "yetlet-request-id",
    "yetlet-session",
];
pub const SIXTEEN_NAMES: [&str; 16] = [
    "yetlet-core",
    "yetlet-actions",
    "yetlet-reactive",
    "yetlet-html",
    "yetlet-dom",
    "yetlet-dom-keyed",
    "yetlet-server-axum",
    "yetlet-request-id",
    "yetlet-session",
    "yetlet-auth",
    "yetlet-rate-limit",
    "yetlet-config",
    "yetlet-secrets",
    "yetlet-signing",
    "yetlet-session-redis",
    "yetlet-wgpu",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cohort {
    OriginalNine,
    Sixteen,
}
impl Cohort {
    pub fn parse(args: &[OsString]) -> std::result::Result<Self, &'static str> {
        match args {
            [] => Ok(Self::OriginalNine),
            [flag, value] if flag == "--cohort" && value == "original-nine" => {
                Ok(Self::OriginalNine)
            }
            [flag, value] if flag == "--cohort" && value == "sixteen" => Ok(Self::Sixteen),
            _ => Err("private-rebuild [--cohort original-nine|sixteen]"),
        }
    }
    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::OriginalNine => &NAMES,
            Self::Sixteen => &SIXTEEN_NAMES,
        }
    }
    pub fn manifests(self) -> &'static [&'static str] {
        match self {
            Self::OriginalNine => &["Cargo.toml"],
            Self::Sixteen => &[
                "Cargo.toml",
                "crates/yetlet-config/Cargo.toml",
                "crates/yetlet-secrets/Cargo.toml",
                "crates/yetlet-signing/Cargo.toml",
                "crates/yetlet-session-redis/Cargo.toml",
                "crates/yetlet-wgpu/Cargo.toml",
            ],
        }
    }
}
fn command(root: &Path, args: &[&str]) -> Result<String> {
    let b = tooling::execute(
        root,
        "git".as_ref(),
        &args.iter().map(OsString::from).collect::<Vec<_>>(),
        true,
    )?;
    String::from_utf8(b).map_err(|_| fail("git encoding"))
}
fn env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| fail(format!("required CI identity {name} unavailable")))
}
pub fn archive_path(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(['\\', '\0', '\n', '\r'])
        && s.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
        && !s.starts_with('/')
}
pub fn aliases() -> BTreeMap<String, String> {
    NAMES
        .into_iter()
        .flat_map(|n| {
            ["LICENSE-MIT", "LICENSE-APACHE"]
                .into_iter()
                .map(move |license| (format!("crates/{n}/{license}"), format!("../../{license}")))
        })
        .collect()
}
fn visit(base: &Path, p: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
    for e in fs::read_dir(p)? {
        let p = e?.path();
        let m = fs::symlink_metadata(&p)?;
        require(!m.file_type().is_symlink(), "selected input symlink")?;
        if m.is_dir() {
            visit(base, &p, out)?
        } else {
            require(m.is_file(), "selected input type")?;
            out.insert(
                p.strip_prefix(base)
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| fail("selected path encoding"))?
                    .replace('\\', "/"),
                tooling::sha(&fs::read(p)?),
            );
        }
    }
    Ok(())
}
pub fn selected(source: &Path, cohort: Cohort) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for f in [
        "Cargo.toml",
        "Cargo.lock",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "rust-toolchain.toml",
    ] {
        out.insert(f.into(), tooling::sha(&fs::read(source.join(f))?));
    }
    for n in cohort.names() {
        visit(source, &source.join("crates").join(n), &mut out)?;
    }
    Ok(out)
}
fn write(out: &Path, name: &str, value: &Value) -> Result<()> {
    fs::write(out.join(name), tooling::json_bytes(value))?;
    Ok(())
}
/// Fixed Cargo steps; bootstrap collection and normal verification are distinct.
pub fn cargo_steps(source: &Path, work: &Path, cohort: Cohort) -> Vec<(String, Vec<OsString>)> {
    let target = work.join("cargo-target");
    if cohort == Cohort::OriginalNine {
        let fetch = vec![
            "fetch".into(),
            "--locked".into(),
            "--manifest-path".into(),
            source.join("Cargo.toml").into_os_string(),
        ];
        let mut package = vec![
            "package".into(),
            "--locked".into(),
            "--offline".into(),
            "--manifest-path".into(),
            source.join("Cargo.toml").into_os_string(),
            "--target-dir".into(),
            target.into_os_string(),
        ];
        for n in NAMES {
            package.extend(["-p".into(), n.into()]);
        }
        return vec![("fetch".into(), fetch), ("package".into(), package)];
    }
    let mut vendor = vec![
        "vendor".into(),
        "--locked".into(),
        "--manifest-path".into(),
        source.join("Cargo.toml").into_os_string(),
    ];
    for m in &cohort.manifests()[1..] {
        vendor.extend(["--sync".into(), source.join(m).into_os_string()]);
    }
    vendor.extend([
        "--versioned-dirs".into(),
        work.join("vendor").into_os_string(),
    ]);
    let mut steps = vec![("vendor".into(), vendor)];
    for bootstrap in [true, false] {
        for n in cohort.names() {
            let mut args = vec![
                "package".into(),
                "--offline".into(),
                "--locked".into(),
                "--manifest-path".into(),
                source
                    .join("crates")
                    .join(n)
                    .join("Cargo.toml")
                    .into_os_string(),
                "--target-dir".into(),
                work.join(if bootstrap {
                    "bootstrap-target"
                } else {
                    "cargo-target"
                })
                .into_os_string(),
                "--config".into(),
                work.join("vendor-config.toml").into_os_string(),
            ];
            if bootstrap {
                args.extend(["--no-verify".into(), "--exclude-lockfile".into()]);
            }
            steps.push((
                format!("{}-{n}", if bootstrap { "bootstrap" } else { "package" }),
                args,
            ));
        }
    }
    steps
}

fn stage_bootstrap(source: &Path, work: &Path, name: &str) -> Result<()> {
    let vendor = work.join("vendor");
    let directory = vendor.join(format!("{name}-{PACKAGE_VERSION}"));
    require(!directory.exists(), "bootstrap package already exists")?;
    tooling::execute(
        source,
        "tar".as_ref(),
        &[
            "--extract".into(),
            "--gzip".into(),
            "--file".into(),
            work.join("bootstrap-target/package")
                .join(format!("{name}-{PACKAGE_VERSION}.crate"))
                .into_os_string(),
            "--directory".into(),
            vendor.into_os_string(),
            "--keep-old-files".into(),
            "--no-same-owner".into(),
            "--no-same-permissions".into(),
        ],
        false,
    )?;
    let mut files = BTreeMap::new();
    visit(&directory, &directory, &mut files)?;
    fs::write(
        directory.join(".cargo-checksum.json"),
        tooling::json_bytes(&json!({"files":files,"package":null})),
    )?;
    Ok(())
}

/// Execute only the packaging phase. CI identity and immutable-source guards remain in `run`.
pub fn build_packages(
    source: &Path,
    work: &Path,
    out: &Path,
    cohort: Cohort,
    cargo: &Path,
    environment: &[(OsString, Option<OsString>)],
    record: &mut Value,
) -> Result<()> {
    if cohort == Cohort::Sixteen {
        fs::create_dir(work.join("vendor"))?;
        let directory = work
            .join("vendor")
            .to_str()
            .ok_or_else(|| fail("vendor path encoding"))?
            .to_owned();
        let escaped = toml::Value::String(directory).to_string();
        fs::write(
            work.join("vendor-config.toml"),
            format!(
                "[source.crates-io]\nreplace-with = \"vendored-sources\"\n[source.vendored-sources]\ndirectory = {escaped}\n"
            ),
        )?;
    }
    let steps = cargo_steps(source, work, cohort);
    record["commands"] = json!(
        steps
            .iter()
            .map(|(_, args)| {
                let mut argv = vec![cargo.to_string_lossy().into_owned()];
                argv.extend(args.iter().map(|a| a.to_string_lossy().into_owned()));
                argv
            })
            .collect::<Vec<_>>()
    );
    for (label, args) in steps {
        let r = process::run(
            &Spec {
                program: cargo.as_os_str().into(),
                args,
                cwd: source.into(),
                env: environment.to_vec(),
                timeout: Duration::from_secs(1200),
                output_limit: 64 * 1024 * 1024,
            },
            &process::CANCELLED,
        );
        let mut log = r.stdout.bytes.clone();
        log.extend(&r.stderr.bytes);
        fs::write(out.join(format!("{label}.log")), log)?;
        record[format!("{label}_exit_code")] = json!(r.exit_code());
        record[format!("{label}_native_status")] = json!({"origin":r.origin.as_str(),"native_exit_code":r.native_code(),"signal":r.signal(),"cleanup_ok":r.cleanup_ok});
        write(out, "BUILD-RECORD.json", record)?;
        if !r.success() || r.stdout.discarded > 0 || r.stderr.discarded > 0 {
            return Err(tooling::Error {
                code: r.exit_code().max(1),
                message: "private rebuild command failed".into(),
            });
        }
        if let Some(name) = label.strip_prefix("bootstrap-") {
            let staged = stage_bootstrap(source, work, name);
            record["bootstrap_staging"][name] = json!({"ok":staged.is_ok(),"verification":false});
            write(out, "BUILD-RECORD.json", record)?;
            staged?;
        }
    }
    Ok(())
}

pub fn run(root: &Path, cohort: Cohort) -> Result<()> {
    let commit = command(root, &["rev-parse", "HEAD"])?;
    let commit = commit.trim();
    require(
        commit == env("GITHUB_SHA")?
            && commit.len() == 40
            && commit.bytes().all(|b| b.is_ascii_hexdigit()),
        "immutable source identity",
    )?;
    require(
        command(root, &["status", "--porcelain"])?.is_empty(),
        "clean source required",
    )?;
    let repository = env("GITHUB_REPOSITORY")?;
    let workflow_ref = env("BUILD_WORKFLOW_REF")?;
    let workflow_sha = env("BUILD_WORKFLOW_SHA")?;
    let git_ref = env("GITHUB_REF")?;
    require(
        repository == "bohewu/yetlet"
            && workflow_sha == commit
            && workflow_ref
                == format!("{repository}/.github/workflows/private-rebuild.yml@{git_ref}"),
        "workflow/source identity mismatch",
    )?;
    require(
        env("GITHUB_EVENT_NAME")? == "workflow_dispatch",
        "private rebuild is manual-only",
    )?;
    run_body(root, cohort, commit, &repository, &git_ref, false)
}

pub(super) fn run_preparation(root: &Path, commit: &str) -> Result<()> {
    run_body(
        root,
        Cohort::Sixteen,
        commit,
        "bohewu/yetlet",
        "refs/tags/v0.1.0",
        true,
    )
}

fn run_body(
    root: &Path,
    cohort: Cohort,
    commit: &str,
    repository: &str,
    git_ref: &str,
    preparation: bool,
) -> Result<()> {
    let runner = PathBuf::from(env("RUNNER_TEMP")?);
    let temp = tempfile::Builder::new()
        .prefix("yetlet-private-rebuild-")
        .tempdir_in(runner)?;
    let work = temp.path();
    let out = root.join(if preparation {
        "target/release-preparation/private"
    } else {
        "target/private-rebuild"
    });
    fs::create_dir_all(out.parent().unwrap())?;
    fs::create_dir(&out)?;
    let result: Result<()> = (|| {
        let archive = work.join("source.tar");
        let source = work.join("source");
        fs::create_dir(&source)?;
        let tree = command(root, &["ls-tree", "-rz", "--full-tree", commit])?;
        let allowed = aliases();
        let mut actual = BTreeMap::new();
        let mut exclusions = vec![];
        for item in tree.split('\0').filter(|s| !s.is_empty()) {
            let (meta, path) = item
                .split_once('\t')
                .ok_or_else(|| fail("archive tree record"))?;
            require(archive_path(path), "unsafe archive path")?;
            let mut parts = meta.split_whitespace();
            let mode = parts.next().ok_or_else(|| fail("archive mode"))?;
            let kind = parts.next().ok_or_else(|| fail("archive kind"))?;
            let oid = parts.next().ok_or_else(|| fail("archive object"))?;
            require(
                kind == "blob" && ["100644", "100755", "120000"].contains(&mode),
                "unsupported archive entry type",
            )?;
            if mode == "120000" {
                actual.insert(path.into(), command(root, &["cat-file", "blob", oid])?);
                exclusions.push(path.to_owned());
            }
        }
        require(actual == allowed, "exact eighteen license aliases required")?;
        tooling::execute(
            root,
            "git".as_ref(),
            &[
                "archive".into(),
                "--format=tar".into(),
                "--output".into(),
                archive.clone().into_os_string(),
                commit.into(),
            ],
            false,
        )?;
        // git created this immutable archive from the validated tree. GNU tar owns extraction;
        // symlinks are excluded and materialized from the two validated root licenses.
        let mut args = vec![
            "--extract".into(),
            "--file".into(),
            archive.clone().into_os_string(),
            "--directory".into(),
            source.clone().into_os_string(),
            "--no-same-owner".into(),
            "--no-same-permissions".into(),
        ];
        for name in exclusions {
            args.push(format!("--exclude={name}").into());
        }
        tooling::execute(root, "tar".as_ref(), &args, false)?;
        for (name, _) in allowed {
            let license = Path::new(&name).file_name().unwrap();
            fs::write(source.join(&name), fs::read(source.join(license))?)?;
        }
        let inputs = selected(&source, cohort)?;
        let canonical =
            serde_json::to_vec(&inputs).map_err(|_| fail("canonical selected inputs"))?;
        let digest = tooling::sha(&canonical);
        let mut source_record = json!({"source_commit":commit,"source_archive_sha256":tooling::sha(&fs::read(&archive)?),"overlays":{},"selected_package_inputs":inputs,"selected_package_input_digest":digest});
        if cohort == Cohort::Sixteen {
            source_record["cohort"] = json!("sixteen");
            source_record["package_manifests"] = json!(cohort.manifests());
        }
        write(&out, "SOURCE-INPUTS.json", &source_record)?;
        let mut tools = BTreeMap::new();
        for tool in ["cargo", "rustc", "rustdoc"] {
            let bytes = tooling::execute(
                root,
                "rustup".as_ref(),
                &[
                    "which".into(),
                    "--toolchain".into(),
                    "1.99.0".into(),
                    tool.into(),
                ],
                true,
            )?;
            tools.insert(
                tool,
                PathBuf::from(
                    String::from_utf8(bytes)
                        .map_err(|_| fail("tool path"))?
                        .trim(),
                ),
            );
        }
        let rustc = String::from_utf8(tooling::execute(
            root,
            tools["rustc"].as_os_str(),
            &["-vV".into()],
            true,
        )?)
        .map_err(|_| fail("rustc identity"))?;
        let cargo = String::from_utf8(tooling::execute(
            root,
            tools["cargo"].as_os_str(),
            &["-V".into()],
            true,
        )?)
        .map_err(|_| fail("cargo identity"))?;
        require(
            rustc.contains("release: 1.99.0\n")
                && rustc.contains("host: x86_64-unknown-linux-gnu\n")
                && cargo.starts_with("cargo 1.99.0 "),
            "pinned GNU tool identity",
        )?;
        let openssl = if cohort == Cohort::Sixteen {
            require(
                env("OPENSSL_STATIC")? == "1",
                "static OpenSSL recipe required",
            )?;
            let prefix = PathBuf::from(env("OPENSSL_DIR")?);
            let binary = prefix.join("bin/openssl");
            let version = String::from_utf8(tooling::execute(
                root,
                binary.as_os_str(),
                &["version".into()],
                true,
            )?)
            .map_err(|_| fail("OpenSSL identity"))?;
            require(
                version.starts_with("OpenSSL 3.5.9 "),
                "selected OpenSSL 3.5.9 recipe required",
            )?;
            Some((prefix, version, tooling::sha(&fs::read(binary)?)))
        } else {
            None
        };
        let cargo_home = work.join("cargo-home");
        let target = work.join("cargo-target");
        fs::create_dir(&cargo_home)?;
        fs::create_dir(&target)?;
        let epoch = command(root, &["show", "-s", "--format=%ct", commit])?;
        let mut environment = vec![
            ("CARGO_HOME".into(), Some(cargo_home.into_os_string())),
            (
                "CARGO_TARGET_DIR".into(),
                Some(target.clone().into_os_string()),
            ),
            ("RUSTC".into(), Some(tools["rustc"].as_os_str().into())),
            ("RUSTDOC".into(), Some(tools["rustdoc"].as_os_str().into())),
            ("SOURCE_DATE_EPOCH".into(), Some(epoch.trim().into())),
            ("CARGO_BUILD_JOBS".into(), Some("4".into())),
            ("RUSTUP_AUTO_INSTALL".into(), Some("0".into())),
        ];
        if let Some((prefix, _, _)) = &openssl {
            environment.extend([
                ("OPENSSL_DIR".into(), Some(prefix.as_os_str().into())),
                ("OPENSSL_STATIC".into(), Some("1".into())),
            ]);
        }
        let mut tool_hashes = BTreeMap::new();
        for (name, path) in &tools {
            tool_hashes.insert(*name, tooling::sha(&fs::read(path)?));
        }
        let now = tooling::utc_now;
        let mut github = BTreeMap::new();
        for k in [
            "GITHUB_REPOSITORY",
            "GITHUB_SHA",
            "GITHUB_REF",
            "GITHUB_RUN_ID",
            "GITHUB_RUN_ATTEMPT",
            "BUILD_WORKFLOW_REF",
            "BUILD_WORKFLOW_SHA",
            "RUNNER_OS",
            "RUNNER_ARCH",
            "ImageOS",
            "ImageVersion",
        ] {
            github.insert(k, std::env::var(k).ok());
        }
        let mut record = json!({"source_commit":commit,"selected_package_input_digest":digest,"cargo":cargo.trim(),"rustc":rustc,"tool_sha256":tool_hashes,"default_workspace_features":true,"profile":"dev verification","fresh_cargo_home":true,"fresh_target_directory":true,"cargo_lock_sha256":tooling::sha(&fs::read(source.join("Cargo.lock"))?),"source_date_epoch":epoch.trim(),"build_jobs":4,"started_at":now(),"github":github});
        if let Some((_, version, sha)) = openssl {
            record
                .as_object_mut()
                .unwrap()
                .remove("default_workspace_features");
            record["default_package_features"] = json!(true);
            record["cohort"] = json!("sixteen");
            record["packages"] = json!(cohort.names());
            record["openssl"] = json!({"version":version.trim(),"executable_sha256":sha});
            let mut locks = BTreeMap::new();
            for manifest in cohort.manifests() {
                let path = Path::new(manifest).with_file_name("Cargo.lock");
                locks.insert(
                    path.to_string_lossy().replace('\\', "/"),
                    tooling::sha(&fs::read(source.join(path))?),
                );
            }
            record["cargo_lock_sha256s"] = json!(locks);
        }
        if preparation {
            super::preparation::record_inputs(root, &out, &mut record)?;
        }
        build_packages(
            &source,
            work,
            &out,
            cohort,
            &tools["cargo"],
            &environment,
            &mut record,
        )?;
        require(
            selected(&source, cohort)? == inputs,
            "package inputs changed",
        )?;
        let mut archives = BTreeMap::new();
        for e in fs::read_dir(target.join("package"))? {
            let p = e?.path();
            if p.extension().is_some_and(|s| s == "crate") {
                let name = p
                    .file_name()
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| fail("archive filename"))?
                    .to_owned();
                let bytes = fs::read(&p)?;
                archives.insert(
                    name.clone(),
                    json!({"sha256":tooling::sha(&bytes),"size":bytes.len()}),
                );
                fs::write(out.join(name), bytes)?;
            }
        }
        require(
            archives.keys().cloned().collect::<BTreeSet<_>>()
                == cohort
                    .names()
                    .iter()
                    .map(|n| format!("{n}-{PACKAGE_VERSION}.crate"))
                    .collect(),
            "exact selected package archives",
        )?;
        record["archives"] = json!(archives);
        record["finished_at"] = json!(now());
        write(&out, "BUILD-RECORD.json", &record)?;
        if preparation {
            return super::preparation::finish(root, &out, &source_record, &record, &archives);
        }
        let repo_url = format!("https://github.com/{repository}");
        let workflow = json!({"repository":repo_url,"path":".github/workflows/private-rebuild.yml","ref":git_ref});
        let mut statement = json!({"_type":"https://in-toto.io/Statement/v1","subject":archives.iter().map(|(n,v)|json!({"name":n,"digest":{"sha256":v["sha256"]}})).collect::<Vec<_>>(),"predicateType":"https://slsa.dev/provenance/v1","predicate":{"buildDefinition":{"buildType":"https://actions.github.io/buildtypes/workflow/v1","externalParameters":{"workflow":workflow,"preparation_only":true,"rust_release":"1.99.0","target":"x86_64-unknown-linux-gnu","default_workspace_features":true,"commands_record":"BUILD-RECORD.json"},"resolvedDependencies":[{"uri":format!("git+{repo_url}"),"digest":{"gitCommit":commit}},{"name":"Cargo.lock","digest":{"sha256":record["cargo_lock_sha256"]}},{"name":"source.tar","digest":{"sha256":source_record["source_archive_sha256"]}}]},"runDetails":{"builder":{"id":"https://github.com/actions/runner/github-hosted"},"metadata":{"invocationId":format!("{repo_url}/actions/runs/{}/attempts/{}",env("GITHUB_RUN_ID")?,env("GITHUB_RUN_ATTEMPT")?),"startedOn":record["started_at"],"finishedOn":record["finished_at"]},"byproducts":[{"name":"SOURCE-INPUTS.json","digest":{"sha256":tooling::sha(&fs::read(out.join("SOURCE-INPUTS.json"))?)}},{"name":"BUILD-RECORD.json","digest":{"sha256":tooling::sha(&fs::read(out.join("BUILD-RECORD.json"))?)}}]}}});
        if cohort == Cohort::Sixteen {
            let definition = &mut statement["predicate"]["buildDefinition"];
            let parameters = &mut definition["externalParameters"];
            parameters
                .as_object_mut()
                .unwrap()
                .remove("default_workspace_features");
            parameters["default_package_features"] = json!(true);
            parameters["cohort"] = json!("sixteen");
            parameters["packages"] = json!(cohort.names());
            parameters["openssl"] = record["openssl"].clone();
            for (path, sha) in record["cargo_lock_sha256s"].as_object().unwrap() {
                if path != "Cargo.lock" {
                    definition["resolvedDependencies"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"name":path,"digest":{"sha256":sha}}));
                }
            }
        }
        write(&out, "PROVENANCE-UNSIGNED.json", &statement)?;
        super::checksums::run("create", &out)?;
        super::checksums::run("verify", &out)
    })();
    let cleanup = temp.close();
    cleanup?;
    result
}
