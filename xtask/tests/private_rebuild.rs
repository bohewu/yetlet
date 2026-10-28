use serde_json::json;
use std::{collections::BTreeSet, ffi::OsString, fs, path::Path};
use yetlet_xtask::{
    Cli, parse,
    release::private::{self, Cohort},
};

#[test]
fn closed_cohort_cli_preserves_legacy_default_and_rejects_extra_inputs() {
    for args in [
        vec!["release", "private-rebuild"],
        vec!["release", "private-rebuild", "--cohort", "original-nine"],
    ] {
        assert_eq!(
            parse(args.into_iter().map(Into::into)).unwrap(),
            Cli::PrivateRebuild {
                cohort: Cohort::OriginalNine
            }
        );
    }
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

fn fixture(source: &Path) {
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
            "[package]\nname=\"{name}\"\nversion=\"0.9.0-alpha.1\"\nedition=\"2024\"\nlicense=\"MIT\"\ndescription=\"Synthetic rebuild fixture\"\n"
        );
        if isolated {
            manifest.push_str("\n[workspace]\n");
            fs::write(dir.join("Cargo.lock"), "version = 4\n").unwrap();
        }
        let dep = |n: &str| format!("{n} = {{ version=\"0.9.0-alpha.1\", path=\"../{n}\" }}\n");
        match *name {
            "yetlet-html" | "yetlet-dom" | "yetlet-dom-keyed" => manifest.push_str(&format!("\n[dependencies]\n{}", dep("yetlet-core"))),
            "yetlet-server-axum" => manifest.push_str(&format!("\n[dependencies]\n{}{}", dep("yetlet-core"), dep("yetlet-html"))),
            "yetlet-request-id" | "yetlet-session" | "yetlet-session-redis" => manifest.push_str(&format!("\n[dependencies]\n{}", dep("yetlet-server-axum"))),
            "yetlet-auth" => manifest.push_str("\n[features]\ndefault=[]\nsession=[\"dep:yetlet-session\"]\n[dependencies]\nyetlet-session={version=\"0.9.0-alpha.1\",path=\"../yetlet-session\",optional=true}\n"),
            "yetlet-rate-limit" | "yetlet-secrets" => manifest.push_str(&format!("\n[dev-dependencies]\n{}", dep("yetlet-config"))),
            "yetlet-config" => manifest.push_str("\n[features]\ndefault=[]\nsecret-provider=[\"dep:yetlet-secrets\"]\n[dependencies]\nyetlet-secrets={version=\"0.9.0-alpha.1\",path=\"../yetlet-secrets\",optional=true}\n"),
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
fn cargo_steps_keep_collection_separate_from_locked_default_verification() {
    let legacy = private::cargo_steps(Path::new("source"), Path::new("work"), Cohort::OriginalNine);
    assert_eq!(legacy.len(), 2);
    assert_eq!(legacy[0].0, "fetch");
    assert_eq!(legacy[1].0, "package");
    assert_eq!(legacy[1].1.iter().filter(|a| *a == "-p").count(), 9);
    let steps = private::cargo_steps(Path::new("source"), Path::new("work"), Cohort::Sixteen);
    assert_eq!(steps.len(), 33);
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
        .map(|n| format!("{n}-0.9.0-alpha.1.crate"))
        .collect();
    let actual: BTreeSet<_> = fs::read_dir(work.join("cargo-target/package"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "crate"))
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(expected, actual);
    for name in Cohort::Sixteen.names() {
        let staged = work.join("vendor").join(format!("{name}-0.9.0-alpha.1"));
        let manifest: toml::Value =
            toml::from_str(&fs::read_to_string(staged.join("Cargo.toml")).unwrap()).unwrap();
        assert_eq!(
            manifest["package"]["version"].as_str(),
            Some("0.9.0-alpha.1")
        );
        assert!(staged.join(".cargo-checksum.json").is_file());
        assert_eq!(record[format!("package-{name}_exit_code")], 0);
        assert_eq!(
            record["bootstrap_staging"][*name],
            json!({"ok":true,"verification":false})
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
