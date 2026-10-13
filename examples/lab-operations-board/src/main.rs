use std::{net::SocketAddr, path::PathBuf};
use yetlet_lab_operations_board::{Board, DemoIdentities};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("YETLET_BOARD_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:3012".into())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("the synthetic lab must bind loopback".into());
    }
    let path = PathBuf::from(
        std::env::var("YETLET_BOARD_DB").unwrap_or_else(|_| "target/lab-operations.sqlite3".into()),
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    let actual = listener.local_addr()?;
    let identities = DemoIdentities::generate()?;
    for (name, code) in identities.terminal_codes() {
        println!("LOCAL_LOGIN {name} {code}");
    }
    let board = Board::open(&path, actual, identities).await?;
    println!("READY http://{actual}/board");
    let router = board.router.clone();
    let stopping = board.stop_signal();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = stopping.send(true);
        })
        .await?;
    board.drain().await?;
    println!("STOPPED streams signalled; admitted database work drained");
    Ok(())
}
