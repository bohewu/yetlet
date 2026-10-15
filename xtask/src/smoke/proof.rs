//! Original real CSS/form/CSP/DOM proof, with synthetic composition labels intact.
use super::{
    Server,
    browser::{Browser, Client},
    http, require,
};
use crate::tooling::{self, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};
fn check(c: &mut Client, r: &mut Value, js: &str, label: &str) -> Result<()> {
    c.check(js)?;
    r["checks"].as_array_mut().unwrap().push(json!(label));
    Ok(())
}
fn navigate(c: &mut Client, base: &str, path: &str) -> Result<()> {
    c.navigate(&format!("{base}{path}"))?;
    c.until(
        &format!(
            "document.readyState==='complete' && location.pathname==={}",
            json!(path)
        ),
        "proof navigation",
    )
}
fn click(c: &mut Client, r: &mut Value, selector: &str) -> Result<()> {
    check(
        c,
        r,
        &format!("document.querySelectorAll({}).length===1", json!(selector)),
        &format!("unique control {selector}"),
    )?;
    c.click(selector)?;
    c.evaluate("document.readyState")?;
    Ok(())
}
fn screenshot(c: &mut Client, out: &Path, name: &str, params: Value) -> Result<()> {
    let value = c.chrome()?.call("Page.captureScreenshot", params)?;
    let data = STANDARD
        .decode(value["data"].as_str().unwrap_or(""))
        .map_err(|_| tooling::fail("screenshot base64"))?;
    fs::write(out.join(name), data)?;
    Ok(())
}
pub fn run(root: &Path, out: &Path) -> Result<()> {
    let mut report = json!({"task_ids":["YT-070","YT-071"],"status":"fail","checks":[],"not_verified":["real OS IME","spoken assistive technology","Firefox","mobile device","generic DOM form mounting"],"cleanup":"pending"});
    report["head"] = json!(super::head(root)?);
    report["source_sha256"] = super::source_hashes(root, &root.join("examples/form-css-proof"))?;
    report["rust_source_sha256"] = super::rust_sources(root)?;
    let mut s = Server::start(
        root,
        out,
        "yetlet-form-css-proof",
        "debug",
        "YETLET_FORM_PROOF_LISTEN",
        &[],
        "/healthz",
    )?;
    report["binary_sha256"] = json!(tooling::sha(&fs::read(
        root.join("target/debug/yetlet-form-css-proof")
    )?));
    let result = (|| {
        let mut b = Browser::start(root, out, false)?;
        report["browser"] = json!(b.version);
        let checks = (|| {
            let c = b.client()?;
            c.chrome()?.call(
                "Emulation.setScriptExecutionDisabled",
                json!({"value":true}),
            )?;
            navigate(c, &s.base, "/proof/native")?;
            let response = http::get(&s.base, "/proof/native")?;
            require(
                response.status == 200
                    && !response
                        .header("content-security-policy")?
                        .contains("unsafe-inline")
                    && !response.text()?.contains("<script"),
                "native response no scripts strict CSP",
            )?;
            report["checks"]
                .as_array_mut()
                .unwrap()
                .push(json!("native response has no scripts and strict CSP"));
            c.chrome()?.call(
                "Emulation.setScriptExecutionDisabled",
                json!({"value":false}),
            )?;
            for (js, label) in [
                (
                    "document.querySelector('#notify').checked && document.querySelector('#priority').value==='normal' && document.querySelector('#notes').value==='Check the latest test report.'",
                    "native initial checkbox/select/textarea properties",
                ),
                (
                    "getComputedStyle(document.querySelector('.form-control')).paddingLeft!=='0px' && [...document.styleSheets].length===2",
                    "actual Bootstrap and application styles loaded",
                ),
                (
                    "['title','notes','priority','notify'].every(id=>document.querySelector('label[for='+id+']').control===document.getElementById(id))",
                    "labels associated with actual controls",
                ),
            ] {
                check(c, &mut report, js, label)?;
            }
            c.chrome()?.call(
                "Emulation.setDeviceMetricsOverride",
                json!({"width":1280,"height":960,"deviceScaleFactor":1,"mobile":false}),
            )?;
            screenshot(
                c,
                out,
                "native-desktop.png",
                json!({"captureBeyondViewport":true}),
            )?;
            c.chrome()?.call(
                "Emulation.setDeviceMetricsOverride",
                json!({"width":390,"height":844,"deviceScaleFactor":1,"mobile":false}),
            )?;
            check(
                c,
                &mut report,
                "document.documentElement.scrollWidth<=390",
                "390px responsive page has no horizontal overflow",
            )?;
            screenshot(c, out, "native-narrow.png", json!({}))?;
            navigate(c, &s.base, "/proof/enhanced")?;
            check(
                c,
                &mut report,
                "document.querySelectorAll('script').length===1",
                "preview is separately selected external JS",
            )?;
            click(c, &mut report, "[data-preview-activate]")?;
            check(
                c,
                &mut report,
                "document.querySelector('[data-form-proof]').dataset.previewState==='mounted'",
                "preview activation",
            )?;
            c.evaluate("(()=>{globalThis.proofOriginal=document.getElementById('title');proofOriginal.focus();proofOriginal.setSelectionRange(2,5);return true;})()")?;
            click(c, &mut report, "[data-preview-activate]")?;
            check(
                c,
                &mut report,
                "document.getElementById('title')===proofOriginal && proofOriginal.selectionStart===2 && proofOriginal.selectionEnd===5",
                "remount retains control identity and selection",
            )?;
            c.evaluate("(()=>{globalThis.beforeCount=Number(document.querySelector('[data-form-proof]').dataset.previewUpdates);proofOriginal.value='<b>literal</b>';proofOriginal.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
            check(
                c,
                &mut report,
                "Number(document.querySelector('[data-form-proof]').dataset.previewUpdates)===beforeCount+1 && document.querySelector('[data-preview-summary]').textContent.includes('<b>literal</b>') && !document.querySelector('[data-preview-summary] b')",
                "one listener after remount and text-only preview sink",
            )?;
            c.evaluate("(()=>{globalThis.beforeComposition=document.querySelector('[data-preview-summary]').textContent;proofOriginal.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));proofOriginal.value='synthetic composition';proofOriginal.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true}));return true;})()")?;
            check(
                c,
                &mut report,
                "document.querySelector('[data-preview-summary]').textContent===beforeComposition",
                "synthetic composition suppresses premature preview",
            )?;
            c.evaluate("proofOriginal.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true}))")?;
            check(
                c,
                &mut report,
                "document.querySelector('[data-preview-summary]').textContent.includes('synthetic composition')",
                "synthetic compositionend uses live value, not OS IME proof",
            )?;
            click(c, &mut report, "[data-preview-later]")?;
            click(c, &mut report, "[data-preview-dispose]")?;
            c.evaluate("(()=>{globalThis.disposedCount=document.querySelector('[data-form-proof]').dataset.previewUpdates;proofOriginal.value='retired';proofOriginal.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
            std::thread::sleep(Duration::from_millis(250));
            check(
                c,
                &mut report,
                "document.querySelector('[data-form-proof]').dataset.previewUpdates===disposedCount",
                "disposed listeners and timer cannot update preview",
            )?;
            click(c, &mut report, "[data-preview-activate]")?;
            click(c, &mut report, "button[type=reset]")?;
            check(
                c,
                &mut report,
                "proofOriginal.value==='Inspect lab backup' && document.querySelector('[data-preview-summary]').textContent.includes('Inspect lab backup')",
                "native reset and preview read restored defaults",
            )?;
            let before = c.chrome()?.api_posts.len();
            click(c, &mut report, "#notify")?;
            check(
                c,
                &mut report,
                "!document.getElementById('notify').checked && document.querySelector('[data-preview-summary]').textContent.includes('notifications: off')",
                "live checkbox is not initial checked attribute",
            )?;
            require(
                c.chrome()?.api_posts.len() == before && c.chrome()?.websockets.is_empty(),
                "local preview no POST/event circuit",
            )?;
            report["checks"]
                .as_array_mut()
                .unwrap()
                .push(json!("local preview has no mutation POST or event circuit"));
            click(c, &mut report, "button[type=submit]")?;
            c.until(
                "location.pathname==='/proof/accepted' && document.readyState==='complete'",
                "native form PRG",
            )?;
            check(
                c,
                &mut report,
                "document.querySelector('h1').textContent==='Form validated'",
                "real native submit and redirect complete",
            )?;
            let d = c.chrome()?;
            require(
                d.exceptions.is_empty()
                    && d.failed_responses.is_empty()
                    && d.requests
                        .iter()
                        .all(|url| url.starts_with(&s.base) || url == "about:blank"),
                "no CSP/JS/network errors or external assets",
            )?;
            report["checks"]
                .as_array_mut()
                .unwrap()
                .push(json!("no CSP/JS/network errors or external assets"));
            Ok(())
        })();
        b.stop()?;
        checks
    })();
    let cleanup = s.stop(false);
    if cleanup.is_ok() {
        report["cleanup"] =
            json!("owned processes reaped, listeners closed, temporary profile removed");
    }
    if result.is_ok() && cleanup.is_ok() {
        report["status"] = json!("pass");
    }
    fs::write(out.join("browser.json"), tooling::json_bytes(&report))?;
    cleanup?;
    result
}
