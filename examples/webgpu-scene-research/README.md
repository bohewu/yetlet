# Independent scene research — YT-023

An unpublished native offscreen experiment using wgpu 27.0.1 and one WGSL
instanced rectangle pipeline. It has no Yetlet dependency, DOM/CSS integration,
window, HTTP server or scene hierarchy. The application explicitly selects the
`gpu` feature and owns all graphics resources. Default features and ordinary
tests require no GPU device. This research fixture is outside the 12 user-facing
sample catalog entries and does not add a tenth library candidate.

From the repository root, after fetching locked dependencies:

```bash
cargo test -p yetlet-webgpu-scene-research --offline
cargo build --release -p yetlet-webgpu-scene-research --features gpu --offline
timeout 60s target/release/scene-bench
```

The benchmark requires a usable native Vulkan adapter. Failure to obtain one is
an explicit execution failure, not a default CI gate or a silently skipped GPU
proof. This fixture does not implement browser WebGPU. The shader is embedded
at Rust build time; wgpu creates the native pipeline once per command, outside
frame timing. No request-time compilation is introduced.

Inputs are deterministic 10,000, 50,000 and 100,000 rectangle instances on a
1024×1024 RGBA8 target. Each instance has eight floats (center, half extent and
opaque RGB color), with six shader-generated vertices. Grid centers are aligned
to pixel centers; colors cycle red/green/blue. A readback checks every node's
center and an unused bottom-right background pixel. No-op/clear-only rendering
fails this proof. Five warmup frames precede 30 sequential timed frames per size.

[Raw samples](measurements.csv) retain all integer nanosecond wall times and
explicit instance/texture/readback sizes. [Actual draw proof](render-proof.txt)
records adapter/backend, checked centers, nonblack pixel counts, image FNV-1a
fingerprints and cleanup. [Source identity](source-identity.json) pins measured
source hashes and environment. [Research report](../../docs/YT-023-WEBGPU-RESEARCH.md)
explains methodology, native-only limitations and the adoption decision.

Timing includes CPU command encoding, submission and waiting for that submission
to complete. It excludes scene preparation, upload, pipeline initialization,
readback and printing. It is neither a GPU timestamp nor display FPS. Memory
columns describe explicit buffer lengths and logical texture bytes, including a
retained proof readback buffer; they are not measured VRAM or RSS. Each workload
destroys its resources before the next allocation; the device and instance are
released at command completion. No server, browser, watcher or daemon is started.
