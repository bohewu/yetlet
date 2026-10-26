//! Native startup consumer; the token is a synthetic configuration placeholder,
//! not an authentication implementation or production credential.
use axum::{Router, extract::State};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use yetlet_config::{Loader, SecretFormat};
use yetlet_core::View;
use yetlet_server_axum::{App, Page};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    server: Server,
    service: Service,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Server {
    bind: SocketAddr,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Service {
    public_message: String,
    token: SecretString,
    attempts: u16,
    enabled: bool,
}

impl Settings {
    // App-owned validation happens before constructing a listener. Exposing a
    // secret for validation is explicit and never becomes a response/log value.
    fn validate(&self) -> Result<(), &'static str> {
        if self.service.token.expose_secret().trim().is_empty()
            || self.service.public_message.is_empty()
            || self.service.public_message.len() > 80
            || self.service.attempts == 0
            || !self.service.enabled
            || !self.server.bind.ip().is_loopback()
        {
            return Err("application configuration validation failed");
        }
        Ok(())
    }
}

fn loader() -> Loader {
    Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/settings"),
        "YTCONFIG",
    )
}

async fn home(State(settings): State<Arc<Settings>>) -> Page {
    // Only an explicitly selected public field enters escaped SSR.
    Page::new(
        "Typed configuration",
        View::text(&settings.service.public_message),
    )
}

fn app(settings: Settings) -> Router {
    App::new()
        .get("/", home)
        .with_state(Arc::new(settings))
        .into_router()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut check = false;
    let mut loader = loader();
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--check" {
            check = true;
        } else if arg == "--dev-secrets" {
            let path = args.next().ok_or("missing development secret file")?;
            loader = loader.development_secrets(path, SecretFormat::Dotenv);
        } else {
            return Err("unsupported example argument".into());
        }
    }
    let settings: Settings = loader.load()?;
    settings.validate()?;
    if check {
        println!("Configuration valid; listener not started.");
        return Ok(());
    }
    let address = settings.server.bind;
    let router = app(settings);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|_| "example listener could not bind")?;
    axum::serve(listener, router)
        .await
        .map_err(|_| "example server failed")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    fn settings(token: &str) -> Settings {
        loader()
            .load_with_env([
                ("YTCONFIG_ENVIRONMENT".into(), "Production".into()),
                ("YTCONFIG_SERVICE__TOKEN".into(), token.into()),
            ])
            .unwrap()
    }

    #[test]
    fn validation_rejects_required_empty_secret_and_invalid_app_values() {
        assert!(settings("").validate().is_err());
        assert!(settings("   ").validate().is_err());
        let mut config = settings("synthetic-token");
        config.service.attempts = 0;
        assert!(config.validate().is_err());
        config.service.attempts = 1;
        config.server.bind = "0.0.0.0:3000".parse().unwrap();
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn typed_state_ssr_and_debug_do_not_expose_secret() {
        let config = settings("001234-synthetic-private");
        config.validate().unwrap();
        assert!(!format!("{config:?}").contains("001234-synthetic-private"));
        let response = app(config)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert!(response.headers().contains_key("content-security-policy"));
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("Production configuration example"));
        assert!(!html.contains("001234-synthetic-private"));
        assert!(!html.contains("token"));
        assert!(!html.contains("<script"));
    }
}
