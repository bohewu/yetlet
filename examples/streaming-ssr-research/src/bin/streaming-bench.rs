//! Direct YT-021 localhost HTTP/1.1 measurements, bounded to three fixed workloads.

use std::{
    error::Error,
    time::{Duration, Instant},
};

use axum::{
    Router,
    body::Bytes,
    http::{Request, header},
};
use http_body_util::{BodyExt, Empty};
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
    time::timeout,
};
use yetlet_streaming_ssr_research::{CSP, Workload, full_html, router};

type Failure = Box<dyn Error + Send + Sync>;

struct Measurement {
    first_body_ns: u128,
    complete_ns: u128,
    lifecycle_ns: u128,
    bytes: usize,
}

async fn request(
    address: std::net::SocketAddr,
    path: &str,
    expected: &[u8],
) -> Result<Measurement, Failure> {
    // Clock starts before TCP connect: includes new connection and HTTP handshake.
    let start = Instant::now();
    let socket = TcpStream::connect(address).await?;
    socket.set_nodelay(true)?;
    let (mut sender, connection) =
        hyper::client::conn::http1::handshake(TokioIo::new(socket)).await?;
    let mut connection_task = tokio::spawn(connection);
    let result = timeout(Duration::from_secs(5), async {
        let response = sender
            .send_request(
                Request::builder()
                    .uri(path)
                    .header(header::HOST, address.to_string())
                    .body(Empty::<Bytes>::new())?,
            )
            .await?;
        if response.status() != 200
            || response.headers().get(header::CONTENT_SECURITY_POLICY)
                != Some(&axum::http::HeaderValue::from_static(CSP))
        {
            return Err("unexpected HTTP status or CSP".into());
        }
        let mut body = response.into_body();
        let mut first_body_ns = None;
        let mut received = Vec::with_capacity(expected.len());
        while let Some(frame) = body.frame().await {
            if let Ok(data) = frame?.into_data()
                && !data.is_empty()
            {
                first_body_ns.get_or_insert_with(|| start.elapsed().as_nanos());
                received.extend_from_slice(&data);
            }
        }
        let complete_ns = start.elapsed().as_nanos();
        if received != expected {
            return Err("HTTP response bytes differ from full SSR".into());
        }
        Ok::<_, Failure>(Measurement {
            first_body_ns: first_body_ns.ok_or("empty HTML body")?,
            complete_ns,
            lifecycle_ns: 0,
            bytes: received.len(),
        })
    })
    .await;
    drop(sender);
    // Join each connection driver before measuring the next request.
    match timeout(Duration::from_secs(2), &mut connection_task).await {
        Ok(joined) => {
            joined??;
        }
        Err(_) => {
            connection_task.abort();
            let _ = connection_task.await;
            return Err("connection driver cleanup timed out; aborted and joined".into());
        }
    }
    let mut measurement = result??;
    measurement.lifecycle_ns = start.elapsed().as_nanos();
    Ok(measurement)
}

async fn measure(
    address: std::net::SocketAddr,
    workloads: &[(&str, Workload)],
) -> Result<(), Failure> {
    const WARMUPS: usize = 3;
    const SAMPLES: usize = 20;
    println!(
        "# protocol=HTTP/1.1,transport=loopback_no_tls,no_compression,new_tcp_per_request,runtime=current_thread,warmups_per_workload_mode={WARMUPS},samples_per_workload_mode={SAMPLES},concurrency=1"
    );
    println!(
        "# first_body_ns=before_TCP_connect_to_first_nonempty_decoded_HTTP_body_data; complete_ns=before_TCP_connect_to_body_EOF; bytes=decoded_HTML_bytes"
    );
    println!(
        "# lifecycle_ns=before_TCP_connect_to_correctness_comparison_and_connection_driver_join; sequential_throughput=1e9/mean_lifecycle_ns requests_per_second; bytes_per_second=bytes*1e9/mean_lifecycle_ns; CSV_printing_and_between_sample_gaps_excluded"
    );
    println!(
        "workload,sample,mode,rows,text_repetitions,tail_delay_ms,bytes,first_body_ns,complete_ns,lifecycle_ns"
    );
    for (name, workload) in workloads {
        let expected = full_html(*workload).await?;
        for mode in ["full", "stream"] {
            for _ in 0..WARMUPS {
                request(address, &format!("/{name}/{mode}"), expected.as_bytes()).await?;
            }
        }
        for sample in 0..SAMPLES {
            // Rotate order within each matched pair; no concurrent workload.
            for offset in 0..2 {
                let mode = ["full", "stream"][(sample + offset) % 2];
                let observed =
                    request(address, &format!("/{name}/{mode}"), expected.as_bytes()).await?;
                println!(
                    "{name},{},{mode},{},{},{},{},{},{},{}",
                    sample + 1,
                    workload.rows,
                    workload.text_repetitions,
                    workload.tail_delay.as_millis(),
                    observed.bytes,
                    observed.first_body_ns,
                    observed.complete_ns,
                    observed.lifecycle_ns
                );
            }
        }
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Failure> {
    let workloads = [
        (
            "small",
            Workload {
                rows: 100,
                text_repetitions: 4,
                tail_delay: Duration::ZERO,
                invalid_row: None,
            },
        ),
        (
            "large",
            Workload {
                rows: 10_000,
                text_repetitions: 4,
                tail_delay: Duration::ZERO,
                invalid_row: None,
            },
        ),
        (
            "delayed",
            Workload {
                rows: 1_000,
                text_repetitions: 4,
                tail_delay: Duration::from_millis(20),
                invalid_row: None,
            },
        ),
    ];
    let mut app = Router::new();
    for (name, workload) in workloads {
        app = app.nest(&format!("/{name}"), router(workload));
    }
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown, stop) = oneshot::channel();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stop.await;
            })
            .await
    });
    let result = measure(address, &workloads).await;
    let _ = shutdown.send(());
    match timeout(Duration::from_secs(3), &mut server).await {
        Ok(joined) => {
            joined??;
        }
        Err(_) => {
            server.abort();
            let _ = server.await;
            return Err("server cleanup timed out; aborted and joined".into());
        }
    }
    // A failed reconnect demonstrates closure of the owned ephemeral listener.
    assert!(TcpStream::connect(address).await.is_err());
    println!(
        "# cleanup=all_connection_drivers_joined,server_joined,ephemeral_listener_reconnect_refused,address={address}"
    );
    result
}
