#![cfg(any(target_os = "linux", windows))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use yetlet::signing::{
    KeyRef, KeyState, LocalProvider, LocalSigner, Policy, PublicKeySet, PublicMaterial,
    SigningService, TrustedKey, generate_self_signed,
};
const PASSWORD: &str = "synthetic-key-test-password";
fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
}
fn directory(path: &Path) {
    fs::create_dir(path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        }
        // Native creator installs the user's protected ACL on Windows as well.
        yetlet_cli::scaffold::create_selected(
            "key_fixture",
            temp.path(),
            yetlet_cli::scaffold::Template::Native,
            false,
            false,
            false,
            false,
        )
        .unwrap();
        let root = temp.path().join("key_fixture");
        write(
            &root.join("Cargo.lock"),
            "version=4\n[[package]]\nname='key_fixture'\nversion='0.1.0'\n",
        );
        for d in [
            "keys",
            "keys/local_dev",
            "keys/local_dev/documents",
            "keys/local_dev/invoices",
        ] {
            directory(&root.join(d));
        }
        write(
            &root.join("base.toml"),
            "# base remains byte-identical\n[unrelated]\nname='unchanged'\n[signing]\ndefault_provider='local_dev'\ndefault_key='documents'\n[signing.providers.local_dev]\nkind='local'\nroot='keys/local_dev'\nmode='self_signed_test'\npassword_env='YETLET_SIGNING_TEST_PASSWORD'\n[signing.providers.local_dev.keys.documents]\nactive_version=''\nversions=[]\n[signing.providers.local_dev.keys.invoices]\nactive_version=''\nversions=[]\n",
        );
        for p in ["production.toml", "development.toml", "staging.toml"] {
            write(
                &root.join(p),
                "# preserve this profile comment\n[unrelated]\nname='profile' # keep this inline comment\n",
            );
        }
        Self { _temp: temp, root }
    }
    fn cmd(&self, action: &str) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_yetlet"));
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().to_ascii_uppercase().starts_with("APP_")
                || k == "YETLET_CONFIG_ROOT"
            {
                c.env_remove(k);
            }
        }
        c.args(["key", action, "--manifest-path"])
            .arg(self.root.join("Cargo.toml"))
            .env("YETLET_SIGNING_TEST_PASSWORD", PASSWORD);
        c
    }
    fn rotate(&self, key: &str) -> Output {
        self.cmd("rotate").args([key, "--yes"]).output().unwrap()
    }
    fn selected(&self, profile: &str) -> serde_json::Value {
        yetlet::config::Loader::new(&self.root, "APP")
            .load_with_env([("APP_ENVIRONMENT".into(), profile.into())])
            .unwrap()
    }
}
fn success(out: Output) -> Output {
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}
fn failure(out: Output) -> Output {
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains(PASSWORD));
    out
}
fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
    let mut f = std::pin::pin!(future);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match f.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => v,
        _ => panic!("local provider unexpectedly asynchronous"),
    }
}
#[test]
fn no_tty_no_secret_and_declared_status_have_no_mutations() {
    let f = Fixture::new();
    let before = fs::read(f.root.join("production.toml")).unwrap();
    let out = success(
        f.cmd("status")
            .env_remove("YETLET_SIGNING_TEST_PASSWORD")
            .output()
            .unwrap(),
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("Active version: (none)"));
    let out = failure(f.cmd("rotate").output().unwrap());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--yes"));
    failure(
        f.cmd("rotate")
            .arg("--yes")
            .env_remove("YETLET_SIGNING_TEST_PASSWORD")
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(f.root.join("production.toml")).unwrap(), before);
    assert!(!f.root.join(".yetlet-key.lock").exists());
    assert_eq!(
        fs::read_dir(f.root.join("keys/local_dev/documents"))
            .unwrap()
            .count(),
        0
    );
}
#[test]
fn rotation_isolated_keys_profiles_comments_and_old_verification() {
    let f = Fixture::new();
    let base = fs::read(f.root.join("base.toml")).unwrap();
    let development = fs::read(f.root.join("development.toml")).unwrap();
    success(f.rotate("documents"));
    let old_private = fs::read(f.root.join("keys/local_dev/documents/v1/private.pem")).unwrap();
    let old_cert = fs::read(f.root.join("keys/local_dev/documents/v1/certificate.pem")).unwrap();
    let r = KeyRef {
        provider: "local_dev".into(),
        key: "documents".into(),
        version: "v1".into(),
    };
    let public = PublicMaterial::from_certificate_pem(&old_cert).unwrap();
    let signer = LocalSigner::from_pem(r.clone(), &old_private, PASSWORD, &old_cert).unwrap();
    let mut service = SigningService::new(
        LocalProvider::new(vec![signer]).unwrap(),
        PublicKeySet::new(vec![(r.clone(), public.clone())]).unwrap(),
        Policy::new(vec![
            TrustedKey::certificate(
                r.clone(),
                &old_cert,
                public.certificate_pin().unwrap(),
                KeyState::Active,
            )
            .unwrap(),
        ])
        .unwrap(),
    );
    let envelope = block_on(service.sign(&r, b"unaltered bytes")).unwrap();
    success(f.rotate("documents"));
    let settings = f.selected("Production");
    let keys = &settings["signing"]["providers"]["local_dev"]["keys"];
    assert_eq!(keys["documents"]["active_version"], "v2");
    assert_eq!(keys["invoices"]["active_version"], "");
    assert_eq!(keys["documents"]["versions"][0]["state"], "retired");
    assert_eq!(
        fs::read(f.root.join("keys/local_dev/documents/v1/private.pem")).unwrap(),
        old_private
    );
    assert_eq!(
        fs::read(f.root.join("keys/local_dev/documents/v1/certificate.pem")).unwrap(),
        old_cert
    );
    service
        .policy_mut()
        .set_state(&r, KeyState::Retired)
        .unwrap();
    block_on(service.verify(b"unaltered bytes", &envelope)).unwrap();
    assert!(block_on(service.verify(b"tampered bytes", &envelope)).is_err());
    success(
        f.cmd("rotate")
            .args(["invoices", "--yes"])
            .env("APP_ENVIRONMENT", "Development")
            .output()
            .unwrap(),
    );
    assert_eq!(
        f.selected("Development")["signing"]["providers"]["local_dev"]["keys"]["invoices"]["active_version"],
        "v1"
    );
    assert_eq!(
        f.selected("Production")["signing"]["providers"]["local_dev"]["keys"]["invoices"]["active_version"],
        ""
    );
    assert_ne!(
        fs::read(f.root.join("development.toml")).unwrap(),
        development
    );
    assert_eq!(fs::read(f.root.join("base.toml")).unwrap(), base);
    let text = fs::read_to_string(f.root.join("production.toml")).unwrap();
    assert!(text.contains("# preserve this profile comment"));
    assert!(text.contains("# keep this inline comment"));
    let out = success(
        f.cmd("status")
            .env_remove("YETLET_SIGNING_TEST_PASSWORD")
            .output()
            .unwrap(),
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Certificate not after:"));
    assert!(text.contains("Policy expiry: none"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(f.root.join("keys/local_dev/documents/v2/private.pem"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
#[test]
fn effective_env_override_is_readable_but_rotation_refuses_before_effects() {
    let f = Fixture::new();
    let name = "APP_SIGNING__DEFAULT_KEY";
    let out = success(f.cmd("status").env(name, "invoices").output().unwrap());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Key: invoices"));
    failure(
        f.cmd("rotate")
            .arg("--yes")
            .env(name, "invoices")
            .output()
            .unwrap(),
    );
    assert!(!f.root.join("keys/local_dev/invoices/v1").exists());
    assert!(!f.root.join(".yetlet-key.lock").exists());
}
#[test]
fn import_requires_replacement_rejects_duplicate_and_wrong_password() {
    let f = Fixture::new();
    success(f.rotate("documents"));
    let material = generate_self_signed("import", PASSWORD, 30).unwrap();
    write(&f.root.join("replacement.pfx"), &material.pkcs12_der);
    let mut base = fs::read_to_string(f.root.join("base.toml")).unwrap();
    base = base.replace("mode='self_signed_test'", "mode='imported'");
    write(&f.root.join("base.toml"), base);
    failure(f.rotate("documents"));
    let before = fs::read(f.root.join("production.toml")).unwrap();
    failure(
        f.cmd("rotate")
            .args(["--yes", "--from"])
            .arg(f.root.join("replacement.pfx"))
            .env("YETLET_SIGNING_TEST_PASSWORD", "wrong")
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(f.root.join("production.toml")).unwrap(), before);
    success(
        f.cmd("rotate")
            .args(["--yes", "--from"])
            .arg(f.root.join("replacement.pfx"))
            .output()
            .unwrap(),
    );
    let before = fs::read(f.root.join("production.toml")).unwrap();
    failure(
        f.cmd("rotate")
            .args(["invoices", "--yes", "--from"])
            .arg(f.root.join("replacement.pfx"))
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(f.root.join("production.toml")).unwrap(), before);
    assert!(!f.root.join("keys/local_dev/invoices/v1").exists());
}
#[test]
fn alternate_settings_root_is_explicit_and_does_not_change_project_path_base() {
    let f = Fixture::new();
    let cfg = f.root.join("settings");
    directory(&cfg);
    for name in ["base.toml", "production.toml"] {
        write(&cfg.join(name), fs::read(f.root.join(name)).unwrap());
    }
    success(
        f.cmd("rotate")
            .arg("--yes")
            .env("YETLET_CONFIG_ROOT", &cfg)
            .output()
            .unwrap(),
    );
    assert!(
        f.root
            .join("keys/local_dev/documents/v1/private.pem")
            .exists()
    );
    assert!(
        fs::read_to_string(cfg.join("production.toml"))
            .unwrap()
            .contains("active_version = \"v1\"")
    );
    assert!(
        !fs::read_to_string(f.root.join("production.toml"))
            .unwrap()
            .contains("active_version")
    );
}
#[test]
fn key_publication_failure_preserves_active_settings() {
    let f = Fixture::new();
    success(f.rotate("documents"));
    let before = fs::read(f.root.join("production.toml")).unwrap();
    directory(&f.root.join("keys/local_dev/documents/v2"));
    failure(f.rotate("documents"));
    assert_eq!(fs::read(f.root.join("production.toml")).unwrap(), before);
    assert!(
        f.root
            .join("keys/local_dev/documents/v1/certificate.pem")
            .exists()
    );
}
#[cfg(unix)]
#[test]
fn unsafe_permissions_symlinks_and_readonly_profile_fail_before_mutation() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    let profile = f.root.join("production.toml");
    let before = fs::read(&profile).unwrap();
    fs::set_permissions(&profile, fs::Permissions::from_mode(0o400)).unwrap();
    failure(f.rotate("documents"));
    fs::set_permissions(&profile, fs::Permissions::from_mode(0o600)).unwrap();
    let mat = generate_self_signed("pfx", PASSWORD, 30).unwrap();
    write(&f.root.join("import.pfx"), mat.pkcs12_der);
    fs::set_permissions(f.root.join("import.pfx"), fs::Permissions::from_mode(0o644)).unwrap();
    failure(
        f.cmd("rotate")
            .args(["--yes", "--from"])
            .arg(f.root.join("import.pfx"))
            .output()
            .unwrap(),
    );
    fs::set_permissions(f.root.join("import.pfx"), fs::Permissions::from_mode(0o600)).unwrap();
    symlink("import.pfx", f.root.join("linked.pfx")).unwrap();
    failure(
        f.cmd("rotate")
            .args(["--yes", "--from"])
            .arg(f.root.join("linked.pfx"))
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(profile).unwrap(), before);
    assert!(!f.root.join("keys/local_dev/documents/v1").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn public_key_parent_supports_checkout_permissions_without_exposing_private_versions() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let parent = fixture.root.join("keys/local_dev/documents");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o755)).unwrap();
    success(fixture.rotate("documents"));
    assert_eq!(
        fs::metadata(&parent).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(
        fs::metadata(parent.join("v1"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(parent.join("v1/private.pem"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let before = fs::read(fixture.root.join("production.toml")).unwrap();
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o775)).unwrap();
    failure(fixture.rotate("documents"));
    assert_eq!(
        fs::read(fixture.root.join("production.toml")).unwrap(),
        before
    );
    assert!(!parent.join("v2").exists());
}

#[test]
fn rotation_preserves_selected_field_array_and_historical_record_comments() {
    for tables in [false, true] {
        for inherited in [false, true] {
            let fixture = Fixture::new();
            success(fixture.rotate("documents"));
            success(fixture.rotate("documents"));
            let profile_path = fixture.root.join("production.toml");
            let profile = fs::read_to_string(&profile_path)
                .unwrap()
                .parse::<toml_edit::DocumentMut>()
                .unwrap();
            let records =
                profile["signing"]["providers"]["local_dev"]["keys"]["documents"]["versions"]
                    .as_array()
                    .unwrap();
            let mut annotated = String::from(
                "[signing.providers.local_dev.keys.documents]\nactive_version = 'v2' # approved signer\n",
            );
            if tables {
                annotated.push_str("# retention note\n");
                for (index, record) in records.iter().enumerate() {
                    annotated.push_str(&format!(
                        "[[signing.providers.local_dev.keys.documents.versions]] # record {index}\n"
                    ));
                    for (field, value) in record.as_inline_table().unwrap().iter() {
                        annotated.push_str(&format!(
                            "{field} = {value} # record {index} {field} annotation\n"
                        ));
                    }
                }
            } else {
                annotated.push_str("versions = [ # retention note\n");
                for (index, record) in records.iter().enumerate() {
                    annotated.push_str(&format!(
                        "  # record {index} annotation\n  {record}, # record {index} retained\n"
                    ));
                }
                annotated.push_str("] # complete version history\n");
            }
            if inherited {
                let mut base = fs::read_to_string(fixture.root.join("base.toml"))
                    .unwrap()
                    .parse::<toml_edit::DocumentMut>()
                    .unwrap();
                let annotated_document = annotated.parse::<toml_edit::DocumentMut>().unwrap();
                base["signing"]["providers"]["local_dev"]["keys"]["documents"] = annotated_document
                    ["signing"]["providers"]["local_dev"]["keys"]["documents"]
                    .clone();
                write(&fixture.root.join("base.toml"), base.to_string());
                write(
                    &profile_path,
                    "# selected profile comment\n[signing.providers.local_dev.keys.documents]\nactive_version = 'v2' # approved signer\n",
                );
            } else {
                write(&profile_path, &annotated);
            }
            let base_before = fs::read(fixture.root.join("base.toml")).unwrap();
            success(fixture.rotate("documents"));
            let after = fs::read_to_string(&profile_path).unwrap();
            assert_eq!(
                fs::read(fixture.root.join("base.toml")).unwrap(),
                base_before
            );
            assert!(after.contains("# approved signer"), "{after}");
            assert!(after.contains("# retention note"), "{after}");
            if tables {
                for index in 0..2 {
                    for field in [
                        "version",
                        "certificate",
                        "private_key",
                        "certificate_pin",
                        "state",
                    ] {
                        assert!(
                            after.contains(&format!("# record {index} {field} annotation")),
                            "{after}"
                        );
                    }
                    assert!(after.contains(&format!("# record {index}\n")), "{after}");
                }
            } else {
                for index in 0..2 {
                    assert!(
                        after.contains(&format!("# record {index} annotation")),
                        "{after}"
                    );
                    assert!(
                        after.contains(&format!("# record {index} retained")),
                        "{after}"
                    );
                }
                assert!(after.contains("# complete version history"), "{after}");
                assert!(
                    after.find("# record 1 retained").unwrap()
                        < after.find("documents/v3/certificate.pem").unwrap(),
                    "{after}"
                );
            }
            assert!(
                toml::from_str::<toml::Value>(&after).is_ok(),
                "tables={tables} inherited={inherited}\n{after}"
            );
            let selected = fixture.selected("Production");
            let key = &selected["signing"]["providers"]["local_dev"]["keys"]["documents"];
            assert_eq!(key["active_version"], "v3");
            assert_eq!(key["versions"].as_array().unwrap().len(), 3);
            assert_eq!(key["versions"][0]["state"], "retired");
            assert_eq!(key["versions"][1]["state"], "retired");
            assert_eq!(key["versions"][2]["state"], "active");
        }
    }
}
