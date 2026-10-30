use serde_json::json;
use std::{collections::BTreeSet, ffi::OsString, fs, path::Path};
use yetlet_xtask::{
    Cli, parse,
    release::private::{self, Cohort},
};

#[test]
fn closed_cohort_cli_preserves_legacy_selection_and_rejects_extra_inputs() {
    assert_eq!(
        parse(["release", "private-rebuild"].map(Into::into)).unwrap(),
        Cli::PrivateRebuild {
            cohort: Cohort::Consolidated
        }
    );
    assert_eq!(
        parse(["release", "private-rebuild", "--cohort", "consolidated"].map(Into::into)).unwrap(),
        Cli::PrivateRebuild {
            cohort: Cohort::Consolidated
        }
    );
    assert_eq!(
        parse(["release", "private-rebuild", "--cohort", "original-nine"].map(Into::into)).unwrap(),
        Cli::PrivateRebuild {
            cohort: Cohort::OriginalNine
        }
    );
    assert_eq!(
        parse(["release", "private-rebuild", "--cohort", "sixteen"].map(Into::into)).unwrap(),
        Cli::PrivateRebuild {
            cohort: Cohort::Sixteen
        }
    );
    for args in [
        vec!["--cohort"],
        vec!["--cohort", "all"],
        vec!["--cohort", ""],
        vec!["--cohort=sixteen"],
        vec!["--cohort", "sixteen", "extra"],
        vec!["--cohort", "sixteen", "--cohort", "original-nine"],
    ] {
        let mut argv: Vec<OsString> = ["release", "private-rebuild"].map(Into::into).to_vec();
        argv.extend(args.into_iter().map(OsString::from));
        assert!(parse(argv).is_err());
    }
}

