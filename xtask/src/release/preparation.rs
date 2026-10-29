//! Closed, unsigned release preparation. No signing or publication authority.
use super::{
    checksums,
    private::{self, Cohort},
};
use crate::{
    smoke::require,
    tooling::{self, Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const REPOSITORY: &str = "bohewu/yetlet";
fn tag() -> String {
    format!("refs/tags/v{}", private::package_version())
}
fn workflow() -> String {
    format!("bohewu/yetlet/.github/workflows/release.yml@{}", tag())
}
const OPENSSL_SHA: &str = "603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a";
const PROVENANCE: &str = "PROVENANCE-UNSIGNED.json";

pub fn commit_token(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn git(root: &Path, args: &[&str]) -> Result<String> {
    String::from_utf8(tooling::execute(
        root,
        "git".as_ref(),
        &args.iter().map(Into::into).collect::<Vec<_>>(),
        true,
    )?)
    .map_err(|_| fail("git identity encoding"))
}
fn identity(root: &Path, expected: &str, env: &BTreeMap<String, String>) -> Result<()> {
    let get = |key: &str| env.get(key).map(String::as_str).unwrap_or("");
    require(commit_token(expected), "explicit lowercase commit required")?;
    require(
        get("GITHUB_REPOSITORY") == REPOSITORY
            && get("GITHUB_EVENT_NAME") == "workflow_dispatch"
            && get("GITHUB_REF") == tag()
            && get("BUILD_WORKFLOW_REF") == workflow()
            && get("GITHUB_SHA") == expected
            && get("BUILD_WORKFLOW_SHA") == expected,
        "release preparation identity mismatch",
    )?;
    for key in ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT"] {
        let value = get(key);
        require(
            !value.is_empty()
                && value.bytes().all(|b| b.is_ascii_digit())
                && value.bytes().any(|b| b != b'0'),
            "invalid run identity",
        )?;
    }
    require(
        git(root, &["rev-parse", "HEAD"])?.trim() == expected
            && git(
                root,
                &["rev-parse", "--verify", &format!("{}^{{commit}}", tag())],
            )?
            .trim()
                == expected,
        "release source/tag mismatch",
    )?;
    git(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            expected,
            "refs/remotes/origin/master",
        ],
    )?;
    require(
        git(root, &["status", "--porcelain"])?.is_empty(),
        "clean source required",
    )
}
fn env_path(name: &str) -> Result<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| fail("required preparation input unavailable"))
}
fn openssl_source(path: &Path) -> Result<String> {
    require(
        fs::symlink_metadata(path)?.is_file(),
        "OpenSSL source must be a regular file",
    )?;
    let digest = tooling::sha(&fs::read(path)?);
    require(
        digest == OPENSSL_SHA,
        "pinned OpenSSL source digest mismatch",
    )?;
    Ok(digest)
}
pub fn run(root: &Path, expected_commit: &str) -> Result<()> {
    identity(root, expected_commit, &std::env::vars().collect())?;
    openssl_source(&env_path("OPENSSL_SOURCE_ARCHIVE")?)?;
    require(
        !root.join("target/release-preparation/public").exists(),
        "public output already exists",
    )?;
    private::run_preparation(root, expected_commit)
}

pub(super) fn record_inputs(root: &Path, out: &Path, record: &mut Value) -> Result<()> {
    record["openssl"]["source_sha256"] =
        json!(openssl_source(&env_path("OPENSSL_SOURCE_ARCHIVE")?)?);
    let mut recipes = BTreeMap::new();
    for (name, path) in [
        ("release-workflow", ".github/workflows/release.yml"),
        ("package-recipe", "xtask/src/release/private.rs"),
        ("preparation-recipe", "xtask/src/release/preparation.rs"),
        ("xtask-Cargo.lock", "xtask/Cargo.lock"),
    ] {
        recipes.insert(name, tooling::sha(&fs::read(root.join(path))?));
    }
    record["recipe_sha256"] = json!(recipes);
    // These fixed tools are the actual PATH-resolved executables used by the recipe.
    for tool in [
        "git",
        "tar",
        "rustup",
        "cc",
        "make",
        "perl",
        "curl",
        "sha256sum",
    ] {
        let path = String::from_utf8(tooling::execute(
            root,
            "sh".as_ref(),
            &["-c".into(), format!("command -v {tool}").into()],
            true,
        )?)
        .map_err(|_| fail("tool identity encoding"))?;
        record["tool_sha256"][tool] = json!(tooling::sha(&fs::read(path.trim())?));
    }
    let runner = env_path("RUNNER_TEMP")?;
    for name in ["openssl-build.log", "openssl-build-result.json"] {
        fs::copy(runner.join(name), out.join(name))?;
    }
    Ok(())
}

