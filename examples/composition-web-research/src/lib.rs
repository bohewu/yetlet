//! F0-B2: one opt-in, private, native Axum Web consumer.
//!
//! Ordinary Rust configures an existing Yetlet App<S> and returns its native
//! Router. This has no hidden runtime, listener, Host registry, game state,
//! worker, Command mailbox or framework-level shutdown abstraction.
//! Listener binding, serving, and graceful drain belong to the caller.

use axum::{Json, Router, extract::State};
use yetlet::{
    view::View,
    web::{App, Page},
};

#[derive(Clone, Debug)]
pub struct SiteState {
    pub title: &'static str,
    pub status: &'static str,
    #[cfg(test)]
    drain_probe: Option<std::sync::Arc<tests::DrainProbe>>,
}

impl SiteState {
    pub fn new(title: &'static str, status: &'static str) -> Self {
        Self {
            title,
            status,
            #[cfg(test)]
            drain_probe: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    EmptyTitle,
    EmptyStatus,
}

async fn home(State(site): State<SiteState>) -> Page {
    Page::new(site.title, View::text(site.title))
}

async fn status(State(site): State<SiteState>) -> Json<&'static str> {
    #[cfg(test)]
    if let Some(gate) = &site.drain_probe {
        use std::sync::atomic::Ordering;
        if gate.hold_next.swap(false, Ordering::SeqCst) {
            gate.entered.notify_one();
            gate.release.notified().await;
        }
    }
    Json(site.status)
}

/// A concrete Pure Web composition root: only configure and resolve routes.
/// No TcpListener is bound, and no native serving future is polled here.
/// The caller retains the existing App::serve/serve_on/into_router choices.
pub fn configure_router(site: SiteState) -> Result<Router, SetupError> {
    let app = App::<SiteState>::new().try_configure(|app| {
        if site.title.is_empty() {
            return Err(SetupError::EmptyTitle);
        }
        if site.status.is_empty() {
            return Err(SetupError::EmptyStatus);
        }
        Ok(app.get("/", home).get("/api/status", status))
    })?;

    Ok(app.with_state::<()>(site).into_router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };
    use tokio::{
        net::TcpListener,
        sync::{Notify, oneshot},
        time::timeout,
    };
    use tower::ServiceExt;

    /// This gate exists only in tests. The normal Web consumer has no
    /// per-request lifecycle state/overhead or artificial worker.
    #[derive(Debug, Default)]
    pub(super) struct DrainProbe {
        pub(super) hold_next: AtomicBool,
        pub(super) entered: Notify,
        pub(super) release: Notify,
    }

    #[tokio::test(flavor = "current_thread")]
    async fn f0b2_router_configured_without_binding_and_uses_native_routes() {
        assert!(matches!(
            configure_router(SiteState::new("", "ready")),
            Err(SetupError::EmptyTitle)
        ));
        assert!(matches!(
            configure_router(SiteState::new("site", "")),
            Err(SetupError::EmptyStatus)
        ));
        let router = configure_router(SiteState::new("Web <&> sample", "ready")).unwrap();
        let response = router
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_SECURITY_POLICY)
                .is_some()
        );
        let html = to_bytes(response.into_body(), 8192).await.unwrap();
        let html = std::str::from_utf8(&html).unwrap();
        assert!(html.contains("<title>Web &lt;&amp;&gt; sample</title>"));
        assert!(html.contains("Web &lt;&amp;&gt; sample"));
        assert!(!html.contains("<script"));

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), 512).await.unwrap().as_ref(),
            b"\"ready\""
        );

        let unknown = router
            .oneshot(
                Request::builder()
                    .uri("/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn f0b2_listener_bind_is_separate_and_conflict_is_native_io() {
        let _router = configure_router(SiteState::new("plain", "ready")).unwrap();
        // Configuration did not bind. The *caller* can choose an ephemeral port.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        let conflict = TcpListener::bind(address).await;
        assert!(conflict.is_err(), "native second bind should not succeed");
        assert_eq!(conflict.unwrap_err().kind(), std::io::ErrorKind::AddrInUse);
        drop(listener);
        let rebound = TcpListener::bind(address).await.unwrap();
        drop(rebound);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn f0b2_real_loopback_http_and_graceful_drain_retains_owned_task() {
        let gate = Arc::new(DrainProbe::default());
        let mut site = SiteState::new("Pure Web SSR", "ready");
        site.drain_probe = Some(Arc::clone(&gate));

        // A configured Router is not a listening server.
        let router = configure_router(site).unwrap();
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        assert!(address.ip().is_loopback());
        let (signal, shutdown) = oneshot::channel::<()>();

        // A caller-owned Tokio runtime is provided by the test runner. The
        // caller owns this native Axum serving task and its completion handle.
        let mut serving = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown.await;
                })
                .await
        });
        let base = format!("http://{address}");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        // An actual successful response, not Router construction or spawn,
        // is the independent observation that serving was driven.
        let ssr = timeout(
            Duration::from_secs(4),
            client.get(format!("{base}/")).send(),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(ssr.status(), StatusCode::OK);
        let body = ssr.text().await.unwrap();
        assert!(body.contains("Pure Web SSR"));
        let api = timeout(
            Duration::from_secs(4),
            client.get(format!("{base}/api/status")).send(),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(api.status(), StatusCode::OK);
        assert_eq!(api.text().await.unwrap(), "\"ready\"");

        // Keep one ordinary API request in-flight when asking native Axum
        // to drain. No additional route, Host, Actor or worker is invented.
        gate.hold_next.store(true, Ordering::SeqCst);
        let pending_client = client.clone();
        let pending_uri = format!("{base}/api/status");
        let pending = tokio::spawn(async move { pending_client.get(pending_uri).send().await });
        timeout(Duration::from_secs(3), gate.entered.notified())
            .await
            .expect("request entered the native handler");

        signal
            .send(())
            .expect("server shutdown listener is still owned");
        assert!(
            timeout(Duration::from_millis(80), &mut serving)
                .await
                .is_err(),
            "a drain observation timeout must not be reported as cleanup complete"
        );
        assert!(
            !serving.is_finished(),
            "the caller retains the serving handle"
        );

        gate.release.notify_one();
        let finished_api = timeout(Duration::from_secs(3), pending)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(finished_api.status(), StatusCode::OK);
        assert_eq!(finished_api.text().await.unwrap(), "\"ready\"");

        // Await the exact same serving task, rather than dropping or aborting
        // it and treating its internal connection lifecycle as joined.
        let done = timeout(Duration::from_secs(4), &mut serving)
            .await
            .expect("native Axum drain was actually observed")
            .expect("owned serving task did not panic");
        done.expect("native serve completed");
        drop(client);

        let rebound = TcpListener::bind(address)
            .await
            .expect("original socket was released after actual completion");
        drop(rebound);
        assert!(
            tokio::runtime::Handle::try_current().is_ok(),
            "caller runtime remains usable after the Web root ends"
        );
    }
}
