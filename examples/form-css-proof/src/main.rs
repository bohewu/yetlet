use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("YETLET_FORM_PROOF_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:3011".to_owned())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("this stateless proof must bind loopback".into());
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet form proof: http://{}/proof/native",
        listener.local_addr()?
    );
    axum::serve(listener, yetlet_form_css_proof::app())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
