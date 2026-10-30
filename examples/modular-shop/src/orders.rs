use axum::{
    Form,
    extract::{DefaultBodyLimit, Extension, Path, State, rejection::FormRejection},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use yetlet::server_axum::{App, Page};
use yetlet::session::{Session, SessionManager, SessionUpdateError};
use yetlet::view::{Element, View};

use crate::{
    AppState, FORM_LIMIT, Failure, ShopSession, current_user,
    database::{DatabaseError, Order},
    identity::logout_form,
    map_session, money, parse_form, require_csrf,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckoutForm {
    csrf: String,
    expected_revision: u64,
}

#[derive(Debug, Clone, Copy)]
enum ClearAfterOrder {
    Changed,
}

fn map_database(error: DatabaseError) -> Failure {
    match error {
        DatabaseError::NotFound => Failure::NotFound,
        DatabaseError::Forbidden => Failure::Forbidden,
        DatabaseError::EmptyCart
        | DatabaseError::InvalidQuantity
        | DatabaseError::UnknownProduct => Failure::InvalidCart,
        DatabaseError::OutOfStock => Failure::OutOfStock,
        DatabaseError::Sqlite(_)
        | DatabaseError::Io(_)
        | DatabaseError::UnsupportedSchema(_)
        | DatabaseError::SchemaMismatch
        | DatabaseError::Overflow
        | DatabaseError::WorkerUnavailable => Failure::Database,
    }
}

fn order_link(order: &Order) -> View {
    Element::new("li")
        .attribute("data-order-id", order.id.to_string())
        .child(
            Element::new("a")
                .attribute("href", format!("/shop/orders/{}", order.id))
                .child(View::text(format!(
                    "Order #{} — {}",
                    order.id,
                    money(order.total_cents)
                ))),
        )
        .into()
}

fn list_view(user: crate::UserId, orders: &[Order], csrf: &str) -> View {
    let mut list = Element::new("ul");
    for order in orders {
        list = list.child(order_link(order));
    }
    if orders.is_empty() {
        list = list.child(Element::new("li").child(View::text("No orders yet.")));
    }
    Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text(format!("{}'s orders", user.display()))))
        .child(Element::new("p").child(View::text(
            "Order ownership is checked on every request; IDs alone never authorize access.",
        )))
        .child(list)
        .child(
            Element::new("a")
                .attribute("href", "/shop/cart")
                .child(View::text("Back to cart")),
        )
        .child(logout_form(csrf))
        .into()
}

fn detail_view(order: &Order, csrf: &str) -> View {
    let mut lines = Element::new("ul");
    for line in &order.lines {
        lines = lines.child(
            Element::new("li")
                .attribute("data-product-id", &line.product_id)
                .child(View::text(format!(
                    "{} × {} — {}",
                    line.product_name,
                    line.quantity,
                    money(line.price_cents.saturating_mul(u32::from(line.quantity)))
                ))),
        );
    }
    Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text(format!("Order #{}", order.id))))
        .child(View::text(format!("Owner: {}", order.owner.display())))
        .child(lines)
        .child(Element::new("p").child(View::text(format!("Total: {}", money(order.total_cents)))))
        .child(
            Element::new("a")
                .attribute("href", "/shop/orders")
                .child(View::text("Back to orders")),
        )
        .child(logout_form(csrf))
        .into()
}

async fn list(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
) -> Result<Page, Failure> {
    let user = current_user(&session)?;
    let csrf = session.csrf_token().map_err(map_session)?;
    let orders = state
        .database
        .orders_for(user)
        .await
        .map_err(map_database)?;
    Ok(Page::new(
        "Yetlet | Modular Shop Orders",
        list_view(user, &orders, &csrf),
    ))
}

async fn detail(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Page, Failure> {
    let user = current_user(&session)?;
    let csrf = session.csrf_token().map_err(map_session)?;
    let order = state
        .database
        .order_for(user, id)
        .await
        .map_err(map_database)?;
    Ok(Page::new(
        "Yetlet | Modular Shop Order",
        detail_view(&order, &csrf),
    ))
}

async fn checkout(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
    form: Result<Form<CheckoutForm>, FormRejection>,
) -> Response {
    let result = async {
        let user = current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let snapshot = session.read().map_err(map_session)?;
        if snapshot.user != Some(user) || snapshot.cart.revision != form.expected_revision {
            return Err(Failure::Conflict);
        }
        if snapshot.cart.lines.is_empty() {
            return Err(Failure::InvalidCart);
        }
        // The Session CSRF identity plus cart revision is stable for this cart
        // snapshot. Retrying after a database commit but before cart cleanup
        // returns the same order instead of decrementing stock twice.
        let checkout_key = format!(
            "{}:{}",
            session.csrf_token().map_err(map_session)?,
            snapshot.cart.revision
        );
        let order = state
            .database
            .place_order(user, snapshot.cart.lines.clone(), checkout_key)
            .await
            .map_err(map_database)?;

        match session.try_update(|shop| {
            if shop.user != Some(user)
                || shop.cart.revision != snapshot.cart.revision
                || shop.cart.lines != snapshot.cart.lines
            {
                return Err(ClearAfterOrder::Changed);
            }
            shop.cart.lines.clear();
            shop.cart.revision = shop
                .cart
                .revision
                .checked_add(1)
                .ok_or(ClearAfterOrder::Changed)?;
            Ok(())
        }) {
            Ok(()) | Err(SessionUpdateError::Application(ClearAfterOrder::Changed)) => {}
            Err(SessionUpdateError::Session(error)) => return Err(map_session(error)),
        }
        Ok::<_, Failure>(Redirect::to(&format!("/shop/orders/{}", order.id)).into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

pub(crate) fn module(sessions: SessionManager<ShopSession>) -> App<AppState> {
    App::<AppState>::new()
        .get("/shop/orders", list)
        .get("/shop/orders/{id}", detail)
        .post("/shop/orders/checkout", checkout)
        .route_layer(DefaultBodyLimit::max(FORM_LIMIT))
        .configure(move |app| sessions.scoped(app))
}
