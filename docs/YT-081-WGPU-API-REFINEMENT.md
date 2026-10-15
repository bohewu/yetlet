# YT-081 — Experimental yetlet-wgpu API Refinement

**Date:** 2026-10-15. **Authority:** owner specifically requested fewer
Experimental API debts *before* accepting the M11 closeout. This is
an additive, scoped follow-up to [cross-crate audit](YT-081-API-CONTRACT-AUDIT.md),
not a Stable Candidate or publication decision. Original M11 GPU
qualification remains bound to its historical exact source; this
follow-up source and its own verification/review must be considered
separately.

## Change contract

| Concern | Before | New Experimental candidate |
| --- | --- | --- |
| CPU validation/batch/picking failures | Raw `Result<_, &'static str>` | `Result<_, SceneError>` with six non-exhaustive typed variants, `Debug`/`Eq`, `Display` and `std::error::Error` |
| Exact bounded diagnostic text | Raw string | Preserved by `SceneError::as_str()` / `Display`; do not parse text to select an error |
| JS/Wasm browser result | `Result<_, JsValue>` | Remains `Result<_, JsValue>`; explicit `cpu_error_js(SceneError)` translates CPU errors at this boundary, while device/loss/validation errors remain separate |
| CPU owner lifecycle | `Lifetime::retire() -> ()` | `Lifetime::retire() -> bool`, true only on first logical retirement, false on repeats; also works after a failed state |
| Browser scene lifecycle | `SceneOwner::dispose() -> ()` | `SceneOwner::dispose() -> bool`, returns true only on first owner retirement. The real renderer and canvas are released on retirement; Drop remains idempotent |
| Consumer Wasm wrapper | Unit-returning app `SceneOwner::dispose()` | Bool-returning wrapper delegates to the candidate; existing JS caller can ignore it, but must still wait for borrowed async futures to settle before `.free()` |

The typed variants are `InvalidRecordLength`,
`InvalidRecordData`, `InvalidViewport`, `InvalidPointer`,
`InvalidUpdateRange`, and `VersionExhausted`. Their messages
remain exactly the previous bounded strings. The enum is marked
`#[non_exhaustive]` to permit future Experimental cases.
Invalid replacement/update and version overflow still fail
**before mutation**. This API change is intentional because the
crate is unpublished, explicitly Experimental and has its real
consumer under the same current checkout; it must not be
misrepresented as an external SemVer-safe patch.

A `true` result from `dispose` means only *first logical
retirement*. It **does not** mean queue work has completed, borrowed
Rust/Wasm futures have released their wrapper, or external GPU memory
has been measured. The existing app-owned loader keeps the wrapper
until initialization/completion futures settle, then frees it
exactly once. The browser error/fallback categories, generation
checking, separate semantic HTML, default-CPU-only features and
picking contracts remain unchanged. No new global shared
`YetletError` or type-erasing trait is introduced.

## Current-source validation

The following actual new-source checks passed before the browser run:

- Isolated CPU crate: **six tests**, including exact
  error variant/message mapping, invalid replacement atomicity,
  first/repeated retirement and failure-to-disposal transitions;
  **one doctest**.
- Real scene consumer: **twelve native tests**.
- Explicit wasm32 `browser` feature and actual consumer wasm32
  `cargo check --offline --locked` both **PASS**.

Native and explicit wasm32 `browser` strict Clippy passed with
`-D warnings` for both crate and consumer; both formatting checks
and the Python harness syntax check also passed.

A separate **current-source Firefox 158/X11/Intel Vulkan** test **PASS**
used the real rebuilt Rust/wgpu WebAssembly, not a JavaScript shader
recreation. It was executed once:

```sh
YT075_FIREFOX_HEADFUL_X11=1 YT081_WGPU_API_REFINEMENT=1 \
  /usr/bin/python3 examples/webgpu-scene-island/verify_candidate.py
```

The new isolated receipt is
`target/m11-closeout/yt081-20261015/browser/api-refinement/result.json`,
SHA-256
`aef12ca8f882140064e3d3f1632791921507c29ec8457cfd7c7afe699a3fc96d`.
Its task ID is `YT-081`; the reused candidate harness retains the original
success status string `passed_yt079_candidate` for this new separate run.
It has **65 executed actions, five independently captured 120/120 pixel
proofs** with black interiors and no mismatches, and **34 unchanged inputs**.
Actual Wasm `dispose` values were inert `[true,false]` and two configured
owners `[true,true]`. Retiring an owner during a real pending queue
completion destroyed one old device, one buffer and unconfigured one
surface; its Wasm wrapper was freed only after the future settled.
Controlled loss/recovery, picking, SSR fallback and the independent counter
also passed. Session/driver/server processes, temporary Firefox profile,
and both loopback ports were cleaned.

The new scene digest is `f5584f407ee4f7b2` (288,758 Wasm bytes);
independent counter digest remains `81c388455a88d98a`. The release
manifest SHA-256 is
`3f0725a1ef8d1d60d79207a392a96044673b5d7735b4facd59167df0038ad8bb`.
The historical [YT-079 receipt](YT-079-VERIFICATION.md) was preserved;
its earlier source identity is not substituted for this new result.

## Scope and non-goals

Memory Session remains synchronous; Redis Session remains async
with CAS/uncertain remote outcomes. DOM event disposal can fail
and continues to return its own domain `Result<bool, JsValue>`;
other owner types keep their currently documented disposal
contracts. Public stable API compatibility fixtures, browser
cross-platform qualification, runtime device-sharing,
3D engine/scene graph, fast-renderer/SLA claims and YT-020
publication are **outside this refinement**.

A [fresh independent read-only reviewer](YT-081-WGPU-API-INDEPENDENT-REVIEW.md)
checked this **new source** and its actual rebuilt Wasm/browser receipt,
consumer migration, typed error variants and idempotent retirement:
**PASS, zero issues and zero required followups**. The previous
[documentation-only YT-081 review](YT-081-INDEPENDENT-REVIEW.md)
remains historical; this source-aware review is the distinct
acceptance gate for the refined Experimental API. YT-081 is now
Done at the selected M11 closeout scope, with no new release approval.
