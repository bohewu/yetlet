//! Opt-in Linux OCI configuration only. No process execution or source export.
use crate::{framework, platform::fs, scaffold};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use toml::Value;

mod delivery;

type Files = Vec<(&'static str, String)>;

const PACKAGES: &[(&str, &[&str])] = &[
    ("yetlet-core", &["binding.rs", "lib.rs"]),
    ("yetlet-html", &["lib.rs"]),
    (
        "yetlet-server-axum",
        &[
            "app.rs",
            "context.rs",
            "document.rs",
            "lib.rs",
            "middleware.rs",
            "response.rs",
        ],
    ),
];
const INHERITED: &str = "version.workspace=true\nedition.workspace=true\nrust-version.workspace=true\nlicense.workspace=true\nrepository.workspace=true\n";
const ROOT_VALUES: &str = "[package]\nversion='0.1.0'\nedition='2024'\nrust-version='1.99'\nlicense='MIT OR Apache-2.0'\nrepository='https://github.com/bohewu/yetlet'\n[dependencies]\naxum='0.8'\ntokio={version='1',features=['macros','net','rt-multi-thread']}\n";

/// Write a NEW protected directory using the existing native no-replace adapter.
/// The operator must prepare and verify both named contexts separately.
pub fn generate(manifest: &Path, output: &Path) -> Result<(), String> {
    generate_inner(manifest, output, || {})
}

fn generate_inner(
    manifest: &Path,
    output: &Path,
    after_inspection: impl FnOnce(),
) -> Result<(), String> {
    generate_checked(manifest, output, render, after_inspection)
}

/// Shared cloud inspection, revalidation and native publication boundary.
/// Renderers supply built-in file names only; the writer rejects tree traversal.
pub(crate) fn generate_checked(
    manifest: &Path,
    output: &Path,
    render: impl FnOnce(&Snapshot) -> Result<Files, String>,
    after_inspection: impl FnOnce(),
) -> Result<(), String> {
    fs::ensure_create_supported()?;
    let output = framework::absolute(output)
        .map_err(|_| "output must be nonempty without parent traversal")?;
    let name = output
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("output requires a Unicode directory name")?;
    // Also reject POSIX spellings that become separators on Windows.
    if name.is_empty() || matches!(name, "." | "..") || name.contains(['/', '\\', '\0']) {
        return Err("invalid output directory component".into());
    }
    let parent = output
        .parent()
        .ok_or("output requires an existing protected parent")?;
    let app = fs::open_app(manifest)?;
    let snapshot = inspect(&app)?;
    let files = render(&snapshot)?;
    after_inspection();
    if snapshot != inspect(&app)? {
        return Err("source snapshot changed during generation; nothing written".into());
    }
    fs::create_project(parent, name, &files)
}

#[derive(PartialEq)]
pub(crate) struct Snapshot {
    pub(crate) manifest: Value,
    pub(crate) lock: Vec<u8>,
    main: Vec<u8>,
    minimal_workspace: String,
    framework_files: BTreeMap<String, Vec<u8>>,
    pub(crate) with_config: bool,
    pub(crate) with_assets: bool,
    with_public_settings: bool,
    selected_frontend: Option<String>,
    app_files: BTreeMap<String, Vec<u8>>,
    inspected_inputs: BTreeMap<String, Vec<u8>>,
}

fn parse(bytes: &[u8]) -> Result<Value, String> {
    std::str::from_utf8(bytes)
        .map_err(|_| "input must be UTF-8")?
        .parse()
        .map_err(|_| "input is invalid TOML".into())
}

fn inspect(app: &fs::App) -> Result<Snapshot, String> {
    let root_entries = fs::app_entries(app, false)?;
    if root_entries.iter().any(|n| {
        matches!(
            n.as_str(),
            "build.rs" | ".cargo" | "tests" | "examples" | "benches"
        )
    }) {
        return Err("only the generated cloud app layout is supported; no build scripts, Cargo config or extra targets/sources".into());
    }
    let manifest = parse(&fs::read_app_file(app, "Cargo.toml")?)?;
    let name = manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .ok_or("generated app package name is required")?;
    scaffold::validate_name(name).map_err(str::to_owned)?;
    let dependencies = manifest
        .get("dependencies")
        .ok_or("generated app dependencies are required")?;
    let mut roots = Vec::new();
    for package in ["yetlet-core", "yetlet-server-axum"] {
        let path = dependencies
            .get(package)
            .and_then(|d| d.get("path"))
            .and_then(Value::as_str)
            .ok_or("both generated framework path dependencies are required")?;
        let path = Path::new(path);
        if !path.is_absolute() {
            return Err("generated framework dependencies must be absolute local paths".into());
        }
        let path = framework::absolute(path).map_err(|_| "unsafe framework dependency path")?;
        if path.file_name() != Some(package.as_ref())
            || path.parent().and_then(Path::file_name) != Some("crates".as_ref())
        {
            return Err("unsupported framework dependency layout".into());
        }
        roots.push(
            path.parent()
                .and_then(Path::parent)
                .ok_or("missing framework root")?
                .to_owned(),
        );
    }
    if roots[0] != roots[1] {
        return Err("framework dependencies must share one workspace".into());
    }
    let with_config = dependencies.get("yetlet-config").is_some();
    let with_assets = dependencies.get("serde_json").is_some();
    let selected = if root_entries.iter().any(|p| p == "base.toml") {
        parse(&fs::read_app_file(app, "base.toml")?)?
            .get("cli")
            .is_some()
    } else {
        false
    };
    let frontend = if selected {
        // Inspect the application's public files only; never read backend secrets
        // or redirect generator reads through ambient runtime environment roots.
        let mut frontend = None;
        for profile in ["Development", "Staging", "Production"] {
            let settings =
                crate::settings::load_from_env(app, [("APP_ENVIRONMENT".into(), profile.into())])?;
            if profile == "Production" {
                frontend = Some(settings.vite.root);
            }
        }
        frontend.unwrap()
    } else {
        String::new()
    };
    let frontend_app = if selected && with_assets {
        Some(fs::frontend_directory(app, Path::new(&frontend))?)
    } else {
        None
    };
    let plugin = if with_assets {
        let package = normalize(&fs::read_app_file(
            frontend_app.as_ref().unwrap_or(app),
            "package.json",
        )?)?;
        let mut expected = include_str!("../templates/vite/tailwind-package.json").to_owned();
        if selected {
            expected = expected.replace(
                "\"build\": \"vite build\"",
                "\"build\": \"vite build\",\n    \"dev\": \"vite --config vite.dev.config.mjs\"",
            );
        }
        package == expected
    } else {
        false
    };
    let mut expected = if selected {
        scaffold::selected_files(name, true, with_config, plugin, with_assets)?
    } else if with_assets {
        scaffold::vite_files(name, &roots[0], true, with_config, plugin)?
    } else if with_config {
        scaffold::config_files(name, &roots[0], true)?
    } else {
        scaffold::files(name, &roots[0], true)?
    };
    let mut expected_manifest = parse(expected[0].1.as_bytes())?;
    if selected {
        // Only normal explicit path overrides of the pinned declarations qualify.
        // Every other manifest key, version, feature and dependency stays exact.
        for package in ["yetlet-core", "yetlet-server-axum", "yetlet-config"] {
            if expected_manifest["dependencies"].get(package).is_some() {
                expected_manifest["dependencies"][package]
                    .as_table_mut()
                    .unwrap()
                    .insert(
                        "path".into(),
                        Value::String(
                            roots[0]
                                .join(format!("crates/{package}"))
                                .to_str()
                                .ok_or("framework path must be UTF-8")?
                                .into(),
                        ),
                    );
            }
        }
    }
    if manifest != expected_manifest {
        return Err("manifest must match an exact generated cloud profile; edited dependencies, features and layouts are unsupported".into());
    }
    let mut source_names: Vec<_> = expected
        .iter()
        .filter_map(|(p, _)| p.strip_prefix("src/"))
        .collect();
    source_names.sort();
    if fs::app_entries(app, true)? != source_names {
        return Err("only exact generated cloud sources are supported".into());
    }
    let mut app_files = BTreeMap::new();
    let mut inspected_inputs = BTreeMap::new();
    let mut newest_input = std::time::SystemTime::UNIX_EPOCH;
    for (path, text) in expected.drain(..) {
        if matches!(
            path,
            "README.md" | ".gitignore" | ".env.example" | "Cargo.toml"
        ) {
            continue;
        }
        let input_path = if selected {
            path.strip_prefix("frontend/")
                .map(|tail| format!("{}/{tail}", frontend.trim_end_matches('/')))
                .unwrap_or_else(|| path.into())
        } else {
            path.into()
        };
        let (input_app, read_path) = match (frontend_app.as_ref(), path.strip_prefix("frontend/")) {
            (Some(frontend_app), Some(relative)) => (frontend_app, relative),
            _ => (app, path),
        };
        let (bytes, modified) = fs::read_app_input(input_app, read_path)?;
        let matches = if selected
            && matches!(
                path,
                "base.toml" | "development.toml" | "staging.toml" | "production.toml"
            ) {
            public_settings_match(&bytes, text.as_bytes())?
        } else {
            normalize(&bytes)? == normalize(text.as_bytes())?
        };
        if !matches {
            return Err("cloud source/public config contract is unverified; only exact generated public inputs are supported".into());
        }
        newest_input = newest_input.max(modified);
        if path.starts_with("src/") || path.ends_with(".toml") {
            app_files.insert(path.into(), bytes.clone());
        }
        inspected_inputs.insert(input_path, bytes);
    }
    let main = app_files
        .remove("src/main.rs")
        .ok_or("missing generated main")?;
    if with_assets {
        app_files.extend(delivery::assets(
            frontend_app.as_ref().unwrap_or(app),
            newest_input,
        )?);
    }
    let lock = fs::read_app_file(app, "Cargo.lock")?;
    crate::app::validate_lock(&parse(&lock)?)?;
    let (minimal_workspace, mut framework_files) = inspect_framework(&roots[0])?;
    if with_config {
        delivery::config_framework(&roots[0], &mut framework_files)?;
    }
    Ok(Snapshot {
        manifest,
        lock,
        main,
        minimal_workspace,
        framework_files,
        with_config,
        with_assets,
        with_public_settings: with_config || selected,
        selected_frontend: selected.then_some(frontend),
        app_files,
        inspected_inputs,
    })
}

// Runtime public tables retain the established exact generated values. Only the
// declared CLI namespace may vary, after all profiles passed the strict loader.
fn public_settings_match(actual: &[u8], expected: &[u8]) -> Result<bool, String> {
    let mut actual = parse(actual)?;
    let mut expected = parse(expected)?;
    actual
        .as_table_mut()
        .ok_or("public settings must be a table")?
        .remove("cli");
    expected
        .as_table_mut()
        .ok_or("public settings must be a table")?
        .remove("cli");
    Ok(actual == expected)
}

fn normalize(bytes: &[u8]) -> Result<String, String> {
    std::str::from_utf8(bytes)
        .map(|s| s.replace("\r\n", "\n"))
        .map_err(|_| "generated source must be UTF-8".into())
}

fn read_framework(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    fs::read_framework(&root.join(relative))
        .map_err(|_| "framework input is missing, oversized or unsafe".into())
}

fn inspect_framework(root: &Path) -> Result<(String, BTreeMap<String, Vec<u8>>), String> {
    let manifest = parse(&read_framework(root, "Cargo.toml")?)?;
    let workspace = manifest
        .get("workspace")
        .ok_or("framework workspace is required")?;
    let baseline: Value = ROOT_VALUES.parse().unwrap();
    // This is a bounded v1 profile, not a general Cargo graph resolver. Preserve
    // exact inherited values and reject a framework baseline we cannot build.
    if workspace.get("resolver").and_then(Value::as_str) != Some("3")
        || workspace.get("package") != baseline.get("package")
        || workspace.get("dependencies") != baseline.get("dependencies")
    {
        return Err(
            "unsupported framework workspace inheritance; expected the inspected v1 baseline"
                .into(),
        );
    }
    let mut minimal: Value = "[workspace]\nresolver='3'\nmembers=['crates/yetlet-core','crates/yetlet-html','crates/yetlet-server-axum']\n".parse().unwrap();
    let minimal_workspace = minimal["workspace"].as_table_mut().unwrap();
    minimal_workspace.insert("package".into(), workspace["package"].clone());
    minimal_workspace.insert("dependencies".into(), workspace["dependencies"].clone());
    let mut files = BTreeMap::new();
    for (package, sources) in PACKAGES {
        let directory = format!("crates/{package}");
        let names = directory_names(&root.join(&directory))?;
        if names
            .iter()
            .any(|n| matches!(n.as_str(), "build.rs" | ".cargo"))
        {
            return Err("framework build scripts and Cargo config are unsupported".into());
        }
        if directory_names(&root.join(&directory).join("src"))? != *sources {
            return Err("unsupported framework source layout".into());
        }
        let path = format!("{directory}/Cargo.toml");
        let bytes = read_framework(root, &path)?;
        validate_package(package, &parse(&bytes)?)?;
        files.insert(path, bytes);
        for source in *sources {
            let path = format!("{directory}/src/{source}");
            files.insert(path.clone(), read_framework(root, &path)?);
        }
    }
    Ok((
        toml::to_string(&minimal).map_err(|_| "cannot render minimal workspace")?,
        files,
    ))
}

fn directory_names(path: &Path) -> Result<Vec<String>, String> {
    // No contents are read through enumeration. Every selected file is opened
    // through the native no-symlink/reparse reader and later rehashed.
    let mut names = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|_| "cannot enumerate framework layout")? {
        names.push(
            entry
                .map_err(|_| "cannot enumerate framework layout")?
                .file_name()
                .into_string()
                .map_err(|_| "framework source names must be Unicode")?,
        );
    }
    names.sort();
    Ok(names)
}

