# Experimental 3D physics laboratory

A separate, application-owned prototype: native HTML describes a synthetic
preset; the first explicit Run loads a private wgpu pipeline and Rapier CPU
physics. No existing Yetlet renderer API changes. This is not a game engine,
engineering/safety simulator, production feature or performance claim.

## Prerequisites

Use this checkout with Rust/Cargo 1.99.0. This is a standalone nested workspace
with its own Cargo.lock, not an addition to the default workspace or sample
catalog. Native SSR needs no GPU, physics feature, Node or WASM tools. The
optional surface needs wasm32-unknown-unknown, wasm-bindgen-cli exactly 0.2.129,
a secure-context loopback origin and a working WebGPU adapter/device.
Scene bounds are in [the reviewed contract](../../docs/YT-192-3D-PHYSICS-LAB-STUDY.md).
Your usual browser may open the page; Wayland is not an application prerequisite.
Actual 3D operation needs the capabilities below. The isolated Firefox/Wayland
receipts in YT-193/194 are bounded observations, not an all-browser support claim.

Explicitly prepare this workspace's locked dependencies once:

```sh
cargo fetch --locked --manifest-path examples/physics-lab/Cargo.toml
```

Offline/frozen commands require that prepared cache. No tool is installed by the
server or asset builder. If needed, prepare the matching bindgen executable in
a local tool directory, without changing the system installation:

```sh
cargo install wasm-bindgen-cli --version 0.2.129 --locked --root "$PWD/target/physics-tools"
```

## Minimal integration

From the repository root, native default serves the initial preset and GET form:

```sh
cargo run --offline --locked --manifest-path examples/physics-lab/Cargo.toml
```

Open `http://127.0.0.1:3016/physics-lab`. Stop with Ctrl+C. The native default
normal dependency graph contains neither Rapier nor wgpu. No Session, Auth,
Settings loader or service is installed by this prototype.

For the explicitly selected browser surface, build separate immutable assets:

```sh
WASM_BINDGEN="$PWD/target/physics-tools/bin/wasm-bindgen" cargo run --offline --locked \
  --manifest-path examples/physics-lab/Cargo.toml --bin build-assets
```

The builder requires the exact tool version, fixes its own output directory
regardless of inherited CARGO_TARGET_DIR, runs a frozen release Wasm build
with the example's `browser` feature, then publishes a content-addressed bundle
and atomically updates its manifest only after success. Restart the native
server to select that bundle, then click **Run**. Source changes require
an explicit rebuild; neither request handling nor activation compiles code.

## Expected result

Without JS/assets/WebGPU, the heading, initial object table and bounded native
GET preset controls remain readable. Run/Step are unavailable without assets/JS.
Opening the page alone loads no physics/GPU Wasm and requests no device.

- **Run** prepares 3D on the first click, shows Preparing while it loads, then
  starts the default24-body scene. Later Run continues the same paused world.
- **Pause** keeps the current world and clears timing debt. During preparation,
  it cancels the pending start; initialization may finish into a paused scene.
- **Reset** validates the current count/gravity/speed, pauses a running world,
  rebuilds that preset and returns paused with zero tick/drop counters. Before
  first Run it uses the native GET form without initializing Wasm. Reset is
  unavailable while preparation or cleanup is pending.

Pointer drag orbits the camera; the wheel zooms. Pause before clicking an object
in the canvas; its HTML selector/detail identifies it. The HTML object selector
also works without pointer hit testing. Live data updates at most four times
per second. **Advanced controls** contains Single Step (exactly1/60s while paused),
numeric camera controls and diagnostic counters; it starts collapsed.

**Retry** appears only after a preparation/GPU failure. It makes one explicit
attempt, preserves a valid CPU world and returns paused; press Run to continue.
Retry also permits another preparation attempt if failure preceded world
creation. Invalid physics still requires Reset. There are no Activate3D or
Dispose buttons; page navigation owns internal GPU retirement. Preparation
observers cancel pending start when hidden/offscreen/zero-size/detached, and
visibility never automatically resumes a world. Reload ends all tab-local state.
There is no server mutation or snapshot persistence.

