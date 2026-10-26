# YT-193 — Isolated 3D physics laboratory prototype

Date: 2026-10-26. Status: **Done at the minimum isolated prototype scope after independent
whole-item Reviewer PASS, zero issues/required follow-ups.**

## Authority and baseline

The maintainer authorizes committing the four reviewed YT-192 research documents
with strict/builtin/no Push, then implementing its minimum isolated prototype,
opt-in/application-owned, no existing renderer API change. Preserve failures;
do not replay successful old qualification; independent Reviewer before Done.
New implementation changes remain unstaged/uncommitted/unpushed.

The research commit is `6614c76f81a297dca3857f1fbe6820e11534ccb3`, tree
`85a9fe3e231311051c8f2af9f699c45d226f00c0`. Direct/hook Guard exit0/PASS,
zero findings; four committed paths/hashes matched and checkout clean, no Push.
The [reviewed YT-192 sections1–7](YT-192-3D-PHYSICS-LAB-STUDY.md) are the
implementation contract. No historical report/source/support statement is rewritten.

## Selected implementation

New `examples/physics-lab` owns a separate unpublished Cargo workspace/lock.
Native default is HTML-first SSR without physics/GPU normal dependencies. Its
explicit browser feature selects Rapier0.36.1 CPU physics and target-specific
wgpu27.0.1/wasm-bindgen0.2.129/web-sys0.3.106. Private code owns preset/clock,
world/last-valid snapshot, 3D primitive/depth/camera pipeline, page controls,
asset generation and lifecycle. The existing yetlet-wgpu crate/API and all
Core/default graphs remain unchanged. No generic scene/engine/service is added.

The normal server accepts only loopback listeners, serves bounded GET inputs,
loads validated source-hashed immutable assets at startup, and keeps initial
HTML meaningful if JS/assets/GPU are absent. Explicit activation alone imports
Wasm. The world starts paused. Limits, fixed step/catch-up/drop/Step/Reset,
paused picking, failure/generation/retry semantics follow the reviewed contract.

A matching bindgen CLI was missing; it is built under the ignored task-local
`target/yt193-tools`, not installed globally. Needed public locked dependencies
were fetched only for the new workspace. Existing root/renderer/example locks,
source, browser profiles, services and system settings are not changed.
Read-only preflight observes X0/X1 sockets and Intel vendor8086/devicea780;
actual browser/adapter must still be recorded before GPU acceptance.

## New verification and retained evidence

Evidence lives under `target/yt193-physics-lab-20261026/evidence/`. The pre-edit
implementation contract and clean source hashes bind scope. New tests exercise
bounded presets/non-overlap, clock debt/pause/refusal, real Rapier gravity/contact,
atomic reset/zero gravity, invalid-state freezing, paused collider selection,
and actual SSR/failure behavior. Initial six tests pass, zero failures. They are
new-consumer checks, not old Core/M11 regression replay.

The exact new Rapier/wgpu Wasm release build and source-hashed asset publication
pass. Initial GPU build reports two nonblocking upstream glam deprecated helper
warnings; private code does not add a public compatibility alias. Native server
build and default normal dependency-tree capture complete separately. Browser
validation uses a task-owned loopback server, existing X11 and disposable Firefox
profile/driver with only the selected dom.webgpu.enabled opt-in. The Browser
plugin/skill is absent and CUA does not supply Firefox; the existing repository
Firefox WebDriver approach is used for this new consumer. No old one-shot script
is run, no old evidence directory is overwritten, no backend/browser is silently
substituted. GPU/runtime acceptance remains pending until actual observations.

Any failed attempt will retain its exact log/result; classify failures and make
only material bounded corrections under the existing Materiality Gate. Cleanup
must stop owned server/driver/browser and remove the task profile. Tests/builds,
source/asset bindings, real rendered/interaction evidence and reviewer result
will be recorded here before Done. YT-096/D1–D6 and YT-020 remain unchanged.


## Retained first-phase browser failure and minimum correction

Two task-owned Firefox154/X11 attempts did not reach a GPU draw. The first
confirmed meaningful initial HTML and no Wasm/module fetch before Activate,
then timed out waiting for activation. No adapter/3D/lifecycle PASS is claimed.
Its driver/server logs and failed result are frozen; no old M11 test was replayed.

The observed indefinite initializing state violated the selected named-failure
boundary. The minimum product correction adds private stage reporting and a
15-second GPU-acquisition UI deadline. Timeout retires the owner, pauses physics
and keeps a borrowed pending initialization alive until settlement; activation/
retry stays disabled while cleanup is pending. Aborting an owner cancels its
owned deadline. This does not cancel a browser driver operation, free an active
Wasm borrow or authorize a second pending owner.

