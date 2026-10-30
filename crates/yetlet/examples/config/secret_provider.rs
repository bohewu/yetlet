//! Listener-free, opt-in secret resolution and narrow module settings.
use axum::{Router, body::Body, extract::State, http::Request};
use serde::Deserialize;
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use yetlet::config::{Loader, SecretBinding};
use yetlet::secrets::{SecretProvider, SecretString, SettingsSecrets};
use yetlet::server_axum::{App, Page};
use yetlet::view::View;

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

mod delivery {
    use yetlet::secrets::{ExposeSecret, SecretString};
    pub struct Settings {
        pub token: SecretString,
        pub attempts: u16,
    }
    pub struct Client {
        token: SecretString,
    }
    impl Client {
        pub fn new(settings: Settings) -> Result<Self, &'static str> {
            if settings.token.expose_secret().trim().is_empty() || settings.attempts == 0 {
                return Err("delivery configuration is invalid");
            }
            Ok(Self {
                token: settings.token,
            })
        }
        pub fn configured(&self) -> bool {
            !self.token.expose_secret().is_empty()
        }
    }
}

struct WebState {
    public_message: String,
    delivery: delivery::Client,
}
async fn home(State(state): State<Arc<WebState>>) -> Page {
    let message = if state.delivery.configured() {
        state.public_message.as_str()
    } else {
        "Service unavailable"
    };
    Page::new("Explicit secret settings", View::text(message))
}
fn application(settings: Settings) -> Result<Router, &'static str> {
    if !settings.server.bind.ip().is_loopback() || !settings.service.enabled {
        return Err("application configuration is invalid");
    }
    let delivery = delivery::Client::new(delivery::Settings {
        token: settings.service.token,
        attempts: settings.service.attempts,
    })?;
    // Web state receives public presentation data and the constructed service,
    // not the whole settings object or an unrestricted provider handle.
    let state = Arc::new(WebState {
        public_message: settings.service.public_message,
        delivery,
    });
    Ok(App::new().get("/", home).with_state(state).into_router())
}

async fn settings(provider: &dyn SecretProvider) -> Result<Settings, yetlet::config::Error> {
    Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/config/settings"),
        "YTCONFIG",
    )
    .load_with_env_and_secrets(
        [("YTCONFIG_ENVIRONMENT".into(), "Production".into())],
        provider,
        &[SecretBinding::new("delivery-token", |s: &mut Settings| {
            &mut s.service.token
        })],
    )
    .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Synthetic local source for this example; a real app injects its separately
    // configured read-only provider. No environment/network discovery occurs.
    let provider = SettingsSecrets::new(BTreeMap::from([(
        "delivery-token".into(),
        SecretString::from("synthetic-provider-only"),
    )]));
    let response = application(settings(&provider).await?)?
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;
    println!("Example request status: {}", response.status());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    #[tokio::test]
    async fn resolved_settings_reach_only_the_selected_service_and_not_ssr() {
        let value = "synthetic-provider-only";
        let provider = SettingsSecrets::new(BTreeMap::from([(
            "delivery-token".into(),
            SecretString::from(value),
        )]));
        let loaded = settings(&provider).await.unwrap();
        assert!(!format!("{loaded:?}").contains(value));
        let response = application(loaded)
            .unwrap()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert!(response.headers().contains_key("content-security-policy"));
        let html = String::from_utf8(to_bytes(response.into_body(), 8192).await.unwrap().to_vec())
            .unwrap();
        assert!(html.contains("Production configuration example"));
        assert!(!html.contains(value) && !html.contains("delivery-token"));
    }
    #[tokio::test]
    async fn missing_and_empty_required_secrets_do_not_construct_an_application() {
        let provider = SettingsSecrets::new(BTreeMap::new());
        assert_eq!(
            settings(&provider).await.unwrap_err(),
            yetlet::config::Error::SecretMissing
        );
        provider
            .replace(BTreeMap::from([(
                "delivery-token".into(),
                SecretString::from(""),
            )]))
            .unwrap();
        assert!(application(settings(&provider).await.unwrap()).is_err());
    }
}