fn write(out: &Path, name: &str, value: &Value) -> Result<()> {
    fs::write(out.join(name), tooling::json_bytes(value))?;
    Ok(())
}

pub(super) fn finish(
    root: &Path,
    out: &Path,
    source: &Value,
    record: &Value,
    archives: &BTreeMap<String, Value>,
) -> Result<()> {
    require(
        archives.keys().cloned().collect::<BTreeSet<_>>()
            == Cohort::Sixteen
                .names()
                .iter()
                .map(|n| format!("{n}-{}.crate", private::package_version()))
                .collect(),
        "exact sixteen archive subjects required",
    )?;
    for (name, value) in archives {
        require(
            tooling::sha(&fs::read(out.join(name))?) == value["sha256"],
            "archive changed before provenance",
        )?;
    }
    write(out, "ARCHIVE-INDEX.json", &json!(archives))?;
    write(
        out,
        "INVENTORY-COVERAGE.json",
        &json!({
            "scope":"sixteen Cargo source packages; 33 recipe phases including separate bootstrap collection and normal offline locked verification",
            "fresh_cargo_and_target":true,"hermetic_environment":false,
            "inventory":"selected source, six locks, recipe and tool hashes; not complete compiled/native/system closure",
            "qualification":"no new browser, CSP, Windows, GPU, security or operational qualification; historical evidence retains its original source and scope",
            "custody":"complete private assembly is runner-local ephemeral; no durable private storage configured",
            "release_approved":false,"unsigned":true
        }),
    )?;
    let mut dependencies = vec![
        json!({"uri":"git+https://github.com/bohewu/yetlet","digest":{"gitCommit":record["source_commit"]}}),
        json!({"name":"source.tar","digest":{"sha256":source["source_archive_sha256"]}}),
    ];
    let locks = record["cargo_lock_sha256s"]
        .as_object()
        .ok_or_else(|| fail("six lock inputs required"))?;
    require(locks.len() == 6, "six lock inputs required")?;
    for manifest in Cohort::Sixteen.manifests() {
        let path = Path::new(manifest).with_file_name("Cargo.lock");
        let key = path.to_string_lossy().replace('\\', "/");
        let digest = locks
            .get(&key)
            .ok_or_else(|| fail("missing selected lock"))?;
        let name = path.parent().and_then(Path::file_name).map_or_else(
            || "Cargo.lock".to_owned(),
            |p| format!("{}-Cargo.lock", p.to_string_lossy()),
        );
        dependencies.push(json!({"name":name,"digest":{"sha256":digest}}));
    }
    for key in ["recipe_sha256", "tool_sha256"] {
        for (name, digest) in record[key]
            .as_object()
            .ok_or_else(|| fail("build input identities required"))?
        {
            dependencies.push(json!({"name":name,"digest":{"sha256":digest}}));
        }
    }
    dependencies.extend([
        json!({"name":"openssl-3.5.9.tar.gz","digest":{"sha256":record["openssl"]["source_sha256"]}}),
        json!({"name":"openssl-executable","digest":{"sha256":record["openssl"]["executable_sha256"]}}),
    ]);
    let mut byproducts = vec![];
    for name in [
        "SOURCE-INPUTS.json",
        "BUILD-RECORD.json",
        "ARCHIVE-INDEX.json",
        "INVENTORY-COVERAGE.json",
    ] {
        byproducts.push(
            json!({"name":name,"digest":{"sha256":tooling::sha(&fs::read(out.join(name))?)}}),
        );
    }
    let github = &record["github"];
    let run_id = github["GITHUB_RUN_ID"]
        .as_str()
        .ok_or_else(|| fail("run identity"))?;
    let attempt = github["GITHUB_RUN_ATTEMPT"]
        .as_str()
        .ok_or_else(|| fail("run identity"))?;
    let statement = json!({
        "_type":"https://in-toto.io/Statement/v1",
        "subject":archives.iter().map(|(name,value)| json!({"name":name,"digest":{"sha256":value["sha256"]}})).collect::<Vec<_>>(),
        "predicateType":"https://slsa.dev/provenance/v1",
        "predicate":{
            "buildDefinition":{
                "buildType":"urn:yetlet:build:cargo-source-package:v1",
                "externalParameters":{"workflow":{"repository":"https://github.com/bohewu/yetlet","path":".github/workflows/release.yml","ref":tag()},"cohort":"sixteen"},
                "internalParameters":{"rust_release":"1.99.0","target":"x86_64-unknown-linux-gnu","default_package_features":true,"openssl":"3.5.9","source_date_epoch":record["source_date_epoch"]},
                "resolvedDependencies":dependencies
            },
            "runDetails":{
                "builder":{"id":"https://github.com/actions/runner/github-hosted"},
                "metadata":{"invocationId":format!("https://github.com/bohewu/yetlet/actions/runs/{run_id}/attempts/{attempt}"),"startedOn":record["started_at"],"finishedOn":record["finished_at"]},
                "byproducts":byproducts
            }
        }
    });
    write(out, PROVENANCE, &statement)?;
    checksums::run("create", out)?;
    checksums::run("verify", out)?;
    stage_public(root, out, archives)?;
    println!(
        "Unsigned preparation only; complete private evidence is runner-local ephemeral, without durable custody or release approval."
    );
    Ok(())
}

