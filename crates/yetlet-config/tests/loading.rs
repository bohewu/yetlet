use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use yetlet_config::{Error, Loader, MAX_FILE_BYTES, SecretFormat};

const BASE: &str =
    "[server]\nport = 3000\nenabled = false\n[service]\ntoken = ''\nname = 'base'\nratio = 1.0\n";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    server: Server,
    service: Service,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Server {
    port: u16,
    enabled: bool,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Service {
    token: SecretString,
    name: String,
    ratio: f64,
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "yt090-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let result = Self(path);
        result.write("base.toml", BASE);
        for (profile, port) in [
            ("development", 3100),
            ("staging", 3200),
            ("production", 3300),
        ] {
            result.write(
                &format!("{profile}.toml"),
                &format!("[server]\nport = {port}\n[service]\nname = '{profile}'\n"),
            );
        }
        result
    }
    fn write(&self, name: &str, text: &str) {
        fs::write(self.0.join(name), text).unwrap();
    }
    fn loader(&self) -> Loader {
        Loader::new(&self.0, "APP")
    }
    fn secret(&self, format: SecretFormat) -> Loader {
        self.loader()
            .development_secrets(self.0.join("private file"), format)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn env(entries: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    entries.iter().map(|(k, v)| (k.into(), v.into())).collect()
}
fn load(loader: Loader, entries: &[(&str, &str)]) -> Result<Settings, Error> {
    loader.load_from_env(env(entries))
}
fn dev() -> [(&'static str, &'static str); 1] {
    [("APP_ENVIRONMENT", "Development")]
}

#[test]
fn profiles_and_safe_default() {
    let f = Fixture::new();
    for (profile, port) in [
        ("Development", 3100),
        ("Staging", 3200),
        ("Production", 3300),
    ] {
        let s = load(f.loader(), &[("APP_ENVIRONMENT", profile)]).unwrap();
        assert_eq!(s.server.port, port);
        assert_eq!(s.service.name, profile.to_ascii_lowercase());
    }
    assert_eq!(load(f.loader(), &[]).unwrap().server.port, 3300);
}

#[test]
fn complete_precedence_nested_types_and_exact_string_secrets() {
    let f = Fixture::new();
    f.write(
        "private file",
        "APP_SERVER__PORT=3400\nAPP_SERVICE__TOKEN='0000123'\nAPP_SERVICE__NAME=local\n",
    );
    let s = load(f.secret(SecretFormat::Dotenv), &dev()).unwrap();
    assert_eq!(s.server.port, 3400);
    assert_eq!(s.service.name, "local");
    assert_eq!(s.service.token.expose_secret(), "0000123");
    let s = load(
        f.secret(SecretFormat::Dotenv),
        &[
            ("APP_ENVIRONMENT", "Development"),
            ("APP_SERVER__PORT", "3500"),
            ("APP_SERVER__ENABLED", "true"),
            ("APP_SERVICE__TOKEN", "false"),
            ("APP_SERVICE__RATIO", "2.5"),
        ],
    )
    .unwrap();
    assert_eq!(s.server.port, 3500);
    assert!(s.server.enabled);
    assert_eq!(s.service.token.expose_secret(), "false");
    assert_eq!(s.service.ratio, 2.5);
}

#[test]
fn development_files_are_explicit_and_other_profiles_never_read_them() {
    let f = Fixture::new();
    // The path does not exist: successful loads prove it was not opened.
    for profile in ["Staging", "Production"] {
        assert!(
            load(
                f.secret(SecretFormat::Dotenv),
                &[("APP_ENVIRONMENT", profile)]
            )
            .is_ok()
        );
    }
    assert!(load(f.secret(SecretFormat::Dotenv), &[]).is_ok());
    assert_eq!(
        load(f.secret(SecretFormat::Dotenv), &dev()).unwrap_err(),
        Error::File
    );
    assert!(load(f.loader(), &dev()).is_ok());
    f.write(".env.local", "malformed and never implicitly read");
    assert!(load(f.loader(), &dev()).is_ok());
}

#[test]
fn selector_is_finite_and_checked_before_files() {
    let loader = || Loader::new("/not/an/application/path", "APP");
    for value in [
        "development",
        "",
        "../private",
        "Staging/../../",
        "C:\\private",
        "Production ",
    ] {
        assert_eq!(
            load(loader(), &[("APP_ENVIRONMENT", value)]).unwrap_err(),
            Error::Environment
        );
    }
}

#[test]
fn invalid_related_keys_and_collisions_fail() {
    let f = Fixture::new();
    for key in [
        "APP_UNKNOWN",
        "APP_SERVER__UNKNOWN",
        "APP_SERVER_PORT",
        "APP_SERVER___PORT",
        "app_SERVER__PORT",
        "APP_server__port",
        "APP_SERVER__",
        "APP__SERVER",
    ] {
        assert_eq!(
            load(f.loader(), &[(key, "123")]).unwrap_err(),
            Error::EnvironmentKey,
            "{key}"
        );
    }
    assert_eq!(
        load(
            f.loader(),
            &[("APP_SERVICE__NAME", "a"), ("APP_SERVICE__NAME", "b")]
        )
        .unwrap_err(),
        Error::EnvironmentKey
    );
    assert_eq!(
        load(
            f.loader(),
            &[
                ("APP_ENVIRONMENT", "Development"),
                ("APP_ENVIRONMENT", "Production")
            ]
        )
        .unwrap_err(),
        Error::EnvironmentKey
    );
    f.write("private file", "APP_SERVICE__NAME=a\nAPP_SERVICE__NAME=b\n");
    assert_eq!(
        load(f.secret(SecretFormat::Dotenv), &dev()).unwrap_err(),
        Error::EnvironmentKey
    );
    assert!(load(f.loader(), &[("OTHER_UNKNOWN", "ignored")]).is_ok());
}

#[test]
fn typed_mismatches_fail_without_coercing_toml_or_overrides() {
    let f = Fixture::new();
    for (key, value) in [
        ("APP_SERVER__PORT", "not-a-port"),
        ("APP_SERVER__PORT", "65536"),
        ("APP_SERVER__ENABLED", "1"),
        ("APP_SERVICE__RATIO", "NaN"),
    ] {
        assert!(load(f.loader(), &[(key, value)]).is_err());
    }
    f.write("production.toml", "[service]\ntoken = 123\n");
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::Type);
    f.write("production.toml", "[server]\nport = '3300'\n");
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::Type);
    f.write("production.toml", "[service]\nunknown = 'value'\n");
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::EnvironmentKey);
}

