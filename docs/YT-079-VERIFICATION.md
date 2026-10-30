# YT-079 — Experimental Renderer Extraction Candidate

**Date:** 2026-10-14. **Backlog: Done at the Experimental opt-in scope.**
Implementation and focused CPU/native/WASM/package checks PASS. The extracted
candidate has **actual browser PASS** at the selected Firefox X11/Intel scope.
[Fresh independent API/dependency/security review](YT-079-INDEPENDENT-REVIEW.md)
**PASS**, zero issues and zero required followups. YT-080 remains a separate item. Earlier implementation-only statements below are
historical; the current browser result takes precedence.

## Current parent-owned candidate browser result

The original guarded local commit `8c524520256e5fd9884d4d023dbdfbd76181c561`
was already complete before recovery; it was not repeated. The parent resumed
the interrupted YT-079 task after confirming both earlier candidate executions
were terminal and all their owned processes/profiles/ports cleaned.

The current `verify_candidate.py` completed with `passed_yt079_candidate`,
using the actual native SSR consumer and extracted crate's release Wasm.
Its preserved receipt is
`target/m11-gpu/yt079-20261014/browser/page-module-hook/result.json`, SHA-256
`b0af8d7c0526b2c35b53801aadd89cd77d9f8b2aa3323d1627ce1eaa262ee253`.
All **65 bracketed action records** completed; five real canvas captures
(initial, replacement, status update, recovered, and post-retirement)
each checked **120/120 colored centers** and black interior. Mouse/keyboard
selection, stale pick rejection, 390 CSS-pixel viewport emulation, bounded
status uploads, controlled real device loss, explicit recovery, unavailable
capability fallback and actual scriptless SSR navigation passed.

The held real queue-completion case observed exactly one old buffer/device/
canvas-context release, no Wasm wrapper free while borrowed, exactly one old
wrapper free after settlement, and an unaffected replacement generation.
Final replacement teardown observed the second wrapper free. Device failures
are controlled injections, not claims of spontaneous hardware failure.

The original `browser/result.json` retains an offscreen setup timeout.
`browser/visible-target/result.json` retains a wrapper-hook timeout: WebDriver's
script-world module import did not instrument the page's actual class instance.
The prepared correction adds a single-purpose same-origin external module to
only the disposable asset copy, preserving strict CSP and all product code.
The hook was loaded in the page module world before instrumenting `.free()`;
its source SHA-256 is
`00e485a2741db4120742a08a3150b44b1cd54684129078679931e8a869cdb019`.
No production asset or assertion threshold was changed to manufacture a pass.
Both earlier failed attempts remain intact and are not relabeled as success.

The successful test deleted its WebDriver session, reaped Firefox, driver and
server, removed the disposable profile/asset copy, closed both loopback ports,
and reported unchanged source/binary/asset hashes. A fresh PID check confirmed
both reported server/driver PIDs absent. Do not repeat this passing browser
configuration solely for redundant proof. The fresh independent review now
passed with zero issues; it verified source and all five PNGs without rerunning
the browser. YT-079 is Done and YT-080 may proceed under its own review gate.

## Authority and preserved input

This is exactly the owner-approved Experimental opt-in extraction in
[M11-G section 7 and section 8](M11-WGPU-PLAN.md),
[ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) and the
[explicit promotion decision](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md).
No useful-speedup prerequisite remains for this Experimental candidate. There
is no speedup, performance, FPS, stability, stable API, production or publication
guarantee. Future 3D is motivation only; no 3D API, engine or scene graph exists.
Repository AGENTS.md, ADRs 0003/0005/0008, current YT-078 source and verification,
prior acceptance receipts and Materiality Gate were read. No workflow or managed
runtime identity is claimed; no child agent was dispatched.

Committed baseline remains `8c524520256e5fd9884d4d023dbdfbd76181c561`.
Extraction uses the newer uncommitted YT-078 source, not the older committed
source. Initial Rust source and consumer lock snapshots are under
`target/m11-gpu/yt079-20261014/implementation/input-source/`. YT-078 application
picking/input/SSR changes are retained; its server, main and legacy WGSL files
are byte-identical to the input snapshot. Historical receipt meanings are not
changed. YT-078 verification and `verify_picking.py` are byte-identical, as are:

