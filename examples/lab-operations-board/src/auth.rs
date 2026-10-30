//! Admin-owned local-account binding and uncached single-host policy authority.
use crate::{
    Failure,
    domain::{LoginSession, Principal},
    sessions::remote_error,
};
use axum::{
    http::request::Parts,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use std::{
    path::{Component, PathBuf},
    time::Duration,
};
use tokio::io::AsyncReadExt;
use yetlet::auth::{Authentication, ResolveFuture};
use yetlet::session_redis::RedisSession;

const ISSUER: &str = "urn:yetlet:admin:local-accounts";
const SCHEME: &str = "admin-session";
const MAX_POLICY_BYTES: u64 = 4096;

#[derive(Clone)]
pub(crate) struct PolicyFile(PathBuf);
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Policy {
    accounts: Accounts,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Accounts {
    alice: Option<Account>,
    bob: Option<Account>,
    viewer: Option<Account>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Account {
    enabled: bool,
    read: bool,
    write: bool,
}
impl PolicyFile {
    pub(crate) fn new(path: PathBuf) -> Result<Self, Failure> {
        if !cfg!(target_os = "linux")
            || !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(Failure::Configuration);
        }
        Ok(Self(path))
    }
    pub(crate) async fn read(&self) -> Result<Policy, Failure> {
        let read = async {
            let mut options = tokio::fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
            let file = options
                .open(&self.0)
                .await
                .map_err(|_| Failure::Unavailable)?;
            let metadata = file.metadata().await.map_err(|_| Failure::Unavailable)?;
            if !metadata.is_file() || metadata.len() > MAX_POLICY_BYTES {
                return Err(Failure::Unavailable);
            }
            let mut bytes = Vec::with_capacity(MAX_POLICY_BYTES as usize + 1);
            file.take(MAX_POLICY_BYTES + 1)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| Failure::Unavailable)?;
            if bytes.len() > MAX_POLICY_BYTES as usize {
                return Err(Failure::Unavailable);
            }
            serde_json::from_slice(&bytes).map_err(|_| Failure::Unavailable)
        };
        tokio::time::timeout(Duration::from_millis(250), read)
            .await
            .map_err(|_| Failure::Unavailable)?
    }
}
impl Policy {
    fn account(&self, user: Principal) -> Result<&Account, Failure> {
        let account = match user.name {
            "alice" => &self.accounts.alice,
            "bob" => &self.accounts.bob,
            "viewer" => &self.accounts.viewer,
            _ => return Err(Failure::Forbidden),
        };
        account
            .as_ref()
            .filter(|account| account.enabled)
            .ok_or(Failure::Forbidden)
    }
    pub(crate) fn authorize(
        &self,
        mut user: Principal,
        project: Option<i64>,
        write: bool,
    ) -> Result<Principal, Failure> {
        let account = self.account(user)?;
        user.write &= account.write;
        if !account.read
            || project.is_some_and(|project| project != user.project)
            || (write && !user.write)
        {
            return Err(Failure::Forbidden);
        }
        Ok(user)
    }
}
fn tenant(user: Principal) -> &'static str {
    if user.project == 1 {
        "project-1"
    } else {
        "project-2"
    }
}
pub(crate) fn check_identity(
    identity: &yetlet::auth::Principal,
    user: Principal,
) -> Result<(), Failure> {
    if identity.issuer() != ISSUER
        || identity.authentication_name() != SCHEME
        || identity.subject() != user.name
        || identity.tenant() != Some(tenant(user))
    {
        return Err(Failure::Forbidden);
    }
    Ok(())
}
pub(crate) fn protected_failure(error: Failure) -> Failure {
    if error == Failure::Unauthorized {
        Failure::LoginRequired
    } else {
        error
    }
}
fn resolve<'a>(parts: &'a mut Parts, policy: &'a PolicyFile) -> ResolveFuture<'a> {
    Box::pin(async move {
        let session = parts
            .extensions
            .get::<RedisSession<LoginSession>>()
            .cloned()
            .ok_or_else(|| Failure::Configuration.into_response())?;
        let reference = session
            .read()
            .await
            .map_err(|error| protected_failure(remote_error(error)).into_response())?;
        // Even explicit anonymous resolution must observe unavailable authority.
        let rules = policy.read().await.map_err(IntoResponse::into_response)?;
        let Some(user) = reference.user else {
            return Ok(None);
        };
        rules.account(user).map_err(IntoResponse::into_response)?;
        // The stored local binding came from verified demo or OIDC login. This
        // namespace is not an assertion about an external IdP issuer/subject.
        yetlet::auth::Principal::from_verified(ISSUER, user.name, SCHEME, Some(tenant(user)))
            .map(Some)
            .map_err(|_| Failure::Configuration.into_response())
    })
}
pub(crate) fn authentication(policy: PolicyFile) -> Authentication<PolicyFile> {
    Authentication::new(SCHEME, policy, resolve, || {
        Redirect::to("/board/login").into_response()
    })
    .expect("static authentication scheme")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::{FromRequestParts, Request},
    };
    const ALLOW: &str = r#"{"accounts":{"alice":{"enabled":true,"read":true,"write":true},"viewer":{"enabled":true,"read":true,"write":true}}}"#;
    #[test]
    fn policy_only_restricts_compiled_authority_and_identity_is_bound() {
        let rules: Policy = serde_json::from_str(ALLOW).unwrap();
        let alice = crate::domain::principal("alice").unwrap();
        let viewer = crate::domain::principal("viewer").unwrap();
        assert!(rules.authorize(alice, Some(1), true).is_ok());
        assert_eq!(
            rules.authorize(alice, Some(2), false),
            Err(Failure::Forbidden)
        );
        assert_eq!(
            rules.authorize(viewer, Some(1), true),
            Err(Failure::Forbidden)
        );
        assert!(!rules.authorize(viewer, None, false).unwrap().write);
        assert_eq!(
            rules.authorize(crate::domain::principal("bob").unwrap(), None, false),
            Err(Failure::Forbidden)
        );
        let identity =
            yetlet::auth::Principal::from_verified(ISSUER, "alice", SCHEME, Some("project-1"))
                .unwrap();
        assert!(check_identity(&identity, alice).is_ok());
        assert_eq!(check_identity(&identity, viewer), Err(Failure::Forbidden));
        for text in [
            r#"{}"#,
            r#"{"accounts":{"admin":null}}"#,
            r#"{"accounts":{"alice":{"enabled":true,"read":true}}}"#,
            r#"{"accounts":{"alice":{"enabled":true,"read":true,"write":true,"project":2}}}"#,
        ] {
            assert!(serde_json::from_str::<Policy>(text).is_err());
        }
    }
    #[tokio::test]
    async fn policy_is_fresh_bounded_and_unavailable_is_not_anonymous() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("policy.json");
        let policy = PolicyFile::new(path.clone()).unwrap();
        assert!(matches!(policy.read().await, Err(Failure::Unavailable)));
        std::fs::write(&path, ALLOW).unwrap();
        let alice = crate::domain::principal("alice").unwrap();
        assert!(
            policy
                .read()
                .await
                .unwrap()
                .authorize(alice, None, true)
                .is_ok()
        );
        let pending = dir.path().join("next");
        std::fs::write(
            &pending,
            r#"{"accounts":{"alice":{"enabled":true,"read":true,"write":false}}}"#,
        )
        .unwrap();
        std::fs::rename(&pending, &path).unwrap();
        assert_eq!(
            policy.read().await.unwrap().authorize(alice, None, true),
            Err(Failure::Forbidden)
        );
        for invalid in [
            "malformed".to_owned(),
            " ".repeat(MAX_POLICY_BYTES as usize + 1),
        ] {
            std::fs::write(&path, invalid).unwrap();
            assert!(matches!(policy.read().await, Err(Failure::Unavailable)));
        }
        #[cfg(unix)]
        {
            std::fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(dir.path(), &path).unwrap();
            assert!(matches!(policy.read().await, Err(Failure::Unavailable)));
        }
        let (mut parts, _) = Request::new(Body::empty()).into_parts();
        assert_eq!(
            resolve(&mut parts, &policy).await.err().unwrap().status(),
            500
        );
        assert_eq!(
            yetlet::auth::Authenticated::from_request_parts(&mut parts, &())
                .await
                .err()
                .unwrap()
                .status(),
            500
        );
        assert_eq!(
            protected_failure(Failure::Unauthorized),
            Failure::LoginRequired
        );
        assert_eq!(
            protected_failure(Failure::Unavailable),
            Failure::Unavailable
        );
    }
}
