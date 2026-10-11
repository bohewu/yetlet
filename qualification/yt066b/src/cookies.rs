//! Private SSR cookie policy; not integrated provider middleware.
//!
//! A fixed name bounds cookie storage. A late prepared Set may replace a newer
//! cookie and require login again; Redis must still reject the retired bearer.
//! No JavaScript, old-ID forwarding, automatic creation or alternate-cookie fallback.
use crate::{Error, Snapshot, Store, Ticket, valid_id};

pub struct CookieScope {
    name: String,
    secure: bool,
}
pub struct Presented {
    identity: Option<String>,
}
pub struct Loaded {
    pub identity: String,
    pub snapshot: Snapshot,
}
#[derive(Debug, PartialEq, Eq)]
pub enum CookieError {
    Invalid,
    Missing,
}
pub enum LoadError {
    Cookie(CookieError),
    Store(Error),
}
impl std::fmt::Debug for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cookie(e) => f.debug_tuple("Cookie").field(e).finish(),
            Self::Store(e) => f.debug_tuple("Store").field(e).finish(),
        }
    }
}

impl CookieScope {
    pub fn new(scope: &str) -> Result<Self, CookieError> {
        if scope.is_empty()
            || scope.len() > 20
            || !scope
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(CookieError::Invalid);
        }
        Ok(Self {
            name: format!("__Host-{scope}"),
            secure: true,
        })
    }
    /// Explicit fixture-only HTTP exception, with a distinct non-__Host name.
    pub fn insecure_for_local_http(mut self) -> Self {
        self.name = self.name.trim_start_matches("__Host-").to_owned();
        self.secure = false;
        self
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn parse<'a>(
        &self,
        headers: impl IntoIterator<Item = &'a str>,
    ) -> Result<Presented, CookieError> {
        let mut identity = None;
        for header in headers {
            for pair in header.split(';') {
                let pair = pair.trim();
                let Some((name, value)) = pair.split_once('=') else {
                    if pair == self.name {
                        return Err(CookieError::Invalid);
                    }
                    continue;
                };
                // Never choose another scope or a legacy per-issuance name.
                if name != self.name {
                    continue;
                }
                if identity.is_some() || !valid_id(value) {
                    return Err(CookieError::Invalid);
                }
                identity = Some(value.to_owned());
            }
        }
        Ok(Presented { identity })
    }
    /// Only a positively resolved create/rotate/replace may set the cookie.
    /// Acknowledged historical success does not bypass current lifecycle checks.
    /// Call for the operation's response, never to refresh an ordinary read.
    pub async fn finalize(
        &self,
        store: &Store,
        operation: &Ticket,
    ) -> Result<Option<String>, Error> {
        let identity = match operation.args[0].as_str() {
            "create" => operation.identity(),
            "rotate" | "replace" => operation.replacement().ok_or(Error::Protocol)?,
            // In particular, successful logout never emits a late Clear.
            _ => return Ok(None),
        };
        let result = store.resolve(operation).await?;
        match result.status.as_str() {
            "ok" => (),
            "unresolved" => return Err(Error::OutcomeUnknown(operation.clone())),
            _ => return Ok(None),
        }
        match store.read(identity).await {
            Ok(_) => {
                // Browser lifetime is only a hint; the Redis absolute expiry is
                // authoritative and is never extended by finalization or loading.
                let ttl_ms = operation.args[6]
                    .parse::<u64>()
                    .map_err(|_| Error::Protocol)?;
                Ok(Some(format!(
                    "{}={identity}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax{}",
                    self.name,
                    ttl_ms.div_ceil(1000),
                    if self.secure { "; Secure" } else { "" }
                )))
            }
            Err(Error::Lifecycle(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }
    pub async fn load(&self, store: &Store, presented: &Presented) -> Result<Loaded, LoadError> {
        let identity = presented
            .identity
            .as_ref()
            .ok_or(LoadError::Cookie(CookieError::Missing))?;
        let snapshot = store.read(identity).await.map_err(LoadError::Store)?;
        Ok(Loaded {
            identity: identity.clone(),
            snapshot,
        })
    }
}
