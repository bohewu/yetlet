//! Application-owned Vite production adapter, copied from the YT-098 experiment.
//! Trusted, quiescent build roots only; no public framework asset API.
mod assets {
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
            edges +=
                node.imports.len() + node.dynamic.len() + node.styles.len() + node.assets.len();
            if edges > MAX_EDGES {
                return Err("too many graph edges");
            }
            if (!node.imports.is_empty() || !node.dynamic.is_empty()) && !node.file.ends_with(".js")
            {
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
        pub(crate) fn load(
            root_path: &Path,
            graph: &Graph,
            entry: &str,
        ) -> Result<Self, &'static str> {
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
    mod tests {
        use super::*;
        use std::{
            path::PathBuf,
            sync::atomic::{AtomicUsize, Ordering},
        };

        pub(crate) struct Build(pub(crate) PathBuf);
        impl Build {
            pub(crate) fn new() -> Self {
                static ID: AtomicUsize = AtomicUsize::new(0);
                let path = std::env::temp_dir().join(format!(
                    "yt098-{}-{}",
                    std::process::id(),
                    ID.fetch_add(1, Ordering::Relaxed)
                ));
                fs::create_dir(&path).unwrap();
                Self(path)
            }
            fn write(&self, file: &str, bytes: &[u8]) {
                let path = self.0.join(file);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, bytes).unwrap();
            }
            fn populate(&self, graph: &Graph) {
                for node in graph.values() {
                    for path in [&node.file]
                        .into_iter()
                        .chain(&node.styles)
                        .chain(&node.assets)
                    {
                        self.write(path, path.as_bytes());
                    }
                }
            }
        }
        impl Drop for Build {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        fn node(file: &str, styles: &[&str], imports: &[&str], dynamic: &[&str]) -> Node {
            Node {
                file: file.into(),
                styles: styles.iter().map(|s| s.to_string()).collect(),
                imports: imports.iter().map(|s| s.to_string()).collect(),
                dynamic: dynamic.iter().map(|s| s.to_string()).collect(),
                ..Node::default()
            }
        }
        fn graph() -> Graph {
            Graph::from([
                (
                    "app".into(),
                    node(
                        "assets/app.js",
                        &["assets/app.css", "assets/shared.css"],
                        &["left", "right"],
                        &["lazy"],
                    ),
                ),
                (
                    "left".into(),
                    node(
                        "assets/left.js",
                        &["assets/left-2.css", "assets/left-1.css"],
                        &["shared"],
                        &[],
                    ),
                ),
                (
                    "right".into(),
                    node(
                        "assets/right.js",
                        &["assets/right.css", "assets/left-1.css"],
                        &["shared"],
                        &[],
                    ),
                ),
                (
                    "shared".into(),
                    node("assets/shared.js", &["assets/shared.css"], &[], &[]),
                ),
                (
                    "lazy".into(),
                    node(
                        "assets/lazy.js",
                        &["assets/shared.css", "assets/lazy.css"],
                        &["shared"],
                        &[],
                    ),
                ),
                ("other".into(), node("assets/other.css", &[], &[], &[])),
            ])
        }
        #[test]
        fn static_dfs_preserves_dependency_and_css_array_order_and_first_path_occurrence() {
            let (build, graph) = (Build::new(), graph());
            build.populate(&graph);
            let snapshot = Snapshot::load(&build.0, &graph, "app").unwrap();
            assert_eq!(
                snapshot.styles,
                [
                    "assets/shared.css",
                    "assets/left-2.css",
                    "assets/left-1.css",
                    "assets/right.css",
                    "assets/app.css"
                ]
            );
            let before = snapshot.links("app").unwrap();
            assert!(!before.contains("lazy"));
            assert!(snapshot.asset("assets/lazy.css").is_some());
            // Serving shared/lazy bytes cannot mutate or append to the initial cascade.
            for path in ["assets/shared.css", "assets/lazy.css", "assets/shared.css"] {
                snapshot.asset(path).unwrap();
            }
            assert_eq!(snapshot.links("app").unwrap(), before);
            assert_eq!(before.matches("assets/shared.css").count(), 1);
            assert!(snapshot.asset("assets/other.css").is_none());
            let other = Snapshot::load(&build.0, &graph, "other").unwrap();
            assert_eq!(other.styles, ["assets/other.css"]);
            assert!(other.script.is_none());
            assert_eq!(other.files.len(), 1);
        }
        #[test]
        fn snapshots_have_strict_mime_no_store_and_no_request_filesystem_dependency() {
            let (build, mut graph) = (Build::new(), graph());
            graph
                .get_mut("app")
                .unwrap()
                .assets
                .push("assets/mark.svg".into());
            build.populate(&graph);
            let snapshot = Snapshot::load(&build.0, &graph, "app").unwrap();
            assert_eq!(
                snapshot.asset("assets/app.js").unwrap().mime,
                "text/javascript; charset=utf-8"
            );
            assert_eq!(
                snapshot.asset("assets/shared.css").unwrap().mime,
                "text/css; charset=utf-8"
            );
            assert_eq!(
                snapshot.asset("assets/mark.svg").unwrap().mime,
                "image/svg+xml"
            );
            assert_eq!(snapshot.cache_control(), "no-store");
            assert!(snapshot.links("bad\"<script>").is_err());
            fs::remove_dir_all(build.0.join("assets")).unwrap();
            assert_eq!(
                snapshot.asset("assets/app.js").unwrap().bytes,
                b"assets/app.js"
            );
            assert!(snapshot.asset("assets/app.js?secret=1").is_none());
        }
        #[test]
        fn refuses_missing_and_cyclic_static_or_dynamic_edges() {
            for dynamic in [false, true] {
                for cycle in [false, true] {
                    let mut graph = graph();
                    let edges = if dynamic {
                        &mut graph.get_mut("shared").unwrap().dynamic
                    } else {
                        &mut graph.get_mut("shared").unwrap().imports
                    };
                    edges.push(if cycle { "app" } else { "missing" }.into());
                    assert_eq!(
                        validate(&graph).unwrap_err(),
                        if cycle {
                            "asset graph cycle"
                        } else {
                            "missing graph reference"
                        }
                    );
                }
            }
            let build = Build::new();
            let graph = graph();
            build.populate(&graph);
            assert!(Snapshot::load(&build.0, &graph, "missing").is_err());
            fs::remove_file(build.0.join("assets/lazy.css")).unwrap();
            assert!(Snapshot::load(&build.0, &graph, "app").is_err());
        }
        #[test]
        fn refuses_traversal_html_urls_queries_and_finite_type_violations() {
            for path in [
                "",
                "../a.css",
                "/a.css",
                "assets/../a.css",
                "https://evil/a.css",
                "//evil/a.css",
                "assets/%2e/a.css",
                "a\\b.css",
                "a\n.css",
                "a.css?x",
                "a.css#x",
                "a//b.css",
                "a\".css",
                "a.png",
                "a.CSS",
                "a.wasm",
                "a.html",
            ] {
                assert!(mime(path).is_err(), "{path:?}");
            }
            let mut g = graph();
            g.get_mut("app").unwrap().styles.push("x.js".into());
            assert!(validate(&g).is_err());
            let mut g = graph();
            g.get_mut("shared").unwrap().file = "shared.svg".into();
            assert!(validate(&g).is_err());
            let mut g = graph();
            g.get_mut("app").unwrap().file = "app.css".into();
            assert!(validate(&g).is_err());
            let mut g = graph();
            g.insert(
                "../key".into(),
                Node {
                    file: "a.css".into(),
                    ..Node::default()
                },
            );
            assert!(validate(&g).is_err());
        }
        #[test]
        fn bounds_graph_files_edges_individual_files_and_total_snapshot() {
            let build = Build::new();
            let g: Graph = (0..=MAX_NODES)
                .map(|i| (format!("n{i}"), node("a.css", &[], &[], &[])))
                .collect();
            assert_eq!(validate(&g).unwrap_err(), "invalid graph node count");
            let mut g = Graph::from([("app".into(), node("app.js", &[], &[], &[]))]);
            g.get_mut("app").unwrap().styles =
                (0..MAX_FILES).map(|i| format!("a{i}.css")).collect();
            assert_eq!(validate(&g).unwrap_err(), "too many asset files");
            g.get_mut("app").unwrap().styles = vec!["a.css".into(); MAX_EDGES + 1];
            assert_eq!(validate(&g).unwrap_err(), "too many graph edges");
            let g = Graph::from([("app".into(), node("app.css", &[], &[], &[]))]);
            build.write("app.css", b"css");
            File::create(build.0.join("app.css"))
                .unwrap()
                .set_len(MAX_FILE + 1)
                .unwrap();
            assert!(matches!(
                Snapshot::load(&build.0, &g, "app"),
                Err("build file exceeds size limit")
            ));
            let mut g = Graph::from([("app".into(), node("app.js", &[], &[], &[]))]);
            build.write("app.js", b"js");
            for i in 0..9 {
                let path = format!("large{i}.css");
                build.write(&path, b"");
                File::create(build.0.join(&path))
                    .unwrap()
                    .set_len(MAX_FILE)
                    .unwrap();
                g.get_mut("app").unwrap().styles.push(path);
            }
            assert!(matches!(
                Snapshot::load(&build.0, &g, "app"),
                Err("asset snapshot exceeds total limit")
            ));
        }
        #[test]
        fn rejects_nonregular_and_relative_roots() {
            let build = Build::new();
            fs::create_dir(build.0.join("directory.css")).unwrap();
            assert_eq!(
                read(&build.0, "directory.css", MAX_FILE).unwrap_err(),
                "build file must be regular"
            );
            assert!(root(Path::new("relative")).is_err());
            assert!(read(&build.0, "missing.css", MAX_FILE).is_err());
        }
        #[cfg(unix)]
        #[test]
        fn rejects_symlinks_even_inside_tree_and_manifest_or_directory_escapes() {
            use std::os::unix::fs::symlink;
            let build = Build::new();
            let outside = Build::new();
            outside.write("outside.css", b"outside");
            build.write("inside.css", b"inside");
            for (link, target) in [
                ("escape.css", outside.0.join("outside.css")),
                ("alias.css", build.0.join("inside.css")),
                ("directory", outside.0.clone()),
            ] {
                symlink(target, build.0.join(link)).unwrap();
                let file = if link == "directory" {
                    "directory/outside.css"
                } else {
                    link
                };
                assert_eq!(
                    read(&build.0, file, MAX_FILE).unwrap_err(),
                    "symlink build path refused"
                );
            }
            fs::create_dir(build.0.join(".vite")).unwrap();
            symlink(
                outside.0.join("outside.css"),
                build.0.join(".vite/manifest.json"),
            )
            .unwrap();
            assert_eq!(
                read(&build.0, ".vite/manifest.json", MAX_MANIFEST).unwrap_err(),
                "symlink build path refused"
            );
            assert!(root(&build.0.join("directory")).is_err());
        }
        #[test]
        fn bounds_manifest_reads_before_json_parse() {
            let build = Build::new();
            build.write(".vite/manifest.json", b"{}");
            File::create(build.0.join(".vite/manifest.json"))
                .unwrap()
                .set_len(MAX_MANIFEST + 1)
                .unwrap();
            assert_eq!(
                read(&build.0, ".vite/manifest.json", MAX_MANIFEST).unwrap_err(),
                "build file exceeds size limit"
            );
        }
    }
}
// Vite schema stays at the application boundary, outside the neutral contract.
use self::assets::{Graph, MAX_MANIFEST, MAX_NODES, Node, safe_relative};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
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

fn load_manifest(root: &Path, entry: &str) -> Result<Snapshot, &'static str> {
    let root = assets::root(root)?;
    let bytes = assets::read(&root, ".vite/manifest.json", MAX_MANIFEST)?;
    let chunks: BTreeMap<String, Chunk> =
        serde_json::from_slice(&bytes).map_err(|_| "invalid Vite manifest")?;
    if chunks.is_empty() || chunks.len() > MAX_NODES {
        return Err("invalid Vite record count");
    }
    let selected = chunks.get(entry).ok_or("Vite entry absent")?;
    if !selected.is_entry || !selected.file.ends_with(".js") {
        return Err("expected Vite JavaScript entry");
    }
    let mut graph = Graph::new();
    for (key, chunk) in chunks {
        if chunk
            .src
            .iter()
            .chain(&chunk.name)
            .any(|s| !safe_relative(s))
            || chunk.names.as_ref().is_some_and(|names| {
                names.len() > MAX_NODES || names.iter().any(|s| !safe_relative(s))
            })
        {
            return Err("unsafe Vite metadata");
        }
        if (chunk.is_entry || chunk.is_dynamic_entry) && !chunk.file.ends_with(".js") {
            return Err("invalid Vite script record");
        }
        graph.insert(
            key,
            Node {
                file: chunk.file,
                styles: chunk.css,
                imports: chunk.imports,
                dynamic: chunk.dynamic_imports,
                assets: chunk.assets,
            },
        );
    }
    Snapshot::load(&root, &graph, entry)
}

