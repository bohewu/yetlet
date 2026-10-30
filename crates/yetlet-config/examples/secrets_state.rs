//! Listener-free native composition example; synthetic values only.
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use yetlet_config::Loader;
use yetlet_core::View;
use yetlet_secrets::{ExposeSecret, SecretError, SecretProvider, SecretString, SettingsSecrets};
use yetlet_server_axum::{App, Page};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    service: ServiceSettings,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceSettings {
    public_message: String,
    token: SecretString,
}

// An ordinary service constructor, with no framework registration or container.
struct DeliveryService {
    secrets: Arc<dyn SecretProvider>,
}

impl DeliveryService {
    fn new(secrets: Arc<dyn SecretProvider>) -> Self {
        Self { secrets }
    }

    async fn configured(&self) -> Result<bool, SecretError> {
        let token = self.secrets.get("delivery-token").await?;
        // Explicit exposure stays inside the consumer. This example makes no
        // external request and does not implement authentication.
        Ok(token.is_some_and(|value| !value.expose_secret().is_empty()))
    }
}

#[derive(Clone)]
struct AppState {
    service: Arc<DeliveryService>,
    public_message: String,
}

fn loader() -> Loader {
    Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/secrets-state-settings"),
        "YTSECRETS",
    )
}

fn provider(settings: ServiceSettings) -> (String, Arc<dyn SecretProvider>) {
    // The public lookup name is an app-owned mapping, not a settings namespace.
    let values = BTreeMap::from([("delivery-token".into(), settings.token)]);
    (
        settings.public_message,
        Arc::new(SettingsSecrets::new(values)),
    )
}

fn app(public_message: String, secrets: Arc<dyn SecretProvider>) -> Router {
    let state = AppState {
        service: Arc::new(DeliveryService::new(secrets)),
        public_message,
    };
    App::new().get("/", home).with_state(state).into_router()
}

