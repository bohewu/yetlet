//! Optional application-owned host CSS coordination. No Core or renderer API.
use crate::{
    platform::{
        fs::{self, App, static_output as files, styles_output},
        process,
    },
    settings::{Settings, Styles},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};
const LIMIT: usize = 2 * 1024 * 1024;
const OWNER: &str = ".yetlet/styles/owner.json";
const PRODUCTION: &str = ".yetlet/styles/production.json";
const OWNER_PENDING: &str = ".yetlet/styles/owner.pending.json";
const PRODUCTION_PENDING: &str = ".yetlet/styles/production.pending.json";
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn error(message: impl std::fmt::Display) -> String {
    format!("CLI_STYLES: {message}")
}
pub(crate) fn enabled(settings: &Settings) -> Option<&Styles> {
    settings.styles.as_ref().filter(|s| s.enabled)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    logical: String,
    file: String,
    sha256: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Candidate {
    css: String,
    inputs: BTreeMap<String, String>,
}

fn inputs(app: &App) -> Result<BTreeMap<String, String>, String> {
    fn visit(
        root: &Path,
        prefix: &str,
        depth: usize,
        excluded: &[PathBuf],
        result: &mut BTreeMap<String, String>,
    ) -> Result<(), String> {
        if depth > 8 || result.len() > 1024 {
            return Err(error("style input bound exceeded"));
        }
        fs::open_directory(&root.join(prefix))?;
        for entry in std::fs::read_dir(root.join(prefix)).map_err(error)? {
            let entry = entry.map_err(error)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| error("non-Unicode style input"))?;
            if matches!(
                name.as_str(),
                "target" | ".yetlet" | "node_modules" | "dist"
            ) {
                continue;
            }
            let relative = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let ty = entry.file_type().map_err(error)?;
            if ty.is_dir() {
                if !excluded
                    .iter()
                    .any(|target| root.join(&relative).starts_with(target))
                {
                    visit(root, &relative, depth + 1, excluded, result)?;
                }
            } else if name.ends_with(".rs") || matches!(name.as_str(), "Cargo.toml" | "Cargo.lock")
            {
                result.insert(
                    relative.clone(),
                    digest(&files::read(root, &relative, LIMIT)?),
                );
            }
        }
        Ok(())
    }
    let root = fs::app_root(app);
    let mut result = BTreeMap::new();
    for name in ["Cargo.toml", "Cargo.lock"] {
        result.insert(name.into(), digest(&files::read(root, name, LIMIT)?));
    }
    let cargo = files::cargo_directories(app)?;
    for directory in ["src", "stylegen"] {
        visit(
            root,
            directory,
            0,
            &[cargo.target.clone(), cargo.build.clone()],
            &mut result,
        )?;
    }
    Ok(result)
}