pub(crate) use assets::Snapshot;
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::{io, sync::Arc};
use yetlet_core::View;
use yetlet_server_axum::{DEFAULT_CSP, render_document};

impl Snapshot {
    pub(crate) fn load_from_env() -> io::Result<Arc<Self>> {
        let selected = std::env::var_os("YETLET_ASSET_ROOT").ok_or_else(|| {
            io::Error::other("Set absolute YETLET_ASSET_ROOT after explicitly building dist")
        })?;
        let snapshot = load_manifest(Path::new(&selected), "app.ts").map_err(|error| {
            io::Error::other(format!(
                "Build dist explicitly and select a trusted asset root: {error}"
            ))
        })?;
        if snapshot.images().len() != 1 {
            return Err(io::Error::other(
                "Expected one static SVG in the Vite build",
            ));
        }
        Ok(Arc::new(snapshot))
    }

    pub(crate) fn page(&self, title: &str, view: View) -> Response {
        let mut response = match render_document(title, &view) {
            Ok(html) => {
                // Renderer escapes the title and body; links contain only validated paths.
                let tags = self.links("static").expect("fixed safe mount");
                Html(html.replacen("</head>", &format!("{tags}</head>"), 1)).into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the page",
            )
                .into_response(),
        };
        headers(&mut response, self.cache_control());
        response
    }
}

