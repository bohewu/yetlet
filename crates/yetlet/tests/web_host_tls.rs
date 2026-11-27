#![cfg(not(target_arch = "wasm32"))]

use std::{
    convert::Infallible,
    future::poll_fn,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Router,
    http::{Method, StatusCode},
    response::{Sse, sse::Event},
    routing::get,
};
use futures::{StreamExt, channel::mpsc};
use rcgen::{BasicConstraints, Certificate, CertificateParams, CertifiedIssuer, IsCa, KeyPair};
use rustls::{ClientConfig, HandshakeKind, RootCertStore, pki_types::ServerName};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::timeout,
};
use tokio_rustls::{TlsConnector, client::TlsStream};
use yetlet::{
    HostGroup,
    composition::{HostBuildError, StartStatus},
    web::{App, HttpProbe, WebDriver, WebHandle, WebHost, WebTls},
};

const LIMIT: Duration = Duration::from_secs(5);
const NAME: &str = "localhost";

fn authority() -> CertifiedIssuer<'static, KeyPair> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    CertifiedIssuer::self_signed(params, KeyPair::generate().unwrap()).unwrap()
}

fn leaf(ca: &CertifiedIssuer<'_, KeyPair>) -> (Certificate, String) {
    let key = KeyPair::generate().unwrap();
    let cert = CertificateParams::new(vec![NAME.to_owned()])
        .unwrap()
        .signed_by(&key, ca)
        .unwrap();
    (cert, key.serialize_pem())
}

fn client(ca: &CertifiedIssuer<'_, KeyPair>) -> TlsConnector {
    let mut roots = RootCertStore::empty();
    roots.add(ca.der().clone()).unwrap();
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
    TlsConnector::from(Arc::new(config))
}

async fn connect(
    client: &TlsConnector,
    addr: SocketAddr,
    name: &'static str,
) -> io::Result<TlsStream<TcpStream>> {
    client
        .connect(
            ServerName::try_from(name).unwrap(),
            TcpStream::connect(addr).await?,
        )
        .await
}

fn assert_leaf(stream: &TlsStream<TcpStream>, expected: &Certificate) {
    let session = stream.get_ref().1;
    assert_eq!(session.handshake_kind(), Some(HandshakeKind::Full));
    assert_eq!(session.peer_certificates().unwrap()[0], *expected.der());
}

// HTTP/1.0 provides a close-delimited SSE body, so each line read below comes
// directly from the one original HTTP response on its verified TLS stream.
async fn request(stream: &mut (impl AsyncWrite + Unpin), path: &str) {
    stream
        .write_all(format!("GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n").as_bytes())
        .await
        .unwrap();
}

async fn line(reader: &mut (impl AsyncBufRead + Unpin)) -> String {
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).await.unwrap() > 0,
        "unexpected end of response"
    );
    line.trim_end_matches(['\r', '\n']).to_owned()
}

async fn headers(reader: &mut (impl AsyncBufRead + Unpin)) -> Vec<String> {
    let status = line(reader).await;
    assert!(
        status.starts_with("HTTP/1.0 200") || status.starts_with("HTTP/1.1 200"),
        "{status}"
    );
    let mut headers = Vec::new();
    loop {
        let header = line(reader).await;
        if header.is_empty() {
            return headers;
        }
        headers.push(header.to_ascii_lowercase());
    }
}

fn assert_released(handle: &WebHandle) {
    assert!(!handle.is_admitting() && !handle.is_ready());
    let _rebound = std::net::TcpListener::bind(handle.local_addr().unwrap()).unwrap();
}

#[test]
fn plaintext_probe_on_https_is_rejected_before_binding() {
    let (leaf, key) = leaf(&authority());
    let tls = WebTls::fixed(leaf.pem().as_bytes(), key.as_bytes()).unwrap();
    let host = App::new()
        .into_host()
        .with_tls(tls)
        .with_http_probe(HttpProbe::new(Method::GET, "/", StatusCode::OK));
    let handle = host.handle();
    let built = HostGroup::new().with_host(host).build();
    assert!(matches!(built, Err(HostBuildError::InvalidHost(_))));
    assert!(handle.local_addr().is_none());
}

