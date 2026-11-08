//! Styles-only replacement. Static export remains strictly create-only.
use super::{App, app_root, open_directory, static_output};
use std::path::Path;

pub(crate) fn replace(
    root: &Path,
    relative: &str,
    old: Option<&[u8]>,
    data: &[u8],
) -> Result<(), String> {
    replace_tracked(root, relative, old, data, &mut false)
}

/// Reports a completed destination rename even if subsequent sync/verification fails.
pub(crate) fn replace_tracked(
    root: &Path,
    relative: &str,
    old: Option<&[u8]>,
    data: &[u8],
    replaced: &mut bool,
) -> Result<(), String> {
    if data.len() > 2 * 1024 * 1024 || !crate::settings::valid_frontend_root(relative) {
        return Err("style output path/size refused".into());
    }
    let relative = Path::new(relative);
    let mut parent = open_directory(root)?;
    for part in relative.parent().unwrap_or(Path::new("")).components() {
        parent = static_output::ensure(&parent, part.as_os_str())?;
    }
    let key = relative.file_name().ok_or("style output name missing")?;
    #[cfg(any(target_os = "linux", windows))]
    return super::imp::styles_output::replace(&parent, key, old, data, replaced).map_err(|_| {
        "style publication refused; inspect output/ownership and reconcile explicitly".into()
    });
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (parent, key, old, replaced);
        Err("style publication unsupported on this platform".into())
    }
}

pub(crate) fn lock(app: &App) -> Result<std::fs::File, String> {
    let private = static_output::ensure(app, ".yetlet".as_ref())?;
    let state = static_output::ensure(&private, "styles".as_ref())?;
    let path = app_root(&state).join("coordinator.lock");
    if !path.exists() {
        static_output::write(app_root(&state), "coordinator.lock", b"")?;
    }
    // Reuse the protected read to reject links, hardlinks and nonregular files.
    static_output::read(app_root(&state), "coordinator.lock", 0)?;
    let file = std::fs::File::open(path).map_err(|_| "style coordinator lock unavailable")?;
    file.try_lock()
        .map_err(|_| "another style coordinator owns this app")?;
    Ok(file)
}

/// Marker removal is the acceptance commit point. All verification precedes the
/// native unlink/disposition; no fallible operation follows successful removal.
pub(crate) fn complete(root: &Path, relative: &str, expected: &[u8]) -> Result<(), String> {
    if static_output::read(root, relative, 2 * 1024 * 1024)? != expected {
        return Err("style pending evidence changed; reconcile explicitly".into());
    }
    let path = Path::new(relative);
    let parent = open_directory(&root.join(path.parent().ok_or("missing pending parent")?))?;
    let key = path.file_name().ok_or("missing pending name")?;
    #[cfg(any(target_os = "linux", windows))]
    return super::imp::static_output::remove_file(&parent, key)
        .map_err(|_| "style pending evidence retained; reconcile explicitly".into());
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (parent, key);
        Err("style publication unsupported on this platform".into())
    }
}
#[cfg(test)]
thread_local! {
    pub(crate) static FAIL_AFTER_RENAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
pub(crate) fn after_rename(key: &std::ffi::OsStr) -> std::io::Result<()> {
    if FAIL_AFTER_RENAME.with(|flag| {
        let mut flag = flag.borrow_mut();
        if flag.as_deref().is_some_and(|name| key == name) {
            *flag = None;
            true
        } else {
            false
        }
    }) {
        Err(std::io::Error::other("injected post-record-rename failure"))
    } else {
        Ok(())
    }
}
