# M28-F2 / YT-282-D2 — Independent opt-in Surface renderer

Status: bounded source-only YT-282-D2 implemented and independently reviewed.
Independent R1/R2 actual FAIL3 and R3 actual FAIL1/P1 remain preserved and
repaired. Independent R4 `wc_job_HeT6FgnG3slurqZg` terminal exit0 returned
actual JSON PASS, `issues=[]`, `required_followups=[]`. The D2
checkpoint passed 22 renderer tests, two consumer tests and Ubuntu Vulkan
verification. The bounded YT-282-F rectangle path below retains independent R1 actual
FAIL2/P2 (peer trust and derived-edge overflow) and R2 actual FAIL1/P2
(startup peer address lookup after C1 acquisition); both were repaired.
Independent R3 `wc_job_m8Cmu3fcywPg-6P_` terminal exit0 returned actual
JSON PASS, `issues=[]`, `required_followups=[]`. The rectangle renderer
source remained unchanged after the earlier R2-source Vulkan proof.
Full YT-282, YT-283/284 and F2 remain open.
This increment follows [ADR 0013](adr/0013-composable-application-and-hosts.md)
and the separately reviewed [D1 Window slot](M28-F2-NATIVE-WINIT-ADAPTER.md).
No published SDK/CLI or reader edition changes.

This dated record retains its original companion package graph and runtime
evidence. YT-292/293 move the renderer and example into yetlet; current consumers
select `yetlet/renderer-wgpu` and select Native separately when needed. Source
links below follow the moved files.

## Selection and ownership

`yetlet-hosts` has no default features. `renderer-wgpu` independently selects
exact `wgpu = 27.0.1`, with only `std`, `vulkan` and `wgsl` enabled, matching
the private research dependency. `native-winit` independently selects Winit
0.30.13. Renderer-only normal/build dependencies do not include Winit, Yetlet,
Composition/C1, HTTP, Axum, Tokio or a UDP adapter. Vulkan's unavoidable native
baseline includes WGPU core/HAL, Ash, raw-window-handle, Naga, GPU allocation
and descriptor libraries. The target-specific WGPU support package name
`wgpu-core-deps-windows-linux-android` does not establish platform qualification.
No browser renderer, game protocol, universal Host/Renderer trait, registry,
DI, scheduler or automatically started worker is added.

[`SurfaceRenderer<'window>`](../crates/yetlet/src/renderer_wgpu/mod.rs) accepts
`impl Into<wgpu::SurfaceTarget<'window>>` plus physical width/height. The module
reexports the exact selected `wgpu` API. A caller may pass an owned safe handle
such as `Arc<Window>` or a borrowed safe target whose lifetime remains enforced.
The renderer owns its Instance, Surface, Device, Queue and pipelines. It does not
create a Window or know about Winit callbacks. An owned handle is retained inside
WGPU's safe Surface; `close()` drops that Surface and its owned handle before
dropping other GPU resources. The caller then closes its WindowSlot. Borrowed
targets remain subject to their Rust lifetime even after explicit close.

Construction is async and executor-independent. It prepares a Vulkan device,
the original neutral triangle pipeline, and a bounded rectangle pipeline. The
caller forwards physical dimensions through `resize(width, height)` before
acquisition, drawing or presentation.

## YT-282-F application-data scene path

`draw()` still draws the original neutral triangle. `draw_rects(&[RectInstance])`
accepts up to `MAX_RECT_INSTANCES` (64) caller-projected rectangles. Each rectangle
supplies an NDC center and positive NDC half-extents with positive Y pointing up.
It also supplies an RGBA color with channels in [0, 1]. The renderer rejects the
entire slice if any value is non-finite, an extent is nonpositive, a color channel
is outside the range, or the count exceeds the maximum. Both derived edges on
each axis, center minus half-extent and center plus half-extent, must also be
finite. Validation precedes surface acquisition and upload. The focused renderer
tests cover positive and negative edge overflow from finite operands in a later
instance.

A fixed 2 KiB vertex buffer holds rectangle instances. A nonempty frame binds
one pipeline and draws six vertices per instance in one draw call. Color writes
replace target pixels. An empty frame can clear, submit and present with
`draw_calls` unchanged. Both entry points share the same frame acquisition,
submission, returned Present, retry, first-cause and failed-frame cleanup path.

