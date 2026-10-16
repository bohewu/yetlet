# YT-075 — Browser Scene Island Activation

**Date:** 2026-10-14. **State:** Selected Ubuntu Firefox 158/X11 experimental-opt-in browser **PASS** and fresh independent read-only reviewer **PASS**, YT-075 Done at the selected scope. The actual Rust/wgpu Wasm drew 120/120 pixels; explicit activation, async stale-owner destruction, sibling isolation, unavailable-device/asset fallback and cleanup passed. See [current selected-browser evidence](YT-075-UBUNTU-BROWSER-VERIFICATION.md) and [verification](YT-075-VERIFICATION.md). The earlier seven Ubuntu Chrome failures and independent reviewer FAIL are retained as history, with all broader GPU/Windows/Chrome capabilities unclaimed.

## Current qualified browser gate (superseding the initial blocked state)

The owner redirected work to the original Ubuntu checkout. Its
application-owned YT-075 Rust/wgpu Wasm bundle was run in a disposable
**Firefox 158 Stable, X11 hardware** session with an opt-in
`dom.webgpu.enabled=true` preference confined to that temporary profile.
The selected real Intel integrated Vulkan GPU rendered all 120 rectangles
with correct colors; the background, SSR, absent/null/rejected devices,
missing/invalid assets, independent counter, late device destruction,
replacement owner and teardown passed. Original evidence and earlier
failed browser attempts are retained and separately identifiable.
[Selected qualification](YT-075-UBUNTU-BROWSER-VERIFICATION.md)
is **PASS for this browser configuration**. A [fresh independent reviewer](YT-075-INDEPENDENT-REVIEW.md)
returned **PASS**, zero issues and zero required followups; YT-075 is
**Done at this experimental opt-in scope**, not broadly supported.
No subsequent M11-G item was started and no commit/push occurred.

## Owner authority and baseline

After the completed YT-074 return inspection, the owner explicitly requested starting YT-075. This activates only the browser WebGPU scene-island proof defined by [the M11 plan](M11-PLAN.md) and [the GPU plan](M11-WGPU-PLAN.md). It does not activate YT-076 through YT-080, promote a public renderer crate, or change the release inventory.

The existing Ubuntu WebCodex project is `agent:bohe-ubuntu-86799946b1954915:yetlet`, bound to the original Ubuntu Yetlet checkout; resolve its private root through the authorized project inventory. The resumed Session is `wc_sess_7cOtmezQB-SJfbJt`. Its initial observation was clean `master` at `76be7aa1107e08e3c7850abc8c68f53301cf0f7c`, four commits ahead of the existing `origin/master` tracking ref; no fetch was performed.
Use this same checkout. Preserve all YT-074 results and ignored evidence. Do not repeat the completed load, file-probe, browser/IME/AT, Windows recovery or Azure work. No commit, push, new worktree, host setting change, tool installation, cloud operation, publication or release action is authorized by this activation.

## Subsequent conditional continuation authority

The owner subsequently permitted reviewer-role review or a local Codex reviewer
and continuing items dependent on YT-075 when it is feasible, reviewing each item
before proceeding. This is conditional sequential scope, not a waiver of drawing,
lifecycle, performance, accessibility or promotion gates. No dependent item has
started; the newly completed YT-075 review opens YT-076 as the next
independently reviewed slice, without automatically starting it.
No commit/push, publication, host modification or unrelated external
environment operation follows.

**Historical pre-Firefox checkpoint (superseded by the selected browser PASS):**

The generalist implementation/build Job is terminal exit 0. The parent's three
Chrome attempts are terminal exit 1 with owned resources cleaned; the final
X11 attempt still returned a null adapter. The previous NVIDIA/Vulkan attempt
reached device initialization but failed real pixels with texture-import errors.
The parent added scoped validation-error propagation; its browser regression
remains unexercised. Stop automatic execution at this boundary and retain all
failure records. The remaining sections preserve historical stages, not renewed
instructions to repeat their commands.

The independent read-only reviewer completed normally and returned FAIL. The
parent corrected its concrete English-language metadata finding using the
existing API and a routed-response test; no further browser was launched.
Required real drawing and the unexecuted browser failure/lifetime/sibling checks
remain open. The language fix is not independently re-reviewed. Resume from
[the current verification and reviewer disposition](YT-075-VERIFICATION.md),
not from the historical implementation commands below.

## Selected proof boundary

The fixture is now [examples/webgpu-scene-island](../examples/webgpu-scene-island/README.md), application-owned rather than a `crates/yetlet-wgpu` extraction. It was created during the admitted implementation continuation below.

Use the planned deterministic synthetic work-record status overview. Keep a meaningful SSR summary and searchable, paginated data alternative. Only an explicit local activation loads the scene's independently hashed external JS/WASM. Ordinary SSR, core/HTML/DOM dependencies, unrelated Todo/Search assets and the persisted Board must remain independent of GPU readiness.

