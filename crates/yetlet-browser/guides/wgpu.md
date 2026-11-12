# yetlet-browser::wgpu — Experimental opt-in flat rectangles

First-party Experimental module included in `yetlet-browser` SDK `0.9.0-alpha.8`. No performance, speedup,
FPS, stable API, production readiness or publication guarantee. This is the
bounded flat renderer extracted from the current `webgpu-scene-island` consumer.
Future 3D is a motivation only; there is no 3D API, scene graph or engine.

Selecting only the `wgpu` feature exposes CPU-only contracts without GPU
dependencies. The browser SDK defaults to DOM, not GPU. Linking does not acquire
a device or initialize anything; ordinary HTML/DOM/server use does not enable
`wgpu-browser`. The root workspace already contains
an older optional native wgpu research example; that is a separate graph.

## Local quickstart

For this repository-local example, use the checkout through a path dependency.
The published SDK is available separately; this example does not imply that
later checkout changes are included in its registry archive:

```toml
[dependencies]
yetlet-browser = { path = "../../crates/yetlet-browser", version = "=0.9.0-alpha.8", default-features = false, features = ["wgpu"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
yetlet-browser = { path = "../../crates/yetlet-browser", version = "=0.9.0-alpha.8", default-features = false, features = ["wgpu-browser"] }
```

The crate-level rustdoc example is compile-checked by `cargo test --doc`:

```rust
use yetlet_browser::wgpu::{Batch, PickViewport, pick_records};
let bytes = [0f32, 0., 0.5, 0.5, 1., 0., 0., 1.]
    .map(f32::to_le_bytes).concat();
let batch = Batch::new(bytes).unwrap();
let viewport = PickViewport { width: 600., height: 400., dpr: 1. };
assert_eq!(pick_records(batch.bytes(), viewport, 300., 200.).unwrap(), Some(1));
```

With `wgpu-browser` and `wasm32-unknown-unknown`, create
`browser::SceneOwner::new(generation, batch, fault_sink)` explicitly. The sink is
an injected Rust function `fn(u32, &str)`; a Wasm application may bridge it to JS.
See the real [application binding](https://github.com/bohewu/yetlet/blob/master/examples/webgpu-scene-island/src/browser.rs).
Call `initialize(HtmlCanvasElement).await`, `resize(css_width, css_height, dpr,
connected)`, then `draw()` on demand. `completion().await` waits for submitted
work, not presentation. Methods take Yetlet-owned CPU types or browser
`HtmlCanvasElement`/`JsValue`; no public wgpu type or arbitrary shader input exists.

The application owns the canvas, generation, scheduling, visibility, DOM
listeners, semantic alternative and explicit retry. Reserve canvas dimensions.
Treat a false resize as suspension. Dispose the old owner before a replacement
can configure the same canvas. Keep the Rust/Wasm owner alive until outstanding
initialization/completion futures settle; only then free the JS wrapper exactly
once. `SceneOwner::dispose(&self) -> bool` returns `true` exactly when the
owner first transitions to disposed (including from a failed state), and
`false` on repeated calls. `Lifetime::retire(&self) -> bool` follows that
same one-time transition rule. This return value reports **logical
retirement**, not completion of outstanding async borrows or immediate
Wasm wrapper deallocation; the caller must still wait before `.free()`.
Both invalidate late callbacks/picks. A new owner gets a new generation
and separately retained CPU data. No shared-device API is
provided. The application loader demonstrates deferred wrapper freeing and
retained CPU data; the crate installs no listeners, RAF loop or ambient device.

## Data, limits and failures

Each record is eight little-endian f32 values: center x/y, positive half width/
height and RGBA. Finite geometry must fit normalized [-1,1], colors [0,1]; at
most 100,000 records (3,200,000 payload bytes). Zero records, partial strides and
invalid ranges fail before mutation/upload. Successful updates/replacements
advance a checked u32 version; exhaustion fails atomically. Picking validates
and reverse-scans O(n) on explicit selection; last drawn overlap wins. Returned
IDs are one-based slots, not business records. Applications preserve ordering
when they want stable semantic identities. CSS-local coordinates exclude right/
bottom exterior edges. DPR is clamped to [1,2]; dimensions are bounded by the
negotiated device texture limit; zero/detached targets suspend.

CPU validation, batch operations and picking return typed
`Result<_, SceneError>`, with non-exhaustive variants:
`InvalidRecordLength`, `InvalidRecordData`, `InvalidUpdateRange`,
`VersionExhausted`, `InvalidViewport`, `InvalidPointer`. The variants
can be matched without parsing error text. `Display` and `as_str()`
preserve bounded existing diagnostic strings. At the Wasm-facing browser
boundary the CPU error maps explicitly to a `JsValue` string; GPU device,
surface, validation and lifecycle failures remain `Result<_, JsValue>`.
Browser initialization errors mean unavailable canvas/adapter/device/configuration or retired owner;
keep HTML usable, dispose and require explicit retry. Runtime categories are
`device-lost`, `surface-lost` (including outdated), `surface-timeout`,
`surface-error`, `resource-exhausted`, `validation-error` and `device-error`.
The injected sink receives bounded category strings, not raw diagnostic HTML.
The application maps them to safe text and fallback and routes by generation.
There is no automatic recovery loop. WGSL is trusted build-time crate source.

Scalar telemetry reports explicit allocations/upload lengths and CPU/wall
wait durations. It does not measure resident VRAM, GPU timestamps, presentation
or FPS. Peak transition buffer bytes include old and replacement allocations.

## Selected Experimental support

The actual extracted candidate passed [YT-079 verification](https://github.com/bohewu/yetlet/blob/master/docs/YT-079-VERIFICATION.md)
and [independent review](https://github.com/bohewu/yetlet/blob/master/docs/YT-079-INDEPENDENT-REVIEW.md) on selected
Firefox 158 / Ubuntu X11 / Intel Vulkan with disposable WebGPU opt-in. See the
[source-bound support matrix](wgpu-support.md) and final YT-080 qualification record.
Windows JS baseline and failed Chrome configurations are not candidate PASS.
No broad browser, physical mobile, native backend, stable API, production or 3D
support is claimed. See [YT-079 verification](https://github.com/bohewu/yetlet/blob/master/docs/YT-079-VERIFICATION.md).

The renderer is now a module of the browser SDK in the root workspace; the
consumer retains its separate workspace/lock. Use the explicit commands:

```sh
cargo test --offline --locked -p yetlet-browser --features wgpu
cargo check --offline --locked -p yetlet-browser --features wgpu-browser --target wasm32-unknown-unknown
cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml
cargo xtask build scene-island
```

Exact existing pins remain wgpu `=27.0.1`, wasm-bindgen `=0.2.129`,
wasm-bindgen-futures `=0.4.79` (consumer/upstream) and web-sys `=0.3.106`.
No upgrade or download is needed for the cached local build.