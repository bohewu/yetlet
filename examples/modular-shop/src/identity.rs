use std::{fmt, io};

use axum::{
    Form,
    extract::{DefaultBodyLimit, Extension, State, rejection::FormRejection},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use subtle::ConstantTimeEq;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};
use yetlet_session::{Session, SessionManager};

use crate::{
    AppState, FORM_LIMIT, Failure, ShopSession, UserId, current_user, hidden, map_session,
    parse_form, require_csrf,
};

const CODE_BYTES: usize = 32;
const CODE_HEX_LEN: usize = CODE_BYTES * 2;

#[derive(Clone)]
pub struct DemoIdentities {
    alice: String,
    bob: String,
}

impl fmt::Debug for DemoIdentities {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DemoIdentities")
            .field("alice", &"[redacted]")
            .field("bob", &"[redacted]")
            .finish()
    }
}

impl DemoIdentities {
    pub fn generate() -> io::Result<Self> {
        Ok(Self {
            alice: random_code()?,
            bob: random_code()?,
        })
    }

    pub fn from_codes(alice: &str, bob: &str) -> io::Result<Self> {
        if !valid_code(alice) || !valid_code(bob) || alice == bob {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "synthetic codes must be distinct 64-character lowercase hexadecimal values",
            ));
        }
        Ok(Self {
            alice: alice.to_owned(),
            bob: bob.to_owned(),
        })
    }

    pub fn terminal_codes(&self) -> (&str, &str) {
        (&self.alice, &self.bob)
    }

    pub(crate) fn authenticate(&self, username: &str, code: &str) -> Option<UserId> {
        let (expected, user, known) = match username {
            "alice" => (self.alice.as_bytes(), Some(UserId::Alice), true),
            "bob" => (self.bob.as_bytes(), Some(UserId::Bob), true),
            _ => (self.alice.as_bytes(), None, false),
        };
        let mut candidate = [0_u8; CODE_HEX_LEN];
        let shape = valid_code(code);
        if code.len() == CODE_HEX_LEN {
            candidate.copy_from_slice(code.as_bytes());
        }
        let equal: bool = candidate.ct_eq(expected).into();
        if known && shape && equal { user } else { None }
    }
}

fn random_code() -> io::Result<String> {
    let mut bytes = [0_u8; CODE_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
    let mut code = String::with_capacity(CODE_HEX_LEN);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        code.push(HEX[(byte >> 4) as usize] as char);
        code.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(code)
}

fn valid_code(value: &str) -> bool {
    value.len() == CODE_HEX_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginForm {
    csrf: String,
    username: String,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CsrfForm {
    csrf: String,
}

fn login_view(csrf: &str, error: Option<&str>) -> View {
    let mut main = Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text("Yetlet Modular Shop Login")))
        .child(Element::new("p").child(View::text(
            "Use a terminal-only synthetic Alice or Bob code. This is not a production identity provider.",
        )));
    if let Some(error) = error {
        main = main.child(
            Element::new("p")
                .attribute("role", "alert")
                .child(View::text(error)),
        );
    }
    main.child(
        Element::new("form")
            .attribute("method", "post")
            .attribute("action", "/identity/login")
            .child(hidden("csrf", csrf))
            .child(
                Element::new("label")
                    .attribute("for", "username")
                    .child(View::text("Synthetic identity (alice or bob)")),
            )
            .child(
                Element::new("input")
                    .attribute("id", "username")
                    .attribute("name", "username")
                    .attribute("type", "text")
                    .attribute("required", "")
                    .attribute("autocomplete", "username"),
            )
            .child(
                Element::new("label")
                    .attribute("for", "code")
                    .child(View::text("Terminal login code")),
            )
            .child(
                Element::new("input")
                    .attribute("id", "code")
                    .attribute("name", "code")
                    .attribute("type", "password")
                    .attribute("required", "")
                    .attribute("autocomplete", "current-password"),
            )
            .child(
                Element::new("button")
                    .attribute("type", "submit")
                    .child(View::text("Sign in and rotate session")),
            ),
    )
    .into()
}

pub(crate) fn logout_form(csrf: &str) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/identity/logout")
        .child(hidden("csrf", csrf))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text("Sign out and rotate session")),
        )
        .into()
}

async fn login_page(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
) -> Result<Page, Failure> {
    if let Some(user) = session.read().map_err(map_session)?.user {
        return Ok(Page::new(
            "Yetlet | Already signed in",
            Element::new("main")
                .attribute("lang", "en")
                .child(
                    Element::new("h1")
                        .child(View::text(format!("Signed in as {}", user.display()))),
                )
                .child(
                    Element::new("a")
                        .attribute(
                            "href",
                            if state.modules.cart {
                                "/shop/cart"
                            } else {
                                "/healthz"
                            },
                        )
                        .child(View::text(if state.modules.cart {
                            "Open cart"
                        } else {
                            "Health"
                        })),
                ),
        ));
    }
    let csrf = session.csrf_token().map_err(map_session)?;
    Ok(Page::new("Yetlet | Shop login", login_view(&csrf, None)))
}

async fn login(
    Extension(session): Extension<Session<ShopSession>>,
    State(state): State<AppState>,
    form: Result<Form<LoginForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let user = state
            .identities
            .authenticate(form.username.trim(), &form.code)
            .ok_or(Failure::InvalidCredentials)?;
        session
            .replace_and_rotate(ShopSession {
                user: Some(user),
                ..ShopSession::default()
            })
            .map_err(map_session)?;
        let target = if state.modules.catalog {
            "/shop/catalog"
        } else {
            "/healthz"
        };
        Ok::<_, Failure>(Redirect::to(target).into_response())
    })();
    match result {
        Ok(response) => response,
        Err(Failure::InvalidCredentials) => {
            let csrf = match session.csrf_token() {
                Ok(value) => value,
                Err(error) => return map_session(error).into_response(),
            };
            (
                axum::http::StatusCode::UNAUTHORIZED,
                Page::new(
                    "Yetlet | Invalid login",
                    login_view(&csrf, Some("Invalid synthetic identity or login code")),
                ),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn logout(
    Extension(session): Extension<Session<ShopSession>>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Response {
    let result = (|| {
        current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session
            .replace_and_rotate(ShopSession::default())
            .map_err(map_session)?;
        Ok::<_, Failure>(Redirect::to("/identity/login").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

pub(crate) fn module(sessions: SessionManager<ShopSession>) -> App<AppState> {
    App::<AppState>::new()
        .get("/identity/login", login_page)
        .post("/identity/login", login)
        .post("/identity/logout", logout)
        .route_layer(DefaultBodyLimit::max(FORM_LIMIT))
        .configure(move |app| sessions.scoped(app))
}
