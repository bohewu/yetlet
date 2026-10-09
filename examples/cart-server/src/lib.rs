//! Server-rendered shopping cart with explicitly installed sessions.
//!
//! The application has no browser runtime. Forms use native HTML, a session-
//! bound CSRF token and redirect-after-POST semantics.

use std::{collections::BTreeMap, fmt, net::SocketAddr, time::Duration};

use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, Extension, rejection::FormRejection},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};
use yetlet_session::{
    Session, SessionConfig, SessionConfigError, SessionError, SessionManager, SessionUpdateError,
};

const MAX_FORM_BODY: usize = 2 * 1024;
const MAX_LINES: usize = 3;
const MAX_QUANTITY: u8 = 20;
const SESSION_TTL: Duration = Duration::from_secs(30 * 60);
const SESSION_CAPACITY: usize = 1_024;

#[derive(Debug)]
pub enum CartAppError {
    NonLoopbackListener,
    Session(SessionConfigError),
}

impl fmt::Display for CartAppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonLoopbackListener => {
                formatter.write_str("the insecure localhost cart must bind a loopback address")
            }
            Self::Session(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CartAppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NonLoopbackListener => None,
            Self::Session(error) => Some(error),
        }
    }
}

impl From<SessionConfigError> for CartAppError {
    fn from(error: SessionConfigError) -> Self {
        Self::Session(error)
    }
}

#[derive(Debug, Clone, Copy)]
struct Product {
    id: &'static str,
    name: &'static str,
    price_cents: u32,
}

const PRODUCTS: [Product; 4] = [
    Product {
        id: "rust-book",
        name: "The Rust Programming Language",
        price_cents: 3_999,
    },
    Product {
        id: "web-handbook",
        name: "Small Web Systems Handbook",
        price_cents: 2_499,
    },
    Product {
        id: "mechanical-keyboard",
        name: "Compact Mechanical Keyboard",
        price_cents: 8_900,
    },
    Product {
        id: "travel-mug",
        name: "Stainless Travel Mug",
        price_cents: 1_250,
    },
];

fn product(id: &str) -> Option<Product> {
    PRODUCTS.iter().copied().find(|product| product.id == id)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cart {
    lines: BTreeMap<String, u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CartError {
    MissingLine,
    LineLimit,
    QuantityLimit,
}

impl Cart {
    fn add(&mut self, product: Product) -> Result<(), CartError> {
        if let Some(quantity) = self.lines.get_mut(product.id) {
            if *quantity >= MAX_QUANTITY {
                return Err(CartError::QuantityLimit);
            }
            *quantity += 1;
            return Ok(());
        }
        if self.lines.len() >= MAX_LINES {
            return Err(CartError::LineLimit);
        }
        self.lines.insert(product.id.to_owned(), 1);
        Ok(())
    }

    fn decrement(&mut self, product: Product) -> Result<(), CartError> {
        let Some(quantity) = self.lines.get_mut(product.id) else {
            return Err(CartError::MissingLine);
        };
        if *quantity == 1 {
            self.lines.remove(product.id);
        } else {
            *quantity -= 1;
        }
        Ok(())
    }

    fn remove(&mut self, product: Product) -> Result<(), CartError> {
        if self.lines.remove(product.id).is_none() {
            return Err(CartError::MissingLine);
        }
        Ok(())
    }

    fn total_cents(&self) -> u32 {
        self.lines
            .iter()
            .filter_map(|(id, quantity)| {
                product(id).map(|product| product.price_cents * u32::from(*quantity))
            })
            .sum()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductForm {
    csrf: String,
    product: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CsrfForm {
    csrf: String,
}

#[derive(Debug, Clone, Copy)]
enum ActionFailure {
    InvalidForm,
    InvalidCsrf,
    UnknownProduct,
    MissingLine,
    LineLimit,
    QuantityLimit,
    SessionUnavailable,
}

impl IntoResponse for ActionFailure {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidForm => (StatusCode::BAD_REQUEST, "Invalid cart form"),
            Self::InvalidCsrf => (StatusCode::FORBIDDEN, "Invalid CSRF token"),
            Self::UnknownProduct => (StatusCode::BAD_REQUEST, "Unknown product"),
            Self::MissingLine => (StatusCode::NOT_FOUND, "Cart line does not exist"),
            Self::LineLimit => (StatusCode::CONFLICT, "Cart line limit reached"),
            Self::QuantityLimit => (StatusCode::CONFLICT, "Product quantity limit reached"),
            Self::SessionUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "Session unavailable"),
        };
        (status, message).into_response()
    }
}

fn map_session(error: SessionError) -> ActionFailure {
    match error {
        SessionError::Unavailable | SessionError::Expired | SessionError::Destroyed => {
            ActionFailure::SessionUnavailable
        }
    }
}

fn map_cart(error: CartError) -> ActionFailure {
    match error {
        CartError::MissingLine => ActionFailure::MissingLine,
        CartError::LineLimit => ActionFailure::LineLimit,
        CartError::QuantityLimit => ActionFailure::QuantityLimit,
    }
}

fn parse_form<T>(form: Result<Form<T>, FormRejection>) -> Result<T, ActionFailure> {
    form.map(|Form(value)| value)
        .map_err(|_| ActionFailure::InvalidForm)
}

fn require_csrf(session: &Session<Cart>, submitted: &str) -> Result<(), ActionFailure> {
    match session.verify_csrf(submitted).map_err(map_session)? {
        true => Ok(()),
        false => Err(ActionFailure::InvalidCsrf),
    }
}

fn update_cart<R>(
    session: &Session<Cart>,
    action: impl FnOnce(&mut Cart) -> Result<R, CartError>,
) -> Result<R, ActionFailure> {
    match session.try_update(action) {
        Ok(value) => Ok(value),
        Err(SessionUpdateError::Session(error)) => Err(map_session(error)),
        Err(SessionUpdateError::Application(error)) => Err(map_cart(error)),
    }
}

fn currency(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn hidden(name: &'static str, value: impl Into<String>) -> View {
    Element::new("input")
        .attribute("type", "hidden")
        .attribute("name", name)
        .attribute("value", value)
        .into()
}

fn product_form(action: &'static str, csrf: &str, product: Product, label: &str) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", action)
        .child(hidden("csrf", csrf))
        .child(hidden("product", product.id))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text(label)),
        )
        .into()
}

fn csrf_form(action: &'static str, csrf: &str, label: &str) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", action)
        .child(hidden("csrf", csrf))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text(label)),
        )
        .into()
}

