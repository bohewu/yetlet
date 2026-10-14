# YT-077 — Dynamic Scene Implementation and Prepared Measurement

**Date:** 2026-10-14. **State: Done at selected matched measurement scope after independent reviewer PASS.**
Implementation/build checks **PASS** and the current
[matched browser measurement](YT-077-MEASUREMENT.md) **PASS**:
24/24 lanes, 2,160 measured workload samples, six sustained lanes,
126 real pixel assertions with zero mismatches and verified cleanup.
The first failed screenshot attempt is retained, not counted as a
performance result. The implementation-only checkpoint and its pending
handoff language below are historical, not the latest browser status.
YT-076 lifecycle/reviewer PASS remains accepted at its original scope.

## Current selected browser result and pending gate

The parent executed the prepared Firefox 158 X11 experimental-opt-in
comparison once; that first attempt failed at 50k Canvas2D due to
a documented Firefox one-row element-screenshot alignment mismatch,
after successful lower-size lanes. A bounded correction mapped known
canvas coordinates into the capture only when its full top row was
white; pixel tolerance, datasets, geometry, samples and performance
SLA were not changed. The second distinct, isolated
`capture-alignment-fix` attempt completed all 24/24 lanes with
no omissions, zero bounded pixel mismatch across 126 capture checks,
same source hashes and owned cleanup. It took 275.30 seconds of
browser process time. All 72 workloads contain 30 measured and five
warmup samples, and six sustained 10k lanes completed.

The [measurement record](YT-077-MEASUREMENT.md) includes exact receipt,
capture classification, per-lane metrics, bundle cost, 1K/10K/50K/100K
scope and the explicit lack of demonstrated GPU speedup. No
browser/renderer breadth or performance target is inferred.
The [fresh independent reviewer](YT-077-INDEPENDENT-REVIEW.md) returned
**PASS**, with zero issues or required followups, after independently
reconstructing all 126 PNG proofs and checking sample math, buffer
bounds, source hashes and cleanup. YT-077 is **Done** in this scoped
measurement; YT-078 requires its own implementation and review.

## Authority and source preservation

The bounded owner assignment authorizes deterministic dynamic batches and one
focused, prepared Canvas2D versus actual Rust/wgpu 27 Wasm comparison. Operational
parameters were selected by the parent, not supplied as a user performance SLA.
Acceptance is correct equal work, bounded allocations/errors/cleanup and honest
reporting; GPU speedup and 60 Hz are not requirements. Source contracts are
[M11-G sections 5, 7 and 8](M11-WGPU-PLAN.md), [YT-077 backlog](BACKLOG.md),
[proposed ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) and
[current YT-076 evidence](YT-076-VERIFICATION.md).

The same checkout remains at `76be7aa1107e08e3c7850abc8c68f53301cf0f7c`.
Prior uncommitted YT-075/076 source and evidence were retained. The parent-supplied
pre-change snapshot, verified against all 14 current file entries before changes,
is `target/m11-gpu/yt077-20261014/implementation/yt076-source-snapshot.zip`.
There was no commit, push, installation, host configuration, Windows/cloud work,
publication, extraction, delegation or YT-078+ implementation.

## Changed files and behavior

| File | Required change |
| --- | --- |
| `examples/webgpu-scene-island/src/lib.rs` | Bounded deterministic grid and status-patch preparation, finite geometry/color and byte/range validation, retained CPU batch and coalesced dirty ranges; focused CPU coverage |
| `examples/webgpu-scene-island/src/browser.rs` | Actual wgpu COPY_DST updates/replacement, count-driven draws, retained CPU data, explicit allocation/upload/startup counters and asynchronous queue completion |
| `examples/webgpu-scene-island/scene-loader.js` | Owner-demand dynamic update/replacement, CPU retention across explicit recovery and small application diagnostics; original lifecycle remains |
| `examples/webgpu-scene-island/comparison-loader.js` | Separate application entry with matched Canvas2D/Rust-GPU workload and out-of-sample correctness capture |
| `examples/webgpu-scene-island/src/server.rs` | `/scene-comparison` external script entry, same strict CSP/SSR/counter, shared fixed canvas CSS dimensions |
| `examples/webgpu-scene-island/build.py` | Include comparison script in current-source content digest; no historical asset reuse |
| `examples/webgpu-scene-island/verify_dynamic.py` | One direct product-specific prepared measurement fixture using existing Firefox/PNG/process helpers |
| `examples/webgpu-scene-island/README.md`, `docs/BACKLOG.md`, this record | Method, actual build evidence and In progress/pending gates |

