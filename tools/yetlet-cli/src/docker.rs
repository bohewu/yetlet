//! Opt-in Linux OCI configuration only. No process execution or source export.
use crate::{framework, platform::fs, scaffold};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use toml::Value;

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
    }) || fs::app_entries(app, true)? != ["main.rs"]
    {
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
    let expected = scaffold::files(name, &roots[0], true)?;
    if manifest != parse(expected[0].1.as_bytes())? {
        return Err("manifest must match the standalone generated native-ssr-v1 profile; edited dependencies, features and layouts are unsupported".into());
    }
    let main = fs::read_app_file(app, "src/main.rs")?;
    let source = std::str::from_utf8(&main).map_err(|_| "cloud source must be UTF-8")?;
    let template = include_str!("../templates/cloud/main.rs");
    let parsed = syn::parse_file(source).map_err(|_| "cloud main source is invalid Rust")?;
    let expected_source =
        syn::parse_file(template).map_err(|_| "built-in cloud contract is invalid")?;
    if parsed != expected_source || source.replace("\r\n", "\n") != template.replace("\r\n", "\n") {
        return Err("cloud main source contract is unverified; only the unedited generated template is supported".into());
    }
    let lock = fs::read_app_file(app, "Cargo.lock")?;
    crate::app::validate_lock(&parse(&lock)?)?;
    let (minimal_workspace, framework_files) = inspect_framework(&roots[0])?;
    Ok(Snapshot {
        manifest,
        lock,
        main,
        minimal_workspace,
        framework_files,
    })
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
    let mut overlay = snapshot.manifest.clone();
    for name in ["yetlet-core", "yetlet-server-axum"] {
        overlay["dependencies"][name]["path"] =
            Value::String(format!("/work/yetlet/crates/{name}"));
    }
    let overlay = toml::to_string(&overlay).map_err(|_| "cannot render app overlay")?;
    // Prove reconstruction changes exactly the two permitted paths.
    let mut roundtrip: Value = overlay.parse().map_err(|_| "cannot parse app overlay")?;
    for name in ["yetlet-core", "yetlet-server-axum"] {
        roundtrip["dependencies"][name]["path"] =
            snapshot.manifest["dependencies"][name]["path"].clone();
    }
    if roundtrip != snapshot.manifest {
        return Err("app overlay changed unsupported semantics".into());
    }
    let name = snapshot.manifest["package"]["name"].as_str().unwrap();
    let dockerfile = include_str!("../templates/docker/Dockerfile").replace("@NAME@", name);
    let mut app = BTreeMap::from([
        ("Cargo.lock".into(), snapshot.lock.clone()),
        ("src/main.rs".into(), snapshot.main.clone()),
    ]);
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
            include_str!("../templates/docker/README.md").replace("@NAME@", name),
        ),
    ];
    let generated: BTreeMap<String, Vec<u8>> = files
        .iter()
        .map(|(p, v)| ((*p).into(), v.as_bytes().to_vec()))
        .collect();
    let contexts = serde_json::json!({
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