The same-target follow-up, without repeating accepted SSR/CPU checks, reports
`adapter` stage, named initialization-timeout, 29 retained initial records,
valid CPU state and zero ticks/submissions. Thus the Wasm module and CPU world
initialize in Firefox, while actual adapter acquisition remains unavailable.
Both failures remain failed; no further X11/adapter retry is selected. Driver
and server stop, both observed Firefox main PIDs are absent, and task profiles
are removed. WebDriver session-delete timed out; its error remains recorded,
with owned process termination checked separately rather than relabeled clean.

All six earlier CPU/SSR tests remain accepted at their recorded source behavior;
only private GPU/controller/builder changes follow. New bounds/source-hashed
assets build successfully. Upstream deprecated camera helper calls from the
initial build are replaced by their canonical right-handed 0–1-depth functions;
the initial two warnings remain in that original log. The asset builder now
fixes its own target directory so inherited Cargo output overrides cannot publish
a stale workspace artifact. No original renderer/source/lock changed.

## Environment follow-up ordering and proposed target delta

The owner recalls an independently launched Wayland environment and later
clarifies that environment verification passed, possibly without a complete
file update. Read-only inspection locates the original
`target/m11-gpu/yt075-20261014/weston-x11-preflight/result.json`: ready=true,
exit0 and no remaining socket. Its exact bytes/hash are retained in the new
prior-environment receipt. This proves that historical isolated Weston preflight,
not this rebuilt host or new application's drawing. The earlier subsequent
Chrome/NVIDIA drawing failure stays a separate result; neither erases the other.

The owner then asks to continue the original prototype first and handle this
environment as the second phase, allowing sequencing judgment. Finish source
review before attempting the changed target. Proposed second-phase candidate:
the same installed Firefox154 and Intel WebGPU requirement, using Wayland on
one task-owned Weston14.0.2 compositor nested under existing X11. Use only one
new socket/profile/attempt, normal GL/Vulkan discovery and dom.webgpu.enabled;
no browser/driver update, forced GPU/unsafe browser flags, desktop-session change
or replay of old qualification. The exact plan requires independent target
review before launch under YT-192's admission boundary.

Only the two official Ubuntu Weston14.0.2-5 packages were downloaded/extracted
under ignored task tools. Dynamic-library resolution and --help succeed using
private library/module paths; no apt installation or global package database,
profile, driver or service is modified. Current host installed Weston remains
absent. A proposed own compositor/socket and browser will be removed/stopped
when that phase ends. Subsequent owned Wayland attempts and cleanup are recorded below.

At that source-phase checkpoint YT-193 remained In progress. Initial3D rendering, browser Run/Step/Reset,
3D picking/camera, actual loss/retry, late completion and disposal evidence are
were still required, followed by final independent acceptance before Done. Source-
only review or an environment preflight cannot substitute for those observations.


## First independent source review and material correction

Independent `/root/yt193_source_and_wayland_admission_review_20261026` returns
FAIL with one P2: CPU picking filtered out static walls before selecting the
nearest collider, so a depth-occluded dynamic body could be selected through an
opaque wall. This violates the reviewed rendered-selection contract, rather
than a nonblocking style suggestion. Source/evidence hashes remained unchanged
during review; native role/model/effective effort matched the saved profile and
central resolver. The original failed review is retained.

The smallest correction computes the nearest hit over all scene colliders;
a nearest static enclosure hit returns no selectable object. One new focused
regression covers wall-obscured and unobstructed rays; the six earlier accepted
tests are not replayed. Only this changed product behavior is verified again.
The same review admits the private Weston/Wayland target plan within its recorded
attempt, ownership, stop and cleanup bounds. It does not grant final runtime
acceptance: actual 3D/browser/lifecycle evidence and final Reviewer remain open.


## Source acceptance and second-phase observations

The source-only follow-up Reviewer passes with zero issues or required source
follow-ups after the static-occlusion fix. The one new regression passes with
six previously accepted cases filtered out. Current source-hashed bundle is
`30389b0ed53fc663e96173c3a9443e02fc8e6428e2a5da0dad99a58ab755693b`.
This accepts the source correction and admitted Wayland plan; whole-item runtime
acceptance remains separate.

The first private Weston/Wayland attempt actually initializes WebGPU, submits
and completes 3D frames, and retains a valid paused CPU snapshot. The captured
`wayland-initial-3d.png` visibly shows cubes, spheres and an opaque depth-tested
enclosure. Firefox154 is unchanged; only the fresh owned profile selects WebGPU.
The compositor uses Mesa llvmpipe for its own GL presentation. That does not
identify the browser's WebGPU adapter, and is not an acceleration claim.

