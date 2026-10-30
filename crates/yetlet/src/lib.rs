//! Yetlet: HTML-first views with explicit optional native integrations.
pub mod view;
pub use view::{Attribute, Element, View};
#[cfg(feature = "actions")]
pub mod actions;
#[cfg(all(feature = "auth", not(target_arch = "wasm32")))]
pub mod auth;
#[cfg(all(feature = "config", not(target_arch = "wasm32")))]
pub mod config;
#[cfg(feature = "html")]
pub mod html;
#[cfg(all(feature = "rate-limit", not(target_arch = "wasm32")))]
pub mod rate_limit;
#[cfg(feature = "reactive")]
pub mod reactive;
#[cfg(all(feature = "request-id", not(target_arch = "wasm32")))]
pub mod request_id;
#[cfg(all(feature = "secrets", not(target_arch = "wasm32")))]
pub mod secrets;
#[cfg(all(feature = "server-axum", not(target_arch = "wasm32")))]
pub mod server_axum;
#[cfg(all(feature = "session", not(target_arch = "wasm32")))]
pub mod session;
#[cfg(all(feature = "session-redis", not(target_arch = "wasm32")))]
pub mod session_redis;
#[cfg(all(feature = "signing", not(target_arch = "wasm32")))]
pub mod signing;
