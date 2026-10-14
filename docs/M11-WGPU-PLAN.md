# M11-G — Official Opt-in wgpu Renderer Plan

**Date:** 2026-10-12. **Planning task:** YT-069. **Implementation proposals:** YT-075–YT-080.
**Status:** Proposed, documentation only. The owner requested planning for `yetlet-wgpu`; no crate extraction, dependency upgrade, browser execution or support declaration is authorized by this document.

Read the [M11 plan](M11-PLAN.md), [product research](M11-PRODUCT-READINESS-RESEARCH.md), [proposed ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) and [original YT-023 research](YT-023-WEBGPU-RESEARCH.md). Product-readiness work does not depend on this line; neither line depends on publishing the original nine candidates.

## 1. Proposed product and promotion boundary

The intended endpoint is a **first-party, explicitly selected, Experimental `yetlet-wgpu` renderer for a bounded 2D scene surface**. Official maintenance, default activation, API stability, package publication and production qualification are different decisions. This plan proposes official opt-in ownership, not a default GPU runtime or a stable release.

Retain the research fixture and its original decision/evidence. A new accepted implementation cannot retroactively turn its Vulkan measurements into browser results. The current `crates/yetlet-core` View remains Text/Element; do not add a GPU scene variant, renderer-global switch, CSS layout engine or mandatory game loop.

| Boundary | Proposed owner |
|---|---|
| Native HTTP/SSR, metadata and ordinary controls | Existing Yetlet/Axum/HTML path; unchanged without opt-in |
| GPU surface | One application-owned canvas, then an optional adapter after proof |
| Scene data and typed local intent | Small application types initially; no prematurely generalized scene graph |
| DOM labels, controls, selection detail and accessible alternative | Application HTML/DOM; not a GPU text editor |
| Remote work | Existing explicit typed action/HTTP boundary; no raw pointer-event circuit |
| Device/buffer/frame lifetime | Explicit renderer owner; no ambient global device or scheduler |
| Native graphics host, if later selected | Separate host adapter and separately qualified targets; not forced into the browser bundle |

A `yetlet-scene` crate, general scene hierarchy, arbitrary shader plugins, text shaping, image pipeline, physics/ECS, game engine, compute framework and general SVG renderer are outside M11-G. Require a second materially different consumer before a shared scene-model extraction. A second instance of the same demo is not sufficient evidence for a general abstraction.

## 2. Baseline: what exists and what it does not prove

**Source-confirmed at `c7ecc49eeb667e145cfb7e764c10b72ba104bbee`:** [research manifest](../examples/webgpu-scene-research/Cargo.toml) enables `wgpu = 27.0.1` only through `gpu`, with std/Vulkan/WGSL; [scene source](../examples/webgpu-scene-research/src/lib.rs) builds a fixed flat grid; [benchmark](../examples/webgpu-scene-research/src/bin/scene-bench.rs) initializes native offscreen resources, uploads once, draws, verifies readback and waits per timed submission. It has no Yetlet dependency, browser canvas surface, dynamic scene update or displayed frame loop.

**Existing measured evidence, not rerun:** [YT-023](YT-023-WEBGPU-RESEARCH.md) retains RTX 4090/Vulkan all-center draw proof at 10k/50k/100k instances. Median CPU encode + submission + completion wall times are 0.033760/0.054286/0.083525 ms. These exclude preparation/upload/setup/readback and are not display FPS, browser paint, pure GPU timestamps, dynamic-update costs or superiority over another renderer. Explicit buffer/texture lengths are not measured resident VRAM/RSS.

**Unknown:** browser portability, adapter refusal/loss, displayed rendering, resize/DPR, update/picking/interaction costs, low-power devices, cross-OS backends, and meaningful bundle/startup tradeoffs. The current 4090 result cannot qualify an integrated GPU or a browser support range.

### Version decision

