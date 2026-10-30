//! Explicit startup bindings into app-owned protected fields.
use crate::config::{Error, Loader, MAX_FILE_BYTES};
use crate::secrets::{ExposeSecret, SecretError, SecretProvider, SecretString};
use serde::de::DeserializeOwned;
use std::{ffi::OsString, fmt};

/// An app-owned name and a typed selector for one protected settings field.
///
/// Bindings are code, not configuration syntax. Names are passed exactly to the
/// injected provider. Neither Debug nor loader errors include names or values.
/// The selector must only select a field in the supplied fresh settings.
///
/// Ordinary strings cannot accidentally receive resolved secrets:
/// ```compile_fail
/// use yetlet::config::SecretBinding;
/// struct Settings { token: String }
/// let binding = SecretBinding::new("service-token", |s: &mut Settings| &mut s.token);
/// ```
pub struct SecretBinding<'a, T> {
    name: &'a str,
    target: fn(&mut T) -> &mut SecretString,
}

impl<'a, T> SecretBinding<'a, T> {
    pub fn new(name: &'a str, target: fn(&mut T) -> &mut SecretString) -> Self {
        Self { name, target }
    }
}

impl<T> fmt::Debug for SecretBinding<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretBinding { .. }")
    }
}

impl Loader {
    /// Load the usual file/environment snapshot, then resolve only the explicit
    /// protected-field bindings. Calling ordinary `load` still makes no lookups.
    /// The provider is the sole source for bound fields, after ordinary source
    /// precedence. Lookup failure never falls back to a literal/default value.
    pub async fn load_with_secrets<T: DeserializeOwned>(
        &self,
        provider: &dyn SecretProvider,
        bindings: &[SecretBinding<'_, T>],
    ) -> Result<T, Error> {
        self.load_with_env_and_secrets(std::env::vars_os(), provider, bindings)
            .await
    }

    /// Explicit-input form of `load_with_secrets`, without process mutation.
    ///
    /// Values move directly into SecretString fields without a TOML/plaintext
    /// intermediate. Each returned secret is limited to MAX_FILE_BYTES. Empty
    /// values are preserved; required/nonempty and business validation belongs
    /// to the app and must run before listener/client construction. A failed
    /// load returns no partially resolved settings. Separate provider lookups
    /// are not an atomic vault snapshot. No refresh/rotation is implied.
    pub async fn load_with_env_and_secrets<T: DeserializeOwned>(
        &self,
        environment: impl IntoIterator<Item = (OsString, OsString)>,
        provider: &dyn SecretProvider,
        bindings: &[SecretBinding<'_, T>],
    ) -> Result<T, Error> {
        let mut settings = self.load_with_env(environment)?;
        for binding in bindings {
            let value = provider
                .get(binding.name)
                .await
                .map_err(|error| match error {
                    SecretError::Unavailable => Error::SecretUnavailable,
                    SecretError::AccessDenied => Error::SecretAccessDenied,
                })?
                .ok_or(Error::SecretMissing)?;
            if value.expose_secret().len() > MAX_FILE_BYTES {
                return Err(Error::Input);
            }
            *(binding.target)(&mut settings) = value;
        }
        Ok(settings)
    }
}
