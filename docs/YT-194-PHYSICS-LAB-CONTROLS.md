# YT-194 — Physics laboratory Run/Pause/Reset controls

Date: 2026-10-26. Status: **Done at the bounded application-controls scope after independent
Reviewer PASS, zero issues/required follow-ups.**

## Authority and bounded contract

The owner requests Run/Pause/Reset, firstRun initializes, SingleStep in Advanced,
Retry only after failure, removal of Activate3D/Dispose buttons, opt-in/safety
preserved, focused verification and independent Reviewer before acceptance.
No Commit/Push. Start at reviewed YT-193 commit61d1ed4, clean. Follow
[ADR0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) and the existing
[application limits](YT-192-3D-PHYSICS-LAB-STUDY.md). This explicit UI decision
supersedes its Activate-then-paused interaction; old research/acceptance and raw
proof remain historical and are not rewritten or replayed.

| Action | Current selected behavior |
|---|---|
| Initial page | Native scenario/GET parameters remain meaningful; no Wasm/device request before first Run; unavailable assets leave Run disabled |
| First Run | One explicit opt-in loads the existing module/world/GPU, shows Preparing, then starts only if the live owner, valid world and canvas remain eligible |
| Pause | Retain poses, stop RAF/ticking, clear timing debt. During preparation cancel pending start; finish preparation paused rather than starting later |
| Later Run | Continue that same paused world/generation from a fresh timestamp; no new module/device/world |
| Reset | Validate current preset before mutation, pause a running world, atomically rebuild within the same limits, zero counters and remain paused. BeforefirstRun native GET requires no physics/GPU; pending preparation/cleanup refuses Reset |
| Single Step | Only in initially collapsed Advanced controls; one fixed1/60s step while ready/paused/visible; no implicit initialization or running-world step |
| Retry | Hidden normally; visible only lost/unavailable, disabled during prior retirement. One explicit attempt preserves valid CPU state and returns paused. Pre-world failures may retry; invalid CPU requires Reset |
| Hidden/offscreen/zero/detached | Cancel pending start or pause active simulation. Return to visibility never auto-resumes |
| Disposal/late callbacks | No public Dispose button; unchanged internal navigation/diagnostic retirement still waits on borrows and rejects stale generations |

The numeric camera controls/counters join SingleStep in Advanced; ordinary
Run/Pause/Reset stay beside the canvas. Reset is the native form's associated
submit button, preserving noJS parameter selection. Preparing blocks duplicates;
Pause may cancel a pending start. Errors retain the existing named failure and
explicit recovery semantics. No new renderer/physics/Core/public API, package,
feature/settings key, dependency/lock or system/browser/driver change is selected.

## Changed files and evidence plan

Only example controller, SSR (including one focused action-contract test), CSS,
README, this record and Backlog/Roadmap/index. Existing GPU/physics/clock/builder
source and manifests/locks are unchanged. Native default remains physics/GPU-free;
server-only code still cannot enter the browser graph. Existing CSP, escaping,
asset hashing, preset validation, resource bounds and failure lifetime remain.

Evidence is retained under `target/yt194-physics-lab-controls-20261026/evidence/`.
One new native SSR test covers primary actions, native Reset association, closed
Advanced/Step, hidden Retry, missing-asset fallback and selected scenario. It
passes without replaying prior cases. Canonical builder/native builds select
changed source-bound assets; unchanged Wasm behavior and prior qualification are
carried forward. JS syntax, Rust format and new-delivery docs/whitespace are
checked separately from browser behavior.

Use the existing repository Firefox WebDriver approach: task-owned loopback
server, fresh profile selecting WebGPU, private Weston14/Wayland and normal-default
NVIDIA Vulkan on the admitted Firefox154 target. No backend/browser sweep, unsafe
flags, system installation or old one-shot scripts. Check only changed controls,
initial noWasm, pending/duplicate/cancel/visibility behavior, Reset from running,
Step and explicit failure/Retry. Label injected cases and preserve any failures.
Cleanup owned sessions/processes/profile/socket. The earlier user play preview
has already stopped and cleaned itself; its profile is not reused.

Only new browser proof and independent whole-item PASS may close this item.
YT-193's old CPU/GPU qualification, original failed receipts and immutable release
scope are retained. YT-096/D1–D6 and YT-020 remain unchanged. No Commit/Push or
publication; final source/evidence bindings and acceptance limits follow below.


## Observed correction and focused runtime evidence

The new SSR test passes1case/3filtered; old cases are not replayed. Native build,
canonical asset builder/Wasm build, JS syntax, Rust format and docs PASS. The
current content-addressed bundle is
`2f6afdf7a4a2b9bf24b0fa67346256e51f1c2d41953f90b4285acf186f306b69`.
Physics/GPU Wasm source is unchanged; only the controller component changes its
bundle hash. The browser loads the same source-bound assets as the native server.

The first two browser attempts retain failed assertion receipts. A diagnostic
continuation localizes a sandbox-counter observation problem: unqualified
primitive assignment updates the WebDriver sandbox rather than the explicit
window counter read by later commands. One smallest counter fix uses window
explicitly, with no product change. The initial controls/noWasm and native
preRun Reset pass and are carried forward.

