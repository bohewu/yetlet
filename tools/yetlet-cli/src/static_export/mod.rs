//! Opt-in finite static HTML build; no router discovery or frontend pipeline.
mod protocol;
#[cfg(test)]
mod tests;
use crate::{
    app::{Completion, Failure},
    platform::{
        fs::{self, static_output as files},
        process,
    },
    settings,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn fail(message: impl Into<String>) -> Failure {
    Failure {
        code: 1,
        message: message.into(),
    }
}
pub(crate) fn valid_bin(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
pub(crate) fn valid_base(value: &str) -> bool {
    protocol::base(value)
}
fn command(
    bin: &str,
    release: bool,
    export: &Path,
    base: &str,
    islands: bool,
    cargo: &files::CargoDirectories,
) -> std::process::Command {
    let mut cmd = crate::app::command(true, release);
    // --target-dir overrides config and both target-dir environment spellings.
    // The explicit build-dir environment overrides config's intermediate path.
    // Both selections are exactly the values checked before any child starts.
    cmd.arg("--target-dir")
        .arg(&cargo.target)
        .env("CARGO_BUILD_BUILD_DIR", &cargo.build);
    cmd.args([
        "--bin",
        bin,
        "--",
        "--yetlet-static-export",
        "1",
        "--output",
    ])
    .arg(export)
    .args(["--base-path", base]);
    if islands {
        cmd.arg("--wasm-islands");
    }
    for (key, _) in std::env::vars_os() {
        let upper = key.to_string_lossy().to_ascii_uppercase();
        if upper.starts_with("APP_")
            || matches!(
                upper.as_str(),
                "YETLET_DEV_SECRETS"
                    | "YETLET_DEVELOPMENT_SECRET_FILE"
                    | "YETLET_CONFIG_ROOT"
                    | "YETLET_DEV_TLS_CERT"
                    | "YETLET_DEV_TLS_KEY"
                    | "YETLET_BROWSER_WATCH"
            )
        {
            cmd.env_remove(key);
        }
    }
    cmd
}
pub(crate) fn execute(
    manifest: &Path,
    bin: &str,
    output: &Path,
    base: &str,
    release: bool,
    islands: bool,
) -> Result<Completion, Failure> {
    if !valid_bin(bin) || !valid_base(base) {
        return Err(fail("invalid static export binary or base path"));
    }
    let app = fs::open_app(manifest).map_err(fail)?;
    let (settings, profile) = settings::load_selected(&app, std::env::vars_os()).map_err(fail)?;
    if settings.target != settings::Target::Native {
        return Err(fail("static export requires cli.target=native"));
    }
    let cargo = files::cargo_directories(&app).map_err(fail)?;
    let (parent, destination) = files::output_path(&app, output, &cargo).map_err(fail)?;
    let before = files::source_snapshot(fs::app_root(&app)).map_err(fail)?;
    let manifest_data = files::read(fs::app_root(&app), "Cargo.toml", 1024 * 1024).map_err(fail)?;
    let package: toml::Value = std::str::from_utf8(&manifest_data)
        .map_err(|_| fail("static manifest must be UTF-8"))?
        .parse()
        .map_err(|_| fail("invalid static package metadata"))?;
    let target = files::ensure(&app, "target".as_ref()).map_err(fail)?;
    let receipts = files::ensure(&target, "yetlet-static".as_ref()).map_err(fail)?;
    let run = format!(
        "run-{:x}-{:x}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| fail("static clock unavailable"))?
            .as_nanos()
    );
    let work = files::new_directory(&receipts, &run).map_err(fail)?;
    let export = files::new_directory(&work, "export").map_err(fail)?;
    let export_path = fs::app_root(&export).to_owned();
    drop(export);
    let mut public_stage = None;
    let mut published = false;
    let mut process_cleanup_verified = false;
    let mut receipt = json!({"schema_version":1,"protocol_version":1,"cli_version":env!("CARGO_PKG_VERSION"),"profile":profile,"cargo_profile":if release {"release"}else{"dev"},"export_bin":bin,"base_path":base,"wasm_islands":islands,"package":{"name":package.get("package").and_then(|p|p.get("name")),"version":package.get("package").and_then(|p|p.get("version"))},"inputs":before,"phase":"prepared","published":false});
    receipt["cargo_directories"] = json!({"target":cargo.target.to_string_lossy(),"build":cargo.build.to_string_lossy(),"selection":"explicit child binding; native path display"});
    let result = (|| -> Result<Completion, Failure> {
        let mut cmd = command(bin, release, &export_path, base, islands, &cargo);
        let completion = match process::run_static(&app, &mut cmd, Duration::from_secs(600)) {
            Ok(completion) => {
                process_cleanup_verified = true;
                completion
            }
            Err(error) => {
                receipt["child"] = json!({"supervision_error": format!("{:?}",error.kind()), "outcome_known": false});
                return Err(Failure {
                    code: 125,
                    message: format!(
                        "static child supervision failed ({:?}); private export retained; cleanup not claimed",
                        error.kind()
                    ),
                });
            }
        };
        receipt["child"] = json!(match &completion {
            Completion::Exited(s) => format!("{s:?}"),
            Completion::Cancelled(s) => format!("cancelled:{s}"),
            _ => "unexpected completion".into(),
        });
        if !matches!(&completion,Completion::Exited(status) if status.success()) {
            receipt["phase"] = json!("child_failed");
            return Ok(completion);
        }
        if files::source_snapshot(fs::app_root(&app)).map_err(fail)? != before {
            return Err(fail("static source/manifest/lock changed during export"));
        }
        let prepared = protocol::prepare(&export_path, base, islands).map_err(fail)?;
        let hashes: BTreeMap<_, _> = prepared
            .files
            .iter()
            .map(|(p, b)| {
                (
                    p.clone(),
                    json!({"size":b.len(),"sha256":format!("{:x}",Sha256::digest(b))}),
                )
            })
            .collect();
        let stage =
            files::new_directory(&parent, &format!(".yetlet-static-{run}")).map_err(fail)?;
        public_stage = Some(fs::app_root(&stage).to_owned());
        for (path, bytes) in prepared.files {
            files::write(fs::app_root(&stage), &path, &bytes).map_err(fail)?;
        }
        // Independently read final staging bytes through protected handles.
        for (path, value) in &hashes {
            let bytes = files::read(fs::app_root(&stage), path, 16 * 1024 * 1024).map_err(fail)?;
            if json!(bytes.len()) != value["size"]
                || json!(format!("{:x}", Sha256::digest(bytes))) != value["sha256"]
            {
                return Err(fail("static staged output changed"));
            }
        }
        receipt["routes"] = json!(prepared.routes);
        receipt["outputs"] = json!(hashes);
        receipt["phase"] = json!("ready_to_publish");
        files::write(
            fs::app_root(&work),
            "receipt-prepared.json",
            &serde_json::to_vec_pretty(&receipt)
                .map_err(|_| fail("static receipt encoding failed"))?,
        )
        .map_err(fail)?;
        files::publish(
            stage,
            &parent,
            destination
                .file_name()
                .ok_or_else(|| fail("static output has no name"))?,
        )
        .map_err(fail)?;
        published = true;
        receipt["phase"] = json!("published");
        Ok(completion)
    })();
    // A supervision error is not proof of descendant retirement. Retain the
    // private export rather than deleting files that an uncertain child may use.
    let mut clean = process_cleanup_verified && files::cleanup(&export_path).is_ok();
    if let Some(stage) = public_stage
        && !published
        && std::fs::symlink_metadata(&stage).is_ok()
    {
        clean &= files::cleanup(&stage).is_ok();
    }
    receipt["published"] = json!(published);
    receipt["process_cleanup_verified"] = json!(process_cleanup_verified);
    receipt["output_present"] = json!(std::fs::symlink_metadata(&destination).is_ok());
    receipt["temporary_cleanup_complete"] = json!(clean);
    if let Err(error) = &result {
        receipt["phase"] = json!("failed");
        receipt["failure_code"] = json!(error.code);
    }
    let saved = serde_json::to_vec_pretty(&receipt)
        .ok()
        .is_some_and(|bytes| files::write(fs::app_root(&work), "receipt.json", &bytes).is_ok());
    if !saved {
        return Err(Failure{code:125,message:"static cleanup or receipt failed; inspect private run; any published output retained".into()});
    }
    if !process_cleanup_verified {
        return result;
    }
    if !clean {
        return Err(Failure {
            code: 125,
            message: "static temporary cleanup failed; private evidence/output retained".into(),
        });
    }
    if published {
        println!(
            "Created static HTML output; private receipt under target/yetlet-static/{run}/receipt.json"
        );
    }
    result
}
