use super::*;
use std::collections::BTreeSet;
fn toml(path: &Path) -> Result<toml::Value> {
    fs::read_to_string(path)?
        .parse()
        .map_err(|_| fail("invalid metrics TOML"))
}
pub fn dependencies(m: &toml::Value) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let mut add = |m: &toml::Value| {
        for key in ["dependencies", "build-dependencies"] {
            if let Some(t) = m.get(key).and_then(toml::Value::as_table) {
                set.extend(t.keys().cloned());
            }
        }
    };
    add(m);
    if let Some(target) = m.get("target").and_then(toml::Value::as_table) {
        for t in target.values() {
            add(t)
        }
    }
    set
}
pub fn source_metrics(path: &Path) -> Result<Value> {
    fn visit(p: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for e in fs::read_dir(p)? {
            let p = e?.path();
            if p.is_dir() {
                visit(&p, files)?
            } else if p.extension().is_some_and(|s| s == "rs") {
                files.push(p)
            }
        }
        Ok(())
    }
    let mut files = vec![];
    visit(path, &mut files)?;
    files.sort();
    let (mut prod, mut test, mut funcs) = (0, 0, 0);
    let re = regex::Regex::new(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?async\s+fn\s+").unwrap();
    for f in &files {
        let text = fs::read_to_string(f)?;
        let lines: Vec<_> = text.lines().collect();
        let n = if f
            .strip_prefix(path)
            .unwrap()
            .components()
            .any(|c| c.as_os_str() == "tests")
        {
            0
        } else {
            lines
                .iter()
                .position(|s| s.trim() == "#[cfg(test)]")
                .unwrap_or(lines.len())
        };
        prod += lines[..n].iter().filter(|s| !s.trim().is_empty()).count();
        test += lines[n..].iter().filter(|s| !s.trim().is_empty()).count();
        funcs += re.find_iter(&lines[..n].join("\n")).count();
    }
    Ok(
        json!({"rust_files":files.len(),"production_nonblank_lines":prod,"test_nonblank_lines":test,"production_async_functions":funcs}),
    )
}
pub fn measure(root: &Path) -> Result<Value> {
    let workspace = toml(&root.join("Cargo.toml"))?;
    let mut packages = BTreeMap::new();
    for m in workspace["workspace"]["members"]
        .as_array()
        .ok_or_else(|| fail("workspace members"))?
    {
        let path = m.as_str().ok_or_else(|| fail("member path"))?;
        let manifest = toml(&root.join(path).join("Cargo.toml"))?;
        let name = manifest["package"]["name"]
            .as_str()
            .ok_or_else(|| fail("package name"))?;
        packages.insert(
            name.to_string(),
            (path.to_string(), dependencies(&manifest)),
        );
    }
    let catalog = toml(&root.join("examples/samples.toml"))?;
    let mut samples = vec![];
    for s in catalog["sample"]
        .as_array()
        .ok_or_else(|| fail("catalog samples"))?
    {
        let mut entries = vec![s["package"].as_str().unwrap().to_owned()];
        entries.extend(
            s["companion_packages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_owned()),
        );
        let mut app: BTreeSet<String> = entries.iter().cloned().collect();
        let mut pending = entries.clone();
        while let Some(name) = pending.pop() {
            let p = packages
                .get(&name)
                .ok_or_else(|| fail("unknown application package"))?;
            for d in &p.1 {
                if packages
                    .get(d)
                    .is_some_and(|p| p.0.starts_with("examples/"))
                    && app.insert(d.clone())
                {
                    pending.push(d.clone())
                }
            }
        }
        let mut totals = json!({"rust_files":0,"production_nonblank_lines":0,"test_nonblank_lines":0,"production_async_functions":0});
        let mut metrics = serde_json::Map::new();
        let mut direct = BTreeSet::new();
        for name in &app {
            let (p, deps) = &packages[name];
            let mut m = source_metrics(&root.join(p))?;
            for (k, v) in totals.as_object_mut().unwrap() {
                *v = json!(v.as_u64().unwrap() + m[k].as_u64().unwrap())
            }
            m["path"] = json!(p);
            m["direct_dependencies"] = json!(deps);
            direct.extend(deps.clone());
            metrics.insert(name.clone(), m);
        }
        let mut optional = serde_json::Map::new();
        for n in s["optional_dependencies"].as_array().unwrap() {
            let n = n.as_str().unwrap();
            if let Some((p, _)) = packages.get(n) {
                optional.insert(n.into(), source_metrics(&root.join(p))?);
            }
        }
        let mut item = json!({"id":s["id"].as_str(),"title":s["title"].as_str(),"kind":s["kind"].as_str(),"browser_runtime":s["browser_runtime"].as_bool(),"launch_command":s["command"].as_str(),"route_count":s["routes"].as_array().unwrap().len(),"entry_packages":entries,"application_packages":app,"package_metrics":metrics,"direct_dependency_count":direct.len(),"direct_dependencies":direct,"workspace_dependencies":direct.iter().filter(|n|packages.contains_key(*n)).collect::<Vec<_>>(),"framework_dependencies":direct.iter().filter(|n|packages.get(*n).is_some_and(|p|p.0.starts_with("crates/"))).collect::<Vec<_>>(),"example_dependencies":direct.iter().filter(|n|packages.get(*n).is_some_and(|p|p.0.starts_with("examples/"))).collect::<Vec<_>>(),"external_dependencies":direct.iter().filter(|n|!packages.contains_key(*n)).collect::<Vec<_>>(),"optional_capabilities":s["optional_dependencies"],"optional_capability_source":optional});
        item.as_object_mut()
            .unwrap()
            .extend(totals.as_object().unwrap().clone());
        samples.push(item);
    }
    Ok(
        json!({"schema_version":1,"methodology":{"application_package_scope":"catalog primary/companion packages plus transitive workspace dependencies under examples/","source_lines":"non-blank physical Rust lines; in-file source before the first #[cfg(test)] is production and the remainder is test source","dependencies":"union of direct non-dev normal/build and target-specific dependencies across application-owned packages","routes":"catalog-declared repository-backed routes"},"samples":samples}),
    )
}
pub fn run(root: &Path, o: &Options) -> Result<()> {
    let report = measure(root)?;
    println!(
        "ID | App packages | Rust files | Production lines | Test lines | Async fns | Routes | Direct deps | Browser"
    );
    for s in report["samples"].as_array().unwrap() {
        println!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {}",
            s["id"].as_str().unwrap(),
            s["application_packages"].as_array().unwrap().len(),
            s["rust_files"],
            s["production_nonblank_lines"],
            s["test_nonblank_lines"],
            s["production_async_functions"],
            s["route_count"],
            s["direct_dependency_count"],
            s["browser_runtime"]
        )
    }
    if o.values.contains_key("--out") {
        let p = o.path("--out", "");
        let absolute = std::env::current_dir()?.join(p);
        let rel = absolute
            .strip_prefix(root)
            .map_err(|_| fail("--out must stay inside repository"))?;
        let p = crate::tooling::safe(root, rel, true)?;
        output(&p, &report)?;
    }
    Ok(())
}