- `target/m11-gpu/yt078-20261014/reviewer/result.json`:
  `491fa16a0900934fe77036745c47e96840a5a1a1cbc7e7793841372a4fee7675` (PASS).
- `target/m11-gpu/yt078-20261014/browser/viewport/result.json`:
  `a265e7feb6aa72191253532d206848dda33687c74bd2e94872fb7ada3894f92d` (selected PASS).

The owner decision, M11 GPU plan and ADR 0008 are also byte-identical to the
pre-work snapshot. No commit, push, fetch, install, detached worktree, server,
browser, host change or root-policy change was performed. All commands were
bounded one-shot operations and completed; there are no task-owned runtime
processes or ports to clean up.

## Actual changed files and consumer

| Files | Delivered behavior |
| --- | --- |
| `crates/yetlet-wgpu/Cargo.toml`, `Cargo.lock`, `README.md` | First-party `publish = false`, Experimental opt-in crate in its own nested workspace; default CPU-only, explicit wasm32 `browser` feature, selected support and quickstart |
| `crates/yetlet-wgpu/src/lib.rs`, `tests.rs` | Proven finite rectangle validation, viewport bounds, validated private CPU batch/version state, dirty ranges, reverse picking and lifetime guards; four crate tests and one doctest |
| `crates/yetlet-wgpu/src/browser.rs`, `scene.wgsl` | Actual device/surface, resize, upload/replacement, draw, completion, scalar telemetry and deterministic disposal; trusted crate-owned build-time shader |
| `examples/webgpu-scene-island/Cargo.toml`, `Cargo.lock` | Real local path consumer; explicit browser feature only for wasm32; registry versions/checksums preserved |
| `examples/webgpu-scene-island/src/lib.rs`, `src/browser.rs` | Application synthetic work-record generator/status patch retained; CPU contracts re-exported and thin Wasm binding delegates every renderer operation to the crate |
| `examples/webgpu-scene-island/scene-loader.js` | Configured context release delegates to the extracted owner; existing DOM scheduling, safe semantic selection, generation bridge and deferred wrapper freeing remain application-owned |
| `examples/webgpu-scene-island/build.py`, `verify_candidate.py`, `README.md` | Release manifest names/hashes extracted source; one prepared direct candidate product test and parent invocation |
| `docs/BACKLOG.md`, this record | YT-079 In progress, focused evidence and explicitly pending browser/review gates |

The reusable crate has no `window.__yt076Fault`, synthetic generator, business
labels, server routes, 120-record product constant or counter. Its browser owner
accepts a validated `Batch`, generation and injected `fn(u32, &str)` fault sink.
The application Rust binding owns the existing global bridge and synthetic data.
Public renderer types do not expose wgpu types. The documented browser boundary
uses `HtmlCanvasElement`/`JsValue`; there is no runtime shader input.

Private Batch storage preserves bounded validation and checked data identity.
Maximum 100,000 rectangles, 32 bytes each; finite normalized geometry and RGBA,
checked ranges, clamped DPR and negotiated texture limits remain. Picking is a
bounded reverse O(n) scan with one-based slot IDs; the application alone maps
slots to stable business records. Successful updates/replacements advance data
version; stale version/generation/retired picks fail. Zero/detached targets
suspend. Errors/categories and explicit recovery responsibilities are documented
in the [crate README](../crates/yetlet-browser/guides/wgpu.md) and public rustdoc.

No initialization occurs on linking. Each explicitly constructed owner owns CPU
data separately from its device. The injected callback retains generation and
lifetime filtering. Disposal invalidates lifetime before resource release.
Because wgpu 27 WebSurface Drop is a no-op, the crate now unconfigures only its
own configured canvas before dropping buffers/device; a pending unconfigured
old initializer cannot unconfigure a replacement. The application retains the
Wasm wrapper until all initialization/completion futures settle and then frees
it once. Actual browser verification of this candidate is still PENDING.

`/scene`, `/scene-comparison`, comparison payload semantics, keyboard/touch/
reduced-motion behavior, accessibility labels, independent counter and no-JS
SSR remain application-owned. All twelve current native test contracts pass
against the extraction. Version-exhaustion mutation testing moved behind the
crate's private Batch state; the public consumer still tests failed-validation
atomicity. Scalar allocation/upload lengths and CPU/wall durations are honest:
no measured VRAM, GPU time, presentation, FPS or speedup claim is made.

