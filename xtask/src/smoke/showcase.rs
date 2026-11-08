//! Direct YT-241 checks, reusing the existing HTTP/browser/process transports.
use super::{
    Server,
    browser::{Bidi, Browser, Client, Launch},
    http, require,
};
use crate::tooling::{self, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use std::{fs, path::Path};

const STRICT: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const OPT_IN: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

fn navigate(c: &mut Client, base: &str, path: &str) -> Result<()> {
    c.navigate(&format!("{base}{path}"))?;
    c.until(
        "document.readyState==='complete' && !!document.getElementById('showcase')",
        "showcase navigation",
    )
}
fn shot(c: &mut Client, out: &Path, name: &str) -> Result<()> {
    let png = match c {
        Client::Chrome(d) => d.call("Page.captureScreenshot", json!({}))?["data"].clone(),
        Client::Firefox(f) => f.command("/screenshot", "GET", None)?,
    };
    fs::write(
        out.join(name),
        STANDARD
            .decode(png.as_str().unwrap_or(""))
            .map_err(|_| tooling::fail("showcase screenshot base64"))?,
    )?;
    Ok(())
}
fn width(c: &mut Client, width: u32) -> Result<()> {
    match c {
        Client::Chrome(d) => {
            d.call(
                "Emulation.setDeviceMetricsOverride",
                json!({"width":width,"height":900,"deviceScaleFactor":1,"mobile":false}),
            )?;
        }
        Client::Firefox(f) => {
            f.command(
                "/window/rect",
                "POST",
                Some(&json!({"width":width,"height":1000})),
            )?;
        }
    }
    Ok(())
}
fn keyboard(c: &mut Client, key: &str) -> Result<()> {
    match c {
        Client::Chrome(d) => {
            let code = if key == "Tab" { 9 } else { 13 };
            for kind in ["keyDown", "keyUp"] {
                d.call("Input.dispatchKeyEvent", json!({"type":kind,"key":key,"code":key,"windowsVirtualKeyCode":code,"nativeVirtualKeyCode":code,"text":if kind=="keyDown" && key=="Enter" {"\r"} else {""},"unmodifiedText":if kind=="keyDown" && key=="Enter" {"\r"} else {""}}))?;
            }
        }
        Client::Firefox(f) => f.keys(if key == "Tab" { "\u{e004}" } else { "\u{e007}" })?,
    }
    Ok(())
}

pub fn run(root: &Path, out: &Path, profile: &str, firefox: bool) -> Result<()> {
    let mut report = json!({"suite":"style-showcase","profile":profile,"status":"fail","viewports":[],"cleanup":"pending"});
    // First prove no-browser-output HTTP behavior with the actual executable.
    let missing = out.join("absent-browser-output");
    let mut native = Server::start(
        root,
        out,
        "yetlet-style-showcase",
        profile,
        "YETLET_SHOWCASE_LISTEN",
        &[("YETLET_SHOWCASE_DIST", missing.to_string_lossy().into())],
        "/showcase",
    )?;
    report["native_server"] = json!({"pid":native.process.id(),"port":native.port});
    let native_result: Result<()> = (|| {
        for path in ["/showcase", "/showcase/interactive"] {
            let response = http::get(&native.base, path)?;
            require(
                response.status == 200
                    && response.header("content-security-policy")? == STRICT
                    && !response.text()?.contains("<script"),
                "no-browser-output native SSR and strict CSP",
            )?;
            fs::write(
                out.join(if path.ends_with("interactive") {
                    "absent-interactive.html"
                } else {
                    "native.html"
                }),
                &response.body,
            )?;
        }
        Ok(())
    })();
    native.stop(false)?;
    native_result?;

    let mut server = Server::start(
        root,
        out,
        "yetlet-style-showcase",
        profile,
        "YETLET_SHOWCASE_LISTEN",
        &[],
        "/showcase",
    )?;
    report["server"] = json!({"pid":server.process.id(),"port":server.port});
    let result = (|| {
        let mut browser = Browser::start_config(
            root,
            out,
            firefox,
            Launch {
                bidi: firefox,
                ..Launch::default()
            },
        )?;
        report["browser"] =
            json!({"version":browser.version,"pid":browser.pid(),"port":browser.port()});
        let checks = (|| {
            let c = browser.client()?;
            if !firefox {
                c.chrome()?.call(
                    "Emulation.setScriptExecutionDisabled",
                    json!({"value":true}),
                )?;
            }
            navigate(c, &server.base, "/showcase")?;
            let native_layout =
                c.evaluate("document.querySelector('#layout .preview').innerHTML")?;
            let css_url =
                c.evaluate("document.querySelector('link[rel=stylesheet]').getAttribute('href')")?;
            let css = http::get(&server.base, css_url.as_str().unwrap())?;
            require(
                css.status == 200
                    && css.header("content-type")? == "text/css; charset=utf-8"
                    && css.header("x-content-type-options")? == "nosniff"
                    && css.header("content-security-policy")? == STRICT,
                "CSS MIME/nosniff/exact strict CSP",
            )?;
            fs::write(out.join("produced.css"), &css.body)?;
            for requested in [390, 768, 1280] {
                width(c, requested)?;
                let actual = c.evaluate("innerWidth")?;
                if !firefox {
                    require(actual == requested, "Chrome actual CSS viewport")?;
                }
                c.equal(
                    "getComputedStyle(document.getElementById('flex-preview')).display",
                    json!("flex"),
                )?;
                c.equal(
                    "getComputedStyle(document.getElementById('grid-preview')).display",
                    json!("grid"),
                )?;
                c.equal("getComputedStyle(document.getElementById('grid-preview')).gridTemplateColumns.split(' ').length", json!(if actual.as_u64().unwrap() < 768 { 1 } else { 3 }))?;
                c.check("document.documentElement.scrollWidth<=innerWidth && !document.querySelector('[style],style,script')")?;
                c.equal(
                    "document.querySelectorAll('link[rel=stylesheet]').length",
                    json!(1),
                )?;
                c.equal("performance.getEntriesByType('resource').filter(e=>e.name.endsWith('/showcase.css')).length", json!(1))?;
                let observation = c.evaluate("(()=>{let g=getComputedStyle(document.getElementById('grid-preview'));return {width:innerWidth,columns:g.gridTemplateColumns,gap:g.gap,rail:getComputedStyle(document.querySelector('.category-nav')).position}})()")?;
                report["viewports"].as_array_mut().unwrap().push(json!({"requested":requested,"observed":observation,"clamped":actual!=requested}));
                if requested != 768 {
                    shot(c, out, &format!("native-{requested}.png"))?;
                }
            }
            // Real native disclosure and real keyboard focus, including disabled tab exclusion.
            c.evaluate(
                "(()=>{document.querySelector('#layout details summary').focus();return true})()",
            )?;
            keyboard(c, "Enter")?;
            report["disclosure_observed"] = c.evaluate("({open:document.querySelector('#layout details').open,active:document.activeElement.tagName,text:document.querySelector('#layout pre').textContent.slice(0,160),scrollWidth:document.documentElement.scrollWidth,innerWidth})")?;
            c.check("document.querySelector('#layout details').open && document.querySelector('#layout pre').textContent.includes('pub fn preview()') && document.documentElement.scrollWidth<=innerWidth")?;
            shot(c, out, "source-desktop.png")?;
            c.evaluate("(()=>{document.getElementById('demo-name').focus();return true})()")?;
            keyboard(c, "Tab")?;
            c.check("document.activeElement.id==='demo-format' && document.activeElement.matches(':focus-visible') && getComputedStyle(document.activeElement).outlineStyle==='solid'")?;
            keyboard(c, "Tab")?;
            keyboard(c, "Tab")?;
            c.check("document.activeElement.textContent==='Reset fields' && document.querySelector('button[disabled]').disabled")?;
            keyboard(c, "Tab")?;
            c.check("document.activeElement.matches('#components .source-panel summary') && !document.activeElement.disabled")?;
            c.click("#select-second")?;
            c.until("document.querySelector('#select-second').getAttribute('aria-current')==='true' && document.readyState==='complete'", "native selected navigation")?;
            c.check("document.querySelector('#select-second').classList.contains('is-selected') && document.querySelector('#select-second .selection-label').textContent==='Selected'")?;
            c.click("[data-choice=theme][data-value=dark]")?;
            c.until("document.getElementById('showcase').classList.contains('theme-dark') && document.readyState==='complete'", "native dark theme navigation")?;
            c.click("[data-choice=density][data-value=compact]")?;
            c.until("document.getElementById('showcase').classList.contains('density-compact') && document.readyState==='complete'", "native density navigation")?;
            c.equal(
                "getComputedStyle(document.getElementById('showcase')).backgroundColor",
                json!("rgb(17, 26, 42)"),
            )?;
            c.equal(
                "getComputedStyle(document.getElementById('density-preview')).paddingTop",
                json!("12px"),
            )?;
            report["native_no_js"] =
                json!("complete gallery; disclosure, keyboard, GET theme/density/selection passed");

            if !firefox {
                c.chrome()?.call(
                    "Emulation.setScriptExecutionDisabled",
                    json!({"value":false}),
                )?;
            }
            navigate(c, &server.base, "/showcase/interactive")?;
            c.until(
                "document.getElementById('showcase').dataset.enhanced==='true'",
                "explicit WASM enhancement",
            )?;
            c.equal(
                "document.querySelector('#layout .preview').innerHTML",
                native_layout,
            )?;
            c.equal(
                "document.querySelector('link[rel=stylesheet]').getAttribute('href')",
                css_url,
            )?;
            require(
                http::get(&server.base, "/showcase/interactive")?
                    .header("content-security-policy")?
                    == OPT_IN,
                "exact existing opt-in WASM CSP",
            )?;
            c.evaluate("(()=>{window.__showcaseRoot=document.getElementById('showcase');window.__choice=document.getElementById('select-second');window.__resources=performance.getEntriesByType('resource').length;return true})()")?;
            c.equal(
                "getComputedStyle(document.getElementById('density-preview')).paddingTop",
                json!("20px"),
            )?;
            c.click("[data-choice=theme][data-value=dark]")?;
            c.click("[data-choice=density][data-value=compact]")?;
            c.click("#select-second")?;
            c.check("document.getElementById('showcase')===window.__showcaseRoot && document.getElementById('select-second')===window.__choice && document.getElementById('showcase').className==='showcase theme-dark density-compact' && document.getElementById('select-second').getAttribute('aria-current')==='true' && document.getElementById('selection-status').textContent==='Current example: Optional enhancement.'")?;
            c.equal(
                "getComputedStyle(document.getElementById('showcase')).backgroundColor",
                json!("rgb(17, 26, 42)"),
            )?;
            c.equal(
                "getComputedStyle(document.getElementById('density-preview')).paddingTop",
                json!("12px"),
            )?;
            c.equal(
                "getComputedStyle(document.getElementById('select-second')).borderTopColor",
                json!("rgb(165, 197, 255)"),
            )?;
            c.check("performance.getEntriesByType('resource').length===window.__resources && !document.querySelector('[style],style') && document.querySelectorAll('link[rel=stylesheet]').length===1")?;
            report["enhanced_computed"] = c.evaluate("({theme:getComputedStyle(document.getElementById('showcase')).backgroundColor,padding:getComputedStyle(document.getElementById('density-preview')).paddingTop,selected:getComputedStyle(document.getElementById('select-second')).borderTopColor})")?;
            c.evaluate("(()=>{document.getElementById('states').scrollIntoView();return true})()")?;
            shot(c, out, "interactive-dark-selected.png")?;
            width(c, 390)?;
            c.evaluate("(()=>{document.querySelector('#states .source-panel').open=true;document.getElementById('states').scrollIntoView();return true})()")?;
            c.check("document.documentElement.scrollWidth<=innerWidth")?;
            shot(c, out, "interactive-narrow-source.png")?;
            c.click("[data-choice=theme][data-value=light]")?;
            c.click("[data-choice=density][data-value=comfortable]")?;
            c.click("#select-first")?;
            c.check("document.getElementById('showcase').className==='showcase theme-light density-comfortable' && document.querySelector('#select-first .selection-label').textContent==='Selected'")?;
            // Explicit listener disposal restores native link behavior.
            let module = c.evaluate("document.querySelector('script[type=module]').src.replace('loader.js','showcase.js')")?;
            match c {
                Client::Chrome(d) => {
                    d.call("Runtime.evaluate", json!({"expression":format!("import({}).then(m=>m.dispose())", module),"awaitPromise":true}))?;
                }
                Client::Firefox(f) => {
                    // Classic WebDriver imports into a sandbox with a separate,
                    // uninitialized module. Use the existing BiDi page realm so
                    // dispose addresses the instance initialized by loader.js.
                    let mut bidi = Bidi::connect(
                        f.capabilities["webSocketUrl"]
                            .as_str()
                            .ok_or_else(|| tooling::fail("Showcase BiDi URL"))?,
                    )?;
                    let tree = bidi.call("browsingContext.getTree", json!({"maxDepth":0}))?;
                    let url = f.evaluate("location.href")?;
                    let contexts = tree["contexts"]
                        .as_array()
                        .ok_or_else(|| tooling::fail("Showcase BiDi contexts"))?;
                    let matches: Vec<_> = contexts.iter().filter(|v| v["url"] == url).collect();
                    require(matches.len() == 1, "one owned Showcase BiDi page")?;
                    let result = bidi.call("script.evaluate", json!({
                        "expression":format!("import({}).then(m=>{{m.dispose();return true}})", module),
                        "target":{"context":matches[0]["context"]}, "awaitPromise":true
                    }))?;
                    require(
                        result["type"] == "success" && result["result"]["value"] == true,
                        "Firefox page-realm disposal",
                    )?;
                    report["firefox_disposal_probe"] = result;
                }
            }
            c.check("!document.getElementById('showcase').hasAttribute('data-enhanced')")?;
            // Dispatch without navigation: a disposed listener must no longer
            // prevent the native link's default action or mutate selection.
            c.check("(()=>{const root=document.getElementById('showcase');const before=root.className;const event=new MouseEvent('click',{bubbles:true,cancelable:true,ctrlKey:false});const link=document.querySelector('[data-choice=theme][data-value=dark]');let prevented;link.addEventListener('click',e=>{prevented=e.defaultPrevented;e.preventDefault()},{once:true});link.dispatchEvent(event);return prevented===false && root.className===before})()")?;
            report["listener_disposal"] = json!(
                "explicit disposal removes enhancement and leaves native link default unclaimed; selection unchanged"
            );
            if let Client::Chrome(d) = c {
                require(
                    d.exceptions.is_empty()
                        && d.failed_responses.is_empty()
                        && d.websockets.is_empty()
                        && d.requests
                            .iter()
                            .all(|u| u.starts_with(&server.base) || u == "about:blank"),
                    "no Chrome CSP/resource/runtime/external/circuit errors",
                )?;
                report["network"] = json!({"requests":d.requests,"exceptions":d.exceptions,"failed_responses":d.failed_responses,"websockets":d.websockets});
            }
            Ok(())
        })();
        browser.stop()?;
        checks
    })();
    let cleanup = server.stop(false);
    if cleanup.is_ok() {
        report["cleanup"] =
            json!("owned trees reaped; listeners/streams closed; browser profile removed");
    }
    if result.is_ok() && cleanup.is_ok() {
        report["status"] = json!("pass");
    }
    fs::write(out.join("showcase.json"), tooling::json_bytes(&report))?;
    cleanup?;
    result
}
