use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen = std::env::var("YETLET_LISTEN").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let address: SocketAddr = listen.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Yetlet SSR demo on http://{address}");
    axum::serve(listener, yetlet_todo_server::app()).await?;
    Ok(())
}
