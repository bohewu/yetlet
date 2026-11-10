use super::*;
use crate::{
    creation::{self, Options},
    scaffold::Template,
    sdk_registry::{self as registry, Record, Registry, Reply},
};
use std::{collections::VecDeque, path::Path, time::Duration};

fn temporary() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

const A6: &str = "0.9.0-alpha.6";
const A7: &str = "0.9.0-alpha.7";
fn row(name: &str, version: &str) -> serde_json::Value {
    serde_json::json!({"name":name,"vers":version,"cksum":"f".repeat(64),"yanked":false,"rust_version":"1.99","features":{"server-axum":[],"config":[],"reactive":[],"dom":[],"dom-keyed":[]},"deps":if name == "yetlet-browser" { vec![serde_json::json!({"name":"yetlet","req":format!("={version}")})] } else {vec![]},"v":2})
}
fn body(rows: &[serde_json::Value]) -> String {
    rows.iter().map(|r| r.to_string() + "\n").collect()
}
fn record(name: &str, rows: &[serde_json::Value]) -> Record {
    let body = body(rows);
    Record {
        name: name.into(),
        sha256: digest(body.as_bytes()),
        body,
        validated_at: 100,
        etag: Some("\"fixture\"".into()),
    }
}
fn request(template: Template, exact: Option<&str>, offline: bool) -> Request<'_> {
    Request {
        exact,
        template,
        settings: false,
        offline,
    }
}
fn policy() -> Policy {
    serde_json::from_str(POLICY).unwrap()
}
fn native() -> Record {
    record(
        "yetlet",
        &[
            row("yetlet", A6),
            row("yetlet", A7),
            row("yetlet", "0.9.0-alpha.8"),
            row("yetlet", "1.0.0"),
        ],
    )
}

struct Fake {
    replies: VecDeque<Result<Reply, String>>,
    calls: usize,
}
impl Fake {
    fn records(records: &[Record]) -> Self {
        Self {
            replies: records
                .iter()
                .map(|r| Ok(Reply::Modified(r.body.clone(), r.etag.clone())))
                .collect(),
            calls: 0,
        }
    }
    fn never() -> Self {
        Self {
            replies: VecDeque::new(),
            calls: 0,
        }
    }
}
impl Registry for Fake {
    fn fetch(&mut self, _: &str, _: Option<&str>, _: Duration) -> Result<Reply, String> {
        self.calls += 1;
        self.replies
            .pop_front()
            .expect("unexpected network request")
    }
}

#[test]
fn exact_semver_and_new_parser_are_bounded() {
    for value in [A6, A7, "1.2.3", "1.2.3-rc.10"] {
        assert_eq!(exact(value).unwrap().to_string(), value);
    }
    for value in [
        "",
        "v1.2.3",
        "=1.2.3",
        "^1.2.3",
        "1.2",
        "*",
        "1.2.3+build",
        "1.2.3-alpha.01",
        "01.2.3",
        " 1.2.3",
        "1.2.3 ",
        "https://example.invalid",
    ] {
        assert!(exact(value).is_err(), "{value}");
    }
    for flags in [
        vec!["new", "demo", "--sdk-version", A7, "--offline"],
        vec!["new", "--sdk-version=0.9.0-alpha.7", "demo", "--offline"],
    ] {
        let args = flags
            .iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>();
        assert!(
            matches!(crate::parse(&args),Ok(crate::Cli::New{sdk_version:Some(v),offline:true,..}) if v == A7)
        );
    }
    for flags in [
        vec!["new", "demo", "--sdk-version"],
        vec!["new", "demo", "--sdk-version="],
        vec!["new", "demo", "--sdk-version", A7, "--sdk-version", A6],
        vec!["new", "demo", "--offline", "--offline"],
        vec!["build", "--offline"],
        vec!["run", "--sdk-version", A7],
    ] {
        assert!(
            crate::parse(
                &flags
                    .iter()
                    .map(std::ffi::OsString::from)
                    .collect::<Vec<_>>()
            )
            .is_err()
        );
    }
}

