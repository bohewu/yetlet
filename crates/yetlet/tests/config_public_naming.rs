#![cfg(not(target_arch = "wasm32"))]
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use yetlet::config::{Error, Loader, SecretFormat};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    retained: String,
    profile: String,
    overridden: u16,
    token: SecretString,
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "yetlet-config-naming-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(
            path.join("base.toml"),
            "retained='base'\nprofile='base'\noverridden=1\ntoken='synthetic-base'\n",
        )
        .unwrap();
        for profile in ["development", "staging", "production"] {
            fs::write(
                path.join(format!("{profile}.toml")),
                format!("profile='{profile}'\noverridden=2\n"),
            )
            .unwrap();
        }
        fs::write(
            path.join("development-secrets.toml"),
            "overridden=3\ntoken='synthetic-development'\n",
        )
        .unwrap();
        Self(path)
    }

    fn loader(&self) -> Loader {
        Loader::new(&self.0, "APP")
            .development_secrets(self.0.join("development-secrets.toml"), SecretFormat::Toml)
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

#[test]
fn explicit_environment_preserves_sources_and_precedence() {
    let fixture = Fixture::new();
    let loader = fixture.loader();
    for (selector, profile, token, overridden) in [
        (None, "production", "synthetic-base", 2),
        (Some("Production"), "production", "synthetic-base", 2),
        (Some("Staging"), "staging", "synthetic-base", 2),
        (
            Some("Development"),
            "development",
            "synthetic-development",
            3,
        ),
    ] {
        for with_override in [false, true] {
            let mut entries = Vec::new();
            if let Some(selector) = selector {
                entries.push(("APP_ENVIRONMENT", selector));
            }
            if with_override {
                entries.push(("APP_OVERRIDDEN", "4"));
                entries.push(("APP_TOKEN", "000123-synthetic-env"));
            }
            let settings: Settings = loader.load_with_env(env(&entries)).unwrap();
            assert_eq!(settings.retained, "base");
            assert_eq!(settings.profile, profile);
            assert_eq!(
                settings.overridden,
                if with_override { 4 } else { overridden }
            );
            assert_eq!(
                settings.token.expose_secret(),
                if with_override {
                    "000123-synthetic-env"
                } else {
                    token
                }
            );
        }
    }
}

#[test]
fn key_category_supports_imports_patterns_and_redacted_diagnostics() {
    use std::error::Error as _;
    use yetlet::config::Error::Key;

    let error = Key;
    assert!(matches!(error, Error::Key));
    assert_eq!(format!("{error:?}"), "Key");
    assert_eq!(
        error.to_string(),
        "unknown, malformed or duplicate configuration key"
    );
    assert!(error.source().is_none());
    {
        use yetlet::config::Error::*;
        assert!(matches!(error, Key));
    }
}

#[cfg(feature = "secret-provider")]
mod secret_provider {
    use super::*;
    use std::collections::BTreeMap;
    use yetlet::config::SecretBinding;
    use yetlet::secrets::{SecretError, SecretFuture, SecretProvider, SettingsSecrets};

    fn binding() -> SecretBinding<'static, Settings> {
        SecretBinding::new("selected-token", |settings| &mut settings.token)
    }

    #[tokio::test]
    async fn explicit_environment_and_secrets_preserves_binding_authority() {
        let fixture = Fixture::new();
        let loader = fixture.loader();
        let provider = SettingsSecrets::new(BTreeMap::from([(
            "selected-token".into(),
            SecretString::from("synthetic-provider"),
        )]));
        let entries = env(&[
            ("APP_ENVIRONMENT", "Development"),
            ("APP_OVERRIDDEN", "4"),
            ("APP_TOKEN", "synthetic-env"),
        ]);
        let settings: Settings = loader
            .load_with_env_and_secrets(entries.clone(), &provider, &[binding()])
            .await
            .unwrap();
        assert_eq!(settings.retained, "base");
        assert_eq!(settings.profile, "development");
        assert_eq!(settings.overridden, 4);
        assert_eq!(settings.token.expose_secret(), "synthetic-provider");
        assert!(!format!("{settings:?}").contains("synthetic-provider"));
        let unbound: Settings = loader
            .load_with_env_and_secrets(entries, &provider, &[])
            .await
            .unwrap();
        assert_eq!(unbound.token.expose_secret(), "synthetic-env");
    }

    struct FailingProvider(Option<SecretError>);

    impl SecretProvider for FailingProvider {
        fn get<'a>(&'a self, name: &'a str) -> SecretFuture<'a> {
            Box::pin(async move {
                assert_eq!(name, "selected-token");
                self.0.map_or(Ok(None), Err)
            })
        }
    }

    #[tokio::test]
    async fn explicit_environment_and_secrets_preserves_closed_errors() {
        let fixture = Fixture::new();
        let loader = fixture.loader();
        for (cause, expected) in [
            (None, Error::SecretMissing),
            (Some(SecretError::Unavailable), Error::SecretUnavailable),
            (Some(SecretError::AccessDenied), Error::SecretAccessDenied),
        ] {
            let provider = FailingProvider(cause);
            let entries = env(&[("APP_TOKEN", "synthetic-no-fallback")]);
            let error = loader
                .load_with_env_and_secrets::<Settings>(entries, &provider, &[binding()])
                .await
                .unwrap_err();
            assert_eq!(error, expected);
            let diagnostics = format!("{error:?} {error}");
            assert!(!diagnostics.contains("selected-token"));
            assert!(!diagnostics.contains("synthetic-no-fallback"));
        }
    }
}
