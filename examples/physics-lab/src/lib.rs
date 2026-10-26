#[cfg(all(feature = "browser", target_arch = "wasm32"))]
mod gpu;
pub mod model;
#[cfg(feature = "browser")]
pub mod physics;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