pub(crate) fn prepare(app: &App, styles: &Styles) -> Result<Candidate, String> {
    let root = fs::app_root(app);
    let cargo = files::cargo_directories(app)?;
    if [&cargo.target, &cargo.build]
        .iter()
        .any(|cache| root.join(&styles.output).starts_with(cache))
    {
        return Err(error("style output overlaps selected app Cargo cache"));
    }
    let _ = ownership(app, styles)?;
    let generator = fs::open_app(&root.join(&styles.manifest))?;
    let manifest = crate::app::parse_file(
        "stylegen/Cargo.toml",
        fs::read_app_file(&generator, "Cargo.toml")?.as_slice(),
    )?;
    let package = &manifest["package"];
    let bin = manifest.get("bin").and_then(toml::Value::as_array);
    if package.get("publish").and_then(toml::Value::as_bool) != Some(false)
        || package
            .get("build")
            .is_some_and(|v| v.as_bool() != Some(false))
        || fs::app_root(&generator)
            .join("build.rs")
            .symlink_metadata()
            .is_ok()
        || manifest.get("target").is_some()
        || ["dependencies", "build-dependencies", "dev-dependencies"]
            .iter()
            .any(|key| {
                manifest
                    .get(*key)
                    .and_then(toml::Value::as_table)
                    .is_some_and(|t| !t.is_empty())
            })
        || !bin.is_some_and(|bins| {
            bins.len() == 1
                && bins[0].get("name").and_then(toml::Value::as_str) == Some(styles.bin.as_str())
                && bins[0].get("path").and_then(toml::Value::as_str) == Some("main.rs")
        })
    {
        return Err(error(
            "requires a private std-only standalone generator with one explicit main.rs binary; no build script or target dependencies",
        ));
    }
    let snapshot = inputs(app)?;
    let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let mut identity = Command::new(&compiler);
    identity.arg("-vV");
    offline(&mut identity);
    let identity = process::capture(&generator, &mut identity, 65536).map_err(error)?;
    let identity = std::str::from_utf8(&identity).map_err(error)?;
    let host = identity
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .filter(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        })
        .ok_or_else(|| error("selected rustc did not report a host target"))?;
    let mut sysroot = Command::new(&compiler);
    sysroot.args(["--print", "sysroot"]);
    offline(&mut sysroot);
    let sysroot = process::capture(&generator, &mut sysroot, 65536).map_err(error)?;
    let sysroot = PathBuf::from(std::str::from_utf8(&sysroot).map_err(error)?.trim());
    if !sysroot.is_absolute() {
        return Err(error("selected rustc sysroot is not absolute"));
    }
    // Keep rebuildable host output under the selected Cargo cache, separate
    // from private ownership records and any verification evidence directory.
    let mut ancestor = cargo.target.as_path();
    let mut suffix = Vec::new();
    while std::fs::symlink_metadata(ancestor)
        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    {
        suffix.push(
            ancestor
                .file_name()
                .ok_or_else(|| error("invalid Cargo target directory"))?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(|| error("invalid Cargo target ancestor"))?;
    }
    let mut target = fs::open_directory(ancestor)?;
    for part in suffix.iter().rev() {
        target = files::ensure(&target, part)?;
    }
    target = files::ensure(&target, "yetlet-styles".as_ref())?;
    target = files::ensure(
        &target,
        digest(root.as_os_str().as_encoded_bytes()).as_ref(),
    )?;
    let mut command = Command::new("cargo");
    command
        .args([
            "run",
            "--frozen",
            "--quiet",
            "--manifest-path",
            "Cargo.toml",
            "--bin",
            &styles.bin,
            "--no-default-features",
            "--target",
            host,
            "--target-dir",
        ])
        .arg(fs::app_root(&target))
        .env("RUSTUP_TOOLCHAIN", &sysroot)
        .env(
            "RUSTC",
            sysroot.join(if cfg!(windows) {
                "bin/rustc.exe"
            } else {
                "bin/rustc"
            }),
        )
        .env("CARGO_BUILD_BUILD_DIR", fs::app_root(&target));
    offline(&mut command);
    eprintln!("CLI_STYLES: generating host CSS ({host}); frozen private Cargo target");
    let css = process::capture(&generator, &mut command, LIMIT).map_err(error)?;
    let css =
        String::from_utf8(css).map_err(|_| error("generator stdout is not complete UTF-8 CSS"))?;
    if css.trim().is_empty() {
        return Err(error("generator stdout is empty"));
    }
    if inputs(app)? != snapshot {
        return Err(error("obsolete CSS candidate discarded; source changed"));
    }
    Ok(Candidate {
        css,
        inputs: snapshot,
    })
}
fn offline(command: &mut Command) {
    command
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
        .env("CARGO_NET_OFFLINE", "true");
}
fn optional(root: &Path, name: &str, limit: usize) -> Result<Option<Vec<u8>>, String> {
    // Absence alone is optional. Unsafe/nonregular/unreadable existing data is an error.
    match std::fs::symlink_metadata(root.join(name)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(error(e)),
        Ok(_) => files::read(root, name, limit).map(Some),
    }
}
fn record(bytes: &[u8]) -> Result<Record, String> {
    serde_json::from_slice(bytes)
        .map_err(|_| error("invalid private style record; reconcile explicitly"))
}
fn no_pending(root: &Path) -> Result<(), String> {
    for name in [OWNER_PENDING, PRODUCTION_PENDING] {
        match std::fs::symlink_metadata(root.join(name)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            _ => {
                return Err(error(
                    "UNACCEPTED pending style publication conflict; preserve evidence and reconcile explicitly",
                ));
            }
        }
    }
    Ok(())
}
// The marker is durable before any accepted output/record replacement. Retain
// exact previous metadata, even when the current record rename already succeeded.
fn pending(
    root: &Path,
    name: &str,
    previous: Option<&[u8]>,
    next: &[u8],
) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "previous": previous.map(std::str::from_utf8).transpose().map_err(error)?,
        "next": std::str::from_utf8(next).map_err(error)?,
    }))
    .map_err(error)?;
    let mut replaced = false;
    if let Err(e) = styles_output::replace_tracked(root, name, None, &bytes, &mut replaced) {
        // Only this attempt's newly renamed marker can be removed. No application
        // output was touched; a successful cleanup allows ordinary correction.
        if replaced {
            styles_output::complete(root, name, &bytes)?;
        }
        return Err(e);
    }
    Ok(bytes)
}
fn ownership(app: &App, styles: &Styles) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>), String> {
    let root = fs::app_root(app);
    no_pending(root)?;
    let owner = optional(root, OWNER, 65536)?;
    let old = optional(root, &styles.output, LIMIT)?;
    match (&owner, &old) {
        (None, None) => (),
        (Some(owner), Some(bytes)) => {
            let owner = record(owner)?;
            if owner.logical != "app-css"
                || owner.file != styles.output
                || owner.sha256 != digest(bytes)
            {
                return Err(error(
                    "generated CSS ownership/content conflict; reconcile explicitly",
                ));
            }
        }
        _ => {
            return Err(error(
                "generated CSS ownership conflict; reconcile explicitly",
            ));
        }
    }
    Ok((owner, old))
}
pub(crate) fn publish(app: &App, styles: &Styles, candidate: &Candidate) -> Result<(), String> {
    let root = fs::app_root(app);
    if candidate.css.len() > LIMIT
        || candidate.css.trim().is_empty()
        || inputs(app)? != candidate.inputs
    {
        return Err(error("obsolete/invalid CSS candidate discarded"));
    }
    let (owner, old) = ownership(app, styles)?;
    let next = serde_json::to_vec(&Record {
        logical: "app-css".into(),
        file: styles.output.clone(),
        sha256: digest(candidate.css.as_bytes()),
    })
    .map_err(error)?;
    if old.as_deref() == Some(candidate.css.as_bytes()) {
        return Ok(());
    }
    let intent = pending(root, OWNER_PENDING, owner.as_deref(), &next)?;
    let mut replaced = false;
    let result = (|| {
        styles_output::replace_tracked(
            root,
            &styles.output,
            old.as_deref(),
            candidate.css.as_bytes(),
            &mut replaced,
        )?;
        #[cfg(test)]
        if FAIL_OWNER.with(|flag| flag.replace(false)) {
            return Err(error("injected ownership metadata failure"));
        }
        styles_output::replace(root, OWNER, owner.as_deref(), &next)?;
        Ok::<(), String>(())
    })();
    if let Err(e) = result {
        if !replaced {
            styles_output::complete(root, OWNER_PENDING, &intent)?;
            return Err(e);
        }
        return Err(error(
            "CSS/ownership replacement may have completed: UNACCEPTED; pending evidence retains prior acceptance; reconcile explicitly",
        ));
    }
    // All writes, directory syncs and boundary checks precede this commit point.
    styles_output::complete(root, OWNER_PENDING, &intent)?;
    eprintln!(
        "CLI_STYLES: complete CSS published ({} bytes)",
        candidate.css.len()
    );
    Ok(())
}
pub(crate) fn accept(
    app: &App,
    styles: &Styles,
    candidate: &Candidate,
    vite: bool,
) -> Result<(), String> {
    let root = fs::app_root(app);
    let _ = ownership(app, styles)?;
    if inputs(app)? != candidate.inputs
        || files::read(root, &styles.output, LIMIT)? != candidate.css.as_bytes()
    {
        return Err(error(
            "build inputs/output changed; production acceptance refused",
        ));
    }
    if vite {
        eprintln!("CLI_STYLES: selected Vite build owns bundled CSS acceptance");
        return Ok(());
    }
    let hash = digest(candidate.css.as_bytes());
    let file = format!("assets/generated/app.{hash}.css");
    match optional(root, &file, LIMIT)? {
        Some(bytes) if bytes == candidate.css.as_bytes() => (),
        Some(_) => return Err(error("existing digest asset mismatches exact CSS bytes")),
        None => styles_output::replace(root, &file, None, candidate.css.as_bytes())?,
    }
    let previous = optional(root, PRODUCTION, 65536)?;
    let next = serde_json::to_vec(&Record {
        logical: "app-css".into(),
        file,
        sha256: hash,
    })
    .map_err(error)?;
    if previous.as_deref() == Some(next.as_slice()) {
        return Ok(());
    }
    let intent = pending(root, PRODUCTION_PENDING, previous.as_deref(), &next)?;
    let mut replaced = false;
    if let Err(e) =
        styles_output::replace_tracked(root, PRODUCTION, previous.as_deref(), &next, &mut replaced)
    {
        if !replaced {
            styles_output::complete(root, PRODUCTION_PENDING, &intent)?;
            return Err(e);
        }
        return Err(error(
            "production record replacement may have completed: UNACCEPTED; preserve pending evidence and reconcile explicitly",
        ));
    }
    styles_output::complete(root, PRODUCTION_PENDING, &intent)?;
    eprintln!("CLI_STYLES: production app-css accepted after dependent build success");
    Ok(())
}