fn headers(response: &mut Response, cache: &'static str) {
    for (name, value) in [
        (header::CONTENT_SECURITY_POLICY, DEFAULT_CSP),
        (header::CACHE_CONTROL, cache),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ] {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
}

async fn asset(State(snapshot): State<Arc<Snapshot>>, uri: Uri) -> Response {
    // Use the original URI spelling, not a decoded filesystem path. No request I/O.
    let selected = uri
        .query()
        .is_none()
        .then(|| {
            uri.path()
                .strip_prefix("/static/")
                .and_then(|path| snapshot.asset(path))
        })
        .flatten();
    let mut response = match selected {
        Some(asset) => ([(header::CONTENT_TYPE, asset.mime)], asset.bytes.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    headers(&mut response, snapshot.cache_control());
    response
}

pub(crate) fn router(snapshot: Arc<Snapshot>) -> Router {
    Router::new()
        .route("/static/{*path}", get(asset))
        .with_state(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use serde_json::json;
    use std::fs;
    use tower::ServiceExt;

    #[test]
    fn vite_schema_entry_metadata_and_references_fail_closed() {
        let build = tempfile::tempdir().unwrap();
        fs::create_dir(build.path().join(".vite")).unwrap();
        fs::create_dir(build.path().join("assets")).unwrap();
        fs::write(build.path().join("assets/app.js"), "export {};").unwrap();
        let valid = json!({"app.ts": {"file": "assets/app.js", "src": "app.ts", "isEntry": true}});
        let write = |value: &serde_json::Value| {
            fs::write(build.path().join(".vite/manifest.json"), value.to_string()).unwrap();
        };
        write(&valid);
        assert!(load_manifest(build.path(), "app.ts").is_ok());
        for (field, bad) in [
            ("isEntry", json!(false)),
            ("src", json!("../secret")),
            ("names", json!(["https://evil.test/a.js"])),
            ("file", json!("assets/../app.js")),
            ("file", json!("assets/app.css")),
            ("imports", json!(["absent"])),
            ("dynamicImports", json!(["app.ts"])),
            ("css", json!(["assets/app.js"])),
            ("assets", json!(["data:image/svg+xml,x"])),
            ("unknown", json!(true)),
        ] {
            let mut value = valid.clone();
            value["app.ts"][field] = bad;
            write(&value);
            assert!(load_manifest(build.path(), "app.ts").is_err(), "{field}");
        }
    }

    #[tokio::test]
    async fn actual_build_has_static_lazy_assets_strict_routes_and_escaped_ssr() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("dist");
        let snapshot = Arc::new(load_manifest(&root, "app.ts").expect("explicit npm build first"));
        assert_eq!(snapshot.images().len(), 1);
        let links = snapshot.links("static").unwrap();
        assert!(links.contains("type=\"module\""));
        assert!(links.matches("rel=\"stylesheet\"").count() >= 2);
        assert!(!links.contains("lazy"));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(".vite/manifest.json")).unwrap()).unwrap();
        assert!(
            manifest["app.ts"]["imports"]
                .as_array()
                .is_some_and(|v| !v.is_empty())
        );
        let mut lazy = false;
        let app = router(snapshot.clone());
        for (key, chunk) in manifest.as_object().unwrap() {
            if chunk["isDynamicEntry"] == true {
                lazy = true;
                assert!(chunk["css"].as_array().is_some_and(|v| !v.is_empty()));
            }
            for path in std::iter::once(chunk["file"].as_str().unwrap()).chain(
                chunk["css"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| v.as_str().unwrap()),
            ) {
                if let Some(asset) = snapshot.asset(path) {
                    let response = app
                        .clone()
                        .oneshot(
                            Request::builder()
                                .uri(format!("/static/{path}"))
                                .body(Body::empty())
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK, "{key}");
                    assert_eq!(response.headers()[header::CONTENT_TYPE], asset.mime);
                    assert_eq!(
                        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
                        "nosniff"
                    );
                    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
                    assert_eq!(
                        response.headers()[header::CONTENT_SECURITY_POLICY],
                        DEFAULT_CSP
                    );
                }
            }
        }
        assert!(lazy);
        for path in [
            "/static/.vite/manifest.json",
            "/static/%2e%2e/Cargo.toml",
            "/static/missing.js",
            "/static/assets/x.js?key=1",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }
        let response = snapshot.page("</title><script>", View::text("Rust <safe & useful>"));
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        let bytes = axum::body::to_bytes(response.into_body(), 16384)
            .await
            .unwrap();
        let html = std::str::from_utf8(&bytes).unwrap();
        assert!(html.contains("&lt;/title&gt;&lt;script&gt;"));
        assert!(html.contains("Rust &lt;safe &amp; useful&gt;"));
        for forbidden in ["<style", " style=", "data:", "https://", "<script>"] {
            assert!(!html.contains(forbidden));
        }
    }
}
