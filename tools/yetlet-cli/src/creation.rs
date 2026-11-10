//! New-project orchestration: resolver result -> exact scaffold -> Cargo lock.
use crate::{
    app::Failure,
    platform::{fs, process},
    scaffold::{self, Template},
    sdk_registry,
    sdk_version::{self as sdk, Request, ResolvedSdk},
};
use std::{path::Path, process::Command};

#[derive(Clone, Copy)]
pub(crate) struct Options {
    pub template: Template,
    pub service: bool,
    pub settings: bool,
    pub tailwind: bool,
    pub vite: bool,
}

pub(crate) fn execute(
    name: &str,
    parent: &Path,
    options: Options,
    exact: Option<&str>,
    offline: bool,
) -> Result<ResolvedSdk, Failure> {
    let offline =
        offline || std::env::var("CARGO_NET_OFFLINE").is_ok_and(|v| v.eq_ignore_ascii_case("true"));
    let request = Request {
        exact,
        template: options.template,
        settings: options.settings || options.vite,
        offline,
    };
    let result = (|| {
        let root = sdk_registry::home()?;
        let now = sdk_registry::now()?;
        create(
            name,
            parent,
            options,
            request,
            &mut sdk_registry::CratesIo,
            &root,
            now,
            || {
                let report = crate::doctor::check();
                if report.ok() {
                    Ok(())
                } else {
                    Err(format!(
                        "CLI_INFRASTRUCTURE: installed tool preflight failed (exit={}): {}",
                        report.exit_code(),
                        report.render(false)
                    ))
                }
            },
            generate_lock,
        )
    })();
    result.map_err(|message: String| Failure {
        code: if message.contains("CLI_INFRASTRUCTURE") {
            125
        } else if message.contains("CLI_CANCELLED") {
            128 + process::cancellation_signal()
        } else {
            1
        },
        message,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create(
    name: &str,
    parent: &Path,
    options: Options,
    request: Request<'_>,
    registry: &mut impl sdk_registry::Registry,
    cache: &Path,
    now: u64,
    tools: impl FnOnce() -> Result<(), String>,
    mut lock: impl FnMut(&Path, bool) -> Result<(), String>,
) -> Result<ResolvedSdk, String> {
    scaffold::validate_name(name).map_err(str::to_owned)?;
    if options.template.browser() && options.service {
        return Err("browser template does not select --service".into());
    }
    if let Some(version) = request.exact {
        sdk::exact(version).map_err(str::to_owned)?;
    }
    fs::ensure_create_supported()?;
    let parent = crate::framework::absolute(parent).map_err(|_| "invalid parent path")?;
    fs::preflight_new(&parent, name)?;
    tools()?;
    eprintln!(
        "SDK resolution mode={}",
        if request.offline { "offline" } else { "online" }
    );
    let sdk = sdk_registry::resolve(request, registry, cache, now)?;
    eprintln!("{}", sdk.notice(now));
    let mut entries = if options.template.browser() {
        crate::browser::files_version(name, options.tailwind, &sdk.version)?
    } else {
        scaffold::selected_files_version(
            name,
            options.service,
            options.settings,
            options.tailwind,
            options.vite,
            &sdk.version,
        )?
    };
    if matches!(
        options.template,
        Template::NativeStyle | Template::BrowserStyle
    ) {
        crate::styles::scaffold(
            &mut entries,
            options.template.browser(),
            options.tailwind,
            options.vite || options.template.browser(),
        )?;
    }
    for (path, data) in &mut entries {
        if *path == "README.md" {
            *data = data.replace("\ncargo generate-lockfile\n", "\n")
                .replace("\ncargo generate-lockfile --offline\n", "\n")
                .replace("Creation writes files only: no Cargo lock, package installation, build, download\nor listener.", "Creation resolved the registry SDK and generated this app's Cargo.lock before publication.\nNo package installation, build or listener is started.")
                .replace("No dependency install, lock generation or process starts\nduring creation.", "Creation generates Cargo.lock in private staging; it does not install dependencies or start the app.")
                .replace("normal dependency lock/cache separately as above", "dependency cache separately as above; retain its generated app lock")
                .replace("--template browser --tailwind", &format!("--template browser --tailwind --sdk-version {}",sdk.version));
            data.push_str("\nThe CLI resolved an exact SDK and generated Cargo.lock at creation. Retain this lock for later frozen builds. See .yetlet/sdk-resolution.json for creation provenance.\n");
        }
        if *path == ".gitignore" {
            data.push_str("!/.yetlet/\n/.yetlet/*\n!/.yetlet/sdk-resolution.json\n");
        }
    }
    fs::create_prepared(&parent, name, &entries, |stage| {
        lock(stage, request.offline)
            .map_err(|error| format!("{error}; selected SDK {}", sdk.version))?;
        // Lock setup is not permitted to rewrite template inputs or the nested
        // Style workspace lock. Revalidate all original emitted bytes.
        for (path, data) in &entries {
            let bytes = fs::read_framework(&stage.join(path))
                .map_err(|_| "SDK_LOCK_MISMATCH: template input unavailable")?;
            if bytes != data.as_bytes() {
                return Err(format!(
                    "SDK_LOCK_MISMATCH: Cargo changed template input {path}"
                ));
            }
        }
        let bytes = fs::read_framework(&stage.join("Cargo.lock"))
            .map_err(|_| "SDK_LOCK_FAILED: app lock missing or unsafe")?;
        validate_lock(&bytes, &sdk)?;
        let receipt = serde_json::json!({"schema":1, "cli_version":env!("CARGO_PKG_VERSION"), "sdk":sdk, "cargo_lock_sha256":sdk::digest(&bytes)});
        Ok(vec![(
            ".yetlet/sdk-resolution.json",
            serde_json::to_string_pretty(&receipt).unwrap() + "\n",
        )])
    })?;
    Ok(sdk)
}

pub(crate) fn generate_lock(stage: &Path, offline: bool) -> Result<(), String> {
    let app = fs::open_directory(stage).map_err(|e| format!("CLI_INFRASTRUCTURE: {e}"))?;
    let mut command = Command::new("cargo");
    command
        .args(["generate-lockfile", "--manifest-path"])
        .arg(stage.join("Cargo.toml"))
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("RUSTUP_SKIP_UPDATE_CHECK", "1")
        .env("CARGO_TERM_COLOR", "never");
    if offline {
        command.arg("--offline").env("CARGO_NET_OFFLINE", "true");
    }
    let output = process::capture_cargo(&app, &mut command).map_err(|e| {
        if e.kind() == std::io::ErrorKind::Interrupted { format!("CLI_CANCELLED: {e}") }
        else if e.kind() == std::io::ErrorKind::TimedOut || e.to_string().starts_with("Cargo lock process failed:") {
            format!("SDK_LOCK_FAILED: {e}; prepare the Cargo dependency cache separately for offline use")
        } else { format!("CLI_INFRASTRUCTURE: Cargo spawn/output/cleanup failed: {e}") }
    })?;
    if !output.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output));
    }
    Ok(())
}

