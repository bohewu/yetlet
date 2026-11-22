use axum::{Router, routing::get};
use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};
use yetlet_host_lifecycle_research::{Application, ShutdownHandle, WebHost};

struct StopOnDrop(ShutdownHandle);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let public =
        WebHost::new(Router::new().route("/", get(|| async { "public response" }))).named("public");
    let admin =
        WebHost::new(Router::new().route("/", get(|| async { "admin response" }))).named("admin");
    let statuses = [public.status(), admin.status()];
    let app = Application::new().with_host(public).with_host(admin);
    let stop = app.shutdown_handle();
    let built = app.build()?;
    let (result, client_result) = thread::scope(|scope| {
        let client = scope.spawn(move || -> io::Result<()> {
            let _stop = StopOnDrop(stop);
            for status in statuses {
                let deadline = Instant::now() + Duration::from_secs(5);
                while !status.admission_open() {
                    if Instant::now() >= deadline {
                        return Err(io::Error::new(io::ErrorKind::TimedOut, "Web startup"));
                    }
                    thread::sleep(Duration::from_millis(2));
                }
                let address = status.endpoint().expect("ready listener address");
                let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
                stream.set_read_timeout(Some(Duration::from_secs(2)))?;
                stream.set_write_timeout(Some(Duration::from_secs(2)))?;
                stream
                    .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                if !response.starts_with("HTTP/1.1 200") {
                    return Err(io::Error::other("unexpected HTTP response"));
                }
                println!(
                    "{address}: {}",
                    response.split("\r\n\r\n").nth(1).unwrap_or("")
                );
            }
            Ok(())
        });
        let result = built.run();
        let client_result = client.join().expect("client panicked");
        (result, client_result)
    });
    let report = result?;
    client_result?;
    println!(
        "{} Hosts joined; application resources released",
        report.hosts.len()
    );
    Ok(())
}
