use std::{
    io,
    path::{Component, Path, PathBuf},
};

/// Preserve the selected spelling, reject traversal and symlinks instead of
/// canonicalizing through them. Linux reads additionally use directory handles.
pub(crate) fn absolute(path: &Path) -> io::Result<PathBuf> {
    if path.as_os_str().is_empty() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(io::Error::other(
            "empty path or parent traversal is forbidden",
        ));
    }
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(path.components().collect())
}

pub(crate) fn validate(path: &Path) -> Result<PathBuf, &'static str> {
    let path = absolute(path).map_err(|_| "framework path must not contain parent traversal")?;
    let root = manifest(&path.join("Cargo.toml"))?;
    let workspace = root
        .get("workspace")
        .ok_or("framework workspace manifest is missing")?;
    if workspace
        .get("package")
        .and_then(|p| p.get("rust-version"))
        .and_then(toml::Value::as_str)
        != Some("1.99")
    {
        return Err("framework must declare the expected Rust 1.99 baseline");
    }
    for name in ["yetlet"] {
        let crate_dir = path.join("crates").join(name);
        let value = manifest(&crate_dir.join("Cargo.toml"))?;
        if value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            != Some(name)
        {
            return Err("framework crate package identity is invalid");
        }
        read(&crate_dir.join("src/lib.rs"))
            .map_err(|_| "framework crate source is unavailable or unsafe")?;
    }
    Ok(path)
}

fn manifest(path: &Path) -> Result<toml::Value, &'static str> {
    let data = read(path).map_err(|_| "framework manifest is unavailable or unsafe")?;
    std::str::from_utf8(&data)
        .map_err(|_| "framework manifest must be UTF-8")?
        .parse()
        .map_err(|_| "framework manifest is invalid TOML")
}

fn read(path: &Path) -> io::Result<Vec<u8>> {
    crate::platform::fs::read_framework(path)
}

// Only the opted-in profile inspects this additional framework package.
pub(crate) fn validate_config(root: &Path) -> Result<(), &'static str> {
    let dir = root.join("crates/yetlet");
    let value = manifest(&dir.join("Cargo.toml"))?;
    if value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        != Some("yetlet")
    {
        return Err("framework configuration package identity is invalid");
    }
    if value
        .get("features")
        .and_then(|f| f.get("config"))
        .is_none()
    {
        return Err("framework configuration feature is unavailable");
    }
    read(&dir.join("src/config/mod.rs"))
        .map_err(|_| "framework configuration source is unavailable or unsafe")?;
    Ok(())
}

pub(crate) fn workspace_baseline() -> toml::Value {
    include_str!("../resources/workspace.toml")
        .parse::<toml::Value>()
        .expect("compiled framework workspace manifest")
}

pub(crate) fn package_version() -> String {
    workspace_baseline()["workspace"]["package"]["version"]
        .as_str()
        .expect("compiled framework version")
        .to_owned()
}
