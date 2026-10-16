//! Configuration-only Compose output for the accepted native cloud profile.
use crate::{docker, image};
use std::path::Path;
use toml::Value;

pub fn generate(manifest: &Path, output: &Path, image: &str) -> Result<(), String> {
    generate_inner(manifest, output, image, || {})
}

fn generate_inner(
    manifest: &Path,
    output: &Path,
    image: &str,
    after_inspection: impl FnOnce(),
) -> Result<(), String> {
    image::validate(image).map_err(str::to_owned)?;
    docker::generate_checked(
        manifest,
        output,
        |snapshot| {
            validate_lock(snapshot)?;
            Ok(vec![
                (
                    "compose.yaml",
                    include_str!("../templates/compose/compose.yaml").replace("@IMAGE@", image),
                ),
                (
                    "README.md",
                    include_str!("../templates/compose/README.md").into(),
                ),
            ])
        },
        after_inspection,
    )
}

// Static consistency for this fixed profile, never a Cargo resolver or a claim
// that arbitrary lockfiles are buildable. The operator still builds --locked.
pub(crate) fn validate_lock(snapshot: &docker::Snapshot) -> Result<(), String> {
    let invalid = || {
        "Cargo.lock is stale or unsupported for the cloud app; refresh it separately offline"
            .to_owned()
    };
    let lock: Value = std::str::from_utf8(&snapshot.lock)
        .map_err(|_| invalid())?
        .parse()
        .map_err(|_| invalid())?;
    if lock.get("version").and_then(Value::as_integer) != Some(4) {
        return Err(invalid());
    }
    let packages = lock
        .get("package")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    let one = |name: &str| -> Result<&Value, String> {
        let mut matches = packages
            .iter()
            .filter(|p| p.get("name").and_then(Value::as_str) == Some(name));
        let package = matches.next().ok_or_else(invalid)?;
        if matches.next().is_some() {
            return Err(invalid());
        }
        Ok(package)
    };
    for (name, expected) in [
        (
            snapshot.manifest["package"]["name"].as_str().unwrap(),
            &["axum", "tokio", "yetlet-core", "yetlet-server-axum"][..],
        ),
        ("yetlet-core", &[][..]),
        ("yetlet-html", &["yetlet-core"][..]),
        (
            "yetlet-server-axum",
            &["axum", "serde", "tower", "yetlet-core", "yetlet-html"][..],
        ),
    ] {
        let package = one(name)?;
        if package.get("version").and_then(Value::as_str) != Some("0.1.0")
            || package.get("source").is_some()
            || package.get("checksum").is_some()
        {
            return Err(invalid());
        }
        let actual = match package.get("dependencies") {
            Some(dependencies) => dependencies
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|d| d.as_str().ok_or_else(invalid))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        if actual != expected {
            return Err(invalid());
        }
    }
    for (name, prefix) in [
        ("axum", "0.8."),
        ("tokio", "1."),
        ("serde", "1."),
        ("tower", "0.5."),
    ] {
        let package = one(name)?;
        if !package
            .get("version")
            .and_then(Value::as_str)
            .is_some_and(|v| {
                v.starts_with(prefix)
                    && v.split('.').count() == 3
                    && v.split('.')
                        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
            })
            || package.get("source").and_then(Value::as_str)
                != Some("registry+https://github.com/rust-lang/crates.io-index")
        {
            return Err(invalid());
        }
    }
    // Reject missing dependency nodes and malformed registry entries throughout
    // the supplied lock, including the cloud runtime's Tokio signal dependency.
    for package in packages {
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        let version = package
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        if packages
            .iter()
            .filter(|p| {
                p.get("name").and_then(Value::as_str) == Some(name)
                    && p.get("version").and_then(Value::as_str) == Some(version)
            })
            .count()
            != 1
        {
            return Err(invalid());
        }
        if package.get("source").is_some()
            && (package.get("source").and_then(Value::as_str)
                != Some("registry+https://github.com/rust-lang/crates.io-index")
                || !package
                    .get("checksum")
                    .and_then(Value::as_str)
                    .is_some_and(|s| {
                        s.len() == 64
                            && s.bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    }))
        {
            return Err(invalid());
        }
        if let Some(dependencies) = package.get("dependencies") {
            for dependency in dependencies.as_array().ok_or_else(invalid)? {
                let parts = dependency
                    .as_str()
                    .ok_or_else(invalid)?
                    .split(' ')
                    .collect::<Vec<_>>();
                if parts.is_empty()
                    || parts.len() > 2
                    || packages
                        .iter()
                        .filter(|p| {
                            p.get("name").and_then(Value::as_str) == Some(parts[0])
                                && (parts.len() == 1
                                    || p.get("version").and_then(Value::as_str) == Some(parts[1]))
                        })
                        .count()
                        != 1
                {
                    return Err(invalid());
                }
            }
        }
    }
    let tokio_dependencies = one("tokio")?
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    if !tokio_dependencies
        .iter()
        .any(|v| v.as_str() == Some("signal-hook-registry"))
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
