# Yetlet Browser

Opt-in DOM, action transport and GPU integrations. DOM adapters share the single
View type from `yetlet`.
Version `0.9.0-alpha.8`; see [GitHub releases](https://github.com/bohewu/yetlet/releases)
for published artifacts.
Use Rust 1.99+, `wasm32-unknown-unknown`, and the matching wasm-bindgen toolchain
when building actual browser assets.

## Minimal integration

Use these exact registry dependencies for the published DOM default:

```toml
[dependencies]
yetlet = { version = "=0.9.0-alpha.8", default-features = false }
yetlet-browser = "=0.9.0-alpha.8"
```

For the current checkout, select matching source revisions for both crates:

```toml
[dependencies]
yetlet = { path = "/absolute/yetlet/crates/yetlet", version = "=0.9.0-alpha.8", default-features = false }
yetlet-browser = { path = "/absolute/yetlet/crates/yetlet-browser", version = "=0.9.0-alpha.8" }
```

Both configurations select the Browser `dom` default and accept the shared View
produced by the typed Tag Builder API in `yetlet`; constructors do not expand
DOM admission or URL/image permissions. Construct a `yetlet::View` once and
pass it to the selected renderer. The dependency is one-way; native applications
need only `yetlet`.

In this checkout, `dom` and `dom-keyed` each enable `yetlet/view`. Browser root
`View`, `Element`, `Attribute` and `view` reexports exist only with either DOM
feature; neither feature enables HTML serialization or native HTTP. The published
alpha.8 package retains its original unconditional View reexports. These source
changes do not alter that registry artifact or require a new feature on an
alpha.8 registry dependency.

The current Browser `actions` feature forwards `yetlet/actions-http` for the
shared HTTP errors/envelopes and adds its target-specific Fetch dependencies.
It does not enable native `web`, View, HTML or DOM. The executor implements the
neutral `ActionExecutor` with its associated HTTP error type; local executors
can use their own domain errors. Published alpha.8 keeps its original combined
Action/HTTP contract, while its registry dependency syntax above remains valid.

## Features and guides

| Capability | Feature | Guide |
|---|---|---|
| Subtree DOM mount | `dom` (default) | [DOM](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/dom.md) |
| Keyed children/listeners | `dom-keyed` | [Keyed DOM](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/dom-keyed.md) |
| HTTP Action executor | `actions` (forwards `yetlet/actions-http` in this checkout) | [Portable contracts and browser transport](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/guides/actions.md) |
| CPU scene model/validation | `wgpu` | [Scene](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/wgpu.md) |
| Actual WebGPU backend | `wgpu-browser` | [Support](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-browser/guides/wgpu-support.md) |

To select only `actions`, `wgpu` or `wgpu-browser`, set `default-features = false`
on the Browser dependency and enable the desired feature explicitly. In this
checkout those selections do not enable View, DOM or HTML. Keeping default
features also selects DOM, including its View dependency.

DOM bindings, HTTP execution and the GPU backend require wasm32. Native static
validation is separate from actual browser qualification. Linking the package
starts no device, fetch, loop or event listener: the application owns activation,
cleanup, errors and trusted endpoint policy. WebGPU does not accelerate DOM/CSS.
No fresh hardware/browser qualification is claimed by this source migration.
APIs remain Experimental; native compile checks do not establish browser, hardware
or production qualification.
