//! Native replacements for the five Wasm builders; CSS builders live in css.rs.
use crate::{build_templates::*, tooling::*};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Build {
    Browser,
    KeyedBindings,
    MultiIslandDashboard,
    RenderBench,
    FormCssProof,
    BoardAssets,
    SceneIsland,
}
impl Build {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "browser" => Self::Browser,
            "keyed-bindings" => Self::KeyedBindings,
            "multi-island-dashboard" => Self::MultiIslandDashboard,
            "render-bench" => Self::RenderBench,
            "form-css-proof" => Self::FormCssProof,
            "board-assets" => Self::BoardAssets,
            "scene-island" => Self::SceneIsland,
            _ => return None,
        })
    }
    pub fn css(self) -> bool {
        matches!(self, Self::FormCssProof | Self::BoardAssets)
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    pub release: bool,
    pub check: bool,
    pub fetch: bool,
}

pub fn preflight(root: &Path, kind: Build) -> Result<(OsString, OsString)> {
    let cargo = tool("cargo", ".cargo/bin")?;
    let bindgen = tool("wasm-bindgen", ".local/share/yetlet-tools/bin")?;
    let version = lock_version(root, "Cargo.lock")?;
    if kind == Build::SceneIsland
        && lock_version(root, "examples/webgpu-scene-island/Cargo.lock")? != version
    {
        return Err(fail("nested/root wasm-bindgen locks disagree"));
    }
    let output = execute(root, &bindgen, &["--version".into()], true)?;
    if std::str::from_utf8(&output).unwrap_or("").trim() != format!("wasm-bindgen {version}") {
        return Err(fail(
            "wasm-bindgen CLI identity/version must exactly match applicable Cargo.lock",
        ));
    }
    let output = execute(root, &cargo, &["--version".into()], true)?;
    if !std::str::from_utf8(&output)
        .unwrap_or("")
        .starts_with("cargo 1.")
    {
        return Err(fail("Cargo identity mismatch"));
    }
    Ok((cargo, bindgen))
}
fn cargo_build(
    root: &Path,
    cargo: &OsStr,
    package: &str,
    release: bool,
    scene: bool,
) -> Result<()> {
    let mut args: Vec<OsString> = [
        "build",
        "--locked",
        "--target",
        "wasm32-unknown-unknown",
        "-p",
        package,
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    if release {
        args.push("--release".into());
    }
    if scene {
        args.extend(["--offline".into(), "--lib".into()]);
        if package == "yetlet-webgpu-scene-island" {
            args.extend([
                "--manifest-path".into(),
                safe(root, "examples/webgpu-scene-island/Cargo.toml", false)?.into_os_string(),
            ]);
        }
    }
    execute(root, cargo, &args, false)?;
    Ok(())
}
fn bind(
    root: &Path,
    bindgen: &OsStr,
    source: &str,
    module: &str,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let stage = tempfile::tempdir()?;
    execute(
        root,
        bindgen,
        &[
            "--target".into(),
            "web".into(),
            "--out-dir".into(),
            stage.path().as_os_str().into(),
            "--out-name".into(),
            module.into(),
            safe(root, source, false)?.into_os_string(),
        ],
        false,
    )?;
    let mut files = BTreeMap::new();
    for name in [format!("{module}.js"), format!("{module}_bg.wasm")] {
        files.insert(name.clone(), fs::read(stage.path().join(name))?);
    }
    stage.close()?;
    Ok(files)
}
pub fn digest(domain: &[u8], files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hash = Sha256::new();
    hash.update(domain);
    for (name, bytes) in files {
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update(bytes);
        hash.update([0]);
    }
    format!("{:x}", hash.finalize())[..16].into()
}
fn assets(
    root: &Path,
    output: &str,
    domain: &[u8],
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<String> {
    let digest = digest(domain, files);
    for (name, data) in files {
        write(root, format!("{output}/assets/{digest}/{name}"), data)?;
    }
    Ok(digest)
}
fn page(template: &str, digest: &str, absolute: bool, ssr: &str) -> String {
    let mut output = template.replace("@@ssr@@", ssr);
    while let Some(start) = output.find("@@") {
        let end = start
            + 2
            + output[start + 2..]
                .find("@@")
                .expect("frozen template placeholder");
        let name = &output[start + 2..end];
        let url = format!("{}assets/{digest}/{name}", if absolute { "/" } else { "" });
        output.replace_range(start..end + 2, &url);
    }
    output
}
pub fn run(root: &Path, kind: Build, options: Options) -> Result<()> {
    if kind.css() {
        return crate::css::run(root, kind, options);
    }
    let (cargo, bindgen) = preflight(root, kind)?;
    let profile = if options.release { "release" } else { "debug" };
    match kind {
        Build::Browser => {
            for package in ["yetlet-todo-browser", "yetlet-search-island"] {
                cargo_build(root, &cargo, package, options.release, false)?;
            }
            let mut files = bind(
                root,
                &bindgen,
                &format!("target/wasm32-unknown-unknown/{profile}/yetlet_todo_browser.wasm"),
                "yetlet_browser",
            )?;
            files.extend(bind(
                root,
                &bindgen,
                &format!("target/wasm32-unknown-unknown/{profile}/yetlet_search_island.wasm"),
                "yetlet_search",
            )?);
            for (name, data) in [
                ("app.js", BROWSER_JS.as_bytes()),
                ("styles.css", BROWSER_CSS.as_bytes()),
                ("favicon.svg", BROWSER_ICON.as_bytes()),
            ] {
                files.insert(name.into(), data.into());
            }
            files.insert(
                "search-loader.js".into(),
                read(root, "assets/search-loader.js")?,
            );
            let d = assets(root, "dist", b"yetlet-v1\0", &files)?;
            let u = |n: &str| format!("/assets/{d}/{n}");
            let manifest = json!({"schema_version":1,"digest":d,"mode":profile,"entry":u("app.js"),"module":u("yetlet_browser.js"),"wasm":u("yetlet_browser_bg.wasm"),"stylesheet":u("styles.css"),"favicon":u("favicon.svg"),"search_loader":u("search-loader.js"),"search_module":u("yetlet_search.js"),"search_wasm":u("yetlet_search_bg.wasm")});
            write(
                root,
                "dist/app.html",
                page(BROWSER_PAGE, &d, true, "").as_bytes(),
            )?;
            write(root, "dist/manifest.json", &json_bytes(&manifest))?;
            println!(
                "PASS browser assets: {d} Wasm bytes: {}",
                files["yetlet_browser_bg.wasm"].len()
            );
        }
        Build::MultiIslandDashboard => {
            for package in [
                "yetlet-dashboard-counter-island",
                "yetlet-dashboard-filter-island",
            ] {
                cargo_build(root, &cargo, package, options.release, false)?;
            }
            let mut files = BTreeMap::new();
            for (key, loader) in [
                ("counter", DASHBOARD_COUNTER_LOADER),
                ("filter", DASHBOARD_FILTER_LOADER),
            ] {
                files.extend(bind(
                    root,
                    &bindgen,
                    &format!(
                        "target/wasm32-unknown-unknown/{profile}/yetlet_dashboard_{key}_island.wasm"
                    ),
                    &format!("yetlet_dashboard_{key}"),
                )?);
                files.insert(format!("{key}-loader.js"), loader.as_bytes().into());
            }
            files.insert("dashboard.css".into(), DASHBOARD_CSS.as_bytes().into());
            let output = "target/multi-island-dashboard-dist";
            let path = safe(root, output, true)?;
            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            let d = assets(root, output, b"yetlet-dashboard-v1\0", &files)?;
            let u = |n: &str| format!("/dashboard-assets/{d}/{n}");
            let manifest = json!({"schema_version":1,"digest":d,"mode":profile,"stylesheet":u("dashboard.css"),"counter":{"loader":u("counter-loader.js"),"module":u("yetlet_dashboard_counter.js"),"wasm":u("yetlet_dashboard_counter_bg.wasm")},"filter":{"loader":u("filter-loader.js"),"module":u("yetlet_dashboard_filter.js"),"wasm":u("yetlet_dashboard_filter_bg.wasm")}});
            write(
                root,
                format!("{output}/manifest.json"),
                &json_bytes(&manifest),
            )?;
            println!("PASS dashboard assets: {d}");
        }
        Build::KeyedBindings | Build::RenderBench => {
            let keyed = kind == Build::KeyedBindings;
            let (package, module, folder, domain, script, bootstrap, styles, template, scope) =
                if keyed {
                    (
                        "yetlet-keyed-bindings",
                        "yetlet_keyed_bindings",
                        "m7b-keyed-bindings",
                        b"yetlet-m7b-keyed-bindings-v1\0".as_slice(),
                        "prototype.js",
                        KEYED_BOOTSTRAP,
                        KEYED_STYLES,
                        KEYED_PAGE,
                        "isolated_m7b_m7c_m7d_keyed_reactivity_lifecycle_prototype",
                    )
                } else {
                    (
                        "yetlet-render-bench",
                        "yetlet_render_bench",
                        "m7a-render-bench",
                        b"yetlet-m7a-render-bench-v1\0".as_slice(),
                        "bench.js",
                        BENCH_BOOTSTRAP,
                        BENCH_STYLES,
                        BENCH_PAGE,
                        "isolated_m7a_render_benchmark",
                    )
                };
            cargo_build(root, &cargo, package, options.release, false)?;
            let mut ssr = String::new();
            if keyed {
                let mut args: Vec<OsString> = [
                    "run",
                    "--quiet",
                    "--locked",
                    "-p",
                    package,
                    "--bin",
                    "render-ssr",
                ]
                .into_iter()
                .map(Into::into)
                .collect();
                if options.release {
                    args.push("--release".into());
                }
                ssr = String::from_utf8(execute(root, &cargo, &args, true)?)
                    .map_err(|_| fail("SSR output not UTF-8"))?
                    .replace("\r\n", "\n");
            }
            let mut files = bind(
                root,
                &bindgen,
                &format!("target/wasm32-unknown-unknown/{profile}/{module}.wasm"),
                module,
            )?;
            files.insert(script.into(), bootstrap.as_bytes().into());
            files.insert("styles.css".into(), styles.as_bytes().into());
            if keyed {
                files.insert("favicon.svg".into(), KEYED_ICON.as_bytes().into());
            }
            let output = format!("target/{folder}/{profile}");
            let d = assets(root, &output, domain, &files)?;
            let u = |n: &str| format!("assets/{d}/{n}");
            let mut manifest = json!({"schema_version":1,"scope":scope,"profile":profile,"digest":d,"entry":u(script),"module":u(&format!("{module}.js")),"wasm":u(&format!("{module}_bg.wasm")),"stylesheet":u("styles.css")});
            if keyed {
                manifest["page"] = json!("index.html");
                manifest["favicon"] = json!(u("favicon.svg"));
            }
            manifest.sort_all_objects();
            write(
                root,
                format!("{output}/index.html"),
                page(template, &d, false, &ssr).as_bytes(),
            )?;
            write(
                root,
                format!("{output}/manifest.json"),
                &json_bytes(&manifest),
            )?;
            println!("PASS isolated build: {profile} {d}");
        }
        Build::SceneIsland => scene(root, &cargo, &bindgen)?,
        _ => unreachable!(),
    }
    check_cancel()
}
fn scene(root: &Path, cargo: &OsStr, bindgen: &OsStr) -> Result<()> {
    let output = "target/webgpu-scene-island-dist";
    let here = "examples/webgpu-scene-island";
    let mut manifest = json!({"schema_version":1});
    for (key, module, package, source) in [
        (
            "scene",
            "scene",
            "yetlet-webgpu-scene-island",
            format!("{here}/target/wasm32-unknown-unknown/release/yetlet_webgpu_scene_island.wasm"),
        ),
        (
            "counter",
            "yetlet_dashboard_counter",
            "yetlet-dashboard-counter-island",
            "target/wasm32-unknown-unknown/release/yetlet_dashboard_counter_island.wasm".into(),
        ),
    ] {
        cargo_build(root, cargo, package, true, true)?;
        let mut files = bind(root, bindgen, &source, module)?;
        files.insert(
            format!("{key}-loader.js"),
            read(root, format!("{here}/{key}-loader.js"))?,
        );
        if key == "scene" {
            files.insert(
                "comparison-loader.js".into(),
                read(root, format!("{here}/comparison-loader.js"))?,
            );
        }
        let d = assets(root, output, b"yt075-v1\0", &files)?;
        manifest[key] = json!({"digest":d,"loader":format!("/scene-assets/{d}/{key}-loader.js"),"module":format!("/scene-assets/{d}/{module}.js"),"wasm":format!("/scene-assets/{d}/{module}_bg.wasm")});
        println!(
            "{key}: {d}; WASM {} bytes",
            files[&format!("{module}_bg.wasm")].len()
        );
    }
    let mut sources = vec![format!("{here}/Cargo.toml"), format!("{here}/Cargo.lock")];
    for (folder, rs_only) in [
        (format!("{here}/src"), true),
        ("crates/yetlet-wgpu/src".into(), false),
    ] {
        if !rs_only {
            sources.extend([
                "crates/yetlet-wgpu/Cargo.toml".into(),
                "crates/yetlet-wgpu/Cargo.lock".into(),
            ]);
        }
        let mut names = fs::read_dir(safe(root, &folder, false)?)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        names.sort();
        for path in names {
            if path.is_file() && (!rs_only || path.extension() == Some(OsStr::new("rs"))) {
                sources.push(
                    path.strip_prefix(root)
                        .map_err(|_| fail("source root mismatch"))?
                        .to_str()
                        .ok_or_else(|| fail("non-UTF-8 source name"))?
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut values = serde_json::Map::new();
    for path in sources {
        if safe(root, &path, false)?.is_file() {
            values.insert(path.clone(), json!(sha(&read(root, path)?)));
        }
    }
    manifest["candidate_sources"] = Value::Object(values);
    // Additive port provenance: keep the original candidate_sources/digests intact.
    manifest["rust_build_sources"] = scene_build_sources(root)?;
    write(
        root,
        format!("{output}/manifest.json"),
        &json_bytes(&manifest),
    )
}
pub fn scene_build_sources(root: &Path) -> Result<Value> {
    let mut values = serde_json::Map::new();
    for name in [
        "xtask/Cargo.toml",
        "xtask/Cargo.lock",
        "xtask/src/build.rs",
        "xtask/src/build_templates.rs",
        "xtask/src/lib.rs",
        "xtask/src/main.rs",
        "xtask/src/tooling.rs",
        "xtask/src/process.rs",
        "xtask/src/process/unix.rs",
        "xtask/src/process/windows.rs",
        ".cargo/config.toml",
        "rust-toolchain.toml",
        "Cargo.toml",
        "Cargo.lock",
        "examples/dashboard-counter-island/Cargo.toml",
        "examples/dashboard-counter-island/src/lib.rs",
        "examples/webgpu-scene-island/scene-loader.js",
        "examples/webgpu-scene-island/counter-loader.js",
        "examples/webgpu-scene-island/comparison-loader.js",
        "examples/webgpu-scene-island/src/scene.wgsl",
    ] {
        values.insert(name.into(), json!(sha(&read(root, name)?)));
    }
    Ok(Value::Object(values))
}
