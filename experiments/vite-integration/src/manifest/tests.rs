use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Build(PathBuf);

impl Build {
    fn new() -> Self {
        static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "yetlet-vite-unit-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("assets")).unwrap();
        fs::create_dir(root.join(".vite")).unwrap();
        for file in [
            "app.js",
            "branch.js",
            "base.js",
            "lazy.js",
            "app.css",
            "branch.css",
            "base.css",
            "lazy.css",
            "mark.svg",
        ] {
            fs::write(root.join("assets").join(file), file).unwrap();
        }
        let build = Self(root);
        build.manifest(&valid());
        build
    }

    fn manifest(&self, value: &Value) {
        fs::write(
            self.0.join(".vite/manifest.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
    }

    fn load(&self) -> Result<Snapshot, &'static str> {
        Snapshot::load(&self.0, "front/plain.ts")
    }
}

impl Drop for Build {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn valid() -> Value {
    json!({
        "front/plain.ts": {
            "file": "assets/app.js", "src": "front/plain.ts", "isEntry": true,
            "imports": ["_branch.js", "_base.js"], "dynamicImports": ["front/lazy.ts"],
            "css": ["assets/app.css", "assets/base.css"]
        },
        "_branch.js": {"file": "assets/branch.js", "imports": ["_base.js"], "css": ["assets/branch.css"]},
        "_base.js": {"file": "assets/base.js", "css": ["assets/base.css"], "assets": ["assets/mark.svg"]},
        "front/lazy.ts": {"file": "assets/lazy.js", "isDynamicEntry": true, "css": ["assets/lazy.css"]},
        "front/mark.svg": {"file": "assets/mark.svg", "src": "front/mark.svg"}
    })
}

#[test]
fn snapshots_valid_graph_dependency_css_order_and_dedup() {
    let build = Build::new();
    let snapshot = build.load().unwrap();
    assert_eq!(
        snapshot.styles,
        ["assets/base.css", "assets/branch.css", "assets/app.css"]
    );
    assert_eq!(snapshot.files.len(), 9);
    assert_eq!(snapshot.svg, "assets/mark.svg");
    assert_eq!(
        snapshot.files["assets/lazy.css"].mime,
        "text/css; charset=utf-8"
    );
    assert_eq!(
        snapshot.files["assets/app.js"].mime,
        "text/javascript; charset=utf-8"
    );
    assert_eq!(snapshot.files["assets/mark.svg"].mime, "image/svg+xml");
    fs::write(build.0.join("assets/app.js"), "changed after startup").unwrap();
    assert_eq!(snapshot.files["assets/app.js"].bytes, b"app.js");
    assert!(!snapshot.files.contains_key(".vite/manifest.json"));
}

#[test]
fn rejects_missing_static_and_dynamic_references() {
    for field in ["imports", "dynamicImports"] {
        let mut value = valid();
        value["front/plain.ts"][field] = json!(["_absent.js"]);
        assert_eq!(
            parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
            "missing manifest reference"
        );
    }
}

#[test]
fn rejects_static_and_dynamic_cycles() {
    for field in ["imports", "dynamicImports"] {
        let mut value = valid();
        value["_base.js"][field] = json!(["front/plain.ts"]);
        assert_eq!(
            parse(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
            "manifest import cycle"
        );
    }
}

#[test]
fn rejects_unsafe_paths_and_asset_types() {
    for path in [
        "../app.js",
        "/assets/app.js",
        "assets/../app.js",
        "https://evil.test/app.js",
        "//evil.test/app.js",
        "assets/%2e%2e/app.js",
        "assets/a\\b.js",
        "assets/a\n.js",
        "assets/a.js?x",
        "assets/a.js#x",
        "assets//a.js",
        "assets/a.png",
    ] {
        let mut value = valid();
        value["front/plain.ts"]["file"] = json!(path);
        assert!(
            parse(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{path:?}"
        );
    }
}

#[test]
fn rejects_bad_schemas_metadata_and_reference_types() {
    for bytes in [
        b"[]".as_slice(),
        b"{}",
        b"null",
        b"{",
        br#"{"app":{"file":5}}"#,
    ] {
        assert!(parse(bytes).is_err());
    }
    for (field, bad) in [
        ("imports", json!("_base.js")),
        ("css", json!(["assets/app.js"])),
        ("isEntry", json!("true")),
        ("surprise", json!(true)),
        ("src", json!("../secret")),
        ("assets", json!(["data:image/svg+xml,x"])),
    ] {
        let mut value = valid();
        value["front/plain.ts"][field] = bad;
        assert!(
            parse(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut value = valid();
    value["_base.js"]["file"] = json!("assets/base.css");
    assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn rejects_missing_files_manifest_and_entry() {
    let build = Build::new();
    fs::remove_file(build.0.join("assets/lazy.css")).unwrap();
    assert!(matches!(build.load(), Err("build asset missing")));
    fs::remove_file(build.0.join(".vite/manifest.json")).unwrap();
    assert!(matches!(build.load(), Err("manifest unavailable")));
    let build = Build::new();
    assert!(Snapshot::load(&build.0, "front/absent.ts").is_err());
}

#[test]
fn enforces_manifest_and_file_bounds() {
    assert!(parse(&vec![b' '; MAX_MANIFEST as usize + 1]).is_err());
    let build = Build::new();
    File::create(build.0.join("assets/app.js"))
        .unwrap()
        .set_len(MAX_FILE + 1)
        .unwrap();
    assert!(matches!(build.load(), Err("build file exceeds size limit")));
    let chunks: BTreeMap<String, Value> = (0..=MAX_RECORDS)
        .map(|i| (format!("chunk{i}"), json!({"file": "assets/app.js"})))
        .collect();
    assert!(parse(&serde_json::to_vec(&chunks).unwrap()).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_asset_symlink_outside_build_directory() {
    let build = Build::new();
    let outside = Build::new();
    fs::remove_file(build.0.join("assets/app.js")).unwrap();
    std::os::unix::fs::symlink(
        outside.0.join("assets/app.js"),
        build.0.join("assets/app.js"),
    )
    .unwrap();
    assert!(matches!(
        build.load(),
        Err("build asset escapes build directory")
    ));
}

#[test]
fn loads_both_actual_vite_profiles_without_a_listener() {
    for profile in ["plain", "tailwind"] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("dist")
            .join(profile);
        let snapshot = Snapshot::load(&root, &format!("front/{profile}.ts")).unwrap();
        let html = crate::document(profile, &snapshot).unwrap();
        assert!(snapshot.styles.len() >= 2);
        assert!(
            snapshot
                .files
                .keys()
                .any(|file| file.contains("lazy") && file.ends_with(".js"))
        );
        assert!(
            snapshot
                .files
                .keys()
                .any(|file| file.contains("lazy") && file.ends_with(".css"))
        );
        assert!(!snapshot.styles.iter().any(|file| file.contains("lazy")));
        assert!(html.contains("&lt;safe &amp; useful&gt;"));
        assert!(html.contains("type=\"module\""));
        assert!(!html.contains("<style"));
        assert!(!html.contains("style="));
        assert!(!html.contains("data:"));
        if let Ok(secret) = std::env::var("BACKEND_ONLY_SECRET") {
            assert!(!secret.is_empty());
            assert!(!html.contains(&secret));
            for asset in snapshot.files.values() {
                assert!(
                    !asset
                        .bytes
                        .windows(secret.len())
                        .any(|bytes| bytes == secret.as_bytes())
                );
            }
        }
    }
}
