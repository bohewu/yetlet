mod config;

use axum::extract::{Path, State};
use std::sync::Arc;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};

async fn home(State(settings): State<Arc<config::Settings>>) -> Page {
    // Explicitly select public data only; Settings has no Serialize or Debug.
    Page::new("Yetlet", View::text(&settings.service.public_message))
}

async fn hello(Path(name): Path<String>) -> Page {
    Page::new(
        "Hello",
        Element::new("main").child(View::text(format!("Hello, {name}!"))),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::load()?;
    let address = settings.validate(false)?;
    let app: axum::Router = App::new()
        .get("/", home)
        .get("/hello/{name}", hello)
        .with_state(Arc::new(settings))
        .into_router();
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
