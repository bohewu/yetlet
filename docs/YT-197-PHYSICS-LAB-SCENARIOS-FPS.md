# YT-197 — Physics laboratory scenarios and estimated render FPS

Date:2026-10-26. Status:**Done after independent whole-item Reviewer PASS,0issues/required followups.**

## Authority and minimum contract

The owner requests commit of the accepted BFCache documents then continuation
of the recommended Lab FPS plus stack/domino scenarios. The two documents were
committed separately with strict/builtin Guard; new YT-197 changes noCommit/Push.
Keep application-owned opt-in, existing Run/Pause/Reset and safety/resource limits.
This slice adds no crate, dependency/lock change, renderer/Core API, generic scene
editor, slope/joint/mesh feature or performance qualification. Preserve old receipts
and do not replay their accepted tests. Independent Reviewer PASS is required
before Done.

Scenario IDs: falling(default),tower(Stack impact),dominoes(Domino chain). Native
GET and browser controls use the same closed IDs; reject unknown names. Count
remains1–64 dynamic bodies plus the same five static enclosure bodies; gravity
0–20m/s²,speed0.25/0.5/1/2,fixed60Hz,max4steps/callback,100msclamp and GPU budgets
remain unchanged. Use existing boxes/spheres and the19-float pose layout. Initial
finite unit rotations and linear velocities must agree in SSR, Rapier and render.
Tower uses alternating three-block layers plus one falling striker when count>=2;
count1 is a single-block degenerate case. Dominoes form a bounded arc, with the
first body leaning/moving toward its neighbor. No endless spawning or auto-retry.

The scene selector stages the next preset. One adjacent sentence explains that
Reset applies controls and pauses; first Run uses the current selection. Changing
selection alone must not mutate the active world or resume it. Reset validates
before replacing the CPU world; existing retirement/invalid-world rules apply.
Native Reset remains useful without JS/Wasm/assets. No additional action buttons.
The example-private Wasm constructor/reset gain a scenario argument; rebuilding
hashed assets/server together is required, with legacy bundles falling back to SSR.

HUD: FPS(estimate) plus active object count, no live announcements. Count successful
draw submissions while running over at least one second of real monotonic time;
this is not GPU completion/presentation FPS or fixed physics Hz. Reuse the existing
frame loop and<=4Hz DOM updates, no independent timer/RAF loop or sample arrays.
Before Run:Not started; during warmup:Measuring; Pause/Step/suspension:Paused;
preparation/failure:Unavailable. Stop/reset/retry clears old samples, so idle time
cannot be carried into a later interval. No target FPS or hardware/SLO guarantee.

## Bounded verification

New targeted Rust tests only: closed scenario parsing,bounded preset/pose data,
new-scene initial Rapier state/contact/cascade and atomic invalid-scenario Reset;
new SSR selection/initial HUD/native rejection. New private FPS counter tests
cover actual draw counts,warmup,reset and invalid time. Compile the changed
native/opt-in Wasm/hash assets with existing locked tools, no install or old test
matrix replay. Actual isolated browser proof covers the two new presets and
estimated HUD lifecycle, staged selection/Reset and usable controls. Reuse the
recorded private Firefox/Wayland path; retain failures and clean owned resources.
Review evidence and source/asset/unchanged-scope bindings; only then set Done.


## Implementation and focused evidence

The minimum contract above was written before implementation. Application-only
Scenario parses falling/tower/dominoes; Parameters/preset generate the new bounded
states. Rapier now consumes the already-serialized initial quaternion/velocity;
no19-float layout or private GPU renderer change. The private Wasm new/reset
signatures append scenario; validation precedes replacement. SSR shares parsing,
selection, shape labels and initial velocities. The loader stages controls,
tracks active scene/count separately and applies them only on firstRun/Reset.