#[tokio::test]
async fn one_group_serves_plain_http_and_fixed_verified_https() {
    let ca = authority();
    let (fixed, key) = leaf(&ca);
    let tls = WebTls::fixed(fixed.pem().as_bytes(), key.as_bytes()).unwrap();
    let plain = App::new()
        .get("/", || async { "plain" })
        .into_host()
        .with_name("plain")
        .with_address("127.0.0.1:0");
    let secure = App::new()
        .get("/", || async { "fixed" })
        .into_host()
        .with_tls(tls)
        .with_driver(WebDriver::OwnedThread)
        .with_name("secure")
        .with_address("127.0.0.1:0");
    let http = plain.handle();
    let https = secure.handle();
    let mut hosts = HostGroup::new()
        .with_host(plain)
        .with_host(secure)
        .build()
        .unwrap();
    let stop = hosts.shutdown_handle();
    timeout(LIMIT, async {
        assert_eq!(
            poll_fn(|cx| hosts.poll_start(cx)).await,
            StartStatus::Running
        );
        let exchange = async {
            let mut plain = TcpStream::connect(http.local_addr().unwrap())
                .await
                .unwrap();
            request(&mut plain, "/").await;
            let mut plain = BufReader::new(plain);
            headers(&mut plain).await;
            let mut body = [0; 5];
            plain.read_exact(&mut body).await.unwrap();
            assert_eq!(&body, b"plain");
            let endpoint = https.local_addr().unwrap();
            let client = client(&ca);
            assert!(connect(&client, endpoint, "wrong.invalid").await.is_err());
            let mut secure = connect(&client, endpoint, NAME).await.unwrap();
            assert_leaf(&secure, &fixed);
            request(&mut secure, "/").await;
            let mut secure = BufReader::new(secure);
            headers(&mut secure).await;
            secure.read_exact(&mut body).await.unwrap();
            assert_eq!(&body, b"fixed");
            stop.request_shutdown();
        };
        let (result, ()) = tokio::join!(hosts.run_async(), exchange);
        result.unwrap();
    })
    .await
    .unwrap();
    assert_released(&http);
    assert_released(&https);
}

#[tokio::test]
async fn rotation_preserves_the_open_sse_response_and_drains_on_shutdown() {
    let ca = authority();
    let (first, first_key) = leaf(&ca);
    let (second, second_key) = leaf(&ca);
    assert_ne!(first.der(), second.der());
    let (tls, reload) = WebTls::reloadable(first.pem().as_bytes(), first_key.as_bytes()).unwrap();

    let (events, receiver) = mpsc::unbounded::<&'static str>();
    let receiver = Arc::new(Mutex::new(Some(receiver)));
    let router =
        Router::new().route(
            "/events",
            get(move || {
                let events = receiver.lock().unwrap().take().expect("one SSE request");
                async move {
                    Sse::new(events.map(|text| Ok::<_, Infallible>(Event::default().data(text))))
                }
            }),
        );
    let web = WebHost::new(router)
        .with_tls(tls)
        .with_driver(WebDriver::CallerRuntime)
        .with_address("127.0.0.1:0");
    let handle = web.handle();
    let mut hosts = HostGroup::new().with_host(web).build().unwrap();
    let stop = hosts.shutdown_handle();

    timeout(LIMIT, async {
        assert_eq!(
            poll_fn(|cx| hosts.poll_start(cx)).await,
            StartStatus::Running
        );
        let exchange = async {
            let addr = handle.local_addr().unwrap();
            let client = client(&ca);
            // Leave a TCP peer connected without starting TLS. It must not
            // block other handshakes or remain owned after shutdown.
            let mut stalled = TcpStream::connect(addr).await.unwrap();
            let mut stream = connect(&client, addr, NAME).await.unwrap();
            assert_leaf(&stream, &first);
            request(&mut stream, "/events").await;
            let mut sse = BufReader::new(stream);
            events.unbounded_send("before").unwrap();
            assert!(
                headers(&mut sse)
                    .await
                    .iter()
                    .any(|h| h == "content-type: text/event-stream")
            );
            assert_eq!(line(&mut sse).await, "data: before");
            assert_eq!(line(&mut sse).await, "");

            assert!(
                reload
                    .reload(second.pem().as_bytes(), first_key.as_bytes())
                    .is_err()
            );
            // A new client has no session to resume when checking the failed swap.
            let fresh_client = self::client(&ca);
            let retained = connect(&fresh_client, addr, NAME).await.unwrap();
            assert_leaf(&retained, &first);
            drop(retained);
            reload
                .reload(second.pem().as_bytes(), second_key.as_bytes())
                .unwrap();
            let fresh = connect(&client, addr, NAME).await.unwrap();
            assert_leaf(&fresh, &second);
            drop(fresh);

            events.unbounded_send("after").unwrap();
            assert_eq!(line(&mut sse).await, "data: after");
            assert_eq!(line(&mut sse).await, "");
            stop.request_shutdown();
            while handle.is_admitting() {
                tokio::task::yield_now().await;
            }
            assert_eq!(stalled.read(&mut [0; 1]).await.unwrap(), 0);
            events.unbounded_send("draining").unwrap();
            drop(events);
            assert_eq!(line(&mut sse).await, "data: draining");
            assert_eq!(line(&mut sse).await, "");
        };
        let (result, ()) = tokio::join!(hosts.run_async(), exchange);
        result.unwrap();
    })
    .await
    .unwrap();
    assert_released(&handle);
}
