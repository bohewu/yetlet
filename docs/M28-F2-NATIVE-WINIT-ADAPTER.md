# M28-F2 / YT-282-D1 — Optional caller-owned Winit foreground Window

- **Date:** 2026-11-19.
- **Status:** Bounded public Native/Winit Window adapter source completed; R1/R2 material defects repaired, new X11 negative regression PASS, independent R3 actual PASS. Full YT-282/F2 remain open.
- **ADR:** 0013 Accepted scoped F2; YT-282-B, C1 Web and C2 UDP independently
  reviewed. Full YT-282/F2 remain open.
- **Feature:** `yetlet-hosts/native-winit`, default off, Winit 0.30.13
  with X11/rwh_06 on tested Ubuntu. No required wgpu/GPU, Axum, Tokio,
  UDP, extra game tick, browser dependency or generic Host Trait.

## Actual public capability and ownership

[`native_winit::WindowSlot`](../crates/yetlet-hosts/src/native_winit.rs)
is a concrete !Send/!Sync foreground-owned Window slot:

- The application creates and runs its own `winit::EventLoop` on the supported
  caller thread, forwarding its real `ApplicationHandler::resumed` and
  `window_event` callbacks. The adapter never starts another thread, EventLoop,
  domain, runtime, renderer or hidden scheduler.
- `WindowSlot::new(WindowAttributes)` has no OS side effect.
  `resumed(&ActiveEventLoop)` actually creates a native OS Window and requests
  redraw, but remains `Created` and **not Ready** until it observes a matching
  `WindowEvent::RedrawRequested` for this actual WindowId, nonzero physical
  size, non-occluded state **and live `Window::is_visible() == Some(true)`**.
  Hidden or unknown-visibility windows remain NotReady; `is_ready()`
  also checks live current visibility even before a new Window event. An
  ignored redraw due to hidden/unknown visibility clears any earlier Ready
  phase without incrementing the accepted redraw counter; wrong IDs are ignored. This confirms only
  foreground **Window readiness**, not GPU drawing, surface presentation or
  game-ready semantics.
- Typed event results include RedrawReady, CloseRequested, Resized,
  Occluded, **Destroyed**, and Ignored. A matching OS Destroyed event
  immediately invalidates the Window slot and ignores subsequent queued
  redraws until a genuinely new Window is created on a later resumed. The caller chooses to stop its Winit loop and domain.
  The public Composition Core's `required_foreground` is marked Ready only
  after the actual redraw observation. Domain state remains uniquely owned
  by the app, not by this Window adapter.
- Repeated resumed while Window exists is idempotent. `suspended` releases a
  live Window before possible re-creation, provided there are no external
  Arc handles; `close` is terminal, idempotent and actually releases the OS
  Window. An outstanding Arc intended for a future separate renderer makes
  close/suspend report WouldBlock rather than falsely claiming completion.
  `Arc::try_unwrap` atomically verifies unique ownership and restores the
  original Arc on failure, so a Weak upgrade cannot race a count-then-drop
  sequence. The caller must release renderer owners before trying again.
- Winit creation failures are io::Error and should be recorded as primary
  through the public Composition Core before caller requests EventLoop exit.
  The public example ignores late Winit callbacks after a required failure
  because the Composition Core deliberately denies further mutable access;
  no `expect()` panic bypasses the original failure and cleanup. No implicit
  Drop Join, GPU teardown or domain shutdown occurs here.

## Focused actual source evidence

The public [`native_winit_foreground`](../crates/yetlet-hosts/examples/native_winit_foreground.rs)
is a standalone source-only consumer using **one** caller-local
`Rc<C1Domain>`, the actual public typed Composition Core with a required
Native Window part, and a main-thread Winit EventLoop.

One bounded real Ubuntu X11 `DISPLAY=:0` run returned actual **exit0**:
- Process 128925 created the OS Window `WindowId(90177538)` at **640×420**,
  reporting `Created, ready=false`.
- Received one actual OS `RedrawRequested` for that same window ID,
  changed its own Window state to Ready, marked the Composition Native part
  Ready and reported root Running. C1 domain revision remained 0.
- Premature close with a temporarily cloned Arc returned WouldBlock as
  expected; dropping the clone allowed final actual Window release.
- After caller-requested event loop exit, Native Window Closed, C1 domain
  Stopped and Composition Complete/Stopped. The process has exited.
- Log expressly contains `gpu_present=false`. No wgpu init, draw/present,
  GPU pixel evidence or screenshot was attempted.

Source receipts:
`.pipeline-output/m28-f2-yt282d1-20261119/native-x11-r1.json`,
`native-x11-r1.stderr.log` and `native-x11-reconciled-r1.json`.
A Python receipt-printing typo happened **after** writing the successful
actual exit0 program output; it was reconciled using that original log and
no second GUI launch.

New feature-specific lib tests cover Prepared/no-OS-side-effect and
Suspended/Closed without false readiness. The real Window example is
separately compiled and linted. Focused feature/dependency graph checks,
Rustfmt and independent Reviewer must PASS before marking D1 Done;
previous F1 native GPU, C1 Web and C2 UDP tests are reused only at their
original exact-source scope, not rerun here.

## R1 findings and bounded negative regression

