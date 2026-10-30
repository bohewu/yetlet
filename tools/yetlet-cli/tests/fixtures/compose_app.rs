//! Structural lock fixture only; real Cargo buildability is a separate gate.
use std::{fs, path::PathBuf};

pub struct Fixture {
    pub root: tempfile::TempDir,
    pub framework: PathBuf,
    pub app: PathBuf,
    pub parent: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        }
        let framework = root.path().join("framework space 框");
        crate::docker_fixture::create(&framework);
        crate::scaffold::create_service("composeapp", &framework, root.path()).unwrap();
        crate::scaffold::create_service("output_parent", &framework, root.path()).unwrap();
        let app = root.path().join("composeapp");
        fs::write(app.join("Cargo.lock"), lock()).unwrap();
        let parent = root.path().join("output_parent");
        Self {
            root,
            framework,
            app,
            parent,
        }
    }
    pub fn manifest(&self) -> PathBuf {
        self.app.join("Cargo.toml")
    }
}

pub fn lock() -> String {
    let mut value: toml::Value = include_str!("../../../../Cargo.lock").parse().unwrap();
    let source = value["package"].as_array().unwrap();
    let mut sdk = source
        .iter()
        .find(|p| p["name"].as_str() == Some("yetlet"))
        .unwrap()
        .clone();
    let normal = crate::docker_fixture::normal_dependencies(false);
    sdk["dependencies"]
        .as_array_mut()
        .unwrap()
        .retain(|v| normal.contains(v.as_str().unwrap().split_whitespace().next().unwrap()));
    let mut packages: Vec<_> = source
        .iter()
        .filter(|p| p.get("source").is_some())
        .cloned()
        .collect();
    packages.push(sdk);
    packages.push(
        "name='composeapp'\nversion='0.1.0'\ndependencies=['axum','tokio','yetlet']\n"
            .parse()
            .unwrap(),
    );
    value["package"] = toml::Value::Array(packages);
    toml::to_string(&value).unwrap()
}
