//! App-owned typed startup settings; RATE_LIMIT_ENABLED=true explicitly opts in.
#![allow(
    clippy::result_large_err,
    reason = "extractors preserve native Axum responses"
)]
use axum::{
    Router,
    http::request::Parts,
    response::Response,
    routing::{MethodRouter, get},
};
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
use yetlet::config::Loader;
use yetlet::rate_limit::{Budget, Key, Policy as RateLimitPolicy, RateLimitLayer};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    enabled: bool,
    policy: RateLimitPolicySettings,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RateLimitPolicySettings {
    #[serde(rename = "interval_ms")]
    refill_interval_ms: u64,
    burst: u32,
    max_keys: usize,
}

impl Settings {
    fn build_rate_limit_policy(&self) -> Result<Option<RateLimitPolicy>, &'static str> {
        // Loading always checks types; disabled settings skip domain validation
        // and construction of Policy, Budget and all limiter request work.
        if !self.enabled {
            return Ok(None);
        }
        RateLimitPolicy::new(
            Duration::from_millis(self.policy.refill_interval_ms),
            self.policy.burst,
            self.policy.max_keys,
        )
        .map(Some)
        .map_err(|_| "invalid rate-limit policy")
    }
}

fn loader() -> Loader {
    Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/rate_limit/settings"),
        "RATE_LIMIT",
    )
}

