//! Opt-in, application-owned OIDC. No provider roles or email grant access.
use crate::sessions::Session;
use crate::{
    AppState, Authentication, CsrfForm, Failure,
    domain::{LoginSession, Principal, principal},
    require_csrf,
};
use axum::{
    Form,
    extract::{
        Query, State,
        rejection::{FormRejection, QueryRejection},
    },
    response::{IntoResponse, Redirect, Response},
};
pub use openidconnect::ClientSecret;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest,
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};
use subtle::ConstantTimeEq;

type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// One explicitly selected issuer and subject-to-existing-local-identity map.
pub struct Oidc {
    client: Client,
    http: reqwest::Client,
    issuer: String,
    subjects: BTreeMap<String, Principal>,
}

impl Oidc {
    pub(super) fn application_origin(&self) -> Result<String, Failure> {
        Ok(self
            .client
            .redirect_uri()
            .ok_or(Failure::Invalid)?
            .url()
            .origin()
            .ascii_serialization())
    }

    pub(super) fn login_csp(&self) -> String {
        let origin = self.client.auth_uri().url().origin().ascii_serialization();
        crate::DEFAULT_CSP.replace(
            "form-action 'self'",
            &format!("form-action 'self' {origin}"),
        )
    }

    /// Discover once using a caller-resolved credential. The application owns
    /// settings/provider composition; already-built clients retain this snapshot.
    pub async fn discover(
        issuer: &str,
        client_id: &str,
        callback: &str,
        subjects: BTreeMap<String, String>,
        secret: ClientSecret,
    ) -> Result<Self, Failure> {
        let issuer = IssuerUrl::new(issuer.to_owned()).map_err(|_| Failure::Invalid)?;
        let url = issuer.url();
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || client_id.is_empty()
            || client_id.len() > 256
            || client_id.chars().any(char::is_control)
            || secret.secret().is_empty()
        {
            return Err(Failure::Invalid);
        }
        let subjects = subject_map(subjects)?;
        let callback = RedirectUrl::new(callback.to_owned()).map_err(|_| Failure::Invalid)?;
        let url = callback.url();
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != "/board/oidc/callback"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(Failure::Invalid);
        }
        let http = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| Failure::Unavailable)?;
        let metadata = CoreProviderMetadata::discover_async(issuer.clone(), &http)
            .await
            .map_err(|_| Failure::Unavailable)?;
        // This slice supports confidential code flow using HTTPS endpoints.
        if metadata.authorization_endpoint().url().scheme() != "https"
            || metadata
                .token_endpoint()
                .is_none_or(|url| url.url().scheme() != "https")
            || metadata.jwks_uri().url().scheme() != "https"
        {
            return Err(Failure::Invalid);
        }
        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(client_id.to_owned()),
            Some(secret),
        )
        .set_redirect_uri(callback);
        Ok(Self {
            client,
            http,
            issuer: issuer.as_str().to_owned(),
            subjects,
        })
    }

    fn start(&self) -> (String, Pending) {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();
        (
            url.to_string(),
            Pending {
                state: state.secret().clone(),
                nonce,
                verifier: verifier.secret().clone(),
                deadline: SystemTime::now() + Duration::from_secs(300),
            },
        )
    }

    async fn finish(&self, code: String, pending: Pending) -> Result<String, Failure> {
        let http = self.http.clone();
        let transport = move |request| {
            let http = http.clone();
            async move {
                let response = openidconnect::AsyncHttpClient::call(&http, request).await;
                if let Ok(response) = &response {
                    eprintln!("OIDC_TOKEN_RESPONSE status={}", response.status().as_u16());
                }
                response
            }
        };
        let response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| Failure::Unavailable)?
            .set_pkce_verifier(PkceCodeVerifier::new(pending.verifier))
            .request_async(&transport)
            .await
            .map_err(|error| {
                let stage = match error {
                    openidconnect::RequestTokenError::ServerResponse(response) => {
                        match response.error().as_ref() {
                            "invalid_client" => "token-invalid-client",
                            "invalid_grant" => "token-invalid-grant",
                            _ => "token-server-rejection",
                        }
                    }
                    openidconnect::RequestTokenError::Request(_) => "token-transport",
                    openidconnect::RequestTokenError::Parse(_, _) => "token-response-format",
                    _ => "token-response",
                };
                eprintln!("OIDC_SIGN_IN_FAILED stage={stage}");
                Failure::Unauthorized
            })?;
        let token = response.id_token().ok_or(Failure::Unauthorized)?;
        let verifier = self.client.id_token_verifier();
        let claims = token.claims(&verifier, &pending.nonce).map_err(|error| {
            let stage = match error {
                openidconnect::ClaimsVerificationError::Expired(_) => "id-token-expired",
                openidconnect::ClaimsVerificationError::InvalidAudience(_) => "id-token-audience",
                openidconnect::ClaimsVerificationError::InvalidIssuer(_) => "id-token-issuer",
                openidconnect::ClaimsVerificationError::InvalidNonce(_) => "id-token-nonce",
                openidconnect::ClaimsVerificationError::SignatureVerification(_) => {
                    "id-token-signature"
                }
                _ => "id-token-claims",
            };
            eprintln!("OIDC_SIGN_IN_FAILED stage={stage}");
            Failure::Unauthorized
        })?;
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                response.access_token(),
                token.signing_alg().map_err(|_| Failure::Unauthorized)?,
                token
                    .signing_key(&verifier)
                    .map_err(|_| Failure::Unauthorized)?,
            )
            .map_err(|_| Failure::Unauthorized)?;
            if actual != *expected {
                return Err(Failure::Unauthorized);
            }
        }
        Ok(claims.subject().as_str().to_owned())
    }
}

