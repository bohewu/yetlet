//! Public CLI projection of the application's existing base/profile settings.
use crate::platform::fs::{self, App};
use serde::Deserialize;
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub(crate) vite: Vite,
    pub(crate) generate: Generate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Vite {
    pub(crate) enabled: bool,
    pub(crate) root: String,
    pub(crate) auto_start: bool,
    pub(crate) auto_build: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Generate {
    pub(crate) k8s: K8s,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct K8s {
    pub(crate) replicas: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            vite: Vite {
                enabled: false,
                root: "frontend/".into(),
                auto_start: false,
                auto_build: false,
            },
            generate: Generate {
                k8s: K8s { replicas: 1 },
            },
        }
    }
}

const INVALID: &str = "CLI_PROJECT settings: invalid public CLI configuration; check base/profile declarations and APP_CLI overrides";

pub(crate) fn load(app: &App) -> Result<Settings, String> {
    load_from_env(app, std::env::vars_os())
}

pub(crate) fn load_from_env(
    app: &App,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
) -> Result<Settings, String> {
    let mut selected_root = None;
    let mut filtered = Vec::new();
    for (key, value) in environment {
        if key == "YETLET_CONFIG_ROOT" {
            let root = PathBuf::from(value);
            if !root.is_absolute() || root.components().any(|c| matches!(c, Component::ParentDir)) {
                return Err(INVALID.into());
            }
            selected_root = Some(root);
        } else {
            // Match case-insensitively only to recognize malformed related names.
            // The shared loader then enforces canonical spelling and Unicode.
            let upper = key.to_string_lossy().to_ascii_uppercase();
            if upper == "APP_ENVIRONMENT" || upper == "APP_CLI" || upper.starts_with("APP_CLI_") {
                filtered.push((key, value));
            }
        }
    }
    let root = selected_root
        .as_deref()
        .unwrap_or_else(|| fs::app_root(app));
    let protected = fs::open_directory(root).map_err(|_| INVALID.to_owned())?;
    let entries = fs::app_directory_entries(&protected, "").map_err(|_| INVALID.to_owned())?;
    let has_files = [
        "base.toml",
        "development.toml",
        "staging.toml",
        "production.toml",
    ]
    .iter()
    .any(|name| entries.iter().any(|entry| entry == name));
    let has_override = filtered.iter().any(|(key, _)| key != "APP_ENVIRONMENT");
    // Pin and validate only the sources selected by the shared profile selector.
    let profile = filtered
        .iter()
        .find(|(key, _)| key == "APP_ENVIRONMENT")
        .map(|(_, value)| value.as_os_str());
    let overlay = match profile.and_then(|value| value.to_str()) {
        None if profile.is_none() => "production.toml",
        Some("Production") => "production.toml",
        Some("Staging") => "staging.toml",
        Some("Development") => "development.toml",
        _ => return Err(INVALID.into()),
    };
    if !has_files {
        if selected_root.is_some() || has_override {
            return Err(INVALID.into());
        }
        return Ok(Settings::default());
    }
    for name in ["base.toml", overlay] {
        fs::read_app_file(&protected, name).map_err(|_| INVALID.to_owned())?;
    }
    // No Development secret source is enabled, and no backend APP overrides enter
    // this projection. Parser/merge/type/ENV behavior belongs to yetlet-config.
    let merged: serde_json::Value = yetlet_config::Loader::new(root, "APP")
        .load_from_env(filtered)
        .map_err(|_| INVALID.to_owned())?;
    let Some(cli) = merged.get("cli") else {
        return Ok(Settings::default());
    };
    let settings: Settings = serde_json::from_value(cli.clone()).map_err(|_| INVALID.to_owned())?;
    if !(1..=100).contains(&settings.generate.k8s.replicas)
        || !valid_frontend_root(&settings.vite.root)
    {
        return Err(INVALID.into());
    }
    Ok(settings)
}

pub(crate) fn valid_frontend_root(root: &str) -> bool {
    !root.is_empty()
        && root.len() <= 240
        && !root.contains([':', '\\', '\0', '\n', '\r'])
        && !root
            .trim_end_matches('/')
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
        && Path::new(root)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: &str = "[cli.vite]\nenabled=true\nroot='frontend/'\nauto_start=true\nauto_build=false\n[cli.generate.k8s]\nreplicas=1\n";
    fn app() -> (tempfile::TempDir, App) {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        // Use the native creator's protected ownership/ACL on Windows as well
        // as its private Unix mode; a raw TempDir may inherit writable ACLs.
        fs::create_project(temp.path(), "settings", &[]).unwrap();
        let app = fs::open_directory(&temp.path().join("settings")).unwrap();
        (temp, app)
    }
    #[test]
    fn same_loader_profile_and_scoped_environment_precedence() {
        let (_temp, app) = app();
        std::fs::write(fs::app_root(&app).join("base.toml"), BASE).unwrap();
        std::fs::write(
            fs::app_root(&app).join("production.toml"),
            "[cli.vite]\nauto_start=false\n",
        )
        .unwrap();
        let settings = load_from_env(
            &app,
            [
                ("APP_CLI__VITE__AUTO_BUILD".into(), "true".into()),
                ("APP_SERVICE__TOKEN".into(), "backend-secret".into()),
                ("APP_CLIENT_ID".into(), "backend-client".into()),
                ("YETLET_DEV_SECRETS".into(), "/missing/secret".into()),
            ],
        )
        .unwrap();
        assert!(!settings.vite.auto_start);
        assert!(settings.vite.auto_build);
    }
    #[test]
    fn legacy_defaults_and_partial_or_invalid_configuration_refusals() {
        let (_temp, app) = app();
        assert!(!load_from_env(&app, []).unwrap().vite.enabled);
        assert!(load_from_env(&app, [("APP_ENVIRONMENT".into(), "invalid".into())]).is_err());
        assert!(load_from_env(&app, [("APP_CLI__VITE__ENABLED".into(), "true".into())]).is_err());
        std::fs::write(fs::app_root(&app).join("base.toml"), BASE).unwrap();
        assert!(load_from_env(&app, []).is_err());
        std::fs::write(fs::app_root(&app).join("production.toml"), "").unwrap();
        for (key, value) in [
            ("APP_CLI__VITE__AUTO_BUILD", "yes"),
            ("APP_CLI__VITE__ROOT", "../escape"),
            ("APP_CLI__GENERATE__K8S__REPLICAS", "101"),
            ("APP_CLI__VITE__UNKNOWN", "true"),
            ("app_cli__vite__enabled", "true"),
        ] {
            let error = load_from_env(&app, [(key.into(), value.into())]).unwrap_err();
            assert_eq!(error, INVALID);
        }
    }
    #[test]
    fn root_is_relative_and_native_unicode_is_preserved() {
        for root in ["frontend/", "ui assets/前端", "web/nested"] {
            assert!(valid_frontend_root(root));
        }
        for root in ["", "/root", "..", "a/../b", "../a", "a\\b", "."] {
            assert!(!valid_frontend_root(root));
        }
    }
}
