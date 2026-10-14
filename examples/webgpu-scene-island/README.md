# YT-075 — Optional browser work-record overview

This unpublished application fixture is an isolated workspace. It contains 120
explicitly synthetic records, with one row-major rectangle per record: red
blocked, green done, blue active. Search, pagination, record detail and reset are
ordinary server-rendered GET controls. No Board storage or remote event circuit
is used. The optional canvas reserves a 600×400 CSS viewport. YT-076 adds
bounded resize/DPR, pause, loss/recovery and teardown behavior; its selected
[Firefox browser and independent review gates passed](../../docs/YT-076-VERIFICATION.md). Picking, performance and public renderer
extraction remain later tasks. **Selected Ubuntu Firefox 158/X11 browser runtime qualification
has passed** with the actual Rust/wgpu Wasm: 120/120 screenshot pixel centers,
owner retirement/replacement, negative fallbacks and independent counter.
Linux Firefox Stable requires `dom.webgpu.enabled=true`, enabled only
in the disposable test profile. A fresh [independent review](../../docs/YT-075-INDEPENDENT-REVIEW.md)
returned **PASS (zero blocking issues)**; YT-075 is Done at this selected scope.
Earlier Chrome/NVIDIA and Firefox headless failures are retained and do not
extend the supported browser scope. See [selected Firefox evidence](../../docs/YT-075-UBUNTU-BROWSER-VERIFICATION.md) and
[current verification](../../docs/YT-075-VERIFICATION.md).

From the repository root, using the installed Rust 1.99 and wasm32 target:

```sh
cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml
cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml
python3 examples/webgpu-scene-island/build.py
YETLET_SCENE_LISTEN=127.0.0.1:3010 cargo run --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml
```

Open `http://127.0.0.1:3010/scene`. The server rejects non-loopback bind addresses.
It reads `target/webgpu-scene-island-dist` by default; an explicit
`YETLET_SCENE_ASSET_ROOT` can select another local asset directory. Stop with
Ctrl-C. Without a manifest or usable assets, the semantic record workflow stays
available. GPU activation is local and explicit; loading the page loads only the
small scene loader and the independent counter. Scene JS/WASM load on activation.
Use Dispose before activating a replacement. After loss/error/unavailability,
Activate performs one explicit retry with a new generation.

The fixed build procedure uses the nested lockfile and exact cached wgpu 27.0.1,
wasm-bindgen 0.2.129, web-sys 0.3.106 and matching bindgen CLI. wgpu defaults are
disabled; only `std`, `webgpu`, `wgsl` are enabled for wasm32, with no native
backends. Cached version-source verification showed `webgpu` enables the browser
bindings and `wgsl` gates `ShaderSource::Wgsl`. Native SSR has no wgpu edge.
`build.py` builds the scene and existing counter independently, hashes each
entry's loader/generated JS/WASM together, and writes exact local manifest paths.
Do not hand-edit generated assets or reuse an old manifest after source changes.
The root Cargo.lock and Todo/Search build procedure remain independent.

### Selected Ubuntu Firefox browser proof

