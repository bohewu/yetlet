//! Minimal SSR/JSON application. No optional provider or Controller needed.
use yetlet::server_axum::prelude::*;

async fn home() -> View {
    View::text("Hello from Yetlet")
}
async fn status() -> Json<&'static str> {
    Json("ready")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .page("/", "Yetlet", home)
        .group("/api", |api| api.get("/status", status))
        .serve()
        .await?;
    Ok(())
}
