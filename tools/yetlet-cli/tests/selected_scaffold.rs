#![cfg(any(target_os = "linux", windows))]
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn inventory(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut paths = vec![std::path::PathBuf::new()];
    while let Some(relative) = paths.pop() {
        for entry in fs::read_dir(root.join(&relative)).unwrap() {
            let entry = entry.unwrap();
            let path = relative.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                paths.push(path);
            } else {
                files.insert(
                    path.to_str().unwrap().replace('\\', "/"),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    files
}

#[test]
fn all_creation_choices_publish_shared_settings_and_complete_frontend_without_tools() {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    for cloud in [false, true] {
        for settings in [false, true] {
            for vite in [false, true] {
                for tailwind in [false, true] {
                    let name = format!("app-{cloud}-{settings}-{vite}-{tailwind}");
                    let mut command = Command::new(env!("CARGO_BIN_EXE_yetlet"));
                    command
                        .args(["new", "--parent"])
                        .arg(root.path())
                        .arg(&name)
                        .env("PATH", root.path().join("no-tools"))
                        .env("CARGO_HOME", root.path().join("no-cache"))
                        .env("APP_SERVICE__TOKEN", "private-sentinel-do-not-copy");
                    for (enabled, flag) in [
                        (cloud, "--service"),
                        (settings, "--settings"),
                        (vite, "--vite"),
                        (tailwind, "--tailwind"),
                    ] {
                        if enabled {
                            command.arg(flag);
                        }
                    }
                    let output = command.output().unwrap();
                    assert!(
                        output.status.success(),
                        "{name}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let app = root.path().join(&name);
                    let before = inventory(&app);
                    assert!(before.values().all(|s| {
                        !String::from_utf8_lossy(s).contains("private-sentinel-do-not-copy")
                    }));
                    let manifest: toml::Value = String::from_utf8(before["Cargo.toml"].clone())
                        .unwrap()
                        .parse()
                        .unwrap();
                    for krate in ["yetlet-core", "yetlet-server-axum"] {
                        assert_eq!(
                            manifest["dependencies"][krate]["version"].as_str(),
                            Some("=0.1.0")
                        );
                        assert!(manifest["dependencies"][krate].get("path").is_none());
                    }
                    assert_eq!(
                        manifest["dependencies"].get("yetlet-config").is_some(),
                        settings || vite
                    );
                    assert_eq!(before.contains_key("src/config.rs"), settings || vite);
                    assert_eq!(
                        manifest["dependencies"].get("axum-server").is_some(),
                        settings || vite
                    );
                    if settings || vite {
                        assert_eq!(
                            manifest["dependencies"]["axum-server"]["optional"].as_bool(),
                            Some(true)
                        );
                        assert!(manifest["features"]["yetlet-dev-tls"].as_array().is_some());
                        assert!(String::from_utf8_lossy(&before[".gitignore"]).contains("/.dev/"));
                    }

                    let base: toml::Value = String::from_utf8(before["base.toml"].clone())
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert_eq!(base["cli"]["rust"]["watch"].as_bool(), Some(false));
                    for key in ["enabled", "auto_start", "auto_build"] {
                        assert_eq!(base["cli"]["vite"][key].as_bool(), Some(vite));
                    }
                    assert_eq!(base["cli"]["vite"]["root"].as_str(), Some("frontend/"));
                    assert_eq!(
                        base["cli"]["generate"]["k8s"]["replicas"].as_integer(),
                        Some(1)
                    );
                    for profile in ["development", "staging", "production"] {
                        assert!(before.contains_key(&format!("{profile}.toml")));
                    }
                    assert!(!before.contains_key("Cargo.lock"));
                    assert!(!app.join("node_modules").exists());
                    assert!(!app.join("target").exists());
                    assert!(!before.contains_key("package.json"));
                    if vite {
                        for file in [
                            "package.json",
                            "package-lock.json",
                            "vite.config.mjs",
                            "vite.dev.config.mjs",
                            "vite.dev.bridge.mjs",
                            "tsconfig.json",
                            ".env.example",
                            "app.ts",
                            "app.css",
                            "companion.ts",
                            "shared.ts",
                            "shared.css",
                            "lazy.ts",
                            "lazy.css",
                            "mark.svg",
                        ] {
                            assert!(
                                before.contains_key(&format!("frontend/{file}")),
                                "{name}: {file}"
                            );
                        }
                        let package: serde_json::Value =
                            serde_json::from_slice(&before["frontend/package.json"]).unwrap();
                        let lock: serde_json::Value =
                            serde_json::from_slice(&before["frontend/package-lock.json"]).unwrap();
                        assert_eq!(
                            package["devDependencies"],
                            lock["packages"][""]["devDependencies"]
                        );
                        assert_eq!(package["scripts"]["build"], "vite build");
                        assert_eq!(
                            package["scripts"]["dev"],
                            "vite --config vite.dev.config.mjs"
                        );
                        assert_eq!(
                            package["devDependencies"]["@tailwindcss/vite"].as_str(),
                            tailwind.then_some("4.3.3")
                        );
                        let config = String::from_utf8_lossy(&before["frontend/vite.config.mjs"]);
                        assert!(config.contains("envDir: root"));
                        assert!(config.contains("plugins: [frontendDotenv, "));
                        assert_eq!(config.contains("tailwindcss()"), tailwind);
                        if tailwind {
                            assert!(
                                config.contains(
                                    "plugins: [frontendDotenv, rustSource, tailwindcss()]"
                                )
                            );
                            assert!(config.contains("YETLET_PROJECT_ROOT"));
                            assert!(
                                String::from_utf8_lossy(&before["frontend/app.css"])
                                    .contains("@source \"../src/main.rs\";")
                            );
                        }
                        assert!(!before.contains_key("src/tailwind.rs"));
                        let main = String::from_utf8_lossy(&before["src/main.rs"]);
                        assert!(
                            main.contains(
                                "vite::Snapshot::load_selected(&settings.cli.vite.root)?"
                            )
                        );
                        assert!(!main.contains("Snapshot::load_from_env"));
                        assert!(!app.join("frontend/dist").exists());
                    } else {
                        assert!(!app.join("frontend").exists());
                        assert_eq!(before.contains_key("tailwind.css"), tailwind);
                        assert_eq!(before.contains_key("src/tailwind.rs"), tailwind);
                    }
                    assert_eq!(command.output().unwrap().status.code(), Some(1));
                    assert_eq!(inventory(&app), before);
                }
            }
        }
    }
    assert!(!root.path().join("no-tools").exists());
    assert!(!root.path().join("no-cache").exists());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 16);
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires an explicit installed Vite 8.3.4 path via YETLET_TEST_VITE_JS; no install or server"]
fn selected_vite_dotenv_is_checked_before_load_on_each_build_and_serve_resolution() {
    let vite =
        Path::new(&std::env::var_os("YETLET_TEST_VITE_JS").expect("set YETLET_TEST_VITE_JS"))
            .canonicalize()
            .unwrap();
    let modules = vite.parent().unwrap().parent().unwrap().parent().unwrap();
    let root = tempfile::tempdir().unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    for tailwind in [false, true] {
        let name = format!("dotenv-{tailwind}");
        yetlet_cli::scaffold::create_selected(&name, root.path(), false, false, tailwind, true)
            .unwrap();
        let app = root.path().join(name);
        let frontend = app.join("ui assets").join("frontend");
        fs::create_dir(frontend.parent().unwrap()).unwrap();
        fs::rename(app.join("frontend"), &frontend).unwrap();
        std::os::unix::fs::symlink(modules, frontend.join("node_modules")).unwrap();
        let output = Command::new("node")
            .args(["--input-type=module", "-e"])
            .arg(r#"
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.cwd();
const { resolveConfig, version } = await import(pathToFileURL(process.argv[1]));
assert.equal(version, '8.3.4');
const outside = join(root, '..', '.env');
fs.writeFileSync(outside, 'VITE_OUTSIDE=outside-marker\n');
const readFileSync = fs.readFileSync;
let deniedPath;
let deniedReads = 0;
fs.readFileSync = function(path, ...args) {
  if (deniedPath && (String(path) === deniedPath || String(path) === outside)) {
    deniedReads++;
    throw new Error('dotenv read before confinement check');
  }
  return readFileSync.call(this, path, ...args);
};
try {
  for (const command of ['build', 'serve']) {
    const mode = command === 'build' ? 'production' : 'development';
    const resolve = () => resolveConfig({
      root, mode, logLevel: 'silent', configLoader: 'native',
      configFile: join(root, command === 'build' ? 'vite.config.mjs' : 'vite.dev.config.mjs'),
    }, command);
    const names = ['.env', '.env.local', `.env.${mode}`, `.env.${mode}.local`];
    for (const [index, name] of names.entries()) {
      fs.writeFileSync(join(root, name), `VITE_LAYER=${index}\nVITE_LOCAL_${index}=local-${index}\nVITE_PRIORITY=file\n`);
    }
    process.env.VITE_PRIORITY = 'inherited';
    const resolved = await resolve();
    assert.equal(resolved.env.VITE_OUTSIDE, undefined);
    assert.equal(resolved.env.VITE_LAYER, '3');
    assert.equal(resolved.env.VITE_PRIORITY, 'inherited');
    for (let index = 0; index < names.length; index++) {
      assert.equal(resolved.env[`VITE_LOCAL_${index}`], `local-${index}`);
    }
    // Resolve again after replacing a previously accepted file, as on config reload.
    for (const name of names) {
      const path = join(root, name);
      const bytes = fs.readFileSync(path);
      fs.unlinkSync(path);
      fs.symlinkSync(outside, path);
      deniedPath = path;
      await assert.rejects(resolve, /Yetlet frontend dotenv must be a regular, unlinked file/);
      assert.equal(deniedReads, 0);
      deniedPath = undefined;
      fs.unlinkSync(path);
      fs.writeFileSync(path, bytes);
    }
    fs.unlinkSync(join(root, '.env'));
    fs.symlinkSync(join(root, 'missing-env'), join(root, '.env'));
    await assert.rejects(resolve, /Yetlet frontend dotenv must be a regular, unlinked file/);
    fs.unlinkSync(join(root, '.env'));
    fs.mkdirSync(join(root, '.env'));
    await assert.rejects(resolve, /Yetlet frontend dotenv must be a regular, unlinked file/);
    fs.rmdirSync(join(root, '.env'));
    for (const name of names.slice(1)) fs.unlinkSync(join(root, name));
  }
  console.log('PASS: plain/Tailwind shared build/serve config checks regular, external-linked, dangling and nonregular dotenv before reads');
} finally {
  fs.readFileSync = readFileSync;
}
"#)
            .arg(modules.join("vite/dist/node/index.js"))
            .current_dir(&frontend)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "tailwind={tailwind}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        println!(
            "tailwind={tailwind}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(!frontend.join("dist").exists());
    }
    root.close().unwrap();
}

#[test]
#[ignore = "explicit offline cached-dependency compilation of the selected release representation with Cargo-owned local overrides"]
fn selected_vite_runtime_settings_and_custom_root_compile_with_explicit_cargo_overrides() {
    let root = tempfile::tempdir().unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    yetlet_cli::scaffold::create_selected("selected-runtime", root.path(), true, false, true, true)
        .unwrap();
    let app = root.path().join("selected-runtime");
    let framework = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let manifest_path = app.join("Cargo.toml");
    let mut manifest: toml::Value = fs::read_to_string(&manifest_path).unwrap().parse().unwrap();
    for krate in ["yetlet-core", "yetlet-server-axum", "yetlet-config"] {
        manifest["dependencies"][krate]
            .as_table_mut()
            .unwrap()
            .insert(
                "path".into(),
                framework
                    .join("crates")
                    .join(krate)
                    .to_str()
                    .unwrap()
                    .into(),
            );
    }
    fs::write(&manifest_path, toml::to_string(&manifest).unwrap()).unwrap();
    let mut config = fs::read_to_string(app.join("src/config.rs")).unwrap();
    config.push_str(
        r#"
#[cfg(test)]
mod selected_cli_settings_tests {
    use super::*;
    #[test]
    fn strict_cli_namespace_keeps_types_unknown_fields_and_bounds() {
        for (key, value) in [
            ("APP_CLI__RUST__WATCH", "1"),
            ("APP_CLI__RUST__UNKNOWN", "false"),
            ("APP_CLI__VITE__ENABLED", "1"),
            ("APP_CLI__VITE__UNKNOWN", "false"),
            ("APP_CLI__GENERATE__K8S__REPLICAS", "0"),
            ("APP_CLI__GENERATE__K8S__REPLICAS", "101"),
            ("APP_CLI__VITE__ROOT", "../outside"),
        ] {
            let loaded = loader().load_from_env::<Settings>([
                ("APP_SERVICE__TOKEN".into(), "synthetic-token".into()),
                (key.into(), value.into()),
            ]);
            assert!(loaded.is_err() || loaded.unwrap().validate(true).is_err(), "{key}");
        }
    }
    #[test]
    fn rust_watch_is_opt_in_and_absent_sections_remain_compatible() {
        let default: Settings = loader().load_from_env([]).unwrap();
        assert!(!default.cli.rust.watch);
        let enabled: Settings = loader().load_from_env([
            ("APP_CLI__RUST__WATCH".into(), "true".into()),
        ]).unwrap();
        assert!(enabled.cli.rust.watch);
        let mut old = serde_json::to_value(serde_json::json!({
            "vite": {"enabled": false, "root": "frontend/", "auto_start": false, "auto_build": false},
            "generate": {"k8s": {"replicas": 1}}
        })).unwrap();
        assert!(!serde_json::from_value::<Cli>(old.clone()).unwrap().rust.watch);
        old["rust"] = serde_json::json!({});
        assert!(serde_json::from_value::<Cli>(old.clone()).is_err());
        old["rust"] = serde_json::json!({"watch": "true"});
        assert!(serde_json::from_value::<Cli>(old).is_err());
    }
}
"#,
    );
    fs::write(app.join("src/config.rs"), config).unwrap();
    let mut adapter = fs::read_to_string(app.join("src/vite.rs")).unwrap();
    adapter.push_str(r###"
#[cfg(test)]
mod selected_frontend_root_tests {
    use super::*;
    #[test]
    fn configurable_root_is_manifest_relative_and_refuses_traversal_or_links() {
        let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let frontend = tempfile::tempdir_in(app).unwrap();
        let name = frontend.path().file_name().unwrap().to_str().unwrap();
        let dist = frontend.path().join("dist");
        std::fs::create_dir_all(dist.join(".vite")).unwrap();
        std::fs::create_dir(dist.join("assets")).unwrap();
        std::fs::write(dist.join("assets/app.js"), "export {};").unwrap();
        std::fs::write(dist.join("assets/mark.svg"), "<svg/>").unwrap();
        std::fs::write(dist.join(".vite/manifest.json"), r##"{"app.ts":{"file":"assets/app.js","src":"app.ts","isEntry":true,"assets":["assets/mark.svg"]}}"##).unwrap();
        assert!(Snapshot::load_selected(name).is_ok());
        for unsafe_root in ["", "../outside", "/absolute", "a/../b", "C:drive", "a\\b", "."] {
            assert!(Snapshot::load_selected(unsafe_root).is_err(), "{unsafe_root}");
        }
        #[cfg(unix)]
        {
            let link = app.join("linked-frontend");
            std::os::unix::fs::symlink(frontend.path(), &link).unwrap();
            assert!(Snapshot::load_selected("linked-frontend").is_err());
            std::fs::remove_file(link).unwrap();
        }
    }
}
"###);
    fs::write(app.join("src/vite.rs"), adapter).unwrap();
    for args in [
        vec!["generate-lockfile", "--offline"],
        vec![
            "check",
            "--offline",
            "--locked",
            "--all-targets",
            "--features",
            "yetlet-dev-tls",
        ],
        vec![
            "test",
            "--offline",
            "--locked",
            "--",
            "--skip",
            "actual_build_has_static_lazy_assets_strict_routes_and_escaped_ssr",
        ],
    ] {
        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&app)
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("target/selected-scaffold-runtime"),
            )
            .env("CARGO_NET_OFFLINE", "true")
            .env("RUSTUP_AUTO_INSTALL", "0")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "cargo {args:?}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
