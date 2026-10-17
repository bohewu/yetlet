use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

const MAX_MANIFEST: u64 = 128 * 1024;
const MAX_FILE: u64 = 2 * 1024 * 1024;
const MAX_TOTAL: usize = 16 * 1024 * 1024;
const MAX_RECORDS: usize = 128;
const MAX_FILES: usize = 256;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Chunk {
    file: String,
    src: Option<String>,
    name: Option<String>,
    names: Option<Vec<String>>,
    #[serde(default)]
    is_entry: bool,
    #[serde(default)]
    is_dynamic_entry: bool,
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    dynamic_imports: Vec<String>,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default)]
    assets: Vec<String>,
}

pub(crate) struct Asset {
    pub(crate) mime: &'static str,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) struct Snapshot {
    pub(crate) files: BTreeMap<String, Asset>,
    pub(crate) entry: String,
    pub(crate) styles: Vec<String>,
    pub(crate) svg: String,
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'/' | b'.' | b'-' | b'_'))
        && value
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}

fn mime(path: &str) -> Result<&'static str, &'static str> {
    if !safe_relative(path) || !path.starts_with("assets/") {
        return Err("unsafe asset path");
    }
    match path.rsplit('.').next() {
        Some("js") => Ok("text/javascript; charset=utf-8"),
        Some("css") => Ok("text/css; charset=utf-8"),
        Some("svg") => Ok("image/svg+xml"),
        _ => Err("unsupported asset type"),
    }
}

fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>, &'static str> {
    let file = File::open(path).map_err(|_| "required build file unavailable")?;
    if !file
        .metadata()
        .map_err(|_| "cannot inspect build file")?
        .is_file()
    {
        return Err("build asset must be a regular file");
    }
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read build file")?;
    if bytes.len() as u64 > max {
        return Err("build file exceeds size limit");
    }
    Ok(bytes)
}

fn parse(bytes: &[u8]) -> Result<BTreeMap<String, Chunk>, &'static str> {
    if bytes.len() as u64 > MAX_MANIFEST {
        return Err("manifest exceeds size limit");
    }
    let chunks: BTreeMap<String, Chunk> =
        serde_json::from_slice(bytes).map_err(|_| "invalid manifest schema")?;
    if chunks.is_empty() || chunks.len() > MAX_RECORDS {
        return Err("invalid manifest record count");
    }
    for (key, chunk) in &chunks {
        if !safe_relative(key)
            || chunk.src.as_deref().is_some_and(|s| !safe_relative(s))
            || chunk.name.as_deref().is_some_and(|s| !safe_relative(s))
            || chunk.names.as_ref().is_some_and(|names| {
                names.len() > MAX_RECORDS || names.iter().any(|name| !safe_relative(name))
            })
        {
            return Err("unsafe manifest metadata");
        }
        let kind = mime(&chunk.file)?;
        if (chunk.is_entry
            || chunk.is_dynamic_entry
            || !chunk.imports.is_empty()
            || !chunk.dynamic_imports.is_empty())
            && !kind.starts_with("text/javascript")
        {
            return Err("non-JavaScript chunk has script metadata");
        }
        for css in &chunk.css {
            if mime(css)? != "text/css; charset=utf-8" {
                return Err("invalid CSS reference");
            }
        }
        for asset in &chunk.assets {
            mime(asset)?;
        }
        for reference in chunk.imports.iter().chain(&chunk.dynamic_imports) {
            let target = chunks.get(reference).ok_or("missing manifest reference")?;
            if !mime(&target.file)?.starts_with("text/javascript") {
                return Err("non-JavaScript import reference");
            }
        }
    }
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for key in chunks.keys() {
        acyclic(key, &chunks, &mut active, &mut done)?;
    }
    Ok(chunks)
}

fn acyclic<'a>(
    key: &'a str,
    chunks: &'a BTreeMap<String, Chunk>,
    active: &mut BTreeSet<&'a str>,
    done: &mut BTreeSet<&'a str>,
) -> Result<(), &'static str> {
    if done.contains(key) {
        return Ok(());
    }
    if !active.insert(key) {
        return Err("manifest import cycle");
    }
    let chunk = &chunks[key];
    for reference in chunk.imports.iter().chain(&chunk.dynamic_imports) {
        acyclic(reference, chunks, active, done)?;
    }
    active.remove(key);
    done.insert(key);
    Ok(())
}

// Dependency-first DFS retains manifest array order and excludes dynamic imports.
fn static_files(
    key: &str,
    chunks: &BTreeMap<String, Chunk>,
    visited: &mut BTreeSet<String>,
    styles: &mut Vec<String>,
    images: &mut BTreeSet<String>,
) {
    if !visited.insert(key.to_owned()) {
        return;
    }
    let chunk = &chunks[key];
    for reference in &chunk.imports {
        static_files(reference, chunks, visited, styles, images);
    }
    for css in &chunk.css {
        if !styles.contains(css) {
            styles.push(css.clone());
        }
    }
    images.extend(chunk.assets.iter().filter(|s| s.ends_with(".svg")).cloned());
}

impl Snapshot {
    pub(crate) fn load(root: &Path, entry: &str) -> Result<Self, &'static str> {
        let root = root
            .canonicalize()
            .map_err(|_| "build directory unavailable")?;
        let manifest = root.join(".vite/manifest.json");
        let canonical_manifest = manifest
            .canonicalize()
            .map_err(|_| "manifest unavailable")?;
        if !canonical_manifest.starts_with(&root) {
            return Err("manifest escapes build directory");
        }
        let chunks = parse(&read_bounded(&canonical_manifest, MAX_MANIFEST)?)?;
        let selected = chunks.get(entry).ok_or("entry absent from manifest")?;
        if !selected.is_entry || !selected.file.ends_with(".js") {
            return Err("expected JavaScript entry");
        }
        let mut paths = BTreeSet::new();
        for chunk in chunks.values() {
            paths.insert(chunk.file.clone());
            paths.extend(chunk.css.iter().cloned());
            paths.extend(chunk.assets.iter().cloned());
        }
        if paths.len() > MAX_FILES {
            return Err("too many build assets");
        }
        let mut files = BTreeMap::new();
        let mut total = 0;
        for path in paths {
            let canonical = root
                .join(&path)
                .canonicalize()
                .map_err(|_| "build asset missing")?;
            if !canonical.starts_with(&root) {
                return Err("build asset escapes build directory");
            }
            let bytes = read_bounded(&canonical, MAX_FILE)?;
            total += bytes.len();
            if total > MAX_TOTAL {
                return Err("build assets exceed total size limit");
            }
            files.insert(
                path.clone(),
                Asset {
                    mime: mime(&path)?,
                    bytes,
                },
            );
        }
        let mut styles = Vec::new();
        let mut images = BTreeSet::new();
        static_files(
            entry,
            &chunks,
            &mut BTreeSet::new(),
            &mut styles,
            &mut images,
        );
        if images.len() != 1 {
            return Err("expected one statically imported SVG");
        }
        Ok(Self {
            files,
            entry: selected.file.clone(),
            styles,
            svg: images.into_iter().next().ok_or("SVG unavailable")?,
        })
    }
}

#[cfg(test)]
mod tests;