pub(crate) fn scaffold(
    entries: &mut Vec<(&'static str, String)>,
    browser: bool,
    tailwind: bool,
    vite: bool,
) -> Result<(), String> {
    entries.extend([
        (
            "stylegen/Cargo.toml",
            include_str!("../templates/styles/manifest.toml").into(),
        ),
        (
            "stylegen/Cargo.lock",
            include_str!("../templates/styles/Cargo.lock").into(),
        ),
        (
            "stylegen/main.rs",
            include_str!("../templates/styles/main.rs").into(),
        ),
        (
            "stylegen/builder.rs",
            include_str!("../templates/styles/builder.rs").into(),
        ),
        (
            "stylegen/styles.rs",
            include_str!("../templates/styles/styles.rs").into(),
        ),
        (
            "stylegen/layout.rs",
            include_str!("../templates/styles/layout.rs").into(),
        ),
        (
            "stylegen/dialog.rs",
            include_str!("../templates/styles/dialog.rs").into(),
        ),
        (
            "src/style_names.rs",
            include_str!("../templates/styles/style_names.rs").into(),
        ),
    ]);
    let output = if vite {
        "frontend/style-output/app.generated.css"
    } else {
        "assets/generated/app.generated.css"
    };
    for (path, content) in entries.iter_mut() {
        match *path {
            "base.toml" => content.push_str(&format!("\n[cli.styles]\nenabled = true\nmanifest = \"stylegen/Cargo.toml\"\nbin = \"app-stylegen\"\noutput = \"{output}\"\n")),
            ".gitignore" => content.push_str("/.yetlet/\n/assets/generated/\n/frontend/style-output/\n/stylegen/target/\n"),
            "README.md" => content.push_str(include_str!("../templates/styles/README.md")),
            "frontend/app.ts" => {
                if browser { *content = format!("import './app.css';\n{content}"); }
                *content = content.replacen("import './app.css';", "import './app.css';\nimport './style-output/app.generated.css';",1);
            }
            "frontend/index.html" if browser => *content = content.replace("  <link rel=\"stylesheet\" href=\"./app.css\">\n", ""),
            "src/lib.rs" if browser => {
                *content = content.replacen("pub mod model;", "mod style_names;\npub mod model;",1);
                *content = content.replace(".attribute(\"class\", \"task-row\")", ".attribute(\"class\", format!(\"task-row {}\", crate::style_names::TOOLBAR))");
                // Keyed updates replace the complete class attribute, including
                // the initial mount. Preserve the selected Style class there.
                *content = content.replace(
                    "element.set_class_name(if task.completed {\n            \"task-row completed\"\n        } else {\n            \"task-row\"\n        });",
                    "element.set_class_name(&format!(\"{} {}\", crate::style_names::TOOLBAR, if task.completed {\n            \"task-row completed\"\n        } else {\n            \"task-row\"\n        }));",
                );
            }
            "src/main.rs" => {
                *content = format!("mod style_names;\n{content}");
                if tailwind {
                    *content = content.replace("\"mx-auto max-w-3xl p-8 font-sans text-slate-900\"", "format!(\"mx-auto max-w-3xl p-8 font-sans text-slate-900 {}\", style_names::SHELL)");
                } else {
                    *content = content.replace("Element::new(\"main\")", "Element::new(\"main\").attribute(\"class\", style_names::SHELL)");
                }
            }
            _ => ()
        }
    }
    if !vite {
        for (path, content) in entries.iter_mut() {
            if *path == "Cargo.toml" {
                for (name, declaration) in [
                    ("sha2", "sha2 = \"=0.10.9\""),
                    (
                        "serde",
                        "serde = { version = \"=1.0.229\", features = [\"derive\"] }",
                    ),
                    ("serde_json", "serde_json = \"=1.0.151\""),
                ] {
                    if content
                        .parse::<toml::Value>()
                        .map_err(|_| "invalid built-in app manifest")?["dependencies"]
                        .get(name)
                        .is_none()
                    {
                        *content = content.replacen(
                            "[dependencies]\n",
                            &format!("[dependencies]\n{declaration}\n"),
                            1,
                        );
                    }
                }
            }
        }
        if !tailwind {
            for (path, content) in entries.iter_mut() {
                if *path == "src/main.rs" {
                    let start = content
                        .find("async fn home(")
                        .ok_or("missing native home")?;
                    let end = content
                        .find(if content.contains("async fn live()") {
                            "async fn live()"
                        } else {
                            "#[tokio::main]"
                        })
                        .ok_or("missing native entry")?;
                    let handlers = include_str!("../templates/tailwind/handlers.rs")
                        .split("#[cfg(test)]")
                        .next()
                        .unwrap()
                        .replace("@ARGUMENTS@", "")
                        .replace("@MESSAGE@", "\"Hello <Yetlet> & Rust!\"")
                        .replace("tailwind::", "styles_asset::")
                        .replace(
                            "\"mx-auto max-w-3xl p-8 font-sans text-slate-900\"",
                            "style_names::SHELL",
                        );
                    content.replace_range(start..end, &handlers);
                    *content = format!("mod styles_asset;\n{content}")
                        .replace(".page(\"/\", \"Yetlet\", home)", ".get(\"/\", home)");
                    *content = content.replace("async fn main() -> Result<(), Box<dyn std::error::Error>> {", "async fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let stylesheet = styles_asset::Stylesheet::load()?;");
                    // The native protected scaffold has one server call, with app or router(...).
                    let token = if content.contains("config::serve(") {
                        "config::serve("
                    } else {
                        "axum::serve("
                    };
                    let start = content.find(token).ok_or("missing native server")?;
                    let rest = &content[start + token.len()..];
                    let prefix = if token == "config::serve(" {
                        "listener, tls,"
                    } else {
                        "listener,"
                    };
                    let after = rest
                        .strip_prefix(prefix)
                        .ok_or("missing native server arguments")?;
                    let end = after
                        .find(if token == "config::serve(" { ", " } else { ")" })
                        .ok_or("missing native router")?;
                    // router(...) has parentheses; use the known finite forms instead.
                    let _ = end;
                    for expression in ["router(settings, readiness)", "router(readiness)", "app"] {
                        let original = format!("{token}{prefix} {expression}");
                        if content.contains(&original) {
                            *content = content.replacen(&original,&format!("{token}{prefix} styles_asset::install({expression}, stylesheet)"),1);
                            break;
                        }
                    }
                }
            }
        } else {
            // Tailwind owns page and install: compose both app-owned consumers there.
            for (path, content) in entries.iter_mut() {
                if *path == "src/main.rs" {
                    *content = format!("mod styles_asset;\n{content}");
                }
                if *path == "src/tailwind.rs" {
                    *content = content.replace("    path: String,", "    generated: Arc<crate::styles_asset::Stylesheet>,\n    path: String,")
                        .replace("            path: format!", "            generated: crate::styles_asset::Stylesheet::load()?,\n            path: format!")
                        .replace("let link = format!(\"<link rel=\\\"stylesheet\\\" href=\\\"{}\\\"></head>\", self.path);", "let link = format!(\"<link rel=\\\"stylesheet\\\" href=\\\"{}\\\"><link rel=\\\"stylesheet\\\" href=\\\"{}\\\"></head>\", self.path, self.generated.path);")
                        .replace("    router\n        .route(", "    crate::styles_asset::install(router, stylesheet.generated.clone())\n        .route(");
                }
            }
        }
        entries.push((
            "src/styles_asset.rs",
            include_str!("../templates/styles/asset.rs").into(),
        ));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    root: PathBuf,
    styles: Styles,
    frontend: String,
}
pub(crate) fn frontend_environment(
    app: &App,
    settings: &Settings,
    command: &mut Command,
) -> std::io::Result<()> {
    command
        .env_remove("YETLET_STYLE_CLI")
        .env_remove("YETLET_INTERNAL_STYLE");
    if let Some(styles) = enabled(settings) {
        let selection = Selection {
            root: fs::app_root(app).into(),
            styles: styles.clone(),
            frontend: settings.vite.root.clone(),
        };
        command
            .env("YETLET_STYLE_CLI", std::env::current_exe()?)
            .env(
                "YETLET_INTERNAL_STYLE",
                serde_json::to_string(&selection).map_err(std::io::Error::other)?,
            );
    }
    Ok(())
}
// Internal process protocol, not a second settings loader. Only the parent's
// already-validated public finite selection crosses the filtered Node boundary.
pub(crate) fn worker(operation: &str) -> Result<(), String> {
    use std::io::{Read, Write};
    let descriptor = std::env::var("YETLET_INTERNAL_STYLE").map_err(error)?;
    if descriptor.len() > 4096 {
        return Err(error("private stage selection too large"));
    }
    let selection: Selection = serde_json::from_str(&descriptor).map_err(error)?;
    let mut settings = Settings::default();
    settings.vite.enabled = true;
    settings.vite.root = selection.frontend;
    if !selection.root.is_absolute()
        || !selection.styles.enabled
        || !selection.styles.validate(&settings.vite)
    {
        return Err(error("invalid private stage selection"));
    }
    let app = fs::open_app(&selection.root.join("Cargo.toml"))?;
    if operation == "prepare" {
        let candidate = prepare(&app, &selection.styles)?;
        std::io::stdout()
            .lock()
            .write_all(&serde_json::to_vec(&candidate).map_err(error)?)
            .map_err(error)?;
    } else if operation == "publish" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .take((4 * LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(error)?;
        if bytes.len() > 4 * LIMIT {
            return Err(error("private stage candidate too large"));
        }
        let candidate: Candidate = serde_json::from_slice(&bytes).map_err(error)?;
        publish(&app, &selection.styles, &candidate)?;
    } else {
        return Err(error("unknown private style stage"));
    }
    Ok(())
}

#[cfg(test)]
thread_local! { static FAIL_OWNER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_style_preserves_class_in_initial_view_and_keyed_updates() {
        let mut entries = crate::browser::files("styled-tasks", false).unwrap();
        scaffold(&mut entries, true, false, true).unwrap();
        let source = &entries.iter().find(|(p, _)| *p == "src/lib.rs").unwrap().1;
        assert!(source.contains("format!(\"task-row {}\", crate::style_names::TOOLBAR)"));
        assert!(
            source
                .contains("element.set_class_name(&format!(\"{} {}\", crate::style_names::TOOLBAR")
        );
        assert!(!source.contains("element.set_class_name(if task.completed"));
    }
    fn fixture() -> (tempfile::TempDir, App, Styles) {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        fs::create_project(
            temp.path(),
            "publisher",
            &[
                (
                    "Cargo.toml",
                    "[package]\nname='publisher'\nversion='0.0.0'\n[workspace]\n".into(),
                ),
                (
                    "Cargo.lock",
                    "version=4\n[[package]]\nname='publisher'\nversion='0.0.0'\n".into(),
                ),
                ("src/main.rs", "fn main(){}".into()),
                ("stylegen/main.rs", "fn main(){}".into()),
            ],
        )
        .unwrap();
        let app = fs::open_app(&temp.path().join("publisher/Cargo.toml")).unwrap();
        let styles = Styles {
            enabled: true,
            manifest: "stylegen/Cargo.toml".into(),
            bin: "app-stylegen".into(),
            output: "assets/generated/app.generated.css".into(),
        };
        (temp, app, styles)
    }
    fn candidate(app: &App, css: &str) -> Candidate {
        Candidate {
            css: css.into(),
            inputs: inputs(app).unwrap(),
        }
    }
    #[test]
    fn first_claim_noop_conflict_stale_and_production_digest_preservation() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let c = candidate(&app, "a { color: red; }\n");
        publish(&app, &s, &c).unwrap();
        let before = std::fs::metadata(root.join(&s.output))
            .unwrap()
            .modified()
            .unwrap();
        publish(&app, &s, &c).unwrap();
        assert_eq!(
            before,
            std::fs::metadata(root.join(&s.output))
                .unwrap()
                .modified()
                .unwrap()
        );
        accept(&app, &s, &c, false).unwrap();
        let accepted = files::read(root, PRODUCTION, 65536).unwrap();
        let record = record(&accepted).unwrap();
        assert_eq!(
            record.file,
            format!("assets/generated/app.{}.css", digest(c.css.as_bytes()))
        );
        assert_eq!(
            files::read(root, &record.file, LIMIT).unwrap(),
            c.css.as_bytes()
        );
        let next = candidate(&app, "a { color: blue; }\n");
        publish(&app, &s, &next).unwrap();
        assert_eq!(files::read(root, PRODUCTION, 65536).unwrap(), accepted);
        std::fs::write(root.join("src/main.rs"), "fn main(){panic!()}").unwrap();
        assert!(publish(&app, &s, &next).unwrap_err().contains("obsolete"));
        assert_eq!(files::read(root, PRODUCTION, 65536).unwrap(), accepted);
        std::fs::write(root.join(&s.output), "manual edit").unwrap();
        assert!(
            publish(&app, &s, &candidate(&app, "b{}\n"))
                .unwrap_err()
                .contains("conflict")
        );
        assert_eq!(std::fs::read(root.join(&s.output)).unwrap(), b"manual edit");
    }
    #[test]
    fn unowned_existing_file_and_mismatched_digest_are_refused() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let c = candidate(&app, "a{}\n");
        styles_output::replace(root, &s.output, None, b"unowned").unwrap();
        assert!(publish(&app, &s, &c).is_err());
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let c = candidate(&app, "a{}\n");
        publish(&app, &s, &c).unwrap();
        let file = format!("assets/generated/app.{}.css", digest(c.css.as_bytes()));
        styles_output::replace(root, &file, None, b"different").unwrap();
        assert!(accept(&app, &s, &c, false).unwrap_err().contains("digest"));
        assert!(!root.join(PRODUCTION).exists());
    }
    #[test]
    fn prewrite_and_postreplace_metadata_failure_are_distinct() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let c = candidate(&app, "a{}\n");
        publish(&app, &s, &c).unwrap();
        let old = files::read(root, OWNER, 65536).unwrap();
        let invalid = candidate(&app, &"a".repeat(LIMIT + 1));
        assert!(publish(&app, &s, &invalid).is_err());
        assert_eq!(
            files::read(root, &s.output, LIMIT).unwrap(),
            c.css.as_bytes()
        );
        FAIL_OWNER.with(|flag| flag.set(true));
        let next = candidate(&app, "b{}\n");
        assert!(publish(&app, &s, &next).unwrap_err().contains("UNACCEPTED"));
        assert_eq!(files::read(root, OWNER, 65536).unwrap(), old);
        assert_eq!(
            files::read(root, &s.output, LIMIT).unwrap(),
            next.css.as_bytes()
        );
        assert!(publish(&app, &s, &next).unwrap_err().contains("conflict"));
    }
    #[test]
    fn post_owner_record_rename_retains_pending_prior_evidence_and_refuses_retry() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let old = candidate(&app, "a{}\n");
        publish(&app, &s, &old).unwrap();
        accept(&app, &s, &old, false).unwrap();
        let prior = files::read(root, OWNER, 65536).unwrap();
        let production = files::read(root, PRODUCTION, 65536).unwrap();
        let next = candidate(&app, "b{}\n");
        styles_output::FAIL_AFTER_RENAME
            .with(|flag| *flag.borrow_mut() = Some("owner.json".into()));
        assert!(publish(&app, &s, &next).unwrap_err().contains("UNACCEPTED"));
        let current = files::read(root, OWNER, 65536).unwrap();
        assert_ne!(current, prior);
        assert_eq!(
            record(&current).unwrap().sha256,
            digest(next.css.as_bytes())
        );
        let intent = files::read(root, OWNER_PENDING, LIMIT).unwrap();
        let evidence: serde_json::Value = serde_json::from_slice(&intent).unwrap();
        assert_eq!(evidence["previous"].as_str().unwrap().as_bytes(), prior);
        assert!(publish(&app, &s, &next).unwrap_err().contains("pending"));
        assert!(prepare(&app, &s).err().unwrap().contains("pending"));
        assert!(
            accept(&app, &s, &next, false)
                .unwrap_err()
                .contains("pending")
        );
        assert_eq!(files::read(root, PRODUCTION, 65536).unwrap(), production);
        // Explicit operator rollback: restore both prior bytes/record, then clear
        // only the reviewed pending marker. Ordinary startup never does this.
        styles_output::replace(
            root,
            &s.output,
            Some(next.css.as_bytes()),
            old.css.as_bytes(),
        )
        .unwrap();
        styles_output::replace(root, OWNER, Some(&current), &prior).unwrap();
        styles_output::complete(root, OWNER_PENDING, &intent).unwrap();
        publish(&app, &s, &next).unwrap();
        accept(&app, &s, &next, false).unwrap();
        no_pending(root).unwrap();
    }
    #[test]
    fn post_production_record_rename_retains_pending_and_explicit_rollforward_recovers() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let old = candidate(&app, "a{}\n");
        publish(&app, &s, &old).unwrap();
        accept(&app, &s, &old, false).unwrap();
        let prior = files::read(root, PRODUCTION, 65536).unwrap();
        let old_file = record(&prior).unwrap().file;
        let next = candidate(&app, "b{}\n");
        publish(&app, &s, &next).unwrap();
        styles_output::FAIL_AFTER_RENAME
            .with(|flag| *flag.borrow_mut() = Some("production.json".into()));
        assert!(
            accept(&app, &s, &next, false)
                .unwrap_err()
                .contains("UNACCEPTED")
        );
        let current = files::read(root, PRODUCTION, 65536).unwrap();
        assert_ne!(current, prior);
        let intent = files::read(root, PRODUCTION_PENDING, LIMIT).unwrap();
        let evidence: serde_json::Value = serde_json::from_slice(&intent).unwrap();
        assert_eq!(evidence["previous"].as_str().unwrap().as_bytes(), prior);
        assert_eq!(
            files::read(root, &old_file, LIMIT).unwrap(),
            old.css.as_bytes()
        );
        assert!(
            accept(&app, &s, &next, false)
                .unwrap_err()
                .contains("pending")
        );
        assert!(publish(&app, &s, &next).unwrap_err().contains("pending"));
        // Explicit operator rollforward: independently verify current record and
        // exact bytes before removing the retained marker, rather than retrying.
        assert_eq!(
            record(&current).unwrap().sha256,
            digest(next.css.as_bytes())
        );
        assert_eq!(
            files::read(root, &record(&current).unwrap().file, LIMIT).unwrap(),
            next.css.as_bytes()
        );
        styles_output::complete(root, PRODUCTION_PENDING, &intent).unwrap();
        accept(&app, &s, &next, false).unwrap();
        no_pending(root).unwrap();
    }
    #[test]
    fn prepublication_write_failure_clears_owned_intent_without_removing_unknown_temp() {
        let (_temp, app, s) = fixture();
        let root = fs::app_root(&app);
        let old = candidate(&app, "a{}\n");
        publish(&app, &s, &old).unwrap();
        let prior = files::read(root, OWNER, 65536).unwrap();
        let name = format!("assets/generated/.yetlet-style-{}", std::process::id());
        files::write(root, &name, b"unknown").unwrap();
        let next = candidate(&app, "b{}\n");
        assert!(publish(&app, &s, &next).is_err());
        assert!(!root.join(OWNER_PENDING).exists());
        assert_eq!(
            files::read(root, &s.output, LIMIT).unwrap(),
            old.css.as_bytes()
        );
        assert_eq!(files::read(root, OWNER, 65536).unwrap(), prior);
        assert_eq!(files::read(root, &name, LIMIT).unwrap(), b"unknown");
        // Operator preserves the unknown evidence outside the output directory.
        std::fs::rename(root.join(&name), root.join("retained-unknown-temp")).unwrap();
        publish(&app, &s, &next).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn linked_output_ancestors_and_records_fail_closed() {
        use std::os::unix::fs::symlink;
        for target in ["assets", ".yetlet"] {
            let (_temp, app, s) = fixture();
            let root = fs::app_root(&app);
            std::fs::create_dir(root.join("redirect")).unwrap();
            symlink("redirect", root.join(target)).unwrap();
            assert!(publish(&app, &s, &candidate(&app, "a{}\n")).is_err());
            assert!(
                std::fs::read_dir(root.join("redirect"))
                    .unwrap()
                    .next()
                    .is_none()
            );
        }
    }
}
