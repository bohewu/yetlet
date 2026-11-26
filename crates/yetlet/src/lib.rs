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
//! | `sfc` | Experimental native build-script compiler for source-checkout single-file components |
//! | `html` | Enables `view`; `html` serializes escaped fragments; HTTP document/header ownership stays separate |
//! | `web` | `web`: native HTTP serving, routing and Tower interoperability; add `html` for pages/SSR |
//! | `session`, `auth`, `auth-session` | Separate native Session/identity APIs and an explicit Memory Session bridge; business authorization stays application-owned |
//! | `session-redis`, `session-redis-tls` | Optional native asynchronous remote Session provider and checked TLS transport |
//! | `request-id`, `rate-limit` | Explicit native request correlation and per-process admission |
//! | `config`, `secrets`, `secret-provider` | Optional native startup loading, secret lookup and their bridge |
//! | `signing` | Optional native signing/verifying API with caller-owned trust and OpenSSL prerequisites |
//! | `actions` | Transport-neutral commands/executors with implementation-owned errors; no serialization dependency |
//! | `actions-http` | Enables `actions`; shared HTTP action errors and serializable wire envelopes |
//! | `command-bridge` | Native bounded command admission and immutable snapshots; independent of HTTP and action executors |
//! | `udp` | Native nonblocking datagrams and managed UDP Hosts; no async runtime |
//! | `native-winit` | Caller-thread Winit Host with multiple windows under one root event loop; no renderer |
//! | `renderer-wgpu` | Independent native Vulkan Surface renderer; no Winit, Web or View dependency |
//! | `reactive` | Synchronous local state graph |
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
#[cfg(feature = "actions-http")]
pub mod actions_http;
#[cfg(all(feature = "auth", not(target_arch = "wasm32")))]
pub mod auth;
#[cfg(all(feature = "command-bridge", not(target_arch = "wasm32")))]
pub mod command_bridge;
pub mod composition;
pub use composition::{Host, HostGroup, ResourceGroup};
#[cfg(all(feature = "config", not(target_arch = "wasm32")))]
pub mod config;
#[cfg(feature = "html")]
pub mod html;
#[cfg(all(feature = "native-winit", not(target_arch = "wasm32")))]
pub mod native_winit;
#[cfg(all(feature = "rate-limit", not(target_arch = "wasm32")))]
pub mod rate_limit;
#[cfg(feature = "reactive")]
pub mod reactive;
#[cfg(all(feature = "renderer-wgpu", not(target_arch = "wasm32")))]
pub mod renderer_wgpu;
#[cfg(all(feature = "request-id", not(target_arch = "wasm32")))]
pub mod request_id;
#[cfg(all(feature = "secrets", not(target_arch = "wasm32")))]
pub mod secrets;
#[cfg(all(feature = "session", not(target_arch = "wasm32")))]
pub mod session;
#[cfg(all(feature = "session-redis", not(target_arch = "wasm32")))]
pub mod session_redis;
#[cfg(all(feature = "sfc", not(target_arch = "wasm32")))]
pub mod sfc;
#[cfg(all(feature = "signing", not(target_arch = "wasm32")))]
pub mod signing;
#[cfg(all(feature = "udp", not(target_arch = "wasm32")))]
pub mod udp;
#[cfg(all(feature = "web", not(target_arch = "wasm32")))]
pub mod web;
