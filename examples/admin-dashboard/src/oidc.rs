//! Startup-only secret-provider composition, separate from the Board's client.
use std::{net::SocketAddr, sync::Arc};
use yetlet_lab_operations_board::{
    Failure,
    oidc::{ClientSecret, Oidc},
};
use yetlet::secrets::{ExposeSecret, SecretProvider};

pub async fn configure(
    settings: &crate::config::Oidc,
    address: SocketAddr,
    secrets: Arc<dyn SecretProvider>,
) -> Result<Oidc, Failure> {
    let secret = secrets
        .get("oidc-client-secret")
        .await
        .map_err(|_| Failure::Unavailable)?
        .filter(|value| !value.expose_secret().is_empty())
        .ok_or(Failure::Invalid)?;
    Oidc::discover(
        &settings.issuer,
        &settings.client_id,
        &format!(
            "{}/board/oidc/callback",
            settings
                .application_origin(address)
                .map_err(|_| Failure::Invalid)?
        ),
        settings.subject_map().map_err(|_| Failure::Invalid)?,
        ClientSecret::new(secret.expose_secret().to_owned()),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Unavailable;
    impl SecretProvider for Unavailable {
        fn get<'a>(&'a self, name: &'a str) -> yetlet::secrets::SecretFuture<'a> {
            assert_eq!(name, "oidc-client-secret");
            Box::pin(async { Err(yetlet::secrets::SecretError::AccessDenied) })
        }
    }
    #[tokio::test]
    async fn injected_provider_failure_has_no_backend_or_credential_details() {
        let settings = crate::config::load_with_env(vec![]).unwrap();
        assert!(matches!(
            configure(
                &settings.oidc,
                "127.0.0.1:3014".parse().unwrap(),
                Arc::new(Unavailable)
            )
            .await,
            Err(Failure::Unavailable)
        ));
    }
}
