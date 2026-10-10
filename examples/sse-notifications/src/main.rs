use yetlet_sse_notifications::{Hub, app, listen_address};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = listen_address(
        &std::env::var("YETLET_SSE_LISTEN").unwrap_or_else(|_| "127.0.0.1:3008".to_owned()),
    )?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet SSE notifications on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app(Hub::new()))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
