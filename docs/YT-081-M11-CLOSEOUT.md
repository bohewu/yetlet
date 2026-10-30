# YT-081 — M11 Selected-Scope Delivery Reconciliation

**Date:** 2026-10-14. **Source baseline:** original Ubuntu `master`,
`e499d2438d9cf5f5c6e1095d2fcff4b1428e15ef` (local, no push).
**Status:** **Done at selected M11 scope** after the earlier
[independent documentation review](YT-081-INDEPENDENT-REVIEW.md)
and a distinct [new-source Experimental GPU API reviewer PASS](YT-081-WGPU-API-INDEPENDENT-REVIEW.md), both with zero final required followups.
This is a *source-bound closeout*, not broad production certification,
release approval or unconditional GPU platform qualification.

## 1. Dependency/authority and exact change boundary

YT-081 depends on **YT-074 and YT-080**, both already Done at their
*selected* scopes. The owner separately authorized M11-P implementation,
the later YT-075–YT-080 GPU sequence with a distinct reviewer at each
task, and finally explicitly accepted an **Experimental opt-in**
`yetlet-wgpu` despite no demonstrated speedup. The two local reviewed
GPU deliveries were committed under distinct strict/builtin-guarded
actions: `8c52452` (YT-075–077) and `e499d24` (YT-078–080).
Neither was pushed or released.

The YT-079 authorization modifies **only** the originally proposed
"useful WebGPU speedup before experimental extraction" rule. It does not
grant Stable Candidate status, production/WebGPU-by-default, supported
3D APIs or YT-020 publication permission; see the
[explicit amendment](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md).

The YT-081 operation documents actual completion, updates source/API
classification navigation and reconciles backlog/roadmap/ADR stages.
It does **not** rerun heavy GUI/browser measurements, change any
renderer/Session/HTTP public signatures, alter host configuration,
start YT-020, create a worktree, commit or push. Previous receipts
and their failures remain immutable under their recorded input hashes.

## 2. M11-P — SSR-first product delivery

| Item(s) | Accepted selected evidence | Deliberate limit |
| --- | --- | --- |
| YT-069–072 | [M11 research/plan](M11-PLAN.md), [form and CSS verification](M11-P-FORM-ASSET-VERIFICATION.md), bounded safe form/HTML/property semantics, retained app-owned stylesheet strategy | Does not choose a general CSS/VDOM/form system or promote all prototypes |
| YT-073 | [Board implementation verification](M11-P-BOARD-VERIFICATION.md): SQLite-backed projects/history/version conflicts, authorization bounds, status/SSE and strict SSR/CSP, with native/browser regression | Noncritical synthetic-auth lab fixture, not university production identity |
| YT-074 | [Current qualification](YT-074-QUALIFICATION.md): real Chewing IME, repaired Firefox/Orca spoken AT path, Chrome/Firefox real browser journeys, owner-amended 62-minute five-cohort mix, **45,000 measured requests and zero HTTP/SSE errors**, independent review PASS | Original synchronized Ubuntu long run **FAIL** remains historical; mixed/staggered pass must not relabel it |
| YT-074 cross-environment | [Native Windows/Azure follow-up](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md): original synchronized full duration **PASS** on Windows (44,850 actual requests) and Azure Ubuntu (45,000), without HTTP/SSE errors | Original Ubuntu storage/fsync stall not repaired; successful environments do not establish HA, production RPO/RTO or universal host support |

**M11-P Done** means a selected complete SSR-first lab application and
selected operating envelope were proven, **not** production-safe
authentication, universal browser/assistive-tech conformance, backup/
restore SLA or hosted deployment approval. The operating-envelope
amendment is already owner-authorized and must remain clearly identified.

## 3. M11-G — separately opt-in Experimental GPU delivery

| Item | Accepted selected evidence | Deliberate limit |
| --- | --- | --- |
| [YT-075](YT-075-UBUNTU-BROWSER-VERIFICATION.md) | Real Rust/wgpu WebAssembly scene island on Firefox 158/X11/Intel Vulkan with a disposable experimental WebGPU pref; 120/120 Canvas centers, SSR fallback, independent reviewer PASS | Failed Linux Chrome adapter/texture paths and Windows standalone JS baseline are **not** candidate support |
| [YT-076](YT-076-VERIFICATION.md) | 11 real lifecycle checks; resized/recovered 120/120 images, controlled device loss/retry, bounds and late-owner retirement, independent reviewer PASS | Controlled loss is not spontaneous hardware recovery proof |
| [YT-077](YT-077-MEASUREMENT.md) | 24/24 matched Canvas2D/WebGPU lanes at 1K–100K, 2,160 measured update samples, 6 sustained lanes, 126 actual pixel proofs; independent [review PASS](YT-077-INDEPENDENT-REVIEW.md) | **No demonstrated WebGPU speedup.** Queue completion is not physical paint or FPS. The first 50K screenshot alignment failure was preserved |
| [YT-078](YT-078-VERIFICATION.md) | CPU picking, stable semantic IDs, stale generation/replacement rejection, safe labels, real mouse/keyboard/SSR, 54 bracketed actions, four 120/120 GPU captures, independent reviewer PASS | Touch/scroll/composition cases include simulations; physical mobile, screenreader speech and true OS reduced-motion selection not requalified |
| [YT-079](YT-079-VERIFICATION.md) | Owner-approved unpublished [Experimental first-party crate](../crates/yetlet-browser/guides/wgpu.md) with CPU default/explicit wasm32 browser feature, real extracted-consumer code path, 65 actions, 5×120 correct pixels, [independent API/security review PASS](YT-079-INDEPENDENT-REVIEW.md) | `publish = false`, no stable API, no speed/FPS claim; future 3D is motivation only |
| [YT-080](YT-080-VERIFICATION.md) | Verified final runtime/source correspondence, 20 source/API/dependency/package checks and separate [read-only reviewer PASS](YT-080-INDEPENDENT-REVIEW.md); distinct default Firefox-profile check proved usable semantic HTML fallback when `navigator.gpu` absent | Only disposable-opt-in Firefox/Ubuntu/X11/Intel configuration has positive **candidate** Wasm GPU proof; default profile = fallback PASS, not default GPU support |