The default `/scene` still has 120 records with original byte-identical geometry
and colors, semantic HTML SSR/search/pagination/detail, independent counter and
explicit GPU activation. Comparison code loads only on `/scene-comparison` and
its driver explicitly selects activation. Canvas2D owns a separate 2D context;
only one canvas is visible/painted in each lane. No GPU fallback paints underneath
it. Normal scenes retain YT-076 resize/DPR clamps, zero/detach/offscreen/hidden
pause, generation guards, error/loss handling and explicit recovery/cleanup.
Dynamic application bytes stay outside clone-per-dispatch LocalGraph. Exact
wgpu `=27.0.1`, wasm-bindgen `=0.2.129` and web-sys `=0.3.106` pins/features are
unchanged, with no core/default GPU dependency or new dependency.

## Parameters frozen before browser execution

| Parameter | Prepared value |
| --- | --- |
| Renderers | Canvas2D full clear/redraw; actual Rust/wgpu WebGPU full clear/render |
| Data | Synthetic grid, row-major stable index, seed 7, RGB statuses; maximum 100,000 records, 32 bytes each |
| Sizes | 1,000 / 10,000 / 50,000 / 100,000 |
| Viewport | Fixed 600×400 CSS and backing pixels, actual DPR 1; mismatch fails, no injected DPR preference |
| Rounds/order | Three independent document/owner rounds per renderer/size in one browser process: round 0 Canvas→GPU, round 1 GPU→Canvas, round 2 Canvas→GPU; ascending sizes within each round |
| Sparse | First contiguous 1% of records; status at index i is `(i + 7 + step) mod 3`; full final canvas rendering on both |
| Full | All records receive the same ordered status operation; no geometry change |
| Burst | Eight sparse status operations before one coalesced demand draw, retaining latest state; no unbounded input queue |
| Samples | Five warmups, then 30 measured samples per workload/lane; steps continue sparse→full→burst identically |
| Sustained | Each 10k lane: 10 seconds of sequential sparse samples with backpressure, raw times and per-sample counters; next operation starts after previous completion and two RAF opportunities |
| Replacement | Fresh seed-9 full scene replacement and canonical sequence reset, correctness checked outside measured workloads on both renderers |
| Deadline | 420-second measurement cutoff including startup; 60 seconds reserved for cleanup; one browser process ≤480 seconds |
| Omission | Remaining/incomplete lanes explicitly labeled not measured/bounded out; partial samples retained, no loser claim and no automatic retry |
| Target | Existing Firefox 158 headful X11, disposable profile with only `dom.webgpu.enabled=true`; actual non-fallback adapter required |
| Observation | 16.67 ms / 60 Hz is a comparison reference only, no acceptance promise |

The expected operational duration is roughly 3–7 minutes including the six
10-second sustained lanes, startup and captures; it is an estimate, not a user
promise. Slow lanes may be bounded out. The fixture never rebuilds, overwrites a
receipt or expands to a second browser. No unsafe/ignore-blocklist flag or
privileged setting is enabled.

Dynamic grid columns are `ceil(sqrt(count*1.5))`, rows are `ceil(count/columns)`.
Integer CSS-pixel left/top boundaries and integer width/height (at least one)
are converted once into f32 normalized center/half-extent bytes. Canvas reconstructs
those integer rectangles from the same bytes; the GPU shader uses them directly.
At 100k each rectangle is one CSS pixel in each dimension. This selected fixture
avoids subpixel antialiasing differences; it does not qualify future rectangles
smaller than one CSS pixel or arbitrary geometry.

## Timing and correctness definitions

Both lanes use the same exported Rust generation/status-patch code, inputs,
operation order and CPU-side canonical bytes. Canvas consumes CPU data, never
impersonates the GPU. The CPU/wasm-bindgen preparation bridge is therefore common
and charged to both lanes. Canvas's standalone minimal JS bundle cost is not
measured: this fixture deliberately shares Rust preparation. GPU has additional
Rust CPU retention/validated updates and a JS recovery copy; that bridge/apply
cost is recorded rather than hidden. Preparation currently validates/copies the
full canonical input into Wasm even for sparse patches; this CPU cost is included
and does not imply a full GPU upload.

