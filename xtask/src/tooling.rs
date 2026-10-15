//! Shared byte/path/process operations used by the seven migrated builders.
use crate::process::{self, Spec};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::Ordering,
    time::Duration,
};

#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub message: String,
}
pub type Result<T> = std::result::Result<T, Error>;
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        fail(format!("file operation: {:?}", e.kind()))
    }
}
pub fn fail(message: impl Into<String>) -> Error {
    Error {
        code: 1,
        message: message.into(),
    }
}
pub fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// Resolve fixed repository inputs/outputs, including existing ancestors of new files.
/// Writes reject symlinks altogether; reads permit contained symlinks only.
pub fn safe(root: &Path, relative: impl AsRef<Path>, write: bool) -> Result<PathBuf> {
    let relative = relative.as_ref();
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(fail("path must be repository-relative without traversal"));
    }
    let canonical = root.canonicalize()?;
    let mut path = canonical.clone();
    for part in relative.components() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) => {
                if write && meta.file_type().is_symlink() {
                    return Err(fail("output symlink rejected"));
                }
                if !path.canonicalize()?.starts_with(&canonical) {
                    return Err(fail("path escapes repository"));
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(path)
}
pub fn read(root: &Path, name: impl AsRef<Path>) -> Result<Vec<u8>> {
    Ok(fs::read(safe(root, name, false)?)?)
}
pub fn text(root: &Path, name: impl AsRef<Path>) -> Result<String> {
    String::from_utf8(read(root, name)?).map_err(|_| fail("input is not UTF-8"))
}
pub fn write(root: &Path, name: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
    check_cancel()?;
    let path = safe(root, name, true)?;
    fs::create_dir_all(path.parent().ok_or_else(|| fail("missing output parent"))?)?;
    fs::write(path, bytes)?;
    Ok(())
}
pub fn json_bytes(value: &Value) -> Vec<u8> {
    let mut s = serde_json::to_string_pretty(value).expect("JSON value");
    s.push('\n');
    s.into_bytes()
}
pub fn check_cancel() -> Result<()> {
    if process::CANCELLED.load(Ordering::Relaxed) {
        Err(Error {
            code: 130,
            message: "cancelled".into(),
        })
    } else {
        Ok(())
    }
}
pub fn tool(name: &str, fallback: &str) -> Result<OsString> {
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let path = dir.join(&executable);
            if path.is_file() {
                return Ok(if path.is_absolute() {
                    path
                } else {
                    std::env::current_dir()?.join(path)
                }
                .into_os_string());
            }
        }
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| fail(format!("missing {name}")))?;
    let path = PathBuf::from(home).join(fallback).join(executable);
    if path.is_file() {
        Ok(path.into_os_string())
    } else {
        Err(fail(format!("missing {name}; no installation attempted")))
    }
}
pub fn execute(cwd: &Path, program: &OsStr, args: &[OsString], capture: bool) -> Result<Vec<u8>> {
    check_cancel()?;
    println!(
        "+ {} {}",
        program.to_string_lossy(),
        args.iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let report = process::run(
        &Spec {
            program: program.to_owned(),
            args: args.to_vec(),
            cwd: cwd.to_owned(),
            env: vec![("RUSTUP_AUTO_INSTALL".into(), Some("0".into()))],
            timeout: Duration::from_secs(1200),
            output_limit: 16 * 1024 * 1024,
        },
        &process::CANCELLED,
    );
    io::stderr().write_all(&report.stderr.bytes)?;
    if !capture {
        io::stdout().write_all(&report.stdout.bytes)?;
    }
    if !report.success() {
        return Err(Error {
            code: report.exit_code(),
            message: format!(
                "native process origin={} native_exit_code={:?} signal={:?} cleanup_ok={}",
                report.origin.as_str(),
                report.native_code(),
                report.signal(),
                report.cleanup_ok
            ),
        });
    }
    if report.stdout.discarded > 0 || report.stderr.discarded > 0 {
        return Err(fail("process capture exceeded byte bound"));
    }
    check_cancel()?;
    Ok(report.stdout.bytes)
}
pub fn lock_version(root: &Path, lock: &str) -> Result<String> {
    let data: toml::Value = text(root, lock)?
        .parse()
        .map_err(|_| fail("invalid lock TOML"))?;
    let packages = data
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| fail("lock package array missing"))?;
    let versions: std::collections::BTreeSet<_> = packages
        .iter()
        .filter(|p| p.get("name").and_then(toml::Value::as_str) == Some("wasm-bindgen"))
        .filter_map(|p| p.get("version").and_then(toml::Value::as_str))
        .collect();
    if versions.len() != 1 {
        return Err(fail("expected exactly one wasm-bindgen lock version"));
    }
    Ok(versions.into_iter().next().unwrap().into())
}

/// UTC Gregorian timestamp from the system clock, without an interpreter or subprocess.
pub fn utc_now() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (seconds / 86400) as i64 + 719468;
    let era = days.div_euclid(146097);
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let clock = seconds % 86400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}+00:00",
        clock / 3600,
        clock / 60 % 60,
        clock % 60
    )
}