Independent Reviewer R1 `wc_job_GLKTBR72UpehCx-h` terminal exit0 returned
**actual FAIL4**: hidden-window false readiness (P1); OS Destroyed allowing
stale redraw (P2); Weak upgrade race between Arc strong_count and Drop (P2);
and delayed event after `fail_required` panicking through `value_mut().expect`
(P2). Original R1 JSON/logs and the first successful source run are preserved.

All four were repaired in the Window adapter/example. A separate **single
bounded real X11 run** of the current source with `--negative` actually
created a hidden Window, rejected an intentionally simulated redraw while
hidden, showed it, then received one **real OS RedrawRequested** at visible
640×420 and marked the original Composition Root Running. The test then
**synthetically injected** Destroyed and a post-failure redraw callback
into the real application (those two are not claimed as OS-origin events).
The slot rejected stale redraw, retained the first deliberately injected
failure, completed Window/C1 cleanup with `Completion::Complete`, and the
application exited **code 0** under its expected negative-case contract.
Original process PID 156741 was verified gone. All of this is non-GPU.

Exact negative receipts: ignored `native-negative-r2.json`,
`native-negative-r2.stderr.log` and `native-negative-accepted-r2.json`.
Current source compile, native feature unit **2/2**, Clippy `-D warnings`,
Rustfmt PASS. The earlier exact feature graphs remain valid because no
manifest/feature source changed in this repair. Independent R2
`wc_job_rL8-YC3_rdfk06va` returned actual FAIL1/P2: a previously Ready
Window could be hidden without a Winit occlusion event and retain Ready
when a matching redraw was ignored. R2 original FAIL is preserved;
this further repair checks live visibility in `is_ready()` and revokes
Ready on matching hidden redraw. Independent R3 `wc_job_MZabD_B-_N9izNpe`
terminal exit0 **actual JSON PASS**, `issues=[]`, `required_followups=[]`.
The earlier R1/R2 FAILs remain preserved as their original results.

## R2 live-visibility invalidation negative case

Current source added a focused `--negative` sequence to the real
Ubuntu X11 example: after actual visible OS redraw and Ready, the caller
hides the same real Window and verifies `is_ready()` becomes false even
without an OS Occluded event; an explicitly **synthetic** matching hidden
RedrawRequested is ignored, changes the slot from Ready to Created and
leaves its accepted redraw count unchanged. The same example then checks
synthetic Destroyed and late-after-failure callbacks, preserves its injected
primary failure, and completes C1/Window cleanup. New OS process PID168678
returned **exit0**, was verified gone, and expressly logged
`gpu_present=false`. The real OS visible redraw is not confused with the
synthetic failure stimuli. `native-negative-r3.json` and
`native-negative-accepted-r3.json` preserve the exact current-source test.
Current native unit 2/2, example compile, Clippy -D warnings and Fmt PASS.
No historical GPU/F1 or unmodified feature checks were rerun.

## YT-282-D1 source review closeout

The final independent Reviewer R3 `wc_job_MZabD_B-_N9izNpe` completed,
exit 0, **actual JSON PASS** with no issues or required followups. The
reviewed native adapter/example source SHA remained unchanged afterward.
The R3 role-directed Codex Job used the original Runner permissions and
recorded model/effort; it is not native `agent_type` formal assurance.
Scoped source, feature and X11 lifecycle evidence support only the optional
Winit Window behavior described above. No GPU present, display pixel
readback, renderer qualification or physical LAN proof is asserted.

## Explicitly deferred

A future separately opt-in `renderer-wgpu` feature is **not implemented
or selected in this slice**. Its actual GPU present, Surface error handling
and engine-quality verification require their own bounded D2 scope.
Native-Winit Window readiness alone cannot qualify GPU game-ready.
Full Game/Network/Web three-way migration via public packages, external
YT-283 custom Host and YT-284 package gate remain open. Physical D4 stays
**Not verified**, and no F3, LAN hardware, UDP protocol/permissions, CLI,
release, Stage/Commit/Push or hosted publication is undertaken.

## Engine interoperability boundary (YT-283, 2026-11-21)

Yetlet's optional `WindowSlot` is a **thin Winit Window lifecycle helper**,
not a Game Engine. The caller owns its `EventLoop`/`ApplicationHandler`, keeps
the original `WindowEvent`, calls `observe_window_event(id, &event)` to update
Window readiness and separately matches raw `KeyboardInput`, `MouseInput`,
`CursorMoved`, `MouseWheel`, `Focused`, `Resized` and `ScaleFactorChanged`.
An `Ignored` WindowSignal does not consume the caller's event. Relative mouse
`DeviceEvent` remains the caller's responsibility; it was not runtime-tested.

The caller can use `window()` for direct Winit Window/cursor methods,
`window_handle()` to give a selected renderer its `Arc<Window>` and
`request_redraw()` to request a frame. Release external handles before close.
An engine already owning a Window may instead attach its Web/Network/custom
resources via `yetlet::composition` and never create `WindowSlot`. No Yetlet
KeyboardEvent, MouseEvent, input mapper, mandatory scheduler, scene or physics
API was added. A future engine is a separate SDK.

The [YT-283 external consumer record](M28-F2-EXTERNAL-CONSUMER-AND-AUDIT-HOST.md#yt-283-final-native-window-host-integration-2026-11-21)
distinguishes real X11 keyboard/mouse/focus/resize observations from untested
DPI changes, cursor capture, relative DeviceEvent and OS suspend callbacks.
