use std::path::PathBuf;

use yetlet_modular_shop::{
    Database, DatabaseOptions, DemoIdentities, ShopModules, app, listen_address,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = database_options(std::env::var("YETLET_SHOP_CATALOG_CACHE_ENABLED"))?;
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
    let database = Database::open_with_options(database_path, options)?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Yetlet modular shop on http://{}", listener.local_addr()?);
    axum::serve(listener, app(address, database, identities, modules)?)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

fn database_options(value: Result<String, std::env::VarError>) -> std::io::Result<DatabaseOptions> {
    Ok(DatabaseOptions {
        catalog_cache: catalog_cache_enabled(value)?,
    })
}

fn catalog_cache_enabled(value: Result<String, std::env::VarError>) -> std::io::Result<bool> {
    let invalid = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "shop catalog-cache configuration invalid",
        )
    };
    let enabled = match value {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "false" => false,
        Ok(value) if value == "true" => true,
        _ => return Err(invalid()),
    };
    if enabled && !cfg!(feature = "catalog-cache") {
        return Err(invalid());
    }
    Ok(enabled)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opt_in_options_map_native_startup_without_settings_loader() {
        assert!(
            !database_options(Err(std::env::VarError::NotPresent))
                .unwrap()
                .catalog_cache
        );
        assert!(!database_options(Ok("false".into())).unwrap().catalog_cache);
        let selected = database_options(Ok("true".into()));
        assert_eq!(
            selected.as_ref().ok().map(|options| options.catalog_cache),
            cfg!(feature = "catalog-cache").then_some(true)
        );
        if let Err(error) = selected {
            assert_eq!(
                error.to_string(),
                "shop catalog-cache configuration invalid"
            );
        }
        assert!(database_options(Ok("PRIVATE_INVALID".into())).is_err());
    }

    #[test]
    fn catalog_cache_selection_is_explicit_default_off_and_redacted() {
        assert!(!catalog_cache_enabled(Err(std::env::VarError::NotPresent)).unwrap());
        assert!(!catalog_cache_enabled(Ok("false".into())).unwrap());
        assert_eq!(
            catalog_cache_enabled(Ok("true".into())).is_ok(),
            cfg!(feature = "catalog-cache")
        );
        assert_eq!(
            catalog_cache_enabled(Ok("PRIVATE_INVALID".into()))
                .unwrap_err()
                .to_string(),
            "shop catalog-cache configuration invalid"
        );
        assert!(
            catalog_cache_enabled(Err(std::env::VarError::NotUnicode(
                std::ffi::OsString::from("PRIVATE_INVALID")
            )))
            .is_err()
        );
    }
}