fn cart_view(cart: &Cart, csrf: &str) -> View {
    let catalog = PRODUCTS.into_iter().map(|product| {
        Element::new("li")
            .attribute("data-product", product.id)
            .child(Element::new("h3").child(View::text(product.name)))
            .child(Element::new("p").child(View::text(currency(product.price_cents))))
            .child(product_form("/cart/add", csrf, product, "Add to cart"))
            .into()
    });

    let lines = cart.lines.iter().filter_map(|(id, quantity)| {
        let product = product(id)?;
        Some(
            Element::new("li")
                .attribute("data-cart-line", product.id)
                .child(Element::new("h3").child(View::text(product.name)))
                .child(Element::new("p").child(View::text(format!("Quantity: {quantity}"))))
                .child(Element::new("p").child(View::text(format!(
                    "Line total: {}",
                    currency(product.price_cents * u32::from(*quantity))
                ))))
                .child(product_form(
                    "/cart/decrement",
                    csrf,
                    product,
                    "Decrease quantity",
                ))
                .child(product_form("/cart/remove", csrf, product, "Remove line"))
                .into(),
        )
    });

    let cart_content: View = if cart.lines.is_empty() {
        Element::new("p")
            .attribute("data-cart-empty", "true")
            .child(View::text("Your cart is empty."))
            .into()
    } else {
        Element::new("div")
            .child(Element::new("ul").children(lines))
            .child(
                Element::new("p")
                    .attribute("data-cart-total", cart.total_cents().to_string())
                    .child(View::text(format!(
                        "Cart total: {}",
                        currency(cart.total_cents())
                    ))),
            )
            .child(csrf_form(
                "/cart/clear",
                csrf,
                "Clear cart and rotate session",
            ))
            .child(csrf_form("/cart/end", csrf, "End session"))
            .into()
    };

    Element::new("main")
        .child(Element::new("h1").child(View::text("Yetlet Session Cart")))
        .child(Element::new("p").child(View::text(
            "Server-rendered forms, bounded server-side state, and no browser runtime.",
        )))
        .child(
            Element::new("section")
                .attribute("aria-labelledby", "catalog-heading")
                .child(
                    Element::new("h2")
                        .attribute("id", "catalog-heading")
                        .child(View::text("Products")),
                )
                .child(Element::new("ul").children(catalog)),
        )
        .child(
            Element::new("section")
                .attribute("aria-labelledby", "cart-heading")
                .child(
                    Element::new("h2")
                        .attribute("id", "cart-heading")
                        .child(View::text("Cart")),
                )
                .child(cart_content),
        )
        .into()
}

