//! End an admitted SSE response and cancel an outstanding TCP receipt on Ctrl+C.

use std::{convert::Infallible, error::Error, io, net::SocketAddr, time::Duration};

use axum::response::sse::{Event, Sse};
use futures_util::stream::{self, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::watch,
    time::sleep,
};
use yetlet::{
    HostGroup,
    web::{App, WebHandle},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let peer: SocketAddr = std::env::args()
        .nth(1)
        .ok_or("usage: sse_shutdown <loopback-peer-address:port>")?
        .parse()?;
    if !peer.ip().is_loopback() || peer.port() == 0 {
        return Err("supply a loopback TCP peer with a nonzero port".into());
    }

    let (stop, mut stopping) = watch::channel(false);
    let stream_stop = stopping.clone();
    let host = App::new()
        .get("/events", move || {
            let mut stream_stop = stream_stop.clone();
            async move {
                let ready = Ok::<_, Infallible>(Event::default().data("ready"));
                let events = stream::iter([ready]).chain(stream::pending());
                Sse::new(events.take_until(async move {
                    let _ = stream_stop.wait_for(|stop| *stop).await;
                }))
            }
        })
        .into_host()
        .with_address("127.0.0.1:0");
    let web = host.handle();
    let mut group = HostGroup::new().with_host(host).build()?;
    let shutdown = group.shutdown_handle();

    let mut worker_stop = stopping.clone();
    let worker = tokio::spawn(async move {
        tokio::select! {
            result = request_receipt(web, peer) => result.map(|()| "received ACK"),
            _ = worker_stop.wait_for(|stop| *stop) => Ok("cancelled on shutdown"),
        }
    });

    // Either owner finishing early also wakes the signal branch. It closes
    // admission while the same watch value releases admitted streams and I/O.
    let (host_result, worker_result, signal_result) = tokio::join!(
        async {
            let result = group.run_async().await;
            stop.send_replace(true);
            result
        },
        async {
            let result = worker.await;
            stop.send_replace(true);
            result
        },
        async {
            let result = tokio::select! {
                signal = tokio::signal::ctrl_c() => signal,
                _ = stopping.wait_for(|stop| *stop) => Ok(()),
            };
            stop.send_replace(true);
            shutdown.request_shutdown();
            result
        }
    );
    host_result.map_err(|error| io::Error::other(error.to_string()))?;
    println!("TCP task joined: {}", worker_result??);
    signal_result?;
    println!("Web Host stopped.");
    Ok(())
}

async fn request_receipt(web: WebHandle, peer: SocketAddr) -> io::Result<()> {
    while !web.is_admitting() {
        sleep(Duration::from_millis(10)).await;
    }
    let address = web
        .local_addr()
        .ok_or_else(|| io::Error::other("admitted host has no address"))?;
    println!("SSE endpoint: http://{address}/events");

    let mut socket = TcpStream::connect(peer).await?;
    socket.write_all(b"RECEIPT?\n").await?;
    println!("Waiting for TCP ACK.");
    let mut reply = [0; 4];
    socket.read_exact(&mut reply).await?;
    if &reply != b"ACK\n" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "expected ACK"));
    }
    Ok(())
}
