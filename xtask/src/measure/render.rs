use super::*;
use crate::smoke::{browser::Browser, static_server::StaticServer};
pub fn production(root: &Path) -> Result<Value> {
    let m: Value = serde_json::from_slice(&fs::read(root.join("dist/manifest.json"))?)
        .map_err(|_| fail("production manifest"))?;
    let d = m["digest"]
        .as_str()
        .ok_or_else(|| fail("production digest"))?;
    let dir = crate::tooling::safe(root, format!("dist/assets/{d}"), false)?;
    let mut files = BTreeMap::new();
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_file() {
            let b = fs::read(&p)?;
            files.insert(
                p.file_name().unwrap().to_string_lossy().into_owned(),
                json!({"bytes":b.len(),"sha256":crate::tooling::sha(&b)}),
            );
        }
    }
    Ok(json!({"digest":d,"mode":m["mode"],"files":files}))
}
pub fn validate(r: &Value, rows: usize, updates: usize, rounds: usize) -> Result<()> {
    require(
        r["config"]["rows"] == rows
            && r["config"]["updates_per_round"] == updates
            && r["config"]["rounds"] == rounds,
        "benchmark config",
    )?;
    let a = r["strategies"]
        .as_array()
        .ok_or_else(|| fail("strategies"))?;
    require(a.len() == 4, "four strategies required")?;
    let mut checksums = None;
    for name in [
        "subtree_replacement",
        "view_tree_patch",
        "keyed_direct_dom",
        "static_template_slots",
    ] {
        let s = a
            .iter()
            .find(|s| s["strategy"] == name)
            .ok_or_else(|| fail("missing strategy"))?;
        require(
            s["final_state_equivalent_all_rounds"] == true
                && s["outside_focus_preserved_all_rounds"] == true
                && s["setup"]["samples"] == rounds
                && s["update"]["samples"] == rounds * updates
                && s["structural"]["samples"] == rounds * 3
                && s["final_state_checksums"]
                    .as_array()
                    .is_some_and(|a| a.len() == rounds),
            "strategy samples/equivalence/focus",
        )?;
        if let Some(ref c) = checksums {
            require(
                c == &s["final_state_checksums"],
                "unequal strategy final state",
            )?
        } else {
            checksums = Some(s["final_state_checksums"].clone())
        }
        require(
            s["inside_input_identity_preserved_all_rounds"] == (name != "subtree_replacement")
                && s["inside_focus_preserved_all_rounds"] == (name != "subtree_replacement")
                && s["untouched_row_identity_preserved_all_rounds"]
                    == (!["subtree_replacement", "view_tree_patch"].contains(&name)),
            "strategy identity semantics",
        )?;
    }
    Ok(())
}
pub fn run(root: &Path, o: &Options) -> Result<()> {
    if o.values.contains_key("--capture-production-before") {
        return output(
            &o.path("--capture-production-before", ""),
            &production(root)?,
        );
    }
    let before: Value = serde_json::from_slice(&fs::read(o.path(
        "--production-before",
        root.join("target/m7a-production-before.json"),
    ))?)
    .map_err(|_| fail("production-before JSON"))?;
    require(
        before == production(root)?,
        "production assets differ from before capture",
    )?;
    let profile = o.string("--profile", "release");
    let path = root.join(format!("target/m7a-render-bench/{profile}"));
    let manifest: Value = serde_json::from_slice(&fs::read(path.join("manifest.json"))?)
        .map_err(|_| fail("benchmark manifest"))?;
    require(manifest["profile"] == profile, "benchmark profile")?;
    let mut assets = json!({});
    let prefix = format!(
        "assets/{}/",
        manifest["digest"]
            .as_str()
            .ok_or_else(|| fail("benchmark digest"))?
    );
    for name in ["entry", "module", "wasm", "stylesheet"] {
        let p = manifest[name]
            .as_str()
            .ok_or_else(|| fail("benchmark manifest URL"))?;
        require(p.starts_with(&prefix), "benchmark URL prefix")?;
        let mut metric = file_metrics(&crate::tooling::safe(&path, p, false)?)?;
        metric["path"] = json!(p);
        assets[name] = metric;
    }
    let temp = tempfile::Builder::new().prefix("yt086-render-").tempdir()?;
    let mut server = StaticServer::start(&path)?;
    let result: Result<(Value, String)> = (|| {
        let mut b = Browser::start(root, temp.path(), false)?;
        let version = b.version.clone();
        let result: Result<Value> = (|| {
            let client = b.client()?;
            client.navigate(&server.base)?;
            client.until(
                "typeof window.runYetletBenchmark==='function'",
                "benchmark export",
            )?;
            let (rows, updates, rounds, seed) = (
                o.number("--rows", 200)?,
                o.number("--updates", 500)?,
                o.number("--rounds", 10)?,
                o.seed_literal()?,
            );
            client.evaluate(&format!("window.__yetletBenchResult=null;window.__yetletBenchError=null;setTimeout(()=>{{try{{window.__yetletBenchResult=window.runYetletBenchmark({rows},{updates},{rounds},{seed})}}catch(e){{window.__yetletBenchError=String(e)}}}},0);true"))?;
            crate::smoke::wait(120, "render benchmark completion", || {
                Ok((client.evaluate(
                    "window.__yetletBenchResult!==null||window.__yetletBenchError!==null",
                )? == true)
                    .then_some(()))
            })?;
            require(
                client.evaluate("window.__yetletBenchError")?.is_null(),
                "benchmark runtime error",
            )?;
            let r = client.evaluate("window.__yetletBenchResult")?;
            validate(&r, rows, updates, rounds)?;
            let c = client.chrome()?;
            require(
                c.websockets.is_empty() && c.failed_responses.is_empty() && c.exceptions.is_empty(),
                "render browser errors",
            )?;
            Ok(r)
        })();
        let cleanup = b.stop();
        cleanup?;
        Ok((result?, version))
    })();
    let cleanup = server.close();
    let closed = temp.close();
    cleanup?;
    closed?;
    let (report, version) = result?;
    let after = production(root)?;
    require(before == after, "production changed during benchmark")?;
    let mut env = environment(root)?;
    env["chrome"] = json!(version);
    output(
        &o.path("--out", root.join("target/m7a-render-strategies.json")),
        &json!({"schema_version":1,"timestamp_utc":crate::tooling::utc_now(),"environment":env,"benchmark_manifest":manifest,"benchmark_assets":assets,"production_assets":{"before_digest":before["digest"],"after_digest":after["digest"],"unchanged":true},"report":report}),
    )?;
    println!("PASS: isolated seeded M7a render strategies");
    Ok(())
}
