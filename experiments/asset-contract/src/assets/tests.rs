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
    g.get_mut("app").unwrap().styles = (0..MAX_FILES).map(|i| format!("a{i}.css")).collect();
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