The actual [GPU support matrix](../crates/yetlet-browser/guides/wgpu-support.md)
is authoritative. Default `yetlet-core`, HTML, ordinary DOM and Axum
server dependency graphs do not gain wgpu; a separate native benchmark
exists historically but does not certify this browser crate. The
candidate is a validated flat-rectangle GPU surface, **not** a game
engine, 3D scene graph or generic accelerated DOM.

## 4. API contract consistency: selected additional review

The maintainer requested a cross-crate API consistency audit with
YT-081. The [source-bound contract report](YT-081-API-CONTRACT-AUDIT.md)
covers all nine original unpublished candidate crates plus Redis and
Experimental GPU, examining construction/handler-first composition,
sync versus async, typed Results, error/lifetime/disposal contracts,
feature/default dependencies and documentation/stability promises.

**Current classification:** APIs are intentionally domain-specific,
with no demonstrated P0/P1 interoperability defect on exercised paths.
The original audit found Experimental GPU CPU string errors and
ambiguous logical `dispose` feedback. The owner explicitly authorized
a narrow [Experimental GPU API refinement](YT-081-WGPU-API-REFINEMENT.md):
CPU errors now have typed `SceneError` variants and `dispose`/`retire`
report first retirement as `bool`, while JavaScript remains an explicit
boundary. This is a new source change **after** the original YT-081
documentation Reviewer PASS and requires a fresh independent review.
The refined source has already passed six CPU tests, one doctest, twelve
consumer tests, native/wasm32 strict Clippy and the separately documented
[real Firefox/Wasm browser probe](YT-081-WGPU-API-REFINEMENT.md)
with five 120/120 GPU screenshots, typed error and disposal assertions
and verified owned cleanup. This does not retroactively change the
historical review's source identity or qualify other browsers.
Memory/Redis semantic parity and generic error/disposal abstractions
remain unselected. No shared `YetletError` or forced async Memory API.

New YT-081 local **offline, locked** regression observations on
Rust/Cargo 1.99 (logs under `target/m11-closeout/yt081-20261014/`):

- Root 34-member workspace: `cargo test --offline --locked --workspace`
  **PASS**, 250 passing assertions, zero failures.
- Isolated Experimental `yetlet-wgpu`: four CPU tests + one doctest
  **PASS**; real scene consumer: twelve tests **PASS**.
- Native server public API compatibility: two compile/behavior
  tests **PASS**.
- Redis provider `--lib`: invocation exit 0, **zero executed**,
  one Docker/Redis-fixture-dependent test ignored. Existing Redis
  three-application single-primary proof remains historical; no
  fresh Redis server integration was run for YT-081.
- Documentation and link/status checks, source-code API
  inventories and final review are distinct; passing unit tests
  do not prove public SemVer or browser compatibility.

## 5. Explicit unresolved work / stop rules

| Open boundary | Current disposition |
| --- | --- |
| **YT-020** — Private/public package release | **Blocked.** No YT-081 source/commit or M11 review refreshes signing provenance, builds all releasable artifacts, authorizes publication, makes a repository public, or expands the original nine |
| Other GPU/browser/OS combinations | Not qualified for extracted candidate. Linux Chrome/NVIDIA failures retained; Windows Chrome/Edge JS-only baseline is not Yetlet Wasm qualification; macOS/Safari/mobile/native Vulkan/D3D12/Metal unverified |
| Performance | YT-077 did not demonstrate a speedup. No FPS/perf SLA, timestamp GPU time, real VRAM accounting or general low-power claims |
| Experimental GPU API / 3D | No stable API, shader-injection interface, 3D API, game engine, device sharing or general scene-graph abstraction; future design follows evidence from another consumer |
| Accessibility/device coverage | No new physical mobile, spoken screenreader, true OS reduced-motion, WebKit, arbitrary IME/browser matrix or accessibility conformance certification |
| Board environment | Original workstation fsync path stall remains; HA, storage backup/recovery, IdP real-user deployment and business RPO/RTO belong to separate selected tests and operators |
| Cross-crate API maintenance | Owner explicitly authorized a narrow Experimental GPU typed-error/first-retirement API change; its new browser/compile/source-review evidence is separate. Other provider semantics and public stable API gates remain unchanged |

A previous historical claim or browser result must not be silently
re-labeled "Done at universal scope"; each existing task remains Done
**only at its previously reviewed selected scope**. Publishing or
deploying is a distinct authority decision, not the consequence
of completing YT-081.

## 6. Closure gate and next independent decision

**M11 status: Done at the individually recorded selected scopes**.
YT-069–YT-080 retain their separate task-review histories. YT-081
passed its initial documentation-only review after a repaired historical
Backlog P2, then the owner-authorized Experimental GPU typed-error/
dispose code refinement passed a **separate current-source read-only
Reviewer** with zero issues and required followups. See the
[original historical review](YT-081-INDEPENDENT-REVIEW.md),
[API refinement evidence](YT-081-WGPU-API-REFINEMENT.md), and
[new-source independent review](YT-081-WGPU-API-INDEPENDENT-REVIEW.md).
No new full browser or Redis server matrix was inferred beyond the
separately performed and documented selected Firefox probe.
No additional YT task or release work is implicitly activated by the
closed planning-to-experimental product roadmap.