#[test]
fn app_owned_settings_missing_fields_and_unknown_base_fields_fail() {
    let f = Fixture::new();
    f.write(
        "base.toml",
        "[server]\nport = 3000\n[service]\nname = 'base'\n",
    );
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::Settings);
    f.write(
        "base.toml",
        &format!("unknown = 'synthetic-private'\n{BASE}"),
    );
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::Settings);
}

#[test]
fn malformed_and_bounded_files_and_nonregular_paths_fail() {
    let f = Fixture::new();
    f.write("production.toml", "[bad\nsecret = 'synthetic-private'\n");
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::Toml);
    f.write("production.toml", &"a".repeat(MAX_FILE_BYTES + 1));
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::File);
    fs::remove_file(f.0.join("production.toml")).unwrap();
    fs::create_dir(f.0.join("production.toml")).unwrap();
    assert_eq!(load(f.loader(), &[]).unwrap_err(), Error::File);
    assert_eq!(
        load(f.loader().defaults("a".repeat(MAX_FILE_BYTES + 1)), &[]).unwrap_err(),
        Error::Input
    );
}

#[test]
fn private_toml_supports_literal_dollars_and_paths_with_spaces() {
    let f = Fixture::new();
    f.write("private file", "[service]\ntoken = '000$literal'\n");
    let s = load(f.secret(SecretFormat::Toml), &dev()).unwrap();
    assert_eq!(s.service.token.expose_secret(), "000$literal");
    // PathBuf joining has no string '/' concatenation requirement.
    assert!(f.0.join("private file").is_absolute());
}

