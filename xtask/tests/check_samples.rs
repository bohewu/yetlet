//! Direct counterparts of every scripts/test_check_samples.py case, plus path/type coverage.
use std::{fs, path::Path};
use toml::Value;
use yetlet_xtask::{repository_root, samples};
fn catalog() -> Value {
    fs::read_to_string(repository_root().join("examples/samples.toml"))
        .unwrap()
        .parse()
        .unwrap()
}
fn mutated(field: &str, value: Value) -> Value {
    let mut data = catalog();
    let todo = data["sample"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"].as_str() == Some("todo"))
        .unwrap();
    todo[field] = value;
    data
}
fn validate(data: &Value) -> Vec<String> {
    samples::check_data(&repository_root(), data).unwrap()
}
fn s(v: &str) -> Value {
    Value::String(v.into())
}
#[test]
fn test_current_catalog_passes() {
    assert_eq!(validate(&catalog()), Vec::<String>::new());
}
#[test]
fn test_false_command_route_browser_and_dependency_claims_fail() {
    for (field,value) in [
        ("command",s("cargo")),
        ("routes", "v = [{path='/ap',source='examples/todo-server/src/lib.rs',markers=['.get(\"/\", root)']}]".parse::<Value>().unwrap()["v"].clone()),
        ("browser_runtime",Value::Boolean(false)),
        ("routes", "v = [{path='/todos',source='examples/todo-server/src/lib.rs',markers=['.group(\"/api\"','\"/todos\"']}]".parse::<Value>().unwrap()["v"].clone()),
        ("kind",s("api")),("optional_dependencies",Value::Array(vec![s("yetlet-does-not-exist")]))
    ] {assert!(!validate(&mutated(field,value)).is_empty(),"{field}");}
}
#[test]
fn test_repository_path_escape_fails() {
    for (field, value) in [
        ("evidence", "docs/../../../../../etc/passwd"),
        ("path", "examples/../../../../../tmp"),
    ] {
        let errors = validate(&mutated(field, s(value)));
        assert!(
            errors
                .iter()
                .any(|s| s.contains("traversal") || s.contains("escapes")),
            "{errors:?}"
        );
    }
}
#[test]
fn test_schema_and_malformed_types_fail_without_crashing() {
    let mut data = catalog();
    data["schema_version"] = Value::Boolean(true);
    assert!(!validate(&data).is_empty());
    for (field, value) in [
        ("command", Value::Integer(123)),
        ("kind", Value::Array(vec![])),
        ("package", Value::Array(vec![])),
        ("package", Value::Table(Default::default())),
    ] {
        assert!(!validate(&mutated(field, value)).is_empty());
    }
}
#[test]
fn all_catalog_fields_reject_wrong_types_and_duplicate_claims() {
    let fields = [
        "id",
        "title",
        "package",
        "path",
        "kind",
        "status",
        "command",
        "companion_packages",
        "routes",
        "browser_runtime",
        "browser_packages",
        "browser_build",
        "optional_dependencies",
        "evidence",
    ];
    for field in fields {
        assert!(
            !validate(&mutated(field, Value::Integer(17))).is_empty(),
            "{field}"
        );
    }
    let mut data = catalog();
    let duplicate = data["sample"][0].clone();
    data["sample"].as_array_mut().unwrap().push(duplicate);
    let failures = validate(&data);
    for kind in [
        "duplicate sample id",
        "duplicate sample package",
        "duplicate sample path",
    ] {
        assert!(failures.iter().any(|s| s.contains(kind)), "{failures:?}");
    }
    for field in [
        "companion_packages",
        "browser_packages",
        "optional_dependencies",
    ] {
        let v = if field == "optional_dependencies" {
            "yetlet-reactive"
        } else {
            "yetlet-todo-browser"
        };
        assert!(!validate(&mutated(field, Value::Array(vec![s(v), s(v)]))).is_empty());
    }
    for build in [
        "cargo xtask build unknown",
        "cargo xtask build browser --release",
        "cargo xtask build multi-island-dashboard",
    ] {
        assert!(!validate(&mutated("browser_build", s(build))).is_empty());
    }
    let mut data = catalog();
    data["sample"][0].as_table_mut().unwrap().remove("title");
    assert!(!validate(&data).is_empty());
    let mut data = catalog();
    data["sample"][0]
        .as_table_mut()
        .unwrap()
        .insert("unexpected".into(), Value::Boolean(true));
    assert!(!validate(&data).is_empty());
}
fn put(root: &Path, name: &str, content: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
fn small_fixture(root: &Path) -> Value {
    put(root, "Cargo.toml", "[workspace]\nmembers=['examples/a']\n");
    put(
        root,
        "examples/a/Cargo.toml",
        "[package]\nname='a'\nversion='0.1.0'\n",
    );
    put(root, "examples/a/README.md", "cargo run -p a\n`/ok`\n");
    put(root, "examples/a/src/lib.rs", "route ok marker\n");
    put(root, "docs/proof.md", "Proof\n");
    "schema_version=1\n[[sample]]\nid='a'\ntitle='A'\npackage='a'\npath='examples/a'\nkind='server-rendered'\nstatus='verified'\ncommand='cargo run -p a'\ncompanion_packages=[]\nroutes=[{path='/ok',source='examples/a/src/lib.rs',markers=['ok marker']}]\nbrowser_runtime=false\nbrowser_packages=[]\nbrowser_build=''\noptional_dependencies=[]\nevidence='docs/proof.md'\n".parse().unwrap()
}
#[test]
fn source_root_safety_commands_routes_roles_and_dependency_sources() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let original = small_fixture(root);
    assert!(samples::check_data(root, &original).unwrap().is_empty());
    let mut v = original.clone();
    v["sample"][0]["routes"][0]["source"] = s("docs/proof.md");
    assert!(
        samples::check_data(root, &v)
            .unwrap()
            .iter()
            .any(|s| s.contains("inside the sample"))
    );
    let mut v = original.clone();
    v["sample"][0]["routes"][0]["markers"] = Value::Array(vec![s("absent")]);
    assert!(
        samples::check_data(root, &v)
            .unwrap()
            .iter()
            .any(|s| s.contains("marker not found"))
    );
    let mut v = original.clone();
    let duplicate = v["sample"][0]["routes"][0].clone();
    v["sample"][0]["routes"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(
        samples::check_data(root, &v)
            .unwrap()
            .iter()
            .any(|s| s.contains("duplicate route"))
    );
    put(
        root,
        "examples/a/README.md",
        "cargo run -p another\n`/ok-extra`\n",
    );
    let failures = samples::check_data(root, &original).unwrap();
    assert!(failures.iter().any(|s| s.contains("exact catalog command")));
    assert!(failures.iter().any(|s| s.contains("not documented")));
    small_fixture(root);
    put(
        root,
        "examples/a/Cargo.toml",
        "[package]\nname='a'\nversion='0.1.0'\n[target.'cfg(unix)'.build-dependencies]\nweb-sys='1'\n",
    );
    assert!(
        samples::check_data(root, &original)
            .unwrap()
            .iter()
            .any(|s| s.contains("browser dependencies"))
    );
}
#[cfg(unix)]
#[test]
fn symlink_escape_and_canonical_duplicate_fail() {
    use std::os::unix::fs::symlink;
    let fixture = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let mut data = small_fixture(root);
    fs::write(outside.path().join("proof.md"), "external").unwrap();
    symlink(outside.path(), root.join("escape")).unwrap();
    data["sample"][0]["evidence"] = s("escape/proof.md");
    assert!(
        samples::check_data(root, &data)
            .unwrap()
            .iter()
            .any(|s| s.contains("escapes"))
    );
    let mut data = small_fixture(root);
    symlink(root.join("examples/a"), root.join("alias")).unwrap();
    let mut copy = data["sample"][0].clone();
    copy["path"] = s("alias");
    copy["id"] = s("b");
    data["sample"].as_array_mut().unwrap().push(copy);
    assert!(
        samples::check_data(root, &data)
            .unwrap()
            .iter()
            .any(|s| s.contains("duplicate sample path"))
    );
}
#[test]
fn route_tokens_are_exact_and_url_paths_exclude_queries() {
    assert!(samples::route_is_documented(
        "See http://localhost:3000/app?x=1#section",
        "/app"
    ));
    assert!(!samples::route_is_documented("`/apple`", "/app"));
}