#[test]
fn selection_filters_availability_and_explicit_errors_are_distinct() {
    let req = request(Template::Native, None, false);
    assert_eq!(
        select(req, &[native()], "registry-refresh", &policy())
            .unwrap()
            .version,
        "1.0.0"
    );
    let mut rows = vec![row("yetlet", A6), row("yetlet", A7)];
    rows[1]["yanked"] = true.into();
    let records = [record("yetlet", &rows)];
    assert_eq!(
        select(req, &records, "registry-refresh", &policy())
            .unwrap()
            .version,
        A6
    );
    assert!(
        select(
            request(Template::Native, Some(A7), false),
            &records,
            "registry-refresh",
            &policy()
        )
        .unwrap_err()
        .starts_with("SDK_VERSION_YANKED")
    );
    assert_eq!(
        select(
            request(Template::Native, Some("0.9.0-alpha.8"), false),
            &[native()],
            "registry-refresh",
            &policy()
        )
        .unwrap()
        .version,
        "0.9.0-alpha.8"
    );
    assert!(
        select(
            request(Template::Native, Some("0.9.0-alpha.99"), false),
            &[native()],
            "offline-cache",
            &policy()
        )
        .unwrap_err()
        .contains("SDK_VERSION_NOT_FOUND")
    );
    rows[0]["yanked"] = true.into();
    assert!(
        select(
            request(Template::NativeStyle, None, false),
            &[record("yetlet", &rows)],
            "registry-refresh",
            &policy()
        )
        .unwrap_err()
        .starts_with("SDK_NO_COMPATIBLE_VERSION")
    );
}

#[test]
fn unchanged_policy_selects_future_semver_for_every_template() {
    let policy = policy();
    for template in [
        Template::Native,
        Template::Browser,
        Template::NativeStyle,
        Template::BrowserStyle,
    ] {
        let mut core = vec![row("yetlet", A7), row("yetlet", "0.8.0")];
        let mut companion = vec![row("yetlet-browser", A7), row("yetlet-browser", "0.8.0")];
        for latest in [
            "0.9.0-alpha.9",
            "0.9.0-alpha.10",
            "1.0.0",
            "2.0.0-rc.1",
            "2.0.0",
        ] {
            core.insert(0, row("yetlet", latest));
            companion.insert(0, row("yetlet-browser", latest));
            let records = [
                record("yetlet", &core),
                record("yetlet-browser", &companion),
            ];
            let sdk = select(
                request(template, None, false),
                &records,
                "registry-refresh",
                &policy,
            )
            .unwrap();
            assert_eq!(sdk.version, latest);
            assert_eq!(sdk.policy_sha256, digest(POLICY.as_bytes()));
            assert_eq!(sdk.prerelease, !exact(latest).unwrap().pre.is_empty());
            assert!(!sdk.notice(100).contains("reviewed"));
            assert_eq!(
                select(
                    request(template, Some(A7), false),
                    &records,
                    "registry-refresh",
                    &policy
                )
                .unwrap()
                .version,
                A7
            );
        }
    }
}

#[test]
fn rust_metadata_does_not_impose_a_cli_admission_ceiling() {
    for rust_version in [
        serde_json::json!("1.100"),
        serde_json::json!("2.5"),
        serde_json::Value::Null,
    ] {
        let mut core = row("yetlet", "2.0.0");
        let mut companion = row("yetlet-browser", "2.0.0");
        core["rust_version"] = rust_version.clone();
        companion["rust_version"] = rust_version;
        let records = [
            record("yetlet", &[row("yetlet", A7), core]),
            record("yetlet-browser", &[row("yetlet-browser", A7), companion]),
        ];
        for template in [Template::Native, Template::Browser] {
            assert_eq!(
                select(
                    request(template, None, false),
                    &records,
                    "registry-refresh",
                    &policy()
                )
                .unwrap()
                .version,
                "2.0.0"
            );
        }
    }
}

