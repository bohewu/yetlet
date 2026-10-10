use yetlet_file_server::{FileStore, app, listen_address, new_token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = listen_address(
        &std::env::var("YETLET_FILES_LISTEN").unwrap_or_else(|_| "127.0.0.1:3006".to_owned()),
    )?;
    let token = match std::env::var("YETLET_FILES_TOKEN") {
        Ok(token) => token,
        Err(std::env::VarError::NotPresent) => {
            let token = new_token()?;
            eprintln!("Development token (local terminal only): {token}");
            token
        }
        Err(error) => return Err(error.into()),
    };
    let store = FileStore::new(token)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Yetlet file server on http://{}", listener.local_addr()?);
    axum::serve(listener, app(store))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
