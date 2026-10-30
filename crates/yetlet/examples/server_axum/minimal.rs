//! Minimal SSR/JSON application. No optional provider or Controller needed.
use yetlet::server_axum::{App, response::Json};
use yetlet::view::View;

async fn home() -> View {
    View::text("Hello from Yetlet")
}
async fn status() -> Json<&'static str> {
    Json("ready")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app: axum::Router = App::new()
        .page("/", "Yetlet", home)
        .group("/api", |api| api.get("/status", status))
        .into_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
