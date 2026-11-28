#![cfg(not(target_arch = "wasm32"))]

use std::{
    convert::Infallible,
    env,
    error::Error,
    fs, io,
    net::SocketAddr,
    time::{Duration, UNIX_EPOCH},
};

use axum::{
    Router,
    response::{Sse, sse::Event},
    routing::get,
};
use futures_util::stream;
use yetlet::{
    HostGroup,
    acme::{AcmeConfig, AcmeHost},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let http: SocketAddr = env::var("ACME_HTTP_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    let https: SocketAddr = env::var("ACME_HTTPS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8443".into())
        .parse()?;
    let mut config = AcmeConfig::new(
        env::var("ACME_DOMAIN")?,
        env::var("ACME_DIRECTORY")?,
        env::var("ACME_STATE_DIR")?,
    )?;
    if let Some(path) = env::var_os("ACME_DIRECTORY_ROOT_PEM") {
        config = config.with_directory_root(path);
    }
    if let Some(path) = env::var_os("ACME_ISSUANCE_ROOT_PEM") {
        config = config.with_issuance_root(path);
    }
    match (
        env::var("ACME_EAB_KEY_ID").ok(),
        env::var_os("ACME_EAB_KEY_FILE"),
    ) {
        (Some(id), Some(path)) => config = config.with_eab(id, &fs::read(path)?),
        (None, None) => (),
        _ => return Err("set both ACME_EAB_KEY_ID and ACME_EAB_KEY_FILE".into()),
    }
    if env::var("ACME_AGREE_TO_TERMS").as_deref() == Ok("1") {
        config = config.with_terms_of_service_agreed();
    }
    if let Ok(seconds) = env::var("ACME_RETRY_SECONDS") {
        config = config.with_retry_policy(
            Duration::from_secs(seconds.parse()?),
            Duration::from_secs(86400),
        )?;
    }
    if let Ok(seconds) = env::var("ACME_RENEW_BEFORE_SECONDS") {
        config = config.with_renew_before(Duration::from_secs(seconds.parse()?))?;
    }

    let plain = Router::new().route("/", get(|| async { "HTTP listener is ready" }));
    let secure = Router::new()
        .route("/", get(|| async { "Verified HTTPS" }))
        .route(
            "/events",
            get(|| async {
                Sse::new(stream::unfold(1u64, |index| async move {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    Some((
                        Ok::<_, Infallible>(Event::default().data(index.to_string())),
                        index + 1,
                    ))
                }))
            }),
        );
    let (plain, acme) = AcmeHost::http01(config, plain, http, secure, https);
    let handle = acme.handle();
    let mut hosts = HostGroup::new().with_host(plain).with_host(acme).build()?;
    let close = hosts.shutdown_handle();
    let status = async move {
        let mut previous = None;
        loop {
            let view = handle.status();
            if previous != Some(view) {
                let millis = |time: std::time::SystemTime| {
                    time.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis())
                };
                let line = serde_json::json!({
                    "expires": view.certificate_expires_at.and_then(millis),
                    "retry": view.retry_at.and_then(millis),
                    "failure": view.last_failure.map(|reason| format!("{reason:?}")),
                    "https_address": view.https_address.map(|address| address.to_string()),
                });
                println!("{line}");
                previous = Some(view);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    };
    let runner = hosts.run_async();
    tokio::pin!(runner);
    tokio::select! {
        result = &mut runner => result.map_err(|e| io::Error::other(e.to_string()))?,
        signal = tokio::signal::ctrl_c() => {
            close.request_shutdown();
            runner.await.map_err(|e| io::Error::other(e.to_string()))?;
            signal?;
        }
        () = status => unreachable!(),
    }
    Ok(())
}