async fn home(State(state): State<AppState>) -> (StatusCode, Page) {
    let (status, message) = match state.service.configured().await {
        Ok(true) => (StatusCode::OK, state.public_message.as_str()),
        Ok(false) => (StatusCode::SERVICE_UNAVAILABLE, "Service not configured"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable"),
    };
    (status, Page::new("Secret provider", View::text(message)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: Settings = loader().load()?;
    let (message, secrets) = provider(settings.service);
    let response = app(message, secrets)
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;
    println!("Example request status: {}", response.status());
    // Consume the response without logging the HTML or any secret.
    to_bytes(response.into_body(), 8192).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::OsString,
        fs,
        sync::{
            Mutex, TryLockError,
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
        thread,
        time::Duration,
    };
    use yetlet_config::SecretFormat;
    use yetlet_secrets::SecretFuture;

    struct PrivateFile(PathBuf);

    impl PrivateFile {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let directory = std::env::temp_dir().join(format!(
                "yetlet-secrets-example-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            fs::write(
                directory.join("private.toml"),
                "[service]\ntoken = '000$development-synthetic'\n",
            )
            .unwrap();
            Self(directory)
        }

        fn loader(&self) -> Loader {
            loader().development_secrets(self.0.join("private.toml"), SecretFormat::Toml)
        }
    }

    impl Drop for PrivateFile {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    // Example-only application policy: every reload writer uses this entry.
    // Acquire the gate before reading inputs, and keep loading, validation,
    // selection and publication synchronous. No lock crosses an await.
    fn reload(
        secrets: &SettingsSecrets,
        gate: &Mutex<()>,
        load: impl FnOnce() -> Result<Settings, yetlet_config::Error>,
    ) -> Result<(), &'static str> {
        let _reload = gate.lock().map_err(|_| "reload unavailable")?;
        let settings = load().map_err(|_| "settings load failed")?;
        // This test application requires a nonempty token. SettingsSecrets
        // itself permits empty strings; validation belongs to the consumer.
        if settings.service.token.expose_secret().is_empty() {
            return Err("application validation failed");
        }
        let selected = BTreeMap::from([("delivery-token".into(), settings.service.token)]);
        secrets
            .replace(selected)
            .map_err(|_| "snapshot publication failed")
    }

    async fn lookup(loader: Loader, environment: &[(&str, &str)]) -> SecretString {
        let settings: Settings = loader
            .load_with_env(
                environment
                    .iter()
                    .map(|(k, v)| (OsString::from(k), OsString::from(v))),
            )
            .unwrap();
        assert!(!format!("{settings:?}").contains("000$development-synthetic"));
        let (_, secrets) = provider(settings.service);
        secrets.get("delivery-token").await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn existing_loader_precedence_flows_into_selected_protected_strings() {
        let private = PrivateFile::new();
        // Production inherits the empty base field; Development overrides it.
        assert_eq!(lookup(loader(), &[]).await.expose_secret(), "");
        assert_eq!(
            lookup(loader(), &[("YTSECRETS_ENVIRONMENT", "Development")])
                .await
                .expose_secret(),
            "development-placeholder"
        );
        assert_eq!(
            lookup(
                private.loader(),
                &[("YTSECRETS_ENVIRONMENT", "Development")]
            )
            .await
            .expose_secret(),
            "000$development-synthetic"
        );
        assert_eq!(
            lookup(
                private.loader(),
                &[
                    ("YTSECRETS_ENVIRONMENT", "Development"),
                    ("YTSECRETS_SERVICE__TOKEN", "false"),
                ]
            )
            .await
            .expose_secret(),
            "false"
        );
    }

    #[tokio::test]
    async fn staging_and_production_ignore_development_secret_selection() {
        let private = PrivateFile::new();
        fs::remove_file(private.0.join("private.toml")).unwrap();
        for profile in ["Staging", "Production"] {
            assert_eq!(
                lookup(private.loader(), &[("YTSECRETS_ENVIRONMENT", profile)])
                    .await
                    .expose_secret(),
                ""
            );
        }
    }

    #[tokio::test]
    async fn explicit_development_file_reload_updates_the_existing_service_handle() {
        let private = PrivateFile::new();
        let selected_loader = private.loader();
        let environment = || {
            [(
                OsString::from("YTSECRETS_ENVIRONMENT"),
                OsString::from("Development"),
            )]
        };
        let first: Settings = selected_loader.load_with_env(environment()).unwrap();
        let secrets = Arc::new(SettingsSecrets::new(BTreeMap::from([(
            "delivery-token".into(),
            first.service.token,
        )])));
        let service = DeliveryService::new(secrets.clone());
        let reload_gate = Mutex::new(());
        let retained = service
            .secrets
            .get("delivery-token")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retained.expose_secret(), "000$development-synthetic");
        assert!(service.configured().await.unwrap());

        // External source changes and adoption are separate, explicit steps.
        fs::write(
            private.0.join("private.toml"),
            "[service]\ntoken = '001$reloaded-synthetic'\n",
        )
        .unwrap();
        assert_eq!(
            service
                .secrets
                .get("delivery-token")
                .await
                .unwrap()
                .unwrap()
                .expose_secret(),
            "000$development-synthetic"
        );
        reload(&secrets, &reload_gate, || {
            selected_loader.load_with_env(environment())
        })
        .unwrap();
        assert_eq!(
            service
                .secrets
                .get("delivery-token")
                .await
                .unwrap()
                .unwrap()
                .expose_secret(),
            "001$reloaded-synthetic"
        );
        assert!(service.configured().await.unwrap());
        assert_eq!(retained.expose_secret(), "000$development-synthetic");
    }

    #[tokio::test]
    async fn competing_reloads_serialize_source_reads_and_snapshot_publication() {
        let private = PrivateFile::new();
        let selected_loader = private.loader();
        let environment = || [("YTSECRETS_ENVIRONMENT".into(), "Development".into())];
        let secrets = SettingsSecrets::new(BTreeMap::from([(
            "delivery-token".into(),
            SecretString::from("initial-synthetic"),
        )]));
        let reload_gate = Mutex::new(());
        let (events, observed) = mpsc::channel();
        let (release, held) = mpsc::channel();
        let runtime = tokio::runtime::Handle::current();

        thread::scope(|scope| {
            let first = scope.spawn({
                let secrets = &secrets;
                let reload_gate = &reload_gate;
                let selected_loader = &selected_loader;
                let events = &events;
                move || {
                    reload(secrets, reload_gate, || {
                        let settings: Settings = selected_loader.load_with_env(environment())?;
                        assert_eq!(
                            settings.service.token.expose_secret(),
                            "000$development-synthetic"
                        );
                        events.send("A read old source").unwrap();
                        held.recv_timeout(Duration::from_secs(5)).unwrap();
                        Ok(settings)
                    })
                }
            });
            assert_eq!(
                observed.recv_timeout(Duration::from_secs(5)).unwrap(),
                "A read old source"
            );

            let second = scope.spawn(|| {
                assert!(matches!(
                    reload_gate.try_lock(),
                    Err(TryLockError::WouldBlock)
                ));
                events.send("B attempted reload").unwrap();
                reload(&secrets, &reload_gate, || {
                    // Reaching the next source read requires A to publish first.
                    assert_eq!(
                        runtime
                            .block_on(secrets.get("delivery-token"))
                            .unwrap()
                            .unwrap()
                            .expose_secret(),
                        "000$development-synthetic"
                    );
                    events.send("B observed A publication").unwrap();
                    let settings: Settings = selected_loader.load_with_env(environment())?;
                    assert_eq!(
                        settings.service.token.expose_secret(),
                        "001$latest-synthetic"
                    );
                    events.send("B read latest source").unwrap();
                    Ok(settings)
                })
            });
            assert_eq!(
                observed.recv_timeout(Duration::from_secs(5)).unwrap(),
                "B attempted reload"
            );
            assert!(matches!(
                reload_gate.try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            assert_eq!(observed.try_recv(), Err(mpsc::TryRecvError::Empty));
            fs::write(
                private.0.join("private.toml"),
                "[service]\ntoken = '001$latest-synthetic'\n",
            )
            .unwrap();
            release.send(()).unwrap();
            first.join().unwrap().unwrap();
            second.join().unwrap().unwrap();
        });

        assert_eq!(
            observed.recv_timeout(Duration::from_secs(5)).unwrap(),
            "B observed A publication"
        );
        assert_eq!(
            observed.recv_timeout(Duration::from_secs(5)).unwrap(),
            "B read latest source"
        );
        assert_eq!(
            secrets
                .get("delivery-token")
                .await
                .unwrap()
                .unwrap()
                .expose_secret(),
            "001$latest-synthetic"
        );
    }

    #[tokio::test]
    async fn failed_reload_retains_snapshot_and_allows_subsequent_valid_reload() {
        let private = PrivateFile::new();
        let selected_loader = private.loader();
        let secrets = Arc::new(SettingsSecrets::new(BTreeMap::new()));
        let service = DeliveryService::new(secrets.clone());
        let reload_gate = Mutex::new(());
        let load = || {
            selected_loader.load_with_env([("YTSECRETS_ENVIRONMENT".into(), "Development".into())])
        };
        reload(&secrets, &reload_gate, load).unwrap();
        let retained = service
            .secrets
            .get("delivery-token")
            .await
            .unwrap()
            .unwrap();

        for (input, expected_error) in [
            ("[service]\ntoken = [\n", "settings load failed"),
            ("[service]\ntoken = ''\n", "application validation failed"),
        ] {
            fs::write(private.0.join("private.toml"), input).unwrap();
            assert_eq!(reload(&secrets, &reload_gate, load), Err(expected_error));
            assert_eq!(
                service
                    .secrets
                    .get("delivery-token")
                    .await
                    .unwrap()
                    .unwrap()
                    .expose_secret(),
                "000$development-synthetic"
            );
            assert!(service.configured().await.unwrap());
        }

        fs::write(
            private.0.join("private.toml"),
            "[service]\ntoken = '002$recovered-synthetic'\n",
        )
        .unwrap();
        reload(&secrets, &reload_gate, load).unwrap();
        assert_eq!(
            service
                .secrets
                .get("delivery-token")
                .await
                .unwrap()
                .unwrap()
                .expose_secret(),
            "002$recovered-synthetic"
        );
        assert_eq!(retained.expose_secret(), "000$development-synthetic");
    }

    async fn response(secrets: Arc<dyn SecretProvider>) -> (StatusCode, String) {
        let response = app("Safe public <message>".into(), secrets)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(response.headers().contains_key("content-security-policy"));
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(!html.contains("synthetic-private"));
        assert!(!html.contains("delivery-token"));
        assert!(!html.contains("<script"));
        (status, html)
    }

    #[tokio::test]
    async fn selected_settings_reach_service_and_state_without_secret_in_ssr_or_debug() {
        let settings: Settings = loader()
            .load_with_env([
                ("YTSECRETS_ENVIRONMENT".into(), "Production".into()),
                (
                    "YTSECRETS_SERVICE__TOKEN".into(),
                    "001-synthetic-private".into(),
                ),
            ])
            .unwrap();
        assert!(!format!("{settings:?}").contains("001-synthetic-private"));
        let (_, secrets) = provider(settings.service);
        let (status, html) = response(secrets).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Safe public &lt;message&gt;"));
    }

    struct Replacement(Result<Option<SecretString>, SecretError>);

    impl SecretProvider for Replacement {
        fn get<'a>(&'a self, _: &'a str) -> SecretFuture<'a> {
            Box::pin(async move {
                tokio::task::yield_now().await;
                self.0.clone()
            })
        }
    }

    #[tokio::test]
    async fn same_service_accepts_custom_provider_and_keeps_failures_out_of_ssr() {
        let (status, _) = response(Arc::new(Replacement(Ok(Some(SecretString::from(
            "replacement-synthetic-private",
        ))))))
        .await;
        assert_eq!(status, StatusCode::OK);
        for result in [
            Ok(None),
            Ok(Some(SecretString::from(""))),
            Err(SecretError::AccessDenied),
            Err(SecretError::Unavailable),
        ] {
            let (status, html) = response(Arc::new(Replacement(result))).await;
            assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
            assert!(!html.contains("AccessDenied"));
            assert!(!html.contains("secret provider unavailable"));
        }
    }
}