#[test]
fn latest_feature_or_dependency_failure_does_not_select_an_older_sdk() {
    for (template, problem) in [
        (Template::Native, "server-axum"),
        (Template::Browser, "dom"),
        (Template::Browser, "dependency"),
    ] {
        let mut core = row("yetlet", "2.0.0");
        let mut companion = row("yetlet-browser", "2.0.0");
        match problem {
            "server-axum" => {
                core["features"].as_object_mut().unwrap().remove(problem);
            }
            "dom" => {
                companion["features"]["dom"] = serde_json::json!(["missing"]);
            }
            _ => companion["deps"][0]["req"] = "=0.8.0".into(),
        }
        let records = [
            record("yetlet", &[row("yetlet", A7), core]),
            record("yetlet-browser", &[row("yetlet-browser", A7), companion]),
        ];
        for exact in [None, Some("2.0.0")] {
            let error = select(
                request(template, exact, false),
                &records,
                "registry-refresh",
                &policy(),
            )
            .unwrap_err();
            assert!(error.starts_with("SDK_INCOMPATIBLE: 2.0.0"), "{error}");
            assert!(error.contains(problem), "{error}");
        }
    }
}

#[test]
fn browser_default_skips_only_missing_or_yanked_pairs() {
    let core = record(
        "yetlet",
        &[
            row("yetlet", A7),
            row("yetlet", "1.0.0"),
            row("yetlet", "2.0.0"),
        ],
    );
    let mut yanked = row("yetlet-browser", "1.0.0");
    yanked["yanked"] = true.into();
    let records = [
        core,
        record("yetlet-browser", &[row("yetlet-browser", A7), yanked]),
    ];
    assert_eq!(
        select(
            request(Template::Browser, None, false),
            &records,
            "registry-refresh",
            &policy()
        )
        .unwrap()
        .version,
        A7
    );
}

#[test]
fn browser_pair_yank_features_and_dependency_are_required() {
    let core = native();
    assert!(
        select(
            request(Template::Browser, Some(A7), false),
            &[core.clone()],
            "registry-refresh",
            &policy()
        )
        .unwrap_err()
        .starts_with("SDK_PACKAGE_UNAVAILABLE")
    );
    let mut companion = row("yetlet-browser", A7);
    companion["yanked"] = true.into();
    assert!(
        select(
            request(Template::Browser, Some(A7), false),
            &[core.clone(), record("yetlet-browser", &[companion])],
            "registry-refresh",
            &policy()
        )
        .unwrap_err()
        .starts_with("SDK_VERSION_YANKED")
    );
    for change in ["feature", "dependency"] {
        let mut companion = row("yetlet-browser", A7);
        match change {
            "feature" => companion["features"]["dom"] = serde_json::json!(["missing"]),
            _ => companion["deps"][0]["req"] = "=0.8.0".into(),
        };
        assert!(
            select(
                request(Template::Browser, Some(A7), false),
                &[core.clone(), record("yetlet-browser", &[companion])],
                "registry-refresh",
                &policy()
            )
            .unwrap_err()
            .starts_with("SDK_INCOMPATIBLE"),
            "{change}"
        );
    }
    let mut core = row("yetlet", A7);
    core["features"].as_object_mut().unwrap().remove("reactive");
    core["features2"] = serde_json::json!({"reactive":[]});
    assert!(
        select(
            request(Template::Browser, Some(A7), false),
            &[
                record("yetlet", &[core]),
                record("yetlet-browser", &[row("yetlet-browser", A7)])
            ],
            "registry-refresh",
            &policy()
        )
        .is_ok()
    );
}

#[test]
fn invalid_index_identity_schema_duplicates_and_checksum_fail_closed() {
    let good = row("yetlet", A7);
    assert!(index("yetlet", &body(&[good.clone(), good.clone()])).is_err());
    for field in ["name", "vers", "cksum", "v"] {
        let mut bad = good.clone();
        bad[field] = match field {
            "v" => 3.into(),
            _ => "invalid".into(),
        };
        assert!(index("yetlet", &body(&[bad])).is_err(), "{field}");
    }
}

