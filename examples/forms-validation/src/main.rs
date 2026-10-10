use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_FORMS_LISTEN").unwrap_or_else(|_| "127.0.0.1:3005".to_owned());
    let address: SocketAddr = listen.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet forms validation sample on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, yetlet_forms_validation::app()).await?;
    Ok(())
}