## Dependency, package and current asset isolation

The root workspace already includes `webgpu-scene-research`, an older native
wgpu research lane. Whole-workspace absence of wgpu is not claimed. Actual named
core/HTML/server/DOM graphs were inspected:

```sh
cargo tree --offline --locked -e normal,build -p yetlet-core -p yetlet-html -p yetlet-server-axum -p yetlet-dom
```

They have no renderer/wgpu edge. The candidate default native and wasm32 graphs
contain only `yetlet-wgpu`; no dependencies or GPU are required. The consumer
native server graph includes CPU-only `yetlet-wgpu` and has no wgpu dependency.
Only explicit wasm32 `browser` enables `std`, `webgpu` and `wgsl`; no Vulkan/native
backend feature is enabled. Graphs are retained as `yt079-graph-*.log` under the
existing implementation evidence directory.

Both candidate and consumer use isolated nested workspaces, not Git worktrees.
Use explicit `--manifest-path` commands below. The consumer lock only replaces
its direct wgpu edge with the new local crate entry/edge. All original registry
package versions/checksums are identical. The new crate lock is a subset of those
same registry versions/checksums. Exact pins remain wgpu `27.0.1`, wasm-bindgen
`0.2.129`, wasm-bindgen-futures `0.4.79` and web-sys `0.3.106`. No upgrade/download
was performed; `yt079-version-isolation.json` records the comparison.

Eight CURRENT live Todo/Search assets were hashed before and after, not rebuilt
or replaced with historical digests. All eight are byte-identical. Root
`Cargo.lock` is byte-identical, SHA-256:
`2e08330c49ff4e1740102a37b36fbe2fd044bd2033d8ea9a6aaf224a402866c3`.
See `yt079-isolation-before.json` and `yt079-isolation-after.json`. The three
owner decision documents and historical YT-078 receipts match those snapshots.

Current scene digest is `68a16c64a18ef9d1`, Wasm 287,094 bytes. Independent
counter remains `81c388455a88d98a`, Wasm 151,613 bytes. The release manifest
contains candidate source SHA-256 entries, including the extracted crate and
consumer manifests/locks/Rust/shader. `yt079-export-proof.json` records actual
generated SceneOwner exports and 48 extracted implementation name entries in
the linked Wasm, including initialization, draw, picking and context release.
This proves bindings reach the crate, not browser pixels. Current binary,
sources, manifest and generated assets are hashed in
`yt079-candidate-input-hashes.json`; bundle sizes/hashes are in
`yt079-bundle.json`. These are raw artifact sizes, not transfer/performance data.

## Actual focused commands and results

Installed Cargo/rustc are 1.99.0; cached tools/dependencies were used offline.
Logs are under `target/m11-gpu/yt079-20261014/implementation/`.

| Command | Result |
| --- | --- |
| `cargo test --offline --locked --manifest-path crates/yetlet-wgpu/Cargo.toml` | PASS: 4 CPU tests and 1 compile-checked doctest, zero failed/ignored/filtered; `yt079-crate-tests.log` |
| `cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | PASS: all 12 existing focused contracts, zero failed/ignored/filtered; `yt079-consumer-tests.log` |
| `cargo fmt --manifest-path crates/yetlet-wgpu/Cargo.toml --check` and the same consumer command | PASS; `yt079-crate-fmt.log`, `yt079-consumer-fmt.log` |
| `cargo clippy --offline --locked --manifest-path crates/yetlet-wgpu/Cargo.toml --all-targets -- -D warnings` and the same consumer command | PASS; `yt079-crate-native-clippy.log`, `yt079-consumer-native-clippy.log` |
| `cargo clippy --offline --locked --manifest-path crates/yetlet-wgpu/Cargo.toml --target wasm32-unknown-unknown --features browser --lib -- -D warnings` | PASS; `yt079-crate-wasm-clippy.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --target wasm32-unknown-unknown --lib -- -D warnings` | PASS; `yt079-consumer-wasm-clippy.log` |
| `cargo check --offline --locked --manifest-path crates/yetlet-wgpu/Cargo.toml --target wasm32-unknown-unknown --features browser` | PASS; `yt079-crate-wasm-check.log` |
| `RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --no-deps --manifest-path crates/yetlet-wgpu/Cargo.toml`, also with `--target wasm32-unknown-unknown --features browser` | PASS after the bounded documentation repair; `yt079-crate-rustdoc-repaired.log`, `yt079-crate-browser-rustdoc.log` |
| `cargo package --offline --locked --allow-dirty --manifest-path crates/yetlet-wgpu/Cargo.toml` | PASS: 9 archive files, actual default package compile verification; nonblocking local-candidate metadata warning, no publication; `yt079-package-verified.log` |
| `cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | PASS native server build; server NOT launched; `yt079-native-build.log` |
| `python3 examples/webgpu-scene-island/build.py` | PASS source-bound release Wasm/bindgen and sibling counter; `yt079-final-release-build.log` |
| Node module syntax for all 3 loaders; Python AST parsing for all 7 application scripts | PASS, no fixture runtime execution; `yt079-syntax.log` |
| `python3 scripts/check_docs.py`; `git diff --check` and new-file whitespace inspection | PASS; `yt079-docs-check.log`, `yt079-whitespace.log` |

