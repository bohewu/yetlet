# YT-192 — 3D physics laboratory: minimum contract and feasibility

Date: 2026-10-26 (Asia/Taipei). Status: **Done at documentation/source-level feasibility scope after independent
Reviewer PASS, zero issues/required followups.**
Scope: documentation and source-level feasibility only. Nothing below is an
implemented package, runnable sample, new GPU support claim or execution receipt.

## 1. Authority and architectural boundary

The maintainer selects a minimum 3D physics-laboratory contract: evaluate wgpu
and Rapier, choose a browser target, scene bounds, time stepping and GPU-failure
behavior, and register Backlog/Roadmap. Keep everything opt-in and
application-owned; no implementation, Commit/Push or publication. Independent
Reviewer acceptance is required before this research item is Done.

Starting source is `40bfded8e0839c8f8acce86c1bdaaf85944c8c3b` on
`docs/yt113-123-opt-in-capability-plan`, clean after the separately authorized
twelve-document YT-191 commit. Its strict/builtin check and hook returned 0,
with three warning-only unchanged Windows example paths and no blocking finding.
No Push occurred; earlier evidence and decisions retain their original scopes.

Follow [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md): HTML
controls and semantic content stay separate from GPU rendering. The existing
[yetlet-wgpu contract](../crates/yetlet-browser/guides/wgpu-support.md) supports bounded flat
rectangles only. Its Batch, packing, picking and SceneOwner APIs are not 3D
interfaces. A proposed private example, `examples/physics-lab`, may later own its
3D pipeline and physics world in a separate unpublished workspace. That path
does not exist and is not added to the runnable sample catalog by this study.

No Core/default server/DOM dependency, public feature, common scene graph,
Component trait, generic game engine, AOP, Settings service or renderer API is
selected. Reuse lessons from the existing
[scene-island consumer](../examples/webgpu-scene-island/README.md), including
explicit assets, scoped lifetime and HTML fallback. Its current loader cannot
be assumed to operate a new 3D runtime. A second consumer may inform future
extraction; it does not grant extraction approval now.

## 2. Technology evaluation and exact candidates

| Component | Candidate and evidence | Decision and remaining proof |
|---|---|---|
| GPU drawing | Existing `wgpu =27.0.1`, wasm-bindgen `=0.2.129`, web-sys `=0.3.106`, Rust/Cargo 1.99; current manifest and cached upstream source expose programmable vertex/fragment and depth-stencil pipeline state | Use a private indexed/instanced 3D pipeline with camera transforms and depth. Reuse the current browser-only `std`, `webgpu`, `wgsl` selection; no upgrade, native backend or GPU physics required |
| CPU physics | crates.io metadata identifies non-yanked `rapier3d =0.36.1`, Apache-2.0, declared Rust minimum 1.86, defaults `dim3`, `f32`, `std` | Proposed exact pin, explicit `default-features = false` with `dim3`, `f32`, `std`. Single-thread CPU rigid bodies/colliders; no parallel, simd8, serialization, soft-body or profiler feature. This selection does not disable every internal SIMD/scalar implementation |
| State/render bridge | Application-owned stable object IDs map to Rapier handles; poses feed the private renderer | No raw physics handles or wgpu types become a Yetlet API. Do not reinterpret the existing flat Batch stride, normalized geometry or slot identity |

The named wgpu baseline is already selected by the existing
[renderer manifest](../crates/yetlet-browser/Cargo.toml) and
[consumer manifest](../examples/webgpu-scene-island/Cargo.toml). The cached
27.0.1 package manifest declares Rust 1.88; its RenderPipelineDescriptor includes
depth-stencil state. This is source evidence for a plausible private pipeline,
not a fresh compilation or browser proof. No floating `latest` or root lock
change is selected.

