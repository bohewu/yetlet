use std::net::SocketAddr;
use yetlet::server_axum::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_REST_LISTEN").unwrap_or_else(|_| "127.0.0.1:3001".to_owned());
    let address: SocketAddr = listen.parse()?;
    if address.port() == 0 {
        // Native listener ownership exposes the OS-selected port.
        let listener = tokio::net::TcpListener::bind(address).await?;
        let actual = listener.local_addr()?;
        println!("Yetlet REST sample on http://{actual}");
        axum::serve(listener, yetlet_rest_api::app()).await?;
    } else {
        println!("Yetlet REST sample on http://{address}");
        App::from_router(yetlet_rest_api::app())
            .serve_on(address)
            .await?;
    }
    Ok(())
}