The existing selected Ubuntu Firefox X11 hardware run was:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_firefox.py
```

This **already completed successfully** with the actual application-generated
Wasm and independently hashed assets; do not overwrite or automatically
repeat its local receipt at
`target/m11-gpu/yt075-20261014/firefox-158-x11-window-owner`.
The single-use WebDriver harness uses a disposable Firefox 158 profile with
only the nondefault experimental `dom.webgpu.enabled=true` preference,
serves the real application SSR on loopback with a temporary copy of the
built assets, and verifies 120 center pixels, black canvas interior,
real delayed-device lifetime, four injected GPU faults, two asset failure
cases, counter isolation and cleanup. Firefox screenshots include a
one-row white *capture edge*; real canvas clear was black one pixel inward.
Linux Firefox default Stable, Chrome/NVIDIA, native Wayland and other
GPU/driver configurations remain outside this selected proof.

The canonical browser command is retained below for a separately selected bounded
continuation. The current attempts are complete; do not rerun automatically or
overwrite their evidence:

```sh
python3 examples/webgpu-scene-island/verify_browser.py
```

The separately authorized NVIDIA-only Vulkan ICD diagnosis can be selected
with `YT075_NVIDIA_ICD_DIAGNOSTIC=1` for this host; it changes only the
owned Chrome process, enables ANGLE/Vulkan, and writes to the distinct ignored
`target/m11-gpu/yt075-20261014/browser-nvidia-icd` directory.
That exact configuration already completed once and **failed** with
`vkCreateInstance() failed: -7` and a null WebGPU adapter. It is not a
passing or supported configuration; do not automatically rerun it.

The subsequent `YT075_PRIME_OFFLOAD_DIAGNOSTIC=1` candidate keeps
default Vulkan ICD discovery, chooses NVIDIA using process-only PRIME
Offload variables and writes to
`target/m11-gpu/yt075-20261014/browser-prime-offload`. It was run once:
Chrome obtained a real NVIDIA/Lovelace adapter, but the WebGPU scene
became unavailable after Chrome logged `Error creating wgpu::Texture`.
It does not qualify canvas drawing; no automatic retry is authorized.

One process-only `YT075_INTEL_GEN12_DIAGNOSTIC=1` Chrome 155
qualification attempt selected the Intel UHD 770/Mesa OpenGL route,
preserved ordinary Vulkan ICD discovery, and wrote separately to
`target/m11-gpu/yt075-20261014/browser-intel-gen12`.
The GPU feature flags were reported enabled but WebGPU adapter acquisition
still returned `null`. This is a **failed** configuration, not proof
that Intel Gen12+ lacks WebGPU support; it must not be rerun automatically.
At that earlier Intel Chrome checkpoint, Firefox 158 Stable was installed,
but WebGPU Linux Stable was not enabled by default and no Firefox run had
been started. The later selected experimental Firefox opt-in is documented
above, separately from these retained historical Chrome failures.

The owner also installed Weston 14.0.2. One owned nested Wayland
preflight passed on the X11 backend, and a separate one-attempt browser
qualification started Weston with process-only PRIME offload. Its EGL
and GL vendors were NVIDIA, and its GL renderer was RTX 4090.
The browser fixture added
`YT075_NESTED_WESTON_DIAGNOSTIC=1`, which expects the separate owner
to start a matching socket named `wayland-yt075-weston-browser` first,
and selects Chrome `--ozone-platform=wayland` with NVIDIA Vulkan.
The browser acquired a genuine NVIDIA/Lovelace adapter but reported
`Error creating wgpu::Texture` and did not draw. This is a failed
experimental browser configuration; **do not automatically repeat**
the test or conflate the nested X11 compositor with a native Wayland
desktop session. Evidence is retained separately in
`target/m11-gpu/yt075-20261014/weston-x11-browser-launch` and
`target/m11-gpu/yt075-20261014/browser-weston-x11`.

It starts only its own loopback server, unique Chrome page and temporary profile,
and records evidence under `target/m11-gpu/yt075-20261014/browser-x11`. The selected
headless configuration uses `--enable-gpu --ozone-platform=x11` with the verified
owner X11 session at DISPLAY `:0` and its existing Xauthority. It changes no host
settings and uses no unsafe GPU/security flags. This follows
[Chromium's headless GPU guidance](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/gpu/using-gpu-hardware-in-headless-chrome.md).
Earlier no-DISPLAY/Vulkan attempts remain under `browser-initial` and `browser`;
their failures are not replaced by this selected configuration.
Required GPU capability or actual pixel failure stops
acceptance. Fault injection is labeled; it does not count as real GPU output.
Actual navigator diagnostics were recorded, but all attempts stopped at the
real-draw gate. The pixel check failed on NVIDIA/Vulkan; initial and X11 attempts
returned null adapters. Later injected errors, pending-owner retirement and
sibling assertions were not reached. The initialization validation-error fix
has rebuilt WASM/Clippy evidence, not an exercised rejection regression.
Native tests/builds do not establish browser or hardware support.

### YT-076 lifecycle continuation

[YT-076 verification](../../docs/YT-076-VERIFICATION.md) records the current
implementation/build slice and pending gates. Demand rendering coalesces resize
and invalidation into at most one RAF per owner. DPR is bounded to 1–2, physical
dimensions are clamped to the acquired device limit, and unchanged dimensions
reuse the configured surface. Zero-sized, detached, offscreen and hidden targets
suspend drawing. Immutable CPU records and semantic HTML survive device loss;
loss/error releases GPU resources without an automatic retry loop. Explicit
activation retries with a new generation. Disposal removes the owner's observers,
listeners, RAF and GPU buffer/device/context. Counters are explicit application
ownership/byte counts, not measured VRAM.

The parent owns one new focused Firefox run after source freeze:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_lifecycle.py
```

The test refuses an existing `target/m11-gpu/yt076-20261014/browser` directory.
It retains source/asset hashes, typed state counters, real resized/recovered
canvas screenshots and owned-resource cleanup, using the same temporary
experimental preference and X11 recipe as the successful YT-075 run. Controlled
real device destruction is injected loss, not hardware failure. DPR override and
held acquisition/validation results are labeled injections over real GPU resources.
Browser validation has not run for YT-076. Do not rerun or overwrite YT-075 evidence.

### YT-077 prepared dynamic comparison

YT-077 is **In progress**: implementation/build only; measurement **Not run**,
parent parameter review and independent review pending. The [measurement method
and build record](../../docs/YT-077-VERIFICATION.md) fixes the operational
parameters before execution. `/scene` remains the 120-record SSR/default entry.
The separate `/scene-comparison` loads a small external comparison script; neither
entry fetches scene JS/WASM or initializes a GPU before explicit activation.
The comparison driver explicitly selects each lane. Canvas2D and real Rust/wgpu
use identical Rust-generated, bounded 32-byte rectangles, colors, seed and updates.
Only the selected canvas paints. Scene data remains application-owned outside
LocalGraph; no renderer crate, shared scene graph or dependency change is added.

Prepared parent command (run once after parameter review, never in the leaf):

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_dynamic.py
```

The output is uniquely owned at `target/m11-gpu/yt077-20261014/browser`; the
fixture refuses an existing directory. Three alternating independent rounds
per renderer/size use 1k/10k/50k/100k records, fixed 600×400 CSS and actual DPR 1,
5 warmups then 30 measured samples for sparse 1%, full status and eight-operation
latest-state bursts. Each 10k lane also gets 10 seconds of sequential sustained
sparse updates with raw per-sample counters. The single Firefox process has an
8-minute bound, with a 420-second measurement cutoff and time reserved for
cleanup. This is an operational bound, not a user performance promise. A bounded
out lane is explicitly not measured; it cannot be reported as a loser.

Canvas2D fully clears/redraws; GPU fully clears/renders the same final state.
Sparse uploads contain only the contiguous dirty 1% range. Queue completion and
RAF opportunities are separate wall observations; neither proves physical paint,
pure GPU time or FPS. Screenshot checks run outside timed samples for both
canvases after initial, sparse, full, burst, sustained and replacement states.
Current-source raw/gzip bundle sizes, actual ResourceTiming transfer/body bytes
(including zero-transfer cached/unknown cases), initialization, CPU preparation,
bridge/apply, upload and encode/submit wall costs are separately retained.
No speedup or 60 Hz acceptance is claimed before real measurement and review.
