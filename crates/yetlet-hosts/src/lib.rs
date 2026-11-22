//! Unpublished forwarding adapters for historical Yetlet consumers.
//!
//! Canonical implementations live behind explicit features in `yetlet`.
//! This temporary companion has no default features and remains only while
//! examples migrate to those modules; it is not an additional SDK package.
#![forbid(unsafe_code)]

/// Forwards the std-only `yetlet::command_bridge`; HTTP serving is separate.
#[cfg(all(feature = "web-axum", not(target_arch = "wasm32")))]
pub mod web_axum;

/// Optional owned nonblocking UDP byte/socket endpoint, without a game codec.
#[cfg(all(feature = "udp", not(target_arch = "wasm32")))]
pub mod udp;

/// Forwards the caller-thread Winit Host and Window slot; no renderer/GPU.
#[cfg(all(feature = "native-winit", not(target_arch = "wasm32")))]
pub mod native_winit;

/// Independent opt-in Vulkan Surface renderer; accepts caller-provided handles.
#[cfg(all(feature = "renderer-wgpu", not(target_arch = "wasm32")))]
pub mod renderer_wgpu;
