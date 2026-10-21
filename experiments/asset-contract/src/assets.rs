//! Application-owned contract: producer adapters supply data, never markup.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::Path,
};

pub(crate) const MAX_MANIFEST: u64 = 128 * 1024;
pub(crate) const MAX_FILE: u64 = 2 * 1024 * 1024;
pub(crate) const MAX_TOTAL: usize = 16 * 1024 * 1024;
pub(crate) const MAX_NODES: usize = 128;
pub(crate) const MAX_FILES: usize = 256;
pub(crate) const MAX_EDGES: usize = 512;
pub(crate) type Graph = BTreeMap<String, Node>;

#[derive(Clone, Debug, Default)]
pub(crate) struct Node {
    pub(crate) file: String,
    pub(crate) styles: Vec<String>,
    pub(crate) imports: Vec<String>,
    pub(crate) dynamic: Vec<String>,
    pub(crate) assets: Vec<String>,
}

pub(crate) struct Asset {
    pub(crate) bytes: Vec<u8>,
    pub(crate) mime: &'static str,
}

pub(crate) struct Snapshot {
    files: BTreeMap<String, Asset>,
    styles: Vec<String>,
    script: Option<String>,
    images: Vec<String>,
}

pub(crate) fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'/' | b'.' | b'-' | b'_'))
        && value
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}

pub(crate) fn mime(path: &str) -> Result<&'static str, &'static str> {
    if !safe_relative(path) {
        return Err("unsafe asset path");
    }
    match path.rsplit('.').next() {
        Some("css") => Ok("text/css; charset=utf-8"),
        Some("js") => Ok("text/javascript; charset=utf-8"),
        Some("svg") => Ok("image/svg+xml"),
        _ => Err("unsupported asset type"),
    }
}

// The operator must freeze the tree during startup; std path checks are not atomic.
pub(crate) fn read(root: &Path, relative: &str, max: u64) -> Result<Vec<u8>, &'static str> {
    if !root.is_absolute() || !safe_relative(relative) {
        return Err("explicit absolute root and safe relative file required");
    }
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
        let metadata = fs::symlink_metadata(&path).map_err(|_| "build file unavailable")?;
        if metadata.file_type().is_symlink() {
            return Err("symlink build path refused");
        }
    }
    // Inspect before open so a stable FIFO/device cannot block startup.
    if !fs::symlink_metadata(&path)
        .map_err(|_| "cannot inspect build file")?
        .is_file()
    {
        return Err("build file must be regular");
    }
    let file = File::open(path).map_err(|_| "cannot open build file")?;
    let metadata = file
        .metadata()
        .map_err(|_| "cannot inspect open build file")?;
    if !metadata.is_file() {
        return Err("build file must be regular");
    }
    if metadata.len() > max {
        return Err("build file exceeds size limit");
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

pub(crate) fn root(path: &Path) -> Result<std::path::PathBuf, &'static str> {
    if !path.is_absolute() {
        return Err("asset root must be absolute");
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| "asset root unavailable")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("asset root must be a real directory");
    }
    path.canonicalize().map_err(|_| "asset root unavailable")
}

fn validate(graph: &Graph) -> Result<(), &'static str> {
    if graph.is_empty() || graph.len() > MAX_NODES {
        return Err("invalid graph node count");
    }
    let mut paths = BTreeSet::new();
    let mut edges = 0;
    for (key, node) in graph {
        if !safe_relative(key) {
            return Err("unsafe graph key");
        }
        mime(&node.file)?;
        paths.insert(&node.file);
        edges += node.imports.len() + node.dynamic.len() + node.styles.len() + node.assets.len();
        if edges > MAX_EDGES {
            return Err("too many graph edges");
        }
        if (!node.imports.is_empty() || !node.dynamic.is_empty()) && !node.file.ends_with(".js") {
            return Err("only JavaScript can import chunks");
        }
        for style in &node.styles {
            if mime(style)? != "text/css; charset=utf-8" {
                return Err("invalid CSS reference");
            }
            paths.insert(style);
        }
        for asset in &node.assets {
            mime(asset)?;
            paths.insert(asset);
        }
        for reference in node.imports.iter().chain(&node.dynamic) {
            let target = graph.get(reference).ok_or("missing graph reference")?;
            if !target.file.ends_with(".js") {
                return Err("import must target JavaScript");
            }
        }
    }
    if paths.len() > MAX_FILES {
        return Err("too many asset files");
    }
    fn visit<'a>(
        key: &'a str,
        graph: &'a Graph,
        active: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<(), &'static str> {
        if done.contains(key) {
            return Ok(());
        }
        if !active.insert(key) {
            return Err("asset graph cycle");
        }
        for next in graph[key].imports.iter().chain(&graph[key].dynamic) {
            visit(next, graph, active, done)?;
        }
        active.remove(key);
        done.insert(key);
        Ok(())
    }
    let (mut active, mut done) = (BTreeSet::new(), BTreeSet::new());
    for key in graph.keys() {
        visit(key, graph, &mut active, &mut done)?;
    }
    Ok(())
}

