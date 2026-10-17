use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::{io, net::SocketAddr, path::PathBuf};
use yetlet_config::{Loader, SecretFormat};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub server: Server,
    pub service: Service,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    host: String,
    port: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub public_message: String,
    token: SecretString,
}

fn invalid() -> io::Error {
    io::Error::other("application configuration invalid")
}

impl Settings {
    pub fn validate(&self, cloud: bool) -> io::Result<SocketAddr> {
        if self.service.token.expose_secret().trim().is_empty()
            || self.service.public_message.is_empty()
            || self.service.public_message.len() > 80
            || self.server.port == 0
            || !(self.server.host == "127.0.0.1" || cloud && self.server.host == "0.0.0.0")
        {
            return Err(invalid());
        }
        // The two accepted host literals and typed nonzero port bound listening.
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .map_err(|_| invalid())
    }
}

fn loader() -> Loader {
    Loader::new(env!("CARGO_MANIFEST_DIR"), "APP")
}

pub fn load() -> io::Result<Settings> {
    let mut loader = loader();
    // Only Development even inspects this explicitly selected, non-APP path.
    // Never discover .env or .env.local, or resolve a path against an unknown cwd.
    if std::env::var_os("APP_ENVIRONMENT").as_deref() == Some(std::ffi::OsStr::new("Development"))
        && let Some(path) = std::env::var_os("YETLET_DEV_SECRETS")
    {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(invalid());
        }
        loader = loader.development_secrets(path, SecretFormat::Dotenv);
    }
    loader.load().map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(profile: &str, token: &str) -> Settings {
        loader()
            .load_from_env([
                ("APP_ENVIRONMENT".into(), profile.into()),
                ("APP_SERVICE__TOKEN".into(), token.into()),
            ])
            .unwrap()
    }

    #[test]
    fn finite_profiles_exact_string_secret_and_app_validation() {
        for profile in ["Development", "Staging", "Production"] {
            let config = settings(profile, "000123");
            assert_eq!(config.service.token.expose_secret(), "000123");
            assert!(config.validate(false).is_ok());
            assert_eq!(
                config.service.public_message,
                format!("{profile} configuration")
            );
        }
        for token in ["", "   "] {
            assert!(settings("Production", token).validate(true).is_err());
        }
        let mut config = settings("Production", "000123");
        config.server.host = "0.0.0.0".into();
        assert!(config.validate(false).is_err());
        assert!(config.validate(true).is_ok());
        config.server.port = 0;
        assert!(config.validate(true).is_err());
        config.server.port = 3000;
        config.server.host = "localhost".into();
        assert!(config.validate(true).is_err());
    }

    #[test]
    fn explicit_development_file_is_lower_priority_than_os() {
        let selected = loader().development_secrets(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env.example"),
            SecretFormat::Dotenv,
        );
        let empty: Settings = selected
            .load_from_env([("APP_ENVIRONMENT".into(), "Development".into())])
            .unwrap();
        assert!(empty.validate(true).is_err());
        let overridden: Settings = selected
            .load_from_env([
                ("APP_ENVIRONMENT".into(), "Development".into()),
                ("APP_SERVICE__TOKEN".into(), "000123".into()),
            ])
            .unwrap();
        assert_eq!(overridden.service.token.expose_secret(), "000123");
        assert!(overridden.validate(true).is_ok());
    }

    #[test]
    fn loader_rejects_unknown_and_invalid_inputs_with_fixed_errors() {
        for (key, value) in [
            ("APP_ENVIRONMENT", "../Production"),
            ("APP_UNKNOWN", "private-value"),
            ("APP_SERVER__PORT", "private-value"),
            ("APP_SERVER__PORT", "65536"),
        ] {
            let result = loader().load_from_env::<Settings>([(key.into(), value.into())]);
            assert!(result.is_err());
            let error = result.err().unwrap();
            assert!(!format!("{error:?}: {error}").contains("private-value"));
        }
        // Even a selected missing development file is irrelevant in other profiles.
        for profile in ["Staging", "Production"] {
            let config: Settings = loader()
                .development_secrets(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("missing-private-file"),
                    SecretFormat::Dotenv,
                )
                .load_from_env([
                    ("APP_ENVIRONMENT".into(), profile.into()),
                    ("APP_SERVICE__TOKEN".into(), "000123".into()),
                ])
                .unwrap();
            assert!(config.validate(true).is_ok());
        }
    }
}
