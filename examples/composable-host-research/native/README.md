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

Run from the repository root:

```sh
cargo test --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --lib
cargo run --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline -- --smoke-ticks 40
```