Private frame-rate.mjs uses three scalar counters and a>=1second real-time window.
Only successful Gpu.draw calls while running are recorded. Source inspection
confirms successful draw submits/presents before returning; completion callbacks
are separate. Stop clears samples; pause/Step/suspension cannot fabricate live
FPS. HUD uses the existing<=4Hz live update seam with aria-live off; no extra RAF,
timer or array. Six-file source-hashed assets include this module. Legacy bundles
are intentionally unavailable to the new server; rebuild/restart keeps the
private Wasm signature and loader paired, with native SSR fallback retained.

| New verification | Result |
|---|---|
| `cargo test --offline --locked --manifest-path examples/physics-lab/Cargo.toml --features browser yt197` | 4passed,9filtered: names/new preset bounds,initial pose/atomic reset,tower impact/domino propagation,native scene/HUD/rejection |
| `node --test examples/physics-lab/frame-rate.test.mjs` | 3passed,0failed: warmup/submission counts,reset/idle separation,invalid/backward time |
| Existing exact bindgen build-assets plus default native server build,offline/locked | PASS; six-file bundle9d025c5bf2c63fa4368ceee08a563998eb94f188e3f0df1578ac9fcf850bb43c |
| Actual new browser observations | 7accepted scoped observations across two retained receipts; details below |

Local artifacts are outside the checkout at
`/tmp/yetlet-yt197-scenes-fps-20261026-m3348cp5/`. Canonical focused command:
`python3 <evidence-directory>/browser.py`. evidence-index.json identifies the
accepted fields from each receipt; an overall FAIL is not relabeled PASS.

Firefox154/Linux/private packaged Weston used a disposable opt-in WebGPU profile,
without forced adapter/backend, system install or permanent configuration change.
Native server listened on ephemeral127.0.0.1. New selected SSR/initial noWasm,
stack firstRun and finite rate/numeric HUD,Pause sample retirement,staged scene
selection preserving world/poses/ticks,paused Reset to dominoes,domino fresh rate
window and Step keeping Paused/no live rate passed. Running numeric HUD was
observed for stack; the domino snapshot rate was positive while its instantaneous
HUD screenshot remained Measuring inside the<=250ms update interval. Both images
were inspected; this is not an assertion of GPU presentation FPS or a benchmark.
Application adapter text supplies only BrowserWebGpu without usable device/vendor
identity, so no hardware/device qualification is inferred.

The first browser receipt is retained FAIL: keyboard option navigation left tower
selected after the preceding new checks passed. One canonical harness correction
uses WebDriver option Click by exact value. The remaining flow only initializes/
pauses a tower as setup,then checks uncompleted selection/Reset/domino/Step; it
does not replay the accepted stack rate assertion. No product repair was needed.
Both allocations closed the owned server/Firefox/WebDriver/compositor and removed
profile/socket. No old accepted tests or business/GPU/benchmark matrix was rerun.
Independent acceptance was pending at that evidence checkpoint; final closure follows.


## Final acceptance and changed paths

Independent whole-item Reviewer PASS,0issues/required followups. All13source/
23evidence bindings,1,126protected tracked hashes,six-file asset bundle and both
browser asset inventories matched through review. Native reviewer/gpt-6-astra/high
role/model/effort telemetry matches the shared-resolver decision. No extra test
or resource launch was performed by review. YT-197 is Done at this bounded scope.

Changed files (13):

- `examples/physics-lab/src/model.rs`, `src/physics.rs`, `src/server.rs`,
  `src/bin/build-assets.rs` (remaining paths relative to that example).
- `examples/physics-lab/loader.js`, `style.css`, `frame-rate.mjs`,
  `frame-rate.test.mjs`, `README.md` (remaining paths relative to that example).
- `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/README.md` and this record.

BFCache's two previously accepted documents were committed first as89de4c8,
strict/builtin directGuard and hookPASS2files/0findings,
tree35b65912091e66288c55928650863bde311b17ca,noPush. New YT-197 changes remain
unstaged/uncommitted, noPush. Only completion metadata and documentation checks
follow the accepted source binding; all runtime/consumer source is preserved.
No other Todo remains outside the YT-096/020 release decisions. Slope presets,
GPU presentation timing and broader browser/device qualification are unselected
followups, not additional acceptance requirements or started work.
