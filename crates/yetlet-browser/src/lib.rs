//! Optional browser SDK; shared View types are defined once by Yetlet.
pub use yetlet::{Attribute, Element, View, view};
#[cfg(feature = "actions")]
pub mod actions;
#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "dom-keyed")]
pub mod dom_keyed;
#[cfg(feature = "wgpu")]
pub mod wgpu;
