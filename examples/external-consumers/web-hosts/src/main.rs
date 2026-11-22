//! Finite, API-only Web hosting on the application's current-thread runtime.

use std::{error::Error, io, net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{sleep, timeout},
};
use yetlet::{
    HostGroup,
    composition::{JoinStatus, ShutdownHandle},
    web::{App, WebHandle, WebHost},
};

struct StopOnDrop(ShutdownHandle);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

async fn ready(handle: &WebHandle) -> io::Result<SocketAddr> {
    timeout(Duration::from_secs(5), async {
        while !handle.is_admitting() {
            sleep(Duration::from_millis(1)).await;
        }
        handle
            .local_addr()
            .ok_or_else(|| io::Error::other("admitted Web Host has no address"))
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Web startup"))?
}

async fn request(address: SocketAddr, path: &str) -> io::Result<String> {
    timeout(Duration::from_secs(3), async {
        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .await?;
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "HTTP request"))?
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // No root GET route and no HTML feature are required for default readiness.
    let public = App::new()
        .get("/api", || async { "public API" })
        .into_host()
        .with_name("public");
    let admin = WebHost::new(
        App::new()
            .get("/admin", || async { "admin API" })
            .into_router(),
    )
    .with_name("admin")
    .with_address("127.0.0.1:0");
    let public_handle = public.handle();
    let admin_handle = admin.handle();
    assert_eq!(public.address(), "127.0.0.1:7878");
    assert!(public_handle.local_addr().is_none());

    let mut hosts = HostGroup::new()
        .with_host(public)
        .with_host(admin)
        .build()?;
    assert!(public_handle.local_addr().is_none(), "build must be inert");
    let shutdown = hosts.shutdown_handle();

    let inspect = async {
        let _stop = StopOnDrop(shutdown);
        let public_address = ready(&public_handle).await?;
        let admin_address = ready(&admin_handle).await?;
        if public_address != "127.0.0.1:7878".parse::<SocketAddr>().unwrap()
            || admin_address.port() == 0
            || public_address == admin_address
        {
            return Err(io::Error::other("unexpected configured Web addresses"));
        }
        for (address, path, expected) in [
            (public_address, "/api", "public API"),
            (admin_address, "/admin", "admin API"),
        ] {
            let response = request(address, path).await?;
            if !response.starts_with("HTTP/1.1 200") || !response.ends_with(expected) {
                return Err(io::Error::other("unexpected API response"));
            }
            println!("{address}{path}: {expected}");
        }
        let root = request(public_address, "/").await?;
        if !root.starts_with("HTTP/1.1 404") {
            return Err(io::Error::other("the API-only root should remain absent"));
        }
        Ok::<_, io::Error>(())
    };

    // The group stays outside both futures. Only the borrowing drive is awaited.
    let (run, requests) = tokio::join!(hosts.run_async(), inspect);
    run.map_err(|error| io::Error::other(error.to_string()))?;
    requests?;
    assert!(!public_handle.is_ready() && !admin_handle.is_ready());
    assert!(!public_handle.is_admitting() && !admin_handle.is_admitting());
    assert!(
        hosts
            .outcomes()
            .iter()
            .all(|host| host.join == JoinStatus::Succeeded)
    );
    println!("Two Web Hosts completed through their retained lifecycle owner.");
    Ok(())
}
