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
    #[serde(default)]
    pub(crate) target: Target,
    #[serde(skip)]
    pub(crate) tls: Tls,
    #[serde(default)]
    pub(crate) rust: Rust,
    pub(crate) vite: Vite,
    pub(crate) generate: Generate,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Target {
    #[default]
    Native,
    Browser,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tls {
    pub(crate) enabled: bool,
    pub(crate) certificate: String,
    pub(crate) private_key: String,
}

impl Tls {
    pub(crate) fn paths(
        &self,
        app: &App,
        frontend: &str,
    ) -> Result<Option<(PathBuf, PathBuf)>, String> {
        if !self.enabled {
            return Ok(None);
        }
        let certificate = tls_path(app, &self.certificate, frontend)?;
        let key = tls_path(app, &self.private_key, frontend)?;
        if certificate == key {
            return Err(INVALID.into());
        }
        Ok(Some((certificate, key)))
    }
}

fn tls_path(app: &App, value: &str, frontend: &str) -> Result<PathBuf, String> {
    if !valid_frontend_root(value)
        || value.ends_with('/')
        || Path::new(value).starts_with(frontend.trim_end_matches('/'))
    {
        return Err(INVALID.into());
    }
    let relative = Path::new(value);
    if let Some(parent) = relative.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::frontend_directory(app, parent).map_err(|_| INVALID.to_owned())?;
    }
    let path = fs::app_root(app).join(relative);
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| INVALID.to_owned())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(INVALID.into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(INVALID.into());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o022 != 0 {
            return Err(INVALID.into());
        }
    }
    Ok(path)
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rust {
    pub(crate) watch: bool,
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
            target: Target::Native,
            tls: Tls::default(),
            rust: Rust::default(),
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
const SOURCE: &str = "CLI_PROJECT settings source: check the public settings directory and required base/profile files for safe ownership and readability";
const PROFILE: &str = "CLI_PROJECT settings profile: select exactly Development, Staging or Production; the default is Production";
const VALUES: &str = "CLI_PROJECT settings values: check public TOML syntax, base-declared keys/types, ranges and target/TLS requirements";

fn loader_hint(error: yetlet_config::Error) -> &'static str {
    use yetlet_config::Error;
    match error {
        Error::File => SOURCE,
        Error::Environment => PROFILE,
        Error::Input | Error::Toml | Error::Key | Error::Type | Error::Settings => VALUES,
        _ => INVALID,
    }
}

pub(crate) fn load(app: &App) -> Result<Settings, String> {
    load_with_env(app, std::env::vars_os())
}

pub(crate) fn load_with_env(
    app: &App,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
) -> Result<Settings, String> {
    load_selected(app, environment).map(|(settings, _)| settings)
}