#[test]
fn dotenv_never_expands_or_runs_shell_and_rejects_malformed_input() {
    let f = Fixture::new();
    for text in [
        "APP_SERVICE__TOKEN=$HOME",
        "APP_SERVICE__TOKEN='${HOME}'",
        "APP_SERVICE__TOKEN=$(whoami)",
        "APP_SERVICE__TOKEN='unclosed",
        "APP_ENVIRONMENT=Production",
        "UNRELATED=private",
    ] {
        f.write("private file", text);
        assert_eq!(
            load(f.secret(SecretFormat::Dotenv), &dev()).unwrap_err(),
            Error::EnvironmentKey
        );
    }
}

#[test]
fn in_memory_defaults_replace_base_file() {
    let f = Fixture::new();
    fs::remove_file(f.0.join("base.toml")).unwrap();
    assert_eq!(
        load(f.loader().defaults(BASE), &[]).unwrap().server.port,
        3300
    );
}

#[test]
fn no_process_environment_side_effects() {
    let f = Fixture::new();
    f.write("private file", "APP_SERVICE__TOKEN='000-synthetic'\n");
    let before: std::collections::BTreeMap<_, _> = std::env::vars_os().collect();
    load(f.secret(SecretFormat::Dotenv), &dev()).unwrap();
    let after: std::collections::BTreeMap<_, _> = std::env::vars_os().collect();
    assert_eq!(before, after);
}

#[test]
fn errors_and_debug_have_no_sensitive_values_paths_or_sources() {
    use std::error::Error as _;
    let f = Fixture::new();
    f.write("private file", "APP_SERVER__PORT=unique-sensitive-value");
    let loader = f.secret(SecretFormat::Dotenv).defaults(BASE);
    let debug = format!("{loader:?}");
    assert!(!debug.contains(&f.0.to_string_lossy().to_string()));
    let error = load(loader, &dev()).unwrap_err();
    assert_eq!(error, Error::Type);
    assert!(error.source().is_none());
    assert!(!format!("{error:?}: {error}").contains("unique-sensitive-value"));
    let config = load(
        f.loader(),
        &[("APP_SERVICE__TOKEN", "unique-sensitive-value")],
    )
    .unwrap();
    assert!(!format!("{config:?}").contains("unique-sensitive-value"));
    f.write("production.toml", "synthetic-private parse error");
    let error = load(f.loader(), &[]).unwrap_err();
    assert_eq!(error, Error::Toml);
    assert_eq!(error.to_string(), "invalid configuration TOML");
}

#[test]
fn invalid_prefixes_base_keys_and_reserved_selector_fail() {
    let f = Fixture::new();
    for prefix in ["", "app", "APP_", "APP__X", "../APP"] {
        assert_eq!(
            load(Loader::new(&f.0, prefix), &[]).unwrap_err(),
            Error::Prefix
        );
    }
    for base in ["environment = 'Development'", "BAD = 'x'", "a__b = 'x'"] {
        assert_eq!(
            load(f.loader().defaults(base), &[]).unwrap_err(),
            Error::EnvironmentKey
        );
    }
}

#[test]
fn oversized_and_nonunicode_environment_values_fail_safely() {
    let f = Fixture::new();
    assert_eq!(
        load(
            f.loader(),
            &[("APP_SERVICE__TOKEN", &"x".repeat(MAX_FILE_BYTES + 1))]
        )
        .unwrap_err(),
        Error::Input
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let entries = [("APP_SERVICE__TOKEN".into(), OsString::from_vec(vec![255]))];
        assert_eq!(
            f.loader().load_from_env::<Settings>(entries).unwrap_err(),
            Error::Input
        );
    }
}

#[cfg(unix)]
#[test]
fn symlink_secret_and_fifo_inputs_are_rejected() {
    use std::os::unix::{ffi::OsStrExt, fs::symlink};
    let f = Fixture::new();
    f.write("real file", "APP_SERVICE__TOKEN=placeholder");
    symlink(f.0.join("real file"), f.0.join("private file")).unwrap();
    assert_eq!(
        load(f.secret(SecretFormat::Dotenv), &dev()).unwrap_err(),
        Error::File
    );
    fs::remove_file(f.0.join("private file")).unwrap();
    let path = std::ffi::CString::new(f.0.join("private file").as_os_str().as_bytes()).unwrap();
    // Synthetic FIFO in this test's owned temporary directory only.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    assert_eq!(
        load(f.secret(SecretFormat::Dotenv), &dev()).unwrap_err(),
        Error::File
    );
}
