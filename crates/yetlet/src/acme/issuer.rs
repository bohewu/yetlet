use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    future::Future,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use instant_acme::{
    Account, AccountBuilder, AuthorizationStatus, ChallengeType, Identifier, NewAccount, NewOrder,
    OrderStatus, RetryPolicy,
};
use rustls::{
    ClientConfig, RootCertStore,
    client::{WebPkiServerVerifier, danger::ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime, pem::PemObject},
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{oneshot, watch},
    time::{Instant, sleep_until, timeout_at},
};

use super::{AcmeConfig, AcmeFailure, AcmeStatus, ChallengeSlot, invalid};
use crate::web::{WebError, WebTls, WebTlsReloader};

const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Deserialize, Serialize)]
struct Identity {
    chain: String,
    key: String,
}

#[derive(Deserialize, Serialize)]
struct Retry {
    at_ms: u64,
    delay_ms: u64,
}

impl Retry {
    fn time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.at_ms)
    }

    fn new(config: &AcmeConfig, previous: Option<&Self>) -> Result<Self, WebError> {
        let delay = previous.map_or(config.retry_initial, |prior| {
            Duration::from_millis(prior.delay_ms)
                .saturating_mul(2)
                .clamp(config.retry_initial, config.retry_max)
        });
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| invalid("ACME requires a valid system clock"))?;
        Ok(Self {
            at_ms: u64::try_from((wall + delay).as_nanos().div_ceil(1_000_000))
                .map_err(|_| invalid("ACME retry deadline overflow"))?,
            delay_ms: delay.as_millis() as u64,
        })
    }
}

#[derive(Deserialize, Serialize)]
struct Stored {
    directory: String,
    domain: String,
    account: Option<serde_json::Value>,
    identity: Option<Identity>,
    retry: Option<Retry>,
    last_failure: Option<AcmeFailure>,
}

struct Store {
    path: PathBuf,
    data: Stored,
    _lock: File,
}

impl Store {
    fn open(config: &AcmeConfig) -> Result<Self, WebError> {
        let mut builder = DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&config.store) {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error.into()),
        }
        let metadata = fs::symlink_metadata(&config.store)?;
        if !metadata.is_dir() {
            return Err(invalid("ACME state path must be a dedicated directory"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(invalid("ACME state directory must be private (mode 0700)"));
            }
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(config.store.join("owner.lock"))?;
        lock.try_lock().map_err(|error| {
            io::Error::other(format!("cannot acquire exclusive ACME state: {error}"))
        })?;
        let path = config.store.join("state.json");
        let data: Stored = match fs::read(&path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| invalid("invalid ACME state record"))?
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Stored {
                directory: config.directory.clone(),
                domain: config.domain.clone(),
                account: None,
                identity: None,
                retry: None,
                last_failure: None,
            },
            Err(error) => return Err(error.into()),
        };
        if data.domain != config.domain || data.directory != config.directory {
            return Err(invalid("ACME state belongs to another domain or directory"));
        }
        Ok(Self {
            path,
            data,
            _lock: lock,
        })
    }

    async fn save(&self) -> Result<(), WebError> {
        let payload = serde_json::to_vec(&self.data)
            .map_err(|_| io::Error::other("cannot write ACME state"))?;
        let path = self.path.clone();
        // The issuer retains this operation through shutdown, keeping the lock held.
        tokio::task::spawn_blocking(move || -> io::Result<()> {
            let directory = path.parent().expect("ACME state file parent");
            let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
            temporary.write_all(&payload)?;
            temporary.as_file().sync_all()?;
            temporary.persist(&path).map_err(|error| error.error)?;
            #[cfg(unix)]
            File::open(directory)?.sync_all()?;
            Ok(())
        })
        .await
        .map_err(io::Error::other)??;
        Ok(())
    }
}

