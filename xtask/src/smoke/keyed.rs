//! Full fixed keyed lifecycle/graph/input fixtures from the original two suites.
use super::{
    browser::{Browser, Client},
    require,
    static_server::StaticServer,
};
use crate::tooling::{Result, fail};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
fn pause(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
fn invoke(c: &mut Client, code: &str) -> Result<()> {
    c.equal(
        &format!("(()=>{{try{{{code};return null;}}catch(error){{return String(error);}}}})()"),
        Value::Null,
    )
}
fn mismatch(c: &mut Client, message: &str) -> Result<()> {
    let v=c.evaluate("(()=>{try{window.yetletBindings.mount_demo();return null;}catch(error){return String(error);}})()")?;
    require(v.as_str().is_some_and(|s| s.contains(message)), message)
}
fn order(c: &mut Client, expected: &str) -> Result<()> {
    c.equal("Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')",json!(expected))
}
fn attr(c: &mut Client, name: &str, value: Value) -> Result<()> {
    c.equal(
        &format!(
            "document.documentElement.getAttribute({})",
            json!(format!("data-yetlet-{name}"))
        ),
        value,
    )
}
fn count(c: &mut Client) -> Result<usize> {
    c.evaluate("document.readyState")?;
    Ok(c.chrome()?.requests.len())
}
pub fn run(root: &Path, out: &Path, profile: &str, firefox: bool) -> Result<()> {
    let assets = root.join("target/m7b-keyed-bindings").join(profile);
    let manifest: Value = serde_json::from_slice(&std::fs::read(assets.join("manifest.json"))?)
        .map_err(|_| fail("keyed manifest JSON"))?;
    require(
        manifest["profile"] == profile,
        "keyed manifest/profile mismatch",
    )?;
    let mut server = StaticServer::start(&assets)?;
    let mut browser = Browser::start(root, out, firefox)?;
    let result = (|| {
        let c = browser.client()?;
        if !firefox {
            c.chrome()?
                .call("Network.setCacheDisabled", json!({"cacheDisabled":true}))?;
        }
        c.navigate(&format!("{}/index.html", server.base))?;
        c.until(
            if firefox {
                "document.documentElement.getAttribute('data-yetlet-m7d-ready')==='true'"
            } else {
                "document.documentElement.getAttribute('data-yetlet-m7b-ready')==='true'"
            },
            "keyed module readiness",
        )?;
        if firefox {
            return firefox_inputs(c);
        }
        attr(c, "m7c-ready", json!("true"))?;
        let policy = c.evaluate(
            "document.querySelector('meta[http-equiv=\"Content-Security-Policy\"]').content",
        )?;
        let policy = policy.as_str().ok_or_else(|| fail("keyed CSP missing"))?;
        require(
            policy.contains("'wasm-unsafe-eval'")
                && !policy.contains("'unsafe-inline'")
                && !policy.contains("'unsafe-eval'"),
            "keyed strict CSP",
        )?;
        c.equal(
            "document.querySelectorAll('[data-yetlet-key]').length",
            json!(3),
        )?;
        c.equal(
            "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')",
            Value::Null,
        )?;
        c.evaluate(r#"(()=>{window.__root=document.querySelector('[data-yetlet-component]');window.__row2=document.querySelector('[data-yetlet-key="2"]');window.__label2=__row2.querySelector('[data-yetlet-slot="label"]');window.__labelText2=__label2.firstChild;__root.setAttribute('data-yetlet-instance','wrong-instance');return true;})()"#)?;
        mismatch(c, "identity mismatch")?;
        c.check(r#"__root.isSameNode(document.querySelector('[data-yetlet-component]')) && __row2.isSameNode(document.querySelector('[data-yetlet-key="2"]')) && __root.getAttribute('data-yetlet-bound')===null"#)?;
        c.evaluate("(()=>{__root.setAttribute('data-yetlet-instance','m7b-demo');__row2.setAttribute('data-yetlet-key','99');return true;})()")?;
        mismatch(c, "unknown row key")?;
        c.check(r#"__row2.isSameNode(document.querySelector('[data-yetlet-key="99"]'))"#)?;
        c.evaluate("(()=>{__row2.setAttribute('data-yetlet-key','2');window.__extra=document.createTextNode('unexpected');__row2.appendChild(__extra);return true;})()")?;
        mismatch(c, "unexpected child node")?;
        c.equal("__root.getAttribute('data-yetlet-bound')", Value::Null)?;
        c.click(r#"[data-yetlet-key="2"] [data-yetlet-action="row"]"#)?;
        pause(50);
        c.equal("__row2.getAttribute('data-completed')", json!("false"))?;
        c.evaluate("(()=>{__extra.remove();__label2.innerHTML='<b>Update typed slots locally: active</b>';return true;})()")?;
        mismatch(c, "label slot mismatch")?;
        c.check(r#"__row2.isSameNode(document.querySelector('[data-yetlet-key="2"]')) && __root.getAttribute('data-yetlet-bound')===null"#)?;
        c.evaluate("(()=>{__label2.textContent='Update typed slots locally: active';window.__labelText2=__label2.firstChild;return true;})()")?;
        invoke(
            c,
            "window.yetletBindings.verify_disposed_handle_isolation()",
        )?;
        c.check(r#"__root.getAttribute('data-yetlet-bound')===null && __row2.isSameNode(document.querySelector('[data-yetlet-key="2"]'))"#)?;
        invoke(c, "window.yetletBindings.mount_demo()")?;
        c.check(r#"__root.isSameNode(document.querySelector('[data-yetlet-component]')) && __root.getAttribute('data-yetlet-bound')==='true' && __row2.isSameNode(document.querySelector('[data-yetlet-key="2"]')) && __label2.isSameNode(document.querySelector('[data-yetlet-key="2"] [data-yetlet-slot="label"]')) && __labelText2.isSameNode(document.querySelector('[data-yetlet-key="2"] [data-yetlet-slot="label"]').firstChild)"#)?;
        let lifecycle = count(c)?;
        c.evaluate(r#"(()=>{window.__input2=document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]');__input2.focus();__input2.value='Edited draft two';__input2.setSelectionRange(7,12);__input2.dispatchEvent(new Event('input',{bubbles:true}));return true;})()"#)?;
        c.until(r#"document.querySelector('[data-yetlet-key="2"]').getAttribute('data-draft')==='Edited draft two'"#,"controlled row input commit")?;
        for expected in [None, Some("3,2,1"), Some("1,2,3")] {
            if let Some(expected) = expected {
                invoke(c, "window.yetletBindings.reverse_demo()")?;
                order(c, expected)?;
            }
            c.check(r#"__input2.isSameNode(document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]')) && document.activeElement===__input2 && __input2.value==='Edited draft two' && __input2.selectionStart===7 && __input2.selectionEnd===12"#)?;
        }
        require(
            count(c)? == lifecycle,
            "input/reorder made no network request",
        )?;
        attr(c, "m7d-event-error", Value::Null)?;
        let requests = count(c)?;
        c.evaluate(r#"(()=>{document.getElementById('outside-focus').focus();document.querySelector('[data-yetlet-key="2"] [data-yetlet-action="row"]').click();return true;})()"#)?;
        c.until(
            "__row2.getAttribute('data-completed')==='true'",
            "typed row event",
        )?;
        c.check(r#"__row2.isSameNode(document.querySelector('[data-yetlet-key="2"]')) && document.querySelector('[data-yetlet-key="2"] [data-yetlet-slot="details"]')!==null && document.activeElement.id==='outside-focus'"#)?;
        require(count(c)? == requests, "local row action no requests")?;
        c.click(r#"[data-yetlet-key="2"] [data-yetlet-action="row"]"#)?;
        c.until(
            "__row2.getAttribute('data-completed')==='false'",
            "conditional slot removal",
        )?;
        c.check(r#"document.querySelector('[data-yetlet-key="2"] [data-yetlet-slot="details"]')===null && __row2.isSameNode(document.querySelector('[data-yetlet-key="2"]')) && document.activeElement.id==='outside-focus'"#)?;
        require(count(c)? == requests, "second local action no requests")?;
        invoke(c, "window.yetletBindings.reverse_demo()")?;
        order(c, "3,2,1")?;
        c.check(r#"__row2.isSameNode(document.querySelector('[data-yetlet-key="2"]'))"#)?;
        invoke(c, "window.yetletBindings.append_demo()")?;
        c.check(r#"document.querySelector('[data-yetlet-key="4"]')!==null"#)?;
        removed_inputs(c, 2, 6, 100)?;
        order(c, "3,2,4")?;
        invoke(c, "window.yetletBindings.reset_demo()")?;
        order(c, "1,2,3")?;
        invoke(c, "window.yetletBindings.dispose_demo()")?;
        invoke(c, "window.yetletBindings.mount_demo()")?;
        c.evaluate(r#"(window.__graphRow2=document.querySelector('[data-yetlet-key="2"]'),true)"#)?;
        for (name, value) in [
            ("m7c-owned", "true"),
            ("m7c-commit-count", "0"),
            ("m7c-notification-count", "0"),
        ] {
            attr(c, name, json!(value))?;
        }
        invoke(c, "window.yetletBindings.batch_demo()")?;
        for (name, value) in [
            ("m7c-batch-messages", "3"),
            ("m7c-batch-passes", "1"),
            ("m7c-batch-commits", "1"),
            ("m7c-commit-count", "1"),
            ("m7c-notification-count", "1"),
        ] {
            attr(c, name, json!(value))?;
        }
        order(c, "3,2,1")?;
        c.check(r#"__graphRow2.isSameNode(document.querySelector('[data-yetlet-key="2"]'))"#)?;
        invoke(c, "window.yetletBindings.reentrant_demo()")?;
        for (name, value) in [
            ("m7c-reentrant-messages", "2"),
            ("m7c-reentrant-passes", "2"),
            ("m7c-reentrant-commits", "2"),
            ("m7c-commit-count", "3"),
            ("m7c-notification-count", "3"),
        ] {
            attr(c, name, json!(value))?;
        }
        for (method, count) in [
            ("verify_subscription_disposal", "4"),
            ("verify_reducer_failure", "5"),
        ] {
            invoke(c, &format!("window.yetletBindings.{method}()"))?;
            attr(c, "m7c-commit-count", json!(count))?;
            attr(c, "m7c-notification-count", json!(count))?;
        }
        invoke(c, "window.yetletBindings.verify_commit_failure_isolation()")?;
        c.check(r#"__graphRow2.isSameNode(document.querySelector('[data-yetlet-key="2"]'))"#)?;
        let disposed=c.evaluate(r#"(()=>{window.__disposedInput=document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]');return __disposedInput.getAttribute('value');})()"#)?;
        let completed = c.evaluate(
            r#"document.querySelector('[data-yetlet-key="2"]').getAttribute('data-completed')"#,
        )?;
        invoke(c, "window.yetletBindings.verify_graph_disposal()")?;
        c.equal(
            "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')",
            Value::Null,
        )?;
        attr(c, "m7c-owned", Value::Null)?;
        c.evaluate("(()=>{__disposedInput.value='after dispose';__disposedInput.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
        pause(100);
        c.equal("__disposedInput.getAttribute('value')", disposed.clone())?;
        c.equal(
            r#"document.querySelector('[data-yetlet-key="2"]').getAttribute('data-draft')"#,
            disposed,
        )?;
        attr(c, "m7d-event-error", Value::Null)?;
        c.click(r#"[data-yetlet-key="2"] [data-yetlet-action="row"]"#)?;
        pause(150);
        c.equal(
            r#"document.querySelector('[data-yetlet-key="2"]').getAttribute('data-completed')"#,
            completed,
        )?;
        require(count(c)? == requests, "all graph/input actions stay local")?;
        let d = c.chrome()?;
        require(
            d.websockets.is_empty() && d.failed_responses.is_empty() && d.exceptions.is_empty(),
            "keyed browser errors/event circuit",
        )?;
        Ok(())
    })();
    let browser_cleanup = browser.stop();
    let server_cleanup = server.close();
    browser_cleanup?;
    server_cleanup?;
    result
}
fn removed_inputs(c: &mut Client, start: usize, end: usize, delay: u64) -> Result<()> {
    c.evaluate(&format!(r#"(()=>{{window.__removedInput=document.querySelector('[data-yetlet-key="1"] [data-yetlet-input="draft"]');window.__removedButton=document.querySelector('[data-yetlet-key="1"] [data-yetlet-action="row"]');__removedInput.focus();__removedInput.setSelectionRange({start},{end});window.__beforeRemove=Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'));return true;}})()"#))?;
    c.check("document.activeElement===__removedInput")?;
    c.equal("__removedInput.selectionStart", json!(start))?;
    c.equal("__removedInput.selectionEnd", json!(end))?;
    invoke(c, "window.yetletBindings.remove_first_demo()")?;
    c.until("!__removedInput.isConnected", "focused keyed removal")?;
    c.check("document.activeElement!==__removedInput && Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'))===__beforeRemove+1")?;
    c.evaluate("(()=>{window.__afterRemove=Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'));__removedInput.value='detached edit';__removedInput.dispatchEvent(new Event('input',{bubbles:true}));__removedButton.click();return true;})()")?;
    pause(delay);
    c.check("Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'))===__afterRemove")?;
    attr(c, "m7d-event-error", Value::Null)
}
fn firefox_inputs(c: &mut Client) -> Result<()> {
    invoke(c, "window.yetletBindings.mount_demo()")?;
    c.until("document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')==='true'","Firefox keyed bind")?;
    let selector = r#"[data-yetlet-key="2"] [data-yetlet-input="draft"]"#;
    c.evaluate(r#"(window.__input2=document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]'),true)"#)?;
    c.click(selector)?;
    c.firefox()?.type_text(selector, "\u{e009}a\u{e003}")?;
    c.firefox()?.type_text(selector, "Firefox draft two")?;
    c.until(r#"document.querySelector('[data-yetlet-key="2"]').getAttribute('data-draft')==='Firefox draft two'"#,"Firefox controlled input")?;
    c.evaluate("(__input2.setSelectionRange(8,13),true)")?;
    c.check("document.activeElement===__input2")?;
    invoke(c, "window.yetletBindings.reverse_demo()")?;
    order(c, "3,2,1")?;
    c.check(r#"__input2.isSameNode(document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]')) && document.activeElement===__input2 && __input2.value==='Firefox draft two' && __input2.selectionStart===8 && __input2.selectionEnd===13"#)?;
    invoke(c, "window.yetletBindings.append_demo()")?;
    removed_inputs(c, 1, 5, 150)?;
    invoke(c, "window.yetletBindings.reset_demo()")?;
    c.evaluate(r#"(()=>{window.__disposedInput=document.querySelector('[data-yetlet-key="2"] [data-yetlet-input="draft"]');window.__disposedValue=__disposedInput.getAttribute('value');return true;})()"#)?;
    invoke(c, "window.yetletBindings.dispose_demo()")?;
    c.equal(
        "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')",
        Value::Null,
    )?;
    c.evaluate("(()=>{__disposedInput.value='after dispose';__disposedInput.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
    pause(150);
    c.check("__disposedInput.getAttribute('value')===__disposedValue")?;
    attr(c, "m7d-event-error", Value::Null)?;
    c.equal(
        "performance.getEntriesByType('resource').filter(e=>e.name.includes('/api/')).length",
        json!(0),
    )
}