fn package_version() -> String {
    let manifest: toml::Value = toml::from_str(include_str!("../../Cargo.toml")).unwrap();
    manifest["workspace"]["package"]["version"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn fixture(source: &Path) {
    let version = package_version();
    fs::create_dir_all(source).unwrap();
    let members: Vec<_> = Cohort::Sixteen
        .names()
        .iter()
        .filter(|n| {
            ![
                "yetlet-config",
                "yetlet-secrets",
                "yetlet-signing",
                "yetlet-session-redis",
                "yetlet-wgpu",
            ]
            .contains(n)
        })
        .map(|n| format!("crates/{n}"))
        .collect();
    fs::write(
        source.join("Cargo.toml"),
        format!(
            "[workspace]\nresolver=\"3\"\nexclude=[\"crates/yetlet-config\",\"crates/yetlet-secrets\",\"crates/yetlet-signing\",\"crates/yetlet-session-redis\",\"crates/yetlet-wgpu\"]\nmembers={}\n",
            toml::Value::Array(members.into_iter().map(toml::Value::String).collect())
        ),
    )
    .unwrap();
    fs::write(source.join("Cargo.lock"), "version = 4\n").unwrap();
    fs::write(source.join("LICENSE-MIT"), "Synthetic fixture license\n").unwrap();
    fs::write(source.join("LICENSE-APACHE"), "Synthetic fixture license\n").unwrap();
    fs::write(
        source.join("rust-toolchain.toml"),
        "[toolchain]\nchannel=\"1.99.0\"\n",
    )
    .unwrap();
    for name in Cohort::Sixteen.names() {
        let dir = source.join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        let isolated = [
            "yetlet-config",
            "yetlet-secrets",
            "yetlet-signing",
            "yetlet-session-redis",
            "yetlet-wgpu",
        ]
        .contains(name);
        let mut manifest = format!(
            "[package]\nname=\"{name}\"\nversion=\"{version}\"\nedition=\"2024\"\nlicense=\"MIT\"\ndescription=\"Synthetic rebuild fixture\"\n"
        );
        if isolated {
            manifest.push_str("\n[workspace]\n");
            fs::write(dir.join("Cargo.lock"), "version = 4\n").unwrap();
        }
        let dep = |n: &str| format!("{n} = {{ version=\"{version}\", path=\"../{n}\" }}\n");
        match *name {
            "yetlet-html" | "yetlet-dom" | "yetlet-dom-keyed" => manifest.push_str(&format!("\n[dependencies]\n{}", dep("yetlet-core"))),
            "yetlet-server-axum" => manifest.push_str(&format!("\n[dependencies]\n{}{}", dep("yetlet-core"), dep("yetlet-html"))),
            "yetlet-request-id" | "yetlet-session" | "yetlet-session-redis" => manifest.push_str(&format!("\n[dependencies]\n{}", dep("yetlet-server-axum"))),
            "yetlet-auth" => manifest.push_str(&format!("\n[features]\ndefault=[]\nsession=[\"dep:yetlet-session\"]\n[dependencies]\nyetlet-session={{version=\"{version}\",path=\"../yetlet-session\",optional=true}}\n")),
            "yetlet-rate-limit" => manifest.push_str(&format!("\n[dev-dependencies]\n{}", dep("yetlet-config"))),
            "yetlet-config" => manifest.push_str(&format!("\n[features]\ndefault=[]\nsecret-provider=[\"dep:yetlet-secrets\"]\n[dependencies]\nyetlet-secrets={{version=\"{version}\",path=\"../yetlet-secrets\",optional=true}}\n")),
            _ => {}
        }
        fs::write(dir.join("Cargo.toml"), manifest).unwrap();
        fs::write(dir.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        fs::write(dir.join("LICENSE-MIT"), "Synthetic fixture license\n").unwrap();
        fs::write(dir.join("LICENSE-APACHE"), "Synthetic fixture license\n").unwrap();
    }
}

#[test]
fn selected_sixteen_inputs_bind_isolated_locks_without_widening_legacy_aliases() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let original = private::selected(temp.path(), Cohort::OriginalNine).unwrap();
    let expanded = private::selected(temp.path(), Cohort::Sixteen).unwrap();
    assert!(original.keys().all(|k| expanded.contains_key(k)));
    for m in &Cohort::Sixteen.manifests()[1..] {
        assert!(
            expanded.contains_key(
                &Path::new(m)
                    .with_file_name("Cargo.lock")
                    .to_string_lossy()
                    .replace('\\', "/")
            )
        );
    }
    let key = "crates/yetlet-session-redis/Cargo.lock";
    fs::write(temp.path().join(key), "changed isolated lock\n").unwrap();
    assert_ne!(
        expanded[key],
        private::selected(temp.path(), Cohort::Sixteen).unwrap()[key]
    );
    assert_eq!(private::aliases().len(), 18);
    assert_eq!(Cohort::OriginalNine.names(), private::NAMES);
    assert_eq!(
        Cohort::Sixteen
            .names()
            .iter()
            .collect::<BTreeSet<_>>()
            .len(),
        16
    );
    assert!(!original.contains_key(key));
}

#[test]
fn cargo_steps_verify_once_and_stage_final_dependency_checksums() {
    let legacy = private::cargo_steps(Path::new("source"), Path::new("work"), Cohort::OriginalNine);
    assert_eq!(legacy.len(), 2);
    assert_eq!(legacy[0].0, "fetch");
    assert_eq!(legacy[1].0, "package");
    assert_eq!(legacy[1].1.iter().filter(|a| *a == "-p").count(), 9);
    let steps = private::cargo_steps(Path::new("source"), Path::new("work"), Cohort::Sixteen);
    assert_eq!(steps.len(), 17);
    assert_eq!(steps[0].1.iter().filter(|a| *a == "--sync").count(), 5);
    for (label, args) in &steps[1..] {
        assert!(args.iter().any(|a| a == "--offline"));
        assert!(args.iter().any(|a| a == "--locked"));
        assert!(!args.iter().any(|a| a == "--all-features"));
        assert_eq!(
            args.iter().any(|a| a == "--no-verify"),
            label.starts_with("bootstrap-")
        );
        assert_eq!(
            args.iter().any(|a| a == "--exclude-lockfile"),
            label.starts_with("bootstrap-")
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn real_cargo_sixteen_alpha_packages_and_normal_verification_refusal() {
    use std::{process::Command, sync::atomic::AtomicBool, time::Duration};
    use yetlet_xtask::process::{self, Spec};
    let version = package_version();
    let temp = tempfile::Builder::new()
        .prefix("yetlet rebuild fixture ")
        .tempdir()
        .unwrap();
    let source = temp.path().join("source");
    fixture(&source);
    let work = temp.path().join("work");
    fs::create_dir(&work).unwrap();
    let out = temp.path().join("records");
    fs::create_dir(&out).unwrap();
    let cargo = Path::new(env!("CARGO"));
    let home = work.join("cargo-home");
    fs::create_dir(&home).unwrap();
    let environment = vec![
        ("CARGO_HOME".into(), Some(home.into_os_string())),
        (
            "CARGO_TARGET_DIR".into(),
            Some(work.join("cargo-target").into_os_string()),
        ),
        ("CARGO_NET_OFFLINE".into(), Some("true".into())),
        ("RUSTUP_AUTO_INSTALL".into(), Some("0".into())),
        ("SOURCE_DATE_EPOCH".into(), Some("1700000000".into())),
    ];
    for manifest in Cohort::Sixteen.manifests() {
        let r = Command::new(cargo)
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(source.join(manifest))
            .envs(environment.iter().map(|(k, v)| (k, v.as_ref().unwrap())))
            .output()
            .unwrap();
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let inputs = private::selected(&source, Cohort::Sixteen).unwrap();
    let mut record = json!({});
    let built = private::build_packages(
        &source,
        &work,
        &out,
        Cohort::Sixteen,
        cargo,
        &environment,
        &mut record,
    );
    if built.is_err() {
        for entry in fs::read_dir(&out).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().is_some_and(|s| s == "log") {
                eprintln!("{}:\n{}", p.display(), fs::read_to_string(&p).unwrap());
            }
        }
    }
    built.unwrap();
    assert_eq!(inputs, private::selected(&source, Cohort::Sixteen).unwrap());
    let expected: BTreeSet<_> = Cohort::Sixteen
        .names()
        .iter()
        .map(|n| format!("{n}-{version}.crate"))
        .collect();
    let actual: BTreeSet<_> = fs::read_dir(work.join("cargo-target/package"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "crate"))
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(expected, actual);
    for name in Cohort::Sixteen.names() {
        let staged = work.join("vendor").join(format!("{name}-{version}"));
        let manifest: toml::Value =
            toml::from_str(&fs::read_to_string(staged.join("Cargo.toml")).unwrap()).unwrap();
        assert_eq!(
            manifest["package"]["version"].as_str(),
            Some(version.as_str())
        );
        assert!(staged.join(".cargo-checksum.json").is_file());
        assert_eq!(record[format!("package-{name}_exit_code")], 0);
        assert_eq!(
            record["verified_staging"][*name]["verification"],
            json!(true)
        );
    }
    // This new negative input must fail normal verification; do not repeat the full pipeline.
    fs::write(
        source.join("crates/yetlet-signing/src/lib.rs"),
        "compile_error!(\"expected fixture verification refusal\");\n",
    )
    .unwrap();
    let (_, args) = private::cargo_steps(&source, &work, Cohort::Sixteen)
        .into_iter()
        .find(|(label, _)| label == "package-yetlet-signing")
        .unwrap();
    let r = process::run(
        &Spec {
            program: cargo.as_os_str().into(),
            args,
            cwd: source,
            env: environment,
            timeout: Duration::from_secs(60),
            output_limit: 1024 * 1024,
        },
        &AtomicBool::new(false),
    );
    assert!(!r.success());
    assert!(
        String::from_utf8_lossy(&r.stderr.bytes).contains("expected fixture verification refusal")
    );
    eprintln!(
        "Expected negative Cargo verification exit {}:\n{}",
        r.exit_code(),
        String::from_utf8_lossy(&r.stderr.bytes)
    );
}

/// Uses the actual two SDK sources; explicitly invoked when the offline cache is complete.
#[test]
#[ignore = "requires the complete existing offline dependency cache"]
fn real_current_consolidated_archives_use_final_predecessor() {
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for e in fs::read_dir(from).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            if e.file_name() == "target" || e.file_name() == "Cargo.lock" {
                continue;
            }
            assert!(!e.file_type().unwrap().is_symlink());
            if p.is_dir() {
                copy(&p, &to.join(e.file_name()));
            } else {
                fs::copy(p, to.join(e.file_name())).unwrap();
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let retained = std::env::var_os("YETLET_PACKAGE_PROOF_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/yt211-single-package-20261030/packaging"));
    fs::create_dir(&retained).unwrap();
    let source = retained.join("source");
    fs::create_dir(&source).unwrap();
    let mut manifest: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("Cargo.toml")).unwrap()).unwrap();
    manifest["workspace"]["members"] = json!(["crates/yetlet", "crates/yetlet-browser"])
        .as_array()
        .unwrap()
        .iter()
        .map(|v| toml::Value::String(v.as_str().unwrap().into()))
        .collect::<Vec<_>>()
        .into();
    manifest["workspace"]
        .as_table_mut()
        .unwrap()
        .remove("exclude");
    manifest.as_table_mut().unwrap().remove("patch");
    fs::write(
        source.join("Cargo.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    fs::copy(root.join("Cargo.lock"), source.join("Cargo.lock")).unwrap();
    for n in Cohort::Consolidated.names() {
        copy(&root.join("crates").join(n), &source.join("crates").join(n));
    }
    let prep = std::process::Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(source.join("Cargo.toml"))
        .output()
        .unwrap();
    fs::write(retained.join("metadata.stderr"), &prep.stderr).unwrap();
    assert!(prep.status.success());
    let work = retained.join("work");
    fs::create_dir(&work).unwrap();
    let out = retained.join("records");
    fs::create_dir(&out).unwrap();
    let environment = vec![
        ("CARGO_NET_OFFLINE".into(), Some("true".into())),
        ("RUSTUP_AUTO_INSTALL".into(), Some("0".into())),
    ];
    let mut record = json!({});
    let result = private::build_packages(
        &source,
        &work,
        &out,
        Cohort::Consolidated,
        Path::new(env!("CARGO")),
        &environment,
        &mut record,
    );
    fs::write(
        retained.join("RESULT.json"),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    result.unwrap();
    assert_eq!(
        record["publication_order"],
        json!(["yetlet", "yetlet-browser"])
    );
    for name in Cohort::Consolidated.names() {
        assert_eq!(record["verified_staging"][*name]["verification"], true);
        assert_eq!(record[format!("package-{name}_exit_code")], 0);
    }
}
