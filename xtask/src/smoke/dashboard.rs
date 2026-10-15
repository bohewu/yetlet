//! Separate bundles, independent lifetimes and missing/malformed SSR fallback.
use super::{
    Server,
    browser::{Browser, Client},
    http, require,
};
use crate::tooling::Result;
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};
const DEFAULT_CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
fn pause(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
fn count(c: &mut Client) -> Result<usize> {
    c.evaluate("document.readyState")?;
    Ok(c.chrome()?.requests.len())
}
pub fn run(root: &Path, out: &Path, profile: &str, firefox: bool) -> Result<()> {
    let assets = root.join("target/multi-island-dashboard-dist");
    let env = [(
        "YETLET_DASHBOARD_ASSET_ROOT",
        assets.to_string_lossy().into_owned(),
    )];
    let mut server = Server::start(
        root,
        out,
        "yetlet-multi-island-dashboard",
        profile,
        "YETLET_DASHBOARD_LISTEN",
        &env,
        "/healthz",
    )?;
    let result = (|| {
        let r = http::get(&server.base, "/dashboard")?;
        require(
            r.status == 200
                && r.text()?.contains("Yetlet Multi-Island Dashboard")
                && r.text()?.contains("id=\"counter-island\"")
                && r.text()?.contains("id=\"filter-island\"")
                && r.text()?.contains("counter-loader.js")
                && r.text()?.contains("filter-loader.js")
                && r.header("content-security-policy")?
                    .contains("'wasm-unsafe-eval'")
                && !r
                    .header("content-security-policy")?
                    .contains("'unsafe-inline'"),
            "dashboard SSR islands",
        )?;
        let mut browser = Browser::start(root, out, firefox)?;
        let checks = (|| {
            let c = browser.client()?;
            let start = if firefox { 0 } else { count(c)? };
            c.navigate(&format!("{}/dashboard", server.base))?;
            c.until("document.getElementById('counter-island')?.getAttribute('data-yetlet-loader-ready')==='true' && document.getElementById('filter-island')?.getAttribute('data-yetlet-loader-ready')==='true'","dashboard loaders")?;
            c.check("performance.getEntriesByType('resource').every(e=>!e.name.includes('yetlet_dashboard_counter.js') && !e.name.includes('yetlet_dashboard_filter.js'))")?;
            if !firefox {
                let d = c.chrome()?;
                let r = &d.requests[start..];
                require(
                    r.iter().any(|s| s.contains("counter-loader.js"))
                        && r.iter().any(|s| s.contains("filter-loader.js"))
                        && !r.iter().any(|s| {
                            s.contains("yetlet_dashboard_counter")
                                || s.contains("yetlet_dashboard_filter")
                        }),
                    "separate deferred bundles",
                )?;
            }
            c.check("(()=>{window.__counterRoot=document.getElementById('counter-island');window.__counterValue=document.getElementById('counter-value');window.__filterRoot=document.getElementById('filter-island');window.__filterInput=document.getElementById('filter-query');return true;})()")?;
            let start = if firefox { 0 } else { count(c)? };
            c.evaluate("document.getElementById('counter-island').dispatchEvent(new Event('yetlet-activate'))")?;
            c.until("document.getElementById('counter-island').getAttribute('data-yetlet-activated')==='true'","counter activation")?;
            c.check("performance.getEntriesByType('resource').some(e=>e.name.includes('yetlet_dashboard_counter_bg.wasm')) && performance.getEntriesByType('resource').every(e=>!e.name.includes('yetlet_dashboard_filter_bg.wasm'))")?;
            if !firefox {
                let r = &c.chrome()?.requests[start..];
                require(
                    r.iter().any(|s| s.contains("yetlet_dashboard_counter.js"))
                        && r.iter()
                            .any(|s| s.contains("yetlet_dashboard_counter_bg.wasm"))
                        && !r.iter().any(|s| s.contains("yetlet_dashboard_filter")),
                    "counter activation owns only counter bundle",
                )?;
            }
            c.equal(
                "document.getElementById('filter-island').getAttribute('data-yetlet-activated')",
                Value::Null,
            )?;
            c.click("#counter-increment")?;
            c.until(
                "document.getElementById('counter-value').textContent==='3'",
                "counter update",
            )?;
            c.check("document.getElementById('counter-island')===window.__counterRoot && document.getElementById('counter-value')===window.__counterValue && document.querySelectorAll('#filter-results li').length===4")?;
            let start = if firefox { 0 } else { count(c)? };
            c.evaluate("document.getElementById('filter-island').dispatchEvent(new Event('yetlet-activate'))")?;
            c.until("document.getElementById('filter-island').getAttribute('data-yetlet-activated')==='true'","filter activation")?;
            if !firefox {
                let r = &c.chrome()?.requests[start..];
                require(
                    r.iter().any(|s| s.contains("yetlet_dashboard_filter.js"))
                        && r.iter()
                            .any(|s| s.contains("yetlet_dashboard_filter_bg.wasm")),
                    "filter owns separate JS/WASM",
                )?;
            }
            c.evaluate("(()=>{const input=document.getElementById('filter-query');input.focus();input.value='web';input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
            c.until(
                "document.getElementById('filter-count').textContent==='1 matching items'",
                "filter update",
            )?;
            c.check("document.getElementById('filter-query')===window.__filterInput && document.activeElement===window.__filterInput && document.querySelector('#filter-results li').textContent==='WebAssembly' && document.getElementById('counter-value').textContent==='3'")?;
            c.evaluate("(()=>{window.__lateCounterBefore=document.getElementById('counter-value').textContent;document.getElementById('counter-delayed').click();window.__counterDisposed=false;window.__yetletDashboard.counter.dispose().then(()=>window.__counterDisposed=true);return true;})()")?;
            c.until("window.__counterDisposed===true", "counter disposal")?;
            pause(350);
            c.check("document.getElementById('counter-value').textContent===window.__lateCounterBefore && document.getElementById('counter-island').getAttribute('data-yetlet-disposed')==='true' && document.getElementById('filter-count').textContent==='1 matching items'")?;
            if !firefox {
                c.click("#counter-increment")?;
                pause(150);
                c.equal(
                    "document.getElementById('counter-value').textContent",
                    json!("3"),
                )?;
            }
            c.evaluate("(()=>{window.__counterRemounted=false;window.__yetletDashboard.counter.mount().then(()=>window.__counterRemounted=true);return true;})()")?;
            c.until("window.__counterRemounted===true", "counter remount")?;
            c.check("document.getElementById('counter-island')===window.__counterRoot && document.getElementById('counter-value')===window.__counterValue")?;
            c.click("#counter-increment")?;
            c.until(
                "document.getElementById('counter-value').textContent==='4'",
                "single remounted listener",
            )?;
            let exceptions = if firefox {
                0
            } else {
                c.chrome()?.exceptions.len()
            };
            c.evaluate("(()=>{window.__filterFailed=false;window.__yetletDashboard.filter.dispose().then(()=>{const input=document.getElementById('filter-query');input.value='rust';input.dispatchEvent(new Event('input',{bubbles:true}));document.getElementById('filter-island').setAttribute('data-island-version','999');return window.__yetletDashboard.filter.mount();}).catch(()=>window.__filterFailed=true);return true;})()")?;
            c.until("window.__filterFailed===true", "invalid filter failure")?;
            c.check("document.getElementById('filter-island').getAttribute('data-yetlet-error')==='mount-failed' && document.getElementById('filter-query')===window.__filterInput && document.querySelector('#filter-results li').textContent==='WebAssembly'")?;
            c.click("#counter-increment")?;
            c.until(
                "document.getElementById('counter-value').textContent==='5'",
                "sibling after filter failure",
            )?;
            if !firefox {
                require(
                    c.chrome()?.exceptions.len() >= exceptions,
                    "retained mount error observations",
                )?;
            }
            c.evaluate("(()=>{document.getElementById('filter-island').setAttribute('data-island-version','1');window.__filterRecovered=false;window.__yetletDashboard.filter.mount().then(()=>window.__filterRecovered=true);return true;})()")?;
            c.until("window.__filterRecovered===true", "filter recovery")?;
            c.check("document.getElementById('filter-query')===window.__filterInput && document.getElementById('filter-count').textContent==='1 matching items' && document.querySelector('#filter-results li').textContent==='Rust handlers' && document.getElementById('counter-value').textContent==='5'")?;
            if !firefox {
                c.evaluate("(()=>{window.__dashboardDisposed=false;Promise.all([window.__yetletDashboard.counter.dispose(),window.__yetletDashboard.filter.dispose()]).then(()=>window.__dashboardDisposed=true);return true;})()")?;
                c.until("window.__dashboardDisposed===true", "dashboard disposal")?;
                c.check("document.getElementById('counter-island').getAttribute('data-yetlet-disposed')==='true' && document.getElementById('filter-island').getAttribute('data-yetlet-disposed')==='true'")?;
            }
            for malformed in [false, true] {
                fallback(root, out, profile, c, malformed, firefox)?;
            }
            if !firefox {
                require(
                    c.chrome()?.websockets.is_empty() && c.chrome()?.failed_responses.is_empty(),
                    "dashboard no event circuit/resource failures",
                )?;
            }
            Ok(())
        })();
        browser.stop()?;
        checks
    })();
    server.stop(false)?;
    result
}
fn fallback(
    root: &Path,
    out: &Path,
    profile: &str,
    c: &mut Client,
    malformed: bool,
    firefox: bool,
) -> Result<()> {
    let assets = tempfile::tempdir()?;
    if malformed {
        fs::write(assets.path().join("manifest.json"), "{ malformed")?;
    }
    let env = [(
        "YETLET_DASHBOARD_ASSET_ROOT",
        assets.path().to_string_lossy().into_owned(),
    )];
    let mut s = Server::start(
        root,
        out,
        "yetlet-multi-island-dashboard",
        profile,
        "YETLET_DASHBOARD_LISTEN",
        &env,
        "/healthz",
    )?;
    let result = (|| {
        let r = http::get(&s.base, "/dashboard")?;
        require(
            r.status == 200 && r.header("content-security-policy")? == DEFAULT_CSP,
            "fallback exact strict CSP",
        )?;
        let html = r.text()?;
        require(
            html.contains("id=\"counter-island\"")
                && html.contains("id=\"filter-island\"")
                && !html.contains("<script"),
            "fallback script-free readable SSR",
        )?;
        let start = if firefox { 0 } else { count(c)? };
        c.navigate(&format!("{}/dashboard", s.base))?;
        c.until("document.getElementById('counter-value')?.textContent==='2' && document.querySelectorAll('#filter-results li').length===4","fallback SSR document")?;
        c.check("document.querySelectorAll('script').length===0 && globalThis.__yetletDashboard===undefined && document.getElementById('counter-island')!==null && document.getElementById('filter-island')!==null && performance.getEntriesByType('resource').every(e=>!e.name.includes('dashboard-assets') && !e.name.includes('loader.js'))")?;
        if !firefox {
            require(
                !c.chrome()?.requests[start..].iter().any(|s| {
                    s.contains("dashboard-assets")
                        || s.contains("dashboard_counter")
                        || s.contains("dashboard_filter")
                        || s.contains("loader.js")
                }),
                "fallback no bundle request",
            )?;
        }
        Ok(())
    })();
    s.stop(false)?;
    result
}
