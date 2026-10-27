# YT-195 — Browser startup and WebGPU failure guidance

Date: 2026-10-26. Status: **Done at guidance/loading and common pre-Wasm contract scope after
independent Reviewer PASS, zero issues/required follow-ups.**

## Authority and scope

The owner selects the ordinary-browser startup contract, then minimum docs and
state guidance, opt-in/safety preserved, no accepted qualification replay,
independent Reviewer before completion, no Commit/Push. Baseline12ed649 is clean.
[ADR0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md),
[YT-194 controls](YT-194-PHYSICS-LAB-CONTROLS.md) and the original application
scene/clock/resource/lifetime bounds continue. No Core/renderer/physics API,
package/feature/settings key, dependency/lock or deployment change.

The owner subsequently clarifies that loading/splash means general Wasm website
startup, before the runtime itself can draw, not only this lab. The
[common pre-Wasm shell contract](WASM-STARTUP.md) is therefore part of this slice:
initial HTML/CSS, custom content/style, page/island placement, explicit readiness,
error/retry and stale-owner rules. Existing browser-template seams are inspected
read-only; YT-196 tracks common-helper/template adoption without pretending it
is implemented now. More lab scenarios and visible FPS remain owner feedback;
no scenario/FPS/benchmark or generic helper/Core API is implemented here.

## Startup contract

| Condition | Minimum guidance and behavior |
|---|---|
| Ordinary browser | Open the example's local URL with JS/Wasm and available WebGPU; Wayland is an existing verified host route, not an app prerequisite or a browser support guarantee |
| Local/remote address | Native listener stays loopback-only. Default127.0.0.1:3016 means the browser's own machine; remote use requires an intentional loopback port forward or a separately provided trusted HTTPS deployment |
| Secure context | Use local loopback/localhost or trusted HTTPS. Check isSecureContext on explicit Run before module/device work; do not recommend insecure-origin, certificate, blocklist or sandbox overrides |
| Files/JS unavailable | Useful initial table/native Reset remain; disabled Run plus static Browser help/noscript explains JS or unavailable server assets |
| First Run | Keep the sole initialization opt-in; prepare once, show local nonblocking loading feedback, then preserve YT-194 start/pending-Pause/visibility behavior |
| Preparation | A decorative per-canvas spinner accompanies the existing accessible status/aria-busy, respects reduced motion and invents no percentage. SSR/controls remain available under existing pending-action guards |
| Failures | Static bounded category/message identifies the affected 3D operation and an appropriate next action, opens Browser help; no raw exception/URL/stack is inserted into UI |
| Retry/timeout | Existing explicit one-attempt Retry, valid CPU retention, invalid-world Reset and borrowed-future retirement remain. Explain disabled Retry while cleanup is pending and reload's loss of tab-local simulation |

This application-owned loading decoration may be styled locally; it is not a
mandatory full-page splash; the general contract is documented separately. UI content does not
expose driver commands or framework implementation details. No automatic device
probe on page load, new fallback backend, auto-retry or browser configuration
change is selected.

## Evidence and bounded validation plan

