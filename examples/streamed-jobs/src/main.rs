//! One job per sender process. The receiver has one slot for its temporary file.
mod receiver;
mod sender;

use std::{error::Error, net::Ipv4Addr, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use yetlet::web::App;

type Failure = Box<dyn Error + Send + Sync>;
const CHUNK: usize = 64 * 1024;
const MAX_BYTES: u64 = 64 * 1024 * 1024;
// Pace this small exercise so a reader can disconnect and observe both phases.
const PACE: Duration = Duration::from_millis(12);
const TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Receipt {
    bytes: u64,
    sha256: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (port, app) = match args.as_slice() {
        [role, port] if role == "receiver" => (
            port,
            App::new()
                .post("/receive", receiver::receive)
                .with_state(std::sync::Arc::new(tokio::sync::Semaphore::new(1)))
                .into_router(),
        ),
        [role, port, receiver_port, path] if role == "sender" => (
            port,
            App::new()
                .post("/jobs/demo", sender::start)
                .get("/jobs/demo/events", sender::events)
                .with_state(sender::Sender::new(receiver_port.parse()?, path.into()))
                .into_router(),
        ),
        _ => {
            return Err(
                "usage: yetlet-streamed-jobs receiver PORT | sender PORT RECEIVER_PORT FILE".into(),
            );
        }
    };
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port.parse::<u16>()?)).await?;
    println!("listening http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
