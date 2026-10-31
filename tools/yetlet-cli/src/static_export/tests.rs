use super::*;
use crate::{Cli, parse};
use serde_json::json;
use std::ffi::OsString;

fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}
fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::create_project(tmp.path(), "static-fixture", &[]).unwrap();
    let root = tmp.path().join("static-fixture");
    fs::create_project(&root, "fragments", &[]).unwrap();
    fs::create_project(&root, "assets", &[]).unwrap();
    std::fs::write(
        root.join("fragments/home.html"),
        "<main>Readable &amp; safe 界</main>",
    )
    .unwrap();
    std::fs::write(root.join("assets/site.css"), "main{color:navy}").unwrap();
    descriptor(
        &root,
        json!({"schema_version":1,"pages":[{"route":"/","title":"<Title & 界>","language":"en-US","fragment":"fragments/home.html","stylesheets":["assets/site.css"],"island_entry":null}],"assets":["assets/site.css"]}),
    );
    (tmp, root)
}
fn descriptor(root: &Path, value: serde_json::Value) {
    std::fs::write(
        root.join("export.json"),
        serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
}
fn value(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(root.join("export.json")).unwrap()).unwrap()
}

#[test]
fn yt223_parser_contract_preserves_ordinary_build() {
    assert_eq!(
        parse(&args(&["build", "--release"])),
        Ok(Cli::Build {
            manifest: "Cargo.toml".into(),
            release: true
        })
    );
    assert_eq!(
        parse(&args(&[
            "build",
            "--export-bin",
            "site-export",
            "--static",
            "--output",
            "artifacts/site-v1",
            "--base-path=/yetlet/",
            "--release",
            "--wasm-islands"
        ])),
        Ok(Cli::BuildStatic {
            manifest: "Cargo.toml".into(),
            export_bin: "site-export".into(),
            output: "artifacts/site-v1".into(),
            base_path: "/yetlet/".into(),
            release: true,
            wasm_islands: true
        })
    );
    for a in [
        vec!["run", "--static"],
        vec!["generate", "static"],
        vec!["build", "--output", "dist"],
        vec!["build", "--static"],
        vec!["build", "--static", "--static"],
        vec!["build", "--static=true"],
        vec!["build", "--export-bin=-unsafe"],
        vec!["build", "--base-path", "//host/"],
        vec!["build", "--wasm-islands"],
    ] {
        assert!(parse(&args(&a)).is_err(), "{a:?}");
    }
    match parse(&args(&["build", "--help"])).unwrap() {
        Cli::Help(h) => {
            assert!(h.contains("--static"));
            assert!(h.contains("create-only"));
            assert!(h.contains("not plan static"));
        }
        _ => panic!(),
    }
}
#[test]
fn yt223_metadata_csp_and_project_urls() {
    let (_tmp, root) = fixture();
    let prepared = protocol::prepare(&root, "/yetlet/", false).unwrap();
    let html = std::str::from_utf8(&prepared.files["index.html"]).unwrap();
    assert!(html.starts_with("<!doctype html><html lang=\"en-US\">"));
    assert!(html.contains("&lt;Title &amp; 界&gt;"));
    assert!(html.contains("/yetlet/assets/site.css"));
    assert!(html.contains("Readable &amp; safe 界"));
    assert!(html.contains("script-src &#39;none&#39;"));
    assert!(!html.contains("<script"));
    assert!(html.find("Content-Security-Policy").unwrap() < html.find("<link").unwrap());
    assert_eq!(prepared.files.len(), 3);
    assert!(prepared.files.contains_key(".nojekyll"));
    assert!(!prepared.files.contains_key("export.json"));
}
#[test]
fn yt223_descriptor_keys_routes_and_limits_fail_closed() {
    let (_tmp, root) = fixture();
    let original = value(&root);
    for route in [
        "/Guide/", "/../", "/guide", "/assets/", "/con/", "/a%2fb/", "//x/", "/x/?q=y", "/x\\y/",
    ] {
        let mut d = original.clone();
        d["pages"][0]["route"] = json!(route);
        descriptor(&root, d);
        assert!(protocol::prepare(&root, "/", false).is_err(), "{route}");
    }
    for d in [
        json!({"schema_version":2,"pages":[],"assets":[]}),
        {
            let mut d = original.clone();
            d["pages"][0]
                .as_object_mut()
                .unwrap()
                .remove("island_entry");
            d
        },
        {
            let mut d = original.clone();
            d["unknown"] = json!(true);
            d
        },
        {
            let mut d = original.clone();
            d["pages"]
                .as_array_mut()
                .unwrap()
                .push(original["pages"][0].clone());
            d
        },
    ] {
        descriptor(&root, d);
        assert!(protocol::prepare(&root, "/", false).is_err());
    }
    let raw = serde_json::to_string(&original).unwrap().replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    std::fs::write(root.join("export.json"), raw).unwrap();
    assert!(protocol::prepare(&root, "/", false).is_err());
    descriptor(&root, original);
    std::fs::write(
        root.join("fragments/home.html"),
        vec![b'a'; 2 * 1024 * 1024 + 1],
    )
    .unwrap();
    assert!(protocol::prepare(&root, "/", false).is_err());
}
#[test]
fn yt223_assets_islands_and_unlisted_directories() {
    let (_tmp, root) = fixture();
    std::fs::write(root.join("assets/entry.mjs"), "export const ready=true;").unwrap();
    let mut d = value(&root);
    d["assets"]
        .as_array_mut()
        .unwrap()
        .push(json!("assets/entry.mjs"));
    d["pages"][0]["island_entry"] = json!("assets/entry.mjs");
    descriptor(&root, d.clone());
    assert!(protocol::prepare(&root, "/", false).is_err());
    let prepared = protocol::prepare(&root, "/", true).unwrap();
    let html = String::from_utf8(prepared.files["index.html"].clone()).unwrap();
    assert!(html.contains("type=\"module\" src=\"/assets/entry.mjs\""));
    assert!(html.contains("wasm-unsafe-eval"));
    std::fs::create_dir(root.join("unlisted")).unwrap();
    assert!(protocol::prepare(&root, "/", true).is_err());
    std::fs::remove_dir(root.join("unlisted")).unwrap();
    d["pages"][0]["island_entry"] = json!("assets/missing.mjs");
    descriptor(&root, d);
    assert!(protocol::prepare(&root, "/", true).is_err());
}
#[test]
fn yt223_no_replace_binary_roundtrip_and_cleanup() {
    let (_tmp, root) = fixture();
    let parent = fs::open_directory(&root).unwrap();
    let stage = files::new_directory(&parent, "stage").unwrap();
    files::write(
        fs::app_root(&stage),
        "guide/index.html",
        b"<main>safe</main>",
    )
    .unwrap();
    files::write(fs::app_root(&stage), "assets/pixel.png", &[0, 255, 7]).unwrap();
    files::publish(stage, &parent, "site".as_ref()).unwrap();
    assert_eq!(
        files::read(&root.join("site"), "assets/pixel.png", 10).unwrap(),
        [0, 255, 7]
    );
    assert!(files::absent(&parent, "site".as_ref()).is_err());
    let other = files::new_directory(&parent, "other").unwrap();
    assert!(files::publish(other, &parent, "site".as_ref()).is_err());
    assert_eq!(
        files::read(&root.join("site"), "guide/index.html", 100).unwrap(),
        b"<main>safe</main>"
    );
    files::cleanup(&root.join("other")).unwrap();
    files::cleanup(&root.join("site")).unwrap();
    assert!(!root.join("site").exists());
}
#[test]
fn yt223_hard_link_and_symlink_reads_refused() {
    let (_tmp, root) = fixture();
    std::fs::hard_link(root.join("assets/site.css"), root.join("linked.css")).unwrap();
    assert!(files::read(&root, "linked.css", 100).is_err());
    assert!(files::read(&root, "assets/site.css", 100).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("fragments", root.join("alias")).unwrap();
        assert!(files::read(&root, "alias/home.html", 100).is_err());
    }
}
#[test]
fn yt223_command_uses_literal_protocol_and_frozen_cargo() {
    let cargo = files::CargoDirectories {
        target: "target".into(),
        build: "target".into(),
    };
    let cmd = command(
        "site-export",
        true,
        Path::new("private export"),
        "/yetlet/",
        true,
        &cargo,
    );
    let args: Vec<_> = cmd
        .get_args()
        .map(|x| x.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        args,
        [
            "run",
            "--frozen",
            "--manifest-path",
            "Cargo.toml",
            "--release",
            "--target-dir",
            "target",
            "--bin",
            "site-export",
            "--",
            "--yetlet-static-export",
            "1",
            "--output",
            "private export",
            "--base-path",
            "/yetlet/",
            "--wasm-islands"
        ]
    );
}