The attempt remains FAIL: its subsequent content-context navigation to
`about:support` is rejected by WebDriver before interaction/lifecycle checks
start. This is a harness failure, not a product draw failure. The smallest
observation correction enables the documented geckodriver
[`--allow-system-access`](https://firefox-source-docs.mozilla.org/testing/geckodriver/Flags.html#allow-system-access)
only on the loopback-owned driver/fresh profile to read its graphics diagnostics.
It does not change a web security policy, GPU backend, sandbox, browser version,
user profile or installed driver. Diagnostics never visit personal content.
The next attempt successfully navigates there but returns empty row selectors;
that second, distinct harness failure is preserved. Its smallest observer fix
waits for asynchronously formatted graphics text and reads visible own-profile
metadata instead of version-sensitive row IDs. No GPU/backend search is added.

All failed JSON/logs/screenshots are frozen before the focused continuation.
Each attempt stops its owned server/driver/compositor, verifies the observed
Firefox main PID absent and Wayland socket removed, and removes its task profile.
The original six CPU/SSR checks, prior M11 evidence and accepted no-Wasm-before-
Activate proof are carried forward, not rerun. Only still-open browser cases
and physical target identity are pending; continuation startup is preparation,
not a new qualification of already accepted behavior.


## Reviewed actual target and remaining runtime results

The formatted diagnostics invalidate the observer's broad-text Intel assertion:
Intel is inactive inventory, not the WebGPU selection. Its retained check label
is explicitly rejected. `wayland-observed-graphics.json` identifies the default
WebGPU adapter as NVIDIA GeForce RTX4090, Vulkan, driver595.99.02,
vendor4318/device9860, DiscreteGpu, nonfallback. Window Protocol is Wayland;
WebRender/GL uses llvmpipe. The private wgpu27 adapter-info string is deliberately
empty upstream, so this is browser default-adapter diagnostics plus source
inspection of normal default adapter selection, not direct app-device telemetry.

Independent observed-target admission passes with zero issues/follow-ups before
continuation on that same installed Firefox154/private Wayland normal-default
NVIDIA target. No forced adapter/driver/backend setting or browser update is
introduced. The original Intel/X11 failures remain unavailable, and historical
Firefox158 evidence remains historical. This application-only target amendment
does not qualify existing renderer support or other browsers/hardware.

The observed flow verifies numeric camera redraw without ticking; a trusted
pointer click on rendered Object24 updates the correct HTML selector/detail;
Single Step advances exactly one tick; Run/Pause freezes ticks; a real hidden
tab pauses and return does not auto-resume. It then stops with a retained FAIL
at a zero-size fixture: width0 leaves a nonzero border box. No actual zero
rectangle was established and no product failure is inferred. One smallest
fixture correction uses display:none and asserts the actual bounding rectangle
is zero before checking suspension. Earlier successful interaction cases are
carried forward rather than replayed.

`browser-wayland-remaining-result.json` passes the still-open cases: actual
zero-size and detached surfaces suspend, Step is refused while non-drawable,
restoration stays paused; atomic64-object/zero-gravity Reset; controlled actual
device destruction freezes CPU poses; explicit Retry advances GPU generation
without resetting/ticking the retained world; an injected old-generation fault
cannot replace the new state; Dispose leaves no RAF. The recovered64-object
screenshot is retained. These are controlled loss/injection cases, not a
spontaneous hardware-fault experiment or throughput measurement.

One last focused lifecycle check, required by the original late-init contract,
uses the same admitted target and unchanged source. All earlier accepted cases
are carried forward. `browser-wayland-lifecycle-result.json` passes duplicate
activation refusal; controlled Dispose at the real pending device-init stage;
settled borrowed future without a late published owner/tick/draw; ignored stale
completion/progress/fault injections; retained sibling HTML/table; fresh explicit
activation after settled retirement into a new valid paused world; final Dispose.
It does not simulate every driver failure or qualify a second active canvas.

All Wayland attempts stop their owned driver/server/compositor, remove their
fresh profile and own socket, and observe the recorded Firefox main PID absent.
There is no system package installation, service change, cloud resource,
default-browser profile change or desktop-session replacement. Private verified
Weston package extractions and evidence stay under ignored target directories.

## Focused verification commands and evidence limits

| New-delivery check | Result and retained receipt |
|---|---|
| `cargo test --offline --locked --manifest-path examples/physics-lab/Cargo.toml --lib --features browser` | Six initial behavioral tests pass; initial-product-tests.log. Not replayed after unrelated private controller/GPU changes |
| Same command with filter `static_enclosure_occludes_hidden_objects_but_clear_ray_selects` | One new occlusion regression passes, six filtered; occlusion-regression.log/json. Source-only follow-up PASS0 |
| `cargo run --offline --locked --manifest-path examples/physics-lab/Cargo.toml --bin build-assets` with task-local exact WASM_BINDGEN | Frozen exact browser-feature release build, canonical bindgen and current hashed bundle PASS; occlusion-corrected-assets.log. Original warnings/intermediate stale-caller receipt retained; no browser uses that intermediate bundle |
| Native `cargo build --offline --locked --manifest-path examples/physics-lab/Cargo.toml --bin yetlet-physics-lab` | PASS, bounded-native-build.log; later browser source correction is physics-only and not in the default native graph |
| Default native `cargo tree --offline --locked --manifest-path examples/physics-lab/Cargo.toml -e normal` | No Rapier, wgpu or wasm-bindgen edge; default-normal-graph.txt |
| Task-owned Firefox WebDriver on admitted private Weston/Wayland target | Actual source-bound 3D plus focused interactions/lifecycle above; preserve all FAIL/PASS JSON, driver/server/compositor logs and screenshots |
| New-delivery formatting/JS syntax/docs/whitespace and preserved source hashes | Source format/syntax PASS; final docs/scope receipt and independent whole-item result recorded on closeout |

This is a working isolated prototype, not a public package/sample-catalog entry,
FPS/capacity benchmark, engineering simulator, cross-platform support claim or
production qualification. The bounds are requested application buffer/pixel/
scene limits, not resident GPU/process memory measurements. GPU behavior is only
observed on the admitted current browser/target. Native default remains useful
without GPU/physics, and Core/old renderer APIs/locks/old raw proof stay unchanged.
No new implementation Commit/Push or publication; YT-096/D1–D6 and YT-020 remain
unchanged. Final independent acceptance is required before completion metadata.


### Visual-recovery closeout observation

Manual inspection of the first recovered64 screenshot finds a blank canvas.
The retained state/lifecycle assertions do not establish a visible recovered
frame; that visual claim remains unaccepted. Driver logs also contain a WebGPU
buffer warning and Wasm unreachable error around teardown, so these are not
silently discarded as a fully clean browser log. No product cause is assumed
from a screenshot alone. One focused same-source/same-target observation checks
current-scene submission/completion before loss and after Retry, holds the image
for a bounded compositor interval and captures live page errors separately from
navigation teardown. It does not replay already accepted CPU/SSR/interactions.
Final runtime/whole-item acceptance waits for those actual observations.


The focused visual observation completes without a source change:
`browser-wayland-visual-result.json` records fresh scene2 submissions/completions
both after Reset64 and after controlled loss/Retry; paused poses remain retained.
The before-loss and after-Retry screenshots visibly contain the same complete
bounded64-body scene. `live_page_errors` is empty before disposal. Waiting for
fresh draw completion and a bounded compositor capture interval closes this
observation; it is not a frame-presentation/FPS guarantee or a replay of other
accepted cases. The earlier blank capture and teardown errors remain retained
and do not count as recovered visual evidence. Their exact causal timing is
not inferred from the later success. Final independent review receives both.


## Final independent acceptance and completion

Independent `/root/yt193_source_and_wayland_admission_review_20261026` returns
**PASS, zero issues, zero required follow-ups** for the whole minimum prototype.
The Reviewer visually inspects both successful recovery images, confirms69
identical records (64 dynamic/five static) and zero ticks before/after Retry,
and verifies all17 source plus28 selected evidence hashes unchanged before/after
review. Saved workspace routing and the central resolver match native registered
reviewer/gpt-6-astra/effective high; no recovery/escalation or test replay occurs.
`final-review-baseline.json`, `final-review-result.json` and bounded terminal/
observed-decision receipts retain acceptance separately from prior source FAIL,
source-only PASS, target admission, runtime FAILs and visual observations.

After PASS, only this completion record and Backlog/Roadmap/index metadata change.
YT-193 is Done at the accepted isolated application scope. All17 changed paths
are listed in the final closeout receipt: the new standalone example's13 files,
this delivery document and three planning/index documents. Existing1,110 tracked
non-target hashes remain unchanged. Staged0; HEAD remains the four-document
YT-192 research commit6614c76; no new Commit/Push, publication or deployment.
Final `cargo xtask check docs` and whitespace/scope closeout follow this metadata
update; source/asset/native/browser qualification is not repeated.

No further item is automatically activated. The next available work, if selected,
is to try/polish this private example under the same scope; sample-catalog
promotion, shared-renderer extraction, another GPU/browser or performance work
needs a separate bounded decision. Existing YT-096/D1–D6 stay Pending/In progress
and YT-020 stays Blocked; neither is changed by this prototype.