pub(crate) fn validate_lock(bytes: &[u8], sdk: &ResolvedSdk) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "SDK_LOCK_MISMATCH: non-UTF8 Cargo lock")?;
    let lock: toml::Value =
        toml::from_str(text).map_err(|_| "SDK_LOCK_MISMATCH: invalid Cargo lock")?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("SDK_LOCK_MISMATCH: no lock packages")?;
    for selected in &sdk.packages {
        let entries: Vec<_> = packages
            .iter()
            .filter(|p| p.get("name").and_then(toml::Value::as_str) == Some(&selected.name))
            .collect();
        if entries.len() != 1
            || entries[0].get("version").and_then(toml::Value::as_str) != Some(&sdk.version)
            || entries[0].get("source").and_then(toml::Value::as_str) != Some(sdk::CARGO_REGISTRY)
            || entries[0].get("checksum").and_then(toml::Value::as_str) != Some(&selected.checksum)
        {
            return Err(format!(
                "SDK_LOCK_MISMATCH: {} version/source/checksum differs from selected registry metadata",
                selected.name
            ));
        }
    }
    for package in packages {
        if package
            .get("source")
            .and_then(toml::Value::as_str)
            .is_some_and(|s| s.starts_with("registry+"))
            && package
                .get("checksum")
                .and_then(toml::Value::as_str)
                .is_none()
        {
            return Err("SDK_LOCK_MISMATCH: missing registry checksum".into());
        }
    }
    Ok(())
}
