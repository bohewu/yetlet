use std::{collections::BTreeMap, error::Error as _, sync::Arc};
use yetlet_secrets::{
    ExposeSecret, SecretError, SecretFuture, SecretProvider, SecretString, SettingsSecrets,
};

#[tokio::test]
async fn snapshot_returns_owned_exact_strings_and_distinguishes_empty_from_missing() {
    let provider = SettingsSecrets::new(BTreeMap::from([
        (
            "private-name".into(),
            SecretString::from("000$synthetic-private"),
        ),
        ("empty".into(), SecretString::from("")),
        ("空白".into(), SecretString::from("  ")),
    ]));
    let value = provider.get("private-name").await.unwrap().unwrap();
    assert_eq!(value.expose_secret(), "000$synthetic-private");
    assert_eq!(
        provider
            .get("empty")
            .await
            .unwrap()
            .unwrap()
            .expose_secret(),
        ""
    );
    assert_eq!(
        provider.get("空白").await.unwrap().unwrap().expose_secret(),
        "  "
    );
    assert!(provider.get("missing").await.unwrap().is_none());
    assert!(provider.get("PRIVATE-NAME").await.unwrap().is_none());
    assert_eq!(format!("{provider:?}"), "SettingsSecrets { .. }");
    assert!(!format!("{value:?}").contains("000$synthetic-private"));
    drop(provider);
    assert_eq!(value.expose_secret(), "000$synthetic-private");
}

struct CustomProvider;

#[tokio::test]
async fn replacing_snapshot_updates_shared_handle_and_preserves_owned_values() {
    let provider = Arc::new(SettingsSecrets::new(BTreeMap::from([
        ("token".into(), SecretString::from("synthetic-v1")),
        ("retained".into(), SecretString::from("other-synthetic")),
        ("removed".into(), SecretString::from("remove-synthetic")),
    ])));
    let shared: Arc<dyn SecretProvider> = provider.clone();
    let old = shared.get("token").await.unwrap().unwrap();
    provider
        .replace(BTreeMap::from([
            ("token".into(), SecretString::from("synthetic-v2")),
            ("retained".into(), SecretString::from("other-synthetic")),
            ("empty".into(), SecretString::from("")),
        ]))
        .unwrap();
    assert_eq!(
        shared.get("token").await.unwrap().unwrap().expose_secret(),
        "synthetic-v2"
    );
    assert_eq!(old.expose_secret(), "synthetic-v1");
    assert_eq!(
        shared
            .get("retained")
            .await
            .unwrap()
            .unwrap()
            .expose_secret(),
        "other-synthetic"
    );
    assert!(shared.get("removed").await.unwrap().is_none());
    assert!(shared.get("missing").await.unwrap().is_none());
    assert_eq!(
        shared.get("empty").await.unwrap().unwrap().expose_secret(),
        ""
    );
    provider.replace(BTreeMap::new()).unwrap();
    assert!(shared.get("token").await.unwrap().is_none());
}

impl SecretProvider for CustomProvider {
    fn get<'a>(&'a self, name: &'a str) -> SecretFuture<'a> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            match name {
                "denied" => Err(SecretError::AccessDenied),
                "offline" => Err(SecretError::Unavailable),
                "present" => Ok(Some(SecretString::from("replacement-synthetic"))),
                _ => Ok(None),
            }
        })
    }
}

#[tokio::test]
async fn custom_provider_is_replaceable_and_send_with_distinct_outcomes() {
    let provider: Arc<dyn SecretProvider> = Arc::new(CustomProvider);
    tokio::spawn(async move {
        assert_eq!(
            provider.get("denied").await.unwrap_err(),
            SecretError::AccessDenied
        );
        assert_eq!(
            provider.get("offline").await.unwrap_err(),
            SecretError::Unavailable
        );
        assert!(provider.get("missing").await.unwrap().is_none());
        assert_eq!(
            provider
                .get("present")
                .await
                .unwrap()
                .unwrap()
                .expose_secret(),
            "replacement-synthetic"
        );
    })
    .await
    .unwrap();
}

#[test]
fn errors_have_fixed_safe_messages_and_no_sources() {
    for (error, message) in [
        (SecretError::Unavailable, "secret provider unavailable"),
        (SecretError::AccessDenied, "secret access denied"),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(error.source().is_none());
        assert!(!format!("{error:?}: {error}").contains("synthetic-private"));
    }
}