fn roots(custom: Option<&Path>) -> Result<RootCertStore, WebError> {
    let mut roots = RootCertStore::empty();
    if let Some(path) = custom {
        for certificate in CertificateDer::pem_file_iter(path)
            .map_err(|_| invalid("cannot read ACME trust roots"))?
        {
            roots
                .add(certificate.map_err(|_| invalid("invalid ACME trust roots"))?)
                .map_err(|_| invalid("invalid ACME trust roots"))?;
        }
    } else {
        roots.add_parsable_certificates(rustls_native_certs::load_native_certs().certs);
    }
    if roots.is_empty() {
        return Err(invalid("no usable ACME trust roots"));
    }
    Ok(roots)
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn validate_identity(
    config: &AcmeConfig,
    verifier: &WebPkiServerVerifier,
    identity: &Identity,
) -> Result<(SystemTime, SystemTime), AcmeFailure> {
    let chain = CertificateDer::pem_slice_iter(identity.chain.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AcmeFailure::Certificate)?;
    let (leaf, intermediates) = chain.split_first().ok_or(AcmeFailure::Certificate)?;
    let name =
        ServerName::try_from(config.domain.as_str()).map_err(|_| AcmeFailure::Certificate)?;
    verifier
        .verify_server_cert(leaf, intermediates, &name, &[], UnixTime::now())
        .map_err(|_| AcmeFailure::Certificate)?;
    WebTls::fixed(identity.chain.as_bytes(), identity.key.as_bytes())
        .map_err(|_| AcmeFailure::Certificate)?;
    validity(leaf)
}

fn validity(leaf: &CertificateDer<'_>) -> Result<(SystemTime, SystemTime), AcmeFailure> {
    let (_, certificate) =
        x509_parser::parse_x509_certificate(leaf.as_ref()).map_err(|_| AcmeFailure::Certificate)?;
    let timestamp = |seconds| -> Result<SystemTime, AcmeFailure> {
        let seconds = u64::try_from(seconds).map_err(|_| AcmeFailure::Certificate)?;
        UNIX_EPOCH
            .checked_add(Duration::from_secs(seconds))
            .ok_or(AcmeFailure::Certificate)
    };
    let dates = certificate.validity();
    let begins = timestamp(dates.not_before.timestamp())?;
    let expires = timestamp(dates.not_after.timestamp())?;
    if expires <= begins {
        return Err(AcmeFailure::Certificate);
    }
    Ok((begins, expires))
}

fn renewal_at(config: &AcmeConfig, (begins, expires): (SystemTime, SystemTime)) -> SystemTime {
    let lifetime = expires
        .duration_since(begins)
        .expect("verified certificate lifetime");
    let fraction = if lifetime < Duration::from_secs(10 * 86400) {
        2
    } else {
        3
    };
    let lead = config
        .renew_before
        .filter(|lead| *lead < lifetime)
        .unwrap_or(lifetime / fraction);
    expires - lead
}

fn deadline(time: SystemTime) -> Instant {
    Instant::now() + time.duration_since(SystemTime::now()).unwrap_or_default()
}

async fn account(
    config: &AcmeConfig,
    transport: &ClientConfig,
    store: &Store,
) -> Result<(Account, Option<serde_json::Value>), Attempt> {
    let connector = HttpsConnectorBuilder::new()
        .with_tls_config(transport.clone())
        .https_only()
        .enable_http1()
        .build();
    let client = Client::builder(TokioExecutor::new()).build(connector);
    let builder: AccountBuilder = Account::builder_with_http(Box::new(client));
    if let Some(credentials) = store.data.account.clone() {
        let credentials = serde_json::from_value(credentials)
            .map_err(|_| invalid("invalid persisted ACME account"))?;
        return builder
            .from_credentials(credentials)
            .await
            .map(|account| (account, None))
            .map_err(|_| Attempt::Failed(AcmeFailure::Account));
    }
    let (account, credentials) = builder
        .create(
            &NewAccount {
                contact: &[],
                terms_of_service_agreed: config.terms_agreed,
                only_return_existing: false,
            },
            config.directory.clone(),
            config.eab.as_ref(),
        )
        .await
        .map_err(|_| Attempt::Failed(AcmeFailure::Account))?;
    let credentials =
        serde_json::to_value(credentials).map_err(|_| invalid("cannot encode the ACME account"))?;
    Ok((account, Some(credentials)))
}

async fn issue(
    config: &AcmeConfig,
    account: &Account,
    challenge: &ChallengeSlot,
) -> Result<Identity, AcmeFailure> {
    let identifiers = [Identifier::Dns(config.domain.clone())];
    let mut order = account
        .new_order(&NewOrder::new(&identifiers))
        .await
        .map_err(|_| AcmeFailure::Order)?;
    let mut authorizations = order.authorizations();
    while let Some(authorization) = authorizations.next().await {
        let mut authorization = authorization.map_err(|_| AcmeFailure::Challenge)?;
        match authorization.status {
            AuthorizationStatus::Valid => continue,
            AuthorizationStatus::Pending => (),
            _ => return Err(AcmeFailure::Challenge),
        }
        let mut proof = authorization
            .challenge(ChallengeType::Http01)
            .ok_or(AcmeFailure::Challenge)?;
        challenge.send_replace(Some((
            proof.token.clone(),
            proof.key_authorization().as_str().to_owned(),
        )));
        proof
            .set_ready()
            .await
            .map_err(|_| AcmeFailure::Challenge)?;
    }
    if order
        .poll_ready(&RetryPolicy::default())
        .await
        .map_err(|_| AcmeFailure::Challenge)?
        != OrderStatus::Ready
    {
        return Err(AcmeFailure::Challenge);
    }
    let key = order
        .finalize()
        .await
        .map_err(|_| AcmeFailure::Certificate)?;
    let chain = order
        .poll_certificate(&RetryPolicy::default())
        .await
        .map_err(|_| AcmeFailure::Certificate)?;
    Ok(Identity { chain, key })
}

enum Attempt {
    Failed(AcmeFailure),
    Store(WebError),
    Stopped,
}

impl From<WebError> for Attempt {
    fn from(error: WebError) -> Self {
        Self::Store(error)
    }
}

async fn network<T>(
    stop: &mut watch::Receiver<()>,
    limit: Instant,
    work: impl Future<Output = Result<T, Attempt>>,
) -> Result<T, Attempt> {
    tokio::select! { biased;
        _ = stop.changed() => Err(Attempt::Stopped),
        result = timeout_at(limit, work) => result.unwrap_or(Err(Attempt::Failed(AcmeFailure::Timeout))),
    }
}

enum Listener {
    Initial(oneshot::Sender<WebTls>),
    Ready(WebTlsReloader),
}

impl Listener {
    fn install(self, identity: &Identity) -> Result<Self, WebError> {
        let (chain, key) = (identity.chain.as_bytes(), identity.key.as_bytes());
        match self {
            Self::Initial(sender) => {
                let (tls, reload) = WebTls::reloadable(chain, key)?;
                sender
                    .send(tls)
                    .map_err(|_| io::Error::other("ACME HTTPS listener stopped"))?;
                Ok(Self::Ready(reload))
            }
            Self::Ready(reload) => {
                reload.reload(chain, key)?;
                Ok(Self::Ready(reload))
            }
        }
    }
}

pub(super) async fn run(
    config: AcmeConfig,
    challenge: ChallengeSlot,
    status: watch::Sender<AcmeStatus>,
    initial: oneshot::Sender<WebTls>,
    mut stop: watch::Receiver<()>,
) -> Result<(), WebError> {
    let (config, mut store, transport, verifier) =
        tokio::task::spawn_blocking(move || -> Result<_, WebError> {
            let store = Store::open(&config)?;
            let transport = ClientConfig::builder_with_provider(provider())
                .with_safe_default_protocol_versions()
                .map_err(io::Error::other)?
                .with_root_certificates(roots(config.directory_root.as_deref())?)
                .with_no_client_auth();
            let verifier = WebPkiServerVerifier::builder_with_provider(
                Arc::new(roots(config.issuance_root.as_deref())?),
                provider(),
            )
            .build()
            .map_err(io::Error::other)?;
            Ok((config, store, transport, verifier))
        })
        .await
        .map_err(io::Error::other)??;
    if stop.has_changed().is_err() {
        return Ok(());
    }
    let mut listener = Listener::Initial(initial);
    let mut due = SystemTime::now();
    let mut expires_at = None;
    if let Some(identity) = &store.data.identity {
        // Keep public expiry available even when the identity no longer verifies.
        let expiry = CertificateDer::from_pem_slice(identity.chain.as_bytes())
            .ok()
            .and_then(|leaf| validity(&leaf).ok())
            .map(|(_, expires)| expires);
        expires_at = expiry;
        if let Ok(dates) = validate_identity(&config, &verifier, identity) {
            listener = listener.install(identity)?;
            due = renewal_at(&config, dates);
        }
    }
    let retry = store.data.retry.as_ref().map(Retry::time);
    if let Some(retry) = retry {
        due = retry;
    }
    status.send_modify(|s| {
        s.certificate_expires_at = expires_at;
        s.retry_at = retry;
        s.last_failure = store.data.last_failure;
    });
    let mut registered = None;
    loop {
        tokio::select! { biased;
            _ = stop.changed() => return Ok(()),
            _ = sleep_until(deadline(due)) => {}
        }
        let limit = Instant::now() + ATTEMPT_TIMEOUT;
        let attempt = async {
            if registered.is_none() {
                let (opened, credentials) =
                    network(&mut stop, limit, account(&config, &transport, &store)).await?;
                if let Some(credentials) = credentials {
                    store.data.account = Some(credentials);
                    store.save().await?;
                }
                registered = Some(opened);
            }
            let identity = network(&mut stop, limit, async {
                let account = registered.as_ref().expect("ACME account");
                issue(&config, account, &challenge)
                    .await
                    .map_err(Attempt::Failed)
            })
            .await?;
            let dates =
                validate_identity(&config, &verifier, &identity).map_err(Attempt::Failed)?;
            Ok((identity, dates))
        }
        .await;
        challenge.send_replace(None);
        match attempt {
            Ok((identity, dates)) => {
                store.data.retry = None;
                store.data.identity = Some(identity);
                store.save().await?;
                if stop.has_changed().is_err() {
                    return Ok(());
                }
                listener =
                    listener.install(store.data.identity.as_ref().expect("new ACME identity"))?;
                due = renewal_at(&config, dates);
                status.send_modify(|s| {
                    s.retry_at = None;
                    s.certificate_expires_at = Some(dates.1);
                });
            }
            Err(Attempt::Failed(failure)) => {
                let retry = Retry::new(&config, store.data.retry.as_ref())?;
                due = retry.time();
                store.data.retry = Some(retry);
                store.data.last_failure = Some(failure);
                store.save().await?;
                status.send_modify(|s| {
                    s.retry_at = Some(due);
                    s.last_failure = Some(failure);
                });
            }
            Err(Attempt::Store(error)) => return Err(error),
            Err(Attempt::Stopped) => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(store: PathBuf) -> AcmeConfig {
        AcmeConfig::new("site.example", "https://ca.example/directory", store).unwrap()
    }

    #[tokio::test]
    async fn cold_retry_survives_without_an_identity_or_account() {
        let parent = tempfile::tempdir().unwrap();
        let config = config(parent.path().join("state"));
        let mut store = Store::open(&config).unwrap();
        assert!(Store::open(&config).is_err());
        let retry = Retry::new(&config, None).unwrap();
        let expected = retry.time();
        store.data.retry = Some(retry);
        store.data.last_failure = Some(AcmeFailure::Account);
        store.save().await.unwrap();
        drop(store);
        let restored = Store::open(&config).unwrap();
        assert!(restored.data.identity.is_none());
        assert!(restored.data.account.is_none());
        assert_eq!(restored.data.retry.as_ref().unwrap().time(), expected);
        assert_eq!(restored.data.last_failure, Some(AcmeFailure::Account));
        assert_eq!(
            Retry::new(&config, restored.data.retry.as_ref())
                .unwrap()
                .delay_ms,
            120_000
        );
    }

    #[tokio::test]
    async fn stored_state_is_bound_to_its_acme_directory_and_domain() {
        let parent = tempfile::tempdir().unwrap();
        let path = parent.path().join("state");
        Store::open(&config(path.clone()))
            .unwrap()
            .save()
            .await
            .unwrap();
        let other =
            AcmeConfig::new("other.example", "https://ca.example/directory", &path).unwrap();
        assert!(Store::open(&other).is_err());
        let other =
            AcmeConfig::new("site.example", "https://other.example/directory", &path).unwrap();
        assert!(Store::open(&other).is_err());
    }
}