fn stage_public(root: &Path, private: &Path, archives: &BTreeMap<String, Value>) -> Result<()> {
    let parent = root.join("target/release-preparation");
    let public = parent.join("public");
    require(!public.exists(), "public output already exists")?;
    let staging = tempfile::Builder::new()
        .prefix(".public-staging-")
        .tempdir_in(&parent)?;
    for name in archives.keys().map(String::as_str).chain([PROVENANCE]) {
        fs::copy(private.join(name), staging.path().join(name))?;
    }
    checksums::run("create", staging.path())?;
    checksums::run("verify", staging.path())?;
    fs::rename(staging.path(), public)?;
    // The verified directory has moved; disarm cleanup of its now-absent old path.
    let _ = staging.keep();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git_fixture(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().into()
    }
    fn fixture_identity(root: &Path) -> (String, BTreeMap<String, String>) {
        git_fixture(root, &["init", "-q"]);
        git_fixture(root, &["config", "user.email", "fixture@example.invalid"]);
        git_fixture(root, &["config", "user.name", "Fixture"]);
        fs::write(root.join("source"), "fixture").unwrap();
        git_fixture(root, &["add", "."]);
        git_fixture(root, &["commit", "-qm", "fixture"]);
        let commit = git_fixture(root, &["rev-parse", "HEAD"]);
        git_fixture(root, &["tag", tag().trim_start_matches("refs/tags/")]);
        git_fixture(root, &["update-ref", "refs/remotes/origin/master", &commit]);
        let env = [
            ("GITHUB_REPOSITORY", REPOSITORY),
            ("GITHUB_EVENT_NAME", "workflow_dispatch"),
            ("GITHUB_REF", &tag()),
            ("BUILD_WORKFLOW_REF", &workflow()),
            ("GITHUB_SHA", &commit),
            ("BUILD_WORKFLOW_SHA", &commit),
            ("GITHUB_RUN_ID", "123"),
            ("GITHUB_RUN_ATTEMPT", "1"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        (commit, env)
    }
    #[test]
    fn co_located_identity_accepts_valid_and_refuses_each_disagreement_before_tools() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let (commit, env) = fixture_identity(root);
        identity(root, &commit, &env).unwrap();
        for (key, bad) in [
            ("GITHUB_REPOSITORY", "fork/yetlet"),
            ("GITHUB_EVENT_NAME", "pull_request"),
            ("GITHUB_EVENT_NAME", "push"),
            ("GITHUB_REF", "refs/heads/master"),
            ("GITHUB_REF", "refs/pull/1/merge"),
            ("GITHUB_REF", "refs/tags/v0.2.0"),
            ("GITHUB_REF", "refs/tags/v0.1.0"),
            (
                "BUILD_WORKFLOW_REF",
                "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0",
            ),
            (
                "BUILD_WORKFLOW_REF",
                "bohewu/yetlet/.github/workflows/private-rebuild.yml@refs/tags/v0.9.0-alpha.1",
            ),
            (
                "BUILD_WORKFLOW_SHA",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            ("GITHUB_SHA", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ("GITHUB_RUN_ID", "../secret"),
            ("GITHUB_RUN_ATTEMPT", "0"),
        ] {
            let mut changed = env.clone();
            changed.insert(key.into(), bad.into());
            assert!(
                identity(root, &commit, &changed).is_err(),
                "accepted {key}={bad}"
            );
        }
        for bad in [
            "",
            "main",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "abc",
        ] {
            assert!(identity(root, bad, &env).is_err());
        }
        fs::write(root.join("source"), "next").unwrap();
        git_fixture(root, &["add", "."]);
        git_fixture(root, &["commit", "-qm", "next"]);
        let next = git_fixture(root, &["rev-parse", "HEAD"]);
        assert!(identity(root, &commit, &env).is_err()); // HEAD disagreement
        git_fixture(root, &["checkout", "-q", &commit]);
        git_fixture(
            root,
            &["tag", "-f", tag().trim_start_matches("refs/tags/"), &next],
        );
        assert!(identity(root, &commit, &env).is_err()); // tag disagreement
        git_fixture(
            root,
            &["tag", "-f", tag().trim_start_matches("refs/tags/"), &commit],
        );
        git_fixture(root, &["tag", "-d", tag().trim_start_matches("refs/tags/")]);
        assert!(identity(root, &commit, &env).is_err()); // missing tag
        git_fixture(root, &["tag", tag().trim_start_matches("refs/tags/")]);
        git_fixture(root, &["checkout", "-q", &next]);
        git_fixture(
            root,
            &["tag", "-f", tag().trim_start_matches("refs/tags/"), &next],
        );
        let mut changed = env;
        changed.insert("GITHUB_SHA".into(), next.clone());
        changed.insert("BUILD_WORKFLOW_SHA".into(), next.clone());
        assert!(identity(root, &next, &changed).is_err()); // source outside origin/master ancestry
        git_fixture(root, &["update-ref", "refs/remotes/origin/master", &next]);
        identity(root, &next, &changed).unwrap();
        fs::write(root.join("untracked"), "dirty").unwrap();
        assert!(identity(root, &next, &changed).is_err());
        assert!(!root.join("target").exists());
    }

    fn keys(v: &Value) -> BTreeSet<&str> {
        v.as_object().unwrap().keys().map(String::as_str).collect()
    }
    fn exact_keys(v: &Value, names: &[&str]) {
        assert_eq!(keys(v), names.iter().copied().collect());
    }
    #[test]
    fn fresh_statement_has_closed_fields_actual_bindings_and_exact_public_allowlist() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let out = root.join("target/release-preparation/private");
        fs::create_dir_all(&out).unwrap();
        let mut archives = BTreeMap::new();
        for name in Cohort::Sixteen.names() {
            let file = format!("{name}-{}.crate", private::package_version());
            let bytes = file.as_bytes();
            archives.insert(
                file.clone(),
                json!({"sha256":tooling::sha(bytes),"size":bytes.len()}),
            );
            fs::write(out.join(&file), bytes).unwrap();
        }
        let mut locks = BTreeMap::new();
        for manifest in Cohort::Sixteen.manifests() {
            let path = Path::new(manifest).with_file_name("Cargo.lock");
            locks.insert(
                path.to_string_lossy().into_owned(),
                tooling::sha(manifest.as_bytes()),
            );
        }
        let source = json!({"source_archive_sha256":tooling::sha(b"actual export")});
        let record = json!({"source_commit":"a".repeat(40),"source_date_epoch":"123", "cargo_lock_sha256s":locks,
            "tool_sha256":{"cargo":tooling::sha(b"cargo"),"rustc":tooling::sha(b"rustc"),"rustdoc":tooling::sha(b"rustdoc")},
            "recipe_sha256":{"release-workflow":tooling::sha(b"recipe")},
            "openssl":{"source_sha256":OPENSSL_SHA,"executable_sha256":tooling::sha(b"openssl")},
            "github":{"GITHUB_RUN_ID":"123","GITHUB_RUN_ATTEMPT":"1"},"started_at":"2026-10-27T00:00:00Z","finished_at":"2026-10-27T00:01:00Z"});
        write(&out, "SOURCE-INPUTS.json", &source).unwrap();
        write(&out, "BUILD-RECORD.json", &record).unwrap();
        fs::write(out.join("private-sentinel.log"), "PRIVATE SENTINEL").unwrap();
        finish(root, &out, &source, &record, &archives).unwrap();
        let statement: Value =
            serde_json::from_slice(&fs::read(out.join(PROVENANCE)).unwrap()).unwrap();
        exact_keys(
            &statement,
            &["_type", "subject", "predicateType", "predicate"],
        );
        exact_keys(&statement["predicate"], &["buildDefinition", "runDetails"]);
        let definition = &statement["predicate"]["buildDefinition"];
        exact_keys(
            definition,
            &[
                "buildType",
                "externalParameters",
                "internalParameters",
                "resolvedDependencies",
            ],
        );
        assert_eq!(
            definition["buildType"],
            "urn:yetlet:build:cargo-source-package:v1"
        );
        exact_keys(&definition["externalParameters"], &["workflow", "cohort"]);
        exact_keys(
            &definition["externalParameters"]["workflow"],
            &["repository", "path", "ref"],
        );
        assert_eq!(definition["externalParameters"]["workflow"]["ref"], tag());
        exact_keys(
            &definition["internalParameters"],
            &[
                "rust_release",
                "target",
                "default_package_features",
                "openssl",
                "source_date_epoch",
            ],
        );
        assert_eq!(statement["subject"].as_array().unwrap().len(), 16);
        for subject in statement["subject"].as_array().unwrap() {
            exact_keys(subject, &["name", "digest"]);
            exact_keys(&subject["digest"], &["sha256"]);
            assert_eq!(
                subject["digest"]["sha256"],
                archives[subject["name"].as_str().unwrap()]["sha256"]
            );
        }
        let deps = definition["resolvedDependencies"].as_array().unwrap();
        for manifest in Cohort::Sixteen.manifests() {
            let path = Path::new(manifest).with_file_name("Cargo.lock");
            let label = path.parent().and_then(Path::file_name).map_or_else(
                || "Cargo.lock".into(),
                |p| format!("{}-Cargo.lock", p.to_string_lossy()),
            );
            let dep = deps.iter().find(|d| d["name"] == label).unwrap();
            assert_eq!(
                dep["digest"]["sha256"],
                record["cargo_lock_sha256s"][path.to_str().unwrap()]
            );
        }
        for (label, digest) in [
            ("source.tar", source["source_archive_sha256"].clone()),
            ("openssl-3.5.9.tar.gz", json!(OPENSSL_SHA)),
            (
                "openssl-executable",
                record["openssl"]["executable_sha256"].clone(),
            ),
            (
                "release-workflow",
                record["recipe_sha256"]["release-workflow"].clone(),
            ),
            ("cargo", record["tool_sha256"]["cargo"].clone()),
        ] {
            assert_eq!(
                deps.iter().find(|d| d["name"] == label).unwrap()["digest"]["sha256"],
                digest
            );
        }
        let details = &statement["predicate"]["runDetails"];
        exact_keys(details, &["builder", "metadata", "byproducts"]);
        exact_keys(&details["builder"], &["id"]);
        exact_keys(
            &details["metadata"],
            &["invocationId", "startedOn", "finishedOn"],
        );
        assert_eq!(details["byproducts"].as_array().unwrap().len(), 4);
        for byproduct in details["byproducts"].as_array().unwrap() {
            exact_keys(byproduct, &["name", "digest"]);
            let name = byproduct["name"].as_str().unwrap();
            assert!(![PROVENANCE, "RELEASE-CHECKSUMS.json"].contains(&name));
            assert_eq!(
                byproduct["digest"]["sha256"],
                tooling::sha(&fs::read(out.join(name)).unwrap())
            );
        }
        let public = root.join("target/release-preparation/public");
        let actual: BTreeSet<_> = fs::read_dir(&public)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        let expected = archives
            .keys()
            .cloned()
            .chain([PROVENANCE.into(), "RELEASE-CHECKSUMS.json".into()])
            .collect();
        assert_eq!(actual, expected);
        for name in &actual {
            assert!(
                !String::from_utf8_lossy(&fs::read(public.join(name)).unwrap())
                    .contains("PRIVATE SENTINEL")
            );
        }
        checksums::run("verify", &out).unwrap();
        checksums::run("verify", &public).unwrap();
        let manifest: Value =
            serde_json::from_slice(&fs::read(out.join("RELEASE-CHECKSUMS.json")).unwrap()).unwrap();
        let serialized = manifest.to_string();
        assert!(serialized.contains(PROVENANCE));
        assert!(serialized.contains("private-sentinel.log"));
        assert!(!serialized.contains("bundle"));
        assert!(!serialized.contains("RELEASE-CHECKSUMS.json"));
        assert!(!statement.to_string().contains("preparation_only"));
        assert!(!statement.to_string().contains(root.to_str().unwrap()));
        assert!(stage_public(root, &out, &archives).is_err()); // no overwrite
    }

    #[test]
    fn invalid_openssl_bytes_and_failed_staging_leave_no_public_ready_directory() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let out = root.join("target/release-preparation/private");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("openssl.tar.gz"), b"wrong archive").unwrap();
        assert!(openssl_source(&out.join("openssl.tar.gz")).is_err());
        fs::write(out.join("phase.log"), b"retained failure sentinel").unwrap();
        let archives = BTreeMap::from([("missing.crate".into(), json!({}))]);
        assert!(stage_public(root, &out, &archives).is_err());
        assert!(!root.join("target/release-preparation/public").exists());
        assert_eq!(
            fs::read(out.join("phase.log")).unwrap(),
            b"retained failure sentinel"
        );
        assert_eq!(fs::read_dir(out.parent().unwrap()).unwrap().count(), 1); // staging cleaned
    }

    #[cfg(unix)]
    #[test]
    fn failing_phase_retains_native_status_and_raw_log_without_final_outputs() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let source = root.join("source");
        let work = root.join("work");
        let out = root.join("private");
        for p in [&source, &work, &out] {
            fs::create_dir(p).unwrap();
        }
        let cargo = root.join("cargo");
        fs::write(
            &cargo,
            "#!/bin/sh\necho PRIVATE_FAILURE_SENTINEL\nexit 17\n",
        )
        .unwrap();
        fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).unwrap();
        let mut record = json!({});
        let error = private::build_packages(
            &source,
            &work,
            &out,
            Cohort::Sixteen,
            &cargo,
            &[],
            &mut record,
        )
        .unwrap_err();
        assert_eq!(error.code, 17);
        assert!(
            fs::read_to_string(out.join("vendor.log"))
                .unwrap()
                .contains("PRIVATE_FAILURE_SENTINEL")
        );
        let record: Value =
            serde_json::from_slice(&fs::read(out.join("BUILD-RECORD.json")).unwrap()).unwrap();
        assert_eq!(record["vendor_native_status"]["native_exit_code"], 17);
        assert_eq!(record["vendor_native_status"]["cleanup_ok"], true);
        assert!(!out.join(PROVENANCE).exists());
        assert!(!out.join("RELEASE-CHECKSUMS.json").exists());
        assert!(!root.join("public").exists());
    }
}
