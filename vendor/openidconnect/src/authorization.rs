use crate::core::CoreResponseType;
use crate::helpers::join_vec;
use crate::{
    AdditionalClaims, AuthDisplay, AuthPrompt, AuthenticationContextClass, CsrfToken, GenderClaim,
    IdToken, JweContentEncryptionAlgorithm, JwsSigningAlgorithm, LanguageTag, LoginHint, Nonce,
    PkceCodeChallenge, RedirectUrl, ResponseType, Scope,
};

use url::Url;

use std::borrow::Cow;
use std::time::Duration;

/// Authentication flow, which determines how the Authorization Server returns the OpenID Connect
/// ID token and OAuth2 access token to the Relying Party.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthenticationFlow<RT: ResponseType> {
    /// Authorization Code Flow.
    ///
    /// The authorization server will return an OAuth2 authorization code. Clients must subsequently
    /// call `Client::exchange_code()` with the authorization code in order to retrieve an
    /// OpenID Connect ID token and OAuth2 access token.
    AuthorizationCode,
    /// Implicit Flow.
    ///
    /// Boolean value indicates whether an OAuth2 access token should also be returned. If `true`,
    /// the Authorization Server will return both an OAuth2 access token and OpenID Connect ID
    /// token. If `false`, it will return only an OpenID Connect ID token.
    Implicit(bool),
    /// Hybrid Flow.
    ///
    /// A hybrid flow according to [OAuth 2.0 Multiple Response Type Encoding Practices](
    ///     https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html). The enum value
    /// contains the desired `response_type`s. See
    /// [Section 3](https://openid.net/specs/openid-connect-core-1_0.html#Authentication) for
    /// details.
    Hybrid(Vec<RT>),
}

