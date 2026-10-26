//! Bounded generated cloud delivery profiles. No frontend execution or source export.
use super::{Snapshot, fs, read_framework};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::SystemTime,
};

pub(super) fn dependencies(snapshot: &Snapshot) -> Vec<&'static str> {
    let mut names = vec!["yetlet-core", "yetlet-server-axum"];
    if snapshot.runtime_settings {
        names.push("yetlet-config");
    }
    names
}

pub(super) fn config_framework(
    root: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    for (directory, sources) in [
        ("crates/yetlet-config", &["lib.rs", "secrets.rs"][..]),
        ("crates/yetlet-secrets", &["lib.rs"][..]),
    ] {
        if super::directory_names(&root.join(directory).join("src"))? != sources
            || super::directory_names(&root.join(directory))?
                .iter()
                .any(|n| matches!(n.as_str(), "build.rs" | ".cargo"))
        {
            return Err("unsupported config/secrets crate layout".into());
        }
    }
    for (path, expected) in [
        (
            "crates/yetlet-config/Cargo.toml",
            include_bytes!("../../../../crates/yetlet-config/Cargo.toml").as_slice(),
        ),
        (
            "crates/yetlet-config/src/lib.rs",
            include_bytes!("../../../../crates/yetlet-config/src/lib.rs").as_slice(),
        ),
        (
            "crates/yetlet-config/src/secrets.rs",
            include_bytes!("../../../../crates/yetlet-config/src/secrets.rs").as_slice(),
        ),
        // Cargo requires this explicitly declared target even when the
        // optional provider feature and example are not being compiled.
        (
            "crates/yetlet-config/examples/secret_provider.rs",
            include_bytes!("../../../../crates/yetlet-config/examples/secret_provider.rs")
                .as_slice(),
        ),
        (
            "crates/yetlet-secrets/Cargo.toml",
            include_bytes!("../../../../crates/yetlet-secrets/Cargo.toml").as_slice(),
        ),
        (
            "crates/yetlet-secrets/src/lib.rs",
            include_bytes!("../../../../crates/yetlet-secrets/src/lib.rs").as_slice(),
        ),
    ] {
        let bytes = read_framework(root, path)?;
        if super::normalize(&bytes)? != super::normalize(expected)? {
            return Err("unsupported config crate source or dependencies".into());
        }
        files.insert(path.into(), bytes);
    }
    Ok(())
}

fn safe(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'-' | b'_'))
        && value.split('/').all(|p| !matches!(p, "" | "." | ".."))
}
fn strings(value: Option<&Value>) -> Result<Vec<&str>, String> {
    match value {
        None => Ok(vec![]),
        Some(v) => v
            .as_array()
            .ok_or("invalid Vite references")?
            .iter()
            .map(|v| {
                v.as_str()
                    .filter(|v| safe(v))
                    .ok_or("unsafe Vite reference".into())
            })
            .collect(),
    }
}
fn asset(path: &str) -> Result<(), String> {
    let file = path
        .strip_prefix("assets/")
        .ok_or("only flat generated assets are supported")?;
    if !safe(file)
        || file.contains('/')
        || file.starts_with('.')
        || !["js", "css", "svg"].contains(&file.rsplit('.').next().unwrap_or(""))
    {
        return Err("unsupported built asset path".into());
    }
    Ok(())
}
fn visit<'a>(
    key: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    active: &mut BTreeSet<&'a str>,
    done: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    if done.contains(key) {
        return Ok(());
    }
    if !active.insert(key) {
        return Err("Vite graph cycle".into());
    }
    for target in edges.get(key).ok_or("missing Vite graph reference")? {
        visit(target, edges, active, done)?;
    }
    active.remove(key);
    done.insert(key);
    Ok(())
}

