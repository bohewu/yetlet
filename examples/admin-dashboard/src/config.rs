//! App-owned settings; no secret or global configuration service is required.
use serde::Deserialize;
use std::{
    ffi::OsString,
    io,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use yetlet_config::{Loader, SecretFormat};
use yetlet_secrets::SecretString;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub cli: Cli,
    pub oidc: Oidc,
    pub session: Session,
    #[serde(skip)]
    root: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub backend: String,
    pub valkey: Valkey,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Valkey {
    pub endpoint: SecretString,
    pub namespace: String,
    pub ttl_seconds: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oidc {
    pub enabled: bool,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: SecretString,
    pub subjects: Subjects,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subjects {
    pub alice: String,
    pub bob: String,
    pub viewer: String,
}
impl Oidc {
    pub fn subject_map(&self) -> io::Result<std::collections::BTreeMap<String, String>> {
        let mut map = std::collections::BTreeMap::new();
        for (name, subject) in [
            ("alice", &self.subjects.alice),
            ("bob", &self.subjects.bob),
            ("viewer", &self.subjects.viewer),
        ] {
            if !subject.is_empty() && map.insert(subject.clone(), name.to_owned()).is_some() {
                return Err(invalid());
            }
        }
        Ok(map)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    host: String,
    port: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Database {
    path: PathBuf,
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

fn invalid() -> io::Error {
    io::Error::other("admin dashboard configuration invalid")
}
fn absolute_clean(path: &Path) -> bool {
    path.is_absolute() && !path.components().any(|c| matches!(c, Component::ParentDir))
}
impl Settings {
    pub fn session_backend(&self) -> io::Result<yetlet_lab_operations_board::SessionBackend> {
        use yetlet_lab_operations_board::SessionBackend;
        match self.session.backend.as_str() {
            "memory" => Ok(SessionBackend::Memory),
            "valkey" => {
                #[cfg(feature = "valkey")]
                {
                    use yetlet_secrets::ExposeSecret;
                    let selected = &self.session.valkey;
                    let endpoint = selected.endpoint.expose_secret();
                    if !endpoint.starts_with("redis://")
                        || endpoint.chars().any(char::is_control)
                        || selected.namespace.is_empty()
                        || selected.namespace.len() > 64
                        || !selected
                            .namespace
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
                        || !(1..=30 * 24 * 60 * 60).contains(&selected.ttl_seconds)
                    {
                        return Err(invalid());
                    }
                    Ok(SessionBackend::Valkey {
                        endpoint: endpoint.to_owned(),
                        namespace: selected.namespace.clone(),
                        ttl: std::time::Duration::from_secs(selected.ttl_seconds),
                    })
                }
                #[cfg(not(feature = "valkey"))]
                {
                    Err(invalid())
                }
            }
            _ => Err(invalid()),
        }
    }
    pub fn address(&self) -> io::Result<SocketAddr> {
        if self.server.host != "127.0.0.1" || self.server.port == 0 {
            return Err(invalid());
        }
        Ok(SocketAddr::from(([127, 0, 0, 1], self.server.port)))
    }
    pub fn database_path(&self) -> PathBuf {
        self.root.join(&self.database.path)
    }
    fn validate(&self) -> io::Result<()> {
        self.address()?;
        self.session_backend()?;
        // Unselected provider values are not opened or logged.
        let _ = (
            &self.session.valkey.endpoint,
            &self.session.valkey.namespace,
            self.session.valkey.ttl_seconds,
        );
        let path = &self.database.path;
        if path.as_os_str().is_empty()
            || path == Path::new(":memory:")
            || path.components().any(|c| matches!(c, Component::ParentDir))
            || (!path.is_absolute()
                && !path
                    .components()
                    .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)))
        {
            return Err(invalid());
        }
        let frontend = &self.cli.vite.root;
        if frontend.is_empty()
            || frontend.len() > 240
            || frontend.contains(['\\', ':'])
            || frontend.starts_with('/')
            || frontend
                .trim_end_matches('/')
                .split('/')
                .any(|p| matches!(p, "" | "." | ".."))
            || !(1..=100).contains(&self.cli.generate.k8s.replicas)
        {
            return Err(invalid());
        }
        // This authenticated multi-route reference uses built static assets.
        // The current dev bridge intentionally supports an anonymous root demo.
        if self.cli.vite.auto_start {
            return Err(invalid());
        }
        let _ = (
            self.cli.rust.watch,
            self.cli.vite.enabled,
            self.cli.vite.auto_build,
        );
        Ok(())
    }
}
pub fn load() -> io::Result<Settings> {
    load_from_env(std::env::vars_os().collect())
}
pub fn load_from_env(vars: Vec<(OsString, OsString)>) -> io::Result<Settings> {
    let root = vars
        .iter()
        .find(|(k, _)| k == "YETLET_CONFIG_ROOT")
        .map(|(_, v)| PathBuf::from(v))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if !absolute_clean(&root) {
        return Err(invalid());
    }
    let mut loader = Loader::new(&root, "APP");
    if let Some((_, path)) = vars
        .iter()
        .find(|(key, _)| key == "YETLET_DEVELOPMENT_SECRET_FILE")
    {
        let path = PathBuf::from(path);
        if !absolute_clean(&path) {
            return Err(invalid());
        }
        loader = loader.development_secrets(path, SecretFormat::Toml);
    }
    let mut settings: Settings = loader.load_from_env(vars).map_err(|_| invalid())?;
    settings.root = root;
    settings.validate()?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yetlet_secrets::ExposeSecret;

    #[test]
    fn session_selection_is_explicit_and_requires_the_compiled_capability() {
        let settings = load_from_env(vec![]).unwrap();
        assert_eq!(settings.session.backend, "memory");
        assert_eq!(settings.session.valkey.ttl_seconds, 1800);
        assert!(load_from_env(vec![("APP_SESSION__BACKEND".into(), "unknown".into())]).is_err());
        let selected = vec![
            ("APP_SESSION__BACKEND".into(), "valkey".into()),
            (
                "APP_SESSION__VALKEY__ENDPOINT".into(),
                "redis://127.0.0.1:6379/".into(),
            ),
        ];
        #[cfg(feature = "valkey")]
        {
            let loaded = load_from_env(selected.clone()).unwrap();
            assert!(matches!(
                loaded.session_backend().unwrap(),
                yetlet_lab_operations_board::SessionBackend::Valkey { ttl, .. }
                    if ttl == std::time::Duration::from_secs(1800)
            ));
            for seconds in [1, 60, 61, 3600, 30 * 24 * 60 * 60] {
                let mut vars = selected.clone();
                vars.push((
                    "APP_SESSION__VALKEY__TTL_SECONDS".into(),
                    seconds.to_string().into(),
                ));
                let loaded = load_from_env(vars).unwrap();
                assert!(matches!(
                    loaded.session_backend().unwrap(),
                    yetlet_lab_operations_board::SessionBackend::Valkey { ttl, .. }
                        if ttl == std::time::Duration::from_secs(seconds)
                ));
            }
            for (key, value) in [
                ("APP_SESSION__VALKEY__ENDPOINT", "http://invalid.example/"),
                ("APP_SESSION__VALKEY__NAMESPACE", "bad/name"),
                ("APP_SESSION__VALKEY__TTL_SECONDS", "0"),
                ("APP_SESSION__VALKEY__TTL_SECONDS", "2592001"),
                ("APP_SESSION__VALKEY__TTL_SECONDS", "18446744073709551615"),
            ] {
                let mut vars: Vec<(OsString, OsString)> =
                    selected.iter().filter(|(k, _)| k != key).cloned().collect();
                vars.push((key.into(), value.into()));
                assert!(load_from_env(vars).is_err());
            }
        }
        #[cfg(not(feature = "valkey"))]
        assert!(load_from_env(selected).is_err());
    }

    #[test]
    fn explicit_development_overlay_and_env_use_existing_loader_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.toml");
        std::fs::write(
            &path,
            "[oidc]\nclient_secret = 'synthetic-private-overlay'\n[session.valkey]\nttl_seconds = 3600\n",
        )
        .unwrap();
        let vars = vec![
            ("APP_ENVIRONMENT".into(), "Development".into()),
            (
                "YETLET_DEVELOPMENT_SECRET_FILE".into(),
                path.as_os_str().to_owned(),
            ),
        ];
        let settings = load_from_env(vars.clone()).unwrap();
        assert_eq!(settings.session.valkey.ttl_seconds, 3600);
        assert_eq!(
            settings.oidc.client_secret.expose_secret(),
            "synthetic-private-overlay"
        );
        let mut overrides = vars;
        overrides.push((
            "APP_OIDC__CLIENT_SECRET".into(),
            "synthetic-environment".into(),
        ));
        overrides.push(("APP_SESSION__VALKEY__TTL_SECONDS".into(), "7200".into()));
        let overridden = load_from_env(overrides).unwrap();
        assert_eq!(overridden.session.valkey.ttl_seconds, 7200);
        assert_eq!(
            overridden.oidc.client_secret.expose_secret(),
            "synthetic-environment"
        );
        for profile in ["Staging", "Production"] {
            let settings = load_from_env(vec![
                ("APP_ENVIRONMENT".into(), profile.into()),
                (
                    "YETLET_DEVELOPMENT_SECRET_FILE".into(),
                    dir.path().join("missing.toml").into_os_string(),
                ),
            ])
            .unwrap();
            assert!(settings.oidc.client_secret.expose_secret().is_empty());
            assert!(!settings.oidc.enabled);
        }
    }

    #[test]
    fn duplicate_subjects_fail_closed_and_no_mapping_assigns_no_access() {
        assert!(
            load_from_env(vec![])
                .unwrap()
                .oidc
                .subject_map()
                .unwrap()
                .is_empty()
        );
        let settings = load_from_env(vec![
            ("APP_OIDC__SUBJECTS__ALICE".into(), "same-subject".into()),
            ("APP_OIDC__SUBJECTS__VIEWER".into(), "same-subject".into()),
        ])
        .unwrap();
        assert!(settings.oidc.subject_map().is_err());
    }

    #[test]
    fn profiles_env_precedence_and_project_relative_database() {
        for profile in ["Development", "Staging", "Production"] {
            let settings = load_from_env(vec![
                ("APP_ENVIRONMENT".into(), profile.into()),
                ("APP_SERVER__PORT".into(), "3034".into()),
            ])
            .unwrap();
            assert_eq!(settings.address().unwrap().port(), 3034);
            assert_eq!(
                settings.database_path(),
                Path::new(env!("CARGO_MANIFEST_DIR")).join("data/board.sqlite3")
            );
            assert!(!settings.cli.vite.auto_start);
            assert!(settings.cli.vite.auto_build);
        }
        assert_eq!(
            load_from_env(vec![]).unwrap().address().unwrap().port(),
            3014
        );
    }
    #[test]
    fn unsafe_or_unsupported_settings_fail_before_listening_without_input_in_errors() {
        for (key, value) in [
            ("APP_ENVIRONMENT", "../Production"),
            ("APP_SERVER__HOST", "0.0.0.0"),
            ("APP_SERVER__PORT", "0"),
            ("APP_DATABASE__PATH", ":memory:"),
            ("APP_DATABASE__PATH", "../private"),
            ("APP_CLI__VITE__ROOT", "../private"),
            ("APP_CLI__VITE__AUTO_START", "true"),
            ("YETLET_CONFIG_ROOT", "relative"),
            ("APP_UNKNOWN", "private"),
        ] {
            let error = load_from_env(vec![(key.into(), value.into())])
                .err()
                .unwrap();
            assert_eq!(error.to_string(), "admin dashboard configuration invalid");
        }
    }
}
