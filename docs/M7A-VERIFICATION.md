# M7a Verification — Render Strategy Benchmark

**Date:** 2026-10-09
**Scope:** YT-025, isolated browser/WASM benchmark only.
**Status:** Locally measured architecture evidence; no production renderer API or M7b authoring/reactivity implementation is selected by this report.

## Question

The current `yetlet-dom::mount` creates a complete DOM subtree, clears the owned target and appends the replacement. M7a measures whether this simple baseline remains suitable for frequent keyed updates, and compares it with three bounded alternatives before Yetlet commits to a VDOM, signals runtime, static-template IR or UI macro system.

## Isolated strategies

| Strategy | Implementation in the benchmark | Intended observation |
|---|---|---|
| `subtree_replacement` | Current `yetlet_dom::mount` over a complete shared `View` | Baseline behavior and allocation/replacement cost |
| `view_tree_patch` | Positional recursive patch of old/new `View` trees | Cost and identity behavior of a minimal VDOM-like prototype |
| `keyed_direct_dom` | Stable key → element table, direct text/attribute changes and keyed moves | Upper-bound evidence for explicit bindings |
| `static_template_slots` | Clone a prebuilt row prototype and cache dynamic text/element slots | Bounded template/slot prototype without a macro compiler |

All benchmark code lives in `examples/render-bench` and generated files under Git-ignored `target/m7a-render-bench`. The Todo Browser and Search Island do not depend on it.

## Reproducible workload

The release-mode measurement used:

- 200 keyed rows;
- 500 seeded toggle operations per measured round;
- 10 measured rounds plus one bounded 50-update warm-up per strategy;
- deterministic rotating strategy order per round;
- after toggles: append one row, remove key 1 and move key 2 to the end;
- equivalent final-state checksum validation on every strategy and round;
- outside focus, owned-input identity/focus and untouched-row identity assertions;
- instrumented DOM API-operation counts.

Commands:

```bash
cargo test -p yetlet-render-bench --all-targets
cargo check -p yetlet-render-bench --target wasm32-unknown-unknown

# Capture production assets before the isolated benchmark build.
python3 scripts/build_browser.py --release
python3 scripts/measure_render_strategies.py \
  --capture-production-before target/m7a-production-before.json

python3 scripts/build_render_bench.py --release
python3 scripts/measure_render_strategies.py \
  --profile release --rows 200 --updates 500 --rounds 10 --seed 7 \
  --production-before target/m7a-production-before.json \
  --out target/m7a-render-strategies.json
```

## Environment and artifact isolation

| Field | Observed |
|---|---|
| Host | Linux x86_64 |
| Rust | rustc 1.99.0 (2026-09-28) |
| Browser | Chrome 155.0.8059.39, headless, cache disabled |
| Python | 3.14.4 |
| Benchmark digest | `71182d03b13e4d13` |
| Benchmark WASM | 165,287 bytes uncompressed; 58,383 local gzip bytes |
| Benchmark JS module | 17,346 bytes uncompressed; 3,847 local gzip bytes |
| Production Browser/Search digest before/after | `3f3c5f074478c525` / `3f3c5f074478c525` — unchanged |

The benchmark build does not write to production `dist/`. The measurement command requires a production snapshot captured before the benchmark build and fails before launching Chrome when that baseline is missing or already differs from current `dist/`. Production asset SHA-256 values and sizes were compared again after the benchmark; all were byte-identical.

## Timing observations

Times are `performance.now()` observations inside one browser process. They include Rust/WASM strategy work and DOM calls, use no per-update JS/WASM crossing, and are not Web Vitals or cross-device performance claims.

