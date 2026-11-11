use std::net::SocketAddr;
use yetlet::server_axum::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_FORMS_LISTEN").unwrap_or_else(|_| "127.0.0.1:3005".to_owned());
    let address: SocketAddr = listen.parse()?;
    if address.port() == 0 {
        // Native listener ownership exposes the OS-selected port.
        let listener = tokio::net::TcpListener::bind(address).await?;
        println!(
            "Yetlet forms validation sample on http://{}",
            listener.local_addr()?
        );
        axum::serve(listener, yetlet_forms_validation::app()).await?;
    } else {
        println!("Yetlet forms validation sample on http://{address}");
        App::from_router(yetlet_forms_validation::app())
            .serve_on(address)
            .await?;
    }
    Ok(())
}
