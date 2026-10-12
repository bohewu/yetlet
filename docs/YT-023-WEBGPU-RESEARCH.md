# YT-023 — Independent optional scene renderer research

## Scope and decision

**2026-10-12 / T06:** retain this **application-owned, explicitly enabled native
scene prototype** as bounded research; **reject promotion to a supported core
renderer or browser-portable API**. Actual native rendering and readable results
at 10k–100k instances justify retaining the experiment. They do not establish an
improvement over DOM, a CPU renderer or another graphics library: no such matched
baseline was measured. No numerical performance threshold was approved.

The [unpublished example](../examples/webgpu-scene-research/README.md) implements
one flat rectangle pipeline with a mature WebGPU-based graphics implementation,
[wgpu 27.0.1](https://docs.rs/wgpu/27.0.1/wgpu/). Its actual backend is **native
Vulkan**, not browser WebGPU. wgpu's versioned documentation distinguishes these
backends; browser support in the upstream library is not evidence of browser
execution by this example. The manifest enables only std, Vulkan and WGSL,
behind the optional `gpu` feature. The package has no Yetlet dependencies;
`cargo tree` without that feature contains only the example itself.

Exactly nine library candidates and 12 user-facing catalog samples remain. The
independent research workspace member has `publish = false`; its binary requires
`gpu`, and default native tests do not request a graphics device. No required CI
GPU-device gate is added. Core, HTML/DOM, browser loader, action transport and
default no-JS/no-WASM SSR are unchanged. [ADR 0003](adr/0003-renderers-and-loading.md)
already separates scene rendering from HTML/DOM, so this experiment does not
change a fundamental architectural decision. It creates no global renderer,
graphics framework, scene hierarchy or DOM/CSS acceleration path.

## Workload and actual rendering proof

[Scene input and tests](../examples/webgpu-scene-research/src/lib.rs),
[WGSL shader](../examples/webgpu-scene-research/src/scene.wgsl) and the
[direct measurement entry](../examples/webgpu-scene-research/src/bin/scene-bench.rs)
are deliberately single-purpose. Workloads are **10,000, 50,000 and 100,000**
instances. All render to the same **1024×1024, single-sample, single-mip,
RGBA8Unorm** offscreen texture, with no depth target or blending. One draw uses
six vertex invocations per instance to form two triangles. Position/extent and
opaque color occupy two Float32x4 attributes, **32 bytes per instance**.

Input is deterministic, without an RNG: columns are ceil(sqrt(node count)), rows
are ceil(count/columns), positions occupy a row-major grid, and integer pixel
centers are converted to normalized coordinates. Half extents are 0.7/columns
and 0.7/rows in normalized coordinates. Colors cycle opaque red, green and blue.
The benchmark renders a static scene repeatedly; it does not measure scene
mutation, per-frame uploads, culling, text, textures, depth, picking or hierarchy.

Before timing each workload, an actual draw is copied to a mapped readback buffer.
Every instance center must equal its assigned RGBA color, and the unused
bottom-right pixel must be opaque black. CPU tests establish distinct in-bounds
centers and exact buffer sizes at all three sizes, and show that wrong-length,
clear-only and corrupted-center images fail the proof. The GPU command passed:

| Instances | GPU-readback centers checked | Nonblack pixels | RGBA FNV-1a fingerprint |
|---:|---:|---:|---|
| 10,000 | 10,000 | 490,000 | `2c38d159746971ab` |
| 50,000 | 50,000 | 450,000 | `b4c8f76aa6f48b7b` |
| 100,000 | 100,000 | 900,000 | `011024551c1c9e1b` |

These checks demonstrate the declared colored instance workload rather than
queue submission or clearing alone. They are not exhaustive rasterization,
visual-quality or accessibility qualification. FNV-1a is only an output
fingerprint, not cryptographic integrity. [Retained proof](../examples/webgpu-scene-research/render-proof.txt)
includes the actual adapter and cleanup record.

## Environment and frame observations

Final run: **2026-10-12 09:06 UTC**, native release optimization, Rust
`1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, Linux
`7.0.0-38-generic`, x86_64, Intel Core i5-13600K. AdapterInfo reports NVIDIA
GeForce RTX 4090, DiscreteGpu, vendor 4318, device 9860, NVIDIA driver 595.99.02,
**Vulkan** backend. Driver/tool configuration, display and CPU/GPU governors were
not changed. Preflight NVIDIA utilization was 0%; this is one local process
without affinity, sustained thermal control or independent repeated-run intervals.

Locked graphics versions are wgpu 27.0.1, wgpu-core 27.0.3, wgpu-hal 27.0.4,
wgpu-types 27.0.1, naga 27.0.3 and pollster 0.4.0. This pinned prototype version
is not advertised as latest. The local source/API and upstream versioned docs
were inspected rather than assuming interfaces from another wgpu version.

Each workload gets **five discarded warmups and 30 retained frames**, ordered
10k, 50k, 100k. `Instant` begins immediately before command encoder creation and
ends after `queue.submit` plus `device.poll(Wait)` for that exact submission.
The wait has a 10-second timeout; the entire command is bounded by 60 seconds.
Each frame completes before the next starts. This measures **CPU encoding +
submission + GPU completion wall time**, including host/driver overhead. It is
not GPU timestamp duration, concurrent throughput, compositor timing or display
FPS. Scene construction, buffer upload, shader/pipeline setup, readback proof,
CSV printing and allocation are outside timing. No GPU timestamp query feature
is required or enabled.

| Instances | Samples | Median ms | Minimum ms | p95 ms | Maximum ms |
|---:|---:|---:|---:|---:|---:|
| 10,000 | 30 | 0.033760 | 0.033236 | 0.071845 | 0.076618 |
| 50,000 | 30 | 0.054286 | 0.053588 | 0.057174 | 0.059741 |
| 100,000 | 30 | 0.083525 | 0.083239 | 0.085052 | 0.086160 |

The p95 is nearest-rank sample 29 of the 30 sorted observations. Median is the
mean of sorted samples 15 and 16. [Raw CSV](../examples/webgpu-scene-research/measurements.csv)
retains all 90 integer nanosecond observations, sizes, warmups and sample IDs;
rounding occurs only in this table. These very short, cache-warm offscreen draws
on a high-end GPU are not forecasts for realistic application frames. Increasing
instance count also changes rectangle area/coverage; this is a declared scene
scaling study, not a controlled isolation of vertex versus fragment cost.

## Memory observations and ownership

| Instances | Instance buffer bytes | Logical color texture bytes | Readback buffer bytes | Total explicit bytes |
|---:|---:|---:|---:|---:|
| 10,000 | 320,000 | 4,194,304 | 4,194,304 | 8,708,608 |
| 50,000 | 1,600,000 | 4,194,304 | 4,194,304 | 9,988,608 |
| 100,000 | 3,200,000 | 4,194,304 | 4,194,304 | 11,588,608 |

Buffer numbers are requested byte lengths; texture bytes are width×height×4
logical storage. Totals include the retained proof readback buffer even during
timed frames. They exclude driver/allocator alignment and metadata, command
buffers, pipeline/device resources, transient upload staging and CPU scene/data
vectors. They are **not measured VRAM, resident allocation or process RSS**.
The readback's 4096-byte row stride meets the copy alignment with no row padding.

Each workload unmaps readback and destroys its instance buffer, readback buffer
and texture before advancing. Draw submissions have completed before destruction.
Pipeline/shader handles, device, queue, adapter and instance are released before
success exit. The command completed with its cleanup line; post-run
`nvidia-smi --query-compute-apps` showed no compute process. No listener, browser,
watcher, daemon or background graphics process was created.

## Reproduction, source identity and verification

Executed final focused commands from the repository root:

```bash
cargo test -p yetlet-webgpu-scene-research --offline
cargo build --release -p yetlet-webgpu-scene-research --features gpu --offline
timeout 60s target/release/scene-bench
cargo fmt -p yetlet-webgpu-scene-research -- --check
cargo clippy -p yetlet-webgpu-scene-research --features gpu --all-targets --offline -- -D warnings
python3 scripts/check_docs.py
python3 scripts/check_samples.py
git diff --check
```

Native CPU tests: **2 passed, 0 failed, 0 ignored, 0 filtered out**; zero doctests.
Release GPU build, actual draw/readback, formatting and strict Clippy pass.
Documentation/reference/backlog and sample checks pass. Dependencies were fetched
once before offline commands. Raw results, exact command exits and source hashes
are also retained under `.pipeline-output/delivery-20261012-docs-research/outputs/T06-*`.
The [source identity record](../examples/webgpu-scene-research/source-identity.json)
pins SHA-256 of measured source, manifest and lockfile in the dirty delivery
checkout based on HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280`, after T01–T05.
It does not transfer evidence to historical signed source
`8e557e760407a3d4b3f5a8b8f833748946be1387`, historical `c388` evidence or a release.

One initial proof-fixture assumption incorrectly expected the top-left corner to
remain black at 100k; the draw legitimately covers it. The single in-place
harness correction checks the unused bottom-right corner; rerun passed. Required
strict Clippy exposed `chunks_exact_to_as_chunks` on Rust 1.99; one iterator-only
product repair and focused rerun passed. Neither failure disproved colored-node
rendering. Initial failure logs are retained. Product repairs used: **1/2**;
operational retries: **0/2**; harness corrections: **1**.

**Browser/WASM, browser CSP and displayed rendering:** not run. No HTML fixture
or inline script is provided. Other GPUs, software adapters, operating systems,
browser WebGPU, Metal, D3D12 and GLES remain unverified. Broader Vulkan/tool
qualification was not run; absent `vulkaninfo` was not treated as absent Vulkan.
Historical broad CPU/browser/Docker/CI qualification was not rerun. Production
adoption and browser portability remain open decisions outside this research.

YT-023 is Done only for this bounded native scene/proof/frame/memory study and
decision. YT-029/YT-021/YT-024/YT-022 remain Done at their research bounds;
YT-068 stays In progress and YT-020 stays Blocked. Established
YT-030/YT-065/YT-066/YT-067 completion limits remain. The next unblockable work is
**T07 / YT-068**, reconciling the nine candidate READMEs and evidence navigation.
No stage, commit, push, signing, publication or external environment change
occurred in T06.
