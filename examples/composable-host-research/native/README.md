# M28-C2 — Private native GameHost (YT-269)

The independent nested `native/` package selects **winit 0.30.13 (X11)**
and **wgpu 27.0.1 (Vulkan)**. Its opt-in dependency graph is separate from
the existing std-only B1/B2/C1 package, root workspace and pure Web/SSR path.
No public Yetlet Host or Renderer trait is introduced.

`GameDriver` alone owns the unchanged C1 `Simulation::step`. Physical
keyboard events queue bounded validated commands for two square players;
fixed logical 60 Hz ticks run separately from the targeted 30 Hz redraw
cadence. `project(snapshot)` produces two colored quads from immutable
snapshots. Window and surface lifetimes belong to the caller-thread winit
ApplicationHandler; the wgpu renderer owns its Device, Queue and Surface.

WASD = player one; arrow keys = player two. Space = Pause; Enter = Resume;
Escape = exit. Losing focus releases both players. Pause clears held input.
Resume clears tracked keys only when the effective state **after preceding
queued Pause/Resume commands** was Paused. A second Resume queued in the
same Tick preserves physical key tracking when it is a reducer no-op, so
release or focus loss still emits Hold(Still). A nonzero-size Surface
receives draws on RedrawRequested,
never on the simulation tick. A bounded `--smoke-ticks N` CLI option exits
automatically for one actual native startup integration attempt; it is not
C3 visual or resize/minimize/performance certification.

Headless C1 and B1/B2 use no winit/wgpu dependency. This C2 package does
not own or create any Tokio runtime, networking, HTTP listener, GPU worker,
or second process. C3 retains the later actual visual/lifecycle qualification.


## Verified bounded Ubuntu native run (C2 only)

On Ubuntu 26.04.1, X11 display `:0`, a 960x640 native winit window and a
wgpu 27 Vulkan Surface were actually created on an NVIDIA GeForce RTX 4090.
The initial 45-control-tick bounded run emitted 24 rendered frames and exited
normally. In one separate owned-app X11 XTest interaction the exact
window/PID were verified, then player-one D, player-two Left, Space Pause,
Enter Resume and corresponding key releases were sent. Its final immutable
Snapshot had control tick 210, active ticks 177, Player One (22,2),
Player Two (0,12), and 106 rendered frames. The test restored original
X11 focus and verified only its own native process ended.

These observations do not replace later C3 actual visual review, resize/
minimize/recovery and performance checks. Do not extrapolate this to other
display servers, GPU backends or native OSes.


## C3 — One bounded native GPU/Window visual and lifecycle check (YT-270)

The independent Native GameHost now accepts an opt-in
`--c3-qa-ticks 480` flag for a **single bounded diagnostic** run.
The unchanged default/`--smoke-ticks` paths do not inject events.
The diagnostic mode injects one 260ms foreground callback delay and
**synthetic** zero-size / Lost / Outdated input to the same guarded
renderer branches. Do not mistake those injections for actual X11
zero-size or genuine GPU device loss.

A separately scoped ignored test driver (see
`.pipeline-output/m28-c3-20261117/visual_lifecycle_x11.py`)
verified exact owned X11 window and process identities, captured real
XWD → PNG from the actual 960×640 Surface, resized the owned native
window to 720×480 and captured a second image, minimized/restored it
through the real Window Manager, captured again, then sent WM_DELETE to
only that window. The original X11 focus was restored, and the owned
WindowId no longer existed after normal exit.

Real screenshots are retained under
`artifacts/verification/m28-c3-20261117/c3-native-{initial,resized,restored}.png`,
with their raw XWD counterparts. Per-image grounded sample colors,
dimensions and SHA256 are in `native-c3-x11-observation.json`; host
runtime logs report Surface configuration, deferred zero size,
synthetic Lost/Outdated reconfigure, occlusion/resize/close and
CPU-side timing. The original failed title-identification harness
attempt is retained separately, not presented as a Native product fail.

During the actual run: 125 control ticks, 61 rendered frames,
two real resize events, four occlusion events, a single WM CloseRequested,
at most four ticks per wake, ~193.583ms dropped debt from one injected
260ms delay. CPU-side render encode/submit/present mean/max
2058.375/7080.547 µs; pure reducer step mean/max 11.116/61.725 µs.
These are single-run CPU timings, not GPU latency or sustained FPS
certification. Three selected C3 clock tests pass. An independent
R1 P2 found that while fully obscured the expired draw deadline could
busy-spin; the bounded repair now waits only for the next Simulation
Tick while hidden and resets the render deadline on restoration.
The one native run emitted only Occluded(false) events, so Occluded(true)
was validated with a focused non-GUI scheduling test, not claimed as
observed live Window behavior. The original GPU run was not repeated.
All previously validated C2/B0/B1/B2/C1 sources/tests remain unchanged.


