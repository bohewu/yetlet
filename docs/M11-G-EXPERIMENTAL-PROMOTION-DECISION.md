# M11-G — Owner Decision: Experimental Opt-In Renderer

**Date:** 2026-10-14. **Authority:** explicit owner decision, not a reviewer inference.
**Implementation:** YT-078–YT-080 remain subject to their individual evidence and independent Reviewer PASS gates.

## Authorized next sequence

The owner requested one local commit of the completed YT-075–YT-077 work after strict/builtin Commit Guard, followed by YT-078, YT-079 and YT-080. Each item must receive an independent reviewer PASS before the next starts. No push, publication, release or default dependency change was authorized. The initial guarded commit is `8c524520256e5fd9884d4d023dbdfbd76181c561`; its source tree matched the scanned index and the checkout was clean afterwards.

## YT-079 promotion decision

The owner explicitly approves a first-party **Experimental, opt-in** `yetlet-wgpu` module/crate despite the absence of a measured WebGPU speedup in YT-077. This replaces the earlier useful-speedup promotion prerequisite for this experimental candidate only. It does not invalidate or hide the measured comparison or promise a faster renderer.

The module README, crate-level/API documentation and support guide must state Experimental status, explicit activation and target limitations. There is **no performance, stable-API, production-readiness, FPS or broad browser-support guarantee**. Keep `publish = false`; core, HTML, ordinary DOM and the default server must not acquire GPU initialization, dependencies or asset cost.

Possible future 3D rendering is a motivation for preserving an optional extension path, not a request or acceptance claim for a 3D engine, scene graph, native backend expansion or 3D API in this delivery. Implement the currently proven flat-scene renderer without speculative abstractions.

## Unchanged gates

YT-078 must prove stable picking and semantic alternatives, including selected keyboard/touch/reduced-motion behavior. YT-079 must extract a usable bounded opt-in candidate, adapt the existing real consumer, preserve lifecycle/data/resource isolation and pass independent API/dependency/security review. YT-080 must qualify the declared candidate configuration with actual current-source browser evidence, fallback and teardown. A supported selected experimental configuration is sufficient; unverified platforms remain explicitly unverified.

The existing [YT-077 measurements](YT-077-MEASUREMENT.md), original failed Chrome configurations and [M11 GPU plan](M11-WGPU-PLAN.md) remain historical/technical evidence. YT-020's release restrictions and YT-081's dependency on YT-080 are unchanged. No task is Done merely because the owner has approved its scope.