/// A request to the authorization endpoint.
pub struct AuthorizationRequest<'a, AD, P, RT>
where
    AD: AuthDisplay,
    P: AuthPrompt,
    RT: ResponseType,
{
    pub(crate) inner: oauth2::AuthorizationRequest<'a>,
    pub(crate) acr_values: Vec<AuthenticationContextClass>,
    pub(crate) authentication_flow: AuthenticationFlow<RT>,
    pub(crate) claims_locales: Vec<LanguageTag>,
    pub(crate) display: Option<AD>,
    pub(crate) id_token_hint: Option<String>,
    pub(crate) login_hint: Option<LoginHint>,
    pub(crate) max_age: Option<Duration>,
    pub(crate) nonce: Nonce,
    pub(crate) prompts: Vec<P>,
    pub(crate) ui_locales: Vec<LanguageTag>,
}
impl<'a, AD, P, RT> AuthorizationRequest<'a, AD, P, RT>
where
    AD: AuthDisplay,
    P: AuthPrompt,
    RT: ResponseType,
{
    /// Appends a new scope to the authorization URL.
    pub fn add_scope(mut self, scope: Scope) -> Self {
        self.inner = self.inner.add_scope(scope);
        self
    }

    /// Appends a collection of scopes to the authorization URL.
    pub fn add_scopes<I>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = Scope>,
    {
        self.inner = self.inner.add_scopes(scopes);
        self
    }

    /// Appends an extra param to the authorization URL.
    ///
    /// This method allows extensions to be used without direct support from
    /// this crate. If `name` conflicts with a parameter managed by this crate, the
    /// behavior is undefined. In particular, do not set parameters defined by
    /// [RFC 6749](https://tools.ietf.org/html/rfc6749) or
    /// [RFC 7636](https://tools.ietf.org/html/rfc7636).
    ///
    /// # Security Warning
    ///
    /// Callers should follow the security recommendations for any OAuth2 extensions used with
    /// this function, which are beyond the scope of
    /// [RFC 6749](https://tools.ietf.org/html/rfc6749).
    pub fn add_extra_param<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.inner = self.inner.add_extra_param(name, value);
        self
    }

    /// Enables the use of [Proof Key for Code Exchange](https://tools.ietf.org/html/rfc7636)
    /// (PKCE).
    ///
    /// PKCE is *highly recommended* for all public clients (i.e., those for which there
    /// is no client secret or for which the client secret is distributed with the client,
    /// such as in a native, mobile app, or browser app).
    pub fn set_pkce_challenge(mut self, pkce_code_challenge: PkceCodeChallenge) -> Self {
        self.inner = self.inner.set_pkce_challenge(pkce_code_challenge);
        self
    }

    /// Requests Authentication Context Class Reference values.
    ///
    /// ACR values should be added in order of preference. The Authentication Context Class
    /// satisfied by the authentication performed is accessible from the ID token via the
    /// [`IdTokenClaims::auth_context_ref()`](crate::IdTokenClaims::auth_context_ref) method.
    pub fn add_auth_context_value(mut self, acr_value: AuthenticationContextClass) -> Self {
        self.acr_values.push(acr_value);
        self
    }

    /// Requests the preferred languages for claims returned by the OpenID Connect Provider.
    ///
    /// Languages should be added in order of preference.
    pub fn add_claims_locale(mut self, claims_locale: LanguageTag) -> Self {
        self.claims_locales.push(claims_locale);
        self
    }

    // TODO: support 'claims' parameter
    // https://openid.net/specs/openid-connect-core-1_0.html#ClaimsParameter

    /// Specifies how the OpenID Connect Provider displays the authentication and consent user
    /// interfaces to the end user.
    pub fn set_display(mut self, display: AD) -> Self {
        self.display = Some(display);
        self
    }

    /// Provides an ID token previously issued by this OpenID Connect Provider as a hint about
    /// the user's identity.
    ///
    /// This field should be set whenever
    /// [`CoreAuthPrompt::None`](crate::core::CoreAuthPrompt::None) is used (see
    /// [`AuthorizationRequest::add_prompt`]), it but may be provided for any authorization
    /// request.
    pub fn set_id_token_hint<AC, GC, JE, JS>(
        mut self,
        id_token_hint: &'a IdToken<AC, GC, JE, JS>,
    ) -> Self
    where
        AC: AdditionalClaims,
        GC: GenderClaim,
        JE: JweContentEncryptionAlgorithm<KeyType = JS::KeyType>,
        JS: JwsSigningAlgorithm,
    {
        self.id_token_hint = Some(id_token_hint.to_string());
        self
    }

    /// Provides the OpenID Connect Provider with a hint about the user's identity.
    ///
    /// The nature of this hint is specific to each provider.
    pub fn set_login_hint(mut self, login_hint: LoginHint) -> Self {
        self.login_hint = Some(login_hint);
        self
    }

    /// Sets a maximum amount of time since the user has last authenticated with the OpenID
    /// Connect Provider.
    ///
    /// If more time has elapsed, the provider forces the user to re-authenticate.
    pub fn set_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Specifies what level of authentication and consent prompts the OpenID Connect Provider
    /// should present to the user.
    pub fn add_prompt(mut self, prompt: P) -> Self {
        self.prompts.push(prompt);
        self
    }

    /// Requests the preferred languages for the user interface presented by the OpenID Connect
    /// Provider.
    ///
    /// Languages should be added in order of preference.
    pub fn add_ui_locale(mut self, ui_locale: LanguageTag) -> Self {
        self.ui_locales.push(ui_locale);
        self
    }

    /// Overrides the `redirect_url` to the one specified.
    pub fn set_redirect_uri(mut self, redirect_url: Cow<'a, RedirectUrl>) -> Self {
        self.inner = self.inner.set_redirect_uri(redirect_url);
        self
    }

    /// Returns the full authorization URL and CSRF state for this authorization
    /// request.
    pub fn url(self) -> (Url, CsrfToken, Nonce) {
        let response_type = match self.authentication_flow {
            AuthenticationFlow::AuthorizationCode => CoreResponseType::Code.to_oauth2(),
            AuthenticationFlow::Implicit(include_token) => {
                if include_token {
                    oauth2::ResponseType::new(
                        [CoreResponseType::IdToken, CoreResponseType::Token]
                            .iter()
                            .map(|response_type| response_type.as_ref())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                } else {
                    CoreResponseType::IdToken.to_oauth2()
                }
            }
            AuthenticationFlow::Hybrid(ref response_types) => oauth2::ResponseType::new(
                response_types
                    .iter()
                    .map(|response_type| response_type.as_ref())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        };
        let (mut inner, nonce) = (
            self.inner
                .set_response_type(&response_type)
                .add_extra_param("nonce", self.nonce.secret().clone()),
            self.nonce,
        );
        if !self.acr_values.is_empty() {
            inner = inner.add_extra_param("acr_values", join_vec(&self.acr_values));
        }
        if !self.claims_locales.is_empty() {
            inner = inner.add_extra_param("claims_locales", join_vec(&self.claims_locales));
        }
        if let Some(ref display) = self.display {
            inner = inner.add_extra_param("display", display.as_ref());
        }
        if let Some(ref id_token_hint) = self.id_token_hint {
            inner = inner.add_extra_param("id_token_hint", id_token_hint);
        }
        if let Some(ref login_hint) = self.login_hint {
            inner = inner.add_extra_param("login_hint", login_hint.secret());
        }
        if let Some(max_age) = self.max_age {
            inner = inner.add_extra_param("max_age", max_age.as_secs().to_string());
        }
        if !self.prompts.is_empty() {
            inner = inner.add_extra_param("prompt", join_vec(&self.prompts));
        }
        if !self.ui_locales.is_empty() {
            inner = inner.add_extra_param("ui_locales", join_vec(&self.ui_locales));
        }

        let (url, state) = inner.url();
        (url, state, nonce)
    }
}
