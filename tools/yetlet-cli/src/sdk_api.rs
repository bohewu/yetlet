//! Selected SDK feature vocabulary, independent of package versions and release records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SdkApi {
    Legacy,
    FeatureModules,
}

impl SdkApi {
    /// A declared module feature chooses the new contract even if its closure is
    /// invalid. Compatibility checks must reject that selection, never fall back.
    pub(crate) fn from_features<'a>(features: impl IntoIterator<Item = &'a str>) -> Self {
        if features
            .into_iter()
            .any(|name| matches!(name, "web" | "view"))
        {
            Self::FeatureModules
        } else {
            Self::Legacy
        }
    }

    /// Finish composed, CLI-owned SDK fields, import paths, and profile guidance.
    /// Keep project text and assets independent of SDK selection.
    pub(crate) fn apply_to_template(
        self,
        entries: &mut [(&'static str, String)],
        browser: bool,
    ) -> Result<(), String> {
        if self == Self::Legacy {
            return Ok(());
        }
        let manifest = entries
            .iter_mut()
            .find(|entry| entry.0 == "Cargo.toml")
            .ok_or("feature-module template is missing Cargo.toml")?;
        manifest.1 = module_manifest(&manifest.1, browser)?;
        if !browser {
            let mut main_seen = false;
            for (path, content) in entries {
                if matches!(
                    *path,
                    "src/main.rs" | "src/tailwind.rs" | "src/vite.rs" | "src/styles_asset.rs"
                ) {
                    *content = web_source(path, content)?;
                    main_seen |= *path == "src/main.rs";
                } else if *path == "README.md" {
                    module_export_guidance(content)?;
                }
            }
            if !main_seen {
                return Err("feature-module template is missing src/main.rs".into());
            }
        }
        Ok(())
    }
}

const MODULE_EXPORT_GUIDANCE: &str = r#"
## Deployment configuration generation

The Docker, Compose and Kubernetes generators currently accept only the retained
legacy `server-axum` source and lock representation. This project's `web`/`html`
SDK profile is not supported by those generators, including with `--service`.
Changing Cargo path overrides or regenerating the lock does not add support.

The `--service` option still selects bind configuration, health routes and
graceful shutdown. Prepare deployment configuration separately for this SDK
profile. Generating configuration for it requires separate exporter support.
"#;

fn module_export_guidance(readme: &mut String) -> Result<(), String> {
    let (_, legacy) = include_str!("../templates/SELECTED-README.md")
        .split_once("\nWith `--service`,")
        .ok_or("missing built-in legacy exporter guidance")?;
    let legacy = format!("\nWith `--service`,{legacy}");
    match readme.matches(&legacy).count() {
        0 => Ok(()),
        1 => {
            *readme = readme.replacen(&legacy, MODULE_EXPORT_GUIDANCE, 1);
            Ok(())
        }
        _ => Err("ambiguous built-in legacy exporter guidance".into()),
    }
}

fn module_manifest(source: &str, browser: bool) -> Result<String, String> {
    let mut manifest: toml::Value = source
        .parse()
        .map_err(|e| format!("feature-module template manifest: {e}"))?;
    let dependencies = if browser {
        manifest
            .get_mut("target")
            .and_then(|target| target.get_mut("cfg(target_arch = \"wasm32\")"))
            .and_then(|target| target.get_mut("dependencies"))
    } else {
        manifest.get_mut("dependencies")
    };
    let dependency = dependencies
        .and_then(|dependencies| dependencies.get_mut("yetlet"))
        .and_then(toml::Value::as_table_mut)
        .ok_or("feature-module template is missing its yetlet dependency")?;
    if dependency
        .get("default-features")
        .and_then(toml::Value::as_bool)
        != Some(false)
    {
        return Err("feature-module template must disable SDK default features".into());
    }
    let selected: Vec<&str> = dependency
        .get("features")
        .and_then(toml::Value::as_array)
        .and_then(|features| features.iter().map(toml::Value::as_str).collect())
        .ok_or("feature-module template has invalid SDK features")?;
    let features: &[&str] = match (browser, selected.as_slice()) {
        (true, ["reactive"]) => &["reactive", "view"],
        (false, ["server-axum"]) => &["web", "html"],
        (false, ["server-axum", "config"]) => &["web", "html", "config"],
        _ => return Err("feature-module template has an unsupported SDK feature selection".into()),
    };
    dependency.insert(
        "features".into(),
        toml::Value::Array(features.iter().map(|feature| (*feature).into()).collect()),
    );
    toml::to_string_pretty(&manifest).map_err(|e| e.to_string())
}

fn web_source(path: &str, source: &str) -> Result<String, String> {
    let mut output = String::with_capacity(source.len());
    let mut imports = 0;
    for line in source.split_inclusive('\n') {
        let code = line.trim_start_matches([' ', '\t']);
        let indent = &line[..line.len() - code.len()];
        let code = code.strip_suffix('\n').unwrap_or(code);
        let replacement = match code {
            "use yetlet::server_axum::{App, Page};" => {
                imports += 1;
                "use yetlet::web::{App, Page};"
            }
            "use yetlet::server_axum::App;" => {
                imports += 1;
                "use yetlet::web::App;"
            }
            "use yetlet::server_axum::prelude::*;" => {
                imports += 1;
                "use yetlet::web::prelude::*;"
            }
            "use yetlet::server_axum::{DEFAULT_CSP, render_document};" => {
                imports += 1;
                "use yetlet::web::{DEFAULT_CSP, render_document};"
            }
            "use yetlet::{server_axum::{render_document, DEFAULT_CSP}, view::View};" => {
                imports += 1;
                "use yetlet::{web::{render_document, DEFAULT_CSP}, view::View};"
            }
            "yetlet::server_axum::DEFAULT_CSP" => "yetlet::web::DEFAULT_CSP",
            _ => code,
        };
        output.push_str(indent);
        output.push_str(replacement);
        if line.ends_with('\n') {
            output.push('\n');
        }
    }
    if imports != 1 {
        return Err(format!(
            "feature-module template {path} requires one canonical Web import"
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarations_select_one_profile_without_a_version_ceiling() {
        assert_eq!(
            SdkApi::from_features(["server-axum", "reactive"]),
            SdkApi::Legacy
        );
        for features in [
            vec!["web"],
            vec!["view"],
            vec!["server-axum", "web"],
            vec!["server-axum", "view"],
        ] {
            assert_eq!(SdkApi::from_features(features), SdkApi::FeatureModules);
        }
    }

    #[test]
    fn module_conversion_changes_only_owned_sdk_fields_and_lines() {
        let marker = "server-axum-server_axum";
        let mut entries =
            crate::scaffold::selected_files_version(marker, false, true, true, false, "0.10.0")
                .unwrap();
        let untouched = "use yetlet::server_axum::App;\nserver-axum-server_axum\n";
        entries.push(("frontend/user.txt", untouched.into()));
        entries.iter_mut().find(|entry| entry.0 == "src/main.rs").unwrap().1
            .push_str("\n// use yetlet::server_axum::App;\nconst USER_TEXT: &str = \"yetlet::server_axum::App\";\n");
        let before = entries.clone();
        SdkApi::FeatureModules
            .apply_to_template(&mut entries, false)
            .unwrap();
        for ((path, content), (_, original)) in entries.iter().zip(&before) {
            if *path == "Cargo.toml" {
                let mut actual: toml::Value = content.parse().unwrap();
                let mut expected: toml::Value = original.parse().unwrap();
                assert_eq!(
                    actual["dependencies"]["yetlet"]["features"]
                        .as_array()
                        .unwrap(),
                    &vec!["web".into(), "html".into(), "config".into()]
                );
                actual["dependencies"]["yetlet"]
                    .as_table_mut()
                    .unwrap()
                    .remove("features");
                expected["dependencies"]["yetlet"]
                    .as_table_mut()
                    .unwrap()
                    .remove("features");
                assert_eq!(actual, expected);
            } else if *path == "README.md" {
                let (old_prefix, old_suffix) = original.split_once("With `--service`,").unwrap();
                let old_suffix = old_suffix
                    .split_once("separate qualification.\n")
                    .unwrap()
                    .1;
                let (new_prefix, new_suffix) = content
                    .split_once("## Deployment configuration generation\n")
                    .unwrap();
                let new_suffix = new_suffix.split_once("exporter support.\n").unwrap().1;
                assert_eq!(new_prefix, old_prefix);
                assert_eq!(new_suffix, old_suffix);
            } else if matches!(*path, "src/main.rs" | "src/tailwind.rs") {
                assert!(content.contains("yetlet::web::"));
                if *path == "src/main.rs" {
                    assert!(content.contains("// use yetlet::server_axum::App;"));
                    assert!(
                        content.contains("const USER_TEXT: &str = \"yetlet::server_axum::App\";")
                    );
                }
            } else {
                assert_eq!(content, original, "{path}");
            }
        }
        assert_eq!(
            entries
                .iter()
                .find(|entry| entry.0 == "frontend/user.txt")
                .unwrap()
                .1,
            untouched
        );
    }

    #[test]
    fn browser_adds_view_without_enabling_native_or_default_features() {
        let mut entries =
            crate::browser::files_version("browser-web-view", false, "0.10.0").unwrap();
        let original = entries.clone();
        SdkApi::FeatureModules
            .apply_to_template(&mut entries, true)
            .unwrap();
        let manifest: toml::Value = entries
            .iter()
            .find(|entry| entry.0 == "Cargo.toml")
            .unwrap()
            .1
            .parse()
            .unwrap();
        let dependencies = &manifest["target"]["cfg(target_arch = \"wasm32\")"]["dependencies"];
        assert_eq!(
            dependencies["yetlet"]["features"].as_array().unwrap(),
            &vec!["reactive".into(), "view".into()]
        );
        assert_eq!(
            dependencies["yetlet"]["default-features"].as_bool(),
            Some(false)
        );
        assert_eq!(
            dependencies["yetlet-browser"]["default-features"].as_bool(),
            Some(false)
        );
        assert_eq!(
            dependencies["yetlet-browser"]["features"]
                .as_array()
                .unwrap(),
            &vec!["dom".into(), "dom-keyed".into()]
        );
        for ((path, content), (_, before)) in entries.iter().zip(original) {
            if *path != "Cargo.toml" {
                assert_eq!(content, &before, "{path}");
            }
        }
    }

    #[test]
    fn legacy_output_is_unchanged_and_unknown_new_template_shapes_fail_closed() {
        let mut entries = crate::scaffold::selected_files_version(
            "legacy",
            true,
            false,
            false,
            false,
            "0.9.0-alpha.7",
        )
        .unwrap();
        let original = entries.clone();
        SdkApi::Legacy
            .apply_to_template(&mut entries, false)
            .unwrap();
        assert_eq!(entries, original);
        let readme = &entries
            .iter()
            .find(|entry| entry.0 == "README.md")
            .unwrap()
            .1;
        assert!(readme.contains("## Service profile (`--service`)"));
        assert!(readme.contains("yetlet generate docker --output"));
        assert!(readme.contains("yetlet generate compose --output"));
        assert!(readme.contains("yetlet generate k8s --output"));
        entries
            .iter_mut()
            .find(|entry| entry.0 == "src/main.rs")
            .unwrap()
            .1
            .replace_range(.., "use yetlet::server_axum::Unknown;\n");
        assert!(
            SdkApi::FeatureModules
                .apply_to_template(&mut entries, false)
                .unwrap_err()
                .contains("canonical Web import")
        );
    }
}
