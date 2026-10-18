//! Internal compile-time filesystem boundary; no public adapter API.
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "linux", windows)))]
mod unsupported;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
use linux as imp;
#[cfg(not(any(target_os = "linux", windows)))]
use unsupported as imp;
#[cfg(windows)]
use windows as imp;
#[cfg(windows)]
pub(in crate::platform) use windows::Executable;

use crate::app;
pub(crate) use imp::{
    App, NEW_DIAGNOSTIC, NEW_SUPPORTED, app_root, create_project, open_directory, read_framework,
};
pub(crate) use imp::{app_directory_entries, app_entries, read_app_file, read_app_input};
use std::path::{Component, Path};

const UNSUPPORTED_CREATE: &str = "new is unsupported on this platform: no verified no-replace/reparse implementation; nothing written";

// Must precede framework inspection/template rendering, retaining refusal ordering.
pub(crate) fn ensure_create_supported() -> Result<(), String> {
    if NEW_SUPPORTED {
        Ok(())
    } else {
        Err(UNSUPPORTED_CREATE.into())
    }
}

pub(crate) fn open_app(manifest: &Path) -> Result<App, String> {
    let files = imp::AppFiles::open(manifest)?;
    let manifest = app::parse_file("Cargo.toml", files.read("Cargo.toml")?)?;
    app::validate_manifest(&manifest)?;
    let lock = app::parse_file("Cargo.lock", files.read("Cargo.lock")?)?;
    app::validate_lock(&lock)?;
    files.finish()
}

// Only exact generated inputs and the flat built Vite asset directory are readable.
fn app_input(name: &str) -> Result<(&str, &str), String> {
    match name {
        "src/main.rs" | "src/config.rs" | "src/vite.rs" => Ok(("src", &name[4..])),
        "dist/.vite/manifest.json" => Ok(("dist/.vite", "manifest.json")),
        "Cargo.toml"
        | "Cargo.lock"
        | "base.toml"
        | "development.toml"
        | "staging.toml"
        | "production.toml"
        | "package.json"
        | "package-lock.json"
        | "vite.config.mjs"
        | "vite.dev.config.mjs"
        | "vite.dev.bridge.mjs"
        | ".env.example"
        | "tsconfig.json"
        | "app.ts"
        | "app.css"
        | "companion.ts"
        | "shared.ts"
        | "shared.css"
        | "lazy.ts"
        | "lazy.css"
        | "mark.svg" => Ok(("", name)),
        _ => {
            let file = name
                .strip_prefix("dist/assets/")
                .ok_or("unsupported app input")?;
            if file.is_empty()
                || file.len() > 200
                || file.starts_with('.')
                || !file
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
                || !["js", "css", "svg"].contains(&file.rsplit('.').next().unwrap_or(""))
            {
                return Err("unsupported built asset path".into());
            }
            Ok(("dist/assets", file))
        }
    }
}

fn app_directory(name: &str) -> Result<(), String> {
    if ["", "src", "dist", "dist/.vite", "dist/assets"].contains(&name) {
        Ok(())
    } else {
        Err("unsupported app directory".into())
    }
}

// Only the scaffold's two fixed children or the generator's cargo overlay are
// writable. This does not expand the separate generator input/read allowlist.
fn generated_subdirectories(files: &[(&str, String)]) -> Result<Vec<&'static str>, String> {
    let mut children = Vec::new();
    for (path, _) in files {
        let directory = match *path {
            "src/main.rs" | "src/config.rs" | "src/tailwind.rs" | "src/vite.rs" => Some("src"),
            "cargo-overlay/Cargo.toml" => Some("cargo-overlay"),
            "frontend/package.json"
            | "frontend/package-lock.json"
            | "frontend/vite.config.mjs"
            | "frontend/vite.dev.config.mjs"
            | "frontend/vite.dev.bridge.mjs"
            | "frontend/tsconfig.json"
            | "frontend/app.ts"
            | "frontend/app.css"
            | "frontend/companion.ts"
            | "frontend/shared.ts"
            | "frontend/shared.css"
            | "frontend/lazy.ts"
            | "frontend/lazy.css"
            | "frontend/mark.svg"
            | "frontend/.env.example" => Some("frontend"),
            name if !name.is_empty()
                && !matches!(name, "." | "..")
                && !name.contains(['/', '\\', '\0']) =>
            {
                None
            }
            _ => return Err("unsupported generated file path".into()),
        };
        if let Some(directory) = directory
            && !children.contains(&directory)
        {
            children.push(directory);
        }
    }
    if children.contains(&"cargo-overlay") && children.len() != 1 {
        return Err("mixed generator and scaffold subdirectories are unsupported".into());
    }
    Ok(children)
}

// Frontend roots are independent of generator inputs and always app-relative.
pub(crate) fn frontend_directory(app: &App, relative: &Path) -> Result<App, String> {
    let text = relative.to_str().ok_or("frontend root must be Unicode")?;
    if text.is_empty()
        || text.contains([':', '\\', '\0'])
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(
            "frontend root must be a nonempty project-relative path without traversal".into(),
        );
    }
    imp::verify_app(app)?;
    let mut path = app_root(app).to_owned();
    let mut selected = None;
    // Each selected descendant must be protected too, not just its last child.
    for component in relative.components() {
        path.push(component.as_os_str());
        selected = Some(open_directory(&path)?);
    }
    imp::verify_app(app)?;
    selected.ok_or_else(|| "frontend root must select a directory".into())
}

/// CLI frontend startup keeps its input separate from generator qualification.
pub(crate) fn read_frontend_manifest(frontend: &App) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "linux")]
    return linux::read_frontend_manifest(frontend);
    #[cfg(not(target_os = "linux"))]
    read_app_file(frontend, "dist/.vite/manifest.json")
}
