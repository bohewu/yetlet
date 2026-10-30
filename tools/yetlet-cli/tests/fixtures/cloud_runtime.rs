// Appended only to a disposable generated app by the explicit CLI product test.
#[cfg(test)]
mod cloud_tests {
    use super::*;

    #[tokio::test]
    async fn health_reports_only_local_process_status() {
        let readiness = Arc::new(AtomicBool::new(true));
        assert_eq!(live().await, StatusCode::OK);
        assert_eq!(ready(Extension(readiness.clone())).await, StatusCode::OK);
        readiness.store(false, Ordering::SeqCst);
        assert_eq!(
            ready(Extension(readiness)).await,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live().await, StatusCode::OK);
    }

    #[test]
    fn configuration_is_strict_and_defaults_stay_local() {
        assert_eq!(
            bind_address(None, None).unwrap().to_string(),
            "127.0.0.1:3000"
        );
        assert_eq!(
            bind_address(Some("0.0.0.0".into()), Some("8080".into()))
                .unwrap()
                .to_string(),
            "0.0.0.0:8080"
        );
        for host in [
            "",
            "localhost",
            "::1",
            "127.0.0.2",
            "192.0.2.1",
            "0.0.0.0 ",
            "[::]",
            "secret",
        ] {
            let error = bind_address(Some(host.into()), None).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(!error.to_string().contains("secret"));
        }
        for port in [
            "", "0", "65536", "-1", "+3000", " 3000", "3000 ", "3.0", "１２", "secret",
        ] {
            let error = bind_address(None, Some(port.into())).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(!error.to_string().contains("secret"));
        }
        for port in ["1", "3000", "65535"] {
            assert_eq!(
                bind_address(None, Some(port.into()))
                    .unwrap()
                    .port()
                    .to_string(),
                port
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            assert!(bind_address(Some(OsString::from_vec(vec![255])), None).is_err());
            assert!(bind_address(None, Some(OsString::from_vec(vec![255]))).is_err());
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn early_sigint_is_registered_before_the_shutdown_future_is_polled() {
        // A genuine early interrupt must not take the default process-kill path.
        let readiness = Arc::new(AtomicBool::new(true));
        let waiting = shutdown_signal(readiness.clone()).unwrap();
        let status = std::process::Command::new("kill")
            .args(["-INT", &std::process::id().to_string()])
            .status()
            .unwrap();
        assert!(status.success());
        waiting.await;
        assert!(!readiness.load(Ordering::SeqCst));
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires native loopback and actual SIGTERM/SIGINT; run explicitly on a capable host"]
    async fn native_health_ssr_signals_readiness_and_inflight_drain() {
        use std::{
            future::IntoFuture,
            io::{Read, Write},
            net::TcpStream,
            process::Command,
            sync::{Mutex, mpsc},
            time::{Duration, Instant},
        };

        fn request(address: SocketAddr, path: &str) -> String {
            let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3)).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .write_all(
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                        .as_bytes(),
                )
                .unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        }

        for signal in ["-TERM", "-INT"] {
            let readiness = Arc::new(AtomicBool::new(true));
            let shutdown = shutdown_signal(readiness.clone()).unwrap();
            let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
                .await
                .unwrap();
            let address = listener.local_addr().unwrap();
            let (entered_tx, entered_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            let release = Arc::new(Mutex::new(release_rx));
            // A test-only slow handler establishes a genuinely in-flight request.
            let app = router(readiness.clone()).route(
                "/__test_drain",
                axum::routing::get(move || {
                    let release = release.clone();
                    let entered = entered_tx.clone();
                    async move {
                        tokio::task::spawn_blocking(move || {
                            entered.send(()).unwrap();
                            release
                                .lock()
                                .unwrap()
                                .recv_timeout(Duration::from_secs(5))
                                .unwrap();
                        })
                        .await
                        .unwrap();
                        "drained"
                    }
                }),
            );
            let server = tokio::spawn(
                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown)
                    .into_future(),
            );
            for path in ["/health/live", "/health/ready"] {
                let response = request(address, path);
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                assert_eq!(response.split_once("\r\n\r\n").unwrap().1, "");
            }
            for (path, text) in [
                ("/", "Hello &lt;Yetlet&gt; &amp; Rust!"),
                ("/hello/%3Cscript%3E%26", "Hello, &lt;script&gt;&amp;!"),
            ] {
                let response = request(address, path);
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                assert!(response.contains("content-security-policy:"), "{response}");
                assert!(
                    response.contains(yetlet::server_axum::DEFAULT_CSP),
                    "{response}"
                );
                assert!(response.contains(text), "{response}");
                assert!(!response.contains("<script"));
            }
            let client = std::thread::spawn(move || request(address, "/__test_drain"));
            entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            assert!(
                Command::new("kill")
                    .args([signal, &std::process::id().to_string()])
                    .status()
                    .unwrap()
                    .success()
            );
            let deadline = Instant::now() + Duration::from_secs(3);
            while readiness.load(Ordering::SeqCst) {
                assert!(
                    Instant::now() < deadline,
                    "real signal did not change readiness"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(
                ready(Extension(readiness)).await,
                StatusCode::SERVICE_UNAVAILABLE
            );
            assert_eq!(live().await, StatusCode::OK);
            assert!(
                !server.is_finished(),
                "server must wait for in-flight handler"
            );
            release_tx.send(()).unwrap();
            let response = client.join().unwrap();
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            assert!(response.ends_with("drained"), "{response}");
            server.await.unwrap().unwrap();
            assert!(TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err());
            eprintln!(
                "{signal}: real signal, readiness 503, in-flight response completed, server reaped, listener closed"
            );
        }
    }
}
