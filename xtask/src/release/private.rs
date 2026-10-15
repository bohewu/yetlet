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
fn selected(source: &Path) -> Result<BTreeMap<String, String>> {
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
    for n in NAMES {
        visit(source, &source.join("crates").join(n), &mut out)?;
    }
    Ok(out)
}
fn write(out: &Path, name: &str, value: &Value) -> Result<()> {
    fs::write(out.join(name), tooling::json_bytes(value))?;
    Ok(())
}
pub fn run(root: &Path) -> Result<()> {
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
    let runner = PathBuf::from(env("RUNNER_TEMP")?);
    let temp = tempfile::Builder::new()
        .prefix("yetlet-private-rebuild-")
        .tempdir_in(runner)?;
    let work = temp.path();
    let out = root.join("target/private-rebuild");
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
        let inputs = selected(&source)?;
        let canonical =
            serde_json::to_vec(&inputs).map_err(|_| fail("canonical selected inputs"))?;
        let digest = tooling::sha(&canonical);
        let source_record = json!({"source_commit":commit,"source_archive_sha256":tooling::sha(&fs::read(&archive)?),"overlays":{},"selected_package_inputs":inputs,"selected_package_input_digest":digest});
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
        let cargo_home = work.join("cargo-home");
        let target = work.join("cargo-target");
        fs::create_dir(&cargo_home)?;
        fs::create_dir(&target)?;
        let epoch = command(root, &["show", "-s", "--format=%ct", commit])?;
        let environment = vec![
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
            target.clone().into_os_string(),
        ];
        for name in NAMES {
            package.extend(["-p".into(), name.into()]);
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
        let commands = [fetch, package];
        let command_record: Vec<_> = commands
            .iter()
            .map(|args| {
                let mut a = vec![tools["cargo"].to_string_lossy().into_owned()];
                a.extend(args.iter().map(|s| s.to_string_lossy().into_owned()));
                a
            })
            .collect();
        let mut record = json!({"source_commit":commit,"selected_package_input_digest":digest,"cargo":cargo.trim(),"rustc":rustc,"tool_sha256":tool_hashes,"default_workspace_features":true,"profile":"dev verification","fresh_cargo_home":true,"fresh_target_directory":true,"cargo_lock_sha256":tooling::sha(&fs::read(source.join("Cargo.lock"))?),"source_date_epoch":epoch.trim(),"build_jobs":4,"commands":command_record,"started_at":now(),"github":github});
        for (args, label) in commands.into_iter().zip(["fetch", "package"]) {
            let r = process::run(
                &Spec {
                    program: tools["cargo"].as_os_str().into(),
                    args,
                    cwd: source.clone(),
                    env: environment.clone(),
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
            write(&out, "BUILD-RECORD.json", &record)?;
            if !r.success() || r.stdout.discarded > 0 || r.stderr.discarded > 0 {
                return Err(tooling::Error {
                    code: r.exit_code().max(1),
                    message: "private rebuild command failed".into(),
                });
            }
        }
        require(selected(&source)? == inputs, "package inputs changed")?;
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
                == NAMES
                    .into_iter()
                    .map(|n| format!("{n}-0.1.0.crate"))
                    .collect(),
            "exact nine package archives",
        )?;
        record["archives"] = json!(archives);
        record["finished_at"] = json!(now());
        write(&out, "BUILD-RECORD.json", &record)?;
        let repo_url = format!("https://github.com/{repository}");
        let workflow = json!({"repository":repo_url,"path":".github/workflows/private-rebuild.yml","ref":git_ref});
        let statement = json!({"_type":"https://in-toto.io/Statement/v1","subject":archives.iter().map(|(n,v)|json!({"name":n,"digest":{"sha256":v["sha256"]}})).collect::<Vec<_>>(),"predicateType":"https://slsa.dev/provenance/v1","predicate":{"buildDefinition":{"buildType":"https://actions.github.io/buildtypes/workflow/v1","externalParameters":{"workflow":workflow,"preparation_only":true,"rust_release":"1.99.0","target":"x86_64-unknown-linux-gnu","default_workspace_features":true,"commands_record":"BUILD-RECORD.json"},"resolvedDependencies":[{"uri":format!("git+{repo_url}"),"digest":{"gitCommit":commit}},{"name":"Cargo.lock","digest":{"sha256":record["cargo_lock_sha256"]}},{"name":"source.tar","digest":{"sha256":source_record["source_archive_sha256"]}}]},"runDetails":{"builder":{"id":"https://github.com/actions/runner/github-hosted"},"metadata":{"invocationId":format!("{repo_url}/actions/runs/{}/attempts/{}",env("GITHUB_RUN_ID")?,env("GITHUB_RUN_ATTEMPT")?),"startedOn":record["started_at"],"finishedOn":record["finished_at"]},"byproducts":[{"name":"SOURCE-INPUTS.json","digest":{"sha256":tooling::sha(&fs::read(out.join("SOURCE-INPUTS.json"))?)}},{"name":"BUILD-RECORD.json","digest":{"sha256":tooling::sha(&fs::read(out.join("BUILD-RECORD.json"))?)}}]}}});
        write(&out, "PROVENANCE-UNSIGNED.json", &statement)?;
        super::checksums::run("create", &out)?;
        super::checksums::run("verify", &out)
    })();
    let cleanup = temp.close();
    cleanup?;
    result
}
