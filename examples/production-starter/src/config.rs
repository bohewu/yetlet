//! Application settings use the same opt-in loader and public CLI namespace.
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::{
    ffi::{OsStr, OsString},
    fmt,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use yetlet::config::{Loader, SecretFormat};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub service: Service,
    pub logging: Logging,
    #[serde(default)]
    pub metrics: Metrics,
    pub cli: Cli,
    #[serde(skip)]
    root: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub host: String,
    pub port: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Database {
    pub path: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub name: String,
    pub api_token: SecretString,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Logging {
    pub level: LogLevel,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Metrics {
    pub enabled: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}
impl LogLevel {
    pub fn filter(self) -> tracing::level_filters::LevelFilter {
        match self {
            Self::Error => tracing::level_filters::LevelFilter::ERROR,
            Self::Warn => tracing::level_filters::LevelFilter::WARN,
            Self::Info => tracing::level_filters::LevelFilter::INFO,
            Self::Debug => tracing::level_filters::LevelFilter::DEBUG,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cli {
    pub rust: Rust,
    pub vite: Vite,
    pub generate: Generate,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rust {
    pub watch: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vite {
    pub enabled: bool,
    pub root: String,
    pub auto_start: bool,
    pub auto_build: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generate {
    pub k8s: K8s,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct K8s {
    pub replicas: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigError;
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("application configuration invalid")
    }
}
impl std::error::Error for ConfigError {}

fn absolute_clean(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
}

fn database_path_valid(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path != Path::new(":memory:")
        && !path.components().any(|p| matches!(p, Component::ParentDir))
        && (path.is_absolute()
            || path
                .components()
                .all(|p| matches!(p, Component::Normal(_) | Component::CurDir)))
}

impl Settings {
    pub fn address(&self) -> Result<SocketAddr, ConfigError> {
        // This anonymous operator board is deliberately local-only in all profiles.
        if self.server.host != "127.0.0.1" || self.server.port == 0 {
            return Err(ConfigError);
        }
        Ok(SocketAddr::from(([127, 0, 0, 1], self.server.port)))
    }
    pub fn database_path(&self) -> PathBuf {
        self.root.join(&self.database.path)
    }
    fn validate(&self) -> Result<(), ConfigError> {
        self.address()?;
        let token = self.service.api_token.expose_secret();
        if token.len() != 64
            || !token
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.service.name.trim().is_empty()
            || self.service.name.chars().count() > 80
            || self.service.name.chars().any(char::is_control)
            || !database_path_valid(&self.database.path)
        {
            return Err(ConfigError);
        }
        Ok(())
    }
}

pub fn load() -> Result<Settings, ConfigError> {
    load_with_env(std::env::vars_os().collect())
}

/// Explicit environment input keeps precedence tests isolated from process globals.
pub fn load_with_env(vars: Vec<(OsString, OsString)>) -> Result<Settings, ConfigError> {
    let value = |name: &str| vars.iter().find(|(k, _)| k == name).map(|(_, v)| v);
    let root = value("YETLET_CONFIG_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if !absolute_clean(&root) {
        return Err(ConfigError);
    }
    let mut loader = Loader::new(&root, "APP");
    if value("APP_ENVIRONMENT").is_some_and(|v| v == OsStr::new("Development"))
        && let Some(path) = value("YETLET_DEV_SECRETS")
    {
        let path = PathBuf::from(path);
        if !absolute_clean(&path) {
            return Err(ConfigError);
        }
        loader = loader.development_secrets(path, SecretFormat::Dotenv);
    }
    let mut settings: Settings = loader.load_with_env(vars).map_err(|_| ConfigError)?;
    settings.root = root;
    settings.validate()?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn vars(profile: &str) -> Vec<(OsString, OsString)> {
        vec![
            ("APP_ENVIRONMENT".into(), profile.into()),
            ("APP_SERVICE__API_TOKEN".into(), "a".repeat(64).into()),
        ]
    }
    #[test]
    fn profiles_are_finite_typed_and_os_overrides_profile() {
        for profile in ["Development", "Staging", "Production"] {
            let mut env = vars(profile);
            let s = load_with_env(env.clone()).unwrap();
            assert_eq!(s.service.name, format!("Issue tracker ({profile})"));
            assert_eq!(
                s.database_path(),
                Path::new(env!("CARGO_MANIFEST_DIR")).join("data/issues.sqlite3")
            );
            env.push(("APP_SERVICE__NAME".into(), "OS selected".into()));
            assert_eq!(load_with_env(env).unwrap().service.name, "OS selected");
        }
        let s = load_with_env(vec![(
            "APP_SERVICE__API_TOKEN".into(),
            "a".repeat(64).into(),
        )])
        .unwrap();
        assert_eq!(s.service.name, "Issue tracker (Production)");
    }
    #[test]
    fn metrics_default_off_and_typed_override() {
        assert!(!load_with_env(vars("Production")).unwrap().metrics.enabled);
        let mut env = vars("Production");
        env.push(("APP_METRICS__ENABLED".into(), "true".into()));
        assert!(load_with_env(env.clone()).unwrap().metrics.enabled);
        env.last_mut().unwrap().1 = "not-a-bool".into();
        assert!(load_with_env(env).is_err());
        // A deployment's older settings files need no new table.
        let directory = tempfile::tempdir().unwrap();
        let base = include_str!("../base.toml").replace("[metrics]\nenabled = false\n\n", "");
        std::fs::write(directory.path().join("base.toml"), base).unwrap();
        std::fs::write(directory.path().join("production.toml"), "").unwrap();
        let mut env = vars("Production");
        env.push(("YETLET_CONFIG_ROOT".into(), directory.path().into()));
        assert!(!load_with_env(env).unwrap().metrics.enabled);
    }

    #[test]
    fn dev_secret_is_explicit_and_below_os_and_unused_elsewhere() {
        let dir = tempfile::tempdir().unwrap();
        let secret = dir.path().join("private.env");
        std::fs::write(
            &secret,
            format!(
                "APP_SERVICE__API_TOKEN={}\nAPP_SERVICE__NAME=\"From secret\"\n",
                "b".repeat(64)
            ),
        )
        .unwrap();
        let mut env = vec![
            ("APP_ENVIRONMENT".into(), "Development".into()),
            ("YETLET_DEV_SECRETS".into(), secret.clone().into_os_string()),
        ];
        let s = load_with_env(env.clone()).unwrap();
        assert_eq!(s.service.name, "From secret");
        assert_eq!(s.service.api_token.expose_secret(), "b".repeat(64));
        env.push(("APP_SERVICE__NAME".into(), "From OS".into()));
        assert_eq!(load_with_env(env).unwrap().service.name, "From OS");
        for profile in ["Staging", "Production"] {
            let mut env = vars(profile);
            env.push((
                "YETLET_DEV_SECRETS".into(),
                dir.path().join("missing.env").into_os_string(),
            ));
            assert!(load_with_env(env).is_ok());
        }
        assert!(load_with_env(vec![("APP_ENVIRONMENT".into(), "Development".into())]).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn drive_relative_and_root_only_database_paths_cannot_depend_on_cwd() {
        for path in [r"C:issues.sqlite3", r"\issues.sqlite3"] {
            let mut env = vars("Development");
            env.push(("APP_DATABASE__PATH".into(), path.into()));
            assert!(load_with_env(env).is_err());
        }
        assert!(database_path_valid(Path::new(r"C:\data\issues.sqlite3")));
        assert!(database_path_valid(Path::new(r"data\issues.sqlite3")));
    }

    #[test]
    fn invalid_startup_has_fixed_nonsecret_errors() {
        for (key, value) in [
            ("APP_SERVICE__API_TOKEN", "PRIVATE_VALUE"),
            ("APP_UNKNOWN", "PRIVATE_VALUE"),
            ("APP_ENVIRONMENT", "../Production"),
            ("APP_SERVER__PORT", "0"),
            ("APP_SERVER__HOST", "0.0.0.0"),
            ("APP_DATABASE__PATH", ":memory:"),
            ("APP_DATABASE__PATH", "../escape"),
            ("YETLET_CONFIG_ROOT", "relative"),
        ] {
            let mut env = vars("Development");
            env.retain(|(k, _)| k != key);
            env.push((key.into(), value.into()));
            let e = load_with_env(env).err().unwrap();
            assert_eq!(e.to_string(), "application configuration invalid");
            assert!(!format!("{e:?}").contains("PRIVATE_VALUE"));
        }
    }
}
