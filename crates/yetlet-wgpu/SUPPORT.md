# Experimental Support Boundary

`yetlet-wgpu` is **Experimental, opt-in and unpublished**, with publication
metadata prepared under [YT-201](../../docs/YT-201-PACKAGE-PREPARATION.md).
There is no performance, speedup, FPS, stable API, production-readiness or broad
browser guarantee. The measured YT-077 comparison did not demonstrate a useful
WebGPU speedup; experimental promotion was separately approved for an optional
extension path. Possible future 3D is motivation, not an implemented 3D API,
scene graph, engine, native backend or promise of future compatibility.

## Selected configuration

The extracted candidate has actual YT-079 browser and independent review PASS
on **Ubuntu 26.04.1, kernel 7.0.0-38-generic, XFCE/X11, Firefox 158.0 Stable
Snap with geckodriver 0.37.1, Intel UHD Graphics 770 / Vulkan**. WebGPU was
explicitly enabled using `dom.webgpu.enabled=true` only in a disposable profile.
Firefox's Snap graphics runtime reported Mesa 25.2.8; the host Chrome/Mesa stack
was different and is not substituted for this observation.

The supported experimental path is a local same-origin/secure-context browser
consumer with separately built, source-hashed JS/WASM assets, meaningful SSR
fallback, and explicit per-canvas activation. The application must keep the
semantic alternative and own visibility, scheduling, input, generations and
bounded explicit retry. Default linking does not create a device or load assets.

| Configuration or behavior | Evidence / boundary |
| --- | --- |
| Selected Firefox 158 X11 + Intel Vulkan + explicit WebGPU preference | Actual extracted Rust/wgpu Wasm drawing, updates, picking, controlled loss/recovery, stale completion and cleanup PASS; YT-079 receipt and independent review |
| Firefox 158 default WebDriver profile without WebGPU preference override | YT-080 actual observation: secure context but `navigator.gpu` absent, named unavailable state; HTML/counter/keyboard/detail navigation PASS. This is fallback evidence, not GPU support |
| Native CPU-only default feature set | CPU contract tests, doctest and package verification; no GPU dependencies or initialization |
| wasm32 CPU-only default feature set | Compile/dependency graph only; no browser GPU runtime claim |
| wasm32 explicit `browser` feature | Current-source build and actual selected consumer evidence |
| Linux Chrome / NVIDIA or Intel, X11 or nested Weston | Earlier application configurations failed; current extracted candidate not qualified on these combinations |
| Windows Chrome / Edge | Standalone JavaScript shader baseline only; not the current candidate's Wasm/SSR integration |
| macOS/Safari, Android/iOS, physical mobile, other browsers/GPUs | Not qualified |
| Native Vulkan, D3D12 or Metal renderer | Not implemented by this browser-only candidate; historical native research is separate |
| Physical touch, real reduced-motion OS preference, screenreader speech | Not newly qualified; earlier viewport and selected events include explicitly labeled emulation/simulation |

## Explicit opt-in

Use a local path dependency, keeping the crate outside default application
features. The consumer and this crate have isolated nested Cargo workspaces;
this is not a detached Git worktree and does not replace the root release set.

```toml
[dependencies]
yetlet-wgpu = { path = "../../crates/yetlet-wgpu", default-features = false }

[target.'cfg(target_arch = "wasm32")'.dependencies]
yetlet-wgpu = { path = "../../crates/yetlet-wgpu", default-features = false, features = ["browser"] }
```

Native defaults expose validated batches, picking and lifetime contracts only.
Only wasm32 with `browser` enables pinned wgpu `=27.0.1`, wasm-bindgen
`=0.2.129` and web-sys `=0.3.106`. The wgpu features are `std`, `webgpu`,
`wgsl`, not native backend features. This checkout uses Rust/Cargo 1.99.
No framework core/HTML/ordinary DOM/default HTTP-server graph depends on the
renderer. The root workspace's older native scene-research dependency is separate.

The sample is `examples/webgpu-scene-island`. Its README provides native build,
independent WASM build, loopback start and disposal steps. Do not rerun historical
one-shot acceptance scripts over their existing evidence directories. They are
receipts of completed scoped tests, not install or production health checks.

## Data and resource contract

The current API renders only a validated flat batch: at most 100,000 records,
each eight little-endian f32 values (center, half extents, RGBA), 32 bytes.
It rejects invalid geometry/colors/counts/ranges before mutation or GPU upload.
Updates/replacements advance checked data versions; picking tokens bind slot,
version and generation. Last drawn overlap wins. Applications map one-based
slots to their business identifiers; arbitrary semantic reordering is not implied.

DPR is bounded to 1–2 and physical dimensions to device limits; zero-size or
detached canvases suspend. There is no built-in animation loop or device sharing.
Dispose the old owner before configuring a replacement on the same canvas.
`SceneOwner::dispose` and `Lifetime::retire` report `true` on the first
retirement, `false` on repeated calls. This is logical retirement and
owned-renderer drop, not permission to free a still-borrowed Wasm wrapper.
Keep the Rust/Wasm object alive until borrowed initialization/completion futures
settle, then free the wrapper once. The actual consumer demonstrates this rule.
The crate unconfigures only its own configured context before dropping resources.

CPU data, picking and viewport validation expose domain-local typed
`SceneError` variants; error text is converted to bounded `JsValue` only
at the browser/Wasm boundary. This does not introduce a shared framework
error type or change the Browser WebGPU resource-failure categories.

Unavailable capability and GPU errors must leave HTML data/actions usable.
The fault sink receives bounded categories tagged with the generation, not raw
HTML. Recovery is explicit and bounded, not an automatic retry loop. Controlled
`device.destroy()` and held queue completion are test injections, not observations
of spontaneous hardware failure. Byte counters are not resident VRAM measurements;
queue completion is not physical presentation or FPS.

## Qualification and release

[YT-079](../../docs/YT-079-VERIFICATION.md) and its
[independent review](../../docs/YT-079-INDEPENDENT-REVIEW.md) bind the actual
candidate and consumer evidence. [YT-080](../../docs/YT-080-VERIFICATION.md)
records final source/artifact correspondence, default-profile observation,
CPU/API/dependency checks and the separate final independent-review verdict.
Consult that source-bound record rather than inferring support from a version
number or from a different browser configuration.

No release, registry installation, publication, source push, API stability or
production support is authorized by this document. The original nine release
candidates and YT-020's blocked release policy are unchanged.
