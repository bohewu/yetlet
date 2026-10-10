use std::net::SocketAddr;
use yetlet_auth_notes::{DemoIdentities, app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen =
        std::env::var("YETLET_AUTH_LISTEN").unwrap_or_else(|_| "127.0.0.1:3010".to_owned());
    let address: SocketAddr = listen.parse()?;
    let identities = match (
        std::env::var("YETLET_AUTH_ALICE_CODE"),
        std::env::var("YETLET_AUTH_BOB_CODE"),
    ) {
        (Ok(alice), Ok(bob)) => DemoIdentities::new(alice, bob)?,
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => {
            let identities = DemoIdentities::generate()?;
            eprintln!(
                "Synthetic Alice login code (local terminal only): {}",
                identities.alice_code()
            );
            eprintln!(
                "Synthetic Bob login code (local terminal only): {}",
                identities.bob_code()
            );
            identities
        }
        _ => {
            return Err(
                "set both YETLET_AUTH_ALICE_CODE and YETLET_AUTH_BOB_CODE, or neither".into(),
            );
        }
    };
    let application = app(address, identities)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "Yetlet Auth Notes sample on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, application)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