Startup separately retains module import wall time, Wasm initialization wall
(including fetch/instantiate), actual ResourceTiming `transferSize`,
`encodedBodySize`, `decodedBodySize`, start/duration and zero-transfer flag.
A zero transfer is cached or unobserved, not proof of zero artifact cost.
Adapter request, device request, pipeline construction and initial upload API
wall times come from Rust performance counters. These CPU durations do not
promise the driver finished compiling. Total initialization includes validation
await/configuration; first usable **matched** frame includes the default 120-record
activation followed by the selected dynamic replacement/draw and queue wait.
Initial dynamic upload CPU wall and uploaded bytes are separately retained.
Current raw and deterministic gzip file sizes are artifact costs, not observed
HTTP compression or transfer. Independent documents share the process HTTP cache;
only the first observed fetch can be cold. No historical native timing or asset
identity substitutes for these observations.

One measured sample is an ordered sparse/full/burst preparation and application
ending in exactly one full final-state demand draw. Raw fields include preparation,
bridge/apply, dirty-upload CPU wall, encode+submit/present-call CPU wall, draw
observation wall, awaited queue-completion wait/wall and two subsequent RAF
opportunity intervals. The draw observation occurs after a RAF polling opportunity;
queue completion may already be finished at that point. Canvas completion wait is
null, not zero GPU time. Completion wall is an update-to-result observation with
asymmetric API completion semantics, not a GPU execution duration or physical
paint. RAF intervals are scheduling/presentation opportunities, never measured
paint/FPS. GPU timestamp queries are not requested. `performance.now()` uses normal
browser timer precision; zero/quantized samples are retained honestly.

Nearest-rank p50/p95/p99 are computed separately per renderer/size/round/workload
and field from the 30 measured samples, excluding warmups and captures. Sustained
distributions use all actual samples in the declared lane, with actual elapsed
wall and counters. The sustained lanes execute identical work per operation for
10 seconds; their final step counts can differ. No completion-derived FPS or
throughput-to-display claim is made. Raw operation order/count/sequence, draw,
coalesced, writes/upload bytes, checksum and allocations remain in sample JSON.
Backpressure admits no intentional dropped input; burst intermediate states are
coalesced and all semantic updates are applied. Exactly one draw and dirty bytes
are asserted per sample.

Both renderers get a fresh out-of-sample full redraw, GPU queue wait where
applicable and two RAF opportunities before an actual current canvas screenshot.
Initial, sparse, full, burst, sustained and replacement captures check bounded
known pixels and black interior. Known CPU colors and an FNV-1a full-byte checksum
are retained; fixed-round/workload/replacement state identities must agree across
renderers. PNGs are independently verified against canonical colors, not against
each other's capture compression. The declared per-channel tolerance is 25,
retained from YT-076 **before execution**; it cannot be adjusted after a failure.
The possible one-row Firefox capture seam is recorded and y=0 is excluded from
interior checks. Sparse-changed records and up to 128 evenly distributed records
are sampled. This is bounded pixel sampling, not a claim that every 100k mark was
captured/verified. Correctness captures/checksums and WebDriver calls are outside
the sample durations, except small draw counters needed to observe completion.

## Bounded allocation and cleanup counters

The active GPU record buffer is at most 3,200,000 bytes. It is reused when capacity
suffices; larger replacements allocate once, destroy the old buffer and record
allocation count/peak transition bytes (at most 6,400,000 record-buffer bytes).
Sparse/burst upload is exactly `count/100*32` bytes in one contiguous write after
coalescing; full status upload is `count*32`. Unchanged draws upload zero bytes.
Geometry/colors/count/ranges and checked index arithmetic are validated before
mutation or affected GPU work. Failed payloads leave prior bytes intact.

Rust retained record capacity is at most 3.2 MB plus 100k dirty bytes; the owner
reports length/capacity explicitly. Canonical JS records and GPU recovery JS
copy are each at most 3.2 MB. Patch payloads are bounded at 3.2 MB (sparse/burst:
32k maximum per operation). Wasm bridge/return scratch and browser GC are
additional transient costs; explicit application capacities are not Wasm heap,
OS RSS or resident VRAM observations. The renderer also reports draw/write/upload
and buffer-allocation counters. Canvas backing is 240,000 pixels; its internal
browser allocation/VRAM is not observed. This test requests no readback buffer,
timestamp query or OS memory inference.