#[test]
fn offline_hit_miss_age_and_online_304_are_explicit() {
    let root = temporary();
    let req = request(Template::Native, None, true);
    assert!(
        registry::resolve(req, &mut Fake::never(), root.path(), 100)
            .unwrap_err()
            .starts_with("SDK_OFFLINE_CACHE_MISS")
    );
    registry::resolve(
        Request {
            offline: false,
            ..req
        },
        &mut Fake::records(&[native()]),
        root.path(),
        100,
    )
    .unwrap();
    let sdk = registry::resolve(req, &mut Fake::never(), root.path(), 200).unwrap();
    assert_eq!(sdk.source, "offline-cache");
    assert!(sdk.notice(200).contains("age=100s"));
    let mut not_modified = Fake {
        replies: VecDeque::from([Ok(Reply::NotModified)]),
        calls: 0,
    };
    let sdk = registry::resolve(
        Request {
            offline: false,
            ..req
        },
        &mut not_modified,
        root.path(),
        250,
    )
    .unwrap();
    assert_eq!(sdk.source, "registry-refresh");
    assert_eq!(sdk.packages[0].validated_at, 250);
    assert!(
        registry::resolve(req, &mut Fake::never(), root.path(), 249)
            .unwrap_err()
            .starts_with("SDK_METADATA_INVALID")
    );
    std::fs::write(root.path().join("crates-io-native.json"), "corrupt").unwrap();
    assert!(
        registry::resolve(req, &mut Fake::never(), root.path(), 300)
            .unwrap_err()
            .starts_with("SDK_METADATA_INVALID")
    );
}

#[test]
fn refreshed_metadata_advances_default_while_offline_reports_cache_age() {
    let root = temporary();
    let req = request(Template::Native, None, false);
    let first = registry::resolve(req, &mut Fake::records(&[native()]), root.path(), 100).unwrap();
    assert_eq!(first.version, "1.0.0");
    let records = [record(
        "yetlet",
        &[row("yetlet", "1.0.0"), row("yetlet", "2.0.0-rc.10")],
    )];
    let next = registry::resolve(req, &mut Fake::records(&records), root.path(), 200).unwrap();
    assert_eq!(next.version, "2.0.0-rc.10");
    assert_eq!(next.source, "registry-refresh");
    let cached = registry::resolve(
        Request {
            offline: true,
            ..req
        },
        &mut Fake::never(),
        root.path(),
        300,
    )
    .unwrap();
    assert_eq!(cached.version, next.version);
    assert_eq!(cached.source, "offline-cache");
    assert!(cached.notice(300).contains("latest known to this cache"));
    assert!(cached.notice(300).contains("age=100s"));
}

