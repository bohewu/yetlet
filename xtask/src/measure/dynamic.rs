//! Real matched Canvas2D/WebGPU lanes. Missing lanes and deadlines never imply speedups.
use super::*;
use crate::smoke::{
    self, Server,
    browser::{Browser, Firefox, Launch},
    png,
};
use crate::tooling;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::time::{Duration, Instant};
pub const SIZES: [usize; 4] = [1000, 10000, 50000, 100000];
pub const WORKLOADS: [&str; 3] = ["sparse", "full", "burst"];
pub fn input_hashes(root: &Path) -> Result<Value> {
    let mut hashes = smoke::rust_sources(root)?;
    let here = root.join("examples/webgpu-scene-island");
    for entry in fs::read_dir(&here)? {
        let path = entry?.path();
        if path.is_file()
            && (path.extension().is_some_and(|e| e == "js" || e == "py")
                || path
                    .file_name()
                    .is_some_and(|n| n == "Cargo.toml" || n == "Cargo.lock"))
        {
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .ok_or_else(|| fail("scene input path"))?
                .replace('\\', "/");
            hashes[name] = json!(tooling::sha(&fs::read(path)?));
        }
    }
    for dir in [
        here.join("src"),
        root.join("target/webgpu-scene-island-dist"),
    ] {
        for (k, v) in smoke::source_hashes(root, &dir)?.as_object().unwrap() {
            hashes[k] = v.clone();
        }
    }
    for name in [
        "Cargo.lock",
        "examples/webgpu-scene-island/target/debug/yetlet-webgpu-scene-island",
    ] {
        hashes[name] = json!(tooling::sha(&fs::read(root.join(name))?));
    }
    Ok(hashes)
}
pub fn measurement_status(result: &Result<()>, omitted: bool, integrity: bool) -> &'static str {
    if !integrity {
        "failed"
    } else {
        match result {
            Err(e) if e.code == 124 => "bounded_partial",
            Err(_) => "failed",
            Ok(()) if omitted => "bounded_partial",
            Ok(()) => "passed_yt077_measurement",
        }
    }
}
pub fn distributions(samples: &[Value]) -> Result<Value> {
    let mut out = json!({});
    for key in [
        "prepareMs",
        "applyMs",
        "uploadCPUms",
        "encodeSubmitCPUms",
        "drawObservedMs",
        "completionWaitMs",
        "completionWallMs",
        "rafOpportunity1Ms",
        "rafOpportunity2Ms",
    ] {
        let mut v = vec![];
        for s in samples {
            if !s[key].is_null() {
                let n = s[key].as_f64().ok_or_else(|| fail("dynamic timing type"))?;
                require(n.is_finite() && n >= 0., "dynamic timing value")?;
                v.push(n);
            }
        }
        if !v.is_empty() {
            v.sort_by(f64::total_cmp);
            let n = v.len();
            out[key] = json!({"p50":v[(n*50).div_ceil(100)-1],"p95":v[(n*95).div_ceil(100)-1],"p99":v[(n*99).div_ceil(100)-1],"n":n});
        }
    }
    Ok(out)
}
fn actual(f: &Firefox, body: &str, args: Value) -> Result<Value> {
    let r=f.command("/execute/async","POST",Some(&json!({"script":format!("const done=arguments[arguments.length-1];(async()=>{{{body}}})().then(value=>done({{ok:true,value}})).catch(e=>done({{ok:false,error:String(e)}}));"),"args":args})))?;
    require(r["ok"] == true, "dynamic browser operation failed")?;
    Ok(r["value"].clone())
}
pub fn pixel_proof(p: &png::Pixels, proof: &Value) -> Result<Value> {
    require(p.width == 600 && p.height == 400, "matched capture shape")?;
    let crop = usize::from((0..p.width).all(|x| p.rgb(x, 0).ok() == Some([255, 255, 255])));
    let points = proof["samples"]
        .as_array()
        .ok_or_else(|| fail("pixel sample list"))?;
    require(!points.is_empty(), "pixel samples required")?;
    for point in points {
        let x = point["x"].as_u64().ok_or_else(|| fail("pixel x"))? as usize;
        let y = point["y"].as_u64().ok_or_else(|| fail("pixel y"))? as usize + crop;
        let c = p.rgb(x, y)?;
        let expected = point["rgb"].as_array().ok_or_else(|| fail("pixel RGB"))?;
        require(expected.len() == 3, "pixel RGB length")?;
        for i in 0..3 {
            let e = expected[i]
                .as_u64()
                .filter(|n| *n <= 255)
                .ok_or_else(|| fail("pixel channel"))?;
            require(
                (i64::from(c[i]) - e as i64).abs() <= 25,
                "fixed pixel tolerance",
            )?;
        }
    }
    let x = proof["interiorBlack"][0]
        .as_u64()
        .ok_or_else(|| fail("black x"))? as usize;
    let cy = (proof["interiorBlack"][1]
        .as_u64()
        .ok_or_else(|| fail("black y"))? as usize)
        .min(399 - crop);
    require(p.rgb(x, cy + crop)? == [0, 0, 0], "black interior")?;
    Ok(
        json!({"verified":points.len(),"mismatches":[],"black":[0,0,0],"capture_canvas_origin":[0,crop],"black_canvas_sample":[x,cy],"black_capture_sample":[x,cy+crop],"capture_top_edge":p.rgb(0,0)?}),
    )
}
fn pixels(f: &Firefox, out: &Path, label: &str, report: &mut Value) -> Result<Value> {
    let proof = actual(f, "return await window.__yt077.capture();", json!([]))?;
    let selector = proof["captureSelector"]
        .as_str()
        .ok_or_else(|| fail("capture selector"))?;
    let id = f.element(selector)?;
    let encoded = f.command(&format!("/element/{id}/screenshot"), "GET", None)?;
    let bytes = STANDARD
        .decode(
            encoded
                .as_str()
                .ok_or_else(|| fail("screenshot encoding"))?,
        )
        .map_err(|_| fail("screenshot base64"))?;
    fs::write(out.join(format!("{label}.png")), &bytes)?;
    let p = png::decode(&bytes)?;
    let mut item = pixel_proof(&p, &proof)?;
    item["proof"] = proof.clone();
    item["label"] = json!(label);
    item["sha256"] = json!(crate::tooling::sha(&bytes));
    report["pixels"].as_array_mut().unwrap().push(item);
    Ok(proof)
}
fn identity(p: &Value) -> Value {
    json!({"sequence":p["sequence"],"checksum":p["checksum"],"samples":p["samples"]})
}
pub fn run(root: &Path) -> Result<()> {
    require(
        std::env::var("YT075_FIREFOX_HEADFUL_X11").ok().as_deref() == Some("1"),
        "Use headful X11 only",
    )?;
    let home = std::env::var_os("HOME").ok_or_else(|| fail("Firefox home"))?;
    let common = PathBuf::from(home.clone()).join("snap/firefox/common");
    require(common.is_dir(), "Existing Firefox fixture unavailable")?;
    let parent = root.join("target/m12/yt086");
    fs::create_dir_all(&parent)?;
    let out = tempfile::Builder::new()
        .prefix("scene-dynamic-")
        .tempdir_in(parent)?
        .keep();
    println!("ARTIFACT_DIRECTORY {}", out.display());
    let started = Instant::now();
    let deadline = started + Duration::from_secs(420);
    let source = root.join("target/webgpu-scene-island-dist");
    let manifest: Value = serde_json::from_slice(&fs::read(source.join("manifest.json"))?)
        .map_err(|_| fail("scene manifest"))?;
    let hashes = input_hashes(root)?;
    let binary = root.join("examples/webgpu-scene-island/target/debug/yetlet-webgpu-scene-island");
    let mut bundle = vec![];
    for (key, module) in [("scene", "scene"), ("counter", "yetlet_dashboard_counter")] {
        let mut names = vec![
            format!("{key}-loader.js"),
            format!("{module}.js"),
            format!("{module}_bg.wasm"),
        ];
        if key == "scene" {
            names.push("comparison-loader.js".into())
        }
        for name in names {
            let path = source
                .join("assets")
                .join(
                    manifest[key]["digest"]
                        .as_str()
                        .ok_or_else(|| fail("bundle digest"))?,
                )
                .join(&name);
            let b = fs::read(path)?;
            bundle.push(json!({"entry":key,"name":name,"rawBytes":b.len(),"gzipBytes":gzip(&b)?,"sha256":crate::tooling::sha(&b)}));
        }
    }
    let mut report = json!({"task_id":"YT-077","port_task":"YT-086","status":"running","lanes":[],"omitted":[],"pixels":[],"bundle":bundle,"manifest":manifest,"input_hashes":hashes,"method":{"sizes":SIZES,"rounds":3,"warmups":5,"samples":30,"workloads":WORKLOADS,"burst_operations":8,"seed":7,"css":[600,400],"dpr":1,"sustained_seconds":10,"percentile":"nearest rank; one final-state demand draw per sample","pixel_tolerance_per_channel":25,"observational_budget_ms":16.67,"gpu_timestamps":"not requested","browser_process_bound_seconds":480,"measurement_cutoff_seconds":420},"profile_changes":["dom.webgpu.enabled=true (temporary only)"],"cleanup":{},"os_rss":"not observed","vram":"not observed"});
    output(&out.join("input-hashes.json"), &hashes)?;
    let temp = tempfile::Builder::new()
        .prefix("yt086-dynamic-")
        .tempdir_in(common.clone())?;
    let assets = temp.path().join("assets-root");
    smoke::scene::copy_tree(&source, &assets)?;
    let mut server = None;
    let mut browser = None;
    let mut browser_started = None;
    let order: Vec<_> = (0..3)
        .flat_map(|r| {
            SIZES.into_iter().flat_map(move |s| {
                if r % 2 == 0 {
                    ["canvas2d", "gpu"]
                } else {
                    ["gpu", "canvas2d"]
                }
                .into_iter()
                .map(move |k| (r, s, k))
            })
        })
        .collect();
    let mut proofs = BTreeMap::new();
    let result: Result<()> = (|| {
        server = Some(Server::start_binary(
            root,
            &out,
            &binary,
            "YETLET_SCENE_LISTEN",
            &[("YETLET_SCENE_ASSET_ROOT", assets.display().to_string())],
            "/healthz",
        )?);
        let base = server.as_ref().unwrap().base.clone();
        let r = smoke::http::get(&base, "/scene?q=done&page=1")?;
        require(
            r.status == 200
                && r.text()?.contains("40 matches; page 2 of 4")
                && r.text()?.contains("<html lang=\"en\">")
                && !r
                    .header("content-security-policy")?
                    .contains("unsafe-inline")
                && !r.text()?.contains("comparison-loader.js"),
            "dynamic default SSR/CSP",
        )?;
        browser_started = Some(Instant::now());
        browser =
            Some(Browser::start_config(
                root,
                &out,
                true,
                Launch {
                    firefox_options: Some(json!({"args":[],"prefs":{"dom.webgpu.enabled":true}})),
                    profile_parent: Some(common),
                    env: vec![
                        (
                            "DISPLAY".into(),
                            Some(std::env::var_os("DISPLAY").unwrap_or(":0".into())),
                        ),
                        (
                            "XAUTHORITY".into(),
                            Some(std::env::var_os("XAUTHORITY").unwrap_or(
                                PathBuf::from(home).join(".Xauthority").into_os_string(),
                            )),
                        ),
                    ],
                    ..Default::default()
                },
            )?);
        let f = browser.as_mut().unwrap().client()?.firefox()?;
        f.deadline = Some(deadline);
        require(
            f.capabilities["browserVersion"]
                .as_str()
                .is_some_and(|s| s.starts_with("158.")),
            "Selected Firefox 158 required",
        )?;
        report["capabilities"] = f.capabilities.clone();
        f.command(
            "/timeouts",
            "POST",
            Some(&json!({"script":17000,"pageLoad":20000})),
        )?;
        f.command(
            "/window/rect",
            "POST",
            Some(&json!({"width":1100,"height":1000})),
        )?;
        for (n, (round, size, kind)) in order.iter().copied().enumerate() {
            if started.elapsed() > Duration::from_secs(400) {
                break;
            }
            tooling::check_cancel()?;
            let label = format!("r{round}-{size}-{kind}");
            f.command(
                "/url",
                "POST",
                Some(&json!({"url":format!("{base}/scene-comparison?lane={label}")})),
            )?;
            smoke::wait(20, "comparison entry", || {
                Ok(
                    (f.evaluate("!!window.__yt077&&document.readyState==='complete'")? == true)
                        .then_some(()),
                )
            })?;
            require(f.evaluate("window.__yt075Scene.snapshot().starts===0&&!performance.getEntriesByType('resource').some(x=>x.name.includes('scene_bg.wasm'))")?==true,"preactivation GPU fetch")?;
            if kind == "gpu" {
                let adapter = actual(
                    f,
                    "const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info?.vendor,architecture:a.info?.architecture,fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null}:null;",
                    json!([]),
                )?;
                require(
                    adapter.is_object() && adapter["fallback"] == false,
                    "Actual non-fallback GPU required",
                )?;
                report["adapter"] = adapter;
            }
            let startup = actual(
                f,
                "return await window.__yt077.setup(arguments[0],arguments[1]);",
                json!([kind, size]),
            )?;
            let lane_index = report["lanes"].as_array().unwrap().len();
            report["lanes"].as_array_mut().unwrap().push(json!({"round":round,"size":size,"renderer":kind,"order":n,"startup":startup,"workloads":[],"status":"running"}));
            pixels(f, &out, &format!("{label}-initial"), &mut report)?;
            for workload in WORKLOADS {
                let mut raw = vec![];
                for start in [0, 10, 20, 30] {
                    let count = 10.min(35 - start);
                    let s = actual(
                        f,
                        "const out=[];for(let i=0;i<arguments[1];i++)out.push(await window.__yt077.sample(arguments[0],arguments[2]+i<5));return out;",
                        json!([workload, count, start]),
                    )?;
                    raw.extend(
                        s.as_array()
                            .ok_or_else(|| fail("dynamic raw samples"))?
                            .clone(),
                    );
                    output(
                        &out.join(format!("{label}-{workload}-samples.json")),
                        &json!(raw),
                    )?;
                }
                let measured: Vec<_> = raw
                    .iter()
                    .filter(|s| s["warmup"] == false)
                    .cloned()
                    .collect();
                require(
                    raw.len() == 35 && measured.len() == 30,
                    "dynamic warmup/raw sample counts",
                )?;
                let proof = pixels(f, &out, &format!("{label}-{workload}"), &mut report)?;
                let key = (round, size, workload);
                let id = identity(&proof);
                if let Some(previous) = proofs.insert(key, id.clone()) {
                    require(previous == id, "Unequal renderer final state")?;
                }
                report["lanes"][lane_index]["workloads"].as_array_mut().unwrap().push(json!({"name":workload,"samples":30,"warmups":5,"distributions":distributions(&measured)?,"state":id}));
            }
            if size == 10000 {
                let sustained = actual(
                    f,
                    "const out=[],begin=performance.now();while(performance.now()-begin<10000)out.push(await window.__yt077.sample('sparse'));return {elapsedMs:performance.now()-begin,samples:out};",
                    json!([]),
                )?;
                output(
                    &out.join(format!("{label}-sustained-samples.json")),
                    &sustained,
                )?;
                let s = sustained["samples"]
                    .as_array()
                    .ok_or_else(|| fail("sustained samples"))?;
                report["lanes"][lane_index]["sustained"] = json!({"elapsedMs":sustained["elapsedMs"],"samples":s.len(),"distributions":distributions(s)?,"policy":"10-second sequential sparse lane, per-sample counters; completion is not FPS"});
                pixels(f, &out, &format!("{label}-sustained"), &mut report)?;
            }
            actual(f, "return await window.__yt077.replace(9);", json!([]))?;
            let proof = pixels(f, &out, &format!("{label}-replacement"), &mut report)?;
            let id = identity(&proof);
            if let Some(previous) = proofs.insert((round, size, "replacement"), id.clone()) {
                require(previous == id, "Unequal replacement state")?;
            }
            f.evaluate("document.getElementById('counter-increment').click()")?;
            require(
                f.evaluate("document.getElementById('counter-value').textContent")? == "3",
                "independent counter",
            )?;
            let disposed = f.evaluate("window.__yt077.dispose()")?;
            require(
                disposed["appCPUBytes"] == 0 && disposed["canvasRafPending"] == false,
                "dynamic app cleanup",
            )?;
            if kind == "gpu" {
                for k in [
                    "ownedBufferBytes",
                    "listeners",
                    "observers",
                    "retainedJSBytes",
                ] {
                    require(disposed["gpu"][k] == 0, "dynamic GPU cleanup")?;
                }
            }
            report["lanes"][lane_index]["cleanup"] = disposed;
            report["lanes"][lane_index]["status"] = json!("completed");
            output(&out.join("result.json"), &report)?;
        }
        Ok(())
    })();
    let mut cleanup = true;
    if let Some(b) = &mut browser {
        if let Ok(f) = b.client().and_then(|c| c.firefox()) {
            f.deadline = None;
        }
        if b.stop().is_err() {
            cleanup = false
        }
    }
    if let Some(s) = &mut server
        && s.stop(false).is_err()
    {
        cleanup = false
    }
    let removed = temp.close().is_ok();
    cleanup &= removed;
    let completed = report["lanes"].as_array().unwrap();
    let omitted:Vec<_>=order.iter().filter(|(r,s,k)|!completed.iter().any(|l|l["round"]==*r&&l["size"]==*s&&l["renderer"]==*k&&l["status"]=="completed")).map(|(r,s,k)|json!({"round":r,"size":s,"renderer":k,"reason":"incomplete/not measured; no losing-renderer claim"})).collect();
    let elapsed = browser_started.map(|s| s.elapsed().as_secs_f64());
    let unchanged = input_hashes(root).is_ok_and(|current| current == hashes);
    let status = measurement_status(
        &result,
        !omitted.is_empty(),
        cleanup
            && unchanged
            && elapsed.is_none_or(|s| s <= 480.)
            && tooling::check_cancel().is_ok(),
    );
    let success = status == "passed_yt077_measurement" && elapsed.is_some();
    report["status"] = json!(status);
    report["omitted"] = json!(omitted);
    report["cleanup"] = json!({"verified":cleanup,"tmp_profile_removed":removed});
    report["inputs_unchanged"] = json!(unchanged);
    report["browser_process_seconds"] = json!(elapsed);
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["error_code"] = json!(result.as_ref().err().map(|e| e.code));
    output(&out.join("result.json"), &report)?;
    if success {
        Ok(())
    } else {
        result?;
        Err(fail(
            "dynamic measurement incomplete or failed; see raw evidence",
        ))
    }
}