fn subject_map(values: BTreeMap<String, String>) -> Result<BTreeMap<String, Principal>, Failure> {
    if values.len() > 3 {
        return Err(Failure::Invalid);
    }
    let mut result = BTreeMap::new();
    for (subject, name) in values {
        let user = principal(&name).ok_or(Failure::Invalid)?;
        if subject.is_empty() || subject.len() > 255 || subject.chars().any(char::is_control) {
            return Err(Failure::Invalid);
        }
        result.insert(subject, user);
    }
    Ok(result)
}

#[derive(Clone)]
#[cfg_attr(feature = "valkey", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "valkey", serde(deny_unknown_fields))]
pub(crate) struct Pending {
    state: String,
    nonce: Nonce,
    verifier: String,
    deadline: SystemTime,
}
impl std::fmt::Debug for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pending { .. }")
    }
}
impl Pending {
    fn accepts(&self, state: &str) -> bool {
        self.deadline > SystemTime::now()
            && bool::from(self.state.as_bytes().ct_eq(state.as_bytes()))
    }
}

pub(crate) async fn begin(
    State(state): State<AppState>,
    session: Session,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Result<Response, Failure> {
    let Form(form) = form.map_err(|_| Failure::Invalid)?;
    require_csrf(&session, &form.csrf).await?;
    let Authentication::Oidc(oidc) = &state.authentication else {
        return Err(Failure::Forbidden);
    };
    let (url, pending) = oidc.start();
    session
        .replace_and_rotate(LoginSession {
            user: None,
            oidc: Some(pending),
        })
        .await?;
    Ok(Redirect::to(&url).into_response())
}

#[derive(Deserialize)]
pub(crate) struct Callback {
    state: Option<String>,
    code: Option<String>,
    iss: Option<String>,
    error: Option<String>,
}
pub(crate) async fn callback(
    State(state): State<AppState>,
    session: Session,
    query: Result<Query<Callback>, QueryRejection>,
) -> Result<Response, Failure> {
    let Authentication::Oidc(oidc) = &state.authentication else {
        return Err(Failure::Forbidden);
    };
    let Query(query) = query.map_err(|_| Failure::Invalid)?;
    // Atomic take prevents duplicate or concurrent callback consumption. Invalid
    // callbacks retire the transaction too. Network failure requires a new login.
    let pending = session.take_oidc().await?.ok_or(Failure::Unauthorized)?;
    if !pending.accepts(query.state.as_deref().unwrap_or_default())
        || query.error.is_some()
        || query
            .iss
            .as_ref()
            .is_some_and(|issuer| issuer != &oidc.issuer)
    {
        return Err(Failure::Unauthorized);
    }
    let code = query
        .code
        .filter(|code| !code.is_empty() && code.len() <= 4096)
        .ok_or(Failure::Invalid)?;
    let subject = oidc.finish(code, pending).await?;
    let Some(principal) = oidc.subjects.get(&subject).copied() else {
        return Ok((
            axum::http::StatusCode::FORBIDDEN,
            crate::page(
                &state,
                "Access not assigned | Lab Operations",
                crate::views::unassigned_identity(&subject),
                false,
            )?,
        )
            .into_response());
    };
    session
        .replace_and_rotate(LoginSession {
            user: Some(principal),
            oidc: None,
        })
        .await?;
    Ok(Redirect::to(&format!("/board/projects/{}/items", principal.project)).into_response())
}

#[cfg(test)]
mod tests;
