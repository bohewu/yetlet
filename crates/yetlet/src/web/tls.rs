//! Optional certificate configuration and TLS transport for managed Web Hosts.

use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use axum::serve::Listener;
use futures_util::{StreamExt, future::BoxFuture, stream::FuturesUnordered};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    server::ParsedCertificate,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
    time::timeout,
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

use super::WebError;

/// Per-host HTTPS configuration. Each constructor takes a leaf-first PEM chain
/// and a matching unencrypted PEM private key. Clients verify trust and names.
pub struct WebTls(watch::Receiver<Arc<ServerConfig>>);

impl WebTls {
    /// Keep this certificate for the lifetime of the Host.
    pub fn fixed(chain_pem: &[u8], key_pem: &[u8]) -> Result<Self, WebError> {
        let (tls, _) = Self::reloadable(chain_pem, key_pem)?;
        Ok(tls)
    }

    /// Return the Host configuration and the handle that replaces its identity.
    pub fn reloadable(
        chain_pem: &[u8],
        key_pem: &[u8],
    ) -> Result<(Self, WebTlsReloader), WebError> {
        let (sender, receiver) = watch::channel(server_config(chain_pem, key_pem)?);
        Ok((Self(receiver), WebTlsReloader(sender)))
    }
}

/// Updates the associated Host without closing existing connections.
#[derive(Clone)]
pub struct WebTlsReloader(watch::Sender<Arc<ServerConfig>>);

impl WebTlsReloader {
    /// Atomically install a complete configuration for subsequently accepted TCP
    /// connections. A rejected certificate/key pair leaves the current one active.
    pub fn reload(&self, chain_pem: &[u8], key_pem: &[u8]) -> Result<(), WebError> {
        let config = server_config(chain_pem, key_pem)?;
        self.0.send(config).map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Web TLS configuration was dropped",
            )
        })?;
        Ok(())
    }
}

fn server_config(chain_pem: &[u8], key_pem: &[u8]) -> Result<Arc<ServerConfig>, WebError> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
    let chain = CertificateDer::pem_slice_iter(chain_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| invalid("invalid PEM certificate chain"))?;
    // with_single_cert checks the leaf and its key. Parse the remaining DER
    // certificates too so malformed intermediates cannot replace a working set.
    for certificate in chain.iter().skip(1) {
        ParsedCertificate::try_from(certificate)
            .map_err(|_| invalid("invalid intermediate certificate"))?;
    }
    let mut keys = PrivateKeyDer::pem_slice_iter(key_pem);
    let key = match (keys.next(), keys.next()) {
        (Some(Ok(key)), None) => key,
        _ => return Err(invalid("expected one unencrypted PEM private key").into()),
    };
    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
            .with_no_client_auth()
            .with_single_cert(chain, key)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    Ok(Arc::new(config))
}

// Handshakes are owned by the listener, so shutdown drops unfinished peers
// immediately while Axum drains already established HTTP responses.
const MAX_HANDSHAKES: usize = 64;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
type Handshake = BoxFuture<'static, Option<(TlsStream<TcpStream>, SocketAddr)>>;

pub(super) struct TlsListener {
    tcp: TcpListener,
    tls: WebTls,
    pending: FuturesUnordered<Handshake>,
}

impl TlsListener {
    pub(super) fn new(tcp: TcpListener, tls: WebTls) -> Self {
        Self {
            tcp,
            tls,
            pending: FuturesUnordered::new(),
        }
    }
}

impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            tokio::select! {
                completed = self.pending.next(), if !self.pending.is_empty() => {
                    if let Some(Some(connection)) = completed {
                        return connection;
                    }
                }
                (socket, addr) = Listener::accept(&mut self.tcp), if self.pending.len() < MAX_HANDSHAKES => {
                    let acceptor = TlsAcceptor::from(self.tls.0.borrow().clone());
                    let handshake = timeout(HANDSHAKE_TIMEOUT, acceptor.accept(socket));
                    self.pending.push(Box::pin(async move {
                        Some((handshake.await.ok()?.ok()?, addr))
                    }));
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}
