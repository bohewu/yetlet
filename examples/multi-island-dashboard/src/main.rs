use std::path::PathBuf;
use yetlet_multi_island_dashboard::{app, listen_address};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = listen_address(
        &std::env::var("YETLET_DASHBOARD_LISTEN").unwrap_or_else(|_| "127.0.0.1:3009".to_owned()),
    )?;
    let asset_root = PathBuf::from(
        std::env::var("YETLET_DASHBOARD_ASSET_ROOT")
            .unwrap_or_else(|_| "target/multi-island-dashboard-dist".to_owned()),
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet multi-island dashboard on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app(asset_root))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
