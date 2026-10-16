//! Original no-JS Cookie fixtures; callers own their already-running base URL.
use super::{
    browser::{Browser, Cdp},
    require, wait,
};
use crate::tooling::{self, Result, fail};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn cookies(c: &mut Cdp, base: &str) -> Result<Vec<Value>> {
    Ok(
        c.call("Network.getCookies", json!({"urls":[base]}))?["cookies"]
            .as_array()
            .ok_or_else(|| fail("Cookie array missing"))?
            .clone(),
    )
}
fn navigate(c: &mut Cdp, url: &str, marker: &str, category: bool) -> Result<()> {
    c.call("Page.navigate", json!({"url":url}))?;
    wait(if category { 5 } else { 3 }, "static SSR response", || {
        if category && c.call("Page.getFrameTree", json!({}))?["frameTree"]["frame"]["url"] != url {
            return Ok(None);
        }
        let document = c.call("DOM.getDocument", json!({}))?;
        let html = match c.call(
            "DOM.getOuterHTML",
            json!({"nodeId":document["root"]["nodeId"]}),
        ) {
            Ok(v) => v["outerHTML"].as_str().unwrap_or("").to_owned(),
            Err(e) if e.message.contains("Could not find node with given id") => return Ok(None),
            Err(e) => return Err(e),
        };
        let needle = if category {
            format!("\"category\":\"{marker}\"")
        } else {
            format!("<p id=\"marker\">{marker}</p>")
        };
        Ok((marker.is_empty() || html.contains(&needle)).then_some(()))
    })
}
pub fn run(root: &Path, out: &Path, base: &str, three: bool) -> Result<()> {
    validate_base(base)?;
    let mut browser = Browser::start(root, out, false)?;
    let mut report = json!({"browser":browser.version});
    let checks: Result<()> = (|| {
        let c = browser.client()?.chrome()?;
        c.call(
            "Emulation.setScriptExecutionDisabled",
            json!({"value":true}),
        )?;
        if three {
            let nav = |c: &mut Cdp, instance: &str, path: &str, category: &str| {
                navigate(
                    c,
                    &format!("{base}/__yt066b3/{instance}{path}"),
                    category,
                    true,
                )
            };
            nav(c, "a", "/create", "created")?;
            let stored = cookies(c, base)?;
            require(
                stored.len() == 1 && stored[0]["name"] == "yt_b3" && stored[0]["httpOnly"] == true,
                "fixed HttpOnly shared-session cookie",
            )?;
            let original = stored[0]["value"].clone();
            nav(c, "b", "/read", "active")?;
            nav(c, "c", "/read", "active")?;
            nav(c, "b", "/rotate", "rotated")?;
            require(
                cookies(c, base)?[0]["value"] != original,
                "replacement cookie rotation",
            )?;
            nav(c, "a", "/control/replay", "")?;
            nav(c, "c", "/read", "retired_or_missing")?;
            let stored = cookies(c, base)?;
            require(
                stored.len() == 1 && stored[0]["value"] == original,
                "original prepared Cookie replay observed",
            )?;
            nav(c, "c", "/create", "created")?;
            nav(c, "a", "/read", "active")?;
            nav(c, "b", "/destroy", "destroyed")?;
            nav(c, "c", "/read", "retired_or_missing")?;
            report["js_disabled"] = json!(true);
            report["three_app_shared_session"] = json!(true);
            report["retired_cookie_replay_rejected"] = json!(true);
            report["explicit_create_recovered"] = json!(true);
            report["cross_app_logout_rejected"] = json!(true);
            report["fixed_cookie_count"] = json!(cookies(c, base)?.len());
        } else {
            let deadline = Instant::now() + Duration::from_secs(45);
            let nav = |c: &mut Cdp, path: &str, marker: &str| {
                require(
                    Instant::now() < deadline,
                    "focused Cookie browser time bound",
                )?;
                navigate(c, &format!("{base}{path}"), marker, false)
            };
            nav(c, "fresh", "FRESH")?;
            let stored = cookies(c, base)?;
            require(
                stored.len() == 1 && stored[0]["name"] == "yt_ssr" && stored[0]["httpOnly"] == true,
                "single HttpOnly SSR cookie",
            )?;
            let replacement = stored[0]["value"].clone();
            nav(c, "auth", "AUTHORIZED")?;
            for i in 0..180 {
                nav(c, &format!("old/{i}"), &format!("OLD-{i}"))?;
            }
            let stored = cookies(c, base)?;
            require(
                stored.len() == 1 && stored[0]["value"] != replacement,
                "late cookie replacement observed",
            )?;
            for (path, marker) in [
                ("auth", "UNAUTHORIZED"),
                ("login", "LOGIN"),
                ("auth", "AUTHORIZED"),
                ("logout", "LOGOUT"),
                ("auth", "UNAUTHORIZED"),
                ("old/180", "OLD-180"),
                ("auth", "UNAUTHORIZED"),
            ] {
                nav(c, path, marker)?;
            }
            require(cookies(c, base)?.len() == 1, "one Cookie after retirement")?;
            report["js_disabled"] = json!(true);
            report["delayed_http_responses"] = json!(180);
            report["cookie_count"] = json!(1);
            report["reordered_login_loss_rejected"] = json!(true);
            report["explicit_login_succeeded"] = json!(true);
            report["logout_and_late_retired_cookie_rejected"] = json!(true);
        }
        Ok(())
    })();
    let cleanup = browser.stop();
    if cleanup.is_ok() {
        report["owned_browser_gone"] = json!(true);
        report["debug_port_closed"] = json!(true);
        report["profile_removed"] = json!(true);
    }
    std::fs::write(out.join("browser.json"), tooling::json_bytes(&report))?;
    cleanup?;
    checks?;
    println!("{report}");
    Ok(())
}
pub fn validate_base(base: &str) -> Result<()> {
    let rest = base
        .strip_prefix("http://127.0.0.1:")
        .ok_or_else(|| fail("fixture needs an owned loopback HTTP base URL"))?;
    let port = rest.split('/').next().unwrap_or("");
    require(
        port.parse::<u16>().is_ok_and(|p| p > 0),
        "fixture base URL port",
    )?;
    require(
        !rest.contains(['@', '?', '#', '\\']) && !base.contains(['\r', '\n']),
        "fixture base URL syntax",
    )
}
