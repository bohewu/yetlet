use std::collections::BTreeMap;

use axum::{
    Form,
    extract::{DefaultBodyLimit, Extension, State, rejection::FormRejection},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};
use yetlet_session::{Session, SessionManager, SessionUpdateError};

use crate::{
    AppState, FORM_LIMIT, Failure, ShopSession, current_user, database::Product, hidden,
    identity::logout_form, map_session, money, parse_form, require_csrf,
};

const MAX_CART_LINES: usize = 8;
const MAX_QUANTITY: u8 = 20;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductForm {
    csrf: String,
    product_id: String,
    expected_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionForm {
    csrf: String,
    expected_revision: u64,
}

#[derive(Debug, Clone, Copy)]
enum CartMutationError {
    Conflict,
    Capacity,
    Quantity,
    Identity,
}

fn map_update(error: SessionUpdateError<CartMutationError>) -> Failure {
    match error {
        SessionUpdateError::Session(error) => map_session(error),
        SessionUpdateError::Application(CartMutationError::Conflict) => Failure::Conflict,
        SessionUpdateError::Application(
            CartMutationError::Capacity | CartMutationError::Quantity,
        ) => Failure::InvalidCart,
        SessionUpdateError::Application(CartMutationError::Identity) => Failure::Unauthenticated,
    }
}

fn product_form(
    action: &'static str,
    csrf: &str,
    product_id: &str,
    revision: u64,
    label: &'static str,
) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", action)
        .child(hidden("csrf", csrf))
        .child(hidden("product_id", product_id))
        .child(hidden("expected_revision", revision.to_string()))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text(label)),
        )
        .into()
}

fn page_view(
    user: crate::UserId,
    products: Vec<Product>,
    lines: &BTreeMap<String, u8>,
    revision: u64,
    csrf: &str,
    orders_enabled: bool,
    catalog_cached: bool,
) -> View {
    let by_id: BTreeMap<_, _> = products
        .iter()
        .map(|product| (product.id.clone(), product.clone()))
        .collect();
    let mut available = Element::new("ul");
    for product in &products {
        available = available.child(
            Element::new("li")
                .attribute("data-product-id", &product.id)
                .child(View::text(format!(
                    "{} — {} — {} in stock ",
                    product.name,
                    money(product.price_cents),
                    product.stock
                )))
                .child(product_form(
                    "/shop/cart/add",
                    csrf,
                    &product.id,
                    revision,
                    "Add",
                )),
        );
    }

    let mut cart = Element::new("ul");
    let mut total = 0_u32;
    for (product_id, quantity) in lines {
        if let Some(product) = by_id.get(product_id) {
            total = total.saturating_add(product.price_cents.saturating_mul(u32::from(*quantity)));
            cart = cart.child(
                Element::new("li")
                    .attribute("data-cart-product", product_id)
                    .child(View::text(format!(
                        "{} × {} — {} ",
                        product.name,
                        quantity,
                        money(product.price_cents.saturating_mul(u32::from(*quantity)))
                    )))
                    .child(product_form(
                        "/shop/cart/decrement",
                        csrf,
                        product_id,
                        revision,
                        "Remove one",
                    )),
            );
        }
    }
    if lines.is_empty() {
        cart = cart.child(Element::new("li").child(View::text("Cart is empty.")));
    }

    let checkout = Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/shop/orders/checkout")
        .child(hidden("csrf", csrf))
        .child(hidden("expected_revision", revision.to_string()))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text("Place transactional mock order")),
        );
    let clear = Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/shop/cart/clear")
        .child(hidden("csrf", csrf))
        .child(hidden("expected_revision", revision.to_string()))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text("Clear cart")),
        );

    let order_navigation: View = if orders_enabled {
        Element::new("span")
            .child(View::text(" | "))
            .child(
                Element::new("a")
                    .attribute("href", "/shop/orders")
                    .child(View::text("Orders")),
            )
            .into()
    } else {
        View::text("")
    };
    let checkout_view: View = if orders_enabled {
        checkout.into()
    } else {
        Element::new("p")
            .attribute("data-orders-disabled", "true")
            .child(View::text(
                "Orders module is disabled for this application build.",
            ))
            .into()
    };

    let mut page = Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text(format!("{}'s modular cart", user.display()))))
        .child(
            Element::new("a")
                .attribute("href", "/shop/catalog")
                .child(View::text("Catalog")),
        )
        .child(order_navigation)
        .child(Element::new("h2").child(View::text("Available products")))
        .child(available)
        .child(Element::new("h2").child(View::text("Cart")))
        .child(cart)
        .child(Element::new("p").child(View::text(format!(
            "{}: {}",
            if catalog_cached {
                "Estimated total"
            } else {
                "Total"
            },
            money(total)
        ))))
        .child(checkout_view)
        .child(clear)
        .child(logout_form(csrf));
    if catalog_cached {
        page = page.child(Element::new("p").child(View::text(
            "Displayed price and stock may be up to two seconds old. Checkout checks current price and stock."
        )));
    }
    page.into()
}

