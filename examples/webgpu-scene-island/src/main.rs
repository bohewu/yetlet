#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use yetlet_webgpu_scene_island::server::{app, listen_address};
    let address = listen_address(
        &std::env::var("YETLET_SCENE_LISTEN").unwrap_or_else(|_| "127.0.0.1:3010".into()),
    )?;
    let root = std::env::var("YETLET_SCENE_ASSET_ROOT")
        .unwrap_or_else(|_| "target/webgpu-scene-island-dist".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Synthetic work overview: http://{}/scene",
        listener.local_addr()?
    );
    axum::serve(listener, app(root.into()))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {}
