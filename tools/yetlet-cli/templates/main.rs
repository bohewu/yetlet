use axum::extract::Path;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};

async fn home() -> View {
    Element::new("main")
        .child(Element::new("h1").child(View::text("Hello <Yetlet> & Rust!")))
        .into()
}

async fn hello(Path(name): Path<String>) -> Page {
    Page::new("Hello", Element::new("main").child(View::text(format!("Hello, {name}!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("YETLET_PORT").unwrap_or_else(|_| "3000".into()).parse()?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    let app: axum::Router = App::new()
        .page("/", "Yetlet", home)
        .get("/hello/{name}", hello)
        .into_router();
    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
