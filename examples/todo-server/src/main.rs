use std::net::SocketAddr;
use yetlet::server_axum::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen = std::env::var("YETLET_LISTEN").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let address: SocketAddr = listen.parse()?;
    if address.port() == 0 {
        // Native listener ownership exposes the OS-selected port.
        let listener = tokio::net::TcpListener::bind(address).await?;
        println!("Yetlet SSR demo on http://{}", listener.local_addr()?);
        axum::serve(listener, yetlet_todo_server::app()).await?;
    } else {
        println!("Yetlet SSR demo on http://{address}");
        App::from_router(yetlet_todo_server::app())
            .serve_on(address)
            .await?;
    }
    Ok(())
}
