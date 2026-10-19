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
    #[serde(skip)]
    root: PathBuf,
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
    fn explicit_development_overlay_and_env_use_existing_loader_precedence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.toml");
        std::fs::write(
            &path,
            "[oidc]\nclient_secret = 'synthetic-private-overlay'\n",
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
        assert_eq!(
            settings.oidc.client_secret.expose_secret(),
            "synthetic-private-overlay"
        );
        let mut overrides = vars;
        overrides.push((
            "APP_OIDC__CLIENT_SECRET".into(),
            "synthetic-environment".into(),
        ));
        assert_eq!(
            load_from_env(overrides)
                .unwrap()
                .oidc
                .client_secret
                .expose_secret(),
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
