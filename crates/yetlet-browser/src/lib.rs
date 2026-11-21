//! Optional browser DOM, action transport and GPU integrations for Yetlet.
//!
//! Import the Cargo package `yetlet-browser` as `yetlet_browser`. The `dom` and
//! `dom-keyed` features enable shared View values and re-export `View`, `Element`,
//! `Attribute` and `view` from `yetlet`. Neither feature enables HTML serialization
//! or native HTTP serving. Disabling default features selects no adapter by itself;
//! actions-only and GPU-only selections do not expose these View reexports.
//! Native SSR does not require this package.
//!
//! # Validate a View
//!
//! With the default `dom` feature, validation also works on a native target:
//!
//! ```
//! # #[cfg(feature = "dom")]
//! # fn main() -> Result<(), yetlet_browser::dom::DomPolicyError> {
//! use yetlet_browser::{Element, View};
//!
//! let view: View = Element::new("p").child(View::text("Hello")).into();
//! yetlet_browser::dom::validate(&view)?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "dom"))]
//! # fn main() {}
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
//! | Default `dom` | Shared View values and native/WASM validation; WASM `dom::mount` replaces one owned subtree |
//! | `dom-keyed` | Shared View values and keyed reconciliation/listener lifetime; actual DOM operations are WASM-only |
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
#[cfg(any(feature = "dom", feature = "dom-keyed"))]
pub use yetlet::{Attribute, Element, View, view};
#[cfg(feature = "actions")]
pub mod actions;
#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "dom-keyed")]
pub mod dom_keyed;
#[cfg(feature = "wgpu")]
pub mod wgpu;