fn routes<F>(rate_limit_policy: Option<RateLimitPolicy>, extract: F) -> Router
where
    F: Fn(&Parts) -> Result<Key, Response> + Clone + Send + Sync + 'static,
{
    let first: MethodRouter = get(|| async { "first" }).post(|| async { "posted" });
    let second = get(|| async { "second" });
    let public = get(|| async { "public" });
    // Return native method routers before constructing any limiter objects.
    let Some(policy) = rate_limit_policy else {
        return Router::new()
            .route("/first", first)
            .route("/second", second)
            .route("/public", public);
    };
    let shared = Budget::new(policy);
    let layer = RateLimitLayer::new(shared, extract);
    Router::new()
        .route("/first", first.route_layer(layer.clone()))
        .route("/second", second.route_layer(layer))
        .route("/public", public)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut check = false;
    for arg in std::env::args_os().skip(1) {
        if arg == "--check" {
            check = true;
        } else {
            return Err("unsupported example argument".into());
        }
    }
    let settings: Settings = loader().load()?;
    let rate_limit_policy = settings.build_rate_limit_policy()?;
    if check {
        println!("Configuration valid; listener not started.");
        return Ok(());
    }
    // One fixed key intentionally makes both endpoints/methods one group total.
    let app = routes(rate_limit_policy, |_: &Parts| {
        Ok(Key::new("example-group").expect("bounded constant"))
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tower::ServiceExt;

    fn snapshot(values: &[(&str, &str)]) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        values
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect()
    }

    #[tokio::test]
    async fn default_and_explicit_off_skip_all_limiter_work() {
        for values in [vec![], vec![("RATE_LIMIT_ENABLED", "false")]] {
            let settings: Settings = loader().load_with_env(snapshot(&values)).unwrap();
            assert!(!settings.enabled);
            assert_eq!(settings.policy.refill_interval_ms, 2000);
            assert_eq!(settings.policy.burst, 5);
            assert_eq!(settings.policy.max_keys, 64);
            let policy = settings.build_rate_limit_policy().unwrap();
            assert!(policy.is_none());
            let app = routes(policy, |_: &Parts| -> Result<Key, Response> {
                panic!("disabled extraction")
            });
            for _ in 0..12 {
                for (path, method) in [
                    ("/first", "GET"),
                    ("/first", "HEAD"),
                    ("/first", "POST"),
                    ("/second", "GET"),
                    ("/second", "HEAD"),
                    ("/public", "GET"),
                ] {
                    let response = app
                        .clone()
                        .oneshot(
                            Request::builder()
                                .method(method)
                                .uri(path)
                                .body(Body::empty())
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert!(!response.headers().contains_key("retry-after"));
                }
            }
        }
    }

    #[tokio::test]
    async fn enabled_settings_share_custom_budget_and_preserve_public_scope() {
        let settings: Settings = loader()
            .load_with_env(snapshot(&[
                ("RATE_LIMIT_ENABLED", "true"),
                ("RATE_LIMIT_POLICY__INTERVAL_MS", "60000"),
                ("RATE_LIMIT_POLICY__BURST", "2"),
                ("RATE_LIMIT_POLICY__MAX_KEYS", "1"),
            ]))
            .unwrap();
        assert_eq!(settings.policy.refill_interval_ms, 60000);
        assert_eq!(settings.policy.max_keys, 1);
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let app = routes(
            settings.build_rate_limit_policy().unwrap(),
            move |_: &Parts| {
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(Key::new("group").unwrap())
            },
        );
        for (path, method, status) in [
            ("/first", "POST", StatusCode::OK),
            ("/second", "HEAD", StatusCode::OK),
            ("/first", "GET", StatusCode::TOO_MANY_REQUESTS),
            ("/second", "GET", StatusCode::TOO_MANY_REQUESTS),
            ("/public", "GET", StatusCode::OK),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            if status == StatusCode::TOO_MANY_REQUESTS {
                assert_eq!(response.headers()["retry-after"], "60");
            }
        }
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn all_checked_in_profiles_default_disabled() {
        for profile in ["Development", "Staging", "Production"] {
            let settings: Settings = loader()
                .load_with_env(snapshot(&[("RATE_LIMIT_ENVIRONMENT", profile)]))
                .unwrap();
            assert!(!settings.enabled);
            assert_eq!(settings.policy.burst, 5);
        }
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "yetlet-rate-limit-settings-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&path).unwrap();
            std::fs::write(path.join("base.toml"), include_str!("settings/base.toml")).unwrap();
            std::fs::write(path.join("production.toml"), "").unwrap();
            Self(path)
        }
        fn loader(&self) -> Loader {
            Loader::new(&self.0, "RATE_LIMIT")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn base_profile_then_environment_precedence() {
        let fixture = Fixture::new();
        std::fs::write(
            fixture.0.join("production.toml"),
            "enabled = true\n[policy]\nburst = 3\n",
        )
        .unwrap();
        let profile: Settings = fixture.loader().load_with_env([]).unwrap();
        assert!(profile.enabled);
        assert_eq!(profile.policy.burst, 3);
        assert_eq!(profile.policy.refill_interval_ms, 2000);
        let overridden: Settings = fixture
            .loader()
            .load_with_env(snapshot(&[
                ("RATE_LIMIT_ENABLED", "false"),
                ("RATE_LIMIT_POLICY__BURST", "2"),
            ]))
            .unwrap();
        assert!(!overridden.enabled);
        assert_eq!(overridden.policy.burst, 2);
    }

    #[test]
    fn missing_malformed_wrong_type_and_unknown_files_fail() {
        let fixture = Fixture::new();
        let profile = fixture.0.join("production.toml");
        std::fs::remove_file(&profile).unwrap();
        assert!(fixture.loader().load_with_env::<Settings>([]).is_err());
        for contents in [
            "[",
            "enabled = 'false'",
            "unknown = 1",
            "[policy]\nunknown = 1",
            "[policy]\nrefill_interval_ms = 2000",
        ] {
            std::fs::write(&profile, contents).unwrap();
            assert!(fixture.loader().load_with_env::<Settings>([]).is_err());
        }
        std::fs::write(&profile, "").unwrap();
        std::fs::remove_file(fixture.0.join("base.toml")).unwrap();
        assert!(fixture.loader().load_with_env::<Settings>([]).is_err());
    }

    #[test]
    fn invalid_environment_types_and_keys_fail_even_when_disabled() {
        for (name, value) in [
            ("RATE_LIMIT_ENABLED", "yes"),
            ("RATE_LIMIT_POLICY__BURST", "4294967296"),
            ("RATE_LIMIT_POLICY__INTERVAL_MS", "invalid"),
            ("RATE_LIMIT_POLICY__MAX_KEYS", "-1"),
            ("RATE_LIMIT_POLICY__UNKNOWN", "1"),
            ("RATE_LIMIT_POLICY__REFILL_INTERVAL_MS", "2000"),
            ("RATE_LIMIT_BURST", "2"),
            ("RATE_LIMIT_ENVIRONMENT", "production"),
        ] {
            assert!(
                loader()
                    .load_with_env::<Settings>(snapshot(&[(name, value)]))
                    .is_err()
            );
        }
    }

    #[test]
    fn enabled_invalid_domains_fail_but_disabled_skips_policy_construction() {
        for (name, value) in [
            ("RATE_LIMIT_POLICY__INTERVAL_MS", "0"),
            ("RATE_LIMIT_POLICY__INTERVAL_MS", "86400001"),
            ("RATE_LIMIT_POLICY__BURST", "0"),
            ("RATE_LIMIT_POLICY__BURST", "4294967295"),
            ("RATE_LIMIT_POLICY__MAX_KEYS", "0"),
            ("RATE_LIMIT_POLICY__MAX_KEYS", "4097"),
            ("RATE_LIMIT_POLICY__INTERVAL_MS", "20000000"),
        ] {
            for enabled in ["true", "false"] {
                let settings: Settings = loader()
                    .load_with_env(snapshot(&[("RATE_LIMIT_ENABLED", enabled), (name, value)]))
                    .unwrap();
                if enabled == "true" {
                    assert!(settings.build_rate_limit_policy().is_err());
                } else {
                    assert!(settings.build_rate_limit_policy().unwrap().is_none());
                }
            }
        }
    }
}
