//! Filesystem operations scoped to static export. Existing generators keep
//! their narrower allowlists; this module never follows links or replaces data.
#[cfg(any(target_os = "linux", windows))]
use super::imp::static_output as native;
use super::{App, app_root, create_project, open_directory};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

fn failure() -> String {
    "static filesystem boundary refused; output preserved; inspect private receipt".into()
}

pub(crate) fn read(root: &Path, relative: &str, limit: usize) -> Result<Vec<u8>, String> {
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|p| !matches!(p, std::path::Component::Normal(_)))
    {
        return Err(failure());
    }
    let parent = open_directory(&root.join(relative.parent().unwrap_or(Path::new(""))))?;
    #[cfg(any(target_os = "linux", windows))]
    return native::read(&parent, relative.file_name().ok_or_else(failure)?, limit)
        .map_err(|_| failure());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (parent, limit);
        Err(failure())
    }
}
pub(crate) fn write(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let rel = Path::new(relative);
    if rel
        .components()
        .any(|p| !matches!(p, std::path::Component::Normal(_)))
    {
        return Err(failure());
    }
    let mut parent = open_directory(root)?;
    if let Some(parts) = rel.parent() {
        for part in parts.components() {
            parent = ensure(&parent, part.as_os_str())?;
        }
    }
    #[cfg(any(target_os = "linux", windows))]
    return native::write(&parent, rel.file_name().ok_or_else(failure)?, bytes)
        .map_err(|_| failure());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (parent, bytes);
        Err(failure())
    }
}
pub(crate) fn ensure(parent: &App, name: &OsStr) -> Result<App, String> {
    let path = app_root(parent).join(name);
    match std::fs::symlink_metadata(&path) {
        Ok(_) => open_directory(&path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            create_project(app_root(parent), name.to_str().ok_or_else(failure)?, &[])?;
            open_directory(&path)
        }
        Err(_) => Err(failure()),
    }
}
pub(crate) fn new_directory(parent: &App, name: &str) -> Result<App, String> {
    absent(parent, name.as_ref())?;
    create_project(app_root(parent), name, &[])?;
    open_directory(&app_root(parent).join(name))
}
pub(crate) fn absent(parent: &App, name: &OsStr) -> Result<(), String> {
    #[cfg(any(target_os = "linux", windows))]
    return native::absent(parent, name)
        .map_err(|_| "static output already exists or is unsafe; select a new directory".into());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (parent, name);
        Err(failure())
    }
}
pub(crate) fn publish(stage: App, parent: &App, name: &OsStr) -> Result<(), String> {
    #[cfg(any(target_os = "linux", windows))]
    return native::publish(stage, parent, name).map_err(|_| failure());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (stage, parent, name);
        Err(failure())
    }
}
fn entries(app: &App) -> Result<Vec<(OsString, bool)>, String> {
    #[cfg(any(target_os = "linux", windows))]
    return native::list(app).map_err(|_| failure());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = app;
        Err(failure())
    }
}
/// Enumerate only protected regular files; bounded before reading their bytes.
pub(crate) fn inventory(root: &Path) -> Result<Vec<String>, String> {
    fn visit(
        app: App,
        prefix: &str,
        depth: usize,
        files: &mut Vec<String>,
        count: &mut usize,
    ) -> Result<(), String> {
        if depth > 8 {
            return Err("static directory depth limit".into());
        }
        for (name, dir) in entries(&app)? {
            *count += 1;
            if *count > 4096 {
                return Err("static entry limit".into());
            }
            let name = name.to_str().ok_or_else(failure)?;
            let rel = format!("{prefix}{name}");
            if dir {
                files.push(format!("{rel}/"));
                visit(
                    open_directory(&app_root(&app).join(name))?,
                    &format!("{rel}/"),
                    depth + 1,
                    files,
                    count,
                )?;
            } else {
                files.push(rel);
                if files.len() > 1536 {
                    return Err("static file count limit".into());
                }
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(open_directory(root)?, "", 0, &mut files, &mut 0)?;
    files.sort();
    Ok(files)
}
/// Cleanup is restricted to an allocated private tree. Links, unfamiliar
/// object types/ownership or excessive entries retain it for inspection.
pub(crate) fn cleanup(root: &Path) -> Result<(), String> {
    fn visit(app: App, depth: usize, count: &mut usize) -> Result<(), String> {
        if depth > 9 {
            return Err(failure());
        }
        for (name, dir) in entries(&app)? {
            *count += 1;
            if *count > 4096 {
                return Err(failure());
            }
            if dir {
                visit(
                    open_directory(&app_root(&app).join(&name))?,
                    depth + 1,
                    count,
                )?;
            } else {
                #[cfg(any(target_os = "linux", windows))]
                native::remove_file(&app, &name).map_err(|_| failure())?;
                #[cfg(not(any(target_os = "linux", windows)))]
                return Err(failure());
            }
        }
        #[cfg(any(target_os = "linux", windows))]
        return native::remove_dir(app).map_err(|_| failure());
        #[cfg(not(any(target_os = "linux", windows)))]
        Err(failure())
    }
    visit(open_directory(root)?, 0, &mut 0)
}
pub(crate) fn source_snapshot(root: &Path) -> Result<BTreeMap<String, String>, String> {
    use sha2::{Digest, Sha256};
    let mut names = vec!["Cargo.toml".to_owned(), "Cargo.lock".to_owned()];
    for name in inventory(&root.join("src"))? {
        if !name.ends_with('/') {
            names.push(format!("src/{name}"));
        }
    }
    match std::fs::symlink_metadata(root.join("build.rs")) {
        Ok(_) => names.push("build.rs".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(_) => return Err(failure()),
    }
    if names.len() > 1281 {
        return Err("static source count limit".into());
    }
    let mut total = 0;
    let mut result = BTreeMap::new();
    for name in names {
        let bytes = read(root, &name, 16 * 1024 * 1024)?;
        total += bytes.len();
        if total > 64 * 1024 * 1024 {
            return Err("static source size limit".into());
        }
        result.insert(name, format!("{:x}", Sha256::digest(bytes)));
    }
    Ok(result)
}
/// One captured selection shared by preflight and Cargo execution. Static mode
/// binds these paths explicitly rather than trying to reproduce Cargo's config
/// inheritance (which could select another directory after preflight).
pub(crate) struct CargoDirectories {
    pub target: PathBuf,
    pub build: PathBuf,
}
pub(crate) fn cargo_directories(app: &App) -> Result<CargoDirectories, String> {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .or_else(|| std::env::var_os("CARGO_BUILD_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    let target = crate::framework::absolute(&app_root(app).join(target)).map_err(|_| failure())?;
    let build = std::env::var_os("CARGO_BUILD_BUILD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| target.clone());
    let build = crate::framework::absolute(&app_root(app).join(build)).map_err(|_| failure())?;
    // Cargo expands braces in build.build-dir. Literal filesystem validation
    // cannot attest a different expanded path; require an explicit plain path.
    if build.as_os_str().to_string_lossy().contains(['{', '}']) {
        return Err("static Cargo build directory must not contain path templates".into());
    }
    Ok(CargoDirectories { target, build })
}
pub(crate) fn output_path(
    app: &App,
    output: &Path,
    cargo: &CargoDirectories,
) -> Result<(App, PathBuf), String> {
    let path = crate::framework::absolute(&app_root(app).join(output)).map_err(|_| failure())?;
    let overlap = |x: &Path| path.starts_with(x) || x.starts_with(&path);
    if path == app_root(app)
        || app_root(app).starts_with(&path)
        || overlap(&app_root(app).join("src"))
        || overlap(&cargo.target)
        || overlap(&cargo.build)
        || overlap(&app_root(app).join("target/yetlet-static"))
    {
        return Err("static output overlaps source/build/private inputs".into());
    }
    let parent = open_directory(path.parent().ok_or_else(failure)?)?;
    // Compare only after protected no-link opens; do not canonicalize through
    // an unchecked symlink/reparse path. This also handles Windows casing.
    let physical_parent = std::fs::canonicalize(app_root(&parent)).map_err(|_| failure())?;
    let physical_app = std::fs::canonicalize(app_root(app)).map_err(|_| failure())?;
    let compared = physical_parent.join(path.file_name().ok_or_else(failure)?);
    // Resolve existing Cargo aliases for comparison only; public destination
    // opens still use the protected no-link boundary above. For an absent cache
    // resolve its existing parent so Windows casing and parent aliases agree.
    for directory in [&cargo.target, &cargo.build] {
        let mut ancestor = directory.as_path();
        let mut suffix = Vec::new();
        while !ancestor.exists() {
            suffix.push(ancestor.file_name().ok_or_else(failure)?);
            ancestor = ancestor.parent().ok_or_else(failure)?;
        }
        let mut actual = std::fs::canonicalize(ancestor).map_err(|_| failure())?;
        for part in suffix.into_iter().rev() {
            actual.push(part);
        }
        if compared.starts_with(&actual) || actual.starts_with(&compared) {
            return Err("static output overlaps the selected Cargo directories".into());
        }
    }
    if compared.starts_with(&physical_app) && !compared.starts_with(physical_app.join("artifacts"))
    {
        return Err("static output inside the app must be below its artifacts directory".into());
    }
    if physical_app.starts_with(&compared) {
        return Err("static output contains app inputs".into());
    }
    absent(&parent, path.file_name().ok_or_else(failure)?)?;
    Ok((parent, path))
}
