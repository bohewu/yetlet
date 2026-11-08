# Yetlet Browser

Opt-in browser integrations sharing the single View type from `yetlet`.
Version `0.9.0-alpha.7`; see [GitHub releases](https://github.com/bohewu/yetlet/releases)
for published artifacts.
Use Rust 1.99+, `wasm32-unknown-unknown`, and the matching wasm-bindgen toolchain
when building actual browser assets.

## Minimal integration

Use these exact registry dependencies once this version is available:

```toml
[dependencies]
yetlet = { version = "=0.9.0-alpha.7", default-features = false }
yetlet-browser = "=0.9.0-alpha.7"
```

For local preparation, select matching source revisions for both crates:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.7", default-features = false }
yetlet-browser = { path = "/absolute/yetlet/crates/yetlet-browser", version = "=0.9.0-alpha.7" }
```

This release accepts the same shared View produced by the typed Tag Builder API
in `yetlet`; constructors do not expand DOM admission or URL/image permissions.
Construct a `yetlet::View` once and pass it to the selected renderer. Browser
root View/Element/Attribute re-exports name the same base types. The dependency
is one-way; native applications need only `yetlet`.

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Subtree DOM mount | `dom` (default) | [DOM](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/dom.md) |
| Keyed children/listeners | `dom-keyed` | [Keyed DOM](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/dom-keyed.md) |
| HTTP Action executor | `actions` | [Portable contracts and browser transport](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/actions.md) |
| CPU scene model/validation | `wgpu` | [Scene](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/wgpu.md) |
| Actual WebGPU backend | `wgpu-browser` | [Support](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/wgpu-support.md) |

DOM bindings, HTTP execution and the GPU backend require wasm32. Native static
validation is separate from actual browser qualification. Linking the package
starts no device, fetch, loop or event listener: the application owns activation,
cleanup, errors and trusted endpoint policy. WebGPU does not accelerate DOM/CSS.
No fresh hardware/browser qualification is claimed by this source migration.
APIs remain Experimental; native compile checks do not establish browser, hardware
or production qualification.