A subsequent attempt finds a real Run integration defect: resume samples
performance.now, while a later RAF callback can carry the earlier frame's time,
so it is falsely rejected as a backward timestamp. The private queue now samples
performance.now at callback execution, consistently with resume. Fixed dt,
wall clamp, debt cap and the CPU clock's real backward/nonfinite rejection stay
unchanged. A focused browser case deliberately supplies an older RAF callback
timestamp and now advances normally. This is a material one-line controller
correction, not a weakened clock check or physics API change.

| Changed behavior | Accepted observation in retained receipts |
|---|---|
| Initial controls and opt-in | browser-controls-result.json: Run/Pause/Reset, Advanced closed with Step inside, Retry hidden, no Activate/Dispose; no physics module/Wasm before Run |
| PreRun Reset | Same receipt: native GET applies8-body preset without initializing Wasm |
| Pending duplicate/Reset/Pause | browser-controls-corrected-result.json: one adapter request/generation, zero ticks; duplicate Run and pending Reset refused; Pause finishes initialization paused |
| Run/Pause/resume and clock | browser-controls-clock-result.json: same world/generation, freeze while paused and continue from fresh time despite controlled older RAF timestamps |
| Running Reset | Same receipt: native invalid input refused; valid12-body/zeroGravity/0.5speed preset resets from running into paused scene, zero counters, same GPU generation |
| Advanced Step | Same receipt: exactly1tick while paused; disabled while running |
| FirstRun start | browser-controls-remaining-result.json: uninterrupted firstRun starts without a second click |
| GPU failure/Retry | browser-controls-final-result.json: controlled device loss exposes Retry; explicit recovery retains poses/ticks, advances generation and returns paused, hiding Retry again |
| Pending background/zero-size | Same receipt: real tab hiding and actual zero bounding rectangle cancel pending start; restoring visibility does not auto-run |
| Pre-world failure/Retry | Same receipt: injected navigator.gpu absence leaves no world and enabled Retry; restored capability retries once into valid paused state |

Every receipt's original overall FAIL remains FAIL. Accepted checks are bounded
observations completed before a later failure, not a relabeling of that run.
After the clock correction, a fixture's adapter gate inadvertently held its
later Retry; the smallest harness fix makes that gate one-shot. A trimmed local
variable then fails the final generation assertion; the smallest fix compares
the retained snapshot directly. Successful controls are carried forward; only
unfinished cases and necessary fixture preparation execute again. All original
logs/results and correction receipts remain preserved. No new backend campaign,
generic harness, old qualification replay or package installation occurs.

## Actual environment and acceptance limits

All changed action cases finish before the final environment assertion fails:
the script assumed the previous NVIDIA default. Current own-browser diagnostics
instead report **Intel(R) Graphics (RPL-S), Vulkan, vendor32902/device42880,
Mesa25.2.8-0ubuntu0.24.04.2, nonfallback**, in Firefox154 with Wayland. No GPU is
forced or driver changed. WebRender/compositor llvmpipe is a separate path.
`browser-graphics.json` and the final failed receipt retain those exact facts.
The target is the same installed browser/privateWayland/normal-default selection;
independent review must assess this observed target amendment together with the
current application proof. No further browser/adapter attempt is selected.

The app's wgpu27 identity string remains empty upstream. Diagnostics describe
the browser default at observation time; source uses normal default selection.
They are not direct app-device telemetry or proof of all adapters across the
flow. Original YT-193 NVIDIA and earlier Intel/X11 failures are not rewritten.
No general GPU/platform support, performance, spontaneous hardware-failure,
production or second-active-canvas qualification follows from this UI change.

Manual inspection of controls-recovered.png confirms the primary three buttons,
visible 3D cubes/spheres and hidden Retry/removed technical buttons. The first
main-flow page-error capture is empty. Routine Firefox startup/teardown messages
remain in logs; no claim of a universally empty browser console is made.
All owned driver/server/compositor processes stop, observed Firefox main PIDs
are absent, fresh profiles and sockets removed. At the pre-review checkpoint, final independent review and
unchanged source/evidence bindings were still required before Done.


## Independent acceptance and completion

Independent `/root/yt194_controls_independent_review_20261026` returns **PASS,
zero issues/required follow-ups** after reviewing source, the new SSR case,
retained browser assertions/limits and the recovered screenshot. All8 source
and18 selected evidence hashes match before/after review;1,120 non-target tracked
files remain unchanged. Native registered role/model/effective effort match the
healthy eligible workspace profile and central resolver: reviewer/gpt-6-astra/high.
No recovery, source edit or runtime replay occurs during review. Acceptance and
bounded telemetry are retained in review-result/review-terminal/observed-decision
receipts; no original failed receipt is rewritten.

The Reviewer admits the observed Firefox154/privateWayland/normal-default Intel
Vulkan metadata amendment at this application-controls scope. It does not turn
the empty app identity into direct device telemetry, erase the assumed-NVIDIA
assertion failure, or expand the earlier support inventory. Original clock and
harness failures remain failed; completed assertions and focused corrected
observations supply the bounded acceptance, not a fictional all-green run.

Only this completion record and Backlog/Roadmap/index metadata change after
PASS. YT-194 is Done; new source/asset/browser qualification is not replayed.
Final docs/whitespace/scope closeout follows the metadata update. The eight paths
are four planning/delivery documents plus example README/controller/SSR/CSS;
physics/GPU/Core/builder sources, manifests/locks and historical proof remain
unchanged. HEAD remains61d1ed4, staged0, no Commit/Push or publication.
No next item is automatically activated; the next bounded step is user play
feedback on these controls. YT-096/D1–D6 and YT-020 retain their prior statuses.
