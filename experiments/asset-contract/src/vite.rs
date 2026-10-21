//! Vite schema stays at the application boundary, outside the neutral contract.
use crate::assets::{self, Graph, MAX_MANIFEST, MAX_NODES, Node, Snapshot, safe_relative};
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

pub(crate) fn load(root: &Path, entry: &str) -> Result<Snapshot, &'static str> {
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