Required YT-075 evidence remains the existing contract: an exact compatible dependency/feature decision, real browser canvas drawing, unavailable/null-adapter/device-request failure fallback, single initialization ownership, retirement during asynchronous initialization, sibling isolation and readable no-JS/missing-asset SSR. A declared bounded owned browser attempt requires capability preflight, fresh target selection, retained evidence and cleanup. Native Vulkan results do not replace browser evidence. Full resize/DPR/loss-recovery qualification, performance comparisons and picking promotion belong to later tasks.

The historical research manifest pins `wgpu = 27.0.1` with optional native `std`, `vulkan` and `wgsl` features. The existing counter-island manifest pins `wasm-bindgen = 0.2.129`. Versioned [wgpu 27.0.1 documentation](https://docs.rs/crate/wgpu/27.0.1) and [its manifest](https://docs.rs/crate/wgpu/27.0.1/source/Cargo.toml.orig) distinguish browser `webgpu` from native backends. At the blocked attempt these were selection inputs only; no dependency was changed or acquired. The admitted continuation verified the cached exact source and built the isolated browser pin/feature combination recorded in [verification](YT-075-VERIFICATION.md).

## Historical inspection and execution blocker

Repository instructions, the current M11/GPU plans, proposed ADR 0008, root Cargo manifest, existing multi-island source and browser-build/smoke patterns were inspected. The repository's saved profile metadata reports `premium`, `openai@5`; it is not a completed profile-health verification or runtime model observation.

Two distinct execution-preflight invocations were rejected by the ChatGPT tool safety layer before returning an execution result:

1. A read-only profile-status/policy inspection invocation. No successful profile-manager result was obtained.
2. A separate read-only toolchain/dependency/asset-baseline inspection invocation. No tool versions, cached-package inventory or computed asset baseline was obtained from it.

Both returned an inability to determine the request's safety. Neither returned a process or Job identifier. No agent dispatch, compiler, product test, browser launch or GPU workload was started by this attempt. Execution-type operations were stopped after the repeated rejection; the blocked operations were not retried through another command, path or dispatch surface. Ordinary file inspection and this documentation update are separate admitted operations.

A focused Runner status observation between the two rejected invocations reported connected/ready and zero active, running, queued, recovering or lost Jobs. The rejection is not evidence that the Runner is offline, that an installed executable is absent, or that WebGPU is unsupported. No hardware or product defect is established. Saved role names/models are not runtime telemetry, and no independent reviewer result exists.

## Historical blocked delivery boundary

The blocked attempt changed only this activation record, [backlog](BACKLOG.md) and [roadmap](ROADMAP.md), delivering no application implementation. At that boundary YT-075 was Blocked, not Done. All subsequent GPU items remained Todo/unactivated. YT-074 retained its accepted scope, YT-081 retained its YT-080 dependency, and YT-020 remained Blocked.

Only direct document inspection was available for that blocked update. The documentation checker, Cargo checks, browser checks, GPU checks and independent review had not run for that delta. Earlier validation was not transferred to those bytes. No staging, commit, fetch or push was part of that delivery.

## Historical admitted implementation continuation

The owner subsequently said "please start" after successful execution preflight. The parent reports profile health `ok`, catalog `current`, eligibility `eligible`, trusted/current configuration and premium tier, with `run_process`, `run_shell` and `run_script` successful and no active prior Job. This is supplied preflight evidence, not a child runtime-model or native managed-role claim. The earlier two tool-safety rejections above are retained as history.

The continuation used the same checkout at `76be7aa1107e08e3c7850abc8c68f53301cf0f7c`. It created only the application fixture, a root workspace exclusion and focused documentation. Cached dependencies were resolved offline into a nested lockfile; the root lockfile and unrelated browser asset bytes remain unchanged. CPU tests, native/WASM builds, package lint and documentation checks are recorded in [YT-075 verification](YT-075-VERIFICATION.md). No browser, GPU workload, installation, host change, cloud/Windows operation, staging, commit, fetch or push was performed.

YT-075 is In progress, not Done. The parent owns one inspected bounded browser acceptance attempt and independent read-only review. This leaf implementation assignment neither ran those gates nor started a dependent item. YT-074 evidence and YT-081/YT-020 gates remain intact.

## Historical implementation handoff boundary

Preserve the current uncommitted implementation and documentation, including failed first-build logs. The original execution restriction is resolved by the supplied parent preflight; no bypass is authorized. Complete the applicable policy/role preflight before any further role dispatch. A role-directed Codex Job is not native managed dispatch.

Inspect the product-specific browser check, run one bounded owned attempt only with the required environment capability, retain actual canvas/adapter/fallback/retirement/sibling evidence and cleanup, then obtain independent read-only review. Do not mark Done from builds, source inspection, self-review or the historical native GPU benchmark. Stop at the selected acceptance result; do not proceed to YT-076 within this assignment or commit/push.
