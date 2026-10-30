//! Complete repository-backed sample catalog validation, including source identity.
use crate::tooling::*;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use toml::Value;
const BROWSER_DEPS: &[&str] = &[
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "web-sys",
    "js-sys",
    "yetlet-browser",
];
fn string(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str)
}
fn strings(v: Option<&Value>) -> Option<Vec<String>> {
    v?.as_array()?
        .iter()
        .map(|v| v.as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .collect()
}
fn load(root: &Path, path: impl AsRef<Path>) -> Result<Value> {
    text(root, path)?
        .parse()
        .map_err(|_| fail("invalid catalog/manifest TOML"))
}
fn contained(
    root: &Path,
    v: Option<&Value>,
    label: &str,
    directory: bool,
    errors: &mut Vec<String>,
) -> Option<PathBuf> {
    let Some(name) = string(v) else {
        errors.push(format!(
            "{label} must be a non-empty repository-relative path"
        ));
        return None;
    };
    let path = match safe(root, name, false) {
        Ok(path) => path,
        Err(e) => {
            errors.push(format!("{label}: {}", e.message));
            return None;
        }
    };
    if (directory && !path.is_dir()) || (!directory && !path.is_file()) {
        errors.push(format!("missing {label}: {name}"));
        return None;
    }
    match path.canonicalize() {
        Ok(path) => Some(path),
        Err(_) => {
            errors.push(format!("unresolved {label}"));
            None
        }
    }
}
pub fn dependency_names(manifest: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut add = |m: &Value| {
        for field in ["dependencies", "build-dependencies"] {
            if let Some(table) = m.get(field).and_then(Value::as_table) {
                names.extend(table.keys().cloned());
            }
        }
    };
    add(manifest);
    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            add(target);
        }
    }
    names
}
pub fn route_is_documented(readme: &str, path: &str) -> bool {
    if Regex::new(r"`(/[^`\s]*)`")
        .unwrap()
        .captures_iter(readme)
        .any(|m| &m[1] == path)
    {
        return true;
    }
    Regex::new(r"https?://[^\s`)>]+")
        .unwrap()
        .find_iter(readme)
        .any(|m| {
            let after = m.as_str().split_once("://").unwrap().1;
            let p = after.find('/').map_or("", |i| &after[i..]);
            p.split(['?', '#']).next().unwrap() == path
        })
}
fn route(
    root: &Path,
    value: &Value,
    label: &str,
    directory: &Path,
    readme: &str,
    errors: &mut Vec<String>,
) -> Result<()> {
    let Some(table) = value.as_table() else {
        errors.push(format!("{label} route must be an inline table"));
        return Ok(());
    };
    if table.len() != 3
        || ["path", "source", "markers"]
            .iter()
            .any(|k| !table.contains_key(*k))
    {
        errors.push(format!(
            "{label} route fields must be exactly: markers, path, source"
        ));
        return Ok(());
    }
    let Some(path) = string(value.get("path"))
        .filter(|s| s.starts_with('/') && !s.chars().any(char::is_whitespace))
    else {
        errors.push(format!("{label} route path must be an absolute path"));
        return Ok(());
    };
    if !route_is_documented(readme, path) {
        errors.push(format!(
            "{label} route is not documented by its sample README: {path}"
        ));
    }
    let Some(source) = contained(
        root,
        value.get("source"),
        &format!("{label} route source"),
        false,
        errors,
    ) else {
        return Ok(());
    };
    if !source.starts_with(directory) {
        errors.push(format!(
            "{label} route source must be inside the sample directory"
        ));
        return Ok(());
    }
    let Some(markers) = strings(value.get("markers")).filter(|v| !v.is_empty()) else {
        errors.push(format!(
            "{label} route markers must be a non-empty string list"
        ));
        return Ok(());
    };
    let source_text = std::fs::read_to_string(source)?;
    for marker in markers {
        if !source_text.contains(&marker) {
            errors.push(format!("{label} route marker not found: {marker:?}"));
        }
    }
    Ok(())
}
pub fn check(root: &Path) -> Result<Vec<String>> {
    check_data(root, &load(root, "examples/samples.toml")?)
}
pub fn check_data(root: &Path, data: &Value) -> Result<Vec<String>> {
    let root = root.canonicalize()?;
    let root = root.as_path();
    let mut errors = Vec::new();
    if data.get("schema_version").and_then(Value::as_integer) != Some(1) {
        errors.push("samples catalog schema_version must be integer 1".into());
    }
    let Some(samples) = data
        .get("sample")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty())
    else {
        errors.push("samples catalog must contain at least one [[sample]]".into());
        return Ok(errors);
    };
    let workspace = load(root, "Cargo.toml")?;
    let raw_members = workspace
        .get("workspace")
        .and_then(|v| v.get("members"))
        .and_then(Value::as_array);
    let mut members = BTreeSet::new();
    let mut packages = BTreeMap::new();
    if let Some(raw) = raw_members {
        members.extend(raw.iter().filter_map(Value::as_str).map(str::to_owned));
    } else {
        errors.push("workspace members must be a list".into());
    }
    for member in &members {
        let Some(path) = contained(
            root,
            Some(&Value::String(format!("{member}/Cargo.toml"))),
            "workspace member manifest",
            false,
            &mut errors,
        ) else {
            continue;
        };
        let manifest: Value = std::fs::read_to_string(path)?
            .parse()
            .map_err(|_| fail("invalid workspace manifest TOML"))?;
        let Some(name) = manifest
            .get("package")
            .and_then(|v| v.get("name"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        else {
            errors.push(format!("workspace member has no package name: {member}"));
            continue;
        };
        if packages.contains_key(name) {
            errors.push(format!("duplicate workspace package name: {name}"));
            continue;
        }
        packages.insert(name.to_owned(), (member.clone(), manifest));
    }
    let mut ids = BTreeSet::new();
    let mut sample_packages = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let id_re = Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    for (index, sample) in samples.iter().enumerate() {
        let label = format!("sample #{}", index + 1);
        let Some(table) = sample.as_table() else {
            errors.push(format!("{label} must be a table"));
            continue;
        };
        let required = [
            "id",
            "title",
            "package",
            "path",
            "kind",
            "status",
            "command",
            "companion_packages",
            "routes",
            "browser_runtime",
            "browser_packages",
            "browser_build",
            "optional_dependencies",
            "evidence",
        ];
        let missing = required
            .iter()
            .filter(|k| !table.contains_key(**k))
            .collect::<Vec<_>>();
        let extra = table
            .keys()
            .filter(|k| !required.contains(&k.as_str()))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!("{label} missing fields: {missing:?}"));
        }
        if !extra.is_empty() {
            errors.push(format!("{label} unsupported fields: {extra:?}"));
        }
        if !missing.is_empty() || !extra.is_empty() {
            continue;
        }
        let id = string(sample.get("id"));
        match id {
            Some(s) if id_re.is_match(s) => {
                if !ids.insert(s) {
                    errors.push(format!("duplicate sample id: {s}"));
                }
            }
            _ => errors.push(format!("{label} has invalid id")),
        }
        let package = string(sample.get("package")).filter(|s| !s.is_empty());
        match package {
            Some(s) => {
                if !sample_packages.insert(s) {
                    errors.push(format!("duplicate sample package: {s}"));
                }
            }
            None => errors.push(format!("{label} has invalid package")),
        }
        let Some(directory) = contained(
            root,
            sample.get("path"),
            &format!("{label} path"),
            true,
            &mut errors,
        ) else {
            continue;
        };
        let relative = directory
            .strip_prefix(root)
            .map_err(|_| fail("sample root escape"))?
            .to_string_lossy()
            .replace('\\', "/");
        if !paths.insert(relative.clone()) {
            errors.push(format!("duplicate sample path: {relative}"));
        }
        if !members.contains(&relative) {
            errors.push(format!("sample path is not a workspace member: {relative}"));
        }
        let Some(manifest_path) = contained(
            root,
            Some(&Value::String(format!("{relative}/Cargo.toml"))),
            "sample manifest",
            false,
            &mut errors,
        ) else {
            continue;
        };
        let manifest: Value = std::fs::read_to_string(manifest_path)?
            .parse()
            .map_err(|_| fail("invalid sample manifest TOML"))?;
        if manifest.get("package").and_then(|p| p.get("name")) != sample.get("package") {
            errors.push(format!("sample package mismatch for {relative}"));
        }
        if package.is_some_and(|p| packages.get(p).is_none_or(|(m, _)| m != &relative)) {
            errors.push(format!(
                "sample package is not resolved by the workspace: {package:?}"
            ));
        }
        let readme = if let Some(path) = contained(
            root,
            Some(&Value::String(format!("{relative}/README.md"))),
            "sample README",
            false,
            &mut errors,
        ) {
            std::fs::read_to_string(path)?
        } else {
            String::new()
        };
        let kind = string(sample.get("kind"));
        if !kind.is_some_and(|s| ["api", "full-stack", "server-rendered"].contains(&s)) {
            errors.push(format!("{label} has unsupported kind"));
        }
        if string(sample.get("status")) != Some("verified") {
            errors.push(format!("{label} has unsupported status"));
        }
        if string(sample.get("title")).is_none_or(|s| s.trim().is_empty()) {
            errors.push(format!("{label} has empty title"));
        }
        match string(sample.get("command")).filter(|s| !s.trim().is_empty()) {
            None => errors.push(format!("{label} has empty command")),
            Some(command) => {
                if package.is_some_and(|p| !command.contains(&format!("cargo run -p {p}"))) {
                    errors.push(format!(
                        "{label} command does not launch package {package:?}"
                    ));
                }
                if !readme.lines().any(|l| l.trim() == command) {
                    errors.push(format!(
                        "sample README does not contain the exact catalog command: {relative}"
                    ));
                }
            }
        }
        let companions = match strings(sample.get("companion_packages")) {
            Some(v) => v,
            None => {
                errors.push(format!("{label} companion_packages must be a string list"));
                Vec::new()
            }
        };
        if companions.iter().collect::<BTreeSet<_>>().len() != companions.len()
            || package.is_some_and(|p| companions.iter().any(|s| s == p))
        {
            errors.push(format!(
                "{label} companion_packages must be unique and exclude the primary package"
            ));
        }
        let mut related = package.into_iter().map(str::to_owned).collect::<Vec<_>>();
        for name in &companions {
            if !packages.contains_key(name) {
                errors.push(format!(
                    "{label} companion package is not in the workspace: {name}"
                ));
            } else {
                related.push(name.clone());
            }
        }
        if let Some(routes) = sample
            .get("routes")
            .and_then(Value::as_array)
            .filter(|v| !v.is_empty())
        {
            let mut seen = BTreeSet::new();
            for (i, r) in routes.iter().enumerate() {
                if let Some(path) = string(r.get("path"))
                    && !seen.insert(path)
                {
                    errors.push(format!("{label} has duplicate route: {path}"));
                }
                route(
                    root,
                    r,
                    &format!("{label} route #{}", i + 1),
                    &directory,
                    &readme,
                    &mut errors,
                )?;
            }
        } else {
            errors.push(format!("{label} routes must be a non-empty list"));
        }
        let dependency_map: BTreeMap<_, _> = related
            .iter()
            .filter_map(|n| {
                packages
                    .get(n)
                    .map(|(_, m)| (n.clone(), dependency_names(m)))
            })
            .collect();
        let deps: BTreeSet<_> = dependency_map.values().flatten().cloned().collect();
        let runtime = sample.get("browser_runtime").and_then(Value::as_bool);
        if runtime.is_none() {
            errors.push(format!("{label} browser_runtime must be boolean"));
        }
        let browser = match strings(sample.get("browser_packages")) {
            Some(v) => v,
            None => {
                errors.push(format!("{label} browser_packages must be a string list"));
                Vec::new()
            }
        };
        if browser.iter().collect::<BTreeSet<_>>().len() != browser.len() {
            errors.push(format!("{label} browser_packages must be unique"));
        }
        let build = string(sample.get("browser_build"));
        if kind == Some("api") {
            if runtime != Some(false) {
                errors.push(format!("{label} kind=api requires browser_runtime=false"));
            }
            if !browser.is_empty() {
                errors.push(format!("{label} kind=api requires empty browser_packages"));
            }
            if build != Some("") {
                errors.push(format!("{label} kind=api requires empty browser_build"));
            }
            for dep in BROWSER_DEPS {
                if deps.contains(*dep) {
                    errors.push(format!("{label} kind=api has browser dependencies: {dep}"));
                }
            }
        }
        if runtime == Some(true) {
            if browser.is_empty() {
                errors.push(format!("{label} browser runtime requires browser_packages"));
            }
            for name in &browser {
                if !companions.contains(name) {
                    errors.push(format!(
                        "{label} browser package must be a declared companion: {name}"
                    ));
                } else if dependency_map
                    .get(name)
                    .is_none_or(|d| !d.contains("wasm-bindgen"))
                {
                    errors.push(format!(
                        "{label} browser package lacks wasm-bindgen: {name}"
                    ));
                }
            }
            match build {
                Some("cargo xtask build browser")
                | Some("cargo xtask build multi-island-dashboard") => {
                    let expected = if build == Some("cargo xtask build browser") {
                        ["yetlet-todo-browser", "yetlet-search-island"]
                    } else {
                        [
                            "yetlet-dashboard-counter-island",
                            "yetlet-dashboard-filter-island",
                        ]
                    };
                    let implementation = text(root, "xtask/src/build.rs")?;
                    let dispatch = text(root, "xtask/src/lib.rs")?;
                    if !dispatch.contains("Build::parse") {
                        errors.push(format!("{label} Rust build dispatch missing"));
                    }
                    for name in &browser {
                        if !expected.contains(&name.as_str())
                            || !implementation.contains(&format!("\"{name}\""))
                        {
                            errors.push(format!(
                                "{label} browser build does not reference package: {name}"
                            ));
                        }
                    }
                    if !root.join("xtask/Cargo.toml").is_file()
                        || !root.join(".cargo/config.toml").is_file()
                    {
                        errors.push(format!("{label} Rust build command is unavailable"));
                    }
                    if !string(sample.get("command")).is_some_and(|s| s.starts_with(build.unwrap()))
                    {
                        errors.push(format!("{label} command/build identity mismatch"));
                    }
                }
                _ => {
                    if let Some(path) = contained(
                        root,
                        sample.get("browser_build"),
                        &format!("{label} browser build"),
                        false,
                        &mut errors,
                    ) {
                        let source = std::fs::read_to_string(path)?;
                        for name in &browser {
                            if !source.contains(name) {
                                errors.push(format!(
                                    "{label} browser build does not reference package: {name}"
                                ));
                            }
                        }
                    }
                }
            }
        } else if runtime == Some(false) {
            if !browser.is_empty() {
                errors.push(format!(
                    "{label} browser_runtime=false requires empty browser_packages"
                ));
            }
            if build != Some("") {
                errors.push(format!(
                    "{label} browser_runtime=false requires empty browser_build"
                ));
            }
            for dep in BROWSER_DEPS {
                if deps.contains(*dep) {
                    errors.push(format!(
                        "{label} API/server sample has browser dependencies: {dep}"
                    ));
                }
            }
        }
        match strings(sample.get("optional_dependencies")) {
            Some(names)
                if names
                    .iter()
                    .all(|n| n == "yetlet" || n.starts_with("yetlet-")) =>
            {
                if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
                    errors.push(format!("{label} optional_dependencies must be unique"));
                }
                for name in names {
                    if !packages.contains_key(&name) {
                        errors.push(format!(
                            "{label} optional dependency is not a workspace package: {name}"
                        ));
                    }
                    if !deps.contains(&name) {
                        errors.push(format!("{label} optional dependency is not direct in primary/companion manifests: {name}"));
                    }
                }
            }
            _ => errors.push(format!(
                "{label} optional_dependencies must contain Yetlet package names"
            )),
        }
        contained(
            root,
            sample.get("evidence"),
            &format!("{label} evidence"),
            false,
            &mut errors,
        );
    }
    println!("Samples: {}", samples.len());
    Ok(errors)
}