impl Snapshot {
    pub(crate) fn load(root_path: &Path, graph: &Graph, entry: &str) -> Result<Self, &'static str> {
        let root = root(root_path)?;
        validate(graph)?;
        let selected = graph.get(entry).ok_or("entry absent from graph")?;
        if !selected.file.ends_with(".js") && !selected.file.ends_with(".css") {
            return Err("entry must be CSS or JavaScript");
        }
        // Static dependency-first DFS; array order and first occurrence of each CSS path win.
        fn initial(
            key: &str,
            graph: &Graph,
            seen: &mut BTreeSet<String>,
            styles: &mut Vec<String>,
            images: &mut Vec<String>,
        ) {
            if !seen.insert(key.to_owned()) {
                return;
            }
            let node = &graph[key];
            for next in &node.imports {
                initial(next, graph, seen, styles, images);
            }
            for path in node
                .styles
                .iter()
                .chain(node.file.ends_with(".css").then_some(&node.file))
            {
                if !styles.contains(path) {
                    styles.push(path.clone());
                }
            }
            for path in &node.assets {
                if path.ends_with(".svg") && !images.contains(path) {
                    images.push(path.clone());
                }
            }
        }
        fn reachable(
            key: &str,
            graph: &Graph,
            seen: &mut BTreeSet<String>,
            paths: &mut BTreeSet<String>,
        ) {
            if !seen.insert(key.to_owned()) {
                return;
            }
            let node = &graph[key];
            paths.insert(node.file.clone());
            paths.extend(node.styles.iter().chain(&node.assets).cloned());
            for next in node.imports.iter().chain(&node.dynamic) {
                reachable(next, graph, seen, paths);
            }
        }
        let (mut styles, mut images) = (Vec::new(), Vec::new());
        initial(entry, graph, &mut BTreeSet::new(), &mut styles, &mut images);
        let mut paths = BTreeSet::new();
        reachable(entry, graph, &mut BTreeSet::new(), &mut paths);
        let (mut files, mut total) = (BTreeMap::new(), 0);
        for path in paths {
            let bytes = read(&root, &path, MAX_FILE)?;
            total += bytes.len();
            if total > MAX_TOTAL {
                return Err("asset snapshot exceeds total limit");
            }
            files.insert(
                path.clone(),
                Asset {
                    mime: mime(&path)?,
                    bytes,
                },
            );
        }
        Ok(Self {
            files,
            styles,
            images,
            script: selected
                .file
                .ends_with(".js")
                .then(|| selected.file.clone()),
        })
    }

    pub(crate) fn asset(&self, path: &str) -> Option<&Asset> {
        self.files.get(path)
    }
    pub(crate) fn images(&self) -> &[String] {
        &self.images
    }
    pub(crate) fn links(&self, mount: &str) -> Result<String, &'static str> {
        if !safe_relative(mount) {
            return Err("unsafe mount");
        }
        let mut tags = String::new();
        for path in &self.styles {
            tags.push_str(&format!(
                "<link rel=\"stylesheet\" href=\"/{mount}/{path}\">"
            ));
        }
        if let Some(script) = &self.script {
            tags.push_str(&format!(
                "<script type=\"module\" src=\"/{mount}/{script}\"></script>"
            ));
        }
        Ok(tags)
    }
    // No filename-hash authenticity claim and no retained-old-build store.
    pub(crate) fn cache_control(&self) -> &'static str {
        "no-store"
    }
}

#[cfg(test)]
pub(crate) mod tests;
