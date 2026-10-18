//! Explicit, opt-in native string secret lookup through ordinary application state.
//!
//! ```
//! use std::{collections::BTreeMap, sync::Arc};
//! use yetlet_secrets::{SecretProvider, SecretString, SettingsSecrets};
//!
//! // Select only the already-loaded settings fields this consumer needs.
//! let values = BTreeMap::from([("mail-token".into(), SecretString::from("example"))]);
//! let provider: Arc<dyn SecretProvider> = Arc::new(SettingsSecrets::new(values));
//! let lookup = provider.get("mail-token"); // Await in application code.
//! ```

#[cfg(target_arch = "wasm32")]
compile_error!("yetlet-secrets is native application secret lookup, not browser state");

use std::{collections::BTreeMap, fmt, future::Future, pin::Pin, sync::RwLock};

pub use secrecy::{ExposeSecret, SecretString};

/// An object-safe asynchronous lookup that can cross native executor threads.
pub type SecretFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<SecretString>, SecretError>> + Send + 'a>>;

/// Purpose-neutral string secrets, supplied explicitly by the application.
///
/// `Ok(None)` means the name is missing; access or backend failures must return
/// an error. `Some` may contain an empty string: validation belongs to the app.
/// Names are exact, case-sensitive application-owned identifiers. Implementors
/// choose lookup timing; this contract promises no refresh, rotation or retry.
/// Keep backend diagnostics, names and secret values out of public errors/logs.
pub trait SecretProvider: Send + Sync {
    fn get<'a>(&'a self, name: &'a str) -> SecretFuture<'a>;
}

/// Bounded provider failures without backend diagnostics or secret identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretError {
    /// The provider cannot currently complete the lookup.
    Unavailable,
    /// The provider refused access; this is not a missing name.
    AccessDenied,
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "secret provider unavailable",
            Self::AccessDenied => "secret access denied",
        })
    }
}

impl std::error::Error for SecretError {}

/// Explicitly replaceable snapshot of selected, already-loaded settings.
///
/// This adapter reads no files or environment variables and changes no loader
/// precedence. Lookup returns an owned protected clone. The application can
/// reload its settings and call [`Self::replace`] on the same shared provider.
/// Already returned values and clients keep their old credentials until the
/// application explicitly reconfigures them.
pub struct SettingsSecrets {
    values: RwLock<BTreeMap<String, SecretString>>,
}

impl SettingsSecrets {
    /// Take ownership of only the fields the application chooses to expose.
    pub fn new(values: BTreeMap<String, SecretString>) -> Self {
        Self {
            values: RwLock::new(values),
        }
    }

    /// Replace the entire selected snapshot, including removing omitted names.
    ///
    /// Existing shared handles see this snapshot on subsequent lookups. Each
    /// lookup sees a complete old or new snapshot; separate lookups are not an
    /// atomic transaction. No file access, credential generation or external
    /// rotation occurs here. There is no source revision or freshness check.
    /// Applications offering reload must serialize loading, validation and
    /// replacement together, before reading any inputs, through one update entry.
    /// A poisoned lock returns [`SecretError::Unavailable`].
    pub fn replace(&self, values: BTreeMap<String, SecretString>) -> Result<(), SecretError> {
        let mut snapshot = self.values.write().map_err(|_| SecretError::Unavailable)?;
        *snapshot = values;
        Ok(())
    }
}

impl fmt::Debug for SettingsSecrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SettingsSecrets { .. }")
    }
}

impl SecretProvider for SettingsSecrets {
    fn get<'a>(&'a self, name: &'a str) -> SecretFuture<'a> {
        Box::pin(async move {
            let snapshot = self.values.read().map_err(|_| SecretError::Unavailable)?;
            Ok(snapshot.get(name).cloned())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn poisoned_snapshot_returns_only_safe_unavailable_errors() {
        let provider = SettingsSecrets::new(BTreeMap::new());
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _snapshot = provider.values.write().unwrap();
            panic!("poison snapshot for test");
        }));
        assert!(panic.is_err());
        assert_eq!(
            provider.get("private-name").await.unwrap_err(),
            SecretError::Unavailable
        );
        assert_eq!(
            provider.replace(BTreeMap::new()),
            Err(SecretError::Unavailable)
        );
        assert_eq!(format!("{provider:?}"), "SettingsSecrets { .. }");
    }
}