The initial strict rustdoc check failed on unescaped numeric ranges interpreted
as intra-doc links. One minimal product-documentation repair rendered ranges/
token arrays as code; both native/browser rustdoc pass. The initial ad hoc graph
assertion incorrectly matched `yetlet-wgpu` as the standalone `wgpu` package;
one smallest assertion correction passed, with no product change. The Node
syntax command initially used absent `/usr/bin/node`; one operational retry
used the verified existing PATH executable and passed. Failure records/logs are
retained; no failing check was relabeled as a PASS receipt.

Counters: product repairs **1/2**, operational retries **1/2**, harness-only
corrections **1/1**. Last observed signature: `Node executable /usr/bin/node
absent`, resolved with installed PATH Node. There is no outstanding failure.

## Historical implementation handoff: prepared browser run

After announcing the visible launch, the parent runs exactly once:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_candidate.py
```

The reserved directory is `target/m11-gpu/yt079-20261014/browser`; it does not
exist at leaf handoff. The fixture refuses existing evidence, checks source-bound
candidate hashes and hashes current source/binary/assets itself. It does no
build/install and uses one headful Firefox 158 X11 session with ONLY disposable
`dom.webgpu.enabled=true`. It reuses established WebDriver/PNG/process helpers
and YT-078's corrected standard BiDi 390 x 700 viewport seam. This is viewport
emulation, not physical mobile support. Deadline 180 seconds, owned process bound
240 seconds with cleanup reserved.

The bounded direct product sample covers actual canvas pixels, known mouse pick
and keyboard selection, data replacement and a 12-record status upload with
honest byte/write counters, narrow responsive selection, controlled real
`device.destroy()` loss with semantic/counter survival and one explicit recovery,
unavailable capability with explicit recovery, and retirement during a held real
queue completion. It checks old wrapper free exactly once after settlement,
one old buffer/device/context release, no stale effect on replacement generation,
final resource counters and actual scriptless SSR search/page/detail navigation.
The loss/availability/held completion boundaries are controlled injections, not
physical GPU failure. Pixel tolerance remains 25 per channel, using the proven
one-row capture seam. This is not the full YT-077 benchmark or a replay of YT-075.

Cleanup always deletes the session, reaps owned server/driver process groups,
checks Firefox exit and closed ports, removes the disposable profile/asset tree
and verifies hashes stayed unchanged. A missing cleanup cannot pass. The leaf
has NOT executed browser pixels, runtime lifecycle, CSP browser enforcement,
input dispatch or accessibility checks. Native SSR/CSP tests and Wasm builds are
separate evidence and do not qualify those runtime outcomes.

Public support says ONLY the previously proven Firefox 158 / Ubuntu X11 / Intel
integrated GPU configuration with disposable WebGPU opt-in; extracted candidate
browser proof is PENDING. Windows JS baseline and failed Chrome configurations
are not candidate PASS. No physical-mobile, broad-browser, native-renderer,
production/stable-API or 3D support is claimed.

There are no additional open architecture decisions or true implementation
blockers in this slice. Parent execution then a DIFFERENT independent reviewer
remain the two required gates. YT-079 stays In progress until accepted. The next
backlog item is YT-080 only after both gates pass; no YT-080 work is authorized or
included in this delivery.
