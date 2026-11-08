#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_SHOWCASE_LISTEN").unwrap_or_else(|_| "127.0.0.1:3011".into());
    let dist = std::env::var_os("YETLET_SHOWCASE_DIST")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "target/style-showcase-dist".into());
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    println!(
        "Yetlet Style Showcase on http://{}/showcase",
        listener.local_addr()?
    );
    axum::serve(listener, yetlet_style_showcase::native::app(&dist)).await?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {}
