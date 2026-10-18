    pub(crate) fn load_selected(frontend: &str) -> io::Result<Arc<Self>> {
        // Frontend root is project-relative even when settings come from another directory.
        let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if frontend.is_empty() || frontend.len() > 240 || frontend.contains('\\')
            || frontend.contains(':') || frontend.starts_with('/')
        {
            return Err(io::Error::other("Invalid frontend root"));
        }
        for part in frontend.trim_end_matches('/').split('/') {
            if matches!(part, "" | "." | "..") {
                return Err(io::Error::other("Invalid frontend root"));
            }
            root.push(part);
            let metadata = std::fs::symlink_metadata(&root)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(io::Error::other("Frontend root must be a real directory"));
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err(io::Error::other("Frontend reparse point refused"));
                }
            }
        }
        let snapshot = load_manifest(&root.join("dist"), "app.ts")
            .map_err(|_| io::Error::other("Build frontend dist before starting the application"))?;