Read current primary API references on2026-10-26:
[Navigator.gpu](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/gpu),
[secure contexts](https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Secure_Contexts)
and [adapter selection](https://developer.mozilla.org/en-US/docs/Web/API/GPU/requestAdapter).
Capability exposure is not proof of a usable adapter/device. Browser version,
platform, policy and GPU state affect availability. The
[Chrome diagnostic reference](https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips)
is used for its diagnostic page and failure distinctions; unsafe bypass advice
on that page is not adopted. No fixed all-browser version matrix is promised.

Only new SSR guidance markup, the changed preflight/error presentation and local
loading/reduced-motion states need focused checks. Existing Run/Pause/Reset/Step,
physics, GPU success/recovery/retirement and support results are carried forward
from YT-193/194, not replayed. The new user-authorized state branches may use
bounded injected browser failures in the existing task-owned Firefox WebDriver
pattern; label these as message/loading evidence, not ordinary-browser/GPU
qualification. No new harness framework or environment search campaign.

Evidence under target/yt195-browser-startup-20261026/evidence retains source,
new native test/build/current assets, formatting/syntax/docs and scoped browser
observations. Owned server/driver/browser/compositor/profile/socket must clean
up. Existing play preview is already stopped/cleaned. Historical raw FAIL/PASS
receipts stay unchanged. Only independent acceptance may close this item.


## Clarified common-shell boundary

The earlier draft restricted loading to the lab; the owner corrects that scope.
The current common guide explicitly addresses ordinary browser-only Wasm apps,
SSR islands and intentional fullscreen shells. It distinguishes Wasm from
WebGPU and requires initial HTML-linked CSS before the Wasm import. Existing
browser tasks/template source already has pre-Wasm loading text and disabled
controls, with stylesheet/static-import limits now recorded honestly. No CLI/
Tasks source, generator qualification or generic runtime is modified. YT-196 is
Todo for the reusable customizable shell; this is the next unblockable follow-up.


## Focused evidence before review

| Check | New result |
|---|---|
| `cargo test --offline --locked --manifest-path examples/physics-lab/Cargo.toml --lib --no-default-features browser_help_and_loading_shell_survive_without_runtime_assets` | PASS1,4filtered. New HTML/noJS/help/fallback/decorative-loading assertions only; prior cases not replayed |
| Canonical `build-assets` with task-local exact wasm-bindgen | PASS; current bundle9f48ec74cf7c4530028d64d5141a6cce906bf66d773909da4829d5639ed34ac6. Published loader matches source; previous Wasm payload remains unchanged |
| Native changed server build | PASS; no manifest/dependency/physics/GPU source change |
| JS syntax, Rust format, whitespace and docs | PASS;75required documents/196backlog items, links/structure/whitespace/DAG |
| New message/loading browser flow | PASS,9bounded checks in browser-guidance-result.json; no old Run/Pause/physics/GPU qualification replay |

The browser flow uses the same installed Firefox154/privateWeston Wayland,
fresh task profile selecting WebGPU and the documented
[reduced-motion preference](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion).
This profile setting is isolated and removed afterward; no user/system/browser
configuration changes. Actual CSS reports reduce=true and animation=none while
the decorative overlay is visible and existing status/aria-busy communicates
preparation. Captured loading-reduced-motion.png and timeout-guidance.png show
the intended regional feedback and recovery text.

Injections exercise only the changed guidance: insecure-context and absent-API
preflight; actual Wasm-instantiation rejection; null adapter; a normally obtained
adapter whose device request deliberately rejects; a controlled watchdog
callback while an adapter promise is held; and an injected unknown fault.
Each produces its bounded category/action, opens help, hides loading, clears
busy state and omits a private diagnostic marker from user text. The timeout
case proves Retry disabled while the borrowed initialization remains pending;
settling that promise enables explicit Retry without ticking. The deadline
injection is not a fresh15second elapsed-time qualification or spontaneous
hardware failure. No device-success/draw/FPS/capacity claim follows.

The existing script/profile/compositor/loopback ownership pattern is reused,
without a new harness framework or browser/backend sweep. Browser, driver,
server and compositor stop; fresh profile/socket are removed and the observed
Firefox main PID is absent. All prior raw failed receipts remain untouched.
Common-shell source inspection does not rerun CLI/browser-tasks qualifications.
A new reusable component is still YT-196 Todo; more scenes and FPS stay separate
owner feedback. Independent Reviewer and unchanged bindings remain required
before completion. No Stage/Commit/Push or publication.


## Independent acceptance and completion

Independent `/root/yt195_startup_guidance_independent_review_20261026` returns
**PASS, zero issues/required follow-ups**. The Reviewer inspects nine source
paths/diffs, prior contracts, shared template seams, the actual focused browser
script/results/logs, new SSR/build/docs receipts and both screenshots. All9
source and12 selected evidence hashes match before/after review;1,121 protected
tracked hashes remain unchanged. HEAD12ed649/staged0 are preserved. Native
registered reviewer/gpt-6-astra/effective high matches the healthy eligible
workspace routing and shared resolver; no recovery or replay occurs in review.
Acceptance and bounded trace receipts remain under the task evidence directory.

Only this completion record and Backlog/Roadmap/index metadata change after
PASS. YT-195 is Done; YT-196 remains Todo for the reusable customizable pre-Wasm
helper/template adoption. The general startup contract is not described as an
already implemented component. More scene variety and FPS remain deferred owner
feedback. No next implementation is automatically activated.

The nine paths are this delivery record, common WASM-STARTUP guide, three index/
planning documents and the example README/controller/SSR/CSS. Physics/GPU/Core,
CLI/browser-tasks/template sources, manifests/locks, existing qualification and
historical raw evidence stay unchanged. Final metadata docs/whitespace/scope
closeout follows; new native/browser qualification is not repeated. No Stage,
Commit/Push, publication or deployment. YT-096/D1–D6 and YT-020 stay unchanged.
