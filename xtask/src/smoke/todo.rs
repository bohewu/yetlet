//! Original Todo/Search Chrome and Firefox journeys, retaining their ordering.
use super::{
    Server,
    browser::{Browser, Client},
    http, require,
};
use crate::tooling::{Result, fail};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
fn pause(ms: u64) -> Result<()> {
    std::thread::sleep(Duration::from_millis(ms));
    crate::tooling::check_cancel()
}
fn list(html: &str) -> Result<String> {
    Ok(regex::Regex::new(r#"<ul class="todo-list">.*?</ul>"#)
        .unwrap()
        .find(html)
        .ok_or_else(|| fail("SSR TodoList missing"))?
        .as_str()
        .into())
}
fn ready(c: &mut Client) -> Result<()> {
    c.until(
        "document.documentElement?.getAttribute('data-yetlet-ready') === 'true'",
        "Todo WASM mount",
    )
}
fn text(c: &mut Client, id: &str, value: &str) -> Result<()> {
    c.equal(
        &format!("document.getElementById({}).textContent", json!(id)),
        json!(value),
    )
}
fn no_api(c: &mut Client, expected: usize) -> Result<()> {
    c.equal(
        "performance.getEntriesByType('resource').filter(e=>e.name.endsWith('/api/todos')).length",
        json!(expected),
    )
}
fn count(c: &mut Client) -> Result<usize> {
    c.evaluate("document.readyState")?;
    match c {
        Client::Chrome(c) => Ok(c.requests.len()),
        _ => Err(fail("CDP request count needs Chrome")),
    }
}
fn exact_post(c: &mut Client, start: usize, title: &str) -> Result<()> {
    let c = c.chrome()?;
    let requests = &c.requests[start..];
    require(
        requests.len() == 1 && requests[0].ends_with("/api/todos"),
        "exactly one semantic HTTP command",
    )?;
    let post = c
        .api_posts
        .last()
        .ok_or_else(|| fail("missing typed POST"))?;
    require(post["method"] == "POST", "typed POST method")?;
    if let Some(raw) = post["postData"].as_str().filter(|s| !s.is_empty()) {
        let body: Value = serde_json::from_str(raw).map_err(|_| fail("POST JSON"))?;
        require(
            body == json!({"type":"create","title":title}),
            "semantic command payload",
        )?;
    }
    Ok(())
}
fn remote(c: &mut Client, title: &str, status: &str) -> Result<()> {
    c.evaluate(&format!("(()=>{{document.getElementById('remote-title').value={};document.getElementById('remote-title').dispatchEvent(new Event('input',{{bubbles:true}}));document.getElementById('create-remote').click();return true;}})()",json!(title)))?;
    c.until(
        &format!(
            "document.getElementById('remote-status').textContent === {}",
            json!(status)
        ),
        "typed remote action",
    )
}
fn snapshot(base: &str, revision: u64, title: Option<&str>) -> Result<()> {
    let r = http::get(base, "/api/todos")?;
    require(
        r.status == 200 && r.header("content-type")?.starts_with("application/json"),
        "typed HTTP snapshot",
    )?;
    let v = r.json()?;
    require(
        v["revision"] == revision,
        "remote revision unchanged/correct",
    )?;
    if let Some(title) = title {
        require(
            v["items"]
                .as_array()
                .and_then(|a| a.last())
                .is_some_and(|i| i["title"] == title),
            "saved remote title",
        )?;
    }
    Ok(())
}
pub fn run(root: &Path, out: &Path, profile: &str, firefox: bool) -> Result<()> {
    let mut server = Server::start(
        root,
        out,
        "yetlet-todo-server",
        profile,
        "YETLET_LISTEN",
        &[],
        "/healthz",
    )?;
    let result = (|| {
        let r = http::get(&server.base, "/")?;
        require(r.status == 200, "SSR status")?;
        let html = r.text()?;
        require(
            !html.contains("<script")
                && !html.contains(".wasm")
                && !r
                    .header("content-security-policy")?
                    .contains("unsafe-inline"),
            "native no-JS SSR/CSP",
        )?;
        let ssr = list(html)?;
        let r = http::get(&server.base, "/app")?;
        require(
            r.status == 200
                && r.header("content-security-policy")?
                    .contains("'wasm-unsafe-eval'")
                && !r
                    .header("content-security-policy")?
                    .contains("unsafe-inline")
                && r.text()?.contains("type=\"module\""),
            "opt-in browser shell",
        )?;
        let mut browser = Browser::start(root, out, firefox)?;
        let checks = (|| {
            let c = browser.client()?;
            c.navigate(&format!("{}/app", server.base))?;
            ready(c)?;
            c.check("(()=>{const error=document.getElementById('remote-title-error');return error.getAttribute('hidden')==='' && error.getClientRects().length===0;})()")?;
            c.equal(
                "document.querySelector('#todo-list-host ul.todo-list').outerHTML",
                json!(ssr),
            )?;
            c.equal(
                "document.querySelectorAll('#todo-list-host li').length",
                json!(2),
            )?;
            c.equal("document.getElementById('todo-list-host').getAttribute('data-yetlet-local-runtime')",json!("keyed-graph"))?;
            c.evaluate(r#"(()=>{window.__yetlet_local_list=document.querySelector('#todo-list-host ul.todo-list');window.__yetlet_local_row=document.querySelector('#todo-list-host [data-id="1"]');return true;})()"#)?;
            c.equal(
                "document.querySelector('[data-id=\"1\"]').getAttribute('data-completed')",
                json!("false"),
            )?;
            pause(300)?;
            let initial_count = if firefox { 0 } else { count(c)? };
            for completed in [true, false] {
                c.click("#toggle-first")?;
                c.until(&format!("document.querySelector('#todo-list-host [data-id=\"1\"]').getAttribute('data-completed') === '{}'",completed),"local state update")?;
                c.check(r#"document.querySelector('#todo-list-host ul.todo-list')===window.__yetlet_local_list && document.querySelector('#todo-list-host [data-id="1"]')===window.__yetlet_local_row"#)?;
                if completed {
                    c.equal(
                        "document.querySelector('[data-id=\"1\"]').className",
                        json!("is-complete"),
                    )?;
                    text(c, "todo-status", "First task: completed")?;
                }
            }
            if firefox {
                snapshot(&server.base, 0, None)?;
                no_api(c, 0)?;
            } else {
                pause(300)?;
                require(count(c)? == initial_count, "local clicks made no requests")?;
                let d = c.chrome()?;
                require(
                    d.websockets.is_empty()
                        && d.failed_responses.is_empty()
                        && d.exceptions.is_empty(),
                    "local browser resource/protocol errors",
                )?;
            }
            if !firefox {
                let before = count(c)?;
                remote(
                    c,
                    "Remote E2E task",
                    "Saved on server (revision 1). Edit the title to create another task.",
                )?;
                c.equal(
                    "document.querySelectorAll('#remote-list-host li').length",
                    json!(3),
                )?;
                c.equal(
                    "document.querySelector('#remote-list-host [data-id=\"3\"]').textContent",
                    json!("Remote E2E task"),
                )?;
                exact_post(c, before, "Remote E2E task")?;
                require(
                    c.chrome()?
                        .api_posts
                        .iter()
                        .filter(|p| p["url"].as_str().is_some_and(|s| s.ends_with("/api/todos")))
                        .count()
                        == 1,
                    "one POST recorded",
                )?;
                snapshot(&server.base, 1, Some("Remote E2E task"))?;
                require(
                    c.chrome()?.exceptions.is_empty() && c.chrome()?.failed_responses.is_empty(),
                    "remote browser errors",
                )?;
                search(c, &server.base, false)?;
                c.navigate(&format!("{}/app", server.base))?;
                ready(c)?;
                let before = count(c)?;
                remote(c, "  ", "Check the task title. Your text is still here.")?;
                exact_post(c, before, "  ")?;
                c.check("(()=>{const error=document.getElementById('remote-title-error');return !error.hasAttribute('hidden') && error.textContent.length>0 && error.getClientRects().length>0 && document.getElementById('remote-title').getAttribute('aria-invalid')==='true';})()")?;
                c.evaluate("(()=>{const input=document.getElementById('remote-title');input.value='Corrected title';input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
                c.check("(()=>{const error=document.getElementById('remote-title-error');return error.getAttribute('hidden')==='' && error.textContent==='' && error.getClientRects().length===0 && !document.getElementById('remote-title').hasAttribute('aria-invalid');})()")?;
                require(
                    c.chrome()?
                        .failed_responses
                        .iter()
                        .any(|s| s.contains("422") && s.contains("/api/todos")),
                    "visible typed 422",
                )?;
                text(
                    c,
                    "remote-list-host",
                    "Server data appears after a remote action",
                )?;
                snapshot(&server.base, 1, None)?;
            }
            lifecycle(c, &server.base, firefox)?;
            if firefox {
                c.evaluate("(()=>{const input=document.getElementById('remote-title');input.value='Firefox saved task';input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
                c.click("#create-remote")?;
                c.until("document.getElementById('remote-status').textContent==='Saved on server (revision 1). Edit the title to create another task.'","Firefox typed POST")?;
                snapshot(&server.base, 1, Some("Firefox saved task"))?;
                c.evaluate("(()=>{const input=document.getElementById('remote-title');input.value='  ';input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
                c.click("#create-remote")?;
                c.until("document.getElementById('remote-status').textContent==='Check the task title. Your text is still here.'","Firefox typed 422")?;
                snapshot(&server.base, 1, None)?;
                search(c, &server.base, true)?;
            } else {
                let before = count(c)?;
                remote(
                    c,
                    "Lifecycle remote task",
                    "Saved on server (revision 2). Edit the title to create another task.",
                )?;
                exact_post(c, before, "Lifecycle remote task")?;
                snapshot(&server.base, 2, Some("Lifecycle remote task"))?;
                require(
                    c.chrome()?.websockets.is_empty(),
                    "no persistent event circuit",
                )?;
            }
            native_controls(c, &server.base)?;
            snapshot(&server.base, if firefox { 1 } else { 2 }, None)?;
            Ok(())
        })();
        let cleanup = browser.stop();
        cleanup?;
        checks
    })();
    server.stop(false)?;
    result
}
// One actual shared-View journey: initial properties, edits outside the keyed
// list, browser reset, constrained links and a native, read-only GET submission.
fn native_controls(c: &mut Client, base: &str) -> Result<()> {
    c.navigate(&format!("{base}/app"))?;
    ready(c)?;
    let initial = json!([
        "Draft task",
        "Draft task",
        "\nKeep this draft local.\nPreview without creating a task.",
        "\nKeep this draft local.\nPreview without creating a task.",
        "normal",
        true,
        true,
        true,
        true,
        true
    ]);
    let values = "(()=>{const title=document.getElementById('native-title'),notes=document.getElementById('native-notes'),select=document.getElementById('native-priority'),check=document.getElementById('native-completed');return [title.value,title.defaultValue,notes.value,notes.defaultValue,select.value,select.options[0].defaultSelected,check.checked,check.defaultChecked,document.getElementById('native-mode').readOnly,document.getElementById('native-disabled').disabled];})()";
    c.equal(values, initial.clone())?;
    c.check("document.getElementById('native-notes').getAttribute('aria-describedby')==='native-hint' && document.getElementById('native-form').getAttribute('action')===new URL('/controls',location.href).href && document.getElementById('native-ssr-link').getAttribute('href')===new URL('/controls',location.href).href")?;
    c.evaluate("(()=>{window.__yt110Title=document.getElementById('native-title');__yt110Title.value='Edited draft';document.getElementById('native-notes').value='Two\\nlines';document.getElementById('native-priority').value='important';document.getElementById('native-completed').checked=false;return true;})()")?;
    c.click("#toggle-first")?;
    c.until(
        "document.getElementById('todo-status').textContent==='First task: completed'",
        "unrelated local update",
    )?;
    c.check("document.getElementById('native-title')===window.__yt110Title && __yt110Title.value==='Edited draft' && document.getElementById('native-notes').value==='Two\\nlines'")?;
    c.check("(()=>{const data=new FormData(document.getElementById('native-form'));return data.get('title')==='Edited draft' && data.get('priority')==='important' && !data.has('completed') && !data.has('ignored') && data.get('mode')==='Preview only';})()")?;
    no_api(c, 0)?;
    c.click("#native-reset")?;
    c.equal(values, initial.clone())?;
    c.click("#native-anchor-link")?;
    c.until(
        "location.hash==='#native-controls'",
        "same-origin fragment navigation",
    )?;
    c.evaluate("(()=>{document.getElementById('native-title').value='Edited draft';document.getElementById('native-notes').value='Two\\nlines';document.getElementById('native-priority').value='important';document.getElementById('native-completed').checked=false;return true;})()")?;
    c.click("#native-submit")?;
    c.until("location.pathname==='/controls' && document.readyState==='complete' && document.getElementById('native-title')?.value==='Edited draft'", "native GET preview")?;
    c.check("(()=>{const query=new URLSearchParams(location.search);return query.get('title')==='Edited draft' && query.get('priority')==='important' && !query.has('completed') && !query.has('ignored') && document.querySelectorAll('script').length===0 && document.getElementById('native-notes').value==='Two\\nlines' && document.getElementById('native-completed').checked===false;})()")?;
    c.evaluate(
        "(()=>{document.getElementById('native-title').value='Later edit';return true;})()",
    )?;
    c.click("#native-reset")?;
    c.equal(
        "document.getElementById('native-title').value",
        json!("Edited draft"),
    )?;
    c.navigate(&format!("{base}/app"))?;
    ready(c)?;
    c.click("#native-ssr-link")?;
    c.until(
        "location.pathname==='/controls' && document.readyState==='complete'",
        "root-relative link",
    )?;
    c.equal(values, initial)?;
    c.check("document.querySelectorAll('script').length===0")?;
    Ok(())
}

fn lifecycle(c: &mut Client, base: &str, firefox: bool) -> Result<()> {
    let queued = if firefox { 0 } else { count(c)? };
    // YT-107 claims pending synchronously; dispose still retires the queued HTTP
    // work, while the already-rendered pending status remains unchanged.
    c.check(r#"(()=>{window.__yetlet_lifecycle_row=document.querySelector('#todo-list-host [data-id="1"]');window.__yetlet_queued_local_status=document.getElementById('todo-status').textContent;const runtime=window.__yetletTodoRuntime;if(!runtime||typeof runtime.dispose!=='function'||typeof runtime.mount!=='function')return false;document.getElementById('remote-title').value='Queued action must not run';document.getElementById('remote-title').dispatchEvent(new Event('input',{bubbles:true}));document.getElementById('toggle-first').click();document.getElementById('create-remote').click();window.__yetlet_queued_remote_status=document.getElementById('remote-status').textContent;runtime.dispose();window.__yetlet_lifecycle='disposed';return window.__yetlet_queued_remote_status==='Saving to server...';})()"#)?;
    pause(400)?;
    c.equal(
        "window.__yetlet_lifecycle_row.getAttribute('data-completed')",
        json!("false"),
    )?;
    c.check("document.getElementById('todo-status').textContent===window.__yetlet_queued_local_status && document.getElementById('remote-status').textContent===window.__yetlet_queued_remote_status")?;
    if !firefox {
        require(count(c)? == queued, "queued actions retired before HTTP")?;
    }
    c.equal(
        "document.documentElement.getAttribute('data-yetlet-ready')",
        Value::Null,
    )?;
    c.equal(
        "document.documentElement.getAttribute('data-yetlet-disposed')",
        json!("true"),
    )?;
    c.check(r#"window.__yetlet_lifecycle_row.isConnected && document.querySelector('#todo-list-host [data-id="1"]')===window.__yetlet_lifecycle_row"#)?;
    c.click("#toggle-first")?;
    c.click("#create-remote")?;
    pause(400)?;
    c.equal(
        "window.__yetlet_lifecycle_row.getAttribute('data-completed')",
        json!("false"),
    )?;
    text(c, "todo-status", "First task: active")?;
    text(c, "remote-status", "Saving to server...")?;
    if firefox {
        no_api(c, 0)?;
    } else {
        require(count(c)? == queued, "disposed listeners inert")?;
    }
    c.evaluate("(()=>{window.__yetletTodoRuntime.mount();return true;})()")?;
    ready(c)?;
    c.equal(
        "document.documentElement.getAttribute('data-yetlet-disposed')",
        Value::Null,
    )?;
    c.check(r#"document.querySelector('#todo-list-host [data-id="1"]')!==window.__yetlet_lifecycle_row && !window.__yetlet_lifecycle_row.isConnected"#)?;
    c.equal(
        "document.getElementById('todo-list-host').getAttribute('data-yetlet-local-runtime')",
        json!("keyed-graph"),
    )?;
    c.evaluate(r#"(()=>{window.__yetlet_remount_row=document.querySelector('#todo-list-host [data-id="1"]');return true;})()"#)?;
    let before = if firefox { 0 } else { count(c)? };
    c.click("#toggle-first")?;
    c.until(
        "window.__yetlet_remount_row.getAttribute('data-completed')==='true'",
        "single remounted local listener",
    )?;
    c.check(
        r#"document.querySelector('#todo-list-host [data-id="1"]')===window.__yetlet_remount_row"#,
    )?;
    text(c, "todo-status", "First task: completed")?;
    if firefox {
        no_api(c, 0)?;
    } else {
        require(count(c)? == before, "remount local event stays local")?;
    }
    c.click("#toggle-first")?;
    c.until(
        "window.__yetlet_remount_row.getAttribute('data-completed')==='false'",
        "second remount action",
    )?;
    let pending = if firefox { 0 } else { count(c)? };
    c.check("(()=>{window.__yetlet_original_fetch=window.fetch.bind(window);window.__yetlet_fetch_gate={started:false,ready:false};window.fetch=(...args)=>{window.__yetlet_fetch_gate.started=true;return window.__yetlet_original_fetch(...args).then(response=>new Promise(resolve=>{window.__yetlet_fetch_gate.ready=true;window.__yetlet_fetch_gate.release=()=>resolve(response);}));};window.__yetlet_retired_remote_status=document.getElementById('remote-status');window.__yetlet_retired_remote_host=document.getElementById('remote-list-host');window.__yetlet_retired_remote_host_before=window.__yetlet_retired_remote_host.textContent;document.getElementById('remote-title').value='  ';document.getElementById('remote-title').dispatchEvent(new Event('input',{bubbles:true}));document.getElementById('create-remote').click();return true;})()")?;
    c.until("Boolean(window.__yetlet_fetch_gate?.ready && typeof window.__yetlet_fetch_gate.release==='function')","retired HTTP response gate")?;
    c.equal(
        "window.__yetlet_retired_remote_status.textContent",
        json!("Saving to server..."),
    )?;
    if firefox {
        no_api(c, 1)?;
    } else {
        require(
            count(c)? == pending + 1,
            "one in-flight retired HTTP request",
        )?;
    }
    c.check("(()=>{window.__yetletTodoRuntime.dispose();window.__yetletTodoRuntime.mount();window.__yetlet_fetch_gate.release();return true;})()")?;
    ready(c)?;
    pause(400)?;
    c.equal(
        "window.__yetlet_retired_remote_status.textContent",
        json!("Saving to server..."),
    )?;
    c.check("window.__yetlet_retired_remote_host.textContent===window.__yetlet_retired_remote_host_before")?;
    text(c, "remote-status", "No remote action sent")?;
    c.evaluate("(window.fetch=window.__yetlet_original_fetch,true)")?;
    snapshot(base, if firefox { 0 } else { 1 }, None)
}
fn search(c: &mut Client, base: &str, firefox: bool) -> Result<()> {
    let r = http::get(base, "/search")?;
    require(
        r.status == 200
            && r.header("content-security-policy")?
                .contains("'wasm-unsafe-eval'")
            && !r
                .header("content-security-policy")?
                .contains("'unsafe-inline'"),
        "Search strict CSP",
    )?;
    let html = r.text()?;
    for token in [
        "data-yetlet-island=\"search\"",
        "search-loader.js",
        "data-island-state=",
        "type=\"module\"",
    ] {
        require(html.contains(token), "Search SSR loader/state markup")?;
    }
    let ssr = list(html)?;
    let start = if firefox { 0 } else { count(c)? };
    c.navigate(&format!("{base}/search"))?;
    c.until("location.pathname==='/search' && document.readyState==='complete' && document.querySelector('#search-island')?.getAttribute('data-yetlet-loader-ready')==='true'","Search loader installation")?;
    pause(400)?;
    c.equal(
        "document.getElementById('search-island').getAttribute('data-yetlet-activated')",
        Value::Null,
    )?;
    c.equal(
        "document.querySelector('#search-results ul.todo-list').outerHTML",
        json!(ssr),
    )?;
    c.check("(()=>{const s=JSON.parse(document.getElementById('search-island').getAttribute('data-island-state'));return s.schema_version===1 && s.component==='yetlet-search' && s.items.length===2;})()")?;
    if firefox {
        c.equal("performance.getEntriesByType('resource').filter(e=>e.name.includes('yetlet_search_bg.wasm')).length",json!(0))?;
        c.firefox()?.type_text("#search-query", "ReNdErErS")?;
        c.until("document.getElementById('search-island').getAttribute('data-yetlet-activated')==='true' && document.querySelectorAll('#search-results li').length===1","W3C keyboard activates Search")?;
        c.equal(
            "document.querySelector('#search-results [data-id=\"1\"]').textContent",
            json!("Share one Rust View across renderers"),
        )?;
        c.check("performance.getEntriesByType('resource').filter(e=>e.name.includes('yetlet_search_bg.wasm')).length>=1")?;
        no_api(c, 0)?;
        c.firefox()?
            .type_text("#search-query", "\u{e009}a\u{e003}")?;
        c.until(
            "document.getElementById('search-query').value===''",
            "W3C keyboard clear",
        )?;
        c.until(
            "document.querySelectorAll('#search-results li').length===2",
            "Search reset SSR list",
        )?;
        no_api(c, 0)?;
        return Ok(());
    }
    let d = c.chrome()?;
    require(
        d.requests[start..]
            .iter()
            .any(|s| s.contains("search-loader.js"))
            && !d.requests[start..]
                .iter()
                .any(|s| s.contains("yetlet_search.js") || s.contains("yetlet_search_bg.wasm")),
        "Search bundle deferred until interaction",
    )?;
    c.evaluate("(()=>{window.__yetlet_original_input=document.getElementById('search-query');window.__yetlet_original_heading=document.querySelector('#search-page h1');document.getElementById('search-query').focus();document.getElementById('search-query').dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
    c.until(
        "document.getElementById('search-island').getAttribute('data-yetlet-activated')==='true'",
        "lazy Search activation",
    )?;
    c.equal(
        "document.getElementById('search-results').firstElementChild.outerHTML",
        json!(ssr),
    )?;
    let d = c.chrome()?;
    require(
        d.requests[start..]
            .iter()
            .any(|s| s.contains("yetlet_search.js"))
            && d.requests[start..]
                .iter()
                .any(|s| s.contains("yetlet_search_bg.wasm")),
        "both Search assets loaded",
    )?;
    pause(300)?;
    let loaded = count(c)?;
    for query in ["ReNdErErS", "not-a-title", ""] {
        c.evaluate(&format!("(()=>{{const input=document.getElementById('search-query');input.value={};input.dispatchEvent(new Event('input',{{bubbles:true}}));return true;}})()",json!(query)))?;
        match query {"ReNdErErS"=>{c.until("document.getElementById('search-count').textContent==='1 matching tasks'","case-insensitive local filtering")?;c.equal("document.querySelectorAll('#search-results li').length",json!(1))?;c.equal("document.querySelector('#search-results li').getAttribute('data-id')",json!("1"))?;c.check("document.getElementById('search-query')===window.__yetlet_original_input && document.querySelector('#search-page h1')===window.__yetlet_original_heading")?;},"not-a-title"=>c.until("document.querySelector('#search-results #search-empty')?.textContent==='No matching tasks'","empty Search")?,_=>c.until(&format!("document.querySelector('#search-results ul.todo-list')?.outerHTML==={}",json!(ssr)),"reset Search fixture")?}
    }
    pause(200)?;
    require(count(c)? == loaded, "typing issues no network requests")?;
    let d = c.chrome()?;
    require(
        d.websockets.is_empty() && d.failed_responses.is_empty() && d.exceptions.is_empty(),
        "Search no browser/protocol/resource failures",
    )?;
    c.navigate(&format!("{base}/search"))?;
    c.until("location.pathname==='/search' && document.querySelector('#search-island')?.getAttribute('data-yetlet-loader-ready')==='true'","incompatible state fixture")?;
    c.evaluate("(()=>{const root=document.getElementById('search-island');const state=JSON.parse(root.getAttribute('data-island-state'));state.schema_version=99;root.setAttribute('data-island-state',JSON.stringify(state));document.getElementById('search-query').dispatchEvent(new Event('input',{bubbles:true}));return true;})()")?;
    c.until("document.getElementById('search-island').getAttribute('data-yetlet-error')==='load-failed'","invalid state fail-closed fallback")?;
    c.equal(
        "document.querySelector('#search-results ul.todo-list')?.outerHTML",
        json!(ssr),
    )?;
    c.equal(
        "document.getElementById('search-island').getAttribute('data-yetlet-activated')",
        Value::Null,
    )
}
