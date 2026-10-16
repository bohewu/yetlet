//! Existing three-app CAS, retirement, unknown-outcome and no-persistence scenarios.
use super::docker::Lab;
use crate::{
    smoke::{self, http, require},
    tooling::{Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};
fn request(
    l: &Lab,
    x: &str,
    p: &str,
    m: &str,
    c: Option<&str>,
    h: Option<&str>,
) -> Result<http::Response> {
    l.request(x, p, m, c, h)
}
fn create(l: &Lab, x: &str, short: bool) -> Result<String> {
    let r = request(
        l,
        x,
        if short { "/short/create" } else { "/create" },
        "POST",
        None,
        None,
    )?;
    require(r.status == 201, "explicit create")?;
    r.cookie()
}
fn reads(l: &Lab, c: &str, count: Option<u64>, status: u16, path: &str) -> Result<()> {
    for x in ["a", "b", "c"] {
        let r = request(l, x, path, "GET", Some(c), None)?;
        require(
            r.status == status && !r.headers.contains_key("set-cookie"),
            "cross-app authoritative read/no Cookie",
        )?;
        if let Some(n) = count {
            require(r.json()?["count"] == n, "cross-app state")?;
        }
    }
    Ok(())
}
fn gate(l: &Lab, x: &str, action: &str) -> Result<()> {
    require(
        request(
            l,
            x,
            &format!("/control/{action}"),
            if action == "entered" { "GET" } else { "POST" },
            None,
            None,
        )?
        .status
            == 200,
        "fixture gate",
    )
}
fn entered(l: &Lab, x: &str) -> Result<()> {
    smoke::wait(2, "fixture entered", || {
        Ok((request(l, x, "/control/entered", "GET", None, None)?.body == b"1").then_some(()))
    })
}
fn redis(l: &Lab, args: &[&str]) -> Result<String> {
    let mut a = vec!["exec", "-T", "redis", "redis-cli", "--raw"];
    a.extend(args);
    l.compose(&a, 120).map(|s| s.trim().into())
}
fn records(l: &Lab) -> Result<BTreeSet<String>> {
    Ok(redis(
        l,
        &[
            "--scan",
            "--pattern",
            &format!("yetlet:redis:v1:{}:session:*", l.namespace),
        ],
    )?
    .lines()
    .filter(|s| !s.is_empty())
    .map(Into::into)
    .collect())
}
fn hidden(l: &Lab, before: &BTreeSet<String>) -> Result<String> {
    let after = records(l)?;
    let new: Vec<_> = after.difference(before).collect();
    require(new.len() == 1, "held create one identity")?;
    Ok(format!("yt_b3={}", new[0].rsplit(':').next().unwrap()))
}
fn joined<T>(r: std::thread::Result<Result<T>>) -> Result<T> {
    r.map_err(|_| fail("Redis scenario worker panic"))?
}
pub fn exercise(l: &Lab) -> Result<Value> {
    let mut out = json!({});
    let c = create(l, "a", false)?;
    reads(l, &c, Some(1), 200, "/read")?;
    let mut apps = BTreeSet::new();
    for _ in 0..12 {
        let r = request(l, "", "/healthz", "GET", None, None)?;
        require(
            r.status == 200 && !r.headers.contains_key("set-cookie"),
            "stateless Redis scope",
        )?;
        apps.insert(r.text()?.to_string());
    }
    require(
        apps == BTreeSet::from(["a".into(), "b".into(), "c".into()]),
        "stateless three-app distribution",
    )?;
    out["three_app_create_read_and_stateless_distribution"] = json!("pass");
    std::thread::scope(|scope| -> Result<()> {
        for x in ["a", "b"] {
            gate(l, x, "reset")?;
        }
        let a = scope.spawn(|| request(l, "a", "/update", "POST", Some(&c), Some("cas")));
        let b = scope.spawn(|| request(l, "b", "/update", "POST", Some(&c), Some("cas")));
        let entered_result = entered(l, "a").and_then(|()| entered(l, "b"));
        let releases = [gate(l, "a", "release"), gate(l, "b", "release")];
        let results = [joined(a.join())?, joined(b.join())?];
        entered_result?;
        for r in releases {
            r?;
        }
        let mut statuses = results.iter().map(|r| r.status).collect::<Vec<_>>();
        statuses.sort();
        require(
            statuses == [200, 409]
                && results
                    .iter()
                    .all(|r| r.json().is_ok_and(|v| v["callbacks"] == 1)),
            "one CAS winner, callbacks once",
        )?;
        reads(l, &c, Some(2), 200, "/read")?;
        out["cross_process_cas_once_only"] = json!("pass");
        for after in [false, true] {
            let before = records(l)?;
            gate(l, "a", "reset")?;
            let pending = scope.spawn(move || {
                request(
                    l,
                    "a",
                    "/create",
                    "POST",
                    None,
                    Some(if after { "after" } else { "before" }),
                )
            });
            let result: Result<(String, http::Response)> = (|| {
                entered(l, "a")?;
                let old = hidden(l, &before)?;
                let r = request(l, "b", "/rotate", "POST", Some(&old), None)?;
                require(r.status == 200, "rotate held identity")?;
                Ok((old, r))
            })();
            let release = gate(l, "a", "release");
            let old = joined(pending.join())?;
            release?;
            let (hidden, r) = result?;
            let fresh = r.cookie()?;
            require(old.status == 201, "held response delivery")?;
            if after {
                let late = old.cookie()?;
                require(
                    late.starts_with("yt_b3=") && late != fresh,
                    "one fixed-name late Cookie",
                )?;
                require(
                    request(l, "c", "/read", "GET", Some(&late), None)?.status == 401,
                    "late Cookie fails closed",
                )?;
                let recovered = create(l, "c", false)?;
                require(
                    request(l, "b", "/read", "GET", Some(&recovered), None)?.status == 200,
                    "explicit Cookie recovery",
                )?;
                out["prepared_response_reordering_permits_login_loss_only"] = json!("pass");
            } else {
                require(
                    !old.headers.contains_key("set-cookie"),
                    "pre-header stale suppression",
                )?;
                out["staleness_observed_by_finalizer"] = json!("pass");
            }
            reads(l, &hidden, None, 401, "/read")?;
            reads(l, &fresh, Some(1), 200, "/read")?;
        }
        let old = create(l, "a", false)?;
        gate(l, "a", "reset")?;
        let pending = {
            let owned_cookie = old.clone();
            scope.spawn(move || {
                request(
                    l,
                    "a",
                    "/destroy",
                    "POST",
                    Some(&owned_cookie),
                    Some("before"),
                )
            })
        };
        let result: Result<String> = (|| {
            entered(l, "a")?;
            create(l, "b", false)
        })();
        let release = gate(l, "a", "release");
        let r = joined(pending.join())?;
        release?;
        let new = result?;
        require(
            !r.headers.contains_key("set-cookie"),
            "logout must not clear later Cookie",
        )?;
        reads(l, &old, None, 401, "/read")?;
        reads(l, &new, Some(1), 200, "/read")?;
        let b = {
            let owned_cookie = new.clone();
            scope.spawn(move || request(l, "b", "/rotate", "POST", Some(&owned_cookie), None))
        };
        let c = {
            let owned_cookie = new.clone();
            scope.spawn(move || request(l, "c", "/rotate", "POST", Some(&owned_cookie), None))
        };
        let r = [joined(b.join())?, joined(c.join())?];
        require(
            r.iter().filter(|r| r.status == 200).count() == 1
                && r.iter()
                    .filter(|r| r.status != 200)
                    .all(|r| !r.headers.contains_key("set-cookie")),
            "one rotation winner/loser Cookie suppression",
        )?;
        let winner = r.iter().find(|r| r.status == 200).unwrap().cookie()?;
        reads(l, &new, None, 401, "/read")?;
        reads(l, &winner, Some(1), 200, "/read")?;
        require(
            request(l, "c", "/destroy", "POST", Some(&winner), None)?.status == 200,
            "revocation",
        )?;
        reads(l, &winner, None, 401, "/read")?;
        out["cross_app_rotate_revoke_delayed_logout"] = json!("pass");
        let uncertain = create(l, "a", false)?;
        gate(l, "a", "reset")?;
        let callbacks =
            request(l, "a", "/read", "GET", Some(&uncertain), None)?.json()?["callbacks"]
                .as_u64()
                .ok_or_else(|| fail("callbacks"))?;
        let pending = {
            let owned_cookie = uncertain.clone();
            scope
                .spawn(move || request(l, "a", "/update", "POST", Some(&owned_cookie), Some("cas")))
        };
        let result = entered(l, "a")
            .and_then(|()| redis(l, &["CLIENT", "PAUSE", "700", "WRITE"]).map(|_| ()));
        let release = gate(l, "a", "release");
        let r = joined(pending.join())?;
        release?;
        result?;
        require(
            r.status == 202
                && r.json()?["category"] == "unknown"
                && r.json()?["callbacks"] == callbacks + 1
                && !r.headers.contains_key("set-cookie"),
            "unknown outcome once-only callback/no Cookie",
        )?;
        require(
            request(l, "a", "/read", "GET", Some(&uncertain), None)?.status == 504,
            "unknown retains slot",
        )?;
        std::thread::sleep(Duration::from_millis(350));
        require(
            request(l, "b", "/update", "POST", Some(&uncertain), None)?.status == 200,
            "subsequent worker update",
        )?;
        let r = request(l, "c", "/rotate", "POST", Some(&uncertain), None)?;
        require(r.status == 200, "subsequent rotation")?;
        let fresh = r.cookie()?;
        reads(l, &fresh, Some(3), 200, "/read")?;
        require(
            request(l, "a", "/operation", "GET", None, None)?.json()?["category"] == "committed",
            "historical operation independent of retirement",
        )?;
        reads(l, &uncertain, None, 401, "/read")?;
        out["unknown_outcome_then_other_worker_update_rotate_and_resolve"] = json!("pass");
        std::thread::sleep(Duration::from_millis(5100));
        require(
            request(l, "a", "/operation", "GET", None, None)?.json()?["category"] == "unresolved",
            "expired result unresolved",
        )?;
        reads(l, &uncertain, None, 401, "/read")?;
        out["expired_result_reconciliation_without_replay"] = json!("pass");
        let c = create(l, "a", false)?;
        gate(l, "a", "reset")?;
        let pending = {
            let owned_cookie = c.clone();
            scope
                .spawn(move || request(l, "a", "/cancel-update", "POST", Some(&owned_cookie), None))
        };
        let result = entered(l, "a")
            .and_then(|()| redis(l, &["CLIENT", "PAUSE", "5000", "WRITE"]).map(|_| ()));
        let release = gate(l, "a", "release");
        let r = joined(pending.join())?;
        release?;
        result?;
        require(
            r.json()?["category"] == "caller_cancelled",
            "cancelled caller",
        )?;
        let waiter = {
            let owned_cookie = c.clone();
            scope.spawn(move || request(l, "a", "/read", "GET", Some(&owned_cookie), None))
        };
        std::thread::sleep(Duration::from_millis(20));
        let busy = request(l, "a", "/read", "GET", Some(&c), None);
        let waited = joined(waiter.join())?;
        require(
            busy?.status == 429 && waited.status == 504,
            "bounded capacity and wait",
        )?;
        let then = Instant::now();
        require(
            request(l, "a", "/control/shutdown", "POST", None, None)?.status == 200
                && then.elapsed() < Duration::from_secs(3),
            "bounded driver drain",
        )?;
        require(
            request(l, "a", "/read", "GET", Some(&c), None)?.status == 503
                && request(l, "a", "/healthz", "GET", None, None)?.status == 200,
            "closed admission/stateless health",
        )?;
        redis(l, &["CLIENT", "UNPAUSE"])?;
        out["caller_cancellation_capacity_wait_and_shutdown"] = json!("pass");
        Ok(())
    })?;
    let retained = create(l, "b", false)?;
    l.compose(&["restart", "app-a"], 120)?;
    l.ready("a")?;
    reads(l, &retained, Some(1), 200, "/read")?;
    l.compose(&["stop", "app-a"], 120)?;
    let mut pool = BTreeSet::new();
    for _ in 0..12 {
        pool.insert(
            request(l, "", "/healthz", "GET", None, None)?
                .text()?
                .to_string(),
        );
    }
    require(
        pool == BTreeSet::from(["b".into(), "c".into()]),
        "two Redis survivors",
    )?;
    l.compose(&["start", "app-a"], 120)?;
    l.ready("a")?;
    reads(l, &retained, Some(1), 200, "/read")?;
    out["app_stop_restart_retains_redis_state"] = json!("pass");
    let short = create(l, "a", true)?;
    std::thread::sleep(Duration::from_millis(300));
    require(
        request(l, "b", "/short/update", "POST", Some(&short), None)?.status == 200,
        "short TTL update",
    )?;
    std::thread::sleep(Duration::from_millis(400));
    reads(l, &short, None, 401, "/short/read")?;
    out["absolute_ttl_not_refreshed_by_update"] = json!("pass");
    l.compose(&["stop", "redis"], 120)?;
    for x in ["a", "b", "c"] {
        require(
            [503, 504].contains(&request(l, x, "/read", "GET", Some(&retained), None)?.status)
                && request(l, x, "/healthz", "GET", None, None)?.status == 200,
            "outage fail closed/stateless",
        )?;
    }
    l.compose(&["start", "redis"], 120)?;
    l.compose(
        &["up", "-d", "--wait", "--wait-timeout", "20", "redis"],
        120,
    )?;
    require(records(l)?.is_empty(), "no-persistence Redis restart")?;
    l.compose(&["restart", "app-a", "app-b", "app-c"], 120)?;
    for x in ["a", "b", "c"] {
        l.ready(x)?;
    }
    reads(l, &retained, None, 401, "/read")?;
    let new = create(l, "c", false)?;
    reads(l, &new, Some(1), 200, "/read")?;
    out["outage_and_controlled_no_persistence_restart"] =
        json!("pass; explicit app restart recovery");
    Ok(out)
}