pub(super) fn assets(
    app: &fs::App,
    newest_input: SystemTime,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if fs::app_directory_entries(app, "dist")? != [".vite", "assets"]
        || fs::app_directory_entries(app, "dist/.vite")? != ["manifest.json"]
    {
        return Err("dist must contain only the generated manifest and assets".into());
    }
    let (manifest, built) = fs::read_app_input(app, "dist/.vite/manifest.json")?;
    if manifest.len() > 128 * 1024 || built < newest_input {
        return Err(
            "Vite manifest missing, oversized or stale; run npm ci and npm run build explicitly"
                .into(),
        );
    }
    let value: Value = serde_json::from_slice(&manifest).map_err(|_| "invalid Vite manifest")?;
    let chunks = value.as_object().ok_or("invalid Vite manifest")?;
    if chunks.is_empty() || chunks.len() > 128 {
        return Err("invalid Vite record count".into());
    }
    if chunks
        .get("app.ts")
        .and_then(|v| v.get("isEntry"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("Vite app entry absent".into());
    }
    let mut files = BTreeSet::new();
    let mut edges = BTreeMap::new();
    let mut edge_count = 0;
    for (key, chunk) in chunks {
        if !safe(key) {
            return Err("unsafe Vite key".into());
        }
        let chunk = chunk.as_object().ok_or("invalid Vite record")?;
        if chunk.keys().any(|k| {
            ![
                "file",
                "src",
                "name",
                "names",
                "isEntry",
                "isDynamicEntry",
                "imports",
                "dynamicImports",
                "css",
                "assets",
            ]
            .contains(&k.as_str())
        }) {
            return Err("unsupported Vite field".into());
        }
        for name in ["src", "name"] {
            if chunk
                .get(name)
                .is_some_and(|v| v.as_str().is_none_or(|s| !safe(s)))
            {
                return Err("unsafe Vite metadata".into());
            }
        }
        if strings(chunk.get("names"))?.len() > 128 {
            return Err("too many Vite names".into());
        }
        let file = chunk
            .get("file")
            .and_then(Value::as_str)
            .ok_or("missing Vite file")?;
        asset(file)?;
        for flag in ["isEntry", "isDynamicEntry"] {
            if let Some(value) = chunk.get(flag) {
                let selected = value.as_bool().ok_or("invalid Vite entry flag")?;
                if selected && !file.ends_with(".js") {
                    return Err("invalid Vite script entry".into());
                }
            }
        }
        files.insert(file.to_owned());
        for field in ["css", "assets"] {
            let paths = strings(chunk.get(field))?;
            edge_count += paths.len();
            for path in paths {
                asset(path)?;
                if field == "css" && !path.ends_with(".css") {
                    return Err("invalid Vite stylesheet".into());
                }
                files.insert(path.to_owned());
            }
        }
        let imports = strings(chunk.get("imports"))?
            .into_iter()
            .chain(strings(chunk.get("dynamicImports"))?)
            .collect::<Vec<_>>();
        edge_count += imports.len();
        for target in &imports {
            if !chunks
                .get(*target)
                .and_then(|v| v.get("file"))
                .and_then(Value::as_str)
                .is_some_and(|s| s.ends_with(".js"))
            {
                return Err("missing or non-script Vite reference".into());
            }
        }
        edges.insert(key.as_str(), imports);
        if files.len() > 256 || edge_count > 512 {
            return Err("Vite graph exceeds limits".into());
        }
    }
    let mut done = BTreeSet::new();
    for key in edges.keys() {
        visit(key, &edges, &mut BTreeSet::new(), &mut done)?;
    }
    let physical = fs::app_directory_entries(app, "dist/assets")?;
    if physical
        != files
            .iter()
            .map(|s| s.strip_prefix("assets/").unwrap().to_owned())
            .collect::<Vec<_>>()
    {
        return Err(
            "dist assets must exactly match the manifest; extra files/source maps are refused"
                .into(),
        );
    }
    let mut output = BTreeMap::new();
    let mut total = manifest.len();
    for path in files {
        let name = format!("dist/{path}");
        let (bytes, modified) = fs::read_app_input(app, &name)?;
        if modified < newest_input {
            return Err("stale Vite asset; rebuild explicitly".into());
        }
        total += bytes.len();
        if total > 16 * 1024 * 1024 {
            return Err("Vite build exceeds total limit".into());
        }
        output.insert(name, bytes);
    }
    output.insert("dist/.vite/manifest.json".into(), manifest);
    Ok(output)
}

pub(super) fn dockerfile(snapshot: &Snapshot, name: &str) -> String {
    let mut text = include_str!("../../templates/docker/Dockerfile").replace("@NAME@", name);
    for source in ["config", "vite"] {
        if if source == "config" {
            snapshot.runtime_settings
        } else {
            snapshot.with_assets
        } {
            text = text.replace(
                "COPY cargo-overlay/",
                &format!(
                    "COPY --from=app-src /src/{source}.rs ./src/{source}.rs\nCOPY cargo-overlay/"
                ),
            );
        }
    }
    let mut runtime = String::new();
    if snapshot.with_assets {
        runtime.push_str("COPY --from=app-src /dist/ /opt/yetlet/assets/\nENV YETLET_ASSET_ROOT=/opt/yetlet/assets\n");
    }
    if snapshot.with_public_settings {
        for file in [
            "base.toml",
            "development.toml",
            "staging.toml",
            "production.toml",
        ] {
            runtime.push_str(&format!(
                "COPY --from=app-src /{file} /opt/yetlet/config/{file}\n"
            ));
        }
    }
    if snapshot.runtime_settings {
        runtime.push_str("ENV YETLET_CONFIG_ROOT=/opt/yetlet/config APP_ENVIRONMENT=Production APP_SERVER__HOST=0.0.0.0 APP_SERVER__PORT=8080\n");
    }
    // Sanitized private exports may use 0600 files and 0700 directories.
    // Only the selected immutable public build assets/configuration become
    // readable by the image's unprivileged runtime identity.
    if snapshot.with_assets || snapshot.with_public_settings {
        let public_roots = match (snapshot.with_assets, snapshot.with_public_settings) {
            (true, true) => "/opt/yetlet/assets /opt/yetlet/config",
            (true, false) => "/opt/yetlet/assets",
            (false, true) => "/opt/yetlet/config",
            (false, false) => unreachable!(),
        };
        runtime.push_str(&format!("RUN chmod -R a+rX {public_roots}\n"));
    }
    text = text.replace("USER 65532:65532", &format!("{runtime}USER 65532:65532"));
    text
}
