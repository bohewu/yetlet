use std::{fs, process::Command};
use yetlet_xtask::{Cli, parse, repository_root};

#[test]
fn qualification_selects_the_original_capabilities_through_two_packages() {
    use std::collections::BTreeSet;
    let root = repository_root();
    let source = fs::read_to_string(root.join(".github/workflows/qualification.yml")).unwrap();
    assert!(source.contains("os: [ubuntu-24.04, windows-2025, macos-15]"));
    assert!(source.contains("packages=(-p yetlet -p yetlet-browser)"));
    let selection = source
        .split("features=(--no-default-features --features '")
        .nth(1)
        .unwrap()
        .split('\'')
        .next()
        .unwrap();
    let features: BTreeSet<_> = selection.split(',').collect();
    assert_eq!(
        features,
        BTreeSet::from([
            "yetlet/html",
            "yetlet/actions",
            "yetlet/reactive",
            "yetlet/server-axum",
            "yetlet/request-id",
            "yetlet/session",
            "yetlet-browser/dom",
            "yetlet-browser/dom-keyed",
            "yetlet-browser/actions",
        ])
    );
    for selected in features {
        let (package, feature) = selected.split_once('/').unwrap();
        let manifest: toml::Value = toml::from_str(
            &fs::read_to_string(root.join("crates").join(package).join("Cargo.toml")).unwrap(),
        )
        .unwrap();
        assert!(manifest["features"].get(feature).is_some(), "{selected}");
    }
    assert_eq!(
        source
            .matches("\"${packages[@]}\" \"${features[@]}\"")
            .count(),
        3
    );
    assert!(!source.contains("-p yetlet-core"));
    assert!(!source.contains("--all-features"));
}

#[test]
fn release_cli_is_closed_and_requires_explicit_lowercase_commit() {
    let commit = "a".repeat(40);
    assert_eq!(
        parse(["release", "preparation", "--expected-commit", &commit].map(Into::into)).unwrap(),
        Cli::ReleasePreparation {
            expected_commit: commit
        }
    );
    for args in [
        vec!["release", "preparation"],
        vec!["release", "preparation", "--expected-commit", "main"],
        vec![
            "release",
            "preparation",
            "--expected-commit",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ],
        vec!["release", "preparation", "--cohort", "original-nine"],
        vec![
            "release",
            "preparation",
            "--expected-commit",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--cohort",
            "sixteen",
        ],
    ] {
        assert!(parse(args.into_iter().map(Into::into)).is_err());
    }
}

#[test]
fn workflow_has_manual_fixed_source_and_only_success_public_upload() {
    let source =
        fs::read_to_string(repository_root().join(".github/workflows/release.yml")).unwrap();
    for required in [
        "on:\n  workflow_dispatch:",
        "record:",
        "RELEASE_RECORD: ${{ inputs.record }}",
        "contents: read",
        "timeout-minutes: 45",
        "ref: ${{ github.sha }}",
        "fetch-depth: 0",
        "persist-credentials: false",
        "actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09",
        "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
        "[[ \"$GITHUB_EVENT_NAME\" == workflow_dispatch ]]",
        "[[ \"$GITHUB_REPOSITORY\" == bohewu/yetlet ]]",
        "[[ \"$GITHUB_REPOSITORY_ID\" == 1409587171 && \"$GITHUB_REPOSITORY_OWNER_ID\" == 35250002 ]]",
        "[[ \"$GITHUB_SHA\" == \"$BUILD_WORKFLOW_SHA\" ]]",
        "assert re.fullmatch('[0-9a-f]{40}', r['source'])",
        "assert r['source'] == os.environ['GITHUB_SHA']",
        "assert os.environ['BUILD_WORKFLOW_REF'] == 'bohewu/yetlet/.github/workflows/release.yml@' + os.environ['GITHUB_REF']",
        "python3 scripts/release_record.py source release.yml",
        "RELEASE_COHORT=' + r['cohort']",
        "OPENSSL_SOURCE_ARCHIVE=%s",
        "if: success()",
        "path: target/release-preparation/public/",
    ] {
        assert!(source.contains(required), "missing {required}");
    }
    for forbidden in [
        "pull_request:",
        "pull_request_target:",
        "workflow_run:",
        "push:",
        "id-token:",
        "v0.1.0",
        "v0.9.0-alpha.1",
        "secrets.",
        "environment:",
        "actions/cache@",
        "if: always()",
        "path: target/release-preparation/\n",
        "path: target/release-preparation/private/",
    ] {
        assert!(!source.contains(forbidden), "unexpected {forbidden}");
    }
    assert_eq!(source.matches("uses:").count(), 3);
    for required in [
        "steps.trusted.outputs.trusted == 'true'",
        "steps.custody_tools.outcome == 'success'",
        "steps.custody_helper.outcome == 'success'",
        "steps.custody_transport.outcome == 'success'",
        "target/release-custody/private-evidence.tar.age",
        "target/release-custody/transport-receipt.json",
        "native_exit=%s",
        "Private custody: NOT SAVED",
        "Durable custody is NOT VERIFIED",
    ] {
        assert!(source.contains(required), "missing {required}");
    }
    assert!(
        source.find("Reject source and workflow identity").unwrap()
            < source
                .find("bash scripts/bootstrap-release-age.sh")
                .unwrap()
    );
    assert!(
        source
            .find("python3 scripts/release_record.py source release.yml")
            .unwrap()
            < source
                .find("bash scripts/bootstrap-release-age.sh")
                .unwrap()
    );
    assert!(
        source.find("Reject source and workflow identity").unwrap()
            < source.find("rustup toolchain install").unwrap()
    );
    assert!(source.find("sha256sum --check").unwrap() < source.find("tar -xzf").unwrap());
    assert!(!source.contains("rm -rf \"$preparation_archive\""));
}

#[test]
fn legacy_private_entry_refuses_release_workflow_identity_before_build() {
    let root = repository_root();
    let head = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let head = String::from_utf8(head.stdout).unwrap();
    let head = head.trim();
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["release", "private-rebuild"])
        .env("GITHUB_SHA", head)
        .env("GITHUB_REPOSITORY", "bohewu/yetlet")
        .env("BUILD_WORKFLOW_SHA", head)
        .env(
            "BUILD_WORKFLOW_REF",
            "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1",
        )
        .env("GITHUB_REF", "refs/tags/v0.9.0-alpha.1")
        .env("GITHUB_EVENT_NAME", "workflow_dispatch")
        .env("RUNNER_TEMP", temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    // The actual checkout is allowed to be dirty during development; both guards precede tools.
    assert!(
        error.contains("clean source required")
            || error.contains("workflow/source identity mismatch"),
        "{error}"
    );
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
