use std::{collections::BTreeMap, ffi::OsString, fs};
use yetlet_xtask::{
    build::{Build, Options, digest},
    css, parse, repository_root, tooling,
};
#[test]
fn exact_css_outputs_match_tracked_legacy_bytes() {
    let root = repository_root();
    for (folder, prefix) in [
        ("examples/form-css-proof", "/form-proof-assets/"),
        ("examples/lab-operations-board", "/board-assets/"),
    ] {
        let expected = css::expected(&root, folder, prefix).unwrap();
        assert_eq!(expected.len(), 25);
        for (name, bytes) in expected {
            assert_eq!(bytes, fs::read(root.join(&name)).unwrap(), "{name}");
        }
    }
}
#[test]
fn svg_xml_and_css_references_fail_closed() {
    for svg in [
        "<svg><script/></svg>",
        "<svg onload='x'/>",
        "<svg><path href='remote'/></svg>",
        "<svg><path style='x'/></svg>",
        "<svg><path fill='url(http://evil)'/></svg>",
        "<svg><foreignObject/></svg>",
        "<svg><path bogus='x'/></svg>",
        "<!DOCTYPE svg><svg/>",
        "<svg>",
        "<html/>",
        "<svg xmlns='http://evil'/>",
    ] {
        assert!(css::validate_svg(svg.as_bytes()).is_err(), "{svg}");
    }
    for css in [
        "url(data:text/javascript,evil)",
        "@import 'https://evil'",
        "url(https://evil)",
        "url(/proof/unknown.svg)",
        "URL(//evil)",
        "u\\72l(evil)",
    ] {
        assert!(css::externalize(css, "/proof/").is_err(), "{css}");
    }
    let (css,outputs,count)=css::externalize("url(\"data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%3E%3Cpath%20d='M0%200'/%3E%3C/svg%3E\")","/proof/").unwrap();
    assert_eq!(count, 1);
    assert_eq!(outputs.len(), 1);
    assert!(!css.contains("data:"));
    assert!(css.starts_with("url(\"/proof/icon-"));
}
#[test]
fn readonly_check_rejects_missing_extra_stale_integrity_and_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let folder = "examples/form-css-proof";
    for name in [
        "assets/upstream/bootstrap.min.css",
        "assets/upstream/LICENSE.bootstrap",
        "assets/app.css",
        "assets/enhance.js",
    ] {
        let dest = root.join(folder).join(name);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(repository_root().join(folder).join(name), dest).unwrap();
    }
    let options = Options {
        check: true,
        ..Default::default()
    };
    assert!(css::run(root, Build::FormCssProof, options).is_err());
    assert!(!root.join(folder).join("assets/generated").exists());
    css::run(root, Build::FormCssProof, Options::default()).unwrap();
    let manifest = root.join(folder).join("assets/manifest.json");
    let before = fs::metadata(&manifest).unwrap().modified().unwrap();
    css::run(root, Build::FormCssProof, options).unwrap();
    assert_eq!(fs::metadata(&manifest).unwrap().modified().unwrap(), before);
    let extra = root.join(folder).join("assets/generated/extra.js");
    fs::write(&extra, b"x").unwrap();
    assert!(css::run(root, Build::FormCssProof, options).is_err());
    fs::remove_file(extra).unwrap();
    fs::write(&manifest, b"stale").unwrap();
    assert!(css::run(root, Build::FormCssProof, options).is_err());
    fs::write(
        root.join(folder).join("assets/upstream/bootstrap.min.css"),
        b"tampered",
    )
    .unwrap();
    assert!(css::expected(root, folder, "/form-proof-assets/").is_err());
    for name in ["../escape", "/absolute", "a/../../escape"] {
        assert!(tooling::safe(root, name, false).is_err());
    }
}
#[cfg(unix)]
#[test]
fn output_symlinks_and_input_escape_are_rejected() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    symlink(other.path(), temp.path().join("escape")).unwrap();
    assert!(tooling::safe(temp.path(), "escape/file", true).is_err());
    assert!(tooling::safe(temp.path(), "escape/file", false).is_err());
}
#[test]
fn version_locks_require_one_exact_version_and_profiles_are_strict() {
    let temp = tempfile::tempdir().unwrap();
    for lock in [
        "package=[]",
        "[[package]]\nname='wasm-bindgen'\nversion='1'\n[[package]]\nname='wasm-bindgen'\nversion='2'",
    ] {
        fs::write(temp.path().join("Cargo.lock"), lock).unwrap();
        assert!(tooling::lock_version(temp.path(), "Cargo.lock").is_err());
    }
    for argv in [
        vec!["build", "browser", "--check"],
        vec!["build", "scene-island", "--release"],
        vec!["build", "board-assets", "--fetch", "--check"],
        vec!["build", "browser", "--release", "--release"],
        vec!["check", "docs", "--release"],
    ] {
        assert!(parse(argv.into_iter().map(OsString::from)).is_err());
    }
    let mut files = BTreeMap::new();
    files.insert("a".into(), b"one".to_vec());
    files.insert("b".into(), b"two".to_vec());
    assert_eq!(digest(b"test\0", &files), "a4edda57a32019bd");
}

#[test]
fn cargo_proxy_keeps_its_invocation_identity() {
    let cargo = tooling::tool("cargo", ".cargo/bin").unwrap();
    assert_eq!(std::path::Path::new(&cargo).file_stem().unwrap(), "cargo");
}

#[cfg(unix)]
#[test]
fn canonical_build_preserves_child_failures_and_exact_version_rejection() {
    use std::{os::unix::fs::PermissionsExt, process::Command};
    let fixture = tempfile::tempdir().unwrap();
    let cargo = fixture.path().join("cargo");
    let bindgen = fixture.path().join("wasm-bindgen");
    // Fixed tool fixtures only. The product runner invokes literal executables, no shell command.
    fs::write(
        &cargo,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'cargo 1.99.0'; exit 0; fi\nexit 37\n",
    )
    .unwrap();
    fs::write(&bindgen, "#!/bin/sh\necho 'wasm-bindgen 0.2.129'\n").unwrap();
    for path in [&cargo, &bindgen] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["build", "render-bench"])
        .env("PATH", fixture.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(37));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("origin=child native_exit_code=Some(37)"),
        "{stderr}"
    );
    assert!(stderr.contains("cleanup_ok=true"));
    fs::write(&bindgen, "#!/bin/sh\necho 'wasm-bindgen 0.2.1299'\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_yetlet-xtask"))
        .args(["build", "browser"])
        .env("PATH", fixture.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("exactly match"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("cargo build"));
    assert!(
        tooling::tool(
            "yetlet-nonexistent-external-tool-yt084",
            ".local/share/yetlet-tools/bin"
        )
        .is_err()
    );
    let lock = tempfile::tempdir().unwrap();
    fs::write(
        lock.path().join("Cargo.lock"),
        "[[package]]\nname='wasm-bindgen'\nversion='0.2.129'\n",
    )
    .unwrap();
    fs::create_dir_all(lock.path().join("examples/webgpu-scene-island")).unwrap();
    fs::write(
        lock.path().join("examples/webgpu-scene-island/Cargo.lock"),
        "[[package]]\nname='wasm-bindgen'\nversion='0.2.128'\n",
    )
    .unwrap();
    assert!(
        yetlet_xtask::build::preflight(lock.path(), Build::SceneIsland)
            .unwrap_err()
            .message
            .contains("disagree")
    );
}
