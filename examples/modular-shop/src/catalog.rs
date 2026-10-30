use axum::extract::State;
use yetlet::server_axum::{App, Page};
use yetlet::view::{Element, View};

use crate::{AppState, Failure, money};

async fn page(State(state): State<AppState>) -> Result<Page, Failure> {
    let products = state
        .database
        .catalog()
        .await
        .map_err(|_| Failure::Database)?;
    let mut list = Element::new("ul");
    for product in products {
        list = list.child(
            Element::new("li")
                .attribute("data-product-id", &product.id)
                .child(Element::new("strong").child(View::text(&product.name)))
                .child(View::text(format!(
                    " — {} — {} in stock",
                    money(product.price_cents),
                    product.stock
                ))),
        );
    }

    let mut navigation = Element::new("nav");
    if state.modules.identity {
        navigation = navigation.child(
            Element::new("a")
                .attribute("href", "/identity/login")
                .child(View::text("Sign in")),
        );
    }
    if state.modules.cart {
        navigation = navigation.child(View::text(" | ")).child(
            Element::new("a")
                .attribute("href", "/shop/cart")
                .child(View::text("Cart")),
        );
    }
    if state.modules.orders {
        navigation = navigation.child(View::text(" | ")).child(
            Element::new("a")
                .attribute("href", "/shop/orders")
                .child(View::text("Orders")),
        );
    }

    Ok(Page::new(
        "Yetlet | Modular Shop Catalog",
        Element::new("main")
            .attribute("lang", "en")
            .child(Element::new("h1").child(View::text("Modular Shop Catalog")))
            .child(Element::new("p").child(View::text(
                if state.database.catalog_cache_enabled() {
                    "Displayed price and stock may be up to two seconds old. Cart actions and checkout check current data."
                } else {
                    "Catalog, identity, cart and orders are ordinary separately installed Rust modules."
                },
            )))
            .child(list)
            .child(navigation),
    ))
}

pub(crate) fn module() -> App<AppState> {
    App::<AppState>::new().get("/shop/catalog", page)
}