async fn cart_page(Extension(session): Extension<Session<Cart>>) -> Result<Page, ActionFailure> {
    let cart = session.read().map_err(map_session)?;
    let csrf = session.csrf_token().map_err(map_session)?;
    Ok(Page::new("Yetlet | Session Cart", cart_view(&cart, &csrf)))
}

async fn add(
    Extension(session): Extension<Session<Cart>>,
    form: Result<Form<ProductForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let product = product(&form.product).ok_or(ActionFailure::UnknownProduct)?;
        update_cart(&session, |cart| cart.add(product))?;
        Ok::<_, ActionFailure>(Redirect::to("/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn decrement(
    Extension(session): Extension<Session<Cart>>,
    form: Result<Form<ProductForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let product = product(&form.product).ok_or(ActionFailure::UnknownProduct)?;
        update_cart(&session, |cart| cart.decrement(product))?;
        Ok::<_, ActionFailure>(Redirect::to("/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn remove(
    Extension(session): Extension<Session<Cart>>,
    form: Result<Form<ProductForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let product = product(&form.product).ok_or(ActionFailure::UnknownProduct)?;
        update_cart(&session, |cart| cart.remove(product))?;
        Ok::<_, ActionFailure>(Redirect::to("/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn clear(
    Extension(session): Extension<Session<Cart>>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session
            .replace_and_rotate(Cart::default())
            .map_err(map_session)?;
        Ok::<_, ActionFailure>(Redirect::to("/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn end_session(
    Extension(session): Extension<Session<Cart>>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session.destroy().map_err(map_session)?;
        Ok::<_, ActionFailure>(Redirect::to("/cart").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

pub fn app(listener: SocketAddr) -> Result<Router, CartAppError> {
    if !listener.ip().is_loopback() {
        return Err(CartAppError::NonLoopbackListener);
    }
    let sessions = SessionManager::<Cart>::new(
        SessionConfig::new("yetlet-cart", SESSION_TTL, SESSION_CAPACITY)?.insecure_for_local_http(),
    );
    let cart_sessions = sessions.clone();
    Ok(App::new()
        .get("/", || async { Redirect::to("/cart") })
        .get("/healthz", || async { "ok" })
        .group("/cart", move |group| {
            group
                .get("/", cart_page)
                .post("/add", add)
                .post("/decrement", decrement)
                .post("/remove", remove)
                .post("/clear", clear)
                .post("/end", end_session)
                .route_layer(DefaultBodyLimit::max(MAX_FORM_BODY))
                .configure(move |group| cart_sessions.scoped(group))
        })
        .into_router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Method, Request, header},
    };
    use tower::ServiceExt;

    struct Observed {
        status: StatusCode,
        headers: HeaderMap,
        body: String,
    }

    async fn send(router: Router, request: Request<Body>) -> Observed {
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
        Observed {
            status,
            headers,
            body: String::from_utf8(body.to_vec()).unwrap(),
        }
    }

    fn get(path: &str, cookie: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method(Method::GET).uri(path);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn post(path: &str, cookie: &str, fields: &[(&str, &str)]) -> Request<Body> {
        let body = fields
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::COOKIE, cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap()
    }

    fn cookie_pair(headers: &HeaderMap) -> String {
        headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    fn csrf(html: &str) -> String {
        let marker = "name=\"csrf\" value=\"";
        let start = html.find(marker).unwrap() + marker.len();
        let rest = &html[start..];
        rest[..rest.find('"').unwrap()].to_owned()
    }

    fn test_app() -> Router {
        app("127.0.0.1:0".parse().unwrap()).unwrap()
    }

    #[test]
    fn insecure_demo_accepts_only_loopback_listeners() {
        assert!(app("127.0.0.1:3002".parse().unwrap()).is_ok());
        assert!(app("[::1]:3002".parse().unwrap()).is_ok());
        assert!(matches!(
            app("0.0.0.0:3002".parse().unwrap()),
            Err(CartAppError::NonLoopbackListener)
        ));
        assert!(matches!(
            app("192.0.2.10:3002".parse().unwrap()),
            Err(CartAppError::NonLoopbackListener)
        ));
    }

    #[test]
    fn cart_enforces_line_and_quantity_limits_without_partial_updates() {
        let mut cart = Cart::default();
        for product in PRODUCTS.into_iter().take(MAX_LINES) {
            cart.add(product).unwrap();
        }
        let before = cart.clone();
        assert_eq!(cart.add(PRODUCTS[MAX_LINES]), Err(CartError::LineLimit));
        assert_eq!(cart, before);

        let mut cart = Cart::default();
        for _ in 0..MAX_QUANTITY {
            cart.add(PRODUCTS[0]).unwrap();
        }
        let before = cart.clone();
        assert_eq!(cart.add(PRODUCTS[0]), Err(CartError::QuantityLimit));
        assert_eq!(cart, before);
    }

    #[tokio::test]
    async fn health_is_outside_session_scope_and_cart_is_strict_ssr() {
        let router = test_app();
        let health = send(router.clone(), get("/healthz", None)).await;
        assert_eq!(health.status, StatusCode::OK);
        assert!(!health.headers.contains_key(header::SET_COOKIE));

        let page = send(router.clone(), get("/cart", None)).await;
        assert_eq!(page.status, StatusCode::OK);
        assert!(page.body.contains("Yetlet Session Cart"));
        assert!(page.body.contains("Your cart is empty."));
        assert!(!page.body.contains("<script"));
        assert!(!page.body.contains("/app"));
        assert!(
            page.headers[header::CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("form-action 'self'")
        );
        let set_cookie = page.headers[header::SET_COOKIE].to_str().unwrap();
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Lax"));
        assert!(set_cookie.contains("Path=/"));
        assert!(!set_cookie.contains("Secure"));
    }

    #[tokio::test]
    async fn add_refresh_and_invalid_csrf_preserve_server_side_cart() {
        let router = test_app();
        let page = send(router.clone(), get("/cart", None)).await;
        let cookie = cookie_pair(&page.headers);
        let token = csrf(&page.body);

        let added = send(
            router.clone(),
            post(
                "/cart/add",
                &cookie,
                &[("csrf", &token), ("product", "rust-book")],
            ),
        )
        .await;
        assert_eq!(added.status, StatusCode::SEE_OTHER);
        assert_eq!(added.headers[header::LOCATION], "/cart");

        let refreshed = send(router.clone(), get("/cart", Some(&cookie))).await;
        assert!(refreshed.body.contains("Quantity: 1"));
        assert!(refreshed.body.contains("Cart total: $39.99"));

        let rejected = send(
            router.clone(),
            post(
                "/cart/add",
                &cookie,
                &[("csrf", "0"), ("product", "rust-book")],
            ),
        )
        .await;
        assert_eq!(rejected.status, StatusCode::FORBIDDEN);
        assert_eq!(rejected.body, "Invalid CSRF token");

        let unchanged = send(router, get("/cart", Some(&cookie))).await;
        assert!(unchanged.body.contains("Quantity: 1"));
        assert!(!unchanged.body.contains("Quantity: 2"));
    }

    #[tokio::test]
    async fn clear_rotates_identity_and_old_cookie_cannot_resume_cart() {
        let router = test_app();
        let page = send(router.clone(), get("/cart", None)).await;
        let old_cookie = cookie_pair(&page.headers);
        let token = csrf(&page.body);
        let added = send(
            router.clone(),
            post(
                "/cart/add",
                &old_cookie,
                &[("csrf", &token), ("product", "travel-mug")],
            ),
        )
        .await;
        assert_eq!(added.status, StatusCode::SEE_OTHER);
        let page = send(router.clone(), get("/cart", Some(&old_cookie))).await;
        let token = csrf(&page.body);

        let cleared = send(
            router.clone(),
            post("/cart/clear", &old_cookie, &[("csrf", &token)]),
        )
        .await;
        assert_eq!(cleared.status, StatusCode::SEE_OTHER);
        let new_cookie = cookie_pair(&cleared.headers);
        assert_ne!(old_cookie, new_cookie);

        let current = send(router.clone(), get("/cart", Some(&new_cookie))).await;
        assert!(current.body.contains("Your cart is empty."));
        let old = send(router, get("/cart", Some(&old_cookie))).await;
        assert!(old.body.contains("Your cart is empty."));
        assert_ne!(cookie_pair(&old.headers), old_cookie);
    }

    #[tokio::test]
    async fn ending_session_emits_removal_cookie() {
        let router = test_app();
        let page = send(router.clone(), get("/cart", None)).await;
        let cookie = cookie_pair(&page.headers);
        let token = csrf(&page.body);
        let ended = send(router, post("/cart/end", &cookie, &[("csrf", &token)])).await;
        assert_eq!(ended.status, StatusCode::SEE_OTHER);
        assert!(
            ended.headers[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .contains("Max-Age=0")
        );
    }
}