#[test]
fn network_and_partial_refresh_never_replace_or_use_stale_set() {
    let root = temporary();
    let req = request(Template::Browser, None, false);
    let records = [
        native(),
        record("yetlet-browser", &[row("yetlet-browser", A7)]),
    ];
    registry::resolve(req, &mut Fake::records(&records), root.path(), 100).unwrap();
    let path = root.path().join("crates-io-browser.json");
    let before = std::fs::read(&path).unwrap();
    for error in ["timeout", "TLS", "HTTP 429"] {
        let mut fake = Fake {
            replies: VecDeque::from([
                Ok(Reply::Modified(records[0].body.clone(), None)),
                Err(format!("SDK_REGISTRY_UNAVAILABLE: {error}")),
            ]),
            calls: 0,
        };
        assert!(
            registry::resolve(req, &mut fake, root.path(), 200)
                .unwrap_err()
                .starts_with("SDK_REGISTRY_UNAVAILABLE")
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
    assert_eq!(
        registry::resolve(
            Request {
                offline: true,
                ..req
            },
            &mut Fake::never(),
            root.path(),
            300
        )
        .unwrap()
        .version,
        A7
    );
    assert!(!root.path().join("crates-io-native.json").exists());
}

fn options(template: Template) -> Options {
    Options {
        template,
        service: false,
        settings: false,
        tailwind: false,
        vite: false,
    }
}
fn fake_lock(path: &Path, sdk: &ResolvedSdk) {
    let mut lock = "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n".to_owned();
    for p in &sdk.packages {
        lock.push_str(&format!(
            "\n[[package]]\nname = {:?}\nversion = {:?}\nsource = {:?}\nchecksum = {:?}\n",
            p.name, sdk.version, CARGO_REGISTRY, p.checksum
        ));
    }
    std::fs::write(path.join("Cargo.lock"), lock).unwrap();
}

#[test]
fn scaffold_pin_receipt_style_lock_repeated_destination_and_failure_cleanup() {
    for template in [
        Template::Native,
        Template::Browser,
        Template::NativeStyle,
        Template::BrowserStyle,
    ] {
        let root = temporary();
        let cache = root.path().join("cache");
        let records = if template.browser() {
            vec![
                native(),
                record("yetlet-browser", &[row("yetlet-browser", A7)]),
            ]
        } else {
            vec![native()]
        };
        let req = request(template, Some(A7), false);
        let sdk = select(req, &records, "registry-refresh", &policy()).unwrap();
        creation::create(
            "demo",
            root.path(),
            options(template),
            req,
            &mut Fake::records(&records),
            &cache,
            100,
            || Ok(()),
            |path, _| {
                fake_lock(path, &sdk);
                Ok(())
            },
        )
        .unwrap();
        let app = root.path().join("demo");
        let manifest = std::fs::read_to_string(app.join("Cargo.toml")).unwrap();
        assert!(manifest.contains(&format!("={A7}")) && !manifest.contains("path ="));
        let receipt: serde_json::Value = serde_json::from_slice(
            &std::fs::read(app.join(".yetlet/sdk-resolution.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["sdk"]["version"], A7);
        assert_eq!(receipt["sdk"]["source"], "registry-refresh");
        let readme = std::fs::read_to_string(app.join("README.md")).unwrap();
        assert!(!readme.contains("cargo generate-lockfile"));
        assert!(!readme.contains("no Cargo lock"));
        let lock = std::fs::read(app.join("Cargo.lock")).unwrap();
        creation::validate_lock(&lock, &sdk).unwrap();
        assert_eq!(receipt["cargo_lock_sha256"], digest(&lock));
        if matches!(template, Template::NativeStyle | Template::BrowserStyle) {
            assert_eq!(
                std::fs::read(app.join("stylegen/Cargo.lock")).unwrap(),
                include_bytes!("../../templates/styles/Cargo.lock")
            );
        }
        assert!(
            creation::create(
                "demo",
                root.path(),
                options(template),
                req,
                &mut Fake::never(),
                &cache,
                200,
                || panic!("existing target must precede probes"),
                |_, _| panic!("existing target must precede Cargo")
            )
            .is_err()
        );
        assert_eq!(std::fs::read(app.join("Cargo.lock")).unwrap(), lock);
        let error = creation::create(
            "failed",
            root.path(),
            options(template),
            Request {
                offline: true,
                ..req
            },
            &mut Fake::never(),
            &cache,
            200,
            || Ok(()),
            |path, _| {
                std::fs::write(path.join("Cargo.lock"), "partial").unwrap();
                Err("SDK_LOCK_FAILED: fixture".into())
            },
        )
        .unwrap_err();
        assert!(error.contains("SDK_LOCK_FAILED"));
        assert!(!root.path().join("failed").exists());
        assert!(!std::fs::read_dir(root.path()).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".yetlet-")
        }));
    }
}

#[test]
fn latest_cargo_failures_preserve_diagnostics_and_never_retry_an_older_sdk() {
    for diagnostic in [
        "rustc is too old for selected SDK",
        "no matching transitive dependency found",
    ] {
        let root = temporary();
        let mut locks = 0;
        let mut registry = Fake::records(&[native()]);
        let error = creation::create(
            "demo",
            root.path(),
            options(Template::Native),
            request(Template::Native, None, false),
            &mut registry,
            &root.path().join("cache"),
            100,
            || Ok(()),
            |stage, _| {
                locks += 1;
                let manifest = std::fs::read_to_string(stage.join("Cargo.toml")).unwrap();
                assert!(manifest.contains("=1.0.0"));
                Err(format!("SDK_LOCK_FAILED: {diagnostic}"))
            },
        )
        .unwrap_err();
        assert!(error.contains(diagnostic), "{error}");
        assert!(error.contains("selected SDK 1.0.0"), "{error}");
        assert_eq!(locks, 1);
        assert_eq!(registry.calls, 1);
        assert!(!root.path().join("demo").exists());
        assert!(!std::fs::read_dir(root.path()).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".yetlet-")
        }));
    }
}

#[test]
fn lock_mismatch_rejects_wrong_version_source_and_checksum() {
    let req = request(Template::Native, Some(A7), false);
    let sdk = select(req, &[native()], "registry-refresh", &policy()).unwrap();
    let root = temporary();
    fake_lock(root.path(), &sdk);
    let bytes = std::fs::read_to_string(root.path().join("Cargo.lock")).unwrap();
    for text in [
        bytes.replace(A7, A6),
        bytes.replace(CARGO_REGISTRY, "git+https://example.invalid"),
        bytes.replace(&"f".repeat(64), &"0".repeat(64)),
    ] {
        assert!(creation::validate_lock(text.as_bytes(), &sdk).is_err());
    }
}

