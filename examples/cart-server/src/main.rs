use std::net::SocketAddr;
use yetlet::server_axum::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_CART_LISTEN").unwrap_or_else(|_| "127.0.0.1:3002".to_owned());
    let address: SocketAddr = listen.parse()?;
    let app = yetlet_cart_server::app(address)?;
    if address.port() == 0 {
        // Native listener ownership exposes the OS-selected port.
        let listener = tokio::net::TcpListener::bind(address).await?;
        let actual = listener.local_addr()?;
        println!("Yetlet cart sample on http://{actual}");
        axum::serve(listener, app).await?;
    } else {
        println!("Yetlet cart sample on http://{address}");
        App::from_router(app).serve_on(address).await?;
    }
    Ok(())
}
