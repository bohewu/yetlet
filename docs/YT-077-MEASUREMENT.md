# YT-077 — Selected Firefox Dynamic Canvas2D / WebGPU Measurement

**Date:** 2026-10-14. **Selected browser measurement:** PASS (24 of 24
declared lanes, no omissions). **Overall YT-077: Done** after fresh
[independent reviewer PASS](YT-077-INDEPENDENT-REVIEW.md) with zero
issues or required followups. The performance observations below are
actual measurements, not a speedup claim, an SLA, GPU timing queries,
OS/VRAM measurements, or generally supported browser configurations.

## Frozen workload and source

The source is the same original Ubuntu checkout on `master`,
`76be7aa1107e08e3c7850abc8c68f53301cf0f7c`, retaining prior
uncommitted YT-075/YT-076 work. The selected browser is Firefox 158
headful X11 with a disposable `dom.webgpu.enabled=true` profile and
actual non-fallback Intel Vulkan WebGPU. The matched Canvas2D and Rust
wgpu 27 WebGPU pipelines load only via the dedicated comparison route.
The original strict-CSP semantic SSR, ordinary `/scene`, independently
hashed scene/counter assets, lockfiles and unrelated Todo/Search files
were not changed by the run.

[Prepared methodology and implementation](YT-077-VERIFICATION.md)
declared the full input before measurement:

- Sizes: 1,000, 10,000, 50,000 and 100,000 synthetic status records,
  bounded at 32 bytes per record, identical generator, seed 7, colors
  and operation sequence on both renderers.
- Three fresh document/owner rounds at each size and each renderer,
  alternating renderer order across rounds.
- Sparse (first 1%), full (100%) and burst (eight sparse operations
  coalesced to one final demand draw) workloads.
- Five warmups plus 30 observed samples per lane/workload. Six
  10-second sustained sparse lanes at 10,000 records (one per
  renderer and round), each under backpressure.
- Actual 600 × 400 CSS/backing viewport and DPR 1. Initial, sparse,
  full, burst, sustained and seed-9 replacement snapshots sampled
  against known canonical RGB, not compared by assuming PNG bytes
  coincide. Predeclared channel tolerance remains 25.
- Separate wall observations for preparation, bridge/apply, upload,
  encode/submit, queue completion and RAF opportunities; bundle raw
  and deterministic-gzip sizes. No GPU timestamp queries and no
  conversion from queue completion or RAF observations to FPS.

## Browser receipts and actual completeness

A first selected attempt at
`target/m11-gpu/yt077-20261014/browser/` finished with **FAIL**
after 57.33 seconds: an element-screenshot alignment error was
detected in a 50,000-record Canvas2D correctness assertion. That
attempt retained its partial 1,000/10,000 lanes and actual failure
and cleaned all owned resources; it is not silently converted into a
performance sample. The subsequent bounded harness correction mapped
canvas coordinates to the screenshot only when **the complete top
capture row is white**; it did not change matched data, color
expectations or `PIXEL_TOLERANCE = 25`.

The final single bounded rerun:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_dynamic.py
```

retains its separate ignored results at
`target/m11-gpu/yt077-20261014/browser/capture-alignment-fix/`.
`result.json` SHA-256:
`c252ed364caaffd542f896c1ce497bb889e2aa15c0f74853d41822fe610ed304`.

Actual runtime results:

| Evidence | Observed |
| --- | --- |
| Native browser test exit | 0, `passed_yt077_measurement` |
| Declared matched lanes | **24/24 completed**, 0 omitted |
| Workload groups | 72, all with 30 measured + 5 warmup samples |
| Measured workload samples | **2,160** |
| Warmup samples | **360**, excluded from distributions |
| Sustained workload samples | **906** across 6 actual timed lanes |
| Real screenshot/known-pixel proof events | **126**, zero bounded RGB mismatches or bad black-interior checks |
| Active GPU record buffer ceiling | 3,200,000 bytes; observed transition peaks within bounds |
| Per-lane cleanup | PASS, zero retained application record bytes and no RAF pending |
| Source/binary/asset hashes after test | Exact match to collected pre-run SHA-256 values |
| Browser process elapsed | 275.30 seconds, below 480-second bound |
| Test/owned cleanup | WebDriver deletion, Firefox exit, owned server and driver reaped, profile removed, both loopback ports closed |

This fixture specifically tests Canvas2D and actual Rust/wgpu/WebAssembly,
not JavaScript pretending to be WebGPU. The receipt retains workload order,
warmups, every raw measured/sustained sample, per-lane resources,
application upload/reallocation counters, asset transfer observations and
real screenshots. The 126 actual snapshots are correctness evidence,
**excluded from individual measured update samples**.

## Selected completion-wall observations

The following entries are the **median across three rounds of each
round's p95 `completionWallMs`**, in milliseconds. They describe
this harness's update-to-observed-result wall time, *not* actual
GPU execution time, displayed FPS, browser-paint completion or a
cross-API-equivalent queue-completion guarantee.

| Records | Canvas2D sparse/full/burst (ms) | WebGPU sparse/full/burst (ms) |
| --- | --- | --- |
| 1,000 | 18 / 18 / 17 | 93 / 99 / 95 |
| 10,000 | 17 / 16 / 17 | 97 / 88 / 91 |
| 50,000 | 14 / 14 / 12 | 86 / 89 / 88 |
| 100,000 | 15 / 16 / 18 | 73 / 72 / 75 |

Canvas2D's completion-wait field is absent, not a fabricated zero
GPU cost; WebGPU's queue work completion is awaited. These
different API completion semantics and Firefox/Linux/Intel adapter
preclude presenting this wall comparison as direct GPU execution
duration. At this selected envelope, **no useful WebGPU speedup is
demonstrated**. The 16.67 ms / 60 Hz reference was declared as an
observation only and was not an approved pass/fail SLA. No winner
claim is inferred from omitted renderer/record counts (there are no
omitted lanes here), and no change is made to the independent
experimental renderer's approval requirements.

## Runtime limits and preserved history

The successful test used the exact current release scene digest
`6ea5f2e01d31af77` (Rust scene Wasm 281,908 raw / 76,077
deterministic gzip bytes), independently hashed counter digest
`81c388455a88d98a`, and application-owned `comparison-loader.js`
(9,794 raw / 3,326 deterministic gzip bytes). Gzip values are
artifact measurements, not delivered compression guarantees.
`ResourceTiming.transferSize=0` is cached/unobserved, not an
assertion of free network startup.

The selected Android/mobile, macOS, Windows browser-WASM, NVIDIA,
Linux Chrome, default Firefox Stable and native Vulkan/D3D12/Metal
backends remain **unqualified** here. GPU memory, OS resident memory,
driver allocations and physical paint are **not observed**. Picking,
keyboard/touch selection and accessibility fall under YT-078.
The performance record does not approve promotion of a public GPU
API. The YT-079 proposal explicitly requires useful-enough envelope,
an independent API/security review and separate explicit approval;
those gates cannot be silently waived by the user's implementation
sequence authorization.

The YT-077 **measurement** gate is PASS and the separate
[independent reviewer](YT-077-INDEPENDENT-REVIEW.md) verified the
frozen method, real samples, 126 screenshots, resource bounds,
source isolation and cleanup. YT-077 is **Done at this selected scope**.
No commit, push, release,
detached worktree or host setting change occurred.
