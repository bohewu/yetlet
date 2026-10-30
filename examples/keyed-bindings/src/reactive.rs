//! Compatibility re-export for the isolated M7b–M7d fixture.
//!
//! The graph implementation moved to the optional `yetlet-reactive` crate in
//! M7g. Keeping this module path avoids coupling the experimental authoring
//! fixture to a second copy of the ownership and batching logic.

pub use yetlet::reactive::*;
