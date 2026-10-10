use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_HELLO_LISTEN").unwrap_or_else(|_| "127.0.0.1:3003".to_owned());
    let address: SocketAddr = listen.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet hello-world sample on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, yetlet_hello_world::app()).await?;
    Ok(())
}