The [YT-194 action contract](../../docs/YT-194-PHYSICS-LAB-CONTROLS.md) supersedes
YT-193's initial Activate-then-Run interaction only. Its historical acceptance,
scene bounds, WebGPU target limitations and prior failures remain unchanged.

## Scene presets and FPS

Choose a Scene: **Falling objects** retains the original mixed drop;
**Stack impact** alternates three-block layers and drops an amber striker onto
blue boxes (count1 has one box); **Domino chain** places boxes on a bounded arc,
with the amber first domino leaning/moving toward the next. **Slope comparison** places balls
and boxes on a fixed25degree ramp: blue bodies use friction0.1,amber0.9,ramp0.2
with Rapier Average combining. Use at least4objects for all four lanes; smaller
counts use a prefix. The active slope legend follows the actual world,not a staged
selection. Rougher boxes can hold while balls roll; this is an educational
contact comparison,not an engineering/material calibration. Count includes the
striker/first domino and remains1–64,with five fixed bodies. Slope reuses the uphill wall slot for its ramp and retains
the landing floor,downhill wall and both side walls; other presets keep their
original floor/four walls. Existing world bounds/invalid-state rules still apply.

The selector and other preset inputs stage the next world. **Reset** applies
those controls and pauses; changing the selector alone preserves the current
world. The first Run uses the currently selected settings. Before Wasm, native
GET Reset renders the same selected scenario/table without starting3D. Shapes
are labeled Box/Sphere; initial rotations and velocities agree with Rapier.

The canvas HUD shows the active scene, **FPS (estimate)** and current world
object count. It measures successful running draw submissions over at least one
second of real monotonic time, not GPU presentation/completion or fixed60Hz
physics. Warmup shows Measuring; Pause/Step/suspension show Paused. A stopped or
reset world clears previous samples. No extra sampling loop, benchmark target or
hardware guarantee is added; existing DOM updates remain at most4times/second.
Details and scoped new acceptance are in [YT-197](../../docs/YT-197-PHYSICS-LAB-SCENARIOS-FPS.md).

## Browser startup and failure help

Open the default URL on the machine running the server. If the server is on
another machine,127.0.0.1 in your browser still means your own machine: set up
an intentional local port forward to the server's loopback listener, or use a
separately provided trusted HTTPS deployment. This example does not install a
proxy/TLS service or bind a public interface for you.

JavaScript/WebAssembly and available WebGPU are required for3D. WebGPU needs a
[secure browser context](https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Secure_Contexts),
such as local loopback/localhost or trusted HTTPS; ordinary LAN HTTP is not a
substitute. [API exposure](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/gpu)
does not prove that the browser will provide an adapter/device. No specific
browser version, default adapter or forced graphics flag is assumed.

Click Run once. The canvas shows a local loading indicator while the accessible
status describes preparation. Browser help is available even before Wasm or if
JS is blocked; noscript explains disabled JavaScript. The indicator is decorative,
uses no fictitious percentage and stops animating for reduced-motion preference.
The shared `createStartupShell` asset manages the regional loading/ready/error
presentation, while this loader keeps actual GPU/CPU lifetime and Retry guards.
The application-owned overlay/initial markup and shared
`--wasm-startup-accent` / `--wasm-startup-track` CSS variables can be restyled.
See [WebAssembly startup](../../docs/WASM-STARTUP.md) for the same helper's
full-page browser-template use and caller-owned async lifetime contract.

