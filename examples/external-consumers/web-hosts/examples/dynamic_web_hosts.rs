//! Start a second API on demand while the controller keeps serving.

use std::{error::Error, io, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{sleep, timeout},
};
use yetlet::{
    HostGroup,
    web::{App, WebHandle},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let controller = App::new()
        .get("/status", || async { "controller online" })
        .into_host()
        .with_address("127.0.0.1:0");
    let handle = controller.handle();
    let mut group = HostGroup::new().with_host(controller).build()?;
    let shutdown = group.shutdown_handle();

    let (run, work) = tokio::join!(group.run_async(), async {
        let result = start_child_while_serving(&handle).await;
        shutdown.request_shutdown();
        result
    });
    run.map_err(|error| io::Error::other(error.to_string()))?;
    println!("Controller joined.");
    work
}

async fn start_child_while_serving(controller: &WebHandle) -> Result<(), Box<dyn Error>> {
    get(controller, "/status", "controller online").await?;

    let child = App::new()
        .get("/job", || async { "job complete" })
        .into_host()
        .with_address("127.0.0.1:0");
    let handle = child.handle();
    let mut group = HostGroup::new().with_host(child).build()?;
    let shutdown = group.shutdown_handle();

    let (run, request) = tokio::join!(group.run_async(), async {
        let result = get(&handle, "/job", "job complete").await;
        shutdown.request_shutdown();
        result
    });
    run.map_err(|error| io::Error::other(error.to_string()))?;
    request?;
    println!("Child joined.");

    get(controller, "/status", "controller online").await?;
    Ok(())
}

/// Wait for ordinary admission, then check a finite HTTP request on the acquired port.
async fn get(handle: &WebHandle, path: &str, expected: &str) -> io::Result<()> {
    timeout(Duration::from_secs(5), async {
        while !handle.is_admitting() {
            sleep(Duration::from_millis(1)).await;
        }
        let address = handle
            .local_addr()
            .ok_or_else(|| io::Error::other("admitted host has no address"))?;
        let mut stream = TcpStream::connect(address).await?;
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await?;
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        if !response.starts_with("HTTP/1.1 200 OK\r\n")
            || response.split_once("\r\n\r\n").map(|(_, body)| body) != Some(expected)
        {
            return Err(io::Error::other(format!("unexpected response from {path}")));
        }
        println!("GET http://{address}{path} returned 200 {expected}");
        Ok(())
    })
    .await?
}
