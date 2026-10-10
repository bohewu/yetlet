use std::path::PathBuf;

use yetlet_modular_shop::{Database, DemoIdentities, ShopModules, app, listen_address};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = listen_address(
        &std::env::var("YETLET_SHOP_LISTEN").unwrap_or_else(|_| "127.0.0.1:3008".to_owned()),
    )?;
    let database_path = PathBuf::from(
        std::env::var("YETLET_SHOP_DB")
            .unwrap_or_else(|_| "target/yetlet-modular-shop.sqlite3".to_owned()),
    );
    let identities = match (
        std::env::var("YETLET_SHOP_ALICE_CODE"),
        std::env::var("YETLET_SHOP_BOB_CODE"),
    ) {
        (Ok(alice), Ok(bob)) => DemoIdentities::from_codes(&alice, &bob)?,
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => {
            let identities = DemoIdentities::generate()?;
            let (alice, bob) = identities.terminal_codes();
            eprintln!("Synthetic Alice code (local terminal only): {alice}");
            eprintln!("Synthetic Bob code (local terminal only): {bob}");
            identities
        }
        _ => return Err("set both YETLET_SHOP_ALICE_CODE and YETLET_SHOP_BOB_CODE".into()),
    };
    let mut modules = ShopModules::default();
    if std::env::var("YETLET_SHOP_DISABLE_ORDERS").as_deref() == Ok("1") {
        modules.orders = false;
    }
    let database = Database::open(database_path)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Yetlet modular shop on http://{}", listener.local_addr()?);
    axum::serve(listener, app(address, database, identities, modules)?)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