The independent [Twin Squares consumer](M28-F2-EXTERNAL-CONSUMER-AND-AUDIT-HOST.md#yt-282-f-application-data-scene-path)
projects its immutable C1 state into two instances. On the earlier R2 source,
Ubuntu X11 and RTX 4090 Vulkan PID 715103 matched revision 2 across HTTP, UDP,
C1, Audit and Native. The third of 97 returned Presents established the
revision-2 pin. PID 669351 and 98 returned Presents record the earlier R1 source.
The consumer owns the game rules and projection.

## Lifecycle and receipts

| Condition | Behavior and evidence |
|---|---|
| Constructed with positive size | `Unconfigured`; all GPU operation counters zero |
| Zero on either physical axis | `DeferredZeroSize`; no configure or frame acquisition; draw returns Deferred |
| Positive resize | Validate device size limit, configure once, then `Configured`; no draw/present implied |
| Device error during configure, including recovery | Capture before marking Configured; OutOfMemory becomes terminal OutOfMemory, Validation/InvalidSurface or Internal becomes terminal SurfaceFailed; retain the original first cause; no retry loop |
| Device loss during first configure or later checked work | Separate Device loss callback yields terminal DeviceLost with original reason/diagnostic; failed configure counts zero and admits no acquire/present; same-call typed OOM stays primary |
| Draw while Configured | At most one acquire, render pass, draw, submit and returned Present. An empty rectangle frame omits the draw |
| Lost / Outdated acquisition | Return Retry, set Unconfigured; caller forwards current physical size before a later draw; no internal retry loop or fake present |
| Timeout acquisition | Return Retry while retaining configuration; caller chooses a later callback |
| OutOfMemory / Other acquisition | Terminal OutOfMemory / Failed; Device OOM takes precedence over WGPU27's generic returned Other; resize and draw reject until close/drop |
| Suboptimal acquired frame | Draw/submit/present once, report suboptimal, then Unconfigured for caller reconfiguration |
| Fatal acquired-view / present diagnostic | Under Rust panic=unwind only, retain the pinned WGPU diagnostic as terminal Fatal; no OOM classification inferred from text; no failed Present receipt |
| Failed frame release | Destroy the acquired texture before discard; retain a discard fatal separately from the primary draw error; close still releases Surface/owned handle but returns FrameCleanupFailed |
| Closed | Surface/owned handle released; repeated close retains any FrameCleanupFailed result; late resize/draw reject and counters remain intact |

`RenderCounters` separates configure/acquire/pass/draw/submit/present calls,
zero-size deferrals, recovery requests and timeouts. `present_calls` increments
only after a real `SurfaceTexture::present()` returns. WGPU 27's
[present API](https://docs.rs/wgpu/27.0.1/wgpu/struct.SurfaceTexture.html#method.present)
returns no driver success value: this receipt establishes a returned presentation
scheduling call after real rendering submission, not GPU completion, scanout,
gameReady or physical LAN behavior. Each renderer installs concrete
[`Device::on_uncaptured_error`](https://docs.rs/wgpu/27.0.1/wgpu/struct.Device.html#method.on_uncaptured_error)
and [`Device::set_device_lost_callback`](https://docs.rs/wgpu/27.0.1/wgpu/struct.Device.html#method.set_device_lost_callback)
callbacks on its one private Device before pipeline creation. Each callback's
per-device one-slot mailbox uses `try_send`: callbacks do not wait for capacity,
format/log, panic, spawn, reenter WGPU or advance C1. The caller checks both
mailboxes after setup, before and after configure/acquire, and before returning
a presentation receipt. Device OOM is
terminal `RenderError::OutOfMemory`; Validation and Internal are terminal
`RenderError::SurfaceFailed`, including unrelated validation failures. No error
is accepted as successful configuration or presentation. DeviceLost is routed
separately by WGPU27, including when the first configure fails before installing
the Surface error sink. Its loss callback is checked after the actual configure
call, before Configured or configure_calls can change. Typed uncaptured errors
are checked first, preserving typed OOM when both callbacks report during that
operation. An older retained primary is never replaced by later loss or errors.
These callbacks do not cover the acquired texture's separate error sink or fatal present/discard
paths; the bounded handling and limits below are part of this contract.

`first_device_error()` exposes the retained `Arc<wgpu::Error>`, including its
original source chain, after terminal rejection or close. The concrete consumer
clones that Arc into its `io::Error` so Composition retains the first cause
through coordinated shutdown. A captured initialization error also returns
through `io::Result`. This is pinned Vulkan WGPU27 configure/acquire boundary
handling, not general device-loss recovery or a GPU completion fence. Errors
delivered asynchronously after a call returns are observed at the next checked
boundary; no GPU completion or future driver success is inferred.

`first_device_loss()` exposes an `Arc<DeviceLostError>` containing the original
`wgpu::DeviceLostReason` and unmodified diagnostic String, when no earlier primary
or same-call typed error won. Loss returns `RenderError::DeviceLost` with phase
Failed; draw and resize reject further work. The consumer clones that Arc into
the existing Composition primary failure path. Constructor setup loss also
returns the retained cause through `io::Error`. The diagnostic survives close.

`first_fatal_error()` exposes an `Arc<FatalError>` when the pinned library only
provides a fatal diagnostic, including its exact operation and diagnostic text.
The consumer retains this Arc in Composition too. `frame_cleanup_error()`
separately retains a failed discard. `close() -> Result<(), RenderError>` drops
the Surface and other owned GPU objects even after such a failure, but keeps
returning `FrameCleanupFailed`. The consumer still closes WindowSlot after
Surface release, retains the cleanup error across callbacks, and leaves
Composition completion Pending rather than falsely reporting Complete.

### Pinned fatal boundaries and cleanup limits

WGPU27's acquired `CoreTexture` has a fresh private error sink; neither Device
error scopes nor its callback can intercept `Texture::create_view` errors there.
The renderer uses safe Rust `catch_unwind` around that exact call, accepting
only its pinned default `wgpu error:` diagnostic. It becomes terminal
`RenderError::Fatal`; the library has formatted away the typed error, so even
an OOM-looking message does not become typed OutOfMemory. Typed Device callback
OOM remains OutOfMemory and its original source chain is preserved.

On a failed draw, the frame remains outside the view-call unwind. Public safe
`Texture::destroy()` first releases its raw Surface texture. In pinned
wgpu-core27.0.3 this snatches `TextureInner::Surface` without a Device-validity
check and drops the raw texture, including Vulkan semaphore Arc references.
Only then is the frame dropped in a separate exact discard boundary. Core's
discard checks Device validity before taking its acquired texture, so merely
catching discard without destroying the raw texture would retain those Arcs
and cause a later Vulkan unconfigure assertion. A recognized discard fatal is
stored as a secondary cleanup failure, never substituted for the primary OOM
or view diagnostic. No frame or Window is deliberately leaked, and failed draw
paths do not submit or present after discovering the failure.

The separate exact `SurfaceTexture::present()` boundary accepts only WGPU's
`Error in Surface::present:` fatal diagnostic. This API sets `presented=true`
before calling its inner implementation, so its unwind does not discard the
frame again. A cloned Texture handle allows explicit destruction if present
failed before core took the acquired raw texture. The renderer becomes terminal,
the consumer receives the cause, and `present_calls` remains unchanged on failure.

These are disposal paths, not recovery of a usable Device. Only these three
foreign calls use scoped `AssertUnwindSafe`; no application callbacks or render
loop are caught. Unknown panic payloads/diagnostics resume unchanged. There is
no process-wide panic hook; the normal panic hook still runs. Abort builds,
allocator abort, double panic inside WGPU/driver destructors, FFI crashes and
non-unwinding platform failures cannot be caught. No safe public WGPU27 API
guarantees recovery in those cases. Native teardown may wait and WGPU's Vulkan
unconfigure ignores a failed device-idle wait; returned object release never
proves GPU completion or successful cleanup inside a faulted driver. No induced
Vulkan OOM or cross-platform fault recovery is claimed.

Source basis: WGPU27.0.1 `src/backend/wgpu_core.rs` (`CoreTexture::create_view`,
fresh acquired sink, `handle_error_fatal`, present/discard); `src/api/surface_texture.rs`
(`present`/Drop); wgpu-core27.0.3 `src/resource.rs` (`Texture::destroy`),
`src/present.rs` (Device validity before acquired-texture take), `src/instance.rs`
(Surface Drop); wgpu-hal27.0.4 `src/vulkan/instance.rs` (unconfigure and semaphore
ownership assertions), `src/vulkan/mod.rs` (queue-present error propagation).
Exact installed dependency source hashes are recorded in `SOURCE-FENCE-R4.json`;
the earlier R3 fence remains unchanged historical evidence.

The caller must forward actual zero sizes unchanged, suppress inappropriate
hidden/occluded redraws, and schedule retry callbacks without spinning. Native
WGPU configure/acquire/present may wait inside driver calls; the renderer adds
no explicit blocking wait, busy loop or `Device::poll` wait.

## Concrete caller-driven consumer

[`native_wgpu_triangle`](../crates/yetlet/examples/native_wgpu_triangle.rs)
requires both features and uses only public Yetlet/host APIs. It owns the one
caller-thread Winit EventLoop and ApplicationHandler. One caller-local
`Rc<C1Domain>` is attached through public Composition and never advanced by the
renderer. Its concrete foreground resource closes the GPU before WindowSlot,
then Composition stops C1. Window Created, matching OS Window Ready, GPU prepared,
configured and first present have distinct logs. Only a real returned present
with Window readiness marks this consumer's foreground part ready.

```sh
cargo run --locked -p yetlet-hosts --no-default-features \
  --features native-winit,renderer-wgpu --example native_wgpu_triangle

# Bounded diagnostic: explicitly synthetic zero dimensions and an actual OS resize.
DISPLAY=:0 cargo run --locked -p yetlet-hosts --no-default-features \
  --features native-winit,renderer-wgpu --example native_wgpu_triangle -- --diagnostic
```

Initial deferred assertions account for both physical axes. An initially zero
OS Window keeps the renderer deferred without configure/acquire/present until
a positive resize. Diagnostic mode also supplies **synthetic initial 0x0** to
the real renderer, checks the initial deferred draw with all counters zero,
then restores the actual positive OS size. This is distinct from an OS zero
Window or minimize event.

The example requires a discrete Vulkan GPU for its hardware proof; the renderer
itself does not impose that device-type restriction. It cooperatively polls
initialization on the caller thread with a five-second deadline, without a new
executor/thread. Native driver FFI cannot be preempted by that deadline. Other
callbacks contain no explicit sleep/wait. Redraw scheduling uses 100 ms future
deadlines and an eight-second proof deadline, with normal exit after two seconds
and at least two presentations. Initialization or GUI/GPU unavailability is a
failed/blocked proof, never a successful substitute renderer.

## Scoped verification — 2026-11-19

Evidence is preserved in `.pipeline-output/m28-f2-yt282d2-20261119/`.
The following checkpoint is **earlier source, before the R1 repair**.
Base HEAD remains `363dde2f83c248a5c099bc6a1fff67c656039145`;
`GPU-SOURCE.json` binds the actual renderer, shader, consumer, native slot,
manifests and lockfile to the executed binary. This is Ubuntu x86_64,
Rust/Cargo 1.99.0, Winit 0.30.13, WGPU 27.0.1, existing X11 `DISPLAY=:0`,
NVIDIA GeForce RTX 4090, Vulkan, driver 595.99.02. No other native platform,
browser, web/UDP integration or physical LAN claim.

- Focused `renderer_wgpu::tests`: 8/8 pass. Pure synthetic transitions cover
  zero on each axis, restoration, Lost/Outdated, loss at zero, Timeout,
  terminal OutOfMemory/Other, oversize and late closed operations. They do not
  manufacture driver loss or GPU presentation evidence.
- Default, renderer-only and native-only library checks; combined consumer
  check/build; renderer-only and combined Clippy with `-D warnings`; package
  Rustfmt all exit 0. Feature graphs verify independent selection and light
  defaults. The only D2 lockfile change adds `wgpu` to `yetlet-hosts`' dependency
  list; existing package records remain byte-identical.
- Actual diagnostic PID 231801, exit 0, process gone after 2.188 seconds:
  WindowId(90177538), initially 640x420; GPU prepared Unconfigured with all
  operation counters zero; actual matching OS redraw establishes Window Ready
  with draw/submit/present still zero; then 20 actual pass/draw/submit/returned
  present calls, 3 configurations and 20 acquisitions. Actual OS resize to
  720x480 reconfigures; original XWD plus lossless PNG capture shows the triangle.
- Actual Surface-owned Window handle prevents early WindowSlot close
  (`WouldBlock`). GPU closes first, late draw/resize reject, Window closes,
  Composition Complete/Stopped, sole C1 owners=1/revision=0/stopped=true.
  A post-exit X11 query returns expected Bad Drawable for that exact Window ID.
- The diagnostic 0x0 dimensions are **synthetic**, applied to the actual GPU
  renderer; no configure/acquire/present occurs. The OS Window was not 0x0.
  Driver Lost/Outdated/Timeout/OutOfMemory and device loss were not observed.
  Recovery requests and timeouts in the actual run are both zero. Pure tests
  cover lifecycle transitions only, not the Device error boundaries identified
  by the later independent R1 FAIL3 or actual driver fault recovery.

### Historical R1 repair checkpoint, subsequently reviewed as R2 FAIL3

Independent R1 `wc_job_JXdnJsk9SpoWU7SL` exited 0 with actual **FAIL3**:
acquisition Device OOM panicked before returning, recovery configure could
panic on InvalidSurface, and the consumer assumed a positive initial size.
`R1-SOURCE-REVIEW-OBSERVATION.json` immutably records that review/source fence
before repair; the original result, transcripts, hashes and failures remain
unchanged. `SOURCE-FENCE-R2.json` records the repaired source for the orchestrator.

- `repair-r2-renderer-tests.*`: 13/13 pass, including five new synthetic
  Device callback/boundary tests. They exercise OOM before acquisition returns
  Other, Lost/Outdated followed by Validation/InvalidSurface **inside the
  configure call**, configure OOM/Internal, queued errors before another call,
  per-device isolation, full/disconnected callback mailboxes, first-cause
  retention in Composition and terminal late-call rejection. No driver fault
  was induced by these tests.
- `repair-r2-consumer-tests.*`: 1/1 pass for initial deferred assertions on
  each zero axis and positive size. Default/native library checks, renderer
  and combined Clippy `-D warnings`, package Rustfmt and combined build exit 0.
  Feature/dependency declarations and lockfile are unchanged by this repair;
  original independent feature graphs remain applicable.
- New exact-source command:
  `DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/1000 target/debug/examples/native_wgpu_triangle --diagnostic`.
  `GPU-SOURCE-R2.json` binds the product SHA and binary;
  `gpu-vulkan-r2.*` records actual PID 279928, exit 0 in 2.163 seconds on the
  same Ubuntu X11 RTX 4090 / Vulkan / driver 595.99.02 environment. Synthetic
  initial 0x0 returns DeferredZeroSize with every operation counter zero;
  actual positive OS size then configures, actual OS redraw marks Window Ready,
  and 20 real pass/draw/submit/returned present calls follow. Actual resize
  reaches 720x480; configure=3, acquire=20, recovery/timeouts=0.
- The exact Window is observed IsViewable and captured in `triangle-r2.xwd`.
  Surface releases before Window close, late calls reject, Composition is
  Complete/Stopped, sole C1 owners=1/revision=0/stopped=true. The process is gone
  and the exact XID query exits 1 with expected Bad Drawable after exit.
  The original `gpu-vulkan-r1.*` PASS remains earlier-source historical evidence.

Actual driver OOM, InvalidSurface, Lost/Outdated/Timeout and OS zero dimensions
were unverified at that checkpoint. The subsequent independent R2 result is
actual FAIL3; the checkpoint above is not acceptance of those fatal paths.

### Historical R2 repair checkpoint, subsequently reviewed as R3 FAIL1

Original `reviewer-result-r2.json`, transcripts, R1/R2 source fences and GPU
receipts remain unchanged. R2 found acquired-view default-sink panic, secondary
invalid-Device discard panic after submission OOM, and fatal Vulkan presentation
panic. The bounded changes implement the three disposal paths above; they do
not modify WGPU, its pin, the shader, D1, Composition or other hosts.

- `repair-r3-20261120-renderer-tests.*`: 18/18 pass; five new explicitly
  synthetic tests use the production fatal-call, release and lifecycle helpers.
  They check original diagnostic/typed first cause, destroy-before-discard,
  secondary cleanup failure, presented-before-inner-unwind, no failed Present
  increment, terminal rejection and propagation of unknown panics.
- `repair-r3-20261120-consumer-tests.*`: 2/2 pass, including the real consumer
  close callback with a synthetic retained discard failure: WindowSlot closes,
  the primary Composition cause survives, completion stays Pending across retries.
  No OS Window or GPU fault is created in that test.
- That checkpoint's renderer/combined Clippy `-D warnings`, package Rustfmt and
  combined example build exit0. Manifests, lockfile and feature declarations
  remain unchanged; prior independent dependency graph receipts still apply.

That checkpoint's normal GPU receipt is `repair-r3-20261120-gpu-vulkan.json`; its
paired source receipt binds exact product and executable SHA256. It records
actual Ubuntu X11 `DISPLAY=:0`, RTX4090 Vulkan, returned present calls, exit and
Surface-before-Window cleanup. This is normal execution; all error/fatal stimuli
above remain synthetic. Subsequent independent R3 returned actual FAIL1/P1;
those results do not establish acceptance of the first-configure loss path.

### Current R3 repair, independent R4 pending — 2026-11-20

Independent R3 `wc_job_LDSjr0DyWe_wcqlr` reached terminal exit0 with actual
**FAIL1/P1**. WGPU27 routes DeviceLost outside on_uncaptured_error, and first
configure failure leaves the Surface sink absent. The previous empty-mailbox
check could count configuration and allow a fatal unconfigured acquire. Original
R1/R2 FAIL3 and R3 FAIL1 JSON, transcripts, fences and GPU evidence remain
unchanged. This repair adds the private Device loss callback and retained concrete
diagnostic described above; it changes no dependency, feature or D1 boundary.

- `repair-r4-20261120-renderer-tests.*`: 22/22 pass, including the previous
  18 and four new synthetic loss cases. They invoke the same callback closure
  installed in production and the actual Lifecycle configure helper: first
  configure loss is terminal with configure/acquire/present all zero; public
  draw/resize reject before GPU access; original reason/diagnostic survives
  Composition retention and close. Same-call OOM wins in either callback order;
  older primary errors survive later loss. Full/disconnected mailboxes return
  without capacity waits and per-device state stays isolated. Setup checks also
  observe queued loss at zero size. These tests induce no real driver loss.
- `repair-r4-20261120-consumer-tests.*`: the existing 2/2 pass, including
  retained secondary discard failure leaving Composition Pending. Renderer and
  combined Clippy `-D warnings`, changed-source Rustfmt and combined build exit0.
  Earlier unrelated D1/F1/HTTP/UDP tests and reviewers are not rerun.
- One new actual Ubuntu 26.04.1 X11 `DISPLAY=:0` RTX4090 Vulkan/595.99.02 run,
  PID81990, exits0 in 2.095 seconds with no forced termination and process gone.
  Actual configure3, acquire/pass/draw/submit/returned present20 each, OS resize
  640x420 to 720x480, recovery/timeouts0. Surface releases before Window; late
  calls reject; sole C1 revision0/stopped/owners1, Composition Complete/Stopped.
  Initial 0x0 is explicitly synthetic. Exact XID is IsViewable during execution
  and absent after exit; raw capture is retained without a new scanout claim.
- `repair-r4-20261120-gpu-source.json` binds product SHA
  `f6d7f0594735e6cb379ea1387467848ceb5e4a03fda2629e49aff0494fc62bb4`
  and executable SHA; `repair-r4-20261120-gpu-vulkan.json` records command, PID,
  counts, exit and cleanup. Product source remains unchanged after execution.

No actual injected driver DeviceLost/OOM, physical LAN/F3, Windows, OS0x0,
abort recovery, GPU completion or scanout proof is claimed. `REPAIR-R3.md` and
`SOURCE-FENCE-R4.json` bind this bounded repair for the orchestrator's independent
R4 review. R4 has not been launched here; D2 stays unchecked and full YT-282 Todo.

No D1 source or D1 consumer was edited or rerun; no previous F1/game, HTTP,
UDP or independent Reviewer was launched. Historical ignored documentation
diagnostics retain their original nonzero receipts; scoped new documentation
checks are separate. Independent review remains the next D2 acceptance step.
After that, the next bounded YT-282 increment is consumer migration/composition
as selected by the orchestrator; YT-283 external three-way/custom Host and
YT-284 package qualification remain later gates. F3 and physical D4 remain open.
