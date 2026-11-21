//! Composable application lifetimes, UI values and optional integrations.
//!
//! Built-in [`HostGroup`], [`Host`] and [`ResourceGroup`] use the std-only
//! [`composition`] engine. They create no executor, thread pool or process.
//! Custom and local Hosts do not need an integration feature.
//!
//! The default feature set is empty. Enable `view` for owned HTML/DOM values;
//! components are ordinary functions returning a `View`. Construction does not
//! escape or validate content: each renderer applies its own output policy.
//! Enable `html` for pure HTML serialization; it enables `view` without a Web
//! runtime, browser bindings, provider or middleware.
//!
//! # First View
//!
//! Enable the `html` feature explicitly:
//!
//! ```
//! # #[cfg(feature = "html")]
//! # fn main() -> Result<(), yetlet::html::RenderError> {
//! use yetlet::{Element, View};
//!
//! let page: View = Element::new("p")
//!     .child(View::text("Hello, <Yetlet>!"))
//!     .into();
//! let html = yetlet::html::render_to_string(&page)?;
//! assert_eq!(html, "<p>Hello, &lt;Yetlet&gt;!</p>");
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "html"))]
//! # fn main() {}
//! ```
//!
//! # Choose features explicitly
//!
//! | Feature | Entry point and responsibility |
//! |---|---|
//! | `view` | `view`: owned HTML/DOM trees, typed tags and binding metadata; no renderer or runtime |
//! | `html` | Enables `view`; `html` serializes escaped fragments; HTTP document/header ownership stays separate |
//! | `server-axum` | `server_axum`: native Axum handlers, pages, routing and Tower interoperability |
//! | `session`, `auth`, `auth-session` | Separate native Session/identity APIs and an explicit Memory Session bridge; business authorization stays application-owned |
//! | `session-redis`, `session-redis-tls` | Optional native asynchronous remote Session provider and checked TLS transport |
//! | `request-id`, `rate-limit` | Explicit native request correlation and per-process admission |
//! | `config`, `secrets`, `secret-provider` | Optional native startup loading, secret lookup and their bridge |
//! | `signing` | Optional native signing/verifying API with caller-owned trust and OpenSSL prerequisites |
//! | `actions`, `reactive` | Typed action boundary and synchronous local state graph |
//! | Default Core | `HostGroup` / `Host` / `ResourceGroup`; advanced `composition` ownership, readiness and original completion; no optional dependencies |
//!
//! Cargo features make APIs available; application startup installs middleware,
//! providers and typed state. Native capability modules are absent on `wasm32`.
//! The separate `yetlet-browser` package supplies opt-in DOM/action/GPU adapters
//! whose DOM features select the same View types. This package has no reverse
//! dependency on browser integrations.
//!
//! # Guides and compatibility
//!
//! Read the [first-app guide](https://github.com/bohewu/yetlet/blob/master/docs/FIRST-APP.md),
//! [View guide](https://github.com/bohewu/yetlet/blob/master/docs/VIEW-COMPONENTS.md)
//! and [API map](https://github.com/bohewu/yetlet/blob/master/docs/API-REFERENCE.md)
//! at the revision you consume. Those links follow repository development;
//! released documentation and checkout documentation can differ. APIs remain
//! Experimental; a successful render is not authentication or authorization.
#[cfg(feature = "view")]
pub mod view;
#[cfg(feature = "view")]
pub use view::{Attribute, Element, View};
#[cfg(feature = "actions")]
pub mod actions;
#[cfg(all(feature = "auth", not(target_arch = "wasm32")))]
pub mod auth;
pub mod composition;
pub use composition::{Host, HostGroup, ResourceGroup};
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