fn validate_package(name: &str, manifest: &Value) -> Result<(), String> {
    let dependencies = match name {
        "yetlet-core" => "[dependencies]\n",
        "yetlet-html" => "[dependencies]\nyetlet-core={version='0.1.0',path='../yetlet-core'}\n",
        "yetlet-server-axum" => {
            "[dependencies]\naxum.workspace=true\nserde='1'\ntower='0.5'\nyetlet-core={version='0.1.0',path='../yetlet-core'}\nyetlet-html={version='0.1.0',path='../yetlet-html'}\n[dev-dependencies]\ntokio.workspace=true\nserde={version='1',features=['derive']}\ntower={version='0.5',features=['util']}\ntower-http={version='0.6',features=['set-header']}\nhttp-body='1'\nhttp-body-util='0.1'\n"
        }
        _ => return Err("unknown framework package".into()),
    };
    let expected: Value = format!("[package]\nname='{name}'\n{INHERITED}{dependencies}")
        .parse()
        .unwrap();
    let mut actual = manifest.clone();
    let package = actual
        .get_mut("package")
        .and_then(Value::as_table_mut)
        .ok_or("framework package is required")?;
    if !package.remove("description").is_some_and(|v| v.is_str()) || actual != expected {
        return Err(
            "unsupported framework package, inheritance, features or dependency closure".into(),
        );
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn inventory(files: &BTreeMap<String, Vec<u8>>) -> serde_json::Value {
    serde_json::Value::Object(
        files
            .iter()
            .map(|(name, bytes)| {
                (
                    name.clone(),
                    serde_json::json!({"sha256": digest(bytes), "bytes": bytes.len()}),
                )
            })
            .collect(),
    )
}

fn export_ignore(names: impl Iterator<Item = String>) -> String {
    let mut rules = std::collections::BTreeSet::new();
    rules.insert("!.dockerignore".to_owned());
    for name in names {
        let path = Path::new(&name);
        let mut parents = path.parent();
        while let Some(parent) = parents {
            if !parent.as_os_str().is_empty() {
                rules.insert(format!("!{}/", parent.to_str().unwrap()));
            }
            parents = parent.parent();
        }
        rules.insert(format!("!{name}"));
    }
    format!("**\n{}\n", rules.into_iter().collect::<Vec<_>>().join("\n"))
}

fn render(snapshot: &Snapshot) -> Result<Files, String> {
    if snapshot.with_public_settings || snapshot.with_assets {
        crate::compose::validate_lock(snapshot)?;
    }
    let mut overlay = snapshot.manifest.clone();
    for name in delivery::dependencies(snapshot) {
        overlay["dependencies"][name]["path"] =
            Value::String(format!("/work/yetlet/crates/{name}"));
    }
    let overlay = toml::to_string(&overlay).map_err(|_| "cannot render app overlay")?;
    // Prove reconstruction changes exactly the two permitted paths.
    let mut roundtrip: Value = overlay.parse().map_err(|_| "cannot parse app overlay")?;
    for name in delivery::dependencies(snapshot) {
        roundtrip["dependencies"][name]["path"] =
            snapshot.manifest["dependencies"][name]["path"].clone();
    }
    if roundtrip != snapshot.manifest {
        return Err("app overlay changed unsupported semantics".into());
    }
    let name = snapshot.manifest["package"]["name"].as_str().unwrap();
    let dockerfile = delivery::dockerfile(snapshot, name);
    let mut app = BTreeMap::from([
        ("Cargo.lock".into(), snapshot.lock.clone()),
        ("src/main.rs".into(), snapshot.main.clone()),
    ]);
    app.extend(snapshot.app_files.clone());
    let mut framework = snapshot.framework_files.clone();
    framework.insert(
        "Cargo.toml".into(),
        snapshot.minimal_workspace.as_bytes().to_vec(),
    );
    let app_ignore = export_ignore(app.keys().cloned());
    let framework_ignore = export_ignore(framework.keys().cloned());
    app.insert(".dockerignore".into(), app_ignore.as_bytes().to_vec());
    framework.insert(".dockerignore".into(), framework_ignore.as_bytes().to_vec());
    let mut files = vec![
        ("Dockerfile", dockerfile),
        (
            ".dockerignore",
            "**\n!Dockerfile\n!cargo-overlay/\n!cargo-overlay/Cargo.toml\n".into(),
        ),
        ("cargo-overlay/Cargo.toml", overlay),
        (
            "framework-workspace.toml",
            snapshot.minimal_workspace.clone(),
        ),
        ("app-src.dockerignore", app_ignore),
        ("yetlet-src.dockerignore", framework_ignore),
        (
            "README.md",
            if snapshot.selected_frontend.is_some() {
                include_str!("../templates/docker/SELECTED-DELIVERY.md").replace("@NAME@", name)
            } else if snapshot.with_config || snapshot.with_assets {
                include_str!("../templates/docker/DELIVERY.md").replace("@NAME@", name)
            } else {
                include_str!("../templates/docker/README.md").replace("@NAME@", name)
            },
        ),
    ];
    let generated: BTreeMap<String, Vec<u8>> = files
        .iter()
        .map(|(p, v)| ((*p).into(), v.as_bytes().to_vec()))
        .collect();
    let mut contexts = serde_json::json!({
        "schema_version": 1,
        "target": "linux/amd64",
        "default_context": inventory(&BTreeMap::from([
            ("Dockerfile".into(), generated["Dockerfile"].clone()),
            ("cargo-overlay/Cargo.toml".into(), generated["cargo-overlay/Cargo.toml"].clone()),
            (".dockerignore".into(), generated[".dockerignore"].clone()),
        ])),
        "app-src": inventory(&app), "yetlet-src": inventory(&framework),
        "generated_files_except_inventory": inventory(&generated),
    });
    if let Some(frontend) = &snapshot.selected_frontend {
        let source_paths: BTreeMap<_, _> = app
            .keys()
            .filter(|path| path.as_str() != ".dockerignore")
            .map(|path| {
                let source = if snapshot.with_assets && path.starts_with("dist/") {
                    format!("{}/{path}", frontend.trim_end_matches('/'))
                } else {
                    path.clone()
                };
                (path.clone(), source)
            })
            .collect();
        contexts["app-src-source-paths"] =
            serde_json::to_value(source_paths).map_err(|_| "cannot render app source mapping")?;
    }
    files.push((
        "build-context-inventory.json",
        format!(
            "{}\n",
            serde_json::to_string_pretty(&contexts).map_err(|_| "cannot render inventory")?
        ),
    ));
    Ok(files)
}

#[cfg(test)]
mod tests;