// Return the already validated profile selection for read-only plan reporting.
pub(crate) fn load_selected(
    app: &App,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
) -> Result<(Settings, &'static str), String> {
    let mut selected_root = None;
    let mut filtered = Vec::new();
    for (key, value) in environment {
        if key == "YETLET_CONFIG_ROOT" {
            let root = PathBuf::from(value);
            if !root.is_absolute() || root.components().any(|c| matches!(c, Component::ParentDir)) {
                return Err(SOURCE.into());
            }
            selected_root = Some(root);
        } else {
            // Match case-insensitively only to recognize malformed related names.
            // The shared loader then enforces canonical spelling and Unicode.
            let upper = key.to_string_lossy().to_ascii_uppercase();
            if upper == "APP_ENVIRONMENT"
                || upper == "APP_CLI"
                || upper.starts_with("APP_CLI_")
                || upper == "APP_SERVER__TLS"
                || upper.starts_with("APP_SERVER__TLS_")
            {
                filtered.push((key, value));
            }
        }
    }
    let root = selected_root
        .as_deref()
        .unwrap_or_else(|| fs::app_root(app));
    let protected = fs::open_directory(root).map_err(|_| SOURCE.to_owned())?;
    let entries = fs::app_directory_entries(&protected, "").map_err(|_| SOURCE.to_owned())?;
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
        _ => return Err(PROFILE.into()),
    };
    let environment = match overlay {
        "development.toml" => "Development",
        "staging.toml" => "Staging",
        _ => "Production",
    };
    if !has_files {
        if selected_root.is_some() || has_override {
            return Err(SOURCE.into());
        }
        return Ok((Settings::default(), environment));
    }
    for name in ["base.toml", overlay] {
        fs::read_app_file(&protected, name).map_err(|_| SOURCE.to_owned())?;
    }
    // No Development secret source is enabled, and no backend APP overrides enter
    // this projection. Parser/merge/type/ENV behavior belongs to yetlet-config.
    let merged: serde_json::Value = yetlet_config::Loader::new(root, "APP")
        .load_with_env(filtered)
        .map_err(|error| loader_hint(error).to_owned())?;
    let mut settings: Settings = match merged.get("cli") {
        Some(cli) => serde_json::from_value(cli.clone()).map_err(|_| VALUES.to_owned())?,
        None => Settings::default(),
    };
    if let Some(tls) = merged.get("server").and_then(|server| server.get("tls")) {
        settings.tls = serde_json::from_value(tls.clone()).map_err(|_| VALUES.to_owned())?;
    }
    if settings.tls.enabled && overlay != "development.toml" {
        return Err(VALUES.into());
    }
    if settings.tls.enabled
        && [&settings.tls.certificate, &settings.tls.private_key]
            .iter()
            .any(|path| {
                !valid_frontend_root(path)
                    || path.ends_with('/')
                    || Path::new(path).starts_with(settings.vite.root.trim_end_matches('/'))
            })
    {
        return Err(VALUES.into());
    }
    if !(1..=100).contains(&settings.generate.k8s.replicas)
        || !valid_frontend_root(&settings.vite.root)
    {
        return Err(VALUES.into());
    }
    if settings.target == Target::Browser
        && (!settings.vite.enabled || settings.rust.watch || settings.tls.enabled)
    {
        return Err("CLI_PROJECT settings: browser target requires Vite enabled, Rust watch disabled and no native TLS".into());
    }
    Ok((settings, environment))
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
    fn browser_target_uses_shared_profile_overrides_and_rejects_native_capabilities() {
        let (_temp, app) = app();
        let base = format!("[cli]\ntarget='native'\n{BASE}[cli.rust]\nwatch=false\n");
        std::fs::write(fs::app_root(&app).join("base.toml"), &base).unwrap();
        std::fs::write(
            fs::app_root(&app).join("production.toml"),
            "[cli]\ntarget='browser'\n",
        )
        .unwrap();
        assert_eq!(load_with_env(&app, []).unwrap().target, Target::Browser);
        let override_target = [("APP_CLI__TARGET".into(), "native".into())];
        assert_eq!(
            load_with_env(&app, override_target).unwrap().target,
            Target::Native
        );
        for overlay in ["[cli]\ntarget='unknown'\n", "[cli]\ntarget=true\n"] {
            std::fs::write(fs::app_root(&app).join("production.toml"), overlay).unwrap();
            assert_eq!(load_with_env(&app, []).unwrap_err(), VALUES);
        }
        for overlay in [
            "[cli]\ntarget='browser'\n[cli.rust]\nwatch=true\n",
            "[cli]\ntarget='browser'\n[cli.vite]\nenabled=false\n",
        ] {
            std::fs::write(fs::app_root(&app).join("production.toml"), overlay).unwrap();
            assert_eq!(
                load_with_env(&app, []).unwrap_err(),
                "CLI_PROJECT settings: browser target requires Vite enabled, Rust watch disabled and no native TLS"
            );
        }
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
        let settings = load_with_env(
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
        assert!(!settings.rust.watch);
    }

    #[test]
    fn rust_watch_defaults_to_false_and_uses_shared_profile_environment_precedence() {
        let (_temp, app) = app();
        assert!(!load_with_env(&app, []).unwrap().rust.watch);
        std::fs::write(fs::app_root(&app).join("base.toml"), BASE).unwrap();
        std::fs::write(fs::app_root(&app).join("production.toml"), "").unwrap();
        assert!(!load_with_env(&app, []).unwrap().rust.watch);

        std::fs::write(
            fs::app_root(&app).join("base.toml"),
            format!("{BASE}[cli.rust]\nwatch=false\n"),
        )
        .unwrap();
        std::fs::write(
            fs::app_root(&app).join("development.toml"),
            "[cli.rust]\nwatch=true\n",
        )
        .unwrap();
        assert!(!load_with_env(&app, []).unwrap().rust.watch);
        let development = [("APP_ENVIRONMENT".into(), "Development".into())];
        assert!(load_with_env(&app, development).unwrap().rust.watch);
        for (profile, value, expected) in [
            ("Production", "true", true),
            ("Development", "false", false),
        ] {
            let settings = load_with_env(
                &app,
                [
                    ("APP_ENVIRONMENT".into(), profile.into()),
                    ("APP_CLI__RUST__WATCH".into(), value.into()),
                ],
            )
            .unwrap();
            assert_eq!(settings.rust.watch, expected);
        }
    }

    #[test]
    fn rust_watch_rejects_partial_unknown_and_wrong_type_settings() {
        let (_temp, app) = app();
        std::fs::write(fs::app_root(&app).join("production.toml"), "").unwrap();
        for rust in [
            "[cli.rust]\n",
            "[cli.rust]\nwatch=true\nunknown=false\n",
            "[cli.rust]\nwatch='true'\n",
            "[cli.rust]\nwatch=1\n",
        ] {
            std::fs::write(
                fs::app_root(&app).join("base.toml"),
                format!("{BASE}{rust}"),
            )
            .unwrap();
            assert_eq!(load_with_env(&app, []).unwrap_err(), VALUES);
        }
        std::fs::write(
            fs::app_root(&app).join("base.toml"),
            format!("{BASE}[cli.rust]\nwatch=false\n"),
        )
        .unwrap();
        for (key, value) in [
            ("APP_CLI__RUST__WATCH", "yes"),
            ("APP_CLI__RUST__WATCH", "1"),
            ("APP_CLI__RUST__UNKNOWN", "false"),
        ] {
            assert_eq!(
                load_with_env(&app, [(key.into(), value.into())]).unwrap_err(),
                VALUES
            );
        }
    }
    #[test]
    fn legacy_defaults_and_partial_or_invalid_configuration_refusals() {
        let (_temp, app) = app();
        assert!(!load_with_env(&app, []).unwrap().vite.enabled);
        assert_eq!(
            load_with_env(&app, [("APP_ENVIRONMENT".into(), "invalid".into())]).unwrap_err(),
            PROFILE
        );
        assert_eq!(
            load_with_env(&app, [("APP_CLI__VITE__ENABLED".into(), "true".into())]).unwrap_err(),
            SOURCE
        );
        assert_eq!(
            load_with_env(
                &app,
                [("YETLET_CONFIG_ROOT".into(), fs::app_root(&app).into())]
            )
            .unwrap_err(),
            SOURCE
        );
        std::fs::write(fs::app_root(&app).join("base.toml"), BASE).unwrap();
        assert_eq!(load_with_env(&app, []).unwrap_err(), SOURCE);
        std::fs::write(fs::app_root(&app).join("production.toml"), "").unwrap();
        for (key, value) in [
            ("APP_CLI__VITE__AUTO_BUILD", "yes"),
            ("APP_CLI__VITE__ROOT", "../escape"),
            ("APP_CLI__GENERATE__K8S__REPLICAS", "101"),
            ("APP_CLI__VITE__UNKNOWN", "true"),
            ("app_cli__vite__enabled", "true"),
        ] {
            let error = load_with_env(&app, [(key.into(), value.into())]).unwrap_err();
            assert_eq!(error, VALUES);
        }
    }
    #[test]
    fn tls_uses_public_development_settings_without_reading_secret_files() {
        let (_temp, app) = app();
        std::fs::write(
            fs::app_root(&app).join("base.toml"),
            format!("{BASE}[server.tls]\nenabled=false\ncertificate=''\nprivate_key=''\n"),
        )
        .unwrap();
        std::fs::write(fs::app_root(&app).join("development.toml"), "[server.tls]\nenabled=true\ncertificate='.dev/certs/localhost.pem'\nprivate_key='.dev/certs/localhost-key.pem'\n").unwrap();
        std::fs::write(fs::app_root(&app).join("production.toml"), "").unwrap();
        let settings = load_with_env(
            &app,
            [
                ("APP_ENVIRONMENT".into(), "Development".into()),
                (
                    "YETLET_DEV_SECRETS".into(),
                    "/missing-private-selector".into(),
                ),
            ],
        )
        .unwrap();
        assert!(settings.tls.enabled);
        assert_eq!(settings.tls.paths(&app, "frontend/").unwrap_err(), INVALID);
        assert!(!load_with_env(&app, []).unwrap().tls.enabled);
        assert!(
            !load_with_env(
                &app,
                [
                    ("APP_ENVIRONMENT".into(), "Development".into()),
                    ("APP_SERVER__TLS__ENABLED".into(), "false".into()),
                ]
            )
            .unwrap()
            .tls
            .enabled
        );
        for (key, value) in [
            ("APP_SERVER__TLS__ENABLED", "true"),
            ("APP_SERVER__TLS__UNKNOWN", "private-value"),
            ("app_server__tls__enabled", "true"),
        ] {
            assert_eq!(
                load_with_env(&app, [(key.into(), value.into())]).unwrap_err(),
                VALUES
            );
        }
        for path in [
            "../secret.pem",
            "/outside.pem",
            "frontend/key.pem",
            "frontend/../key.pem",
        ] {
            assert_eq!(
                load_with_env(
                    &app,
                    [
                        ("APP_ENVIRONMENT".into(), "Development".into()),
                        ("APP_SERVER__TLS__PRIVATE_KEY".into(), path.into()),
                    ]
                )
                .unwrap_err(),
                VALUES
            );
        }
    }

    #[test]
    fn source_checks_still_precede_profile_selection_and_keep_generic_fallback() {
        let (_temp, app) = app();
        let missing = fs::app_root(&app).join("synthetic-missing-root");
        for root in [PathBuf::from("synthetic-relative-root"), missing] {
            assert_eq!(
                load_with_env(
                    &app,
                    [
                        ("YETLET_CONFIG_ROOT".into(), root.into_os_string()),
                        ("APP_ENVIRONMENT".into(), "synthetic-profile".into()),
                    ],
                )
                .unwrap_err(),
                SOURCE
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            assert_eq!(
                load_with_env(
                    &app,
                    [("APP_ENVIRONMENT".into(), OsString::from_vec(vec![0xff]))],
                )
                .unwrap_err(),
                PROFILE
            );
        }
        assert_eq!(loader_hint(yetlet_config::Error::Prefix), INVALID);
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
