use std::{net::SocketAddr, path::PathBuf};

use yetlet_sqlite_crud::{Database, app, listen_address};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = listen_address(
        &std::env::var("YETLET_SQLITE_LISTEN").unwrap_or_else(|_| "127.0.0.1:3007".to_owned()),
    )?;
    let path = PathBuf::from(
        std::env::var("YETLET_SQLITE_PATH")
            .unwrap_or_else(|_| "target/yetlet-sqlite-crud.sqlite3".to_owned()),
    );
    let database = Database::open(path)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet SQLite CRUD on http://{} using {}",
        listener.local_addr()?,
        database.path().display()
    );
    axum::serve(listener, app(database))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