async fn page(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
) -> Result<Page, Failure> {
    let user = current_user(&session)?;
    let snapshot = session.read().map_err(map_session)?;
    let csrf = session.csrf_token().map_err(map_session)?;
    let products = state
        .database
        .catalog()
        .await
        .map_err(|_| Failure::Database)?;
    Ok(Page::new(
        "Yetlet | Modular Shop Cart",
        page_view(
            user,
            products,
            &snapshot.cart.lines,
            snapshot.cart.revision,
            &csrf,
            state.modules.orders,
            state.database.catalog_cache_enabled(),
        ),
    ))
}

async fn add(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
    form: Result<Form<ProductForm>, FormRejection>,
) -> Response {
    let result = async {
        let user = current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let product = state
            .database
            .product(form.product_id.clone())
            .await
            .map_err(|_| Failure::InvalidCart)?;
        if product.stock == 0 {
            return Err(Failure::OutOfStock);
        }
        session
            .try_update(|shop| {
                if shop.user != Some(user) {
                    return Err(CartMutationError::Identity);
                }
                if shop.cart.revision != form.expected_revision {
                    return Err(CartMutationError::Conflict);
                }
                let new_line = !shop.cart.lines.contains_key(&form.product_id);
                if new_line && shop.cart.lines.len() >= MAX_CART_LINES {
                    return Err(CartMutationError::Capacity);
                }
                let quantity = shop.cart.lines.entry(form.product_id).or_insert(0);
                if *quantity >= MAX_QUANTITY {
                    return Err(CartMutationError::Quantity);
                }
                *quantity += 1;
                shop.cart.revision = shop
                    .cart
                    .revision
                    .checked_add(1)
                    .ok_or(CartMutationError::Conflict)?;
                Ok(())
            })
            .map_err(map_update)?;
        Ok::<_, Failure>(Redirect::to("/shop/cart").into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn decrement(
    Extension(session): Extension<Session<ShopSession>>,
    form: Result<Form<ProductForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session
            .try_update(|shop| {
                if shop.user != Some(user) {
                    return Err(CartMutationError::Identity);
                }
                if shop.cart.revision != form.expected_revision {
                    return Err(CartMutationError::Conflict);
                }
                let quantity = shop
                    .cart
                    .lines
                    .get_mut(&form.product_id)
                    .ok_or(CartMutationError::Quantity)?;
                if *quantity == 1 {
                    shop.cart.lines.remove(&form.product_id);
                } else {
                    *quantity -= 1;
                }
                shop.cart.revision = shop
                    .cart
                    .revision
                    .checked_add(1)
                    .ok_or(CartMutationError::Conflict)?;
                Ok(())
            })
            .map_err(map_update)?;
        Ok::<_, Failure>(Redirect::to("/shop/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn clear(
    Extension(session): Extension<Session<ShopSession>>,
    form: Result<Form<RevisionForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session
            .try_update(|shop| {
                if shop.user != Some(user) {
                    return Err(CartMutationError::Identity);
                }
                if shop.cart.revision != form.expected_revision {
                    return Err(CartMutationError::Conflict);
                }
                shop.cart.lines.clear();
                shop.cart.revision = shop
                    .cart
                    .revision
                    .checked_add(1)
                    .ok_or(CartMutationError::Conflict)?;
                Ok(())
            })
            .map_err(map_update)?;
        Ok::<_, Failure>(Redirect::to("/shop/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

pub(crate) fn module(sessions: SessionManager<ShopSession>) -> App<AppState> {
    App::<AppState>::new()
        .get("/shop/cart", page)
        .post("/shop/cart/add", add)
        .post("/shop/cart/decrement", decrement)
        .post("/shop/cart/clear", clear)
        .route_layer(DefaultBodyLimit::max(FORM_LIMIT))
        .configure(move |app| sessions.scoped(app))
}