Upstream [wgpu 27.0.1 documentation](https://docs.rs/crate/wgpu/27.0.1) and [its manifest](https://docs.rs/crate/wgpu/27.0.1/source/Cargo.toml.orig) distinguish browser WebGPU from native backends. Upstream [release notes](https://github.com/gfx-rs/wgpu/releases) also show later releases, including 30.0.1 and a browser failed-adapter handling fix. That is a reason to verify exact version behavior, not evidence that the Yetlet fixture exhibits that bug.

YT-075 must first select an exact compatible pin and feature set using versioned primary documentation and the existing Rust/wasm-bindgen lock constraints. Staying on 27 for an isolated proof or performing a bounded upgrade are both possible; **a 27-to-30 upgrade is not a mandatory M11 dependency**. Any upgrade, download or lockfile change needs the later implementation authority. Preserve the original measured source. Do not use a floating `latest` dependency or change all browser entries to accommodate GPU bindings silently.

## 3. Browser-first vertical slice

**Proposed fixture name, not an existing package:** `examples/webgpu-scene-island`.

Use a dense **work-record status/history overview** as the analytical job: identify clusters of state and select a record/time bucket for detail. The fixture may use deterministic synthetic datasets explicitly labeled synthetic; the real Board continues to use persisted records without depending on this renderer. Each rectangular mark has one defined record/bucket meaning. No decorative particles, meaningless 3D or promise to render HTML controls as pixels.

One page initially contains ordinary SSR heading, explanation, status summary, a paginated searchable data alternative and an inert canvas host. An explicit local activation control loads only this island's external hashed JS/WASM. The GPU surface is optional; essential records, selected detail and actions remain available without it. Do not create 100,000 hidden accessibility nodes to mirror 100,000 marks.

Use the [existing multi-island pattern](../examples/multi-island-dashboard/README.md): bounded manifest/schema, separate assets, owner-scoped activation, readable SSR on failure, independent sibling lifetimes. This is reuse of a demonstrated pattern, not a claim of a generic island API.

### Required initial behavior

| Condition | Required result in the future proof |
|---|---|
| No JS, absent/invalid manifest or missing WASM | Readable, navigable SSR summary/list; no broken required action |
| Insecure/unavailable WebGPU, null adapter or failed device request | Named unavailable/error state and the semantic alternative; no exception-induced page failure |
| Supported adapter | One actual colored scene displayed in the canvas with correctness assertions and fresh rendered evidence |
| Repeated activation while initializing | At most one current initialization and one owned surface; no duplicated listener/RAF loops |
| Disposal during asynchronous initialization | Retired result is discarded/destroyed; no late mount into an old or replacement owner |
| Sibling island active | Its DOM/state/listeners are unchanged; GPU failure cannot disable it |
| Navigation/remote intent | Pointer/filter changes stay local; a deliberate semantic remote command alone may invoke HTTP |

The [WebGPU specification](https://gpuweb.github.io/gpuweb/) defines secure-context APIs and fallible adapter/device acquisition. Capability detection and actual failure handling are required even where upstream browser support is advertised. Public browser-version availability is not a substitute for probing the exact approved environment.

Do not broaden `yetlet-dom` simply to mount this canvas: SSR can emit the host and the optional adapter can own it directly. The default SSR path must not link or initialize wgpu, pull GPU files, or become dependent on GPU readiness.

## 4. Lifecycle and resource contract

**P — state model:** inert → initializing → ready → suspended/ready; failures enter unavailable or lost; any state can become disposed. A later remount gets a new generation. Exact Rust types/signatures are intentionally not promised yet.

| Area | Bounded contract and future tests |
|---|---|
| Initialization | Asynchronous browser startup; no copying native blocking `pollster`/channel waits into the browser event loop. Own every pending completion with a generation/lifetime guard |
| Viewport | Separate CSS dimensions, physical canvas dimensions and scene coordinates; scale by bounded DPR; zero-size/detached hosts suspend rendering; clamp to device limits before allocation |
| Resize/DPR | Reconfigure target once for a coalesced latest size; replace old attachments safely; avoid reallocating every unchanged frame; test repeated resize and DPR changes |
| Frame scheduling | At most one scheduled animation callback per owner. Render on demand for static scenes; coalesce invalidations. Stop/pause offscreen/hidden work according to documented policy; cap resumed time deltas |
| Loss/errors | Distinguish unavailable capability, invalid input, device/surface lifecycle failures and resource exhaustion at the application boundary without pretending every native error exists identically in browsers |
| Recovery | No unbounded reinitialization loop. Retain CPU data/semantic UI, release old resources, report loss and allow a bounded explicit user retry. Successful recovery uses a new generation |
| Disposal | Invalidate lifetime before cancelling callbacks, removing owned listeners/observers and dropping resources. No drawing into retired canvas or completing old picks against a new scene |
| Ownership | Default one scene/device owner for the slice. Do not destroy an externally owned shared device; any future borrowed-device mode must define separate ownership explicitly |
| Teardown evidence | Owner callbacks/listeners/allocations return to the known baseline; browser/OS resource metrics are observations, not proof of immediate driver memory reclamation |

WGSL is a trusted build-time asset. Browser/driver shader and pipeline initialization is legitimate renderer startup work and measured separately; it is not Rust/WASM compilation in an HTTP request. No runtime user shader text is accepted. No unsafe-inline/eval CSP relaxation, remote script, privileged request context or server secret is introduced.

Malformed scene data must fail before issuing the affected GPU work: finite numeric coordinates/colors, explicit maximum item count, checked size arithmetic, valid indices/IDs, bounded dimensions and device feature/limit negotiation. No adapter diagnostic may leak credentials or raw untrusted error text into HTML.

## 5. Dynamic updates, correctness and fair comparison

**Proposed workloads:** retain the original static 10k/50k/100k fixture as historical comparison only. Add equivalent browser datasets at 1k and 10k for all feasible renderers, with dense 50k/100k lanes for Canvas2D/WebGPU. Fix viewport, coverage, colors, seed, state and operation order within each matched comparison. Report an omitted renderer at larger sizes as not measured, not a measured loser.

Primary comparison is **Canvas2D versus WebGPU** for the dense flat surface. DOM/SVG are additional smaller-scale baselines where semantically appropriate; they are not required to build an entire parallel framework. Never compare unrelated historical DOM list timings with Vulkan draw times.

| Workload | Measurement/acceptance intent |
|---|---|
| Cold activation | Network bytes actually transferred, raw/compressed artifact sizes separately, WASM initialization, adapter/pipeline setup and first usable frame |
| Static scene | Correct output and idle scheduling; no permanent redraw solely to report high FPS |
| Sparse updates | Update 1% of mark state, aggregate dirty ranges; measure CPU preparation, bytes uploaded, submission and event-to-usable-result latency |
| Full updates/replacement | Change all mark state and replace the scene; bound allocation and protect lifetime/generation |
| Burst input | Coalesce multiple changes into a latest render without unbounded queues or lost semantic final state |
| Resize/pan/zoom | Stable coordinate transform, bounded resource churn and accessible reset/zoom alternatives |
| Picking | Select known IDs with pointer/touch and semantic list controls; reject stale asynchronous results after replacement/disposal |
| Sustained use | Declared-duration frame-time distribution, CPU/RSS trend, explicit allocations and dropped/coalesced frame counts; low-power/device scope recorded |

A future report must separate CPU preparation, upload, encode/submit, asynchronous GPU completion, optional timestamp queries, RAF cadence and browser presentation. Submission completion is not paint. Correctness readback/screenshot capture must not silently contaminate timed frames. Startup includes costs excluded by the original native study. GPU timestamp support is optional; lack of it does not justify relabeling wall time as GPU time.

Use warm-up, alternating/randomized strategy order, repeated independent runs and declared sample counts. Proposed interactive target is a 60 Hz frame budget for the selected dynamic workload, **not an approved acceptance number or an achieved result**. Record p50/p95/p99 frame/event latency and startup cost; retain raw measurements and actual input/image evidence. Approve workload-specific limits before YT-077 execution. GPU need not win at every size: promotion requires a demonstrated useful envelope worth its bundle/maintenance cost.

Do not put the entire 100k mutable scene or GPU handles inside a clone-per-dispatch `LocalGraph` state by default. The existing graph is synchronous and `State: Clone`; whether to reuse it for small selection/control state or use a dedicated bounded dirty-range owner must be measured. This is an integration choice, not permission to change the graph into an async/global renderer scheduler.

## 6. Input, accessibility and fallback

The primary visual is one GPU canvas, with readable HTML labels, controls, selection detail and a paginated/searchable data view. A fallback renderer must not double-paint the scene underneath the active GPU; the semantic data alternative can remain intentionally available.

Pointer coordinates must map from client/CSS coordinates through DPR, viewport and scene transforms to a stable semantic ID. Define topmost/overlap behavior, cancellation, pointer capture, scrolling and touch behavior. Keep keyboard selection/search and reset/zoom buttons available without hover or drag. DOM text entry retains native Chinese IME rather than implementing an in-canvas text field.

Use a bounded CPU spatial/index lookup first for the flat batch if it meets the selected budget; GPU ID-buffer picking is a later measured alternative, not a prerequisite. Do not synchronously read back every frame for hover. Tooltips/detail render through safe text/Views; data labels are not HTML.

Respect reduced motion, pause offscreen/hidden rendering, reserve canvas dimensions and offer a lower-DPR/quality policy under explicit bounds. Mobile portrait prioritizes summary, filter, selection detail and a usable data list; touch/landscape/GPU support remain qualified only if selected and executed. Accessible fallback cannot be just the words "WebGPU unsupported" where important product data/actions are otherwise lost.

## 7. Extraction into `yetlet-wgpu`

**Owner amendment, 2026-10-14:** [Explicit promotion approval](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md) permits YT-079 as an **Experimental, opt-in** candidate without a speedup guarantee, after YT-078 and each independent review gate. The useful-envelope criterion above no longer blocks this experimental extraction; retain the YT-077 results and all correctness, lifecycle, isolation and target-limit requirements. Potential future 3D rendering does not add 3D implementation scope.

YT-079 may proceed only after YT-075–YT-078 evidence, independent review and explicit promotion authorization. The useful first crate is a small adapter owning initialization, resize/invalidation, flat batch rendering, errors and deterministic disposal. Keep backend-neutral application data separate from device resources.

The normal application surface should use narrow Yetlet-owned types. Do not expose wgpu types accidentally throughout core or claim that wrapping them guarantees effortless upstream upgrades. An advanced backend-coupled interop module may be considered separately with explicit version/lifetime expectations; it is not required by this first slice. Error categories and constructor names in planning are concepts, not compile-tested APIs.

Dependency and artifact acceptance:

- No dependency edge from core/HTML/default server/simple DOM into the renderer; default SSR does not acquire GPU startup or request cost.
- Only explicitly selected target/backend features are enabled. Browser and native backend graphs are inspected separately.
- Build browser entry packages independently. Capture current-source release asset baselines before later modifications and compare unrelated Todo/Search assets afterwards; do not substitute old historical digests.
- A lockfile/wasm-bindgen change affecting other bundles requires explicit review, or isolation in a bounded nested workspace if appropriate. No detached worktree is needed. Do not silently relax the byte-isolation gate.
- No renderer initialization merely from linking the crate; no GPU-device requirement in default CPU CI. CPU contract tests, browser build checks and real GPU execution are separate evidence classes.
- Keep `publish = false` for the candidate until separately approved release preparation. Do not silently expand YT-020's original nine-package inventory or reuse their signatures.

## 8. Qualification and delivery gates

| Task | Inputs → bounded output | Gate |
|---|---|---|
| YT-075 | YT-023 + reviewed M11 plan → exact dependency/feature decision and browser canvas island with real draw plus SSR fallback | Separate source/build/browser authority; one bounded owned attempt, current evidence and independent review |
| YT-076 | Browser slice → full lifecycle, async-retirement, resize/DPR/loss/recovery and cleanup contract | Negative/lifecycle tests; no unknown surviving process or render owner |
| YT-077 | Stable lifecycle → matched dynamic correctness/startup/frame/bundle comparison | Approved workloads/targets; no fabricated speedup or completion-to-FPS conversion |
| YT-078 | Measured renderer → stable selection/picking, semantic data alternative and keyboard/touch/reduced-motion behavior | Current rendered/accessibility evidence and security-safe text/URL sinks |
| YT-079 | Evidence above → explicit promote/defer decision and, only if approved, official opt-in Experimental crate | Independent API/dependency/security review; a defer outcome remains a research outcome, not a delivered crate |
| YT-080 | Selected candidate/targets → declared configuration/support limits and scoped regression record | Actual selected browser/GPU evidence, fallback, cleanup and repeat review; no publication/SLA claim |

**Browser-first target policy:** qualify at least one explicitly selected real browser/OS/adapter combination and unavailable/loss fallback before calling the browser candidate usable. Broad support claims need broader evidence; include an integrated/low-power GPU before claiming that class. Linux Vulkan remains the historical research lane. Windows D3D12 and macOS Metal are optional separately selected native support lanes, not automatic blockers to a browser-only first release. Safari/Firefox/mobile GPU support is not inferred from their DOM fallback; exact fallback and GPU states must be reported separately.

This narrows the preceding conversation's illustrative proposal: testing every native backend and upgrading to the latest wgpu are not mandatory just to establish a first official browser opt-in. The actual repository IDs are allocated by [M11 plan](M11-PLAN.md), superseding the earlier unregistered YT-069–YT-074 sketch.

**Stopping rule:** no numerical or API promotion from the existing Vulkan result alone; no rerun solely for redundant proof after a bounded acceptance succeeds. Failed/inconclusive GPU gates leave the product line unblocked and the fixture explicitly experimental. M11 closeout must preserve unmet gates rather than relabel a deferred crate as shipped.