## D2 — Two real Native GameHost processes, loopback UDP (YT-272)

The opt-in `d2-loopback` binary launches as `host` or
`client 127.0.0.1:<actual-port>`, each one independent OS process
composing a caller-thread winit GameHost with a private NetworkHost or
NetworkClient. Neither is a per-Host process/thread. Host alone owns
D1 HostFixture and C1 Simulation. Client keeps the latest read-only
D1 ReplicaFixture; both GPU renderers display immutable Scene data.
The only network is explicit 127.0.0.1 UDP, with no wildcard bind,
root Yetlet network dependency, firewall changes or LAN exposure.

Host binds port zero, reports its actual endpoint, and accepts only
one loopback peer Source for this bounded demo. D1 canonical Codec
version 1 and Session/Epoch/Peer/Sequence/Tick checks apply before
authority admission. Up to 8 nonblocking 513-byte datagram receives
per wake, one remote assignment, up to three Hello responses and
20 full Snapshot datagrams/s keep processing bounded. The scripted
demo moves Host Player One East and Client Player Two West. This
tests true datagrams and rendered state, not keyboard interaction,
reliable transport or hostile peer authentication.

Actual Ubuntu RTX4090/Vulkan X11 bounded E2E:
Host PID 900380, <selected-endpoint>; Client PID 900381,
127.0.0.1:32801. Both report identical authoritative Revision/Tick
90 with player X positions 22 and 0. Two real 960×640 Native Window
PNG screenshots have identical SHA256. Real Windows are closed via
WM_DELETE; both Exit 0, exact Window IDs disappear, Socket and GPU
ownership released, original X11 focus restored.

Retained exact real screenshots:
`artifacts/verification/m28-d2-20261118/d2-{host,client}-actual-window-r3.png`
and source XWD files; exact SHA256 and process/socket counters in
`d2-real-process-x11-udp-observation-r3.json`. The first two harness
attempts are retained as failed evidence (preflight Git status and a
Client Window positioned 5px beyond the screen), not hidden.

Focused Native-only tests:

    cargo test --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --lib d2_

The Host/Client Demo is separately selected and is not a default
Yetlet service or runtime. D3 controlled packet loss/reorder,
reconnect, headless networking and D4 physical LAN remain separate.
Tokens/port matching are **not** cryptographic authentication.

### D2 independent R1 findings and bounded fixes

Original Reviewer R1 `wc_job_YfHgg6cq0ElmcFbT` actual FAIL with three
material P2s is retained. In the corrected Client, rendering waits for
a fully validated D1 Snapshot rather than showing fabricated players.
The 300-control-Tick bounded exit now releases the owned Socket and
Window even if the Window was obscured or zero frames rendered; it
reports a no-frame result separately. The private Native manifest also
restores `default-run = "yetlet-native-gamehost-research"`, so the
previous C2 `cargo run ... --smoke-ticks` guidance remains selectable
without naming a binary; D2 always explicitly uses `--bin d2-loopback`.
Two focused non-GUI regressions, Fmt/Clippy/Native build and Cargo
metadata checks PASS. The original two-PID wgpu/UDP E2E was not rerun
for these post-review repairs; it remains evidence only for the
unchanged successful synchronized path. Independent R2 pending.

### D2 independent R2 bounded Tick repair

Independent R2 `wc_job__nWJv7-x7VPY-qLT` actual FAIL found that
one delayed callback could yield four due ticks when 299 of the
selected 300 had already been processed. Host could then publish
through Tick303 before exiting. The private foreground D2 scheduler
now caps each due batch by the **remaining control-Tick budget before
calling the sole C1 owner step**; selected tests assert 299+4→300,
298+4→300, and zero work after exhaustion. Three/three selected D2
binary regressions PASS with Fmt/Clippy and Native Build. R2 FAIL
and prior R1 FAIL stay immutable; independent R3 pending. The prior
successful two-PID UDP/wgpu run was not relaunched after this source
repair; its original positive-path evidence remains retained.

Run from the repository root:

```sh
cargo test --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --lib
cargo run --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline -- --smoke-ticks 40
```