Dynamic CPU bytes survive loss through the application recovery copy and are
reapplied to a fresh explicit generation. On final comparison disposal the CPU
copies, Canvas backing/context, RAF and GPU resources are released. Existing
YT-076 observers/listeners/generation teardown remains owned. The browser fixture
records actual allocated ports/PIDs/profile, source/binary/current asset hashes,
per-lane owner cleanup and final session deletion, owned process reap/browser exit,
profile removal and port closure on success/failure. Source hashes are compared
again at completion. There is no shared smoke infrastructure change.

## Actual implementation/build verification

All final required checks passed; logs are at
`target/m11-gpu/yt077-20261014/implementation/`.

| Command | Actual evidence |
| --- | --- |
| `cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | 9 passed, 0 failed/ignored/filtered; `cpu-test-final.log` |
| `cargo fmt --manifest-path examples/webgpu-scene-island/Cargo.toml --check` | PASS; `fmt.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --all-targets -- -D warnings` | PASS; `native-clippy-final.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --target wasm32-unknown-unknown --lib -- -D warnings` | PASS; `wasm-clippy-final.log` |
| `cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | PASS, server not launched; `native-build.log` |
| `python3 examples/webgpu-scene-island/build.py` | Release build/bindgen PASS; `release-build-final.log` |
| `node --input-type=module --check` for three loaders; `ast.parse` for five fixture Python files | PASS; `syntax.log`, no browser execution |
| `python3 scripts/check_docs.py` | PASS including this record; `docs-check.log` |
| `git diff --check` and focused untracked whitespace checks | PASS; `whitespace.log` |
| SHA-256 of eight current Todo/Search assets and both lockfiles | All unchanged; `isolation-before.json`, `isolation-after.json` |

CPU tests retain all seven YT-076 tests and add the four selected dynamic counts,
finite/bounded deterministic geometry, exact initial/patch colors, sparse dirty
range/no untouched mutation, eight updates coalesced to latest state, full status,
replacement growth, invalid byte length/count/color/geometry and invalid/overflowing
indices with atomic rejection. SSR tests retain default no-assets behavior,
search/pagination/escaping, language, strict CSP and sibling fallback.

Current scene digest is `6ea5f2e01d31af77`; scene Wasm is 281,908 raw / 76,077 gzip
bytes. Scene generated JS is 58,838 / 11,755; scene loader 11,367 / 3,485;
comparison-only loader 9,794 / 3,326. Per-file sizes/SHA-256 and current manifest
are in `bundle-cost.json`. Gzip is deterministic Python gzip with mtime 0,
not server transfer evidence. Counter digest remains `81c388455a88d98a` with
151,613 raw Wasm bytes; both locks and eight unrelated assets are unchanged.

## Historical implementation and handoff checkpoint (before browser measurement)

Initial strict Clippy found the Rust 1.99 constant-chunk API lint, a collapsible
conditional and lexical RefCell borrow scope around an await. These are
`product_failure` against the explicitly required lint gate. Materiality Gate
was reread; repair 1 changed those exact API/scope issues. Final checks passed.
Source inspection then identified a concrete async-lifetime gap: an owner could
free a Rust object while its new queue-completion Promise held a borrow. Repair 2
tracks those pending completions per generation, disposes resources immediately
and defers object freeing until completion settles; retired completion cannot
affect its replacement. This preserves the original YT-076 retirement contract.
Materiality Gate was reread before the repair; release build and JS/Python syntax
were rerun successfully. Product repairs used: **2/2**. An initial snapshot-creation command encountered
an already existing parent-provided implementation directory before mutation;
operational retry 1 inspected and verified the existing snapshot instead of
overwriting it. Operational retries used: **1/2**. Harness repairs/executions: 0.
Last failure signature is resolved; there is no current blocking failure.

No server/browser/background runtime was started by this leaf; all bounded build
commands completed. Measurement results, screenshot pixels, adapter, runtime
startup/transfer and dynamic browser lifecycle remain **Not run**, with no speedup
claim. Parent reviews these parameters, then runs exactly once:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_dynamic.py
```

The unique `target/m11-gpu/yt077-20261014/browser` result directory does not yet
exist. Prepared outputs are `input-hashes.json`, current `result.json`, source
identity/manifest/bundle data, per-lane raw sample/sustained JSON, per-state PNGs
and owned server/geckodriver logs. The parent then performs independent review
before changing YT-077 status. YT-077 stays In progress; YT-078 is the next dependent
backlog item only after that gate. No later work is authorized in this leaf.
