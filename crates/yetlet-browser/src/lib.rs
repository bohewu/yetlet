//! Optional browser integrations for the same [`View`] used by native Yetlet.
//!
//! Import the Cargo package `yetlet-browser` as `yetlet_browser`. Its re-exported
//! [`View`], [`Element`] and [`Attribute`] are the original `yetlet` types, not
//! another UI model. Native SSR does not require this package.
//!
//! # Validate a View
//!
//! With the default `dom` feature, validation also works on a native target:
//!
//! ```
//! use yetlet_browser::{Element, View};
//!
//! let view: View = Element::new("p").child(View::text("Hello")).into();
//! yetlet_browser::dom::validate(&view)?;
//! # Ok::<(), yetlet_browser::dom::DomPolicyError>(())
//! ```
//!
//! This checks the DOM renderer's finite vocabulary; it does not mount anything.
//! Actual DOM mounting requires `wasm32-unknown-unknown` and a browser-owned host.
//! Browser URL resolution can still reject navigation accepted by static validation.
//!
//! # Select an adapter
//!
//! | Feature | Availability and responsibility |
//! |---|---|
//! | Default `dom` | Native/WASM validation; WASM `dom::mount` replaces one owned subtree |
//! | `dom-keyed` | Explicit keyed reconciliation/listener lifetime; actual DOM operations are WASM-only |
//! | `actions` | Shared typed actions plus WASM HTTP execution; no implicit authentication, CSRF or retry policy |
//! | `wgpu` | CPU-side bounded scene APIs, independent of DOM/CSS |
//! | `wgpu-browser` | Explicit WASM WebGPU backend, requiring browser/device support |
//!
//! Selecting a feature compiles an API; the application still owns activation,
//! target elements, listeners and failure handling. Native-only secrets/providers
//! must never enter browser builds or serialized state. WebGPU is a separate
//! renderer, not an accelerator for HTML/CSS.
//!
//! Follow the [DOM guide](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/dom.md)
//! and [API map](https://github.com/bohewu/yetlet/blob/master/docs/API-REFERENCE.md)
//! at the matching revision. These links follow repository development; the SDK
//! remains Experimental and no new browser/GPU qualification follows from docs.
pub use yetlet::{Attribute, Element, View, view};
#[cfg(feature = "actions")]
pub mod actions;
#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "dom-keyed")]
pub mod dom_keyed;
#[cfg(feature = "wgpu")]
pub mod wgpu;
