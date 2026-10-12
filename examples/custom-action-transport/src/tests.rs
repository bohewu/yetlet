use super::*;
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::oneshot, task::JoinHandle};
use yetlet_actions::LocalExecutor;

// Direct product fixture: owns exactly one listener and one connection task.
struct Peer {
    endpoint: SocketAddr,
    task: JoinHandle<std::io::Result<()>>,
}

impl Peer {
    async fn start<F, Fut>(handler: F) -> Self
    where
        F: FnOnce(TcpStream) -> Fut + Send + 'static,
        Fut: Future<Output = std::io::Result<()>> + Send,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            timeout(Duration::from_secs(2), async move {
                let (socket, _) = listener.accept().await?;
                drop(listener);
                handler(socket).await
            })
            .await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "fixture deadline"))?
        });
        Self { endpoint, task }
    }

    async fn finish(mut self) -> std::io::Result<()> {
        let joined = timeout(Duration::from_secs(3), &mut self.task).await;
        let result = match joined {
            Ok(result) => result.unwrap(),
            Err(_) => {
                self.task.abort();
                let _ = (&mut self.task).await;
                panic!("peer exceeded cleanup deadline");
            }
        };
        assert!(TcpStream::connect(self.endpoint).await.is_err());
        println!(
            "cleanup: peer joined; client dropped; {} reconnect refused",
            self.endpoint
        );
        result
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        // Panic fallback prevents a detached task; normal paths join before asserting outcomes.
        self.task.abort();
    }
}

fn executor(peer: &Peer) -> TcpExecutor {
    TcpExecutor::new(peer.endpoint, Duration::from_secs(1)).unwrap()
}

#[tokio::test]
async fn explicit_selection_and_typed_network_result() {
    let peer = Peer::start(serve_connection).await;
    let local = LocalExecutor::new(|command: Add| {
        Ok(Sum {
            value: command.left + command.right,
        })
    });
    let local_result = local
        .execute(Add {
            left: 20,
            right: 22,
        })
        .await;
    assert!(
        !peer.task.is_finished(),
        "local execution must not contact the listener"
    );
    let started = std::time::Instant::now();
    let result = executor(&peer)
        .execute(Add {
            left: 20,
            right: 22,
        })
        .await;
    let elapsed = started.elapsed();
    peer.finish().await.unwrap();
    assert_eq!(local_result, Ok(Sum { value: 42 }));
    assert_eq!(result, Ok(Sum { value: 42 }));
    println!(
        "typed real TCP Add(20,22) -> Sum(42); request=22B reply=26B (+4B each header); elapsed_ns={}",
        elapsed.as_nanos()
    );
}

#[tokio::test]
async fn actual_wire_payloads_and_malformed_commands() {
    for (request, expected) in [
        (r#"{"left":20,"right":22}"#, r#"{"result":"ok","value":42}"#),
        ("not json", r#"{"result":"error","code":"invalid_command"}"#),
        (
            r#"{"left":1,"right":2,"event":"click"}"#,
            r#"{"result":"error","code":"invalid_command"}"#,
        ),
    ] {
        let peer = Peer::start(serve_connection).await;
        let mut client = framed(TcpStream::connect(peer.endpoint).await.unwrap());
        client
            .send(request.as_bytes().to_vec().into())
            .await
            .unwrap();
        let response = client.next().await.unwrap().unwrap();
        let eof = client.next().await;
        drop(client);
        peer.finish().await.unwrap();
        println!(
            "wire request_bytes={} request={} reply_bytes={} reply={}",
            request.len(),
            request,
            response.len(),
            String::from_utf8_lossy(&response)
        );
        assert_eq!(response.as_ref(), expected.as_bytes());
        assert!(eof.is_none(), "one response then server closes");
    }
}

#[tokio::test]
async fn remote_validation_is_not_a_success() {
    let peer = Peer::start(serve_connection).await;
    let result = executor(&peer)
        .execute(Add {
            left: u32::MAX,
            right: 1,
        })
        .await;
    peer.finish().await.unwrap();
    assert_eq!(
        result,
        Err(ActionError::Remote {
            status: 422,
            code: "overflow".into()
        })
    );
    println!("typed overflow -> Remote(422, overflow); no retry");
}

#[tokio::test]
async fn oversized_and_truncated_frames_close_without_dispatch() {
    for bytes in [
        ((MAX_FRAME_BYTES + 1) as u32).to_be_bytes().to_vec(),
        vec![0, 0, 0, 10, b'{'],
    ] {
        let peer = Peer::start(serve_connection).await;
        let mut client = TcpStream::connect(peer.endpoint).await.unwrap();
        client.write_all(&bytes).await.unwrap();
        client.shutdown().await.unwrap();
        drop(client);
        let result = peer.finish().await;
        assert!(result.is_err());
        println!("invalid frame {:?} rejected before command decode", bytes);
    }
}

#[tokio::test]
async fn malformed_and_missing_responses_are_not_successes() {
    for response in [
        Some("not json"),
        Some(r#"{"result":"ok","value":"bad"}"#),
        None,
    ] {
        let peer = Peer::start(move |socket| async move {
            let mut wire = framed(socket);
            let _request = wire.next().await.unwrap()?;
            if let Some(response) = response {
                wire.send(response.as_bytes().to_vec().into()).await?;
            }
            wire.close().await
        })
        .await;
        let result = executor(&peer).execute(Add { left: 1, right: 2 }).await;
        peer.finish().await.unwrap();
        assert_eq!(result, Err(ActionError::InvalidResponse));
        println!("bad peer reply {:?} -> InvalidResponse", response);
    }
}

#[tokio::test]
async fn deadline_and_caller_cancellation_close_owned_client() {
    for caller_cancels in [false, true] {
        let (seen_tx, seen_rx) = oneshot::channel();
        let peer = Peer::start(move |socket| async move {
            let mut wire = framed(socket);
            let _request = wire.next().await.unwrap()?;
            seen_tx.send(()).unwrap();
            // The client sends no next frame: this resolves only when its socket closes.
            if wire.next().await.is_some() {
                return Err(std::io::Error::other("expected disconnect"));
            }
            Ok(())
        })
        .await;
        let adapter = TcpExecutor::new(peer.endpoint, Duration::from_millis(50)).unwrap();
        let result = {
            let mut request = Box::pin(adapter.execute(Add { left: 1, right: 2 }));
            let result = if caller_cancels {
                tokio::select! {
                    _ = seen_rx => None,
                    result = &mut request => Some(result),
                }
            } else {
                Some(request.as_mut().await)
            };
            drop(request);
            result
        };
        peer.finish().await.unwrap();
        if caller_cancels {
            assert_eq!(result, None);
        } else {
            assert_eq!(result, Some(Err(ActionError::Network)));
        }
        println!(
            "{} closes client socket; peer sees EOF",
            if caller_cancels {
                "caller cancellation"
            } else {
                "50ms deadline"
            }
        );
    }
}

#[tokio::test]
async fn endpoint_restriction_and_network_failure() {
    assert!(matches!(
        TcpExecutor::new("192.0.2.1:1234".parse().unwrap(), Duration::from_secs(1)),
        Err(ActionError::InvalidEndpoint)
    ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = listener.local_addr().unwrap();
    drop(listener);
    let result = TcpExecutor::new(endpoint, Duration::from_secs(1))
        .unwrap()
        .execute(Add { left: 1, right: 2 })
        .await;
    assert_eq!(result, Err(ActionError::Network));
    println!(
        "cleanup: dropped {} listener; refused connect -> Network",
        endpoint
    );
}
