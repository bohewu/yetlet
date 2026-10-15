use super::*;
use crate::smoke::{Server, browser::Browser, http};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
const MARKERS: &str = r#"(()=>{const observer=new MutationObserver(records=>{for(const r of records){if(r.type!=='attributes')continue;if(r.attributeName==='data-yetlet-ready'&&r.target.getAttribute('data-yetlet-ready')==='true')performance.mark('yetlet:app-ready');if(r.attributeName==='data-yetlet-activated'&&r.target.getAttribute('data-yetlet-activated')==='true')performance.mark('yetlet:island-ready');}});observer.observe(document,{subtree:true,childList:true,attributes:true,attributeFilter:['data-yetlet-ready','data-yetlet-activated']});})();"#;
pub fn assets(root: &Path, m: &Value) -> Result<Value> {
    let digest = m["digest"].as_str().ok_or_else(|| fail("asset digest"))?;
    require(
        digest.len() == 16
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid digest",
    )?;
    let prefix = format!("/assets/{digest}/");
    let mut out = json!({});
    for label in [
        "entry",
        "module",
        "wasm",
        "search_loader",
        "search_module",
        "search_wasm",
        "stylesheet",
        "favicon",
    ] {
        let name = m[label]
            .as_str()
            .and_then(|s| s.strip_prefix(&prefix))
            .ok_or_else(|| fail("manifest URL"))?;
        require(
            !name.contains(['/', '\\']) && !name.is_empty() && name != "." && name != "..",
            "asset name",
        )?;
        let mut metric = file_metrics(&crate::tooling::safe(
            root,
            format!("dist/assets/{digest}/{name}"),
            false,
        )?)?;
        metric["name"] = json!(name);
        out[label] = metric;
    }
    Ok(out)
}
pub fn first_byte(port: u16) -> Result<f64> {
    let mut s = TcpStream::connect(("127.0.0.1", port))?;
    s.set_read_timeout(Some(Duration::from_secs(5)))?;
    s.set_write_timeout(Some(Duration::from_secs(5)))?;
    let request = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nAccept: text/html\r\n\r\n"
    );
    let then = Instant::now();
    s.write_all(request.as_bytes())?;
    let mut b = [0];
    s.read_exact(&mut b)?;
    let ms = then.elapsed().as_secs_f64() * 1000.;
    require(b == *b"H", "HTTP first byte")?;
    Ok(ms)
}
pub fn run(root: &Path, o: &Options) -> Result<()> {
    let profile = o.string("--profile", "release");
    let m: Value = serde_json::from_slice(&fs::read(root.join("dist/manifest.json"))?)
        .map_err(|_| fail("manifest JSON"))?;
    require(m["mode"] == profile, "profile/manifest mismatch")?;
    let assets = assets(root, &m)?;
    let temp = tempfile::Builder::new()
        .prefix("yt086-baseline-")
        .tempdir()?;
    let out = temp.path();
    let mut server = Server::start(
        root,
        out,
        "yetlet-todo-server",
        &profile,
        "YETLET_LISTEN",
        &[],
        "/healthz",
    )?;
    let result: Result<Value> = (|| {
        let mut routes = json!({});
        for path in ["/", "/search", "/app"] {
            let r = http::get(&server.base, path)?;
            require(r.status == 200, "baseline route status")?;
            let script = r.body.windows(7).any(|s| s == b"<script");
            if path == "/" {
                require(!script, "static SSR script")?
            }
            routes[path] = json!({"html_bytes":r.body.len(),"status":r.status,"has_browser_script":script,"content_security_policy":r.headers.get("content-security-policy")});
        }
        for _ in 0..5 {
            first_byte(server.port)?;
        }
        let mut times = vec![];
        for _ in 0..o.number("--samples", 30)? {
            times.push(first_byte(server.port)?)
        }
        let mut browser = Browser::start(root, out, false)?;
        let version = browser.version.clone();
        let measured: Result<(Value, Value)> = (|| {
            let c = browser.client()?.chrome()?;
            c.call("Network.setCacheDisabled", json!({"cacheDisabled":true}))?;
            c.call(
                "Page.addScriptToEvaluateOnNewDocument",
                json!({"source":MARKERS}),
            )?;
            let (mut app, mut island) = (vec![], vec![]);
            for _ in 0..o.number("--browser-runs", 5)? {
                c.call(
                    "Page.navigate",
                    json!({"url":format!("{}/app",server.base)}),
                )?;
                crate::smoke::wait(18, "Wasm app ready", || {
                    Ok((c.evaluate(
                        "document.documentElement?.getAttribute('data-yetlet-ready')==='true'",
                    )? == true)
                        .then_some(()))
                })?;
                app.push(
                    c.evaluate(
                        "performance.getEntriesByName('yetlet:app-ready').at(-1)?.startTime",
                    )?
                    .as_f64()
                    .ok_or_else(|| fail("missing app timing mark"))?,
                );
                let before = c.requests.len();
                c.call(
                    "Page.navigate",
                    json!({"url":format!("{}/search",server.base)}),
                )?;
                crate::smoke::wait(18, "Search bootstrap", || {
                    Ok((c.evaluate("document.querySelector('#search-island')?.getAttribute('data-yetlet-loader-ready')==='true'")?==true).then_some(()))
                })?;
                require(
                    !c.requests[before..]
                        .iter()
                        .any(|s| s.contains("yetlet_search_bg.wasm")),
                    "eager Search Wasm",
                )?;
                c.evaluate("(()=>{performance.mark('yetlet:island-trigger');document.querySelector('#search-query').dispatchEvent(new Event('input',{bubbles:true}));return true})()")?;
                crate::smoke::wait(18, "Search activated", || {
                    Ok((c.evaluate("document.querySelector('#search-island')?.getAttribute('data-yetlet-activated')==='true'")?==true).then_some(()))
                })?;
                island.push(c.evaluate("(()=>{const a=performance.getEntriesByName('yetlet:island-trigger').at(-1),b=performance.getEntriesByName('yetlet:island-ready').at(-1);return a&&b?b.startTime-a.startTime:null})()")?.as_f64().ok_or_else(||fail("missing island timing mark"))?);
            }
            require(
                c.websockets.is_empty() && c.exceptions.is_empty() && c.failed_responses.is_empty(),
                "baseline browser errors",
            )?;
            Ok((distribution(&app)?, distribution(&island)?))
        })();
        let cleanup = browser.stop();
        cleanup?;
        let (app, island) = measured?;
        let mut env = environment(root)?;
        env["chrome"] = json!(version);
        let sum = |names: &[&str]| {
            names
                .iter()
                .map(|s| assets[*s]["gzip_bytes"].as_u64().unwrap())
                .sum::<u64>()
        };
        Ok(
            json!({"schema_version":1,"timestamp_utc":crate::tooling::utc_now(),"scope":"local_loopback_informational_not_a_performance_gate","profile":profile,"environment":env,"binary_sha256":crate::tooling::sha(&fs::read(root.join(format!("target/{profile}/yetlet-todo-server{}",std::env::consts::EXE_SUFFIX)))?),"asset_manifest_digest":m["digest"],"assets":assets,"asset_sets":{"full_browser_runtime_gzip_bytes":sum(&["entry","module","wasm"]),"island_initial_loader_gzip_bytes":assets["search_loader"]["gzip_bytes"],"island_activated_runtime_gzip_bytes":sum(&["search_module","search_wasm"])},"routes":routes,"latency":{"ssr_request_to_first_response_byte_ms":distribution(&times)?,"chrome_app_ready_navigation_ms":app,"chrome_search_island_trigger_to_ready_ms":island,"cache_disabled":true,"test_browser":"headless Chrome","injected_instrumentation":true},"warmup_requests":5,"limitations":["Loopback only, one host, one headless Chrome; no WAN/TLS/reverse proxy.","Timing begins after TCP connect and includes HTTP headers.","Gzip estimates are not wire bytes; cache disabled; no cold-process/concurrent load.","DevTools instrumentation only; no framework comparison or production SLA."]}),
        )
    })();
    let cleanup = server.stop(false);
    let closed = temp.close();
    cleanup?;
    closed?;
    let report = result?;
    output(
        &o.path("--out", root.join("target/m5-baseline.json")),
        &report,
    )?;
    println!("PASS: bounded informational M5 measurement");
    Ok(())
}
