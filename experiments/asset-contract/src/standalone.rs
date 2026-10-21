//! Standalone compiler output needs no Vite/Node manifest or runtime process.
use crate::assets::{Graph, Node, Snapshot};
use std::path::Path;

pub(crate) fn load(root: &Path) -> Result<Snapshot, &'static str> {
    let graph = Graph::from([(
        "standalone".into(),
        Node {
            file: "assets/standalone.css".into(),
            ..Node::default()
        },
    )]);
    Snapshot::load(root, &graph, "standalone")
}
