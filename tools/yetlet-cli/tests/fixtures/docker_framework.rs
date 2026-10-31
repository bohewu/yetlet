//! Closed published SDK fixture using the compiled source closure.
use std::{collections::BTreeSet, fs, path::Path};
#[path = "../../src/framework_files.rs"]
mod files;
pub fn create(root: &Path) {
    fs::create_dir_all(root).unwrap();
    let mut value = include_str!("../../resources/workspace.toml")
        .parse::<toml::Value>()
        .unwrap();
    value["workspace"]["members"] =
        toml::Value::Array(vec![toml::Value::String("crates/yetlet".into())]);
    value["workspace"].as_table_mut().unwrap().remove("exclude");
    fs::write(root.join("Cargo.toml"), toml::to_string(&value).unwrap()).unwrap();
    for (path, bytes) in files::FILES.iter() {
        let dest = root.join(path);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(dest, bytes).unwrap();
    }
}

#[allow(dead_code)] // Reused by Compose/Kubernetes targets; Docker-only target needs only create.
pub fn normal_dependencies(runtime_settings: bool) -> std::collections::BTreeSet<String> {
    let mut names = BTreeSet::from_iter(["axum", "serde", "tower"].map(str::to_owned));
    if runtime_settings {
        names.extend(["toml", "dotenvy", "libc"].map(str::to_owned));
    }
    names
}
