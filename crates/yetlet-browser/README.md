# Yetlet Browser

Opt-in browser integrations sharing the single View type from `yetlet`.
Unreleased `0.9.0-alpha.5`; these package names are not yet registry-published.
Use Rust 1.99+, `wasm32-unknown-unknown`, and the matching wasm-bindgen toolchain
when building actual browser assets.

## Minimal integration

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.5", default-features = false }
yetlet-browser = { path = "/absolute/yetlet/crates/yetlet-browser", version = "=0.9.0-alpha.5" }
```

Construct a `yetlet::View` once and pass it to the selected renderer. Browser
root View/Element/Attribute re-exports name the same base types. The dependency
is one-way; native applications need only `yetlet`.

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Subtree DOM mount | `dom` (default) | [DOM](guides/dom.md) |
| Keyed children/listeners | `dom-keyed` | [Keyed DOM](guides/dom-keyed.md) |
| HTTP Action executor | `actions` | [Portable contracts and browser transport](../yetlet/guides/actions.md) |
| CPU scene model/validation | `wgpu` | [Scene](guides/wgpu.md) |
| Actual WebGPU backend | `wgpu-browser` | [Support](guides/wgpu-support.md) |

DOM bindings, HTTP execution and the GPU backend require wasm32. Native static
validation is separate from actual browser qualification. Linking the package
starts no device, fetch, loop or event listener: the application owns activation,
cleanup, errors and trusted endpoint policy. WebGPU does not accelerate DOM/CSS.
No fresh hardware/browser qualification is claimed by this source migration.