#[test]
fn post_creation_skipped_browser_build_preserves_manifest_locks_and_receipt() {
    let latest = "2.0.0-rc.10";
    let root = temporary();
    let records = [
        record("yetlet", &[row("yetlet", A7), row("yetlet", latest)]),
        record(
            "yetlet-browser",
            &[row("yetlet-browser", A7), row("yetlet-browser", latest)],
        ),
    ];
    let req = request(Template::Browser, None, false);
    let sdk = select(req, &records, "registry-refresh", &policy()).unwrap();
    creation::create(
        "demo",
        root.path(),
        options(Template::Browser),
        req,
        &mut Fake::records(&records),
        &root.path().join("cache"),
        100,
        || Ok(()),
        |path, _| {
            fake_lock(path, &sdk);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(sdk.version, latest);
    let app = root.path().join("demo");
    let base = app.join("base.toml");
    std::fs::write(
        &base,
        std::fs::read_to_string(&base)
            .unwrap()
            .replace("auto_build = true", "auto_build = false"),
    )
    .unwrap();
    let paths = ["Cargo.toml", "Cargo.lock", ".yetlet/sdk-resolution.json"];
    let before = paths.map(|p| std::fs::read(app.join(p)).unwrap());
    assert!(matches!(
        crate::app::run(&app.join("Cargo.toml"), false, false).unwrap(),
        crate::app::Completion::Skipped
    ));
    assert_eq!(before, paths.map(|p| std::fs::read(app.join(p)).unwrap()));
}

#[test]
#[ignore = "opt-in real crates.io integration; ordinary tests use fixtures"]
fn live_registry_native_browser_and_default_locks() {
    let root = temporary();
    let cache = root.path().join("cache");
    let now = registry::now().unwrap();
    for (name, template, version) in [
        ("nativeapp", Template::Native, Some(A7)),
        ("browserapp", Template::Browser, Some(A7)),
        ("defaultapp", Template::Native, None),
    ] {
        let req = request(template, version, false);
        let sdk = creation::create(
            name,
            root.path(),
            options(template),
            req,
            &mut registry::CratesIo,
            &cache,
            now,
            || Ok(()),
            super::super::creation::generate_lock,
        )
        .unwrap();
        if let Some(version) = version {
            assert_eq!(sdk.version, version);
        }
        creation::validate_lock(
            &std::fs::read(root.path().join(name).join("Cargo.lock")).unwrap(),
            &sdk,
        )
        .unwrap();
    }
}

#[test]
#[ignore = "opt-in registry SDK/template compile compatibility; no browser runtime qualification"]
fn registry_template_compile_compatibility() {
    let root = temporary();
    let cache = root.path().join("cache");
    let now = registry::now().unwrap();
    for (index, (template, version)) in [
        (Template::Native, A6),
        (Template::Native, A7),
        (Template::Browser, A6),
        (Template::Browser, A7),
        (Template::NativeStyle, A7),
        (Template::BrowserStyle, A7),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("compat-{index}");
        creation::create(
            &name,
            root.path(),
            options(template),
            request(template, Some(version), false),
            &mut registry::CratesIo,
            &cache,
            now,
            || Ok(()),
            creation::generate_lock,
        )
        .unwrap();
        let app = root.path().join(&name);
        let mut command = std::process::Command::new("cargo");
        command
            .args(["check", "--locked", "--manifest-path"])
            .arg(app.join("Cargo.toml"))
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("RUSTUP_SKIP_UPDATE_CHECK", "1");
        if template.browser() {
            command.args(["--target", "wasm32-unknown-unknown"]);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{} SDK {version}: {}",
            template.selector(),
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!(
            "COMPILE PASS {} SDK {version} ({})",
            template.selector(),
            if template.browser() {
                "wasm32"
            } else {
                "native"
            }
        );
    }
}

fn vendor_package(root: &Path, name: &str, version: &str, features: &[&str]) {
    let dir = root.join(format!("{name}-{version}"));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let mut manifest =
        format!("[package]\nname={name:?}\nversion={version:?}\nedition=\"2024\"\n[features]\n");
    for feature in features {
        manifest.push_str(&format!("{feature:?}=[]\n"));
    }
    std::fs::write(dir.join("Cargo.toml"), &manifest).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(dir.join(".cargo-checksum.json"),serde_json::json!({"package":"f".repeat(64),"files":{"Cargo.toml":digest(manifest.as_bytes()),"src/lib.rs":digest(b"")}}).to_string()).unwrap();
}

#[test]
fn real_cargo_fixture_source_creates_reproducible_native_and_browser_locks() {
    let latest = "2.0.0-rc.10";
    let root = temporary();
    let vendor = root.path().join("vendor");
    vendor_package(&vendor, "yetlet", latest, &["server-axum", "reactive"]);
    vendor_package(&vendor, "yetlet-browser", latest, &["dom", "dom-keyed"]);
    vendor_package(&vendor, "axum", "0.8.9", &[]);
    vendor_package(
        &vendor,
        "tokio",
        "1.53.3",
        &["macros", "net", "rt-multi-thread"],
    );
    vendor_package(&vendor, "wasm-bindgen", "0.2.129", &[]);
    vendor_package(
        &vendor,
        "web-sys",
        "0.3.85",
        &[
            "Document",
            "Element",
            "Event",
            "EventTarget",
            "HtmlElement",
            "HtmlInputElement",
            "Node",
            "Window",
        ],
    );
    for template in [Template::Native, Template::Browser] {
        let mut prior = None;
        for _ in 0..2 {
            let destination = temporary();
            let records = if template.browser() {
                vec![
                    record("yetlet", &[row("yetlet", A7), row("yetlet", latest)]),
                    record("yetlet-browser", &[row("yetlet-browser", latest)]),
                ]
            } else {
                vec![record(
                    "yetlet",
                    &[row("yetlet", A7), row("yetlet", latest)],
                )]
            };
            let sdk = creation::create(
                "demo",
                destination.path(),
                options(template),
                request(template, None, false),
                &mut Fake::records(&records),
                &root.path().join("metadata"),
                100,
                || Ok(()),
                |stage, _| {
                    // Only this injected test runner replaces the Cargo source with
                    // local fixture packages. Production has no registry override flag.
                    let output = std::process::Command::new("cargo")
                        .args(["generate-lockfile", "--offline", "--manifest-path"])
                        .arg(stage.join("Cargo.toml"))
                        .args([
                            "--config",
                            "source.crates-io.replace-with=\"fixture\"",
                            "--config",
                        ])
                        .arg(format!(
                            "source.fixture.directory={}",
                            serde_json::to_string(&vendor).unwrap()
                        ))
                        .env("CARGO_HOME", root.path().join("cargo-home"))
                        .env("RUSTUP_AUTO_INSTALL", "0")
                        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
                        .current_dir(stage)
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    Ok(())
                },
            )
            .unwrap();
            assert_eq!(sdk.version, latest);
            let app = destination.path().join("demo");
            let bytes = std::fs::read(app.join("Cargo.lock")).unwrap();
            creation::validate_lock(&bytes, &sdk).unwrap();
            let current = (std::fs::read(app.join("Cargo.toml")).unwrap(), bytes);
            if let Some(prior) = prior {
                assert_eq!(current, prior);
            }
            prior = Some(current);
        }
    }
}

#[test]
fn concurrent_cache_refreshes_leave_one_complete_set() {
    let root = temporary();
    std::thread::scope(|scope| {
        for _ in 0..2 {
            let path = root.path();
            scope.spawn(move || {
                registry::resolve(
                    request(Template::Browser, None, false),
                    &mut Fake::records(&[
                        native(),
                        record("yetlet-browser", &[row("yetlet-browser", A7)]),
                    ]),
                    path,
                    100,
                )
                .unwrap();
            });
        }
    });
    assert_eq!(
        registry::resolve(
            request(Template::Browser, None, true),
            &mut Fake::never(),
            root.path(),
            200
        )
        .unwrap()
        .packages
        .len(),
        2
    );
}
