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
            let mut yaml =
                include_str!("../templates/compose/compose.yaml").replace("@IMAGE@", image);
            let mut env = String::new();
            if snapshot.with_assets {
                env.push_str("      YETLET_ASSET_ROOT: '/opt/yetlet/assets'\n");
            }
            if snapshot.runtime_settings {
                env.push_str("      YETLET_CONFIG_ROOT: '/opt/yetlet/config'\n      APP_ENVIRONMENT: 'Production'\n      APP_SERVER__HOST: '0.0.0.0'\n      APP_SERVER__PORT: '8080'\n      APP_SERVICE__TOKEN: '${APP_SERVICE__TOKEN:?Set APP_SERVICE__TOKEN}'\n");
            }
            yaml = yaml.replace("    ports:", &format!("{env}    ports:"));
            let mut readme = include_str!("../templates/compose/README.md").to_owned();
            if snapshot.runtime_settings || snapshot.with_assets {
                readme.push_str(include_str!("../templates/docker/PROFILE-RUNTIME.md"));
            }
            Ok(vec![("compose.yaml", yaml), ("README.md", readme)])
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
    let mut app_dependencies = vec!["axum", "tokio", "yetlet-core", "yetlet-server-axum"];
    if snapshot.with_assets {
        app_dependencies.extend(["serde", "serde_json", "tempfile", "tower"]);
    }
    if snapshot.runtime_settings {
        app_dependencies.extend(["secrecy", "serde", "yetlet-config"]);
    }
    // Source inspection already requires the exact generated manifest. Cargo
    // records its optional TLS dependencies even when the feature is disabled.
    if snapshot
        .manifest
        .get("features")
        .and_then(|features| features.get("yetlet-dev-tls"))
        .is_some()
    {
        app_dependencies.extend(["axum-server", "rustls"]);
    }
    app_dependencies.sort();
    app_dependencies.dedup();
    for (name, expected) in [
        (
            snapshot.manifest["package"]["name"].as_str().unwrap(),
            app_dependencies.as_slice(),
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
    if snapshot.runtime_settings {
        let package = one("yetlet-config")?;
        if package.get("version").and_then(Value::as_str) != Some("0.0.0")
            || package.get("source").is_some()
            || package.get("checksum").is_some()
            || package
                .get("dependencies")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                != Some(vec!["dotenvy", "libc", "serde", "toml"])
        {
            return Err(invalid());
        }
    }
    for (name, version, enabled) in [
        (
            "serde",
            "1.0.229",
            snapshot.with_assets || snapshot.runtime_settings,
        ),
        ("serde_json", "1.0.149", snapshot.with_assets),
        ("secrecy", "0.10.3", snapshot.runtime_settings),
        ("toml", "0.9.12+spec-1.1.0", snapshot.runtime_settings),
        ("dotenvy", "0.15.7", snapshot.runtime_settings),
    ] {
        if enabled && one(name)?.get("version").and_then(Value::as_str) != Some(version) {
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
        if (snapshot.with_assets || snapshot.runtime_settings)
            && package.get("source").is_none()
            && ![
                snapshot.manifest["package"]["name"].as_str().unwrap(),
                "yetlet-core",
                "yetlet-html",
                "yetlet-server-axum",
            ]
            .contains(&name)
            && !(snapshot.runtime_settings && name == "yetlet-config")
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
