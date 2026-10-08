# ADR 0003 — Dual-target renderers and opt-in browser loading

- **Status:** M2 dual renderer and M3 lazy Search Island/versioned state v1 locally validated; generic resumability and template IR remain proposed
- **Date:** 2026-10-08
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md)

## Context

Yetlet aims to reuse Rust UI logic across SSR and CSR while minimizing browser payload for server-rendered pages. Native HTML serialization, browser DOM APIs, and WebGPU scene rendering require different capabilities. Forcing all of them through a single rendering implementation would invite unnecessary abstraction.

## Proposal

- Share **component semantics and source**, not necessarily binaries. Generate native executables, browser WASM, external bootstrap scripts, and manifests as separate build artifacts.
- **Server:** A native HTML renderer emits complete HTML. Pages without interactive components ship no JS or WASM.
- **SSR + islands:** Preserve the server-rendered markup. A tiny external JS loader resolves a build-time asset reference and mounts/activates only the selected island on interaction. Full resumability is not required for MVP.
- **Browser:** A more complete Rust/WASM runtime creates and updates DOM nodes directly.
- **Bootstrap:** Use the smallest practical implementation, including plain JS. Making every bootstrap byte Rust/WASM is not itself a goal.
- Generate/hash all code and assets at build/deploy time; never compile new code in the HTTP request path.
- A future **GPU scene renderer** is separate from HTML/DOM; WebGPU is not advertised as a general DOM/CSS accelerator.

## Unresolved design: UI representation

### Option A — Minimal shared `View` tree

Represent a small set of nodes/values to validate both native HTML and browser DOM rendering. This is inexpensive to implement but may introduce allocation and tree-update overhead.

### Option B — Compile-time static templates with dynamic slots

A future `ui!` macro could extract fixed HTML/templates and dynamic slots. Native rendering would serialize these slots; the browser would clone templates and update bound values, avoiding a full VDOM diff. However, macro complexity, event captures, and hydration identity are substantial design risks.

**Decision gate:** Use the smallest working representation in M1/M2. First prove the same component source can render in both targets, then measure whether macro/slot optimization is necessary. Do not claim a performance win without benchmarks.

## Hydration and state

- SSR HTML and browser code require stable, verifiable component/instance identities. Hydration must not take over unrelated DOM.
- Escape serialized state correctly and include schema/asset versions. Incompatible versions must explicitly fail or fall back.
- Browser state must not contain secrets, repository objects, or server handles.
- Avoid placing raw JSON in inline script blocks; tests must cover CSP and XSS boundaries.

## Alternatives considered

- **VDOM everywhere:** Uniform design but can add runtime costs; requires measurement.
- **Server-streamed UI updates:** Conflicts with local-first and optional transport goals.
- **Full resumability from day one:** Would require constrained Rust closures/captures and stable handler IDs; defer as research.
- **Canvas for every HTML element:** Would amount to rebuilding CSS layout, selection, IME, and accessibility infrastructure.

## Verification and current evidence

**M2 — 2026-10-08:** The same shared `View` source renders through native HTML and browser WASM/direct-DOM implementations. A headless Chrome test confirmed byte-identical initial TodoList markup, local state updates, and no WebSocket. Browser mode uses a tiny JS bootstrap and content-addressed assets. DOM updates currently replace one owned subtree rather than performing fine-grained subscriptions. See [M2 verification](../M2-VERIFICATION.md).

**M3 — 2026-10-08:** Native `/search` returns complete SSR HTML. An external loader waits for an input/focus event to load Search WASM. Headless Chrome confirmed initial DOM preservation, local search filtering, and no event circuit. YT-017 added an allowlisted, size-bounded `SearchStateV1` JSON payload and verified fail-closed behavior for incompatible versions. This is **not** generic resumability. See [M3 verification](../M3-VERIFICATION.md).

Remaining measurements include compressed WASM bytes, SSR response bytes, and browser mount time. No unmeasured performance comparisons are implied.
