//! Closed, compiled source closure for native delivery; no arbitrary source export.
use crate::platform::fs;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(crate) use crate::framework_files::FILES;

pub(crate) fn collect(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    fn walk(root: &Path, relative: &Path, found: &mut BTreeSet<String>) -> Result<(), String> {
        for entry in
            std::fs::read_dir(root.join(relative)).map_err(|_| "framework layout unavailable")?
        {
            let entry = entry.map_err(|_| "framework layout unavailable")?;
            let path = relative.join(entry.file_name());
            let kind = entry
                .file_type()
                .map_err(|_| "framework layout unavailable")?;
            if kind.is_symlink() {
                return Err("framework source links are unsupported".into());
            }
            if kind.is_dir() {
                if relative == Path::new("crates/yetlet") && entry.file_name() == "target" {
                    continue;
                }
                walk(root, &path, found)?;
            } else if kind.is_file() {
                if path == Path::new("crates/yetlet/Cargo.lock") {
                    continue;
                }
                found.insert(
                    path.to_str()
                        .ok_or("framework source names must be Unicode")?
                        .replace('\\', "/"),
                );
            } else {
                return Err("unsupported framework source kind".into());
            }
        }
        Ok(())
    }
    let mut found = BTreeSet::new();
    walk(root, Path::new("crates/yetlet"), &mut found)?;
    if found != FILES.iter().map(|(p, _)| (*p).to_owned()).collect() {
        return Err("unsupported framework source layout".into());
    }
    let mut files = BTreeMap::new();
    for (path, expected) in FILES {
        let bytes = fs::read_framework(&root.join(path))
            .map_err(|_| "framework input is missing, oversized or unsafe")?;
        let text = std::str::from_utf8(&bytes).map_err(|_| "framework source must be UTF-8")?;
        let expected =
            std::str::from_utf8(expected).map_err(|_| "compiled framework source must be UTF-8")?;
        if text.replace("\r\n", "\n") != expected.replace("\r\n", "\n") {
            return Err("unsupported framework source or dependency closure".into());
        }
        files.insert((*path).to_owned(), bytes);
    }
    Ok(files)
}

pub(crate) fn normal_dependencies(runtime_settings: bool) -> BTreeSet<String> {
    // The delivery profile accepts only server-axum and optionally config.
    // Cargo locks only activated optional dependencies for a path dependency.
    let mut names = BTreeSet::from_iter(["axum", "serde", "tower"].map(str::to_owned));
    if runtime_settings {
        names.extend(["toml", "dotenvy", "libc"].map(str::to_owned));
    }
    names
}