Rapier's [official setup guide](https://rapier.rs/docs/user_guides/rust/getting_started/)
separates Rust 3D physics and optional features. The exact candidate/features,
license and declared compiler minimum come from
[registry metadata](https://crates.io/api/v1/crates/rapier3d/0.36.1), read on the
study date and retained locally. Rust minimum metadata is not proof that the
entire transitive graph compiles for this Wasm target. No Rapier source package,
lockfile, dependencies, npm binding or executable was fetched/built by this work.
Current upstream guides identify the 0.36 family; do not transplant older
simd-stable flags or older pipeline signatures from unrelated examples.

**Feasibility conclusion: conditional GO for a later isolated prototype.** The
libraries have complementary responsibilities and a plausible matching Rust
baseline. Exact joint dependency resolution, Wasm build, bundle cost, browser
adapter/device behavior, numerical behavior and scene responsiveness are
unverified. Resolve/pin the example's own lock and perform those new checks only
after separate implementation authority; do not present this study as their PASS.

## 3. Browser and opt-in target

| Selection | Contract |
|---|---|
| First GPU candidate | Ubuntu 26.04.1 desktop, X11, Intel UHD 770/Vulkan path, Firefox 154.0 as observed in currently installed application/platform text metadata. This is the intended first new proof target, not qualified support |
| Historical reference | YT-079/080 qualified Firefox 158 with selected Intel/X11 settings. The installed 154 metadata differs; historical 158 PASS does not attest to 154 or this new pipeline. Do not relabel or rerun those old receipts |
| Profile and capability | One disposable Firefox profile may explicitly select `dom.webgpu.enabled=true`, following the earlier narrow opt-in approach. Require a secure-context loopback origin, available navigator.gpu and successful adapter/device acquisition. No blocklist, certificate, CSP or security bypass |
| Other paths | Default-profile unavailable behavior is a fallback case. Chrome/NVIDIA, Windows, Safari/mobile, native graphics backends and other adapters remain outside the first implementation target |

Only installed text metadata was read; no browser, driver or GPU was launched.
Before a later run, record actual executable/build, OS/session, adapter/backend,
driver and negotiated limits. If the declared Firefox/Intel/X11 candidate cannot
be selected, do not substitute another browser or claim old support; retain the
blocked/unavailable result and review a revised target before another path.
No browser installation, update, preference or system-driver change is authorized
by this document. One unavailable candidate is not permission for an environment
search campaign.

The proposed application's native default serves useful SSR without a physics
or GPU normal dependency. Browser code is separately built behind the example's
own default-off `browser` feature, with versioned same-origin assets. A small
loader and explicit Activate control own the per-canvas opt-in; ordinary page
load must not fetch/initialize Rapier/wgpu Wasm or request a device. No new ambient
environment key or Settings-loader requirement is selected here.

SSR offers a heading, initial scenario/parameter summary and object table, plus
native GET forms to select the bounded preset inputs. No JS, missing assets,
unavailable GPU or failed activation must leave those initial data/actions
readable. Live Run/Step controls are unavailable until activation succeeds;
the SSR initial table is not a pretend live simulation. Browser controls use
semantic HTML buttons/inputs/status, not canvas-rendered text controls.

## 4. Minimal experience and scene limits

The first scenario is an explicitly synthetic educational sandbox, not a
safety/engineering simulator. It demonstrates gravity, cube/sphere collisions,
an orbit camera, object inspection and pause/step/reset. It has no networking
per tick, authoritative multiplayer, database, saved/replayed physics snapshot,
model upload, arbitrary shaders, joints, soft bodies, fluid, terrain or editor.

All following numbers are selected initial scope/resource bounds, not measured
capacity, FPS, precision or hardware/SLO guarantees. Reject invalid inputs before
constructing/replacing a world; do not silently truncate bodies or clamp bad data.

| Area | Selected bound/default |
|---|---|
| Dynamic objects | Default 24; integer count 1–64. Cubes and spheres only, one collider/body each, built in a fixed preset order with non-overlapping initial placement |
| Static enclosure | One floor plus four walls, one body/collider each: at most 69 bodies/colliders per world. No runtime spawning/deletion in version one; count changes rebuild while paused |
| Units and geometry | Meters, seconds, kilograms; right-handed world with Y up. Cubes have full edge 0.5–1 m; sphere radius 0.25–0.5 m; each dynamic mass 1 kg, initially at rest. Preset centers fit x/z ±8 m and y 1–8 m inside the enclosure |
| Gravity and time scale | Downward gravity magnitude 0–20 m/s², default 9.81. Speed enum 0.25×, 0.5×, 1×, 2×, default 1×. Apply parameter/count changes only while paused; validate the complete selection before reset |
| World validity | Finite transforms/velocities required. A coordinate outside ±100 m, or an invalid/nonfinite state, pauses with a named invalid-simulation status. Keep the last valid display snapshot; do not repair corrupted state silently |
| Physics settings | Version-pinned Rapier defaults otherwise; dynamic-body CCD enabled, CCD substep limit 1. This engine setting is distinct from the four scheduler steps per callback below |
| Rendering | Two shared fixed primitive meshes, at most 2,048 triangles combined; simple directional shading/depth, no textures/shadows/MSAA/post-processing. Application-owned mesh/index/instance/camera buffers total at most 1 MiB of explicit requested bytes |
| Canvas | Default 640×400 CSS pixels; effective DPR 1–2, negotiated device limit, each physical dimension ≤2,048 and total physical area ≤2,097,152 pixels. Reduce rendering resolution to meet the budget without mutating CSS layout or physics; zero/detached surfaces suspend |
| Retained state/UI | One current physics world, current and last-valid pose snapshots, one selected stable object ID, fixed scalar counters and last status; no unbounded contact/frame history. Live HTML table/detail refresh at most 4 Hz; status changes may announce immediately without continuous live-region chatter |

A paused atomic Reset may temporarily stage one replacement world before
swapping/dropping the old one: at most two worlds and 138 bodies/colliders during
that transition. Each world separately obeys the scene limits; failed construction
keeps the prior world unchanged. No replacement queue is retained.

Pixel/buffer counts do not measure resident VRAM or total process/Wasm memory;
Rapier and the browser also allocate internal resources. No hard whole-process
memory bound follows from these limits. A bounded body count is not a measured
throughput result.

Use the same camera/projection and current pose snapshot for rendering and
selection. The private projection must explicitly match WebGPU's depth interval
0–1; orbit math/normal transforms are rendering-owned. Canvas hit selection is
available only while ready/paused, after the current scene/camera versions have
been drawn; otherwise request that draw and require a later click. A running
world must be paused before canvas picking. Cast the CPU physics ray against
that same valid stationary snapshot and map the hit to its stable application
ID. This avoids querying a newer world while a paced frame still shows an older
pose. Do not read a GPU picking buffer every frame or imply that the existing
flat rectangle picker works in 3D. Keep an HTML object selector
and numeric camera controls/reset available for keyboard use; pointer drag/wheel
are additional local controls. No drag-body manipulation is selected.

## 5. Time stepping and UI actions

Use a fixed physics step `dt = 1/60` second; RAF is a scheduling opportunity,
not the physics clock or proof of presentation rate. Rapier's
[integration guidance](https://rapier.rs/docs/user_guides/rust/integration_parameters/)
explains dt, human-scale units and CCD settings. The bounded accumulator policy
below is this application's selection, not a Yetlet scheduler or Rapier default.

The world starts **paused**, even after successful GPU activation. Run/Resume
starts a fresh monotonic timestamp; do not charge initialization/pause time.
While running and visible, each callback:

1. Reject nonfinite/backward timestamp input by pausing; otherwise compute the
   elapsed wall delta and update the previous timestamp.
2. Admit at most 100 ms of that delta, multiply it by the selected speed and
   add it to the simulation accumulator. Count the discarded scaled excess
   separately as scheduler-discarded simulated time.
3. Execute at most four complete steps of exactly dt, decrementing the
   accumulator and advancing the checked tick count only for completed steps.
4. If one or more whole steps remain after the cap, discard those whole steps
   and count their time; retain only a remainder smaller than dt. Never carry
   an unbounded catch-up queue or grow dt to hide a slow frame.
5. Publish the valid current poses and draw at most once. Rendering may be
   paced to at most 60 submissions/second; skip submissions between pacing
   points without changing the step size or claiming a 60-FPS result.

Simulation time is completed ticks × dt, not wall time. Discard counters describe
scheduler truncation, not all paused/hidden time or engine CCD time loss.
No pose interpolation is selected initially: displayed/pickable poses are the
same latest valid completed step. Fixed stepping helps comparison, but it is
not a cross-device bitwise replay guarantee. Rapier's
[determinism contract](https://rapier.rs/docs/user_guides/rust/determinism/)
requires additional conditions for cross-platform identity; enhanced-determinism
and such a guarantee are not selected by this prototype.

| Action/condition | Defined behavior |
|---|---|
| Pause | Stop physics and recurring RAF; clear accumulated debt and previous wall timestamp. Retain world/poses; a camera-only interaction may request one draw without ticking |
| Single Step | Only while ready/paused: execute exactly one dt independent of speed/wall time, update table/draw, remain paused; never count it twice through RAF |
| Reset / Apply | Only while paused: validate all selected inputs, construct the bounded preset, replace the world atomically and advance the scene generation. Zero tick/drop counters and remain paused; invalidate old handle/selection callbacks |
| Hidden, out of view, zero-size or detached canvas | Automatically pause/suspend, clear timing debt and cancel recurring RAF. Becoming visible does not resume physics; explicit Resume begins from a new timestamp |
| Invalid physics state | Freeze the last valid snapshot with an error status, stop ticking and require explicit Reset. Never continue stepping a corrupted world or label its current state valid |

Input handlers do not execute inside a physics step. Parameter drafts remain
local until Apply; disable duplicate pending Apply/Reset/Step/activation actions.
Keep at most one pending state-changing action, not an unbounded event queue.
A stale scene-generation action is rejected. Reload/navigation ends tab-local
simulation; there is no persistent recovery or automatic remote write/retry.

## 6. GPU failures, ownership and recovery

WebGPU device loss can occur even after acquisition; Mozilla's
[GPUDevice.lost documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost)
describes the lifetime promise and secure-context boundary. Initialization and
runtime failure handling are part of the proposed app, not proof supplied by
support/version metadata. Preserve the existing strict CSP pattern: trusted
build-time WGSL and same-origin hashed JS/Wasm; no inline script, eval, remote
shader text or secret-bearing hydration state.

| Condition | Required app behavior |
|---|---|
| No WebGPU/insecure context/null adapter/device-request rejection | Named unavailable state, no ticking, initial HTML/preset controls intact. No hidden software/native fallback or repeated device requests |
| Loss/validation/out-of-memory/surface configuration or submission error | Pause physics, stop recurring draw work, retain the CPU world and last valid snapshot, and show a finite safe error category. Never claim the last frame is current or continue invisible background simulation |
| Resize to zero/detach | Suspend without configuring a zero-sized target; retain CPU state. A valid size can restore drawing readiness but not automatically resume time |
| Explicit Retry | One attempt per deliberate action, only after the prior owner is retired and outstanding borrows settled. Create a fresh GPU generation, re-upload the retained valid state and return paused on success; no automatic retry/backoff loop |
| Dispose/navigation/replacement | Invalidate generation first, stop RAF, detach owned observers/listeners, retire GPU resources, then free a Wasm wrapper only after its borrowed async futures settle. Reset/dispose invalidates old picks and control callbacks |
| Late init/completion/fault | Check both live owner and generation; dispose a late-acquired device/resource rather than publishing into a replacement canvas, advancing physics or clearing a newer fault |

CPU physics ownership is independent of replaceable GPU resources. A GPU Retry
must not implicitly reset/replay the world or advance simulation time. Invalid
physics blocks Retry-as-repair: explicit validated Reset is required before a
new renderer can use it. Keep failed old-generation callbacks from overwriting
that decision. At most one live owner and one queued draw belong to the canvas;
initialization/cleanup holds no permission to mount a second owner prematurely.

A returned GPU queue completion is not presentation, FPS or successful delivery
to the user. Any later controlled loss injection must be labeled an injected
case, not a claim to reproduce spontaneous driver/hardware failure.

## 7. Later implementation admission and minimum proof

This research can finish with a reviewed contract while implementation proof
remains Not run. A separately approved next slice would first prepare the new
isolated example's exact lock/feature graph and source-bound browser target,
then implement only the bounded scenario and its HTML/physics/GPU boundary.
No automatic activation of another backlog item or old one-shot scripts follows.

| Future focused proof, only after implementation authority | Required observation |
|---|---|
| Compile/graph | Exact Rapier/wgpu candidate compiles to Wasm with the selected binds; normal native SSR has no physics/GPU edge; no root/old renderer lock migration |
| Input/clock | Boundaries rejected without partial mutation; fixed dt, max-four catch-up, counted dropped debt, Pause/Step/Reset/speed/background transitions match this contract |
| Physical scenario | Selected floor/cube/sphere cases produce finite bounded states with expected gravity/contact behavior and declared numeric tolerances; no claim of engineering-grade accuracy |
| Actual 3D interaction | Depth/camera/occlusion and rendered cube/sphere positions match the new source-bound scenario; object selection maps the rendered pose to the correct HTML ID/detail |
| Fallback/lifecycle | No-JS/assets/unavailable paths keep SSR usable; new consumer handles loss, late completion, repeated activation, explicit retry, disposal and sibling isolation with owned cleanup |
| Evidence limits | Report actual executable/adapter/version, failures and cleanup; only observed cases pass. Record CPU-step/draw costs or bundle bytes if collected, without a benchmark campaign or inferred FPS/support |

No new harness framework or exhaustive browser matrix is selected. Future
acceptance can use focused product tests and the existing build/browser workflow
with new source-bound outputs; do not overwrite/replay completed M11 receipts.
If a version/target is unavailable, the result stays unavailable rather than
prompting an unbounded upgrade, shader/backend search or new infrastructure task.

Open execution facts: exact joint dependency resolution and Wasm compile; actual
Firefox154/Intel/X11 availability and successful adapter/device negotiation;
contact tolerances, resource costs and measured responsiveness. These are
implementation prerequisites/results, not unresolved choices of this contract
or reasons to pretend that today's research ran the prototype. No SLA, arbitrary
browser support, full physics engine, crate extraction or release approval follows.

## 8. Research verification and acceptance record

Only four Markdown paths are selected: this study, [Backlog](BACKLOG.md),
[Roadmap](ROADMAP.md) and the [repository index](README.md). Reader publication
selection, root product README, current package/support docs, ADR decisions,
source/manifests/locks/templates/tests and historical proof stay unchanged.

Evidence under `target/yt192-physics-lab-study-20261026/evidence/` retains the
pre-edit contract, clean source hashes, registry metadata, installed-text browser
snapshot and source/primary-reference notes. docs.rs version-page reads were
unavailable through the web tool and the full WebGPU specification exceeded its
response limit; no dependency/tool installation or validator repair followed.
Use cached versioned wgpu source, official registry metadata and primary project/
browser documentation instead; tool limitations are not compiler/runtime results.

Only new-delivery `cargo xtask check docs`, whitespace and bounded scope/source
preservation checks are selected. No build/test/rustdoc/browser/GPU/service/cloud
or old qualification run is performed. At the initial draft checkpoint,
independent acceptance was pending; the completed result is recorded below.
No new sample/runtime, stage/Commit/Push or publication. YT-096/D1–D6 and YT-020 remain unchanged.


### Initial document checks

New-delivery `cargo xtask check docs` returns 0: 75 required documents,
192 backlog items, internal links/Markdown/whitespace/DAG PASS. Tracked
`git diff --check` passes; the new study's no-index whitespace diagnostics are
empty (exit 1 denotes the content difference). Exactly four Markdown paths
change, staged zero, and all 1,109 non-target tracked hashes remain unchanged.
Sixteen repository references plus two cached upstream source files bind the
baseline; registry metadata confirms the selected Rapier version/features.
These are source/document checks only. Compile, browser/GPU, physics behavior,
resource/performance and new-platform qualification remain Not run. Independent
read-only review was the remaining acceptance step at this draft checkpoint.


### Independent acceptance and closeout

Independent Reviewer `/root/yt192_physics_lab_contract_independent_review_20261026`
returns PASS, zero issues, required followups and optional notes. Native trace
matches registered reviewer/model/effective effort under the saved healthy,
current and eligible workspace profile and the central resolver. Four reviewed
source hashes, nine evidence hashes and all 1,109 non-target tracked hashes
remain unchanged before/after review; HEAD/branch/status and staged-zero match.

Only this acceptance/status paragraph and Backlog/Roadmap/index completion
metadata follow PASS. Contract sections 1–7 remain byte-identical to the reviewed
text. Final new-document/whitespace and preservation checks are retained beside
the initial check, profile/resolver/trace and accepted result.

YT-192 is Done at documentation/source-level feasibility scope. The selected
versions, scene bounds, browser candidate, fixed-step and loss/recovery rules
are a reviewed proposal for a separately authorized example. No dependency
resolution/lock generation, compiler/physics/browser/GPU proof, sample/runtime,
new tests, old qualification replay or environment change occurred. Research
completion does not mark the proposed implementation or an unqualified target
Done. No Core/flat-renderer API or original release scope changes.

Four Markdown changes remain unstaged/uncommitted/unpushed/unpublished.
Next possible work is the separately authorized minimal isolated prototype in
section 7; it is not activated here. YT-096/D1–D6 and YT-020 remain unchanged.
