#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address =
        std::env::var("YETLET_PHYSICS_LISTEN").unwrap_or_else(|_| "127.0.0.1:3016".into());
    let address: std::net::SocketAddr = address.parse()?;
    if !address.ip().is_loopback() {
        return Err("loopback listener required".into());
    }
    let root = std::env::var_os("YETLET_PHYSICS_ASSETS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/dist")
        });
    let app = yetlet_physics_lab::server::app(root);
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("READY http://{}/physics-lab", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {}