| Strategy | Setup median / p95 | Update median / p95 (5,000 samples) | Structural median / p95 (30 samples) | Total round median / p95 |
|---|---:|---:|---:|---:|
| Subtree replacement | 0.400 / 0.600 ms | 0.400 / 0.500 ms | 0.400 / 0.500 ms | **213.500 / 228.200 ms** |
| View-tree patch | 0.400 / 0.600 ms | 0.100 / 0.200 ms | 0.300 / 0.500 ms | **41.700 / 42.900 ms** |
| Keyed direct DOM | 0.500 / 0.700 ms | 0.000 / 0.000 ms | 0.000 / 0.100 ms | **2.000 / 2.100 ms** |
| Static template slots | 0.500 / 0.700 ms | 0.000 / 0.000 ms | 0.000 / 0.000 ms | **1.900 / 2.600 ms** |

Sub-millisecond values are limited by the timer resolution and should be interpreted through round totals and operation counts, not as exact zero-cost updates.

The table is transcribed from the retained local report `target/m7a-render-strategies.json`, timestamped `2026-10-09T02:02:22+00:00`. A repeated local run can vary; documentation is intentionally tied to this identified report rather than a later rerun.

## Instrumented operations across ten measured rounds

These counts describe explicit DOM operations performed or logically created by each prototype. They do **not** expose browser-engine internal layout, allocation or clone work.

| Strategy | Update node creations | Update text / attribute mutations | Structural creations / insertions / removals / moves |
|---|---:|---:|---:|
| Subtree replacement | 2,015,000 | 0 / 0 (new nodes replace old values) | 12,110 / 12,110 / 30 / 0 |
| View-tree patch | 0 | 5,000 / 10,000 | 20 / 20 / 10 / 0; plus 4,000 text and 8,012 attribute rewrites from positional reorder |
| Keyed direct DOM | 0 | 5,000 / 10,000 | 20 / 20 / 10 / 10 |
| Static template slots | 0 | 5,000 / 10,000 | 20 / 10 / 10 / 10 |

The lower recorded insertion count for static slots reflects a single `cloneNode(true)` call per row: engine-internal cloned child links are not counted as separate caller-issued DOM operations.

## Identity, focus and correctness

| Strategy | Outside focus | Owned input identity/focus | Untouched keyed-row identity after reorder | Equivalent final state |
|---|---|---|---|---|
| Subtree replacement | Preserved | Lost / lost | Lost | PASS |
| View-tree patch | Preserved | Preserved / preserved | Lost under positional reorder | PASS |
| Keyed direct DOM | Preserved | Preserved / preserved | Preserved | PASS |
| Static template slots | Preserved | Preserved / preserved | Preserved | PASS |

All four strategies produced the same final checksum for the same operation sequence in every round. No WebSocket, server event forwarding or production route was involved.

## Interpretation and decision boundary

The measured workload shows that repeated whole-subtree replacement performs substantially more node creation and loses owned input/keyed-row identity. A positional `View` patch reduces replacement cost and preserves stable non-list nodes, but its reorder behavior rewrites many rows and loses semantic keyed identity. In this benchmark, explicit keyed bindings and cached template slots produced similar round totals and preserved identity.

This evidence supports **continuing M7b with a small keyed-binding/static-slot authoring prototype rather than adopting a general VDOM by default**. It does not select a final renderer or justify a procedural macro yet. M7b must evaluate ergonomics, conditional trees, event captures, error handling, SSR identity, memory/layout effects and realistic component composition before a public API decision.

## Limitations

- One local machine and one Chrome process; no Firefox/Safari/mobile timing comparison.
- No layout/paint, heap, GC, concurrent workload or long-lived leak profile.
- The positional tree patch is intentionally bounded and not production VDOM code.
- The template prototype uses runtime `cloneNode`, not compile-time Rust template extraction.
- The direct and slot strategies model an optimized known-shape list; general conditional component trees can have different costs.
- No comparison with React, Leptos, Dioxus, Yew, Sycamore or other frameworks.
- Results are architecture evidence, not a performance SLA or release gate.

The complete machine-readable report is generated at `target/m7a-render-strategies.json` and is intentionally not committed.
