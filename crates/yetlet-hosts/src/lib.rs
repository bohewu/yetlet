//! Optional concrete native adapters for Yetlet.
//!
//! The crate has no default features. Native Web, UDP and GPU dependencies are
//! never forced on ordinary Yetlet callers. No universal Host trait or
//! application scheduler is introduced.
#![forbid(unsafe_code)]

/// Temporary std-only domain bridge; native HTTP workers live in `yetlet::web`.
#[cfg(feature = "web-axum")]
pub mod web_axum;

/// Optional owned nonblocking UDP byte/socket endpoint, without a game codec.
#[cfg(feature = "udp")]
pub mod udp;

/// Opt-in caller-owned Winit foreground Window slot; no renderer/GPU.
#[cfg(feature = "native-winit")]
pub mod native_winit;

/// Independent opt-in Vulkan Surface renderer; accepts caller-provided handles.
#[cfg(feature = "renderer-wgpu")]
pub mod renderer_wgpu;