| Message category | Next action |
|---|---|
| 3D files unavailable / disabled Run | Check JavaScript/site-module permissions; the server operator builds the matching assets using the command above and restarts the server. Native table/Reset remain useful |
| Secure context required | Use local loopback/localhost or a trusted HTTPS endpoint; then choose Retry |
| WebGPU unavailable | Inspect browser/platform capability and graphics diagnostics, then retry only after resolving the reported restriction |
| Program load/start failure | Check server JS/Wasm files, matching asset manifest, MIME types and allowed module/Wasm execution; then Retry |
| Adapter/device/canvas preparation failure | Inspect browser graphics diagnostics; API presence alone does not establish device access; then Retry |
| Timeout | Pending cleanup keeps Retry disabled. If it remains stuck, reload, accepting loss of tab-local simulation |
| Device loss/render failure | The valid CPU state is retained; explicit Retry restores a paused scene. Invalid physics requires Reset instead |

Firefox diagnostics are at about:support; Chrome diagnostics are at chrome://gpu.
These are pages to inspect, not browser policies to bypass. Keep certificate,
secure-origin, sandbox and GPU blocklist protections intact. The application
never edits your browser preferences or selects an alternate backend silently.

The UI uses bounded category messages, opens Browser help on failure and avoids
raw exceptions/URLs/stacks. The developer diagnostic seam remains local to this
prototype. [YT-195](../../docs/YT-195-BROWSER-STARTUP-GUIDANCE.md) records what is
newly checked; earlier Run/Pause/Reset/physics/GPU qualification is not rerun.

## Configuration and errors

The example alone reads `YETLET_PHYSICS_LISTEN` (default 127.0.0.1:3016, loopback
required) and `YETLET_PHYSICS_ASSETS` (default its target/dist). These are not Core
settings keys or capability switches. The native server loads/validates its six
content-hashed assets at startup; missing/corrupt assets keep native SSR available.

GET inputs: scenario falling/tower/dominoes/slope (default falling), count 1–64 (default24), gravity 0–20 m/s² (9.81), speed one of
0.25/0.5/1/2 (1). Unknown/invalid inputs reject with 400. Physics has five fixed
static bodies, fixed 60Hz dt, max four scheduler steps/callback, 100ms elapsed
clamp and counted discarded debt. Hidden/offscreen/detached/zero-size canvases
pause; visibility never automatically resumes. Invalid physics retains the last
valid snapshot and requires Reset. No automatic retry or catch-up queue exists. GPU acquisition has a 15-second
UI deadline. A timed-out borrowed initialization is retired but never freed
prematurely; further activation/retry stays disabled until it settles. If it
does not settle, reload instead of creating a second pending owner.

GPU-owned primitive/transform/camera buffers are bounded to 1MiB requested bytes;
canvas dimensions/area and scene counts follow the contract. Those values are
not resident VRAM, a whole-process memory cap or a measured performance limit.
Only trusted built-time WGSL and source-hashed same-origin JS/Wasm are used.
The narrow wasm-unsafe-eval CSP token enables Wasm; unsafe-inline/eval and
arbitrary shader/module input remain unavailable.

`window.physicsLabDiagnostics` is an application-local observation/control seam
for this disposable prototype: snapshot, explicit controlled device loss,
generation-tagged injected fault and disposal. It is not a framework API,
authorization boundary, deployed operations endpoint or spontaneous-failure proof.

## API reference

The private implementation is [preset/clock](src/model.rs), [physics](src/physics.rs),
[browser renderer](src/gpu.rs), [page controller](loader.js) and [SSR](src/server.rs).
The `browser` feature is default-off; wgpu/Web API dependencies are wasm-only,
while optional CPU tests explicitly select physics through that same feature.
Default native execution never creates a physics world/device.

[YT-193 delivery](../../docs/YT-193-3D-PHYSICS-LAB-PROTOTYPE.md) records actual new
verification, failures, browser/adapter identity and cleanup; instructions above
are not acceptance evidence. Original M11/flat-renderer evidence stays unchanged.


YT-196 introduced shared JS/CSS in the five-file bundle; YT-197 adds the private
frame-rate module as its sixth hashed file and the example-private scenario
argument. Rebuild assets and restart the server after upgrading. Legacy bundles
are rejected by the new six-file loader and keep native SSR available.
This is an unpublished application asset migration, not a renderer/Core API
change or a replay of historical GPU qualification.
